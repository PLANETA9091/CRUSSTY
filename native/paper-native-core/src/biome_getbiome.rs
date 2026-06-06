pub const SUMMARY_FIELDS: usize = 4;

const MAX_OFFSET: f64 = 0.4500000001;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const NEXT_MUL: i64 = 6_364_136_223_846_793_005;
const NEXT_ADD: i64 = 1_442_695_040_888_963_407;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BiomeGetBiomeSummary {
    pub count: u64,
    pub sum_values: u64,
    pub value_checksum: u64,
    pub last_value: u64,
}

pub fn current_batch_summary(
    seeds: &[i64],
    block_xs: &[i32],
    block_ys: &[i32],
    block_zs: &[i32],
) -> BiomeGetBiomeSummary {
    run_batch_summary(seeds, block_xs, block_ys, block_zs, current_corner)
}

pub fn optimized_batch_summary(
    seeds: &[i64],
    block_xs: &[i32],
    block_ys: &[i32],
    block_zs: &[i32],
) -> BiomeGetBiomeSummary {
    run_batch_summary(seeds, block_xs, block_ys, block_zs, optimized_corner)
}

fn run_batch_summary<F>(
    seeds: &[i64],
    block_xs: &[i32],
    block_ys: &[i32],
    block_zs: &[i32],
    mut corner: F,
) -> BiomeGetBiomeSummary
where
    F: FnMut(i64, i32, i32, i32) -> i32,
{
    debug_assert_eq!(seeds.len(), block_xs.len());
    debug_assert_eq!(seeds.len(), block_ys.len());
    debug_assert_eq!(seeds.len(), block_zs.len());

    let iterations = seeds.len();
    let mut sum = 0u64;
    let mut checksum = 0u64;
    let mut last_value = 0u64;

    for i in 0..iterations {
        let value = corner(seeds[i], block_xs[i], block_ys[i], block_zs[i]) as u64;
        sum = sum.wrapping_add(value);
        last_value = value;
        checksum = mix64(
            checksum
                ^ value
                ^ ((i as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    BiomeGetBiomeSummary {
        count: iterations as u64,
        sum_values: sum,
        value_checksum: checksum,
        last_value,
    }
}

fn current_corner(biome_zoom_seed: i64, block_x: i32, block_y: i32, block_z: i32) -> i32 {
    let x_minus_2 = block_x.wrapping_sub(2);
    let y_minus_2 = block_y.wrapping_sub(2);
    let z_minus_2 = block_z.wrapping_sub(2);
    let x = x_minus_2 >> 2;
    let y = y_minus_2 >> 2;
    let z = z_minus_2 >> 2;
    let quart_x = f64::from(x_minus_2 & 3) / 4.0;
    let quart_y = f64::from(y_minus_2 & 3) / 4.0;
    let quart_z = f64::from(z_minus_2 & 3) / 4.0;
    let mut smallest = 0;
    let mut smallest_distance = f64::INFINITY;

    for corner in 0..8 {
        let use_lower_x = (corner & 4) == 0;
        let use_lower_y = (corner & 2) == 0;
        let use_lower_z = (corner & 1) == 0;
        let sample_x = if use_lower_x { x } else { x.wrapping_add(1) };
        let sample_y = if use_lower_y { y } else { y.wrapping_add(1) };
        let sample_z = if use_lower_z { z } else { z.wrapping_add(1) };
        let x_noise = if use_lower_x { quart_x } else { quart_x - 1.0 };
        let y_noise = if use_lower_y { quart_y } else { quart_y - 1.0 };
        let z_noise = if use_lower_z { quart_z } else { quart_z - 1.0 };
        let distance = fiddled_distance(
            biome_zoom_seed,
            sample_x,
            sample_y,
            sample_z,
            x_noise,
            y_noise,
            z_noise,
        );
        if smallest_distance > distance {
            smallest = corner;
            smallest_distance = distance;
        }
    }

    smallest
}

fn optimized_corner(biome_zoom_seed: i64, block_x: i32, block_y: i32, block_z: i32) -> i32 {
    let x_minus_2 = block_x.wrapping_sub(2);
    let y_minus_2 = block_y.wrapping_sub(2);
    let z_minus_2 = block_z.wrapping_sub(2);
    let x = x_minus_2 >> 2;
    let y = y_minus_2 >> 2;
    let z = z_minus_2 >> 2;
    let quart_x = f64::from(x_minus_2 & 3) / 4.0;
    let quart_y = f64::from(y_minus_2 & 3) / 4.0;
    let quart_z = f64::from(z_minus_2 & 3) / 4.0;
    let mut smallest = 0;
    let mut smallest_distance = f64::INFINITY;

    for corner in 0..8 {
        let use_lower_x = (corner & 4) == 0;
        let use_lower_y = (corner & 2) == 0;
        let use_lower_z = (corner & 1) == 0;
        let quart_xx = if use_lower_x { quart_x } else { quart_x - 1.0 };
        let quart_yy = if use_lower_y { quart_y } else { quart_y - 1.0 };
        let quart_zz = if use_lower_z { quart_z } else { quart_z - 1.0 };

        let mut lower_bound_y = 0.0;
        let mut lower_bound_z = 0.0;
        if corner != 0 {
            let lower_bound_x = square((quart_xx.abs() - MAX_OFFSET).max(0.0));
            lower_bound_y = square((quart_yy.abs() - MAX_OFFSET).max(0.0));
            lower_bound_z = square((quart_zz.abs() - MAX_OFFSET).max(0.0));
            if smallest_distance < lower_bound_x + lower_bound_y + lower_bound_z {
                continue;
            }
        }

        let sample_x = if use_lower_x { x } else { x.wrapping_add(1) };
        let sample_y = if use_lower_y { y } else { y.wrapping_add(1) };
        let sample_z = if use_lower_z { z } else { z.wrapping_add(1) };

        let mut seed = next(biome_zoom_seed, sample_x as i64);
        seed = next(seed, sample_y as i64);
        seed = next(seed, sample_z as i64);
        seed = next(seed, sample_x as i64);
        seed = next(seed, sample_y as i64);
        seed = next(seed, sample_z as i64);
        let offset_x = get_fiddle(seed);
        let square_x = square(quart_xx + offset_x);
        if corner != 0 && smallest_distance < square_x + lower_bound_y + lower_bound_z {
            continue;
        }

        seed = next(seed, biome_zoom_seed);
        let offset_y = get_fiddle(seed);
        let square_y = square(quart_yy + offset_y);
        if corner != 0 && smallest_distance < square_x + square_y + lower_bound_z {
            continue;
        }

        seed = next(seed, biome_zoom_seed);
        let offset_z = get_fiddle(seed);
        let distance = square_x + square_y + square(quart_zz + offset_z);
        if smallest_distance > distance {
            smallest = corner;
            smallest_distance = distance;
        }
    }

    smallest
}

fn fiddled_distance(
    seed: i64,
    x: i32,
    y: i32,
    z: i32,
    x_noise: f64,
    y_noise: f64,
    z_noise: f64,
) -> f64 {
    let mut value = next(seed, x as i64);
    value = next(value, y as i64);
    value = next(value, z as i64);
    value = next(value, x as i64);
    value = next(value, y as i64);
    value = next(value, z as i64);
    let fiddle_x = get_fiddle(value);
    value = next(value, seed);
    let fiddle_y = get_fiddle(value);
    value = next(value, seed);
    let fiddle_z = get_fiddle(value);
    square(z_noise + fiddle_z) + square(y_noise + fiddle_y) + square(x_noise + fiddle_x)
}

#[inline]
fn next(left: i64, right: i64) -> i64 {
    let factor = left.wrapping_mul(NEXT_MUL).wrapping_add(NEXT_ADD);
    left.wrapping_mul(factor).wrapping_add(right)
}

#[inline]
fn get_fiddle(seed: i64) -> f64 {
    (((seed >> 24) & 1023) - 512) as f64 * (0.9 / 1024.0)
}

#[inline]
fn square(value: f64) -> f64 {
    value * value
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
    fn current_and_optimized_match_on_regular_inputs() {
        let seeds = [
            0,
            1,
            -1,
            1_234_567_890_123_456_789,
            -987_654_321_012_345_678,
        ];
        let xs = [0, 1, -2, 30_000_000, -30_000_000];
        let ys = [-64, 0, 12, 128, 320];
        let zs = [0, -1, 2, -30_000_000, 30_000_000];

        let current = current_batch_summary(&seeds, &xs, &ys, &zs);
        let optimized = optimized_batch_summary(&seeds, &xs, &ys, &zs);

        assert_eq!(current, optimized);
        assert_eq!(current.count, 5);
    }

    #[test]
    fn repeated_runs_are_stable() {
        let seeds = [0x1211_0B10_DE, -0x7fff_0000_1234_5678];
        let xs = [1234, -5678];
        let ys = [63, -64];
        let zs = [-4321, 8765];

        let first = optimized_batch_summary(&seeds, &xs, &ys, &zs);
        let second = optimized_batch_summary(&seeds, &xs, &ys, &zs);

        assert_eq!(first, second);
        assert_eq!(first, current_batch_summary(&seeds, &xs, &ys, &zs));
    }

    #[test]
    fn zero_iterations_are_empty() {
        let empty_long: [i64; 0] = [];
        let empty_int: [i32; 0] = [];
        let summary = current_batch_summary(&empty_long, &empty_int, &empty_int, &empty_int);
        assert_eq!(summary, BiomeGetBiomeSummary::default());
    }
}
