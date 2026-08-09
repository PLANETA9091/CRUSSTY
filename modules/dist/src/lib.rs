//! dist — the v1 dist engine as a c-plugin (V2-DESIGN milestone 1).
//!
//! Port of `crates/mod-native` (engine.rs: UDP protocol, leases, fencing) plus
//! dist-paper's `DistNodeDriver` / `Metrics` / `RegionManager` / `RegionHasher`
//! (driver.rs + kernel.rs + the DistKernel Java helper). Run as a native
//! module inside the v2 launcher it turns any Paper-family kernel into a dist
//! node without the JNI bridge: metrics come from the kernel's own
//! getTickTimesNanos, region chunks are force-loaded through Bukkit, and the
//! oracle-facing wire protocol runs in pure Rust on a background thread.

mod driver;
mod engine;
mod kernel;

use cplug_abi::{CPluginApi, JavaVmPtr};

/// The single required export (cplug-abi contract).
///
/// # Safety
/// `api` must be a valid CPluginApi owned by the agent, `vm` the live
/// JavaVM pointer, `options` a NUL-terminated string for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    _options: *const std::ffi::c_char,
) -> i32 {
    cplug_sdk::init(api, vm);

    let cfg = driver::Config::from_env();
    eprintln!(
        "[dist] cplugin_init: engine as module — oracle={} node={} bench={} commit-secs={}",
        cfg.oracle_addr, cfg.node_id, cfg.bench, cfg.commit_secs
    );

    if !driver::claim_single_instance(cfg.node_id) {
        eprintln!(
            "[dist] init denied: single-runtime claim already held by this process \
             (live engine threads still run the previous mapping); reload keeps the old library"
        );
        return 1;
    }
    eprintln!("[dist] single-runtime claim held");

    let rc = driver::start(&cfg);
    if rc != 0 {
        eprintln!("[dist] engine start failed rc={rc}");
        return rc;
    }
    eprintln!("[dist] engine running (drained on kernel main thread)");
    0
}