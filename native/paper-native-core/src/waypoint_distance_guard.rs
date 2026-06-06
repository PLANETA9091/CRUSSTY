pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const RANGE_TAG: u64 = 0xD0F1_3A22_7C54_6B91;
const REALLY_FAR_DISTANCE_BITS: u32 = 0x43A6_0000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaypointDistanceGuardSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_at_or_beyond_range_summary(
    iterations: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
    range: &[f64],
) -> WaypointDistanceGuardSummary {
    run_summary(
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        range,
        Mode::OldAtOrBeyondRange,
    )
}

pub fn guarded_at_or_beyond_range_summary(
    iterations: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
    range: &[f64],
) -> WaypointDistanceGuardSummary {
    run_summary(
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        range,
        Mode::GuardedAtOrBeyondRange,
    )
}

pub fn old_really_far_summary(
    iterations: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
) -> WaypointDistanceGuardSummary {
    run_summary(
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        &[],
        Mode::OldReallyFar,
    )
}

pub fn guarded_really_far_summary(
    iterations: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
) -> WaypointDistanceGuardSummary {
    run_summary(
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        &[],
        Mode::GuardedReallyFar,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldAtOrBeyondRange,
    GuardedAtOrBeyondRange,
    OldReallyFar,
    GuardedReallyFar,
}

fn run_summary(
    iterations: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
    range: &[f64],
    mode: Mode,
) -> WaypointDistanceGuardSummary {
    if iterations == 0 {
        return WaypointDistanceGuardSummary::default();
    }

    let len = source_x.len();
    debug_assert_eq!(source_y.len(), len);
    debug_assert_eq!(source_z.len(), len);
    debug_assert_eq!(receiver_x.len(), len);
    debug_assert_eq!(receiver_y.len(), len);
    debug_assert_eq!(receiver_z.len(), len);
    if matches!(mode, Mode::OldAtOrBeyondRange | Mode::GuardedAtOrBeyondRange) {
        debug_assert_eq!(range.len(), len);
    }
    debug_assert!(len.is_power_of_two());

    let shape_digest = input_digest(
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        range,
    );

    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let index = match mode {
            Mode::OldAtOrBeyondRange | Mode::GuardedAtOrBeyondRange => (iteration * 31) & (len - 1),
            Mode::OldReallyFar | Mode::GuardedReallyFar => (iteration * 43) & (len - 1),
        };
        let value = match mode {
            Mode::OldAtOrBeyondRange => old_at_or_beyond_range_once(
                index,
                source_x,
                source_y,
                source_z,
                receiver_x,
                receiver_y,
                receiver_z,
                range,
            ),
            Mode::GuardedAtOrBeyondRange => guarded_at_or_beyond_range_once(
                index,
                source_x,
                source_y,
                source_z,
                receiver_x,
                receiver_y,
                receiver_z,
                range,
            ),
            Mode::OldReallyFar => {
                old_really_far_once(index, source_x, source_y, source_z, receiver_x, receiver_y, receiver_z)
            }
            Mode::GuardedReallyFar => {
                guarded_really_far_once(index, source_x, source_y, source_z, receiver_x, receiver_y, receiver_z)
            }
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((len as u64) << 11),
        );
    }

    WaypointDistanceGuardSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn old_at_or_beyond_range_once(
    index: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
    range: &[f64],
) -> bool {
    let dx = (source_x[index] - receiver_x[index]) as f32;
    let dy = (source_y[index] - receiver_y[index]) as f32;
    let dz = (source_z[index] - receiver_z[index]) as f32;
    let range = range[index] as f32;
    ((dx * dx + dy * dy + dz * dz).sqrt()) >= range
}

fn guarded_at_or_beyond_range_once(
    index: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
    range: &[f64],
) -> bool {
    let dx = (source_x[index] - receiver_x[index]) as f32;
    let dy = (source_y[index] - receiver_y[index]) as f32;
    let dz = (source_z[index] - receiver_z[index]) as f32;
    let range = range[index] as f32;
    let abs_dx = dx.abs();
    let abs_dy = dy.abs();
    let abs_dz = dz.abs();
    if abs_dx >= range || abs_dy >= range || abs_dz >= range {
        return true;
    }
    let half_range = range * 0.5;
    if abs_dx < half_range && abs_dy < half_range && abs_dz < half_range {
        return false;
    }
    ((dx * dx + dy * dy + dz * dz).sqrt()) >= range
}

fn old_really_far_once(
    index: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
) -> bool {
    let dx = (source_x[index] - receiver_x[index]) as f32;
    let dy = (source_y[index] - receiver_y[index]) as f32;
    let dz = (source_z[index] - receiver_z[index]) as f32;
    ((dx * dx + dy * dy + dz * dz).sqrt()) > f32::from_bits(REALLY_FAR_DISTANCE_BITS)
}

fn guarded_really_far_once(
    index: usize,
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
) -> bool {
    let dx = (source_x[index] - receiver_x[index]) as f32;
    let dy = (source_y[index] - receiver_y[index]) as f32;
    let dz = (source_z[index] - receiver_z[index]) as f32;
    let abs_dx = dx.abs();
    let abs_dy = dy.abs();
    let abs_dz = dz.abs();
    let really_far = f32::from_bits(REALLY_FAR_DISTANCE_BITS);
    if abs_dx > really_far || abs_dy > really_far || abs_dz > really_far {
        return true;
    }
    let half_range = really_far * 0.5;
    if abs_dx < half_range && abs_dy < half_range && abs_dz < half_range {
        return false;
    }
    ((dx * dx + dy * dy + dz * dz).sqrt()) > really_far
}

fn input_digest(
    source_x: &[f64],
    source_y: &[f64],
    source_z: &[f64],
    receiver_x: &[f64],
    receiver_y: &[f64],
    receiver_z: &[f64],
    range: &[f64],
) -> u64 {
    mix64(
        RANGE_TAG
            ^ floats_digest(source_x, 0x1656_67B1_9E37_79F9)
            ^ floats_digest(source_y, 0x85EB_CA77_C2B2_AE63)
            ^ floats_digest(source_z, 0x27D4_EB2F_1656_67C5)
            ^ floats_digest(receiver_x, 0x94D0_49BB_1331_11EB)
            ^ floats_digest(receiver_y, 0xD6E8_FD93_59A1_2B4D)
            ^ floats_digest(receiver_z, 0xC2B2_AE3D_27D4_EB4F)
            ^ floats_digest(range, 0x9E37_79B9_7F4A_7C15),
    )
}

fn floats_digest(values: &[f64], tag: u64) -> u64 {
    let mut digest = mix64(tag ^ (values.len() as u64));
    for (index, value) in values.iter().enumerate() {
        digest = mix64(
            digest
                ^ value.to_bits()
                ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
        );
    }
    digest
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
    fn old_and_guarded_match_on_regular_inputs() {
        let size = 8usize;
        let mut source_x = vec![0.0; size];
        let mut source_y = vec![0.0; size];
        let mut source_z = vec![0.0; size];
        let mut receiver_x = vec![0.0; size];
        let mut receiver_y = vec![0.0; size];
        let mut receiver_z = vec![0.0; size];
        let mut range = vec![16.0; size];
        for i in 0..size {
            source_x[i] = i as f64;
            source_y[i] = (i * 2) as f64;
            source_z[i] = (i * 3) as f64;
            receiver_x[i] = source_x[i] + 8.0;
            receiver_y[i] = source_y[i] - 2.0;
            receiver_z[i] = source_z[i] + 1.0;
            range[i] = 16.0 + i as f64;
        }
        assert_eq!(
            old_at_or_beyond_range_summary(64, &source_x, &source_y, &source_z, &receiver_x, &receiver_y, &receiver_z, &range),
            guarded_at_or_beyond_range_summary(64, &source_x, &source_y, &source_z, &receiver_x, &receiver_y, &receiver_z, &range)
        );
        assert_eq!(
            old_really_far_summary(64, &source_x, &source_y, &source_z, &receiver_x, &receiver_y, &receiver_z),
            guarded_really_far_summary(64, &source_x, &source_y, &source_z, &receiver_x, &receiver_y, &receiver_z)
        );
    }
}
