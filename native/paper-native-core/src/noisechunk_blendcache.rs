pub const SUMMARY_FIELDS: usize = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoiseChunkBlendCacheSummary {
    pub count: u64,
    pub selected: u64,
    pub sink_bits: u64,
}

pub fn old_empty_blender_summary(iterations: usize, size_xz: usize) -> NoiseChunkBlendCacheSummary {
    run_summary(iterations, size_xz, true)
}

pub fn new_empty_blender_summary(iterations: usize, size_xz: usize) -> NoiseChunkBlendCacheSummary {
    run_summary(iterations, size_xz, false)
}

fn run_summary(iterations: usize, size_xz: usize, old: bool) -> NoiseChunkBlendCacheSummary {
    if iterations == 0 || size_xz == 0 {
        return NoiseChunkBlendCacheSummary::default();
    }

    let len = size_xz * size_xz;
    let mut sum = 0.0f64;
    let mut selected = 0u64;

    for iteration in 0..iterations {
        if old {
            let mut alpha = vec![0.0f64; len];
            let mut offset = vec![0.0f64; len];
            alpha.fill(1.0);
            offset.fill(0.0);
            let index = (iteration & 1) % len;
            sum += alpha[index];
            sum += offset[index];
        } else {
            sum += 1.0;
            sum += 0.0;
        }
        selected += 2;
    }

    NoiseChunkBlendCacheSummary {
        count: iterations as u64,
        selected,
        sink_bits: canonical_double_bits(sum),
    }
}

#[inline]
fn canonical_double_bits(value: f64) -> u64 {
    if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_match_on_regular_inputs() {
        let old = old_empty_blender_summary(32, 5);
        let new = new_empty_blender_summary(32, 5);

        assert_eq!(old, new);
        assert_eq!(old.count, 32);
        assert_eq!(old.selected, 64);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let summary = old_empty_blender_summary(0, 5);

        assert_eq!(summary, NoiseChunkBlendCacheSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = new_empty_blender_summary(128, 3);
        let second = new_empty_blender_summary(128, 3);

        assert_eq!(first, second);
    }
}
