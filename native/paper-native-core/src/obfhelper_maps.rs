use std::collections::HashMap;

pub const SUMMARY_FIELDS: usize = 3;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const OBFHELPER_TAG: u64 = 0x9A4D_B1F0_66A7_2D15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObfHelperMapsKind {
    OldStreamDefault,
    DirectMaps,
    PresizedStringPool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObfHelperMapsSummary {
    pub class_count: u64,
    pub entry_count: u64,
    pub fingerprint: u64,
}

#[derive(Clone, Debug)]
pub struct ObfHelperMapsFixture<'a> {
    pub class_mapped_names: &'a [String],
    pub class_original_names: &'a [String],
    pub method_counts: &'a [usize],
    pub field_counts: &'a [usize],
    pub method_mapped_names: &'a [String],
    pub method_mapped_descriptors: &'a [String],
    pub method_original_names: &'a [String],
    pub method_original_descriptors: &'a [String],
    pub field_mapped_names: &'a [String],
    pub field_original_names: &'a [String],
}

#[derive(Clone, Debug)]
struct LoadedMappings {
    by_obf: HashMap<String, ClassMapping>,
    by_mojang: HashMap<String, ClassMapping>,
    _pool_len: usize,
}

#[derive(Clone, Debug)]
struct ClassMapping {
    obf_name: String,
    mojang_name: String,
    methods_by_obf: HashMap<String, String>,
    fields_by_obf: HashMap<String, String>,
    stripped_methods: HashMap<String, String>,
}

#[derive(Clone, Debug)]
struct StringPool {
    pool: HashMap<String, String>,
}

impl StringPool {
    fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: HashMap::with_capacity(capacity),
        }
    }

    fn string(&mut self, value: &str) -> String {
        if let Some(existing) = self.pool.get(value) {
            return existing.clone();
        }

        let owned = value.to_owned();
        self.pool.insert(owned.clone(), owned.clone());
        owned
    }

    fn len(&self) -> usize {
        self.pool.len()
    }
}

pub fn old_stream_default_summary(
    fixture: &ObfHelperMapsFixture<'_>,
) -> Result<ObfHelperMapsSummary, ObfHelperMapsError> {
    let loaded = build_old_stream_default(fixture)?;
    Ok(summary_from_loaded(&loaded))
}

pub fn direct_maps_summary(
    fixture: &ObfHelperMapsFixture<'_>,
) -> Result<ObfHelperMapsSummary, ObfHelperMapsError> {
    let loaded = build_direct_maps(fixture, false)?;
    Ok(summary_from_loaded(&loaded))
}

