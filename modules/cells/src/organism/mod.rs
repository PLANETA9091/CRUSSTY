//! The world as a collection of neurons: each world region maps to one
//! `Neuron`, neighbors form synapses. Spikes produced by overloaded neurons
//! drive the tick scheduler (`driver`) and the method optimizer (`optimize`).

use crate::neuron::params::EPSP_GAIN;
use crate::neuron::Neuron;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

/// A region key for the world map: a (x, z) grid cell. Kept intentionally
/// small; the grid pitch is fixed at build time (see `map`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    pub x: i32,
    pub z: i32,
}

/// Load observed in a region within this tick: entity count and chunk writes.
/// Converted to EPSP current in `step` via the load->uA mapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct Load {
    pub entities: f32,
    pub writes: f32,
}

/// The organism state needed by the tick driver and the optimizer.
pub struct Organism {
    cells: HashMap<Region, Cell>,
}

struct Cell {
    neuron: Neuron,
    /// Load observed this tick (fed into the neuron on the next step).
    load: f32,
    /// Inhibitory input from neighbors' spikes.
    inhibition: f32,
    /// Fired in the current simulation step? (drives the tick decision bits)
    fired: bool,
}

impl Organism {
    fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    /// Ensure a neuron exists for `region`, default HL2/3 at rest.
    pub fn ensure(&mut self, region: Region) {
        self.cells.entry(region).or_insert_with(|| Cell {
            neuron: Neuron::new(),
            load: 0.0,
            inhibition: 0.0,
            fired: false,
        });
    }

    /// Accumulate world load on a region. Called by the JVMTI/Bukkit side
    /// whenever an entity or chunk event occurs in the region.
    pub fn feed_load(&mut self, region: Region, entities: f32, writes: f32) {
        let cell = self.cells.entry(region).or_insert_with(|| Cell {
            neuron: Neuron::new(),
            load: 0.0,
            inhibition: 0.0,
            fired: false,
        });
        cell.load += entities * EPSP_GAIN + writes * EPSP_GAIN * 0.5;
    }

    /// One simulation step for every cell. `dt_ms` is the neuron integration
    /// step; EPSP and IPSP are combined into the synaptic current. Afterwards
    /// `fired` flags are consumed by the scheduler (spike bits).
    pub fn step(&mut self, dt_ms: f32) {
        // Loop 1: integrate each neuron from its own EPSP minus the inhibition
        // that already accumulated during the previous exchange.
        for cell in self.cells.values_mut() {
            let i_syn = cell.load - cell.inhibition;
            cell.fired = cell.neuron.step(dt_ms, i_syn);
            cell.load = 0.0; // consumed
            cell.inhibition = 0.0;
        }
    }

    /// After integration, neighbors of a fired cell receive inhibition next
    /// step (IPSP). This is the "tor" synaptic exchange.
    pub fn exchange(&mut self) {
        let mut inhibitory: Vec<(Region, f32)> = Vec::new();
        for (region, cell) in self.cells.iter() {
            if cell.fired {
                // Each fired cell inhibits its (up to 4) grid neighbors.
                for n in neighbors(*region) {
                    inhibitory.push((n, 1.0));
                }
            }
        }
        let mut peak = HashMap::<Region, f32>::new();
        for (r, w) in inhibitory {
            *peak.entry(r).or_default() += w;
        }
        for (r, w) in peak {
            if let Some(cell) = self.cells.get_mut(&r) {
                cell.inhibition += w * crate::neuron::params::IPSP_GAIN;
            }
        }
    }

    /// Regions that fired in this step (spike bits for the scheduler).
    pub fn fired_regions(&self) -> Vec<Region> {
        self.cells
            .iter()
            .filter(|(_, c)| c.fired)
            .map(|(r, _)| *r)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

fn neighbors(r: Region) -> [Region; 4] {
    [
        Region { x: r.x + 1, z: r.z },
        Region { x: r.x - 1, z: r.z },
        Region { x: r.x, z: r.z + 1 },
        Region { x: r.x, z: r.z - 1 },
    ]
}

static ORGANISM: OnceLock<Mutex<Organism>> = OnceLock::new();

/// The process-wide organism (spawned by `cplugin_init`).
pub fn global() -> &'static Mutex<Organism> {
    ORGANISM.get_or_init(|| Mutex::new(Organism::new()))
}

/// Called from the c-plugin init; reserves the organism.
pub fn start() {
    let _ = global();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neuron::params::SPIKE_THRESHOLD;

    #[test]
    fn feeding_load_does_not_fire_a_resting_cell() {
        let mut org = Organism::new();
        let r = Region { x: 0, z: 0 };
        org.ensure(r);
        org.feed_load(r, 5.0, 10.0);
        org.step(crate::neuron::params::DT_MS);
        assert!(!org.fired_regions().contains(&r));
    }

    #[test]
    fn heavy_load_fires_and_spreads_inhibition() {
        let mut org = Organism::new();
        let hot = Region { x: 2, z: 2 };
        org.ensure(hot);
        // A neighbor exists so the exchange has someone to inhibit.
        org.ensure(Region { x: 3, z: 2 });
        // Huge repeated load on `hot` bursts it.
        for _ in 0..40 {
            org.feed_load(hot, 500.0, 0.0);
            org.step(crate::neuron::params::DT_MS);
            org.exchange();
        }
        // At minimum hot + the neighbor exist.
        assert!(org.len() >= 2);
    }

    #[test]
    fn exchange_is_idempotent_on_empty_organism() {
        let mut org = Organism::new();
        org.step(1.0);
        org.exchange();
        assert!(org.fired_regions().is_empty());
    }

    #[test]
    fn neuron_rest_state_is_used() {
        let mut org = Organism::new();
        let r = Region { x: 1, z: 1 };
        org.ensure(r);
        let cell = org.cells.get(&r).unwrap();
        assert!(cell.neuron.v < SPIKE_THRESHOLD);
        assert_eq!(cell.neuron.refractory, 0);
    }
}