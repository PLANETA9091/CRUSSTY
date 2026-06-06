pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginNameJoinSummary {
    pub count: u64,
    pub total_length: u64,
    pub checksum: u64,
    pub last_hash: u64,
}

pub fn string_join_summary(
    iterations: usize,
    names: &[String],
    delimiter: &str,
) -> PluginNameJoinSummary {
    run_summary(iterations, names, delimiter, false)
}

pub fn manual_join_summary(
    iterations: usize,
    names: &[String],
    delimiter: &str,
) -> PluginNameJoinSummary {
    run_summary(iterations, names, delimiter, true)
}

fn run_summary(
    iterations: usize,
    names: &[String],
    delimiter: &str,
    manual: bool,
) -> PluginNameJoinSummary {
    if iterations == 0 {
        return PluginNameJoinSummary::default();
    }

    let delimiter_hash = java_string_hash(delimiter) as i64 as u64;
    let mut total_length = 0u64;
    let mut checksum = 0u64;
    let mut last_hash = 0u64;

    for iteration in 0..iterations {
        let joined = if manual {
            manual_join(names, delimiter)
        } else {
            names.join(delimiter)
        };
        let length = joined.len() as u64;
        let hash = java_string_hash(&joined) as i64 as u64;
        total_length += length;
        last_hash = hash;
        checksum = mix64(
            checksum
                ^ length
                ^ hash
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ delimiter_hash,
        );
    }

    PluginNameJoinSummary {
        count: iterations as u64,
        total_length,
        checksum,
        last_hash,
    }
}

fn manual_join(names: &[String], delimiter: &str) -> String {
    if names.is_empty() {
        return String::new();
    }
    if names.len() == 1 {
        return names[0].clone();
    }

    let mut result = String::with_capacity(names.iter().map(|value| value.len()).sum::<usize>() + delimiter.len() * (names.len() - 1));
    result.push_str(&names[0]);
    for name in &names[1..] {
        result.push_str(delimiter);
        result.push_str(name);
    }
    result
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

    #[test]
    fn string_and_manual_match_on_regular_inputs() {
        let names = vec![
            "Plugin000 (1.0.0)".to_string(),
            "Plugin001 (1.0.1)".to_string(),
            "Plugin002 (1.0.2)".to_string(),
        ];
        let string = string_join_summary(256, &names, ", ");
        let manual = manual_join_summary(256, &names, ", ");
        assert_eq!(string, manual);
    }

    #[test]
    fn empty_names_are_empty() {
        let names: Vec<String> = Vec::new();
        assert_eq!(string_join_summary(0, &names, ", "), PluginNameJoinSummary::default());
        assert_eq!(manual_join_summary(64, &names, ", ").total_length, 0);
    }

    #[test]
    fn repeated_runs_are_stable() {
        let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let first = string_join_summary(128, &names, "\n - ");
        let second = string_join_summary(128, &names, "\n - ");
        assert_eq!(first, second);
    }
}
