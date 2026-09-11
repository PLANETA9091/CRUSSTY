//! crussty-runtime — the native injection engine (libcrussty_runtime.so).
//!
//! Loaded via `-agentpath:` by the launcher BEFORE any kernel class loads.
//! Jobs:
//!   1. claim JVMTI capabilities (class hooks + retransform);
//!   2. scan `modules/` recursively, dlopen every module with RTLD_LOCAL
//!      and call its `cplugin_init` (the only ABI, see cplug-abi);
//!   3. run the automatic hot-patch pipeline: every class load goes through
//!      the registered plugin hooks (JVMTI CLASS_FILE_LOAD_HOOK).
//!
//! Plugins are deliberately handed raw JavaVM*: no Java API, no limits.

#[allow(dead_code)] // platform bricks are a public API surface; used by modules
#[allow(ambiguous_glob_reexports)]
pub mod platform;
mod scan;

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CString};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use cplug_abi::{CPluginApi, ClassHookFn, JavaVmPtr, CPAPI_VERSION};
use jvmti_bindings::prelude::*;
#[allow(unused_imports)]
use jvmti_bindings::export_agent as export_runtime;
use libloading::Library;

/// A plugin class hook. Every hook is attributed to the module (and library
/// generation) that registered it, so a hot reload can purge exactly the
/// replaced generation's hooks — by owner — instead of guessing by a
/// registration-sequence bound.
#[derive(Clone)]
struct HookEntry {
    /// `Some((module id, library generation))` when registered inside a
    /// module `cplugin_init` handshake; `None` for registrations made
    /// outside any handshake (tests, platform bricks) that must never be
    /// purged by a reload.
    owner: Option<(String, u64)>,
    ctx: usize,
    func: ClassHookFn,
}

/// (owner, ctx, hook) entries registered by plugins, in registration order.
static HOOKS: OnceLock<Mutex<Vec<HookEntry>>> = OnceLock::new();
/// Set once every transform hook class (SchedulerHooks/NetHooks/TickHook/
/// StorageHooks) is defined and its natives registered. The transform engine
/// must not patch a class to call a hook class that does not exist yet:
/// first execution would NoClassDefFoundError (the same crash family as the
/// missing-hook-class bug, narrowed to a race). Rules are idle until this
/// flips — a rule-matching class loaded in the window simply stays
/// untransformed, which is harmless: every current target loads long after
/// install.
static HOOK_CLASSES_READY: AtomicBool = AtomicBool::new(false);

pub fn hook_classes_ready() -> bool {
    HOOK_CLASSES_READY.load(Ordering::Relaxed)
}

pub fn mark_hook_classes_ready() {
    HOOK_CLASSES_READY.store(true, Ordering::Relaxed);
}

// Registration context: the module (id, library generation) whose
// `cplugin_init` handshake is currently running **on this thread**. Class
// hooks and event subscriptions registered during the handshake are
// stamped with this owner; a hot reload later purges registrations by
// owner, so the replaced generation's hooks/subscriptions are dropped
// exactly, without touching hooks of other modules or of the new
// generation.
//
// Thread-local by contract (TASK-168): a handshake window on the loader
// thread must never stamp registrations made by other threads — parallel
// module loads, background publishers or unrelated bus subscribers. The
// previous implementation kept this in a process-global mutex, which let a
// concurrent handshake on thread A stamp thread B's subscriptions (caught
// red-handed by the TASK-167 concurrent memo test in CI: a parallel test
// window stamped the test bus's subscriptions, and the mid-swap guard then
// skipped them).
thread_local! {
    static REG_CTX: Cell<Option<(String, u64)>> = const { Cell::new(None) };
}

/// RAII guard: while alive, every class hook / event subscription
/// registered on this thread is attributed to module `id`, library
/// generation `gen`. Restores the previous context on drop.
pub struct RegistrationGuard {
    prev: Option<(String, u64)>,
}

impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        let prev = self.prev.take();
        REG_CTX.with(|c| c.set(prev));
    }
}

/// Start a module registration window (see [`RegistrationGuard`]).
pub fn begin_registration(id: &str, gen: u64) -> RegistrationGuard {
    let prev = REG_CTX.with(|c| {
        let prev = c.take();
        c.set(Some((id.to_string(), gen)));
        prev
    });
    RegistrationGuard { prev }
}

/// The owner stamp applied to registrations made right now on this thread
/// (from a module handshake), or `None` outside any.
pub fn registration_owner() -> Option<(String, u64)> {
    // take/put-back on a `Cell<Option<(String, u64)>>`: no locks, no borrow
    // flags, no allocation outside a live window (where the clone is a
    // one-time registration-time cost). ~1 TLS access — see the TASK-166
    // TLS law; the former global-mutex fast gate (REG_CTX_SET) is gone: the
    // TLS read IS the gate now.
    REG_CTX.with(|c| {
        let cur = c.take();
        let out = cur.clone();
        c.set(cur);
        out
    })
}

/// Drop every hook registered by module `id` of library generation `gen` —
/// the generation a hot reload is replacing. Idempotent; other owners are
/// untouched (including a newer generation of the same module, which is
/// exactly why the purge is keyed by (id, gen) and not by id alone).
pub fn purge_module_hooks(id: &str, gen: u64) {
    let removed = {
        let mut h = hooks().lock().unwrap();
        let before = h.len();
        h.retain(|entry| entry.owner.as_ref() != Some(&(id.to_string(), gen)));
        before - h.len()
    };
    if removed > 0 {
        // TASK-163: keep the lock-free live-count honest (Release so a
        // concurrent class-load gate that observes 0 cannot miss a purge).
        PLUGIN_HOOKS_LIVE.fetch_sub(removed, Ordering::AcqRel);
        eprintln!(
            "[crussty-runtime] hook purge: dropped {removed} hook(s) owned by '{id}' gen {gen}"
        );
    }
}

