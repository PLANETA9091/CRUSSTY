use crate::improved_noise::{ImprovedNoise, ImprovedNoiseError};

pub const SUMMARY_FIELDS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImprovedNoiseFloorError {
    InvalidPermutationLength,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImprovedNoiseFloorSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[inline]
pub fn current_mth_floor_summary(
    permutation: &[u8],
    iterations: usize,
) -> Result<ImprovedNoiseFloorSummary, ImprovedNoiseFloorError> {
    floor_summary(permutation, iterations, false)
}

#[inline]
pub fn math_floor_summary(
    permutation: &[u8],
    iterations: usize,
) -> Result<ImprovedNoiseFloorSummary, ImprovedNoiseFloorError> {
    floor_summary(permutation, iterations, true)
}

fn floor_summary(
    permutation: &[u8],
    iterations: usize,
    math_floor: bool,
) -> Result<ImprovedNoiseFloorSummary, ImprovedNoiseFloorError> {
    let noise = ImprovedNoise::new(permutation, 17.375, 201.625, 93.125)
        .map_err(|code| match code {
            ImprovedNoiseError::InvalidPermutationLength => ImprovedNoiseFloorError::InvalidPermutationLength,
            ImprovedNoiseError::LengthMismatch => ImprovedNoiseFloorError::InvalidPermutationLength,
        })?;
    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for i in 0..iterations {
        let value = if math_floor {
            noise.noise_math_floor(x(i), y(i), z(i), y_scale(i), y_max(i))
        } else {
            noise.noise(x(i), y(i), z(i), y_scale(i), y_max(i))
        };
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64) << 7),
        );
    }

    Ok(ImprovedNoiseFloorSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    })
}

#[inline]
fn x(i: usize) -> f64 {
    ((i as f64 * 37.03125) % 4_194_304.0) - 2_097_152.0
}

#[inline]
fn y(i: usize) -> f64 {
    ((i as f64 * 13.015625) % 768.0) - 128.0
}

#[inline]
fn z(i: usize) -> f64 {
    ((i as f64 * 53.046875) % 4_194_304.0) - 2_097_152.0
}

#[inline]
fn y_scale(i: usize) -> f64 {
    if (i & 7) == 0 { 0.25 } else { 0.0 }
}

#[inline]
fn y_max(i: usize) -> f64 {
    if (i & 15) == 0 { 0.5 } else { 0.0 }
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
    fn current_and_math_floor_match() {
        let permutation = permutation();
        let current = current_mth_floor_summary(&permutation, 25_000).unwrap();
        let math = math_floor_summary(&permutation, 25_000).unwrap();
        assert_eq!(current, math);
    }

    #[test]
    fn rejects_bad_permutation_length() {
        assert_eq!(
            current_mth_floor_summary(&[0; 255], 1),
            Err(ImprovedNoiseFloorError::InvalidPermutationLength)
        );
    }

    fn permutation() -> [u8; 256] {
        let mut permutation = [0u8; 256];
        for (i, value) in permutation.iter_mut().enumerate() {
            *value = i as u8;
        }

        let mut state = 0x5eed1234u32;
        for i in 0..permutation.len() {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            let random_int = state % (256 - i) as u32;
            permutation.swap(i, i + random_int as usize);
        }

        permutation
    }
}
