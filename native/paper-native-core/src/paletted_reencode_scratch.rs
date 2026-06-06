use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 4;

const SIZE: usize = 4096;
const PALETTE: usize = 512;
const BITS: usize = 9;
const VALUES_PER_LONG: usize = 64 / BITS;
const VALUE_MASK: u64 = (1u64 << BITS) - 1;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PalettedReencodeScratchSummary {
    pub count: u64,
    pub guard: i64,
    pub checksum: u64,
    pub last: i64,
}

struct Data {
    storage: Vec<i32>,
    remap: Vec<i32>,
    packed: Vec<u64>,
}

static DATA: OnceLock<Data> = OnceLock::new();

pub fn old_newarray_summary(iterations: usize) -> PalettedReencodeScratchSummary {
    run_summary(iterations, Shape::OldNewArray)
}

pub fn scratch_threadlocal_summary(iterations: usize) -> PalettedReencodeScratchSummary {
    run_summary(iterations, Shape::ScratchThreadLocal)
}

pub fn direct_packed_summary(iterations: usize) -> PalettedReencodeScratchSummary {
    run_summary(iterations, Shape::DirectPacked)
}

#[derive(Clone, Copy)]
enum Shape {
    OldNewArray,
    ScratchThreadLocal,
    DirectPacked,
}

fn run_summary(iterations: usize, shape: Shape) -> PalettedReencodeScratchSummary {
    if iterations == 0 {
        return PalettedReencodeScratchSummary::default();
    }

    let data = DATA.get_or_init(build_data);
    let mut scratch = Vec::new();
    let mut guard = 0i64;
    let mut checksum = 0u64;
    let mut last = 0i64;

    for iteration in 0..iterations {
        let value = match shape {
            Shape::OldNewArray => {
                let values = old_reencode(&data.storage, &data.remap);
                consume(&values)
            }
            Shape::ScratchThreadLocal => {
                scratch_reencode(&data.storage, &data.remap, &mut scratch);
                consume(&scratch[..SIZE])
            }
            Shape::DirectPacked => {
                direct_packed_reencode(&data.packed, SIZE, &data.remap, &mut scratch);
                consume(&scratch[..SIZE])
            }
        };

        guard = guard.wrapping_add(value);
        last = value;
        checksum = mix64(checksum ^ value as u64 ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)));
    }

    PalettedReencodeScratchSummary {
        count: iterations as u64,
        guard,
        checksum,
        last,
    }
}

fn build_data() -> Data {
    let mut random = JavaRandom::new(12110);
    let mut storage = vec![0i32; SIZE];
    let mut remap = vec![0i32; PALETTE];

    for (i, value) in remap.iter_mut().enumerate() {
        *value = ((i as i32 * 31) + 17) & 1023;
    }

    for i in 0..storage.len() {
        if (i & 7) == 0 {
            storage[i] = random.next_int(PALETTE as i32);
        } else {
            storage[i] = storage[i - 1];
        }
    }

    let packed = pack(&storage);
    Data {
        storage,
        remap,
        packed,
    }
}

fn old_reencode(storage: &[i32], remap: &[i32]) -> Vec<i32> {
    let mut values = storage.to_vec();
    remap_values(&mut values, remap);
    values
}

fn scratch_reencode(storage: &[i32], remap: &[i32], scratch: &mut Vec<i32>) {
    if scratch.len() < storage.len() {
        scratch.resize(storage.len(), 0);
    }
    scratch[..storage.len()].copy_from_slice(storage);
    remap_values(&mut scratch[..storage.len()], remap);
}

fn remap_values(values: &mut [i32], remap: &[i32]) {
    let mut previous_input = -1;
    let mut previous_output = -1;
    for value in values {
        let input = *value;
        if input != previous_input {
            previous_input = input;
            previous_output = remap[input as usize];
        }
        *value = previous_output;
    }
}

fn direct_packed_reencode(packed_storage: &[u64], size: usize, remap: &[i32], scratch: &mut Vec<i32>) {
    if scratch.len() < size {
        scratch.resize(size, 0);
    }

    let mut previous_input = -1;
    let mut previous_output = -1;
    let mut output_index = 0usize;
    for &raw in packed_storage {
        if output_index >= size {
            break;
        }

        let mut packed = raw;
        let end = (output_index + VALUES_PER_LONG).min(size);
        while output_index < end {
            let value = (packed & VALUE_MASK) as i32;
            if value != previous_input {
                previous_input = value;
                previous_output = remap[value as usize];
            }
            scratch[output_index] = previous_output;
            output_index += 1;
            packed >>= BITS;
        }
    }
}

fn pack(storage: &[i32]) -> Vec<u64> {
    let mut packed = vec![0u64; (storage.len() + VALUES_PER_LONG - 1) / VALUES_PER_LONG];
    let mut packed_index = 0usize;
    let mut input_index = 0usize;

    while input_index <= storage.len() - VALUES_PER_LONG {
        let mut value = 0u64;
        for offset in (0..VALUES_PER_LONG).rev() {
            value <<= BITS;
            value |= storage[input_index + offset] as u64 & VALUE_MASK;
        }
        packed[packed_index] = value;
        packed_index += 1;
        input_index += VALUES_PER_LONG;
    }

    let remaining = storage.len() - input_index;
    if remaining > 0 {
        let mut value = 0u64;
        for offset in (0..remaining).rev() {
            value <<= BITS;
            value |= storage[input_index + offset] as u64 & VALUE_MASK;
        }
        packed[packed_index] = value;
    }

    packed
}

fn consume(values: &[i32]) -> i64 {
    let mut checksum = 0i64;
    let mut i = 0usize;
    while i < values.len() {
        checksum = checksum
            .wrapping_mul(1_315_423_911)
            .wrapping_add(values[i] as i64);
        i += 17;
    }
    checksum
}

#[derive(Clone, Copy)]
struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & -bound) == bound {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
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
    fn all_shapes_match() {
        let old = old_newarray_summary(64);
        let scratch = scratch_threadlocal_summary(64);
        let direct = direct_packed_summary(64);

        assert_eq!(old, scratch);
        assert_eq!(old, direct);
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            old_newarray_summary(0),
            PalettedReencodeScratchSummary::default()
        );
    }
}
