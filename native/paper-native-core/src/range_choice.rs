#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeChoiceError {
    InvalidInputLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioKind {
    InConstantOutDynamic,
    InDynamicOutConstant,
    BothConstant,
    BothDynamic,
}

pub const SUMMARY_FIELDS: usize = 2;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RangeChoiceSummary {
    pub checksum: u64,
    pub for_index_calls: u64,
}

#[inline]
pub fn old_fill_array_summary(
    inputs: &[f64],
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    scenario: ScenarioKind,
) -> Result<RangeChoiceSummary, RangeChoiceError> {
    fill_array_summary(inputs, block_x, block_y, block_z, scenario, Mode::Old)
}

#[inline]
pub fn optimized_fill_array_summary(
    inputs: &[f64],
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    scenario: ScenarioKind,
) -> Result<RangeChoiceSummary, RangeChoiceError> {
    fill_array_summary(inputs, block_x, block_y, block_z, scenario, Mode::Optimized)
}

#[derive(Clone, Copy)]
enum Mode {
    Old,
    Optimized,
}

#[derive(Clone, Copy)]
enum FunctionKind {
    Constant(f64),
    Linear {
        scale_x: f64,
        scale_y: f64,
        scale_z: f64,
        offset: f64,
    },
}

#[derive(Clone, Copy)]
struct ScenarioDefinition {
    in_range: FunctionKind,
    out_range: FunctionKind,
    optimized: OptimizedMode,
}

#[derive(Clone, Copy)]
enum OptimizedMode {
    Old,
    ConstantIn { in_value: f64 },
    ConstantOut { out_value: f64 },
    ConstantBoth { in_value: f64, out_value: f64 },
}

fn fill_array_summary(
    inputs: &[f64],
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    scenario: ScenarioKind,
    mode: Mode,
) -> Result<RangeChoiceSummary, RangeChoiceError> {
    if inputs.len() != block_x.len() || inputs.len() != block_y.len() || inputs.len() != block_z.len() {
        return Err(RangeChoiceError::InvalidInputLength);
    }

    let definition = scenario_definition(scenario);
    let mut checksum = 0u64;
    let mut for_index_calls = 0u64;

    for i in 0..inputs.len() {
        let input = inputs[i];
        let in_range = input >= 0.5 && input < 2.0;
        let value = match mode {
            Mode::Old => {
                for_index_calls = for_index_calls.wrapping_add(1);
                if in_range {
                    evaluate(definition.in_range, block_x[i], block_y[i], block_z[i])
                } else {
                    evaluate(definition.out_range, block_x[i], block_y[i], block_z[i])
                }
            }
            Mode::Optimized => match definition.optimized {
                OptimizedMode::Old => {
                    for_index_calls = for_index_calls.wrapping_add(1);
                    if in_range {
                        evaluate(definition.in_range, block_x[i], block_y[i], block_z[i])
                    } else {
                        evaluate(definition.out_range, block_x[i], block_y[i], block_z[i])
                    }
                }
                OptimizedMode::ConstantIn { in_value } => {
                    if in_range {
                        in_value
                    } else {
                        for_index_calls = for_index_calls.wrapping_add(1);
                        evaluate(definition.out_range, block_x[i], block_y[i], block_z[i])
                    }
                }
                OptimizedMode::ConstantOut { out_value } => {
                    if in_range {
                        for_index_calls = for_index_calls.wrapping_add(1);
                        evaluate(definition.in_range, block_x[i], block_y[i], block_z[i])
                    } else {
                        out_value
                    }
                }
                OptimizedMode::ConstantBoth {
                    in_value,
                    out_value,
                } => {
                    if in_range {
                        in_value
                    } else {
                        out_value
                    }
                }
            },
        };

        checksum = mix64(
            checksum
                ^ value.to_bits()
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((inputs.len() as u64) << 11),
        );
    }

    Ok(RangeChoiceSummary {
        checksum,
        for_index_calls,
    })
}

