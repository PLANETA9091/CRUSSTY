pub const SUMMARY_FIELDS: usize = 2;

const MIX_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CarverIterationSummary {
    pub count: u64,
    pub checksum: u64,
}

pub fn foreach_summary(
    iterations: usize,
    set_offsets: &[i32],
    values: &[i32],
) -> CarverIterationSummary {
    run_summary(iterations, set_offsets, values, false)
}

pub fn indexed_summary(
    iterations: usize,
    set_offsets: &[i32],
    values: &[i32],
) -> CarverIterationSummary {
    run_summary(iterations, set_offsets, values, true)
}

fn run_summary(
    iterations: usize,
    set_offsets: &[i32],
    values: &[i32],
    indexed: bool,
) -> CarverIterationSummary {
    if iterations == 0 || set_offsets.len() < 2 || values.is_empty() {
        return CarverIterationSummary::default();
    }
    debug_assert_eq!(set_offsets[0], 0);
    debug_assert_eq!(set_offsets.last().copied().unwrap_or_default() as usize, values.len());

    let set_count = set_offsets.len() - 1;
    let mut checksum = MIX_SEED;

    for iteration in 0..iterations {
        let set_index = iteration % set_count;
        let start = set_offsets[set_index] as usize;
        let end = set_offsets[set_index + 1] as usize;

        if indexed {
            let mut carver_index = 0usize;
            let mut index = start;
            while index < end {
                checksum = mix(checksum, values[index], carver_index);
                carver_index += 1;
                index += 1;
            }
        } else {
            for (carver_index, value) in values[start..end].iter().copied().enumerate() {
                checksum = mix(checksum, value, carver_index);
            }
        }
    }

    CarverIterationSummary {
        count: iterations as u64,
        checksum,
    }
}

#[inline]
fn mix(mut checksum: u64, value: i32, index: usize) -> u64 {
    checksum ^= i64::from(value).wrapping_mul(0x9E37_79B1) as u64 + index as u64;
    checksum = checksum.rotate_left(13);
    checksum.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreach_and_indexed_match_on_regular_inputs() {
        let (set_offsets, values) = build_layout();
        let foreach = foreach_summary(8_000, &set_offsets, &values);
        let indexed = indexed_summary(8_000, &set_offsets, &values);

        assert_eq!(foreach, indexed);
        assert_eq!(foreach.count, 8_000);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (set_offsets, values) = build_layout();
        let summary = foreach_summary(0, &set_offsets, &values);

        assert_eq!(summary, CarverIterationSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (set_offsets, values) = build_layout();
        let first = indexed_summary(4_096, &set_offsets, &values);
        let second = indexed_summary(4_096, &set_offsets, &values);

        assert_eq!(first, second);
    }

    #[test]
    fn empty_values_are_empty() {
        let summary = foreach_summary(128, &[0, 0], &[]);

        assert_eq!(summary, CarverIterationSummary::default());
    }

    fn build_layout() -> (Vec<i32>, Vec<i32>) {
        let mut offsets = Vec::with_capacity(10);
        let mut values = Vec::with_capacity(36);
        let mut offset = 0i32;

        for size in 0..=8 {
            offsets.push(offset);
            for i in 0..size {
                values.push(size * 31 + i);
                offset += 1;
            }
        }
        offsets.push(offset);
        (offsets, values)
    }
}