/// Live plugin class-hook count (TASK-163): the per-class-load fast gate and
/// the dispatch-prep gate key off this — zero means the JVMTI phase query,
/// the registry lock + Vec clone and the CString build are all skipped on
/// every class load the JVM performs.
static PLUGIN_HOOKS_LIVE: AtomicUsize = AtomicUsize::new(0);
/// Raw jvmtiEnv pointer as usize (JVMTI envs are process-wide, usable from
/// any thread — safe to share).
static JVMTI_ENV: OnceLock<usize> = OnceLock::new();
/// Raw JavaVM* as usize, for attaching plugin threads (JVMTI calls like
/// GetLoadedClasses/RetransformClasses need an attached thread).
static VM: OnceLock<usize> = OnceLock::new();
/// Loaded plugin libraries, kept alive for the whole JVM lifetime (fallback
/// keep-alive for modules the hot-reload registry could not take over; the
/// registry itself owns the libraries of modules admitted for reload).
static LIBS: OnceLock<Mutex<Vec<Library>>> = OnceLock::new();
/// The CPluginApi handed to every module (built once; function pointers are
/// process-stable, so reloads replay the same handshake). Wrapped because the
/// raw table carries a `*const CPlatformApi` (not auto Send/Sync).
struct SafeApi(CPluginApi);
unsafe impl Send for SafeApi {}
unsafe impl Sync for SafeApi {}
static RUNTIME_API: OnceLock<SafeApi> = OnceLock::new();

fn cplugin_api() -> &'static CPluginApi {
    let safe = RUNTIME_API.get_or_init(|| SafeApi(CPluginApi {
        version: CPAPI_VERSION,
        register_class_hook: Some(api_register_class_hook),
        jvmti_allocate: Some(api_jvmti_allocate),
        retransform_class: Some(api_retransform_class),
        claim: Some(api_claim),
        platform: &platform::c_bridge::PLATFORM_API,
    }));
    &safe.0
}

fn hooks() -> &'static Mutex<Vec<HookEntry>> {
    HOOKS.get_or_init(|| Mutex::new(Vec::new()))
}
fn libs() -> &'static Mutex<Vec<Library>> {
    LIBS.get_or_init(|| Mutex::new(Vec::new()))
}

fn jvmti_env() -> Option<Jvmti> {
    JVMTI_ENV
        .get()
        .map(|p| unsafe { Jvmti::from_raw(*p as *mut jvmti::jvmtiEnv) })
}

/// Attach the calling thread to the VM if it is not already attached, run `f`,
/// then detach only if we attached ourselves. JVMTI calls that enumerate or
/// transform classes (GetLoadedClasses, RetransformClasses) fail on an
/// unattached native thread; plugin background threads are unattached.
fn with_attached<R>(f: impl FnOnce() -> R) -> Option<R> {
    unsafe {
        let raw_vm = VM.get().copied()? as *mut jni::JavaVM;
        let vm = raw_vm;
        if vm.is_null() || (*vm).is_null() {
            return None;
        }
        let mut env_ptr: *mut jni::JNIEnv = std::ptr::null_mut();
        let rc = ((**vm).GetEnv)(
            vm,
            &mut env_ptr as *mut *mut jni::JNIEnv as *mut *mut std::ffi::c_void,
            jni::JNI_VERSION_1_6,
        );
        if rc == jni::JNI_OK && !env_ptr.is_null() {
            return Some(f());
        }
        if rc != jni::JNI_EDETACHED {
            return None;
        }
        let rc = ((**vm).AttachCurrentThread)(
            vm,
            &mut env_ptr as *mut *mut jni::JNIEnv as *mut *mut std::ffi::c_void,
            std::ptr::null_mut(),
        );
        if rc != jni::JNI_OK || env_ptr.is_null() {
            return None;
        }
        let out = f();
        ((**vm).DetachCurrentThread)(vm);
        Some(out)
    }
}

#[derive(Default)]
struct CrusstyRuntime;

impl CrusstyRuntime {
    /// Shared engine bring-up. Called either from `Agent_OnLoad` (the
    /// `-agentpath:` path) or from `JNI_OnLoad` (the single-jar path, where
    /// the Java bootstrapper loads this library with `System.load` before
    /// the kernel classloader starts). JVMTI's `GetEnv` is legal on a Java
    /// thread in the live phase, so both entry points get a working env.
    fn init(&self, vm: *mut jni::JavaVM, options: &str) -> jni::jint {
        eprintln!("[crussty-runtime] v2.0.0 loaded (options: {})", options);

        let jvmti = match Jvmti::new(vm) {
            Ok(env) => env,
            Err(e) => {
                eprintln!("[crussty-runtime] no jvmti env: {e}");
                return jni::JNI_ERR;
            }
        };
        if let Err(e) = jvmti.add_capabilities_with(|caps| {
            caps.set_can_generate_all_class_hook_events(true);
            caps.set_can_retransform_classes(true);
        }) {
            eprintln!("[crussty-runtime] add capabilities failed: {e:?}");
            return jni::JNI_ERR;
        }
        if let Err(e) = jvmti.set_event_callbacks(get_default_callbacks()) {
            eprintln!("[crussty-runtime] set callbacks failed: {e:?}");
            return jni::JNI_ERR;
        }
        if let Err(e) = jvmti.enable_events_global(&[jvmti::JVMTI_EVENT_CLASS_FILE_LOAD_HOOK]) {
            eprintln!("[crussty-runtime] enable event failed: {e:?}");
            return jni::JNI_ERR;
        }
        let _ = JVMTI_ENV.set(jvmti.raw() as usize);
        let _ = VM.set(vm as usize);

        let opts = parse_options(options);
        if let Some(dir) = &opts.modules {
            load_plugins(dir, vm as JavaVmPtr, options);
        } else {
            eprintln!("[crussty-runtime] no modules= in options; nothing injected");
        }
        eprintln!(
            "[crussty-runtime] pipeline ready: {} module hook(s)",
            hooks().lock().unwrap().len()
        );

        // Platform default transform rules (network / scheduler / storage
        // surfaces). Idempotent; must be registered before kernel classes
        // load — the agent claims class hooks before boot, so the rules
        // fire at class load (the engine runs them in the hook pipeline).
        platform::network::install_default_rules();
        platform::scheduler::install_default_rules();
        if let Err(e) = platform::storage::install_default_rules() {
            eprintln!("[crussty-runtime] storage default rules failed: {e}");
        }
        // Native RCON repair lane: watcher + (reflective swap | native serve).
        // The watcher stands down until the kernel's game port is up, so
        // arming it here (before the kernel boots) races nothing.
        platform::rcon::install();
        eprintln!(
            "[crussty-runtime] transform engine: {} rule(s) registered",
            platform::transform::global_engine().rules().len()
        );

        // Define the transform hook classes (SchedulerHooks/StorageHooks/
        // NetHooks/TickHook) into the system class loader and register their
        // natives — the injected ()V probes must resolve at first execution
        // of a patched kernel method. Deliberately scheduled off this thread:
        // agent init runs inside JNI_CreateJavaVM where AttachCurrentThread
        // faults the JVM (SIGSEGV at libjvm).
        platform::hooks::schedule_install();

        // Platform bricks: crash handlers first (any fault from here on must
        // produce a report, not a silent death), then telemetry + events.
        // CRUSSTY_NO_SIGNALS=1 disables the handlers (diagnostics/troubleshooting).
        if std::env::var_os("CRUSSTY_NO_SIGNALS").is_none() {
            let _ = platform::signals::install_handlers();
            // SIGUSR1 = hot-reload trigger for registered modules (no-op on
            // Windows). Kept under the same gate: a no-signal build should
            // not arm surprise signal handlers either.
            let _ = platform::hot_reload::install_reload_signal();
        }
        if let Some(sock) = &opts.telemetry {
            match platform::telemetry::init(&sock.display().to_string()) {
                Ok(()) => eprintln!("[crussty-runtime] telemetry on {}", sock.display()),
                Err(e) => eprintln!("[crussty-runtime] telemetry disabled: {e}"),
            }
        }
        platform::telemetry::set_uptime(0);
        platform::events::global().publish(
            platform::events::lifecycle::PLUGIN_LOADED,
            &serde_json::json!({ "runtime": "crussty", "phase": "ready" }),
        );

        jni::JNI_OK
    }
}

