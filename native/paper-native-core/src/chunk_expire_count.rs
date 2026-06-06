use std::collections::HashMap;

pub const SUMMARY_FIELDS: usize = 4;

const SECTION_SHIFT: i32 = 6;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChunkExpireCountSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn dynamic_compute_hot_summary(
    iterations: usize,
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(iterations, section_bits, chunk_bits, Scenario::Hot, Mode::DynamicCompute)
}

pub fn dynamic_manual_hot_summary(
    iterations: usize,
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(iterations, section_bits, chunk_bits, Scenario::Hot, Mode::DynamicManual)
}

pub fn cached_compute_hot_summary(
    iterations: usize,
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(iterations, section_bits, chunk_bits, Scenario::Hot, Mode::CachedCompute)
}

pub fn cached_hybrid_hot_summary(
    iterations: usize,
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(iterations, section_bits, chunk_bits, Scenario::Hot, Mode::CachedHybrid)
}

pub fn cached_manual_hot_summary(
    iterations: usize,
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(iterations, section_bits, chunk_bits, Scenario::Hot, Mode::CachedManual)
}

pub fn dynamic_compute_cold_summary(
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(0, section_bits, chunk_bits, Scenario::Cold, Mode::DynamicCompute)
}

pub fn dynamic_manual_cold_summary(
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(0, section_bits, chunk_bits, Scenario::Cold, Mode::DynamicManual)
}

pub fn cached_compute_cold_summary(
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(0, section_bits, chunk_bits, Scenario::Cold, Mode::CachedCompute)
}

pub fn cached_hybrid_cold_summary(
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(0, section_bits, chunk_bits, Scenario::Cold, Mode::CachedHybrid)
}

pub fn cached_manual_cold_summary(
    section_bits: usize,
    chunk_bits: usize,
) -> ChunkExpireCountSummary {
    run_summary(0, section_bits, chunk_bits, Scenario::Cold, Mode::CachedManual)
}

#[derive(Clone, Copy)]
enum Scenario {
    Hot,
    Cold,
}

#[derive(Clone, Copy)]
enum Mode {
    DynamicCompute,
    DynamicManual,
    CachedCompute,
    CachedHybrid,
    CachedManual,
}

impl Mode {
    fn tag(self) -> u64 {
        match self {
            Self::DynamicCompute => 0x0D55_0001,
            Self::DynamicManual => 0x0D55_0002,
            Self::CachedCompute => 0x0D55_0003,
            Self::CachedHybrid => 0x0D55_0004,
            Self::CachedManual => 0x0D55_0005,
        }
    }
}

fn run_summary(
    iterations: usize,
    section_bits: usize,
    chunk_bits: usize,
    scenario: Scenario,
    mode: Mode,
) -> ChunkExpireCountSummary {
    let data = BenchData::new(section_bits, chunk_bits);
    let scenario_iterations = match scenario {
        Scenario::Hot => iterations,
        Scenario::Cold => data.op_count,
    };
    if scenario_iterations == 0 {
        return ChunkExpireCountSummary::default();
    }

    let value = run_once(&data, scenario, mode, scenario_iterations);
    let shape_digest = mix64(
        mode.tag()
            ^ ((section_bits as u64) << 8)
            ^ ((chunk_bits as u64) << 16)
            ^ ((data.op_count as u64) << 24)
            ^ match scenario {
                Scenario::Hot => 0xABCD_1000,
                Scenario::Cold => 0xABCD_2000,
            },
    );
    let mut checksum = 0u64;
    for iteration in 0..scenario_iterations {
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((scenario_iterations as u64) << 11),
        );
    }

    ChunkExpireCountSummary {
        count: scenario_iterations as u64,
        total: value.wrapping_mul(scenario_iterations as u64),
        checksum,
        last_total: value,
    }
}

fn run_once(data: &BenchData, scenario: Scenario, mode: Mode, iterations: usize) -> u64 {
    let mut counts = ExpireCounts::new(data, mode);
    match scenario {
        Scenario::Hot => counts.seed_hot(),
        Scenario::Cold => {}
    }

    let mut sum = 0u64;
    match scenario {
        Scenario::Hot => {
            for iteration in 0..iterations {
                let index = (iteration.wrapping_mul(31)) & (data.op_count - 1);
                counts.add_expire_count(data.chunk_x[index], data.chunk_z[index]);
                counts.remove_expire_count(data.chunk_x[index], data.chunk_z[index]);
                sum = sum.wrapping_add(data.chunk_keys[index]);
            }
        }
        Scenario::Cold => {
            for iteration in 0..data.op_count {
                let index = (iteration.wrapping_mul(37)) & (data.op_count - 1);
                counts.add_expire_count(data.chunk_x[index], data.chunk_z[index]);
                sum = sum.wrapping_add(data.chunk_keys[index]);
            }
        }
    }
    sum ^ counts.checksum()
}

struct BenchData {
    chunk_x: Vec<i32>,
    chunk_z: Vec<i32>,
    chunk_keys: Vec<u64>,
    section_keys: Vec<u64>,
    chunks_per_section: usize,
    op_count: usize,
}

