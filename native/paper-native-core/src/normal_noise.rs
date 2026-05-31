use crate::perlin_noise::PerlinNoise;

pub const INPUT_FACTOR: f64 = 1.018_126_888_217_522_7;

#[inline]
pub fn get_value(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    x: f64,
    y: f64,
    z: f64,
) -> f64 {
    let d = x * INPUT_FACTOR;
    let d1 = y * INPUT_FACTOR;
    let d2 = z * INPUT_FACTOR;
    (first.get_value_direct_math_wrap(x, y, z) + second.get_value_direct_math_wrap(d, d1, d2)) * value_factor
}

#[inline]
fn get_value_scaled_second(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    x: f64,
    y: f64,
    z: f64,
    second_x: f64,
    second_y: f64,
    second_z: f64,
) -> f64 {
    (first.get_value_direct_math_wrap(x, y, z)
        + second.get_value_direct_math_wrap(second_x, second_y, second_z))
        * value_factor
}

#[inline]
fn scaled_second(value: f64) -> f64 {
    value * INPUT_FACTOR
}

#[inline]
pub fn fill_positions(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    dst: &mut [f64],
) -> Result<(), &'static str> {
    if xs.len() != ys.len() || xs.len() != zs.len() {
        return Err("input lengths do not match");
    }
    if dst.len() != xs.len() {
        return Err("destination length does not match input length");
    }

    for index in 0..xs.len() {
        dst[index] = get_value_scaled_second(
            first,
            second,
            value_factor,
            xs[index],
            ys[index],
            zs[index],
            scaled_second(xs[index]),
            scaled_second(ys[index]),
            scaled_second(zs[index]),
        );
    }

    Ok(())
}

#[inline]
pub fn fill_scaled_positions(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    xz_scale: f64,
    y_scale: f64,
    dst: &mut [f64],
) -> Result<(), &'static str> {
    if block_x.len() != block_y.len() || block_x.len() != block_z.len() {
        return Err("input lengths do not match");
    }
    if dst.len() != block_x.len() {
        return Err("destination length does not match input length");
    }

    for index in 0..block_x.len() {
        let noise_x = block_x[index] as f64 * xz_scale;
        let noise_y = block_y[index] as f64 * y_scale;
        let noise_z = block_z[index] as f64 * xz_scale;
        dst[index] = get_value_scaled_second(
            first,
            second,
            value_factor,
            noise_x,
            noise_y,
            noise_z,
            scaled_second(noise_x),
            scaled_second(noise_y),
            scaled_second(noise_z),
        );
    }

    Ok(())
}

#[inline]
pub fn fill_shifted_positions_in_place(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    shift_x_and_dst: &mut [f64],
    shift_y: &[f64],
    shift_z: &[f64],
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    xz_scale: f64,
    y_scale: f64,
) -> Result<(), &'static str> {
    if shift_x_and_dst.len() != shift_y.len()
        || shift_x_and_dst.len() != shift_z.len()
        || shift_x_and_dst.len() != block_x.len()
        || shift_x_and_dst.len() != block_y.len()
        || shift_x_and_dst.len() != block_z.len()
    {
        return Err("input lengths do not match");
    }

    for index in 0..shift_x_and_dst.len() {
        let noise_x = block_x[index] as f64 * xz_scale + shift_x_and_dst[index];
        let noise_y = block_y[index] as f64 * y_scale + shift_y[index];
        let noise_z = block_z[index] as f64 * xz_scale + shift_z[index];
        shift_x_and_dst[index] = get_value_scaled_second(
            first,
            second,
            value_factor,
            noise_x,
            noise_y,
            noise_z,
            scaled_second(noise_x),
            scaled_second(noise_y),
            scaled_second(noise_z),
        );
    }

    Ok(())
}

#[inline]
pub fn fill_shift_positions(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    block_x: &[i32],
    block_y: &[i32],
    block_z: &[i32],
    dst: &mut [f64],
) -> Result<(), &'static str> {
    if block_x.len() != block_y.len() || block_x.len() != block_z.len() {
        return Err("input lengths do not match");
    }
    if dst.len() != block_x.len() {
        return Err("destination length does not match input length");
    }

    for index in 0..block_x.len() {
        let noise_x = block_x[index] as f64 * 0.25;
        let noise_y = block_y[index] as f64 * 0.25;
        let noise_z = block_z[index] as f64 * 0.25;
        dst[index] = get_value_scaled_second(
            first,
            second,
            value_factor,
            noise_x,
            noise_y,
            noise_z,
            scaled_second(noise_x),
            scaled_second(noise_y),
            scaled_second(noise_z),
        ) * 4.0;
    }

    Ok(())
}

