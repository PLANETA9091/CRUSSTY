pub const SUMMARY_FIELDS: usize = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OreFeatureLoopError {
    LengthMismatch,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OreFeatureLoopSummary {
    pub checksum: u64,
}

#[derive(Clone, Copy)]
pub struct OreFeatureLoopConfig {
    pub width: i32,
    pub height: i32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub origin_z: i32,
}

#[derive(Clone, Copy)]
pub struct OreFeatureLoopArrays<'a> {
    pub center_x: &'a [f64],
    pub center_y: &'a [f64],
    pub center_z: &'a [f64],
    pub radius: &'a [f64],
    pub min_x: &'a [i32],
    pub min_y: &'a [i32],
    pub min_z: &'a [i32],
    pub max_x: &'a [i32],
    pub max_y: &'a [i32],
    pub max_z: &'a [i32],
}

#[inline]
pub fn old_loop_summary(
    arrays: OreFeatureLoopArrays<'_>,
    config: OreFeatureLoopConfig,
) -> Result<OreFeatureLoopSummary, OreFeatureLoopError> {
    loop_summary(arrays, config, false)
}

#[inline]
pub fn optimized_loop_summary(
    arrays: OreFeatureLoopArrays<'_>,
    config: OreFeatureLoopConfig,
) -> Result<OreFeatureLoopSummary, OreFeatureLoopError> {
    loop_summary(arrays, config, true)
}

fn loop_summary(
    arrays: OreFeatureLoopArrays<'_>,
    config: OreFeatureLoopConfig,
    optimized: bool,
) -> Result<OreFeatureLoopSummary, OreFeatureLoopError> {
    let len = arrays.center_x.len();
    if arrays.center_y.len() != len
        || arrays.center_z.len() != len
        || arrays.radius.len() != len
        || arrays.min_x.len() != len
        || arrays.min_y.len() != len
        || arrays.min_z.len() != len
        || arrays.max_x.len() != len
        || arrays.max_y.len() != len
        || arrays.max_z.len() != len
    {
        return Err(OreFeatureLoopError::LengthMismatch);
    }

    let mut checksum = 0u64;
    let width_height = config.width.wrapping_mul(config.height);

    for index in 0..len {
        let center_x = arrays.center_x[index];
        let center_y = arrays.center_y[index];
        let center_z = arrays.center_z[index];
        let radius = arrays.radius[index];

        let min_x = arrays.min_x[index];
        let min_y = arrays.min_y[index];
        let min_z = arrays.min_z[index];
        let max_x = arrays.max_x[index];
        let max_y = arrays.max_y[index];
        let max_z = arrays.max_z[index];

        for x in min_x..=max_x {
            let d5 = (x as f64 + 0.5 - center_x) / radius;
            let d5_squared = d5 * d5;
            if d5_squared < 1.0 {
                for y in min_y..=max_y {
                    let d6 = (y as f64 + 0.5 - center_y) / radius;
                    let d5d6_squared = if optimized {
                        d5_squared + d6 * d6
                    } else {
                        d5 * d5 + d6 * d6
                    };
                    if d5d6_squared < 1.0 {
                        for z in min_z..=max_z {
                            let d7 = (z as f64 + 0.5 - center_z) / radius;
                            if d5d6_squared + d7 * d7 < 1.0 && !is_outside_build_height(y) {
                                let position = x
                                    .wrapping_sub(config.origin_x)
                                    .wrapping_add((y.wrapping_sub(config.origin_y)).wrapping_mul(config.width))
                                    .wrapping_add(
                                        (z.wrapping_sub(config.origin_z))
                                            .wrapping_mul(width_height),
                                    );
                                checksum = mix(checksum, position);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(OreFeatureLoopSummary { checksum })
}

#[inline]
fn is_outside_build_height(y: i32) -> bool {
    y < -64 || y >= 320
}

#[inline]
fn mix(checksum: u64, value: i32) -> u64 {
    checksum
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (value as u32 as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimized_matches_old_on_simple_blob() {
        let arrays = OreFeatureLoopArrays {
            center_x: &[0.5],
            center_y: &[0.5],
            center_z: &[0.5],
            radius: &[1.2],
            min_x: &[0],
            min_y: &[0],
            min_z: &[0],
            max_x: &[1],
            max_y: &[1],
            max_z: &[1],
        };
        let config = OreFeatureLoopConfig {
            width: 18,
            height: 10,
            origin_x: -7,
            origin_y: -5,
            origin_z: -7,
        };
        assert_eq!(
            old_loop_summary(arrays, config).unwrap(),
            optimized_loop_summary(arrays, config).unwrap()
        );
    }
}
