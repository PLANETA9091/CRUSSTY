//! The dist engine — ported 1:1 from the v1 `crates/mod-native` (Java-less).
//!
//! Ownership split stays the same as v1, minus the JNI bridge: the KERNEL
//! (via the driver, on the server's main thread) feeds real metrics and
//! commits; Rust owns the wire protocol (UDP to oracle, Message codec,
//! lease state, fencing).
//!
//! v1 note kept: the native side never calls into the JVM; the kernel
//! *pulls* events via `poll_event` instead of us invoking callbacks.

use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Events surfaced to the driver, encoded as `type << 32 | region`.
pub const EVENT_LEASE_GRANT: u64 = 1;
pub const EVENT_LEASE_REVOKED: u64 = 2;

/// Event type constants, widened — the driver matches on `(ev >> 32) as u32`.
pub const EVENT_GRANT_U32: u32 = EVENT_LEASE_GRANT as u32;
pub const EVENT_REVOKED_U32: u32 = EVENT_LEASE_REVOKED as u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RegionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub u64);

/// A lease grants a node the right (and duty) to simulate a region.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Lease {
    pub region: RegionId,
    pub owner: NodeId,
    pub issued_at_tick: u64,
    pub expires_at_tick: u64,
    /// Monotonic fencing token — prevents a stale node from
    /// keeping a region it no longer owns (etcd/ZK practice).
    pub fencing_token: u64,
}

/// Node health snapshot, sent by nodes, used for balancing.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NodeHealth {
    pub node: NodeId,
    /// Higher = stronger machine (Crussty-style benchmark score).
    pub bench_score: f64,
    /// Current CPU-ish load 0.0..1.0 (1.0 = saturated).
    pub load: f64,
    /// Approx RTT to oracle in ms.
    pub ping_ms: u32,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct RegionCommit {
    pub region: RegionId,
    pub tick: u64,
    pub state_hash: [u8; 32],
}

/// Wire protocol messages, JSON-encoded for debug-ability in v1.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Message {
    Ping { nonce: u64 },
    /// Pong carries the oracle's current logical tick so nodes can estimate
    /// oracle-time (Lamport logical clock; the oracle stamps commit ticks).
    Pong { nonce: u64, tick: u64 },
    /// Node -> Oracle, every heartbeat_secs.
    Heartbeat(NodeHealth),
    /// Node -> Oracle, periodic region state commitment.
    Commit(RegionCommit),
    /// Oracle -> Node: "you own this region until expires_at_tick".
    LeaseGrant(Lease),
    /// Oracle -> Node: "release this region NOW, it has been reassigned".
    LeaseRevoked(Lease),
    /// Oracle -> Node: handshake for cross-node ownership transfer.
    RegionTransfer {
        region: RegionId,
        from: NodeId,
        to: NodeId,
        nonce: u64,
    },
    TransferAccept { region: RegionId, nonce: u64 },
    TransferDone { region: RegionId, nonce: u64 },
}

impl Lease {
    #[allow(dead_code)] // stable v1 API surface; used by control-plane wiring
    pub fn is_valid_at(&self, tick: u64) -> bool {
        self.issued_at_tick <= tick && tick < self.expires_at_tick
    }
}

pub struct Engine {
    sock: Arc<UdpSocket>,
    oracle: SocketAddr,
    node_id: NodeId,
    bench: f64,
    /// fencing token per owned region — stale/replayed leases must lose.
    fences: Mutex<HashMap<u32, u64>>,
    /// tombstone per revoked region: grants with token <= tombstone are stale
    /// (a re-grant to the NEW owner is what revoked us; the duplicate of the
    /// OLD grant must not resurrect the region on this node).
    revoked: Mutex<HashMap<u32, u64>>,
    /// last (local monotonic time, oracle logical tick) sync from Pong.
    sync: Mutex<Option<(std::time::Instant, u64)>>,
    last_ping: Mutex<std::time::Instant>,
    events: Mutex<VecDeque<u64>>,
    stop: Arc<AtomicBool>,
}

static ENGINE: Mutex<Option<Arc<Engine>>> = Mutex::new(None);

fn event(typ: u64, region: u32) -> u64 {
    (typ << 32) | region as u64
}

/// Fresh nonce for Ping (best-effort uniqueness — just needs to differ).
fn nonce() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

impl Engine {
    fn send(&self, msg: &Message) {
        if let Ok(payload) = serde_json::to_vec(msg) {
            let _ = self.sock.send_to(&payload, self.oracle);
        }
    }

    fn ping(&self) {
        self.send(&Message::Ping { nonce: nonce() });
    }

    fn heartbeat(&self, load: f64, ping_ms: u32) {
        self.send(&Message::Heartbeat(NodeHealth {
            node: self.node_id,
            bench_score: self.bench,
            load,
            ping_ms,
        }));
    }

    fn commit(&self, region: u32, tick: u64, state_hash: [u8; 32]) {
        self.send(&Message::Commit(RegionCommit {
            region: RegionId(region),
            tick,
            state_hash,
        }));
    }

    /// Estimated current oracle logical tick (from the last Pong), or None
    /// if never synced. tick_ms = oracle maintenance granularity.
    fn oracle_tick(&self, tick_ms: u64) -> Option<u64> {
        let (at, tick) = *self.sync.lock().unwrap().as_ref()?;
        let elapsed_ms = at.elapsed().as_millis() as u64;
        Some(tick + elapsed_ms / tick_ms.max(1))
    }
}

