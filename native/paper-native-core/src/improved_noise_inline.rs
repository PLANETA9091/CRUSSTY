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
pub enum ImprovedNoiseInlineError {
    InvalidPermutationLength,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImprovedNoiseInlineSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImprovedNoiseInlineKind {
    OldPMethod,
    InlineByteAccess,
    FlatGradient,
    Arithmetic,
    SwitchGradient,
}

pub fn loop_summary(
    permutation: &[u8],
    iterations: usize,
    kind: ImprovedNoiseInlineKind,
) -> Result<ImprovedNoiseInlineSummary, ImprovedNoiseInlineError> {
    let noise = Noise::new(permutation)?;
    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        let x = (((iteration as f64) * 0.03125) % 2048.0) - 1024.0;
        let y = (((iteration as f64) * 0.015625) % 512.0) - 256.0;
        let z = (((iteration as f64) * 0.046875) % 2048.0) - 1024.0;
        let value = noise.noise(x, y, z, 0.0, 0.0, kind);
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((iteration as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(11)),
        );
    }

    Ok(ImprovedNoiseInlineSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        value_checksum: checksum,
        last_bits,
    })
}

#[derive(Clone, Debug)]
struct Noise {
    permutation: [u8; PERMUTATION_LENGTH],
}

impl Noise {
    fn new(permutation: &[u8]) -> Result<Self, ImprovedNoiseInlineError> {
        if permutation.len() != PERMUTATION_LENGTH {
            return Err(ImprovedNoiseInlineError::InvalidPermutationLength);
        }
        let mut copy = [0u8; PERMUTATION_LENGTH];
        copy.copy_from_slice(permutation);
        Ok(Self { permutation: copy })
    }

    fn noise(
        &self,
        x: f64,
        y: f64,
        z: f64,
        y_scale: f64,
        y_max: f64,
        kind: ImprovedNoiseInlineKind,
    ) -> f64 {
        let floor_x = floor(x);
        let floor_y = floor(y);
        let floor_z = floor(z);
        let delta_x = x - floor_x as f64;
        let delta_y = y - floor_y as f64;
        let delta_z = z - floor_z as f64;
        let y_offset = if y_scale != 0.0 {
            let limited_y = if y_max >= 0.0 && y_max < delta_y {
                y_max
            } else {
                delta_y
            };
            floor(limited_y / y_scale + 1.0e-7_f32 as f64) as f64 * y_scale
        } else {
            0.0
        };

        match kind {
            ImprovedNoiseInlineKind::OldPMethod => self.sample_old(
                floor_x,
                floor_y,
                floor_z,
                delta_x,
                delta_y - y_offset,
                delta_z,
                delta_y,
            ),
            ImprovedNoiseInlineKind::InlineByteAccess => self.sample_inline(
                floor_x,
                floor_y,
                floor_z,
                delta_x,
                delta_y - y_offset,
                delta_z,
                delta_y,
                grad_dot,
            ),
            ImprovedNoiseInlineKind::FlatGradient => self.sample_inline(
                floor_x,
                floor_y,
                floor_z,
                delta_x,
                delta_y - y_offset,
                delta_z,
                delta_y,
                flat_grad_dot,
            ),
            ImprovedNoiseInlineKind::Arithmetic => self.sample_arithmetic(
                floor_x,
                floor_y,
                floor_z,
                delta_x,
                delta_y - y_offset,
                delta_z,
                delta_y,
            ),
            ImprovedNoiseInlineKind::SwitchGradient => self.sample_inline(
                floor_x,
                floor_y,
                floor_z,
                delta_x,
                delta_y - y_offset,
                delta_z,
                delta_y,
                switch_grad_dot,
            ),
        }
    }

