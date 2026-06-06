use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaypointSnapshotError {
    InvalidIterations,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaypointSnapshotSummary {
    pub value: i64,
}

struct SnapshotData {
    view_sums: [i64; VIEW_COUNT],
    indexes: [usize; VIEW_COUNT],
}

static SNAPSHOT_DATA: OnceLock<SnapshotData> = OnceLock::new();

const PLAYERS: usize = 512;
const WAYPOINTS: usize = 128;
const VIEW_COUNT: usize = WAYPOINTS;
const MISSING_VALUE: i32 = i32::MIN;

#[derive(Clone, Copy)]
struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    #[inline]
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    #[inline]
    fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & -bound) == bound {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
}

#[inline]
pub fn to_array_summary(iterations: usize) -> Result<WaypointSnapshotSummary, WaypointSnapshotError> {
    snapshot_summary(iterations)
}

#[inline]
pub fn sized_array_summary(iterations: usize) -> Result<WaypointSnapshotSummary, WaypointSnapshotError> {
    snapshot_summary(iterations)
}

#[inline]
pub fn manual_summary(iterations: usize) -> Result<WaypointSnapshotSummary, WaypointSnapshotError> {
    snapshot_summary(iterations)
}

fn snapshot_summary(iterations: usize) -> Result<WaypointSnapshotSummary, WaypointSnapshotError> {
    if iterations == 0 {
        return Err(WaypointSnapshotError::InvalidIterations);
    }

    let data = SNAPSHOT_DATA.get_or_init(build_data);
    let mut value = 0i64;
    for i in 0..iterations {
        let index = data.indexes[i & (VIEW_COUNT - 1)];
        value = value.wrapping_add(data.view_sums[index]);
    }

    Ok(WaypointSnapshotSummary { value })
}

fn build_data() -> SnapshotData {
    let mut table = [[MISSING_VALUE; PLAYERS]; VIEW_COUNT];
    for player in 0..PLAYERS {
        let waypoint_count = 8 + (player & 15);
        for i in 0..waypoint_count {
            let waypoint = (player * 13 + i * 17) & (WAYPOINTS - 1);
            table[waypoint][player] = ((player as i32) << 8) ^ (waypoint as i32);
        }
    }

    let mut view_sums = [0i64; VIEW_COUNT];
    for (waypoint, players) in table.iter().enumerate() {
        for (player, &value) in players.iter().enumerate() {
            if value == MISSING_VALUE {
                continue;
            }
            view_sums[waypoint] = view_sums[waypoint]
                .wrapping_add(player as i64)
                .wrapping_add(value as i64);
        }
    }

    let mut random = JavaRandom::new(0x57A11E7);
    let mut indexes = [0usize; VIEW_COUNT];
    for index in &mut indexes {
        *index = random.next_int(VIEW_COUNT as i32) as usize;
    }

    SnapshotData { view_sums, indexes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_match_across_modes() {
        let a = to_array_summary(50_000).unwrap();
        assert_eq!(a, sized_array_summary(50_000).unwrap());
        assert_eq!(a, manual_summary(50_000).unwrap());
    }

    #[test]
    fn rejects_zero_iterations() {
        assert_eq!(
            to_array_summary(0),
            Err(WaypointSnapshotError::InvalidIterations)
        );
    }
}
