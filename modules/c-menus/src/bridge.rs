//! JNI bridge to dev.crussty.cmenus.Bridge.
//!
//! The Bridge class is compiled at build time (scripts/build_bridge.sh →
//! bridge-build/dev/crussty/cmenus/Bridge.class, committed), embedded here,
//! defined ONCE in the kernel's class loader (the loader of
//! org.bukkit.Bukkit, so its org.bukkit references resolve) and invoked via
//! cached static method ids.
//!
//! All wrappers take a &JniEnv (main thread) and never panic: JNI failures
//! return Option/default. Pending exceptions are always cleared.

use jvmti_bindings::prelude::*;
use std::sync::OnceLock;

use cplug_sdk::classes::{find_class, method, ClassRef};
use cplug_sdk::{claim, jni_util::clear_exception};

const BRIDGE_NAME: &str = "dev/crussty/cmenus/Bridge";
const BRIDGE_BYTES: &[u8] = include_bytes!("../bridge-build/dev/crussty/cmenus/Bridge.class");

#[allow(dead_code)]
pub const BILLBOARD_FIXED: u8 = 0;
#[allow(dead_code)]
pub const BILLBOARD_CENTER: u8 = 1;

#[allow(dead_code)]
struct BridgeIds {
    cls: ClassRef,
    spawn_text: usize,
    spawn_item: usize,
    spawn_vanilla: usize,
    remove: usize,
    valid: usize,
    transform: usize,
    teleport: usize,
    props: usize,
    billboard: usize,
    text: usize,
    item_cmd: usize,
    eye: usize,
    loc: usize,
    snapshot: usize,
    by_uuid: usize,
    world_name: usize,
    world_obj: usize,
    update_inv: usize,
    send_pack: usize,
    msg: usize,
    cmd: usize,
    sound: usize,
}

static BRIDGE: OnceLock<Option<BridgeIds>> = OnceLock::new();

/// Define + cache the Bridge (idempotent). Must run on the main thread after
/// the kernel is up. Returns false when the kernel classes are unreachable.
pub fn ensure(env: &JniEnv) -> bool {
    if let Some(Some(_)) = BRIDGE.get() {
        return true;
    }
    let built = define(env);
    let ok = built.is_some();
    let _ = BRIDGE.set(built);
    ok
}

fn define(env: &JniEnv) -> Option<BridgeIds> {
    // Resolve the Bukkit class loader so Bridge's org.bukkit references work.
    let bukkit = find_class("org/bukkit/Bukkit")?;
    let class_cls = find_class("java/lang/Class")?;
    let get_loader = method(env, class_cls.as_jclass(), "getClassLoader", "()Ljava/lang/ClassLoader;")?;
    let loader = env.call_object_method(bukkit.as_jclass(), get_loader as jni::jmethodID, &[]);
    let _ = clear_exception(env);
    if loader.is_null() {
        return None;
    }

    let name = BRIDGE_NAME;
    if !claim(&format!("class:{name}")) {
        // Another module already defined our class name — find and reuse.
        let _ = clear_exception(env);
    }
    let local = env.define_class(name, loader, BRIDGE_BYTES)?;
    let gref = env.new_global_ref(local);
    env.delete_local_ref(local);
    let _ = clear_exception(env);
    if gref.is_null() {
        return None;
    }
    let cls = ClassRef(gref);

    let jc = cls.0;
    let m = |n: &str, s: &str| -> usize {
        let mid = env.get_static_method_id(jc, n, s);
        if mid.is_none() {
            let _ = clear_exception(env);
            eprintln!("[c-menus] bridge method missing: {n}{s}");
        }
        mid.map(|m| m as usize).unwrap_or(0)
    };

    Some(BridgeIds {
        cls,
        spawn_text: m("spawnTextDisplay", "(Ljava/lang/Object;DDD)Ljava/lang/Object;"),
        spawn_item: m("spawnItemDisplay", "(Ljava/lang/Object;DDDI)Ljava/lang/Object;"),
        spawn_vanilla: m("spawnItemVanilla", "(Ljava/lang/Object;DDDLjava/lang/String;)Ljava/lang/Object;"),
        remove: m("removeEntity", "(Ljava/lang/Object;)V"),
        valid: m("entityValid", "(Ljava/lang/Object;)I"),
        transform: m("applyTransform", "(Ljava/lang/Object;DDDDDDFFFII)V"),
        teleport: m("teleport", "(Ljava/lang/Object;DDDI)V"),
        props: m("displayProps", "(Ljava/lang/Object;FFFFIIZ)V"),
        billboard: m("billboard", "(Ljava/lang/Object;I)V"),
        text: m("applyText", "(Ljava/lang/Object;Ljava/lang/String;IZIIIB)V"),
        item_cmd: m("itemCmd", "(Ljava/lang/Object;I)V"),
        eye: m("playerEye", "(Ljava/lang/Object;)Ljava/lang/String;"),
        loc: m("playerLoc", "(Ljava/lang/Object;)Ljava/lang/String;"),
        snapshot: m("playerSnapshot", "()Ljava/lang/String;"),
        by_uuid: m("playerByUuid", "(Ljava/lang/String;)Ljava/lang/Object;"),
        world_name: m("playerWorld", "(Ljava/lang/Object;)Ljava/lang/String;"),
        world_obj: m("playerWorldObj", "(Ljava/lang/Object;)Ljava/lang/Object;"),
        update_inv: m("updateInv", "(Ljava/lang/Object;)V"),
        send_pack: m("sendPack", "(Ljava/lang/Object;Ljava/lang/String;Ljava/lang/String;Z)V"),
        msg: m("msg", "(Ljava/lang/Object;Ljava/lang/String;)V"),
        cmd: m("consoleCmd", "(Ljava/lang/String;)V"),
        sound: m("sound", "(Ljava/lang/Object;Ljava/lang/String;FF)V"),
    })
}

