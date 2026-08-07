//! ASM-based classfile rewriting through an injected Java helper (M3).
//!
//! Pure-Rust patching (`weave`) cannot recompute StackMapTable frames of
//! major-65 classes. ASM (on the kernel classpath via libraries/) can. The
//! SDK defines a compiled Java helper class into the loader of a TARGET
//! class (so it resolves ASM from the kernel's scope) and delegates the
//! rewrite request to it over JNI. Frames are recomputed with
//! COMPUTE_FRAMES; the rewritten bytes are handed back for a byte hook or a
//! retransform.
//!
//! Every module carries its own SDK copy (RTLD_LOCAL => private statics), so
//! the classes it defines must be unique per .so — otherwise the second
//! module's define_class hits `LinkageError: duplicate class definition`.
//! `unique_base_name()` derives `dev/dist/SdkAsmHelper` + the address of OUR
//! OWN static (each .so gets a stable distinct name), and the embedded
//! helper class bytes are renamed to match before being defined.

use crate::jni_util::{clear_exception, with_attached};
use jvmti_bindings::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

// Compiled from asm-src/dev/dist/SdkAsmHelper.java against ASM 9.8
// (libraries/org/ow2/asm/asm/9.8/asm-9.8.jar) — see asm-build/.
const HELPER_CLASS: &[u8] = include_bytes!("../asm-build/dev/dist/SdkAsmHelper.class");
const HELPER_CLASS_1: &[u8] = include_bytes!("../asm-build/dev/dist/SdkAsmHelper$1.class");
const HELPER_CLASS_1_1: &[u8] = include_bytes!("../asm-build/dev/dist/SdkAsmHelper$1$1.class");
const HELPER_CLASS_2: &[u8] = include_bytes!("../asm-build/dev/dist/SdkAsmHelper$2.class");

const ORIG_BASE: &str = "dev/dist/SdkAsmHelper";
const ORIG_1: &str = "dev/dist/SdkAsmHelper$1";
const ORIG_1_1: &str = "dev/dist/SdkAsmHelper$1$1";
const ORIG_2: &str = "dev/dist/SdkAsmHelper$2";

/// Unique base name for THIS module's copy of the helper (and its synthetic
/// companion classes `$1`, `$1$1`, `$2`). The address of our own static
/// differs per .so (mirrors `main_thread::runnable_class_name()`).
pub fn unique_base_name() -> &'static str {
    // A per-.so static's address, NOT a function pointer: the compiler may
    // fold `&fn as usize` to a constant (observed value: 1), which would
    // collapse every module's helper onto the same name and trip the
    // duplicate-class-definition guard.
    static NAME: OnceLock<Box<str>> = OnceLock::new();
    static UNIQUE_SENTINEL: u8 = 0;
    NAME.get_or_init(|| {
        let addr = &UNIQUE_SENTINEL as *const u8 as usize;
        format!("{ORIG_BASE}{addr:x}").into_boxed_str()
    })
}

fn u16at(b: &[u8], p: usize) -> u16 {
    u16::from_be_bytes([b[p], b[p + 1]])
}

/// Rename the helper classes to module-unique names: rebuild the constant
/// pool, replacing every CONSTANT_Utf8 whose payload equals one of the
/// original names. All CP references are by index (not offset), so the
/// attribute/method section is copied verbatim and stays consistent — the
/// rewrite never moves code bytes.
type Renamed = (Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>);

fn rename_helper_classes(name: &str) -> Renamed {
    let map = HashMap::from([
        (ORIG_BASE, name.to_string()),
        (ORIG_1, format!("{name}$1")),
        (ORIG_1_1, format!("{name}$1$1")),
        (ORIG_2, format!("{name}$2")),
    ]);
    (
        rename_class(HELPER_CLASS, &map),
        rename_class(HELPER_CLASS_1, &map),
        rename_class(HELPER_CLASS_1_1, &map),
        rename_class(HELPER_CLASS_2, &map),
    )
}

