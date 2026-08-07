//! Our own HH-style channel dynamics, informed by (not copied from) the
//! kinetics cited in the Eyal 2016 human-cell model. We use the standard
//! exponential-Euler (`cnexp`) integrator — the same *method* NEURON uses —
//! but the equations and structure below are our design for the organisms's
//! discrete `dt ≈ 1 ms` time base.
//!
//! Gating variables follow the classic form:
//!   m' = (minf - m)/mtau,  h' = (hinf - h)/htau
//! where rate functions use the `efun` shape (standard in HH fits)
//!   a = Ra * qa * efun((tha - vm)/qa),  b = Rb * qa * efun((vm - tha)/qa)
//!   mtau = 1/tadj/(a+b),  minf = a/(a+b)
//! and `tadj = q10^((oper_temp - ref_temp)/10)`.
//!
//! Exponential Euler integrates `${x}' = (x_inf - x)/tau` exactly for
//! constant coefficients over the step: x += (x_inf - x)·(1 - e^(-dt/tau)).
//! For dt much smaller than a gating tau this is stable without ODE solves.

use super::params::*;

/// `efun(z) = z/(exp(z)-1)` with a guard at z≈0 (avoids the singularity;
/// value tends to `1 - z/2`).
#[inline]
pub fn efun(z: f32) -> f32 {
    let a = z.abs();
    if a < 1e-6 {
        1.0 - z / 2.0
    } else {
        z / (z.exp() - 1.0)
    }
}

/// Temperature factor: `q10^((oper_temp - ref_temp)/10)`.
#[inline]
pub fn temperature_factor(celsius: f32) -> f32 {
    Q10.powf((celsius - REF_TEMP) / 10.0)
}

/// Na+ gating: steady states and time constants for m and h at a given
/// membrane potential (HH-style kinetics).
#[derive(Debug, Clone, Copy)]
pub struct Sodium {
    pub minf: f32,
    pub hinf: f32,
    pub mtau: f32,
    pub htau: f32,
}

/// Compute Na+ gating equations.
#[inline]
pub fn sodium(v_mv: f32, celsius: f32) -> Sodium {
    let tadj = temperature_factor(celsius);
    let v = v_mv;

    // m activation: a = Ra·qa·efun((tha - v)/qa), b = Rb·qa·efun((v - tha)/qa)
    let a_m = RA * QA * efun((THA - v) / QA);
    let b_m = RB * QA * efun((v - THA) / QA);
    let mtau = 1.0 / tadj / (a_m + b_m);
    let minf = a_m / (a_m + b_m);

    // h inactivation: a = Rd·qi·efun((thi1 - v)/qi), b = Rg·qi·efun((v - thi2)/qi)
    let a_h = RD * QI * efun((THI1 - v) / QI);
    let b_h = RG * QI * efun((v - THI2) / QI);
    let htau = 1.0 / tadj / (a_h + b_h);
    let hinf = 1.0 / (1.0 + ((v - THINF) / QINF).exp());

    Sodium {
        minf,
        hinf,
        mtau,
        htau,
    }
}

/// K+ gating (delayed rectifier): steady state and time constant for `n`.
#[inline]
pub fn potassium(v_mv: f32, celsius: f32) -> (f32, f32) {
    let tadj = temperature_factor(celsius);
    let a = RN_A * efun((THN - v_mv) / QN);
    let b = RN_B * efun((v_mv - THN) / QN);
    let ntau = 1.0 / tadj / (a + b);
    let ninf = a / (a + b);
    (ninf, ntau)
}

/// Exponential-Euler step for a gating variable with known steady-state and
/// time constant at the current voltage.
#[inline]
pub fn gate_step(x: f32, x_inf: f32, tau: f32, dt_ms: f32) -> f32 {
    if tau <= 0.0 || !tau.is_finite() {
        return x_inf;
    }
    x + (x_inf - x) * (1.0 - (-dt_ms / tau).exp())
}

/// Ionic currents (µA/cm²) for a given state and NMDA conductance.
#[inline]
pub fn ionic_current(
    v: f32,
    m: f32,
    h: f32,
    n: f32,
    g_nmda: f32,
    celsius: f32,
) -> f32 {
    let tadj = temperature_factor(celsius);
    let gna = tadj * GNA_BAR * m * m * m * h;
    let ina = gna * (v - ENA);
    // kv: ik = gk·(v - ek)
    let gk = tadj * GK_BAR * n * n * n * n;
    let ik = gk * (v - EK);
    // NMDA boxed as a linear leak-like term (slow excitation).
    let inmda = g_nmda * (v - 0.0); // reversal ≈ 0 mV for NMDA
    ina + ik + inmda
}

/// NMDA conductance decay toward 0 (slow excitatory time constant).
#[inline]
pub fn nmda_step(g: f32, dt_ms: f32) -> f32 {
    // tau ~ 100 ms in the classic NMDA model — decays slowly relative to dt=1.
    const TAU_NMDA_MS: f32 = 80.0;
    g * (-dt_ms / TAU_NMDA_MS).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efun_avoids_singularity_at_zero() {
        assert!((efun(0.0) - 1.0).abs() < 1e-3);
        assert!((efun(1e-9) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn efun_handles_negative() {
        // z/(exp(z)-1) for z = -1 ≈ -1/(0.368-1) ≈ 1.58
        let z = -1.0f32;
        let expect = z / (z.exp() - 1.0);
        assert!((efun(z) - expect).abs() < 1e-3);
    }

    #[test]
    fn gate_step_reaches_steady_state() {
        let mut x = 0.3f32;
        for _ in 0..2000 {
            x = gate_step(x, 0.9, 2.0, 1.0);
        }
        assert!((x - 0.9).abs() < 1e-3);
    }

    #[test]
    fn resting_potential_produces_no_spike() {
        // At rest, the Na+ activation gate is nearly closed (minf small);
        // hinf is ~0.69 at v=-70 (thinf=-65) — the Mainen fit for HL2/3.
        let s = sodium(V_REST, OPER_TEMP);
        assert!(s.minf < 0.05, "minf={}", s.minf);
        assert!(s.hinf > 0.6 && s.hinf < 0.75, "hinf={}", s.hinf);
    }
}