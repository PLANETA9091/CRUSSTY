pub const PERMUTATION_LENGTH: usize = 256;
pub const SUMMARY_FIELDS: usize = 4;

const GRADIENT: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

const FLAT_GRADIENT: [i32; 64] = [
    1, 1, 0, 0,
    -1, 1, 0, 0,
    1, -1, 0, 0,
    -1, -1, 0, 0,
    1, 0, 1, 0,
    -1, 0, 1, 0,
    1, 0, -1, 0,
    -1, 0, -1, 0,
    0, 1, 1, 0,
    0, -1, 1, 0,
    0, 1, -1, 0,
    0, -1, -1, 0,
    1, 1, 0, 0,
    0, -1, 1, 0,
    -1, 1, 0, 0,
    0, -1, -1, 0,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImprovedNoiseDerivativeError {
    InvalidPermutationLength,
    LengthMismatch,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImprovedNoiseDerivativeSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImprovedNoiseDerivativeKind {
    Old,
    Inline,
    IntTable,
    FlatGradient,
}

pub fn derivative_summary(
    permutation: &[u8],
    grid_x: &[i32],
    grid_y: &[i32],
    grid_z: &[i32],
    delta_x: &[f64],
    delta_y: &[f64],
    delta_z: &[f64],
    iterations: usize,
    kind: ImprovedNoiseDerivativeKind,
) -> Result<ImprovedNoiseDerivativeSummary, ImprovedNoiseDerivativeError> {
    if permutation.len() != PERMUTATION_LENGTH {
        return Err(ImprovedNoiseDerivativeError::InvalidPermutationLength);
    }
    let len = grid_x.len();
    if grid_y.len() != len
        || grid_z.len() != len
        || delta_x.len() != len
        || delta_y.len() != len
        || delta_z.len() != len
        || (len == 0 && iterations != 0)
    {
        return Err(ImprovedNoiseDerivativeError::LengthMismatch);
    }

    let noise = Noise::new(permutation);
    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        let index = iteration % len;
        let mut values = [0.0f64; 3];
        let result = noise.sample_with_derivative(
            grid_x[index],
            grid_y[index],
            grid_z[index],
            delta_x[index],
            delta_y[index],
            delta_z[index],
            &mut values,
            kind,
        );
        let combined = result + values[0] + values[1] + values[2];
        sum += combined;
        last_bits = result.to_bits()
            ^ values[0].to_bits().rotate_left(7)
            ^ values[1].to_bits().rotate_left(17)
            ^ values[2].to_bits().rotate_left(29);
        checksum = mix64(
            checksum
                ^ last_bits
                ^ combined.to_bits().rotate_left(11)
                ^ ((iteration as u64).wrapping_mul(0xD1B5_4A32_D192_ED03)),
        );
    }

    Ok(ImprovedNoiseDerivativeSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    })
}

#[derive(Clone, Debug)]
struct Noise {
    permutation: [u8; PERMUTATION_LENGTH],
    int_permutation: [i32; PERMUTATION_LENGTH],
}