/// Rename a single classfile in place-of-CP. Returns None on malformed input
/// (the embedded classes are trusted, but a corrupt build must fail loudly).
fn rename_class(src: &[u8], remap: &HashMap<&str, String>) -> Option<Vec<u8>> {
    if src.len() < 10 || u32::from_be_bytes(src[0..4].try_into().ok()?) != 0xCAFE_BABE {
        return None;
    }
    let cp_count = u16at(src, 8) as usize;
    let mut p = 10usize;
    let mut out = Vec::with_capacity(src.len() + 128);
    out.extend_from_slice(&src[..10]);
    let mut idx = 1usize;
    let pairs = desc_pairs(remap);
    while idx < cp_count {
        let start = p;
        let tag = src[p];
        p += 1;
        match tag {
            1 => {
                let l = u16at(src, p) as usize;
                p += 2;
                let payload = &src[p..p + l];
                p += l;
                let txt = std::str::from_utf8(payload).ok();
                if let Some(new) = txt.and_then(|t| remap.get(t)) {
                    out.push(1);
                    out.extend_from_slice(&(new.len() as u16).to_be_bytes());
                    out.extend_from_slice(new.as_bytes());
                } else if let Some(t) = txt {
                    // Not a bare class name: it may still be a descriptor or
                    // signature embedding the helper classes (`L<orig>;`),
                    // e.g. `$1`'s synthetic constructor
                    // `(Ldev/dist/SdkAsmHelper$1;ILorg/objectweb/asm/…;)V`.
                    // The JVM resolves those to class loads, so they must be
                    // renamed too or it looks up the ORIGINAL name.
                    let mut rw = t.to_string();
                    for (a, b) in &pairs {
                        if a != b {
                            rw = rw.replace(a.as_str(), b.as_str());
                        }
                    }
                    if rw.as_bytes() != payload {
                        out.push(1);
                        out.extend_from_slice(&(rw.len() as u16).to_be_bytes());
                        out.extend_from_slice(rw.as_bytes());
                    } else {
                        out.extend_from_slice(&src[start..p]);
                    }
                } else {
                    out.extend_from_slice(&src[start..p]);
                }
            }
            7 | 8 | 16 | 19 | 20 => {
                p += 2;
                out.extend_from_slice(&src[start..p]);
            }
            9 | 10 | 11 | 12 | 17 | 18 => {
                p += 4;
                out.extend_from_slice(&src[start..p]);
            }
            15 => {
                p += 3;
                out.extend_from_slice(&src[start..p]);
            }
            3 | 4 => {
                p += 4;
                out.extend_from_slice(&src[start..p]);
            }
            5 | 6 => {
                p += 8;
                out.extend_from_slice(&src[start..p]);
                idx += 1; // long/double occupies two slots
            }
            _ => return None,
        }
        idx += 1;
    }
    out.extend_from_slice(&src[p..]);
    Some(out)
}

/// Descriptor-form (`L<new>;`) rewrite pairs, longest-first so `$1$1` is
/// matched before `$1` (the base's `L…;` never prefixes its companions).
fn desc_pairs(remap: &HashMap<&str, String>) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = remap
        .iter()
        .map(|(orig, new)| (format!("L{orig};"), format!("L{new};")))
        .collect();
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    pairs
}

// ---------------------------------------------------------------------------
// JNI bridge: define the helper into the target loader, call rewrite()
// ---------------------------------------------------------------------------

/// Definition cache for THIS module's helper classes. Keyed on the unique
/// name we own; stores a process-lifetime global ref to the outer class.
static HELPER_GREF: OnceLock<usize> = OnceLock::new();

/// Ensure the helper classes are defined into `loader` and return a global
/// jclass ref to the outer `SdkAsmHelper` (cached per module).
///
/// `loader` must be a class loader that can resolve `org/objectweb/asm/*` —
/// i.e. the loader of the target kernel class (parent-first chain reaches the
/// system loader where libraries/ lives). Define the classes once; a second
/// define_class with the same name would throw LinkageError.
pub fn ensure_defined(env: &JniEnv, loader: jni::jobject) -> Option<jni::jclass> {
    if let Some(addr) = HELPER_GREF.get() {
        return Some(*addr as jni::jclass);
    }
    let name = unique_base_name().to_string();
    let (b, b1, b11, b2) = rename_helper_classes(&name);
    let (Some(b), Some(b1), Some(b11), Some(b2)) = (b, b1, b11, b2) else {
        eprintln!("[cplug-sdk] asm: embedded helper class corrupt (rename failed)");
        return None;
    };
    let names = [
        name.as_str(),
        &format!("{name}$1"),
        &format!("{name}$1$1"),
        &format!("{name}$2"),
    ];
    let bytes = [&b[..], &b1[..], &b11[..], &b2[..]];
    let mut outer: Option<jni::jclass> = None; // null until first define
    for (n, by) in names.iter().zip(bytes.iter()) {
        match env.define_class(n, loader, by) {
            Some(cls) => {
                if outer.is_none() {
                    outer = Some(cls);
                } else {
                    env.delete_local_ref(cls);
                }
            }
            None => {
                // log the actual reason (JNI DefineClass failure leaves a
                // pending exception worth describing).
                if env.exception_check() {
                    eprintln!("[cplug-sdk] asm: define_class({n}) threw:");
                    env.exception_describe();
                    env.exception_clear();
                } else {
                    eprintln!("[cplug-sdk] asm: define_class({n}) returned null");
                }
                return None;
            }
        }
    }
    // The classes are returned by DefineClass, so there's no need to locale
    // them again — `FindClass` would use the calling thread's context loader
    // (native-attached threads have a context that can't see the kernel's
    // loader), and a re-find of a just-defined class may also hit race with
    // redefine. Use the local ref the define gave us directly.
    let Some(outer) = outer else {
        eprintln!("[cplug-sdk] asm: no outer class from define_class");
        return None;
    };
    let gref = env.new_global_ref(outer);
    env.delete_local_ref(outer);
    HELPER_GREF
        .set(gref as usize)
        .map_err(|_| ())
        .expect("helper defined twice");
    warmup(env, gref);
    Some(gref)
}

