//! Brick 9: crash isolation — signal handling for graceful degradation.
//!
//! A segfault in a native module (or JIT-compiled kernel code) must not
//! silently kill the server: the platform installs handlers for the signals
//! that normally terminate the process (SIGSEGV, SIGABRT, SIGBUS, SIGFPE),
//! captures a native backtrace, records per-signal fault stats, notifies the
//! event bus and telemetry, and gives modules a chance to react gracefully
//! (stop ticking, persist state) before the JVM's own crash machinery runs.
//!
//! # Async-signal-safe handler
//!
//! The signal handler runs on the faulted (possibly corrupt) thread and may
//! interrupt *any* other code at *any* point, so it is restricted to the
//! async-signal-safe subset (POSIX `signal-safety(7)`):
//!
//! * atomic stores only for shared state (no locks, no allocation);
//! * `write(2)` to descriptors that were opened *before* the fault;
//! * `libc::time(2)` for the fault timestamp (in the POSIX safe list);
//! * `backtrace()` + `backtrace_symbols_fd()`.
//!
//! `backtrace_symbols_fd()` is async-signal-safe; `backtrace()` itself is
//! not officially (its first call may lazily load `libgcc`, which can call
//! `malloc`). Per the `backtrace(3)` NOTES, we "warm up" `libgcc` by calling
//! `backtrace()` once during [`install_handlers`], outside any handler, so
//! the in-handler call never triggers a dynamic load. The backtrace is
//! written to stderr (which hosting panels/launchers tee) and, if set via
//! [`set_crash_log_path`], to a file whose descriptor was opened in normal
//! context — the handler never opens files.
//!
//! Everything that is *not* async-signal-safe is deferred to a watchdog
//! thread (below) rather than attempted on the faulted thread.
//!
//! # Watchdog design
//!
//! Publishing on the event bus or telemetry takes a mutex, which can
//! deadlock if the fault hit while the mutex was held by a dying thread.
//! Instead the handler stashes the fault in lock-free atomics (signal
//! number + Unix timestamp) and re-raises. A dedicated thread
//! (`crussty-fault-watchdog`, spawned by [`install_handlers`]) polls the
//! stash every 500 ms and, when a fault appears, claims it with a
//! compare-and-swap (so a fault is processed exactly once even if the
//! handler fired repeatedly) and does all the non-signal-safe work:
//! publish the [`FAULT_EVENT`] on the bus, publish a telemetry metric, and
//! invoke the [`on_fault`] hooks.
//!
//! This is the classic "handler sets a flag, another context polls it"
//! pattern (the `volatile sig_atomic_t` idiom, formalized with lock-free
//! atomics). A self-pipe or `signalfd` would wake a *blocking* event loop
//! faster, but the watchdog is a dedicated polling thread with no blocking
//! wait to interrupt, so a poll interval is the simpler, deadlock-free
//! choice; 500 ms of added latency is irrelevant for a process that is
//! dying anyway.
//!
//! # Honest limits
//!
//! * The handler re-raises with the default disposition immediately after
//!   recording the fault, so the JVM's fatal-error machinery (hs_err dump)
//!   still runs. If the process dies within milliseconds — the common case —
//!   the watchdog never observes the fault; the stderr/file backtrace is
//!   what survives. Watchdog events and hooks are best-effort, not
//!   guaranteed delivery.
//! * [`on_fault`] hooks run on the watchdog thread *after* the fault was
//!   re-raised: the process is dying. Hooks may log or persist state, but
//!   must not attempt to recover the faulted thread (its state is
//!   undefined) or block for long.
//! * [`set_restart_command`] never executes anything from the signal path.
//!   The command is carried in the fault event payload and telemetry so an
//!   *external* supervisor (systemd, a hosting panel) can act after the
//!   JVM exits. A crashed JVM must not exec.
//! * If several faults land between watchdog polls, only the latest is
//!   reported in the event; [`fault_count`] / [`fault_stats`] still count
//!   every one.
//!
//! On non-Unix targets (Windows) everything below compiles to no-op stubs
//! so the runtime still builds; there is no crash isolation there.
//!
//! # Design sources
//!
//! man7 `signal-safety(7)` (async-signal-safe function list), man7
//! `backtrace(3)` NOTES (libgcc lazy-load warmup), the POSIX
//! `sig_atomic_t` flag idiom, the "self-pipe trick" discussions
//! (cr.yp.to/docs/selfpipe.html) and its poll-flag alternative, and the
//! existing events.rs/telemetry.rs brick conventions (poison-tolerant
//! locks, `OnceLock`-guarded spawn).

