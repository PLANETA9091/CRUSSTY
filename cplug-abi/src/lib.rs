//! cplug-abi — the ONLY contract between the crussty-runtime and modules.
//!
//! Modules are native shared libraries in `modules/` (recursive scan). Each
//! must export:
//!
//! ```c
//! int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options);
//! ```
//!
//! Everything else is raw JVM access (JNI/JVMTI via the JavaVM*): no Java API,
//! no limits. The api gives three tiny services: register a class-file hook
//! (the automatic hot-patch pipeline) and allocate replacement bytes through
//! the JVMTI allocator.

#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_void};

pub const CPAPI_VERSION: u32 = 3;

/// Optional plugin hook, invoked for every class load before the class is
/// defined (JVMTI CLASS_FILE_LOAD_HOOK). The hot-patch pipeline: plugins get
/// the raw class bytes and may return patched bytes.
///
/// Contract:
/// - `out_data`/`out_len` must be set only when replacing the class; the
///   replacement buffer must come from `api.jvmti_allocate`.
/// - return 0  -> patched: use *out_data/*out_len as the new class bytes;
///   return != 0 -> keep original bytes (plugin error or no-op).
pub type ClassHookFn = unsafe extern "C" fn(
    ctx: *mut c_void,
    name: *const c_char,
    class_data: *const u8,
    class_data_len: usize,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32;

/// Agent -> plugin services.
#[repr(C)]
pub struct CPluginApi {
    pub version: u32,
    /// Register `hook` (with plugin-owned `ctx`) in the runtime's patch pipeline.
    pub register_class_hook:
        Option<unsafe extern "C" fn(ctx: *mut c_void, hook: ClassHookFn) -> i32>,
    /// Allocate a buffer via JVMTI (freed by the VM with Deallocate).
    pub jvmti_allocate: Option<unsafe extern "C" fn(size: usize) -> *mut u8>,
    /// Retransform a loaded class by internal name; re-enters the plugin hook
    /// pipeline with its current bytes so a hook may patch it post-load.
    /// Returns 0 on success, negative on failure (class not loaded, etc.).
    pub retransform_class: Option<unsafe extern "C" fn(name: *const c_char) -> i32>,
    /// Claim a unique global resource key before registering it with the
    /// JVM — e.g. "class:a/b/C" for DefineClass and "native:a/b/C#name:sig"
    /// for RegisterNatives. Guarantees two modules never collide: returns 0
    /// when the key is free (or already owned by the same `owner`) and -1
    /// when another module owns it (the caller must skip the registration).
    /// `owner` is the caller's plugin handle (e.g. the CPluginApi pointer).
    pub claim: Option<unsafe extern "C" fn(owner: usize, key: *const c_char) -> i32>,
}

/// Opaque JavaVM* (cast to your JNI bindings' JavaVM type on either side).
pub type JavaVmPtr = *mut c_void;

/// `cplugin_init(api, vm, options) -> i32` — the single required export.
/// options carries runtime info, e.g. "modules=<dir>;versions=<dir>;kernel=<jar>".
pub type CPluginInit =
    unsafe extern "C" fn(api: *const CPluginApi, vm: JavaVmPtr, options: *const c_char) -> i32;
