//! Brick 6b: network pipeline hooks — packet-level interception before Bukkit.
//!
//! Modules hook the kernel's network stack (Netty pipeline): inbound packets
//! can be inspected, mutated, or dropped before they become Bukkit events;
//! outbound packets likewise. This is where packet-level anticheat, protocol
//! emulation (ViaVersion-style), and Geyser-latency fixes live.
//!
//! # Packet flow (1.21.10 kernel)
//!
//! Every packet travels the Netty pipeline through two dedicated codecs:
//!
//! - **inbound**: `net.minecraft.network.PacketDecoder.decode(ChannelHandlerContext,
//!   ByteBuf, List)` — raw frame bytes → `Packet` object; the packet is then
//!   routed by `Connection.channelRead0` to the packet listener of the current
//!   protocol state (`ServerStatusPacketListenerImpl`,
//!   `ServerLoginPacketListenerImpl`, `ServerConfigurationPacketListenerImpl`,
//!   `ServerGamePacketListenerImpl`).
//! - **outbound**: `net.minecraft.network.PacketEncoder.encode(ChannelHandlerContext,
//!   Packet, ByteBuf)` — `Packet` object → raw frame bytes.
//!
//! The hooks in this brick operate on raw bytes, so they are state-agnostic:
//! a packet is identified by `(conn_id, direction, state, payload)` and the
//! platform tracks `state` per connection as it is negotiated. The transform
//! rules installed by [`install_default_rules`] make these two codecs — plus
//! the handshake/protocol-switch points — call the Java adapter at method
//! entry, which forwards every packet to [`run_hooks`] on the Netty
//! event-loop thread.
//!
//! # Java hook contract
//!
//! The adapter is a single class, `dev.crussty.hooks.NetHooks`, compiled into
//! the kernel jar (same mechanism as the crussty bridge classes). Because the
//! transform engine only injects `()V` static calls (see `transform.rs`), the
//! contract is split in two:
//!
//! ## Triggers (injected by the transform rules; all `public static void` `()V`)
//!
//! | Rule target (class, method) | Injected helper |
//! |---|---|
//! | `net/minecraft/network/PacketDecoder.decode` | `NetHooks.onDecode()` |
//! | `net/minecraft/network/PacketEncoder.encode` | `NetHooks.onEncode()` |
//! | `net/minecraft/server/network/ServerHandshakePacketListenerImpl.handleIntention` | `NetHooks.onIntention()` |
//! | `net/minecraft/network/Connection.setupInboundProtocol` | `NetHooks.onProtocolSwap()` |
//! | `net/minecraft/network/Connection.channelInactive` | `NetHooks.onChannelInactive()` |
//!
//! Each trigger runs at method entry on the Netty thread, before any packet
//! work. It extracts the calling frame's values (JVMTI `GetLocalVariable` on
//! the injected frame; the adapter wave claims `can_access_local_variables`)
//! and forwards them to the bridge below.
//!
//! ## Adapter bridge (full-signature contract, `RegisterNatives` into this crate)
//!
//! ```text
//! public static native int  onInbound(byte[] payload, int packetId, long connId);
//! public static native int  onOutbound(byte[] payload, int packetId, long connId);
//! public static native void onHandshake(long connId, int intendedState);
//! public static native void onProtocol(long connId, int newState);
//! public static native void onClose(long connId);
//! ```
//!
//! - `payload` is the raw frame (packet id VarInt included, compression
//!   already undone by the earlier pipeline stage); `packetId` is the decoded
//!   id; `connId` is assigned by the adapter from the `Channel`
//!   (`Connection.channel`, a public field) and must be unique among live
//!   connections.
//! - The native side builds a [`Packet`] (direction from the call site,
//!   state resolved from the connection registry) and runs it through
//!   [`run_hooks`]. The verdict is returned to Java as an `int`:
//!   `0` = pass, `1` = drop, `2` = disconnect.
//! - `onHandshake` is called by the `onIntention` trigger with the intention
//!   state (0 = status, 2 = login; `ClientIntentionPacket.intention()`), and
//!   `onProtocol` by the `onProtocolSwap` trigger for every later swap
//!   (login → configuration → play); both feed the state machine via
//!   [`set_conn_state`]. `onClose` is called by the `onChannelInactive`
//!   trigger and feeds [`detach_conn`].
//!
//! **Sending a kick.** When a hook returns `Verdict::Disconnect`, the reason
//! lives in `packet.disconnect_reason`; the native layer hands it back to the
//! adapter, which kicks on the Netty thread with
//! `connection.disconnect(Component.literal(reason))`
//! (`net.minecraft.network.Connection.disconnect(Component)`, verified in the
//! 1.21.10 jar). If the hook left no reason the adapter falls back to
//! `Component.literal("Disconnected")`. The packet itself never reaches the
//! listener: `run_hooks` returns before the codec proceeds.
//!
//! # Connection state machine
//!
//! Protocol states are tracked per connection with `u8` codes matching the
//! classic `Packet.state` contract: `0` = handshake, `1` = status, `2` =
//! login, `3` = play. The 1.20.5+ configuration phase is folded into `3`
//! (play) by the adapter — the four-state machine keeps the `Packet.state`
//! contract stable and covers every real transition:
//!
//! ```text
//! Handshake(0) ──► Status(1)      (server-list ping; terminal)
//!      │
//!      └────────► Login(2) ──► Play(3)   (incl. configuration, 1.20.5+)
//! ```
//!
//! Legal transitions: `0→{0,1,2}`, `1→{1}`, `2→{2,3}`, `3→{3}` (self
//! transitions are no-ops; anything else is rejected by [`set_conn_state`]
//! and leaves the tracked state unchanged). A tracked connection's state is
//! authoritative: [`run_hooks`] overwrites `packet.state` from it; for
//! untracked connections the adapter-supplied state passes through.
//!
//! # Connection registry
//!
//! [`attach_conn`]/[`detach_conn`] maintain a bounded per-connection table
//! (LRU, [`MAX_CONNS`] = 4096, oldest evicted on overflow) with an optional
//! player UUID (raw 128-bit RFC 4122 value) bound at login. Eviction drops
//! the tracked state; the packet flow continues with adapter-supplied states.
//! [`conn_count`] and [`conns`] feed telemetry and admin modules.
//!
//! # Telemetry counters
//!
//! [`run_hooks`] keeps running totals (`network.packets_in`,
//! `network.packets_out`, `network.dropped`) in lock-free atomics and
//! publishes them into the telemetry snapshot through
//! [`super::telemetry::publish_metric`] at most once per second per counter
//! (coalesced reporting keeps the publish out of the per-packet hot path and
//! the snapshot bounded). [`packet_counters`] exposes the totals directly.
//!
//! # Research notes
//!
//! Decisions grounded in: `javap` inspection of the shipped Purpur 1.21.10
//! kernel jar (Mojang-mapped; verified `PacketDecoder.decode`, the erased
//! `PacketEncoder.encode(ChannelHandlerContext, Packet, ByteBuf)` descriptor,
//! `Connection.setupInboundProtocol(ProtocolInfo, PacketListener)`,
//! `ServerHandshakePacketListenerImpl.handleIntention(ClientIntentionPacket)`,
//! `Connection.channelInactive(ChannelHandlerContext)`,
//! `Connection.disconnect(Component)`); mappings.dev (1.21.x codec shapes,
//! `ServerGamePacketListenerImpl.shouldHandleMessage`); the
//! minecraft-how-it-works book (1.21.x path: `handleIntention` picks
//! STATUS/LOGIN and swaps the protocol tables, login → configuration →
//! play handoff via `setupInboundProtocol`/`setupOutboundProtocol`);
//! minecraft.wiki protocol states (handshaking is the initial state, switched
//! by the intention packet / login success; configuration since 1.20.5);
//! netty.io `ByteToMessageDecoder`/`MessageToByteEncoder` semantics; and the
//! ProtocolLib/PacketEvents interception pattern (hook the codecs, not the
//! listeners). The bounded LRU registry is a stamp-based design: a global
//! monotonic clock stamps every touch (O(1)), eviction scans for the minimum
//! stamp only on overflow, replacing the classic `LinkedHashMap(accessOrder)`
//! idiom whose deque bookkeeping cost O(conns) per touch.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock, RwLock};
use std::time::Instant;

