pub const SUMMARY_FIELDS: usize = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoiseChunkInterpolatorArraySummary {
    pub count: u64,
    pub selected: u64,
    pub sink_bits: u64,
}

pub fn list_summary(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseChunkInterpolatorArraySummary {
    run_summary(iterations, interpolators, cell_count_xz, cell_count_y, Variant::List)
}

pub fn indexed_list_summary(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseChunkInterpolatorArraySummary {
    run_summary(iterations, interpolators, cell_count_xz, cell_count_y, Variant::IndexedList)
}

pub fn array_summary(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
) -> NoiseChunkInterpolatorArraySummary {
    run_summary(iterations, interpolators, cell_count_xz, cell_count_y, Variant::Array)
}

fn run_summary(
    iterations: usize,
    interpolators: usize,
    cell_count_xz: usize,
    cell_count_y: usize,
    variant: Variant,
) -> NoiseChunkInterpolatorArraySummary {
    if iterations == 0 || interpolators == 0 || cell_count_xz == 0 || cell_count_y == 0 {
        return NoiseChunkInterpolatorArraySummary::default();
    }

    let mut batch = initialize(interpolators);
    let mut sum = 0.0f64;
    let mut selected = 0u64;

    for iteration in 0..iterations {
        variant.reset(&mut batch, iteration);
        variant.update_for_y(&mut batch, y(iteration));
        variant.update_for_x(&mut batch, x(iteration));
        variant.update_for_z(&mut batch, z(iteration));
        variant.accumulate_sink(&batch, &mut sum);
        selected += (interpolators * 4) as u64;
    }

    NoiseChunkInterpolatorArraySummary {
        count: iterations as u64,
        selected,
        sink_bits: canonical_double_bits(sum),
    }
}

fn initialize(interpolators: usize) -> Vec<Interpolator> {
    let mut batch = Vec::with_capacity(interpolators);
    for i in 0..interpolators {
        batch.push(Interpolator::new(
            0.1 + i as f64 * 0.01,
            0.2 + i as f64 * 0.01,
            0.3 + i as f64 * 0.01,
        ));
    }
    batch
}

fn x(i: usize) -> f64 {
    (((i as u64).wrapping_mul(1_185) % 65_536) as f64 / 32.0) - 1_024.0
}

fn y(i: usize) -> f64 {
    (((i as u64).wrapping_mul(833) % 49_152) as f64 / 64.0) - 128.0
}

fn z(i: usize) -> f64 {
    (((i as u64).wrapping_mul(3_395) % 131_072) as f64 / 64.0) - 1_024.0
}

#[derive(Clone, Copy)]
enum Variant {
    List,
    IndexedList,
    Array,
}

impl Variant {
    fn reset(self, batch: &mut [Interpolator], iteration: usize) {
        match self {
            Variant::List => {
                for (index, interpolator) in batch.iter_mut().enumerate() {
                    interpolator.reset(iteration, index);
                }
            }
            Variant::IndexedList | Variant::Array => {
                for index in 0..batch.len() {
                    batch[index].reset(iteration, index);
                }
            }
        }
    }

    fn update_for_y(self, batch: &mut [Interpolator], value: f64) {
        match self {
            Variant::List => {
                for interpolator in batch.iter_mut() {
                    interpolator.update_for_y(value);
                }
            }
            Variant::IndexedList | Variant::Array => {
                for index in 0..batch.len() {
                    batch[index].update_for_y(value);
                }
            }
        }
    }

    fn update_for_x(self, batch: &mut [Interpolator], value: f64) {
        match self {
            Variant::List => {
                for interpolator in batch.iter_mut() {
                    interpolator.update_for_x(value);
                }
            }
            Variant::IndexedList | Variant::Array => {
                for index in 0..batch.len() {
                    batch[index].update_for_x(value);
                }
            }
        }
    }

    fn update_for_z(self, batch: &mut [Interpolator], value: f64) {
        match self {
            Variant::List => {
                for interpolator in batch.iter_mut() {
                    interpolator.update_for_z(value);
                }
            }
            Variant::IndexedList | Variant::Array => {
                for index in 0..batch.len() {
                    batch[index].update_for_z(value);
                }
            }
        }
    }

    fn accumulate_sink(self, batch: &[Interpolator], sink: &mut f64) {
        match self {
            Variant::List => {
                for interpolator in batch {
                    *sink += interpolator.value;
                }
            }
            Variant::IndexedList | Variant::Array => {
                for index in 0..batch.len() {
                    *sink += batch[index].value;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Interpolator {
    noise000: f64,
    noise001: f64,
    noise100: f64,
    noise101: f64,
    noise010: f64,
    noise011: f64,
    noise110: f64,
    noise111: f64,
    value_xz00_delta: f64,
    value_xz10_delta: f64,
    value_xz01_delta: f64,
    value_xz11_delta: f64,
    value_xz00: f64,
    value_xz10: f64,
    value_xz01: f64,
    value_xz11: f64,
    value_z0: f64,
    value_z1: f64,
    value_z_delta: f64,
    value: f64,
}

impl Interpolator {
    fn new(seed0: f64, seed1: f64, seed2: f64) -> Self {
        Self {
            noise000: seed0,
            noise001: seed1,
            noise100: seed2,
            noise101: seed0 + seed1,
            noise010: seed1 + seed2,
            noise011: seed0 + seed2,
            noise110: seed0 - seed1,
            noise111: seed2 - seed0,
            value_xz00_delta: 0.0,
            value_xz10_delta: 0.0,
            value_xz01_delta: 0.0,
            value_xz11_delta: 0.0,
            value_xz00: 0.0,
            value_xz10: 0.0,
            value_xz01: 0.0,
            value_xz11: 0.0,
            value_z0: 0.0,
            value_z1: 0.0,
            value_z_delta: 0.0,
            value: 0.0,
        }
    }

    fn reset(&mut self, iteration: usize, index: usize) {
        let base = iteration as f64 * 0.03125 + index as f64 * 0.125;
        self.noise000 = base + 0.1;
        self.noise001 = base + 0.2;
        self.noise100 = base + 0.3;
        self.noise101 = base + 0.4;
        self.noise010 = base + 0.5;
        self.noise011 = base + 0.6;
        self.noise110 = base + 0.7;
        self.noise111 = base + 0.8;
        self.value_xz00_delta = self.noise010 - self.noise000;
        self.value_xz10_delta = self.noise110 - self.noise100;
        self.value_xz01_delta = self.noise011 - self.noise001;
        self.value_xz11_delta = self.noise111 - self.noise101;
    }

    fn update_for_y(&mut self, y: f64) {
        self.value_xz00 = self.noise000 + y * self.value_xz00_delta;
        self.value_xz10 = self.noise100 + y * self.value_xz10_delta;
        self.value_xz01 = self.noise001 + y * self.value_xz01_delta;
        self.value_xz11 = self.noise101 + y * self.value_xz11_delta;
    }

    fn update_for_x(&mut self, x: f64) {
        self.value_z0 = self.value_xz00 + x * (self.value_xz10 - self.value_xz00);
        self.value_z1 = self.value_xz01 + x * (self.value_xz11 - self.value_xz01);
        self.value_z_delta = self.value_z1 - self.value_z0;
    }

    fn update_for_z(&mut self, z: f64) {
        self.value = self.value_z0 + z * self.value_z_delta;
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
    fn list_and_array_match_on_regular_inputs() {
        let list = list_summary(8, 5, 4, 48);
        let indexed = indexed_list_summary(8, 5, 4, 48);
        let array = array_summary(8, 5, 4, 48);

        assert_eq!(list, indexed);
        assert_eq!(list, array);
        assert_eq!(list.count, 8);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let summary = list_summary(0, 5, 4, 48);

        assert_eq!(summary, NoiseChunkInterpolatorArraySummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = array_summary(4, 3, 2, 5);
        let second = array_summary(4, 3, 2, 5);

        assert_eq!(first, second);
    }
}
