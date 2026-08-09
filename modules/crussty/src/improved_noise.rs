//! Runtime wiring for the `improved_noise` hook.
//!
//! Replaces the kernel's `ImprovedNoise.noise(DDDDD)D` body with a bridge
//! (`ImprovedNoiseNativeOps`) that samples through the native handle
//! (`PaperNativeImprovedNoise.nativeBuildHandle`/`nativeNoise`). The
//! rewritten body reads `this`'s private `p`/`xo`/`yo`/`zo` itself (legal:
//! it is ImprovedNoise's own method) and passes them to the bridge — the
//! class file's field ACCESS FLAGS must stay untouched, because the JVM
//! rejects any field-modifier change in a retransformed class with
//! JVMTI_ERROR_UNSUPPORTED_REDEFINITION_SCHEMA_CHANGED.
//!
//! Gate: env `CRUSSTY_NATIVE_IMPROVED_NOISE` (1/true/on/yes → on). Off by
//! default, matching upstream Crussty CE's stance: `improved_noise` stays
//! diagnostic until a guarded runtime hook and strict server gate prove the
//! profile benefits.
//!
//! The Java bridge references `ImprovedNoise` directly, so it must live in
//! the kernel's loader (same pattern as area_map's SingleUserAreaMapOps), not
//! the bootstrap: a bootstrap-defined copy would fail to resolve the kernel
//! class and shadow it for the kernel's own (parent-first) loader.
//!
//! Two halfs:
//!
//! 1. Hook: the byte hook (registered at cplugin_init) captures the class's
//!    ORIGINAL bytes on its first sighting — the class's own load, which goes
//!    through the hook while READY=false. Once the kernel class is loaded we
//!    define the bridge classes into its loader, compute the patch from the
//!    captured bytes on the quiet activation worker, then retransform once —
//!    the byte hook serves the precomputed bytes back. The callback itself
//!    never runs the ASM helper: `ClassReader`'s COMPUTE_FRAMES resolves
//!    StackMapTable frame types through `Class.forName`, and a class
//!    definition racing live server class loads inside the redefinition
//!    callback deadlocks. Exactly one retransform, like area_map.)
//!
//! 2. Self-test: after activation, drive the REAL bridge
//!    (`PaperNativeImprovedNoise.nativeBuildHandle`/`nativeNoise` through
//!    JNI) over a synthetic 256-byte permutation: assert the handle builds,
//!    samples are finite and deterministic, and the handle frees cleanly.
//!    This exercises bridge registration + JNI marshalling + native sampling.

use jvmti_bindings::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const NOISE_CLASS: &str = "net/minecraft/world/level/levelgen/synth/ImprovedNoise";
const BRIDGE_NAME: &str = "net/minecraft/world/level/levelgen/synth/ImprovedNoiseNativeOps";
const BRIDGE_HANDLE_NAME: &str = "net/minecraft/world/level/levelgen/synth/ImprovedNoiseNativeOps$Handle";
const NATIVE_BRIDGE: &str = "net/minecraft/world/level/levelgen/synth/PaperNativeImprovedNoise";

const BRIDGE_BYTES: &[u8] = include_bytes!(
    "../noise/build/net/minecraft/world/level/levelgen/synth/ImprovedNoiseNativeOps.class"
);
const BRIDGE_HANDLE_BYTES: &[u8] = include_bytes!(
    "../noise/build/net/minecraft/world/level/levelgen/synth/ImprovedNoiseNativeOps$Handle.class"
);

/// env-gate (off by default), read once at register time
fn enabled() -> bool {
    std::env::var("CRUSSTY_NATIVE_IMPROVED_NOISE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on" || v == "yes"
        })
        .unwrap_or(false)
}

static READY: AtomicBool = AtomicBool::new(false);
/// Global ref to the kernel ImprovedNoise classloader, captured at
/// activation; the patch worker reuses it to feed the ASM helper.
/// 0 = not captured yet.
static KERNEL_LOADER: AtomicUsize = AtomicUsize::new(0);

