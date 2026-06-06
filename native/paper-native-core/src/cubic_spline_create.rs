pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CubicSplineCreateSummary {
    pub count: u64,
    pub sink_bits: u64,
    pub checksum: u64,
    pub last_pair_bits: u64,
}

pub fn old_iterator_summary(
    iterations: usize,
    min_values: &[f32],
    max_values: &[f32],
) -> CubicSplineCreateSummary {
    run_summary(iterations, min_values, max_values, scan_iterator)
}

pub fn index_summary(
    iterations: usize,
    min_values: &[f32],
    max_values: &[f32],
) -> CubicSplineCreateSummary {
    run_summary(iterations, min_values, max_values, scan_index)
}

fn run_summary<F>(
    iterations: usize,
    min_values: &[f32],
    max_values: &[f32],
    mut scan: F,
) -> CubicSplineCreateSummary
where
    F: FnMut(&[f32], &[f32]) -> (f32, f32),
{
    debug_assert_eq!(min_values.len(), max_values.len());
    if iterations == 0 || min_values.is_empty() {
        return CubicSplineCreateSummary::default();
    }

    let mut sink = 0.0f32;
    let mut checksum = 0u64;
    let mut last_pair_bits = 0u64;

    for iteration in 0..iterations {
        let (min_value, max_value) = scan(min_values, max_values);
        sink += min_value + max_value;
        let min_bits = canonical_float_bits(min_value);
        let max_bits = canonical_float_bits(max_value);
        last_pair_bits = pack_pair(min_bits, max_bits);
        checksum = mix64(
            checksum
                ^ u64::from(min_bits)
                ^ u64::from(max_bits).rotate_left(17)
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    CubicSplineCreateSummary {
        count: iterations as u64,
        sink_bits: u64::from(canonical_float_bits(sink)),
        checksum,
        last_pair_bits,
    }
}

fn scan_iterator(min_values: &[f32], max_values: &[f32]) -> (f32, f32) {
    let mut min_value = f32::INFINITY;
    let mut max_value = f32::NEG_INFINITY;
    for (min_candidate, max_candidate) in min_values.iter().zip(max_values.iter()) {
        min_value = java_min(min_value, *min_candidate);
        max_value = java_max(max_value, *max_candidate);
    }
    (min_value, max_value)
}

fn scan_index(min_values: &[f32], max_values: &[f32]) -> (f32, f32) {
    let mut min_value = f32::INFINITY;
    let mut max_value = f32::NEG_INFINITY;
    for index in 0..min_values.len() {
        min_value = java_min(min_value, min_values[index]);
        max_value = java_max(max_value, max_values[index]);
    }
    (min_value, max_value)
}

#[inline]
fn java_min(left: f32, right: f32) -> f32 {
    if left.is_nan() || right.is_nan() {
        f32::NAN
    } else if left == 0.0 && right == 0.0 {
        if left.is_sign_negative() || right.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else if left <= right {
        left
    } else {
        right
    }
}

#[inline]
fn java_max(left: f32, right: f32) -> f32 {
    if left.is_nan() || right.is_nan() {
        f32::NAN
    } else if left == 0.0 && right == 0.0 {
        if left.is_sign_positive() || right.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else if left >= right {
        left
    } else {
        right
    }
}

#[inline]
fn canonical_float_bits(value: f32) -> u32 {
    if value.is_nan() {
        0x7fc0_0000
    } else {
        value.to_bits()
    }
}

#[inline]
fn pack_pair(min_bits: u32, max_bits: u32) -> u64 {
    (u64::from(min_bits) << 32) | u64::from(max_bits)
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
    fn iterator_and_index_match_on_regular_values() {
        let min_values = [-2.0, -1.25, 0.0, 1.0, -0.5, 0.25, -1.75, 0.75];
        let max_values = [1.0, 1.25, 2.5, 3.0, 4.75, 2.25, 1.75, 3.5];
        let old = old_iterator_summary(1024, &min_values, &max_values);
        let index = index_summary(1024, &min_values, &max_values);

        assert_eq!(old, index);
        assert_eq!(old.count, 1024);
        assert_eq!(
            old.last_pair_bits,
            pack_pair((-2.0f32).to_bits(), 4.75f32.to_bits())
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        let values = [1.0, 2.0, 3.0];
        let summary = old_iterator_summary(0, &values, &values);
        assert_eq!(summary, CubicSplineCreateSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let min_values = [-2.0, -1.875, -1.75, -1.625, -1.5, -1.375];
        let max_values = [1.0, 1.25, 1.5, 1.75, 2.0, 2.25];
        let first = index_summary(512, &min_values, &max_values);
        let second = index_summary(512, &min_values, &max_values);

        assert_eq!(first, second);
    }

    #[test]
    fn signed_zero_matches_java_min_max_rules() {
        let min_values = [0.0, -0.0];
        let max_values = [-0.0, 0.0];
        let summary = old_iterator_summary(1, &min_values, &max_values);

        assert_eq!(summary.last_pair_bits, pack_pair((-0.0f32).to_bits(), 0.0f32.to_bits()));
    }
}
