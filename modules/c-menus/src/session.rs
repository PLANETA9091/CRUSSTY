//! Per-player menu sessions: entity lifecycle, per-tick animation update,
//! hover raycast, input routing, page switching.
//!
//! All JNI work happens on the main thread (the tick pump or the input
//! drain). Entity jobject references are stored as raw usize global refs and
//! are only dereferenced there.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use jvmti_bindings::prelude::*;

use crate::bridge::{self, Xform};
use crate::easing::{loop_wave, loop_wave01, segment_progress, Ease};
use crate::geom::{menu_to_world, Aabb, Basis, Ray, Vec3};
use crate::input::{Btn, InputEvent, PACK_ACCEPTED, PACK_DOWNLOADED};
use crate::menu::{legacy, parse_argb, Action, AnchorMode, Element, MenuDef};
use crate::App;

const TELEPORT_DUR: i32 = 2;
const INTERP_DUR: i32 = 2;
const HIDE_SCALE: f64 = 1e-5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SState {
    Open,
    Closing,
}

pub struct Session {
    pub menu: Arc<MenuDef>,
    pub state: SState,
    pub opened_tick: u64,
    pub close_started: u64,
    pub page: i32,
    pub hovered: Option<usize>,
    pub hover_t: Vec<f64>,
    pub hover_fired: Vec<bool>,
    /// jobject global ref of the player's world.
    pub world: usize,
    pub world_name: String,
    /// jobject global refs, one per element (0 = spawn failed).
    pub ents: Vec<usize>,
    pub last_bg: Vec<u32>,
    pub last_sneak: Option<bool>,
    /// Fixed anchor for world-mode menus.
    pub anchor_world: Option<Vec3>,
    pub anchor_yaw: f64,
}

pub struct SessionMgr {
    pub sessions: HashMap<String, Session>,
    /// Players we already pushed the resource pack to.
    pub pack_sent: HashSet<String>,
}

impl SessionMgr {
    pub fn new() -> Self {
        SessionMgr {
            sessions: HashMap::new(),
            pack_sent: HashSet::new(),
        }
    }

    fn anchor_at(menu: &MenuDef, eye: [f64; 5]) -> (Vec3, f64) {
        let b = Basis::from_yaw_deg(eye[3] + menu.anchor.yaw_offset);
        let p = match menu.anchor.mode {
            AnchorMode::Hud => Vec3::new(eye[0], eye[1], eye[2])
                .add(Vec3::new(b.fx * menu.anchor.distance, menu.anchor.height, b.fz * menu.anchor.distance)),
            AnchorMode::World => Vec3::new(eye[0], eye[1], eye[2]),
        };
        (p, eye[3] + menu.anchor.yaw_offset)
    }

    // -------------------------------------------------------------------
    // Open / close
    // -------------------------------------------------------------------

