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

/// TASK-194: snapshot content generation. Every mutator of the shared
/// snapshot data (publish_metric, set_mem, set_uptime, set_modules,
/// set_server_name, the 1s refresh thread, the test resets) bumps this
/// counter while holding the data lock; the JSON cache below compares its
/// cached generation against it and reserializes only on a mismatch.
/// Relaxed orderings are sufficient: every load/store happens while the
/// caller holds the snapshot data mutex, and the mutex lock/unlock pair
/// provides the happens-before edge between mutator and reader.
static SNAP_GEN: AtomicU64 = AtomicU64::new(0);

fn bump_snapshot_gen() {
    SNAP_GEN.fetch_add(1, Ordering::Relaxed);
}

/// TASK-194: pre-encoded snapshot JSON, split around the tps value span.
/// `prefix` + `suffix` are the byte ranges of a full serde serialization
/// of the snapshot before/after the `"tps":` VALUE — built by serde itself
/// (never hand-rolled), so the cache-hit output is byte-identical to a
/// fresh serialize as long as the content generation is unchanged. The
/// live value (tps, ring-fed, changes per call in production) is patched
/// between the two ranges at read time from the TASK-198 tps memo;
/// `scratch_a`/`scratch_b` are reused so the dirty path does not
/// allocate in the steady state either.
struct SnapshotJsonCache {
    gen: u64,
    prefix: Vec<u8>,
    suffix: Vec<u8>,
    /// TASK-198: incremental 60s-window tps accumulator + cached serde
    /// bytes for the live tps value (see [`TpsMemo`]). Replaces the
    /// TASK-194 `tps_buf` — the per-hit serde f64 format it served is
    /// gone from the steady-state read path entirely.
    tps_memo: TpsMemo,
    scratch_a: Vec<u8>,
    scratch_b: Vec<u8>,
    valid: bool,
}

/// TASK-198: incremental state for the live tps on the snapshot READ path.
///
/// WHY: the landed snapshot_json lines are an empty-ring shape — the bench
/// never pushes ticks, so `ring_tps()` returns None after one load. In
/// production the per-tick path feeds the ring and `ring_tps()` scans up
/// to RING_CAP (4096) slots per read (measured 1290 ns/op warm vs the
/// landed 92 ns line): the TASK-194 cache win is masked behind that scan
/// plus a per-read serde f64 format.
///
/// VALUE CONTRACT: the memo serves EXACTLY the verbatim expression
/// `ring_tps().unwrap_or_else(current_tps)`:
/// - Under monotone ingest (the epoch contract: one clock form per
///   process on the per-tick path; TSC and vDSO monotone) the in-window
///   population is a contiguous suffix of push order, so an
///   add-on-consume + forward-evict-cursor accumulator reproduces the
///   backward scan's (sum, n) exactly; the tps formula is applied to the
///   same integers, so the f64 bits match.
/// - Any anomaly (torn slot read, ts regression/dip = clock-form switch,
///   ring reset, cursor lap past RING_CAP) forces a VERBATIM rebuild —
///   the rebuild loop is a byte-for-byte clone of `ring_tps()`'s scan,
///   so it is exact for arbitrary ring content.
/// - The fallback arm (ring empty / newest sample in flight) serves
///   `current_tps()` keyed on `TPS_LAST_BITS` — the same
///   `unwrap_or_else` shape.
/// - RACE CLASS (documented, not eliminated): with multiple concurrent
///   producers a slot may be observed in flight (ts store not landed);
///   the memo consumes the contiguous oldest written run while the
///   verbatim scan consumes the contiguous newest one. Single-producer
///   ingest (the production scheduler boundary thread) is exact; the
///   multi-producer transient differs by in-flight samples — the same
///   nondeterminism class the scan's own torn-guard exhibits.
/// - THROUGHPUT BOUND: the cursor guards force a verbatim rebuild once
///   the 60s window no longer fits the ring (>~68 ticks/s sustained).
///   Beyond the bound the memo is cost-neutral (one scan per read, same
///   as the verbatim shape), never worse.
///
/// The memo lives INSIDE the cache struct: one lock, one leaf, and the
/// steady-state resolve is two atomic loads + two integer compares.
struct TpsMemo {
    /// false = next resolve must rebuild (anomaly or first use).
    valid: bool,
    /// true = tps/bytes describe a ring-derived value; false = fallback
    /// (bits-keyed) value.
    ring_keyed: bool,
    /// Fallback key: TPS_LAST_BITS at resolve time.
    k_bits: u64,
    /// Absolute push index consumed THROUGH (every issued push below this
    /// is reflected in a_sum/a_n or deliberately skipped as in-flight).
    a_head: u64,
    /// Absolute push index of the oldest still-counted sample (evict
    /// cursor; only moves forward).
    a_evict: u64,
    /// Sum of tick_ns over [a_evict, a_head).
    a_sum: u64,
    /// Count of samples over [a_evict, a_head).
    a_n: u64,
    /// Stored ts (+1 form) of the last consumed sample — monotonicity
    /// sentinel for regression/dip detection.
    last_ts: u64,
    /// Resolved value (ring formula or fallback bits).
    tps: f64,
    /// serde-formatted bytes of `tps` (produced by serde itself, never
    /// hand-rolled — the TASK-194 byte-identity discipline).
    bytes_len: usize,
    bytes: [u8; 40],
}

impl Default for SnapshotJsonCache {
    fn default() -> Self {
        SnapshotJsonCache {
            gen: 0,
            prefix: Vec::new(),
            suffix: Vec::new(),
            tps_memo: TpsMemo {
                valid: false,
                ring_keyed: false,
                k_bits: 0,
                a_head: 0,
                a_evict: 0,
                a_sum: 0,
                a_n: 0,
                last_ts: 0,
                tps: 0.0,
                bytes_len: 0,
                bytes: [0; 40],
            },
            scratch_a: Vec::new(),
            scratch_b: Vec::new(),
            valid: false,
        }
    }
}

/// Cache lock discipline (TASK-199 revision): the cache lock is the ONLY
/// lock on the HIT path — a hit assembles the output purely from the
/// cached ranges + memo bytes and never touches the live snapshot, so the
/// SNAP_GEN load on the hit path is lock-free and linearized (stale gen =
/// coherent pre-mutation snapshot; bumped gen = the miss path below). The
/// data lock is taken ONLY on the miss/rebuild path, in cache -> data
/// order, with the generation RE-VERIFIED under the data lock (another
/// reader may have refilled while we waited). Safety invariants: nothing
/// that holds the data lock may acquire the cache lock (every mutator
/// bumps SNAP_GEN with one atomic fetch_add while holding the data lock
/// and never touches the cache; no other site locks this static —
/// grep-verified single acquisition point), so no inverse order can form.
/// The cache lock itself remains a leaf. No other code touches this
/// static.
static SNAP_JSON_CACHE: Mutex<SnapshotJsonCache> = Mutex::new(SnapshotJsonCache {
    gen: 0,
    prefix: Vec::new(),
    suffix: Vec::new(),
    tps_memo: TpsMemo {
        valid: false,
        ring_keyed: false,
        k_bits: 0,
        a_head: 0,
        a_evict: 0,
        a_sum: 0,
        a_n: 0,
        last_ts: 0,
        tps: 0.0,
        bytes_len: 0,
        bytes: [0; 40],
    },
    scratch_a: Vec::new(),
    scratch_b: Vec::new(),
    valid: false,
});