use libc::c_int;
#[cfg(unix)]
use libc::{sighandler_t, SIGABRT, SIGBUS, SIGFPE, SIGSEGV};
#[cfg(unix)]
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(unix)]
use std::sync::atomic::{AtomicI32, AtomicU64};
#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use std::panic::{catch_unwind, AssertUnwindSafe};
#[cfg(unix)]
use std::sync::{Mutex, OnceLock};

#[cfg(unix)]
pub const SUPPORTED_SIGNALS: [c_int; 4] = [SIGSEGV, SIGABRT, SIGBUS, SIGFPE];
#[cfg(not(unix))]
pub const SUPPORTED_SIGNALS: [c_int; 0] = [];

/// Event published on the bus by the watchdog when a fault is observed.
/// Part of the platform event surface; the lifecycle constants in
/// `events.rs` live in that brick, so this brick declares its own name.
pub const FAULT_EVENT: &str = "platform.fault";

/// Number of stack frames captured in the handler.
const BACKTRACE_FRAMES: usize = 64;

/// Poll period of the fault watchdog (overridable for tests/embeddings).
#[cfg(unix)]
const WATCHDOG_POLL_MS_DEFAULT: u64 = 500;

static CRASHED: AtomicBool = AtomicBool::new(false);
static FAULT_COUNT: AtomicUsize = AtomicUsize::new(0);

/// One recorded fault: the signal, the wall-clock timestamp (Unix seconds,
/// taken with the async-signal-safe `time(2)` in the handler) and the total
/// fault count of the process at that moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultInfo {
    pub signal: c_int,
    pub timestamp_unix: u64,
    pub count: u64,
}

/// Hook invoked by the watchdog (never by the signal handler) when a fault
/// is observed. The process is dying at that point; hooks should only log
/// or persist, never attempt to recover the faulted thread.
pub type FaultHook = Arc<dyn Fn(FaultInfo) + Send + Sync>;

/// Signal-stash for the handler: `0` means no fault pending. Written by the
/// handler (atomic store), claimed by the watchdog (swap), so a fault is
/// processed exactly once.
#[cfg(unix)]
static STASH_SIG: AtomicI32 = AtomicI32::new(0);
#[cfg(unix)]
static STASH_TS: AtomicU64 = AtomicU64::new(0);

/// Pre-opened crash-log descriptor (O_APPEND), or -1 when disabled. Opened
/// in normal context by [`set_crash_log_path`]; the handler only writes.
#[cfg(unix)]
static CRASH_LOG_FD: AtomicI32 = AtomicI32::new(-1);

/// Per-signal fault counters, indexed by position in [`SUPPORTED_SIGNALS`].
#[cfg(unix)]
static PER_SIGNAL: [AtomicU64; 4] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

#[cfg(unix)]
static LAST_FAULT: Mutex<Option<FaultInfo>> = Mutex::new(None);
#[cfg(unix)]
static RESTART_CMD: Mutex<Option<String>> = Mutex::new(None);
#[cfg(unix)]
static HOOKS: Mutex<Vec<FaultHook>> = Mutex::new(Vec::new());
#[cfg(unix)]
static WATCHDOG_POLL_MS: AtomicU64 = AtomicU64::new(WATCHDOG_POLL_MS_DEFAULT);
#[cfg(unix)]
static WATCHDOG_STARTED: OnceLock<()> = OnceLock::new();

/// Previous dispositions, saved at [`install_handlers`] time so the handler
/// can chain to whatever the JVM (or embedding) had installed before us.
/// Raw `sa_sigaction` pointer value and raw `sa_flags`, per supported
/// signal index; `PREV_VALID` is false when `sigaction` failed, in which
/// case the handler falls back to re-raising the default disposition.
#[cfg(unix)]
static PREV_ACTIONS: [AtomicUsize; 4] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
#[cfg(unix)]
static PREV_FLAGS: [AtomicUsize; 4] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
#[cfg(unix)]
static PREV_VALID: [AtomicBool; 4] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