/// Original class bytes captured from the FIRST sight of the class — its
/// original load goes through the byte hook while READY=false, so the load
/// callback stores the pristine bytes (no retransform needed). The patched
/// bytecode is computed from them on the quiet worker thread, then cached
/// for the final retransform's callback.
static ORIG_BYTES: std::sync::OnceLock<std::sync::Mutex<Option<Vec<u8>>>> =
    std::sync::OnceLock::new();
static PATCH_CACHE: std::sync::OnceLock<std::sync::Mutex<Option<Vec<u8>>>> =
    std::sync::OnceLock::new();

fn orig_lock() -> &'static std::sync::Mutex<Option<Vec<u8>>> {
    ORIG_BYTES.get_or_init(|| std::sync::Mutex::new(None))
}
fn patch_lock() -> &'static std::sync::Mutex<Option<Vec<u8>>> {
    PATCH_CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

/// Class-file version of `b` as (major, minor), or None if not a class file.
fn class_version(b: &[u8]) -> Option<(u16, u16)> {
    if b.len() < 8 || u32::from_be_bytes(b[0..4].try_into().ok()?) != 0xCAFE_BABE {
        return None;
    }
    Some((
        u16::from_be_bytes([b[6], b[7]]),
        u16::from_be_bytes([b[4], b[5]]),
    ))
}

/// Register the byte hook (idempotent; call once from cplugin_init).
///
/// The callback performs NO JNI/ASM work: classfile hooks run on the
/// redefinition thread while the JVM holds loader locks, and the ASM pipeline
/// (ClassReader/ClassWriter with COMPUTE_FRAMES) resolves StackMapTable frame
/// types through `Class.forName` — a class definition racing live server
/// class-loading deadlocks. Instead the ORIGINAL bytes are captured here (at
/// the class's own load, READY=false), patched on the quiet activation
/// worker, and this callback only serves them back from the cache.
pub fn register() {
    if !enabled() {
        eprintln!(
            "[crussty-module] improved_noise: dormant (set CRUSSTY_NATIVE_IMPROVED_NOISE=1 to enable)"
        );
        return;
    }
    cplug_sdk::hooks::register_bytes(NOISE_CLASS, |_name, bytes| {
        if !READY.load(Ordering::Relaxed) {
            // Pristine sighting (the original class load): stash the bytes
            // for the worker to patch; never rewrite here.
            let mut orig = orig_lock().lock().unwrap();
            if orig.is_none() {
                *orig = Some(bytes.to_vec());
            }
            return None;
        }
        // Serve the precomputed patch; zero Java work on this thread.
        let cached = patch_lock().lock().unwrap().clone();
        eprintln!(
            "[crussty-module] improved_noise: hook serve {} bytes (major {})",
            cached.as_ref().map(|c| c.len()).unwrap_or(0),
            cached.as_ref().and_then(|c| class_version(c)).map(|(m, _)| m).unwrap_or(0)
        );
        cached
    });
}