    fn sample_old(
        &self,
        grid_x: i32,
        grid_y: i32,
        grid_z: i32,
        delta_x: f64,
        weird_delta_y: f64,
        delta_z: f64,
        delta_y: f64,
    ) -> f64 {
        let i = self.p(grid_x);
        let i1 = self.p(grid_x + 1);
        let i2 = self.p(i + grid_y);
        let i3 = self.p(i + grid_y + 1);
        let i4 = self.p(i1 + grid_y);
        let i5 = self.p(i1 + grid_y + 1);
        let d = grad_dot(self.p(i2 + grid_z), delta_x, weird_delta_y, delta_z);
        let d1 = grad_dot(self.p(i4 + grid_z), delta_x - 1.0, weird_delta_y, delta_z);
        let d2 = grad_dot(self.p(i3 + grid_z), delta_x, weird_delta_y - 1.0, delta_z);
        let d3 = grad_dot(
            self.p(i5 + grid_z),
            delta_x - 1.0,
            weird_delta_y - 1.0,
            delta_z,
        );
        let d4 = grad_dot(self.p(i2 + grid_z + 1), delta_x, weird_delta_y, delta_z - 1.0);
        let d5 = grad_dot(
            self.p(i4 + grid_z + 1),
            delta_x - 1.0,
            weird_delta_y,
            delta_z - 1.0,
        );
        let d6 = grad_dot(
            self.p(i3 + grid_z + 1),
            delta_x,
            weird_delta_y - 1.0,
            delta_z - 1.0,
        );
        let d7 = grad_dot(
            self.p(i5 + grid_z + 1),
            delta_x - 1.0,
            weird_delta_y - 1.0,
            delta_z - 1.0,
        );
        let d8 = smoothstep(delta_x);
        let d9 = smoothstep(delta_y);
        let d10 = smoothstep(delta_z);
        lerp3(d8, d9, d10, d, d1, d2, d3, d4, d5, d6, d7)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_inline(
        &self,
        grid_x: i32,
        grid_y: i32,
        grid_z: i32,
        delta_x: f64,
        weird_delta_y: f64,
        delta_z: f64,
        delta_y: f64,
        dot: fn(i32, f64, f64, f64) -> f64,
    ) -> f64 {
        let permutation = &self.permutation;
        let i = permutation[(grid_x as usize) & 0xff] as i32;
        let i1 = permutation[((grid_x + 1) as usize) & 0xff] as i32;
        let i2 = permutation[((i + grid_y) as usize) & 0xff] as i32;
        let i3 = permutation[((i + grid_y + 1) as usize) & 0xff] as i32;
        let i4 = permutation[((i1 + grid_y) as usize) & 0xff] as i32;
        let i5 = permutation[((i1 + grid_y + 1) as usize) & 0xff] as i32;
        let d = dot(
            permutation[((i2 + grid_z) as usize) & 0xff] as i32,
            delta_x,
            weird_delta_y,
            delta_z,
        );
        let d1 = dot(
            permutation[((i4 + grid_z) as usize) & 0xff] as i32,
            delta_x - 1.0,
            weird_delta_y,
            delta_z,
        );
        let d2 = dot(
            permutation[((i3 + grid_z) as usize) & 0xff] as i32,
            delta_x,
            weird_delta_y - 1.0,
            delta_z,
        );
        let d3 = dot(
            permutation[((i5 + grid_z) as usize) & 0xff] as i32,
            delta_x - 1.0,
            weird_delta_y - 1.0,
            delta_z,
        );
        let d4 = dot(
            permutation[((i2 + grid_z + 1) as usize) & 0xff] as i32,
            delta_x,
            weird_delta_y,
            delta_z - 1.0,
        );
        let d5 = dot(
            permutation[((i4 + grid_z + 1) as usize) & 0xff] as i32,
            delta_x - 1.0,
            weird_delta_y,
            delta_z - 1.0,
        );
        let d6 = dot(
            permutation[((i3 + grid_z + 1) as usize) & 0xff] as i32,
            delta_x,
            weird_delta_y - 1.0,
            delta_z - 1.0,
        );
        let d7 = dot(
            permutation[((i5 + grid_z + 1) as usize) & 0xff] as i32,
            delta_x - 1.0,
            weird_delta_y - 1.0,
            delta_z - 1.0,
        );
        let d8 = smoothstep(delta_x);
        let d9 = smoothstep(delta_y);
        let d10 = smoothstep(delta_z);
        lerp3(d8, d9, d10, d, d1, d2, d3, d4, d5, d6, d7)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_arithmetic(
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
        let dx = delta_x * delta_x * delta_x * (delta_x * (delta_x * 6.0 - 15.0) + 10.0);
        let dy = delta_y * delta_y * delta_y * (delta_y * (delta_y * 6.0 - 15.0) + 10.0);
        let dz = delta_z * delta_z * delta_z * (delta_z * (delta_z * 6.0 - 15.0) + 10.0);
        let x00 = d + dx * (d1 - d);
        let x01 = d2 + dx * (d3 - d2);
        let x10 = d4 + dx * (d5 - d4);
        let x11 = d6 + dx * (d7 - d6);
        let y0 = x00 + dy * (x01 - x00);
        let y1 = x10 + dy * (x11 - x10);
        y0 + dz * (y1 - y0)
    }

    #[inline]
    fn p(&self, index: i32) -> i32 {
        self.permutation[(index as usize) & 0xff] as i32
    }
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
fn grad_dot(grad_index: i32, x_factor: f64, y_factor: f64, z_factor: f64) -> f64 {
    let gradient = GRADIENT[(grad_index & 15) as usize];
    gradient[0] as f64 * x_factor + gradient[1] as f64 * y_factor + gradient[2] as f64 * z_factor
}

#[inline]
fn flat_grad_dot(grad_index: i32, x_factor: f64, y_factor: f64, z_factor: f64) -> f64 {
    let offset = ((grad_index & 15) as usize) << 2;
    FLAT_GRADIENT[offset] * x_factor
        + FLAT_GRADIENT[offset + 1] * y_factor
        + FLAT_GRADIENT[offset + 2] * z_factor
}

#[inline]
fn switch_grad_dot(grad_index: i32, x_factor: f64, y_factor: f64, z_factor: f64) -> f64 {
    match grad_index & 15 {
        0 | 12 => x_factor + y_factor + 0.0 * z_factor,
        1 => -x_factor + y_factor + 0.0 * z_factor,
        2 => x_factor - y_factor + 0.0 * z_factor,
        3 => -x_factor - y_factor + 0.0 * z_factor,
        4 => x_factor + 0.0 * y_factor + z_factor,
        5 => -x_factor + 0.0 * y_factor + z_factor,
        6 => x_factor + 0.0 * y_factor - z_factor,
        7 => -x_factor + 0.0 * y_factor - z_factor,
        8 => 0.0 * x_factor + y_factor + z_factor,
        9 | 13 => 0.0 * x_factor - y_factor + z_factor,
        10 => 0.0 * x_factor + y_factor - z_factor,
        11 | 15 => 0.0 * x_factor - y_factor - z_factor,
        14 => -x_factor + y_factor + 0.0 * z_factor,
        _ => unreachable!(),
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
    fn variants_match_old_path() {
        let permutation = permutation();
        let old = loop_summary(&permutation, 16_384, ImprovedNoiseInlineKind::OldPMethod).unwrap();
        assert_eq!(
            old,
            loop_summary(&permutation, 16_384, ImprovedNoiseInlineKind::InlineByteAccess).unwrap()
        );
        assert_eq!(
            old,
            loop_summary(&permutation, 16_384, ImprovedNoiseInlineKind::FlatGradient).unwrap()
        );
        assert_eq!(
            old,
            loop_summary(&permutation, 16_384, ImprovedNoiseInlineKind::Arithmetic).unwrap()
        );
        assert_eq!(
            old,
            loop_summary(&permutation, 16_384, ImprovedNoiseInlineKind::SwitchGradient).unwrap()
        );
    }

    #[test]
    fn rejects_bad_permutation() {
        assert_eq!(
            loop_summary(&[0u8; 255], 1, ImprovedNoiseInlineKind::OldPMethod),
            Err(ImprovedNoiseInlineError::InvalidPermutationLength)
        );
    }

    fn permutation() -> [u8; PERMUTATION_LENGTH] {
        let mut permutation = [0u8; PERMUTATION_LENGTH];
        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }
        let mut state = 0x5eed1234u32;
        for index in 0..PERMUTATION_LENGTH {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            let random_int = (state as usize) % (PERMUTATION_LENGTH - index);
            permutation.swap(index, index + random_int);
        }
        permutation
    }
}
