use std::hint::black_box;
use std::time::Instant;

use paper_native_chunk_encode_core::{
    encode_light_data, encode_section_data, LightEncodeInput, SectionEncodeInput, LIGHT_UPDATE_BYTES,
};

const DEFAULT_ROUNDS: usize = 8;
const DEFAULT_ITERATIONS: usize = 20_000;
const SECTIONS: usize = 24;

fn main() {
    let rounds = setting("NATIVE_CHUNK_ENCODE_ROUNDS", DEFAULT_ROUNDS);
    let iterations = setting("NATIVE_CHUNK_ENCODE_ITERATIONS", DEFAULT_ITERATIONS);
    let section_fixture = SectionFixture::new();
    let light_fixture = LightFixture::new();

    let mut best_section = u128::MAX;
    let mut best_light = u128::MAX;
    let mut digest = 0u64;

    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iterations {
            let mut out = Vec::with_capacity(section_fixture.expected_len);
            encode_section_data(&section_fixture.input(), &mut out).unwrap();
            digest ^= checksum(&out);
            black_box(&out);
        }
        best_section = best_section.min(start.elapsed().as_nanos());

        let start = Instant::now();
        for _ in 0..iterations {
            let mut out = Vec::with_capacity(light_fixture.expected_len);
            encode_light_data(&light_fixture.input(), &mut out).unwrap();
            digest ^= checksum(&out);
            black_box(&out);
        }
        best_light = best_light.min(start.elapsed().as_nanos());
    }

    println!("section_encode_best_ms={:.3}", best_section as f64 / 1_000_000.0);
    println!("section_encode_ns_per_call={:.1}", best_section as f64 / iterations as f64);
    println!("light_encode_best_ms={:.3}", best_light as f64 / 1_000_000.0);
    println!("light_encode_ns_per_call={:.1}", best_light as f64 / iterations as f64);
    println!("digest={digest}");
}

fn setting(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

struct SectionFixture {
    non_empty_counts: Vec<i16>,
    state_bits: Vec<u8>,
    state_palette_offsets: Vec<i32>,
    state_palette_bytes: Vec<u8>,
    state_storage_offsets: Vec<i32>,
    state_storage_longs: Vec<i64>,
    biome_bits: Vec<u8>,
    biome_palette_offsets: Vec<i32>,
    biome_palette_bytes: Vec<u8>,
    biome_storage_offsets: Vec<i32>,
    biome_storage_longs: Vec<i64>,
    expected_len: usize,
}

impl SectionFixture {
    fn new() -> Self {
        let mut fixture = Self {
            non_empty_counts: Vec::with_capacity(SECTIONS),
            state_bits: Vec::with_capacity(SECTIONS),
            state_palette_offsets: vec![0],
            state_palette_bytes: Vec::new(),
            state_storage_offsets: vec![0],
            state_storage_longs: Vec::new(),
            biome_bits: Vec::with_capacity(SECTIONS),
            biome_palette_offsets: vec![0],
            biome_palette_bytes: Vec::new(),
            biome_storage_offsets: vec![0],
            biome_storage_longs: Vec::new(),
            expected_len: 0,
        };

        for section in 0..SECTIONS {
            fixture.non_empty_counts.push(256 + section as i16);
            fixture.state_bits.push(8);
            fixture.state_palette_bytes.extend_from_slice(&[0x03, section as u8, 0x01, 0x02]);
            fixture.state_palette_offsets.push(fixture.state_palette_bytes.len() as i32);
            for word in 0..256 {
                fixture
                    .state_storage_longs
                    .push(((section as i64) << 32) ^ word as i64 ^ 0x55AA55AA);
            }
            fixture.state_storage_offsets.push(fixture.state_storage_longs.len() as i32);

            fixture.biome_bits.push(2);
            fixture.biome_palette_bytes.extend_from_slice(&[0x01, section as u8]);
            fixture.biome_palette_offsets.push(fixture.biome_palette_bytes.len() as i32);
            fixture.biome_storage_longs.push(section as i64 * 17);
            fixture.biome_storage_offsets.push(fixture.biome_storage_longs.len() as i32);
        }

        let mut out = Vec::new();
        encode_section_data(&fixture.input(), &mut out).unwrap();
        fixture.expected_len = out.len();
        fixture
    }

    fn input(&self) -> SectionEncodeInput<'_> {
        SectionEncodeInput {
            non_empty_counts: &self.non_empty_counts,
            state_bits: &self.state_bits,
            state_palette_offsets: &self.state_palette_offsets,
            state_palette_bytes: &self.state_palette_bytes,
            state_storage_offsets: &self.state_storage_offsets,
            state_storage_longs: &self.state_storage_longs,
            biome_bits: &self.biome_bits,
            biome_palette_offsets: &self.biome_palette_offsets,
            biome_palette_bytes: &self.biome_palette_bytes,
            biome_storage_offsets: &self.biome_storage_offsets,
            biome_storage_longs: &self.biome_storage_longs,
        }
    }
}

struct LightFixture {
    sky_updates: Vec<u8>,
    block_updates: Vec<u8>,
    expected_len: usize,
}

impl LightFixture {
    fn new() -> Self {
        let sky_updates = vec![0x5A; LIGHT_UPDATE_BYTES * 24];
        let block_updates = vec![0xA5; LIGHT_UPDATE_BYTES * 18];
        let mut fixture = Self {
            sky_updates,
            block_updates,
            expected_len: 0,
        };
        let mut out = Vec::new();
        encode_light_data(&fixture.input(), &mut out).unwrap();
        fixture.expected_len = out.len();
        fixture
    }

    fn input(&self) -> LightEncodeInput<'_> {
        LightEncodeInput {
            sky_y_mask_longs: &[0x00FF_FFFF],
            block_y_mask_longs: &[0x0003_FFFF],
            empty_sky_y_mask_longs: &[0],
            empty_block_y_mask_longs: &[0],
            sky_updates: &self.sky_updates,
            sky_update_count: 24,
            block_updates: &self.block_updates,
            block_update_count: 18,
        }
    }
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
