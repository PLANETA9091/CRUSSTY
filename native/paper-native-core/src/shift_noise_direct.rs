pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShiftNoiseDirectSummary {
    pub count: u64,
    pub xor_bits: u64,
    pub checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShiftNoiseDirectVariant {
    CurrentDefault,
    DirectDefault,
    CurrentA,
    DirectA,
    CurrentB,
    DirectB,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShiftNoiseDirectError {
    InvalidInputLength,
    InvalidVariant,
}

pub fn current_default_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    run_summary(
        iterations,
        block_x,
        block_y,
        block_z,
        ShiftNoiseDirectVariant::CurrentDefault,
        true,
    )
}

pub fn direct_default_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    run_summary(
        iterations,
        block_x,
        block_y,
        block_z,
        ShiftNoiseDirectVariant::DirectDefault,
        false,
    )
}

pub fn current_a_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    run_summary(
        iterations,
        block_x,
        block_y,
        block_z,
        ShiftNoiseDirectVariant::CurrentA,
        true,
    )
}

pub fn direct_a_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    run_summary(
        iterations,
        block_x,
        block_y,
        block_z,
        ShiftNoiseDirectVariant::DirectA,
        false,
    )
}

pub fn current_b_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    run_summary(
        iterations,
        block_x,
        block_y,
        block_z,
        ShiftNoiseDirectVariant::CurrentB,
        true,
    )
}

pub fn direct_b_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    run_summary(
        iterations,
        block_x,
        block_y,
        block_z,
        ShiftNoiseDirectVariant::DirectB,
        false,
    )
}

fn run_summary(
    iterations: usize,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    variant: ShiftNoiseDirectVariant,
    current: bool,
) -> Result<ShiftNoiseDirectSummary, ShiftNoiseDirectError> {
    if iterations == 0 {
        return Ok(ShiftNoiseDirectSummary::default());
    }

    let len = block_x.len();
    if len == 0 || len != block_y.len() || len != block_z.len() || iterations > len {
        return Err(ShiftNoiseDirectError::InvalidInputLength);
    }

    let mut xor_bits = 0u64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        let value = match (variant, current) {
            (ShiftNoiseDirectVariant::CurrentDefault, true) => current_default(block_x[iteration], block_y[iteration], block_z[iteration]),
            (ShiftNoiseDirectVariant::DirectDefault, false) => direct_default(block_x[iteration], block_y[iteration], block_z[iteration]),
            (ShiftNoiseDirectVariant::CurrentA, true) => current_a(block_x[iteration], block_z[iteration]),
            (ShiftNoiseDirectVariant::DirectA, false) => direct_a(block_x[iteration], block_z[iteration]),
            (ShiftNoiseDirectVariant::CurrentB, true) => current_b(block_x[iteration], block_z[iteration]),
            (ShiftNoiseDirectVariant::DirectB, false) => direct_b(block_x[iteration], block_z[iteration]),
            _ => return Err(ShiftNoiseDirectError::InvalidVariant),
        };
        let bits = canonical_double_bits(value);
        xor_bits ^= bits;
        last_bits = bits;
        checksum = mix64(
            checksum
                ^ bits
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ variant.marker(),
        );
    }

    Ok(ShiftNoiseDirectSummary {
        count: iterations as u64,
        xor_bits,
        checksum,
        last_bits,
    })
}

fn current_default(block_x: i32, block_y: i32, block_z: i32) -> f64 {
    current_helper(block_x as f64, block_y as f64, block_z as f64)
}

fn direct_default(block_x: i32, block_y: i32, block_z: i32) -> f64 {
    direct_compute(block_x as f64, block_y as f64, block_z as f64)
}

fn current_a(block_x: i32, block_z: i32) -> f64 {
    current_helper(block_x as f64, 0.0, block_z as f64)
}

fn direct_a(block_x: i32, block_z: i32) -> f64 {
    direct_compute(block_x as f64, 0.0, block_z as f64)
}

