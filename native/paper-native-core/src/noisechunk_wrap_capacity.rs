pub const SUMMARY_FIELDS: usize = 8;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SUMMARY_TAG: u64 = 0x4E43_5752_4150_4341;
const FASTUTIL_MAX_ARRAY_SIZE: usize = 1 << 30;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoiseChunkWrapCapacitySummary {
    pub samples: u64,
    pub variants: u64,
    pub total_entries: u64,
    pub total_initial_n: u64,
    pub total_initial_max_fill: u64,
    pub total_final_n: u64,
    pub total_growths: u64,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapShape {
    pub n: usize,
    pub max_fill: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoiseChunkWrapCapacityError {
    InvalidInputLength,
    InvalidExpected,
    InvalidEntryCount,
    InvalidLoadFactor,
    TooLarge,
}

pub fn shape_summary(
    entries: &[i32],
    expected_sizes: &[i32],
    load_factors: &[f32],
    iterations: usize,
) -> Result<NoiseChunkWrapCapacitySummary, NoiseChunkWrapCapacityError> {
    if expected_sizes.len() != load_factors.len() {
        return Err(NoiseChunkWrapCapacityError::InvalidInputLength);
    }
    if iterations == 0 {
        return Ok(NoiseChunkWrapCapacitySummary::default());
    }

    let mut summary = NoiseChunkWrapCapacitySummary {
        samples: (entries.len() as u64).wrapping_mul(iterations as u64),
        variants: expected_sizes.len() as u64,
        checksum: mix64(
            SUMMARY_TAG
                ^ entries.len() as u64
                ^ ((expected_sizes.len() as u64) << 32)
                ^ ((iterations as u64) << 17),
        ),
        ..NoiseChunkWrapCapacitySummary::default()
    };

    for round in 0..iterations {
        for (sample_index, &entry_count) in entries.iter().enumerate() {
            let entry_count = usize::try_from(entry_count)
                .map_err(|_| NoiseChunkWrapCapacityError::InvalidEntryCount)?;
            for (variant_index, (&expected, &load_factor)) in expected_sizes
                .iter()
                .zip(load_factors.iter())
                .enumerate()
            {
                let expected = usize::try_from(expected)
                    .map_err(|_| NoiseChunkWrapCapacityError::InvalidExpected)?;
                let initial = shape_for_expected(expected, load_factor)?;
                let (final_shape, growths) =
                    final_shape_for_entries(initial, load_factor, entry_count)?;

                summary.total_entries = summary.total_entries.wrapping_add(entry_count as u64);
                summary.total_initial_n = summary.total_initial_n.wrapping_add(initial.n as u64);
                summary.total_initial_max_fill = summary
                    .total_initial_max_fill
                    .wrapping_add(initial.max_fill as u64);
                summary.total_final_n = summary.total_final_n.wrapping_add(final_shape.n as u64);
                summary.total_growths = summary.total_growths.wrapping_add(growths as u64);
                summary.checksum = mix64(
                    summary.checksum
                        ^ ((round as u64).wrapping_mul(MIX_GAMMA))
                        ^ ((sample_index as u64).wrapping_mul(MIX_GAMMA))
                        ^ ((variant_index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                        ^ ((entry_count as u64) << 1)
                        ^ ((expected as u64) << 17)
                        ^ ((load_factor.to_bits() as u64) << 32)
                        ^ ((initial.n as u64) << 7)
                        ^ ((initial.max_fill as u64) << 11)
                        ^ ((final_shape.n as u64) << 23)
                        ^ ((final_shape.max_fill as u64) << 29)
                        ^ growths as u64,
                );
            }
        }
    }

    Ok(summary)
}

pub fn shape_for_expected(
    expected: usize,
    load_factor: f32,
) -> Result<MapShape, NoiseChunkWrapCapacityError> {
    let n = array_size(expected, load_factor)?;
    Ok(MapShape {
        n,
        max_fill: max_fill(n, load_factor)?,
    })
}

pub fn final_shape_for_entries(
    initial: MapShape,
    load_factor: f32,
    entries: usize,
) -> Result<(MapShape, usize), NoiseChunkWrapCapacityError> {
    validate_load_factor(load_factor)?;
    let mut shape = initial;
    let mut growths = 0usize;
    while entries > shape.max_fill {
        if shape.n >= FASTUTIL_MAX_ARRAY_SIZE {
            return Err(NoiseChunkWrapCapacityError::TooLarge);
        }
        shape.n = shape
            .n
            .checked_mul(2)
            .ok_or(NoiseChunkWrapCapacityError::TooLarge)?;
        if shape.n > FASTUTIL_MAX_ARRAY_SIZE {
            return Err(NoiseChunkWrapCapacityError::TooLarge);
        }
        shape.max_fill = max_fill(shape.n, load_factor)?;
        growths += 1;
    }
    Ok((shape, growths))
}

fn array_size(expected: usize, load_factor: f32) -> Result<usize, NoiseChunkWrapCapacityError> {
    validate_load_factor(load_factor)?;

    let target = ((expected as f64) / (load_factor as f64)).ceil();
    if !target.is_finite() || target > FASTUTIL_MAX_ARRAY_SIZE as f64 {
        return Err(NoiseChunkWrapCapacityError::TooLarge);
    }

    let target = target.max(2.0) as usize;
    let size = target.next_power_of_two();
    if size > FASTUTIL_MAX_ARRAY_SIZE {
        return Err(NoiseChunkWrapCapacityError::TooLarge);
    }
    Ok(size)
}

fn max_fill(n: usize, load_factor: f32) -> Result<usize, NoiseChunkWrapCapacityError> {
    validate_load_factor(load_factor)?;
    Ok((((n as f64) * (load_factor as f64)).ceil() as usize).min(n - 1))
}

fn validate_load_factor(load_factor: f32) -> Result<(), NoiseChunkWrapCapacityError> {
    if load_factor.is_finite() && load_factor > 0.0 && load_factor < 1.0 {
        Ok(())
    } else {
        Err(NoiseChunkWrapCapacityError::InvalidLoadFactor)
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
    fn fastutil_shapes_match_observed_noisechunk_report() {
        assert_eq!(
            shape_for_expected(2048, 0.75).unwrap(),
            MapShape {
                n: 4096,
                max_fill: 3072,
            }
        );
        assert_eq!(
            shape_for_expected(2048, 0.95).unwrap(),
            MapShape {
                n: 4096,
                max_fill: 3892,
            }
        );
        assert_eq!(
            shape_for_expected(12288, 0.75).unwrap(),
            MapShape {
                n: 16384,
                max_fill: 12288,
            }
        );
        assert_eq!(
            shape_for_expected(12289, 0.75).unwrap(),
            MapShape {
                n: 32768,
                max_fill: 24576,
            }
        );
        assert_eq!(
            shape_for_expected(16384, 0.75).unwrap(),
            MapShape {
                n: 32768,
                max_fill: 24576,
            }
        );
    }

    #[test]
    fn final_shape_tracks_rehash_thresholds() {
        let initial = shape_for_expected(2048, 0.75).unwrap();
        let (final_shape, growths) = final_shape_for_entries(initial, 0.75, 17115).unwrap();

        assert_eq!(
            final_shape,
            MapShape {
                n: 32768,
                max_fill: 24576,
            }
        );
        assert_eq!(growths, 3);

        let initial = shape_for_expected(8192, 0.75).unwrap();
        let (final_shape, growths) = final_shape_for_entries(initial, 0.75, 17115).unwrap();

        assert_eq!(
            final_shape,
            MapShape {
                n: 32768,
                max_fill: 24576,
            }
        );
        assert_eq!(growths, 1);

        let initial = shape_for_expected(16384, 0.75).unwrap();
        let (final_shape, growths) = final_shape_for_entries(initial, 0.75, 17115).unwrap();

        assert_eq!(
            final_shape,
            MapShape {
                n: 32768,
                max_fill: 24576,
            }
        );
        assert_eq!(growths, 0);
    }

    #[test]
    fn summary_is_stable() {
        let first = shape_summary(&[17115, 50, 55], &[2048, 16384], &[0.75, 0.75], 1).unwrap();
        let second = shape_summary(&[17115, 50, 55], &[2048, 16384], &[0.75, 0.75], 1).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.samples, 3);
        assert_eq!(first.variants, 2);
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(
            shape_summary(&[1], &[2048], &[], 1),
            Err(NoiseChunkWrapCapacityError::InvalidInputLength)
        );
        assert_eq!(
            shape_for_expected(2048, 1.0),
            Err(NoiseChunkWrapCapacityError::InvalidLoadFactor)
        );
    }
}