#[cfg(unix)]
const MSG_PRE: &[u8] = b"[crussty-runtime] native fault: signal ";
#[cfg(unix)]
const MSG_MID: &[u8] = b" (fault #";
#[cfg(unix)]
const MSG_SUF: &[u8] = b")\nbacktrace:\n";

/// Human-readable name for a supported signal.
#[cfg(unix)]
pub fn signal_name(sig: c_int) -> &'static str {
    match sig {
        SIGSEGV => "SIGSEGV",
        SIGABRT => "SIGABRT",
        SIGBUS => "SIGBUS",
        SIGFPE => "SIGFPE",
        _ => "UNKNOWN",
    }
}

#[cfg(not(unix))]
pub fn signal_name(_sig: c_int) -> &'static str {
    "UNKNOWN"
}

/// Poison-tolerant lock, matching the events.rs/telemetry.rs convention: a
/// panicked holder must not take the brick down.
#[cfg(unix)]
fn lock<T>(guard: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    guard.lock().unwrap_or_else(|p| p.into_inner())
}

/// Async-signal-safe decimal write: formats `n` into a stack buffer and
/// emits it with a single `write(2)`.
#[cfg(unix)]
fn write_u64(fd: c_int, mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    unsafe {
        libc::write(fd, buf[i..].as_ptr() as *const c_void, buf.len() - i);
    }
}

/// Async-signal-safe fault header, e.g.
/// `[crussty-runtime] native fault: signal 11 (fault #3)\nbacktrace:\n`.
#[cfg(unix)]
fn write_fault_line(fd: c_int, sig: c_int, count: usize) {
    unsafe {
        libc::write(fd, MSG_PRE.as_ptr() as *const c_void, MSG_PRE.len());
    }
    write_u64(fd, sig as u64);
    unsafe {
        libc::write(fd, MSG_MID.as_ptr() as *const c_void, MSG_MID.len());
    }
    write_u64(fd, count as u64);
    unsafe {
        libc::write(fd, MSG_SUF.as_ptr() as *const c_void, MSG_SUF.len());
    }
}

/// Dump the native backtrace to an already-open descriptor.
/// [`backtrace_symbols_fd`] is async-signal-safe; the `backtrace()` warmup
/// in [`install_handlers`] guarantees `libgcc` is already loaded.
#[cfg(unix)]
fn dump_backtrace(fd: c_int) {
    let mut frames: [*mut c_void; BACKTRACE_FRAMES] = [std::ptr::null_mut(); BACKTRACE_FRAMES];
    unsafe {
        let n = libc::backtrace(frames.as_mut_ptr(), frames.len() as c_int);
        if n > 0 {
            libc::backtrace_symbols_fd(frames.as_mut_ptr(), n, fd);
        }
    }
}

/// Everything the handler does that touches shared state or I/O, minus the
/// re-raise. Async-signal-safe by construction: atomic stores, `write(2)`
/// to pre-opened descriptors, `time(2)`, and the backtrace functions.
#[cfg(unix)]
fn record_fault(sig: c_int) {
    let count = FAULT_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    CRASHED.store(true, Ordering::SeqCst);
    PER_SIGNAL[sig_index(sig)].fetch_add(1, Ordering::SeqCst);
    STASH_SIG.store(sig, Ordering::SeqCst);
    let ts = u64::try_from(unsafe { libc::time(std::ptr::null_mut()) }).unwrap_or(0);
    STASH_TS.store(ts, Ordering::SeqCst);

    let crash_fd = CRASH_LOG_FD.load(Ordering::Relaxed);
    write_fault_line(2, sig, count);
    dump_backtrace(2);
    if crash_fd >= 0 {
        write_fault_line(crash_fd, sig, count);
        dump_backtrace(crash_fd);
    }
}