#[inline]
pub fn fill_shift_a(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    block_x: &[i32],
    block_z: &[i32],
    dst: &mut [f64],
) -> Result<(), &'static str> {
    if block_x.len() != block_z.len() {
        return Err("input lengths do not match");
    }
    if dst.len() != block_x.len() {
        return Err("destination length does not match input length");
    }

    for index in 0..block_x.len() {
        let noise_x = block_x[index] as f64 * 0.25;
        let noise_z = block_z[index] as f64 * 0.25;
        dst[index] = get_value_scaled_second(
            first,
            second,
            value_factor,
            noise_x,
            0.0,
            noise_z,
            scaled_second(noise_x),
            0.0,
            scaled_second(noise_z),
        ) * 4.0;
    }

    Ok(())
}

#[inline]
pub fn fill_shift_b(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    block_x: &[i32],
    block_z: &[i32],
    dst: &mut [f64],
) -> Result<(), &'static str> {
    if block_x.len() != block_z.len() {
        return Err("input lengths do not match");
    }
    if dst.len() != block_x.len() {
        return Err("destination length does not match input length");
    }

    for index in 0..block_x.len() {
        let noise_x = block_z[index] as f64 * 0.25;
        let noise_y = block_x[index] as f64 * 0.25;
        dst[index] = get_value_scaled_second(
            first,
            second,
            value_factor,
            noise_x,
            noise_y,
            0.0,
            scaled_second(noise_x),
            scaled_second(noise_y),
            0.0,
        ) * 4.0;
    }

    Ok(())
}

#[inline]
pub fn fill_vertical(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    x: f64,
    start_y: f64,
    y_step: f64,
    z: f64,
    dst: &mut [f64],
) {
    let second_x = scaled_second(x);
    let second_z = scaled_second(z);
    for (index, value) in dst.iter_mut().enumerate() {
        let noise_y = start_y + index as f64 * y_step;
        *value = get_value_scaled_second(
            first,
            second,
            value_factor,
            x,
            noise_y,
            z,
            second_x,
            scaled_second(noise_y),
            second_z,
        );
    }
}

