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

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CString};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use cplug_abi::{CPluginApi, ClassHookFn, JavaVmPtr, CPAPI_VERSION};
use jvmti_bindings::prelude::*;
#[allow(unused_imports)]
use jvmti_bindings::export_agent as export_runtime;
use libloading::Library;

/// (ctx, hook) pairs registered by plugins, in registration order.
static HOOKS: OnceLock<Mutex<Vec<(usize, ClassHookFn)>>> = OnceLock::new();
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

/// Number of plugin hooks registered so far — the baseline a hot reload
/// truncates to before it unloads the old library (see
/// [`truncate_hooks_to`]; both live here next to the registry they guard).
pub fn hook_registration_seq() -> usize {
    hooks().lock().unwrap().len()
}

/// Drop every hook registered at/after `seq` (a module's `cplugin_init` ran
/// between `seq` and the call). dlclose leaves hooks pointing into unmapped
/// code otherwise: the next ClassFileLoadHook would SIGSEGV on a stale
/// callback. Only the module under reload is affected: its hooks were the
/// ones appended since `seq`.
pub fn truncate_hooks_to(seq: usize) {
    let mut h = hooks().lock().unwrap();
    if h.len() > seq {
        eprintln!(
            "[crussty-runtime] hook purge: dropped {} stale hook(s) from unloaded module",
            h.len() - seq
        );
        h.truncate(seq);
    }
}
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

fn hooks() -> &'static Mutex<Vec<(usize, ClassHookFn)>> {
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
            "[crussty-runtime] pipeline ready: {} plugin hook(s)",
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
        let name = if name.is_null() {
            "<unknown>".to_string()
        } else {
            // The VM's name buffer is not guaranteed NUL-terminated right
            // after the name (it can be reused with leftovers) — bound the
            // read to 128 bytes.
            let mut end = 0usize;
            unsafe {
                while end < 128 && *name.add(end) != 0 {
                    end += 1;
                }
                std::str::from_utf8(std::slice::from_raw_parts(name.cast::<u8>(), end))
                    .unwrap_or("<bad-utf8>")
                    .to_string()
            }
        };
        let registered = hooks().lock().unwrap().clone();

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
        //    output (the engine's included).
        for (ctx, f) in &registered {
            let mut out: *mut u8 = std::ptr::null_mut();
            let mut out_len: usize = 0;
            let rc = unsafe {
                f(
                    *ctx as *mut c_void,
                    name.as_ptr() as *const c_char,
                    current,
                    current_len,
                    &mut out,
                    &mut out_len,
                )
            };
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

/// Publish the `platform.class_loaded` lifecycle event for a class load.
/// Zero-weight on the class-load path: the payload is only built when the
/// bus actually has subscribers (the hook runs on the class-loading thread,
/// which must stay cheap).
fn publish_class_loaded(name: &str, bytes_len: usize) {
    let bus = platform::events::global();
    if bus.has_subscribers(platform::lifecycle::CLASS_LOADED) {
        bus.publish(
            platform::lifecycle::CLASS_LOADED,
            &serde_json::json!({ "name": name, "bytes": bytes_len }),
        );
    }
}

/// Trampolines handed to plugins through CPluginApi.
unsafe extern "C" fn api_register_class_hook(ctx: *mut c_void, hook: ClassHookFn) -> i32 {
    hooks().lock().unwrap().push((ctx as usize, hook));
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
        eprintln!("[crussty-runtime] no plugins found under {}", root.display());
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
        let rc = unsafe { init(api, vm, c_options.as_ptr()) };
        eprintln!("[crussty-runtime] plugin {} -> init rc={rc}", plugin.id);
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
                        "[crussty-runtime] plugin {} not admitted to hot reload: {e}",
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
}