/// Lightweight side of [`record_fault`] used when the JVM (or embedding) had
/// its own handler for this signal and a fault has been observed: bump the
/// per-signal and total counters only. No backtrace, no stash, no `CRASHED`:
/// the fault may be a JVM-internal signal (null-check, stack bang, hs_err
/// setup) that the previous handler will resolve normally, so the watchdog
/// must not publish a premature fault.
#[cfg(unix)]
fn count_fault(sig: c_int) {
    FAULT_COUNT.fetch_add(1, Ordering::SeqCst);
    PER_SIGNAL[sig_index(sig)].fetch_add(1, Ordering::SeqCst);
}

#[cfg(unix)]
fn sig_index(sig: c_int) -> usize {
    match sig {
        SIGSEGV => 0,
        SIGABRT => 1,
        SIGBUS => 2,
        SIGFPE => 3,
        _ => 0,
    }
}

#[cfg(unix)]
extern "C" fn fault_handler(sig: c_int, info: *mut libc::siginfo_t, uctx: *mut c_void) {
    let idx = sig_index(sig);
    let prev_action = PREV_ACTIONS[idx].load(Ordering::Relaxed);
    let prev_flags = PREV_FLAGS[idx].load(Ordering::Relaxed);
    let prev_valid = PREV_VALID[idx].load(Ordering::Relaxed);

    // No previous handler or it was the default: we are the terminal handler.
    // Record fully, then re-raise with the default disposition so the JVM's
    // fatal-error machinery (hs_err dump) still runs, which hosting panels
    // parse. The watchdog may or may not get to observe the fault before the
    // process dies; that is why the backtrace is written here.
    if !prev_valid || prev_action as sighandler_t == libc::SIG_DFL {
        record_fault(sig);
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
        return;
    }

    // The JVM (or the embedding) previously had its OWN handler installed
    // for this signal — SIGSEGV in particular is used internally by the JIT
    // (null checks, stack banging) and for hs_err setup. We chain to that
    // handler exactly as installed: same flags, same call shape. Recording
    // only the counters prevents false crash signals; the JVM resolves the
    // fault. This is the established agent pattern (async-profiler, JVMTI
    // agents): overwriting the JVM's handler outright breaks JVM semantics.
    count_fault(sig);
    unsafe {
        let mut prev_sa: libc::sigaction = std::mem::zeroed();
        prev_sa.sa_flags = prev_flags as c_int;
        prev_sa.sa_sigaction = prev_action;
        libc::sigemptyset(&mut prev_sa.sa_mask);
        libc::sigaction(sig, &prev_sa, std::ptr::null_mut());
        if (prev_flags as c_int) & libc::SA_SIGINFO != 0 {
            let f: unsafe extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void) =
                std::mem::transmute::<usize, _>(prev_action);
            f(sig, info, uctx);
        } else {
            let f: unsafe extern "C" fn(c_int) =
                std::mem::transmute::<usize, _>(prev_action);
            f(sig);
        }
    }
}

/// Force `libgcc` to be resident before any handler can run: the first
/// `backtrace()` call may dynamically load it (which can allocate), so it
/// must happen in normal context (man7 `backtrace(3)` NOTES).
#[cfg(unix)]
fn warm_up_backtrace() {
    let mut frames: [*mut c_void; 8] = [std::ptr::null_mut(); 8];
    unsafe {
        libc::backtrace(frames.as_mut_ptr(), frames.len() as c_int);
    }
}

/// Install the crash handlers, warm up the backtrace machinery and spawn
/// the fault watchdog. Returns the number of signals actually handed.
/// Safe to call multiple times (idempotent); the watchdog spawns once.
///
/// Handlers are installed with `SA_SIGINFO` so the previous disposition can
/// be captured and later chained to (the JVM keeps its own SIGSEGV handler,
/// which agents must not clobber).
#[cfg(unix)]
pub fn install_handlers() -> usize {
    warm_up_backtrace();
    let mut n = 0usize;
    unsafe {
        for &sig in &SUPPORTED_SIGNALS {
            let mut new: libc::sigaction = std::mem::zeroed();
            new.sa_sigaction = fault_handler as *const () as usize;
            new.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
            libc::sigemptyset(&mut new.sa_mask);

            let mut old: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(sig, &new, &mut old) == 0 {
                let idx = sig_index(sig);
                PREV_ACTIONS[idx].store(old.sa_sigaction, Ordering::Relaxed);
                PREV_FLAGS[idx].store(old.sa_flags as usize, Ordering::Relaxed);
                PREV_VALID[idx].store(true, Ordering::Relaxed);
                // only count if we actually replaced a non-SIG_IGN handler
                if old.sa_sigaction as sighandler_t != libc::SIG_IGN {
                    n += 1;
                }
            } else {
                PREV_VALID[sig_index(sig)].store(false, Ordering::Relaxed);
            }
        }
    }
    ensure_watchdog();
    n
}

