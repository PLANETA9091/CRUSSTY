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

mod scan;

use std::ffi::{c_char, c_void, CString};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use cplug_abi::{CPluginApi, ClassHookFn, JavaVmPtr, CPAPI_VERSION};
use jvmti_bindings::prelude::*;
#[allow(unused_imports)]
use jvmti_bindings::export_agent as export_runtime;
use libloading::Library;

/// (ctx, hook) pairs registered by plugins, in registration order.
static HOOKS: OnceLock<Mutex<Vec<(usize, ClassHookFn)>>> = OnceLock::new();
/// Raw jvmtiEnv pointer as usize (JVMTI envs are process-wide, usable from
/// any thread — safe to share).
static JVMTI_ENV: OnceLock<usize> = OnceLock::new();
/// Raw JavaVM* as usize, for attaching plugin threads (JVMTI calls like
/// GetLoadedClasses/RetransformClasses need an attached thread).
static VM: OnceLock<usize> = OnceLock::new();
/// Loaded plugin libraries, kept alive for the whole JVM lifetime.
static LIBS: OnceLock<Mutex<Vec<Library>>> = OnceLock::new();

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

impl Agent for CrusstyRuntime {
    fn on_load(&self, vm: *mut jni::JavaVM, options: &str) -> jni::jint {
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

        jni::JNI_OK
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
        if registered.is_empty() {
            return;
        }

        let mut current: *const u8 = class_data;
        let mut current_len = class_data_len as usize;
        // Holds the chained replacement bytes; kept alive by binding until the
        // end of the hook, then freed naturally.
        let mut pending: Option<Vec<u8>> = None;
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
            _ => {}
        }
    }
    o
}

/// Discover + dlopen + init every module in the modules tree.
fn load_plugins(root: &std::path::Path, vm: JavaVmPtr, options: &str) {
    let api = CPluginApi {
        version: CPAPI_VERSION,
        register_class_hook: Some(api_register_class_hook),
        jvmti_allocate: Some(api_jvmti_allocate),
        retransform_class: Some(api_retransform_class),
    };
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
        let rc = unsafe { init(&api, vm, c_options.as_ptr()) };
        eprintln!("[crussty-runtime] plugin {} -> init rc={rc}", plugin.id);
        libs().lock().unwrap().push(lib);
    }
}
