//! Kernel-facing JNI: world/region ops through the `DistKernel` Java helper
//! (defined into the KERNEL's class loader, where Bukkit API resolves) and
//! load metrics from `MinecraftServer.getTickTimesNanos()`.
//!
//! Port of v1 dist-paper's kernel half: RegionManager.forceLoad/forceUnload
//! (via DistKernel.forceChunks) + RegionHasher (via DistKernel.hashRegion) +
//! Metrics (via tick times, not the plugin event bus).

use cplug_sdk::classes::{self, static_method};
use cplug_sdk::jni_util::clear_exception;
use jvmti_bindings::prelude::*;
use std::sync::{Mutex, OnceLock};

const HELPER_NAME: &str = "dev/dist/DistKernel";
const HELPER_BYTES: &[u8] = include_bytes!("../build/dev/dist/DistKernel.class");
const BUKKIT_CLASS: &str = "org/bukkit/Bukkit";
const SERVER_CLASS: &str = "net/minecraft/server/MinecraftServer";

/// Resolved static method ids + a global ref to the helper class (lives for
/// the process; never deleted). Global refs are thread-safe by JNI contract.
#[derive(Clone, Copy)]
struct Cached {
    helm: jni::jclass,
    force: jni::jmethodID,
    hashr: jni::jmethodID,
}

// JNI global refs are safe to share across threads (delete from any thread).
unsafe impl Send for Cached {}
unsafe impl Sync for Cached {}

static CACHE: OnceLock<Mutex<Option<Cached>>> = OnceLock::new();

fn cache() -> &'static Mutex<Option<Cached>> {
    CACHE.get_or_init(|| Mutex::new(None))
}

/// True once the helper class + method ids are resolved.
pub fn is_ready() -> bool {
    cache().lock().unwrap().is_some()
}

/// Define the helper into the kernel's class loader and resolve the static
/// method ids. Called from a main-thread job; concurrency-guarded.
pub fn ensure_ready(env: &JniEnv) {
    let mut guard = cache().lock().unwrap();
    if guard.is_some() {
        return;
    }
    match define(env) {
        Ok(c) => {
            *guard = Some(c);
            eprintln!("[dist] kernel ready: DistKernel defined in kernel loader");
        }
        Err(e) => eprintln!("[dist] kernel helper failed: {e}"),
    }
}

fn define(env: &JniEnv) -> Result<Cached, String> {
    // Bukkit and the helper both live in the kernel's loader; grab that loader
    // through java.lang.Class.getClassLoader on the Bukkit class.
    let bukkit = classes::find_class(BUKKIT_CLASS)
        .ok_or_else(|| format!("{BUKKIT_CLASS} not loaded"))?;
    let class_cls = env.find_class("java/lang/Class").ok_or("no Class class")?;
    let get_loader = env
        .get_method_id(class_cls, "getClassLoader", "()Ljava/lang/ClassLoader;")
        .ok_or("no getClassLoader")?;
    let loader = env.call_object_method(bukkit.as_jclass(), get_loader, &[]);
    if loader.is_null() {
        clear_exception(env);
        return Err("no loader for Bukkit".into());
    }
    let helm = env.define_class(HELPER_NAME, loader, HELPER_BYTES);
    env.delete_local_ref(loader);
    env.delete_local_ref(class_cls);
    let helm = helm.ok_or_else(|| {
        clear_exception(env);
        format!("define_class({HELPER_NAME}) failed")
    })?;
    let force = env
        .get_static_method_id(helm, "forceChunks", "(IIIZ)V")
        .ok_or_else(|| -> String {
            clear_exception(env);
            "no forceChunks".into()
        })?;
    let hashr = env
        .get_static_method_id(helm, "hashRegion", "(III)[B")
        .ok_or_else(|| -> String {
            clear_exception(env);
            "no hashRegion".into()
        })?;
    let gref = env.new_global_ref(helm);
    env.delete_local_ref(helm);
    Ok(Cached { helm: gref, force, hashr })
}

/// Force-load or release the chunk square of a region (kernel loader world).
pub fn force_chunks(
    env: &JniEnv,
    region: i32,
    chunks_per_side: i32,
    regions_per_row: i32,
    force: bool,
) {
    if let Some(c) = cache().lock().unwrap().as_ref() {
        env.call_static_void_method(
            c.helm,
            c.force,
            &[
                jni::jvalue { i: region },
                jni::jvalue { i: chunks_per_side },
                jni::jvalue { i: regions_per_row },
                jni::jvalue { z: force as u8 },
            ],
        );
        clear_exception(env);
    }
}

/// SHA-256 of a region's loaded chunk data, or None on error.
pub fn hash_region(
    env: &JniEnv,
    region: i32,
    chunks_per_side: i32,
    regions_per_row: i32,
) -> Option<[u8; 32]> {
    let c = cache().lock().unwrap().as_ref().copied()?;
    let obj = env.call_static_object_method(
        c.helm,
        c.hashr,
        &[
            jni::jvalue { i: region },
            jni::jvalue { i: chunks_per_side },
            jni::jvalue { i: regions_per_row },
        ],
    );
    if clear_exception(env) || obj.is_null() {
        return None;
    }
    let len = env.get_array_length(obj as jni::jarray).min(32) as usize;
    let mut buf = vec![0i8; len];
    env.get_byte_array_region(obj as jni::jbyteArray, 0, len as i32, &mut buf);
    env.delete_local_ref(obj);
    let mut out = [0u8; 32];
    for (i, b) in buf.iter().enumerate() {
        out[i] = *b as u8;
    }
    Some(out)
}

/// Load 0.0..1.0 from the kernel's own tick times. 50ms/tick == 1.0.
/// Falls back to 0.0 when the server isn't up yet.
pub fn load(env: &JniEnv) -> f64 {
    let Some(mc) = classes::find_class(SERVER_CLASS) else {
        return 0.0;
    };
    let Some(get_server) = static_method(
        env,
        mc.as_jclass(),
        "getServer",
        &format!("()L{SERVER_CLASS};"),
    ) else {
        return 0.0;
    };
    let server = env.call_static_object_method(mc.as_jclass(), get_server as jni::jmethodID, &[]);
    if server.is_null() || clear_exception(env) {
        return 0.0;
    }
    let Some(get_times) = env.get_method_id(mc.as_jclass(), "getTickTimesNanos", "()[J") else {
        env.delete_local_ref(server);
        return 0.0;
    };
    let arr = env.call_object_method(server, get_times, &[]);
    env.delete_local_ref(server);
    if arr.is_null() || clear_exception(env) {
        return 0.0;
    }
    let len = env.get_array_length(arr as jni::jarray).max(0) as usize;
    if len == 0 {
        env.delete_local_ref(arr);
        return 0.0;
    }
    let mut ticks = vec![0i64; len];
    env.get_long_array_region(arr as jni::jlongArray, 0, len as i32, &mut ticks);
    env.delete_local_ref(arr);
    let sum: i64 = ticks.iter().sum();
    let mean_ns = sum as f64 / len.max(1) as f64;
    (mean_ns / 50.0e6).min(1.0)
}

/// Version of the bridge (smoke-test parity with v1 `DistNode.version()`).
#[allow(dead_code)] // stable v1 API surface; smoke-test parity
pub fn version() -> i32 {
    1
}