    pub fn open(&mut self, env: &JniEnv, app: &App, uuid: &str, menu_id: &str, tick: u64) -> bool {
        let Some(menu) = app.menus_read().get(menu_id).cloned() else {
            if let Some(player) = bridge::player_by_uuid(env, uuid) {
                bridge::msg(env, player, &format!("&c[c-menus] menu not found: &f{menu_id}"));
                env.delete_local_ref(player);
            }
            return false;
        };
        self.close_instant(env, uuid);
        let Some(player) = bridge::player_by_uuid(env, uuid) else {
            return false;
        };
        let Some(world) = bridge::player_world_obj(env, player) else {
            env.delete_local_ref(player);
            return false;
        };
        let world_name = bridge::player_world_name(env, player);
        let world_g = env.new_global_ref(world);
        env.delete_local_ref(world);
        let Some(eye) = bridge::player_eye(env, player) else {
            env.delete_global_ref(world_g);
            env.delete_local_ref(player);
            return false;
        };
        let (base_pos, yaw) = Self::anchor_at(&menu, eye);
        let b = Basis::from_yaw_deg(yaw);

        let n = menu.elements.len();
        let mut ents = Vec::with_capacity(n);
        let mut last_bg = Vec::with_capacity(n);

        for el in &menu.elements {
            let p = menu_to_world(base_pos, b, el.pos[0], el.pos[1], el.pos[2]);
            let spawned = match el.kind.kind.as_str() {
                "text" => bridge::spawn_text(env, world_g, p.x, p.y, p.z),
                "model" => {
                    let cmd = crate::menu::model_cmd(&el.kind.model).unwrap_or(1001);
                    bridge::spawn_item(env, world_g, p.x, p.y, p.z, cmd)
                }
                "item" => {
                    let item = &el.kind.item;
                    let mat = if item.contains(':') { item.clone() } else { format!("minecraft:{item}") };
                    bridge::spawn_vanilla(env, world_g, p.x, p.y, p.z, &mat)
                }
                _ => None,
            };
            match spawned {
                Some(e) => {
                    let g = env.new_global_ref(e);
                    env.delete_local_ref(e);
                    bridge::props(env, g, el.view_range, el.shadow_radius, el.shadow_strength, el.brightness);
                    bridge::billboard(env, g, el.billboard);
                    let initial_bg;
                    if el.kind.kind == "text" {
                        let base = parse_argb(&el.kind.bg);
                        let start_mult = el.anim.enter.as_ref().map(|en| en.from.bg_alpha).unwrap_or(1.0);
                        initial_bg = scaled_bg(base, start_mult, 0.0, false);
                        bridge::apply_text(
                            env,
                            g,
                            &legacy(&el.kind.text),
                            initial_bg,
                            el.kind.see_through,
                            el.kind.shadow,
                            el.kind.line_width,
                            align_num(&el.kind.align),
                        );
                    } else {
                        initial_bg = 0;
                    }
                    last_bg.push(initial_bg);
                    // initial transform: the enter "from" state
                    let (sm, off, rot_extra, _) = enter_state(el, 0.0);
                    let s0 = el.scale[0] * sm;
                    let sy0 = el.scale[1] * sm;
                    let sz0 = el.scale[2] * sm;
                    bridge::apply_transform(
                        env,
                        g,
                        &Xform {
                            tx: off[0], ty: off[1], tz: off[2],
                            sx: s0.max(HIDE_SCALE), sy: sy0.max(HIDE_SCALE), sz: sz0.max(HIDE_SCALE),
                            yaw: (el.rot[1] + rot_extra) as f32,
                            pitch: el.rot[0] as f32,
                            roll: el.rot[2] as f32,
                            interp_delay: 0,
                            interp_dur: 0,
                        },
                    );
                    ents.push(g as usize);
                }
                None => {
                    eprintln!("[c-menus] failed to spawn element {} for {}", el.id, uuid);
                    ents.push(0);
                    last_bg.push(0);
                }
            }
        }

        let s = Session {
            page: 0,
            menu,
            state: SState::Open,
            opened_tick: tick,
            close_started: 0,
            hovered: None,
            hover_t: vec![0.0; n],
            hover_fired: vec![false; n],
            world: world_g as usize,
            world_name,
            ents,
            last_bg,
            last_sneak: None,
            anchor_world: match base_pos {
                p => Some(p),
            },
            anchor_yaw: yaw,
        };
        run_actions(env, app, uuid, player, &s.menu.on_open);
        env.delete_local_ref(player);
        self.sessions.insert(uuid.to_string(), s);
        true
    }

    pub fn close(&mut self, env: &JniEnv, app: &App, uuid: &str, tick: u64) {
        if let Some(s) = self.sessions.get_mut(uuid) {
            if s.state == SState::Open {
                s.state = SState::Closing;
                s.close_started = tick;
                s.hovered = None;
                if let Some(player) = bridge::player_by_uuid(env, uuid) {
                    run_actions(env, app, uuid, player, &s.menu.on_close);
                    env.delete_local_ref(player);
                }
            }
        }
    }

    /// Remove entities immediately (no exit animation).
    pub fn close_instant(&mut self, env: &JniEnv, uuid: &str) {
        if let Some(mut s) = self.sessions.remove(uuid) {
            for e in s.ents.drain(..) {
                if e != 0 {
                    let o = e as jni::jobject;
                    bridge::remove(env, o);
                    env.delete_global_ref(o);
                }
            }
            if s.world != 0 {
                env.delete_global_ref(s.world as jni::jobject);
            }
        }
    }

    // -------------------------------------------------------------------
    // Per-tick update
    // -------------------------------------------------------------------

