use std::collections::{HashMap, HashSet};

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SETUP_TAG: u64 = 0xD6E8_FD93_59A1_2B4D;
const MISSING_TAG: u64 = 0xA24B_AED4_9C3B_1F15;
const VALIDATE_TAG: u64 = 0xC2B2_AE3D_27D4_EB4F;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginLoadingAllocationSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_default_capacity_setup_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> PluginLoadingAllocationSummary {
    run_summary(
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        Mode::OldDefaultCapacitySetup,
    )
}

pub fn new_presized_setup_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> PluginLoadingAllocationSummary {
    run_summary(
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        Mode::NewPresizedSetup,
    )
}

pub fn old_eager_missing_set_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> PluginLoadingAllocationSummary {
    run_summary(
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        Mode::OldEagerMissingSet,
    )
}

pub fn new_lazy_missing_set_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> PluginLoadingAllocationSummary {
    run_summary(
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        Mode::NewLazyMissingSet,
    )
}

pub fn old_eager_validate_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> PluginLoadingAllocationSummary {
    run_summary(
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        Mode::OldEagerValidate,
    )
}

pub fn new_lazy_validate_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> PluginLoadingAllocationSummary {
    run_summary(
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        Mode::NewLazyValidate,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldDefaultCapacitySetup,
    NewPresizedSetup,
    OldEagerMissingSet,
    NewLazyMissingSet,
    OldEagerValidate,
    NewLazyValidate,
}

impl Mode {
    fn tag(self) -> u64 {
        match self {
            Mode::OldDefaultCapacitySetup | Mode::NewPresizedSetup => SETUP_TAG,
            Mode::OldEagerMissingSet | Mode::NewLazyMissingSet => MISSING_TAG,
            Mode::OldEagerValidate | Mode::NewLazyValidate => VALIDATE_TAG,
        }
    }
}

fn run_summary(
    iterations: usize,
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
    mode: Mode,
) -> PluginLoadingAllocationSummary {
    if iterations == 0 {
        return PluginLoadingAllocationSummary::default();
    }

    debug_assert_eq!(provider_names.len(), provided_aliases.len());
    debug_assert_eq!(
        dependencies.len(),
        provider_names.len() * dependencies_per_provider
    );

    let provider_name_hashes = provider_names
        .iter()
        .map(|name| java_string_hash(name))
        .collect::<Vec<_>>();
    let shape_digest = input_digest(
        mode.tag(),
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
    );
    let present = match mode {
        Mode::OldEagerValidate | Mode::NewLazyValidate => {
            Some(build_present_set(provider_names, provided_aliases))
        }
        _ => None,
    };

    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = match mode {
            Mode::OldDefaultCapacitySetup => load_order_setup_once(
                provider_names,
                provided_aliases,
                &provider_name_hashes,
                false,
            ),
            Mode::NewPresizedSetup => load_order_setup_once(
                provider_names,
                provided_aliases,
                &provider_name_hashes,
                true,
            ),
            Mode::OldEagerMissingSet => missing_dependency_scan_once(
                provider_names,
                provided_aliases,
                dependencies,
                dependencies_per_provider,
                false,
            ),
            Mode::NewLazyMissingSet => missing_dependency_scan_once(
                provider_names,
                provided_aliases,
                dependencies,
                dependencies_per_provider,
                true,
            ),
            Mode::OldEagerValidate => validate_no_miss_once(
                provider_names,
                dependencies,
                dependencies_per_provider,
                present.as_ref().expect("present set"),
                false,
            ),
            Mode::NewLazyValidate => validate_no_miss_once(
                provider_names,
                dependencies,
                dependencies_per_provider,
                present.as_ref().expect("present set"),
                true,
            ),
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((provider_names.len() as u64) << 7)
                ^ ((dependencies_per_provider as u64) << 23),
        );
    }

    PluginLoadingAllocationSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn load_order_setup_once(
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    provider_name_hashes: &[i32],
    presized: bool,
) -> usize {
    let provider_count = provider_names.len();
    let expected_capacity = expected_collection_capacity(provider_count);
    let mut loaded = if presized {
        Vec::with_capacity(provider_count)
    } else {
        Vec::new()
    };
    let mut providers_to_load = if presized {
        HashMap::with_capacity(expected_capacity)
    } else {
        HashMap::new()
    };
    let mut loaded_plugins = if presized {
        HashSet::with_capacity(expected_capacity)
    } else {
        HashSet::new()
    };
    let mut plugins_provided = if presized {
        HashMap::with_capacity(expected_capacity)
    } else {
        HashMap::new()
    };
    let mut dependencies = if presized {
        HashMap::with_capacity(expected_capacity)
    } else {
        HashMap::new()
    };
    let mut soft_dependencies = if presized {
        HashMap::with_capacity(expected_capacity)
    } else {
        HashMap::new()
    };

    for (index, name) in provider_names.iter().enumerate() {
        let name = name.as_str();
        providers_to_load.insert(name, index);
        if let Some(alias) = provided_aliases[index].as_deref() {
            plugins_provided.insert(alias, name);
        }
        dependencies.insert(name, index);
        if (provider_name_hashes[index] & 3) == 0 {
            soft_dependencies.insert(name, index);
        }
        loaded_plugins.insert(name);
        loaded.push(name);
    }

    loaded.len()
        + providers_to_load.len()
        + loaded_plugins.len()
        + plugins_provided.len()
        + dependencies.len()
        + soft_dependencies.len()
}

fn missing_dependency_scan_once(
    provider_names: &[String],
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
    lazy_missing_set: bool,
) -> usize {
    let provider_count = provider_names.len();
    let mut providers_to_load = HashSet::with_capacity(expected_collection_capacity(provider_count));
    let loaded_plugins = HashSet::<&str>::new();
    let mut plugins_provided = HashMap::new();

    for (index, name) in provider_names.iter().enumerate() {
        let name = name.as_str();
        providers_to_load.insert(name);
        if let Some(alias) = provided_aliases[index].as_deref() {
            plugins_provided.insert(alias, name);
        }
    }

    let mut missing = 0usize;
    for provider_index in 0..provider_count {
        if lazy_missing_set {
            let mut missing_hard_dependencies = None;
            for dependency in dependency_slice(dependencies, provider_index, dependencies_per_provider) {
                let dependency = dependency.as_str();
                if loaded_plugins.contains(dependency) {
                    continue;
                }
                if !providers_to_load.contains(dependency)
                    && !plugins_provided.contains_key(dependency)
                {
                    missing_hard_dependencies
                        .get_or_insert_with(|| {
                            HashSet::with_capacity(expected_collection_capacity(
                                dependencies_per_provider,
                            ))
                        })
                        .insert(dependency);
                }
            }
            if let Some(missing_hard_dependencies) = missing_hard_dependencies {
                missing += missing_hard_dependencies.len();
            }
        } else {
            let mut missing_hard_dependencies =
                HashSet::with_capacity(expected_collection_capacity(dependencies_per_provider));
            for dependency in dependency_slice(dependencies, provider_index, dependencies_per_provider) {
                let dependency = dependency.as_str();
                if loaded_plugins.contains(dependency) {
                    continue;
                }
                if !providers_to_load.contains(dependency)
                    && !plugins_provided.contains_key(dependency)
                {
                    missing_hard_dependencies.insert(dependency);
                }
            }
            missing += missing_hard_dependencies.len();
        }
    }

    missing
}

fn validate_no_miss_once(
    provider_names: &[String],
    dependencies: &[String],
    dependencies_per_provider: usize,
    present: &HashSet<&str>,
    lazy_missing_list: bool,
) -> usize {
    let mut missing = 0usize;
    for provider_index in 0..provider_names.len() {
        if lazy_missing_list {
            let mut missing_dependencies = None;
            for dependency in dependency_slice(dependencies, provider_index, dependencies_per_provider) {
                let dependency = dependency.as_str();
                if !present.contains(dependency) {
                    missing_dependencies
                        .get_or_insert_with(|| {
                            Vec::with_capacity(expected_collection_capacity(
                                dependencies_per_provider,
                            ))
                        })
                        .push(dependency);
                }
            }
            if let Some(missing_dependencies) = missing_dependencies {
                missing += missing_dependencies.len();
            }
        } else {
            let mut missing_dependencies = Vec::new();
            for dependency in dependency_slice(dependencies, provider_index, dependencies_per_provider) {
                let dependency = dependency.as_str();
                if !present.contains(dependency) {
                    missing_dependencies.push(dependency);
                }
            }
            missing += missing_dependencies.len();
        }
    }
    missing
}

fn build_present_set<'a>(
    provider_names: &'a [String],
    provided_aliases: &'a [Option<String>],
) -> HashSet<&'a str> {
    let mut present = HashSet::with_capacity(expected_collection_capacity(provider_names.len() * 2));
    for (index, name) in provider_names.iter().enumerate() {
        present.insert(name.as_str());
        if let Some(alias) = provided_aliases[index].as_deref() {
            present.insert(alias);
        }
    }
    present
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
    provided_aliases: &[Option<String>],
    dependencies: &[String],
    dependencies_per_provider: usize,
) -> u64 {
    mix64(
        tag
            ^ strings_digest(provider_names, 0x1656_67B1_9E37_79F9)
            ^ optional_strings_digest(provided_aliases, 0x85EB_CA77_C2B2_AE63)
            ^ strings_digest(dependencies, 0x27D4_EB2F_1656_67C5)
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

fn optional_strings_digest(values: &[Option<String>], tag: u64) -> u64 {
    let mut digest = mix64(tag ^ (values.len() as u64));
    for (index, value) in values.iter().enumerate() {
        let hash = value
            .as_ref()
            .map_or(0xFFFF_FFFFu64, |value| java_string_hash(value) as i64 as u64);
        digest = mix64(digest ^ hash ^ ((index as u64).wrapping_mul(MIX_GAMMA)));
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

    fn create_providers(count: usize, dependencies_per_provider: usize) -> (Vec<String>, Vec<Option<String>>, Vec<String>) {
        let mut provider_names = Vec::with_capacity(count);
        let mut provided_aliases = Vec::with_capacity(count);
        let mut dependencies = Vec::with_capacity(count * dependencies_per_provider);
        for provider_index in 0..count {
            provider_names.push(format!("Plugin{provider_index}"));
            provided_aliases.push(if (provider_index & 7) == 0 {
                Some(format!("Alias{provider_index}"))
            } else {
                None
            });
            for dependency_index in 0..dependencies_per_provider {
                dependencies.push(format!(
                    "Plugin{}",
                    (provider_index + count - dependency_index - 1) % count
                ));
            }
        }
        (provider_names, provided_aliases, dependencies)
    }

    #[test]
    fn old_new_setup_match_on_regular_inputs() {
        let (provider_names, provided_aliases, dependencies) = create_providers(64, 4);
        assert_eq!(
            old_default_capacity_setup_summary(128, &provider_names, &provided_aliases, &dependencies, 4),
            new_presized_setup_summary(128, &provider_names, &provided_aliases, &dependencies, 4),
        );
    }

    #[test]
    fn eager_and_lazy_missing_paths_match_with_hits_and_misses() {
        let provider_names = vec![
            "Plugin0".to_string(),
            "Plugin1".to_string(),
            "Plugin2".to_string(),
        ];
        let provided_aliases = vec![Some("Alias0".to_string()), None, None];
        let dependencies = vec![
            "Plugin1".to_string(),
            "Missing0".to_string(),
            "Alias0".to_string(),
            "Missing1".to_string(),
            "Plugin2".to_string(),
            "Missing2".to_string(),
        ];
        assert_eq!(
            old_eager_missing_set_summary(96, &provider_names, &provided_aliases, &dependencies, 2),
            new_lazy_missing_set_summary(96, &provider_names, &provided_aliases, &dependencies, 2),
        );
        assert_eq!(
            old_eager_validate_summary(96, &provider_names, &provided_aliases, &dependencies, 2),
            new_lazy_validate_summary(96, &provider_names, &provided_aliases, &dependencies, 2),
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (provider_names, provided_aliases, dependencies) = create_providers(8, 2);
        assert_eq!(
            old_default_capacity_setup_summary(0, &provider_names, &provided_aliases, &dependencies, 2),
            PluginLoadingAllocationSummary::default(),
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (provider_names, provided_aliases, dependencies) = create_providers(24, 3);
        let first = new_lazy_validate_summary(64, &provider_names, &provided_aliases, &dependencies, 3);
        let second = new_lazy_validate_summary(64, &provider_names, &provided_aliases, &dependencies, 3);
        assert_eq!(first, second);
    }
}
