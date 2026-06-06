pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginNameLogSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_treeset_summary(
    iterations: usize,
    paper_names: &[String],
    bukkit_names: &[String],
) -> PluginNameLogSummary {
    run_summary(iterations, paper_names, bukkit_names, false)
}

pub fn new_arraylist_sort_summary(
    iterations: usize,
    paper_names: &[String],
    bukkit_names: &[String],
) -> PluginNameLogSummary {
    run_summary(iterations, paper_names, bukkit_names, true)
}

fn run_summary(
    iterations: usize,
    paper_names: &[String],
    bukkit_names: &[String],
    optimized: bool,
) -> PluginNameLogSummary {
    if iterations == 0 {
        return PluginNameLogSummary::default();
    }

    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = if optimized {
            new_run(paper_names, bukkit_names)
        } else {
            old_run(paper_names, bukkit_names)
        } as u64;
        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((paper_names.len() as u64) << 1)
                ^ ((bukkit_names.len() as u64) << 17),
        );
    }

    PluginNameLogSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn old_run(paper_names: &[String], bukkit_names: &[String]) -> usize {
    let mut paper = std::collections::BTreeSet::new();
    let mut bukkit = std::collections::BTreeSet::new();
    paper.extend(paper_names.iter().cloned());
    bukkit.extend(bukkit_names.iter().cloned());
    paper.len() + bukkit.len() + paper.first().map_or(0, |value| value.len()) + bukkit.last().map_or(0, |value| value.len())
}

fn new_run(paper_names: &[String], bukkit_names: &[String]) -> usize {
    let mut paper = paper_names.to_vec();
    let mut bukkit = bukkit_names.to_vec();
    sort_and_deduplicate(&mut paper);
    sort_and_deduplicate(&mut bukkit);
    paper.len() + bukkit.len() + paper.first().map_or(0, |value| value.len()) + bukkit.last().map_or(0, |value| value.len())
}

fn sort_and_deduplicate(names: &mut Vec<String>) {
    names.sort_unstable();
    names.dedup();
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
        let paper = create_names("Paper", 64);
        let bukkit = create_names("Bukkit", 448);
        assert_eq!(old_treeset_summary(256, &paper, &bukkit), new_arraylist_sort_summary(256, &paper, &bukkit));
    }

    #[test]
    fn zero_iterations_are_empty() {
        let paper = create_names("Paper", 8);
        let bukkit = create_names("Bukkit", 8);
        assert_eq!(old_treeset_summary(0, &paper, &bukkit), PluginNameLogSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let paper = create_names("Paper", 16);
        let bukkit = create_names("Bukkit", 16);
        assert_eq!(new_arraylist_sort_summary(64, &paper, &bukkit), new_arraylist_sort_summary(64, &paper, &bukkit));
    }
}
