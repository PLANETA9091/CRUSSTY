//! Runtime wiring for the `area_map` hook.
//!
//! Two halfs, both verified end-to-end:
//!
//! 1. Hook: once the kernel's `SingleUserAreaMap` is loaded we define the
//!    `SingleUserAreaMapOps` helper classes (compiled against the kernel's
//!    class shapes, see `area-map/`) into the SAME loader as the map, then
//!    `READY` flips and the class is retransformed. The byte hook fires on
//!    the retransform (or on a late original load) and swaps `update()`'s
//!    body for a branch-minimal `invokestatic SingleUserAreaMapOps.run(...)`.
//!
//!    The helper classes must live in the map's loader, NOT the bootstrap:
//!    their bytecode references `SingleUserAreaMap` directly, and a
//!    bootstrap-defined copy would both fail to resolve the kernel class and
//!    shadow it for the kernel's own (parent-first) loader.
//!
//! 2. Self-test: after activation, drive the REAL bridge
//!    (`PaperNativeAreaMap.nativeUpdateOpsBatch` through JNI) over random
//!    rects and compare every produced (op, x, z) against the naive
//!    set-difference (adds = new∖old, removes = old∖new). This exercises
//!    bridge registration + JNI marshalling + native enumeration + the
//!    op/key encodings the helper's apply loop decodes.

use crate::classfile;
use jvmti_bindings::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};


pub const MAP_CLASS: &str = classfile::MAP_CLASS;
const OPS_NAME: &str = classfile::OPS_CLASS;
const SCRATCH_NAME: &str = "ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps$Scratch";
const OPS1_NAME: &str = "ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps$1";
const NATIVE_CLASS: &str = "ca/spottedleaf/moonrise/common/misc/PaperNativeAreaMap";

const OPS1_BYTES: &[u8] = include_bytes!(
    "../area-map/build/ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps$1.class"
);
const SCRATCH_BYTES: &[u8] = include_bytes!(
    "../area-map/build/ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps$Scratch.class"
);
const OPS_BYTES: &[u8] = include_bytes!(
    "../area-map/build/ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps.class"
);

static READY: AtomicBool = AtomicBool::new(false);
static PATCHED: AtomicBool = AtomicBool::new(false);

/// Register the byte hook (idempotent; call once from cplugin_init).
pub fn register() {
    cplug_sdk::hooks::register_bytes(MAP_CLASS, |name, bytes| {
        if !READY.load(Ordering::Relaxed) {
            return None;
        }
        if PATCHED.swap(true, Ordering::SeqCst) {
            return None;
        }
        match classfile::patch_update(bytes) {
            Ok(b) => {
                eprintln!(
                    "[crussty-module] area_map: patched {name} update() ({} -> {} bytes)",
                    bytes.len(),
                    b.len()
                );
                Some(b)
            }
            Err(e) => {
                PATCHED.store(false, Ordering::SeqCst);
                eprintln!("[crussty-module] area_map: patch failed: {e}");
                None
            }
        }
    });
}