use super::publish_metric;
use super::transform::{global_engine, Injection, Rule};

/// Opaque packet handle — the concrete Netty ByteBuf is not exposed across
/// the ABI; modules work on raw bytes and metadata.
pub struct Packet {
    /// Direction of travel.
    pub direction: Direction,
    /// Protocol state (0 = handshake, 1 = status, 2 = login, 3 = play).
    /// Overwritten by the connection registry when the conn is tracked.
    pub state: u8,
    /// Raw payload bytes (packet id included).
    pub payload: Vec<u8>,
    /// Connection id assigned by the adapter.
    pub conn_id: u64,
    /// Kick reason produced by a hook returning [`Verdict::Disconnect`];
    /// the Java adapter sends it via `Connection.disconnect(Component)`.
    pub disconnect_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Inbound,
    Outbound,
}

/// What the platform should do with a packet after hooks run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Pass the (possibly modified) packet through.
    Pass,
    /// Drop it silently.
    Drop,
    /// Close the connection (the reason goes in `Packet.disconnect_reason`).
    Disconnect,
}

pub type PacketHookFn = Arc<dyn Fn(&mut Packet) -> Verdict + Send + Sync>;

/// Hook registry: an RCU-published immutable snapshot (TASK-164) — `add_hook`
/// rebuilds and republishes the slice (rare, init-time) through an
/// `rcu::ArcCell`; the per-packet path reads a generation-checked per-thread
/// memo (one acquire load + one Arc clone off TLS), no lock at all.
static HOOKS: LazyLock<super::rcu::ArcCell<[PacketHookFn]>> =
    LazyLock::new(super::rcu::ArcCell::new);

thread_local! {
    static HOOKS_MEMO: RefCell<Option<(u64, Arc<[PacketHookFn]>)>> = const { RefCell::new(None) };
}

/// Generation-checked per-thread hook snapshot (TASK-164). Hooks may freely
/// call `add_hook` — the memo guard is never held across hook invocation.
fn hooks_snapshot() -> Arc<[PacketHookFn]> {
    let cell = &*HOOKS;
    let gen = cell.gen();
    if gen == 0 {
        return Vec::new().into();
    }
    HOOKS_MEMO.with(|m| {
        if let Ok(borrowed) = m.try_borrow() {
            if let Some((g, a)) = borrowed.as_ref() {
                if *g == gen {
                    return Arc::clone(a);
                }
            }
        }
        let fresh = cell.load_arc().unwrap_or_else(|| Vec::new().into());
        if let Ok(mut slot) = m.try_borrow_mut() {
            *slot = Some((gen, Arc::clone(&fresh)));
        }
        fresh
    })
}

/// Live packet-hook count (TASK-163): the per-packet fast gate — zero (the
/// default until a module registers) skips the snapshot read entirely.
static PACKET_HOOKS_LIVE: AtomicUsize = AtomicUsize::new(0);

/// Modules register packet hooks at init (order = registration order).
pub fn add_hook(f: PacketHookFn) {
    let cell = &*HOOKS;
    let cur = cell.load_arc().unwrap_or_else(|| Vec::new().into());
    let mut v: Vec<PacketHookFn> = Vec::with_capacity(cur.len() + 1);
    v.extend(cur.iter().cloned());
    v.push(f);
    let live = v.len();
    cell.store(v.into());
    PACKET_HOOKS_LIVE.store(live, Ordering::Release);
}

