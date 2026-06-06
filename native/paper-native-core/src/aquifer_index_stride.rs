use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 4;

const MIN_GRID_X: i32 = 3;
const MIN_GRID_Y: i32 = -2;
const MIN_GRID_Z: i32 = 5;
const GRID_SIZE_X: i32 = 11;
const GRID_SIZE_Y: i32 = 9;
const GRID_SIZE_Z: i32 = 10;
const Y_STRIDE: i32 = GRID_SIZE_X * GRID_SIZE_Z;
const Z_STRIDE: i32 = GRID_SIZE_X;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AquiferIndexStrideSummary {
    pub count: u64,
    pub result: i64,
    pub value_checksum: u64,
    pub last_value: i64,
}

#[derive(Debug)]
struct Model {
    location_cache: Vec<i64>,
}

impl Model {
    fn new() -> Self {
        Self {
            location_cache: create_location_cache(),
        }
    }

    fn old_loop_summary(&self, iterations: usize) -> AquiferIndexStrideSummary {
        run_loop_summary(iterations, |iteration| {
            self.old_sample(sample_x(iteration), sample_y(iteration), sample_z(iteration))
        })
    }

    fn new_loop_summary(&self, iterations: usize) -> AquiferIndexStrideSummary {
        run_loop_summary(iterations, |iteration| {
            self.new_sample(sample_x(iteration), sample_y(iteration), sample_z(iteration))
        })
    }

    fn old_sample(&self, x: i32, y: i32, z: i32) -> i32 {
        let grid_x = grid_x(x - 5);
        let grid_y = grid_y(y + 1);
        let grid_z = grid_z(z - 5);

        let mut closest = i32::MAX;
        let mut second = i32::MAX;
        let mut third = i32::MAX;
        let mut fourth = i32::MAX;
        let mut closest_index = 0i32;
        let mut second_index = 0i32;
        let mut third_index = 0i32;
        let mut fourth_index = 0i32;
        let mut checksum = 0i32;

        for x_offset in 0..=1 {
            for y_offset in -1..=1 {
                for z_offset in 0..=1 {
                    let index = get_index(grid_x + x_offset, grid_y + y_offset, grid_z + z_offset);
                    let packed = self.location_cache[index as usize];
                    let dx = unpack_x(packed) - x;
                    let dy = unpack_y(packed) - y;
                    let dz = unpack_z(packed) - z;
                    let distance = dx
                        .wrapping_mul(dx)
                        .wrapping_add(dy.wrapping_mul(dy))
                        .wrapping_add(dz.wrapping_mul(dz));
                    checksum ^= index.wrapping_add(distance);
                    if closest >= distance {
                        fourth_index = third_index;
                        third_index = second_index;
                        second_index = closest_index;
                        closest_index = index;
                        fourth = third;
                        third = second;
                        second = closest;
                        closest = distance;
                    } else if second >= distance {
                        fourth_index = third_index;
                        third_index = second_index;
                        second_index = index;
                        fourth = third;
                        third = second;
                        second = distance;
                    } else if third >= distance {
                        fourth_index = third_index;
                        third_index = index;
                        fourth = third;
                        third = distance;
                    } else if fourth >= distance {
                        fourth_index = index;
                        fourth = distance;
                    }
                }
            }
        }

        checksum ^ closest ^ second ^ third ^ fourth ^ closest_index ^ second_index ^ third_index ^ fourth_index
    }

    fn new_sample(&self, x: i32, y: i32, z: i32) -> i32 {
        let grid_x = grid_x(x - 5);
        let grid_y = grid_y(y + 1);
        let grid_z = grid_z(z - 5);
        let base_index = get_index(grid_x, grid_y - 1, grid_z);

        let mut closest = i32::MAX;
        let mut second = i32::MAX;
        let mut third = i32::MAX;
        let mut fourth = i32::MAX;
        let mut closest_index = 0i32;
        let mut second_index = 0i32;
        let mut third_index = 0i32;
        let mut fourth_index = 0i32;
        let mut checksum = 0i32;

        for x_offset in 0..=1 {
            let x_index = base_index + x_offset;
            for y_offset in -1..=1 {
                let row_index = x_index + (y_offset + 1) * Y_STRIDE;
                for z_offset in 0..=1 {
                    let index = row_index + z_offset * Z_STRIDE;
                    let packed = self.location_cache[index as usize];
                    let dx = unpack_x(packed) - x;
                    let dy = unpack_y(packed) - y;
                    let dz = unpack_z(packed) - z;
                    let distance = dx
                        .wrapping_mul(dx)
                        .wrapping_add(dy.wrapping_mul(dy))
                        .wrapping_add(dz.wrapping_mul(dz));
                    checksum ^= index.wrapping_add(distance);
                    if closest >= distance {
                        fourth_index = third_index;
                        third_index = second_index;
                        second_index = closest_index;
                        closest_index = index;
                        fourth = third;
                        third = second;
                        second = closest;
                        closest = distance;
                    } else if second >= distance {
                        fourth_index = third_index;
                        third_index = second_index;
                        second_index = index;
                        fourth = third;
                        third = second;
                        second = distance;
                    } else if third >= distance {
                        fourth_index = third_index;
                        third_index = index;
                        fourth = third;
                        third = distance;
                    } else if fourth >= distance {
                        fourth_index = index;
                        fourth = distance;
                    }
                }
            }
        }

        checksum ^ closest ^ second ^ third ^ fourth ^ closest_index ^ second_index ^ third_index ^ fourth_index
    }
}

