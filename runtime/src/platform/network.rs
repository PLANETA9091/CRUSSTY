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
//! listeners). The bounded LRU registry follows the standard
//! `LinkedHashMap(accessOrder)` idiom, expressed with `VecDeque` + `HashMap`.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
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

static HOOKS: OnceLock<Mutex<Vec<PacketHookFn>>> = OnceLock::new();

/// Modules register packet hooks at init (order = registration order).
pub fn add_hook(f: PacketHookFn) {
    HOOKS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push(f);
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
// Per-connection registry (bounded LRU)
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
}

#[derive(Default)]
struct ConnRegistry {
    map: HashMap<u64, ConnEntry>,
    /// LRU order: front = least recently used, back = most recently used.
    order: VecDeque<u64>,
}

impl ConnRegistry {
    fn touch(&mut self, conn_id: u64) {
        if let Some(pos) = self.order.iter().position(|&c| c == conn_id) {
            self.order.remove(pos);
            self.order.push_back(conn_id);
        }
    }
}

static CONNS: LazyLock<Mutex<ConnRegistry>> =
    LazyLock::new(|| Mutex::new(ConnRegistry::default()));

/// Register a connection. A re-attach of a live conn only refreshes the
/// player UUID and reports `false` (not newly created). Evicts the least
/// recently used conn when the table is at [`MAX_CONNS`].
pub fn attach_conn(conn_id: u64, player_uuid: Option<u128>) -> bool {
    let mut reg = CONNS.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(entry) = reg.map.get_mut(&conn_id) {
        entry.player_uuid = player_uuid;
        reg.touch(conn_id);
        return false;
    }
    if reg.map.len() >= MAX_CONNS {
        if let Some(oldest) = reg.order.pop_front() {
            reg.map.remove(&oldest);
        }
    }
    reg.map.insert(
        conn_id,
        ConnEntry {
            player_uuid,
            state: ProtocolState::Handshake,
        },
    );
    reg.order.push_back(conn_id);
    true
}

/// Forget a connection (called by the `onClose` hook on channel inactive).
/// Returns `true` if the conn was tracked.
pub fn detach_conn(conn_id: u64) -> bool {
    let mut reg = CONNS.lock().unwrap_or_else(|p| p.into_inner());
    reg.order.retain(|&c| c != conn_id);
    reg.map.remove(&conn_id).is_some()
}

/// Advance the connection's protocol state; used by the handshake and
/// protocol-swap hooks. Returns `false` (state unchanged) for unknown conns,
/// unknown state codes, or transitions outside the machine above.
pub fn set_conn_state(conn_id: u64, state_code: u8) -> bool {
    let Some(to) = ProtocolState::from_code(state_code) else {
        return false;
    };
    let mut reg = CONNS.lock().unwrap_or_else(|p| p.into_inner());
    let Some(entry) = reg.map.get_mut(&conn_id) else {
        return false;
    };
    if !transition_legal(entry.state, to) {
        return false;
    }
    entry.state = to;
    reg.touch(conn_id);
    true
}

/// Tracked protocol-state code for a connection, if any.
pub fn state_of(conn_id: u64) -> Option<u8> {
    CONNS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .map
        .get(&conn_id)
        .map(|e| e.state.code())
}

/// Tracked details (uuid + state) for a connection, if any.
pub fn conn_info(conn_id: u64) -> Option<ConnInfo> {
    CONNS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .map
        .get(&conn_id)
        .map(|e| ConnInfo {
            player_uuid: e.player_uuid,
            state: e.state.code(),
        })
}

/// Number of currently tracked connections.
pub fn conn_count() -> usize {
    CONNS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .map
        .len()
}

/// Tracked connection ids in LRU order (least recently used first).
pub fn conns() -> Vec<u64> {
    CONNS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .order
        .iter()
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Telemetry counters
// ---------------------------------------------------------------------------

/// Coalescing interval for metric publishes: the snapshot sees the current
/// totals at most once per second per counter.
const COUNTER_PUBLISH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

static PACKETS_IN: AtomicU64 = AtomicU64::new(0);
static PACKETS_OUT: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct CounterPub {
    last_in: Option<Instant>,
    last_out: Option<Instant>,
    last_dropped: Option<Instant>,
}

static COUNTER_PUB: Mutex<CounterPub> = Mutex::new(CounterPub {
    last_in: None,
    last_out: None,
    last_dropped: None,
});

enum CounterKind {
    In,
    Out,
    Dropped,
}

/// Publish the running total, coalesced to one push per second per counter.
fn publish_counter(kind: CounterKind, total: u64) {
    let mut pub_state = COUNTER_PUB.lock().unwrap_or_else(|p| p.into_inner());
    let (slot, name) = match kind {
        CounterKind::In => (&mut pub_state.last_in, "network.packets_in"),
        CounterKind::Out => (&mut pub_state.last_out, "network.packets_out"),
        CounterKind::Dropped => (&mut pub_state.last_dropped, "network.dropped"),
    };
    let now = Instant::now();
    if slot.is_none_or(|t| now.duration_since(t) >= COUNTER_PUBLISH_INTERVAL) {
        *slot = Some(now);
        publish_metric(name, total as f64, Some("packets"), None);
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

    let hooks = HOOKS
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default();
    for h in hooks {
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
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn packet(direction: Direction, conn_id: u64, state: u8, payload: &[u8]) -> Packet {
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
}
