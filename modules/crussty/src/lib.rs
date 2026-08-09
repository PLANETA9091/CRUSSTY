//! crussty — the Crussty CE native surface as a c-plugin.
//!
//! The Crussty CE project ships 4 Rust crates exporting 283 `Java_*` JNI
//! symbols (waypoint/ore/jigsaw/ticket/area_map/noise/chunk-encode/...). Its
//! Java bridge classes are NOT part of the CE repo — so this plugin rebuilds
//! them: for every export we define a `public static native` bridge class in
//! the exact package the export name implies (bootstrap loader) and register
//! the resolved symbol via RegisterNatives. Result: the whole native surface
//! is callable from ANY Paper-family kernel without the fork.
//!
//! Timing: cplugin_init may not touch the JVM, so injection happens on a
//! background thread 3s after init (VM is up by then; define_class with a
//! null loader + RegisterNatives need no kernel classes).
//!
//! The kernel hot-path wirings (area_map update batching etc.) are separate
//! byte hooks on top of this surface — see the project docs.

mod area_map;
mod bridge_class;
mod classfile;
mod improved_noise;
mod jni_table;
mod loader;

use cplug_abi::{CPluginApi, JavaVmPtr};
use jvmti_bindings::prelude::*;
use std::collections::HashMap;
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::ffi::{c_char, c_void, CString};
use std::path::PathBuf;
use std::time::Duration;

const MAIN_LIB: &str = "paper_native_jni";
const CHUNK_LIB: &str = "paper_native_chunk_encode_jni";

/// Bundled native library filename for this platform: Crussty CE ships
/// `libpaper_native_jni.so` on Linux; Windows builds produce
/// `paper_native_jni.dll` (no `lib` prefix on MSVC).
fn native_lib_name(base: &str) -> String {
    format!("{}{}{}", DLL_PREFIX, base, DLL_SUFFIX)
}

/// The single required export (cplug-abi contract).
///
/// # Safety
/// `api` must point at a valid CPluginApi owned by the agent, `vm` must be
/// the live JavaVM pointer handed to us, `options` a NUL-terminated string
/// owned by the agent for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    _options: *const c_char,
) -> i32 {
    cplug_sdk::init(api, vm);
    eprintln!("[crussty-module] cplugin_init: injecting Crussty CE native surface in background");
    area_map::register();
    improved_noise::register();
    std::thread::spawn(inject_surface);
    0
}

/// Locate this plugin's directory via dladdr on our own entry symbol.
fn plugin_dir() -> Option<PathBuf> {
    #[repr(C)]
    struct DlInfo {
        dli_fname: *const c_char,
        dli_fbase: *mut c_void,
        dli_sname: *const c_char,
        dli_saddr: *mut c_void,
    }
    unsafe extern "C" {
        fn dladdr(addr: *const c_void, info: *mut DlInfo) -> i32;
    }
    let mut info = DlInfo {
        dli_fname: std::ptr::null(),
        dli_fbase: std::ptr::null_mut(),
        dli_sname: std::ptr::null(),
        dli_saddr: std::ptr::null_mut(),
    };
    let rc = unsafe { dladdr(cplugin_init as *const c_void, &mut info) };
    if rc == 0 || info.dli_fname.is_null() {
        return None;
    }
    let path = unsafe { std::ffi::CStr::from_ptr(info.dli_fname) }
        .to_str()
        .ok()?;
    Some(PathBuf::from(path).parent()?.to_path_buf())
}