impl BenchData {
    fn new(section_bits: usize, chunk_bits: usize) -> Self {
        let section_count = 1usize << section_bits;
        let chunks_per_section = 1usize << chunk_bits;
        let op_count = section_count * chunks_per_section;
        let section_side = 1usize << (section_bits / 2);
        let chunk_mask = (1i32 << SECTION_SHIFT) - 1;
        let mut chunk_x = vec![0i32; op_count];
        let mut chunk_z = vec![0i32; op_count];
        let mut chunk_keys = vec![0u64; op_count];
        let mut section_keys = vec![0u64; section_count];

        let mut index = 0usize;
        for section in 0..section_count {
            let section_x = ((section & (section_side - 1)) as i32) - ((section_side >> 1) as i32);
            let section_z = ((section >> (section_bits / 2)) as i32) - ((section_side >> 1) as i32);
            section_keys[section] = chunk_key(section_x, section_z);
            for offset in 0..chunks_per_section {
                let x = (section_x << SECTION_SHIFT) + (((offset * 3 + section) as i32) & chunk_mask);
                let z = (section_z << SECTION_SHIFT) + (((offset * 5 + (section >> 3)) as i32) & chunk_mask);
                chunk_x[index] = x;
                chunk_z[index] = z;
                chunk_keys[index] = chunk_key(x, z);
                index += 1;
            }
        }

        Self {
            chunk_x,
            chunk_z,
            chunk_keys,
            section_keys,
            chunks_per_section,
            op_count,
        }
    }
}

struct ExpireCounts<'a> {
    data: &'a BenchData,
    mode: Mode,
    section_to_chunk_counts: HashMap<u64, HashMap<u64, i32>>,
}

impl<'a> ExpireCounts<'a> {
    fn new(data: &'a BenchData, mode: Mode) -> Self {
        Self {
            data,
            mode,
            section_to_chunk_counts: HashMap::new(),
        }
    }

    fn seed_hot(&mut self) {
        for (section, section_key) in self.data.section_keys.iter().enumerate() {
            let mut counts = HashMap::with_capacity(self.data.chunks_per_section);
            let start = section * self.data.chunks_per_section;
            for offset in 0..self.data.chunks_per_section {
                counts.insert(self.data.chunk_keys[start + offset], 1);
            }
            self.section_to_chunk_counts.insert(*section_key, counts);
        }
    }

    fn add_expire_count(&mut self, chunk_x: i32, chunk_z: i32) {
        let chunk_key_value = chunk_key(chunk_x, chunk_z);
        let shift = self.region_chunk_shift();
        let section_key = chunk_key(chunk_x >> shift, chunk_z >> shift);

        match self.mode {
            Mode::DynamicCompute | Mode::CachedCompute => {
                self.section_to_chunk_counts
                    .entry(section_key)
                    .or_default()
                    .entry(chunk_key_value)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            Mode::DynamicManual | Mode::CachedManual | Mode::CachedHybrid => {
                let counts = self.section_to_chunk_counts.entry(section_key).or_default();
                let entry = counts.entry(chunk_key_value).or_insert(0);
                *entry += 1;
            }
        }
    }

    fn remove_expire_count(&mut self, chunk_x: i32, chunk_z: i32) {
        let chunk_key_value = chunk_key(chunk_x, chunk_z);
        let shift = self.region_chunk_shift();
        let section_key = chunk_key(chunk_x >> shift, chunk_z >> shift);

        let mut remove_section = false;
        if let Some(counts) = self.section_to_chunk_counts.get_mut(&section_key) {
            let remove_chunk = match counts.get_mut(&chunk_key_value) {
                Some(count) => {
                    *count -= 1;
                    *count == 0
                }
                None => false,
            };
            if remove_chunk {
                counts.remove(&chunk_key_value);
            }
            remove_section = counts.is_empty();
        }
        if remove_section {
            self.section_to_chunk_counts.remove(&section_key);
        }
    }

    fn checksum(&self) -> u64 {
        let mut sum = 0u64;
        let shift = self.region_chunk_shift();
        for index in 0..self.data.op_count {
            let section_key = chunk_key(self.data.chunk_x[index] >> shift, self.data.chunk_z[index] >> shift);
            if let Some(counts) = self.section_to_chunk_counts.get(&section_key) {
                sum = sum.wrapping_add(*counts.get(&self.data.chunk_keys[index]).unwrap_or(&0) as u64);
            }
        }
        sum
    }

    fn region_chunk_shift(&self) -> i32 {
        match self.mode {
            Mode::DynamicCompute | Mode::DynamicManual => SECTION_SHIFT,
            Mode::CachedCompute | Mode::CachedHybrid | Mode::CachedManual => SECTION_SHIFT,
        }
    }
}

#[inline]
fn chunk_key(x: i32, z: i32) -> u64 {
    ((z as i64 as u64) << 32) | ((x as u32) as u64)
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
    fn hot_variants_match() {
        let dynamic = dynamic_compute_hot_summary(1024, 6, 3);
        let manual = dynamic_manual_hot_summary(1024, 6, 3);
        let cached_compute = cached_compute_hot_summary(1024, 6, 3);
        let cached_hybrid = cached_hybrid_hot_summary(1024, 6, 3);
        let cached_manual = cached_manual_hot_summary(1024, 6, 3);

        assert_eq!(dynamic.total, manual.total);
        assert_eq!(dynamic.total, cached_compute.total);
        assert_eq!(dynamic.total, cached_hybrid.total);
        assert_eq!(dynamic.total, cached_manual.total);
    }

    #[test]
    fn cold_variants_match() {
        let dynamic = dynamic_compute_cold_summary(6, 3);
        let manual = dynamic_manual_cold_summary(6, 3);
        let cached_compute = cached_compute_cold_summary(6, 3);
        let cached_hybrid = cached_hybrid_cold_summary(6, 3);
        let cached_manual = cached_manual_cold_summary(6, 3);

        assert_eq!(dynamic.total, manual.total);
        assert_eq!(dynamic.total, cached_compute.total);
        assert_eq!(dynamic.total, cached_hybrid.total);
        assert_eq!(dynamic.total, cached_manual.total);
    }
}