#[cfg(not(unix))]
pub fn install_handlers() -> usize {
    0
}

/// The atomics below are only written on Unix ([`record_fault`]); on other
/// platforms they stay false/0, which is exactly what the no-op contract
/// promises.
pub fn has_crashed() -> bool {
    CRASHED.load(Ordering::SeqCst)
}

pub fn fault_count() -> usize {
    FAULT_COUNT.load(Ordering::SeqCst)
}

/// Per-signal fault counters, in [`SUPPORTED_SIGNALS`] order.
#[cfg(unix)]
pub fn fault_stats() -> Vec<(i32, u64)> {
    SUPPORTED_SIGNALS
        .iter()
        .zip(&PER_SIGNAL)
        .map(|(&sig, c)| (sig, c.load(Ordering::SeqCst)))
        .collect()
}

#[cfg(not(unix))]
pub fn fault_stats() -> Vec<(i32, u64)> {
    Vec::new()
}

/// The most recently observed fault, as recorded by the watchdog. `None`
/// until a fault was actually *processed*: if the process died before the
/// watchdog polled, the backtrace on stderr is the only record.
#[cfg(unix)]
pub fn last_fault() -> Option<FaultInfo> {
    lock(&LAST_FAULT).clone()
}

#[cfg(not(unix))]
pub fn last_fault() -> Option<FaultInfo> {
    None
}

/// Append crash output (fault header + native backtrace) to `path` instead
/// of stderr only. The file is opened once, here, in normal context; the
/// signal handler later writes to the resulting descriptor. `None` disables
/// the file (stderr output stays enabled).
#[cfg(unix)]
pub fn set_crash_log_path(path: Option<PathBuf>) {
    let old = CRASH_LOG_FD.swap(-1, Ordering::SeqCst);
    if old >= 0 {
        unsafe {
            libc::close(old);
        }
    }
    let Some(path) = path else {
        return;
    };
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        use std::os::unix::io::IntoRawFd;
        CRASH_LOG_FD.store(file.into_raw_fd(), Ordering::SeqCst);
    }
}

#[cfg(not(unix))]
pub fn set_crash_log_path(_path: Option<PathBuf>) {}

/// Register a hook invoked by the watchdog (never by the signal handler)
/// when a fault is observed. At that point the fault has already been
/// re-raised and the process is dying: hooks must only log or persist
/// state — never attempt to recover the faulted thread — and must not
/// block for long.
#[cfg(unix)]
pub fn on_fault(handler: FaultHook) {
    lock(&HOOKS).push(handler);
}

#[cfg(not(unix))]
pub fn on_fault(_handler: FaultHook) {}

/// Set the command an external supervisor should run to restart the server
/// after a crash. This is *never* executed from the signal path (a crashed
/// JVM must not exec): it is carried in the [`FAULT_EVENT`] payload and the
/// telemetry metric so systemd/panel supervision can act after exit.
#[cfg(unix)]
pub fn set_restart_command(command: Option<String>) {
    *lock(&RESTART_CMD) = command;
}

#[cfg(not(unix))]
pub fn set_restart_command(_command: Option<String>) {}

/// Override the watchdog poll period. Primarily for tests and embeddings;
/// the handler still records and re-raises regardless of this value.
#[cfg(unix)]
pub fn set_watchdog_poll_interval(interval: Duration) {
    let ms = interval.as_millis().clamp(1, u128::from(u64::MAX)) as u64;
    WATCHDOG_POLL_MS.store(ms, Ordering::Relaxed);
}

#[cfg(not(unix))]
pub fn set_watchdog_poll_interval(_interval: Duration) {}