    pub fn tick(&mut self, env: &JniEnv, app: &App, tick: u64) {
        let now_online: HashSet<String> =
            bridge::snapshot(env).into_iter().map(|(u, _, _)| u).collect();

        // teardown sessions of players that left
        let gone: Vec<String> = self
            .sessions
            .keys()
            .filter(|u| !now_online.contains(*u))
            .cloned()
            .collect();
        for u in gone {
            self.close_instant(env, &u);
        }
        self.pack_sent.retain(|u| now_online.contains(u));

        // resource pack on first sighting
        if app.cfg_read().pack.send_on_join {
            for uuid in &now_online {
                if self.pack_sent.insert(uuid.clone()) {
                    if let Some(player) = bridge::player_by_uuid(env, uuid) {
                        let cfg = app.cfg_read();
                        bridge::send_pack(env, player, &cfg.pack.url, &cfg.effective_sha1(), cfg.pack.force);
                        env.delete_local_ref(player);
                    }
                }
            }
        }

        // deferred closes (from click actions)
        let to_close: Vec<String> = {
            let mut q = app.close_queue.lock().unwrap();
            q.drain(..).collect()
        };
        for uuid in to_close {
            self.close(env, app, &uuid, tick);
        }

        // deferred opens
        let due: Vec<(String, String)> = {
            let mut q = app.open_queue.lock().unwrap();
            let mut out = Vec::new();
            q.retain(|(uuid, menu, at)| {
                if tick >= *at {
                    out.push((uuid.clone(), menu.clone()));
                    false
                } else {
                    true
                }
            });
            out
        };
        for (uuid, menu) in due {
            if now_online.contains(&uuid) {
                self.open(env, app, &uuid, &menu, tick);
            }
        }

        // input events
        let events = app.router.drain();
        for ev in events {
            self.route(env, app, ev, tick);
        }

        // animate open sessions
        let uuids: Vec<String> = self.sessions.keys().cloned().collect();
        for uuid in uuids {
            let Some(player) = bridge::player_by_uuid(env, &uuid) else { continue };
            let finished = self.update_session(env, app, &uuid, &player, tick);
            env.delete_local_ref(player);
            if finished {
                self.close_instant(env, &uuid);
                if let Some(p) = bridge::player_by_uuid(env, &uuid) {
                    bridge::update_inv(env, p);
                    env.delete_local_ref(p);
                }
            }
        }
    }

    /// Returns true when the session finished closing.
    fn update_session(
        &mut self,
        env: &JniEnv,
        app: &App,
        uuid: &str,
        player: &jni::jobject,
        tick: u64,
    ) -> bool {
        let Some(s) = self.sessions.get_mut(uuid) else { return false };
        let Some(eye) = bridge::player_eye(env, *player) else { return false };

        // world change -> silent reopen in the new world
        let world_name = bridge::player_world_name(env, *player);
        if world_name != s.world_name {
            let menu_id = s.menu.id.clone();
            s.world_name = world_name;
            self.close_instant(env, uuid);
            self.open(env, app, uuid, &menu_id, tick);
            return false;
        }

        let menu = s.menu.clone();
        let t_open = (tick.saturating_sub(s.opened_tick)) as f64;
        let (base_pos, yaw) = Self::anchor_at(&menu, eye);
        let b = Basis::from_yaw_deg(yaw);
        let _ = yaw;

        // closing finished?
        if s.state == SState::Closing {
            let max_exit = menu
                .elements
                .iter()
                .filter_map(|el| el.anim.exit.as_ref())
                .map(|e| e.delay + e.dur)
                .fold(0.0f64, f64::max);
            let t_close = (tick.saturating_sub(s.close_started)) as f64;
            if t_close > max_exit + 2.0 {
                return true;
            }
        }

        // ---------- hover detection ----------
        if s.state == SState::Open {
            let ray = Ray::from_look(eye[0], eye[1], eye[2], eye[3], eye[4]).to_menu_space(base_pos, b);
            let mut best: Option<(usize, f64)> = None;
            for (i, el) in menu.elements.iter().enumerate() {
                if !el.hoverable || el.page != s.page || i >= s.ents.len() || s.ents[i] == 0 {
                    continue;
                }
                if t_open < enter_total(el) {
                    continue;
                }
                let hit = el.hitbox();
                let sc = el.scale;
                let bx = Aabb::centered(
                    el.pos[0], el.pos[1], el.pos[2],
                    hit[0] * sc[0], hit[1] * sc[1], hit[2] * sc[2],
                );
                if let Some(d) = bx.ray_hit(ray) {
                    if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                        best = Some((i, d));
                    }
                }
            }
            let new_hover = best.map(|(i, _)| i);
            if new_hover != s.hovered {
                s.hovered = new_hover;
                if let Some(hi) = new_hover {
                    if !s.hover_fired[hi] {
                        s.hover_fired[hi] = true;
                        run_actions(env, app, uuid, *player, &menu.elements[hi].on_hover);
                    }
                }
            }
        } else {
            s.hovered = None;
        }