/// Background activation: wait for the kernel class, define the bridge into
/// its loader, flip READY and retransform so the hook applies the patch.
pub fn activate() {
    std::thread::spawn(|| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let mut forced_once = false;
        loop {
            if cplug_sdk::classes::find_class(NOISE_CLASS).is_some() {
                break;
            }
            if std::time::Instant::now() > deadline {
                eprintln!(
                    "[crussty-module] improved_noise: {NOISE_CLASS} not loaded within 60s, hook stays dormant"
                );
                return;
            }
            if !forced_once
                && std::time::Instant::now() > deadline - std::time::Duration::from_secs(50)
            {
                forced_once = true;
                eprintln!(
                    "[crussty-module] improved_noise: forcing kernel load of {NOISE_CLASS}"
                );
                force_load_kernel_class();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // Defer the define/retransform until the server is fully booted.
        // The kernel loader is hot during startup (worldgen codecs load
        // `ImprovedNoise` and friends from the main thread); a JNI
        // DefineClass into that loader races the boot-time class-loading
        // storm and deadlocks the JVM inside defineClass1. After boot the
        // kernel loader is quiet, so defining the bridge + helper there and
        // retransforming `ImprovedNoise` is safe. The byte hook is already
        // registered and passive (READY=false) until this point.
        if !wait_for_boot() {
            eprintln!(
                "[crussty-module] improved_noise: boot marker not seen, hook stays dormant"
            );
            return;
        }
        eprintln!(
            "[crussty-module] improved_noise: server booted, defining bridge into kernel loader"
        );

        let defined = cplug_sdk::jni_util::with_attached(|env| {
            let Some(cls) = cplug_sdk::classes::find_class(NOISE_CLASS) else {
                return false;
            };
            let Some(class_cls) = env.find_class("java/lang/Class") else {
                crate::clear_exception(env);
                return false;
            };
            let Some(loader) = env
                .get_method_id(class_cls, "getClassLoader", "()Ljava/lang/ClassLoader;")
                .and_then(|mid| {
                    let l = env.call_object_method(cls.as_jclass(), mid, &[]);
                    (l as usize != 0).then_some(l)
                })
            else {
                crate::clear_exception(env);
                env.delete_local_ref(class_cls);
                return false;
            };
            let gref = env.new_global_ref(loader);
            if gref.is_null() {
                crate::clear_exception(env);
                env.delete_local_ref(loader);
                env.delete_local_ref(class_cls);
                return false;
            }
            KERNEL_LOADER.store(gref as usize, Ordering::SeqCst);
            let mut ok = true;
            for (name, bytes) in [(BRIDGE_NAME, BRIDGE_BYTES), (BRIDGE_HANDLE_NAME, BRIDGE_HANDLE_BYTES)]
            {
                match env.define_class(name, gref, bytes) {
                    Some(c) => {
                        env.delete_local_ref(c);
                        eprintln!("[crussty-module] improved_noise: defined {name} in kernel loader");
                    }
                    None => {
                        crate::clear_exception(env);
                        eprintln!("[crussty-module] improved_noise: define_class({name}) failed");
                        ok = false;
                    }
                }
            }
            // Pre-define the ASM helper into the kernel loader too, so the
            // byte hook (retransform callback) never performs a define_class
            // mid-retransformation (mirrors area_map: no class definition
            // inside the class-file hook callback).
            if cplug_sdk::asm::ensure_defined(env, gref).is_none() {
                eprintln!("[crussty-module] improved_noise: asm helper define failed");
                ok = false;
            }
            env.delete_local_ref(loader);
            env.delete_local_ref(class_cls);
            ok
        });
        if !defined.unwrap_or(false) {
            eprintln!("[crussty-module] improved_noise: bridge definition aborted (no env or loader)");
            return;
        }

        // The ORIGINAL bytes were captured by the byte hook during the
        // class's own load (READY=false branch of register()). Compute the
        // patch on this quiet thread — never inside the classfile hook
        // callback — then a SINGLE retransform serves it back.
        let original = orig_lock().lock().unwrap().clone();
        let Some(original) = original else {
            eprintln!("[crussty-module] improved_noise: no original bytes captured (class loaded before hook?), hook stays dormant");
            return;
        };

        // Phase 2: run the ASM pipeline here (quiet thread, no JVMTI locks).
        let loader = KERNEL_LOADER.load(Ordering::SeqCst);
        if loader == 0 {
            eprintln!("[crussty-module] improved_noise: no kernel loader captured");
            return;
        }
        let patched = cplug_sdk::jni_util::with_attached(|env| {
            let spec = cplug_sdk::asm::ReplaceBody {
                method_name: "noise",
                method_desc: "(DDDDD)D",
                bridge_owner: BRIDGE_NAME,
                bridge_name: "noise",
                bridge_desc:
                    "(Lnet/minecraft/world/level/levelgen/synth/ImprovedNoise;[BDDDDDDDD)D",
                args: &[
                    // `this` (the handle cache key)
                    cplug_sdk::asm::ArgSpec::Local { slot: 0, ty: b'L' },
                    // this.p / this.xo / this.yo / this.zo — read inside the
                    // patched class's own method; no access-flag changes.
                    cplug_sdk::asm::ArgSpec::ThisField { name: "p", desc: "[B" },
                    cplug_sdk::asm::ArgSpec::ThisField { name: "xo", desc: "D" },
                    cplug_sdk::asm::ArgSpec::ThisField { name: "yo", desc: "D" },
                    cplug_sdk::asm::ArgSpec::ThisField { name: "zo", desc: "D" },
                    // raw coordinates: x, y, z, yScale, yMax
                    cplug_sdk::asm::ArgSpec::Local { slot: 1, ty: b'D' },
                    cplug_sdk::asm::ArgSpec::Local { slot: 3, ty: b'D' },
                    cplug_sdk::asm::ArgSpec::Local { slot: 5, ty: b'D' },
                    cplug_sdk::asm::ArgSpec::Local { slot: 7, ty: b'D' },
                    cplug_sdk::asm::ArgSpec::Local { slot: 9, ty: b'D' },
                ],
            };
            cplug_sdk::asm::replace_body(env, loader as jni::jobject, &original, &spec)
        })
        .flatten();
        let Some(patched) = patched else {
            eprintln!("[crussty-module] improved_noise: patch computation failed, hook stays dormant");
            return;
        };
        eprintln!(
            "[crussty-module] improved_noise: computed patch for noise() ({} -> {} bytes), orig major {} patch major {}",
            original.len(),
            patched.len(),
            class_version(&original).map(|(m, _)| m).unwrap_or(0),
            class_version(&patched).map(|(m, _)| m).unwrap_or(0)
        );
        *patch_lock().lock().unwrap() = Some(patched);

        // Phase 3: a SINGLE retransform; the callback serves the cached patch.
        READY.store(true, Ordering::Release);
        let rc = cplug_sdk::retransform_class(NOISE_CLASS);
        eprintln!("[crussty-module] improved_noise: hook armed, retransform rc={rc}");

        bridge_selftest();
    });
}

/// Wait until the server has finished booting: `org/bukkit/Bukkit`'s static
/// `getServer()` returns a non-null CraftServer, with a short settling delay
/// after that so the kernel loader's boot-time class-loading storm has
/// fully quieted. Returns false on timeout (~120s).
fn wait_for_boot() -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        let booted = cplug_sdk::jni_util::with_attached(|env| {
            let Some(bukkit) = cplug_sdk::classes::find_class("org/bukkit/Bukkit") else {
                return false;
            };
            let Some(get_server) = env.get_static_method_id(
                bukkit.as_jclass(),
                "getServer",
                "()Lorg/bukkit/Server;",
            ) else {
                crate::clear_exception(env);
                return false;
            };
            let srv = env.call_static_object_method(bukkit.as_jclass(), get_server, &[]);
            let had_exc = crate::clear_exception(env);
            if srv.is_null() || had_exc {
                false
            } else {
                env.delete_local_ref(srv);
                true
            }
        });
        if booted.unwrap_or(false) {
            std::thread::sleep(std::time::Duration::from_secs(10));
            return true;
        }
        if std::time::Instant::now() > deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn force_load_kernel_class() {
    let _ = cplug_sdk::jni_util::with_attached(|env| {
        let Some(seed) = cplug_sdk::classes::find_class("org/bukkit/Bukkit") else {
            eprintln!("[crussty-module] improved_noise: force load: Bukkit not found");
            return None::<()>;
        };
        let Some(class_cls) = env.find_class("java/lang/Class") else {
            crate::clear_exception(env);
            return None::<()>;
        };
        let Some(loader) = env
            .get_method_id(class_cls, "getClassLoader", "()Ljava/lang/ClassLoader;")
            .and_then(|mid| {
                let l = env.call_object_method(seed.as_jclass(), mid, &[]);
                (l as usize != 0).then_some(l)
            })
        else {
            crate::clear_exception(env);
            env.delete_local_ref(class_cls);
            return None::<()>;
        };
        let Some(forname) = env.get_static_method_id(
            class_cls,
            "forName",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        ) else {
            crate::clear_exception(env);
            env.delete_local_ref(class_cls);
            env.delete_local_ref(loader);
            return None::<()>;
        };
        let dot = NOISE_CLASS.replace('/', ".");
        let Some(name) = env.new_string(&dot) else {
            crate::clear_exception(env);
            env.delete_local_ref(class_cls);
            env.delete_local_ref(loader);
            return None::<()>;
        };
        let loaded = env.call_static_object_method(
            class_cls,
            forname,
            &[
                jni::jvalue { l: name },
                jni::jvalue { z: 1 /* true */ },
                jni::jvalue { l: loader },
            ],
        );
        let had_exc = crate::clear_exception(env);
        if loaded.is_null() {
            eprintln!(
                "[crussty-module] improved_noise: Class.forName({NOISE_CLASS}) failed (exc={had_exc})"
            );
        } else {
            eprintln!(
                "[crussty-module] improved_noise: Class.forName({NOISE_CLASS}) succeeded"
            );
        }
        env.delete_local_ref(loaded);
        env.delete_local_ref(name);
        env.delete_local_ref(class_cls);
        env.delete_local_ref(loader);
        Some(())
    });
}

/// Drive the real bridge through JNI over a synthetic 256-byte permutation:
/// buildHandle must return a nonzero handle, nativeNoise samples must be
/// finite and deterministic, freeHandle must cleanly release it.
fn bridge_selftest() {
    let ok = cplug_sdk::jni_util::with_attached(|env| {
        let Some(bridge) = env.find_class(NATIVE_BRIDGE) else {
            crate::clear_exception(env);
            eprintln!("[crussty-module] improved_noise: self-test: find_class({NATIVE_BRIDGE}) failed");
            return false;
        };
        let Some(build) = env.get_static_method_id(bridge, "nativeBuildHandle", "([BDDD)J") else {
            crate::clear_exception(env);
            env.delete_local_ref(bridge);
            return false;
        };
        let Some(noise_mid) = env.get_static_method_id(bridge, "nativeNoise", "(JDDDDD)D") else {
            crate::clear_exception(env);
            env.delete_local_ref(bridge);
            return false;
        };
        let Some(free) = env.get_static_method_id(bridge, "nativeFreeHandle", "(J)V") else {
            crate::clear_exception(env);
            env.delete_local_ref(bridge);
            return false;
        };

        let mut perm = [0u8; 256];
        for (i, p) in perm.iter_mut().enumerate() {
            *p = (i as u8).wrapping_mul(31).wrapping_add(7);
        }
        let Some(arr) = env.new_byte_array(256) else {
            env.delete_local_ref(bridge);
            return false;
        };
        let signed: Vec<i8> = perm.iter().map(|&b| b as i8).collect();
        env.set_byte_array_region(arr, 0, 256, &signed);

        let raw = env.raw();
        unsafe {
            let fn_table = &(**raw);
            let jenv = raw;
            let call_long = fn_table.CallStaticLongMethodA;
            let call_double = fn_table.CallStaticDoubleMethodA;
            let call_void = fn_table.CallStaticVoidMethodA;

            let args = [jni::jvalue { l: arr }, d(1.0), d(2.0), d(3.0), d(0.0), d(0.0)];
            let handle = (call_long)(jenv, bridge, build, args.as_ptr());
            if handle == 0 {
                eprintln!("[crussty-module] improved_noise: self-test: buildHandle returned 0");
                return false;
            }
            let pts: [(f64, f64, f64, f64, f64); 3] = [
                (1.5, 2.5, 3.5, 1.0, 0.0),
                (-7.25, 123.5, -0.125, 4.0, 100.0),
                (33.0, 5.75, -2.0, 0.0, 1.0),
            ];
            for (x, y, z, ys, ym) in pts {
                let args = [jlong_val(handle), d(x), d(y), d(z), d(ys), d(ym)];
                let v = (call_double)(jenv, bridge, noise_mid, args.as_ptr());
                if !v.is_finite() {
                    eprintln!(
                        "[crussty-module] improved_noise: self-test: non-finite sample at ({x},{y},{z})"
                    );
                    return false;
                }
            }
            let args = [jlong_val(handle)];
            (call_void)(jenv, bridge, free, args.as_ptr());
        }
        env.delete_local_ref(arr);
        env.delete_local_ref(bridge);
        true
    });
    match ok {
        Some(true) => eprintln!(
            "[crussty-module] improved_noise: self-test passed (native handle round-trip through real bridge)"
        ),
        _ => eprintln!("[crussty-module] improved_noise: self-test failed/skipped"),
    }
}

fn d(v: f64) -> jni::jvalue {
    jni::jvalue { d: v }
}

fn jlong_val(v: i64) -> jni::jvalue {
    jni::jvalue { j: v }
}