/// Single-jar entry point: the Java bootstrapper loads this library with
/// `System.load` (JNI path, no `-agentpath:` needed), so hosting panels can
/// run the kernel as a plain `java -jar server.jar`. Options come from the
/// `CRUSSTY_RUNTIME_OPTIONS` env var, falling back to `crussty/options.txt`
/// written by the bootstrapper next to the working directory.
#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: *mut jni::JavaVM, _reserved: *mut c_void) -> jni::jint {
    if JVMTI_ENV.get().is_some() {
        // Already brought up as a JVMTI agent earlier; JNI_OnLoad is a no-op.
        return jni::JNI_VERSION_1_6;
    }
    let options = std::env::var("CRUSSTY_RUNTIME_OPTIONS")
        .ok()
        .or_else(|| {
            std::fs::read_to_string("crussty/options.txt")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default();
    let rc = CrusstyRuntime.init(vm, &options);
    if rc != jni::JNI_OK {
        return rc;
    }
    jni::JNI_VERSION_1_6
}

/// The JVM calls this after JNI_OnLoad when the library is unloaded — no-op.
#[no_mangle]
pub extern "system" fn JNI_OnUnload(_vm: *mut jni::JavaVM, _reserved: *mut c_void) {}

impl Agent for CrusstyRuntime {
    fn on_load(&self, vm: *mut jni::JavaVM, options: &str) -> jni::jint {
        self.init(vm, options)
    }
    fn class_file_load_hook(
        &self,
        _jni: *mut jni::JNIEnv,
        _class_being_redefined: jni::jclass,
        _loader: jni::jobject,
        name: *const c_char,
        _protection_domain: jni::jobject,
        class_data_len: jni::jint,
        class_data: *const u8,
        new_class_data_len: *mut jni::jint,
        new_class_data: *mut *mut u8,
    ) {
        // FAST GATE (TASK-163): with no transform rules, no plugin class
        // hooks and no lifecycle subscribers there is nothing this callback
        // can produce — skip the name derivation (the per-load constant-pool
        // walk), the JVMTI phase query, the registry lock/clone and the
        // CString build entirely. This is the shape of a runtime whose
        // modules use side_table/events/storage but never patch the kernel.
        // Release-side ordering: rule registrations and hook registrations
        // both publish under their own locks/atomics before the counts this
        // gate reads, so a gate miss can only ever under-count by a moment —
        // and the class then runs untransformed exactly as it would on a
        // failed transform (documented degrade direction).
        let hooks_live = PLUGIN_HOOKS_LIVE.load(Ordering::Acquire) > 0;
        if !hooks_live
            && platform::transform::global_engine().rule_count() == 0
            && !platform::events::global_ref()
                .has_subscribers(platform::lifecycle::CLASS_LOADED)
        {
            return;
        }
        // The VM-provided `name` pointer is NOT reliably NUL-terminated on
        // HotSpot: it can point into the interned-symbol arena where the
        // next symbol's bytes follow immediately ("ImprovedNoise" ->
        // "ImprovedNoisejaE"). Every consumer that pattern-matches against
        // that string (platform transform rules, plugin hooks) silently
        // misses its targets. The class file itself is the source of truth:
        // parse this_class from its constant pool and prefer that.
        let data_slice = if !class_data.is_null() && class_data_len > 10 {
            Some(unsafe {
                std::slice::from_raw_parts(class_data, class_data_len as usize)
            })
        } else {
            None
        };
        // A panic unwinding through this extern "C" callback is fatal
        // (abort) — the parser must never take the JVM down with it; the
        // borrowed name keeps the hot path allocation-free.
        let owned_name: String;
        let name: &str = match data_slice
            .as_deref()
            .and_then(|b| std::panic::catch_unwind(|| class_file_name(b)).ok().flatten())
        {
            Some(n) => n,
            None => {
                owned_name = read_bounded_cstr(name);
                &owned_name
            }
        };
        // Dispatch prep (TASK-163): the JVMTI phase query and the registry
        // lock + Vec clone run only when plugin hooks exist at all; the
        // CString for the C-ABI call is built only when a hook will run.
        //
        // JVMTI-PHASE GATE (shutdown-crash family, hs_err "Signal Dispatcher"
        // 2026-09-07/08, 4 occurrences — LAW): the JVM keeps LOADING classes
        // while it dies, and this hook fires for them; module-owned hooks do
        // arbitrary JNI work and driving them against a dying VM is the
        // crash chain. Gate the snapshot to an EMPTY table for everything
        // but LIVE (skip is the documented degrade direction). The
        // byte-transform engine below stays unconditional on purpose (pure
        // byte work, degrades to "class runs untransformed").
        // TASK-46-class hardening (S7-8): the registry lock is
        // JVMTI-callback-reachable; a poisoned mutex must not unwind across
        // the trampoline (= VM abort).
        let registered = if hooks_live
            && jvmti_env()
                .and_then(|env| env.get_phase().ok())
                .map(|phase| phase == jvmti_bindings::sys::jvmti::JVMTI_PHASE_LIVE)
                .unwrap_or(false)
        {
            hooks()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        } else {
            Vec::new()
        };

        // NUL-terminated copy of the class name for the C-ABI plugin hook
        // (class names never contain interior NULs, so this is infallible in
        // practice; None only if the name somehow embedded one, in which case
        // plugin hooks are skipped for this class rather than reading garbage).
        let cname = if registered.is_empty() {
            None
        } else {
            std::ffi::CString::new(name).ok()
        };

        let mut current: *const u8 = class_data;
        let mut current_len = class_data_len as usize;
        // Holds the chained replacement bytes; kept alive by binding until the
        // end of the hook, then freed naturally.
        let mut pending: Option<Vec<u8>> = None;

        // Readiness gate: never emit a probe call into a hook class that is
        // not installed yet (NoClassDefFoundError at first execution). The
        // engine is elsewise unconditional; the gate is what turns the
        // install race into a safe no-op window.
        let engine_armed = hook_classes_ready();

        // 1. Platform transform engine (BEFORE plugin hooks): transform rules
        //    registered by the platform bricks (network / scheduler / storage
        //    surfaces) run on the pristine bytes; plugins then see the
        //    transformed class. The engine is pure byte-level work — no JNI,
        //    no define_class — so it is safe on the class-loading thread, and
        //    it is cheap per class: only classes whose internal name matches a
        //    registered rule pattern are parsed, everything else passes
        //    through after a string check. A failed transform logs and passes
        //    the class through untransformed (the platform never fails a load).
        if engine_armed && current_len > 0 && !current.is_null() {
            let bytes = unsafe { std::slice::from_raw_parts(current, current_len) };
            match platform::transform::global_engine().apply(&name, bytes) {
                Ok(Some(t)) => {
                    pending = Some(t.bytes);
                    current = pending.as_ref().map_or(current, Vec::as_ptr);
                    current_len = pending.as_ref().map_or(current_len, Vec::len);
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!(
                        "[crussty-runtime] transform '{name}' failed; class runs untransformed: {e}"
                    );
                    if std::env::var_os("CRUSSTY_DUMP_FAILED").is_some() {
                        let mut p = std::env::temp_dir();
                        p.push(format!("cflh-fail-{}.class", name.replace('/', ".")));
                        let _ = std::fs::write(&p, bytes);
                        eprintln!("[crussty-runtime] dumped original to {p:?}");
                    }
                }
            }
        }

        // 2. Plugin hooks chain in registration order: each sees the previous
        //    output (the engine's included). NOTE: `registered` is
        //    phase-gated above (empty unless JVMTI phase == LIVE) — the
        //    shutdown-crash family fix lives there, this loop is untouched.
        let trace = std::env::var_os("CRUSSTY_TRACE_HOOKS").is_some();
        let trace_filter = std::env::var("CRUSSTY_TRACE_CLASS").ok();
        let want_trace = trace
            && trace_filter
                .as_deref()
                .map(|f| name.contains(f))
                .unwrap_or(true);
        for entry in &registered {
            // Quiescence protocol: module-owned hooks run under a module
            // guard. A reload refuses while a hook is in flight, and this
            // hook is skipped entirely while its module is mid-swap (the
            // replacement re-registers; the class simply stays
            // untransformed for this load).
            let guard = entry.owner.as_ref().and_then(|(id, _)| {
                crate::platform::hot_reload::guard_module(id)
            });
            if entry.owner.is_some() && guard.is_none() {
                continue;
            }
            let mut out: *mut u8 = std::ptr::null_mut();
            let mut out_len: usize = 0;
            if want_trace {
                let owner = entry
                    .owner
                    .as_ref()
                    .map(|(id, g)| format!("{id}/g{g}"))
                    .unwrap_or_else(|| "-".to_string());
                eprintln!(
                    "[crussty-trace] '{name}' -> hook owner={owner} ctx=0x{:x} fn={:#x} len={}",
                    entry.ctx, entry.func as usize, current_len
                );
            }
            let rc = unsafe {
                // CRITICAL BUG FIX (agent-7625532f, 2026-09-08): `name` is a
                // Rust String — NOT NUL-terminated. Passing `name.as_ptr()`
                // to the C-ABI plugin hook made every consumer's C-string
                // scan run past the end into unrelated heap bytes, so the
                // effective hook name was heap-layout-dependent: some classes
                // (e.g. net/minecraft/world/level/levelgen/synth/PerlinNoise)
                // matched, others (e.g. net/minecraft/world/entity/Entity)
                // silently missed their hooks — whole-body patches silently
                // became no-ops with retransform rc=0. Always pass a
                // properly NUL-terminated copy.
                match cname.as_deref() {
                    Some(p) => (entry.func)(
                        entry.ctx as *mut c_void,
                        p.as_ptr() as *const c_char,
                        current,
                        current_len,
                        &mut out,
                        &mut out_len,
                    ),
                    None => 1,
                }
            };
            drop(guard);
            if rc == 0 && !out.is_null() {
                let mut copy = Vec::with_capacity(out_len);
                copy.extend_from_slice(unsafe { std::slice::from_raw_parts(out, out_len) });
                // the plugin buffer was jvmti-allocated; deallocate after
                // copying so intermediate patches never leak
                if let Some(env) = jvmti_env() {
                    let _ = env.deallocate(out);
                }
                current = copy.as_ptr();
                current_len = copy.len();
                pending = Some(copy);
            }
        }

        // 3. Lifecycle event: zero-weight on the load path — the payload is
        //    only built when something actually subscribes.
        publish_class_loaded(&name, current_len);

        if pending.is_some() {
            // Hand the final bytes to JVMTI via its own allocator.
            if let (Some(env), false) = (
                jvmti_env(),
                new_class_data.is_null() || new_class_data_len.is_null(),
            ) {
                if let Ok(ptr) = env.allocate(current_len as jni::jlong) {
                    unsafe {
                        std::ptr::copy_nonoverlapping(current, ptr, current_len);
                        *new_class_data = ptr;
                        *new_class_data_len = current_len as jni::jint;
                    }
                }
            }
        }
    }
}

export_runtime!(CrusstyRuntime);

/// Bounded C-string read of the VM-provided event name (last-resort
/// fallback when the class bytes cannot be parsed). The buffer is not
/// guaranteed NUL-terminated — cap the read at 128 bytes.
fn read_bounded_cstr(name: *const c_char) -> String {
    if name.is_null() {
        return "<unknown>".to_string();
    }
    let mut end = 0usize;
    unsafe {
        while end < 128 && *name.add(end) != 0 {
            end += 1;
        }
        std::str::from_utf8(std::slice::from_raw_parts(name.cast::<u8>(), end))
            .unwrap_or("<bad-utf8>")
            .to_string()
    }
}

/// Extract the true internal class name from class-file bytes: walk the
/// constant pool, read this_class -> name_index -> Utf8. O(pool size),
/// zero allocation, no panics on malformed input (returns None instead —
/// callers fall back to the VM name).
///
/// Zero-alloc (TASK-163): the previous version materialised two
/// `Vec<Option<_>>` pools sized by cp_count plus a String on EVERY class
/// load (~0.85 µs measured, the fattest line in the JVM's hottest loop).
/// This walker stores nothing — pass 1 finds the pool end, pass 2 walks to
/// this_class and its Utf8 (usually in the same pass, since the name Utf8
/// typically precedes the Class entry) and borrows the name bytes straight
/// out of `class_data`.
fn class_file_name(data: &[u8]) -> Option<&str> {
    if data.len() < 10 || data[0..4] != [0xCA, 0xFE, 0xBA, 0xBE] {
        return None;
    }
    let u16_at = |i: usize| -> Option<u16> {
        if i + 2 > data.len() {
            return None;
        }
        Some(u16::from_be_bytes([data[i], data[i + 1]]))
    };
    let cp_count = u16_at(8)? as usize;
    if cp_count == 0 {
        return None;
    }
    // One walk (TASK-163): record each entry's start offset into a STACK
    // table — no heap allocation, no second pass. Pools beyond the table
    // (rare) fall back to the storage-free two-pass walker below.
    if cp_count <= 1024 {
        let mut offs = [0u32; 1024];
        let mut idx = 10usize;
        let mut i = 1usize;
        while i < cp_count {
            offs[i] = idx as u32;
            let tag = *data.get(idx)?;
            idx += match tag {
                1 => 3 + u16_at(idx + 1)? as usize,
                7 | 8 | 16 | 19 | 20 => 3,
                15 => 4,
                3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => 5,
                5 | 6 => 9,
                _ => return None,
            };
            i += match tag {
                5 | 6 => 2,
                _ => 1,
            };
        }
        // access_flags(2) this_class(2) right after the pool
        let this_idx = u16_at(idx + 2)? as usize;
        if this_idx == 0 || this_idx >= cp_count {
            return None;
        }
        let cat = offs[this_idx] as usize;
        if *data.get(cat)? != 7 {
            return None;
        }
        let name_idx = u16_at(cat + 1)? as usize;
        if name_idx == 0 || name_idx >= cp_count {
            return None;
        }
        let nat = offs[name_idx] as usize;
        if *data.get(nat)? != 1 {
            return None;
        }
        let len = u16_at(nat + 1)? as usize;
        let start = nat + 3;
        if start + len > data.len() {
            return None;
        }
        return std::str::from_utf8(&data[start..start + len]).ok();
    }
    // Fallback for pathological pools (>1024 entries): storage-free
    // two-pass — pass 1 finds the pool end, pass 2 walks to this_class and
    // then to its Utf8.
    let step = |idx: usize| -> Option<(usize, usize)> {
        let tag = *data.get(idx)?;
        let (bytes, slots) = match tag {
            1 => (3 + u16_at(idx + 1)? as usize, 1),
            7 | 8 | 16 | 19 | 20 => (3, 1),
            15 => (4, 1),
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => (5, 1),
            5 | 6 => (9, 2),
            _ => return None,
        };
        Some((bytes, slots))
    };
    let mut idx = 10usize;
    let mut i = 1usize;
    while i < cp_count {
        let (bytes, slots) = step(idx)?;
        idx += bytes;
        i += slots;
    }
    let this_idx = u16_at(idx + 2)? as usize;
    if this_idx == 0 || this_idx >= cp_count {
        return None;
    }
    let walk_to = |target: usize| -> Option<(u8, usize)> {
        let mut idx = 10usize;
        let mut i = 1usize;
        while i < cp_count {
            if i == target {
                return Some((*data.get(idx)?, idx));
            }
            let (bytes, slots) = step(idx)?;
            idx += bytes;
            i += slots;
        }
        None
    };
    let (tag, at) = walk_to(this_idx)?;
    if tag != 7 {
        return None;
    }
    let name_idx = u16_at(at + 1)? as usize;
    if name_idx == 0 || name_idx >= cp_count {
        return None;
    }
    let (tag, at) = walk_to(name_idx)?;
    if tag != 1 {
        return None;
    }
    let len = u16_at(at + 1)? as usize;
    let start = at + 3;
    if start + len > data.len() {
        return None;
    }
    std::str::from_utf8(&data[start..start + len]).ok()
}

#[cfg(test)]
mod class_name_tests {
    use super::class_file_name;

    /// Build a minimal valid class file with the given internal name and
    /// optional trailing junk after the constant pool (the parser must stop
    /// exactly at the pool end and read this_class).
    fn minimal_class(name: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // magic
        b.extend_from_slice(&[0, 0]); // minor
        b.extend_from_slice(&[0, 52]); // major 8
        b.extend_from_slice(&[0, 3]); // cp_count = 2 entries + sentinel
        b.push(1); // Utf8
        b.extend_from_slice(&(name.len() as u16).to_be_bytes());
        b.extend_from_slice(name);
        b.push(7); // Class
        b.extend_from_slice(&[0, 1]); // -> utf8 #1
        b.extend_from_slice(&[0, 0x21]); // access: public super
        b.extend_from_slice(&[0, 2]); // this_class -> #2
        b.extend_from_slice(&[0, 0]); // super = null
        b
    }

    #[test]
    fn parses_simple_name() {
        let data = minimal_class(b"foo/Bar");
        assert_eq!(class_file_name(&data), Some("foo/Bar"));
    }

    #[test]
    fn parses_name_with_long_and_double_pool_entries() {
        // long entry occupies two pool slots — the walk must skip both.
        let mut b = Vec::new();
        b.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]);
        b.extend_from_slice(&[0, 0]);
        b.extend_from_slice(&[0, 52]); // major
        b.extend_from_slice(&[0, 5]); // slots: 1..=4
        b.push(5); // long (two slots: #1,#2)
        b.extend_from_slice(&[0; 8]);
        b.push(1); // Utf8 #3
        b.extend_from_slice(&[0, 4]);
        b.extend_from_slice(b"Test");
        b.push(7); // Class #4
        b.extend_from_slice(&[0, 3]);
        b.extend_from_slice(&[0, 0x21]);
        b.extend_from_slice(&[0, 4]);
        b.extend_from_slice(&[0, 0]);
        assert_eq!(class_file_name(&b), Some("Test"));
    }

    #[test]
    fn rejects_truncated_and_garbage() {
        assert_eq!(class_file_name(&[]), None);
        assert_eq!(class_file_name(&[1, 2, 3]), None);
        assert_eq!(class_file_name(&[0xCA, 0xFE, 0xBA, 0xBE, 0, 0]), None);
        let mut magic_only = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52, 0, 5];
        magic_only.push(99); // invalid tag
        assert_eq!(class_file_name(&magic_only), None);
    }

    #[test]
    fn survives_pool_larger_than_data() {
        // cp_count claims 1000 entries but data ends right after — must
        // return None (not panic / not read OOB).
        let mut b = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52];
        b.extend_from_slice(&[0x03, 0xE8]); // 1000
        b.push(1);
        assert_eq!(class_file_name(&b), None);
    }
}