        // ---------- per-element animation ----------
        let page = s.page;
        let hovered = s.hovered;
        let hover_ts: Vec<f64> = s.hover_t.clone();
        for (i, el) in menu.elements.iter().enumerate() {
            let ent = s.ents[i];
            if ent == 0 {
                continue;
            }
            let ent_obj = ent as jni::jobject;

            // hover progress (needs &mut self, so advance separately below)
            let _ = hover_ts[i];

            let (mut sm, mut off, mut rot_extra, mut bg_mult) = enter_state(el, t_open);

            // exit
            if s.state == SState::Closing {
                if let Some(e) = el.anim.exit.as_ref() {
                    let t_close = (tick.saturating_sub(s.close_started)) as f64;
                    if let Some(p) = segment_progress(t_close, e.delay, e.dur) {
                        let k = e.ease.apply(p);
                        let to = &e.from; // for exit, "from"/"to" carries the TARGET state
                        off[0] += to.offset[0] * k;
                        off[1] += to.offset[1] * k;
                        off[2] += to.offset[2] * k;
                        sm *= 1.0 - (1.0 - to.scale) * k;
                        rot_extra += to.rot_y * k;
                        bg_mult *= 1.0 - (1.0 - to.bg_alpha) * k;
                    }
                }
            }

            // hover easing + idle oscillation
            let hover_add = if let Some(h) = el.anim.hover.as_ref() {
                let target = if hovered == Some(i) { 1.0 } else { 0.0 };
                let dt = if target > hover_ts[i] { 1.0 } else { -1.0 };
                let next = (hover_ts[i] + dt / h.dur.max(1.0)).clamp(0.0, 1.0);
                s.hover_t[i] = next;
                if next <= 0.0 {
                    s.hover_fired[i] = false;
                }
                let k = Ease::apply(h.ease, next);
                sm *= 1.0 - (1.0 - h.scale) * k;
                off[1] += h.lift * k;
                h.bg_alpha_add * k
            } else {
                0.0
            };

            if let Some(idle) = el.anim.idle.as_ref() {
                if let Some(f) = idle.float.as_ref() {
                    off[1] += f.amp * loop_wave(t_open, f.period);
                }
                if let Some(sp) = idle.spin.as_ref() {
                    if sp.period > 0.0 {
                        rot_extra += (t_open / sp.period) * 360.0;
                    }
                }
                if let Some(pl) = idle.pulse.as_ref() {
                    sm *= 1.0 + pl.amp * loop_wave01(t_open, pl.period);
                }
            }

            let visible = el.page == page && s.state == SState::Open;
            if !visible {
                if el.page != page {
                    sm *= HIDE_SCALE;
                    bg_mult = 0.0;
                }
            }

            let p = menu_to_world(base_pos, b, el.pos[0] + off[0], el.pos[1] + off[1], el.pos[2] + off[2]);
            bridge::teleport(env, ent_obj, p.x, p.y, p.z, TELEPORT_DUR);
            let min_sc = if visible { 1e-4 } else { HIDE_SCALE };
            bridge::apply_transform(
                env,
                ent_obj,
                &Xform {
                    tx: 0.0,
                    ty: 0.0,
                    tz: 0.0,
                    sx: (el.scale[0] * sm).max(min_sc),
                    sy: (el.scale[1] * sm).max(min_sc),
                    sz: (el.scale[2] * sm).max(min_sc),
                    yaw: (el.rot[1] + rot_extra) as f32,
                    pitch: el.rot[0] as f32,
                    roll: el.rot[2] as f32,
                    interp_delay: 0,
                    interp_dur: INTERP_DUR,
                },
            );

            // text background alpha changes
            if el.kind.kind == "text" {
                let argb = scaled_bg(parse_argb(&el.kind.bg), bg_mult, hover_add, el.page != page);
                if argb != s.last_bg[i] {
                    s.last_bg[i] = argb;
                    bridge::apply_text(
                        env,
                        ent_obj,
                        &legacy(&el.kind.text),
                        argb,
                        el.kind.see_through,
                        el.kind.shadow,
                        el.kind.line_width,
                        align_num(&el.kind.align),
                    );
                }
            }
        }
        false
    }

    // -------------------------------------------------------------------
    // Input routing
    // -------------------------------------------------------------------

    pub fn route(&mut self, env: &JniEnv, app: &App, ev: InputEvent, tick: u64) {
        match ev {
            InputEvent::Click { uuid, btn, yaw, pitch } => {
                let Some(s) = self.sessions.get_mut(&uuid) else { return };
                if s.state != SState::Open {
                    return;
                }
                let menu = s.menu.clone();
                let page = s.page;
                let anchor_world = s.anchor_world.unwrap_or(Vec3::ZERO);
                let anchor_yaw = s.anchor_yaw;
                let Some(player) = bridge::player_by_uuid(env, &uuid) else { return };
                let Some(eye) = bridge::player_eye(env, player) else {
                    env.delete_local_ref(player);
                    return;
                };
                let base_pos = match menu.anchor.mode {
                    AnchorMode::Hud => {
                        let b = Basis::from_yaw_deg(eye[3]);
                        Vec3::new(eye[0], eye[1], eye[2]).add(
                            Vec3::new(b.fx * menu.anchor.distance, menu.anchor.height, b.fz * menu.anchor.distance),
                        )
                    }
                    AnchorMode::World => anchor_world,
                };
                let yaw_v = yaw.map(f64::from).unwrap_or(eye[3]);
                let pitch_v = pitch.map(f64::from).unwrap_or(eye[4]);
                let b = Basis::from_yaw_deg(anchor_yaw);
                let ray = Ray::from_look(eye[0], eye[1], eye[2], yaw_v, pitch_v).to_menu_space(base_pos, b);
                let mut best: Option<(usize, f64)> = None;
                for (i, el) in menu.elements.iter().enumerate() {
                    if !el.hoverable || el.page != page {
                        continue;
                    }
                    let hit = el.hitbox();
                    let sc = el.scale;
                    let bx = Aabb::centered(
                        el.pos[0], el.pos[1], el.pos[2],
                        hit[0] * sc[0], hit[1] * sc[1], hit[2] * sc[2],
                    );
                    if let Some(d) = bx.ray_hit(ray) {
                        if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                            best = Some((i, d));
                        }
                    }
                }
                if let Some((i, _)) = best {
                    let key = match btn {
                        Btn::Left => "left",
                        Btn::Right => "right",
                    };
                    let actions = menu.elements[i].on_click.get(key).cloned();
                    if let Some(actions) = actions {
                        run_actions(env, app, &uuid, player, &actions);
                    }
                }
                env.delete_local_ref(player);
            }
            InputEvent::Back { uuid } => {
                self.close(env, app, &uuid, tick);
            }
            InputEvent::Sneak { uuid, down } => {
                if let Some(s) = self.sessions.get_mut(&uuid) {
                    if s.last_sneak != Some(down) {
                        s.last_sneak = Some(down);
                        crate::publish(
                            app,
                            "cmenus/sneak",
                            &format!("{{\"uuid\":\"{uuid}\",\"down\":{down}}}"),
                        );
                    }
                }
            }
            InputEvent::Scroll { uuid, dir, .. } => {
                let Some(s) = self.sessions.get_mut(&uuid) else { return };
                let pages = s.menu.pages;
                if pages <= 0 {
                    return;
                }
                s.page = (s.page + dir as i32).rem_euclid(pages);
                crate::publish(
                    app,
                    "cmenus/page",
                    &format!("{{\"uuid\":\"{uuid}\",\"page\":{}}}", s.page),
                );
            }
            InputEvent::Command { uuid, line } => {
                let root = app.cfg_read().command.clone();
                let rest = line[root.len()..].trim().to_string();
                let mut it = rest.split_whitespace();
                match it.next() {
                    Some("open") => {
                        if let Some(menu) = it.next() {
                            app.open_queue
                                .lock()
                                .unwrap()
                                .push((uuid, menu.to_string(), tick + 1));
                        }
                    }
                    Some("close") => {
                        self.close(env, app, &uuid, tick);
                    }
                    Some("list") => {
                        if let Some(player) = bridge::player_by_uuid(env, &uuid) {
                            let names: Vec<String> = app.menus_read().keys().cloned().collect();
                            bridge::msg(
                                env,
                                player,
                                &format!("&6[c-menus]&7 menus: &f{}", names.join(", ")),
                            );
                            env.delete_local_ref(player);
                        }
                    }
                    _ => {
                        if let Some(player) = bridge::player_by_uuid(env, &uuid) {
                            bridge::msg(
                                env,
                                player,
                                &format!(
                                    "&6[c-menus]&7 usage: &f/{root} open <id> | close | list"
                                ),
                            );
                            env.delete_local_ref(player);
                        }
                    }
                }
            }
            InputEvent::PackStatus { uuid, status } => {
                crate::publish(
                    app,
                    "cmenus/pack_status",
                    &format!("{{\"uuid\":\"{uuid}\",\"status\":{status}}}"),
                );
                let cfg = app.cfg_read();
                if cfg.auto_open.enabled
                    && cfg.auto_open.on == "pack_accepted"
                    && (status == PACK_ACCEPTED || status == PACK_DOWNLOADED)
                {
                    app.open_queue
                        .lock()
                        .unwrap()
                        .push((uuid, cfg.auto_open.menu.clone(), tick + cfg.auto_open.delay_ticks));
                }
            }
        }
    }
}

