//! Brick 7: telemetry channel — local socket for hosting panels and admins.
//!
//! Exposes a Unix domain socket (JSON lines protocol) with live metrics:
//! TPS, memory, loaded modules, and arbitrary module-published metrics.
//! Panels (Pterodactyl etc.) and monitoring tools connect without any
//! plugin or web server.
//!
//! # Wire protocol (Unix only)
//!
//! One request/response exchange per connection, then the server closes:
//!
//! ```text
//! client:  "stats\n"            (or nothing / EOF)
//! server:  {"runtime_version": "...", ...}\n
//! ```
//!
//! The request is a single line read with a 1s idle timeout. `stats`
//! (case-insensitive) and empty requests get the JSON snapshot as one
//! newline-terminated line (NDJSON). Unknown requests get
//! `400 bad request\n`. If more than [`MAX_HANDLERS`] connections are being
//! served at once, the new connection gets `503 busy\n` immediately.
//!
//! # Design notes
//!
//! - Thread-per-connection accept loop (the std-docs pattern for
//!   [`std::os::unix::net::UnixListener`]) with a bounded-concurrency guard
//!   so a stalled client can never exhaust resources.
//! - Socket reads and writes have timeouts (1s / 5s) so a dead client never
//!   hangs a handler slot.
//! - The 1s refresh thread (uptime + `/proc/self/status` VmRSS/VmHWM) only
//!   touches the shared snapshot; it never calls into the JVM.
//! - TPS is derived from raw tick durations fed by modules/transform hooks
//!   ([`push_tick_time`]) over a sliding 1-minute window: tps = 1000/avg_ms.
//!
//! On non-Unix targets the socket machinery compiles away and
//! [`init`] is a no-op; the data API (snapshot, setters, metrics) still
//! works everywhere so the Windows build stays functional.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::net::Shutdown;
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::thread;

/// One published metric value (double or labeled).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// The panel-facing snapshot serialized to JSON.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Snapshot {
    /// crussty-runtime version (crate version).
    pub runtime_version: String,
    /// Server name, see [`set_server_name`].
    pub server_name: String,
    /// Unix timestamp (seconds) of the first snapshot initialization.
    pub started_at: u64,
    pub uptime_secs: u64,
    pub tps: f64,
    pub mem_used_mb: u64,
    pub mem_max_mb: u64,
    pub loaded_modules: Vec<String>,
    pub metrics: Vec<Metric>,
}

/// Hard cap on module-published metrics: [`publish_metric`] drops anything
/// past this so the snapshot can never grow unboundedly.
pub const MAX_METRICS: usize = 4096;

/// Maximum concurrently served client connections. Additional connections
/// receive an immediate `503 busy\n` line (bounded-concurrency admission
/// control, the standard production pattern for thread-per-connection
/// servers).
#[cfg(unix)]
pub const MAX_HANDLERS: usize = 16;

/// Snapshot storage. The outer mutex allows swapping the snapshot in tests;
/// the inner mutex guards the data itself.
static SNAPSHOT: Mutex<Option<Arc<Mutex<Snapshot>>>> = Mutex::new(None);
#[cfg(unix)]
static LISTENER: OnceLock<PathBuf> = OnceLock::new();
#[cfg(unix)]
static ACTIVE_HANDLERS: AtomicUsize = AtomicUsize::new(0);

/// Lock-free tick ring (TASK-164): the per-tick TPS ingestion writes two
/// relaxed atomic stores + one fetch_add — no mutex, no window scan, no
/// division on the hot path. The 60s-window average is computed lazily at
/// `snapshot()` time (panel-cold path).
const RING_CAP: usize = 4096;
static RING_HEAD: AtomicU64 = AtomicU64::new(0);
static RING_TS: [AtomicU64; RING_CAP] = [const { AtomicU64::new(0) }; RING_CAP];
static RING_NS: [AtomicU64; RING_CAP] = [const { AtomicU64::new(0) }; RING_CAP];
/// Process-relative monotonic epoch for ring timestamps (nanos since epoch,
/// 0 = empty slot; stored values are offset by +1).
static RING_EPOCH: OnceLock<Instant> = OnceLock::new();

/// Last TPS value as raw f64 bits (TASK-162): the per-tick store is a single
/// relaxed atomic write — no snapshot lock, no Arc clone, no inner mutex.
/// `snapshot()` overlays this onto the returned Snapshot, so readers see the
/// latest value exactly as before; the static starts at 0 bits = 0.0 f64.
static TPS_LAST_BITS: AtomicU64 = AtomicU64::new(0);