fn ids() -> Option<&'static BridgeIds> {
    match BRIDGE.get() {
        Some(Some(b)) => Some(b),
        _ => None,
    }
}

fn jstr<'a>(env: &JniEnv, s: &str) -> jni::jstring {
    env.new_string_utf(s).unwrap_or(std::ptr::null_mut())
}

fn obj(v: jni::jobject) -> jni::jvalue {
    jni::jvalue { l: v }
}
fn dbl(v: f64) -> jni::jvalue {
    jni::jvalue { d: v }
}
fn flt(v: f32) -> jni::jvalue {
    jni::jvalue { f: v }
}
fn int(v: i32) -> jni::jvalue {
    jni::jvalue { i: v }
}
fn boolean(v: bool) -> jni::jvalue {
    jni::jvalue { z: v as u8 }
}

// ---------------------------------------------------------------------------
// Wrappers
// ---------------------------------------------------------------------------

pub fn spawn_text(env: &JniEnv, world: jni::jobject, x: f64, y: f64, z: f64) -> Option<jni::jobject> {
    let b = ids()?;
    let r = env.call_static_object_method(
        b.cls.0,
        b.spawn_text as jni::jmethodID,
        &[obj(world), dbl(x), dbl(y), dbl(z)],
    );
    let ok = !r.is_null();
    let _ = clear_exception(env);
    ok.then_some(r)
}

pub fn spawn_item(env: &JniEnv, world: jni::jobject, x: f64, y: f64, z: f64, cmd: i32) -> Option<jni::jobject> {
    let b = ids()?;
    let r = env.call_static_object_method(
        b.cls.0,
        b.spawn_item as jni::jmethodID,
        &[obj(world), dbl(x), dbl(y), dbl(z), int(cmd)],
    );
    let _ = clear_exception(env);
    (!r.is_null()).then_some(r)
}

pub fn spawn_vanilla(env: &JniEnv, world: jni::jobject, x: f64, y: f64, z: f64, material: &str) -> Option<jni::jobject> {
    let b = ids()?;
    let s = jstr(env, material);
    let r = env.call_static_object_method(
        b.cls.0,
        b.spawn_vanilla as jni::jmethodID,
        &[obj(world), dbl(x), dbl(y), dbl(z), obj(s)],
    );
    env.delete_local_ref(s);
    let _ = clear_exception(env);
    (!r.is_null()).then_some(r)
}

pub fn remove(env: &JniEnv, ent: jni::jobject) {
    if let Some(b) = ids() {
        env.call_static_void_method(b.cls.0, b.remove as jni::jmethodID, &[obj(ent)]);
        let _ = clear_exception(env);
    }
}

