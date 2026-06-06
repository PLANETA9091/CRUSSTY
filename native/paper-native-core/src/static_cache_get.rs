pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StaticCacheGetSummary {
    pub count: u64,
    pub sum: i64,
    pub checksum: u64,
    pub last_value: i64,
}

pub fn old_batch_summary(
    iterations: usize,
    min_x: i32,
    min_z: i32,
    size_x: usize,
    size_z: usize,
    values: &[i32],
) -> StaticCacheGetSummary {
    run_batch_summary(
        iterations,
        min_x,
        min_z,
        size_x,
        size_z,
        values,
        old_get,
    )
}

pub fn new_batch_summary(
    iterations: usize,
    min_x: i32,
    min_z: i32,
    size_x: usize,
    size_z: usize,
    values: &[i32],
) -> StaticCacheGetSummary {
    run_batch_summary(
        iterations,
        min_x,
        min_z,
        size_x,
        size_z,
        values,
        new_get,
    )
}

fn run_batch_summary<F>(
    iterations: usize,
    min_x: i32,
    min_z: i32,
    size_x: usize,
    size_z: usize,
    values: &[i32],
    mut get_value: F,
) -> StaticCacheGetSummary
where
    F: FnMut(i32, i32, i32, i32, usize, usize, &[i32]) -> i32,
{
    if iterations == 0 || values.is_empty() || size_x == 0 || size_z == 0 {
        return StaticCacheGetSummary::default();
    }
    debug_assert_eq!(values.len(), size_x * size_z);

    let size_i32 = size_x as i32;
    let mut sum = 0i64;
    let mut checksum = 0u64;
    let mut last_value = 0i64;

    for iteration in 0..iterations {
        let x = min_x + ((iteration as i32).wrapping_mul(17).rem_euclid(size_i32));
        let z = min_z + ((iteration as i32).wrapping_mul(31).rem_euclid(size_i32));
        let value = get_value(x, z, min_x, min_z, size_x, size_z, values);
        let value_i64 = i64::from(value);
        sum = sum.wrapping_add(value_i64);
        last_value = value_i64;
        checksum = mix64(
            checksum
                ^ value_i64 as u64
                ^ ((x as i64 as u64) << 1)
                ^ ((z as i64 as u64) << 33)
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    StaticCacheGetSummary {
        count: iterations as u64,
        sum,
        checksum,
        last_value,
    }
}

fn old_get(
    x: i32,
    z: i32,
    min_x: i32,
    min_z: i32,
    size_x: usize,
    size_z: usize,
    values: &[i32],
) -> i32 {
    if !contains(x, z, min_x, min_z, size_x, size_z) {
        panic!("Requested out of range value ({x},{z})");
    }
    values[get_index(x, z, min_x, min_z, size_z)]
}

fn new_get(
    x: i32,
    z: i32,
    min_x: i32,
    min_z: i32,
    size_x: usize,
    size_z: usize,
    values: &[i32],
) -> i32 {
    let offset_x = x - min_x;
    let offset_z = z - min_z;
    if offset_x < 0 || offset_x >= size_x as i32 || offset_z < 0 || offset_z >= size_z as i32 {
        panic!("Requested out of range value ({x},{z})");
    }
    values[offset_x as usize * size_z + offset_z as usize]
}

#[inline]
fn contains(x: i32, z: i32, min_x: i32, min_z: i32, size_x: usize, size_z: usize) -> bool {
    let offset_x = x - min_x;
    let offset_z = z - min_z;
    offset_x >= 0 && offset_x < size_x as i32 && offset_z >= 0 && offset_z < size_z as i32
}

#[inline]
fn get_index(x: i32, z: i32, min_x: i32, min_z: i32, size_z: usize) -> usize {
    let offset_x = (x - min_x) as usize;
    let offset_z = (z - min_z) as usize;
    offset_x * size_z + offset_z
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
    fn old_and_new_match_on_regular_inputs() {
        let values = build_values(5);
        let old = old_batch_summary(1_024, -2, -2, 5, 5, &values);
        let new = new_batch_summary(1_024, -2, -2, 5, 5, &values);

        assert_eq!(old, new);
        assert_eq!(old.count, 1_024);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let values = build_values(3);
        let summary = old_batch_summary(0, -1, -1, 3, 3, &values);

        assert_eq!(summary, StaticCacheGetSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let values = build_values(7);
        let first = new_batch_summary(512, -3, -3, 7, 7, &values);
        let second = new_batch_summary(512, -3, -3, 7, 7, &values);

        assert_eq!(first, second);
    }

    #[test]
    fn out_of_range_panics_match_old_and_new() {
        let values = build_values(3);
        let old = std::panic::catch_unwind(|| old_get(100, 100, -1, -1, 3, 3, &values));
        let new = std::panic::catch_unwind(|| new_get(100, 100, -1, -1, 3, 3, &values));

        assert!(old.is_err());
        assert!(new.is_err());
    }

    fn build_values(size: usize) -> Vec<i32> {
        let mut values = Vec::with_capacity(size * size);
        let min = -(size as i32 / 2);
        for x in min..(min + size as i32) {
            for z in min..(min + size as i32) {
                values.push((x * 31) ^ z);
            }
        }
        values
    }
}
