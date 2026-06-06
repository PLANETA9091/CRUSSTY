pub const PERMUTATION_LENGTH: usize = 256;
pub const SUMMARY_FIELDS: usize = 4;

const FLAT_GRADIENT: [f64; 64] = [
    1.0, 1.0, 0.0, 0.0,
    -1.0, 1.0, 0.0, 0.0,
    1.0, -1.0, 0.0, 0.0,
    -1.0, -1.0, 0.0, 0.0,
    1.0, 0.0, 1.0, 0.0,
    -1.0, 0.0, 1.0, 0.0,
    1.0, 0.0, -1.0, 0.0,
    -1.0, 0.0, -1.0, 0.0,
    0.0, 1.0, 1.0, 0.0,
    0.0, -1.0, 1.0, 0.0,
    0.0, 1.0, -1.0, 0.0,
    0.0, -1.0, -1.0, 0.0,
    1.0, 1.0, 0.0, 0.0,
    0.0, -1.0, 1.0, 0.0,
    -1.0, 1.0, 0.0, 0.0,
    0.0, -1.0, -1.0, 0.0,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImprovedNoiseError {
    InvalidPermutationLength,
    LengthMismatch,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImprovedNoiseSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Debug)]
pub struct ImprovedNoise {
    permutation: [u8; PERMUTATION_LENGTH],
    xo: f64,
    yo: f64,
    zo: f64,
}

impl ImprovedNoise {
    pub fn new(
        permutation: &[u8],
        xo: f64,
        yo: f64,
        zo: f64,
    ) -> Result<Self, ImprovedNoiseError> {
        if permutation.len() != PERMUTATION_LENGTH {
            return Err(ImprovedNoiseError::InvalidPermutationLength);
        }

        let mut copy = [0u8; PERMUTATION_LENGTH];
        copy.copy_from_slice(permutation);
        Ok(Self {
            permutation: copy,
            xo,
            yo,
            zo,
        })
    }

    #[inline]
    pub fn noise(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let d = x + self.xo;
        let d1 = y + self.yo;
        let d2 = z + self.zo;
        let floor_x = d.floor();
        let floor_y = d1.floor();
        let floor_z = d2.floor();
        let d3 = d - floor_x;
        let d4 = d1 - floor_y;
        let d5 = d2 - floor_z;
        let d7 = if y_scale != 0.0 {
            let d6 = if y_max >= 0.0 && y_max < d4 { y_max } else { d4 };
            (floor(d6 / y_scale + 1.0e-7_f32 as f64) as f64) * y_scale
        } else {
            0.0
        };

        self.sample_and_lerp(floor_x as i32, floor_y as i32, floor_z as i32, d3, d4 - d7, d5, d4)
    }

    #[inline]
    pub fn noise_no_y_scale(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = x + self.xo;
        let d1 = y + self.yo;
        let d2 = z + self.zo;
        let floor_x = d.floor();
        let floor_y = d1.floor();
        let floor_z = d2.floor();
        let d3 = d - floor_x;
        let d4 = d1 - floor_y;
        let d5 = d2 - floor_z;

        self.sample_and_lerp(floor_x as i32, floor_y as i32, floor_z as i32, d3, d4, d5, d4)
    }

    #[inline]
    pub fn noise_math_floor(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let d = x + self.xo;
        let d1 = y + self.yo;
        let d2 = z + self.zo;
        let floor_x = d.floor();
        let floor_y = d1.floor();
        let floor_z = d2.floor();
        let d3 = d - floor_x;
        let d4 = d1 - floor_y;
        let d5 = d2 - floor_z;
        let d7 = if y_scale != 0.0 {
            let d6 = if y_max >= 0.0 && y_max < d4 { y_max } else { d4 };
            (d6 / y_scale + 1.0e-7_f32 as f64).floor() * y_scale
        } else {
            0.0
        };

        self.sample_and_lerp(floor_x as i32, floor_y as i32, floor_z as i32, d3, d4 - d7, d5, d4)
    }

    #[inline]
    pub fn y_origin(&self) -> f64 {
        self.yo
    }