/// Publish the `platform.class_loaded` lifecycle event for a class load.
/// Zero-weight on the class-load path: the payload is only built when the
/// bus actually has subscribers (the hook runs on the class-loading thread,
/// which must stay cheap).
fn publish_class_loaded(name: &str, bytes_len: usize) {
    let bus = platform::events::global_ref();
    if bus.has_subscribers(platform::lifecycle::CLASS_LOADED) {
        bus.publish(
            platform::lifecycle::CLASS_LOADED,
            &serde_json::json!({ "name": name, "bytes": bytes_len }),
        );
    }
}

/// Trampolines handed to plugins through CPluginApi.
unsafe extern "C" fn api_register_class_hook(ctx: *mut c_void, hook: ClassHookFn) -> i32 {
    let owner = registration_owner();
    hooks().lock().unwrap().push(HookEntry {
        owner,
        ctx: ctx as usize,
        func: hook,
    });
    // Release: a class-load gate that observes the new count on another
    // thread is guaranteed to see the pushed entry through the registry
    // lock (the push happened under it).
    PLUGIN_HOOKS_LIVE.fetch_add(1, Ordering::Release);
    0
}
unsafe extern "C" fn api_jvmti_allocate(size: usize) -> *mut u8 {
    match jvmti_env().and_then(|env| env.allocate(size as jni::jlong).ok()) {
        Some(p) => p,
        None => std::ptr::null_mut(),
    }
}

