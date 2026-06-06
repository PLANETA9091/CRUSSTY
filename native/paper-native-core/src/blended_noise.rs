pub const SUMMARY_FIELDS: usize = 4;

const ROUND_OFF: f64 = 33_554_432.0;
const XZ_MULTIPLIER: f64 = 684.412;
const Y_MULTIPLIER: f64 = 684.412;
const XZ_FACTOR: f64 = 80.0;
const Y_FACTOR: f64 = 160.0;
const SMEAR_SCALE_MULTIPLIER: f64 = 8.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlendedNoiseSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Debug)]
struct ModelImprovedNoise {
    xo: f64,
    yo: f64,
    zo: f64,
}

impl ModelImprovedNoise {
    fn new(seed: u32) -> Self {
        Self {
            xo: (((seed >> 1) & 255) as f64) * 0.03125,
            yo: (((seed >> 9) & 255) as f64) * 0.03125,
            zo: (((seed >> 17) & 255) as f64) * 0.03125,
        }
    }

    fn noise(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let dx = x + self.xo;
        let dy = y + self.yo;
        let dz = z + self.zo;
        let a = dx * 0.000_001 + dy * 0.000_003 - dz * 0.000_002;
        let b = (dx - dy) * (dz + y_scale + y_max) * 0.000_000_000_001;
        a + b
    }
}

#[derive(Clone, Debug)]
struct ModelPerlin {
    noise_levels: Vec<ModelImprovedNoise>,
}

impl ModelPerlin {
    fn new(size: usize, seed: u32) -> Self {
        let mut noise_levels = Vec::with_capacity(size);
        for index in 0..size {
            noise_levels.push(ModelImprovedNoise::new(seed.wrapping_add((index as u32).wrapping_mul(0x9E37))));
        }
        Self { noise_levels }
    }

    #[inline]
    fn get_octave_noise(&self, octave: usize) -> &ModelImprovedNoise {
        &self.noise_levels[self.noise_levels.len() - 1 - octave]
    }

    #[inline]
    fn copy_octaves(&self) -> Vec<ModelImprovedNoise> {
        let mut octaves = Vec::with_capacity(self.noise_levels.len());
        for octave in 0..self.noise_levels.len() {
            octaves.push(self.get_octave_noise(octave).clone());
        }
        octaves
    }
}

#[derive(Clone, Debug)]
struct OldBlendedNoise {
    min_limit_noise: ModelPerlin,
    max_limit_noise: ModelPerlin,
    main_noise: ModelPerlin,
}

impl OldBlendedNoise {
    fn new() -> Self {
        Self {
            min_limit_noise: ModelPerlin::new(16, 0x7a17),
            max_limit_noise: ModelPerlin::new(16, 0x51b9),
            main_noise: ModelPerlin::new(8, 0x3411),
        }
    }

    fn compute(&self, block_x: i32, block_y: i32, block_z: i32) -> f64 {
        let d = (block_x as f64) * XZ_MULTIPLIER;
        let d1 = (block_y as f64) * Y_MULTIPLIER;
        let d2 = (block_z as f64) * XZ_MULTIPLIER;
        let d3 = d / XZ_FACTOR;
        let d4 = d1 / Y_FACTOR;
        let d5 = d2 / XZ_FACTOR;
        let d6 = Y_MULTIPLIER * SMEAR_SCALE_MULTIPLIER;
        let d7 = d6 / Y_FACTOR;
        let mut d8 = 0.0;
        let mut d9 = 0.0;
        let mut d10 = 0.0;
        let mut d11 = 1.0;

        for octave in 0..self.main_noise.noise_levels.len() {
            let octave_noise = self.main_noise.get_octave_noise(octave);
            d10 += octave_noise.noise(
                wrap(d3 * d11),
                wrap(d4 * d11),
                wrap(d5 * d11),
                d7 * d11,
                d4 * d11,
            ) / d11;
            d11 *= 0.5;
        }

        let d12 = (d10 / 10.0 + 1.0) / 2.0;
        let flag1 = d12 >= 1.0;
        let flag2 = d12 <= 0.0;
        d11 = 1.0;

        for octave in 0..self.min_limit_noise.noise_levels.len() {
            let d13 = wrap(d * d11);
            let d14 = wrap(d1 * d11);
            let d15 = wrap(d2 * d11);
            let d16 = d6 * d11;
            if !flag1 {
                let octave_noise = self.min_limit_noise.get_octave_noise(octave);
                d8 += octave_noise.noise(d13, d14, d15, d16, d1 * d11) / d11;
            }
            if !flag2 {
                let octave_noise = self.max_limit_noise.get_octave_noise(octave);
                d9 += octave_noise.noise(d13, d14, d15, d16, d1 * d11) / d11;
            }
            d11 *= 0.5;
        }

        lerp(d12, d8 / 512.0, d9 / 512.0) / 128.0
    }
}

#[derive(Clone, Debug)]
struct CachedBlendedNoise {
    min_limit_octaves: Vec<ModelImprovedNoise>,
    max_limit_octaves: Vec<ModelImprovedNoise>,
    main_octaves: Vec<ModelImprovedNoise>,
}