/// TASK-194 A/B toggle: ON = generation-gated pre-encoded cache (O(buffer)
/// assemble on a clean generation); OFF = the pre-TASK-194 in-place
/// reserialize shape, kept verbatim (the byte-identity tests re-derive it
/// independently).
const TELEM_SNAP_CACHE: bool = true;

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
    bump_snapshot_gen(); // TASK-194: invalidate the pre-encoded JSON cache
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

/// TASK-194: serialize the current snapshot JSON straight into `out`
/// WITHOUT reserializing when the snapshot content is unchanged since the
/// last call. Two paths behind the [`TELEM_SNAP_CACHE`] toggle:
///
/// - ON (default): the snapshot's JSON is cached as a prefix/suffix pair
///   split around the `"tps":` value, built by serde itself at fill time
///   (serialize the locked snapshot twice — tps 0.0 and tps 1.0 — and split
///   at the first differing byte, which is the tps digit; the two bodies
///   are equal-length and otherwise byte-identical). On a cache hit
///   (content generation [`SNAP_GEN`] unchanged) the output is assembled as
///   prefix + fresh tps bytes + suffix — an O(buffer) copy with the ONLY
///   live value (tps, ring-fed, changes per call in production) patched
///   in; the tps bytes come from the same serde_json f64 serializer, so
///   the output is byte-identical to a fresh serialize by construction.
///   On a generation mismatch the cache is rebuilt (two cold serializations
///   per dirty call — once per publish batch / refresh-second, never per
///   read) and the call is served from the just-built cache. Neither path
///   allocates in the steady state (all buffers live in the cache static).
///   This replaces the TASK-181 every-call `serde_json::to_writer`, which
///   paid the full ~9 µs serialize on every panel read (measured 8929 ns
///   for 32 labeled metrics on 0e65e4d).
/// - OFF: the TASK-181 shape kept verbatim — serialize under the data lock
///   with a transient tps override. The byte-identity tests re-derive this
///   shape independently.
///
/// The output is byte-identical to `serde_json::to_string(&snapshot())`
/// (same derived Serialize, same field order, same tps contract) on BOTH
/// paths — asserted in-suite by `snapshot_json_cache_parity` and solo by
/// the C-entry test `telemetry_snapshot_inplace_byte_identical`.
///
/// TPS contract identical to [`snapshot`]: the serialized value carries
/// `ring_tps().unwrap_or(current_tps)`. On the OFF path the value is read
/// before the data lock and applied as a transient in-place override of
/// the stored field under the lock, restored right after serialization (no
/// lock holder can observe the override mid-call; the stored field has no
/// reader that does not override it). On the ON path the stored field is
/// never read or written at all — the TASK-198 tps memo resolves the SAME
/// value (bit-identical under the monotone-ingest epoch contract; any
/// anomaly falls back to a verbatim [`ring_tps`] rebuild — see the
/// [`TpsMemo`] contract) together with its serde-formatted bytes, so the
/// steady-state read pays NO ring scan and NO f64 format; the cached
/// bytes are patched into the cached ranges, and the cache fill itself
/// serializes with tps overridden to 0.0/1.0 (restored immediately), so
/// the stored value is never baked into the cache either.
///
/// Lock ordering (TASK-199 revision): the cache lock is the ONLY lock on
/// the hit path. The hit assembles the output exclusively from the cached
/// ranges (built atomically under the data lock at fill time) plus the
/// memo bytes — the live snapshot content is NEVER read on a hit, so the
/// data lock there served only to make the SNAP_GEN load race-free. That
/// load is now lock-free and LINEARIZED: a reader that loads the
/// pre-bump generation serves the coherent pre-mutation ranges (its read
/// happened before the mutation — linearized at the load); a reader that
/// sees the bumped generation takes the miss path, acquires the data
/// lock, re-verifies the generation under it (another reader may have
/// refilled while waiting) and rebuilds fresh from the locked content.
/// No torn output is possible by construction: the ranges swap atomically
/// under the cache lock, and a mixed old-prefix/new-suffix can only be
/// assembled from ranges that were never stored together. The data lock
/// is therefore taken ONLY on the miss/rebuild path (cache -> data
/// nesting unchanged and still the only nesting; mutators only bump
/// SNAP_GEN under the data lock and never touch the cache — no inverse
/// order can form). The caller may hold its own buffer mutex around this
/// with no inverse order anywhere. Single call site (the C entry);
/// #[inline(never)] keeps the serializer out of the caller's code region
/// (TASK-181).
#[inline(never)]
#[allow(dead_code)] // TASK-181 E2b bisect (caller parked in c_bridge patch)
pub(crate) fn snapshot_json_write(out: &mut Vec<u8>) {
    if !TELEM_SNAP_CACHE {
        // Pre-TASK-194 shape, kept verbatim for the A/B toggle.
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
        return;
    }
    // TASK-194: generation-gated cache; TASK-198: tps memo under the same
    // leaf lock; TASK-199: the hit path runs under the CACHE LOCK ONLY.
    // The struct destructure splits the field borrows (the borrow checker
    // cannot prove disjointness through `cache.field` alone inside one
    // fn).
    let mut cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
    let SnapshotJsonCache {
        gen: cached_gen,
        valid: cached_valid,
        prefix,
        suffix,
        tps_memo,
        scratch_a,
        scratch_b,
    } = &mut *cache;
    // Resolve the live tps + its serde bytes (steady state: zero ring
    // scan, zero format — two loads + compares). Byte-identical to the
    // verbatim expression by the TpsMemo value contract.
    if !tps_memo_resolve(tps_memo) {
        // Unreachable (f64 serialization is infallible): serve the legacy
        // `{}` fallback and invalidate both layers so the next call retries.
        tps_memo.valid = false;
        *cached_valid = false;
        out.clear();
        out.extend_from_slice(b"{}");
        return;
    }
    let tps_bytes_len = tps_memo.bytes_len;
    let tps_bytes = &tps_memo.bytes[..tps_bytes_len];
    // TASK-199 lock-free linearized generation load (see the locking
    // note above): a clean generation = serve the coherent cached
    // ranges WITHOUT touching the live snapshot; a bumped generation =
    // the miss path below.
    let gen = SNAP_GEN.load(Ordering::Relaxed);
    if *cached_valid && *cached_gen == gen {
        out.extend_from_slice(prefix);
        out.extend_from_slice(tps_bytes);
        out.extend_from_slice(suffix);
        return;
    }
    // Miss (first call, generation bump, or a stale-cache race): take
    // the data lock, RE-VERIFY the generation under it (another reader
    // may have refilled the cache while we waited — avoid a needless
    // reserialize), then rebuild the prefix/suffix pair from the locked
    // snapshot.
    let snap = snapshot_arc();
    let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
    let gen = SNAP_GEN.load(Ordering::Relaxed);
    if *cached_valid && *cached_gen == gen {
        out.extend_from_slice(prefix);
        out.extend_from_slice(tps_bytes);
        out.extend_from_slice(suffix);
        return;
    }
    let saved = s.tps;
    s.tps = 0.0;
    scratch_a.clear();
    let res_a = serde_json::to_writer(&mut *scratch_a, &*s);
    s.tps = 1.0;
    scratch_b.clear();
    let res_b = serde_json::to_writer(&mut *scratch_b, &*s);
    s.tps = saved;
    if res_a.is_err() || res_b.is_err() {
        // Unreachable for this Serialize impl, kept for parity with the
        // legacy `{}` fallback.
        *cached_valid = false;
        out.clear();
        out.extend_from_slice(b"{}");
        return;
    }
    match split_tps_span(scratch_a, scratch_b) {
        Some((p, span)) => {
            prefix.clear();
            prefix.extend_from_slice(&scratch_a[..p]);
            suffix.clear();
            suffix.extend_from_slice(&scratch_a[p + span..]);
            *cached_gen = gen;
            *cached_valid = true;
            // Serve this call from the just-built cache (same assemble
            // path as the hit branch; the tps bytes come from the memo).
            out.extend_from_slice(prefix);
            out.extend_from_slice(tps_bytes);
            out.extend_from_slice(suffix);
        }
        None => {
            // Unreachable: tps 0.0 vs 1.0 always differs at exactly one
            // byte of the tps value span. Serve the legacy fallback and
            // leave the cache invalid so the next call retries the fill.
            *cached_valid = false;
            out.clear();
            out.extend_from_slice(b"{}");
        }
    }
}