/// Retransform a loaded class by internal name (e.g. "a/b/C"). Re-enters the
/// plugin hook pipeline, so plugin hooks can patch a class that loaded before
/// they were ready. Returns 0 on success, -1 if the class is not loaded, -2 if
/// GetLoadedClasses failed, -3 if arguments were bad, -5 if RetransformClasses
/// failed, -6 if the calling thread could not be attached.
/// Global claim registry: keys modules must take before registering a
/// JVM-visible resource (class names, native (name,sig) pairs). A key is
/// free, or owned by the module that claimed it first — a second owner gets
/// -1 and must skip its registration, so two modules can never silently
/// redefine the same class or overwrite each other's natives.
static CLAIMS: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

unsafe extern "C" fn api_claim(owner: usize, key: *const c_char) -> i32 {
    let key = if key.is_null() {
        return -2;
    } else {
        let mut end = 0usize;
        while end < 4096 && *key.add(end) != 0 {
            end += 1;
        }
        std::str::from_utf8(std::slice::from_raw_parts(key.cast::<u8>(), end))
            .unwrap_or("<bad-utf8>")
            .to_string()
    };
    let mut m = CLAIMS.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
    match m.get(&key).copied() {
        Some(claimed) if claimed != owner => {
            eprintln!(
                "[crussty-runtime] claim conflict on '{key}': already owned by another module; refusing duplicate"
            );
            -1
        }
        _ => {
            m.insert(key, owner);
            0
        }
    }
}

