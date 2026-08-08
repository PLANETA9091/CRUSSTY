//! Brick 10: hot reload — swap a module's .so/.dll without restarting.
//!
//! Modules load via libloading (dlopen). This brick provides a module
//! registry whose entries own their `Library` (dlclose on drop), plus
//! `reload_module` which dlopens the path fresh, runs the module's init
//! handshake, and only then swaps the registry entry and dlcloses the old
//! library. If the replacement fails to load *or* to init, the old library
//! stays active (crash resilience).
//!
//! # Unload safety (dlclose pitfalls)
//!
//! `dlclose` is a request, not a command: the mapping is released only when
//! the reference count drops to zero, and it is a hard crash to unmap a
//! library while a thread is still executing its code (or its TLS
//! destructors later run against unmapped pages — see glibc libc-alpha
//! "dlclose crash during C++ thread_local destructor", 2025). Rust modules
//! that use `thread_local` with `Drop` register `__cxa_thread_atexit`
//! destructors which keep the mapping resident until every thread exits
//! (rust-lang/rust#59629); dlclose then may not actually unload, and a
//! subsequent dlopen of the same path can return the same handle with
//! non-reinitialized statics. Module authors must therefore avoid TLS
//! destructors, and this brick never dlcloses a library that has in-flight
//! calls.
//!
//! The protocol: the platform hook dispatcher must wrap every entry into
//! module code with [`enter_module`] / [`leave_module`] (or the RAII
//! [`guard_module`]). `reload_module` refuses to swap a module with a
//! non-zero active-call counter (`Err("module busy")`), so a swap can only
//! dlclose a library that is provably quiescent.
//!
//! # Honest limits (hooked class-file hooks)
//!
//! The regenerated module re-runs `cplugin_init` with the same handshake and
//! re-registers its class-file hooks in the runtime pipeline. The pipeline
//! cannot attribute hooks to a module, so hooks registered by the *old*
//! mapping stay registered until the next class load dereferences them:
//! after a reload, that is a dangling call into an unmapped library. The
//! busy-counter protocol therefore does NOT make a reload safe for modules
//! that registered hooks. Reloading a module is a development/ops tool for
//! quiescent, hook-light modules (or runtimes whose dispatcher tracks hook
//! ownership); it is not a crash-isolated feature until the dispatcher
//! integrates [`enter_module`]/[`leave_module`].
//!
//! # Trigger
//!
//! [`install_reload_signal`] arms a UNIX SIGUSR1 trigger: the handler only
//! sets an atomic flag (async-signal-safe), and a dedicated reloader thread
//! polls the flag and reloads every registered module — dlopen/init/dlclose
//! are far from async-signal-safe and must never run inside the handler
//! (same flag-and-watchdog pattern as `signals.rs`). On non-Unix targets the
//! trigger compiles to a no-op stub so the runtime still builds.
//!
//! # Lock discipline
//!
//! dlopen/init/dlclose run arbitrary module code; a host must never hold its
//! registry lock across that code (cf. I-Machine "Plugin Hot-Reload: Three
//! dlopen Constraints", 2026). The registry mutex is therefore taken only
//! for mechanical bookkeeping (busy check, entry swap); the replacement's
//! dlopen + init run outside it, and the old library is dlclosed outside it.
//! A global [`SWAP_LOCK`] additionally serializes all swaps/registrations.
//!
//! # Windows
//!
//! libloading maps dlopen/dlsym/dlclose to LoadLibraryW/GetProcAddress/
//! FreeLibrary, so the registry and reload logic are fully portable; the
//! unit tests exercise bookkeeping and busy/init-failure semantics without
//! ever dlopening a real library.

use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{c_char, CString};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use cplug_abi::{CPluginApi, JavaVmPtr};

/// The module init handshake (the only ABI, see cplug-abi):
/// `cplugin_init(api, vm, options) -> i32`; nonzero means "do not use me".
type InitFn = unsafe extern "C" fn(*const CPluginApi, JavaVmPtr, *const c_char) -> i32;

