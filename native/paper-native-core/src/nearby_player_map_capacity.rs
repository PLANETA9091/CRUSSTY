pub const SUMMARY_FIELDS: usize = 4;

const DEFAULT_TAG: u64 = 0xC5E5_0DA4_64BA_3F9D;
const PRESIZED_TAG: u64 = 0x91E1_49D6_4D0A_A21B;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NearbyPlayerMapCapacitySummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn default_capacity_summary(
    iterations: usize,
    player_count: usize,
) -> NearbyPlayerMapCapacitySummary {
    run_summary(iterations, player_count, Mode::Default)
}

pub fn presized_capacity_summary(
    iterations: usize,
    player_count: usize,
) -> NearbyPlayerMapCapacitySummary {
    run_summary(iterations, player_count, Mode::Presized)
}

#[derive(Clone, Copy)]
enum Mode {
    Default,
    Presized,
}

impl Mode {
    fn tag(self) -> u64 {
        match self {
            Mode::Default => DEFAULT_TAG,
            Mode::Presized => PRESIZED_TAG,
        }
    }
}

fn run_summary(
    iterations: usize,
    player_count: usize,
    mode: Mode,
) -> NearbyPlayerMapCapacitySummary {
    if iterations == 0 {
        return NearbyPlayerMapCapacitySummary::default();
    }

    let presized = matches!(mode, Mode::Presized);
    let shape_digest = mix64(mode.tag() ^ ((player_count as u64) << 17));
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = run_once(player_count, presized);
        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64) << 11),
        );
    }

    NearbyPlayerMapCapacitySummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn run_once(player_count: usize, presized: bool) -> u64 {
    let expected = if presized { player_count.max(16) } else { 16 };
    let map_rehashes = simulate_map_rehashes(expected, player_count);
    (map_rehashes * 2) as u64
}

fn simulate_map_rehashes(expected: usize, inserts: usize) -> usize {
    let mut size = 0usize;
    let mut capacity = array_size(expected, 0.75_f32);
    let mut fill_limit = max_fill(capacity, 0.75_f32);
    let mut rehashes = 0usize;

    for _ in 0..inserts {
        if size + 1 > fill_limit {
            capacity *= 2;
            fill_limit = max_fill(capacity, 0.75_f32);
            rehashes += 1;
        }
        size += 1;
    }

    rehashes
}

fn array_size(expected: usize, load_factor: f32) -> usize {
    let needed = ((expected as f64) / (load_factor as f64)).ceil() as usize;
    next_power_of_two(needed).max(2)
}

fn max_fill(capacity: usize, load_factor: f32) -> usize {
    (((capacity as f64) * (load_factor as f64)).ceil() as usize)
        .min(capacity.saturating_sub(1))
}

fn next_power_of_two(value: usize) -> usize {
    value.next_power_of_two()
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
    fn default_and_presized_match_known_rehash_counts() {
        let default = default_capacity_summary(8, 50);
        let presized = presized_capacity_summary(8, 50);
        assert_eq!(default.last_total, 4);
        assert_eq!(default.total, 32);
        assert_eq!(presized.last_total, 0);
        assert_eq!(presized.total, 0);
        assert_ne!(default.checksum, presized.checksum);
    }

    #[test]
    fn larger_player_count_matches_known_rehash_counts() {
        let default = default_capacity_summary(8, 500);
        let presized = presized_capacity_summary(8, 500);
        assert_eq!(default.last_total, 10);
        assert_eq!(default.total, 80);
        assert_eq!(presized.last_total, 0);
        assert_eq!(presized.total, 0);
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(default_capacity_summary(0, 50), NearbyPlayerMapCapacitySummary::default());
    }
}