    #[inline]
    fn sample_and_lerp(
        &self,
        grid_x: i32,
        grid_y: i32,
        grid_z: i32,
        delta_x: f64,
        weird_delta_y: f64,
        delta_z: f64,
        delta_y: f64,
    ) -> f64 {
        let permutation = &self.permutation;
        let i = permutation[(grid_x as usize) & 0xff] as i32;
        let i1 = permutation[((grid_x + 1) as usize) & 0xff] as i32;
        let i2 = permutation[((i + grid_y) as usize) & 0xff] as i32;
        let i3 = permutation[((i + grid_y + 1) as usize) & 0xff] as i32;
        let i4 = permutation[((i1 + grid_y) as usize) & 0xff] as i32;
        let i5 = permutation[((i1 + grid_y + 1) as usize) & 0xff] as i32;
        let o0 = ((permutation[((i2 + grid_z) as usize) & 0xff] & 15) as usize) << 2;
        let o1 = ((permutation[((i4 + grid_z) as usize) & 0xff] & 15) as usize) << 2;
        let o2 = ((permutation[((i3 + grid_z) as usize) & 0xff] & 15) as usize) << 2;
        let o3 = ((permutation[((i5 + grid_z) as usize) & 0xff] & 15) as usize) << 2;
        let o4 = ((permutation[((i2 + grid_z + 1) as usize) & 0xff] & 15) as usize) << 2;
        let o5 = ((permutation[((i4 + grid_z + 1) as usize) & 0xff] & 15) as usize) << 2;
        let o6 = ((permutation[((i3 + grid_z + 1) as usize) & 0xff] & 15) as usize) << 2;
        let o7 = ((permutation[((i5 + grid_z + 1) as usize) & 0xff] & 15) as usize) << 2;
        let d = FLAT_GRADIENT[o0] * delta_x
            + FLAT_GRADIENT[o0 + 1] * weird_delta_y
            + FLAT_GRADIENT[o0 + 2] * delta_z;
        let d1 = FLAT_GRADIENT[o1] * (delta_x - 1.0)
            + FLAT_GRADIENT[o1 + 1] * weird_delta_y
            + FLAT_GRADIENT[o1 + 2] * delta_z;
        let d2 = FLAT_GRADIENT[o2] * delta_x
            + FLAT_GRADIENT[o2 + 1] * (weird_delta_y - 1.0)
            + FLAT_GRADIENT[o2 + 2] * delta_z;
        let d3 = FLAT_GRADIENT[o3] * (delta_x - 1.0)
            + FLAT_GRADIENT[o3 + 1] * (weird_delta_y - 1.0)
            + FLAT_GRADIENT[o3 + 2] * delta_z;
        let d4 = FLAT_GRADIENT[o4] * delta_x
            + FLAT_GRADIENT[o4 + 1] * weird_delta_y
            + FLAT_GRADIENT[o4 + 2] * (delta_z - 1.0);
        let d5 = FLAT_GRADIENT[o5] * (delta_x - 1.0)
            + FLAT_GRADIENT[o5 + 1] * weird_delta_y
            + FLAT_GRADIENT[o5 + 2] * (delta_z - 1.0);
        let d6 = FLAT_GRADIENT[o6] * delta_x
            + FLAT_GRADIENT[o6 + 1] * (weird_delta_y - 1.0)
            + FLAT_GRADIENT[o6 + 2] * (delta_z - 1.0);
        let d7 = FLAT_GRADIENT[o7] * (delta_x - 1.0)
            + FLAT_GRADIENT[o7 + 1] * (weird_delta_y - 1.0)
            + FLAT_GRADIENT[o7 + 2] * (delta_z - 1.0);
        let d8 = smoothstep(delta_x);
        let d9 = smoothstep(delta_y);
        let d10 = smoothstep(delta_z);
        lerp3(d8, d9, d10, d, d1, d2, d3, d4, d5, d6, d7)
    }
}

pub fn noise_batch_summary(
    permutation: &[u8],
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    y_scales: &[f64],
    y_maxes: &[f64],
    xo: f64,
    yo: f64,
    zo: f64,
    iterations: usize,
) -> Result<ImprovedNoiseSummary, ImprovedNoiseError> {
    let input_len = xs.len();
    if ys.len() != input_len
        || zs.len() != input_len
        || y_scales.len() != input_len
        || y_maxes.len() != input_len
        || (input_len == 0 && iterations != 0)
    {
        return Err(ImprovedNoiseError::LengthMismatch);
    }

    let noise = ImprovedNoise::new(permutation, xo, yo, zo)?;
    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    let mut index = 0usize;
    for iteration in 0..iterations {
        let x = xs[index];
        let y = ys[index];
        let z = zs[index];
        let y_scale = y_scales[index];
        let value = if y_scale == 0.0 {
            noise.noise_no_y_scale(x, y, z)
        } else {
            noise.noise(x, y, z, y_scale, y_maxes[index])
        };
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iteration as u64).rotate_left(17)),
        );
        index += 1;
        if index == input_len {
            index = 0;
        }
    }

    Ok(ImprovedNoiseSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    })
}

