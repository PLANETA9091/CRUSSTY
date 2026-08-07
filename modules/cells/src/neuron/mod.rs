//! Single-compartment HL2/3-inspired cell dynamics — our own model.
//!
//! A `Neuron` carries the membrane state (v, Na gates m/h, K gate n, NMDA
//! conductance) and a refractory counter. `step` integrates one `dt_ms` tick
//! of channel dynamics given a synaptic input current, then reports whether
//! the cell fired (spike) in this step.

pub mod channels;
pub mod params;

use channels::{gate_step, ionic_current, nmda_step, potassium, sodium};
use params::*;

/// Membrane state of one cell. All fields are pub so the organism map can
/// inspect/hot-reload them; per-cell state is tiny (≈ 6 f32 + counter).
#[derive(Debug, Clone)]
pub struct Neuron {
    pub v: f32,
    pub m: f32,
    pub h: f32,
    pub n: f32,
    pub g_nmda: f32,
    /// Ticks remaining in the refractory period; 0 = can fire.
    pub refractory: u32,
}

impl Default for Neuron {
    fn default() -> Self {
        let s = sodium(V_REST, OPER_TEMP);
        let (ninf, _) = potassium(V_REST, OPER_TEMP);
        Self {
            v: V_REST,
            m: s.minf,
            h: s.hinf,
            n: ninf,
            g_nmda: 0.0,
            refractory: 0,
        }
    }
}

impl Neuron {
    /// A fresh HL2/3 neuron at rest.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the neuron by `dt_ms` under a total synaptic input current
    /// `i_syn` (µA/cm²: EPSP from world load minus IPSP from neighbors).
    /// Returns true if a spike fired in this step.
    pub fn step(&mut self, dt_ms: f32, i_syn: f32) -> bool {
        if self.refractory > 0 {
            self.refractory -= 1;
            return false;
        }

        let celsius = OPER_TEMP;
        let na = sodium(self.v, celsius);
        let (ninf, ntau) = potassium(self.v, celsius);

        // Exponential Euler integration for the gates (stable for dt < tau).
        let dt = dt_ms.max(0.0);
        self.m = gate_step(self.m, na.minf, na.mtau, dt);
        self.h = gate_step(self.h, na.hinf, na.htau, dt);
        self.n = gate_step(self.n, ninf, ntau, dt);
        self.g_nmda = nmda_step(self.g_nmda, dt);

        // Membrane: dv = (1/Cm)·I·dt with the four currents.
        let i_ion = ionic_current(self.v, self.m, self.h, self.n, self.g_nmda, celsius);
        let leak = GL * (EL - self.v);
        self.v += (dt_ms / CM) * (i_ion + leak + i_syn);

        // Spike detection.
        if self.v >= SPIKE_THRESHOLD {
            self.refractory = REFRACTORY_TICKS;
            self.v = V_REST; // reset (near-instant repolarization for the tick)
            return true;
        }
        false
    }

    /// Apply an EPSP (excitatory input) and an IPSP (inhibitory input)
    /// directly into the integration as a single synaptic current.
    #[inline]
    pub fn synaptic_input(load: f32, inhibition: f32) -> f32 {
        EPSP_GAIN * load - IPSP_GAIN * inhibition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rested_neuron_does_not_fire() {
        let mut n = Neuron::default();
        for _ in 0..100 {
            assert!(!n.step(DT_MS, 0.0), "no input must not spike");
        }
    }

    #[test]
    fn sustained_input_fires_then_refracts() {
        let mut n = Neuron::default();
        let mut spikes = 0u32;
        for _ in 0..80 {
            if n.step(DT_MS, 200.0) {
                spikes += 1;
            }
        }
        // A strong sustained input must produce spikes (a few, bounded by
        // refraction): at most ~80/4 = 20 in 80 steps.
        assert!(spikes >= 2, "expected at least two spikes, got {spikes}");
        assert!(spikes <= 20, "refraction must limit spiking, got {spikes}");
    }

    #[test]
    fn refractory_blocks_immediate_refire() {
        let mut n = Neuron::default();
        // First spike.
        n.step(DT_MS, 250.0);
        // Immediately after, still in refractory: no fire even with input.
        n.refractory = 1;
        assert!(!n.step(DT_MS, 250.0));
    }

    #[test]
    fn resting_gates_are_physiological() {
        let n = Neuron::default();
        assert!(n.v >= -75.0 && n.v <= -60.0, "v={}", n.v);
        // h ~0.69 at rest (thinf=-65, Mainen fit) — not near-inactivation.
        assert!(n.h > 0.6 && n.h < 0.75, "h={}", n.h);
    }
}