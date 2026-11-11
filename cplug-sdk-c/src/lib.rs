//! cplug-sdk-c — C ABI binding of cplug-sdk.
//!
//! Lets modules written in C, C++, Python (ctypes), JavaScript (QuickJS
//! shim), Zig, Go etc. use the same convenience layer as Rust modules:
//! pattern hooks, byte hooks, cross-loader class resolution, main-thread
//! dispatch, kernel-ready notifications and logging — without hand-rolling
//! the heavy JNI/JVMTI patterns the SDK implements.
//!
//! Build: `staticlib`; modules link it into their own `.so` (or a C shim
//! dlopens it). Contract mirrors `include/cplug-sdk.h` — keep in lockstep.

#![allow(non_camel_case_types)]

use cplug_abi::{CPluginApi, JavaVmPtr};
use jvmti_bindings::prelude::*;
use std::ffi::{c_char, c_void, CString};
use std::ptr;

pub type cplug_hook_fn = unsafe extern "C" fn(ctx: *mut c_void, name: *const c_char);
pub type cplug_byte_hook_fn = unsafe extern "C" fn(
    ctx: *mut c_void,
    name: *const c_char,
    data: *const u8,
    len: usize,
    out_len: *mut usize,
) -> *const u8;
pub type cplug_ready_fn = unsafe extern "C" fn(ctx: *mut c_void);
pub type cplug_main_fn = unsafe extern "C" fn(ctx: *mut c_void, env: *mut c_void);

/// Raw pointers are not Send/Sync by default; JVMTI hands these callbacks to
/// class-load threads, so we promise they are safe to share.
struct CbBox {
    cb: cplug_hook_fn,
    ctx: *mut c_void,
}
unsafe impl Send for CbBox {}
unsafe impl Sync for CbBox {}

struct ByteCbBox {
    cb: cplug_byte_hook_fn,
    ctx: *mut c_void,
}
unsafe impl Send for ByteCbBox {}
unsafe impl Sync for ByteCbBox {}

struct ReadyCbBox {
    cb: cplug_ready_fn,
    ctx: *mut c_void,
}
unsafe impl Send for ReadyCbBox {}

struct MainCbBox {
    cb: cplug_main_fn,
    ctx: *mut c_void,
}
unsafe impl Send for MainCbBox {}

fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    cplug_sdk::jni_util::cstr(p)
}

/// Must be called first, from `cplugin_init`: stores the vm, registers the
/// SDK's single pipeline hook. Equivalent to `cplug_sdk::init`.
///
/// # Safety
/// `api` must be the CPluginApi from the runtime, `vm` the live JavaVM*.
#[no_mangle]
pub unsafe extern "C" fn cplug_sdk_init(api: *const CPluginApi, vm: JavaVmPtr) {
    unsafe { cplug_sdk::init(api, vm) };
}

/// Raw JavaVM* stored by `cplug_sdk_init` (for JNI bindings that need it).
#[no_mangle]
pub extern "C" fn cplug_sdk_vm() -> JavaVmPtr {
    cplug_sdk::vm()
}

/// Register a name-only pattern hook: `cb(ctx, name)` fires on every class
/// load matching `pattern`. Glob syntax like the SDK: `*` any run (incl. '/'),
/// `?` one char.
///
/// # Safety
/// `pattern` must be a valid C string; `cb` must remain valid for the whole
/// module lifetime (hooks are never unregistered).
#[no_mangle]
pub unsafe extern "C" fn cplug_sdk_hook_register(
    pattern: *const c_char,
    ctx: *mut c_void,
    cb: Option<cplug_hook_fn>,
) -> i32 {
    let Some(pattern) = cstr(pattern) else {
        return -1;
    };
    let Some(cb) = cb else { return -2 };
    let boxed = Box::new(CbBox { cb, ctx });
    cplug_sdk::hooks::register(pattern, move |name| {
        let Ok(cname) = CString::new(name) else {
            return;
        };
        unsafe { (boxed.cb)(boxed.ctx, cname.as_ptr()) };
    });
    0
}

/// Register a byte-level hook: `cb(ctx, name, data, len, &out_len)` fires on
/// every class load matching `pattern`; return a pointer to replacement
/// class bytes (length in `*out_len`), or NULL to keep the originals. The
/// returned buffer is copied by the SDK and then freed with `free(3)` — it
/// must be heap-allocated by the module (e.g. `malloc`).
///
/// # Safety
/// `pattern` must be a valid C string; `cb` must remain valid for the whole
/// module lifetime.
#[no_mangle]
pub unsafe extern "C" fn cplug_sdk_hook_register_bytes(
    pattern: *const c_char,
    ctx: *mut c_void,
    cb: Option<cplug_byte_hook_fn>,
) -> i32 {
    let Some(pattern) = cstr(pattern) else {
        return -1;
    };
    let Some(cb) = cb else { return -2 };
    let boxed = Box::new(ByteCbBox { cb, ctx });
    cplug_sdk::hooks::register_bytes(pattern, move |name, data| {
        let Ok(cname) = CString::new(name) else {
            return None;
        };
        let mut out_len: usize = 0;
        let out = unsafe {
            (boxed.cb)(
                boxed.ctx,
                cname.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut out_len,
            )
        };
        if out.is_null() {
            return None;
        }
        let bytes = unsafe { std::slice::from_raw_parts(out, out_len) }.to_vec();
        unsafe { libc::free(out as *mut c_void) };
        Some(bytes)
    });
    0
}

