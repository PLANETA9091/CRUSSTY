//! Driver (port of v1 dist-paper `DistNodeDriver` + `Metrics`): the per-tick
//! loop that drains engine events, runs the kernel half on the server's main
//! thread (via cplug-sdk main-thread Runnable recipe) and performs
//! heartbeat/commit at the v1 cadences.
//!
//! v1 drove this with a BukkitRunnable on the plugin scheduler; without a
//! plugin the SDK queues jobs onto `MinecraftServer.execute` instead.

use crate::engine;
use crate::kernel as kernel_api;
use jvmti_bindings::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

const TICKS_PER_SECOND: u64 = 20;
const HEARTBEAT_TICKS: u64 = TICKS_PER_SECOND; // 1/s
const ORACLE_TICK_MS: u64 = 1000;
const PING_MS: u32 = 5;

#[derive(Clone)]
pub struct Config {
    pub oracle_addr: String,
    pub node_id: u64,
    pub bench: f64,
    pub commit_secs: u64,
    pub chunks_per_side: u32,
    pub regions_per_row: u32,
}

impl Config {
    /// Env overrides, mirroring v1 (DIST_ORACLE_ADDR / DIST_NODE_ID /
    /// DIST_BENCH) plus the v1 config.yml knobs as env.
    pub fn from_env() -> Self {
        Self {
            oracle_addr: env_or("DIST_ORACLE_ADDR", "127.0.0.1:5555"),
            node_id: env_or_long("DIST_NODE_ID", 1).max(1) as u64,
            bench: env_or_double("DIST_BENCH", 100.0),
            commit_secs: env_or_long("DIST_COMMIT_SECS", 5).max(1) as u64,
            chunks_per_side: env_or_long("DIST_REGION_CHUNKS_PER_SIDE", 8).max(1) as u32,
            regions_per_row: env_or_long("DIST_REGIONS_PER_ROW", 4).max(1) as u32,
        }
    }
}

fn env_or(key: &str, def: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => v,
        _ => def.to_string(),
    }
}

fn env_or_long(key: &str, def: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(def)
}

fn env_or_double(key: &str, def: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(def)
}

#[derive(Default)]
struct DriverState {
    ready: bool,
    owned: HashSet<u32>,
    tick: u64,
    last_heartbeat_tick: u64,
    last_commit_tick: u64,
    /// Announced "oracle unreachable" once; reset when a pong shows up so a
    /// fresh outage still voices itself.
    oracle_unreachable_announced: bool,
}

/// Start the engine and the periodic main-thread driver loop.
/// Returns 0 on success, engine error code otherwise.
pub fn start(cfg: &Config) -> i32 {
    let rc = engine::start(&cfg.oracle_addr, cfg.node_id, cfg.bench);
    if rc != 0 {
        return rc;
    }
    let state = Arc::new(Mutex::new(DriverState::default()));
    let cfg = cfg.clone();
    std::thread::Builder::new()
        .name("dist-driver".into())
        .spawn(move || schedule_main_loop(state, cfg))
        .ok();
    0
}

/// Every ~100ms deliver one main-thread iteration; re-arm until the kernel
/// shuts us down. Keeps all Bukkit-touching work on the server thread.
fn schedule_main_loop(state: Arc<Mutex<DriverState>>, cfg: Config) {
    const POLL_MS: u64 = 100;
    // cplugin_init runs on the JVMTI OnLoad thread BEFORE the VM is ready;
    // the first JNI attach here (via SDK run_on_main_thread's flush worker)
    // would AttachCurrentThread during VM init and SIGSEGV. Give the VM the
    // same grace the SDK's on_kernel_ready uses (see hooks.rs docs).
    std::thread::sleep(std::time::Duration::from_secs(3));
    loop {
        if !engine::running() {
            return;
        }
        let state = state.clone();
        let cfg = cfg.clone();
        cplug_sdk::run_on_main_thread(move |env| main_loop_iteration(&state, &cfg, env));
        std::thread::sleep(std::time::Duration::from_millis(POLL_MS));
    }
}

fn main_loop_iteration(state: &Arc<Mutex<DriverState>>, cfg: &Config, env: &JniEnv) {
    if !state.lock().unwrap().ready {
        kernel_api::ensure_ready(env);
        if !kernel_api::is_ready() {
            return;
        }
        state.lock().unwrap().ready = true;
        eprintln!("[dist] driver armed");
    }
    let mut s = state.lock().unwrap();
    s.tick += 1;

    // drain control events (lease grants / revocations)
    loop {
        let ev = engine::poll_event();
        if ev == 0 {
            break;
        }
        let typ = (ev >> 32) as u32;
        let region = (ev & 0xFFFF_FFFF) as u32;
        match typ {
            engine::EVENT_GRANT_U32 => {
                s.owned.insert(region);
                kernel_api::force_chunks(
                    env,
                    region as i32,
                    cfg.chunks_per_side as i32,
                    cfg.regions_per_row as i32,
                    true,
                );
                cplug_sdk::log_info(&format!("[dist] lease granted region={region}"));
            }
            engine::EVENT_REVOKED_U32 => {
                s.owned.remove(&region);
                kernel_api::force_chunks(
                    env,
                    region as i32,
                    cfg.chunks_per_side as i32,
                    cfg.regions_per_row as i32,
                    false,
                );
                cplug_sdk::log_info(&format!("[dist] lease revoked region={region}"));
            }
            _ => {}
        }
    }

    // heartbeat once per second with real kernel load
    if s.tick.saturating_sub(s.last_heartbeat_tick) >= HEARTBEAT_TICKS {
        s.last_heartbeat_tick = s.tick;
        let load = kernel_api::load(env);
        engine::heartbeat(load, PING_MS);
        if s.tick.is_multiple_of(HEARTBEAT_TICKS * 60) {
            let ot = engine::oracle_tick(ORACLE_TICK_MS);
            if ot >= 0 {
                s.oracle_unreachable_announced = false;
                eprintln!("[dist] tick-sync oracleTick={ot}");
            } else if !s.oracle_unreachable_announced {
                s.oracle_unreachable_announced = true;
                eprintln!("[dist] tick-sync: oracle unreachable (no pong)");
            }
        }
    }

    // commit owned regions every commit interval
    let commit_ticks = cfg.commit_secs * TICKS_PER_SECOND;
    if s.tick.saturating_sub(s.last_commit_tick) >= commit_ticks && !s.owned.is_empty() {
        s.last_commit_tick = s.tick;
        for region in s.owned.clone() {
            match kernel_api::hash_region(
                env,
                region as i32,
                cfg.chunks_per_side as i32,
                cfg.regions_per_row as i32,
            ) {
                Some(hash) => {
                    let t = oracle_tick_or_wall();
                    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
                    cplug_sdk::log_info(&format!("[dist] commit region={region} tick={t} hash={hex}"));
                    engine::commit(region, t, hash);
                }
                None => cplug_sdk::log_info(&format!("[dist] commit region={region} hash failed")),
            }
        }
    }
}

fn oracle_tick_or_wall() -> u64 {
    let t = engine::oracle_tick(ORACLE_TICK_MS);
    if t > 0 {
        t as u64
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}
