use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 4;

const SIZE: usize = 1 << 20;
const MASK: usize = SIZE - 1;
const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const OLD_STATUS_STEP: usize = 17;
const ACCESSIBLE_STEP: usize = 31;

const NULL_STATUS: u8 = 0;
const INACCESSIBLE_STATUS: u8 = 1;
const FULL_STATUS: u8 = 2;
const BLOCK_TICKING_STATUS: u8 = 3;
const ENTITY_TICKING_STATUS: u8 = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EntityLookupStatusSummary {
    pub count: u64,
    pub value: i64,
    pub checksum: u64,
    pub last_value: i64,
}

pub fn old_status_summary(iterations: usize) -> EntityLookupStatusSummary {
    run_summary(iterations, OLD_STATUS_STEP, |data, index| status_ordinal(old_status(data, index)))
}

pub fn direct_status_summary(iterations: usize) -> EntityLookupStatusSummary {
    run_summary(iterations, OLD_STATUS_STEP, |data, index| status_ordinal(direct_status(data, index)))
}

pub fn old_accessible_summary(iterations: usize) -> EntityLookupStatusSummary {
    run_summary(iterations, ACCESSIBLE_STEP, |data, index| {
        if old_status(data, index).is_accessible() {
            1
        } else {
            0
        }
    })
}

pub fn direct_accessible_summary(iterations: usize) -> EntityLookupStatusSummary {
    run_summary(iterations, ACCESSIBLE_STEP, |data, index| {
        if direct_status(data, index).is_accessible() {
            1
        } else {
            0
        }
    })
}

fn run_summary<F>(iterations: usize, step: usize, mut map: F) -> EntityLookupStatusSummary
where
    F: FnMut(&EntityLookupData, usize) -> i32,
{
    if iterations == 0 {
        return EntityLookupStatusSummary::default();
    }

    let data = data();
    let mut value = 0i32;
    let mut checksum = 0u64;
    let mut last_value = 0i32;

    for iteration in 0..iterations {
        let index = iteration.wrapping_mul(step) & MASK;
        let mapped = map(data, index);
        value = value.wrapping_add(mapped);
        last_value = mapped;
        checksum = mix64(
            checksum
                ^ (mapped as u32 as u64)
                ^ ((index as u64) << 8)
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
        );
    }

    EntityLookupStatusSummary {
        count: iterations as u64,
        value: i64::from(value),
        checksum,
        last_value: i64::from(last_value),
    }
}

struct EntityLookupData {
    always_ticking: Vec<u8>,
    statuses: Vec<u8>,
}

impl EntityLookupData {
    fn new() -> Self {
        let mut always_ticking = Vec::with_capacity(SIZE);
        let mut statuses = Vec::with_capacity(SIZE);

        for index in 0..SIZE {
            always_ticking.push(((index & 127) == 0) as u8);
            statuses.push(status_code(index));
        }

        Self {
            always_ticking,
            statuses,
        }
    }
}

static DATA: OnceLock<EntityLookupData> = OnceLock::new();

fn data() -> &'static EntityLookupData {
    DATA.get_or_init(EntityLookupData::new)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Visibility {
    Hidden,
    Tracked,
    Ticking,
}

impl Visibility {
    #[inline]
    fn is_accessible(self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

#[inline]
fn old_status(data: &EntityLookupData, index: usize) -> Visibility {
    if data.always_ticking[index] != 0 {
        return Visibility::Ticking;
    }

    let status = data.statuses[index];
    let status = if status == NULL_STATUS {
        INACCESSIBLE_STATUS
    } else {
        status
    };
    visibility_from_full_chunk_status(status)
}

#[inline]
fn direct_status(data: &EntityLookupData, index: usize) -> Visibility {
    if data.always_ticking[index] != 0 {
        return Visibility::Ticking;
    }

    match data.statuses[index] {
        NULL_STATUS | INACCESSIBLE_STATUS => Visibility::Hidden,
        ENTITY_TICKING_STATUS => Visibility::Ticking,
        FULL_STATUS | BLOCK_TICKING_STATUS => Visibility::Tracked,
        _ => Visibility::Hidden,
    }
}

#[inline]
fn visibility_from_full_chunk_status(status: u8) -> Visibility {
    if status >= ENTITY_TICKING_STATUS {
        Visibility::Ticking
    } else if status >= FULL_STATUS {
        Visibility::Tracked
    } else {
        Visibility::Hidden
    }
}

#[inline]
fn status_ordinal(visibility: Visibility) -> i32 {
    match visibility {
        Visibility::Hidden => 0,
        Visibility::Tracked => 1,
        Visibility::Ticking => 2,
    }
}

#[inline]
fn status_code(index: usize) -> u8 {
    if index & 31 == 0 {
        NULL_STATUS
    } else {
        1 + (((index.wrapping_mul(13)) & 3) as u8)
    }
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
    fn old_and_direct_status_summaries_match() {
        let old = old_status_summary(128_000);
        let direct = direct_status_summary(128_000);

        assert_eq!(old, direct);
        assert_eq!(old.count, 128_000);
    }

    #[test]
    fn old_and_direct_accessible_summaries_match() {
        let old = old_accessible_summary(128_000);
        let direct = direct_accessible_summary(128_000);

        assert_eq!(old, direct);
        assert_eq!(old.count, 128_000);
    }

    #[test]
    fn lookup_cases_match_expected_visibility() {
        let data = data();

        assert_eq!(old_status(data, 0), Visibility::Ticking);
        assert_eq!(direct_status(data, 0), Visibility::Ticking);
        assert_eq!(old_status(data, 1), Visibility::Tracked);
        assert_eq!(direct_status(data, 1), Visibility::Tracked);
        assert_eq!(old_status(data, 4), Visibility::Hidden);
        assert_eq!(direct_status(data, 4), Visibility::Hidden);
        assert_eq!(old_status(data, 32), Visibility::Hidden);
        assert_eq!(direct_status(data, 32), Visibility::Hidden);
        assert!(old_status(data, 1).is_accessible());
        assert!(direct_status(data, 1).is_accessible());
        assert!(!old_status(data, 4).is_accessible());
        assert!(!direct_status(data, 4).is_accessible());
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(old_status_summary(0), EntityLookupStatusSummary::default());
        assert_eq!(direct_accessible_summary(0), EntityLookupStatusSummary::default());
    }
}
