//! Method optimizer: neuron spikes mark chronically overloaded cells, and the
//! corresponding hot paths become candidates for conservative bytecode
//! re-optimization through the SDK weave/hooks (roadmap stage 4).
//!
//! This module currently provides the spike→candidate accounting; the actual
//! patching via `cplug_sdk::hooks::register_bytes`/`retransform_class` lands
//! with the kernel wiring (DESIGN.md §5.4).

use crate::organism::Region;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Minimum spikes over the window before a cell's methods are "plastic".
const PLASTICITY_THRESHOLD: u32 = 5;
/// Spike window length in ticks.
const WINDOW_TICKS: u64 = 50;

#[derive(Default)]
struct Plasticity {
    /// region -> (spike count in current window, window id)
    spikes: HashMap<Region, (u32, u64)>,
}

static PLASTIC: LazyLock<Mutex<Plasticity>> = LazyLock::new(|| Mutex::new(Plasticity::default()));

/// Feed the fired regions into the plasticity window. Returns the regions
/// that crossed the threshold this tick (optimization candidates).
pub fn observe_fired(regions: &[Region], current_tick: u64) -> Vec<Region> {
    let mut g = PLASTIC.lock().unwrap_or_else(|p| p.into_inner());
    g.observe(regions, current_tick)
}

impl Plasticity {
    /// The window logic on an owned instance (shared by the runtime global and
    /// the tests, so unit tests don't race on process-wide state).
    fn observe(&mut self, regions: &[Region], current_tick: u64) -> Vec<Region> {
        let mut out = Vec::new();
        for r in regions {
            let e = self.spikes.entry(*r).or_insert((0, current_tick));
            if current_tick.saturating_sub(e.1) >= WINDOW_TICKS {
                // Window expired: restart the count on the fresh window.
                e.1 = current_tick;
                e.0 = 1;
            } else {
                e.0 += 1;
                e.1 = current_tick;
            }
            if e.0 >= PLASTICITY_THRESHOLD {
                out.push(*r);
                e.0 = 0; // consumed
            }
        }
        // GC expired entries (simple sliding: drop windows older than WINDOW).
        self.spikes
            .retain(|_, (_, w)| current_tick.saturating_sub(*w) <= WINDOW_TICKS);
        out
    }
}

/// Regions currently above the plasticity threshold (for telemetry).
pub fn plastic_regions() -> Vec<Region> {
    let g = PLASTIC.lock().unwrap_or_else(|p| p.into_inner());
    g.spikes
        .iter()
        .filter(|(_, (c, _))| *c > 0)
        .map(|(r, _)| *r)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_crossing_threshold_become_candidates() {
        let mut p = Plasticity::default();
        let r = Region { x: 0, z: 0 };
        let fired = vec![r];
        let mut candidates = Vec::new();
        for tick in 0..10u64 {
            candidates = p.observe(&fired, tick);
        }
        // 10 ticks of firing > threshold 5 → the region must appear once.
        assert!(candidates.contains(&r));
    }

    #[test]
    fn quiet_window_resets() {
        let mut p = Plasticity::default();
        let r = Region { x: 1, z: 1 };
        for tick in 0..3u64 {
            p.observe(&[r], tick);
        }
        // After a long quiet window the count is gone.
        let cands = p.observe(&[r], 500);
        assert!(!cands.contains(&r));
    }
}