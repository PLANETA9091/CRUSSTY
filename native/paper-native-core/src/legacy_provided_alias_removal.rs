use std::collections::{HashMap, HashSet};

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const ALIAS_REMOVAL_TAG: u64 = 0x94D0_49BB_1331_11EB;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LegacyProvidedAliasRemovalSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_values_removeif_summary(
    iterations: usize,
    provider_names: &[String],
    aliases: &[String],
    aliases_per_provider: usize,
) -> LegacyProvidedAliasRemovalSummary {
    run_summary(
        iterations,
        provider_names,
        aliases,
        aliases_per_provider,
        Mode::OldValuesRemoveIf,
    )
}

pub fn new_reverse_alias_remove_summary(
    iterations: usize,
    provider_names: &[String],
    aliases: &[String],
    aliases_per_provider: usize,
) -> LegacyProvidedAliasRemovalSummary {
    run_summary(
        iterations,
        provider_names,
        aliases,
        aliases_per_provider,
        Mode::NewReverseAliasRemove,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldValuesRemoveIf,
    NewReverseAliasRemove,
}

fn run_summary(
    iterations: usize,
    provider_names: &[String],
    aliases: &[String],
    aliases_per_provider: usize,
    mode: Mode,
) -> LegacyProvidedAliasRemovalSummary {
    if iterations == 0 {
        return LegacyProvidedAliasRemovalSummary::default();
    }

    debug_assert_eq!(aliases.len(), provider_names.len() * aliases_per_provider);

    let shape_digest = input_digest(
        ALIAS_REMOVAL_TAG,
        provider_names,
        aliases,
        aliases_per_provider,
    );
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = match mode {
            Mode::OldValuesRemoveIf => {
                old_values_removeif_once(provider_names, aliases, aliases_per_provider)
            }
            Mode::NewReverseAliasRemove => {
                new_reverse_alias_remove_once(provider_names, aliases, aliases_per_provider)
            }
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((provider_names.len() as u64) << 7)
                ^ ((aliases_per_provider as u64) << 23),
        );
    }

    LegacyProvidedAliasRemovalSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn old_values_removeif_once(
    provider_names: &[String],
    aliases: &[String],
    aliases_per_provider: usize,
) -> usize {
    let mut plugins_provided =
        HashMap::with_capacity(expected_collection_capacity(aliases.len()));
    for (provider_index, provider_name) in provider_names.iter().enumerate() {
        let provider_name = provider_name.as_str();
        for alias in alias_slice(aliases, provider_index, aliases_per_provider) {
            plugins_provided.insert(alias.as_str(), provider_name);
        }
    }

    let mut total = 0usize;
    for provider_name in provider_names {
        let provider_name = provider_name.as_str();
        plugins_provided.retain(|_, owner| *owner != provider_name);
        total += plugins_provided.len();
    }
    total
}

fn new_reverse_alias_remove_once(
    provider_names: &[String],
    aliases: &[String],
    aliases_per_provider: usize,
) -> usize {
    let mut plugins_provided =
        HashMap::with_capacity(expected_collection_capacity(aliases.len()));
    let mut provided_by_plugin =
        HashMap::with_capacity(expected_collection_capacity(provider_names.len()));

    for (provider_index, provider_name) in provider_names.iter().enumerate() {
        let provider_name = provider_name.as_str();
        for alias in alias_slice(aliases, provider_index, aliases_per_provider) {
            let alias = alias.as_str();
            let replaced = plugins_provided.insert(alias, provider_name);
            add_provided_alias(&mut provided_by_plugin, provider_name, alias);
            if let Some(replaced) = replaced {
                remove_provided_alias(&mut provided_by_plugin, replaced, alias);
            }
        }
    }

    let mut total = 0usize;
    for provider_name in provider_names {
        remove_provided_aliases(
            &mut plugins_provided,
            &mut provided_by_plugin,
            provider_name.as_str(),
        );
        total += plugins_provided.len();
    }
    total
}

fn add_provided_alias<'a>(
    provided_by_plugin: &mut HashMap<&'a str, HashSet<&'a str>>,
    provider_name: &'a str,
    alias: &'a str,
) {
    provided_by_plugin
        .entry(provider_name)
        .or_insert_with(HashSet::new)
        .insert(alias);
}

