pub const SUMMARY_FIELDS: usize = 4;

const FALLBACK_SEED_LO: i64 = -7046029254386353131;
const FALLBACK_SEED_HI: i64 = 7640891576956012809;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AquiferPositionalLocationSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub value_checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy, Debug)]
struct Xoroshiro128PlusPlus {
    seed_lo: u64,
    seed_hi: u64,
}

impl Xoroshiro128PlusPlus {
    fn new(seed_lo: i64, seed_hi: i64) -> Self {
        Self {
            seed_lo: seed_lo as u64,
            seed_hi: seed_hi as u64,
        }
    }

    fn next_long(&mut self) -> u64 {
        let seed_lo = self.seed_lo;
        let seed_hi = self.seed_hi;
        let result = seed_lo.wrapping_add(seed_hi).rotate_left(17).wrapping_add(seed_lo);
        let mixed = seed_hi ^ seed_lo;
        self.seed_lo = seed_lo.rotate_left(49) ^ mixed ^ (mixed << 21);
        self.seed_hi = mixed.rotate_left(28);
        result
    }
}

#[derive(Clone, Copy, Debug)]
struct XoroshiroRandomSource {
    random_number_generator: Xoroshiro128PlusPlus,
}

impl XoroshiroRandomSource {
    fn new(seed_lo: i64, seed_hi: i64) -> Self {
        Self {
            random_number_generator: Xoroshiro128PlusPlus::new(seed_lo, seed_hi),
        }
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        next_bounded_int_with(|| self.random_number_generator.next_long(), bound)
    }
}

pub fn old_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
) -> AquiferPositionalLocationSummary {
    run_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt, |x, y, z| {
        old_aquifer_location(seed_lo_salt, seed_hi_salt, x, y, z)
    })
}

pub fn direct_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
) -> AquiferPositionalLocationSummary {
    run_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt, |x, y, z| {
        direct_aquifer_location(seed_lo_salt, seed_hi_salt, x, y, z)
    })
}

fn run_batch_summary<F>(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
    mut sample: F,
) -> AquiferPositionalLocationSummary
where
    F: FnMut(i32, i32, i32) -> i64,
{
    debug_assert_eq!(xs.len(), ys.len());
    debug_assert_eq!(xs.len(), zs.len());

    let iterations = xs.len();
    let mut sum = 0i64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for i in 0..iterations {
        let value = sample(xs[i], ys[i], zs[i]);
        sum = sum.wrapping_add(value);
        last_bits = value as u64;
        checksum = mix64(
            checksum
                ^ last_bits
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(13))
                ^ (seed_lo_salt as u64)
                ^ (seed_hi_salt as u64),
        );
    }

    AquiferPositionalLocationSummary {
        count: iterations as u64,
        sum_bits: sum as u64,
        value_checksum: checksum,
        last_bits,
    }
}

fn old_aquifer_location(seed_lo_salt: i64, seed_hi_salt: i64, x: i32, y: i32, z: i32) -> i64 {
    let (seed_lo, seed_hi) = positional_seed(seed_lo_salt, seed_hi_salt, x, y, z);
    let mut random_source = XoroshiroRandomSource::new(seed_lo, seed_hi);
    block_pos_as_long(
        from_grid_x(x, random_source.next_int(10)),
        from_grid_y(y, random_source.next_int(9)),
        from_grid_z(z, random_source.next_int(10)),
    )
}

fn direct_aquifer_location(seed_lo_salt: i64, seed_hi_salt: i64, x: i32, y: i32, z: i32) -> i64 {
    let (seed_lo, seed_hi) = positional_seed(seed_lo_salt, seed_hi_salt, x, y, z);
    let mut state = Xoroshiro128PlusPlus::new(seed_lo, seed_hi);
    let x_offset = next_bounded_int_with(|| state.next_long(), 10);
    let y_offset = next_bounded_int_with(|| state.next_long(), 9);
    let z_offset = next_bounded_int_with(|| state.next_long(), 10);
    block_pos_as_long(
        from_grid_x(x, x_offset),
        from_grid_y(y, y_offset),
        from_grid_z(z, z_offset),
    )
}

