pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const OWNERS: [&str; 6] = [
    "Lorg/bukkit/World;",
    "Lorg/bukkit/block/Block;",
    "Lorg/bukkit/entity/Player;",
    "Lorg/bukkit/inventory/ItemStack;",
    "Lnet/minecraft/world/entity/Entity;",
    "[Lorg/bukkit/entity/Player;",
];
const QUERIES: [&str; 6] = [
    "org/bukkit/entity/Player",
    "org/bukkit/World",
    "org/bukkit/inventory/ItemStack",
    "net/minecraft/world/entity/Entity",
    "[Lorg/bukkit/entity/Player;",
    "missing/Owner",
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OwnableRuleSummary {
    pub count: u64,
    pub matches: u64,
    pub checksum: u64,
    pub last_match: u64,
}

pub fn old_stream_summary(iterations: usize) -> OwnableRuleSummary {
    run_summary(iterations, old_stream_matches)
}

pub fn new_loop_summary(iterations: usize) -> OwnableRuleSummary {
    run_summary(iterations, new_loop_matches)
}

fn run_summary<F>(iterations: usize, mut matches_owner: F) -> OwnableRuleSummary
where
    F: FnMut(&str) -> bool,
{
    if iterations == 0 {
        return OwnableRuleSummary::default();
    }

    let mut matches = 0u64;
    let mut checksum = 0u64;
    let mut last_match = 0u64;

    for iteration in 0..iterations {
        let owner = QUERIES[iteration % QUERIES.len()];
        let matched = matches_owner(owner);
        if matched {
            matches += 1;
        }
        last_match = u64::from(matched);
        checksum = mix64(
            checksum
                ^ hash_str(owner)
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ if matched {
                    0xD6E8_FEB8_6659_FD93
                } else {
                    0xA5A3_58B5_C9CB_4F1D
                },
        );
    }

    OwnableRuleSummary {
        count: iterations as u64,
        matches,
        checksum,
        last_match,
    }
}

fn old_stream_matches(owner: &str) -> bool {
    OWNERS
        .iter()
        .map(|descriptor| descriptor_to_owner(descriptor))
        .any(|candidate| candidate == owner)
}

fn new_loop_matches(owner: &str) -> bool {
    for descriptor in OWNERS {
        if matches_owner(descriptor, owner) {
            return true;
        }
    }
    false
}

fn matches_owner(descriptor: &str, owner: &str) -> bool {
    if descriptor.len() > 1 && descriptor.starts_with('L') && descriptor.ends_with(';') {
        let owner_descriptor = &descriptor[1..descriptor.len() - 1];
        owner_descriptor == owner
    } else {
        descriptor == owner
    }
}

fn descriptor_to_owner(descriptor: &str) -> &str {
    if descriptor.len() > 1 && descriptor.starts_with('L') && descriptor.ends_with(';') {
        &descriptor[1..descriptor.len() - 1]
    } else {
        descriptor
    }
}

#[inline]
fn hash_str(value: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in value.as_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
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
    fn old_and_new_summaries_match() {
        let old = old_stream_summary(12_000);
        let new = new_loop_summary(12_000);

        assert_eq!(old, new);
        assert_eq!(old.matches, 10_000);
    }

    #[test]
    fn owner_queries_match_expected_results() {
        let expected = [true, true, true, true, true, false];
        for (query, expected) in QUERIES.iter().zip(expected) {
            assert_eq!(old_stream_matches(query), expected);
            assert_eq!(new_loop_matches(query), expected);
        }
    }

    #[test]
    fn descriptor_conversion_matches_arrays_and_classes() {
        assert_eq!(
            descriptor_to_owner("Lorg/bukkit/entity/Player;"),
            "org/bukkit/entity/Player"
        );
        assert_eq!(
            descriptor_to_owner("[Lorg/bukkit/entity/Player;"),
            "[Lorg/bukkit/entity/Player;"
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(old_stream_summary(0), OwnableRuleSummary::default());
    }
}
