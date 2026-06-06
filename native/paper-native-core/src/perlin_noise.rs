use std::convert::TryFrom;

use crate::improved_noise::{ImprovedNoise, ImprovedNoiseError, PERMUTATION_LENGTH};

pub const SUMMARY_FIELDS: usize = 4;
const ROUND_OFF: f64 = 33_554_432.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerlinNoiseError {
    InvalidOctaveCount,
    InvalidInputLength,
    InvalidPermutationLength,
    InvalidVariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerlinGetValueVariant {
    Delegating,
    Direct,
    DirectLocal,
    DirectLocalGuarded,
    DirectNoYScale,
    DirectMathWrap,
}

impl TryFrom<u8> for PerlinGetValueVariant {
    type Error = PerlinNoiseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Delegating),
            1 => Ok(Self::Direct),
            2 => Ok(Self::DirectLocal),
            3 => Ok(Self::DirectLocalGuarded),
            4 => Ok(Self::DirectNoYScale),
            5 => Ok(Self::DirectMathWrap),
            _ => Err(PerlinNoiseError::InvalidVariant),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PerlinNoiseSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Debug)]
struct NoiseLevel {
    noise: ImprovedNoise,
    amplitude: f64,
    y_origin: f64,
    input_factor: f64,
    value_factor: f64,
}

#[derive(Clone, Debug)]
pub struct PerlinNoise {
    levels: Vec<NoiseLevel>,
}

impl PerlinNoise {
    pub fn new_from_flat(
        permutations: &[u8],
        active: &[u8],
        y_origins: &[f64],
        amplitudes: &[f64],
        lowest_freq_input_factor: f64,
        lowest_freq_value_factor: f64,
    ) -> Result<Self, PerlinNoiseError> {
        let octaves = active.len();
        if octaves == 0 || y_origins.len() != octaves || amplitudes.len() != octaves {
            return Err(PerlinNoiseError::InvalidOctaveCount);
        }
        if permutations.len() != octaves * PERMUTATION_LENGTH {
            return Err(PerlinNoiseError::InvalidPermutationLength);
        }

        let mut levels = Vec::with_capacity(
            active.iter().filter(|&&enabled| enabled != 0).count(),
        );
        let mut input_factor = lowest_freq_input_factor;
        let mut value_factor = lowest_freq_value_factor;
        for octave in 0..octaves {
            if active[octave] == 0 {
                input_factor *= 2.0;
                value_factor /= 2.0;
                continue;
            }

            let offset = octave * PERMUTATION_LENGTH;
            let noise = ImprovedNoise::new(
                &permutations[offset..offset + PERMUTATION_LENGTH],
                0.0,
                0.0,
                0.0,
            )
            .map_err(|code| match code {
                ImprovedNoiseError::InvalidPermutationLength => PerlinNoiseError::InvalidPermutationLength,
                ImprovedNoiseError::LengthMismatch => PerlinNoiseError::InvalidInputLength,
            })?;
            levels.push(NoiseLevel {
                noise,
                amplitude: amplitudes[octave],
                y_origin: y_origins[octave],
                input_factor,
                value_factor,
            });

            input_factor *= 2.0;
            value_factor /= 2.0;
        }

        Ok(Self { levels })
    }

