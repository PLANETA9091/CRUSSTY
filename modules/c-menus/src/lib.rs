//! c-menus — fully customizable animated in-world menus for the CRUSSTY
//! platform (Paper 26.3, display entities + core shaders resource pack).
//!
//! Boot path:
//!   cplugin_init → register packet hook + stash the platform table
//!   on_kernel_ready(org.bukkit.Bukkit) → main thread:
//!     define the Bridge class, load menus/*.json, start the tick pump
//!   tick pump (scheduler brick, main thread): sessions update, input drain

mod bridge;
mod config;
mod easing;
mod geom;
mod input;
mod menu;
mod pack;
mod session;

use cplug_abi::{CPluginApi, CPacket, CPlatformApi, JavaVmPtr};
use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::config::Config;
use crate::input::{InputRouter, PLAY_STATE};
use crate::menu::MenuDef;
use crate::session::SessionMgr;

pub struct App {
    pub cfg: RwLock<Config>,
    pub menus: RwLock<HashMap<String, Arc<MenuDef>>>,
    pub router: InputRouter,
    pub sessions: Mutex<SessionMgr>,
    pub open_queue: Mutex<Vec<(String, String, u64)>>,
    pub close_queue: Mutex<Vec<String>>,
    pub tick: AtomicU64,
    pub module_dir: Mutex<Option<PathBuf>>,
    pub ready: AtomicBool,
    platform: AtomicPtr<CPlatformApi>,
}

impl App {
    fn new(cfg: Config, dir: Option<PathBuf>) -> Self {
        App {
            cfg: RwLock::new(cfg),
            menus: RwLock::new(HashMap::new()),
            router: InputRouter::new(),
            sessions: Mutex::new(SessionMgr::new()),
            open_queue: Mutex::new(Vec::new()),
            close_queue: Mutex::new(Vec::new()),
            tick: AtomicU64::new(0),
            module_dir: Mutex::new(dir),
            ready: AtomicBool::new(false),
            platform: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    fn set_platform(&self, p: *const CPlatformApi) {
        self.platform.store(p as *mut CPlatformApi, Ordering::SeqCst);
    }

    fn platform(&self) -> Option<&'static CPlatformApi> {
        let p = self.platform.load(Ordering::SeqCst);
        if p.is_null() {
            None
        } else {
            Some(unsafe { &*p })
        }
    }

    pub fn cfg_read(&self) -> std::sync::RwLockReadGuard<'_, Config> {
        self.cfg.read().unwrap()
    }

    pub fn menus_read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, Arc<MenuDef>>> {
        self.menus.read().unwrap()
    }

    pub fn publish_event(&self, event: &str, payload_json: &str) {
        if let Some(p) = self.platform() {
            if let Some(f) = p.events_publish {
                let c_event = match std::ffi::CString::new(event) {
                    Ok(s) => s,
                    Err(_) => return,
                };
                let c_payload = std::ffi::CString::new(payload_json).unwrap_or_default();
                unsafe { f(c_event.as_ptr(), c_payload.as_ptr()) };
            }
        }
    }

    fn inject_tick(&self) {
        if let Some(p) = self.platform() {
            if let Some(f) = p.scheduler_inject {
                let tag = b"c-menus.tick\0";
                unsafe {
                    f(
                        tag.as_ptr() as *const c_char,
                        tick_trampoline,
                        std::ptr::null_mut(),
                    );
                }
            }
        }
    }
}

static APP: OnceLock<App> = OnceLock::new();

fn app() -> Option<&'static App> {
    APP.get()
}

// ---------------------------------------------------------------------------
// Command root (shared with the input router; standalone for tests)
// ---------------------------------------------------------------------------

static COMMAND_ROOT: RwLock<String> = RwLock::new(String::new());

pub fn command_root() -> String {
    COMMAND_ROOT.read().unwrap().clone()
}

pub fn set_command_root(s: &str) {
    *COMMAND_ROOT.write().unwrap() = s.to_string();
}

pub fn publish(app: &App, event: &str, json: &str) {
    app.publish_event(event, json);
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// # Safety
/// `api` must be the CPluginApi from the runtime, `vm` the live JavaVM*, and
/// `options` a NUL-terminated options string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    options: *const c_char,
) -> i32 {
    cplug_sdk::init(api, vm);

    let opts = cplug_sdk::jni_util::cstr(options).unwrap_or("").to_string();
    let modules_dir = config::modules_dir_from_options(&opts);
    let dir_opt = modules_dir.as_deref().and_then(config::module_dir);
    let (cfg, dir) = match &dir_opt {
        Some(d) => {
            let (c, d) = config::load(d);
            (c, Some(d))
        }
        None => (Config::default(), None),
    };
    set_command_root(&cfg.command);

    let app = App::new(cfg, dir);
    if let Some(a) = api.as_ref() {
        app.set_platform(a.platform);
        // packet-level input hook (netty threads; decode only, no JNI)
        if let Some(p) = a.platform.as_ref() {
            if let Some(add) = p.network_add_hook {
                add(packet_hook, std::ptr::null_mut());
            }
        }
    }
    let _ = APP.set(app);

    eprintln!(
        "[c-menus] cplugin_init ok (modules dir resolved: {})",
        dir_opt.is_some()
    );

    cplug_sdk::on_kernel_ready("org.bukkit.Bukkit", boot);
    0
}