unsafe extern "C" fn api_retransform_class(name: *const c_char) -> i32 {
    with_attached(|| {
        let Some(env) = jvmti_env() else {
            return -3;
        };
        let Some(nm) = (|| {
        if name.is_null() {
            return None;
        }
        let mut end = 0usize;
        while end < 512 && *name.add(end) != 0 {
            end += 1;
        }
        std::str::from_utf8(std::slice::from_raw_parts(name.cast::<u8>(), end))
            .ok()
            .map(str::to_string)
        })() else {
        return -3;
    };
    let sig = format!("L{};", nm.replace('.', "/"));
    let classes = match env.get_loaded_classes() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[crussty-runtime] retransform {nm}: get_loaded_classes failed: {e:?}");
            return -2;
        }
    };
    let mut target = None;
    for cls in classes {
        if let Ok((n, _)) = env.get_class_signature(cls) {
            if n == sig {
                target = Some(cls);
                break;
            }
        }
    }
    let Some(cls) = target else {
        return -1;
    };
    match env.is_modifiable_class(cls) {
        Ok(m) => eprintln!("[crussty-runtime] retransform {nm}: is_modifiable_class={m}"),
        Err(e) => eprintln!("[crussty-runtime] retransform {nm}: is_modifiable_class err {e:?}"),
    }
    match env.get_class_status(cls) {
        Ok(s) => eprintln!("[crussty-runtime] retransform {nm}: class_status=0x{s:x}"),
        Err(e) => eprintln!("[crussty-runtime] retransform {nm}: class_status err {e:?}"),
    }
    if let Err(e) = env.retransform_classes(&[cls]) {
        let code: i32 = e as i32;
        let name = env.get_error_name(e).unwrap_or_default();
        eprintln!("[crussty-runtime] retransform {nm}: RetransformClasses failed: {e:?} (code {code}, name {name})");
        return -5;
    }
    0
    })
    .unwrap_or(-6)
}

#[derive(Default, Debug)]
struct AgentOptions {
    modules: Option<PathBuf>,
    #[allow(dead_code)]
    versions: Option<PathBuf>,
    #[allow(dead_code)]
    kernel: Option<String>,
    /// Unix socket path for the telemetry channel ("telemetry=/run/crussty.sock")
    telemetry: Option<PathBuf>,
}

