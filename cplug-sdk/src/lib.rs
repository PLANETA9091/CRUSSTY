//! cplug-sdk — convenience layer for module authors.
//!
//! Philosophy: convenience, NOT restriction. Every helper here implements a
//! "super heavy" JNI/JVMTI pattern once (class resolution across loaders,
//! main-thread execution, kernel logging, bytecode weaving) so plugins don't
//! hand-roll them. Raw JNI/JVMTI stays fully available — the SDK never holds
//! exclusive capabilities, it is a pure Rust library on top of cplug-abi.
//!
//! Usage (from cplugin_init):
//! ```ignore
//! cplug_sdk::init(api, vm);
//! cplug_sdk::on_kernel_ready("org/bukkit/Bukkit", || {
//!     cplug_sdk::run_on_main_thread(|_env| {
//!         cplug_sdk::log::info("kernel is up, we are on the main thread");
//!     });
//! });
//! ```
//!
//! Everything is optional: a plugin may ignore the SDK entirely.

pub mod asm;
pub mod classes;
pub mod hooks;
pub mod log;
pub mod main_thread;
pub mod weave;

mod sdk_glob;
pub mod jni_util;

pub use hooks::on_kernel_ready;
pub use log::log_info;
pub use main_thread::run_on_main_thread;

use cplug_abi::{CPluginApi, JavaVmPtr};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The JavaVM* passed to cplugin_init (raw).
static VM: AtomicUsize = AtomicUsize::new(0);

/// The runtime's jvmti_allocate fn pointer (for handing replacement class bytes
/// to the JVM — freed by the runtime with JVMTI Deallocate).
static ALLOC: AtomicUsize = AtomicUsize::new(0);

/// Agent's retransform_class fn pointer (retroactive class patching).
static RETRANSFORM: AtomicUsize = AtomicUsize::new(0);

/// Retransform a loaded class by internal name ("a/b/C"). Re-enters the agent
/// pipeline: registered byte hooks see the class again and may patch it.
/// Returns 0 on success, negative on failure.
pub fn retransform_class(name: &str) -> i32 {
    let addr = RETRANSFORM.load(Ordering::Relaxed);
    if addr == 0 {
        return -4;
    }
    let f: unsafe extern "C" fn(*const std::ffi::c_char) -> i32 =
        unsafe { std::mem::transmute(addr) };
    let c = match std::ffi::CString::new(name) {
        Ok(c) => c,
        Err(_) => return -3,
    };
    unsafe { f(c.as_ptr()) }
}

/// Must be called first, from `cplugin_init` (attached thread). Stores the vm
/// and registers the SDK's single pipeline hook (the runtime's automatic
/// class-load dispatch used by all SDK-level pattern hooks).
///
/// # Safety
/// `api` must be the CPluginApi from the runtime, `vm` the live JavaVM* passed
/// to cplugin_init.
pub unsafe fn init(api: *const CPluginApi, vm: JavaVmPtr) {
    if let Some(api_ref) = unsafe { api.as_ref() } {
        ALLOC.store(
            api_ref.jvmti_allocate.map(|f| f as usize).unwrap_or(0),
            Ordering::Relaxed,
        );
        RETRANSFORM.store(
            api_ref.retransform_class.map(|f| f as usize).unwrap_or(0),
            Ordering::Relaxed,
        );
        // register the SDK dispatch hook once (idempotent per process)
        if let Some(register) = api_ref.register_class_hook {
            unsafe { register(std::ptr::null_mut(), sdk_dispatch_hook) };
        }
    }
    VM.store(vm as usize, Ordering::Relaxed);
}

pub fn vm() -> JavaVmPtr {
    VM.load(Ordering::Relaxed) as JavaVmPtr
}

/// Single entry into the runtime pipeline: forwards every class load to the
/// SDK's pattern-matching hook registries (names and bytes). Returns a
/// patched class only when a byte hook replaced it.
unsafe extern "C" fn sdk_dispatch_hook(
    _ctx: *mut std::ffi::c_void,
    name: *const std::ffi::c_char,
    class_data: *const u8,
    class_data_len: usize,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    let nm = jni_util::cstr(name);
    if let Some(nm) = &nm {
        hooks::dispatch(nm);
        if !class_data.is_null() && !out_data.is_null() && !out_len.is_null() {
            let data = unsafe { std::slice::from_raw_parts(class_data, class_data_len) };
            let alloc_addr = ALLOC.load(Ordering::Relaxed);
            if alloc_addr != 0 {
                if let Some(patched) = hooks::dispatch_bytes(nm, data) {
                    let alloc: unsafe extern "C" fn(usize) -> *mut u8 =
                        unsafe { std::mem::transmute(alloc_addr) };
                    let ptr = alloc(patched.len());
                    if !ptr.is_null() {
                        unsafe {
                            std::ptr::copy_nonoverlapping(patched.as_ptr(), ptr, patched.len());
                            *out_data = ptr;
                            *out_len = patched.len();
                        }
                        return 0;
                    }
                }
            }
        }
    }
    1
}