pub fn fill_positions(
    noise: &ImprovedNoise,
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    y_scales: &[f64],
    y_maxes: &[f64],
    dst: &mut [f64],
) -> Result<(), ImprovedNoiseError> {
    let len = xs.len();
    if ys.len() != len
        || zs.len() != len
        || y_scales.len() != len
        || y_maxes.len() != len
        || dst.len() != len
    {
        return Err(ImprovedNoiseError::LengthMismatch);
    }

    for index in 0..len {
        let x = xs[index];
        let y = ys[index];
        let z = zs[index];
        let y_scale = y_scales[index];
        dst[index] = if y_scale == 0.0 {
            noise.noise_no_y_scale(x, y, z)
        } else {
            noise.noise(x, y, z, y_scale, y_maxes[index])
        };
    }

    Ok(())
}

pub fn fill_positions_no_y_scale(
    noise: &ImprovedNoise,
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    dst: &mut [f64],
) -> Result<(), ImprovedNoiseError> {
    let len = xs.len();
    if ys.len() != len || zs.len() != len || dst.len() != len {
        return Err(ImprovedNoiseError::LengthMismatch);
    }

    for index in 0..len {
        let x = xs[index];
        let y = ys[index];
        let z = zs[index];
        dst[index] = noise.noise_no_y_scale(x, y, z);
    }

    Ok(())
}

#[inline]
fn floor(value: f64) -> i32 {
    let i = value as i32;
    if value < i as f64 { i - 1 } else { i }
}

#[inline]
fn smoothstep(value: f64) -> f64 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

