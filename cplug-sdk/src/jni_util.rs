//! JNI plumbing sugar: attach/detach with RAII, exception->Result, cstr.

use crate::vm;
use jvmti_bindings::prelude::*;
use std::ffi::c_char;

/// Attach the current thread to the JVM if needed, run `f` with a JNI env.
///
/// Standard idiom (Oracle JNI spec): probe with `GetEnv` first; only call
/// `AttachCurrentThread` when detached, and only `DetachCurrentThread` if we
/// attached ourselves. Attaching/detaching an already-attached thread
/// (e.g. the runtime's OnLoad thread, or a class-load hook thread) is fatal.
/// Returns None if the VM is gone or attach fails.
pub fn with_attached<R>(f: impl FnOnce(&JniEnv) -> R) -> Option<R> {
    unsafe {
        let raw_vm = vm();
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

/// Bounded C-string read (the VM's name buffers are not reliably
/// NUL-terminated right after the name — see hello plugin notes).
/// Safe because the read is bounds-checked (max 128 bytes) and the pointer
/// is null-checked; callers pass pointers the VM owns for the call duration.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe {
        let mut end = 0usize;
        while end < 128 && *p.add(end) != 0 {
            end += 1;
        }
        std::str::from_utf8(std::slice::from_raw_parts(p.cast::<u8>(), end)).ok()
    }
}

/// Convert a dotted name to internal form: "org.bukkit.Bukkit" -> "org/bukkit/Bukkit".
pub fn to_internal(name: &str) -> String {
    name.replace('.', "/")
}

/// If a JNI call left a pending exception, describe it to stderr and clear it.
/// Returns whether an exception was pending.
pub fn clear_exception(env: &JniEnv) -> bool {
    if env.exception_check() {
        env.exception_describe();
        env.exception_clear();
        true
    } else {
        false
    }
}