impl CachedBlendedNoise {
    fn new() -> Self {
        let min_limit_noise = ModelPerlin::new(16, 0x7a17);
        let max_limit_noise = ModelPerlin::new(16, 0x51b9);
        let main_noise = ModelPerlin::new(8, 0x3411);
        Self {
            min_limit_octaves: min_limit_noise.copy_octaves(),
            max_limit_octaves: max_limit_noise.copy_octaves(),
            main_octaves: main_noise.copy_octaves(),
        }
    }

    fn compute(&self, block_x: i32, block_y: i32, block_z: i32) -> f64 {
        let d = (block_x as f64) * XZ_MULTIPLIER;
        let d1 = (block_y as f64) * Y_MULTIPLIER;
        let d2 = (block_z as f64) * XZ_MULTIPLIER;
        let d3 = d / XZ_FACTOR;
        let d4 = d1 / Y_FACTOR;
        let d5 = d2 / XZ_FACTOR;
        let d6 = Y_MULTIPLIER * SMEAR_SCALE_MULTIPLIER;
        let d7 = d6 / Y_FACTOR;
        let mut d8 = 0.0;
        let mut d9 = 0.0;
        let mut d10 = 0.0;
        let mut d11 = 1.0;

        for octave_noise in &self.main_octaves {
            d10 += octave_noise.noise(
                wrap(d3 * d11),
                wrap(d4 * d11),
                wrap(d5 * d11),
                d7 * d11,
                d4 * d11,
            ) / d11;
            d11 *= 0.5;
        }

        let d12 = (d10 / 10.0 + 1.0) / 2.0;
        let flag1 = d12 >= 1.0;
        let flag2 = d12 <= 0.0;
        d11 = 1.0;

        for octave in 0..self.min_limit_octaves.len() {
            let d13 = wrap(d * d11);
            let d14 = wrap(d1 * d11);
            let d15 = wrap(d2 * d11);
            let d16 = d6 * d11;
            if !flag1 {
                let octave_noise = &self.min_limit_octaves[octave];
                d8 += octave_noise.noise(d13, d14, d15, d16, d1 * d11) / d11;
            }
            if !flag2 {
                let octave_noise = &self.max_limit_octaves[octave];
                d9 += octave_noise.noise(d13, d14, d15, d16, d1 * d11) / d11;
            }
            d11 *= 0.5;
        }

        lerp(d12, d8 / 512.0, d9 / 512.0) / 128.0
    }
}

pub fn old_loop_summary(iterations: usize) -> BlendedNoiseSummary {
    let noise = OldBlendedNoise::new();
    run_loop_summary(iterations, |iteration| {
        let block_x = block_x(iteration);
        let block_y = block_y(iteration);
        let block_z = block_z(iteration);
        noise.compute(block_x, block_y, block_z)
    })
}

pub fn cached_loop_summary(iterations: usize) -> BlendedNoiseSummary {
    let noise = CachedBlendedNoise::new();
    run_loop_summary(iterations, |iteration| {
        let block_x = block_x(iteration);
        let block_y = block_y(iteration);
        let block_z = block_z(iteration);
        noise.compute(block_x, block_y, block_z)
    })
}

fn run_loop_summary<F>(iterations: usize, mut sample: F) -> BlendedNoiseSummary
where
    F: FnMut(usize) -> f64,
{
    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        let value = sample(iteration);
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((iteration as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(11)),
        );
    }

    BlendedNoiseSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    }
}

#[inline]
fn block_x(iteration: usize) -> i32 {
    ((iteration as i32).wrapping_mul(31)) & 0xFFFF
}

#[inline]
fn block_y(iteration: usize) -> i32 {
    ((iteration as i32).wrapping_mul(7)) & 0x17F
}

#[inline]
fn block_z(iteration: usize) -> i32 {
    ((iteration as i32).wrapping_mul(53)) & 0xFFFF
}

#[inline]
fn wrap(value: f64) -> f64 {
    value - (value / ROUND_OFF + 0.5).floor() * ROUND_OFF
}

#[inline]
fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
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
    fn old_and_cached_summaries_match() {
        let old = old_loop_summary(4096);
        let cached = cached_loop_summary(4096);
        assert_eq!(old, cached);
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = cached_loop_summary(10_000);
        let second = cached_loop_summary(10_000);
        assert_eq!(first, second);
        assert_eq!(first.count, 10_000);
        assert_ne!(first.value_checksum, 0);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let summary = old_loop_summary(0);
        assert_eq!(summary, BlendedNoiseSummary::default());
    }

    #[test]
    fn wrap_matches_java_perlin_wrap_formula() {
        for value in [
            -1.0e40,
            -67_108_864.75,
            -33_554_432.5,
            -30_000_000.0,
            -0.25,
            0.0,
            0.25,
            30_000_000.0,
            33_554_432.5,
            67_108_864.75,
            1.0e40,
        ] {
            let expected = value - (value / ROUND_OFF + 0.5).floor() * ROUND_OFF;
            assert_eq!(wrap(value).to_bits(), expected.to_bits(), "{value}");
        }
    }
}