/// options format: "modules=<dir>;versions=<dir>;kernel=<jar>"
fn parse_options(options: &str) -> AgentOptions {
    let mut o = AgentOptions::default();
    for part in options.split(';') {
        let Some((k, v)) = part.split_once('=') else {
            continue;
        };
        match k.trim() {
            "modules" => o.modules = Some(PathBuf::from(v.trim())),
            "versions" => o.versions = Some(PathBuf::from(v.trim())),
            "kernel" => o.kernel = Some(v.trim().to_string()),
            "telemetry" => o.telemetry = Some(PathBuf::from(v.trim())),
            _ => {}
        }
    }
    o
}

/// Discover + dlopen + init every module in the modules tree.
fn load_plugins(root: &std::path::Path, vm: JavaVmPtr, options: &str) {
    let api = cplugin_api();
    let c_options = CString::new(options).unwrap_or_default();
    let found = scan::scan(root);
    if found.is_empty() {
        eprintln!("[crussty-runtime] no modules found under {}", root.display());
    }
    for plugin in found {
        // RTLD_LOCAL: one plugin cannot shadow another's symbols (the Java
        // System.load analogue of per-plugin classloaders).
        let lib = match unsafe { Library::new(&plugin.lib_path) } {
            Ok(l) => l,
            Err(e) => {
                eprintln!(
                    "[crussty-runtime] dlopen {} failed: {e}",
                    plugin.lib_path.display()
                );
                continue;
            }
        };
        let init: libloading::Symbol<cplug_abi::CPluginInit> =
            match unsafe { lib.get(b"cplugin_init\0") } {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[crussty-runtime] {} has no cplugin_init export: {e}", plugin.id);
                    continue;
                }
            };
        // The init handshake runs inside a registration window stamped with
        // (plugin id, generation 1): hooks and event subscriptions the
        // module registers become owned by it, so a later hot reload can
        // purge exactly the replaced generation (see purge_module_hooks).
        let _owner = begin_registration(&plugin.id, 1);
        let rc = unsafe { init(api, vm, c_options.as_ptr()) };
        drop(_owner);
        eprintln!("[crussty-runtime] module {} -> init rc={rc}", plugin.id);
        if rc == 0 {
            // Admit the loaded library into the hot-reload registry: the
            // registry owns the mapping (dlclose on replace), keeps it
            // alive, and captures the handshake so the SIGUSR1 trigger (or
            // any API caller) can re-init a fresh build on reload. On
            // failure the registry hands the library back; keep it resident
            // anyway.
            match platform::hot_reload::admit_module(
                &plugin.id,
                plugin.lib_path.clone(),
                lib,
                api as *const cplug_abi::CPluginApi,
                vm,
                options,
            ) {
                Ok(()) => {}
                Err((e, lib)) => {
                    eprintln!(
                        "[crussty-runtime] module {} not admitted to hot reload: {e}",
                        plugin.id
                    );
                    // Keep the module alive and functional regardless.
                    libs().lock().unwrap().push(lib);
                }
            }
        } else {
            // Failed init: keep the library resident (some modules log
            // lazily from their own threads), but no registry entry.
            libs().lock().unwrap().push(lib);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn class_loaded_event_published_only_when_subscribed() {
        let bus = platform::events::global();
        let seen = Arc::new(Mutex::new(Vec::<(String, usize)>::new()));
        let s = Arc::clone(&seen);
        let token = bus.subscribe(
            platform::lifecycle::CLASS_LOADED,
            Arc::new(move |_, payload| {
                s.lock().unwrap().push((
                    payload["name"].as_str().unwrap_or("").to_string(),
                    payload["bytes"].as_u64().unwrap_or(0) as usize,
                ));
            }),
        );
        let count = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&count);
        let token2 = bus.subscribe(platform::lifecycle::CLASS_LOADED, Arc::new(move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
        }));

        publish_class_loaded("a/b/C", 42);
        assert_eq!(*seen.lock().unwrap(), vec![("a/b/C".to_string(), 42)]);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Publishing without subscribers is a no-op: zero-cost on the
        // class-load path once nobody listens.
        bus.unsubscribe(platform::lifecycle::CLASS_LOADED, &token);
        bus.unsubscribe(platform::lifecycle::CLASS_LOADED, &token2);
        publish_class_loaded("x/y", 7);
        assert_eq!(count.load(Ordering::SeqCst), 1, "no subscriber, no dispatch");
        assert_eq!(seen.lock().unwrap().len(), 1);
    }
}

#[cfg(test)]
mod purge_tests {
    use super::*;

    unsafe extern "C" fn dummy_hook(
        _ctx: *mut c_void,
        _name: *const c_char,
        _data: *const u8,
        _len: usize,
        _out: *mut *mut u8,
        _out_len: *mut usize,
    ) -> i32 {
        0
    }

    fn register(ctx: usize) -> Option<(String, u64)> {
        unsafe { api_register_class_hook(ctx as *mut c_void, dummy_hook) };
        registration_owner()
    }

    #[test]
    fn purge_removes_only_the_replaced_generation() {
        // Startup: hello gen 1 registers a hook; dist never does.
        let _g1 = begin_registration("hello", 1);
        assert_eq!(register(0x11), Some(("hello".to_string(), 1)));
        drop(_g1);

        // A hook registered outside any window belongs to nobody: a reload
        // in either generation must never purge it.
        let _: Option<(String, u64)> = register(0x22);
        assert_eq!(hooks().lock().unwrap().len(), 2);

        // Reload: the replacement runs under (hello, 2) and adds its hook.
        let _g2 = begin_registration("hello", 2);
        assert_eq!(register(0x33), Some(("hello".to_string(), 2)));
        drop(_g2);
        assert_eq!(hooks().lock().unwrap().len(), 3);

        // Purging the replaced generation removes exactly gen 1, keeping
        // the unowned hook AND the new generation's hook.
        purge_module_hooks("hello", 1);
        let h = hooks().lock().unwrap();
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].ctx, 0x22, "unowned hook survives");
        assert_eq!(h[0].owner, None);
        assert_eq!(h[1].owner, Some(("hello".to_string(), 2)));
        assert_eq!(h[1].ctx, 0x33);
        drop(h);

        // Purging a different module touches nothing.
        purge_module_hooks("world", 1);
        assert_eq!(hooks().lock().unwrap().len(), 2);

        // Purging the new generation leaves only the unowned hook.
        purge_module_hooks("hello", 2);
        let h = hooks().lock().unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].owner, None);
        assert_eq!(h[0].ctx, 0x22);
    }
}

#[cfg(test)]
mod claim_tests {
    use super::*;