fn scenario_definition(scenario: ScenarioKind) -> ScenarioDefinition {
    match scenario {
        ScenarioKind::InConstantOutDynamic => ScenarioDefinition {
            in_range: FunctionKind::Constant(0.25),
            out_range: FunctionKind::Linear {
                scale_x: -0.35,
                scale_y: 0.08,
                scale_z: 0.19,
                offset: -0.75,
            },
            optimized: OptimizedMode::ConstantIn { in_value: 0.25 },
        },
        ScenarioKind::InDynamicOutConstant => ScenarioDefinition {
            in_range: FunctionKind::Linear {
                scale_x: 0.17,
                scale_y: -0.03,
                scale_z: 0.11,
                offset: 0.5,
            },
            out_range: FunctionKind::Constant(-0.75),
            optimized: OptimizedMode::ConstantOut { out_value: -0.75 },
        },
        ScenarioKind::BothConstant => ScenarioDefinition {
            in_range: FunctionKind::Constant(0.25),
            out_range: FunctionKind::Constant(-0.75),
            optimized: OptimizedMode::ConstantBoth {
                in_value: 0.25,
                out_value: -0.75,
            },
        },
        ScenarioKind::BothDynamic => ScenarioDefinition {
            in_range: FunctionKind::Linear {
                scale_x: 0.17,
                scale_y: -0.03,
                scale_z: 0.11,
                offset: 0.5,
            },
            out_range: FunctionKind::Linear {
                scale_x: -0.35,
                scale_y: 0.08,
                scale_z: 0.19,
                offset: -0.75,
            },
            optimized: OptimizedMode::Old,
        },
    }
}

#[inline]
fn evaluate(function: FunctionKind, x: i32, y: i32, z: i32) -> f64 {
    match function {
        FunctionKind::Constant(value) => value,
        FunctionKind::Linear {
            scale_x,
            scale_y,
            scale_z,
            offset,
        } => linear(x, y, z, scale_x, scale_y, scale_z, offset),
    }
}

#[inline]
fn linear(x: i32, y: i32, z: i32, scale_x: f64, scale_y: f64, scale_z: f64, offset: f64) -> f64 {
    x as f64 * scale_x + y as f64 * scale_y + z as f64 * scale_z + offset
}

#[inline]
fn mix64(value: u64) -> u64 {
    let mixed = value ^ (value << 13) ^ (value >> 7) ^ (value << 17);
    if mixed == 0 { 1 } else { mixed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_optimized_match_on_all_scenarios() {
        let inputs = [0.0, 0.25, 0.75, 2.5];
        let block_x = [1, 2, 3, 4];
        let block_y = [5, 6, 7, 8];
        let block_z = [9, 10, 11, 12];

        for scenario in [
            ScenarioKind::InConstantOutDynamic,
            ScenarioKind::InDynamicOutConstant,
            ScenarioKind::BothConstant,
            ScenarioKind::BothDynamic,
        ] {
            let old = old_fill_array_summary(&inputs, &block_x, &block_y, &block_z, scenario).unwrap();
            let optimized = optimized_fill_array_summary(&inputs, &block_x, &block_y, &block_z, scenario).unwrap();
            assert_eq!(old.checksum, optimized.checksum);
        }
    }

    #[test]
    fn optimized_for_index_counts_match_specialized_fill_paths() {
        let inputs = [0.0, 0.25, 0.75, 1.5, 2.5];
        let block_x = [1, 2, 3, 4, 5];
        let block_y = [6, 7, 8, 9, 10];
        let block_z = [11, 12, 13, 14, 15];

        let in_constant_out_dynamic = optimized_fill_array_summary(
            &inputs,
            &block_x,
            &block_y,
            &block_z,
            ScenarioKind::InConstantOutDynamic,
        )
        .unwrap();
        let in_dynamic_out_constant = optimized_fill_array_summary(
            &inputs,
            &block_x,
            &block_y,
            &block_z,
            ScenarioKind::InDynamicOutConstant,
        )
        .unwrap();
        let both_constant = optimized_fill_array_summary(
            &inputs,
            &block_x,
            &block_y,
            &block_z,
            ScenarioKind::BothConstant,
        )
        .unwrap();
        let both_dynamic = optimized_fill_array_summary(
            &inputs,
            &block_x,
            &block_y,
            &block_z,
            ScenarioKind::BothDynamic,
        )
        .unwrap();

        assert_eq!(in_constant_out_dynamic.for_index_calls, 3);
        assert_eq!(in_dynamic_out_constant.for_index_calls, 2);
        assert_eq!(both_constant.for_index_calls, 0);
        assert_eq!(both_dynamic.for_index_calls, inputs.len() as u64);
    }

    #[test]
    fn rejects_bad_lengths() {
        let inputs = [0.0, 1.0];
        let block_x = [1];
        let block_y = [2, 3];
        let block_z = [4, 5];
        assert_eq!(
            old_fill_array_summary(&inputs, &block_x, &block_y, &block_z, ScenarioKind::BothDynamic),
            Err(RangeChoiceError::InvalidInputLength)
        );
    }
}