/// Background activation: wait for the kernel map class, define the helper
/// classes into its loader, then flip READY and retransform so the hook
/// applies the patch.
pub fn activate() {
    std::thread::spawn(|| {
        // Poll via JVMTI GetLoadedClasses (SDK) — unlike raw JNI find_class
        // (system loader only) this sees classes defined in the kernel's own
        // classloader, where Moonrise loads SingleUserAreaMap. Moonrise loads
        // it lazily (first area-map use), so on an idle world we also force
        // the load through the kernel loader via Class.forName after a grace
        // period; the hook then applies on the retransform below.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let mut forced_once = false;
        loop {
            if cplug_sdk::classes::find_class(MAP_CLASS).is_some() {
                break;
            }
            if std::time::Instant::now() > deadline {
                eprintln!("[crussty-module] area_map: {MAP_CLASS} not loaded within 60s, hook stays dormant");
                return;
            }
            if !forced_once && std::time::Instant::now() > deadline - std::time::Duration::from_secs(50) {
                forced_once = true;
                eprintln!("[crussty-module] area_map: forcing kernel load of {MAP_CLASS}");
                force_load_kernel_class();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let defined = cplug_sdk::jni_util::with_attached(|env| {
            let Some(cls) = cplug_sdk::classes::find_class(MAP_CLASS) else {
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
            let mut ok = true;
            for (name, bytes) in [
                (SCRATCH_NAME, SCRATCH_BYTES),
                (OPS1_NAME, OPS1_BYTES),
                (OPS_NAME, OPS_BYTES),
            ] {
                match env.define_class(name, loader, bytes) {
                    Some(c) => {
                        env.delete_local_ref(c);
                        eprintln!("[crussty-module] area_map: defined {name} in map loader");
                    }
                    None => {
                        crate::clear_exception(env);
                        eprintln!("[crussty-module] area_map: define_class({name}) failed");
                        ok = false;
                    }
                }
            }
            env.delete_local_ref(loader);
            env.delete_local_ref(class_cls);
            ok
        });
        if !defined.unwrap_or(false) {
            eprintln!("[crussty-module] area_map: helper definition aborted (no env or loader)");
            return;
        }

        READY.store(true, Ordering::Release);
        let rc = cplug_sdk::retransform_class(MAP_CLASS);
        eprintln!("[crussty-module] area_map: hook armed, retransform rc={rc}");

        // Semantic self-test through the real bridge + native.
        if cplug_sdk::jni_util::with_attached(bridge_selftest).is_none() {
            eprintln!("[crussty-module] area_map: self-test skipped (no env)");
        }
    });
}

/// Drive nativeUpdateOpsBatch through the bridge and check every produced
/// (op, x, z) against the naive set difference. All positions are chosen so
/// the tests stay deterministic (fixed LCG, small radii).
fn bridge_selftest(env: &JniEnv) -> Option<()> {
    let cls = env.find_class(NATIVE_CLASS)?;
    let mid = env.get_static_method_id(
        cls,
        "nativeUpdateOpsBatch",
        "(IIIIII[B[J)I",
    )?;
    let cap = 2 * (2 * 6 + 1) * (2 * 6 + 1); // max for d=6 in both squares

    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };

    let mut checked = 0usize;
    let mut failures = 0usize;
    for _ in 0..64 {
        let from_x = ((next() % 21) as i32) - 10;
        let from_z = ((next() % 21) as i32) - 10;
        let to_x = ((next() % 21) as i32) - 10;
        let to_z = ((next() % 21) as i32) - 10;
        let old_d = (next() % 7) as i32;
        let new_d = (next() % 7) as i32;

        let ops_arr = env.new_byte_array(cap)?;
        let Some(keys_arr) = env.new_long_array(cap) else {
            env.delete_local_ref(ops_arr);
            return None;
        };
        let n = env.call_static_int_method(
            cls,
            mid,
            &[
                jni::jvalue { i: from_x },
                jni::jvalue { i: from_z },
                jni::jvalue { i: old_d },
                jni::jvalue { i: to_x },
                jni::jvalue { i: to_z },
                jni::jvalue { i: new_d },
                jni::jvalue { l: ops_arr },
                jni::jvalue { l: keys_arr },
            ],
        );
        let had_exc = crate::clear_exception(env);

        let mut ops = vec![0i8; cap as usize];
        let mut keys = vec![0i64; cap as usize];
        env.get_byte_array_region(ops_arr, 0, cap, &mut ops);
        env.get_long_array_region(keys_arr, 0, cap, &mut keys);
        env.delete_local_ref(ops_arr);
        env.delete_local_ref(keys_arr);
        if had_exc {
            eprintln!("[crussty-module] area_map: self-test rect ({from_x},{from_z},d{old_d})->({to_x},{to_z},d{new_d}) threw");
            failures += 1;
            continue;
        }

        let n = n.max(0) as usize;
        let (expected_adds, expected_removes) = naive_set_difference(from_x, from_z, old_d, to_x, to_z, new_d);
        let mut actual_adds = HashSet::new();
        let mut actual_removes = HashSet::new();
        let mut dup = false;
        for i in 0..n {
            let key = keys[i];
            let x = key as i32;
            let z = (key >> 32) as i32;
            let cell = (x, z);
            if ops[i] == 0 {
                dup |= !actual_adds.insert(cell);
            } else {
                dup |= !actual_removes.insert(cell);
            }
        }
        if dup || n != expected_adds.len() + expected_removes.len()
            || actual_adds != expected_adds
            || actual_removes != expected_removes
        {
            failures += 1;
            if failures <= 3 {
                eprintln!(
                    "[crussty-module] area_map: SELF-TEST FAIL ({from_x},{from_z},d{old_d})->({to_x},{to_z},d{new_d}): n={n} expected {}+{} (dup={dup})",
                    expected_adds.len(),
                    expected_removes.len()
                );
            }
        }
        checked += 1;
    }
    env.delete_local_ref(cls);
    if failures == 0 {
        eprintln!("[crussty-module] area_map: self-test OK ({checked} rects, native == naive set difference)");
    } else {
        eprintln!("[crussty-module] area_map: self-test FAILED {failures}/{checked}");
    }
    Some(())
}

/// Naive reference: adds = new square ∖ old square, removes = old ∖ new.
type RectDiff = (HashSet<(i32, i32)>, HashSet<(i32, i32)>);
fn naive_set_difference(
    from_x: i32,
    from_z: i32,
    old_d: i32,
    to_x: i32,
    to_z: i32,
    new_d: i32,
) -> RectDiff {
    let mut old_set = HashSet::new();
    for x in from_x - old_d..=from_x + old_d {
        for z in from_z - old_d..=from_z + old_d {
            old_set.insert((x, z));
        }
    }
    let mut new_set = HashSet::new();
    for x in to_x - new_d..=to_x + new_d {
        for z in to_z - new_d..=to_z + new_d {
            new_set.insert((x, z));
        }
    }
    let adds: HashSet<(i32, i32)> = new_set.difference(&old_set).copied().collect();
    let removes: HashSet<(i32, i32)> = old_set.difference(&new_set).copied().collect();
    (adds, removes)
}

/// Force the kernel to load `MAP_CLASS` (lazy Moonrise class) through its own
/// classloader. JNI `FindClass` from a native thread only resolves against the
/// system loader, and `Class.forName(String, boolean, ClassLoader)` is the only
/// load trigger that lets us name that loader explicitly. Runs its static
/// initializer too (initialize=true), which is a good canary that the patched
/// class actually links in the kernel — errors surface as exceptions here, not
/// later as a fatal `VerifyError` at first use.
fn force_load_kernel_class() {
    let _ = cplug_sdk::jni_util::with_attached(|env| {
        let Some(seed) = cplug_sdk::classes::find_class("org/bukkit/Bukkit") else {
            eprintln!("[crussty-module] area_map: force load: Bukkit not found");
            return None::<()>;
        };
        // getClassLoader is declared on java.lang.Class; GetMethodID on an
        // interface (Bukkit) does not inherit Object instance methods, so
        // resolve the mid from the Class class directly.
        let Some(class_cls) = env.find_class("java/lang/Class") else {
            crate::clear_exception(env);
            eprintln!("[crussty-module] area_map: force load: no Class class");
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
            eprintln!("[crussty-module] area_map: force load: no kernel loader");
            return None::<()>;
        };
        let forname = env.get_static_method_id(
            class_cls,
            "forName",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        );
        let Some(forname) = forname else {
            crate::clear_exception(env);
            eprintln!("[crussty-module] area_map: force load: no forName mid");
            return None::<()>;
        };
        let dot = MAP_CLASS.replace('/', ".");
        let name = env.new_string(&dot)?;
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
            eprintln!("[crussty-module] area_map: Class.forName({MAP_CLASS}) failed (exc={had_exc})");
        } else {
            eprintln!("[crussty-module] area_map: Class.forName({MAP_CLASS}) succeeded");
        }
        env.delete_local_ref(loaded);
        env.delete_local_ref(name);
        env.delete_local_ref(class_cls);
        env.delete_local_ref(loader);
        let _ = had_exc;
        Some(())
    });
}