/// Split point between two serializations of the SAME snapshot whose only
/// difference is the tps value (0.0 vs 1.0 — same byte length, one digit):
/// returns the index of the first differing byte and the length of the tps
/// value span (3: `0.0`). The tps field appears exactly once in the top-
/// level field order, and the bodies are otherwise identical, so the first
/// diff is necessarily inside that span. None = misaligned bodies (never
/// happens for this Serialize impl; guarded so the caller can fall back
/// instead of panicking inside an extern "C" call chain).
fn split_tps_span(a: &[u8], b: &[u8]) -> Option<(usize, usize)> {
    if a.len() != b.len() {
        return None;
    }
    let p = a.iter().zip(b.iter()).position(|(x, y)| x != y)?;
    if p + 3 > a.len() {
        return None;
    }
    Some((p, 3))
}

/// Test-only: drop the shared snapshot so the next publish starts a fresh
/// list. The list itself has no removal API by design (append-only up to
/// the cap); full-state tests need a way back out of it. Never call from
/// a test that runs in parallel with metric-reading tests — filtered
/// single-test runs only.
#[cfg(test)]
pub(crate) fn test_reset_snapshot() {
    *SNAPSHOT.lock().unwrap_or_else(|p| p.into_inner()) = None;
    bump_snapshot_gen(); // TASK-194: the swap must invalidate the JSON cache
}