fn model() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(Model::new)
}

pub fn old_loop_summary(iterations: usize) -> AquiferIndexStrideSummary {
    model().old_loop_summary(iterations)
}

pub fn new_loop_summary(iterations: usize) -> AquiferIndexStrideSummary {
    model().new_loop_summary(iterations)
}

fn run_loop_summary<F>(iterations: usize, mut sample: F) -> AquiferIndexStrideSummary
where
    F: FnMut(usize) -> i32,
{
    let mut result = 0i64;
    let mut checksum = 0u64;
    let mut last_value = 0i64;

    for iteration in 0..iterations {
        let value = sample(iteration);
        result ^= value as i64;
        last_value = value as i64;
        checksum = mix64(
            checksum
                ^ (value as u64)
                ^ ((iteration as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(11)),
        );
    }

    AquiferIndexStrideSummary {
        count: iterations as u64,
        result,
        value_checksum: checksum,
        last_value,
    }
}

fn create_location_cache() -> Vec<i64> {
    let mut cache = Vec::with_capacity((GRID_SIZE_X * GRID_SIZE_Y * GRID_SIZE_Z) as usize);
    for y in 0..GRID_SIZE_Y {
        for z in 0..GRID_SIZE_Z {
            for x in 0..GRID_SIZE_X {
                let grid_x = MIN_GRID_X + x;
                let grid_y = MIN_GRID_Y + y;
                let grid_z = MIN_GRID_Z + z;
                let noise = mix(grid_x, grid_y, grid_z) as u32;
                let block_x = from_grid_x(grid_x, (noise & 9) as i32);
                let block_y = from_grid_y(grid_y, ((noise >> 4) & 7) as i32);
                let block_z = from_grid_z(grid_z, ((noise >> 8) & 9) as i32);
                cache.push(pack(block_x, block_y, block_z));
            }
        }
    }
    cache
}

#[inline]
fn get_index(grid_x: i32, grid_y: i32, grid_z: i32) -> i32 {
    ((grid_y - MIN_GRID_Y) * GRID_SIZE_Z + (grid_z - MIN_GRID_Z)) * GRID_SIZE_X + (grid_x - MIN_GRID_X)
}

#[inline]
fn grid_x(x: i32) -> i32 {
    x >> 4
}

#[inline]
fn grid_y(y: i32) -> i32 {
    y.div_euclid(12)
}

#[inline]
fn grid_z(z: i32) -> i32 {
    z >> 4
}

#[inline]
fn from_grid_x(grid_x: i32, offset: i32) -> i32 {
    (grid_x << 4) + offset
}

#[inline]
fn from_grid_y(grid_y: i32, offset: i32) -> i32 {
    grid_y * 12 + offset
}

#[inline]
fn from_grid_z(grid_z: i32, offset: i32) -> i32 {
    (grid_z << 4) + offset
}

#[inline]
fn pack(x: i32, y: i32, z: i32) -> i64 {
    (((x as i64) & 0x1F_FFFF) << 42) | (((y as i64) & 0x1F_FFFF) << 21) | ((z as i64) & 0x1F_FFFF)
}

#[inline]
fn unpack_x(packed: i64) -> i32 {
    ((packed << 22) >> 43) as i32
}

#[inline]
fn unpack_y(packed: i64) -> i32 {
    ((packed << 43) >> 43) as i32
}

#[inline]
fn unpack_z(packed: i64) -> i32 {
    ((packed << 1) >> 43) as i32
}

#[inline]
fn mix(x: i32, y: i32, z: i32) -> i32 {
    let mut value = (x.wrapping_mul(0x045D_9F3B)
        ^ y.wrapping_mul(0x119D_E1F3)
        ^ z.wrapping_mul(0x27D4_EB2D)) as u32;
    value ^= value >> 16;
    value = value.wrapping_mul(0x045D_9F3B);
    value ^= value >> 15;
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
fn sample_x(iteration: usize) -> i32 {
    let grid = MIN_GRID_X + (iteration as i32).wrapping_mul(17).rem_euclid(GRID_SIZE_X - 1);
    (grid << 4) + 5 + ((iteration as i32) & 15)
}

#[inline]
fn sample_y(iteration: usize) -> i32 {
    let grid = MIN_GRID_Y + 1 + (iteration as i32).wrapping_mul(7).rem_euclid(GRID_SIZE_Y - 2);
    grid * 12 - 1 + (iteration as i32).rem_euclid(12)
}

#[inline]
fn sample_z(iteration: usize) -> i32 {
    let grid = MIN_GRID_Z + (iteration as i32).wrapping_mul(31).rem_euclid(GRID_SIZE_Z - 1);
    (grid << 4) + 5 + ((iteration as i32) & 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_shapes_match() {
        for i in 0..10_000 {
            let x = sample_x(i);
            let y = sample_y(i);
            let z = sample_z(i);
            assert_eq!(model().old_sample(x, y, z), model().new_sample(x, y, z));
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
        assert_eq!(old_loop_summary(0), AquiferIndexStrideSummary::default());
        assert_eq!(new_loop_summary(0), AquiferIndexStrideSummary::default());
    }
}
