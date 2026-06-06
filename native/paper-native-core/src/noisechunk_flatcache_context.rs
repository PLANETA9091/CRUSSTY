use std::hint::black_box;

pub const SUMMARY_FIELDS: usize = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoiseChunkFlatCacheContextSummary {
    pub count: u64,
    pub selected: u64,
    pub sink_bits: u64,
}

pub fn old_false_context_summary(
    iterations: usize,
    size_xz: usize,
) -> NoiseChunkFlatCacheContextSummary {
    false_context_summary(iterations, size_xz, true)
}

pub fn new_false_context_summary(
    iterations: usize,
    size_xz: usize,
) -> NoiseChunkFlatCacheContextSummary {
    false_context_summary(iterations, size_xz, false)
}

pub fn old_true_context_summary(
    iterations: usize,
    size_xz: usize,
) -> NoiseChunkFlatCacheContextSummary {
    true_context_summary(iterations, size_xz)
}

pub fn new_true_context_summary(
    iterations: usize,
    size_xz: usize,
) -> NoiseChunkFlatCacheContextSummary {
    true_context_summary(iterations, size_xz)
}

fn false_context_summary(
    iterations: usize,
    size_xz: usize,
    allocate_context: bool,
) -> NoiseChunkFlatCacheContextSummary {
    if iterations == 0 || size_xz == 0 {
        return NoiseChunkFlatCacheContextSummary::default();
    }

    let len = size_xz * size_xz;
    let mut sum = 0.0f64;
    let mut selected = 0u64;
    for iteration in 0..iterations {
        let values = vec![0.0f64; len];
        if allocate_context {
            black_box(Box::new(MutableSinglePointContext::default()));
        }
        sum += values[(iteration & 1) % len];
        selected += 1;
        black_box(values);
    }

    NoiseChunkFlatCacheContextSummary {
        count: iterations as u64,
        selected,
        sink_bits: canonical_double_bits(sum),
    }
}

fn true_context_summary(
    iterations: usize,
    size_xz: usize,
) -> NoiseChunkFlatCacheContextSummary {
    if iterations == 0 || size_xz == 0 {
        return NoiseChunkFlatCacheContextSummary::default();
    }

    let len = size_xz * size_xz;
    let mut sum = 0.0f64;
    let mut selected = 0u64;
    for iteration in 0..iterations {
        let mut values = vec![0.0f64; len];
        let mut context = MutableSinglePointContext::default();
        values[0] = f64::from(context.set(iteration as i32, 0, iteration as i32).block_x());
        sum += values[0];
        selected += 1;
        black_box(context);
        black_box(values);
    }

    NoiseChunkFlatCacheContextSummary {
        count: iterations as u64,
        selected,
        sink_bits: canonical_double_bits(sum),
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct MutableSinglePointContext {
    block_x: i32,
    block_y: i32,
    block_z: i32,
}

impl MutableSinglePointContext {
    fn set(&mut self, block_x: i32, block_y: i32, block_z: i32) -> &Self {
        self.block_x = block_x;
        self.block_y = block_y;
        self.block_z = block_z;
        self
    }

    fn block_x(&self) -> i32 {
        self.block_x
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
    fn false_paths_match_on_regular_inputs() {
        let old = old_false_context_summary(64, 5);
        let new = new_false_context_summary(64, 5);

        assert_eq!(old, new);
        assert_eq!(old.count, 64);
        assert_eq!(old.selected, 64);
    }

    #[test]
    fn true_paths_match_on_regular_inputs() {
        let old = old_true_context_summary(32, 5);
        let new = new_true_context_summary(32, 5);

        assert_eq!(old, new);
        assert_eq!(old.count, 32);
        assert_eq!(old.selected, 32);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let summary = old_false_context_summary(0, 5);

        assert_eq!(summary, NoiseChunkFlatCacheContextSummary::default());
    }

    #[test]
    fn zero_size_is_empty() {
        let summary = new_true_context_summary(8, 0);

        assert_eq!(summary, NoiseChunkFlatCacheContextSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = old_true_context_summary(128, 3);
        let second = old_true_context_summary(128, 3);

        assert_eq!(first, second);
    }
}