#[inline]
pub fn fill_cell(
    first: &PerlinNoise,
    second: &PerlinNoise,
    value_factor: f64,
    cell_width: usize,
    cell_height: usize,
    base_x: f64,
    base_y: f64,
    base_z: f64,
    xz_scale: f64,
    y_scale: f64,
    dst: &mut [f64],
) -> Result<(), &'static str> {
    let expected = cell_width
        .checked_mul(cell_width)
        .and_then(|value| value.checked_mul(cell_height))
        .ok_or("cell dimensions overflow")?;
    if dst.len() != expected {
        return Err("destination length does not match cell dimensions");
    }

    let mut index = 0;
    for y in (0..cell_height).rev() {
        let noise_y = (base_y + y as f64) * y_scale;
        let second_y = scaled_second(noise_y);
        for x in 0..cell_width {
            let noise_x = (base_x + x as f64) * xz_scale;
            let second_x = scaled_second(noise_x);
            for z in 0..cell_width {
                let noise_z = (base_z + z as f64) * xz_scale;
                dst[index] = get_value_scaled_second(
                    first,
                    second,
                    value_factor,
                    noise_x,
                    noise_y,
                    noise_z,
                    second_x,
                    second_y,
                    scaled_second(noise_z),
                );
                index += 1;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::improved_noise::PERMUTATION_LENGTH;

    #[test]
    fn folded_get_value_matches_manual_formula() {
        let (permutations_a, active, y_origins, amplitudes) = model_octaves(16, 0x5eed_1234);
        let (permutations_b, _, _, _) = model_octaves(16, 0x1234_abcd);
        let x_origins = vec![0.0; active.len()];
        let z_origins = vec![0.0; active.len()];
        let (input_factor, value_factor) = model_factors(16);
        let first = PerlinNoise::new_from_flat_with_origins(
            &permutations_a,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();
        let second = PerlinNoise::new_from_flat_with_origins(
            &permutations_b,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        for &(x, y, z) in &[
            (-1024.25, -64.0, 31.5),
            (0.0, 63.75, 0.125),
            (30_000_000.0, 319.0, -29_999_999.5),
        ] {
            let d = x * INPUT_FACTOR;
            let d1 = y * INPUT_FACTOR;
            let d2 = z * INPUT_FACTOR;
            let expected = (first.get_value_direct_math_wrap(x, y, z)
                + second.get_value_direct_math_wrap(d, d1, d2))
                * value_factor;
            assert_eq!(get_value(&first, &second, value_factor, x, y, z).to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn batched_get_value_paths_match_scalar_order() {
        let (permutations_a, active, y_origins, amplitudes) = model_octaves(16, 0x7788_99aa);
        let (permutations_b, _, _, _) = model_octaves(16, 0xaabb_ccdd);
        let x_origins = vec![0.0; active.len()];
        let z_origins = vec![0.0; active.len()];
        let (input_factor, value_factor) = model_factors(16);
        let first = PerlinNoise::new_from_flat_with_origins(
            &permutations_a,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();
        let second = PerlinNoise::new_from_flat_with_origins(
            &permutations_b,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        let mut vertical = [0.0; 9];
        fill_vertical(&first, &second, value_factor, 12.5, -64.0, 8.0, -7.25, &mut vertical);
        for (index, value) in vertical.iter().enumerate() {
            let expected = get_value(&first, &second, value_factor, 12.5, -64.0 + index as f64 * 8.0, -7.25);
            assert_eq!(value.to_bits(), expected.to_bits());
        }

        let mut cell = [0.0; 4 * 4 * 8];
        fill_cell(
            &first,
            &second,
            value_factor,
            4,
            8,
            -16.0,
            32.0,
            48.0,
            0.25,
            0.5,
            &mut cell,
        )
        .unwrap();

        let mut index = 0;
        for y in (0..8).rev() {
            for x in 0..4 {
                for z in 0..4 {
                    let expected = get_value(
                        &first,
                        &second,
                        value_factor,
                        (-16.0 + x as f64) * 0.25,
                        (32.0 + y as f64) * 0.5,
                        (48.0 + z as f64) * 0.25,
                    );
                    assert_eq!(cell[index].to_bits(), expected.to_bits());
                    index += 1;
                }
            }
        }
    }

    #[test]
    fn fill_positions_matches_scalar_get_value() {
        let (permutations_a, active, y_origins, amplitudes) = model_octaves(16, 0x1357_9bdf);
        let (permutations_b, _, _, _) = model_octaves(16, 0x2468_ace0);
        let x_origins = vec![0.0; active.len()];
        let z_origins = vec![0.0; active.len()];
        let (input_factor, value_factor) = model_factors(16);
        let first = PerlinNoise::new_from_flat_with_origins(
            &permutations_a,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();
        let second = PerlinNoise::new_from_flat_with_origins(
            &permutations_b,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        let xs = [-1024.25, 0.0, 30_000_000.0, -7.5];
        let ys = [-64.0, 63.75, 319.0, 0.125];
        let zs = [31.5, 0.125, -29_999_999.5, 8.25];
        let mut dst = [0.0; 4];

        fill_positions(&first, &second, value_factor, &xs, &ys, &zs, &mut dst).unwrap();

        for index in 0..dst.len() {
            let expected = get_value(&first, &second, value_factor, xs[index], ys[index], zs[index]);
            assert_eq!(dst[index].to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn direct_block_position_batches_match_scalar_paths() {
        let (permutations_a, active, y_origins, amplitudes) = model_octaves(16, 0x3344_5566);
        let (permutations_b, _, _, _) = model_octaves(16, 0x7788_99aa);
        let x_origins = vec![0.0; active.len()];
        let z_origins = vec![0.0; active.len()];
        let (input_factor, value_factor) = model_factors(16);
        let first = PerlinNoise::new_from_flat_with_origins(
            &permutations_a,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();
        let second = PerlinNoise::new_from_flat_with_origins(
            &permutations_b,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        let xs = [4, -8, 16, 31];
        let ys = [64, -16, 0, 320];
        let zs = [20, -24, 32, -63];
        let mut scaled = [0.0; 4];
        fill_scaled_positions(&first, &second, value_factor, &xs, &ys, &zs, 0.5, 0.25, &mut scaled).unwrap();
        for index in 0..scaled.len() {
            let expected = get_value(
                &first,
                &second,
                value_factor,
                xs[index] as f64 * 0.5,
                ys[index] as f64 * 0.25,
                zs[index] as f64 * 0.5,
            );
            assert_eq!(scaled[index].to_bits(), expected.to_bits());
        }

        let mut shift = [0.0; 4];
        fill_shift_positions(&first, &second, value_factor, &xs, &ys, &zs, &mut shift).unwrap();
        for index in 0..shift.len() {
            let expected = get_value(
                &first,
                &second,
                value_factor,
                xs[index] as f64 * 0.25,
                ys[index] as f64 * 0.25,
                zs[index] as f64 * 0.25,
            ) * 4.0;
            assert_eq!(shift[index].to_bits(), expected.to_bits());
        }

        let mut shift_a = [0.0; 4];
        fill_shift_a(&first, &second, value_factor, &xs, &zs, &mut shift_a).unwrap();
        for index in 0..shift_a.len() {
            let expected = get_value(
                &first,
                &second,
                value_factor,
                xs[index] as f64 * 0.25,
                0.0,
                zs[index] as f64 * 0.25,
            ) * 4.0;
            assert_eq!(shift_a[index].to_bits(), expected.to_bits());
        }

        let mut shift_b = [0.0; 4];
        fill_shift_b(&first, &second, value_factor, &xs, &zs, &mut shift_b).unwrap();
        for index in 0..shift_b.len() {
            let expected = get_value(
                &first,
                &second,
                value_factor,
                zs[index] as f64 * 0.25,
                xs[index] as f64 * 0.25,
                0.0,
            ) * 4.0;
            assert_eq!(shift_b[index].to_bits(), expected.to_bits());
        }

        let mut shift_x = [0.25, -1.5, 2.75, 0.0];
        let shift_y = [-0.5, 1.25, 0.0, 3.5];
        let shift_z = [1.0, 0.0, -2.25, 4.0];
        fill_shifted_positions_in_place(
            &first,
            &second,
            value_factor,
            &mut shift_x,
            &shift_y,
            &shift_z,
            &xs,
            &ys,
            &zs,
            0.5,
            0.25,
        )
        .unwrap();
        for index in 0..shift_x.len() {
            let expected = get_value(
                &first,
                &second,
                value_factor,
                xs[index] as f64 * 0.5 + [0.25, -1.5, 2.75, 0.0][index],
                ys[index] as f64 * 0.25 + shift_y[index],
                zs[index] as f64 * 0.5 + shift_z[index],
            );
            assert_eq!(shift_x[index].to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn fill_positions_rejects_length_mismatch() {
        let (permutations_a, active, y_origins, amplitudes) = model_octaves(16, 0xdead_beef);
        let (permutations_b, _, _, _) = model_octaves(16, 0xfeed_cafe);
        let x_origins = vec![0.0; active.len()];
        let z_origins = vec![0.0; active.len()];
        let (input_factor, value_factor) = model_factors(16);
        let first = PerlinNoise::new_from_flat_with_origins(
            &permutations_a,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();
        let second = PerlinNoise::new_from_flat_with_origins(
            &permutations_b,
            &active,
            &x_origins,
            &y_origins,
            &z_origins,
            &amplitudes,
            input_factor,
            value_factor,
        )
        .unwrap();

        let xs = [1.0, 2.0];
        let ys = [3.0];
        let zs = [4.0, 5.0];
        let mut dst = [0.0; 2];

        assert_eq!(
            fill_positions(&first, &second, value_factor, &xs, &ys, &zs, &mut dst).unwrap_err(),
            "input lengths do not match"
        );
    }

    fn model_octaves(octaves: usize, base_seed: u32) -> (Vec<u8>, Vec<u8>, Vec<f64>, Vec<f64>) {
        let mut permutations = Vec::with_capacity(octaves * PERMUTATION_LENGTH);
        let mut active = Vec::with_capacity(octaves);
        let mut y_origins = Vec::with_capacity(octaves);
        let mut amplitudes = Vec::with_capacity(octaves);

        for octave in 0..octaves {
            let seed = base_seed.wrapping_add((octave as u32).wrapping_mul(0x9E37_79B9));
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