#[allow(dead_code)]
pub fn valid(env: &JniEnv, ent: jni::jobject) -> bool {
    let Some(b) = ids() else { return false };
    let r = env.call_static_int_method(b.cls.0, b.valid as jni::jmethodID, &[obj(ent)]);
    let _ = clear_exception(env);
    r != 0
}

/// Full display transform update with client interpolation.
pub struct Xform {
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
    pub sx: f64,
    pub sy: f64,
    pub sz: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub interp_delay: i32,
    pub interp_dur: i32,
}

pub fn apply_transform(env: &JniEnv, ent: jni::jobject, t: &Xform) {
    if let Some(b) = ids() {
        env.call_static_void_method(
            b.cls.0,
            b.transform as jni::jmethodID,
            &[
                obj(ent),
                dbl(t.tx), dbl(t.ty), dbl(t.tz),
                dbl(t.sx), dbl(t.sy), dbl(t.sz),
                flt(t.yaw), flt(t.pitch), flt(t.roll),
                int(t.interp_delay), int(t.interp_dur),
            ],
        );
        let _ = clear_exception(env);
    }
}

pub fn teleport(env: &JniEnv, ent: jni::jobject, x: f64, y: f64, z: f64, dur: i32) {
    if let Some(b) = ids() {
        env.call_static_void_method(
            b.cls.0,
            b.teleport as jni::jmethodID,
            &[obj(ent), dbl(x), dbl(y), dbl(z), int(dur)],
        );
        let _ = clear_exception(env);
    }
}

pub fn props(env: &JniEnv, ent: jni::jobject, view_range: f64, shadow_r: f64, shadow_s: f64, brightness: [i32; 2]) {
    if let Some(b) = ids() {
        env.call_static_void_method(
            b.cls.0,
            b.props as jni::jmethodID,
            &[
                obj(ent),
                flt(view_range as f32),
                flt(shadow_r as f32),
                flt(shadow_s as f32),
                int(brightness[0]),
                int(brightness[1]),
                boolean(false),
            ],
        );
        let _ = clear_exception(env);
    }
}

pub fn billboard(env: &JniEnv, ent: jni::jobject, mode: u8) {
    if let Some(b) = ids() {
        env.call_static_void_method(b.cls.0, b.billboard as jni::jmethodID, &[obj(ent), int(mode as i32)]);
        let _ = clear_exception(env);
    }
}

pub fn apply_text(
    env: &JniEnv,
    ent: jni::jobject,
    legacy: &str,
    bg_argb: u32,
    see_through: bool,
    shadow: bool,
    line_width: i32,
    align: u8,
) {
    if let Some(b) = ids() {
        let s = jstr(env, legacy);
        env.call_static_void_method(
            b.cls.0,
            b.text as jni::jmethodID,
            &[
                obj(ent),
                obj(s),
                int(bg_argb as i32),
                boolean(see_through),
                boolean(shadow),
                int(line_width),
                int(align as i32),
                int(255), // opacity byte: leave fully opaque
            ],
        );
        env.delete_local_ref(s);
        let _ = clear_exception(env);
    }
}

#[allow(dead_code)]
pub fn item_cmd(env: &JniEnv, ent: jni::jobject, cmd: i32) {
    if let Some(b) = ids() {
        env.call_static_void_method(b.cls.0, b.item_cmd as jni::jmethodID, &[obj(ent), int(cmd)]);
        let _ = clear_exception(env);
    }
}

/// Eye transform "x,y,z,yaw,pitch".
pub fn player_eye(env: &JniEnv, player: jni::jobject) -> Option<[f64; 5]> {
    let b = ids()?;
    let r = env.call_static_object_method(b.cls.0, b.eye as jni::jmethodID, &[obj(player)]);
    let _ = clear_exception(env);
    parse_csv_obj(env, r)
}

#[allow(dead_code)]
pub fn player_loc(env: &JniEnv, player: jni::jobject) -> Option<[f64; 5]> {
    let b = ids()?;
    let r = env.call_static_object_method(b.cls.0, b.loc as jni::jmethodID, &[obj(player)]);
    let _ = clear_exception(env);
    parse_csv_obj(env, r)
}