    pub fn new_from_flat_with_origins(
        permutations: &[u8],
        active: &[u8],
        x_origins: &[f64],
        y_origins: &[f64],
        z_origins: &[f64],
        amplitudes: &[f64],
        lowest_freq_input_factor: f64,
        lowest_freq_value_factor: f64,
    ) -> Result<Self, PerlinNoiseError> {
        let octaves = active.len();
        if octaves == 0
            || x_origins.len() != octaves
            || y_origins.len() != octaves
            || z_origins.len() != octaves
            || amplitudes.len() != octaves
        {
            return Err(PerlinNoiseError::InvalidOctaveCount);
        }
        if permutations.len() != octaves * PERMUTATION_LENGTH {
            return Err(PerlinNoiseError::InvalidPermutationLength);
        }

        let mut levels = Vec::with_capacity(
            active.iter().filter(|&&enabled| enabled != 0).count(),
        );
        let mut input_factor = lowest_freq_input_factor;
        let mut value_factor = lowest_freq_value_factor;
        for octave in 0..octaves {
            if active[octave] == 0 {
                input_factor *= 2.0;
                value_factor /= 2.0;
                continue;
            }

            let offset = octave * PERMUTATION_LENGTH;
            let noise = ImprovedNoise::new(
                &permutations[offset..offset + PERMUTATION_LENGTH],
                x_origins[octave],
                y_origins[octave],
                z_origins[octave],
            )
            .map_err(|code| match code {
                ImprovedNoiseError::InvalidPermutationLength => PerlinNoiseError::InvalidPermutationLength,
                ImprovedNoiseError::LengthMismatch => PerlinNoiseError::InvalidInputLength,
            })?;
            levels.push(NoiseLevel {
                noise,
                amplitude: amplitudes[octave],
                y_origin: y_origins[octave],
                input_factor,
                value_factor,
            });

            input_factor *= 2.0;
            value_factor /= 2.0;
        }

        Ok(Self { levels })
    }

    pub fn get_value(
        &self,
        x: f64,
        y: f64,
        z: f64,
        y_scale: f64,
        y_max: f64,
        use_fixed_y: bool,
    ) -> f64 {
        if y_scale == 0.0 {
            return if use_fixed_y {
                self.get_value_fixed_y_no_y_scale(x, z)
            } else {
                self.get_value_direct_no_y_scale(x, y, z)
            };
        }

        let mut result = 0.0;
        if use_fixed_y {
            for level in &self.levels {
                let value = level.noise.noise(
                    wrap(x * level.input_factor),
                    -level.y_origin,
                    wrap(z * level.input_factor),
                    y_scale * level.input_factor,
                    y_max * level.input_factor,
                );
                result += level.amplitude * value * level.value_factor;
            }
        } else {
            for level in &self.levels {
                let value = level.noise.noise(
                    wrap(x * level.input_factor),
                    wrap(y * level.input_factor),
                    wrap(z * level.input_factor),
                    y_scale * level.input_factor,
                    y_max * level.input_factor,
                );
                result += level.amplitude * value * level.value_factor;
            }
        }

        result
    }

    #[inline]
    fn get_value_fixed_y_no_y_scale(&self, x: f64, z: f64) -> f64 {
        let mut result = 0.0;
        for level in &self.levels {
            let value = level.noise.noise_no_y_scale(
                wrap(x * level.input_factor),
                -level.y_origin,
                wrap(z * level.input_factor),
            );
            result += level.amplitude * value * level.value_factor;
        }

        result
    }

    pub fn get_value_delegating(&self, x: f64, y: f64, z: f64) -> f64 {
        self.get_value(x, y, z, 0.0, 0.0, false)
    }

    pub fn get_value_direct(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut result = 0.0;
        for level in &self.levels {
            let value = level.noise.noise(
                wrap(x * level.input_factor),
                wrap(y * level.input_factor),
                wrap(z * level.input_factor),
                0.0,
                0.0,
            );
            result += level.amplitude * value * level.value_factor;
        }

        result
    }

    pub fn get_value_direct_local(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut result = 0.0;
        let levels = &self.levels;

        for i in 0..levels.len() {
            let level = &levels[i];
            let value = level.noise.noise(
                wrap(x * level.input_factor),
                wrap(y * level.input_factor),
                wrap(z * level.input_factor),
                0.0,
                0.0,
            );
            result += level.amplitude * value * level.value_factor;
        }

        result
    }

    pub fn get_value_direct_local_guarded(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut result = 0.0;
        let levels = &self.levels;

        for i in 0..levels.len() {
            let level = &levels[i];
            let value = level.noise.noise(
                wrap(x * level.input_factor),
                wrap(y * level.input_factor),
                wrap(z * level.input_factor),
                0.0,
                0.0,
            );
            result += level.amplitude * value * level.value_factor;
        }

        result
    }

