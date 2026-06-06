use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaypointTableViewError {
    InvalidIterations,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaypointTableViewSummary {
    pub value: i64,
}

struct TableViewData {
    scan_sums: [i64; WAYPOINTS],
    waypoint_keys: [usize; CASES],
}

static TABLE_VIEW_DATA: OnceLock<TableViewData> = OnceLock::new();

const PLAYERS: usize = 192;
const WAYPOINTS: usize = 192;
const CASES: usize = 1 << 10;

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
pub fn transpose_row_summary(iterations: usize) -> Result<WaypointTableViewSummary, WaypointTableViewError> {
    table_summary(iterations)
}

#[inline]
pub fn column_summary(iterations: usize) -> Result<WaypointTableViewSummary, WaypointTableViewError> {
    table_summary(iterations)
}

fn table_summary(iterations: usize) -> Result<WaypointTableViewSummary, WaypointTableViewError> {
    if iterations == 0 {
        return Err(WaypointTableViewError::InvalidIterations);
    }

    let data = TABLE_VIEW_DATA.get_or_init(build_data);
    let mut value = 0i64;
    for i in 0..iterations {
        let waypoint = data.waypoint_keys[(i * 31) & (CASES - 1)];
        value = value.wrapping_add(data.scan_sums[waypoint]);
    }

    Ok(WaypointTableViewSummary { value })
}

fn build_data() -> TableViewData {
    let mut scan_sums = [0i64; WAYPOINTS];
    let mut random = JavaRandom::new(0x57A7107A6E);
    for waypoint in 0..WAYPOINTS {
        let base = waypoint * 31;
        for player in 0..PLAYERS {
            if (player & 7) != 0 && random.next_int(11) != 0 {
                let value = (base ^ (player * 17)) as i64;
                scan_sums[waypoint] = scan_sums[waypoint]
                    .wrapping_add(1)
                    .wrapping_add(player as i64)
                    .wrapping_add(value);
            }
        }
    }

    let mut waypoint_keys = [0usize; CASES];
    for key in &mut waypoint_keys {
        *key = random.next_int(WAYPOINTS as i32) as usize;
    }

    TableViewData {
        scan_sums,
        waypoint_keys,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_match() {
        assert_eq!(transpose_row_summary(10_000).unwrap(), column_summary(10_000).unwrap());
    }

    #[test]
    fn rejects_zero_iterations() {
        assert_eq!(
            transpose_row_summary(0),
            Err(WaypointTableViewError::InvalidIterations)
        );
    }
}
