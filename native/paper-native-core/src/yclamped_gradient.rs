pub const SUMMARY_FIELDS: usize = 4;

const CANONICAL_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct YClampedGradientSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

pub fn current_batch_summary(
    block_ys: &[i32],
    from_ys: &[i32],
    to_ys: &[i32],
    from_values: &[f64],
    to_values: &[f64],
) -> YClampedGradientSummary {
    run_batch_summary(block_ys, from_ys, to_ys, from_values, to_values, |block_y, from_y, to_y, from_value, to_value| {
        current_compute(block_y, from_y, to_y, from_value, to_value)
    })
}

pub fn optimized_batch_summary(
    block_ys: &[i32],
    from_ys: &[i32],
    to_ys: &[i32],
    from_values: &[f64],
    to_values: &[f64],
) -> YClampedGradientSummary {
    run_batch_summary(block_ys, from_ys, to_ys, from_values, to_values, |block_y, from_y, to_y, from_value, to_value| {
        optimized_compute(block_y, from_y, to_y, from_value, to_value)
    })
}

fn run_batch_summary<F>(
    block_ys: &[i32],
    from_ys: &[i32],
    to_ys: &[i32],
    from_values: &[f64],
    to_values: &[f64],
    mut sample: F,
) -> YClampedGradientSummary
where
    F: FnMut(i32, i32, i32, f64, f64) -> f64,
{
    debug_assert_eq!(block_ys.len(), from_ys.len());
    debug_assert_eq!(block_ys.len(), to_ys.len());
    debug_assert_eq!(block_ys.len(), from_values.len());
    debug_assert_eq!(block_ys.len(), to_values.len());

    let iterations = block_ys.len();
    let mut sum = 0u64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for i in 0..iterations {
        let value = sample(block_ys[i], from_ys[i], to_ys[i], from_values[i], to_values[i]);
        let value_bits = double_to_long_bits(value);
        sum = sum.wrapping_add(value_bits);
        last_bits = value_bits;
        checksum = mix64(
            checksum
                ^ value_bits
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    YClampedGradientSummary {
        count: iterations as u64,
        sum_bits: sum,
        value_checksum: checksum,
        last_bits,
    }
}

fn current_compute(block_y: i32, from_y: i32, to_y: i32, from_value: f64, to_value: f64) -> f64 {
    clamped_map(block_y as f64, from_y as f64, to_y as f64, from_value, to_value)
}

fn optimized_compute(block_y: i32, from_y: i32, to_y: i32, from_value: f64, to_value: f64) -> f64 {
    let delta = (block_y as f64 - from_y as f64) / (to_y as f64 - from_y as f64);
    if delta < 0.0 {
        return from_value;
    }
    if delta > 1.0 {
        return to_value;
    }
    from_value + delta * (to_value - from_value)
}

#[inline]
fn clamped_map(input: f64, input_min: f64, input_max: f64, output_min: f64, output_max: f64) -> f64 {
    clamped_lerp(output_min, output_max, inverse_lerp(input, input_min, input_max))
}

#[inline]
fn clamped_lerp(start: f64, end: f64, delta: f64) -> f64 {
    if delta < 0.0 {
        return start;
    }
    if delta > 1.0 {
        return end;
    }
    lerp(delta, start, end)
}

#[inline]
fn inverse_lerp(delta: f64, start: f64, end: f64) -> f64 {
    (delta - start) / (end - start)
}

#[inline]
fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

#[inline]
fn double_to_long_bits(value: f64) -> u64 {
    if value.is_nan() {
        CANONICAL_NAN_BITS
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
    fn current_and_optimized_match_on_regular_inputs() {
        let block_ys = [0, 1, 7, -3];
        let from_ys = [0, -2, 4, -8];
        let to_ys = [8, 3, 9, 2];
        let from_values = [0.25, -1.5, 3.0, 2.75];
        let to_values = [1.25, 4.0, -2.5, -0.25];

        let current = current_batch_summary(&block_ys, &from_ys, &to_ys, &from_values, &to_values);
        let optimized = optimized_batch_summary(&block_ys, &from_ys, &to_ys, &from_values, &to_values);

        assert_eq!(current, optimized);
        assert_eq!(current.count, 4);
        assert_ne!(current.value_checksum, 0);
    }

    #[test]
    fn canonical_nan_paths_match() {
        let block_ys = [0, 3];
        let from_ys = [3, 3];
        let to_ys = [3, 3];
        let from_values = [1.0, -2.0];
        let to_values = [5.0, 8.0];

        let current = current_batch_summary(&block_ys, &from_ys, &to_ys, &from_values, &to_values);
        let optimized = optimized_batch_summary(&block_ys, &from_ys, &to_ys, &from_values, &to_values);

        assert_eq!(current, optimized);
        assert_eq!(current.last_bits, CANONICAL_NAN_BITS);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let empty: [i32; 0] = [];
        let empty_d: [f64; 0] = [];

        let summary = current_batch_summary(&empty, &empty, &empty, &empty_d, &empty_d);
        assert_eq!(summary, YClampedGradientSummary::default());
    }
}