/// Ticks older than this fall out of the TPS window.
const TPS_WINDOW_SECS: u64 = 60;

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn new_default_snapshot() -> Snapshot {
    Snapshot {
        runtime_version: env!("CARGO_PKG_VERSION").to_string(),
        server_name: "crussty".to_string(),
        started_at: unix_now_secs(),
        ..Snapshot::default()
    }
}

/// Get-or-create the shared snapshot handle (poison-tolerant: a panicked
/// holder must not take the whole telemetry brick down).
fn snapshot_arc() -> Arc<Mutex<Snapshot>> {
    let mut guard = SNAPSHOT.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(s) = guard.as_ref() {
        return Arc::clone(s);
    }
    let s = Arc::new(Mutex::new(new_default_snapshot()));
    *guard = Some(Arc::clone(&s));
    s
}

pub fn snapshot() -> Snapshot {
    let mut s = snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    // TASK-164: TPS comes from the lock-free ring (lazy 60s window average);
    // `set_tps` still wins when no tick has ever been ingested, so external
    // callers keep their existing semantics.
    s.tps = ring_tps().unwrap_or_else(current_tps);
    s
}

/// Modules publish metrics; the panel reads them in the snapshot.
/// Publish is capped at [`MAX_METRICS`] entries; excess metrics are dropped.
pub fn publish_metric(
    name: &str,
    value: f64,
    unit: Option<&str>,
    labels: Option<HashMap<String, String>>,
) {
    let snap = snapshot_arc();
    let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
    if s.metrics.len() >= MAX_METRICS {
        return;
    }
    s.metrics.push(Metric {
        name: name.to_string(),
        value,
        labels,
        unit: unit.map(str::to_string),
    });
}

/// Exact pre-parse gate for the C entry (TASK-180): `true` once the metric
/// list is at [`MAX_METRICS`] — from then on [`publish_metric`] drops every
/// further metric, so a caller that still has to PARSE its labels payload
/// may skip that work (the parsed map would be unobservable). The check
/// takes the same two uncontended locks the publish itself would take
/// (~30-40 ns) — an order of magnitude below the serde parse it saves.
/// Recomputed under the lock every call, so snapshot swaps in tests cannot
/// desynchronize it.
pub fn metrics_full() -> bool {
    let snap = snapshot_arc();
    let s = snap.lock().unwrap_or_else(|p| p.into_inner());
    s.metrics.len() >= MAX_METRICS
}

/// TASK-181: serialize the current snapshot JSON straight into `out`
/// WITHOUT the deep clone the `snapshot()` + `serde_json::to_string` pair
/// pays (that clone allocates every String / labels map in the snapshot —
/// up to [`MAX_METRICS`] metrics — and is then dropped untouched by the
/// serializer). The output is byte-identical to
/// `serde_json::to_string(&snapshot())` (same derived Serialize, same
/// field order, `to_string` is `to_writer` into a String).
///
/// TPS contract identical to [`snapshot`]: the serialized value carries
/// `ring_tps().unwrap_or(current_tps)`. The value is read BEFORE the data
/// lock is taken (both sources are lock-free atomics — TASK-162/164), then
/// applied as a transient in-place override of the stored field under the
/// lock and restored right after serialization: no lock holder can observe
/// the override mid-call, and the stored field itself has no reader that
/// does not override it (every reader goes through [`snapshot`]-shaped
/// clones). On a panic the runtime aborts (the only caller is an
/// extern "C" entry), so the override cannot outlive the call.
///
/// Lock ordering: takes the snapshot data lock only (a leaf — nothing it
/// calls takes another lock); the caller may hold its own buffer mutex
/// around this with no inverse order anywhere. The ring scan that may run
/// under the lock is the bounded lock-free RING_CAP loop, not a mutex.
/// Single call site (the C entry); #[inline(never)] keeps the serializer
/// out of the caller's code region — its cost is serde-dominated and it
/// must not perturb unrelated hot-loop placement (TASK-181 iteration 1).
#[inline(never)]
#[allow(dead_code)] // TASK-181 E2b bisect (caller parked in c_bridge patch)
pub(crate) fn snapshot_json_write(out: &mut Vec<u8>) {
    let tps = ring_tps().unwrap_or_else(current_tps);
    let snap = snapshot_arc();
    let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
    let saved = s.tps;
    s.tps = tps;
    let res = serde_json::to_writer(&mut *out, &*s);
    s.tps = saved;
    if res.is_err() {
        // Unreachable for this Serialize impl (no fallible parts), kept for
        // parity with the legacy `{}` fallback.
        out.clear();
        out.extend_from_slice(b"{}");
    }
}