// ---------------------------------------------------------------------------
// Connection state machine
// ---------------------------------------------------------------------------

/// Protocol states in the u8 `Packet.state` contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    /// 0 — `HANDSHAKING`: the intention packet chooses status or login.
    Handshake = 0,
    /// 1 — `STATUS`: server-list ping; terminal.
    Status = 1,
    /// 2 — `LOGIN`: authentication, encryption, compression.
    Login = 2,
    /// 3 — `PLAY` (and the 1.20.5+ configuration phase, folded in).
    Play = 3,
}

impl ProtocolState {
    /// The u8 code as carried in [`Packet::state`].
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Decode a `u8` protocol-state code.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Handshake),
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            3 => Some(Self::Play),
            _ => None,
        }
    }
}

/// Legal state machine transitions (self-transitions are no-ops).
fn transition_legal(from: ProtocolState, to: ProtocolState) -> bool {
    use ProtocolState::*;
    matches!(
        (from, to),
        (Handshake, Handshake | Status | Login)
            | (Status, Status)
            | (Login, Login | Play)
            | (Play, Play)
    )
}

// ---------------------------------------------------------------------------
// Per-connection registry (stamp-based bounded LRU)
// ---------------------------------------------------------------------------

/// Hard cap on concurrently tracked connections; the oldest LRU entry is
/// evicted when a new connection overflows the table.
pub const MAX_CONNS: usize = 4096;

/// One tracked connection.
#[derive(Debug, Clone)]
pub struct ConnInfo {
    /// Raw 128-bit player UUID (RFC 4122) bound at login, if known yet.
    pub player_uuid: Option<u128>,
    /// Current protocol state code (see [`ProtocolState`]).
    pub state: u8,
}

struct ConnEntry {
    player_uuid: Option<u128>,
    state: ProtocolState,
    /// LRU stamp: a value from the process-global monotonic clock, taken at
    /// the last touch. Larger = more recently used. An O(1) replacement for
    /// the old VecDeque order (whose `position()` scan made every touch
    /// O(conns) and every detach O(conns)).
    stamp: u64,
}

#[derive(Default)]
struct ConnRegistry {
    map: HashMap<u64, ConnEntry>,
}

/// Process-global LRU clock (one tick per touch; wraps after ~584M years of
/// per-packet touches — not a concern).
static LRU_CLOCK: AtomicU64 = AtomicU64::new(0);

static CONNS: LazyLock<RwLock<ConnRegistry>> =
    LazyLock::new(|| RwLock::new(ConnRegistry::default()));

fn lru_touch(entry: &mut ConnEntry) {
    entry.stamp = LRU_CLOCK.fetch_add(1, Ordering::Relaxed);
}

/// Register a connection. A re-attach of a live conn only refreshes the
/// player UUID and reports `false` (not newly created). Evicts the least
/// recently used conn when the table is at [`MAX_CONNS`] (an O(n) scan, but
/// only paid on overflow — never on the per-packet path).
pub fn attach_conn(conn_id: u64, player_uuid: Option<u128>) -> bool {
    let mut reg = CONNS.write().unwrap_or_else(|p| p.into_inner());
    if let Some(entry) = reg.map.get_mut(&conn_id) {
        entry.player_uuid = player_uuid;
        lru_touch(entry);
        return false;
    }
    if reg.map.len() >= MAX_CONNS {
        if let Some((oldest, _)) = reg
            .map
            .iter()
            .min_by_key(|(_, e)| e.stamp)
            .map(|(k, e)| (*k, e.stamp))
        {
            reg.map.remove(&oldest);
            // TASK-164 mirror: the evicted conn must not answer probes.
            conn_mirror_del(oldest);
        }
    }
    let stamp = LRU_CLOCK.fetch_add(1, Ordering::Relaxed);
    reg.map.insert(
        conn_id,
        ConnEntry {
            player_uuid,
            state: ProtocolState::Handshake,
            stamp,
        },
    );
    // TASK-164 mirror: the per-packet state_of probe reads this table.
    conn_mirror_put(conn_id, ProtocolState::Handshake.code());
    // TASK-163 gate mirror: keep the live-conn count honest (the per-packet
    // state_of gate trusts 0 = empty table). TASK-182: the store moved under
    // the write lock — the count is serialized with the map mutation, so
    // lock-free `conn_count` readers can only lag an ACTIVE writer.
    CONNS_LIVE.store(reg.map.len(), Ordering::Release);
    drop(reg);
    true
}

/// Forget a connection (called by the `onClose` hook on channel inactive).
/// Returns `true` if the conn was tracked. O(1) (was O(n) `retain`).
pub fn detach_conn(conn_id: u64) -> bool {
    // TASK-182: one write round-trip (was write + read); the count store
    // happens under the same lock that removed the row.
    let mut reg = CONNS.write().unwrap_or_else(|p| p.into_inner());
    let removed = reg.map.remove(&conn_id).is_some();
    if removed {
        conn_mirror_del(conn_id);
        CONNS_LIVE.store(reg.map.len(), Ordering::Release);
    }
    removed
}

/// Live tracked-connection count (TASK-163): the per-packet `state_of` gate.
static CONNS_LIVE: AtomicUsize = AtomicUsize::new(0);