    pub fn get_value_direct_no_y_scale(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut result = 0.0;
        let levels = &self.levels;

        for i in 0..levels.len() {
            let level = &levels[i];
            let value = level.noise.noise_no_y_scale(
                wrap(x * level.input_factor),
                wrap(y * level.input_factor),
                wrap(z * level.input_factor),
            );
            result += level.amplitude * value * level.value_factor;
        }

        result
    }

    pub fn get_value_direct_math_wrap(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut result = 0.0;
        for level in &self.levels {
            let value = level.noise.noise_no_y_scale(
                wrap_math(x * level.input_factor),
                wrap_math(y * level.input_factor),
                wrap_math(z * level.input_factor),
            );
            result += level.amplitude * value * level.value_factor;
        }

        result
    }

    pub fn get_value_variant(
        &self,
        x: f64,
        y: f64,
        z: f64,
        variant: PerlinGetValueVariant,
    ) -> f64 {
        match variant {
            PerlinGetValueVariant::Delegating => self.get_value_delegating(x, y, z),
            PerlinGetValueVariant::Direct => self.get_value_direct(x, y, z),
            PerlinGetValueVariant::DirectLocal => self.get_value_direct_local(x, y, z),
            PerlinGetValueVariant::DirectLocalGuarded => {
                self.get_value_direct_local_guarded(x, y, z)
            }
            PerlinGetValueVariant::DirectNoYScale => self.get_value_direct_no_y_scale(x, y, z),
            PerlinGetValueVariant::DirectMathWrap => self.get_value_direct_math_wrap(x, y, z),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_value_batch_summary(
    permutations: &[u8],
    active: &[u8],
    y_origins: &[f64],
    amplitudes: &[f64],
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    y_scales: &[f64],
    y_maxes: &[f64],
    use_fixed_ys: &[u8],
    lowest_freq_input_factor: f64,
    lowest_freq_value_factor: f64,
    iterations: usize,
) -> Result<PerlinNoiseSummary, PerlinNoiseError> {
    let input_len = xs.len();
    if ys.len() != input_len
        || zs.len() != input_len
        || y_scales.len() != input_len
        || y_maxes.len() != input_len
        || use_fixed_ys.len() != input_len
        || (input_len == 0 && iterations != 0)
    {
        return Err(PerlinNoiseError::InvalidInputLength);
    }

    let perlin = PerlinNoise::new_from_flat(
        permutations,
        active,
        y_origins,
        amplitudes,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
    )?;

    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    let mut index = 0usize;
    for iteration in 0..iterations {
        let value = perlin.get_value(
            xs[index],
            ys[index],
            zs[index],
            y_scales[index],
            y_maxes[index],
            use_fixed_ys[index] != 0,
        );
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                ^ ((iteration as u64).rotate_left(29)),
        );
        index += 1;
        if index == input_len {
            index = 0;
        }
    }

    Ok(PerlinNoiseSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn get_value_variant_batch_summary(
    permutations: &[u8],
    active: &[u8],
    y_origins: &[f64],
    amplitudes: &[f64],
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    lowest_freq_input_factor: f64,
    lowest_freq_value_factor: f64,
    iterations: usize,
    variant: PerlinGetValueVariant,
) -> Result<PerlinNoiseSummary, PerlinNoiseError> {
    let input_len = xs.len();
    if ys.len() != input_len || zs.len() != input_len || (input_len == 0 && iterations != 0) {
        return Err(PerlinNoiseError::InvalidInputLength);
    }

    let perlin = PerlinNoise::new_from_flat(
        permutations,
        active,
        y_origins,
        amplitudes,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
    )?;

    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    let mut index = 0usize;
    for iteration in 0..iterations {
        let value = perlin.get_value_variant(xs[index], ys[index], zs[index], variant);
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                ^ ((iteration as u64).rotate_left(29)),
        );
        index += 1;
        if index == input_len {
            index = 0;
        }
    }

    Ok(PerlinNoiseSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    })
}

#[inline]
fn wrap(value: f64) -> f64 {
    value - (value / ROUND_OFF + 0.5).floor() * ROUND_OFF
}

#[inline]
fn wrap_math(value: f64) -> f64 {
    wrap(value)
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
    fn rejects_bad_shapes() {
        let (permutations, active, y_origins, amplitudes) = model_octaves(4);
        assert_eq!(
            get_value_batch_summary(
                &permutations[..permutations.len() - 1],
                &active,
                &y_origins,
                &amplitudes,
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                1.0,
                1.0,
                0,
            ),
            Err(PerlinNoiseError::InvalidPermutationLength)
        );
        assert_eq!(
            get_value_batch_summary(
                &permutations,
                &active,
                &y_origins,
                &amplitudes,
                &[1.0],
                &[],
                &[1.0],
                &[0.0],
                &[0.0],
                &[0],
                1.0,
                1.0,
                1,
            ),
            Err(PerlinNoiseError::InvalidInputLength)
        );
    }

    #[test]
    fn deterministic_summary_is_stable() {
        let (permutations, active, y_origins, amplitudes) = model_octaves(16);
        let xs = [-30_000_000.0, -1024.25, 0.0, 8192.5, 30_000_000.0];
        let ys = [-64.0, -1.25, 0.5, 63.75, 319.0];
        let zs = [-29_999_999.5, -128.0, 7.25, 1024.0, 29_999_999.75];
        let y_scales = [0.0, 0.125, 0.0, 0.25, 0.0];
        let y_maxes = [0.0, 0.25, 0.0, 0.5, 0.0];
        let use_fixed = [0, 1, 0, 1, 0];
        let (input_factor, value_factor) = model_factors(16);

        let first = get_value_batch_summary(
            &permutations,
            &active,
            &y_origins,
            &amplitudes,
            &xs,
            &ys,
            &zs,
            &y_scales,
            &y_maxes,
            &use_fixed,
            input_factor,
            value_factor,
            4096,
        )
        .unwrap();
        let second = get_value_batch_summary(
            &permutations,
            &active,
            &y_origins,
            &amplitudes,
            &xs,
            &ys,
            &zs,
            &y_scales,
            &y_maxes,
            &use_fixed,
            input_factor,
            value_factor,
            4096,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.count, 4096);
        assert_ne!(first.value_checksum, 0);
    }

    #[test]
    fn active_octaves_keep_original_frequency_spacing() {
        let (permutations, active, y_origins, amplitudes) = model_octaves(8);
        let (input_factor, value_factor) = model_factors(8);
        let perlin = PerlinNoise::new_from_flat(
            &permutations,
            &active,
            &y_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        assert_eq!(
            perlin.levels.len(),
            active.iter().filter(|&&enabled| enabled != 0).count()
        );

        let mut expected_input_factor = input_factor;
        let mut expected_value_factor = value_factor;
        let mut active_index = 0usize;
        for &enabled in &active {
            if enabled != 0 {
                let level = &perlin.levels[active_index];
                assert_eq!(level.input_factor.to_bits(), expected_input_factor.to_bits());
                assert_eq!(level.value_factor.to_bits(), expected_value_factor.to_bits());
                active_index += 1;
            }
            expected_input_factor *= 2.0;
            expected_value_factor /= 2.0;
        }
    }

    #[test]
    fn fixed_y_changes_the_sample_path() {
        let (permutations, active, y_origins, amplitudes) = model_octaves(8);
        let (input_factor, value_factor) = model_factors(8);
        let perlin = PerlinNoise::new_from_flat(
            &permutations,
            &active,
            &y_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        let normal = perlin.get_value(12.25, 5.5, -31.75, 0.125, 0.25, false);
        let fixed = perlin.get_value(12.25, 5.5, -31.75, 0.125, 0.25, true);
        assert_ne!(normal.to_bits(), fixed.to_bits());
    }

    #[test]
    fn zero_y_scale_matches_direct_no_y_scale_paths() {
        let (permutations, active, y_origins, amplitudes) = model_octaves(8);
        let (input_factor, value_factor) = model_factors(8);
        let perlin = PerlinNoise::new_from_flat(
            &permutations,
            &active,
            &y_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        let x = 12.25;
        let y = 5.5;
        let z = -31.75;

        let normal = perlin.get_value(x, y, z, 0.0, 123.0, false);
        let expected_normal = perlin.get_value_direct_no_y_scale(x, y, z);
        assert_eq!(normal.to_bits(), expected_normal.to_bits());

        let fixed = perlin.get_value(x, y, z, 0.0, 123.0, true);
        let mut expected_fixed = 0.0;
        for level in &perlin.levels {
            let value = level.noise.noise_no_y_scale(
                wrap(x * level.input_factor),
                -level.y_origin,
                wrap(z * level.input_factor),
            );
            expected_fixed += level.amplitude * value * level.value_factor;
        }

        assert_eq!(fixed.to_bits(), expected_fixed.to_bits());
    }

    #[test]
    fn variant_batch_matches_delegating_summary() {
        let (permutations, active, y_origins, amplitudes) = model_octaves(16);
        let xs = [-30_000_000.0, -1024.25, 0.0, 8192.5, 30_000_000.0];
        let ys = [-64.0, -1.25, 0.5, 63.75, 319.0];
        let zs = [-29_999_999.5, -128.0, 7.25, 1024.0, 29_999_999.75];
        let (input_factor, value_factor) = model_factors(16);
        let base = get_value_batch_summary(
            &permutations,
            &active,
            &y_origins,
            &amplitudes,
            &xs,
            &ys,
            &zs,
            &[0.0; 5],
            &[0.0; 5],
            &[0; 5],
            input_factor,
            value_factor,
            4096,
        )
        .unwrap();

        for variant in [
            PerlinGetValueVariant::Delegating,
            PerlinGetValueVariant::Direct,
            PerlinGetValueVariant::DirectLocal,
            PerlinGetValueVariant::DirectLocalGuarded,
            PerlinGetValueVariant::DirectNoYScale,
            PerlinGetValueVariant::DirectMathWrap,
        ] {
            let summary = get_value_variant_batch_summary(
                &permutations,
                &active,
                &y_origins,
                &amplitudes,
                &xs,
                &ys,
                &zs,
                input_factor,
                value_factor,
                4096,
                variant,
            )
            .unwrap();

            assert_eq!(summary, base, "{variant:?}");
        }
    }

    #[test]
    fn wrap_matches_java_math_floor_formula() {
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

    fn model_octaves(octaves: usize) -> (Vec<u8>, Vec<u8>, Vec<f64>, Vec<f64>) {
        let mut permutations = Vec::with_capacity(octaves * PERMUTATION_LENGTH);
        let mut active = Vec::with_capacity(octaves);
        let mut y_origins = Vec::with_capacity(octaves);
        let mut amplitudes = Vec::with_capacity(octaves);

        for octave in 0..octaves {
            let seed = (0x5eed_1234u32).wrapping_add((octave as u32).wrapping_mul(0x9E37_79B9));
            permutations.extend_from_slice(&permutation(seed));
            active.push(if (octave & 3) != 2 { 1 } else { 0 });
            y_origins.push(((seed >> 8) & 0xff) as f64 / 256.0);
            amplitudes.push(1.0 / (1 + (octave & 3)) as f64);
        }

        (permutations, active, y_origins, amplitudes)
    }

    fn model_factors(octaves: usize) -> (f64, f64) {
        let first_octave = -(octaves as i32) / 2;
        let negative_first_octave = -first_octave;
        (
            2.0f64.powi(-negative_first_octave),
            2.0f64.powi(octaves as i32 - 1) / (2.0f64.powi(octaves as i32) - 1.0),
        )
    }

    fn permutation(seed: u32) -> [u8; PERMUTATION_LENGTH] {
        let mut permutation = [0u8; PERMUTATION_LENGTH];
        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }

        let mut state = seed;
        for i in 0..PERMUTATION_LENGTH {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let random_int = state % ((PERMUTATION_LENGTH - i) as u32);
            permutation.swap(i, i + random_int as usize);
        }
        permutation
    }
}