#[inline]
fn positional_seed(seed_lo_salt: i64, seed_hi_salt: i64, x: i32, y: i32, z: i32) -> (i64, i64) {
    let mut seed_lo = mth_get_seed(x, y, z) ^ seed_lo_salt;
    let mut seed_hi = seed_hi_salt;
    if (seed_lo | seed_hi) == 0 {
        seed_lo = FALLBACK_SEED_LO;
        seed_hi = FALLBACK_SEED_HI;
    }
    (seed_lo, seed_hi)
}

#[inline]
fn next_bounded_int_with<F>(mut next_long: F, bound: i32) -> i32
where
    F: FnMut() -> u64,
{
    debug_assert!(bound > 0);
    let bound_u64 = bound as u32 as u64;
    let mut product = (next_long() as u32 as u64).wrapping_mul(bound_u64);
    let mut low = product & 0xFFFF_FFFF;
    if low < bound_u64 {
        let threshold = (0u32.wrapping_sub(bound as u32) % (bound as u32)) as u64;
        while low < threshold {
            product = (next_long() as u32 as u64).wrapping_mul(bound_u64);
            low = product & 0xFFFF_FFFF;
        }
    }
    (product >> 32) as i32
}

#[inline]
fn mth_get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut value = (x.wrapping_mul(3_129_871) as i64) ^ (z as i64).wrapping_mul(116_129_781) ^ (y as i64);
    value = value
        .wrapping_mul(value)
        .wrapping_mul(42_317_861)
        .wrapping_add(value.wrapping_mul(11));
    value >> 16
}

#[inline]
fn block_pos_as_long(x: i32, y: i32, z: i32) -> i64 {
    (((x as i64) & 67_108_863) << 38) | (((y as i64) & 4095)) | (((z as i64) & 67_108_863) << 12)
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

    const PRIMARY_SEED_LO: i64 = 0x1234_5678_9ABC_DEF0u64 as i64;
    const PRIMARY_SEED_HI: i64 = 0x0FED_CBA9_8765_4321u64 as i64;

    #[test]
    fn old_and_direct_locations_match() {
        for i in 0..10_000i32 {
            let x = i.wrapping_mul(17).rem_euclid(2_000_001) - 1_000_000;
            let y = i.wrapping_mul(7).rem_euclid(64) - 16;
            let z = i.wrapping_mul(31).rem_euclid(2_000_001) - 1_000_000;
            assert_eq!(
                old_aquifer_location(PRIMARY_SEED_LO, PRIMARY_SEED_HI, x, y, z),
                direct_aquifer_location(PRIMARY_SEED_LO, PRIMARY_SEED_HI, x, y, z)
            );
        }
    }

    #[test]
    fn zero_seed_fallback_matches_old_path() {
        assert_eq!(
            old_aquifer_location(0, 0, 0, 0, 0),
            direct_aquifer_location(0, 0, 0, 0, 0)
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let xs = [0, 1, -1, 12345, -54321, 999_999];
        let ys = [0, 7, -16, 31, 47, 12];
        let zs = [0, -1, 1, -12345, 54321, -999_999];
        let first = old_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);
        let second = old_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);
        let third = direct_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);
        assert_eq!(first, second);
        assert_eq!(first, third);
        assert_eq!(first.count, xs.len() as u64);
        assert_ne!(first.value_checksum, 0);
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            old_batch_summary(&[], &[], &[], PRIMARY_SEED_LO, PRIMARY_SEED_HI),
            AquiferPositionalLocationSummary::default()
        );
        assert_eq!(
            direct_batch_summary(&[], &[], &[], PRIMARY_SEED_LO, PRIMARY_SEED_HI),
            AquiferPositionalLocationSummary::default()
        );
    }
}