/// Advance the connection's protocol state; used by the handshake and
/// protocol-swap hooks. Returns `false` (state unchanged) for unknown conns,
/// unknown state codes, or transitions outside the machine above.
pub fn set_conn_state(conn_id: u64, state_code: u8) -> bool {
    let Some(to) = ProtocolState::from_code(state_code) else {
        return false;
    };
    let mut reg = CONNS.write().unwrap_or_else(|p| p.into_inner());
    let Some(entry) = reg.map.get_mut(&conn_id) else {
        return false;
    };
    if !transition_legal(entry.state, to) {
        return false;
    }
    entry.state = to;
    lru_touch(entry);
    conn_mirror_put(conn_id, to.code());
    true
}

/// Tracked protocol-state code for a connection, if any. Lock-free on the
/// per-packet path (TASK-164): an open-addressed atomic mirror table
/// (linear probing, tombstone on remove) fed by the registry writers — the
/// RwLock map below stays the source of truth for everything cold.
pub fn state_of(conn_id: u64) -> Option<u8> {
    // TASK-163 fast gate: zero tracked conns (before the first player) is
    // one acquire load — no probe, no lock round-trip.
    if CONNS_LIVE.load(Ordering::Acquire) == 0 {
        return None;
    }
    let mut i = conn_slot(conn_id);
    let key = conn_key(conn_id);
    loop {
        let k = CONN_KEY[i].load(Ordering::Acquire);
        if k == key {
            return Some(CONN_STATE_MIRROR[i].load(Ordering::Relaxed) as u8);
        }
        if k == 0 {
            // Clean empty slot: linear probing never places a key past an
            // empty slot, so the key is definitively absent.
            return None;
        }
        i = (i + 1) & (CONN_TAB - 1); // tombstone (u64::MAX) or foreign key
    }
}

/// Atomic conn mirror table size: power of two, ~4x the conn cap so probes
/// stay at one slot on average.
const CONN_TAB: usize = 16384;
const CONN_TOMBSTONE: u64 = u64::MAX;
static CONN_KEY: [AtomicU64; CONN_TAB] = [const { AtomicU64::new(0) }; CONN_TAB];
static CONN_STATE_MIRROR: [AtomicU64; CONN_TAB] = [const { AtomicU64::new(0) }; CONN_TAB];

#[inline]
fn conn_slot(conn_id: u64) -> usize {
    (conn_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 20) as usize & (CONN_TAB - 1)
}

/// Mirror keys are stored XOR-flipped so a real conn_id can never equal the
/// `0` empty sentinel or the `u64::MAX` tombstone (conn_id 0 is a legal id —
/// without the flip `state_of(0)` would match clean-empty slots).
#[inline]
fn conn_key(conn_id: u64) -> u64 {
    conn_id ^ 0x8000_0000_0000_0000
}

/// Mirror write: insert or update `conn_id -> state` (callers hold the
/// registry write lock, so mirror writes are serialized).
fn conn_mirror_put(conn_id: u64, state: u8) {
    let key = conn_key(conn_id);
    let mut i = conn_slot(conn_id);
    loop {
        let k = CONN_KEY[i].load(Ordering::Relaxed);
        if k == key {
            CONN_STATE_MIRROR[i].store(u64::from(state), Ordering::Release);
            return;
        }
        if k == 0 || k == CONN_TOMBSTONE {
            if CONN_KEY[i]
                .compare_exchange(k, key, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                CONN_STATE_MIRROR[i].store(u64::from(state), Ordering::Release);
                return;
            }
        }
        i = (i + 1) & (CONN_TAB - 1);
    }
}

/// Mirror write: remove `conn_id` (tombstone).
fn conn_mirror_del(conn_id: u64) {
    let key = conn_key(conn_id);
    let mut i = conn_slot(conn_id);
    loop {
        let k = CONN_KEY[i].load(Ordering::Relaxed);
        if k == key {
            CONN_KEY[i].store(CONN_TOMBSTONE, Ordering::Release);
            CONN_STATE_MIRROR[i].store(0, Ordering::Release);
            return;
        }
        if k == 0 {
            return; // not present
        }
        i = (i + 1) & (CONN_TAB - 1);
    }
}

/// Tracked details (uuid + state) for a connection, if any.
pub fn conn_info(conn_id: u64) -> Option<ConnInfo> {
    CONNS
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .map
        .get(&conn_id)
        .map(|e| ConnInfo {
            player_uuid: e.player_uuid,
            state: e.state.code(),
        })
}

/// TASK-182 A/B toggle: when true, `conn_count` is served lock-free from the
/// TASK-163 atomic mirror; when false, the legacy RwLock shape below runs
/// verbatim (the OFF arm exists only as the A/B bench reference).
const CONN_COUNT_ATOMIC: bool = true;

/// Number of currently tracked connections. TASK-182: served lock-free from
/// the TASK-163 atomic mirror (`CONNS_LIVE`) instead of a RwLock read
/// round-trip. The registry writers publish the count while still holding
/// the write lock, so the atomic can only lag an ACTIVE writer — a finished
/// attach/detach is always visible. The legacy shape is kept verbatim behind
/// [`CONN_COUNT_ATOMIC`] for the A/B bench.
pub fn conn_count() -> usize {
    if CONN_COUNT_ATOMIC {
        CONNS_LIVE.load(Ordering::Acquire)
    } else {
        CONNS
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .map
            .len()
    }
}

/// Tracked connection ids in LRU order (least recently used first).
pub fn conns() -> Vec<u64> {
    let reg = CONNS.read().unwrap_or_else(|p| p.into_inner());
    let mut rows: Vec<(u64, u64)> =
        reg.map.iter().map(|(k, e)| (*k, e.stamp)).collect();
    rows.sort_by_key(|(_, stamp)| *stamp);
    rows.into_iter().map(|(id, _)| id).collect()
}

// ---------------------------------------------------------------------------
// Telemetry counters
// ---------------------------------------------------------------------------

static PACKETS_IN: AtomicU64 = AtomicU64::new(0);
static PACKETS_OUT: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);