#[inline]
fn lerp2(
    delta_x: f64,
    delta_y: f64,
    value00: f64,
    value10: f64,
    value01: f64,
    value11: f64,
) -> f64 {
    lerp(
        delta_y,
        lerp(delta_x, value00, value10),
        lerp(delta_x, value01, value11),
    )
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn lerp3(
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    value000: f64,
    value100: f64,
    value010: f64,
    value110: f64,
    value001: f64,
    value101: f64,
    value011: f64,
    value111: f64,
) -> f64 {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, value000, value100, value010, value110),
        lerp2(delta_x, delta_y, value001, value101, value011, value111),
    )
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
        let permutation = permutation();
        assert_eq!(
            noise_batch_summary(&permutation[..255], &[], &[], &[], &[], &[], 0.0, 0.0, 0.0, 0),
            Err(ImprovedNoiseError::InvalidPermutationLength)
        );
        assert_eq!(
            noise_batch_summary(&permutation, &[1.0], &[], &[1.0], &[0.0], &[0.0], 0.0, 0.0, 0.0, 1),
            Err(ImprovedNoiseError::LengthMismatch)
        );
        assert_eq!(
            noise_batch_summary(&permutation, &[], &[], &[], &[], &[], 0.0, 0.0, 0.0, 1),
            Err(ImprovedNoiseError::LengthMismatch)
        );
    }

    #[test]
    fn deterministic_batch_summary_is_stable() {
        let permutation = permutation();
        let xs = [-12.75, -1.5, 0.0, 19.25, 1024.03125];
        let ys = [8.5, -4.25, 0.125, 31.75, -127.5];
        let zs = [3.25, 7.5, -64.125, 0.5, 255.875];
        let y_scales = [0.0, 0.25, 0.0, 8.0, 0.125];
        let y_maxes = [0.0, 0.5, 0.0, 4.0, -1.0];

        let first = noise_batch_summary(
            &permutation,
            &xs,
            &ys,
            &zs,
            &y_scales,
            &y_maxes,
            17.375,
            201.625,
            93.125,
            4096,
        )
        .unwrap();
        let second = noise_batch_summary(
            &permutation,
            &xs,
            &ys,
            &zs,
            &y_scales,
            &y_maxes,
            17.375,
            201.625,
            93.125,
            4096,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.count, 4096);
        assert_ne!(first.value_checksum, 0);
    }

    #[test]
    fn y_scale_changes_the_sample_path() {
        let permutation = permutation();
        let noise = ImprovedNoise::new(&permutation, 17.375, 201.625, 93.125).unwrap();
        let unscaled = noise.noise(13.03125, -2.75, 41.5, 0.0, 0.0);
        let scaled = noise.noise(13.03125, -2.75, 41.5, 0.25, 0.5);

        assert_ne!(unscaled.to_bits(), scaled.to_bits());
    }

    #[test]
    fn no_y_scale_path_matches_zero_scale_noise() {
        let permutation = permutation();
        let noise = ImprovedNoise::new(&permutation, 17.375, 201.625, 93.125).unwrap();

        for &(x, y, z) in &[
            (13.03125, -2.75, 41.5),
            (-1024.5, 0.125, 2048.25),
            (0.0, 63.999, -0.03125),
        ] {
            assert_eq!(
                noise.noise_no_y_scale(x, y, z).to_bits(),
                noise.noise(x, y, z, 0.0, 0.0).to_bits()
            );
        }
    }

    #[test]
    fn fill_positions_zero_y_scale_ignores_y_max() {
        let permutation = permutation();
        let noise = ImprovedNoise::new(&permutation, 17.375, 201.625, 93.125).unwrap();
        let xs = [-12.75, -1.5, 0.0, 19.25, 1024.03125];
        let ys = [8.5, -4.25, 0.125, 31.75, -127.5];
        let zs = [3.25, 7.5, -64.125, 0.5, 255.875];
        let y_scales = [0.0; 5];
        let y_maxes = [0.5, -0.25, 1.0, 4.0, -1.0];
        let mut output = [0.0; 5];

        fill_positions(&noise, &xs, &ys, &zs, &y_scales, &y_maxes, &mut output).unwrap();

        for index in 0..output.len() {
            let expected = noise.noise_no_y_scale(xs[index], ys[index], zs[index]);
            assert_eq!(output[index].to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn fill_positions_matches_scalar_noise() {
        let permutation = permutation();
        let noise = ImprovedNoise::new(&permutation, 17.375, 201.625, 93.125).unwrap();
        let xs = [-12.75, -1.5, 0.0, 19.25, 1024.03125];
        let ys = [8.5, -4.25, 0.125, 31.75, -127.5];
        let zs = [3.25, 7.5, -64.125, 0.5, 255.875];
        let y_scales = [0.0, 0.25, 0.0, 8.0, 0.125];
        let y_maxes = [0.0, 0.5, 0.0, 4.0, -1.0];
        let mut output = [0.0; 5];

        fill_positions(&noise, &xs, &ys, &zs, &y_scales, &y_maxes, &mut output).unwrap();

        for index in 0..output.len() {
            let expected = noise.noise(xs[index], ys[index], zs[index], y_scales[index], y_maxes[index]);
            assert_eq!(output[index].to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn fill_positions_no_y_scale_matches_scalar_noise() {
        let permutation = permutation();
        let noise = ImprovedNoise::new(&permutation, 17.375, 201.625, 93.125).unwrap();
        let xs = [-12.75, -1.5, 0.0, 19.25, 1024.03125];
        let ys = [8.5, -4.25, 0.125, 31.75, -127.5];
        let zs = [3.25, 7.5, -64.125, 0.5, 255.875];
        let mut output = [0.0; 5];

        fill_positions_no_y_scale(&noise, &xs, &ys, &zs, &mut output).unwrap();

        for index in 0..output.len() {
            let expected = noise.noise_no_y_scale(xs[index], ys[index], zs[index]);
            assert_eq!(output[index].to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn fill_positions_rejects_length_mismatch() {
        let permutation = permutation();
        let noise = ImprovedNoise::new(&permutation, 17.375, 201.625, 93.125).unwrap();
        let mut output = [0.0; 2];

        assert_eq!(
            fill_positions(
                &noise,
                &[1.0, 2.0],
                &[3.0],
                &[4.0, 5.0],
                &[0.0, 0.0],
                &[0.0, 0.0],
                &mut output
            ),
            Err(ImprovedNoiseError::LengthMismatch)
        );
        assert_eq!(
            fill_positions_no_y_scale(&noise, &[1.0, 2.0], &[3.0], &[4.0, 5.0], &mut output),
            Err(ImprovedNoiseError::LengthMismatch)
        );
    }

    fn permutation() -> [u8; PERMUTATION_LENGTH] {
        let mut permutation = [0u8; PERMUTATION_LENGTH];
        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }

        let mut state = 0x5eed_1234u32;
        for i in 0..PERMUTATION_LENGTH {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let random_int = state % ((PERMUTATION_LENGTH - i) as u32);
            permutation.swap(i, i + random_int as usize);
        }
        permutation
    }
}
