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
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
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
use std::sync::OnceLock;
#[cfg(unix)]
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// Sliding-window TPS estimator, fed raw tick durations.
static TPS: Mutex<TpsWindow> = Mutex::new(TpsWindow {
    samples: VecDeque::new(),
    sum_ns: 0,
});

/// Ticks older than this fall out of the TPS window.
const TPS_WINDOW_SECS: u64 = 60;

/// Sliding-window TPS estimator.
///
/// Keeps the last [`TPS_WINDOW_SECS`] seconds of tick durations in a
/// ring-buffer style deque (time-based eviction, bounded memory) and
/// computes tps = 1000 / avg_ms. A running sum keeps each push O(1)
/// amortized. `Instant` (monotonic clock) is used for timestamps so wall
/// clock adjustments can never distort the window.
#[derive(Default)]
struct TpsWindow {
    samples: VecDeque<(Instant, u64)>,
    sum_ns: u64,
}

impl TpsWindow {
    fn push(&mut self, now: Instant, tick_ns: u64) {
        let cutoff = now
            .checked_sub(Duration::from_secs(TPS_WINDOW_SECS))
            .unwrap_or(now);
        while let Some(&(t, _)) = self.samples.front() {
            if t >= cutoff {
                break;
            }
            if let Some((_, d)) = self.samples.pop_front() {
                self.sum_ns -= d;
            }
        }
        self.samples.push_back((now, tick_ns));
        self.sum_ns += tick_ns;
    }

    fn avg_ms(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        Some(self.sum_ns as f64 / self.samples.len() as f64 / 1_000_000.0)
    }

    fn tps(&self) -> f64 {
        match self.avg_ms() {
            Some(ms) if ms > 0.0 => 1000.0 / ms,
            _ => 0.0,
        }
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.samples.clear();
        self.sum_ns = 0;
    }
}

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
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
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

pub fn set_tps(v: f64) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .tps = v;
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
/// tps = 1000 / avg_ms.
pub fn push_tick_time(tick_ns: u64) {
    let tps = {
        let mut w = TPS.lock().unwrap_or_else(|p| p.into_inner());
        w.push(Instant::now(), tick_ns);
        w.tps()
    };
    set_tps(tps);
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
        TPS.lock().unwrap_or_else(|p| p.into_inner()).clear();
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
        let mut w = TpsWindow::default();
        assert_eq!(w.tps(), 0.0);

        // 100 samples of 50ms -> avg 50ms -> tps 20
        let t0 = Instant::now();
        for i in 0..100 {
            w.push(t0 + Duration::from_millis(i * 50), 50_000_000);
        }
        assert!((w.tps() - 20.0).abs() < 1e-6);

        // samples older than the 60s window are evicted
        let mut w2 = TpsWindow::default();
        w2.push(t0, 50_000_000);
        w2.push(t0 + Duration::from_secs(61), 40_000_000);
        assert!((w2.tps() - 25.0).abs() < 1e-6);
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