/// Test-only: drop the shared snapshot so the next publish starts a fresh
/// list. The list itself has no removal API by design (append-only up to
/// the cap); full-state tests need a way back out of it. Never call from
/// a test that runs in parallel with metric-reading tests — filtered
/// single-test runs only.
#[cfg(test)]
pub(crate) fn test_reset_snapshot() {
    *SNAPSHOT.lock().unwrap_or_else(|p| p.into_inner()) = None;
}

/// Store the latest TPS (external callers; the per-tick path feeds the ring
/// instead — see [`push_tick_time_at`]).
pub fn set_tps(v: f64) {
    TPS_LAST_BITS.store(v.to_bits(), Ordering::Relaxed);
}

/// Read the current TPS without touching any lock (for hot readers).
fn current_tps() -> f64 {
    f64::from_bits(TPS_LAST_BITS.load(Ordering::Relaxed))
}

#[cfg(test)]
mod bench_hotpath {
    //! Release-only A/B bench: `cargo test --release -- --ignored --nocapture bench_push_tick`.
    use super::*;

    /// Per-tick ingestion cost INCLUDING the caller's clock read (the
    /// `push_tick_time` public API reads Instant::now itself).
    #[test]
    #[ignore]
    fn bench_push_tick_time() {
        push_tick_time(16_666_667); // pre-touch: ring init off the clock
        let iters = 200_000u32;
        let rounds = 5;
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for i in 0..iters {
                push_tick_time(16_666_667 + u64::from(i % 1_000));
            }
            best = best.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // TASK-164: the production per-tick shape — the tick boundary has
        // already read the clock, so ingestion alone is what the server pays.
        let at = Instant::now();
        for _ in 0..10_000u32 {
            push_tick_time_at(at, 16_666_667);
        }
        let mut best_at = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for i in 0..iters {
                push_tick_time_at(at, 16_666_667 + u64::from(i % 1_000));
            }
            best_at = best_at.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // The vDSO clock read alone — the hardware floor every
        // clock-reading path (tick boundary included) carries.
        let mut best_clock = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                std::hint::black_box(Instant::now());
            }
            best_clock = best_clock.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH telemetry: push_tick_time {:.0} ns/op (incl clock), push_tick_time_at {:.0} ns/op, clock_read {:.0} ns/op (min of {rounds}x{iters})",
            best * 1e9,
            best_at * 1e9,
            best_clock * 1e9
        );
    }
}

pub fn set_mem(used_mb: u64, max_mb: u64) {
    let snap = snapshot_arc();
    let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
    s.mem_used_mb = used_mb;
    s.mem_max_mb = max_mb;
}

pub fn set_uptime(secs: u64) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .uptime_secs = secs;
}

pub fn set_modules(names: Vec<String>) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .loaded_modules = names;
}

pub fn set_server_name(name: &str) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .server_name = name.to_string();
}

/// Feed a raw tick duration (ns) from a module or transform hook; the
/// sliding 1-minute window automatically updates the snapshot TPS as
/// tps = 1000 / avg_ms. Reads the clock itself (the scheduler boundary path
/// uses [`push_tick_time_at`] to share its own clock read).
pub fn push_tick_time(tick_ns: u64) {
    push_tick_time_at(Instant::now(), tick_ns);
}

/// Same ingestion with a caller-supplied timestamp: the tick boundary has
/// already read the clock when it computes the tick duration, so the window
/// reuses that instant instead of paying a second clock read per tick
/// (TASK-162). Lock-free (TASK-164): one fetch_add + two atomic stores —
/// the window average is computed lazily by [`snapshot`].
pub fn push_tick_time_at(at: Instant, tick_ns: u64) {
    let i = (RING_HEAD.fetch_add(1, Ordering::Relaxed) & (RING_CAP as u64 - 1)) as usize;
    RING_NS[i].store(tick_ns, Ordering::Relaxed);
    let epoch = *RING_EPOCH.get_or_init(Instant::now);
    let ts = at.saturating_duration_since(epoch).as_nanos() as u64;
    // Offset by +1 so 0 stays the "empty slot" sentinel.
    RING_TS[i].store(ts.saturating_add(1), Ordering::Release);
}