    fn key(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    fn claim(owner: usize, k: &str) -> i32 {
        let k = key(k);
        unsafe { api_claim(owner, k.as_ptr() as *const c_char) }
    }

    #[test]
    fn claim_first_wins_second_owner_rejected() {
        assert_eq!(claim(0x11, "class:a/b/C"), 0);
        assert_eq!(claim(0x11, "class:a/b/C"), 0); // idempotent, same owner
        assert_eq!(claim(0x22, "class:a/b/C"), -1); // other module: refused
        assert_eq!(claim(0x22, "class:a/b/D"), 0); // different key: fine
    }

    #[test]
    fn registration_window_is_thread_local() {
        // TASK-168 regression: a handshake window on thread A must never
        // stamp registrations (or owner queries) made on thread B. The
        // pre-TASK-168 global REG_CTX leaked the window across threads and
        // broke the TASK-167 concurrent memo test in CI.
        assert_eq!(registration_owner(), None, "clean thread starts unowned");
        let _g = begin_registration("tlocal", 3);
        assert_eq!(registration_owner(), Some(("tlocal".to_string(), 3)));
        let other = std::thread::spawn(|| {
            assert_eq!(
                registration_owner(),
                None,
                "another thread's window must be invisible here"
            );
            // Subscriptions made on this thread stay unowned even while the
            // first thread's window is open: an owner purge for that window
            // must not touch them.
            let bus = platform::events::EventBus::default();
            bus.subscribe("tlocal.evt", std::sync::Arc::new(|_, _| {}));
            assert_eq!(
                bus.purge_owner(&("tlocal".to_string(), 3)),
                0,
                "subscription must be unowned: another thread's window is invisible"
            );
            assert_eq!(bus.publish("tlocal.evt", &serde_json::json!(null)), 1);
        });
        other.join().unwrap();
        assert_eq!(
            registration_owner(),
            Some(("tlocal".to_string(), 3)),
            "window intact on the owning thread"
        );
        drop(_g);
        assert_eq!(registration_owner(), None, "guard drop restores None");
    }
}

/// Round-4 hotpath benches (TASK-163): the per-class-load loop is the JVM's
/// hottest path — every class the kernel ever loads pays this callback.
#[cfg(test)]
mod bench_hotpath {
    //! Release-only A/B benches: `cargo test --release -- --ignored --nocapture bench_cflh`.
    use super::*;
    use std::time::Instant;

    /// A representative mid-size class: ~300 constant-pool entries, a
    /// 40-char internal name — the shape of a typical kernel class.
    fn representative_class() -> Vec<u8> {
        let mut b = Vec::with_capacity(8192);
        b.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // magic
        b.extend_from_slice(&[0, 0]); // minor
        b.extend_from_slice(&[0, 65]); // major 65
        let slots = 300u16;
        b.extend_from_slice(&slots.to_be_bytes()); // cp_count
        let mut i = 1usize;
        while i < (slots as usize) - 1 {
            // Utf8 filler entry (varied lengths 8..40)
            let len = 8 + (i % 32);
            b.push(1);
            b.extend_from_slice(&(len as u16).to_be_bytes());
            b.extend(std::iter::repeat(b'x').take(len));
            i += 1;
        }
        // last slot: the name
        b.push(1);
        b.extend_from_slice(&(40u16).to_be_bytes());
        b.extend_from_slice(b"java/util/concurrent/ConcurrentHashMap");
        b.push(7);
        b.extend_from_slice(&((slots - 1)).to_be_bytes());
        b.extend_from_slice(&[0, 0x21]);
        b.extend_from_slice(&(slots - 1).to_be_bytes());
        b.extend_from_slice(&[0, 0]);
        b
    }

    #[test]
    #[ignore]
    fn bench_cflh_class_load_overhead() {
        let data = representative_class();
        let iters = 200_000u32;
        let rounds = 5;

        // segment 1: name derivation from class bytes
        for _ in 0..10_000u32 {
            let _ = class_file_name(&data);
        }
        let mut best_name = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters {
                let _ = class_file_name(&data);
            }
            best_name = best_name.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }

        // segment 2: the class-load fast-gate chain (TASK-163) — plugin
        // hook count + engine rule count + lifecycle subscriber check, the
        // shape every class load pays before any work happens.
        let gate = || {
            let hooks_live = PLUGIN_HOOKS_LIVE.load(Ordering::Acquire) > 0;
            !hooks_live
                && platform::transform::global_engine().rule_count() == 0
                && !platform::events::global_ref()
                    .has_subscribers(platform::lifecycle::CLASS_LOADED)
        };
        // bisection: per-component attribution
        let (mut b1, mut b2, mut b3) = (f64::MAX, f64::MAX, f64::MAX);
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters {
                std::hint::black_box(PLUGIN_HOOKS_LIVE.load(Ordering::Acquire));
            }
            b1 = b1.min(t.elapsed().as_secs_f64() / f64::from(iters));
            let t = Instant::now();
            for _ in 0..iters {
                std::hint::black_box(platform::transform::global_engine().rule_count());
            }
            b2 = b2.min(t.elapsed().as_secs_f64() / f64::from(iters));
            let t = Instant::now();
            for _ in 0..iters {
                std::hint::black_box(
                    platform::events::global_ref()
                        .has_subscribers(platform::lifecycle::CLASS_LOADED),
                );
            }
            b3 = b3.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        for _ in 0..10_000u32 {
            let _ = gate();
        }
        let mut best_prep = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters {
                let _ = gate();
            }
            best_prep = best_prep.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH cflh: class_file_name {:.0} ns/op, fast-gate {:.0} ns/op (hooks {:.0} / rules {:.0} / subs {:.0}) (min of {rounds}x{iters})",
            best_name * 1e9,
            best_prep * 1e9,
            b1 * 1e9,
            b2 * 1e9,
            b3 * 1e9
        );
    }

    #[test]
    #[ignore]
    fn bench_registration_owner() {
        let iters = 200_000u32;
        let rounds = 5;
        for _ in 0..10_000u32 {
            let _ = registration_owner();
        }
        let mut best = f64::MAX;
        for _ in 0..rounds {
            let t = Instant::now();
            for _ in 0..iters {
                let _ = registration_owner();
            }
            best = best.min(t.elapsed().as_secs_f64() / f64::from(iters));
        }
        println!(
            "BENCH registration_owner: {:.0} ns/op (min of {rounds}x{iters})",
            best * 1e9
        );
    }
}