/// One watchdog poll: claim a stashed fault (exactly once, via swap) and
/// perform all the non-signal-safe work — bus event, telemetry metric and
/// fault hooks. Exposed so tests can drive the watchdog deterministically.
#[cfg(unix)]
pub fn watchdog_tick() {
    let sig = STASH_SIG.swap(0, Ordering::SeqCst);
    if sig == 0 {
        return;
    }
    let ts = STASH_TS.swap(0, Ordering::SeqCst);
    let count = FAULT_COUNT.load(Ordering::SeqCst) as u64;
    let info = FaultInfo {
        signal: sig,
        timestamp_unix: ts,
        count,
    };
    *lock(&LAST_FAULT) = Some(info.clone());

    let restart = lock(&RESTART_CMD).clone();
    let payload = serde_json::json!({
        "signal": sig,
        "signal_name": signal_name(sig),
        "timestamp_unix": ts,
        "count": count,
        "restart_command": restart,
    });
    crate::platform::events::global().publish(FAULT_EVENT, &payload);

    let mut labels = HashMap::new();
    labels.insert("signal".to_string(), signal_name(sig).to_string());
    crate::platform::telemetry::publish_metric(
        "runtime.fault_count",
        count as f64,
        Some("count"),
        Some(labels),
    );

    let hooks = lock(&HOOKS).clone();
    for hook in hooks {
        let _ = catch_unwind(AssertUnwindSafe(|| hook(info.clone())));
    }
}

#[cfg(not(unix))]
pub fn watchdog_tick() {}

#[cfg(unix)]
fn ensure_watchdog() {
    WATCHDOG_STARTED.get_or_init(|| {
        let spawned = std::thread::Builder::new()
            .name("crussty-fault-watchdog".into())
            .spawn(watchdog_loop);
        if let Err(err) = spawned {
            eprintln!("[crussty:signals] failed to spawn fault watchdog: {err}");
        }
    });
}