/// Serializes swaps/registrations so only one happens at a time.
static SWAP_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
/// The module registry: id -> owned library + bookkeeping.
static REGISTRY: OnceLock<Mutex<HashMap<String, LoadedModule>>> = OnceLock::new();

/// Test-only seam: stubs the dlopen+init of a *replacement* library so unit
/// tests can simulate load/init failures without any real .so.
/// `Err(msg)` = dlopen/symbol failure; `Ok(rc)` with `rc != 0` = init
/// failure. `Ok(0)` is not representable (tests have no real `Library` to
/// install) and is rejected by [`acquire_replacement`].
#[cfg(test)]
type AcquireStub = fn(&Path) -> Result<i32, String>;

#[cfg(test)]
static STUB_ACQUIRE: OnceLock<Mutex<Option<AcquireStub>>> = OnceLock::new();

/// One registered module. The `Library` is owned: dropped (dlclose) when the
/// module is replaced or the process exits.
struct LoadedModule {
    /// `None` only for test-fabricated entries (unit tests cannot dlopen).
    lib: Option<Library>,
    path: PathBuf,
    /// `CPluginApi*` replayed to every `cplugin_init` handshake. Stored as
    /// `usize`: raw pointers are `!Send`, and the registry lives in a
    /// `static Mutex` (same trick as the runtime's `VM` static in lib.rs).
    api: usize,
    /// `JavaVM*` replayed to the handshake (usize, same reasoning).
    vm: usize,
    /// NUL-terminated options string replayed to the handshake.
    options: CString,
    loaded_at_unix: u64,
    /// Return code of the active library's `cplugin_init` (0 = healthy).
    init_rc: i32,
    /// In-flight calls into module code; reload refuses while > 0.
    active: usize,
    /// A reload is in progress: `enter_module` refuses new entries so the
    /// old library cannot gain callers between the busy check and dlclose.
    swapping: bool,
}

/// Panel-facing snapshot of one registered module.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleInfo {
    pub id: String,
    pub path: PathBuf,
    pub loaded_at_unix: u64,
    pub in_use: bool,
    pub init_rc: i32,
}

/// RAII guard for a module-call critical section: calls [`leave_module`] on
/// drop even if the hook panics, so a panic cannot leave the busy counter
/// stuck and block future reloads.
pub struct ModuleGuard {
    id: String,
    armed: bool,
}

impl Drop for ModuleGuard {
    fn drop(&mut self) {
        if self.armed {
            leave_module(&self.id);
        }
    }
}

fn swap_lock() -> &'static Mutex<()> {
    SWAP_LOCK.get_or_init(|| Mutex::new(()))
}

fn registry() -> &'static Mutex<HashMap<String, LoadedModule>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
fn acquire_stub() -> &'static Mutex<Option<AcquireStub>> {
    STUB_ACQUIRE.get_or_init(|| Mutex::new(None))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// dlopen `path`, look up `cplugin_init`, and run the init handshake with
/// the registered `api`/`vm`/`options`. `Ok` only when init returned 0. The
/// returned `Library` is loaded with libloading's platform defaults (unix:
/// RTLD_LAZY | RTLD_LOCAL — one module cannot shadow another's symbols).
#[cfg(not(test))]
fn acquire_replacement(path: &Path, api: usize, vm: usize, options: &CString) -> Result<Library, String> {
    // SAFETY: the library handle is owned by the returned Library; the init
    // symbol is borrowed from it and dropped before it (Symbol<'lib, T>).
    let lib = unsafe { Library::new(path) }
        .map_err(|e| format!("dlopen {}: {e}", path.display()))?;
    let init: Symbol<InitFn> = unsafe { lib.get(b"cplugin_init\0") }
        .map_err(|e| format!("{}: missing cplugin_init export: {e}", path.display()))?;
    // SAFETY: init is the module's exported handshake (cplug-abi: api points
    // at a live CPluginApi, vm at the JavaVM, options at a NUL-terminated
    // buffer alive for the call).
    let rc = unsafe { init(api as *const CPluginApi, vm as JavaVmPtr, options.as_ptr()) };
    if rc != 0 {
        return Err(format!("cplugin_init returned {rc}"));
    }
    Ok(lib)
}

