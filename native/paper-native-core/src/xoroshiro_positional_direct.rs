pub const SUMMARY_FIELDS: usize = 4;

const FALLBACK_SEED_LO: i64 = -7046029254386353131;
const FALLBACK_SEED_HI: i64 = 7640891576956012809;
const FLOAT_UNIT: f32 = 5.9604645E-8f32;
const DOUBLE_UNIT: f32 = 1.110223E-16f32;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XoroshiroPositionalDirectSummary {
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

    fn next_float_bits(&mut self) -> u64 {
        let bits = (self.random_number_generator.next_long() >> 40) as f32;
        (bits * FLOAT_UNIT).to_bits() as u64
    }

    fn next_double_bits(&mut self) -> u64 {
        let bits = (self.random_number_generator.next_long() >> 11) as f32;
        ((bits * DOUBLE_UNIT) as f64).to_bits()
    }
}

pub fn old_float_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
) -> XoroshiroPositionalDirectSummary {
    run_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt, |x, y, z| {
        let (seed_lo, seed_hi) = positional_seed(seed_lo_salt, seed_hi_salt, x, y, z);
        let mut random_source = XoroshiroRandomSource::new(seed_lo, seed_hi);
        random_source.next_float_bits()
    })
}

pub fn direct_float_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
) -> XoroshiroPositionalDirectSummary {
    run_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt, |x, y, z| {
        direct_float_bits(seed_lo_salt, seed_hi_salt, x, y, z)
    })
}

pub fn old_double_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
) -> XoroshiroPositionalDirectSummary {
    run_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt, |x, y, z| {
        let (seed_lo, seed_hi) = positional_seed(seed_lo_salt, seed_hi_salt, x, y, z);
        let mut random_source = XoroshiroRandomSource::new(seed_lo, seed_hi);
        random_source.next_double_bits()
    })
}

pub fn direct_double_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
) -> XoroshiroPositionalDirectSummary {
    run_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt, |x, y, z| {
        direct_double_bits(seed_lo_salt, seed_hi_salt, x, y, z)
    })
}

fn run_batch_summary<F>(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    seed_lo_salt: i64,
    seed_hi_salt: i64,
    mut sample: F,
) -> XoroshiroPositionalDirectSummary
where
    F: FnMut(i32, i32, i32) -> u64,
{
    debug_assert_eq!(xs.len(), ys.len());
    debug_assert_eq!(xs.len(), zs.len());

    let iterations = xs.len();
    let mut sum = 0u64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;

    for i in 0..iterations {
        let value_bits = sample(xs[i], ys[i], zs[i]);
        sum = sum.wrapping_add(value_bits);
        last_bits = value_bits;
        checksum = mix64(
            checksum
                ^ value_bits
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((iterations as u64).rotate_left(13))
                ^ (seed_lo_salt as u64)
                ^ (seed_hi_salt as u64),
        );
    }

    XoroshiroPositionalDirectSummary {
        count: iterations as u64,
        sum_bits: sum,
        value_checksum: checksum,
        last_bits,
    }
}

fn direct_float_bits(seed_lo_salt: i64, seed_hi_salt: i64, x: i32, y: i32, z: i32) -> u64 {
    let (seed_lo, seed_hi) = positional_seed(seed_lo_salt, seed_hi_salt, x, y, z);
    let first_long = first_long(seed_lo, seed_hi);
    (((first_long >> 40) as f32) * FLOAT_UNIT).to_bits() as u64
}

fn direct_double_bits(seed_lo_salt: i64, seed_hi_salt: i64, x: i32, y: i32, z: i32) -> u64 {
    let (seed_lo, seed_hi) = positional_seed(seed_lo_salt, seed_hi_salt, x, y, z);
    let first_long = first_long(seed_lo, seed_hi);
    ((((first_long >> 11) as f32) * DOUBLE_UNIT) as f64).to_bits()
}

#[inline]
fn first_long(seed_lo: i64, seed_hi: i64) -> u64 {
    let seed_lo = seed_lo as u64;
    let seed_hi = seed_hi as u64;
    seed_lo.wrapping_add(seed_hi).rotate_left(17).wrapping_add(seed_lo)
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
fn mth_get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut value = (x.wrapping_mul(3_129_871) as i64) ^ (z as i64).wrapping_mul(116_129_781) ^ (y as i64);
    value = value
        .wrapping_mul(value)
        .wrapping_mul(42_317_861)
        .wrapping_add(value.wrapping_mul(11));
    value >> 16
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
    fn float_old_and_direct_match_for_random_inputs() {
        let xs = [0, 17, -233, 2048, -1048576];
        let ys = [0, -12, 7, 31, 63];
        let zs = [0, 9, -4096, 1024, -1];

        let old = old_float_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);
        let direct = direct_float_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);

        assert_eq!(old, direct);
        assert_eq!(old.count, xs.len() as u64);
        assert_ne!(old.value_checksum, 0);
    }

    #[test]
    fn double_old_and_direct_match_for_random_inputs() {
        let xs = [0, 17, -233, 2048, -1048576];
        let ys = [0, -12, 7, 31, 63];
        let zs = [0, 9, -4096, 1024, -1];

        let old = old_double_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);
        let direct = direct_double_batch_summary(&xs, &ys, &zs, PRIMARY_SEED_LO, PRIMARY_SEED_HI);

        assert_eq!(old, direct);
        assert_eq!(old.count, xs.len() as u64);
        assert_ne!(old.last_bits, 0);
    }

    #[test]
    fn zero_seed_fallback_matches_between_old_and_direct_paths() {
        let xs = [0];
        let ys = [0];
        let zs = [0];

        let old_float = old_float_batch_summary(&xs, &ys, &zs, 0, 0);
        let direct_float = direct_float_batch_summary(&xs, &ys, &zs, 0, 0);
        let old_double = old_double_batch_summary(&xs, &ys, &zs, 0, 0);
        let direct_double = direct_double_batch_summary(&xs, &ys, &zs, 0, 0);

        assert_eq!(old_float, direct_float);
        assert_eq!(old_double, direct_double);
    }
}
