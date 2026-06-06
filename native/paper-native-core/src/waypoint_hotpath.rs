use std::collections::HashSet;
use std::sync::OnceLock;

const SIZE: usize = 1 << 15;
const MASK: usize = SIZE - 1;
const MANAGER_SIZE: usize = 1 << 11;
const MANAGER_MASK: usize = MANAGER_SIZE - 1;
const REALLY_FAR_DISTANCE: f64 = 332.0;
const FRAC_BIAS: f64 = 17_592_186_044_416.0;

struct HotPathData {
    source_x: Vec<f64>,
    source_y: Vec<f64>,
    source_z: Vec<f64>,
    receiver_x: Vec<f64>,
    receiver_y: Vec<f64>,
    receiver_z: Vec<f64>,
    range: Vec<f64>,
    chunk_x: Vec<i32>,
    chunk_z: Vec<i32>,
    chunk_key: Vec<i64>,
    sent_chunks: HashSet<i64>,
    manager_rows: Vec<ManagerRow>,
}

struct ManagerRow {
    players: Vec<i32>,
    keys: Vec<i32>,
    values: Vec<i32>,
    key_set: HashSet<i32>,
}

struct MthTables {
    asin: [f64; 257],
    cos: [f64; 257],
}

static HOTPATH_DATA: OnceLock<HotPathData> = OnceLock::new();
static MTH_TABLES: OnceLock<MthTables> = OnceLock::new();

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

    #[inline]
    fn next_double(&mut self) -> f64 {
        let high = (self.next(26) as i64) << 27;
        let low = self.next(27) as i64;
        ((high + low) as f64) / ((1u64 << 53) as f64)
    }

    #[inline]
    fn next_double_between(&mut self, origin: f64, bound: f64) -> f64 {
        let value = self.next_double() * (bound - origin) + origin;
        if value < bound {
            value
        } else {
            next_down(bound)
        }
    }
}

#[inline]
pub fn old_azimuth_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::OldAzimuth)
}

#[inline]
pub fn direct_azimuth_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::DirectAzimuth)
}

#[inline]
pub fn old_at_or_beyond_range_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::OldAtOrBeyondRange)
}

#[inline]
pub fn guarded_at_or_beyond_range_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::GuardedAtOrBeyondRange)
}

#[inline]
pub fn old_really_far_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::OldReallyFar)
}

#[inline]
pub fn guarded_really_far_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::GuardedReallyFar)
}

#[inline]
pub fn old_chunk_visible_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::OldChunkVisible)
}

#[inline]
pub fn cached_chunk_visible_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::CachedChunkVisible)
}

#[inline]
pub fn old_waypoint_manager_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::OldWaypointManager)
}

#[inline]
pub fn optimized_waypoint_manager_value(iterations: usize) -> f64 {
    run_value(iterations, Shape::OptimizedWaypointManager)
}

#[derive(Clone, Copy)]
enum Shape {
    OldAzimuth,
    DirectAzimuth,
    OldAtOrBeyondRange,
    GuardedAtOrBeyondRange,
    OldReallyFar,
    GuardedReallyFar,
    OldChunkVisible,
    CachedChunkVisible,
    OldWaypointManager,
    OptimizedWaypointManager,
}

fn run_value(iterations: usize, shape: Shape) -> f64 {
    if iterations == 0 {
        return 0.0;
    }

    let data = HOTPATH_DATA.get_or_init(build_data);
    let mut value = 0.0f64;
    for iteration in 0..iterations {
        value += match shape {
            Shape::OldAzimuth => old_azimuth(data, (iteration * 17) & MASK) as f64,
            Shape::DirectAzimuth => direct_azimuth(data, (iteration * 17) & MASK) as f64,
            Shape::OldAtOrBeyondRange => {
                bool_value(old_at_or_beyond_range(data, (iteration * 31) & MASK))
            }
            Shape::GuardedAtOrBeyondRange => {
                bool_value(guarded_at_or_beyond_range(data, (iteration * 31) & MASK))
            }
            Shape::OldReallyFar => bool_value(old_really_far(data, (iteration * 43) & MASK)),
            Shape::GuardedReallyFar => {
                bool_value(guarded_really_far(data, (iteration * 43) & MASK))
            }
            Shape::OldChunkVisible => bool_value(old_chunk_visible(data, (iteration * 47) & MASK)),
            Shape::CachedChunkVisible => {
                bool_value(cached_chunk_visible(data, (iteration * 47) & MASK))
            }
            Shape::OldWaypointManager => old_waypoint_manager(data, (iteration * 53) & MANAGER_MASK),
            Shape::OptimizedWaypointManager => {
                optimized_waypoint_manager(data, (iteration * 53) & MANAGER_MASK)
            }
        };
    }
    value
}

#[inline]
fn bool_value(value: bool) -> f64 {
    if value {
        1.0
    } else {
        0.0
    }
}