/// Last publish time (ms since process start) per counter. The per-packet
/// path takes NO lock: a `compare_exchange` claims the once-per-second
/// publish slot, so concurrent threads coalesce without a mutex.
static LAST_PUB_IN: AtomicU64 = AtomicU64::new(0);
static LAST_PUB_OUT: AtomicU64 = AtomicU64::new(0);
static LAST_PUB_DROPPED: AtomicU64 = AtomicU64::new(0);

static START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn now_ms() -> u64 {
    START.elapsed().as_millis() as u64
}

enum CounterKind {
    In,
    Out,
    Dropped,
}

/// Publish the running total, coalesced to one push per second per counter.
fn publish_counter(kind: CounterKind, total: u64) {
    let (cell, name) = match kind {
        CounterKind::In => (&LAST_PUB_IN, "network.packets_in"),
        CounterKind::Out => (&LAST_PUB_OUT, "network.packets_out"),
        CounterKind::Dropped => (&LAST_PUB_DROPPED, "network.dropped"),
    };
    // TASK-163: the In/Out throttle is COUNT-based (sample every 4096th
    // packet) instead of wall-clock: the sampled value is the monotonic
    // running total either way, and the per-packet path loses its
    // SystemTime::now() vDSO call entirely. The Dropped counter stays
    // wall-clock throttled — drops are rare, the clock is free there.
    match kind {
        CounterKind::In | CounterKind::Out => {
            if total & 4095 == 0 {
                publish_metric(name, total as f64, Some("packets"), None);
            }
        }
        CounterKind::Dropped => {
            let now = now_ms();
            let last = cell.load(Ordering::Relaxed);
            if now.saturating_sub(last) >= 1000
                && cell
                    .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                publish_metric(name, total as f64, Some("packets"), None);
            }
        }
    }
}

/// Running packet totals `(packets_in, packets_out, packets_dropped)`.
pub fn packet_counters() -> (u64, u64, u64) {
    (
        PACKETS_IN.load(Ordering::Relaxed),
        PACKETS_OUT.load(Ordering::Relaxed),
        DROPPED.load(Ordering::Relaxed),
    )
}

// ---------------------------------------------------------------------------
// Hook pipeline
// ---------------------------------------------------------------------------

/// Called by the network adapter on the Netty thread for every packet.
///
/// For tracked connections `packet.state` is overwritten from the connection
/// registry (the state machine is authoritative). Counters are bumped for
/// every packet; a hook returning [`Verdict::Drop`] bumps and publishes
/// `network.dropped`; [`Verdict::Disconnect`] stops the chain and the adapter
/// kicks the conn with `packet.disconnect_reason`.
///
/// Hot-path discipline: no allocation, no Mutex — the hooks snapshot is one
/// `Arc` clone off an immutable slice, the registry read is a `RwLock` read,
/// and counter publishes are atomic CAS claims.
pub fn run_hooks(mut packet: Packet) -> Verdict {
    if let Some(state) = state_of(packet.conn_id) {
        packet.state = state;
    }
    let dir_kind = match packet.direction {
        Direction::Inbound => (&PACKETS_IN, CounterKind::In),
        Direction::Outbound => (&PACKETS_OUT, CounterKind::Out),
    };
    let total = dir_kind.0.fetch_add(1, Ordering::Relaxed) + 1;
    publish_counter(dir_kind.1, total);

    // TASK-163 fast gate: with zero registered hooks — the default until a
    // module adds one — skip the registry read entirely.
    if PACKET_HOOKS_LIVE.load(Ordering::Acquire) > 0 {
        let hooks = hooks_snapshot();
        for h in hooks.iter() {
            match h(&mut packet) {
                Verdict::Pass => continue,
                Verdict::Drop => {
                    let dropped = DROPPED.fetch_add(1, Ordering::Relaxed) + 1;
                    publish_counter(CounterKind::Dropped, dropped);
                    return Verdict::Drop;
                }
                Verdict::Disconnect => return Verdict::Disconnect,
            }
        }
    }
    Verdict::Pass
}

// ---------------------------------------------------------------------------
// Default transform rules
// ---------------------------------------------------------------------------

const HOOK_CLASS: &str = "dev.crussty.hooks.NetHooks";

/// JVM descriptors of the hooked kernel methods (verified via `javap` on the
/// shipped Purpur 1.21.10 jar).
const DESCR_DECODE: &str = "(Lio/netty/channel/ChannelHandlerContext;Lio/netty/buffer/ByteBuf;Ljava/util/List;)V";
const DESCR_ENCODE: &str = "(Lio/netty/channel/ChannelHandlerContext;Lnet/minecraft/network/protocol/Packet;Lio/netty/buffer/ByteBuf;)V";
const DESCR_INTENTION: &str = "(Lnet/minecraft/network/protocol/handshake/ClientIntentionPacket;)V";
const DESCR_PROTOCOL: &str = "(Lnet/minecraft/network/ProtocolInfo;Lnet/minecraft/network/PacketListener;)V";
const DESCR_CHANNEL_CTX: &str = "(Lio/netty/channel/ChannelHandlerContext;)V";

static RULES_INSTALLED: OnceLock<()> = OnceLock::new();

