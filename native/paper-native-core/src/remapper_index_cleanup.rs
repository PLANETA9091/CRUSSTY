use std::collections::{HashMap, HashSet};

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const CLEANUP_TAG: u64 = 0xA73C_D8E2_917B_4D35;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RemapperIndexCleanupSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_eager_cleanup_summary(
    iterations: usize,
    input_paths: &[String],
    input_hashes: &[String],
    remapped_hashes: &[String],
    remapped_paths: &[String],
    skipped_hashes: &[String],
) -> RemapperIndexCleanupSummary {
    run_summary(
        iterations,
        input_paths,
        input_hashes,
        remapped_hashes,
        remapped_paths,
        skipped_hashes,
        Mode::OldEagerCleanup,
    )
}

pub fn new_lazy_cleanup_summary(
    iterations: usize,
    input_paths: &[String],
    input_hashes: &[String],
    remapped_hashes: &[String],
    remapped_paths: &[String],
    skipped_hashes: &[String],
) -> RemapperIndexCleanupSummary {
    run_summary(
        iterations,
        input_paths,
        input_hashes,
        remapped_hashes,
        remapped_paths,
        skipped_hashes,
        Mode::NewLazyCleanup,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldEagerCleanup,
    NewLazyCleanup,
}

fn run_summary(
    iterations: usize,
    input_paths: &[String],
    input_hashes: &[String],
    remapped_hashes: &[String],
    remapped_paths: &[String],
    skipped_hashes: &[String],
    mode: Mode,
) -> RemapperIndexCleanupSummary {
    if iterations == 0 {
        return RemapperIndexCleanupSummary::default();
    }

    debug_assert_eq!(input_paths.len(), input_hashes.len());
    debug_assert_eq!(remapped_hashes.len(), remapped_paths.len());

    let shape_digest = input_digest(
        input_paths,
        input_hashes,
        remapped_hashes,
        remapped_paths,
        skipped_hashes,
    );
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = match mode {
            Mode::OldEagerCleanup => old_eager_cleanup_once(
                input_paths,
                input_hashes,
                remapped_hashes,
                remapped_paths,
                skipped_hashes,
            ),
            Mode::NewLazyCleanup => new_lazy_cleanup_once(
                input_paths,
                input_hashes,
                remapped_hashes,
                remapped_paths,
                skipped_hashes,
            ),
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((input_paths.len() as u64) << 7)
                ^ ((remapped_hashes.len() as u64) << 23)
                ^ ((skipped_hashes.len() as u64) << 41),
        );
    }

    RemapperIndexCleanupSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn old_eager_cleanup_once(
    input_paths: &[String],
    input_hashes: &[String],
    remapped_hashes: &[String],
    remapped_paths: &[String],
    skipped_hashes: &[String],
) -> usize {
    let hash_cache = hash_cache(input_paths, input_hashes);
    let input_hash_set = hash_cache.values().copied().collect::<HashSet<_>>();
    let remapped = remapped_map(remapped_hashes, remapped_paths);
    let skipped = skipped_set(skipped_hashes);
    let mut result = 0usize;

    for (hash, path) in &remapped {
        if input_hash_set.contains(hash) {
            result += path.len();
        }
    }
    for hash in &skipped {
        if input_hash_set.contains(hash) {
            result += 1;
        }
    }
    for path in input_paths {
        if let Some(hash) = hash_cache.get(path.as_str()) {
            if skipped.contains(hash) {
                result += 1;
            } else if remapped.contains_key(hash) {
                result += 2;
            }
        }
    }

    result
}

fn new_lazy_cleanup_once(
    input_paths: &[String],
    input_hashes: &[String],
    remapped_hashes: &[String],
    remapped_paths: &[String],
    skipped_hashes: &[String],
) -> usize {
    let hash_cache = hash_cache(input_paths, input_hashes);
    let remapped = remapped_map(remapped_hashes, remapped_paths);
    let skipped = skipped_set(skipped_hashes);
    let mut result = 0usize;

    if remapped.len() + skipped.len() != hash_cache.len() {
        result = result.saturating_sub(1);
    }
    for path in input_paths {
        if let Some(hash) = hash_cache.get(path.as_str()) {
            if skipped.contains(hash) {
                result += 1;
            } else if remapped.contains_key(hash) {
                result += 2;
            }
        }
    }

    result
}

fn hash_cache<'a>(input_paths: &'a [String], input_hashes: &'a [String]) -> HashMap<&'a str, &'a str> {
    let mut cache = HashMap::with_capacity(expected_collection_capacity(input_paths.len()));
    for (path, hash) in input_paths.iter().zip(input_hashes) {
        cache.insert(path.as_str(), hash.as_str());
    }
    cache
}

