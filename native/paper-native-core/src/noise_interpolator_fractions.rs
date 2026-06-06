pub const SUMMARY_FIELDS: usize = 3;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const DIVISION_MARKER: u64 = 0x4456_4953_494f_4e31;
const ARRAY_MARKER: u64 = 0x4152_5241_595f_4631;
const CELL_WIDTH: usize = 4;
const CELL_HEIGHT: usize = 8;

const NOISE_000: f64 = -0.37;
const NOISE_001: f64 = 0.12;
const NOISE_100: f64 = 0.44;
const NOISE_101: f64 = -0.73;
const NOISE_010: f64 = 0.21;
const NOISE_011: f64 = 0.58;
const NOISE_110: f64 = -0.19;
const NOISE_111: f64 = 0.86;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoiseInterpolatorFractionsSummary {
    pub count: u64,
    pub sink_bits: u64,
    pub checksum: u64,
}

pub fn division_summary(iterations: usize) -> NoiseInterpolatorFractionsSummary {
    if iterations == 0 {
        return NoiseInterpolatorFractionsSummary::default();
    }

    let mut sink = 0.0f64;
    let mut checksum = 0u64;
    for iteration in 0..iterations {
        let in_cell_x = iteration & (CELL_WIDTH - 1);
        let in_cell_y = (iteration >> 2) & (CELL_HEIGHT - 1);
        let in_cell_z = (iteration >> 5) & (CELL_WIDTH - 1);
        let value = compute_with_division(
            CELL_WIDTH,
            CELL_HEIGHT,
            in_cell_x,
            in_cell_y,
            in_cell_z,
        );
        sink += value;
        checksum = mix_summary(checksum, value, iteration, CELL_WIDTH, CELL_HEIGHT, DIVISION_MARKER);
    }

    NoiseInterpolatorFractionsSummary {
        count: iterations as u64,
        sink_bits: canonical_double_bits(sink),
        checksum,
    }
}

pub fn array_summary(
    iterations: usize,
    cell_width_fractions: &[f64],
    cell_height_fractions: &[f64],
) -> NoiseInterpolatorFractionsSummary {
    if iterations == 0 || cell_width_fractions.is_empty() || cell_height_fractions.is_empty() {
        return NoiseInterpolatorFractionsSummary::default();
    }

    let cell_width = cell_width_fractions.len() - 1;
    let cell_height = cell_height_fractions.len() - 1;

    let mut sink = 0.0f64;
    let mut checksum = 0u64;
    for iteration in 0..iterations {
        let in_cell_x = iteration & (CELL_WIDTH - 1);
        let in_cell_y = (iteration >> 2) & (CELL_HEIGHT - 1);
        let in_cell_z = (iteration >> 5) & (CELL_WIDTH - 1);
        let value = compute_with_arrays(
            cell_width_fractions,
            cell_height_fractions,
            in_cell_x,
            in_cell_y,
            in_cell_z,
        );
        sink += value;
        checksum = mix_summary(checksum, value, iteration, cell_width, cell_height, ARRAY_MARKER);
    }

    NoiseInterpolatorFractionsSummary {
        count: iterations as u64,
        sink_bits: canonical_double_bits(sink),
        checksum,
    }
}

pub fn compute_with_division(
    cell_width: usize,
    cell_height: usize,
    in_cell_x: usize,
    in_cell_y: usize,
    in_cell_z: usize,
) -> f64 {
    lerp3(
        in_cell_x as f64 / cell_width as f64,
        in_cell_y as f64 / cell_height as f64,
        in_cell_z as f64 / cell_width as f64,
        NOISE_000,
        NOISE_100,
        NOISE_010,
        NOISE_110,
        NOISE_001,
        NOISE_101,
        NOISE_011,
        NOISE_111,
    )
}

pub fn compute_with_arrays(
    cell_width_fractions: &[f64],
    cell_height_fractions: &[f64],
    in_cell_x: usize,
    in_cell_y: usize,
    in_cell_z: usize,
) -> f64 {
    lerp3(
        cell_width_fractions[in_cell_x],
        cell_height_fractions[in_cell_y],
        cell_width_fractions[in_cell_z],
        NOISE_000,
        NOISE_100,
        NOISE_010,
        NOISE_110,
        NOISE_001,
        NOISE_101,
        NOISE_011,
        NOISE_111,
    )
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
fn mix_summary(
    checksum: u64,
    value: f64,
    iteration: usize,
    cell_width: usize,
    cell_height: usize,
    marker: u64,
) -> u64 {
    mix64(
        checksum
            ^ canonical_double_bits(value)
            ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
            ^ ((cell_width as u64) << 17)
            ^ ((cell_height as u64) << 33)
            ^ marker,
    )
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

    #[test]
    fn division_and_array_match_on_regular_inputs() {
        assert_grid_matches(4, 8);
    }

    #[test]
    fn alternate_shape_matches_regular_inputs() {
        assert_grid_matches(3, 7);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let summary = division_summary(0);

        assert_eq!(summary, NoiseInterpolatorFractionsSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let cell_width_fractions = fractions(4);
        let cell_height_fractions = fractions(8);
        let first = array_summary(512, &cell_width_fractions, &cell_height_fractions);
        let second = array_summary(512, &cell_width_fractions, &cell_height_fractions);

        assert_eq!(first, second);
    }

    fn assert_grid_matches(cell_width: usize, cell_height: usize) {
        let cell_width_fractions = fractions(cell_width);
        let cell_height_fractions = fractions(cell_height);

        for x in 0..=cell_width {
            for y in 0..=cell_height {
                for z in 0..=cell_width {
                    let division = compute_with_division(cell_width, cell_height, x, y, z);
                    let array = compute_with_arrays(
                        &cell_width_fractions,
                        &cell_height_fractions,
                        x,
                        y,
                        z,
                    );

                    assert_eq!(division.to_bits(), array.to_bits());
                }
            }
        }
    }

    fn fractions(denominator: usize) -> Vec<f64> {
        let mut ret = Vec::with_capacity(denominator + 1);
        for i in 0..=denominator {
            ret.push(i as f64 / denominator as f64);
        }
        ret
    }
}
