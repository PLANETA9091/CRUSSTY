pub const SUMMARY_FIELDS: usize = 5;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginClassLoaderGroupSummary {
    pub count: u64,
    pub result_sum: i64,
    pub attempts: u64,
    pub checksum: u64,
    pub last_result: i64,
}

pub fn old_lookup_summary(
    iterations: usize,
    loader_names: &[String],
    result_lengths: &[i32],
    requester_index: usize,
    query: &str,
) -> PluginClassLoaderGroupSummary {
    run_summary(
        iterations,
        loader_names,
        result_lengths,
        requester_index,
        query,
        false,
    )
}

pub fn skip_requester_lookup_summary(
    iterations: usize,
    loader_names: &[String],
    result_lengths: &[i32],
    requester_index: usize,
    query: &str,
) -> PluginClassLoaderGroupSummary {
    run_summary(
        iterations,
        loader_names,
        result_lengths,
        requester_index,
        query,
        true,
    )
}

fn run_summary(
    iterations: usize,
    loader_names: &[String],
    result_lengths: &[i32],
    requester_index: usize,
    query: &str,
    skip_requester: bool,
) -> PluginClassLoaderGroupSummary {
    if iterations == 0 {
        return PluginClassLoaderGroupSummary::default();
    }

    debug_assert_eq!(loader_names.len(), result_lengths.len());
    debug_assert!(requester_index < loader_names.len());
    let requester_name_len = loader_names[requester_index].len() as u64;

    let mut result_sum = 0i64;
    let mut attempts_total = 0u64;
    let mut checksum = 0u64;
    let mut last_result = 0i64;

    for iteration in 0..iterations {
        let lookup = lookup(
            loader_names,
            result_lengths,
            requester_index,
            query,
            skip_requester,
        );
        let result_value = lookup.result_length.unwrap_or(1) as i64;
        let result_index = lookup.result_index.map_or(u64::MAX, |index| index as u64);

        result_sum += result_value;
        attempts_total += lookup.attempts;
        last_result = result_value;
        checksum = mix64(
            checksum
                ^ (result_value as u64)
                ^ result_index
                ^ (lookup.attempts << 32)
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((loader_names.len() as u64) << 7)
                ^ (requester_name_len << 23),
        );
    }

    PluginClassLoaderGroupSummary {
        count: iterations as u64,
        result_sum,
        attempts: attempts_total,
        checksum,
        last_result,
    }
}

fn lookup(
    loader_names: &[String],
    result_lengths: &[i32],
    requester_index: usize,
    query: &str,
    skip_requester: bool,
) -> LookupResult {
    let mut attempts = 1u64;
    if loader_names[requester_index] == query {
        return LookupResult {
            result_index: Some(requester_index),
            result_length: Some(result_lengths[requester_index]),
            attempts,
        };
    }

    for (index, name) in loader_names.iter().enumerate() {
        if skip_requester && index == requester_index {
            continue;
        }

        attempts += 1;
        if name == query {
            return LookupResult {
                result_index: Some(index),
                result_length: Some(result_lengths[index]),
                attempts,
            };
        }
    }

    LookupResult {
        result_index: None,
        result_length: None,
        attempts,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LookupResult {
    result_index: Option<usize>,
    result_length: Option<i32>,
    attempts: u64,
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

    fn create_loaders(count: usize) -> (Vec<String>, Vec<i32>) {
        let mut loader_names = Vec::with_capacity(count);
        let mut result_lengths = Vec::with_capacity(count);
        loader_names.push("bench.RequesterClass".to_string());
        result_lengths.push("java.lang.String".len() as i32);

        for index in 1..count {
            loader_names.push(if index == count / 2 {
                "bench.OtherClass".to_string()
            } else {
                format!("bench.Loader{index}")
            });
            result_lengths.push("java.lang.Integer".len() as i32);
        }

        (loader_names, result_lengths)
    }

    #[test]
    fn hit_requester_matches_between_shapes() {
        let (loader_names, result_lengths) = create_loaders(16);
        assert_eq!(
            old_lookup_summary(
                256,
                &loader_names,
                &result_lengths,
                0,
                "bench.RequesterClass"
            ),
            skip_requester_lookup_summary(
                256,
                &loader_names,
                &result_lengths,
                0,
                "bench.RequesterClass"
            )
        );
    }

    #[test]
    fn skip_requester_saves_one_attempt_on_miss_and_other_hit() {
        let (loader_names, result_lengths) = create_loaders(16);
        let old_miss = old_lookup_summary(
            128,
            &loader_names,
            &result_lengths,
            0,
            "bench.MissingClass",
        );
        let skip_miss = skip_requester_lookup_summary(
            128,
            &loader_names,
            &result_lengths,
            0,
            "bench.MissingClass",
        );
        let old_other = old_lookup_summary(
            128,
            &loader_names,
            &result_lengths,
            0,
            "bench.OtherClass",
        );
        let skip_other = skip_requester_lookup_summary(
            128,
            &loader_names,
            &result_lengths,
            0,
            "bench.OtherClass",
        );

        assert_eq!(old_miss.result_sum, skip_miss.result_sum);
        assert_eq!(old_other.result_sum, skip_other.result_sum);
        assert_eq!(old_miss.attempts - skip_miss.attempts, 128);
        assert_eq!(old_other.attempts - skip_other.attempts, 128);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (loader_names, result_lengths) = create_loaders(4);
        assert_eq!(
            old_lookup_summary(
                0,
                &loader_names,
                &result_lengths,
                0,
                "bench.MissingClass"
            ),
            PluginClassLoaderGroupSummary::default()
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (loader_names, result_lengths) = create_loaders(12);
        let first = skip_requester_lookup_summary(
            64,
            &loader_names,
            &result_lengths,
            0,
            "bench.OtherClass",
        );
        let second = skip_requester_lookup_summary(
            64,
            &loader_names,
            &result_lengths,
            0,
            "bench.OtherClass",
        );

        assert_eq!(first, second);
    }
}