/// Test-only: zero the tick ring + the fallback tps bits (the same state
/// the in-suite `reset_state` builds). The tps memo inside the JSON cache
/// needs no explicit invalidation — its key (head / newest stored ts /
/// fallback bits) cannot survive a zeroed ring, so the next read rebuilds.
#[cfg(test)]
pub(crate) fn test_reset_ring() {
    RING_HEAD.store(0, Ordering::Relaxed);
    for slot in RING_TS.iter() {
        slot.store(0, Ordering::Relaxed);
    }
    for slot in RING_NS.iter() {
        slot.store(0, Ordering::Relaxed);
    }
    TPS_LAST_BITS.store(0, Ordering::Relaxed);
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

    /// TASK-194 A/B: snapshot JSON write with the 32-labeled fixture used
    /// by the c_bridge end-to-end bench (comparable numbers; the C entry
    /// adds only the SNAP_BUF lock + NUL push). BEFORE arm = the exact
    /// pre-TASK-194 expression (in-place to_writer, transient tps
    /// override); AFTER arm = the generation-gated cache (O(buffer)
    /// assemble on hit). Runs SOLO (resets the shared snapshot; leaves 32
    /// metrics published — harmless, every solo bench resets its own
    /// starting state).
    #[test]
    #[ignore]
    fn bench_snapshot_json_cache_ab() {
        test_reset_snapshot();

        // same fixture shape as bench_telemetry_snapshot_json
        for i in 0..32u32 {
            let mut labels = HashMap::new();
            labels.insert("region".to_string(), "eu".to_string());
            labels.insert("host".to_string(), format!("host-{i}"));
            labels.insert("module".to_string(), format!("mod-{i}"));
            labels.insert("idx".to_string(), i.to_string());
            publish_metric(
                &format!("c.bench.ab.{i}"),
                i as f64,
                Some("ms"),
                Some(labels),
            );
        }

        let iters = 200_000u32;
        let rounds = 5;
        let mut buf = Vec::with_capacity(4096);

        // BEFORE arm: pre-TASK-194 expression, verbatim. The fold-sum over
        // the output forces EVERY output byte to be observable (the C-entry
        // bench is immune to elision via its extern call boundary; this
        // inline expression needs the fold). The fold is symmetric in both
        // arms (~50 ns of L1-hot reads). NOTE: the writer must be &mut Vec
        // — `&mut *buf` on an owned Vec would deref to a fixed-size &mut
        // [u8] slice writer (WriteZero on the first byte after clear()).
        for _ in 0..10_000u32 {
            buf.clear();
            let tps = ring_tps().unwrap_or_else(current_tps);
            let snap = snapshot_arc();
            let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
            let saved = s.tps;
            s.tps = tps;
            let res = serde_json::to_writer(&mut buf, &*s);
            s.tps = saved;
            if res.is_err() {
                buf.clear();
                buf.extend_from_slice(b"{}");
            }
            std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_before = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                buf.clear();
                let tps = ring_tps().unwrap_or_else(current_tps);
                let snap = snapshot_arc();
                let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
                let saved = s.tps;
                s.tps = tps;
                let res = serde_json::to_writer(&mut buf, &*s);
                s.tps = saved;
                if res.is_err() {
                    buf.clear();
                    buf.extend_from_slice(b"{}");
                }
                std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_before =
                best_before.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // AFTER arm: generation-gated cache (first call fills, rest hit);
        // same symmetric observability fold as the BEFORE arm.
        let mut buf2 = Vec::with_capacity(4096);
        for _ in 0..10_000u32 {
            buf2.clear();
            snapshot_json_write(&mut buf2);
            std::hint::black_box(buf2.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_after = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                buf2.clear();
                snapshot_json_write(&mut buf2);
                std::hint::black_box(buf2.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_after =
                best_after.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH snapshot_json_cache A/B (32 labeled, direct write): before {:.0} ns/op, after {:.0} ns/op (min of {rounds}x{iters})",
            best_before * 1e9,
            best_after * 1e9
        );
    }

    /// TASK-198 A/B: the tps resolve + format on the snapshot READ path.
    ///
    /// CONTEXT (the round's finding): the landed snapshot_json lines
    /// (92 ns e2e / 124 ns direct) are an EMPTY-RING shape — the bench
    /// never pushes ticks, so `ring_tps()` returns None after one load.
    /// In production the per-tick path feeds the ring; `ring_tps()` then
    /// scans RING_CAP (4096) slots per read with Acquire loads — the
    /// TASK-194 cache win is silently masked behind a ~µs-scale scan.
    ///
    /// Arms (min of rounds x iters, solo):
    /// - empty_e2e:      snapshot_json_write with an empty ring (the
    ///                   landed-line shape, continuity).
    /// - scan_warm:      VERBATIM pre-198 per-read resolve+format
    ///                   (`ring_tps().unwrap_or_else(current_tps)` + serde
    ///                   f64 format into a reused buffer) on a warm ring
    ///                   (1400 fabricated monotone samples, 50 ms apart —
    ///                   steady 20 tps, ~1200 in the 60 s window). This is
    ///                   what every production panel read pays today.
    /// - warm_e2e:       snapshot_json_write per read on the warm ring.
    /// - tick_then_read: one push + one snapshot_json_write per iter — the
    ///                   amortized production shape (a tick lands between
    ///                   panel reads).
    ///
    /// The AFTER implementation (TASK-198 tps memo) must keep the resolved
    /// value bit-identical to `ring_tps().unwrap_or_else(current_tps)` —
    /// asserted in-suite by tps_memo parity tests, and the bench itself
    /// re-asserts it against the scan arm before timing (phase 2).
    #[test]
    #[ignore]
    fn bench_tps_memo_ab() {
        test_reset_snapshot();
        test_reset_ring();

        let iters = 200_000u32;
        let rounds = 5;
        let mut buf = Vec::with_capacity(4096);

        // ---- empty-ring e2e (landed-line shape, continuity) ----
        for _ in 0..10_000u32 {
            buf.clear();
            snapshot_json_write(&mut buf);
            std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_empty = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                buf.clear();
                snapshot_json_write(&mut buf);
                std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_empty = best_empty.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- warm ring: 1400 monotone fabricated samples, 50 ms apart ----
        // ts in process-relative ns (the TSC epoch form); 0 = empty sentinel
        // handled by push_tick_time_ts's +1 offset. 1400 x 50 ms = 70 s of
        // history; the 60 s window keeps the newest ~1200.
        for i in 0..1400u64 {
            push_tick_time_ts(1_000_000_000 + i * 50_000_000, 50_000_000);
        }

        // ---- scan_warm: verbatim pre-198 resolve+format ----
        let mut tps_buf = Vec::with_capacity(32);
        let mut scan_value = 0.0f64;
        for _ in 0..1_000u32 {
            tps_buf.clear();
            let tps = ring_tps().unwrap_or_else(current_tps);
            scan_value = tps;
            if serde_json::to_writer(&mut tps_buf, &tps).is_err() {
                tps_buf.clear();
                tps_buf.extend_from_slice(b"0");
            }
            std::hint::black_box(tps_buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_scan = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                tps_buf.clear();
                let tps = ring_tps().unwrap_or_else(current_tps);
                scan_value = tps;
                if serde_json::to_writer(&mut tps_buf, &tps).is_err() {
                    tps_buf.clear();
                    tps_buf.extend_from_slice(b"0");
                }
                std::hint::black_box(tps_buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_scan = best_scan.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- memo arm (TASK-198): cache lock + tps memo resolve ----
        // Parity canary first: the memo value must equal the scan arm's
        // value bit-for-bit on this fabricated monotone feed.
        {
            let mut cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
            assert!(
                tps_memo_resolve(&mut cache.tps_memo),
                "f64 format failed (unreachable)"
            );
            assert_eq!(
                cache.tps_memo.tps.to_bits(),
                scan_value.to_bits(),
                "memo value diverged from the verbatim scan"
            );
        }
        let mut memo_value = 0.0f64;
        for _ in 0..1_000u32 {
            let mut cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
            if tps_memo_resolve(&mut cache.tps_memo) {
                memo_value = cache.tps_memo.tps;
            }
            std::hint::black_box(memo_value.to_bits());
        }
        let mut best_memo = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                let mut cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
                if tps_memo_resolve(&mut cache.tps_memo) {
                    memo_value = cache.tps_memo.tps;
                }
                std::hint::black_box(memo_value.to_bits());
            }
            best_memo = best_memo.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- warm_e2e: snapshot_json_write on the warm ring ----
        for _ in 0..10_000u32 {
            buf.clear();
            snapshot_json_write(&mut buf);
            std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_warm = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                buf.clear();
                snapshot_json_write(&mut buf);
                std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_warm = best_warm.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- tick_then_read: push 1 + read 1 per iter ----
        let mut tick_ts = 1_000_000_000 + 1400 * 50_000_000;
        for _ in 0..10_000u32 {
            tick_ts += 50_000_000;
            push_tick_time_ts(tick_ts, 50_000_000);
            buf.clear();
            snapshot_json_write(&mut buf);
            std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_ttr = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                tick_ts += 50_000_000;
                push_tick_time_ts(tick_ts, 50_000_000);
                buf.clear();
                snapshot_json_write(&mut buf);
                std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_ttr = best_ttr.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // teardown: leave a clean state for whatever runs next (solo)
        test_reset_ring();

        println!(
            "BENCH tps_memo A/B: empty_e2e {:.0} ns/op, scan_warm {:.0} ns/op (tps {:.2}), memo_warm {:.0} ns/op, warm_e2e {:.0} ns/op, tick_then_read {:.0} ns/op (min of {rounds}x{iters})",
            best_empty * 1e9,
            best_scan * 1e9,
            scan_value,
            best_memo * 1e9,
            best_warm * 1e9,
            best_ttr * 1e9
        );
    }

    /// TASK-199 A/B: the snapshot cache-hit path with and without the
    /// snapshot data lock. Arm A = the verbatim pre-199 shape (cache lock
    /// + tps memo + DATA LOCK + generation load + assemble); arm B = the
    /// TASK-199 lock-free gen-gated hit (cache lock + memo + one linearized
    /// SNAP_GEN load + assemble — the data lock is gone from the hit
    /// path). Steady generation (hit regime), 32-labeled fixture, same
    /// symmetric fold observability as the cache A/B. Runs SOLO.
    #[test]
    #[ignore]
    fn bench_snapshot_hit_ab() {
        test_reset_snapshot();

        for i in 0..32u32 {
            let mut labels = HashMap::new();
            labels.insert("region".to_string(), "eu".to_string());
            labels.insert("host".to_string(), format!("host-{i}"));
            labels.insert("module".to_string(), format!("mod-{i}"));
            labels.insert("idx".to_string(), i.to_string());
            publish_metric(
                &format!("c.bench.hit.{i}"),
                i as f64,
                Some("ms"),
                Some(labels),
            );
        }

        let iters = 200_000u32;
        let rounds = 5;
        let mut buf = Vec::with_capacity(4096);

        // warm the cache + memo so both arms run in the hit regime
        for _ in 0..10_000u32 {
            buf.clear();
            snapshot_json_write(&mut buf);
            std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }

        // ---- arm A: pre-199 verbatim (data lock on the hit path) ----
        for _ in 0..1_000u32 {
            buf.clear();
            let mut cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
            let SnapshotJsonCache {
                gen: cached_gen,
                valid: cached_valid,
                prefix,
                suffix,
                tps_memo,
                scratch_a: _scratch_a,
                scratch_b: _scratch_b,
            } = &mut *cache;
            if !tps_memo_resolve(tps_memo) {
                *cached_valid = false;
            } else {
                let tps_bytes_len = tps_memo.bytes_len;
                let tps_bytes = &tps_memo.bytes[..tps_bytes_len];
                let snap = snapshot_arc();
                let s = snap.lock().unwrap_or_else(|p| p.into_inner());
                let gen = SNAP_GEN.load(Ordering::Relaxed);
                if *cached_valid && *cached_gen == gen {
                    buf.extend_from_slice(prefix);
                    buf.extend_from_slice(tps_bytes);
                    buf.extend_from_slice(suffix);
                } else {
                    drop(s);
                    drop(cache);
                    snapshot_json_write(&mut buf);
                }
            }
            std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
        }
        let mut best_locked = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                buf.clear();
                let mut cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
                let SnapshotJsonCache {
                    gen: cached_gen,
                    valid: cached_valid,
                    prefix,
                    suffix,
                    tps_memo,
                    scratch_a: _scratch_a2,
                    scratch_b: _scratch_b2,
                } = &mut *cache;
                if !tps_memo_resolve(tps_memo) {
                    *cached_valid = false;
                } else {
                    let tps_bytes_len = tps_memo.bytes_len;
                    let tps_bytes = &tps_memo.bytes[..tps_bytes_len];
                    let snap = snapshot_arc();
                    let s = snap.lock().unwrap_or_else(|p| p.into_inner());
                    let gen = SNAP_GEN.load(Ordering::Relaxed);
                    if *cached_valid && *cached_gen == gen {
                        buf.extend_from_slice(prefix);
                        buf.extend_from_slice(tps_bytes);
                        buf.extend_from_slice(suffix);
                    } else {
                        drop(s);
                        drop(cache);
                        snapshot_json_write(&mut buf);
                    }
                }
                std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_locked = best_locked.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        // ---- arm B: TASK-199 lock-free gen-gated hit ----
        let mut best_lockfree = f64::MAX;
        for _ in 0..rounds {
            let start = Instant::now();
            for _ in 0..iters {
                buf.clear();
                snapshot_json_write(&mut buf);
                std::hint::black_box(buf.iter().fold(0u8, |a, b| a.wrapping_add(*b)));
            }
            best_lockfree = best_lockfree.min(start.elapsed().as_secs_f64() / f64::from(iters));
        }

        println!(
            "BENCH snapshot_hit A/B (32 labeled, steady gen): locked {:.0} ns/op, lockfree {:.0} ns/op (min of {rounds}x{iters})",
            best_locked * 1e9,
            best_lockfree * 1e9
        );
    }
}

pub fn set_mem(used_mb: u64, max_mb: u64) {
    let snap = snapshot_arc();
    let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
    s.mem_used_mb = used_mb;
    s.mem_max_mb = max_mb;
    bump_snapshot_gen(); // TASK-194
}

pub fn set_uptime(secs: u64) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .uptime_secs = secs;
    bump_snapshot_gen(); // TASK-194
}

pub fn set_modules(names: Vec<String>) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .loaded_modules = names;
    bump_snapshot_gen(); // TASK-194
}

pub fn set_server_name(name: &str) {
    snapshot_arc()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .server_name = name.to_string();
    bump_snapshot_gen(); // TASK-194
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
    let epoch = *RING_EPOCH.get_or_init(Instant::now);
    let ts = at.saturating_duration_since(epoch).as_nanos() as u64;
    ring_store(ts, tick_ns);
}

/// TASK-195: precomputed-timestamp ingestion — the tick boundary's TSC
/// clock already yields process-relative nanoseconds, so the ring takes
/// the stamp directly instead of re-converting an Instant against
/// [`RING_EPOCH`] (one OnceLock load + one 128-bit multiply less per
/// tick). EPOCH CONTRACT: within a process the per-tick path uses exactly
/// ONE of [`push_tick_time_at`] (vDSO epoch) / this fn (TSC epoch) — ring
/// timestamps are difference-consumed only, so either alone is sound and
/// mixing is not (solo benches may cross forms; every tps-asserting test
/// resets the ring and uses a single form).
pub fn push_tick_time_ts(ts_ns: u64, tick_ns: u64) {
    ring_store(ts_ns, tick_ns);
}

fn ring_store(ts_ns: u64, tick_ns: u64) {
    let i = (RING_HEAD.fetch_add(1, Ordering::Relaxed) & (RING_CAP as u64 - 1)) as usize;
    RING_NS[i].store(tick_ns, Ordering::Relaxed);
    // Offset by +1 so 0 stays the "empty slot" sentinel.
    RING_TS[i].store(ts_ns.saturating_add(1), Ordering::Release);
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

// ---------------------------------------------------------------------------
// TASK-198: tps memo — incremental 60s-window accumulator + cached serde
// bytes for the snapshot read path. Value contract on [`TpsMemo`].
// ---------------------------------------------------------------------------

/// Resolve the live tps into `m` (value + serde-formatted bytes). Returns
/// false only on the unreachable f64-format failure (the caller serves the
/// legacy `{}` fallback and invalidates both cache layers).
///
/// Steady state: two relaxed/acquire loads + compares, zero ring scan,
/// zero formatting. Slow states: bounded incremental advance + evict, or
/// the verbatim rebuild (a clone of [`ring_tps`]'s scan — exact for
/// arbitrary ring content).
///
/// Call discipline: caller holds the SNAP_JSON_CACHE lock (the memo is a
/// field of the cache struct); the ring itself is lock-free.
fn tps_memo_resolve(m: &mut TpsMemo) -> bool {
    let head_now = RING_HEAD.load(Ordering::Relaxed);
    let newest_stored = if head_now > 0 {
        RING_TS[((head_now - 1) & (RING_CAP as u64 - 1)) as usize].load(Ordering::Acquire)
    } else {
        0
    };
    let ring_now = head_now > 0 && newest_stored != 0;
    let bits_now = TPS_LAST_BITS.load(Ordering::Relaxed);

    // Fast path 1: ring steady — no push since the last resolve, mode
    // match. (a_head == head_now implies the newest slot was consumed;
    // its stored value cannot change without a new push.)
    if m.valid && ring_now && m.ring_keyed && m.a_head == head_now {
        return true;
    }
    // Fast path 2: fallback steady (ring empty or newest in flight) —
    // keyed on the fallback bits.
    if m.valid && !ring_now && !m.ring_keyed && m.k_bits == bits_now {
        return true;
    }

    if !ring_now {
        // Fallback value, verbatim `unwrap_or_else(current_tps)` shape.
        // The accumulator state is untouched: nothing changed the ring
        // (a reset zeroes head/slots — caught below on the next ring
        // read), so a later resume stays sound.
        m.ring_keyed = false;
        m.k_bits = bits_now;
        m.tps = f64::from_bits(bits_now);
        m.valid = true;
        return tps_memo_format(m);
    }

    // Ring-derived value. Decide between the incremental resume and the
    // verbatim rebuild.
    let resume_ok = m.valid
        && m.a_evict <= m.a_head
        && m.a_head <= head_now
        && head_now - m.a_head <= RING_CAP as u64
        && head_now - m.a_evict <= RING_CAP as u64
        && newest_stored >= m.last_ts;

    if resume_ok {
        let cutoff = (newest_stored - 1).saturating_sub(TPS_WINDOW_SECS * 1_000_000_000);
        if tps_memo_advance(m, head_now, cutoff) && tps_memo_evict(m, cutoff) {
            if m.a_head == head_now {
                // Fully caught up: the monotonicity sentinel advances to
                // the newest consumed sample. On an in-flight stop the
                // sentinel stays at the last CONSUMED slot so the hole
                // slots cannot trip the dip check on the retry.
                m.last_ts = newest_stored;
            }
            m.ring_keyed = true;
            m.tps = tps_formula(m.a_sum, m.a_n);
            m.valid = true;
            return tps_memo_format(m);
        }
        // anomaly (torn / dip): fall through to the verbatim rebuild
    }

    tps_memo_rebuild(m, head_now, newest_stored)
}

/// Consume new pushes [m.a_head, head_now) into the accumulator: add every
/// written sample (window membership is enforced by the evict cursor —
/// under monotone ingest membership is a contiguous suffix of push order,
/// so a forward cursor reproduces the backward scan exactly). Stops at the
/// first in-flight slot (ts store not landed) — the effective head stays
/// there and a later resolve picks the sample up. Returns false on a torn
/// read or a ts dip (caller rebuilds).
fn tps_memo_advance(m: &mut TpsMemo, head_now: u64, cutoff: u64) -> bool {
    let mask = RING_CAP as u64 - 1;
    while m.a_head < head_now {
        let idx = (m.a_head & mask) as usize;
        let ts = RING_TS[idx].load(Ordering::Acquire);
        if ts == 0 {
            // In-flight push: consume up to here. The slot's ts is the
            // only in-flight marker (ring_store writes NS before TS).
            break;
        }
        let ns = RING_NS[idx].load(Ordering::Relaxed);
        if RING_TS[idx].load(Ordering::Acquire) != ts || ts < m.last_ts {
            // Torn (slot rewritten mid-read) or ts regression/dip
            // (clock-form switch / non-monotone producer).
            return false;
        }
        m.a_sum = m.a_sum.wrapping_add(ns);
        m.a_n += 1;
        m.a_head += 1;
        m.last_ts = ts;
        // Evict eagerly when the just-consumed sample is already outside
        // the window (batch spanning > 60s between two reads): the cursor
        // only moves forward, so membership stays a contiguous suffix.
        while m.a_evict < m.a_head {
            let e_idx = (m.a_evict & mask) as usize;
            let e_ts = RING_TS[e_idx].load(Ordering::Acquire);
            if e_ts == 0 || e_ts - 1 >= cutoff {
                break;
            }
            let e_ns = RING_NS[e_idx].load(Ordering::Relaxed);
            if RING_TS[e_idx].load(Ordering::Acquire) != e_ts {
                return false;
            }
            m.a_sum = m.a_sum.wrapping_sub(e_ns);
            m.a_n -= 1;
            m.a_evict += 1;
        }
    }
    true
}

/// Evict from the cursor while the oldest counted sample fell out of the
/// window (cutoff advanced via the newest sample). Returns false on a torn
/// read (caller rebuilds).
fn tps_memo_evict(m: &mut TpsMemo, cutoff: u64) -> bool {
    let mask = RING_CAP as u64 - 1;
    while m.a_evict < m.a_head {
        let idx = (m.a_evict & mask) as usize;
        let ts = RING_TS[idx].load(Ordering::Acquire);
        if ts == 0 || ts - 1 >= cutoff {
            break;
        }
        let ns = RING_NS[idx].load(Ordering::Relaxed);
        if RING_TS[idx].load(Ordering::Acquire) != ts {
            return false;
        }
        m.a_sum = m.a_sum.wrapping_sub(ns);
        m.a_n -= 1;
        m.a_evict += 1;
    }
    true
}

/// Verbatim rebuild: a byte-for-byte clone of [`ring_tps`]'s backward scan
/// (same guards, same window test) that additionally records the
/// accumulator state. Exact for arbitrary ring content. All issued pushes
/// are marked consumed (a_head = head_now): under the single-producer
/// epoch contract every issued push's stores are visible to the backward
/// Acquire walk (Release/Acquire propagation — a visible newer slot
/// implies all older stores of the same producer), so the scan's break
/// only fires on the never-issued tail or the documented multi-producer
/// in-flight race class.
fn tps_memo_rebuild(m: &mut TpsMemo, head_now: u64, newest_stored: u64) -> bool {
    let mask = RING_CAP as u64 - 1;
    let cutoff = (newest_stored - 1).saturating_sub(TPS_WINDOW_SECS * 1_000_000_000);
    let mut sum = 0u64;
    let mut n = 0u64;
    let mut oldest_kept = head_now;
    for k in 0..RING_CAP {
        let p = head_now.wrapping_sub(k as u64 + 1);
        let i = (p & mask) as usize;
        let ts = RING_TS[i].load(Ordering::Acquire);
        if ts == 0 {
            break; // never-written tail (partial ring) — nothing below is issued
        }
        if ts - 1 < cutoff {
            continue;
        }
        let ns = RING_NS[i].load(Ordering::Relaxed);
        if RING_TS[i].load(Ordering::Acquire) != ts {
            continue;
        }
        sum += ns;
        n += 1;
        oldest_kept = p;
    }
    if n == 0 {
        // The scan returns None here (defensive: the newest visible sample
        // always passes the window test unless torn-skipped):
        // fallback value, empty counted range.
        m.a_head = head_now;
        m.a_evict = head_now;
        m.a_sum = 0;
        m.a_n = 0;
        m.last_ts = newest_stored;
        m.ring_keyed = false;
        m.k_bits = TPS_LAST_BITS.load(Ordering::Relaxed);
        m.tps = f64::from_bits(m.k_bits);
    } else {
        m.a_head = head_now;
        m.a_evict = oldest_kept;
        m.a_sum = sum;
        m.a_n = n;
        m.last_ts = newest_stored;
        m.ring_keyed = true;
        m.tps = tps_formula(sum, n);
    }
    m.valid = true;
    tps_memo_format(m)
}

/// The ring_tps tail, shared verbatim: tps = 1000 / avg_ms over the
/// counted samples (0.0 for a non-positive average — unreachable for real
/// tick durations).
fn tps_formula(sum: u64, n: u64) -> f64 {
    let avg_ms = sum as f64 / n as f64 / 1_000_000.0;
    if avg_ms > 0.0 {
        1000.0 / avg_ms
    } else {
        0.0
    }
}

/// serde-format `m.tps` into the memo's fixed buffer (serde itself produces
/// the bytes — never hand-rolled). Returns false on the unreachable format
/// failure.
fn tps_memo_format(m: &mut TpsMemo) -> bool {
    let total = m.bytes.len();
    let mut rest: &mut [u8] = &mut m.bytes[..];
    let ok = serde_json::to_writer(&mut rest, &m.tps).is_ok();
    if ok {
        m.bytes_len = total - rest.len();
    } else {
        m.bytes_len = 0;
        m.valid = false;
    }
    ok
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
            bump_snapshot_gen(); // TASK-194: refresh thread mutates the snapshot
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
        bump_snapshot_gen(); // TASK-194: the swap must invalidate the JSON cache
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

    /// TASK-194 in-suite parity corpus: the cache path must emit
    /// byte-identical JSON to a fresh serde serialization through every
    /// state transition — first call (cache fill), repeat call (hit),
    /// publish_metric / set_mem / set_uptime / set_modules /
    /// set_server_name mutations (generation bumps → refill), a tps change
    /// WITHOUT a content bump (variable-length patch across the split),
    /// and a ring-fed tps. The legacy expression is re-derived inline
    /// (pre-TASK-194 shape) exactly like the solo C-entry test does.
    #[test]
    fn snapshot_json_cache_parity() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();

        // The exact pre-TASK-194 expression, re-derived independently.
        let legacy = |out: &mut Vec<u8>| {
            let tps = ring_tps().unwrap_or_else(current_tps);
            let snap = snapshot_arc();
            let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
            let saved = s.tps;
            s.tps = tps;
            let res = serde_json::to_writer(&mut *out, &*s);
            s.tps = saved;
            if res.is_err() {
                out.clear();
                out.extend_from_slice(b"{}");
            }
        };

        let mut legacy_buf = Vec::new();
        let mut cached_buf = Vec::new();

        // Concurrency guard for the parallel suite: c_bridge's pregate test
        // publishes metrics and scheduler tests feed the tick ring WITHOUT
        // telemetry's TEST_LOCK (each module has its own lock static), so a
        // concurrent mutation between the two captures would legitimately
        // change the bytes. A capture round is only asserted when BOTH the
        // content generation and the ring head are stable across it — any
        // concurrent publish or tick push voids the round and retries, so
        // the byte-identity assertion is deterministic, never flaky.
        fn capture_pair(
            legacy: &dyn Fn(&mut Vec<u8>),
            legacy_buf: &mut Vec<u8>,
            cached_buf: &mut Vec<u8>,
            what: &'static str,
        ) {
            let probe = || {
                (
                    SNAP_GEN.load(Ordering::Relaxed),
                    RING_HEAD.load(Ordering::Relaxed),
                )
            };
            for _ in 0..1000 {
                legacy_buf.clear();
                cached_buf.clear();
                let p0 = probe();
                legacy(legacy_buf);
                let p1 = probe();
                snapshot_json_write(cached_buf);
                let p2 = probe();
                if p0 == p1 && p1 == p2 {
                    assert_eq!(legacy_buf, cached_buf, "{what}");
                    return;
                }
                // concurrent mutation voided the round — retry
            }
            panic!("snapshot state never settled for {what}");
        }

        // empty state: fresh snapshot, cache fills on the first call.
        // (The byte-identity of fill and hit paths is asserted by every
        // capture_pair below: the first capture after any generation bump
        // exercises the refill, the immediately following one exercises
        // the steady hit.)
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "empty state (fill)");
        capture_pair(
            &legacy,
            &mut legacy_buf,
            &mut cached_buf,
            "empty state (steady hit)",
        );

        // mixed metrics: labeled, unlabeled, unit-only
        let mut labels = HashMap::new();
        labels.insert("region".to_string(), "eu".to_string());
        labels.insert("idx".to_string(), 7.to_string());
        publish_metric("cache.parity.labeled", 1.0, Some("ms"), Some(labels));
        publish_metric("cache.parity.plain", 2.5, None, None);
        publish_metric("cache.parity.unit", 3.0, Some("kb"), None);
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "mixed metrics (refill)");
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "mixed metrics (hit)");

        // setters mutate the base fields → generation bumps → refill
        set_mem(11, 22);
        set_uptime(33);
        set_modules(vec!["m1".to_string(), "m2".to_string()]);
        set_server_name("parity-host");
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "after setters (refill)");

        // tps change WITHOUT a content bump: same generation, different
        // tps byte length ("0.0" -> "19.98765") — exercises the
        // variable-length patch across the split point.
        set_tps(19.98765);
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "tps patch, same gen");

        // ring-fed tps: the window average drives the value in both arms.
        // The VALUE itself is order-dependent in a shared suite (earlier
        // tests leave fabricated samples inside the 60s window), so only
        // byte-identity is asserted, plus the field being a number.
        for _ in 0..50 {
            push_tick_time(50_000_000);
        }
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "ring-fed tps");
        legacy_buf.clear();
        snapshot_json_write(&mut legacy_buf);
        let v: serde_json::Value = serde_json::from_slice(&legacy_buf).unwrap();
        assert!(v["tps"].as_f64().is_some(), "ring-fed tps is a JSON number");

        // MAX_METRICS-cap publish drops (no bump — early return) must not
        // desync the cache: the output still matches a fresh serialize.
        let n = snapshot().metrics.len();
        for i in 0..(MAX_METRICS - n + 10) {
            publish_metric(&format!("cache.parity.cap.{i}"), i as f64, None, None);
        }
        capture_pair(&legacy, &mut legacy_buf, &mut cached_buf, "cap overflow (drop, no bump)");
        assert_eq!(snapshot().metrics.len(), MAX_METRICS);
    }

    /// TASK-198 in-suite: the tps memo serves the verbatim
    /// `ring_tps().unwrap_or_else(current_tps)` value through every state
    /// transition — bits-keyed fallback, incremental advance (single +
    /// batch), 61s eviction, ts dip (clock-form-switch guard -> verbatim
    /// rebuild), ring reset, burst beyond RING_CAP (lap guard -> rebuild),
    /// and the emptied-ring fallback. Full-JSON byte-identity against the
    /// legacy expression (capture with stability probes, bounded retry —
    /// the snapshot_json_cache_parity discipline) plus a bits assert
    /// against a fresh scan.
    #[test]
    fn tps_memo_parity_and_states() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        reset_state();

        let legacy = |buf: &mut Vec<u8>| {
            buf.clear();
            let tps = ring_tps().unwrap_or_else(current_tps);
            let snap = snapshot_arc();
            let mut s = snap.lock().unwrap_or_else(|p| p.into_inner());
            let saved = s.tps;
            s.tps = tps;
            let res = serde_json::to_writer(&mut *buf, &*s);
            s.tps = saved;
            if res.is_err() {
                buf.clear();
                buf.extend_from_slice(b"{}");
            }
        };
        fn probe() -> (u64, u64, u64) {
            (
                SNAP_GEN.load(Ordering::Relaxed),
                RING_HEAD.load(Ordering::Relaxed),
                TPS_LAST_BITS.load(Ordering::Relaxed),
            )
        }
        fn capture(
            legacy: &dyn Fn(&mut Vec<u8>),
            legacy_buf: &mut Vec<u8>,
            cached_buf: &mut Vec<u8>,
            what: &str,
        ) {
            for _ in 0..1000 {
                legacy_buf.clear();
                cached_buf.clear();
                let p0 = probe();
                legacy(legacy_buf);
                let p1 = probe();
                snapshot_json_write(cached_buf);
                let p2 = probe();
                if p0 == p1 && p1 == p2 {
                    assert_eq!(legacy_buf, cached_buf, "{what}");
                    // the memo's resolved value must equal a fresh scan;
                    // re-probe so the compare itself is inside a stable
                    // window
                    let p3 = probe();
                    if p2 != p3 {
                        continue;
                    }
                    let fresh = ring_tps().unwrap_or_else(current_tps);
                    let cache = SNAP_JSON_CACHE.lock().unwrap_or_else(|p| p.into_inner());
                    assert_eq!(
                        cache.tps_memo.tps.to_bits(),
                        fresh.to_bits(),
                        "memo bits diverged: {what}"
                    );
                    return;
                }
                // concurrent mutation voided the round — retry
            }
            panic!("snapshot state never settled for {what}");
        }

        let mut legacy_buf = Vec::new();
        let mut cached_buf = Vec::new();

        // 1) empty ring, set_tps fallback: two captures (fill + steady)
        set_tps(7.25);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "fallback fill");
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "fallback steady");

        // 2) monotone feed — the first ring capture resumes the
        //    accumulator from zero (add-on-consume, no eviction yet)
        for i in 1..=10u64 {
            push_tick_time_ts(1_000_000_000 + i * 50_000_000, 50_000_000);
        }
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "ring resume-from-zero");
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "ring steady fast path");

        // 3) incremental advance: one push, then a four-sample batch
        push_tick_time_ts(1_000_000_000 + 11 * 50_000_000, 50_000_000);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "advance x1");
        for i in 12..=15u64 {
            push_tick_time_ts(1_000_000_000 + i * 50_000_000, 50_000_000);
        }
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "advance batch");

        // 4) 61s silence then a tick: every older sample leaves the 60s
        //    window, only the new one stays (n=1)
        push_tick_time_ts(1_000_000_000 + 15 * 50_000_000 + 61_000_000_000, 50_000_000);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "61s eviction -> n=1");

        // 5) ts dip (the clock-form-switch guard): the new sample sits
        //    10s BELOW the previous newest — resume must refuse and the
        //    verbatim rebuild must reproduce the scan exactly
        push_tick_time_ts(1_000_000_000 + 15 * 50_000_000 + 51_000_000_000, 50_000_000);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "ts dip -> rebuild");

        // 6) ring reset mid-life (head regression): rebuild via resume
        //    refusal
        reset_state();
        push_tick_time_ts(5_000_000_000, 50_000_000);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "reset -> rebuild");

        // 7) burst beyond RING_CAP: 5000 x 1ms samples wrap the ring with
        //    the whole window still inside — the lap guard forces the
        //    rebuild regime; parity must hold per read
        reset_state();
        for i in 1..=5000u64 {
            push_tick_time_ts(2_000_000_000 + i * 1_000_000, 1_000_000);
        }
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "burst > RING_CAP rebuild");
        push_tick_time_ts(2_000_000_000 + 5001 * 1_000_000, 1_000_000);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "lap guard rebuild");

        // 8) emptied ring: fallback bits again, then set_tps interplay
        test_reset_ring();
        set_tps(19.5);
        capture(&legacy, &mut legacy_buf, &mut cached_buf, "emptied ring fallback");

        // teardown for the parallel suite
        test_reset_ring();
    }

    /// TASK-199 in-suite: the lock-free gen-gated hit must serve ONLY
    /// coherent output under concurrent mutation. A mutator thread bumps
    /// the content generation (set_mem — unbounded bumps, unlike the
    /// append-capped publish_metric) while the reader loops
    /// snapshot_json_write: every output must parse as JSON, carry a
    /// numeric tps and stay within the metrics cap. A torn
    /// old-prefix/new-suffix mix would break the parse — impossible by
    /// construction (ranges swap under the cache lock the reader holds);
    /// a stale-but-coherent pre-mutation snapshot is ALLOWED (the
    /// linearization point argument). Phase 1 = rebuild-heavy (hot
    /// mutator), phase 2 = hit-heavy raced (rare mutator), phase 3 =
    /// quiet steady hits.
    #[test]
    fn snapshot_json_lockfree_hit_coherence() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        reset_state();
        for i in 0..4u32 {
            publish_metric(&format!("race.{i}"), i as f64, None, None);
        }

        fn check(buf: &mut Vec<u8>, reads: usize, what: &str) {
            for _ in 0..reads {
                buf.clear();
                snapshot_json_write(buf);
                let v: serde_json::Value = serde_json::from_slice(buf)
                    .unwrap_or_else(|e| panic!("{what}: output is not valid JSON: {e}"));
                assert!(v["tps"].is_number(), "{what}: tps must stay a number");
                let arr = v["metrics"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{what}: metrics array missing"));
                assert!(arr.len() <= MAX_METRICS, "{what}: metrics cap exceeded");
            }
        }

        // phase 1: rebuild-heavy — a hot mutator forces the miss path
        // under the data lock while the reader reads
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop2 = stop.clone();
        let hot = thread::spawn(move || {
            let mut i = 0u64;
            while !stop2.load(Ordering::Relaxed) {
                set_mem(i & 0xFFF, 8192);
                i += 1;
            }
        });
        let mut buf = Vec::with_capacity(8192);
        check(&mut buf, 10_000, "rebuild-heavy");
        stop.store(true, Ordering::Relaxed);
        hot.join().unwrap();

        // phase 2: hit-heavy raced — the mutator bumps rarely, the reader
        // mostly takes the lock-free hit and occasionally rebuilds
        let stop3 = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop4 = stop3.clone();
        let rare = thread::spawn(move || {
            while !stop4.load(Ordering::Relaxed) {
                for _ in 0..256 {
                    if stop4.load(Ordering::Relaxed) {
                        break;
                    }
                    std::hint::black_box(1);
                }
                set_mem(4095, 8192);
            }
        });
        check(&mut buf, 10_000, "hit-heavy raced");
        stop3.store(true, Ordering::Relaxed);
        rare.join().unwrap();

        // phase 3: quiet steady hits (no mutator at all)
        check(&mut buf, 10_000, "quiet steady");

        reset_state();
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
    fn tsc_epoch_ring_window_math() {
        // TASK-195: the TSC-fed ring (push_tick_time_ts, caller-supplied
        // process-relative ns) must produce identical window semantics to
        // the Instant-fed path — same math on the same ring.
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        for i in 0..100u64 {
            push_tick_time_ts(1_000_000_000 + i * 50_000_000, 50_000_000);
        }
        let s = snapshot();
        assert!((s.tps - 20.0).abs() < 1e-6, "tps {} != 20", s.tps);

        reset_state();
        push_tick_time_ts(1_000_000_000, 50_000_000);
        push_tick_time_ts(1_000_000_000 + 61_000_000_000, 40_000_000);
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
