pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DensityAp2MinMaxFillSummary {
    pub count: u64,
    pub sink_bits: u64,
    pub checksum: u64,
    pub last_bits: u64,
}

pub fn old_summary(
    scenario_index: usize,
    length: usize,
    iterations: usize,
) -> DensityAp2MinMaxFillSummary {
    run_summary(scenario_index, length, iterations, false)
}

pub fn new_summary(
    scenario_index: usize,
    length: usize,
    iterations: usize,
) -> DensityAp2MinMaxFillSummary {
    run_summary(scenario_index, length, iterations, true)
}

fn run_summary(
    scenario_index: usize,
    length: usize,
    iterations: usize,
    optimized: bool,
) -> DensityAp2MinMaxFillSummary {
    if length == 0 || iterations == 0 {
        return DensityAp2MinMaxFillSummary::default();
    }

    let Some(scenario) = scenario(scenario_index) else {
        return DensityAp2MinMaxFillSummary::default();
    };

    let mut array = vec![0.0f64; length];
    let mut scratch = Vec::new();
    let mut sink = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        if optimized {
            new_fill_array(scenario, &mut array, &mut scratch);
        } else {
            old_fill_array(scenario, &mut array);
        }
        let value = array[(iteration.wrapping_mul(31)) & (length - 1)];
        sink += value;
        last_bits = canonical_double_bits(value);
        checksum = mix_summary(checksum, last_bits, iteration, scenario_index, optimized);
    }

    DensityAp2MinMaxFillSummary {
        count: iterations as u64,
        sink_bits: canonical_double_bits(sink),
        checksum,
        last_bits,
    }
}

fn old_fill_array(scenario: Scenario, array: &mut [f64]) {
    fill_linear(scenario.argument1, array);
    match scenario.kind {
        Kind::Min => {
            let bound = scenario.argument2.min_value;
            for (index, value) in array.iter_mut().enumerate() {
                *value = if *value < bound {
                    *value
                } else {
                    value_min(*value, scenario.argument2.compute(index))
                };
            }
        }
        Kind::Max => {
            let bound = scenario.argument2.max_value;
            for (index, value) in array.iter_mut().enumerate() {
                *value = if *value > bound {
                    *value
                } else {
                    value_max(*value, scenario.argument2.compute(index))
                };
            }
        }
    }
}

fn new_fill_array(scenario: Scenario, array: &mut [f64], scratch: &mut Vec<f64>) {
    fill_linear(scenario.argument1, array);
    match scenario.kind {
        Kind::Min => {
            if scenario.argument1.max_value < scenario.argument2.min_value {
                return;
            }
            if scenario.argument1.min_value > scenario.argument2.max_value {
                fill_linear(scenario.argument2, array);
                return;
            }
            if scenario.argument1.min_value >= scenario.argument2.min_value {
                scratch.resize(array.len(), 0.0);
                fill_linear(scenario.argument2, scratch);
                for (value, right) in array.iter_mut().zip(scratch.iter()) {
                    *value = value_min(*value, *right);
                }
                return;
            }

            let bound = scenario.argument2.min_value;
            for (index, value) in array.iter_mut().enumerate() {
                *value = if *value < bound {
                    *value
                } else {
                    value_min(*value, scenario.argument2.compute(index))
                };
            }
        }
        Kind::Max => {
            if scenario.argument1.min_value > scenario.argument2.max_value {
                return;
            }
            if scenario.argument1.max_value < scenario.argument2.min_value {
                fill_linear(scenario.argument2, array);
                return;
            }
            if scenario.argument1.max_value <= scenario.argument2.max_value {
                scratch.resize(array.len(), 0.0);
                fill_linear(scenario.argument2, scratch);
                for (value, right) in array.iter_mut().zip(scratch.iter()) {
                    *value = value_max(*value, *right);
                }
                return;
            }

            let bound = scenario.argument2.max_value;
            for (index, value) in array.iter_mut().enumerate() {
                *value = if *value > bound {
                    *value
                } else {
                    value_max(*value, scenario.argument2.compute(index))
                };
            }
        }
    }
}

fn fill_linear(function: LinearFunction, array: &mut [f64]) {
    for (index, value) in array.iter_mut().enumerate() {
        *value = function.compute(index);
    }
}