/// 60s-window TPS over the ring, computed at snapshot time (cold). The
/// window reference is the NEWEST sample's timestamp — no clock read, no
/// epoch dependency — so the eviction semantics are testable with fabricated
/// instants and identical to the old wall-window shape in production (the
/// newest sample is always the just-pushed tick).
fn ring_tps() -> Option<f64> {
    let head = RING_HEAD.load(Ordering::Relaxed);
    if head == 0 {
        return None;
    }
    let newest_i = ((head - 1) & (RING_CAP as u64 - 1)) as usize;
    let newest_ts = RING_TS[newest_i].load(Ordering::Acquire);
    if newest_ts == 0 {
        return None;
    }
    let cutoff = (newest_ts - 1).saturating_sub(TPS_WINDOW_SECS * 1_000_000_000);
    let mut sum = 0u64;
    let mut n = 0u64;
    for k in 0..RING_CAP {
        let i = (head.wrapping_sub(k as u64 + 1) & (RING_CAP as u64 - 1)) as usize;
        let ts = RING_TS[i].load(Ordering::Acquire);
        if ts == 0 {
            break; // reached the not-yet-written part of the ring
        }
        if ts - 1 < cutoff {
            continue; // outside the 60s window
        }
        let ns = RING_NS[i].load(Ordering::Relaxed);
        // Torn-read guard: the slot was rewritten mid-scan — skip it.
        if RING_TS[i].load(Ordering::Acquire) != ts {
            continue;
        }
        sum += ns;
        n += 1;
    }
    if n == 0 {
        return None;
    }
    let avg_ms = sum as f64 / n as f64 / 1_000_000.0;
    Some(if avg_ms > 0.0 { 1000.0 / avg_ms } else { 0.0 })
}

/// Bind the telemetry socket and start the accept + refresh threads.
/// Idempotent: a second call with a different path is a no-op.
#[cfg(unix)]
pub fn init(socket_path: &str) -> std::io::Result<()> {
    if LISTENER.get().is_some() {
        return Ok(());
    }
    let path = PathBuf::from(socket_path);
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    let listener = UnixListener::bind(&path)?;
    let _ = LISTENER.set(path);
    spawn_refresh_thread()?;
    thread::Builder::new()
        .name("crussty-telemetry".into())
        .spawn(move || serve(listener))?;
    Ok(())
}

#[cfg(not(unix))]
pub fn init(_socket_path: &str) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn serve(listener: UnixListener) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue; // accept errors are transient; keep serving
        };
        let Some(guard) = HandlerGuard::try_acquire() else {
            let mut s = stream;
            let _ = s.set_write_timeout(Some(Duration::from_secs(5)));
            let _ = s.write_all(b"503 busy\n");
            let _ = s.shutdown(Shutdown::Write);
            continue;
        };
        let _ = thread::Builder::new()
            .name("crussty-telemetry-conn".into())
            .spawn(move || handle_client(stream, guard));
    }
}

#[cfg(unix)]
fn handle_client(mut stream: UnixStream, _guard: HandlerGuard) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let request = read_request(&mut stream).unwrap_or_default();
    match request.to_ascii_lowercase().as_str() {
        "" | "stats" => {
            let snap = snapshot();
            let line = match serde_json::to_string(&snap) {
                Ok(json) => {
                    let mut line = json;
                    line.push('\n');
                    line
                }
                Err(_) => "500 internal error\n".to_string(),
            };
            let _ = stream.write_all(line.as_bytes());
            let _ = stream.flush();
        }
        _ => {
            let _ = stream.write_all(b"400 bad request\n");
        }
    }
    let _ = stream.shutdown(Shutdown::Write);
}

