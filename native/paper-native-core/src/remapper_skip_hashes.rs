use std::collections::HashSet;

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SKIP_HASHES_TAG: u64 = 0xE3A4_9D71_83D5_2A6B;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RemapperSkipHashesSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_stream_summary(iterations: usize, content: &str) -> RemapperSkipHashesSummary {
    run_summary(iterations, content, Mode::OldStream)
}

pub fn new_loop_summary(iterations: usize, content: &str) -> RemapperSkipHashesSummary {
    run_summary(iterations, content, Mode::NewLoop)
}

#[derive(Clone, Copy)]
enum Mode {
    OldStream,
    NewLoop,
}

fn run_summary(iterations: usize, content: &str, mode: Mode) -> RemapperSkipHashesSummary {
    if iterations == 0 {
        return RemapperSkipHashesSummary::default();
    }

    let content_digest = mix64(SKIP_HASHES_TAG ^ (java_string_hash(content) as i64 as u64));
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let hashes = match mode {
            Mode::OldStream => parse_old(content),
            Mode::NewLoop => parse_new(content),
        };
        let value = hashes.len() as u64;
        let set_digest = hashes_digest(&hashes);
        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ set_digest
                ^ content_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
        );
    }

    RemapperSkipHashesSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn parse_old(content: &str) -> HashSet<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|hash| !hash.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_new(content: &str) -> HashSet<String> {
    let mut hashes = HashSet::with_capacity(16);
    for line in content.lines() {
        let hash = line.trim();
        if !hash.is_empty() {
            hashes.insert(hash.to_string());
        }
    }
    hashes
}

fn hashes_digest(hashes: &HashSet<String>) -> u64 {
    let mut sorted = hashes.iter().collect::<Vec<_>>();
    sorted.sort_unstable();
    let mut digest = mix64((sorted.len() as u64) ^ 0xB492_B66F_BE98_F273);
    for (index, value) in sorted.iter().enumerate() {
        digest = mix64(
            digest
                ^ (java_string_hash(value) as i64 as u64)
                ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
        );
    }
    digest
}

fn java_string_hash(value: &str) -> i32 {
    let mut hash = 0i32;
    for unit in value.encode_utf16() {
        hash = hash.wrapping_mul(31).wrapping_add(i32::from(unit));
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

    const CONTENT: &str = "
        4A37B57559F00C6EEA84EE9F026316135B45312748676A1B2640B13C2B844CBD
        A6B5ED97F43A5CF5BBAF00A7C8CD23C5AFC9BD003F849875AF8B36E6CF77D01D

        59EF7D515A653E1B04DEF778EAC675F715DCE14DD506E8F39E2C68ECD2893987
          FF76AF20C7ACF327FF2A28FB2DBD6694E3F946503E72635A5F7B6CB2E64FC014
        C6AF31A9C24D9A3B71E94C0FE0FDCF6C18C7BF8AEF5C095512AC65B5ECEBA933
        67F8733CBDCEC008EC7038CAE5E9199DB53E00639FAC8A0A2A4E86822566A8A8
        21090B930F00D2C23D05BBC1014EBA1283C27253033EA73D2CAA47EA34632570
        C5CCF591F1676C87DFC4AD7EE FCD7B4E3DE1A769EA359ABC0823926D1CD1C583

        ";

    #[test]
    fn old_and_new_parse_same_set() {
        assert_eq!(parse_old(CONTENT), parse_new(CONTENT));
    }

    #[test]
    fn old_and_new_summaries_match() {
        assert_eq!(old_stream_summary(128, CONTENT), new_loop_summary(128, CONTENT));
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            new_loop_summary(0, CONTENT),
            RemapperSkipHashesSummary::default(),
        );
    }
}