impl Noise {
    fn new(permutation: &[u8]) -> Self {
        let mut copy = [0u8; PERMUTATION_LENGTH];
        let mut int_copy = [0i32; PERMUTATION_LENGTH];
        copy.copy_from_slice(permutation);
        for (index, value) in permutation.iter().enumerate() {
            int_copy[index] = *value as i32;
        }
        Self {
            permutation: copy,
            int_permutation: int_copy,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_with_derivative(
        &self,
        grid_x: i32,
        grid_y: i32,
        grid_z: i32,
        delta_x: f64,
        delta_y: f64,
        delta_z: f64,
        noise_values: &mut [f64; 3],
        kind: ImprovedNoiseDerivativeKind,
    ) -> f64 {
        match kind {
            ImprovedNoiseDerivativeKind::Old => {
                let i = self.p(grid_x);
                let i1 = self.p(grid_x + 1);
                let i2 = self.p(i + grid_y);
                let i3 = self.p(i + grid_y + 1);
                let i4 = self.p(i1 + grid_y);
                let i5 = self.p(i1 + grid_y + 1);
                let i6 = self.p(i2 + grid_z);
                let i7 = self.p(i4 + grid_z);
                let i8 = self.p(i3 + grid_z);
                let i9 = self.p(i5 + grid_z);
                let i10 = self.p(i2 + grid_z + 1);
                let i11 = self.p(i4 + grid_z + 1);
                let i12 = self.p(i3 + grid_z + 1);
                let i13 = self.p(i5 + grid_z + 1);
                sample(i6, i7, i8, i9, i10, i11, i12, i13, delta_x, delta_y, delta_z, noise_values)
            }
            ImprovedNoiseDerivativeKind::Inline => {
                let permutation = &self.permutation;
                let i = permutation[(grid_x as usize) & 0xff] as i32;
                let i1 = permutation[((grid_x + 1) as usize) & 0xff] as i32;
                let i2 = permutation[((i + grid_y) as usize) & 0xff] as i32;
                let i3 = permutation[((i + grid_y + 1) as usize) & 0xff] as i32;
                let i4 = permutation[((i1 + grid_y) as usize) & 0xff] as i32;
                let i5 = permutation[((i1 + grid_y + 1) as usize) & 0xff] as i32;
                let i6 = permutation[((i2 + grid_z) as usize) & 0xff] as i32;
                let i7 = permutation[((i4 + grid_z) as usize) & 0xff] as i32;
                let i8 = permutation[((i3 + grid_z) as usize) & 0xff] as i32;
                let i9 = permutation[((i5 + grid_z) as usize) & 0xff] as i32;
                let i10 = permutation[((i2 + grid_z + 1) as usize) & 0xff] as i32;
                let i11 = permutation[((i4 + grid_z + 1) as usize) & 0xff] as i32;
                let i12 = permutation[((i3 + grid_z + 1) as usize) & 0xff] as i32;
                let i13 = permutation[((i5 + grid_z + 1) as usize) & 0xff] as i32;
                sample(i6, i7, i8, i9, i10, i11, i12, i13, delta_x, delta_y, delta_z, noise_values)
            }
            ImprovedNoiseDerivativeKind::IntTable => {
                let permutation = &self.int_permutation;
                let i = permutation[(grid_x as usize) & 0xff];
                let i1 = permutation[((grid_x + 1) as usize) & 0xff];
                let i2 = permutation[((i + grid_y) as usize) & 0xff];
                let i3 = permutation[((i + grid_y + 1) as usize) & 0xff];
                let i4 = permutation[((i1 + grid_y) as usize) & 0xff];
                let i5 = permutation[((i1 + grid_y + 1) as usize) & 0xff];
                let i6 = permutation[((i2 + grid_z) as usize) & 0xff];
                let i7 = permutation[((i4 + grid_z) as usize) & 0xff];
                let i8 = permutation[((i3 + grid_z) as usize) & 0xff];
                let i9 = permutation[((i5 + grid_z) as usize) & 0xff];
                let i10 = permutation[((i2 + grid_z + 1) as usize) & 0xff];
                let i11 = permutation[((i4 + grid_z + 1) as usize) & 0xff];
                let i12 = permutation[((i3 + grid_z + 1) as usize) & 0xff];
                let i13 = permutation[((i5 + grid_z + 1) as usize) & 0xff];
                sample(i6, i7, i8, i9, i10, i11, i12, i13, delta_x, delta_y, delta_z, noise_values)
            }
            ImprovedNoiseDerivativeKind::FlatGradient => {
                let i = self.p(grid_x);
                let i1 = self.p(grid_x + 1);
                let i2 = self.p(i + grid_y);
                let i3 = self.p(i + grid_y + 1);
                let i4 = self.p(i1 + grid_y);
                let i5 = self.p(i1 + grid_y + 1);
                let i6 = self.p(i2 + grid_z);
                let i7 = self.p(i4 + grid_z);
                let i8 = self.p(i3 + grid_z);
                let i9 = self.p(i5 + grid_z);
                let i10 = self.p(i2 + grid_z + 1);
                let i11 = self.p(i4 + grid_z + 1);
                let i12 = self.p(i3 + grid_z + 1);
                let i13 = self.p(i5 + grid_z + 1);
                sample_flat(
                    i6, i7, i8, i9, i10, i11, i12, i13, delta_x, delta_y, delta_z, noise_values,
                )
            }
        }
    }

    #[inline]
    fn p(&self, index: i32) -> i32 {
        self.permutation[(index as usize) & 0xff] as i32
    }
}

#[allow(clippy::too_many_arguments)]
fn sample(
    i6: i32,
    i7: i32,
    i8: i32,
    i9: i32,
    i10: i32,
    i11: i32,
    i12: i32,
    i13: i32,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    noise_values: &mut [f64; 3],
) -> f64 {
    let ints = GRADIENT[(i6 & 15) as usize];
    let ints1 = GRADIENT[(i7 & 15) as usize];
    let ints2 = GRADIENT[(i8 & 15) as usize];
    let ints3 = GRADIENT[(i9 & 15) as usize];
    let ints4 = GRADIENT[(i10 & 15) as usize];
    let ints5 = GRADIENT[(i11 & 15) as usize];
    let ints6 = GRADIENT[(i12 & 15) as usize];
    let ints7 = GRADIENT[(i13 & 15) as usize];
    let d = dot(ints, delta_x, delta_y, delta_z);
    let d1 = dot(ints1, delta_x - 1.0, delta_y, delta_z);
    let d2 = dot(ints2, delta_x, delta_y - 1.0, delta_z);
    let d3 = dot(ints3, delta_x - 1.0, delta_y - 1.0, delta_z);
    let d4 = dot(ints4, delta_x, delta_y, delta_z - 1.0);
    let d5 = dot(ints5, delta_x - 1.0, delta_y, delta_z - 1.0);
    let d6 = dot(ints6, delta_x, delta_y - 1.0, delta_z - 1.0);
    let d7 = dot(ints7, delta_x - 1.0, delta_y - 1.0, delta_z - 1.0);
    finish_sample(
        ints[0], ints[1], ints[2],
        ints1[0], ints1[1], ints1[2],
        ints2[0], ints2[1], ints2[2],
        ints3[0], ints3[1], ints3[2],
        ints4[0], ints4[1], ints4[2],
        ints5[0], ints5[1], ints5[2],
        ints6[0], ints6[1], ints6[2],
        ints7[0], ints7[1], ints7[2],
        d, d1, d2, d3, d4, d5, d6, d7,
        delta_x, delta_y, delta_z, noise_values,
    )
}

#[allow(clippy::too_many_arguments)]
fn sample_flat(
    i6: i32,
    i7: i32,
    i8: i32,
    i9: i32,
    i10: i32,
    i11: i32,
    i12: i32,
    i13: i32,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    noise_values: &mut [f64; 3],
) -> f64 {
    let o = ((i6 & 15) as usize) << 2;
    let o1 = ((i7 & 15) as usize) << 2;
    let o2 = ((i8 & 15) as usize) << 2;
    let o3 = ((i9 & 15) as usize) << 2;
    let o4 = ((i10 & 15) as usize) << 2;
    let o5 = ((i11 & 15) as usize) << 2;
    let o6 = ((i12 & 15) as usize) << 2;
    let o7 = ((i13 & 15) as usize) << 2;
    let gx = FLAT_GRADIENT[o];
    let gy = FLAT_GRADIENT[o + 1];
    let gz = FLAT_GRADIENT[o + 2];
    let gx1 = FLAT_GRADIENT[o1];
    let gy1 = FLAT_GRADIENT[o1 + 1];
    let gz1 = FLAT_GRADIENT[o1 + 2];
    let gx2 = FLAT_GRADIENT[o2];
    let gy2 = FLAT_GRADIENT[o2 + 1];
    let gz2 = FLAT_GRADIENT[o2 + 2];
    let gx3 = FLAT_GRADIENT[o3];
    let gy3 = FLAT_GRADIENT[o3 + 1];
    let gz3 = FLAT_GRADIENT[o3 + 2];
    let gx4 = FLAT_GRADIENT[o4];
    let gy4 = FLAT_GRADIENT[o4 + 1];
    let gz4 = FLAT_GRADIENT[o4 + 2];
    let gx5 = FLAT_GRADIENT[o5];
    let gy5 = FLAT_GRADIENT[o5 + 1];
    let gz5 = FLAT_GRADIENT[o5 + 2];
    let gx6 = FLAT_GRADIENT[o6];
    let gy6 = FLAT_GRADIENT[o6 + 1];
    let gz6 = FLAT_GRADIENT[o6 + 2];
    let gx7 = FLAT_GRADIENT[o7];
    let gy7 = FLAT_GRADIENT[o7 + 1];
    let gz7 = FLAT_GRADIENT[o7 + 2];
    let d = gx as f64 * delta_x + gy as f64 * delta_y + gz as f64 * delta_z;
    let d1 = gx1 as f64 * (delta_x - 1.0) + gy1 as f64 * delta_y + gz1 as f64 * delta_z;
    let d2 = gx2 as f64 * delta_x + gy2 as f64 * (delta_y - 1.0) + gz2 as f64 * delta_z;
    let d3 = gx3 as f64 * (delta_x - 1.0) + gy3 as f64 * (delta_y - 1.0) + gz3 as f64 * delta_z;
    let d4 = gx4 as f64 * delta_x + gy4 as f64 * delta_y + gz4 as f64 * (delta_z - 1.0);
    let d5 = gx5 as f64 * (delta_x - 1.0) + gy5 as f64 * delta_y + gz5 as f64 * (delta_z - 1.0);
    let d6 = gx6 as f64 * delta_x + gy6 as f64 * (delta_y - 1.0) + gz6 as f64 * (delta_z - 1.0);
    let d7 = gx7 as f64 * (delta_x - 1.0) + gy7 as f64 * (delta_y - 1.0) + gz7 as f64 * (delta_z - 1.0);
    finish_sample(
        gx, gy, gz, gx1, gy1, gz1, gx2, gy2, gz2, gx3, gy3, gz3, gx4, gy4, gz4, gx5, gy5,
        gz5, gx6, gy6, gz6, gx7, gy7, gz7, d, d1, d2, d3, d4, d5, d6, d7, delta_x,
        delta_y, delta_z, noise_values,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_sample(
    gx: i32,
    gy: i32,
    gz: i32,
    gx1: i32,
    gy1: i32,
    gz1: i32,
    gx2: i32,
    gy2: i32,
    gz2: i32,
    gx3: i32,
    gy3: i32,
    gz3: i32,
    gx4: i32,
    gy4: i32,
    gz4: i32,
    gx5: i32,
    gy5: i32,
    gz5: i32,
    gx6: i32,
    gy6: i32,
    gz6: i32,
    gx7: i32,
    gy7: i32,
    gz7: i32,
    d: f64,
    d1: f64,
    d2: f64,
    d3: f64,
    d4: f64,
    d5: f64,
    d6: f64,
    d7: f64,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    noise_values: &mut [f64; 3],
) -> f64 {
    let d8 = smoothstep(delta_x);
    let d9 = smoothstep(delta_y);
    let d10 = smoothstep(delta_z);
    let d11 = lerp3(
        d8, d9, d10, gx as f64, gx1 as f64, gx2 as f64, gx3 as f64, gx4 as f64,
        gx5 as f64, gx6 as f64, gx7 as f64,
    );
    let d12 = lerp3(
        d8, d9, d10, gy as f64, gy1 as f64, gy2 as f64, gy3 as f64, gy4 as f64,
        gy5 as f64, gy6 as f64, gy7 as f64,
    );
    let d13 = lerp3(
        d8, d9, d10, gz as f64, gz1 as f64, gz2 as f64, gz3 as f64, gz4 as f64,
        gz5 as f64, gz6 as f64, gz7 as f64,
    );
    let d14 = lerp2(d9, d10, d1 - d, d3 - d2, d5 - d4, d7 - d6);
    let d15 = lerp2(d10, d8, d2 - d, d6 - d4, d3 - d1, d7 - d5);
    let d16 = lerp2(d8, d9, d4 - d, d5 - d1, d6 - d2, d7 - d3);
    let d17 = smoothstep_derivative(delta_x);
    let d18 = smoothstep_derivative(delta_y);
    let d19 = smoothstep_derivative(delta_z);
    let d20 = d11 + d17 * d14;
    let d21 = d12 + d18 * d15;
    let d22 = d13 + d19 * d16;
    noise_values[0] += d20;
    noise_values[1] += d21;
    noise_values[2] += d22;
    lerp3(d8, d9, d10, d, d1, d2, d3, d4, d5, d6, d7)
}

#[inline]
fn dot(gradient: [i32; 3], x_factor: f64, y_factor: f64, z_factor: f64) -> f64 {
    gradient[0] as f64 * x_factor + gradient[1] as f64 * y_factor + gradient[2] as f64 * z_factor
}

#[inline]
fn smoothstep(value: f64) -> f64 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

#[inline]
fn smoothstep_derivative(value: f64) -> f64 {
    30.0 * value * value * (value - 1.0) * (value - 1.0)
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
    fn variants_match_old_path() {
        let permutation = permutation();
        let grid_x = [-2, -1, 0, 1, 128, 4097, -16384];
        let grid_y = [-9, 0, 3, 15, 64, 127, -384];
        let grid_z = [-11, 7, 19, 255, -2048, 12000, 42];
        let delta_x = [0.0, 0.125, 0.25, 0.5, 0.75, 0.9375, 0.999];
        let delta_y = [0.0, 0.2, 0.4, 0.6, 0.8, 0.875, 0.99];
        let delta_z = [0.0, 0.03125, 0.333, 0.5, 0.667, 0.875, 0.999];
        let old = derivative_summary(
            &permutation,
            &grid_x,
            &grid_y,
            &grid_z,
            &delta_x,
            &delta_y,
            &delta_z,
            4096,
            ImprovedNoiseDerivativeKind::Old,
        )
        .unwrap();
        for kind in [
            ImprovedNoiseDerivativeKind::Inline,
            ImprovedNoiseDerivativeKind::IntTable,
            ImprovedNoiseDerivativeKind::FlatGradient,
        ] {
            assert_eq!(
                old,
                derivative_summary(
                    &permutation,
                    &grid_x,
                    &grid_y,
                    &grid_z,
                    &delta_x,
                    &delta_y,
                    &delta_z,
                    4096,
                    kind,
                )
                .unwrap()
            );
        }
    }

    fn permutation() -> [u8; PERMUTATION_LENGTH] {
        let mut permutation = [0u8; PERMUTATION_LENGTH];
        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }
        let mut state = 0x6d2b79f5u32;
        for index in 0..PERMUTATION_LENGTH {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let random_int = (state as usize) % (PERMUTATION_LENGTH - index);
            permutation.swap(index, index + random_int);
        }
        permutation
    }
}
