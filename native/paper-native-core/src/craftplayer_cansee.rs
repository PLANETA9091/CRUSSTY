use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 4;

const ENTITY_COUNT: usize = 8192;
const SELF_INDEX: usize = 0;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const CANSEE_TAG: u64 = 0xC4A5_EED1_234A_BCD1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CraftPlayerCanSeeSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn current_empty_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::CurrentEmpty)
}

pub fn guarded_empty_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::GuardedEmpty)
}

pub fn candidate_empty_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::CandidateEmpty)
}

pub fn current_populated_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::CurrentPopulated)
}

pub fn guarded_populated_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::GuardedPopulated)
}

pub fn candidate_populated_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::CandidatePopulated)
}

pub fn chunkmap_candidate_empty_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::ChunkMapCandidateEmpty)
}

pub fn chunkmap_candidate_populated_summary(iterations: usize) -> CraftPlayerCanSeeSummary {
    run_summary(iterations, Scenario::ChunkMapCandidatePopulated)
}

#[derive(Clone, Copy)]
enum Scenario {
    CurrentEmpty,
    GuardedEmpty,
    CandidateEmpty,
    CurrentPopulated,
    GuardedPopulated,
    CandidatePopulated,
    ChunkMapCandidateEmpty,
    ChunkMapCandidatePopulated,
}

impl Scenario {
    fn tag(self) -> u64 {
        match self {
            Self::CurrentEmpty => 0x101,
            Self::GuardedEmpty => 0x102,
            Self::CandidateEmpty => 0x103,
            Self::CurrentPopulated => 0x104,
            Self::GuardedPopulated => 0x105,
            Self::CandidatePopulated => 0x106,
            Self::ChunkMapCandidateEmpty => 0x107,
            Self::ChunkMapCandidatePopulated => 0x108,
        }
    }
}

fn run_summary(iterations: usize, scenario: Scenario) -> CraftPlayerCanSeeSummary {
    if iterations == 0 {
        return CraftPlayerCanSeeSummary::default();
    }

    let data = data();
    let value = run_once(data, scenario);
    let shape_digest = mix64(CANSEE_TAG ^ scenario.tag() ^ (ENTITY_COUNT as u64));
    let mut checksum = 0u64;
    for iteration in 0..iterations {
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
        );
    }

    CraftPlayerCanSeeSummary {
        count: iterations as u64,
        total: value.wrapping_mul(iterations as u64),
        checksum,
        last_total: value,
    }
}

fn run_once(data: &CanSeeData, scenario: Scenario) -> u64 {
    let mut sum = 0u64;
    for index in 0..ENTITY_COUNT {
        let visible = match scenario {
            Scenario::CurrentEmpty => current_can_see(data, index, false),
            Scenario::GuardedEmpty => guarded_can_see(data, index, false),
            Scenario::CandidateEmpty => candidate_can_see(data, index, false),
            Scenario::ChunkMapCandidateEmpty => {
                if index == SELF_INDEX {
                    continue;
                }
                candidate_can_see(data, index, false)
            }
            Scenario::CurrentPopulated => current_can_see(data, index, true),
            Scenario::GuardedPopulated => guarded_can_see(data, index, true),
            Scenario::CandidatePopulated => candidate_can_see(data, index, true),
            Scenario::ChunkMapCandidatePopulated => {
                if index == SELF_INDEX {
                    continue;
                }
                candidate_can_see(data, index, true)
            }
        };
        sum += u64::from(visible);
    }
    sum
}

struct CanSeeData {
    visible_by_default: Vec<bool>,
    populated: Vec<bool>,
}

impl CanSeeData {
    fn new() -> Self {
        let mut visible_by_default = vec![false; ENTITY_COUNT];
        let mut populated = vec![false; ENTITY_COUNT];
        visible_by_default[SELF_INDEX] = true;
        for index in 1..ENTITY_COUNT {
            visible_by_default[index] = (index & 3) != 0;
            populated[index] = index < 768;
        }
        Self {
            visible_by_default,
            populated,
        }
    }
}

static DATA: OnceLock<CanSeeData> = OnceLock::new();

fn data() -> &'static CanSeeData {
    DATA.get_or_init(CanSeeData::new)
}

#[inline]
fn current_can_see(data: &CanSeeData, index: usize, populated: bool) -> bool {
    index == SELF_INDEX || (data.visible_by_default[index] ^ contains(index, populated, data))
}

#[inline]
fn guarded_can_see(data: &CanSeeData, index: usize, populated: bool) -> bool {
    index == SELF_INDEX
        || (data.visible_by_default[index]
            ^ (populated && contains(index, populated, data)))
}

#[inline]
fn candidate_can_see(data: &CanSeeData, index: usize, populated: bool) -> bool {
    data.visible_by_default[index] ^ (populated && contains(index, populated, data))
}

#[inline]
fn contains(index: usize, populated: bool, data: &CanSeeData) -> bool {
    populated && data.populated[index]
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
    fn empty_variants_match() {
        let current = current_empty_summary(8);
        let guarded = guarded_empty_summary(8);
        let candidate = candidate_empty_summary(8);
        assert_eq!(current.total, guarded.total);
        assert_eq!(current.total, candidate.total);
    }

    #[test]
    fn populated_variants_match() {
        let current = current_populated_summary(8);
        let guarded = guarded_populated_summary(8);
        let candidate = candidate_populated_summary(8);
        let chunkmap = chunkmap_candidate_populated_summary(8);
        assert_eq!(current.total, guarded.total);
        assert_eq!(current.total, candidate.total);
        assert!(chunkmap.total < current.total);
    }
}
