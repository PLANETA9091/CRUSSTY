pub const SUMMARY_FIELDS: usize = 2;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlacedFeatureTraversalSummary {
    pub count: u64,
    pub hash: u64,
}

#[derive(Clone, Copy)]
struct Pos {
    x: i32,
    y: i32,
    z: i32,
}

struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    fn new(seed: i64) -> Self {
        Self {
            seed: ((seed as u64) ^ Self::MULTIPLIER) & Self::MASK,
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

pub fn recursive_summary(seed: i64, traversals: usize) -> PlacedFeatureTraversalSummary {
    let mut random = JavaRandom::new(seed);
    let mut summary = PlacedFeatureTraversalSummary {
        count: 0,
        hash: 0xCBF2_9CE4_8422_2325,
    };
    let start = Pos { x: 8, y: 64, z: 8 };

    for _ in 0..traversals {
        traverse_filter_even_x(&mut random, &mut summary, start);
    }

    summary
}

#[inline]
fn traverse_filter_even_x(
    random: &mut JavaRandom,
    summary: &mut PlacedFeatureTraversalSummary,
    pos: Pos,
) {
    if (pos.x & 1) == 0 {
        traverse_repeat_range_1_3(random, summary, pos);
    }
}

#[inline]
fn traverse_repeat_range_1_3(
    random: &mut JavaRandom,
    summary: &mut PlacedFeatureTraversalSummary,
    pos: Pos,
) {
    let count = 1 + random.next_int(3);
    for _ in 0..count {
        traverse_random_offset(random, summary, pos);
    }
}

#[inline]
fn traverse_random_offset(
    random: &mut JavaRandom,
    summary: &mut PlacedFeatureTraversalSummary,
    pos: Pos,
) {
    let offset = Pos {
        x: pos.x + random.next_int(16),
        y: pos.y + random.next_int(4),
        z: pos.z + random.next_int(16),
    };
    traverse_fanout(random, summary, offset);
}

#[inline]
fn traverse_fanout(
    random: &mut JavaRandom,
    summary: &mut PlacedFeatureTraversalSummary,
    pos: Pos,
) {
    for i in 0..3 {
        let candidate = Pos {
            x: pos.x + i,
            y: pos.y + (i & 1),
            z: pos.z - i,
        };
        traverse_filter_even_z(random, summary, candidate);
    }
}

#[inline]
fn traverse_filter_even_z(
    random: &mut JavaRandom,
    summary: &mut PlacedFeatureTraversalSummary,
    pos: Pos,
) {
    if (pos.z & 1) == 0 {
        traverse_repeat_range_0_2(random, summary, pos);
    }
}

#[inline]
fn traverse_repeat_range_0_2(
    random: &mut JavaRandom,
    summary: &mut PlacedFeatureTraversalSummary,
    pos: Pos,
) {
    let count = random.next_int(3);
    for _ in 0..count {
        accept(summary, pos);
    }
}

#[inline]
fn accept(summary: &mut PlacedFeatureTraversalSummary, pos: Pos) {
    summary.count = summary.count.wrapping_add(1);
    summary.hash ^= pos.x as i64 as u64;
    summary.hash = summary.hash.wrapping_mul(0x0000_0100_0000_01B3);
    summary.hash ^= pos.y as i64 as u64;
    summary.hash = summary.hash.wrapping_mul(0x0000_0100_0000_01B3);
    summary.hash ^= pos.z as i64 as u64;
    summary.hash = summary.hash.wrapping_mul(0x0000_0100_0000_01B3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_traversal_keeps_initial_hash() {
        assert_eq!(
            recursive_summary(1, 0),
            PlacedFeatureTraversalSummary {
                count: 0,
                hash: 0xCBF2_9CE4_8422_2325
            }
        );
    }

    #[test]
    fn traversal_is_deterministic() {
        assert_eq!(recursive_summary(99, 10_000), recursive_summary(99, 10_000));
        assert_ne!(recursive_summary(99, 10_000), recursive_summary(100, 10_000));
    }
}