fn build_data() -> HotPathData {
    let mut random = JavaRandom::new(0x51A7E11A17);
    let mut source_x = vec![0.0; SIZE];
    let mut source_y = vec![0.0; SIZE];
    let mut source_z = vec![0.0; SIZE];
    let mut receiver_x = vec![0.0; SIZE];
    let mut receiver_y = vec![0.0; SIZE];
    let mut receiver_z = vec![0.0; SIZE];
    let mut range = vec![0.0; SIZE];
    let mut chunk_x = vec![0; SIZE];
    let mut chunk_z = vec![0; SIZE];
    let mut chunk_key = vec![0; SIZE];
    let mut sent_chunks = HashSet::with_capacity(SIZE);

    for i in 0..SIZE {
        source_x[i] = random.next_double_between(-12_000.0, 12_000.0);
        source_y[i] = random.next_double_between(-64.0, 320.0);
        source_z[i] = random.next_double_between(-12_000.0, 12_000.0);

        let radius = if (i & 7) == 0 {
            random.next_double_between(333.0, 900.0)
        } else if (i & 7) == 1 {
            random.next_double_between(166.0, 333.5)
        } else {
            random.next_double_between(0.0, 120.0)
        };

        let angle = random.next_double_between(0.0, std::f64::consts::PI * 2.0);
        receiver_x[i] = source_x[i] + angle.cos() * radius;
        receiver_y[i] = source_y[i] + random.next_double_between(-24.0, 24.0);
        receiver_z[i] = source_z[i] + angle.sin() * radius;
        range[i] = 16.0 + ((i & 31) as f64) * 4.0;

        let x = random.next_int(16_384) - 8192;
        let z = random.next_int(16_384) - 8192;
        chunk_x[i] = x;
        chunk_z[i] = z;
        chunk_key[i] = pack_chunk_key(x, z);
        if (i & 3) != 0 {
            sent_chunks.insert(chunk_key[i]);
        }
    }

    let mut manager_rows = Vec::with_capacity(MANAGER_SIZE);
    for i in 0..MANAGER_SIZE {
        let player_count = 24 + (i & 31);
        let connected_count = 6 + (i & 15);
        let base = (i as i32).wrapping_mul(97);
        let mut players = Vec::with_capacity(player_count);
        let mut keys = Vec::with_capacity(connected_count);
        let mut values = Vec::with_capacity(connected_count);
        let mut key_set = HashSet::with_capacity(connected_count * 2);
        for j in 0..player_count {
            let player = base + j as i32;
            players.push(player);
            if j < connected_count {
                keys.push(player);
                values.push(base ^ player.wrapping_mul(3));
                key_set.insert(player);
            }
        }
        manager_rows.push(ManagerRow {
            players,
            keys,
            values,
            key_set,
        });
    }

    HotPathData {
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        range,
        chunk_x,
        chunk_z,
        chunk_key,
        sent_chunks,
        manager_rows,
    }
}

#[inline]
fn old_azimuth(data: &HotPathData, index: usize) -> f32 {
    direct_azimuth(data, index)
}

#[inline]
fn direct_azimuth(data: &HotPathData, index: usize) -> f32 {
    mth_atan2(
        data.receiver_x[index] - data.source_x[index],
        data.source_z[index] - data.receiver_z[index],
    ) as f32
}

#[inline]
fn old_at_or_beyond_range(data: &HotPathData, index: usize) -> bool {
    let dx = (data.source_x[index] - data.receiver_x[index]) as f32;
    let dy = (data.source_y[index] - data.receiver_y[index]) as f32;
    let dz = (data.source_z[index] - data.receiver_z[index]) as f32;
    java_sqrt(dx * dx + dy * dy + dz * dz) >= data.range[index] as f32
}

#[inline]
fn guarded_at_or_beyond_range(data: &HotPathData, index: usize) -> bool {
    let range = data.range[index] as f32;
    let dx = (data.source_x[index] - data.receiver_x[index]) as f32;
    let dy = (data.source_y[index] - data.receiver_y[index]) as f32;
    let dz = (data.source_z[index] - data.receiver_z[index]) as f32;
    let half_range = range * 0.5;
    if dx.abs() < half_range && dy.abs() < half_range && dz.abs() < half_range {
        return false;
    }
    java_sqrt(dx * dx + dy * dy + dz * dz) >= range
}

#[inline]
fn old_really_far(data: &HotPathData, index: usize) -> bool {
    let dx = (data.source_x[index] - data.receiver_x[index]) as f32;
    let dy = (data.source_y[index] - data.receiver_y[index]) as f32;
    let dz = (data.source_z[index] - data.receiver_z[index]) as f32;
    java_sqrt(dx * dx + dy * dy + dz * dz) > REALLY_FAR_DISTANCE as f32
}