/// Test-only twin of [`acquire_replacement`]: never dlopens. The stub
/// reports what a replacement's load/init would have said (`Err(msg)` =
/// dlopen/symbol failure, `Ok(rc)` with `rc != 0` = init failure). `Ok(0)`
/// is not representable — tests have no real `Library` to install — and is
/// rejected so tests cannot accidentally "succeed" a reload.
#[cfg(test)]
fn acquire_replacement(_path: &Path, _api: usize, _vm: usize, _options: &CString) -> Result<Library, String> {
    let hook = *acquire_stub().lock().unwrap();
    let Some(f) = hook else {
        return Err(
            "hot_reload: unit tests cannot dlopen a real library; set the acquire stub"
                .to_string(),
        );
    };
    match f(_path) {
        Err(msg) => Err(msg),
        Ok(rc) if rc != 0 => Err(format!("cplugin_init returned {rc}")),
        Ok(_) => Err(
            "hot_reload: a test stub cannot simulate a successful dlopen \
             (tests have no real Library to install)"
                .to_string(),
        ),
    }
}

/// Register a module: dlopens `path`, runs the init handshake with
/// `api`/`vm`/`options`, and on success the registry takes ownership of the
/// loaded `Library` (it will be dlclosed when the module is replaced). A
/// module whose dlopen or init fails is not registered — the registry is
/// untouched.
pub fn register_module(
    id: &str,
    path: PathBuf,
    api: *const CPluginApi,
    vm: JavaVmPtr,
    options: &str,
) -> Result<(), String> {
    let options = CString::new(options)
        .map_err(|_| format!("module '{id}': options contain an interior NUL"))?;
    let lib = acquire_replacement(&path, api as usize, vm as usize, &options)?;
    insert_entry(id, path, lib, api as usize, vm as usize, options)
        .map_err(|(e, _)| e)
}

/// Admit a module the caller already dlopened and initialized — the
/// runtime's startup path (`load_plugins`) performs the first `cplugin_init`
/// itself, so admitting avoids a second dlopen/init of the same file. Takes
/// ownership of the loaded `Library` (dlclose on replace) and captures the
/// handshake so a later [`reload_module`] can re-init the replacement.
///
/// On failure the caller gets the `Library` back (`Err((reason, lib))`) so
/// it can decide how to keep the mapping alive.
pub fn admit_module(
    id: &str,
    path: PathBuf,
    lib: Library,
    api: *const CPluginApi,
    vm: JavaVmPtr,
    options: &str,
) -> Result<(), (String, Library)> {
    let options = match CString::new(options) {
        Ok(o) => o,
        Err(_) => {
            return Err((
                format!("module '{id}': options contain an interior NUL"),
                lib,
            ))
        }
    };
    insert_entry(id, path, lib, api as usize, vm as usize, options)
}

/// Insert an already-acquired, already-initialized library into the
/// registry. The `Library` must be live: the registry owns it from here on
/// (dlclose on reload/replace). Refuses duplicate ids; on failure the
/// library is returned so the caller can keep the mapping alive itself.
fn insert_entry(
    id: &str,
    path: PathBuf,
    lib: Library,
    api: usize,
    vm: usize,
    options: CString,
) -> Result<(), (String, Library)> {
    let _swap = swap_lock().lock().unwrap();
    let mut reg = registry().lock().unwrap();
    if reg.contains_key(id) {
        return Err((format!("module '{id}' is already registered"), lib));
    }
    reg.insert(
        id.to_string(),
        LoadedModule {
            lib: Some(lib),
            path,
            api,
            vm,
            options,
            loaded_at_unix: unix_now(),
            init_rc: 0,
            active: 0,
            swapping: false,
        },
    );
    Ok(())
}