fn align_num(a: &str) -> u8 {
    match a {
        "center" => 1,
        "right" => 2,
        _ => 0,
    }
}

/// Enter state at t_open: (scale mult, offset, extra rot_y, bg mult).
fn enter_state(el: &Element, t_open: f64) -> (f64, [f64; 3], f64, f64) {
    let mut sm = 1.0;
    let mut off = [0.0f64; 3];
    let mut rot = 0.0f64;
    let mut bg = 1.0;
    if let Some(e) = el.anim.enter.as_ref() {
        // during the delay window the element holds its "from" state (k=0)
        let k = match segment_progress(t_open, e.delay, e.dur) {
            Some(p) => Ease::apply(e.ease, p),
            None => 0.0,
        };
        off[0] += e.from.offset[0] * (1.0 - k);
        off[1] += e.from.offset[1] * (1.0 - k);
        off[2] += e.from.offset[2] * (1.0 - k);
        // lerp(from.scale -> 1) and lerp(from.bg_alpha -> 1)
        sm *= e.from.scale + (1.0 - e.from.scale) * k;
        rot += e.from.rot_y * (1.0 - k);
        bg *= e.from.bg_alpha + (1.0 - e.from.bg_alpha) * k;
    }
    (sm, off, rot, bg)
}

fn enter_total(el: &Element) -> f64 {
    el.anim.enter.as_ref().map(|e| e.delay + e.dur).unwrap_or(0.0)
}

