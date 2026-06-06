pub const SUMMARY_FIELDS: usize = 4;

const CHUNK_OFFSETS: [[i32; 2]; 13] = [
    [0, 0],
    [-2, -1],
    [-1, -1],
    [0, -1],
    [1, -1],
    [-3, 0],
    [-2, 0],
    [-1, 0],
    [1, 0],
    [-2, 1],
    [-1, 1],
    [0, 1],
    [1, 1],
];

const BLOCK_OFFSET_X: [i32; 13] = [0, -32, -16, 0, 16, -48, -32, -16, 16, -32, -16, 0, 16];
const BLOCK_OFFSET_Z: [i32; 13] = [0, -16, -16, -16, -16, 0, 0, 0, 0, 16, 16, 16, 16];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AquiferSurfaceSamplingSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Shape {
    Old,
    New,
}

pub fn old_loop_summary(iterations: usize) -> AquiferSurfaceSamplingSummary {
    run_loop_summary(iterations, Shape::Old)
}

pub fn new_loop_summary(iterations: usize) -> AquiferSurfaceSamplingSummary {
    run_loop_summary(iterations, Shape::New)
}

fn run_loop_summary(iterations: usize, shape: Shape) -> AquiferSurfaceSamplingSummary {
    let mut sum = 0i64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        let value = match shape {
            Shape::Old => old_surface_sample(
                old_x(iteration),
                old_y(iteration),
                old_z(iteration),
            ),
            Shape::New => new_surface_sample(
                old_x(iteration),
                old_y(iteration),
                old_z(iteration),
            ),
        };
        sum = sum.wrapping_add(value as i64);
        last_bits = value as u32 as u64;
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((iteration as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(7)),
        );
    }

    AquiferSurfaceSamplingSummary {
        count: iterations as u64,
        sum_bits: sum as u64,
        value_checksum: checksum,
        last_bits,
    }
}

fn old_surface_sample(x: i32, y: i32, z: i32) -> i32 {
    let mut min_surface = i32::MAX;
    let mut result = 0i32;
    let above_probe_y = y.wrapping_add(12);
    let below_probe_y = y.wrapping_sub(12);
    let mut center_fluid = false;

    for offset in &CHUNK_OFFSETS {
        let sample_x = x.wrapping_add(section_to_block_coord(offset[0]));
        let sample_z = z.wrapping_add(section_to_block_coord(offset[1]));
        let surface = preliminary_surface_level(sample_x, sample_z);
        let adjusted_surface = surface.wrapping_add(8);
        let center = offset[0] == 0 && offset[1] == 0;
        if center && below_probe_y > adjusted_surface {
            return fluid_status(x, y, z);
        }

        let above_surface = above_probe_y > adjusted_surface;
        if above_surface || center {
            let fluid = fluid_status(sample_x, adjusted_surface, sample_z);
            if (fluid & 3) != 0 {
                if center {
                    center_fluid = true;
                }
                if above_surface {
                    return fluid;
                }
            }
        }

        min_surface = min_surface.min(surface);
        result ^= mix(sample_x, sample_z);
    }

    result.wrapping_add(min_surface)
        .wrapping_add(if center_fluid { 17 } else { 0 })
}

fn new_surface_sample(x: i32, y: i32, z: i32) -> i32 {
    let mut min_surface = i32::MAX;
    let mut result = 0i32;
    let above_probe_y = y.wrapping_add(12);
    let below_probe_y = y.wrapping_sub(12);
    let mut center_fluid = false;

    for i in 0..BLOCK_OFFSET_X.len() {
        let sample_x = x.wrapping_add(BLOCK_OFFSET_X[i]);
        let sample_z = z.wrapping_add(BLOCK_OFFSET_Z[i]);
        let surface = preliminary_surface_level(sample_x, sample_z);
        let adjusted_surface = surface.wrapping_add(8);
        let center = i == 0;
        if center && below_probe_y > adjusted_surface {
            return fluid_status(x, y, z);
        }

        let above_surface = above_probe_y > adjusted_surface;
        if above_surface || center {
            let fluid = fluid_status(sample_x, adjusted_surface, sample_z);
            if (fluid & 3) != 0 {
                if center {
                    center_fluid = true;
                }
                if above_surface {
                    return fluid;
                }
            }
        }

        min_surface = min_surface.min(surface);
        result ^= mix(sample_x, sample_z);
    }

    result.wrapping_add(min_surface)
        .wrapping_add(if center_fluid { 17 } else { 0 })
}

#[inline]
fn preliminary_surface_level(x: i32, z: i32) -> i32 {
    (mix(x >> 2, z >> 2) & 127) - 32
}

#[inline]
fn fluid_status(x: i32, y: i32, z: i32) -> i32 {
    if y < -54 {
        3
    } else {
        (mix(x, z).wrapping_add(y)) & 1
    }
}

#[inline]
fn section_to_block_coord(section_coord: i32) -> i32 {
    section_coord << 4
}

#[inline]
fn mix(x: i32, z: i32) -> i32 {
    let mut value = x
        .wrapping_mul(0x45D9_F3B)
        .wrapping_add(z.wrapping_mul(0x119D_E1F3)) as u32;
    value ^= value >> 16;
    value = value.wrapping_mul(0x045D_9F3B);
    value ^= value >> 16;
    value as i32
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[inline]
fn old_x(iteration: usize) -> i32 {
    ((iteration as i32).wrapping_mul(17)) & 0xFFFF
}

#[inline]
fn old_y(iteration: usize) -> i32 {
    ((iteration as i32) & 383).wrapping_sub(64)
}

#[inline]
fn old_z(iteration: usize) -> i32 {
    ((iteration as i32).wrapping_mul(31)) & 0xFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_shapes_match() {
        for i in 0..10_000 {
            let x = old_x(i);
            let y = old_y(i);
            let z = old_z(i);
            assert_eq!(old_surface_sample(x, y, z), new_surface_sample(x, y, z));
        }
    }

    #[test]
    fn repeated_runs_are_stable() {
        let first = old_loop_summary(4096);
        let second = old_loop_summary(4096);
        let third = new_loop_summary(4096);
        assert_eq!(first, second);
        assert_eq!(first, third);
        assert_eq!(first.count, 4096);
        assert_ne!(first.value_checksum, 0);
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(old_loop_summary(0), AquiferSurfaceSamplingSummary::default());
        assert_eq!(new_loop_summary(0), AquiferSurfaceSamplingSummary::default());
    }
}