/// Reload a registered module: dlopen its path fresh, run `cplugin_init`
/// with the module's registered api/vm/options, and only if that succeeds
/// swap the registry entry and dlclose the old library.
///
/// Fails without touching the registry when: the id is unknown, the module
/// has in-flight calls (`Err("module busy")` — the platform dispatcher's
/// [`enter_module`]/[`leave_module`] protocol), a reload is already running,
/// or the replacement fails to load or to init (the old library stays
/// active — crash resilience).
pub fn reload_module(id: &str) -> Result<(), String> {
    let _swap = swap_lock().lock().unwrap();

    // Reserve the slot: fail fast if busy, then block new entries for the
    // rest of the swap so the old library cannot gain callers between the
    // busy check and its dlclose.
    let (path, api, vm, options) = {
        let mut reg = registry().lock().unwrap();
        let entry = reg
            .get_mut(id)
            .ok_or_else(|| format!("module '{id}' is not registered"))?;
        if entry.active > 0 {
            return Err(format!(
                "module '{id}' is busy: {} in-flight call(s)",
                entry.active
            ));
        }
        if entry.swapping {
            return Err(format!("module '{id}' is already being reloaded"));
        }
        entry.swapping = true;
        (entry.path.clone(), entry.api, entry.vm, entry.options.clone())
    };

    // dlopen + init the replacement OUTSIDE the registry lock: module code
    // must never run under our locks (deadlock-free callbacks, no lock held
    // across user code).
    //
    // Class-hook ownership: the replacement's cplugin_init registers its
    // hooks between `hook_seq` and now; the old library's, registered before
    // `hook_seq`, are truncated right before its dlclose so the pipeline
    // never calls into unmapped code (see crate::truncate_hooks_to).
    let hook_seq = crate::hook_registration_seq();
    let replacement = acquire_replacement(&path, api, vm, &options);

    let mut reg = registry().lock().unwrap();
    match replacement {
        Err(e) => {
            if let Some(entry) = reg.get_mut(id) {
                entry.swapping = false;
            }
            Err(e)
        }
        Ok(new_lib) => {
            let entry = reg
                .get_mut(id)
                .ok_or_else(|| format!("module '{id}' vanished during reload"))?;
            let old = entry.lib.replace(new_lib);
            entry.loaded_at_unix = unix_now();
            entry.init_rc = 0;
            entry.swapping = false;
            drop(reg);
            // The old library's hooks are stale: purge them before the
            // dlclose so a class load cannot dispatch into unmapped code.
            crate::truncate_hooks_to(hook_seq);
            // dlclose outside the lock. The old library is provably
            // quiescent: active was 0 at the check and swapping blocked new
            // entries until this point.
            drop(old);
            Ok(())
        }
    }
}

/// Mark entry into a module's code. Returns `false` (and does nothing) if
/// the id is unknown or a reload of that module is in progress — callers
/// must then skip running module code.
///
/// The platform hook dispatcher MUST pair every call into module code with
/// [`leave_module`] (or use [`guard_module`] for panic safety).
pub fn enter_module(id: &str) -> bool {
    let mut reg = registry().lock().unwrap();
    match reg.get_mut(id) {
        Some(e) if !e.swapping => {
            e.active += 1;
            true
        }
        _ => false,
    }
}

/// Mark exit from a module's code; releases one [`enter_module`] slot.
/// Unknown ids are a no-op.
pub fn leave_module(id: &str) {
    if let Some(e) = registry().lock().unwrap().get_mut(id) {
        e.active = e.active.saturating_sub(1);
    }
}

/// RAII variant of [`enter_module`]: enters the module and returns a guard
/// that leaves it on drop (panic-safe for the hook dispatcher). `None` when
/// the module is unknown or mid-reload.
pub fn guard_module(id: &str) -> Option<ModuleGuard> {
    enter_module(id).then(|| ModuleGuard {
        id: id.to_string(),
        armed: true,
    })
}