/// Execute one trivial MAK_FIELDS_PUBLIC rewrite through the helper so ASM
/// classes (`org/objectweb/asm/ClassReader`, `ClassWriter`, the MethodVisitor
/// and ClassVisitor companions) get resolved NOW, on the defining thread —
/// never lazily from inside a class-file hook / retransform callback where a
/// class definition under the kernel loader's lock can deadlock.
fn warmup(env: &JniEnv, gref: jni::jclass) {
    let _ = env
        .get_static_method_id(gref, "rewrite", "([B[B)[B")
        .and_then(|mid| {
            // op MAKE_FIELDS_PUBLIC, zero fields: parses and re-emits the
            // input through ASM. Feed it our own (valid) helper class bytes.
            let spec = [1u8, 2, 0];
            let spec_arr = env.new_byte_array(3)?;
            let spec_signed: Vec<i8> = spec.iter().map(|&b| b as i8).collect();
            env.set_byte_array_region(spec_arr, 0, 3, &spec_signed);
            let src_arr = env.new_byte_array(HELPER_CLASS.len() as i32)?;
            let src_signed: Vec<i8> = HELPER_CLASS.iter().map(|&b| b as i8).collect();
            env.set_byte_array_region(src_arr, 0, HELPER_CLASS.len() as i32, &src_signed);
            let _res = env.call_static_object_method(
                gref,
                mid,
                &[
                    jni::jvalue { l: src_arr },
                    jni::jvalue { l: spec_arr },
                ],
            );
            env.delete_local_ref(spec_arr);
            env.delete_local_ref(src_arr);
            Some(())
        });
    clear_exception(env);
}

/// Spec for the REPLACE_BODY rewrite (op 1): swap a method's body for
/// `invokestatic bridgeOwner.bridgeName(bridgeDesc)` where each arg is
/// either a local slot of the patched method or a field read of `this`.
pub struct ReplaceBody<'a> {
    pub method_name: &'a str,
    pub method_desc: &'a str,
    pub bridge_owner: &'a str,
    pub bridge_name: &'a str,
    pub bridge_desc: &'a str,
    /// Argument sources in call order (must match `bridge_desc`'s parameter
    /// types exactly).
    pub args: &'a [ArgSpec<'a>],
}

/// One bridge-call argument. `Local` loads a local variable of the patched
/// method (slot 0 = `this` for instance methods) with a JVM type char
/// (I/J/D/F/Z/L/[) selecting the load opcode. `ThisField` emits
/// `aload 0; getfield <patched-class>.<name>:<desc>` — reading a PRIVATE
/// field of the patched class from inside its own method is legal, while
/// changing the field's access flags in the retransformed class file is
/// not (the JVM rejects it with JVMTI_ERROR_UNSUPPORTED_REDEFINITION_SCHEMA_CHANGED).
#[derive(Clone, Copy, Debug)]
pub enum ArgSpec<'a> {
    Local { slot: u16, ty: u8 },
    ThisField { name: &'a str, desc: &'a str },
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn serialize(spec: &ReplaceBody) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 + 8 * spec.args.len());
    out.push(1); // version
    out.push(1); // op REPLACE_BODY
    push_str(&mut out, spec.method_name);
    push_str(&mut out, spec.method_desc);
    push_str(&mut out, spec.bridge_owner);
    push_str(&mut out, spec.bridge_name);
    push_str(&mut out, spec.bridge_desc);
    out.push(spec.args.len() as u8);
    for arg in spec.args {
        match arg {
            ArgSpec::Local { slot, ty } => {
                out.extend_from_slice(&slot.to_be_bytes());
                out.push(*ty);
            }
            ArgSpec::ThisField { name, desc } => {
                out.extend_from_slice(&0u16.to_be_bytes());
                out.push(b'@');
                push_str(&mut out, name);
                push_str(&mut out, desc);
            }
        }
    }
    out
}