fn remapped_map<'a>(
    remapped_hashes: &'a [String],
    remapped_paths: &'a [String],
) -> HashMap<&'a str, &'a str> {
    let mut remapped = HashMap::with_capacity(expected_collection_capacity(remapped_hashes.len()));
    for (hash, path) in remapped_hashes.iter().zip(remapped_paths) {
        remapped.insert(hash.as_str(), path.as_str());
    }
    remapped
}

fn skipped_set(skipped_hashes: &[String]) -> HashSet<&str> {
    let mut skipped = HashSet::with_capacity(expected_collection_capacity(skipped_hashes.len()));
    for hash in skipped_hashes {
        skipped.insert(hash.as_str());
    }
    skipped
}

fn input_digest(
    input_paths: &[String],
    input_hashes: &[String],
    remapped_hashes: &[String],
    remapped_paths: &[String],
    skipped_hashes: &[String],
) -> u64 {
    mix64(
        CLEANUP_TAG
            ^ strings_digest(input_paths, 0x1656_67B1_9E37_79F9)
            ^ strings_digest(input_hashes, 0x85EB_CA77_C2B2_AE63)
            ^ strings_digest(remapped_hashes, 0x27D4_EB2F_1656_67C5)
            ^ strings_digest(remapped_paths, 0x94D0_49BB_1331_11EB)
            ^ strings_digest(skipped_hashes, 0xD6E8_FD93_59A1_2B4D),
    )
}

fn strings_digest(values: &[String], tag: u64) -> u64 {
    let mut digest = mix64(tag ^ (values.len() as u64));
    for (index, value) in values.iter().enumerate() {
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

fn expected_collection_capacity(expected_size: usize) -> usize {
    if expected_size < 3 {
        expected_size + 1
    } else if expected_size < (1 << 30) {
        (expected_size as f32 / 0.75_f32 + 1.0_f32) as usize
    } else {
        usize::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        let mut inputs = Vec::new();
        let mut hashes = Vec::new();
        let mut remapped_hashes = Vec::new();
        let mut remapped_paths = Vec::new();
        let mut skipped_hashes = Vec::new();
        for index in 0..12 {
            let path = format!("plugin-{index}.jar");
            let hash = hex_hash(index);
            inputs.push(path);
            hashes.push(hash.clone());
            if index < 4 {
                remapped_hashes.push(hash);
                remapped_paths.push(format!("plugin-{index}-remapped.jar"));
            } else {
                skipped_hashes.push(hash);
            }
        }
        (inputs, hashes, remapped_hashes, remapped_paths, skipped_hashes)
    }

    #[test]
    fn old_and_new_are_stable_on_all_cached_shape() {
        let (inputs, hashes, remapped_hashes, remapped_paths, skipped_hashes) = fixture();
        let old = old_eager_cleanup_summary(
            128,
            &inputs,
            &hashes,
            &remapped_hashes,
            &remapped_paths,
            &skipped_hashes,
        );
        let new = new_lazy_cleanup_summary(
            128,
            &inputs,
            &hashes,
            &remapped_hashes,
            &remapped_paths,
            &skipped_hashes,
        );
        assert_eq!(
            old,
            old_eager_cleanup_summary(
                128,
                &inputs,
                &hashes,
                &remapped_hashes,
                &remapped_paths,
                &skipped_hashes,
            )
        );
        assert_eq!(
            new,
            new_lazy_cleanup_summary(
                128,
                &inputs,
                &hashes,
                &remapped_hashes,
                &remapped_paths,
                &skipped_hashes,
            )
        );
        assert!(old.last_total > new.last_total);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (inputs, hashes, remapped_hashes, remapped_paths, skipped_hashes) = fixture();
        assert_eq!(
            old_eager_cleanup_summary(
                0,
                &inputs,
                &hashes,
                &remapped_hashes,
                &remapped_paths,
                &skipped_hashes,
            ),
            RemapperIndexCleanupSummary::default(),
        );
    }

    fn hex_hash(value: usize) -> String {
        let repeated = format!("{:X}", 0x10000000usize | value);
        repeated.repeat(8)[..64].to_string()
    }
}