/// Run `cb(ctx)` on a fresh thread exactly once, when `class_name` has loaded
/// (kernel-ready notification). Safe to call from `cplugin_init`.
///
/// # Safety
/// `class_name` must be a valid C string; `cb` stays valid for the module
/// lifetime.
#[no_mangle]
pub unsafe extern "C" fn cplug_sdk_on_kernel_ready(
    class_name: *const c_char,
    ctx: *mut c_void,
    cb: Option<cplug_ready_fn>,
) -> i32 {
    let Some(class_name) = cstr(class_name) else {
        return -1;
    };
    let Some(cb) = cb else { return -2 };
    let boxed = Box::new(ReadyCbBox { cb, ctx });
    cplug_sdk::hooks::on_kernel_ready(class_name, move || {
        unsafe { (boxed.cb)(boxed.ctx) };
    });
    0
}

/// Queue `cb(ctx, env)` to run on the server's main thread, with an attached
/// JNI env. Safe from any thread, any time (jobs wait while the kernel
/// boots). `env` is a raw JNIEnv* valid only for the callback duration.
///
/// # Safety
/// `cb` must remain valid for the whole module lifetime.
#[no_mangle]
pub unsafe extern "C" fn cplug_sdk_run_on_main_thread(
    ctx: *mut c_void,
    cb: Option<cplug_main_fn>,
) -> i32 {
    let Some(cb) = cb else { return -2 };
    let boxed = Box::new(MainCbBox { cb, ctx });
    cplug_sdk::main_thread::run_on_main_thread(move |env| {
        unsafe { (boxed.cb)(boxed.ctx, env.raw() as *mut c_void) };
    });
    0
}

/// Resolve a loaded class by name ("org/bukkit/Bukkit" or dotted), across all
/// class loaders. Returns a JNI global ref (process-lifetime, cached) or
/// NULL if not loaded yet. Must be called from an attached thread (e.g.
/// inside a main-thread callback).
#[no_mangle]
pub extern "C" fn cplug_sdk_find_class(name: *const c_char) -> *mut c_void {
    let Some(name) = cstr(name) else {
        return ptr::null_mut();
    };
    match cplug_sdk::classes::find_class(name) {
        Some(c) => c.as_jclass() as *mut c_void,
        None => ptr::null_mut(),
    }
}

/// Like `cplug_sdk_find_class`, but polls every 200 ms until the class loads
/// or `timeout_ms` elapses.
#[no_mangle]
pub extern "C" fn cplug_sdk_wait_class(name: *const c_char, timeout_ms: u64) -> *mut c_void {
    let Some(name) = cstr(name) else {
        return ptr::null_mut();
    };
    match cplug_sdk::classes::wait_class(name, timeout_ms) {
        Some(c) => c.as_jclass() as *mut c_void,
        None => ptr::null_mut(),
    }
}

/// Re-run the class-file hook chain for an already-loaded class
/// (JVMTI retransform). 1 on success, 0 on failure.
#[no_mangle]
pub extern "C" fn cplug_sdk_retransform_class(name: *const c_char) -> i32 {
    let Some(name) = cstr(name) else {
        return 0;
    };
    cplug_sdk::classes::retransform(name) as i32
}

/// Log an info message through the kernel logger (stderr until the kernel
/// is up).
#[no_mangle]
pub extern "C" fn cplug_sdk_log_info(msg: *const c_char) {
    if let Some(msg) = cstr(msg) {
        cplug_sdk::log::info(msg);
    }
}

/// Log a warning through the kernel logger.
#[no_mangle]
pub extern "C" fn cplug_sdk_log_warn(msg: *const c_char) {
    if let Some(msg) = cstr(msg) {
        cplug_sdk::log::warn(msg);
    }
}

/// Clear a pending JNI exception on the current thread (describe + clear).
/// 1 if an exception was pending.
#[no_mangle]
pub extern "C" fn cplug_sdk_clear_exception(env: *mut c_void) -> i32 {
    if env.is_null() {
        return 0;
    }
    unsafe { cplug_sdk::jni_util::clear_exception(&JniEnv::from_raw(env as *mut jni::JNIEnv)) as i32 }
}

/// Attach the current thread if needed and return a JNIEnv* (NULL on
/// failure). The thread stays attached; detach with
/// `cplug_sdk_detach_current_thread` only if YOU attached it.
#[no_mangle]
pub extern "C" fn cplug_sdk_attach_current_thread() -> *mut c_void {
    let vm = cplug_sdk::vm() as *mut jni::JavaVM;
    unsafe {
        if vm.is_null() || (*vm).is_null() {
            return ptr::null_mut();
        }
        let mut env_ptr: *mut jni::JNIEnv = ptr::null_mut();
        let rc = ((**vm).GetEnv)(
            vm,
            &mut env_ptr as *mut *mut jni::JNIEnv as *mut *mut c_void,
            jni::JNI_VERSION_1_6,
        );
        if rc == jni::JNI_OK && !env_ptr.is_null() {
            return env_ptr as *mut c_void;
        }
        if rc != jni::JNI_EDETACHED {
            return ptr::null_mut();
        }
        let rc = ((**vm).AttachCurrentThread)(
            vm,
            &mut env_ptr as *mut *mut jni::JNIEnv as *mut *mut c_void,
            ptr::null_mut(),
        );
        if rc != jni::JNI_OK {
            return ptr::null_mut();
        }
        env_ptr as *mut c_void
    }
}

/// Detach the current thread (only if it was attached by
/// `cplug_sdk_attach_current_thread`).
#[no_mangle]
pub extern "C" fn cplug_sdk_detach_current_thread() {
    let vm = cplug_sdk::vm() as *mut jni::JavaVM;
    unsafe {
        if !vm.is_null() && !(*vm).is_null() {
            ((**vm).DetachCurrentThread)(vm);
        }
    }
}