fn current_b(block_x: i32, block_z: i32) -> f64 {
    current_helper(block_z as f64, block_x as f64, 0.0)
}

fn direct_b(block_x: i32, block_z: i32) -> f64 {
    direct_compute(block_z as f64, block_x as f64, 0.0)
}

#[inline]
fn current_helper(x: f64, y: f64, z: f64) -> f64 {
    noise_value(x * 0.25, y * 0.25, z * 0.25) * 4.0
}

#[inline]
fn direct_compute(x: f64, y: f64, z: f64) -> f64 {
    noise_value(x * 0.25, y * 0.25, z * 0.25) * 4.0
}

#[inline]
fn noise_value(x: f64, y: f64, z: f64) -> f64 {
    x * 0.125 + y * -0.375 + z * 0.25
}

impl ShiftNoiseDirectVariant {
    fn marker(self) -> u64 {
        match self {
            ShiftNoiseDirectVariant::CurrentDefault => 0x4342_4C00_4446_4C54,
            ShiftNoiseDirectVariant::DirectDefault => 0x4442_4C00_4446_4C54,
            ShiftNoiseDirectVariant::CurrentA => 0x4342_4C00_4446_4C41,
            ShiftNoiseDirectVariant::DirectA => 0x4442_4C00_4446_4C41,
            ShiftNoiseDirectVariant::CurrentB => 0x4342_4C00_4446_4C42,
            ShiftNoiseDirectVariant::DirectB => 0x4442_4C00_4446_4C42,
        }
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

    fn build_samples(len: usize) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
        let mut block_x = Vec::with_capacity(len);
        let mut block_y = Vec::with_capacity(len);
        let mut block_z = Vec::with_capacity(len);

        for i in 0..len {
            block_x.push((i as i32 * 97) - 31_000);
            block_y.push((i as i32 * 53) - 127);
            block_z.push((i as i32 * 71) - 29_000);
        }

        (block_x, block_y, block_z)
    }

    #[test]
    fn current_and_direct_match_on_regular_inputs() {
        let (block_x, block_y, block_z) = build_samples(1 << 14);

        let pairs = [
            (
                current_default_summary(16_384, &block_x, &block_y, &block_z).unwrap(),
                direct_default_summary(16_384, &block_x, &block_y, &block_z).unwrap(),
            ),
            (
                current_a_summary(16_384, &block_x, &block_y, &block_z).unwrap(),
                direct_a_summary(16_384, &block_x, &block_y, &block_z).unwrap(),
            ),
            (
                current_b_summary(16_384, &block_x, &block_y, &block_z).unwrap(),
                direct_b_summary(16_384, &block_x, &block_y, &block_z).unwrap(),
            ),
        ];

        for (current, direct) in pairs {
            assert_eq!(current.count, direct.count);
            assert_eq!(current.xor_bits, direct.xor_bits);
            assert_eq!(current.last_bits, direct.last_bits);
        }
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (block_x, block_y, block_z) = build_samples(1 << 4);
        let summary = current_default_summary(0, &block_x, &block_y, &block_z).unwrap();
        assert_eq!(summary, ShiftNoiseDirectSummary::default());
    }

    #[test]
    fn rejects_bad_shapes() {
        let (block_x, block_y, block_z) = build_samples(16);
        assert_eq!(
            current_default_summary(17, &block_x, &block_y, &block_z).unwrap_err(),
            ShiftNoiseDirectError::InvalidInputLength
        );
        assert_eq!(
            current_default_summary(8, &block_x[..8], &block_y[..7], &block_z[..8]).unwrap_err(),
            ShiftNoiseDirectError::InvalidInputLength
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (block_x, block_y, block_z) = build_samples(1 << 12);
        let first = direct_b_summary(4_096, &block_x, &block_y, &block_z).unwrap();
        let second = direct_b_summary(4_096, &block_x, &block_y, &block_z).unwrap();
        assert_eq!(first, second);
    }
}
