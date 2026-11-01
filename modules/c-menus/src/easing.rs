//! Easing curves and animation interpolation.
//!
//! All functions map progress `t` (already clamped 0..=1) to an eased
//! 0..=1 value. Overshoot curves may exceed the range (out_back), which is
//! intended for scale pops.

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ease {
    Linear,
    #[serde(alias = "in")]
    InQuad,
    #[serde(alias = "out")]
    OutQuad,
    InOutQuad,
    InCubic,
    #[default]
    OutCubic,
    InOutCubic,
    OutQuart,
    OutQuint,
    #[serde(alias = "sine")]
    InOutSine,
    #[serde(alias = "back")]
    OutBack,
    #[serde(alias = "elastic")]
    OutElastic,
    #[serde(alias = "bounce")]
    OutBounce,
}

impl Ease {
    /// Parse with the legacy alias table (kept for the visual harness parity).
    pub fn parse(s: &str) -> Ease {
        match s {
            "linear" => Ease::Linear,
            "in_quad" | "in" => Ease::InQuad,
            "out_quad" | "out" => Ease::OutQuad,
            "in_out_quad" => Ease::InOutQuad,
            "in_cubic" => Ease::InCubic,
            "out_cubic" => Ease::OutCubic,
            "in_out_cubic" => Ease::InOutCubic,
            "out_quart" => Ease::OutQuart,
            "out_quint" => Ease::OutQuint,
            "in_out_sine" | "sine" => Ease::InOutSine,
            "out_back" | "back" => Ease::OutBack,
            "out_elastic" | "elastic" => Ease::OutElastic,
            "out_bounce" | "bounce" => Ease::OutBounce,
            _ => Ease::OutCubic,
        }
    }

    pub fn apply(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Linear => t,
            Ease::InQuad => t * t,
            Ease::OutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Ease::InOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Ease::InCubic => t * t * t,
            Ease::OutCubic => 1.0 - (1.0 - t).powi(3),
            Ease::InOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Ease::OutQuart => 1.0 - (1.0 - t).powi(4),
            Ease::OutQuint => 1.0 - (1.0 - t).powi(5),
            Ease::InOutSine => -(f64::cos(std::f64::consts::PI * t) - 1.0) / 2.0,
            Ease::OutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Ease::OutElastic => {
                let c4 = (2.0 * std::f64::consts::PI) / 3.0;
                if t <= 0.0 {
                    0.0
                } else if t >= 1.0 {
                    1.0
                } else {
                    (2.0_f64).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Ease::OutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

/// Progress inside a timed segment: (elapsed - delay) / duration clamped,
/// or None while still in the delay window.
pub fn segment_progress(elapsed: f64, delay_ticks: f64, dur_ticks: f64) -> Option<f64> {
    if dur_ticks <= 0.0 {
        return Some(1.0);
    }
    let e = elapsed - delay_ticks;
    if e < 0.0 {
        None
    } else {
        Some((e / dur_ticks).min(1.0))
    }
}

/// Smooth looping oscillator in [-1, 1]: sin(2*pi * ticks / period).
pub fn loop_wave(ticks: f64, period_ticks: f64) -> f64 {
    if period_ticks <= 0.0 {
        return 0.0;
    }
    let a = 2.0 * std::f64::consts::PI * (ticks / period_ticks);
    a.sin()
}

/// Smooth looping oscillator in [0, 1].
pub fn loop_wave01(ticks: f64, period_ticks: f64) -> f64 {
    (loop_wave(ticks, period_ticks) + 1.0) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eased_endpoints() {
        for e in [
            Ease::Linear,
            Ease::InQuad,
            Ease::OutQuad,
            Ease::InOutQuad,
            Ease::InCubic,
            Ease::OutCubic,
            Ease::InOutCubic,
            Ease::OutQuart,
            Ease::OutQuint,
            Ease::InOutSine,
            Ease::OutBack,
            Ease::OutElastic,
            Ease::OutBounce,
        ] {
            assert!((e.apply(0.0)).abs() < 1e-9, "{:?} start", e);
            assert!((e.apply(1.0) - 1.0).abs() < 1e-9, "{:?} end", e);
        }
    }

    #[test]
    fn out_back_overshoots() {
        assert!(Ease::OutBack.apply(0.7) > 1.0);
    }

    #[test]
    fn clamped() {
        assert_eq!(Ease::OutCubic.apply(-3.0), 0.0);
        assert_eq!(Ease::OutCubic.apply(5.0), 1.0);
    }

    #[test]
    fn segments() {
        assert_eq!(segment_progress(1.0, 5.0, 10.0), None);
        assert_eq!(segment_progress(5.0, 5.0, 10.0), Some(0.0));
        assert_eq!(segment_progress(10.0, 5.0, 10.0), Some(0.5));
        assert_eq!(segment_progress(100.0, 5.0, 10.0), Some(1.0));
    }

    #[test]
    fn waves() {
        assert!((loop_wave(0.0, 40.0)).abs() < 1e-9);
        assert!((loop_wave01(0.0, 40.0) - 0.5).abs() < 1e-9);
        assert!((loop_wave(10.0, 40.0) - 1.0).abs() < 1e-6);
    }
}
