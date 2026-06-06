pub const SUMMARY_FIELDS: usize = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoiseInterpolatorSliceSummary {
    pub count: u64,
    pub selected: u64,
    pub sink_bits: u64,
}

pub fn old_jagged_summary(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseInterpolatorSliceSummary {
    run_old_jagged(iterations, interpolators, cell_count_xz, cell_count_y)
}

pub fn flat_summary(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseInterpolatorSliceSummary {
    run_flat(iterations, interpolators, cell_count_xz, cell_count_y)
}

fn run_old_jagged(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseInterpolatorSliceSummary {
    if iterations == 0 || interpolators == 0 || cell_count_xz == 0 || cell_count_y == 0 {
        return NoiseInterpolatorSliceSummary::default();
    }

    let mut sum = 0.0f64;
    let mut selected = 0u64;
    for iteration in 0..iterations {
        let mut batch = Vec::with_capacity(interpolators);
        for interpolator in 0..interpolators {
            let mut item = OldInterpolator::new(cell_count_y, cell_count_xz);
            item.fill(interpolator, iteration);
            batch.push(item);
        }

        for item in &batch {
            for z in 0..cell_count_xz {
                for y in 0..cell_count_y {
                    sum += item.select(z, y);
                    selected += 1;
                }
            }
        }
    }

    NoiseInterpolatorSliceSummary {
        count: iterations as u64,
        selected,
        sink_bits: canonical_double_bits(sum),
    }
}

fn run_flat(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseInterpolatorSliceSummary {
    if iterations == 0 || interpolators == 0 || cell_count_xz == 0 || cell_count_y == 0 {
        return NoiseInterpolatorSliceSummary::default();
    }

    let mut sum = 0.0f64;
    let mut selected = 0u64;
    for iteration in 0..iterations {
        let mut batch = Vec::with_capacity(interpolators);
        for interpolator in 0..interpolators {
            let mut item = FlatInterpolator::new(cell_count_y, cell_count_xz);
            item.fill(interpolator, iteration);
            batch.push(item);
        }

        for item in &batch {
            for z in 0..cell_count_xz {
                for y in 0..cell_count_y {
                    sum += item.select(z, y);
                    selected += 1;
                }
            }
        }
    }

    NoiseInterpolatorSliceSummary {
        count: iterations as u64,
        selected,
        sink_bits: canonical_double_bits(sum),
    }
}

struct OldInterpolator {
    slice0: Vec<Vec<f64>>,
    slice1: Vec<Vec<f64>>,
}

impl OldInterpolator {
    fn new(cell_count_y: usize, cell_count_xz: usize) -> Self {
        Self {
            slice0: vec![vec![0.0; cell_count_y + 1]; cell_count_xz + 1],
            slice1: vec![vec![0.0; cell_count_y + 1]; cell_count_xz + 1],
        }
    }

    fn fill(&mut self, interpolator: usize, iteration: usize) {
        for z in 0..self.slice0.len() {
            for y in 0..self.slice0[z].len() {
                self.slice0[z][y] = value(interpolator, iteration, 0, z, y);
                self.slice1[z][y] = value(interpolator, iteration, 1, z, y);
            }
        }
    }

    fn select(&self, z: usize, y: usize) -> f64 {
        self.slice0[z][y]
            + self.slice0[z + 1][y]
            + self.slice1[z][y]
            + self.slice1[z + 1][y]
            + self.slice0[z][y + 1]
            + self.slice0[z + 1][y + 1]
            + self.slice1[z][y + 1]
            + self.slice1[z + 1][y + 1]
    }
}

struct FlatInterpolator {
    slice0: Vec<f64>,
    slice1: Vec<f64>,
    row_stride: usize,
    max_z: usize,
}

impl FlatInterpolator {
    fn new(cell_count_y: usize, cell_count_xz: usize) -> Self {
        let row_stride = cell_count_y + 1;
        let max_z = cell_count_xz;
        Self {
            slice0: vec![0.0; (max_z + 1) * row_stride],
            slice1: vec![0.0; (max_z + 1) * row_stride],
            row_stride,
            max_z,
        }
    }

    fn fill(&mut self, interpolator: usize, iteration: usize) {
        for z in 0..=self.max_z {
            let mut index = z * self.row_stride;
            for y in 0..self.row_stride {
                self.slice0[index] = value(interpolator, iteration, 0, z, y);
                self.slice1[index] = value(interpolator, iteration, 1, z, y);
                index += 1;
            }
        }
    }

    fn select(&self, z: usize, y: usize) -> f64 {
        let z0 = z * self.row_stride + y;
        let z1 = z0 + self.row_stride;
        self.slice0[z0]
            + self.slice0[z1]
            + self.slice1[z0]
            + self.slice1[z1]
            + self.slice0[z0 + 1]
            + self.slice0[z1 + 1]
            + self.slice1[z0 + 1]
            + self.slice1[z1 + 1]
    }
}

#[inline]
fn value(interpolator: usize, iteration: usize, slice: usize, z: usize, y: usize) -> f64 {
    (interpolator as u64 * 0x1f1f_1f1f
        + iteration as u64 * 131
        + slice as u64 * 17
        + z as u64 * 7
        + y as u64) as f64
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
    fn old_and_flat_match_on_regular_inputs() {
        let old = old_jagged_summary(8, 5, 3, 7);
        let flat = flat_summary(8, 5, 3, 7);

        assert_eq!(old, flat);
        assert_eq!(old.count, 8);
        assert_eq!(old.selected, 8 * 5 * 3 * 7);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let summary = old_jagged_summary(0, 5, 3, 7);

        assert_eq!(summary, NoiseInterpolatorSliceSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = flat_summary(4, 3, 2, 5);
        let second = flat_summary(4, 3, 2, 5);

        assert_eq!(first, second);
    }

    #[test]
    fn zero_shapes_are_empty() {
        assert_eq!(flat_summary(8, 0, 2, 5), NoiseInterpolatorSliceSummary::default());
        assert_eq!(flat_summary(8, 3, 0, 5), NoiseInterpolatorSliceSummary::default());
        assert_eq!(flat_summary(8, 3, 2, 0), NoiseInterpolatorSliceSummary::default());
    }
}