pub fn presized_string_pool_summary(
    fixture: &ObfHelperMapsFixture<'_>,
) -> Result<ObfHelperMapsSummary, ObfHelperMapsError> {
    let loaded = build_direct_maps(fixture, true)?;
    Ok(summary_from_loaded(&loaded))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObfHelperMapsError {
    InvalidInputLength,
    InvalidCount,
    DuplicateKey,
}

fn build_old_stream_default(
    fixture: &ObfHelperMapsFixture<'_>,
) -> Result<LoadedMappings, ObfHelperMapsError> {
    validate_fixture(fixture)?;

    let mut pool = StringPool::new();
    let mut maps = Vec::with_capacity(fixture.class_mapped_names.len());
    let mut method_index = 0usize;
    let mut field_index = 0usize;

    for class_index in 0..fixture.class_mapped_names.len() {
        let methods = build_methods(
            &mut pool,
            fixture,
            class_index,
            method_index,
            false,
        )?;
        method_index += fixture.method_counts[class_index];

        let fields = build_fields(
            &mut pool,
            fixture,
            class_index,
            field_index,
            false,
        )?;
        field_index += fixture.field_counts[class_index];

        maps.push(ClassMapping {
            obf_name: fixture.class_mapped_names[class_index].replace('/', "."),
            mojang_name: fixture.class_original_names[class_index].replace('/', "."),
            methods_by_obf: methods,
            fields_by_obf: fields,
            stripped_methods: build_stripped_methods(
                &mut pool,
                fixture,
                class_index,
                method_index - fixture.method_counts[class_index],
                false,
            )?,
        });
    }

    Ok(LoadedMappings {
        by_obf: build_by_obf(&maps)?,
        by_mojang: build_by_mojang(&maps)?,
        _pool_len: pool.len(),
    })
}

fn build_direct_maps(
    fixture: &ObfHelperMapsFixture<'_>,
    presized_pool: bool,
) -> Result<LoadedMappings, ObfHelperMapsError> {
    validate_fixture(fixture)?;

    let mut pool = if presized_pool {
        StringPool::with_capacity(expected_collection_capacity(rough_string_pool_inputs(fixture)))
    } else {
        StringPool::new()
    };
    let mut by_obf = HashMap::with_capacity(expected_collection_capacity(fixture.class_mapped_names.len()));
    let mut by_mojang = HashMap::with_capacity(expected_collection_capacity(fixture.class_mapped_names.len()));
    let mut method_index = 0usize;
    let mut field_index = 0usize;

    for class_index in 0..fixture.class_mapped_names.len() {
        let methods = build_methods(
            &mut pool,
            fixture,
            class_index,
            method_index,
            true,
        )?;
        method_index += fixture.method_counts[class_index];

        let fields = build_fields(
            &mut pool,
            fixture,
            class_index,
            field_index,
            true,
        )?;
        field_index += fixture.field_counts[class_index];

        let map = ClassMapping {
            obf_name: fixture.class_mapped_names[class_index].replace('/', "."),
            mojang_name: fixture.class_original_names[class_index].replace('/', "."),
            methods_by_obf: methods,
            fields_by_obf: fields,
            stripped_methods: build_stripped_methods(
                &mut pool,
                fixture,
                class_index,
                method_index - fixture.method_counts[class_index],
                true,
            )?,
        };
        put_unique(&mut by_obf, map.obf_name.clone(), map.clone())?;
        put_unique(&mut by_mojang, map.mojang_name.clone(), map)?;
    }

    Ok(LoadedMappings {
        by_obf,
        by_mojang,
        _pool_len: pool.len(),
    })
}

fn build_methods(
    pool: &mut StringPool,
    fixture: &ObfHelperMapsFixture<'_>,
    class_index: usize,
    method_start: usize,
    presized: bool,
) -> Result<HashMap<String, String>, ObfHelperMapsError> {
    let method_count = fixture.method_counts[class_index];
    let mut methods = if presized {
        HashMap::with_capacity(expected_collection_capacity(method_count))
    } else {
        HashMap::new()
    };
    for offset in 0..method_count {
        let index = method_start + offset;
        let key = method_key(
            &fixture.method_mapped_names[index],
            &fixture.method_mapped_descriptors[index],
        );
        let mapped = pool.string(&key);
        let value = pool.string(&fixture.method_original_names[index]);
        methods.insert(mapped, value);
    }
    Ok(methods)
}

fn build_fields(
    pool: &mut StringPool,
    fixture: &ObfHelperMapsFixture<'_>,
    class_index: usize,
    field_start: usize,
    presized: bool,
) -> Result<HashMap<String, String>, ObfHelperMapsError> {
    let field_count = fixture.field_counts[class_index];
    let mut fields = if presized {
        HashMap::with_capacity(expected_collection_capacity(field_count))
    } else {
        HashMap::new()
    };
    for offset in 0..field_count {
        let index = field_start + offset;
        let mapped = pool.string(&fixture.field_mapped_names[index]);
        let value = pool.string(&fixture.field_original_names[index]);
        fields.insert(mapped, value);
    }
    Ok(fields)
}

fn build_stripped_methods(
    pool: &mut StringPool,
    fixture: &ObfHelperMapsFixture<'_>,
    class_index: usize,
    method_start: usize,
    presized: bool,
) -> Result<HashMap<String, String>, ObfHelperMapsError> {
    let method_count = fixture.method_counts[class_index];
    let mut stripped = if presized {
        HashMap::with_capacity(expected_collection_capacity(method_count))
    } else {
        HashMap::new()
    };
    for offset in 0..method_count {
        let index = method_start + offset;
        let stripped_key = stripped_method_key(
            &fixture.method_mapped_names[index],
            &fixture.method_original_descriptors[index],
        );
        let stripped_once = pool.string(&stripped_key);
        let pooled = pool.string(&stripped_once);
        let value = pool.string(&fixture.method_original_names[index]);
        stripped.insert(pooled, value);
    }
    Ok(stripped)
}

fn build_by_obf(maps: &[ClassMapping]) -> Result<HashMap<String, ClassMapping>, ObfHelperMapsError> {
    let mut by_obf = HashMap::with_capacity(expected_collection_capacity(maps.len()));
    for map in maps {
        put_unique(&mut by_obf, map.obf_name.clone(), map.clone())?;
    }
    Ok(by_obf)
}

fn build_by_mojang(
    maps: &[ClassMapping],
) -> Result<HashMap<String, ClassMapping>, ObfHelperMapsError> {
    let mut by_mojang = HashMap::with_capacity(expected_collection_capacity(maps.len()));
    for map in maps {
        put_unique(&mut by_mojang, map.mojang_name.clone(), map.clone())?;
    }
    Ok(by_mojang)
}

fn summary_from_loaded(loaded: &LoadedMappings) -> ObfHelperMapsSummary {
    let mut fingerprint = mix64(
        OBFHELPER_TAG
            ^ (loaded.by_obf.len() as u64)
            ^ ((loaded.by_mojang.len() as u64) << 17),
    );

    let mut entry_count = 0u64;
    let mut classes = loaded.by_obf.values().collect::<Vec<_>>();
    classes.sort_unstable_by(|left, right| left.obf_name.cmp(&right.obf_name));
    for (index, class) in classes.iter().enumerate() {
        entry_count += (class.methods_by_obf.len() + class.fields_by_obf.len() + class.stripped_methods.len()) as u64;
        fingerprint = mix64(
            fingerprint
                ^ digest_class(class)
                ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
        );
    }
    let mut mojang_classes = loaded.by_mojang.values().collect::<Vec<_>>();
    mojang_classes.sort_unstable_by(|left, right| left.mojang_name.cmp(&right.mojang_name));
    for (index, class) in mojang_classes.iter().enumerate() {
        fingerprint = mix64(
            fingerprint
                ^ digest_class(class)
                ^ ((index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)),
        );
    }

    ObfHelperMapsSummary {
        class_count: loaded.by_obf.len() as u64,
        entry_count,
        fingerprint,
    }
}

fn digest_class(class: &ClassMapping) -> u64 {
    let mut digest = mix64(
        java_string_hash(&class.obf_name) as i64 as u64
            ^ ((java_string_hash(&class.mojang_name) as i64 as u64) << 32),
    );
    digest = mix64(digest ^ digest_map(&class.methods_by_obf));
    digest = mix64(digest ^ digest_map(&class.fields_by_obf));
    digest = mix64(digest ^ digest_map(&class.stripped_methods));
    digest
}

fn digest_map(map: &HashMap<String, String>) -> u64 {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_unstable_by(|left, right| left.0.cmp(right.0).then_with(|| left.1.cmp(right.1)));
    let mut digest = mix64(map.len() as u64);
    for (index, (key, value)) in entries.into_iter().enumerate() {
        digest = mix64(
            digest
                ^ (java_string_hash(key) as i64 as u64)
                ^ ((java_string_hash(value) as i64 as u64) << 32)
                ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
        );
    }
    digest
}

fn validate_fixture(fixture: &ObfHelperMapsFixture<'_>) -> Result<(), ObfHelperMapsError> {
    let class_count = fixture.class_mapped_names.len();
    if class_count == 0
        || fixture.class_original_names.len() != class_count
        || fixture.method_counts.len() != class_count
        || fixture.field_counts.len() != class_count
    {
        return Err(ObfHelperMapsError::InvalidInputLength);
    }

    let method_total = fixture
        .method_counts
        .iter()
        .try_fold(0usize, |total, &count| total.checked_add(count).ok_or(ObfHelperMapsError::InvalidCount))?;
    let field_total = fixture
        .field_counts
        .iter()
        .try_fold(0usize, |total, &count| total.checked_add(count).ok_or(ObfHelperMapsError::InvalidCount))?;

    if fixture.method_mapped_names.len() != method_total
        || fixture.method_mapped_descriptors.len() != method_total
        || fixture.method_original_names.len() != method_total
        || fixture.method_original_descriptors.len() != method_total
        || fixture.field_mapped_names.len() != field_total
        || fixture.field_original_names.len() != field_total
    {
        return Err(ObfHelperMapsError::InvalidInputLength);
    }

    Ok(())
}

fn rough_string_pool_inputs(fixture: &ObfHelperMapsFixture<'_>) -> usize {
    let method_total = fixture.method_counts.iter().sum::<usize>();
    let field_total = fixture.field_counts.iter().sum::<usize>();
    fixture.class_mapped_names.len() * 2 + method_total * 4 + field_total * 2
}

fn expected_collection_capacity(expected_size: usize) -> usize {
    if expected_size < 3 {
        expected_size + 1
    } else if expected_size < (1 << 30) {
        ((expected_size as f32 / 0.75f32) + 1.0f32) as usize
    } else {
        usize::MAX
    }
}

fn method_key(method_name: &str, method_descriptor: &str) -> String {
    let mut key = String::with_capacity(method_name.len() + method_descriptor.len());
    key.push_str(method_name);
    key.push_str(method_descriptor);
    key
}

fn stripped_method_key(method_name: &str, method_descriptor: &str) -> String {
    let key = method_key(method_name, method_descriptor);
    if let Some(end) = key.find(')') {
        key[..=end].to_string()
    } else {
        key
    }
}

fn put_unique<V>(
    map: &mut HashMap<String, V>,
    key: String,
    value: V,
) -> Result<(), ObfHelperMapsError> {
    if map.insert(key, value).is_some() {
        return Err(ObfHelperMapsError::DuplicateKey);
    }
    Ok(())
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
    fn rejects_bad_fixture_shapes() {
        let class_mapped = vec!["a/A".to_string()];
        let class_original = vec!["b/B".to_string()];
        let method_counts = vec![1usize];
        let field_counts = vec![1usize];
        let methods_mapped = vec!["m".to_string()];
        let methods_mapped_desc = vec!["()V".to_string()];
        let methods_original = vec!["orig".to_string()];
        let methods_original_desc = vec!["()V".to_string()];
        let fields_mapped = vec!["f".to_string()];
        let fields_original = vec!["fo".to_string()];

        let fixture = ObfHelperMapsFixture {
            class_mapped_names: &class_mapped,
            class_original_names: &class_original,
            method_counts: &method_counts,
            field_counts: &field_counts,
            method_mapped_names: &methods_mapped,
            method_mapped_descriptors: &methods_mapped_desc,
            method_original_names: &methods_original,
            method_original_descriptors: &methods_original_desc,
            field_mapped_names: &fields_mapped,
            field_original_names: &fields_original,
        };

        assert!(old_stream_default_summary(&fixture).is_ok());
        assert!(direct_maps_summary(&fixture).is_ok());
        assert!(presized_string_pool_summary(&fixture).is_ok());
    }
}
