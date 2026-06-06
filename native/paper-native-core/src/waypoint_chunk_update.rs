use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 1;

const SIZE: usize = 1 << 16;
const MASK: usize = SIZE - 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaypointChunkUpdateSummary {
    pub value: i64,
}

struct ChunkUpdateData {
    last_x: Vec<i32>,
    last_z: Vec<i32>,
    current_x: Vec<i32>,
    current_z: Vec<i32>,
    last_key: Vec<i64>,
    current_key: Vec<i64>,
}

static DATA: OnceLock<ChunkUpdateData> = OnceLock::new();

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
    fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }
}

pub fn distance_summary(iterations: usize) -> WaypointChunkUpdateSummary {
    run_summary(iterations, Mode::Distance)
}

pub fn long_key_summary(iterations: usize) -> WaypointChunkUpdateSummary {
    run_summary(iterations, Mode::LongKey)
}

#[derive(Clone, Copy)]
enum Mode {
    Distance,
    LongKey,
}

fn run_summary(iterations: usize, mode: Mode) -> WaypointChunkUpdateSummary {
    let data = DATA.get_or_init(build_data);
    let mut value = 0i64;
    for i in 0..iterations {
        let index = (i * 37) & MASK;
        let changed = match mode {
            Mode::Distance => distance_changed(data, index),
            Mode::LongKey => long_key_changed(data, index),
        };
        value += i64::from(changed);
    }
    WaypointChunkUpdateSummary { value }
}

fn build_data() -> ChunkUpdateData {
    let mut random = JavaRandom::new(0xC8A17C6A11E7);
    let mut data = ChunkUpdateData {
        last_x: vec![0; SIZE],
        last_z: vec![0; SIZE],
        current_x: vec![0; SIZE],
        current_z: vec![0; SIZE],
        last_key: vec![0; SIZE],
        current_key: vec![0; SIZE],
    };

    for i in 0..SIZE {
        let x = random.next_int(200_000) - 100_000;
        let z = random.next_int(200_000) - 100_000;
        let (dx, dz) = match i & 7 {
            0 | 1 | 2 | 3 => (0, 0),
            4 | 5 => (if random.next_boolean() { 1 } else { -1 }, 0),
            6 => (0, if random.next_boolean() { 1 } else { -1 }),
            _ => (random.next_int(7) - 3, random.next_int(7) - 3),
        };

        data.last_x[i] = x;
        data.last_z[i] = z;
        data.current_x[i] = x + dx;
        data.current_z[i] = z + dz;
        data.last_key[i] = chunk_key(data.last_x[i], data.last_z[i]);
        data.current_key[i] = chunk_key(data.current_x[i], data.current_z[i]);
    }

    data
}

#[inline]
fn distance_changed(data: &ChunkUpdateData, index: usize) -> bool {
    let dx = (data.current_x[index] - data.last_x[index]).abs();
    let dz = (data.current_z[index] - data.last_z[index]).abs();
    dx.max(dz) > 0
}

#[inline]
fn long_key_changed(data: &ChunkUpdateData, index: usize) -> bool {
    data.current_key[index] != data.last_key[index]
}

#[inline]
fn chunk_key(x: i32, z: i32) -> i64 {
    ((x as u32 as u64) | ((z as u32 as u64) << 32)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_and_long_key_match() {
        assert_eq!(distance_summary(250_000), long_key_summary(250_000));
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(distance_summary(0), WaypointChunkUpdateSummary::default());
    }
}
