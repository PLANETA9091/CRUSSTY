pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerEntityDeltaIdentitySummary {
    pub count: u64,
    pub sends: u64,
    pub checksum: u64,
    pub last_distance_bits: u64,
}

pub fn old_distance_summary(
    iterations: usize,
    current_x: &[f64],
    current_y: &[f64],
    current_z: &[f64],
    last_x: &[f64],
    last_y: &[f64],
    last_z: &[f64],
) -> ServerEntityDeltaIdentitySummary {
    run_summary(
        iterations, None, current_x, current_y, current_z, last_x, last_y, last_z,
    )
}

pub fn identity_guard_summary(
    iterations: usize,
    same_identity: &[i8],
    current_x: &[f64],
    current_y: &[f64],
    current_z: &[f64],
    last_x: &[f64],
    last_y: &[f64],
    last_z: &[f64],
) -> ServerEntityDeltaIdentitySummary {
    run_summary(
        iterations,
        Some(same_identity),
        current_x,
        current_y,
        current_z,
        last_x,
        last_y,
        last_z,
    )
}

fn run_summary(
    iterations: usize,
    same_identity: Option<&[i8]>,
    current_x: &[f64],
    current_y: &[f64],
    current_z: &[f64],
    last_x: &[f64],
    last_y: &[f64],
    last_z: &[f64],
) -> ServerEntityDeltaIdentitySummary {
    if iterations == 0 || current_x.is_empty() {
        return ServerEntityDeltaIdentitySummary::default();
    }
    debug_assert!(current_x.len().is_power_of_two());
    debug_assert_eq!(current_y.len(), current_x.len());
    debug_assert_eq!(current_z.len(), current_x.len());
    debug_assert_eq!(last_x.len(), current_x.len());
    debug_assert_eq!(last_y.len(), current_x.len());
    debug_assert_eq!(last_z.len(), current_x.len());
    if let Some(same_identity) = same_identity {
        debug_assert_eq!(same_identity.len(), current_x.len());
    }

    let mask = current_x.len() - 1;
    let mut sends = 0u64;
    let mut checksum = 0u64;
    let mut last_distance_bits = 0u64;

    for iteration in 0..iterations {
        let index = iteration.wrapping_mul(31) & mask;
        let same = same_identity.map_or(false, |values| values[index] != 0);
        let (send, distance_bits) = if same {
            (false, 0)
        } else {
            let distance = distance_sqr(
                current_x[index],
                current_y[index],
                current_z[index],
                last_x[index],
                last_y[index],
                last_z[index],
            );
            (
                should_send_motion(distance, current_x[index], current_y[index], current_z[index]),
                distance.to_bits(),
            )
        };

        if send {
            sends += 1;
        }
        last_distance_bits = distance_bits;
        checksum = mix64(
            checksum
                ^ distance_bits.rotate_left((iteration & 63) as u32)
                ^ ((send as u64) << 47)
                ^ ((index as u64) << 1)
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64).rotate_left(17)),
        );
    }

    ServerEntityDeltaIdentitySummary {
        count: iterations as u64,
        sends,
        checksum,
        last_distance_bits,
    }
}

#[inline]
fn should_send_motion(distance_sqr: f64, current_x: f64, current_y: f64, current_z: f64) -> bool {
    distance_sqr > 1.0E-7
        || (distance_sqr > 0.0 && length_sqr(current_x, current_y, current_z) == 0.0)
}

#[inline]
fn distance_sqr(
    current_x: f64,
    current_y: f64,
    current_z: f64,
    last_x: f64,
    last_y: f64,
    last_z: f64,
) -> f64 {
    let dx = current_x - last_x;
    let dy = current_y - last_y;
    let dz = current_z - last_z;
    dx * dx + dy * dy + dz * dz
}

#[inline]
fn length_sqr(x: f64, y: f64, z: f64) -> f64 {
    x * x + y * y + z * z
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
    fn old_and_identity_guard_match_on_regular_inputs() {
        let samples = build_samples(1 << 8);
        let old = old_distance_summary(
            16_384,
            &samples.current_x,
            &samples.current_y,
            &samples.current_z,
            &samples.last_x,
            &samples.last_y,
            &samples.last_z,
        );
        let guarded = identity_guard_summary(
            16_384,
            &samples.same_identity,
            &samples.current_x,
            &samples.current_y,
            &samples.current_z,
            &samples.last_x,
            &samples.last_y,
            &samples.last_z,
        );

        assert_eq!(old, guarded);
        assert_eq!(old.count, 16_384);
        assert_eq!(old.sends, 4_096);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let samples = build_samples(1 << 4);
        let summary = old_distance_summary(
            0,
            &samples.current_x,
            &samples.current_y,
            &samples.current_z,
            &samples.last_x,
            &samples.last_y,
            &samples.last_z,
        );

        assert_eq!(summary, ServerEntityDeltaIdentitySummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let samples = build_samples(1 << 6);
        let first = identity_guard_summary(
            4_096,
            &samples.same_identity,
            &samples.current_x,
            &samples.current_y,
            &samples.current_z,
            &samples.last_x,
            &samples.last_y,
            &samples.last_z,
        );
        let second = identity_guard_summary(
            4_096,
            &samples.same_identity,
            &samples.current_x,
            &samples.current_y,
            &samples.current_z,
            &samples.last_x,
            &samples.last_y,
            &samples.last_z,
        );

        assert_eq!(first, second);
    }

    #[test]
    fn tiny_stop_delta_from_zero_current_still_sends() {
        let current_x = [0.0, 1.0];
        let current_y = [0.0, 0.0];
        let current_z = [0.0, 0.0];
        let last_x = [0.00001, 1.0];
        let last_y = [0.0, 0.0];
        let last_z = [0.0, 0.0];
        let same_identity = [0, 1];

        let old = old_distance_summary(
            1, &current_x, &current_y, &current_z, &last_x, &last_y, &last_z,
        );
        let guarded = identity_guard_summary(
            1,
            &same_identity,
            &current_x,
            &current_y,
            &current_z,
            &last_x,
            &last_y,
            &last_z,
        );

        assert_eq!(old, guarded);
        assert_eq!(old.sends, 1);
    }

    struct Samples {
        same_identity: Vec<i8>,
        current_x: Vec<f64>,
        current_y: Vec<f64>,
        current_z: Vec<f64>,
        last_x: Vec<f64>,
        last_y: Vec<f64>,
        last_z: Vec<f64>,
    }

    fn build_samples(size: usize) -> Samples {
        let mut same_identity = Vec::with_capacity(size);
        let mut current_x = Vec::with_capacity(size);
        let mut current_y = Vec::with_capacity(size);
        let mut current_z = Vec::with_capacity(size);
        let mut last_x = Vec::with_capacity(size);
        let mut last_y = Vec::with_capacity(size);
        let mut last_z = Vec::with_capacity(size);

        for i in 0..size {
            let lx = ((i & 255) as f64) * 0.01;
            let ly = (((i >> 8) & 63) as f64) * 0.02;
            let lz = (((i >> 4) & 127) as f64) * -0.015;
            last_x.push(lx);
            last_y.push(ly);
            last_z.push(lz);
            if (i & 3) == 0 {
                current_x.push(lx + 0.001);
                current_y.push(ly);
                current_z.push(lz - 0.001);
                same_identity.push(0);
            } else {
                current_x.push(lx);
                current_y.push(ly);
                current_z.push(lz);
                same_identity.push(1);
            }
        }

        Samples {
            same_identity,
            current_x,
            current_y,
            current_z,
            last_x,
            last_y,
            last_z,
        }
    }
}
