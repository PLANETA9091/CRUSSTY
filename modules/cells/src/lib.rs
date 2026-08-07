//! c-cells — the server as a simulated living organism.
//!
//! The c-plugin entry point initializes the SDK and spawns the organism.
//! The neuron dynamics live in `neuron`; the world-to-neuron map, synaptic
//! neighbors and spike signals are in `organism`; the tick-driver and the
//! method optimizer (neuron-driven re-optimization) in `driver`/`optimize`.

pub mod neuron;
pub mod organism;
pub mod driver;
pub mod optimize;

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
    if std::env::var_os("CRUSSTY_CELLS").is_some_and(|v| v == "0") {
        eprintln!("[cells] disabled (CRUSSTY_CELLS=0)");
        return 0;
    }

    cplug_sdk::init(api, vm);
    eprintln!("[cells] organism waking (HL2/3 Cm≈0.46 µF/cm²)");

    organism::start();
    0
}