/// Drain the socket, dispatch grants/revocations into the event queue.
/// Re-pings every 60s to keep the oracle-tick estimate fresh.
fn recv_loop(engine: Arc<Engine>) {
    let mut buf = vec![0u8; 65536];
    while !engine.stop.load(Ordering::Relaxed) {
        if engine.last_ping.lock().unwrap().elapsed() >= Duration::from_secs(60) {
            engine.ping();
            *engine.last_ping.lock().unwrap() = std::time::Instant::now();
        }
        match engine.sock.recv_from(&mut buf) {
            Ok((len, _)) => {
                if let Ok(msg) = serde_json::from_slice::<Message>(&buf[..len]) {
                    dispatch(&engine, msg);
                }
            }
            Err(_) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

fn dispatch(engine: &Engine, msg: Message) {
    match msg {
        Message::Ping { nonce } => engine.send(&Message::Pong { nonce, tick: 0 }),
        Message::Pong { nonce: _, tick } => {
            *engine.sync.lock().unwrap() = Some((std::time::Instant::now(), tick));
        }
        Message::LeaseGrant(lease) => {
            if lease.owner != engine.node_id {
                return;
            }
            let mut fences = engine.fences.lock().unwrap();
            let mut revoked = engine.revoked.lock().unwrap();
            if let Some(tombstone) = revoked.get(&lease.region.0) {
                if *tombstone >= lease.fencing_token {
                    return; // this grant predates our revocation — stale
                }
                revoked.remove(&lease.region.0);
            }
            if let Some(existing) = fences.get(&lease.region.0) {
                if *existing > lease.fencing_token {
                    return; // stale
                }
            }
            fences.insert(lease.region.0, lease.fencing_token);
            engine
                .events
                .lock()
                .unwrap()
                .push_back(event(EVENT_LEASE_GRANT, lease.region.0));
        }
        Message::LeaseRevoked(lease) => {
            engine.fences.lock().unwrap().remove(&lease.region.0);
            engine
                .revoked
                .lock()
                .unwrap()
                .insert(lease.region.0, lease.fencing_token);
            engine
                .events
                .lock()
                .unwrap()
                .push_back(event(EVENT_LEASE_REVOKED, lease.region.0));
        }
        Message::Heartbeat(_) | Message::Commit(_) => {}
        Message::RegionTransfer { .. }
        | Message::TransferAccept { .. }
        | Message::TransferDone { .. } => {}
    }
}

/// Start the node engine. Returns 0 on success, -1 if already running,
/// -2 on bad oracle address.
pub fn start(oracle_addr: &str, node_id: u64, bench: f64) -> i32 {
    if ENGINE.lock().unwrap().is_some() {
        return -1;
    }
    let oracle: SocketAddr = match oracle_addr.parse() {
        Ok(a) => a,
        Err(_) => return -2,
    };
    let Some(sock) = UdpSocket::bind("0.0.0.0:0").ok() else {
        return -2;
    };
    let _ = sock.set_read_timeout(Some(Duration::from_millis(20)));
    let engine = Arc::new(Engine {
        sock: Arc::new(sock),
        oracle,
        node_id: NodeId(node_id),
        bench,
        fences: Mutex::new(HashMap::new()),
        revoked: Mutex::new(HashMap::new()),
        sync: Mutex::new(None),
        last_ping: Mutex::new(std::time::Instant::now()),
        events: Mutex::new(VecDeque::new()),
        stop: Arc::new(AtomicBool::new(false)),
    });
    *ENGINE.lock().unwrap() = Some(engine.clone());

    engine.ping(); // prime the oracle-tick estimate
    std::thread::Builder::new()
        .name("dist-recv".into())
        .spawn(move || recv_loop(engine))
        .ok();
    0
}

/// Stop the node run (recv thread exits on next timeout tick).
#[allow(dead_code)] // stable v1 API surface; control-plane shutdown wiring
pub fn stop() {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(engine) = guard.take() {
        engine.stop.store(true, Ordering::Relaxed);
    }
}

/// Pop the next pending event (lease grant / revocation). 0 = nothing new.
/// Event encoding: `type << 32 | region`.
pub fn poll_event() -> u64 {
    ENGINE
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|e| e.events.lock().unwrap().pop_front())
        .unwrap_or(0)
}

/// Feed a fresh load measurement; the Rust side keeps the protocol timing.
pub fn heartbeat(load: f64, ping_ms: u32) -> i32 {
    match ENGINE.lock().unwrap().as_ref() {
        Some(e) => {
            e.heartbeat(load, ping_ms);
            0
        }
        None => -1,
    }
}

/// Commit the current state hash of an owned region.
pub fn commit(region: u32, tick: u64, state_hash: [u8; 32]) -> i32 {
    match ENGINE.lock().unwrap().as_ref() {
        Some(e) => {
            e.commit(region, tick, state_hash);
            0
        }
        None => -1,
    }
}

/// Estimated current oracle logical tick (synced via Pong), or -1 if the
/// bridge has never heard a Pong. tick_ms = oracle maintenance granularity.
pub fn oracle_tick(tick_ms: u64) -> i64 {
    match ENGINE.lock().unwrap().as_ref() {
        Some(e) => e.oracle_tick(tick_ms).map(|t| t as i64).unwrap_or(-1),
        None => -1,
    }
}

pub fn running() -> bool {
    ENGINE.lock().unwrap().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_encoding_roundtrip() {
        assert_eq!((EVENT_LEASE_GRANT << 32) | 7, event(EVENT_LEASE_GRANT, 7));
    }

    #[test]
    fn lease_validity_window() {
        let l = Lease {
            region: RegionId(1),
            owner: NodeId(2),
            issued_at_tick: 10,
            expires_at_tick: 20,
            fencing_token: 5,
        };
        assert!(!l.is_valid_at(9));
        assert!(l.is_valid_at(10));
        assert!(l.is_valid_at(19));
        assert!(!l.is_valid_at(20));
    }
}