/// Register the default transform rules that route kernel packet handling
/// through [`run_hooks`]. Idempotent; call before kernel classes load (the
/// runtime claims class hooks before the kernel starts, so rules fire at
/// class load). Requires the `NetHooks` adapter class in the kernel jar —
/// until then the injected calls resolve lazily and never execute.
pub fn install_default_rules() {
    RULES_INSTALLED.get_or_init(|| {
        let engine = global_engine();
        // Inbound codec: raw frame bytes arrive here first.
        engine.register(Rule::new(
            "net/minecraft/network/PacketDecoder",
            "decode",
            DESCR_DECODE,
            Injection::MethodEntry,
            format!("{HOOK_CLASS}.onDecode"),
        ));
        // Outbound codec: raw frame bytes leave here.
        engine.register(Rule::new(
            "net/minecraft/network/PacketEncoder",
            "encode",
            DESCR_ENCODE,
            Injection::MethodEntry,
            format!("{HOOK_CLASS}.onEncode"),
        ));
        // Handshake handler: intention packet picks status/login.
        engine.register(Rule::new(
            "net/minecraft/server/network/ServerHandshakePacketListenerImpl",
            "handleIntention",
            DESCR_INTENTION,
            Injection::MethodEntry,
            format!("{HOOK_CLASS}.onIntention"),
        ));
        // Every later state swap (login -> configuration -> play).
        engine.register(Rule::new(
            "net/minecraft/network/Connection",
            "setupInboundProtocol",
            DESCR_PROTOCOL,
            Injection::MethodEntry,
            format!("{HOOK_CLASS}.onProtocolSwap"),
        ));
        // Conn teardown: forget the registry entry.
        engine.register(Rule::new(
            "net/minecraft/network/Connection",
            "channelInactive",
            DESCR_CHANNEL_CTX,
            Injection::MethodEntry,
            format!("{HOOK_CLASS}.onChannelInactive"),
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serializes tests that touch global state (hooks, registry, counters,
    /// engine rules) which live for the whole test process.
    pub(super) static TEST_LOCK: Mutex<()> = Mutex::new(());

    pub(super) fn packet(direction: Direction, conn_id: u64, state: u8, payload: &[u8]) -> Packet {
        Packet {
            direction,
            state,
            payload: payload.to_vec(),
            conn_id,
            disconnect_reason: None,
        }
    }

    #[test]
    fn hooks_drop_and_pass() {
        let _guard = TEST_LOCK.lock().unwrap();
        let h = Arc::new(|p: &mut Packet| {
            if p.state == 9 {
                Verdict::Drop
            } else {
                Verdict::Pass
            }
        });
        add_hook(h);
        let keep = packet(Direction::Inbound, 0, 3, &[]);
        assert_eq!(run_hooks(keep), Verdict::Pass);
        let drop = packet(Direction::Inbound, 0, 9, &[]);
        assert_eq!(run_hooks(drop), Verdict::Drop);
    }

    #[test]
    fn conn_registry_lru_eviction() {
        let _guard = TEST_LOCK.lock().unwrap();
        for i in 0..(MAX_CONNS + 10) as u64 {
            assert!(attach_conn(i, None));
        }
        assert_eq!(conn_count(), MAX_CONNS);
        let all = conns();
        assert_eq!(all.len(), MAX_CONNS);
        // the ten oldest were evicted; the newest are present
        assert!(!all.contains(&0));
        assert!(!all.contains(&9));
        assert!(all.contains(&((MAX_CONNS + 9) as u64)));
        // evicted conns are no longer tracked
        assert_eq!(state_of(0), None);
    }

    #[test]
    fn conn_registry_attach_detach_and_uuid() {
        let _guard = TEST_LOCK.lock().unwrap();
        assert!(attach_conn(7, Some(0x1234)));
        assert_eq!(state_of(7), Some(ProtocolState::Handshake.code()));
        // re-attach refreshes the uuid but is not a new entry
        assert!(!attach_conn(7, Some(0x5678)));
        assert_eq!(conn_info(7).map(|c| c.player_uuid), Some(Some(0x5678)));
        assert_eq!(conn_count(), 1);
        assert!(detach_conn(7));
        assert!(!detach_conn(7));
        assert_eq!(conn_count(), 0);
    }

    #[test]
    fn state_tracking_transitions() {
        let _guard = TEST_LOCK.lock().unwrap();
        attach_conn(1, None);
        // handshake -> login
        assert!(set_conn_state(1, 2));
        assert_eq!(state_of(1), Some(2));
        // login -> status is illegal
        assert!(!set_conn_state(1, 1));
        assert_eq!(state_of(1), Some(2));
        // login -> play
        assert!(set_conn_state(1, 3));
        assert!(!set_conn_state(1, 9)); // unknown code
        assert_eq!(state_of(1), Some(3));
        // unknown conn: no-op
        assert!(!set_conn_state(999, 3));
        // status is terminal
        attach_conn(2, None);
        assert!(set_conn_state(2, 1));
        assert!(!set_conn_state(2, 3));
        assert!(!set_conn_state(2, 2));
        // detach + re-attach resets to handshake
        detach_conn(2);
        attach_conn(2, None);
        assert_eq!(state_of(2), Some(0));
    }

    #[test]
    fn run_hooks_state_comes_from_registry() {
        let _guard = TEST_LOCK.lock().unwrap();
        let seen = Arc::new(Mutex::new(0u8));
        let hook_seen = seen.clone();
        add_hook(Arc::new(move |p: &mut Packet| {
            *hook_seen.lock().unwrap() = p.state;
            Verdict::Pass
        }));
        attach_conn(42, None);
        assert!(set_conn_state(42, ProtocolState::Login.code()));
        assert!(set_conn_state(42, ProtocolState::Play.code()));
        // adapter says handshake (0); the registry overrides to play (3)
        assert_eq!(run_hooks(packet(Direction::Inbound, 42, 0, &[])), Verdict::Pass);
        assert_eq!(*seen.lock().unwrap(), 3);
    }

    #[test]
    fn counters_increment_per_direction() {
        let _guard = TEST_LOCK.lock().unwrap();
        let (in0, out0, drop0) = packet_counters();
        for _ in 0..5 {
            run_hooks(packet(Direction::Inbound, 0, 3, &[]));
        }
        for _ in 0..3 {
            run_hooks(packet(Direction::Outbound, 0, 3, &[]));
        }
        // payload-gated drop hook: benign for every other test's packets
        let h = Arc::new(|p: &mut Packet| {
            if p.payload == b"kill" {
                Verdict::Drop
            } else {
                Verdict::Pass
            }
        });
        add_hook(h);
        run_hooks(packet(Direction::Inbound, 0, 3, b"kill"));
        run_hooks(packet(Direction::Inbound, 0, 3, b"kill"));
        let (i, o, d) = packet_counters();
        // dropped packets still counted as inbound
        assert_eq!(i - in0, 7);
        assert_eq!(o - out0, 3);
        assert_eq!(d - drop0, 2);
    }

    #[test]
    fn disconnect_carries_reason() {
        let _guard = TEST_LOCK.lock().unwrap();
        let seen = Arc::new(Mutex::new(None::<String>));
        let hook_seen = seen.clone();
        // payload-gated so later tests are unaffected
        let h = Arc::new(move |p: &mut Packet| {
            if p.payload == b"kick" {
                p.disconnect_reason = Some("speedhack".to_string());
                *hook_seen.lock().unwrap() = p.disconnect_reason.clone();
                Verdict::Disconnect
            } else {
                Verdict::Pass
            }
        });
        add_hook(h);
        assert_eq!(
            run_hooks(packet(Direction::Outbound, 0, 3, b"kick")),
            Verdict::Disconnect
        );
        assert_eq!(seen.lock().unwrap().as_deref(), Some("speedhack"));
        // a Disconnect without a reason leaves the field None for the
        // adapter to fall back on
        assert_eq!(
            run_hooks(packet(Direction::Outbound, 0, 3, b"kick")),
            Verdict::Disconnect
        );
    }

    #[test]
    fn install_default_rules_is_idempotent() {
        let _guard = TEST_LOCK.lock().unwrap();
        // Other bricks register into the same global engine, so only count
        // the NetHooks rules this brick owns; the OnceLock guarantees exactly
        // five are ever installed, even with repeat calls.
        let nethooks = || {
            global_engine()
                .rules()
                .iter()
                .filter(|r| r.helper.contains(&format!("{HOOK_CLASS}.")))
                .count()
        };
        install_default_rules();
        install_default_rules();
        assert_eq!(nethooks(), 5);
        let expected = [
            "onDecode",
            "onEncode",
            "onIntention",
            "onProtocolSwap",
            "onChannelInactive",
        ];
        let helpers: Vec<String> = global_engine()
            .rules()
            .iter()
            .map(|r| r.helper.clone())
            .collect();
        for name in expected {
            assert!(helpers.iter().any(|h| h.ends_with(name)), "missing {name} rule");
        }
    }

    // ------------------------------------------------------- hotpath benches
    //
    // Release-only A/B benches: `cargo test --release -- --ignored --nocapture
    // bench_`. Same-process, min-of-rounds ns/op. Each bench cleans up its
    // registry rows so ordinary tests stay independent of run order.

    /// Attach every id in `0..range` (ignoring pre-existing rows), used to
    /// force a full table. Drives each conn to Play through the legal
    /// Handshake -> Login -> Play path so later `set_conn_state` touches are
    /// legal self-transitions (the touch code must actually run).
    pub(super) fn bench_fill(range: u64) {
        for i in 0..range {
            attach_conn(i, None);
            set_conn_state(i, ProtocolState::Login.code());
            set_conn_state(i, ProtocolState::Play.code());
        }
    }

    /// Detach every id in `0..range` so later tests see an empty registry.
    pub(super) fn bench_drain(range: u64) {
        for i in 0..range {
            detach_conn(i);
        }
    }

    #[test]
    #[ignore]
    fn bench_run_hooks_packet_path() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 200_000u32;
        let rounds = 5;
        // one registered no-op hook (payload-gated pass so it never drops)
        add_hook(Arc::new(|_p: &mut Packet| Verdict::Pass));
        bench_fill(64);
        // warmup
        for i in 0..10_000u64 {
            let _ = run_hooks(packet(Direction::Inbound, i % 64, 0, &[1, 2, 3]));
        }
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                let _ = run_hooks(packet(Direction::Inbound, i % 64, 0, &[1, 2, 3]));
            }
            best = best.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        bench_drain(64);
        println!(
            "BENCH run_hooks: {:.0} ns/op (1 hook, 64 conns, min of {rounds}x{iters})",
            best * 1e9
        );
    }

    #[test]
    #[ignore]
    fn bench_registry_full_table() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 200_000u32;
        let rounds = 5;
        let table = MAX_CONNS as u64;
        // start clean: earlier tests (e.g. the LRU eviction census) may have
        // left rows in Handshake state that would make the touch loop hit its
        // illegal-transition early return instead of the real touch path.
        bench_drain(table + 32);
        bench_fill(table);
        // warmup
        for i in 0..10_000u64 {
            let _ = state_of(i % table);
        }
        let mut best_state = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                let _ = state_of(i % table);
            }
            best_state = best_state.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        // touch path: legal self-transition Play -> Play on the full table
        let mut best_touch = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                let _ = set_conn_state(i % table, ProtocolState::Play.code());
            }
            best_touch = best_touch.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        let mut best_detach = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                detach_conn(i % table);
                attach_conn(i % table, None);
            }
            best_detach = best_detach.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        // random-access touch: xorshift ids. Sequential ids hide the scan cost
        // (each sequential touch finds its row at the deque front); a random
        // pattern is the realistic churn shape (many conns, arbitrary order).
        // Re-drive rows to Play first: the detach bench above re-created every
        // row as Handshake, which would early-return on the transition check.
        for i in 0..table {
            set_conn_state(i, ProtocolState::Login.code());
            set_conn_state(i, ProtocolState::Play.code());
        }
        let mut xs = 0x9E37_79B9_7F4A_7C15u64;
        let mut best_rand_touch = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters as u64 {
                xs ^= xs << 13;
                xs ^= xs >> 7;
                xs ^= xs << 17;
                let _ = set_conn_state(xs % table, ProtocolState::Play.code());
            }
            best_rand_touch = best_rand_touch.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        let mut best_rand_detach = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters as u64 {
                xs ^= xs << 13;
                xs ^= xs >> 7;
                xs ^= xs << 17;
                let id = xs % table;
                detach_conn(id);
                attach_conn(id, None);
            }
            best_rand_detach = best_rand_detach.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        bench_drain(table);
        println!(
            "BENCH registry: state_of {:.0} ns/op, touch seq {:.0} ns/op, touch rand {:.0} ns/op, detach+reattach seq {:.0} ns/op, detach+reattach rand {:.0} ns/op (full {MAX_CONNS}-conn table, min of {rounds}x{iters})",
            best_state * 1e9,
            best_touch * 1e9,
            best_rand_touch * 1e9,
            best_detach * 1e9,
            best_rand_detach * 1e9
        );
    }

    #[test]
    #[ignore]
    fn bench_conn_count() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 200_000u32;
        let rounds = 5;
        // empty-table line: the steady state before the first player joins
        bench_drain(MAX_CONNS as u64 + 16);
        let _ = std::hint::black_box(conn_count());
        let mut best_empty = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters {
                let _ = std::hint::black_box(conn_count());
            }
            best_empty = best_empty.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        // full-table line: the periodic-publisher steady state
        let table = MAX_CONNS as u64;
        bench_fill(table);
        let _ = std::hint::black_box(conn_count());
        let mut best_full = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters {
                let _ = std::hint::black_box(conn_count());
            }
            best_full = best_full.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        bench_drain(table);
        assert_eq!(conn_count(), 0);
        println!(
            "BENCH conn_count: empty {:.1} ns/op, full({MAX_CONNS}) {:.1} ns/op (min of {rounds}x{iters})",
            best_empty * 1e9,
            best_full * 1e9
        );
    }

    #[test]
    #[ignore]
    fn bench_run_hooks_concurrent_4t() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 50_000u32;
        let threads = 4;
        let rounds = 5;
        add_hook(Arc::new(|_p: &mut Packet| Verdict::Pass));
        bench_fill(64);
        // warmup
        let _ = run_hooks(packet(Direction::Inbound, 0, 0, &[1, 2, 3]));
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            let handles: Vec<_> = (0..threads)
                .map(|_| {
                    std::thread::spawn(move || {
                        for i in 0..iters as u64 {
                            let _ = run_hooks(packet(Direction::Inbound, i % 64, 0, &[1, 2, 3]));
                        }
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
            best = best.min(t.elapsed().as_secs_f64() / f64::from(iters * threads));
        }
        bench_drain(64);
        println!(
            "BENCH run_hooks concurrent x{threads}: {:.0} ns/op (per-op wall, 1 hook, 64 conns, min of {rounds}x{iters}/t)",
            best * 1e9
        );
    }
}

#[cfg(test)]
mod bench_default_shape {
    //! Release-only A/B bench (TASK-163): `cargo test --release -- --ignored --nocapture bench_packet_default`.
    use super::*;
    use super::tests::{bench_drain, bench_fill, packet, TEST_LOCK};
    use std::time::Instant;

    /// Per-packet cost on the production-default shape: zero plugin hooks,
    /// zero connections (before the first player joins).
    #[test]
    #[ignore]
    fn bench_run_hooks_default_shape() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 200_000u32;
        let rounds = 5;
        // warmup
        for i in 0..10_000u64 {
            let _ = run_hooks(packet(Direction::Inbound, i, 0, &[1, 2, 3]));
        }
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                let _ = run_hooks(packet(Direction::Inbound, i, 0, &[1, 2, 3]));
            }
            best = best.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH run_hooks(default 0 hooks 0 conns): {:.0} ns/op (min of {rounds}x{iters})",
            best * 1e9
        );
    }

    /// FRAME cost: the run_hooks pipeline alone with a zero-length payload
    /// (no heap alloc/free for the packet bytes — the framework share of the
    /// per-packet cost; the payload-carrying variants above include one
    /// Vec malloc+free that the packet bytes require).
    #[test]
    #[ignore]
    fn bench_run_hooks_frame() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 200_000u32;
        let rounds = 5;
        for i in 0..10_000u64 {
            let _ = run_hooks(packet(Direction::Inbound, i, 0, &[]));
        }
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                let _ = run_hooks(packet(Direction::Inbound, i, 0, &[]));
            }
            best = best.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH run_hooks(frame, empty payload, 0 hooks 0 conns): {:.0} ns/op (min of {rounds}x{iters})",
            best * 1e9
        );
    }

    /// Per-packet cost with live connections but still zero plugin hooks —
    /// the steady state of a serving server without packet plugins.
    #[test]
    #[ignore]
    fn bench_run_hooks_64conns_nohooks() {
        let _guard = TEST_LOCK.lock().unwrap();
        let iters = 200_000u32;
        let rounds = 5;
        bench_fill(64);
        for i in 0..10_000u64 {
            let _ = run_hooks(packet(Direction::Inbound, i % 64, 0, &[1, 2, 3]));
        }
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for i in 0..iters as u64 {
                let _ = run_hooks(packet(Direction::Inbound, i % 64, 0, &[1, 2, 3]));
            }
            best = best.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        bench_drain(64);
        println!(
            "BENCH run_hooks(0 hooks, 64 conns): {:.0} ns/op (min of {rounds}x{iters})",
            best * 1e9
        );
    }
}