fn parse_csv_obj(env: &JniEnv, r: jni::jobject) -> Option<[f64; 5]> {
    if r.is_null() {
        return None;
    }
    let s = env.get_string_utf(r as jni::jstring);
    env.delete_local_ref(r);
    let s = s?;
    let mut it = s.split(',');
    let mut out = [0.0f64; 5];
    for v in out.iter_mut() {
        *v = it.next()?.parse().ok()?;
    }
    Some(out)
}

/// All online players as (uuid, name, world).
pub fn snapshot(env: &JniEnv) -> Vec<(String, String, String)> {
    let Some(b) = ids() else { return Vec::new() };
    let r = env.call_static_object_method(b.cls.0, b.snapshot as jni::jmethodID, &[]);
    let _ = clear_exception(env);
    if r.is_null() {
        return Vec::new();
    }
    let s = env.get_string_utf(r as jni::jstring);
    env.delete_local_ref(r);
    let mut out = Vec::new();
    if let Some(s) = s {
        for rec in s.split(';') {
            if rec.is_empty() {
                continue;
            }
            let mut f = rec.split('|');
            if let (Some(u), Some(n)) = (f.next(), f.next()) {
                let w = f.next().unwrap_or("");
                out.push((u.to_string(), n.to_string(), w.to_string()));
            }
        }
    }
    out
}

pub fn player_by_uuid(env: &JniEnv, uuid: &str) -> Option<jni::jobject> {
    let b = ids()?;
    let s = jstr(env, uuid);
    let r = env.call_static_object_method(b.cls.0, b.by_uuid as jni::jmethodID, &[obj(s)]);
    env.delete_local_ref(s);
    let _ = clear_exception(env);
    (!r.is_null()).then_some(r)
}

pub fn player_world_name(env: &JniEnv, player: jni::jobject) -> String {
    let Some(b) = ids() else { return String::new() };
    let r = env.call_static_object_method(b.cls.0, b.world_name as jni::jmethodID, &[obj(player)]);
    let _ = clear_exception(env);
    if r.is_null() {
        return String::new();
    }
    let s = env.get_string_utf(r as jni::jstring);
    env.delete_local_ref(r);
    s.unwrap_or_default()
}

pub fn player_world_obj(env: &JniEnv, player: jni::jobject) -> Option<jni::jobject> {
    let b = ids()?;
    let r = env.call_static_object_method(b.cls.0, b.world_obj as jni::jmethodID, &[obj(player)]);
    let _ = clear_exception(env);
    (!r.is_null()).then_some(r)
}

pub fn update_inv(env: &JniEnv, player: jni::jobject) {
    if let Some(b) = ids() {
        env.call_static_void_method(b.cls.0, b.update_inv as jni::jmethodID, &[obj(player)]);
        let _ = clear_exception(env);
    }
}

pub fn send_pack(env: &JniEnv, player: jni::jobject, url: &str, sha1: &str, force: bool) {
    if let Some(b) = ids() {
        let u = jstr(env, url);
        let h = jstr(env, sha1);
        env.call_static_void_method(
            b.cls.0,
            b.send_pack as jni::jmethodID,
            &[obj(player), obj(u), obj(h), boolean(force)],
        );
        env.delete_local_ref(u);
        env.delete_local_ref(h);
        let _ = clear_exception(env);
    }
}

pub fn msg(env: &JniEnv, player: jni::jobject, legacy: &str) {
    if let Some(b) = ids() {
        let s = jstr(env, legacy);
        env.call_static_void_method(b.cls.0, b.msg as jni::jmethodID, &[obj(player), obj(s)]);
        env.delete_local_ref(s);
        let _ = clear_exception(env);
    }
}

pub fn console_cmd(env: &JniEnv, cmd: &str) {
    if let Some(b) = ids() {
        let s = jstr(env, cmd);
        env.call_static_void_method(b.cls.0, b.cmd as jni::jmethodID, &[obj(s)]);
        env.delete_local_ref(s);
        let _ = clear_exception(env);
    }
}

pub fn sound(env: &JniEnv, player: jni::jobject, key: &str, volume: f32, pitch: f32) {
    if let Some(b) = ids() {
        let s = jstr(env, key);
        env.call_static_void_method(
            b.cls.0,
            b.sound as jni::jmethodID,
            &[obj(player), obj(s), flt(volume), flt(pitch)],
        );
        env.delete_local_ref(s);
        let _ = clear_exception(env);
    }
}
