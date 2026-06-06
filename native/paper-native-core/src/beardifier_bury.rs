pub const SUMMARY_FIELDS: usize = 4;

const CANONICAL_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BeardifierBurySummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

pub fn current_batch_summary(xs: &[f64], ys: &[f64], zs: &[f64]) -> BeardifierBurySummary {
    run_batch_summary(xs, ys, zs, |x, y, z| current_compute(x, y, z))
}

pub fn optimized_batch_summary(xs: &[f64], ys: &[f64], zs: &[f64]) -> BeardifierBurySummary {
    run_batch_summary(xs, ys, zs, |x, y, z| optimized_compute(x, y, z))
}

fn run_batch_summary<F>(xs: &[f64], ys: &[f64], zs: &[f64], mut compute: F) -> BeardifierBurySummary
where
    F: FnMut(f64, f64, f64) -> f64,
{
    debug_assert_eq!(xs.len(), ys.len());
    debug_assert_eq!(xs.len(), zs.len());

    let iterations = xs.len();
    let mut sum = 0u64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for i in 0..iterations {
        let value_bits = double_to_long_bits(compute(xs[i], ys[i], zs[i]));
        sum = sum.wrapping_add(value_bits);
        last_bits = value_bits;
        checksum = mix64(
            checksum
                ^ value_bits
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    BeardifierBurySummary {
        count: iterations as u64,
        sum_bits: sum,
        value_checksum: checksum,
        last_bits,
    }
}

fn current_compute(x: f64, y: f64, z: f64) -> f64 {
    let len = sqrt(x * x + y * y + z * z);
    clamped_map(len, 0.0, 6.0, 1.0, 0.0)
}

fn optimized_compute(x: f64, y: f64, z: f64) -> f64 {
    let len = sqrt(x * x + y * y + z * z);
    if len > 6.0 {
        return 0.0;
    }
    1.0 - len / 6.0
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
fn sqrt(value: f64) -> f64 {
    value.sqrt()
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
        let xs = [0.0, 1.0, -2.5, 3.5];
        let ys = [0.0, -4.0, 1.25, 5.5];
        let zs = [0.0, 2.0, -3.75, 1.0];

        let current = current_batch_summary(&xs, &ys, &zs);
        let optimized = optimized_batch_summary(&xs, &ys, &zs);

        assert_eq!(current, optimized);
        assert_eq!(current.count, 4);
        assert_ne!(current.value_checksum, 0);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let empty: [f64; 0] = [];
        let summary = current_batch_summary(&empty, &empty, &empty);
        assert_eq!(summary, BeardifierBurySummary::default());
    }

    #[test]
    fn nan_inputs_canonicalize_like_java_double_bits() {
        let xs = [f64::NAN];
        let ys = [0.0];
        let zs = [0.0];

        let current = current_batch_summary(&xs, &ys, &zs);
        let optimized = optimized_batch_summary(&xs, &ys, &zs);

        assert_eq!(current, optimized);
        assert_eq!(current.last_bits, CANONICAL_NAN_BITS);
    }
}