#[inline]
fn guarded_really_far(data: &HotPathData, index: usize) -> bool {
    let dx = (data.source_x[index] - data.receiver_x[index]) as f32;
    let dy = (data.source_y[index] - data.receiver_y[index]) as f32;
    let dz = (data.source_z[index] - data.receiver_z[index]) as f32;
    let really_far = REALLY_FAR_DISTANCE as f32;
    if dx > really_far
        || dx < -really_far
        || dy > really_far
        || dy < -really_far
        || dz > really_far
        || dz < -really_far
    {
        return true;
    }
    let half_range = (REALLY_FAR_DISTANCE * 0.5) as f32;
    if dx.abs() < half_range && dy.abs() < half_range && dz.abs() < half_range {
        return false;
    }
    java_sqrt(dx * dx + dy * dy + dz * dz) > really_far
}

#[inline]
fn old_chunk_visible(data: &HotPathData, index: usize) -> bool {
    data.sent_chunks
        .contains(&pack_chunk_key(data.chunk_x[index], data.chunk_z[index]))
}

#[inline]
fn cached_chunk_visible(data: &HotPathData, index: usize) -> bool {
    data.sent_chunks.contains(&data.chunk_key[index])
}

#[inline]
fn old_waypoint_manager(data: &HotPathData, index: usize) -> f64 {
    let row = &data.manager_rows[index];
    let snapshot: Vec<(i32, i32)> = row
        .keys
        .iter()
        .copied()
        .zip(row.values.iter().copied())
        .collect();
    let mut sum = 0i64;
    for (key, value) in snapshot {
        sum += i64::from(key) + i64::from(value);
    }
    for &player in &row.players {
        if !row.key_set.contains(&player) {
            sum += i64::from(player);
        }
    }
    sum as f64
}

#[inline]
fn optimized_waypoint_manager(data: &HotPathData, index: usize) -> f64 {
    let row = &data.manager_rows[index];
    let mut sum = 0i64;
    for (&key, &value) in row.keys.iter().zip(&row.values) {
        sum += i64::from(key) + i64::from(value);
    }
    for &player in &row.players {
        if !row.key_set.contains(&player) {
            sum += i64::from(player);
        }
    }
    sum as f64
}

#[inline]
fn java_sqrt(value: f32) -> f32 {
    (f64::from(value).sqrt()) as f32
}

fn mth_atan2(mut y: f64, mut x: f64) -> f64 {
    let square = x * x + y * y;
    if square.is_nan() {
        return f64::NAN;
    }

    let negative_y = y < 0.0;
    if negative_y {
        y = -y;
    }
    let negative_x = x < 0.0;
    if negative_x {
        x = -x;
    }
    let swapped = y > x;
    if swapped {
        std::mem::swap(&mut x, &mut y);
    }

    let inverse = fast_inv_sqrt(square);
    x *= inverse;
    y *= inverse;

    let tables = MTH_TABLES.get_or_init(build_mth_tables);
    let biased = FRAC_BIAS + y;
    let index = (biased.to_bits() as u32) as usize;
    let asin = tables.asin[index];
    let cos = tables.cos[index];
    let rounded = biased - FRAC_BIAS;
    let error = y * cos - x * rounded;
    let correction = (6.0 + error * error) * error * (1.0 / 6.0);
    let mut result = asin + correction;

    if swapped {
        result = std::f64::consts::FRAC_PI_2 - result;
    }
    if negative_x {
        result = std::f64::consts::PI - result;
    }
    if negative_y {
        result = -result;
    }
    result
}

fn build_mth_tables() -> MthTables {
    let mut asin = [0.0; 257];
    let mut cos = [0.0; 257];
    for index in 0..257 {
        let value = (index as f64) / 256.0;
        let angle = value.asin();
        asin[index] = angle;
        cos[index] = angle.cos();
    }
    MthTables { asin, cos }
}

#[inline]
fn fast_inv_sqrt(value: f64) -> f64 {
    let half = 0.5 * value;
    let bits = 6_910_469_410_427_058_090u64.wrapping_sub(value.to_bits() >> 1);
    let estimate = f64::from_bits(bits);
    estimate * (1.5 - half * estimate * estimate)
}

#[inline]
fn pack_chunk_key(chunk_x: i32, chunk_z: i32) -> i64 {
    (((chunk_z as i64) << 32) | i64::from(chunk_x as u32)) as i64
}

#[inline]
fn next_down(value: f64) -> f64 {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return -f64::MIN_POSITIVE;
    }
    let bits = value.to_bits();
    if value > 0.0 {
        f64::from_bits(bits - 1)
    } else {
        f64::from_bits(bits + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_direct_azimuth_values_match() {
        assert_eq!(old_azimuth_value(100_000).to_bits(), direct_azimuth_value(100_000).to_bits());
    }

    #[test]
    fn guarded_range_shapes_match() {
        assert_eq!(old_at_or_beyond_range_value(100_000), guarded_at_or_beyond_range_value(100_000));
        assert_eq!(old_really_far_value(100_000), guarded_really_far_value(100_000));
    }

    #[test]
    fn cached_chunk_visibility_matches() {
        assert_eq!(old_chunk_visible_value(100_000), cached_chunk_visible_value(100_000));
    }

    #[test]
    fn manager_shapes_match() {
        assert_eq!(old_waypoint_manager_value(100_000), optimized_waypoint_manager_value(100_000));
    }
}