/// Final background ARGB: base alpha scaled by `mult` plus `add`, zero when hidden.
fn scaled_bg(base: u32, mult: f64, add: f64, hidden: bool) -> u32 {
    if base == 0 || hidden {
        return 0;
    }
    let mut alpha = f64::from(((base >> 24) & 0xFF) as u8) / 255.0;
    alpha *= mult.max(0.0);
    alpha += add;
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    (base & 0x00FF_FFFF) | (a << 24)
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

pub fn run_actions(env: &JniEnv, app: &App, uuid: &str, player: jni::jobject, actions: &[Action]) {
    for a in actions {
        match a {
            Action::Sound { sound, volume, pitch } => {
                bridge::sound(env, player, sound, *volume, *pitch);
            }
            Action::Msg { text } => {
                bridge::msg(env, player, &legacy(text));
            }
            Action::Cmd { cmd } => {
                bridge::console_cmd(env, cmd);
            }
            Action::Event { event, payload } => {
                let json = format!("{{\"uuid\":\"{uuid}\",\"payload\":{payload}}}");
                crate::publish(app, event, &json);
            }
            Action::Close => {
                app.close_queue.lock().unwrap().push(uuid.to_string());
            }
            Action::Open { menu } => {
                let tick = app.tick.load(std::sync::atomic::Ordering::Relaxed);
                app.open_queue
                    .lock()
                    .unwrap()
                    .push((uuid.to_string(), menu.clone(), tick + 1));
            }
        }
    }
}
