use std::collections::HashSet;

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const BUILD_TAG: u64 = 0xD4F5_25B8_25AA_7B19;
const REMOVE_TAG: u64 = 0x84EB_4B7C_D6D8_43A5;
const PLUGIN_NAME: &str = "TargetPlugin";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpigotLoadOrderDependencySummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_load_after_build_summary(iterations: usize, load_after: &[String]) -> SpigotLoadOrderDependencySummary {
    run_summary(
        iterations,
        load_after,
        &[],
        &[],
        0,
        Mode::OldLoadAfterBuild,
    )
}

pub fn new_load_after_build_summary(iterations: usize, load_after: &[String]) -> SpigotLoadOrderDependencySummary {
    run_summary(
        iterations,
        load_after,
        &[],
        &[],
        0,
        Mode::NewLoadAfterBuild,
    )
}

pub fn old_removed_count_summary(
    iterations: usize,
    provider_names: &[String],
    hard_dependencies: &[String],
    soft_dependencies: &[String],
    dependencies_per_provider: usize,
) -> SpigotLoadOrderDependencySummary {
    run_summary(
        iterations,
        provider_names,
        hard_dependencies,
        soft_dependencies,
        dependencies_per_provider,
        Mode::OldRemovedCount,
    )
}

pub fn new_removed_count_summary(
    iterations: usize,
    provider_names: &[String],
    hard_dependencies: &[String],
    soft_dependencies: &[String],
    dependencies_per_provider: usize,
) -> SpigotLoadOrderDependencySummary {
    run_summary(
        iterations,
        provider_names,
        hard_dependencies,
        soft_dependencies,
        dependencies_per_provider,
        Mode::NewRemovedCount,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldLoadAfterBuild,
    NewLoadAfterBuild,
    OldRemovedCount,
    NewRemovedCount,
}

impl Mode {
    fn tag(self) -> u64 {
        match self {
            Mode::OldLoadAfterBuild | Mode::NewLoadAfterBuild => BUILD_TAG,
            Mode::OldRemovedCount | Mode::NewRemovedCount => REMOVE_TAG,
        }
    }
}

fn run_summary(
    iterations: usize,
    primary: &[String],
    hard_dependencies: &[String],
    soft_dependencies: &[String],
    dependencies_per_provider: usize,
    mode: Mode,
) -> SpigotLoadOrderDependencySummary {
    if iterations == 0 {
        return SpigotLoadOrderDependencySummary::default();
    }

    match mode {
        Mode::OldLoadAfterBuild | Mode::NewLoadAfterBuild => {}
        Mode::OldRemovedCount | Mode::NewRemovedCount => {
            debug_assert_eq!(primary.len(), hard_dependencies.len() / dependencies_per_provider);
            debug_assert_eq!(primary.len(), soft_dependencies.len() / dependencies_per_provider);
            debug_assert_eq!(
                hard_dependencies.len(),
                soft_dependencies.len()
            );
        }
    }

    let shape_digest = match mode {
        Mode::OldLoadAfterBuild | Mode::NewLoadAfterBuild => {
            strings_digest(primary, mode.tag())
        }
        Mode::OldRemovedCount | Mode::NewRemovedCount => input_digest(
            mode.tag(),
            primary,
            hard_dependencies,
            soft_dependencies,
            dependencies_per_provider,
        ),
    };

    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = match mode {
            Mode::OldLoadAfterBuild => old_load_after_build_once(primary),
            Mode::NewLoadAfterBuild => new_load_after_build_once(primary),
            Mode::OldRemovedCount => old_removed_count_once(
                primary,
                hard_dependencies,
                soft_dependencies,
                dependencies_per_provider,
            ),
            Mode::NewRemovedCount => new_removed_count_once(
                primary,
                hard_dependencies,
                soft_dependencies,
                dependencies_per_provider,
            ),
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((primary.len() as u64) << 7)
                ^ ((dependencies_per_provider as u64) << 23),
        );
    }

    SpigotLoadOrderDependencySummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn old_load_after_build_once(load_after: &[String]) -> usize {
    let mut built = Vec::new();
    built.extend(load_after.iter().cloned());
    built.extend(load_after.iter().cloned());
    built.len()
}

fn new_load_after_build_once(load_after: &[String]) -> usize {
    let mut built = Vec::with_capacity(load_after.len() * 2);
    built.extend(load_after.iter().cloned());
    built.extend(load_after.iter().cloned());
    built.len()
}

fn old_removed_count_once(
    provider_names: &[String],
    hard_dependencies: &[String],
    soft_dependencies: &[String],
    dependencies_per_provider: usize,
) -> usize {
    let mut removed = 0usize;
    for provider_index in 0..provider_names.len() {
        let mut dependencies = HashSet::new();
        for dependency in dependency_slice(hard_dependencies, provider_index, dependencies_per_provider) {
            dependencies.insert(dependency.as_str());
        }
        for dependency in dependency_slice(soft_dependencies, provider_index, dependencies_per_provider) {
            dependencies.insert(dependency.as_str());
        }

        let provider_name = provider_names[provider_index].as_str();
        if provider_name == PLUGIN_NAME || dependencies.contains(PLUGIN_NAME) {
            removed += 1;
        }
    }
    removed
}

fn new_removed_count_once(
    provider_names: &[String],
    hard_dependencies: &[String],
    soft_dependencies: &[String],
    dependencies_per_provider: usize,
) -> usize {
    let mut removed = 0usize;
    for provider_index in 0..provider_names.len() {
        let provider_name = provider_names[provider_index].as_str();
        if provider_name == PLUGIN_NAME
            || dependency_slice(hard_dependencies, provider_index, dependencies_per_provider)
                .iter()
                .any(|dependency| dependency.as_str() == PLUGIN_NAME)
            || dependency_slice(soft_dependencies, provider_index, dependencies_per_provider)
                .iter()
                .any(|dependency| dependency.as_str() == PLUGIN_NAME)
        {
            removed += 1;
        }
    }
    removed
}

fn dependency_slice(
    dependencies: &[String],
    provider_index: usize,
    dependencies_per_provider: usize,
) -> &[String] {
    let start = provider_index * dependencies_per_provider;
    &dependencies[start..start + dependencies_per_provider]
}

fn input_digest(
    tag: u64,
    provider_names: &[String],
    hard_dependencies: &[String],
    soft_dependencies: &[String],
    dependencies_per_provider: usize,
) -> u64 {
    mix64(
        tag
            ^ strings_digest(provider_names, 0x1656_67B1_9E37_79F9)
            ^ strings_digest(hard_dependencies, 0x85EB_CA77_C2B2_AE63)
            ^ strings_digest(soft_dependencies, 0x27D4_EB2F_1656_67C5)
            ^ ((dependencies_per_provider as u64) << 37),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_load_after(count: usize) -> Vec<String> {
        (0..count).map(|index| format!("Dependency{index}")).collect()
    }

    fn create_providers(count: usize, dependencies_per_provider: usize) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut provider_names = Vec::with_capacity(count);
        let mut hard = Vec::with_capacity(count * dependencies_per_provider);
        let mut soft = Vec::with_capacity(count * dependencies_per_provider);
        for provider_index in 0..count {
            provider_names.push(format!("Plugin{provider_index}"));
            for dependency_index in 0..dependencies_per_provider {
                hard.push(format!("Hard{provider_index}_{dependency_index}"));
                soft.push(format!("Soft{provider_index}_{dependency_index}"));
            }
            let last = provider_index * dependencies_per_provider + dependencies_per_provider - 1;
            if (provider_index & 3) == 0 {
                hard[last] = PLUGIN_NAME.to_string();
            } else if (provider_index & 3) == 1 {
                soft[last] = PLUGIN_NAME.to_string();
            }
        }
        (provider_names, hard, soft)
    }

    #[test]
    fn build_shapes_match_on_regular_inputs() {
        let load_after = create_load_after(16);
        assert_eq!(
            old_load_after_build_summary(64, &load_after),
            new_load_after_build_summary(64, &load_after),
        );
    }

    #[test]
    fn removed_count_shapes_match_on_regular_inputs() {
        let (provider_names, hard, soft) = create_providers(32, 6);
        assert_eq!(
            old_removed_count_summary(64, &provider_names, &hard, &soft, 6),
            new_removed_count_summary(64, &provider_names, &hard, &soft, 6),
        );
    }

    #[test]
    fn removed_count_handles_name_match_and_dependency_match() {
        let provider_names = vec![
            "TargetPlugin".to_string(),
            "Plugin1".to_string(),
            "Plugin2".to_string(),
        ];
        let hard = vec![
            "Hard0".to_string(),
            "Other0".to_string(),
            "Hard1".to_string(),
            "TargetPlugin".to_string(),
            "Hard2".to_string(),
            "Other2".to_string(),
        ];
        let soft = vec![
            "Soft0".to_string(),
            "Other0".to_string(),
            "Soft1".to_string(),
            "Other1".to_string(),
            "Soft2".to_string(),
            "TargetPlugin".to_string(),
        ];
        assert_eq!(
            old_removed_count_summary(16, &provider_names, &hard, &soft, 2),
            new_removed_count_summary(16, &provider_names, &hard, &soft, 2),
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        let load_after = create_load_after(8);
        assert_eq!(
            old_load_after_build_summary(0, &load_after),
            SpigotLoadOrderDependencySummary::default(),
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (provider_names, hard, soft) = create_providers(24, 4);
        let first = new_removed_count_summary(48, &provider_names, &hard, &soft, 4);
        let second = new_removed_count_summary(48, &provider_names, &hard, &soft, 4);
        assert_eq!(first, second);
    }
}