/// Apply a REPLACE_BODY rewrite to `class_bytes` through the Java helper.
/// Returns the rewritten class bytes, or None (lastError() printed on
/// failure). The helper class must already be defined in `loader`
/// (see `ensure_defined`).
pub fn replace_body(
    env: &JniEnv,
    loader: jni::jobject,
    class_bytes: &[u8],
    spec: &ReplaceBody,
) -> Option<Vec<u8>> {
    let cls = ensure_defined(env, loader)?;
    let mid = env.get_static_method_id(cls, "rewrite", "([B[B)[B")?;
    let spec_bytes = serialize(spec);
    let src_arr = env.new_byte_array(class_bytes.len() as i32)?;
    let src_signed: Vec<i8> = class_bytes.iter().map(|&b| b as i8).collect();
    env.set_byte_array_region(src_arr, 0, class_bytes.len() as i32, &src_signed);
    let spec_arr = env.new_byte_array(spec_bytes.len() as i32)?;
    let spec_signed: Vec<i8> = spec_bytes.iter().map(|&b| b as i8).collect();
    env.set_byte_array_region(spec_arr, 0, spec_bytes.len() as i32, &spec_signed);
    let res = env.call_static_object_method(cls, mid, &[
        jni::jvalue { l: src_arr },
        jni::jvalue { l: spec_arr },
    ]);
    let had_exc = clear_exception(env);
    env.delete_local_ref(spec_arr);
    env.delete_local_ref(src_arr);
    if res.is_null() {
        if !had_exc {
            // helper returned null: fetch lastError() for the log
            let _ = with_attached(|env| {
                if let Some(e) = env.get_static_method_id(cls, "lastError", "()Ljava/lang/String;") {
                    let err = env.call_static_object_method(cls, e, &[]);
                    if !err.is_null() {
                        if let Some(msg) = env.get_string_utf(err as jni::jstring) {
                            eprintln!("[cplug-sdk] asm::replace_body failed: {msg}");
                        }
                        env.delete_local_ref(err);
                    }
                }
                Some(())
            });
        }
        return None;
    }
    let arr = res as jni::jbyteArray;
    let len = env.get_array_length(arr as jni::jarray);
    let mut signed = vec![0i8; len as usize];
    env.get_byte_array_region(arr, 0, len, &mut signed);
    env.delete_local_ref(arr);
    Some(signed.iter().map(|&b| b as u8).collect())
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_preserves_structure_and_changes_this_class() {
        let name = format!("{ORIG_BASE}abc123");
        let (renamed, renamed1, renamed11, renamed2) = rename_helper_classes(&name);
        let (renamed, renamed1, renamed11, renamed2) = match (renamed, renamed1, renamed11, renamed2) {
            (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
            _ => panic!("rename failed"),
        };
        assert!(
            !renamed.is_empty()
                && !renamed1.is_empty()
                && !renamed11.is_empty()
                && !renamed2.is_empty()
        );

        // this_class name must now be the unique name; CP count unchanged.
        for (orig, re) in [
            (HELPER_CLASS, &renamed),
            (HELPER_CLASS_1, &renamed1),
            (HELPER_CLASS_1_1, &renamed11),
            (HELPER_CLASS_2, &renamed2),
        ] {
            assert_eq!(u16at(orig, 8), u16at(re, 8), "cp_count changed");
            // after the CP come access_flags(2) then this_class (a CP index)
            let cp_bytes = cp_section_len(re);
            let this_idx = u16at(re, 10 + cp_bytes + 2) as usize;
            let this_name = class_name_at(re, this_idx);
            assert!(
                this_name.starts_with(&name),
                "this_class {this_name} != {name}"
            );
            assert!(
                !this_name.contains("SdkAsmHelper$1"),
                "companion leaked into this_class"
            );
        }
    }

    #[test]
    fn rename_removes_original_descriptor_refs() {
        // $1/$1$1 carry synthetic constructors whose descriptors embed the
        // helper classes as `Ldev/dist/SdkAsmHelper$1;` etc. These must be
        // renamed as well: the JVM resolves them to class loads at verify
        // time and would otherwise fail with NoClassDefFoundError.
        let name = format!("{ORIG_BASE}abc123");
        let (renamed, renamed1, renamed11, renamed2) = rename_helper_classes(&name);
        let (renamed, renamed1, renamed11, renamed2) = match (renamed, renamed1, renamed11, renamed2) {
            (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
            _ => panic!("rename failed"),
        };
        // Every Utf8 in each renamed class: bare original names and
        // `L<orig>;` descriptor forms must be gone.
        for b in [&renamed, &renamed1, &renamed11, &renamed2] {
            let count = u16at(b, 8) as usize;
            let mut p = 10usize;
            let mut idx = 1usize;
            while idx < count {
                let tag = b[p];
                p += 1;
                if tag == 1 {
                    let l = u16at(b, p) as usize;
                    p += 2;
                    let u = String::from_utf8_lossy(&b[p..p + l]).into_owned();
                    p += l;
                    for orig in [ORIG_1, ORIG_1_1, ORIG_2] {
                        assert!(
                            u != orig && !u.contains(&format!("L{orig};")),
                            "surviving original ref {orig} in `{u}`"
                        );
                    }
                } else {
                    match tag {
                        7 | 8 | 16 | 19 | 20 => p += 2,
                        9 | 10 | 11 | 12 | 17 | 18 => p += 4,
                        15 => p += 3,
                        3 | 4 => p += 4,
                        5 | 6 => {
                            p += 8;
                            idx += 1;
                        }
                        _ => panic!("corrupt cp tag {tag}"),
                    }
                }
                idx += 1;
            }
        }
    }

    #[test]
    fn embedded_helper_is_wellformed() {
        for (label, b) in [
            ("outer", HELPER_CLASS),
            ("$1", HELPER_CLASS_1),
            ("$1$1", HELPER_CLASS_1_1),
            ("$2", HELPER_CLASS_2),
        ] {
            assert!(
                u32::from_be_bytes(b[0..4].try_into().unwrap()) == 0xCAFE_BABE,
                "{label} magic"
            );
            assert!(u16at(b, 8) >= 3, "{label} empty cp");
        }
    }

    fn cp_section_len(b: &[u8]) -> usize {
        // walk the CP exactly like rename_class to find where it ends
        let count = u16at(b, 8) as usize;
        let mut p = 10usize;
        let mut idx = 1usize;
        while idx < count {
            let tag = b[p];
            p += 1;
            match tag {
                1 => {
                    let l = u16at(b, p) as usize;
                    p += 2 + l;
                }
                7 | 8 | 16 | 19 | 20 => p += 2,
                9 | 10 | 11 | 12 | 17 | 18 => p += 4,
                15 => p += 3,
                3 | 4 => p += 4,
                5 | 6 => {
                    p += 8;
                    idx += 1;
                }
                _ => return 0,
            }
            idx += 1;
        }
        p - 10
    }

    /// Resolve the name of the class referenced by CP index `class_idx`
    /// (`CONSTANT_Class` -> `CONSTANT_Utf8`).
    fn class_name_at(b: &[u8], class_idx: usize) -> String {
        let count = u16at(b, 8) as usize;
        let mut p = 10usize;
        let mut idx = 1usize;
        while idx < count {
            let entry_pos = p;
            let tag = b[p];
            p += 1;
            match tag {
                1 => {
                    let l = u16at(b, p) as usize;
                    p += 2 + l;
                }
                7 | 8 | 16 | 19 | 20 => {
                    if tag == 7 && idx == class_idx {
                        let name_idx = u16at(b, entry_pos + 1) as usize;
                        return utf8_at(b, name_idx);
                    }
                    p += 2;
                }
                9 | 10 | 11 | 12 | 17 | 18 => p += 4,
                15 => p += 3,
                3 | 4 => p += 4,
                5 | 6 => {
                    p += 8;
                    idx += 1;
                }
                _ => return String::new(),
            }
            idx += 1;
        }
        String::new()
    }

    fn utf8_at(b: &[u8], utf8_idx: usize) -> String {
        let count = u16at(b, 8) as usize;
        let mut p = 10usize;
        let mut idx = 1usize;
        while idx < count {
            let tag = b[p];
            p += 1;
            match tag {
                1 => {
                    let l = u16at(b, p) as usize;
                    p += 2;
                    let txt = String::from_utf8_lossy(&b[p..p + l]).into_owned();
                    if idx == utf8_idx {
                        return txt;
                    }
                    p += l;
                }
                7 | 8 | 16 | 19 | 20 => p += 2,
                9 | 10 | 11 | 12 | 17 | 18 => p += 4,
                15 => p += 3,
                3 | 4 => p += 4,
                5 | 6 => {
                    p += 8;
                    idx += 1;
                }
                _ => return String::new(),
            }
            idx += 1;
        }
        String::new()
    }
}
