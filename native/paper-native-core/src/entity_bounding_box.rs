pub const SUMMARY_FIELDS: usize = 4;

const BOX_SINK_SIZE: usize = 256;
const OLD_STEP: usize = 31;
const DIRECT_STEP: usize = 31;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EntityBoundingBoxSummary {
    pub count: u64,
    pub value_bits: u64,
    pub checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Box3 {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityBoundingBoxError {
    InvalidInputLength,
    InvalidShape,
}

pub fn old_make_then_set_summary(
    iterations: usize,
    widths: &[f32],
    heights: &[f32],
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
) -> Result<EntityBoundingBoxSummary, EntityBoundingBoxError> {
    run_summary(
        iterations,
        widths,
        heights,
        xs,
        ys,
        zs,
        false,
        OLD_STEP,
    )
}

pub fn direct_dimensions_set_summary(
    iterations: usize,
    widths: &[f32],
    heights: &[f32],
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
) -> Result<EntityBoundingBoxSummary, EntityBoundingBoxError> {
    run_summary(
        iterations,
        widths,
        heights,
        xs,
        ys,
        zs,
        true,
        DIRECT_STEP,
    )
}

fn run_summary(
    iterations: usize,
    widths: &[f32],
    heights: &[f32],
    xs: &[f64],
    ys: &[f64],
    zs: &[f64],
    direct: bool,
    step: usize,
) -> Result<EntityBoundingBoxSummary, EntityBoundingBoxError> {
    if iterations == 0 {
        return Ok(EntityBoundingBoxSummary::default());
    }

    let len = widths.len();
    if len == 0
        || len != heights.len()
        || len != xs.len()
        || len != ys.len()
        || len != zs.len()
    {
        return Err(EntityBoundingBoxError::InvalidInputLength);
    }
    if !len.is_power_of_two() {
        return Err(EntityBoundingBoxError::InvalidShape);
    }

    let mask = len - 1;
    let mut box_sink = [Box3::default(); BOX_SINK_SIZE];
    let mut input_sink = Box3::default();
    let mut value = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for iteration in 0..iterations {
        let index = iteration.wrapping_mul(step) & mask;
        let input = make_bounding_box(widths[index], heights[index], xs[index], ys[index], zs[index]);
        if !direct {
            input_sink = input;
        }
        let box3 = if direct {
            direct_set_bounding_box(widths[index], heights[index], xs[index], ys[index], zs[index])
        } else {
            old_set_bounding_box(input)
        };
        box_sink[iteration & (BOX_SINK_SIZE - 1)] = box3;
        value += box3.min_x + box3.max_y * 0.25 + box3.max_z * 0.125;
        last_bits = box3.max_z.to_bits();
        checksum = mix64(
            checksum
                ^ box3.min_x.to_bits()
                ^ box3.min_y.to_bits().rotate_left(7)
                ^ box3.max_x.to_bits().rotate_left(13)
                ^ box3.max_y.to_bits().rotate_left(23)
                ^ box3.max_z.to_bits().rotate_left(31)
                ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
        );
    }

    value += sink_boxes(&box_sink);
    if !direct {
        value += input_sink.min_y;
    }

    Ok(EntityBoundingBoxSummary {
        count: iterations as u64,
        value_bits: value.to_bits(),
        checksum,
        last_bits,
    })
}

fn sink_boxes(box_sink: &[Box3; BOX_SINK_SIZE]) -> f64 {
    let mut value = 0.0f64;
    for box3 in box_sink {
        value += box3.min_y + box3.max_x * 0.03125;
    }
    value
}

fn old_set_bounding_box(box3: Box3) -> Box3 {
    set_bounding_box(box3.min_x, box3.min_y, box3.min_z, box3.max_x, box3.max_y, box3.max_z)
}

fn direct_set_bounding_box(width: f32, height: f32, x: f64, y: f64, z: f64) -> Box3 {
    let half_width = width * 0.5;
    set_bounding_box(
        x - half_width as f64,
        y,
        z - half_width as f64,
        x + half_width as f64,
        y + height as f64,
        z + half_width as f64,
    )
}

fn make_bounding_box(width: f32, height: f32, x: f64, y: f64, z: f64) -> Box3 {
    let half_width = width * 0.5;
    Box3::new(
        x - half_width as f64,
        y,
        z - half_width as f64,
        x + half_width as f64,
        y + height as f64,
        z + half_width as f64,
    )
}

fn set_bounding_box(
    min_x: f64,
    min_y: f64,
    min_z: f64,
    mut max_x: f64,
    mut max_y: f64,
    mut max_z: f64,
) -> Box3 {
    let mut len = max_x - min_x;
    if len < 0.0 {
        max_x = min_x;
    }
    if len > 64.0 {
        max_x = min_x + 64.0;
    }

    len = max_y - min_y;
    if len < 0.0 {
        max_y = min_y;
    }
    if len > 64.0 {
        max_y = min_y + 64.0;
    }

    len = max_z - min_z;
    if len < 0.0 {
        max_z = min_z;
    }
    if len > 64.0 {
        max_z = min_z + 64.0;
    }

    Box3::new(min_x, min_y, min_z, max_x, max_y, max_z)
}

impl Box3 {
    fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x: java_min(min_x, max_x),
            min_y: java_min(min_y, max_y),
            min_z: java_min(min_z, max_z),
            max_x: java_max(min_x, max_x),
            max_y: java_max(min_y, max_y),
            max_z: java_max(min_z, max_z),
        }
    }
}

#[inline]
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else if a <= b {
        a
    } else {
        b
    }
}

#[inline]
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else if a >= b {
        a
    } else {
        b
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
    fn old_and_direct_summaries_match_on_regular_inputs() {
        let widths = [0.6f32, 0.98, 1.4, 2.0, 4.0, 16.0, 80.0, 0.25];
        let heights = [1.8f32, 0.9, 2.9, 4.0, 64.0, 70.0, 0.5, 1.0];
        let xs = [0.0, 1.0, -1.0, 3.5, -7.25, 9.0, 12.0, -14.0];
        let ys = [64.0, 65.0, 66.0, -8.0, 0.0, 32.0, 48.0, 80.0];
        let zs = [0.0, -1.0, 2.0, -3.5, 7.25, -9.0, 12.0, 14.0];

        let old = old_make_then_set_summary(12_000, &widths, &heights, &xs, &ys, &zs).unwrap();
        let direct = direct_dimensions_set_summary(12_000, &widths, &heights, &xs, &ys, &zs).unwrap();

        assert_eq!(old.count, 12_000);
        assert_eq!(direct.count, 12_000);
        assert_ne!(old.value_bits, 0);
        assert_ne!(direct.value_bits, 0);
    }

    #[test]
    fn rejects_bad_shapes() {
        let widths = [1.0f32, 2.0];
        let heights = [1.0f32];
        let xs = [0.0, 1.0];
        let ys = [0.0, 1.0];
        let zs = [0.0, 1.0];

        assert_eq!(
            old_make_then_set_summary(1, &widths, &heights, &xs, &ys, &zs),
            Err(EntityBoundingBoxError::InvalidInputLength)
        );
        assert_eq!(
            direct_dimensions_set_summary(1, &widths, &heights, &xs, &ys, &zs),
            Err(EntityBoundingBoxError::InvalidInputLength)
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            old_make_then_set_summary(0, &[], &[], &[], &[], &[]).unwrap(),
            EntityBoundingBoxSummary::default()
        );
        assert_eq!(
            direct_dimensions_set_summary(0, &[], &[], &[], &[], &[]).unwrap(),
            EntityBoundingBoxSummary::default()
        );
    }
}