/// Background injection worker: dlopen the bundled .so files, define every
/// bridge class, RegisterNatives every export, then fire one live proof call.
fn inject_surface() {
    std::thread::sleep(Duration::from_secs(3));
    let Some(dir) = plugin_dir() else {
        eprintln!("[crussty-module] cannot locate plugin dir (dladdr failed)");
        return;
    };

    // Bundled native libs live in <plugin>/native/ (a subdir without
    // cplugin.json so the agent's plugin scan skips them).
    let native_dir = dir.join("native");
    let main_name = native_lib_name(MAIN_LIB);
    let chunk_name = native_lib_name(CHUNK_LIB);
    let main_so = if native_dir.join(&main_name).exists() {
        native_dir.join(&main_name)
    } else {
        dir.join(&main_name)
    };
    let chunk_so = if native_dir.join(&chunk_name).exists() {
        native_dir.join(&chunk_name)
    } else {
        dir.join(&chunk_name)
    };
    if !main_so.exists() {
        eprintln!(
            "[crussty-module] missing {}: looked in {} and {} (the published libs ship in modules/crussty/native/ — see README)",
            native_lib_name(MAIN_LIB),
            native_dir.join(&main_name).display(),
            dir.join(&main_name).display()
        );
        return;
    }
    let main = match unsafe { loader::NativeLib::new(&main_so) } {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[crussty-module] dlopen {} failed: {e}", native_lib_name(MAIN_LIB));
            return;
        }
    };
    let chunk = if chunk_so.exists() {
        match unsafe { loader::NativeLib::new(&chunk_so) } {
            Ok(l) => Some(l),
            Err(e) => {
                eprintln!("[crussty-module] dlopen {} failed (continuing without it): {e}", native_lib_name(CHUNK_LIB));
                None
            }
        }
    } else {
        None
    };

    eprintln!("[crussty-module] native libs: {main_so:?} (+ {chunk_so:?})");

    let mut n_classes = 0usize;
    let mut n_natives = 0usize;
    let mut n_missing = 0usize;
    let result = with_attached(|env| {
        eprintln!("[crussty-module] attached: injecting bridge classes");
        for (table, lib) in [
            (jni_table::MAIN_JNI_TABLE, Some(&main)),
            (jni_table::CHUNK_JNI_TABLE, chunk.as_ref()),
        ] {
            let Some(lib) = lib else { continue };
            let mut by_class: HashMap<&str, Vec<(&str, &str, &str)>> = HashMap::new();
            for e in table {
                by_class
                    .entry(e.class)
                    .or_default()
                    .push((e.method, e.sig, e.symbol));
            }
            for class in jni_table::MAIN_BRIDGE_CLASSES
                .iter()
                .chain(jni_table::CHUNK_BRIDGE_CLASSES.iter())
            {
                let Some(methods) = by_class.get(class) else {
                    continue;
                };
                match define_and_register(env, lib, class, methods) {
                    Ok((c, n, m)) => {
                        n_classes += c;
                        n_natives += n;
                        n_missing += m;
                    }
                    Err(e) => eprintln!("[crussty-module] {class}: {e}"),
                }
            }
        }
        eprintln!("[crussty-module] injection loop done");
        Some(())
    });

    if result.is_none() {
        eprintln!("[crussty-module] injection aborted: no JNI env (VM not ready?)");
        return;
    }

    eprintln!(
        "[crussty-module] native surface live: {n_classes} bridge classes, {n_natives} natives registered ({} symbols unresolved)",
        n_missing
    );

    with_attached(|env| {
        live_proof(env);
        Some(())
    });

    area_map::activate();
    improved_noise::activate();
}

/// Define one bridge class and register all its natives.
/// Returns (classes, natives, missing symbols).
fn define_and_register(
    env: &JniEnv,
    lib: &loader::NativeLib,
    class: &str,
    methods: &[(&str, &str, &str)],
) -> Result<(usize, usize, usize), String> {
    // Conflict guard: never define a bridge class another module owns.
    if !cplug_sdk::claim(&format!("class:{class}")) {
        eprintln!("[crussty-module] class '{class}' claimed by another module; skipping");
        return Err(format!("class {class} already claimed"));
    }
    let pairs: Vec<(&str, &str)> = methods.iter().map(|(m, s, _)| (*m, *s)).collect();
    let bytes = bridge_class::bridge_class_bytes(class, &pairs);
    let Some(cls) = env.define_class(class, std::ptr::null_mut(), &bytes) else {
        clear_exception(env);
        return Err("define_class failed".into());
    };

    let mut names: Vec<CString> = Vec::with_capacity(methods.len());
    let mut sigs: Vec<CString> = Vec::with_capacity(methods.len());
    let mut natives: Vec<jni::JNINativeMethod> = Vec::with_capacity(methods.len());
    let mut missing = 0usize;
    for (m, s, sym) in methods {
        // Conflict guard: skip natives another module already registered
        // under the same (class, name, signature) — registering twice would
        // silently repoint the existing native.
        if !cplug_sdk::claim(&format!("native:{class}#{m}:{s}")) {
            eprintln!("[crussty-module] native {class}#{m}{s} claimed by another module; skipping");
            missing += 1;
            continue;
        }
        let Some(ptr) = lib.symbol(sym) else {
            missing += 1;
            continue;
        };
        names.push(CString::new(*m).map_err(|_| "name has NUL".to_string())?);
        sigs.push(CString::new(*s).map_err(|_| "sig has NUL".to_string())?);
        let name = names.last().unwrap().as_ptr();
        let sig = sigs.last().unwrap().as_ptr();
        natives.push(jni::JNINativeMethod {
            name,
            signature: sig,
            fnPtr: ptr,
        });
    }
    let reg = if natives.is_empty() {
        Ok(())
    } else {
        env.register_natives(cls, &natives)
    };
    env.delete_local_ref(cls);
    if let Err(code) = reg {
        clear_exception(env);
        return Err(format!("register_natives failed (code {code})"));
    }
    Ok((1, natives.len(), missing))
}

