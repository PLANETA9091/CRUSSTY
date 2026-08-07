//! Tick driver: turns neuron spikes into "tick these regions now" decisions.
//!
//! Wiring to the kernel happens through the existing SDK hooks
//! (`cplug_sdk::on_kernel_ready`, `cplug_sdk::run_on_main_thread`) and the
//! platform scheduler. The actual starving/api boundary is added in stage 3
//! of the roadmap (DESIGN.md §5); this module currently exposes the decision
//! surface the scheduler consumes.

use crate::neuron::params::DT_MS;
use crate::organism::{global, Region};
use std::sync::atomic::{AtomicU64, Ordering};

static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// Advance the whole organism by one server tick (a few neuron sub-steps)
/// and perform the synaptic exchange. Returns the regions that fired this
/// tick — the "tick candidates" for the kernel.
pub fn tick() -> Vec<Region> {
    let mut org = global().lock().unwrap_or_else(|p| p.into_inner());
    for _ in 0..3 {
        org.step(DT_MS);
    }
    org.exchange();
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    org.fired_regions()
}

/// Total neuron ticks simulated since module load (diagnostics).
pub fn tick_count() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}