/// Snapshot of the registry, sorted by id, for a panel/console.
pub fn list_modules() -> Vec<ModuleInfo> {
    let reg = registry().lock().unwrap();
    let mut out: Vec<ModuleInfo> = reg
        .iter()
        .map(|(id, e)| ModuleInfo {
            id: id.clone(),
            path: e.path.clone(),
            loaded_at_unix: e.loaded_at_unix,
            in_use: e.active > 0,
            init_rc: e.init_rc,
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Sorted ids of every registered module — the reloadable module set.
pub fn loaded_libraries() -> Vec<String> {
    let mut ids: Vec<String> = registry().lock().unwrap().keys().cloned().collect();
    ids.sort();
    ids
}

/// Raw swap primitive (kept for backwards compatibility): swap a caller-held
/// `Library` with a fresh one from `new_path`. The caller must have quiesced
/// module code beforehand — this path has no busy tracking. Prefer
/// [`reload_module`] for registry-managed modules.
pub fn swap_library(
    lib: &mut Library,
    new_path: &Path,
    api: *const CPluginApi,
    vm: JavaVmPtr,
    options: &str,
) -> Result<(), String> {
    let _guard = swap_lock().lock().unwrap();
    let options = CString::new(options)
        .map_err(|_| "swap_library: options contain an interior NUL".to_string())?;
    let new: Library = acquire_replacement(new_path, api as usize, vm as usize, &options)?;
    // The old library stays alive until this swap function returns, then the
    // caller's `Library` is replaced; libloading drops the old one (dlclose).
    *lib = new;
    Ok(())
}

// ---------------------------------------------------------------------------
// Reload trigger: SIGUSR1 (UNIX only).
// ---------------------------------------------------------------------------

/// Set by the SIGUSR1 handler (async-signal-safe atomic store), claimed by
/// the reloader thread. `swap`-style claim: a burst of signals coalesces
/// into exactly one reload pass.
#[cfg(unix)]
static RELOAD_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(unix)]
static TRIGGER_INSTALLED: OnceLock<()> = OnceLock::new();

/// Poll period of the reloader thread.
#[cfg(unix)]
const RELOAD_POLL_MS: u64 = 200;

#[cfg(unix)]
extern "C" fn on_reload_signal(_sig: libc::c_int) {
    RELOAD_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Arm the hot-reload trigger: SIGUSR1 requests a reload of every registered
/// module. Mitigation of handler safety: the handler only stores an atomic
/// flag; the actual dlopen/init/dlclose work runs on a dedicated reloader
/// thread, so repeated signals coalesce and the handler stays
/// async-signal-safe (the flag-and-watchdog pattern of `signals.rs`).
/// Idempotent — only the first call arms anything.
///
/// On non-Unix targets (Windows) this is a no-op stub: there are no POSIX
/// signals to arm, and the registry plumbing still works via console/API
/// callers of [`reload_module`].
#[cfg(unix)]
pub fn install_reload_signal() -> Result<(), String> {
    TRIGGER_INSTALLED.get_or_init(|| {
        let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
        sa.sa_sigaction = on_reload_signal as *const () as usize;
        sa.sa_flags = libc::SA_RESTART;
        // SAFETY: sa is fully zeroed + configured above; the mask is a plain
        // libc sigset being emptied before use in sigaction.
        unsafe { libc::sigemptyset(&mut sa.sa_mask) };
        let rc = unsafe { libc::sigaction(libc::SIGUSR1, &sa, std::ptr::null_mut()) };
        if rc != 0 {
            eprintln!(
                "[crussty:hot-reload] sigaction(SIGUSR1) failed (rc {rc}); reload trigger disabled"
            );
            return;
        }
        match std::thread::Builder::new()
            .name("crussty-hot-reload".to_string())
            .spawn(reload_loop)
        {
            Ok(_) => eprintln!(
                "[crussty:hot-reload] SIGUSR1 reload trigger armed ({} module(s) reloadable)",
                loaded_libraries().len()
            ),
            Err(e) => eprintln!("[crussty:hot-reload] reloader thread failed to spawn: {e}"),
        }
    });
    Ok(())
}

/// Non-Unix stub: no POSIX signals on Windows; `reload_module` stays
/// callable through API/panel paths, and the runtime still builds.
#[cfg(not(unix))]
pub fn install_reload_signal() -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn reload_loop() {
    use std::sync::atomic::Ordering;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(RELOAD_POLL_MS));
        if !RELOAD_REQUESTED.swap(false, Ordering::SeqCst) {
            continue;
        }
        let ids = loaded_libraries();
        eprintln!(
            "[crussty:hot-reload] reload requested: {} module(s)",
            ids.len()
        );
        for id in &ids {
            match reload_module(id) {
                Ok(()) => {
                    eprintln!("[crussty:hot-reload] '{id}' reloaded (dlopen'd fresh + re-init)");
                }
                Err(e) => {
                    eprintln!("[crussty:hot-reload] '{id}' reload skipped: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn register_fake(
        id: &str,
        path: PathBuf,
        loaded_at_unix: u64,
        init_rc: i32,
        active: usize,
        swapping: bool,
    ) {
        registry().lock().unwrap().insert(
            id.to_string(),
            LoadedModule {
                lib: None,
                path,
                api: 0,
                vm: 0,
                options: CString::new("").expect("empty CString"),
                loaded_at_unix,
                init_rc,
                active,
                swapping,
            },
        );
    }

    /// Stub: replacement "loads" but its init fails with rc 7.
    fn stub_init_failure(_p: &Path) -> Result<i32, String> {
        Ok(7)
    }

    /// Stub: replacement dlopen fails.
    fn stub_load_failure(_p: &Path) -> Result<i32, String> {
        Err("stub: dlopen failed".to_string())
    }

    fn find(id: &str) -> ModuleInfo {
        list_modules()
            .into_iter()
            .find(|m| m.id == id)
            .expect("module present")
    }

    /// Tests run in parallel threads and share the global acquire stub;
    /// every test that reads or writes the stub takes this lock so its
    /// assertions see a deterministic stub.
    static TEST_STUB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn stub_guard() -> std::sync::MutexGuard<'static, ()> {
        TEST_STUB_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn list_modules_reports_bookkeeping() {
        register_fake("beta", PathBuf::from("/m/libbeta.so"), 2000, 5, 2, false);
        register_fake("alpha", PathBuf::from("/m/libalpha.so"), 1000, 0, 0, false);

        // Tests share the global registry, so only assert on our entries.
        let infos = list_modules();
        let a = infos.iter().find(|m| m.id == "alpha").expect("alpha present");
        assert_eq!(a.path, PathBuf::from("/m/libalpha.so"));
        assert_eq!(a.loaded_at_unix, 1000);
        assert_eq!(a.init_rc, 0);
        assert!(!a.in_use);

        let b = infos.iter().find(|m| m.id == "beta").expect("beta present");
        assert_eq!(b.path, PathBuf::from("/m/libbeta.so"));
        assert_eq!(b.loaded_at_unix, 2000);
        assert_eq!(b.init_rc, 5);
        assert!(b.in_use, "active counter > 0 shows as in_use");

        let ids = loaded_libraries();
        assert!(ids.contains(&"alpha".to_string()));
        assert!(ids.contains(&"beta".to_string()));
        assert_eq!(ids, {
            let mut sorted = ids.clone();
            sorted.sort();
            sorted
        }, "loaded_libraries is sorted");
    }

    #[test]
    fn enter_leave_toggles_busy_counter() {
        register_fake("busy-1", PathBuf::from("/m/busy.so"), 1, 0, 0, false);

        assert!(enter_module("busy-1"));
        assert!(find("busy-1").in_use);
        assert!(enter_module("busy-1"), "counter is 2 now");
        leave_module("busy-1");
        assert!(find("busy-1").in_use, "one call still in flight");
        leave_module("busy-1");
        assert!(!find("busy-1").in_use);

        assert!(!enter_module("ghost"), "unknown id is refused");
        leave_module("ghost"); // no-op, no panic
    }

    #[test]
    fn guard_releases_on_drop() {
        register_fake("guard-1", PathBuf::from("/m/guard.so"), 1, 0, 0, false);

        let g = guard_module("guard-1").expect("guard for registered module");
        assert!(find("guard-1").in_use);
        drop(g);
        assert!(!find("guard-1").in_use, "drop leaves the module");
        assert!(guard_module("ghost").is_none());
    }

    #[test]
    fn reload_unknown_module_errors() {
        let err = reload_module("ghost").unwrap_err();
        assert!(err.contains("not registered"), "got: {err}");
    }

    #[test]
    fn reload_busy_module_errors_without_loading() {
        let _serial = stub_guard();
        register_fake("busy-2", PathBuf::from("/m/busy2.so"), 1, 0, 1, false);
        *acquire_stub().lock().unwrap() = Some(stub_init_failure);

        let err = reload_module("busy-2").unwrap_err();
        assert!(err.contains("busy"), "busy check precedes any load: got: {err}");
        assert_eq!(find("busy-2").loaded_at_unix, 1, "entry untouched");
    }

    #[test]
    fn reload_while_swapping_errors() {
        register_fake("swapflag-1", PathBuf::from("/m/swap.so"), 1, 0, 0, true);

        let err = reload_module("swapflag-1").unwrap_err();
        assert!(err.contains("already being reloaded"), "got: {err}");
    }

    #[test]
    fn reload_init_failure_keeps_old_entry() {
        let _serial = stub_guard();
        register_fake("initfail-1", PathBuf::from("/m/initfail.so"), 42, 0, 0, false);
        *acquire_stub().lock().unwrap() = Some(stub_init_failure);

        let err = reload_module("initfail-1").unwrap_err();
        assert!(err.contains("cplugin_init returned 7"), "got: {err}");

        // Crash resilience: the old entry survives, untouched.
        let m = find("initfail-1");
        assert_eq!(m.path, PathBuf::from("/m/initfail.so"));
        assert_eq!(m.loaded_at_unix, 42);
        assert_eq!(m.init_rc, 0);
        assert!(!m.in_use);

        // The swap flag was cleared: a second reload is not stuck and fails
        // the same way instead of reporting "already being reloaded".
        let err2 = reload_module("initfail-1").unwrap_err();
        assert!(err2.contains("cplugin_init returned 7"), "got: {err2}");
    }

    #[test]
    fn reload_load_failure_keeps_old_entry() {
        let _serial = stub_guard();
        register_fake("loadfail-1", PathBuf::from("/m/loadfail.so"), 42, 0, 0, false);
        *acquire_stub().lock().unwrap() = Some(stub_load_failure);

        let err = reload_module("loadfail-1").unwrap_err();
        assert!(err.contains("dlopen failed"), "got: {err}");

        let m = find("loadfail-1");
        assert_eq!(m.loaded_at_unix, 42);
        assert_eq!(m.init_rc, 0);
        assert!(!m.in_use);
    }

    #[test]
    fn reload_without_stub_fails_cleanly_in_tests() {
        let _serial = stub_guard();
        register_fake("nostub-1", PathBuf::from("/m/nostub.so"), 1, 0, 0, false);
        *acquire_stub().lock().unwrap() = None;

        let err = reload_module("nostub-1").unwrap_err();
        assert!(err.contains("cannot dlopen"), "got: {err}");
        assert_eq!(find("nostub-1").loaded_at_unix, 1);
    }

    #[test]
    fn register_module_failure_does_not_register() {
        let _serial = stub_guard();
        *acquire_stub().lock().unwrap() = Some(stub_load_failure);
        let err = register_module(
            "reg-fail",
            PathBuf::from("/m/regfail.so"),
            std::ptr::null(),
            std::ptr::null_mut(),
            "modules=/m",
        )
        .unwrap_err();
        assert!(err.contains("dlopen failed"), "got: {err}");
        assert!(
            !loaded_libraries().contains(&"reg-fail".to_string()),
            "failed registration must not appear"
        );
    }
}
