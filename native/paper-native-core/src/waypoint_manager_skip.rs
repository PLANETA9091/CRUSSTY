use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 1;
const CASES: usize = 1 << 10;
const CASE_MASK: usize = CASES - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaypointManagerSkipError {
    InvalidIterations,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaypointManagerSkipSummary {
    pub value: i64,
}

#[derive(Clone, Copy)]
enum Shape {
    CurrentPlayerFull,
    SkipPlayerFull,
    CurrentPlayerPartial,
    SkipPlayerPartial,
    CurrentWaypointFull,
    SkipWaypointFull,
    CurrentWaypointPartial,
    SkipWaypointPartial,
}

struct ManagerSkipData {
    current_player_full: [i64; CASES],
    skip_player_full: [i64; CASES],
    current_player_partial: [i64; CASES],
    skip_player_partial: [i64; CASES],
    current_waypoint_full: [i64; CASES],
    skip_waypoint_full: [i64; CASES],
    current_waypoint_partial: [i64; CASES],
    skip_waypoint_partial: [i64; CASES],
}

static MANAGER_SKIP_DATA: OnceLock<ManagerSkipData> = OnceLock::new();

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
pub fn current_player_full_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::CurrentPlayerFull)
}

#[inline]
pub fn skip_player_full_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::SkipPlayerFull)
}

#[inline]
pub fn current_player_partial_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::CurrentPlayerPartial)
}

#[inline]
pub fn skip_player_partial_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::SkipPlayerPartial)
}

#[inline]
pub fn current_waypoint_full_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::CurrentWaypointFull)
}

#[inline]
pub fn skip_waypoint_full_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::SkipWaypointFull)
}

#[inline]
pub fn current_waypoint_partial_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::CurrentWaypointPartial)
}

#[inline]
pub fn skip_waypoint_partial_summary(
    iterations: usize,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    run_summary(iterations, Shape::SkipWaypointPartial)
}

fn run_summary(
    iterations: usize,
    shape: Shape,
) -> Result<WaypointManagerSkipSummary, WaypointManagerSkipError> {
    if iterations == 0 {
        return Err(WaypointManagerSkipError::InvalidIterations);
    }

    let data = MANAGER_SKIP_DATA.get_or_init(build_data);
    let values = match shape {
        Shape::CurrentPlayerFull => &data.current_player_full,
        Shape::SkipPlayerFull => &data.skip_player_full,
        Shape::CurrentPlayerPartial => &data.current_player_partial,
        Shape::SkipPlayerPartial => &data.skip_player_partial,
        Shape::CurrentWaypointFull => &data.current_waypoint_full,
        Shape::SkipWaypointFull => &data.skip_waypoint_full,
        Shape::CurrentWaypointPartial => &data.current_waypoint_partial,
        Shape::SkipWaypointPartial => &data.skip_waypoint_partial,
    };

    let player_shape = matches!(
        shape,
        Shape::CurrentPlayerFull
            | Shape::SkipPlayerFull
            | Shape::CurrentPlayerPartial
            | Shape::SkipPlayerPartial
    );
    let stride = if player_shape { 53usize } else { 59usize };
    let mut value = 0i64;
    for i in 0..iterations {
        value = value.wrapping_add(values[(i * stride) & CASE_MASK]);
    }

    Ok(WaypointManagerSkipSummary { value })
}

fn build_data() -> ManagerSkipData {
    let mut random = JavaRandom::new(0x571A11E7BEEF42);
    let mut data = ManagerSkipData {
        current_player_full: [0; CASES],
        skip_player_full: [0; CASES],
        current_player_partial: [0; CASES],
        skip_player_partial: [0; CASES],
        current_waypoint_full: [0; CASES],
        skip_waypoint_full: [0; CASES],
        current_waypoint_partial: [0; CASES],
        skip_waypoint_partial: [0; CASES],
    };

    for index in 0..CASES {
        let base = (index as i32).wrapping_mul(257);
        let player_count = 24 + (index & 31);
        let waypoint_count = 24 + ((index * 7) & 31);
        let player_id = base + 3;
        let entity_waypoint_id = base + 100_000;

        let players = java_hashset_order(base, player_count, player_count * 2);
        let mut waypoints: Vec<i32> = (0..waypoint_count)
            .map(|offset| base + offset as i32)
            .collect();
        waypoints.push(entity_waypoint_id);

        let player_full = player_map(&waypoints, player_id, index, false);
        let player_partial = player_map(&waypoints, player_id, index, true);
        let waypoint_full = waypoint_map(&players, player_id, &mut random, false);
        let waypoint_partial = waypoint_map(&players, player_id, &mut random, true);

        data.current_player_full[index] = current_player(player_id, &waypoints, &player_full);
        data.skip_player_full[index] = skip_player(player_id, &waypoints, &player_full);
        data.current_player_partial[index] = current_player(player_id, &waypoints, &player_partial);
        data.skip_player_partial[index] = skip_player(player_id, &waypoints, &player_partial);
        data.current_waypoint_full[index] = current_waypoint(player_id, &players, &waypoint_full);
        data.skip_waypoint_full[index] = skip_waypoint(player_id, &players, &waypoint_full);
        data.current_waypoint_partial[index] = current_waypoint(player_id, &players, &waypoint_partial);
        data.skip_waypoint_partial[index] = skip_waypoint(player_id, &players, &waypoint_partial);
    }

    data
}