/// Live calls through the injected bridge, chosen for determinism:
///  1. PaperNativeNormalNoise.nativeCheck() -> jboolean, always true —
///     proves the bridge class defines + registers correctly and the symbol
///     resolves + executes.
///  2. PaperNativeTicketSetSearch.binarySummary(iter, dst) -> jint — a real
///     benchmark kernel that writes SUMMARY_FIELDS longs into an array
///     (proves array passthrough on the same class).
fn live_proof(env: &JniEnv) {
    // Full internal name — this bridge class lives in net.minecraft.* (the
    // JNI export Java_net_minecraft_..._PaperNativeNormalNoise_* implies it).
    if let Some(cls) =
        env.find_class("net/minecraft/world/level/levelgen/synth/PaperNativeNormalNoise")
    {
        if let Some(mid) = env.get_static_method_id(cls, "nativeCheck", "()Z") {
            let ok = env.call_static_int_method(cls, mid, &[]);
            let _ = clear_exception(env);
            eprintln!("[crussty-module] live proof: normalNoise.nativeCheck() = {ok}");
        } else {
            let _ = clear_exception(env);
            eprintln!("[crussty-module] live proof: nativeCheck unresolved");
        }
        env.delete_local_ref(cls);
    } else {
        let _ = clear_exception(env);
        eprintln!("[crussty-module] live proof: find_class(PaperNativeNormalNoise) failed");
    }

    let Some(cls) = env.find_class("PaperNativeTicketSetSearch") else {
        let _ = clear_exception(env);
        eprintln!("[crussty-module] live proof: find_class(PaperNativeTicketSetSearch) failed");
        return;
    };
    let Some(mid) = env.get_static_method_id(cls, "binarySummary", "(I[J)I") else {
        let _ = clear_exception(env);
        eprintln!("[crussty-module] live proof: get_static_method_id failed");
        env.delete_local_ref(cls);
        return;
    };
    let Some(arr) = env.new_long_array(1) else {
        eprintln!("[crussty-module] live proof: new_long_array failed");
        env.delete_local_ref(cls);
        return;
    };
    let written =
        env.call_static_int_method(cls, mid, &[jni::jvalue { i: 1000 }, jni::jvalue { l: arr }]);
    let _ = clear_exception(env);
    env.delete_local_ref(arr);
    env.delete_local_ref(cls);
    eprintln!("[crussty-module] live proof: ticketset binarySummary(1000) wrote {written} long(s)");
}

/// Attach the current thread if needed, run `f` with a JNI env, detach only
/// if we attached. Standard Oracle-JNI GetEnv-first idiom.
fn with_attached<R>(f: impl FnOnce(&JniEnv) -> R) -> Option<R> {
    unsafe {
        let raw_vm = cplug_sdk::vm();
        let vm = raw_vm as *mut jni::JavaVM;
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
            return Some(f(&JniEnv::from_raw(env_ptr)));
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
        let env = JniEnv::from_raw(env_ptr);
        let out = f(&env);
        ((**vm).DetachCurrentThread)(vm);
        Some(out)
    }
}

/// Clear a pending exception silently. Transient lookups (find_class before
/// the kernel is up, getLogger before Bukkit.server is set) throw routinely;
/// exception_describe would print scary "Exception in thread" traces to the
/// server log for a condition we fully expect.
fn clear_exception(env: &JniEnv) -> bool {
    if env.exception_check() {
        env.exception_clear();
        true
    } else {
        false
    }
}