fn boot() {
    cplug_sdk::run_on_main_thread(|env| {
        let Some(app) = app() else { return };
        if !bridge::ensure(env) {
            eprintln!("[c-menus] FATAL: bridge class could not be defined (Bukkit unreachable)");
            return;
        }
        let n = reload_menus(app);
        app.ready.store(true, Ordering::SeqCst);
        // start the per-tick pump on the main thread
        if app.platform().map(|p| p.scheduler_inject.is_some()).unwrap_or(false) {
            app.inject_tick();
            eprintln!("[c-menus] ready: {n} menu(s), tick pump armed, pack sha1={}", app.cfg_read().effective_sha1());
        } else {
            eprintln!("[c-menus] ready: {n} menu(s), but NO platform scheduler — menus idle");
        }
        start_pump_watchdog(n);
    });
}

fn reload_menus(app: &App) -> usize {
    let dir_guard = app.module_dir.lock().unwrap();
    let Some(dir) = dir_guard.clone() else {
        eprintln!("[c-menus] no module dir — no menus loaded (drop the module into modules/c-menus)");
        return 0;
    };
    drop(dir_guard);
    match config::load_menus(&dir) {
        Ok(files) => {
            let mut map = app.menus.write().unwrap();
            let mut n = 0;
            for (name, body) in &files {
                match menu::parse_menu(body) {
                    Ok(def) => {
                        map.insert(def.id.clone(), Arc::new(def));
                        n += 1;
                    }
                    Err(e) => eprintln!("[c-menus] menu {name}: {e}"),
                }
            }
            n
        }
        Err(e) => {
            eprintln!("[c-menus] {e}");
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Tick pump (main thread)
// ---------------------------------------------------------------------------

unsafe extern "C" fn tick_trampoline(_ctx: *mut c_void) {
    tick_once();
    let Some(app) = app() else { return };
    if app.ready.load(Ordering::SeqCst) {
        app.inject_tick();
    }
}

/// One animation/input tick on the main thread.
fn tick_once() {
    let Some(app) = app() else { return };
    let tick = app.tick.fetch_add(1, Ordering::Relaxed) + 1;
    if tick % 400 == 0 {
        eprintln!("[c-menus] tick {tick} alive");
    }
    let ok = cplug_sdk::jni_util::with_attached(|env| {
        // bounded local ref frame: everything inside is cleaned up together
        if env.push_local_frame(128).is_err() {
            return false;
        }
        if let Ok(mut mgr) = app.sessions.lock() {
            mgr.tick(env, app, tick);
        }
        if app.cfg_read().reload_every_ticks > 0 && tick % app.cfg_read().reload_every_ticks == 0 {
            reload_menus(app);
        }
        env.pop_local_frame(std::ptr::null_mut());
        cplug_sdk::jni_util::clear_exception(env);
        true
    })
    .unwrap_or(false);

    if !ok {
        // VM is shutting down or the thread attach failed: stop the pump.
        app.ready.store(false, Ordering::SeqCst);
    }
}

/// Watchdog: when the platform scheduler never fires (transform rules did not
/// match this kernel), fall back to a self-chaining main-thread pump via the
/// SDK's MinecraftServer.execute bridge (~20 tps — still smooth).
fn start_pump_watchdog(menus: usize) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let Some(app) = app() else { return };
        if app.tick.load(Ordering::Relaxed) > 0 || !app.ready.load(Ordering::SeqCst) {
            return;
        }
        eprintln!("[c-menus] scheduler silent — switching to execute() fallback pump");
        while app.ready.load(Ordering::SeqCst) {
            cplug_sdk::run_on_main_thread(|_env| {
                tick_once();
            });
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let _ = menus;
    });
}

// ---------------------------------------------------------------------------
// Packet hook (Netty threads — NO JNI here)
// ---------------------------------------------------------------------------

unsafe extern "C" fn packet_hook(pkt: *const CPacket, _ctx: *mut c_void) -> i32 {
    let Some(app) = app() else { return 0 };
    let p = unsafe { &*pkt };
    if p.state != PLAY_STATE || p.payload.is_null() || p.payload_len == 0 {
        return 0;
    }
    let frame = unsafe { std::slice::from_raw_parts(p.payload, p.payload_len) };
    let Some((id, rest)) = input::split_packet(frame) else {
        return 0;
    };

    if p.direction == 1 {
        // outbound: bind conn -> player uuid on the play login packet
        if id == input::CB_LOGIN {
            app.router.on_outbound_login(p.conn_id, rest);
        }
        return 0;
    }

    let session_open = app
        .router
        .uuid_of(p.conn_id)
        .map(|uuid| app.sessions.lock().map(|m| m.sessions.contains_key(&uuid)).unwrap_or(false))
        .unwrap_or(false);

    app.router.on_inbound(p.conn_id, id, rest, session_open)
}