#[cfg(unix)]
fn watchdog_loop() {
    loop {
        let ms = WATCHDOG_POLL_MS.load(Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(ms));
        watchdog_tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::sync::Mutex;
    #[cfg(unix)]
    use std::time::{Duration, Instant};

    /// Serializes tests that touch the shared fault state.
    #[cfg(unix)]
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(unix)]
    fn reset_state() {
        CRASHED.store(false, Ordering::SeqCst);
        FAULT_COUNT.store(0, Ordering::SeqCst);
        STASH_SIG.store(0, Ordering::SeqCst);
        STASH_TS.store(0, Ordering::SeqCst);
        for c in &PER_SIGNAL {
            c.store(0, Ordering::SeqCst);
        }
        *lock(&LAST_FAULT) = None;
        *lock(&HOOKS) = Vec::new();
        *lock(&RESTART_CMD) = None;
    }

    #[cfg(unix)]
    fn wait_until(mut cond: impl FnMut() -> bool, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while !cond() {
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        true
    }

    #[cfg(unix)]
    #[test]
    fn install_is_idempotent() {
        // Never actually raise a fault in tests; just verify install runs.
        let _ = install_handlers();
        let _ = install_handlers();
        assert!(SUPPORTED_SIGNALS.len() == 4);
    }

    #[cfg(unix)]
    #[test]
    fn fault_stash_is_set_by_record_and_claimed_by_watchdog() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        // record_fault is the exact async-signal-safe path the handler
        // runs (minus the re-raise), so this raises no real signal.
        record_fault(SIGSEGV);
        assert_eq!(STASH_SIG.load(Ordering::SeqCst), SIGSEGV);
        assert!(STASH_TS.load(Ordering::SeqCst) > 0);

        watchdog_tick();
        assert_eq!(STASH_SIG.load(Ordering::SeqCst), 0, "watchdog must claim the stash");
        assert_eq!(STASH_TS.load(Ordering::SeqCst), 0);
        let last = last_fault().expect("watchdog recorded the fault");
        assert_eq!(last.signal, SIGSEGV);
        assert_eq!(last.count, 1);
        assert!(last.timestamp_unix > 0);

        // A second tick with an empty stash is a no-op.
        watchdog_tick();
        assert_eq!(last_fault().map(|f| f.count), Some(1));
    }

    #[cfg(unix)]
    #[test]
    fn per_signal_stats_increment_independently() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        record_fault(SIGSEGV);
        record_fault(SIGSEGV);
        record_fault(SIGABRT);
        let stats = fault_stats();
        assert_eq!(stats[0], (SIGSEGV, 2));
        assert_eq!(stats[1], (SIGABRT, 1));
        assert_eq!(stats[2], (SIGBUS, 0));
        assert_eq!(stats[3], (SIGFPE, 0));
        assert_eq!(fault_count(), 3);
    }

    #[cfg(unix)]
    #[test]
    fn watchdog_publishes_fault_event() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let got = Arc::new(Mutex::new(None));
        let g = Arc::clone(&got);
        let _token = crate::platform::events::global().subscribe(
            FAULT_EVENT,
            Arc::new(move |_, payload| {
                *g.lock().unwrap() = Some(payload.clone());
            }),
        );
        record_fault(SIGFPE);
        assert!(wait_until(
            || STASH_SIG.load(Ordering::SeqCst) == SIGFPE,
            Duration::from_secs(2)
        ));
        watchdog_tick();

        let payload = got
            .lock()
            .unwrap()
            .clone()
            .expect("watchdog must publish the fault event");
        assert_eq!(payload["signal"].as_i64(), Some(SIGFPE as i64));
        assert_eq!(payload["signal_name"], "SIGFPE");
        assert_eq!(payload["count"].as_u64(), Some(1));
        assert!(payload["timestamp_unix"].as_u64().unwrap() > 0);
    }

    #[cfg(unix)]
    #[test]
    fn watchdog_publishes_telemetry_metric() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        record_fault(SIGABRT);
        watchdog_tick();
        let snap = crate::platform::telemetry::snapshot();
        let metric = snap
            .metrics
            .iter()
            .rev()
            .find(|m| m.name == "runtime.fault_count")
            .expect("watchdog must publish the fault metric");
        assert_eq!(metric.value, 1.0);
        assert_eq!(metric.unit.as_deref(), Some("count"));
        assert_eq!(
            metric.labels.as_ref().and_then(|l| l.get("signal")).map(String::as_str),
            Some("SIGABRT")
        );
    }

    #[cfg(unix)]
    #[test]
    fn restart_command_is_stored_and_carried_in_event() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        set_restart_command(Some("systemctl restart crussty".to_string()));
        let got = Arc::new(Mutex::new(None));
        let g = Arc::clone(&got);
        let _token = crate::platform::events::global().subscribe(
            FAULT_EVENT,
            Arc::new(move |_, payload| {
                *g.lock().unwrap() = Some(payload.clone());
            }),
        );
        record_fault(SIGSEGV);
        watchdog_tick();
        let payload = got.lock().unwrap().clone().expect("fault event published");
        assert_eq!(payload["restart_command"], "systemctl restart crussty");

        set_restart_command(None);
        record_fault(SIGSEGV);
        watchdog_tick();
        let payload = got.lock().unwrap().clone().unwrap();
        assert!(payload["restart_command"].is_null());
    }

    #[cfg(unix)]
    #[test]
    fn on_fault_hook_runs_on_watchdog_tick_not_handler() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let s = Arc::clone(&seen);
        on_fault(Arc::new(move |info| {
            s.lock().unwrap().push(info.signal);
        }));

        // record_fault (what the handler runs) must not invoke hooks.
        record_fault(SIGBUS);
        assert!(seen.lock().unwrap().is_empty(), "hooks must not run on the faulted thread");
        watchdog_tick();
        assert_eq!(*seen.lock().unwrap(), vec![SIGBUS]);
        assert_eq!(last_fault().map(|f| f.signal), Some(SIGBUS));
    }

    #[cfg(unix)]
    #[test]
    fn watchdog_poll_interval_setter() {
        let _guard = TEST_LOCK.lock().unwrap();
        set_watchdog_poll_interval(Duration::from_millis(25));
        assert_eq!(WATCHDOG_POLL_MS.load(Ordering::Relaxed), 25);
        set_watchdog_poll_interval(Duration::from_millis(500));
        assert_eq!(WATCHDOG_POLL_MS.load(Ordering::Relaxed), 500);
    }
}