fn remove_provided_alias<'a>(
    provided_by_plugin: &mut HashMap<&'a str, HashSet<&'a str>>,
    provider_name: &'a str,
    alias: &'a str,
) {
    let remove_provider = if let Some(aliases) = provided_by_plugin.get_mut(provider_name) {
        aliases.remove(alias);
        aliases.is_empty()
    } else {
        false
    };

    if remove_provider {
        provided_by_plugin.remove(provider_name);
    }
}

fn remove_provided_aliases<'a>(
    plugins_provided: &mut HashMap<&'a str, &'a str>,
    provided_by_plugin: &mut HashMap<&'a str, HashSet<&'a str>>,
    provider_name: &'a str,
) {
    if let Some(aliases) = provided_by_plugin.remove(provider_name) {
        for alias in aliases {
            if matches!(plugins_provided.get(alias).copied(), Some(owner) if owner == provider_name)
            {
                plugins_provided.remove(alias);
            }
        }
    }
}

fn alias_slice(
    aliases: &[String],
    provider_index: usize,
    aliases_per_provider: usize,
) -> &[String] {
    let start = provider_index * aliases_per_provider;
    &aliases[start..start + aliases_per_provider]
}

fn input_digest(
    tag: u64,
    provider_names: &[String],
    aliases: &[String],
    aliases_per_provider: usize,
) -> u64 {
    mix64(
        tag
            ^ strings_digest(provider_names, 0x1656_67B1_9E37_79F9)
            ^ strings_digest(aliases, 0x85EB_CA77_C2B2_AE63)
            ^ ((aliases_per_provider as u64) << 37),
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

    fn create_providers(count: usize, aliases_per_provider: usize) -> (Vec<String>, Vec<String>) {
        let mut provider_names = Vec::with_capacity(count);
        let mut aliases = Vec::with_capacity(count * aliases_per_provider);
        for provider_index in 0..count {
            provider_names.push(format!("Plugin{provider_index}"));
            for alias_index in 0..aliases_per_provider {
                aliases.push(alias_for(provider_index, alias_index, count));
            }
        }
        (provider_names, aliases)
    }

    fn alias_for(provider_index: usize, alias_index: usize, count: usize) -> String {
        match alias_index {
            0 => format!("Alias{provider_index}"),
            1 => format!("Shared{}", provider_index & 63),
            2 if (provider_index & 3) == 0 => {
                format!("Plugin{}", (provider_index + 1) % count)
            }
            2 => format!("Shadow{}", provider_index & 31),
            3 => format!("Bucket{}", provider_index % 17),
            _ => format!("Extra{alias_index}_{provider_index}"),
        }
    }

    #[test]
    fn old_and_reverse_index_match_on_regular_inputs() {
        let (provider_names, aliases) = create_providers(128, 4);
        assert_eq!(
            old_values_removeif_summary(32, &provider_names, &aliases, 4),
            new_reverse_alias_remove_summary(32, &provider_names, &aliases, 4),
        );
    }

    #[test]
    fn old_and_reverse_index_match_with_heavy_alias_collisions() {
        let provider_names = vec![
            "Plugin0".to_string(),
            "Plugin1".to_string(),
            "Plugin2".to_string(),
            "Plugin3".to_string(),
        ];
        let aliases = vec![
            "Alias0".to_string(),
            "Shared".to_string(),
            "Alias1".to_string(),
            "Shared".to_string(),
            "Alias2".to_string(),
            "Shared".to_string(),
            "Alias3".to_string(),
            "Shared".to_string(),
        ];
        assert_eq!(
            old_values_removeif_summary(24, &provider_names, &aliases, 2),
            new_reverse_alias_remove_summary(24, &provider_names, &aliases, 2),
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (provider_names, aliases) = create_providers(8, 4);
        assert_eq!(
            old_values_removeif_summary(0, &provider_names, &aliases, 4),
            LegacyProvidedAliasRemovalSummary::default(),
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (provider_names, aliases) = create_providers(24, 5);
        let first = new_reverse_alias_remove_summary(64, &provider_names, &aliases, 5);
        let second = new_reverse_alias_remove_summary(64, &provider_names, &aliases, 5);
        assert_eq!(first, second);
    }
}
