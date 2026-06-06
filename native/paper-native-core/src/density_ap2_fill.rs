pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DensityAp2FillSummary {
    pub count: u64,
    pub sink_bits: u64,
    pub checksum: u64,
    pub last_bits: u64,
}

pub fn old_flat_summary(length: usize, iterations: usize) -> DensityAp2FillSummary {
    run_summary(length, iterations, Shape::OldFlat)
}

pub fn scratch_flat_summary(length: usize, iterations: usize) -> DensityAp2FillSummary {
    run_summary(length, iterations, Shape::ScratchFlat)
}

pub fn old_nested_summary(length: usize, iterations: usize) -> DensityAp2FillSummary {
    run_summary(length, iterations, Shape::OldNested)
}

pub fn scratch_nested_summary(length: usize, iterations: usize) -> DensityAp2FillSummary {
    run_summary(length, iterations, Shape::ScratchNested)
}

fn run_summary(length: usize, iterations: usize, shape: Shape) -> DensityAp2FillSummary {
    if length == 0 || iterations == 0 {
        return DensityAp2FillSummary::default();
    }

    let mut array = vec![0.0f64; length];
    let mut scratch = Scratch::default();
    let mut sink = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        shape.fill_array(&mut array, &mut scratch);
        let value = array[(iteration.wrapping_mul(31)) & (length - 1)];
        sink += value;
        last_bits = canonical_double_bits(value);
        checksum = mix_summary(checksum, last_bits, iteration, shape.marker());
    }

    DensityAp2FillSummary {
        count: iterations as u64,
        sink_bits: canonical_double_bits(sink),
        checksum,
        last_bits,
    }
}

#[derive(Clone, Copy)]
enum Shape {
    OldFlat,
    ScratchFlat,
    OldNested,
    ScratchNested,
}

impl Shape {
    fn fill_array(self, array: &mut [f64], scratch: &mut Scratch) {
        match self {
            Shape::OldFlat => old_add(Source::A, Source::B, array),
            Shape::ScratchFlat => scratch_add(Source::A, Source::B, array, scratch),
            Shape::OldNested => {
                fill_source(Source::A, array);
                let mut doubles = vec![0.0f64; array.len()];
                old_add(Source::B, Source::C, &mut doubles);
                add_into(array, &doubles);
            }
            Shape::ScratchNested => {
                fill_source(Source::A, array);
                let doubles = scratch.borrow(array.len());
                fill_source(Source::B, doubles);
                let mut nested = vec![0.0f64; array.len()];
                fill_source(Source::C, &mut nested);
                add_into(doubles, &nested);
                add_into(array, doubles);
                scratch.release();
            }
        }
    }

    fn marker(self) -> u64 {
        match self {
            Shape::OldFlat => 0x4f4c_4446_4c41_5431,
            Shape::ScratchFlat => 0x5343_5246_4c41_5431,
            Shape::OldNested => 0x4f4c_444e_4553_5431,
            Shape::ScratchNested => 0x5343_524e_4553_5431,
        }
    }
}

#[derive(Clone, Copy)]
enum Source {
    A,
    B,
    C,
}

impl Source {
    fn value(self, index: usize) -> f64 {
        match self {
            Source::A => index as f64 * 0.03125 + 3.0,
            Source::B => index as f64 * 0.0625 - 11.0,
            Source::C => index as f64 * 0.015625 + 19.0,
        }
    }
}

#[derive(Default)]
struct Scratch {
    values: Vec<f64>,
    in_use: bool,
}

impl Scratch {
    fn borrow(&mut self, length: usize) -> &mut [f64] {
        debug_assert!(!self.in_use);
        self.in_use = true;
        if self.values.len() < length {
            self.values.resize(length, 0.0);
        }
        &mut self.values[..length]
    }

    fn release(&mut self) {
        self.in_use = false;
    }
}

fn old_add(left: Source, right: Source, array: &mut [f64]) {
    fill_source(left, array);
    let mut doubles = vec![0.0f64; array.len()];
    fill_source(right, &mut doubles);
    add_into(array, &doubles);
}

fn scratch_add(left: Source, right: Source, array: &mut [f64], scratch: &mut Scratch) {
    fill_source(left, array);
    let doubles = scratch.borrow(array.len());
    fill_source(right, doubles);
    add_into(array, doubles);
    scratch.release();
}

fn fill_source(source: Source, array: &mut [f64]) {
    for (index, value) in array.iter_mut().enumerate() {
        *value = source.value(index);
    }
}

fn add_into(array: &mut [f64], values: &[f64]) {
    for index in 0..array.len() {
        array[index] += values[index];
    }
}

#[inline]
fn mix_summary(checksum: u64, value_bits: u64, iteration: usize, marker: u64) -> u64 {
    mix64(checksum ^ value_bits ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)) ^ marker)
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
    fn flat_shapes_match() {
        let old = old_flat_summary(64, 128);
        let scratch = scratch_flat_summary(64, 128);

        assert_eq!(old.count, scratch.count);
        assert_eq!(old.sink_bits, scratch.sink_bits);
        assert_eq!(old.last_bits, scratch.last_bits);
    }

    #[test]
    fn nested_shapes_match() {
        let old = old_nested_summary(64, 128);
        let scratch = scratch_nested_summary(64, 128);

        assert_eq!(old.count, scratch.count);
        assert_eq!(old.sink_bits, scratch.sink_bits);
        assert_eq!(old.last_bits, scratch.last_bits);
    }

    #[test]
    fn zero_inputs_are_empty() {
        assert_eq!(old_flat_summary(0, 128), DensityAp2FillSummary::default());
        assert_eq!(old_flat_summary(64, 0), DensityAp2FillSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = scratch_nested_summary(128, 32);
        let second = scratch_nested_summary(128, 32);

        assert_eq!(first, second);
    }
}
