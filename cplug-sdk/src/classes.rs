//! Class resolution that works across ALL class loaders.
//!
//! JNI FindClass from a native thread only sees the system loader — kernel
//! classes (org/bukkit/*, net/minecraft/*) live in Paper's own loader and are
//! invisible to it. The SDK resolves them via JVMTI GetLoadedClasses and caches
//! a process-lifetime global ref per class.

use crate::jni_util::{clear_exception, with_attached};
use jvmti_bindings::prelude::*;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// A jclass global ref to a loaded class, safe across threads.
///
/// The ref is owned by the process-lifetime cache (find_class leaks the ref
/// into CACHE on purpose). ClassRef itself is a non-owning view — it must NOT
/// delete the ref on drop, or the next find_class from cache would hand out a
/// dangling jclass (GetMethodID on it SIGSEGVs at +0x10).
pub struct ClassRef(pub jni::jclass);

// JNI global refs are thread-safe by contract (delete from any thread), so
// raw *mut c_void wrapped here may cross threads.
unsafe impl Send for ClassRef {}
unsafe impl Sync for ClassRef {}

impl ClassRef {
    pub fn as_jclass(&self) -> jni::jclass {
        self.0
    }
}

static CACHE: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, usize>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Resolve a loaded class by name ("org/bukkit/Bukkit" or dotted) and cache
/// it forever. Returns None if the class is not loaded yet.
pub fn find_class(name: &str) -> Option<ClassRef> {
    let internal = crate::jni_util::to_internal(name);
    if let Some(addr) = cache().lock().unwrap().get(&internal) {
        return Some(ClassRef(*addr as jni::jclass));
    }
    let signature = format!("L{internal};");
    with_attached(|env| {
        let jvmti = Jvmti::new(crate::vm() as *mut jni::JavaVM).ok()?;
        let classes = jvmti.get_loaded_classes().ok()?;
        let mut found = None;
        for cls in &classes {
            // Only accept classes that have FINISHED definition: throughout
            // init (Paper's remap redefines many boot classes) GetLoadedClasses
            // can return a class whose InstanceKlass is not laid out yet —
            // GetMethodID on it derefs null fields (SIGSEGV).
            let Ok(status) = jvmti.get_class_status(*cls) else {
                env.delete_local_ref(*cls);
                continue;
            };
            const INITIALIZED: i32 = 4; // JVMTI_CLASS_STATUS_INITIALIZED (jvmti.h)
            if status & INITIALIZED == 0 {
                env.delete_local_ref(*cls);
                continue;
            }
            if let Ok((sig, _)) = jvmti.get_class_signature(*cls) {
                if sig == signature {
                    found = Some(*cls);
                    continue; // keep this local ref for the caller
                }
            }
            env.delete_local_ref(*cls);
        }
        let cls = found?;
        let gref = env.new_global_ref(cls);
        env.delete_local_ref(cls);
        let _ = clear_exception(env);
        cache()
            .lock()
            .unwrap()
            .insert(internal.clone(), gref as usize);
        Some(ClassRef(gref))
    })
    .flatten()
}

/// Poll for the class until it is loaded (kernel-ready convenience):
/// returns immediately if present, otherwise retries every 200ms up to
/// `timeout_ms`. Returns None on timeout.
pub fn wait_class(name: &str, timeout_ms: u64) -> Option<ClassRef> {
    let mut waited = 0u64;
    loop {
        if let Some(c) = find_class(name) {
            return Some(c);
        }
        waited += 200;
        if waited >= timeout_ms {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

/// Resolve a method id on a class (instance method). Clears pending
/// exceptions (NoSuchMethodError) so callers can retry cleanly.
pub fn method(env: &JniEnv, cls: jni::jclass, name: &str, desc: &str) -> Option<usize> {
    let m = env.get_method_id(cls, name, desc);
    if m.is_none() {
        let _ = clear_exception(env);
    }
    m.map(|mid| mid as usize)
}

/// Resolve a static method id. Clears pending exceptions on failure.
pub fn static_method(env: &JniEnv, cls: jni::jclass, name: &str, desc: &str) -> Option<usize> {
    let m = env.get_static_method_id(cls, name, desc);
    if m.is_none() {
        let _ = clear_exception(env);
    }
    m.map(|mid| mid as usize)
}

/// Resolve + call a static method that returns a String, and read it.
/// Clears any pending exception either way.
pub fn static_string(env: &JniEnv, cls: jni::jclass, name: &str, desc: &str) -> Option<String> {
    let mid = static_method(env, cls, name, desc)?;
    let obj = env.call_static_object_method(cls, mid as jni::jmethodID, &[]);
    if obj.is_null() {
        let _ = clear_exception(env);
        return None;
    }
    let s = env.get_string_utf(obj as jni::jstring);
    let _ = clear_exception(env);
    s
}

/// Re-run the class-file hook chain for a loaded class (JVMTI
/// RetransformClasses): the VM calls every byte hook again, synchronously on
/// the calling thread, with the class's ORIGINAL bytes. The canonical way to
/// instrument a class that was already loaded (and possibly already used)
/// without a definition race. Must be called from an attached thread (e.g.
/// inside run_on_main_thread).
pub fn retransform(name: &str) -> bool {
    let Some(cls) = find_class(name) else {
        return false;
    };
    with_attached(|env| {
        let jvmti = Jvmti::new(crate::vm() as *mut jni::JavaVM).ok()?;
        // Capabilities are PER-ENVIRONMENT: GetEnv hands out a fresh env
        // each call, so the agent's can_retransform_classes does not apply
        // here — this env must add it itself (legal in the live phase).
        if let Err(e) = jvmti.add_capabilities_with(|caps| {
            caps.set_can_retransform_classes(true);
        }) {
            eprintln!("[cplug-sdk] add_capabilities failed: {e:?}");
            return Some(false);
        }
        let r = jvmti.retransform_classes(&[cls.as_jclass()]);
        let _ = clear_exception(env);
        match r {
            Ok(()) => Some(true),
            Err(e) => {
                eprintln!("[cplug-sdk] retransform {name} failed: {e:?}");
                Some(false)
            }
        }
    })
    .flatten()
    .unwrap_or(false)
}