fn scenario(index: usize) -> Option<Scenario> {
    let low = LinearFunction::new(0.001, 0.0, 0.0, 4.095);
    let high = LinearFunction::new(0.002, 10.0, 10.0, 18.19);
    let overlap_a = LinearFunction::new(0.003, -2.0, -2.0, 10.285);
    let overlap_b = LinearFunction::new(-0.002, 7.0, -1.19, 7.0);

    match index {
        0 => Some(Scenario::new(Kind::Min, low, high)),
        1 => Some(Scenario::new(Kind::Min, high, low)),
        2 => Some(Scenario::new(Kind::Max, high, low)),
        3 => Some(Scenario::new(Kind::Max, low, high)),
        4 => Some(Scenario::new(Kind::Min, overlap_a, overlap_b)),
        5 => Some(Scenario::new(Kind::Max, overlap_a, overlap_b)),
        6 => Some(Scenario::new(Kind::Min, low, overlap_b)),
        7 => Some(Scenario::new(Kind::Max, low, overlap_b)),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Scenario {
    kind: Kind,
    argument1: LinearFunction,
    argument2: LinearFunction,
}

impl Scenario {
    fn new(kind: Kind, argument1: LinearFunction, argument2: LinearFunction) -> Self {
        Self {
            kind,
            argument1,
            argument2,
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Min,
    Max,
}

#[derive(Clone, Copy)]
struct LinearFunction {
    scale: f64,
    offset: f64,
    min_value: f64,
    max_value: f64,
}

impl LinearFunction {
    fn new(scale: f64, offset: f64, min_value: f64, max_value: f64) -> Self {
        Self {
            scale,
            offset,
            min_value,
            max_value,
        }
    }

    fn compute(self, index: usize) -> f64 {
        index as f64 * self.scale + self.offset
    }
}

#[inline]
fn value_min(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        return f64::NAN;
    }
    if left == right {
        if left == 0.0 {
            return if left.is_sign_negative() || right.is_sign_negative() {
                -0.0
            } else {
                0.0
            };
        }
        return left;
    }
    if left < right { left } else { right }
}

#[inline]
fn value_max(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        return f64::NAN;
    }
    if left == right {
        if left == 0.0 {
            return if left.is_sign_positive() || right.is_sign_positive() {
                0.0
            } else {
                -0.0
            };
        }
        return left;
    }
    if left > right { left } else { right }
}

#[inline]
fn mix_summary(
    checksum: u64,
    value_bits: u64,
    iteration: usize,
    scenario_index: usize,
    optimized: bool,
) -> u64 {
    mix64(
        checksum
            ^ value_bits
            ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
            ^ ((scenario_index as u64) << 17)
            ^ if optimized { 0x4e45_574d_4d41_5831 } else { 0x4f4c_444d_4d41_5831 },
    )
}

#[inline]
fn canonical_double_bits(value: f64) -> u64 {
    if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_match_on_all_scenarios() {
        for index in 0..8 {
            let old = old_summary(index, 512, 128);
            let new = new_summary(index, 512, 128);

            assert_eq!(old.count, new.count);
            assert_eq!(old.sink_bits, new.sink_bits);
            assert_eq!(old.last_bits, new.last_bits);
        }
    }

    #[test]
    fn zero_inputs_are_empty() {
        assert_eq!(old_summary(0, 0, 128), DensityAp2MinMaxFillSummary::default());
        assert_eq!(old_summary(0, 512, 0), DensityAp2MinMaxFillSummary::default());
    }

    #[test]
    fn bad_scenario_is_empty() {
        assert_eq!(old_summary(99, 512, 128), DensityAp2MinMaxFillSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = new_summary(4, 512, 64);
        let second = new_summary(4, 512, 64);

        assert_eq!(first, second);
    }

    #[test]
    fn java_min_and_max_zero_semantics_match() {
        assert_eq!(value_min(0.0, -0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(value_min(-0.0, 0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(value_max(0.0, -0.0).to_bits(), 0.0f64.to_bits());
        assert_eq!(value_max(-0.0, 0.0).to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn java_min_and_max_nan_semantics_match() {
        assert!(value_min(f64::NAN, 1.0).is_nan());
        assert!(value_min(1.0, f64::NAN).is_nan());
        assert!(value_max(f64::NAN, 1.0).is_nan());
        assert!(value_max(1.0, f64::NAN).is_nan());
    }
}