fn java_hashset_order(base: i32, count: usize, initial_capacity: usize) -> Vec<i32> {
    let capacity = table_size_for(initial_capacity);
    let mut buckets: Vec<Vec<i32>> = vec![Vec::new(); capacity];
    for offset in 0..count {
        let value = base + offset as i32;
        let bucket = (java_hash(value) as usize) & (capacity - 1);
        buckets[bucket].push(value);
    }

    let mut values = Vec::with_capacity(count);
    for bucket in buckets {
        values.extend(bucket);
    }
    values
}

fn table_size_for(capacity: usize) -> usize {
    let mut n = capacity.saturating_sub(1);
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

#[inline]
fn java_hash(value: i32) -> i32 {
    let bits = value as u32;
    (bits ^ (bits >> 16)) as i32
}

fn player_map(waypoints: &[i32], player_id: i32, index: usize, partial: bool) -> Vec<(i32, i32)> {
    let mut entries = Vec::with_capacity(waypoints.len());
    for &waypoint in waypoints {
        if waypoint != player_id
            && (!partial || ((waypoint as i64 + index as i64) % 5) != 0)
        {
            entries.push((waypoint, waypoint ^ player_id));
        }
    }
    entries
}

fn waypoint_map(
    players: &[i32],
    waypoint: i32,
    random: &mut JavaRandom,
    partial: bool,
) -> Vec<(i32, i32)> {
    let mut entries = Vec::with_capacity(players.len());
    for &player in players {
        if player != waypoint
            && (!partial || ((player as i64 + random.next_int(3) as i64) % 5) != 0)
        {
            entries.push((player, player ^ waypoint));
        }
    }
    entries
}

fn current_player(player_id: i32, waypoints: &[i32], map: &[(i32, i32)]) -> i64 {
    let mut sum = consume_connections(map);
    for &waypoint in waypoints {
        if !contains_key(map, waypoint) {
            sum = sum.wrapping_add(create_connection_cost(player_id, waypoint));
        }
    }
    sum
}

fn skip_player(player_id: i32, waypoints: &[i32], map: &[(i32, i32)]) -> i64 {
    let mut sum = consume_connections(map);
    let has_all_connections = map.len() >= waypoints.len()
        || (map.len() + 1 == waypoints.len() && waypoints.contains(&player_id));
    if !has_all_connections {
        for &waypoint in waypoints {
            if !contains_key(map, waypoint) {
                sum = sum.wrapping_add(create_connection_cost(player_id, waypoint));
            }
        }
    }
    sum
}

fn current_waypoint(waypoint: i32, players: &[i32], map: &[(i32, i32)]) -> i64 {
    let mut sum = consume_connections(map);
    for &player in players {
        if !contains_key(map, player) {
            sum = sum.wrapping_add(create_connection_cost(player, waypoint));
        }
    }
    sum
}

fn skip_waypoint(waypoint: i32, players: &[i32], map: &[(i32, i32)]) -> i64 {
    let mut sum = consume_connections(map);
    let has_all_connections = map.len() >= players.len()
        || (map.len() + 1 == players.len() && players.contains(&waypoint));
    if !has_all_connections {
        for &player in players {
            if !contains_key(map, player) {
                sum = sum.wrapping_add(create_connection_cost(player, waypoint));
            }
        }
    }
    sum
}

fn consume_connections(map: &[(i32, i32)]) -> i64 {
    let mut sum = 0i64;
    for &(key, value) in map {
        sum = sum.wrapping_add(update_connection_cost(key, value));
    }
    sum
}

#[inline]
fn contains_key(map: &[(i32, i32)], key: i32) -> bool {
    map.iter().any(|&(candidate, _)| candidate == key)
}

#[inline]
fn update_connection_cost(a: i32, b: i32) -> i64 {
    (a as i64).wrapping_mul(31) ^ (b as i64)
}

#[inline]
fn create_connection_cost(player: i32, waypoint: i32) -> i64 {
    if player == waypoint {
        0
    } else {
        (player as i64).wrapping_mul(17) ^ (waypoint as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_matches_current_for_full_rows() {
        assert_eq!(
            current_player_full_summary(25_000).unwrap(),
            skip_player_full_summary(25_000).unwrap()
        );
        assert_eq!(
            current_waypoint_full_summary(25_000).unwrap(),
            skip_waypoint_full_summary(25_000).unwrap()
        );
    }

    #[test]
    fn skip_matches_current_for_partial_rows() {
        assert_eq!(
            current_player_partial_summary(25_000).unwrap(),
            skip_player_partial_summary(25_000).unwrap()
        );
        assert_eq!(
            current_waypoint_partial_summary(25_000).unwrap(),
            skip_waypoint_partial_summary(25_000).unwrap()
        );
    }

    #[test]
    fn rejects_zero_iterations() {
        assert_eq!(
            current_player_full_summary(0),
            Err(WaypointManagerSkipError::InvalidIterations)
        );
    }
}