/// Read one request line (bounded), tolerating clients that send nothing:
/// EOF or an idle timeout is treated as an empty request.
#[cfg(unix)]
fn read_request(stream: &mut UnixStream) -> std::io::Result<String> {
    let mut buf = [0u8; 128];
    let mut used = 0usize;
    loop {
        if used >= buf.len() {
            break;
        }
        match stream.read(&mut buf[used..]) {
            Ok(0) => break, // client half-closed: empty request
            Ok(n) => {
                used += n;
                if buf[..used].contains(&b'\n') {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break; // idle client: serve a snapshot rather than hang a slot
            }
            Err(e) => return Err(e),
        }
    }
    Ok(String::from_utf8_lossy(&buf[..used]).trim().to_string())
}

/// Bounded-concurrency slot for handler threads; the slot is released when
/// the guard drops (panic-safe admission control).
#[cfg(unix)]
struct HandlerGuard;

#[cfg(unix)]
impl HandlerGuard {
    fn try_acquire() -> Option<Self> {
        let prev = ACTIVE_HANDLERS.fetch_add(1, Ordering::Relaxed);
        if prev >= MAX_HANDLERS {
            ACTIVE_HANDLERS.fetch_sub(1, Ordering::Relaxed);
            return None;
        }
        Some(HandlerGuard)
    }
}

#[cfg(unix)]
impl Drop for HandlerGuard {
    fn drop(&mut self) {
        ACTIVE_HANDLERS.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Every 1s: refresh uptime and (on Linux) RSS memory from
/// `/proc/self/status`. This thread never calls into the JVM — it only
/// updates the shared snapshot.
#[cfg(unix)]
fn spawn_refresh_thread() -> std::io::Result<()> {
    thread::Builder::new()
        .name("crussty-telemetry-refresh".into())
        .spawn(|| loop {
            thread::sleep(Duration::from_secs(1));
            let now = unix_now_secs();
            let (used_kb, max_kb) = read_proc_mem_kb();
            let snap = snapshot_arc();
            let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
            s.uptime_secs = now.saturating_sub(s.started_at);
            if used_kb > 0 || max_kb > 0 {
                s.mem_used_mb = used_kb / 1024;
                s.mem_max_mb = max_kb / 1024;
            }
        })?;
    Ok(())
}

/// RSS from `/proc/self/status` in kB: VmRSS (current) and VmHWM (peak
/// resident set, the "high water mark"). Falls back to (0, 0) when the
/// file is unreadable so a manual `set_mem` value is never clobbered.
#[cfg(target_os = "linux")]
fn read_proc_mem_kb() -> (u64, u64) {
    let Ok(content) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let mut used_kb = 0u64;
    let mut max_kb = 0u64;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            used_kb = parse_kb_value(rest);
        } else if let Some(rest) = line.strip_prefix("VmHWM:") {
            max_kb = parse_kb_value(rest);
        }
    }
    (used_kb, max_kb)
}

#[cfg(not(target_os = "linux"))]
fn read_proc_mem_kb() -> (u64, u64) {
    (0, 0)
}

#[cfg(target_os = "linux")]
fn parse_kb_value(rest: &str) -> u64 {
    rest.split_whitespace()
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serializes tests that touch global state (the shared snapshot and
    /// the TPS window live for the whole test process).
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        *SNAPSHOT.lock().unwrap_or_else(|p| p.into_inner()) = None;
        RING_HEAD.store(0, Ordering::Relaxed);
        for slot in RING_TS.iter() {
            slot.store(0, Ordering::Relaxed);
        }
        for slot in RING_NS.iter() {
            slot.store(0, Ordering::Relaxed);
        }
        TPS_LAST_BITS.store(0, Ordering::Relaxed);
    }

    #[test]
    fn snapshot_json_has_all_fields() {
        let s = Snapshot {
            runtime_version: "2.0.0".to_string(),
            server_name: "test-host".to_string(),
            started_at: 1234,
            uptime_secs: 42,
            tps: 20.0,
            mem_used_mb: 100,
            mem_max_mb: 200,
            loaded_modules: vec!["hello".to_string()],
            metrics: vec![Metric {
                name: "tick_ms".to_string(),
                value: 50.0,
                labels: None,
                unit: Some("ms".to_string()),
            }],
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(v["runtime_version"], "2.0.0");
        assert_eq!(v["server_name"], "test-host");
        assert_eq!(v["started_at"].as_u64(), Some(1234));
        assert_eq!(v["uptime_secs"].as_u64(), Some(42));
        assert_eq!(v["tps"].as_f64(), Some(20.0));
        assert_eq!(v["mem_used_mb"].as_u64(), Some(100));
        assert_eq!(v["mem_max_mb"].as_u64(), Some(200));
        assert_eq!(v["loaded_modules"][0], "hello");
        assert_eq!(v["metrics"][0]["name"], "tick_ms");
        assert_eq!(v["metrics"][0]["unit"], "ms");
        assert!(v["metrics"][0].get("labels").is_none());
    }

    #[test]
    fn setters_write_to_snapshot() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        set_server_name("alpha");
        set_tps(19.5);
        set_mem(10, 20);
        set_uptime(7);
        set_modules(vec!["a".to_string(), "b".to_string()]);
        let s = snapshot();
        assert_eq!(s.server_name, "alpha");
        assert_eq!(s.tps, 19.5);
        assert_eq!(s.mem_used_mb, 10);
        assert_eq!(s.mem_max_mb, 20);
        assert_eq!(s.uptime_secs, 7);
        assert_eq!(s.loaded_modules, vec!["a".to_string(), "b".to_string()]);
        assert!(s.started_at > 0);
        assert!(!s.runtime_version.is_empty());
    }

    #[test]
    fn metric_cap_is_enforced() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        for i in 0..(MAX_METRICS + 10) {
            publish_metric(&format!("m{i}"), i as f64, None, None);
        }
        assert_eq!(snapshot().metrics.len(), MAX_METRICS);
    }

    #[test]
    fn tps_window_math() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();

        // 100 samples of 50ms -> avg 50ms -> tps 20. Timestamps are
        // fabricated relative to the real clock (the ring's window is
        // wall-relative), starting at now so nothing pre-dates the epoch.
        let t0 = Instant::now();
        for i in 0..100 {
            push_tick_time_at(t0 + Duration::from_millis(i * 50), 50_000_000);
        }
        let s = snapshot();
        assert!((s.tps - 20.0).abs() < 1e-6, "tps {} != 20", s.tps);

        // samples older than the 60s window are evicted. The window
        // reference is the NEWEST sample's timestamp, so a sample stamped
        // 61s after the first one pushes it out of the 60s window (instants
        // may be fabricated into the future; pre-epoch ones would clamp to
        // the epoch and never look old).
        reset_state();
        let t = Instant::now();
        push_tick_time_at(t, 50_000_000);
        push_tick_time_at(t + Duration::from_secs(61), 40_000_000);
        let s2 = snapshot();
        assert!((s2.tps - 25.0).abs() < 1e-6, "tps {} != 25", s2.tps);
    }

    #[test]
    fn push_tick_time_drives_tps() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        for _ in 0..100 {
            push_tick_time(50_000_000);
        }
        assert!((snapshot().tps - 20.0).abs() < 1e-6);
    }

    #[cfg(unix)]
    #[test]
    fn handler_slots_are_bounded() {
        let mut held = Vec::new();
        while let Some(g) = HandlerGuard::try_acquire() {
            held.push(g);
        }
        assert!(held.len() <= MAX_HANDLERS);
        assert!(HandlerGuard::try_acquire().is_none());
        let _ = held.pop();
        assert!(HandlerGuard::try_acquire().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn init_connect_receive_roundtrip() {
        let _guard = TEST_LOCK.lock().unwrap();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "crussty-telemetry-{}-{}.sock",
            std::process::id(),
            nanos
        ));
        let path_str = path.to_str().unwrap().to_string();
        init(&path_str).unwrap();

        // "stats" request -> JSON snapshot, one line, then EOF.
        let mut s = UnixStream::connect(&path_str).unwrap();
        s.write_all(b"stats\n").unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.ends_with('\n'));
        let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
        assert!(v["started_at"].as_u64().unwrap() > 0);
        assert!(!v["runtime_version"].as_str().unwrap().is_empty());
        assert_eq!(v["server_name"], "crussty");
        assert!(v.get("uptime_secs").is_some());
        assert!(v.get("mem_used_mb").is_some());
        assert!(v.get("loaded_modules").is_some());

        // empty request (client half-closes) -> JSON snapshot.
        let mut s2 = UnixStream::connect(&path_str).unwrap();
        s2.shutdown(Shutdown::Write).unwrap();
        let mut out2 = String::new();
        s2.read_to_string(&mut out2).unwrap();
        let v2: serde_json::Value = serde_json::from_str(out2.trim()).unwrap();
        assert!(v2["started_at"].as_u64().unwrap() > 0);

        // unknown request -> 400.
        let mut s3 = UnixStream::connect(&path_str).unwrap();
        s3.write_all(b"bogus\n").unwrap();
        let mut out3 = String::new();
        s3.read_to_string(&mut out3).unwrap();
        assert!(out3.starts_with("400"));

        let _ = std::fs::remove_file(&path);
    }
}
