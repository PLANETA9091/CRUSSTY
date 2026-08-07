//! Constants for the c-cells neuron model.
//!
//! Values marked "cited" come from the human HL2/3 pyramidal cell model
//! (Eyal et al. 2016, eLife; repo `neuron_sim/human_cell`). We *use* these
//! measured quantities as data — we do not port the NEURON code itself.
//! The dynamics implemented in `channels.rs` are our own design.

/// Membrane capacitance of human HL2/3 PCs (µF/cm²) — the paper's headline
/// value. Fitted per-cell in the repo: 0.46–0.49; we adopt the 0.460 fit
/// (half the "universal" ~1); a low Cm means a fast membrane response.
pub const CM: f32 = 0.460;

/// Resting potential (mV).
pub const V_REST: f32 = -70.0;

/// Na+ reversal potential (mV).
pub const ENA: f32 = 50.0;
/// K+ reversal potential (mV).
pub const EK: f32 = -90.0;
/// Leak reversal potential (mV).
pub const EL: f32 = -70.0;

/// Leak conductance (mS/cm²).
pub const GL: f32 = 0.1;

/// Max Na+ conductance (mS/cm²), scaled from the value cited in the
/// Mainen-style channel kinetics used by the Eyal 2016 model.
pub const GNA_BAR: f32 = 0.12;
/// Max K+ conductance (mS/cm²).
pub const GK_BAR: f32 = 0.036;
/// Max NMDA conductance (mS/cm², slow excitatory input).
pub const GNMDA_BAR: f32 = 0.05;

/// Na+ activation: half-voltage (mV) and slope (mV) — cited values
/// (Mainen-type kinetics in the Eyal 2016 model; higher threshold for HL2/3).
pub const THA: f32 = -35.0;
pub const QA: f32 = 9.0;
/// Na+ activation opening/closing rates (/ms) — cited kinetics.
pub const RA: f32 = 0.182;
pub const RB: f32 = 0.124;
/// Na+ inactivation half-voltages (mV) and slope — cited kinetics.
pub const THI1: f32 = -50.0;
pub const THI2: f32 = -75.0;
pub const QI: f32 = 5.0;
/// Inactivation steady-state half-voltage and slope — cited kinetics.
pub const THINF: f32 = -65.0;
pub const QINF: f32 = 6.2;
/// Inactivation opening/recovery rates (/ms) — cited kinetics.
pub const RG: f32 = 0.0091;
pub const RD: f32 = 0.024;

/// K+ activation half-voltage and slope (mV) — cited kinetics.
pub const THN: f32 = -25.0;
pub const QN: f32 = 10.0;
/// K+ opening/closing rates (/ms) — cited kinetics.
pub const RN_A: f32 = 0.05;
pub const RN_B: f32 = 0.1;

/// Temperature factor: `q10^((oper_temp - ref_temp)/10)` — cited convention.
pub const Q10: f32 = 2.3;
pub const REF_TEMP: f32 = 23.0;
pub const OPER_TEMP: f32 = 37.0; // 37 °C — human body temperature

/// Spike threshold (mV) above which the cell fires and enters refraction.
pub const SPIKE_THRESHOLD: f32 = -40.0;
/// After a spike the cell is refractory for this many server ticks (physical
/// "fatigue"): it cannot fire again until `refractory_ticks` elapse.
pub const REFRACTORY_TICKS: u32 = 4;

/// Synaptic input strength per unit of world "load" (EPSP current, µA/cm²).
/// One unit of load = one entity in the cell, one chunk write, etc.
pub const EPSP_GAIN: f32 = 0.8;
/// Inhibitory synaptic gain between neighboring cells (µA/cm² per spike).
pub const IPSP_GAIN: f32 = 0.6;

/// Integration step for the neuron dynamics (ms). The server ticks at 50 Hz
/// (20 ms); the design doc uses dt ≈ 1 ms → up to 20 sub-steps per tick.
pub const DT_MS: f32 = 1.0;

/// Load per entity used to convert entity counts to EPSP units.
pub const LOAD_PER_ENTITY: f32 = 1.0;
/// Load per chunk write (block change event).
pub const LOAD_PER_WRITE: f32 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_factor_is_positive() {
        let tadj = Q10.powf((OPER_TEMP - REF_TEMP) / 10.0);
        assert!(tadj > 1.0 && tadj < 10.0, "tadj={tadj}");
    }

    #[test]
    fn human_cm_is_about_half_the_universal_value() {
        // Cited finding: human HL2/3 Cm ≈ 0.46–0.49 vs the "universal" ~1.
        let cm = CM;
        assert!((cm - 0.460).abs() < 1e-6);
        assert!(cm < 0.6, "cm={cm}");
    }
}
