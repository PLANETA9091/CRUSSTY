use crate::{plugin_name_join, plugin_name_log};

pub const SUMMARY_FIELDS: usize = 8;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginStartupRollupSummary {
    pub count: u64,
    pub join_total_length: u64,
    pub log_total: u64,
    pub checksum: u64,
    pub join_last_hash: u64,
    pub log_last_total: u64,
    pub join_checksum: u64,
    pub log_checksum: u64,
}

pub fn old_summary(
    iterations: usize,
    names: &[String],
    delimiter: &str,
    paper_names: &[String],
    bukkit_names: &[String],
) -> PluginStartupRollupSummary {
    run_summary(iterations, names, delimiter, paper_names, bukkit_names, false, false)
}

pub fn new_summary(
    iterations: usize,
    names: &[String],
    delimiter: &str,
    paper_names: &[String],
    bukkit_names: &[String],
) -> PluginStartupRollupSummary {
    run_summary(iterations, names, delimiter, paper_names, bukkit_names, true, true)
}

fn run_summary(
    iterations: usize,
    names: &[String],
    delimiter: &str,
    paper_names: &[String],
    bukkit_names: &[String],
    manual_join: bool,
    optimized_log: bool,
) -> PluginStartupRollupSummary {
    if iterations == 0 {
        return PluginStartupRollupSummary::default();
    }

    let join = if manual_join {
        plugin_name_join::manual_join_summary(iterations, names, delimiter)
    } else {
        plugin_name_join::string_join_summary(iterations, names, delimiter)
    };
    let log = if optimized_log {
        plugin_name_log::new_arraylist_sort_summary(iterations, paper_names, bukkit_names)
    } else {
        plugin_name_log::old_treeset_summary(iterations, paper_names, bukkit_names)
    };

    let checksum = mix64(
        join.checksum
            ^ log.checksum
            ^ join.total_length
            ^ (log.total << 1)
            ^ join.last_hash
            ^ (log.last_total << 2)
            ^ ((iterations as u64).wrapping_mul(MIX_GAMMA)),
    );

    PluginStartupRollupSummary {
        count: iterations as u64,
        join_total_length: join.total_length,
        log_total: log.total,
        checksum,
        join_last_hash: join.last_hash,
        log_last_total: log.last_total,
        join_checksum: join.checksum,
        log_checksum: log.checksum,
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

    fn create_names(prefix: &str, count: usize) -> Vec<String> {
        let mut names = Vec::with_capacity(count + count / 16);
        for i in 0..count {
            let name = format!("{prefix}{:03}", i);
            names.push(name.clone());
            if (i & 15) == 0 {
                names.push(name);
            }
        }
        names
    }

    #[test]
    fn old_and_new_match_on_regular_inputs() {
        let names = create_names("Plugin", 64);
        let paper = create_names("Paper", 32);
        let bukkit = create_names("Bukkit", 96);
        assert_eq!(
            old_summary(256, &names, ", ", &paper, &bukkit),
            new_summary(256, &names, ", ", &paper, &bukkit)
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        let names = create_names("Plugin", 8);
        let paper = create_names("Paper", 8);
        let bukkit = create_names("Bukkit", 8);
        assert_eq!(
            old_summary(0, &names, ", ", &paper, &bukkit),
            PluginStartupRollupSummary::default()
        );
    }
}
