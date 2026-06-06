use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 4;

const SIZE: usize = 4096;
const PALETTE: usize = 512;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PalettedReencodeRemapCacheSummary {
    pub count: u64,
    pub guard: i64,
    pub checksum: u64,
    pub last: i64,
}

struct Data {
    storage: Vec<i32>,
}

static DATA: OnceLock<Data> = OnceLock::new();

pub fn current_previous_only_summary(iterations: usize) -> PalettedReencodeRemapCacheSummary {
    run_summary(iterations, Shape::CurrentPreviousOnly)
}

pub fn cached_palette_ids_summary(iterations: usize) -> PalettedReencodeRemapCacheSummary {
    run_summary(iterations, Shape::CachedPaletteIds)
}

#[derive(Clone, Copy)]
enum Shape {
    CurrentPreviousOnly,
    CachedPaletteIds,
}

fn run_summary(iterations: usize, shape: Shape) -> PalettedReencodeRemapCacheSummary {
    if iterations == 0 {
        return PalettedReencodeRemapCacheSummary::default();
    }

    let data = DATA.get_or_init(build_data);
    let mut values_scratch = Vec::new();
    let mut remap_scratch = Vec::new();
    let mut guard = 0i64;
    let mut checksum = 0u64;
    let mut last = 0i64;

    for iteration in 0..iterations {
        let value = match shape {
            Shape::CurrentPreviousOnly => {
                current_reencode(&data.storage, &mut values_scratch);
                consume(&values_scratch[..SIZE])
            }
            Shape::CachedPaletteIds => {
                cached_reencode(&data.storage, &mut values_scratch, &mut remap_scratch);
                consume(&values_scratch[..SIZE])
            }
        };

        guard = guard.wrapping_add(value);
        last = value;
        checksum = mix64(checksum ^ value as u64 ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)));
    }

    PalettedReencodeRemapCacheSummary {
        count: iterations as u64,
        guard,
        checksum,
        last,
    }
}

fn build_data() -> Data {
    let mut random = JavaRandom::new(12110);
    let mut storage = vec![0i32; SIZE];
    for i in 0..storage.len() {
        if (i & 7) == 0 {
            storage[i] = random.next_int(PALETTE as i32);
        } else {
            storage[i] = storage[i - 1];
        }
    }
    Data { storage }
}

fn current_reencode(storage: &[i32], values: &mut Vec<i32>) {
    if values.len() < storage.len() {
        values.resize(storage.len(), 0);
    }
    values[..storage.len()].copy_from_slice(storage);

    let mut palette = IdentityIdMap::new(PALETTE);
    let mut previous_input = -1;
    let mut previous_output = -1;
    for value in &mut values[..storage.len()] {
        let input = *value;
        if input != previous_input {
            previous_input = input;
            previous_output = palette.id_for(input as usize);
        }
        *value = previous_output;
    }
}

fn cached_reencode(storage: &[i32], values: &mut Vec<i32>, remap: &mut Vec<i32>) {
    if values.len() < storage.len() {
        values.resize(storage.len(), 0);
    }
    values[..storage.len()].copy_from_slice(storage);
    if remap.len() < PALETTE {
        remap.resize(PALETTE, -1);
    }
    remap[..PALETTE].fill(-1);

    let mut palette = IdentityIdMap::new(PALETTE);
    let mut previous_input = -1;
    let mut previous_output = -1;
    for value in &mut values[..storage.len()] {
        let input = *value;
        if input != previous_input {
            previous_input = input;
            let mut remapped = remap[input as usize];
            if remapped == -1 {
                remapped = palette.id_for(input as usize);
                remap[input as usize] = remapped;
            }
            previous_output = remapped;
        }
        *value = previous_output;
    }
}

struct IdentityIdMap {
    ids: Vec<i32>,
    next_id: i32,
}

impl IdentityIdMap {
    fn new(expected_size: usize) -> Self {
        Self {
            ids: vec![-1; expected_size],
            next_id: 0,
        }
    }

    fn id_for(&mut self, value: usize) -> i32 {
        let existing = self.ids[value];
        if existing != -1 {
            return existing;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.ids[value] = id;
        id
    }
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
    fn cached_and_current_match() {
        assert_eq!(
            current_previous_only_summary(64),
            cached_palette_ids_summary(64)
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            cached_palette_ids_summary(0),
            PalettedReencodeRemapCacheSummary::default()
        );
    }
}
