pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const REQUIRED_TAG: u64 = 0xA24B_AED4_9C3B_1F15;
const SOFT_TAG: u64 = 0xC2B2_AE3D_27D4_EB4F;
const LOAD_BEFORE_TAG: u64 = 0x1656_67B1_9E37_79F9;
const LOAD_AFTER_TAG: u64 = 0x85EB_CA77_C2B2_AE63;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PluginMetaDependencySummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_stream_summary(
    iterations: usize,
    names: &[String],
    required: &[bool],
    join_classpath: &[bool],
    load: &[i32],
) -> PluginMetaDependencySummary {
    run_summary(
        iterations,
        names,
        required,
        join_classpath,
        load,
        Mode::OldStream,
    )
}

pub fn new_loop_summary(
    iterations: usize,
    names: &[String],
    required: &[bool],
    join_classpath: &[bool],
    load: &[i32],
) -> PluginMetaDependencySummary {
    run_summary(
        iterations,
        names,
        required,
        join_classpath,
        load,
        Mode::NewLoop,
    )
}

pub fn cached_summary(
    iterations: usize,
    names: &[String],
    required: &[bool],
    join_classpath: &[bool],
    load: &[i32],
) -> PluginMetaDependencySummary {
    run_summary(
        iterations,
        names,
        required,
        join_classpath,
        load,
        Mode::Cached,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldStream,
    NewLoop,
    Cached,
}

fn run_summary(
    iterations: usize,
    names: &[String],
    required: &[bool],
    join_classpath: &[bool],
    load: &[i32],
    mode: Mode,
) -> PluginMetaDependencySummary {
    if iterations == 0 {
        return PluginMetaDependencySummary::default();
    }

    debug_assert_eq!(names.len(), required.len());
    debug_assert_eq!(names.len(), join_classpath.len());
    debug_assert_eq!(names.len(), load.len());

    match mode {
        Mode::OldStream => run_allocating_summary(
            iterations,
            names,
            required,
            join_classpath,
            load,
            true,
        ),
        Mode::NewLoop => run_allocating_summary(
            iterations,
            names,
            required,
            join_classpath,
            load,
            false,
        ),
        Mode::Cached => run_cached_summary(iterations, names, required, join_classpath, load),
    }
}

fn run_allocating_summary(
    iterations: usize,
    names: &[String],
    required: &[bool],
    join_classpath: &[bool],
    load: &[i32],
    old_stream: bool,
) -> PluginMetaDependencySummary {
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let (required_join_classpath, soft_join_classpath, load_before, load_after) = if old_stream {
            (
                old_required_join_classpath(required, join_classpath),
                old_soft_join_classpath(required, join_classpath),
                old_load_before(load),
                old_load_after(load),
            )
        } else {
            (
                new_required_join_classpath(required, join_classpath),
                new_soft_join_classpath(required, join_classpath),
                new_load_before(load),
                new_load_after(load),
            )
        };

        record_iteration(
            names,
            required.len(),
            join_classpath.len(),
            load.len(),
            iteration,
            &required_join_classpath,
            &soft_join_classpath,
            &load_before,
            &load_after,
            &mut total,
            &mut checksum,
            &mut last_total,
        );
    }

    PluginMetaDependencySummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn run_cached_summary(
    iterations: usize,
    names: &[String],
    required: &[bool],
    join_classpath: &[bool],
    load: &[i32],
) -> PluginMetaDependencySummary {
    let cached = CachedDependencyLists::new(required, join_classpath, load);
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        record_iteration(
            names,
            required.len(),
            join_classpath.len(),
            load.len(),
            iteration,
            cached.required_join_classpath(),
            cached.soft_join_classpath(),
            cached.load_before(),
            cached.load_after(),
            &mut total,
            &mut checksum,
            &mut last_total,
        );
    }

    PluginMetaDependencySummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn record_iteration(
    names: &[String],
    required_len: usize,
    join_classpath_len: usize,
    load_len: usize,
    iteration: usize,
    required_join_classpath: &[usize],
    soft_join_classpath: &[usize],
    load_before: &[usize],
    load_after: &[usize],
    total: &mut u64,
    checksum: &mut u64,
    last_total: &mut u64,
) {
    let iteration_total = (required_join_classpath.len()
        + soft_join_classpath.len()
        + load_before.len()
        + load_after.len()) as u64;
    let iteration_checksum = iteration_digest(
        names,
        required_join_classpath,
        soft_join_classpath,
        load_before,
        load_after,
    );

    *total += iteration_total;
    *last_total = iteration_total;
    *checksum = mix64(
        *checksum
            ^ iteration_checksum
            ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
            ^ ((names.len() as u64) << 1)
            ^ ((required_len as u64) << 17)
            ^ ((join_classpath_len as u64) << 21)
            ^ ((load_len as u64) << 29),
    );
}

fn iteration_digest(
    names: &[String],
    required_join_classpath: &[usize],
    soft_join_classpath: &[usize],
    load_before: &[usize],
    load_after: &[usize],
) -> u64 {
    mix64(
        list_digest(names, required_join_classpath, REQUIRED_TAG)
            ^ list_digest(names, soft_join_classpath, SOFT_TAG)
            ^ list_digest(names, load_before, LOAD_BEFORE_TAG)
            ^ list_digest(names, load_after, LOAD_AFTER_TAG),
    )
}

fn list_digest(names: &[String], indexes: &[usize], tag: u64) -> u64 {
    let mut digest = mix64(tag ^ (indexes.len() as u64));
    for (position, index) in indexes.iter().copied().enumerate() {
        let name_hash = java_string_hash(&names[index]) as i64 as u64;
        digest = mix64(
            digest
                ^ name_hash
                ^ ((position as u64).wrapping_mul(MIX_GAMMA)),
        );
    }
    digest
}

fn old_required_join_classpath(required: &[bool], join_classpath: &[bool]) -> Vec<usize> {
    required
        .iter()
        .zip(join_classpath.iter())
        .enumerate()
        .filter(|(_, (required, join_classpath))| **required && **join_classpath)
        .map(|(index, _)| index)
        .collect()
}

fn old_soft_join_classpath(required: &[bool], join_classpath: &[bool]) -> Vec<usize> {
    required
        .iter()
        .zip(join_classpath.iter())
        .enumerate()
        .filter(|(_, (required, join_classpath))| !**required && **join_classpath)
        .map(|(index, _)| index)
        .collect()
}

fn old_load_before(load: &[i32]) -> Vec<usize> {
    load
        .iter()
        .enumerate()
        .filter(|(_, load)| **load == 2)
        .map(|(index, _)| index)
        .collect()
}

fn old_load_after(load: &[i32]) -> Vec<usize> {
    load
        .iter()
        .enumerate()
        .filter(|(_, load)| **load == 1)
        .map(|(index, _)| index)
        .collect()
}

fn new_required_join_classpath(required: &[bool], join_classpath: &[bool]) -> Vec<usize> {
    new_collect(required.len(), |index| required[index] && join_classpath[index])
}

fn new_soft_join_classpath(required: &[bool], join_classpath: &[bool]) -> Vec<usize> {
    new_collect(required.len(), |index| !required[index] && join_classpath[index])
}

fn new_load_before(load: &[i32]) -> Vec<usize> {
    new_collect(load.len(), |index| load[index] == 2)
}

fn new_load_after(load: &[i32]) -> Vec<usize> {
    new_collect(load.len(), |index| load[index] == 1)
}

fn new_collect<F>(len: usize, mut predicate: F) -> Vec<usize>
where
    F: FnMut(usize) -> bool,
{
    if len == 0 {
        return Vec::new();
    }

    let mut indexes = Vec::new();
    for index in 0..len {
        if predicate(index) {
            if indexes.is_empty() {
                indexes.reserve(len);
            }
            indexes.push(index);
        }
    }
    indexes
}

struct CachedDependencyLists {
    required_join_classpath: Vec<usize>,
    soft_join_classpath: Vec<usize>,
    load_before: Vec<usize>,
    load_after: Vec<usize>,
}

impl CachedDependencyLists {
    fn new(required: &[bool], join_classpath: &[bool], load: &[i32]) -> Self {
        Self {
            required_join_classpath: new_required_join_classpath(required, join_classpath),
            soft_join_classpath: new_soft_join_classpath(required, join_classpath),
            load_before: new_load_before(load),
            load_after: new_load_after(load),
        }
    }

    fn required_join_classpath(&self) -> &[usize] {
        &self.required_join_classpath
    }

    fn soft_join_classpath(&self) -> &[usize] {
        &self.soft_join_classpath
    }

    fn load_before(&self) -> &[usize] {
        &self.load_before
    }

    fn load_after(&self) -> &[usize] {
        &self.load_after
    }
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

    fn create_inputs(count: usize) -> (Vec<String>, Vec<bool>, Vec<bool>, Vec<i32>) {
        let mut names = Vec::with_capacity(count);
        let mut required = Vec::with_capacity(count);
        let mut join_classpath = Vec::with_capacity(count);
        let mut load = Vec::with_capacity(count);

        for index in 0..count {
            names.push(format!("Dependency{index:03}"));
            required.push((index & 1) == 0);
            join_classpath.push((index % 3) != 0);
            load.push((index % 3) as i32);
        }

        (names, required, join_classpath, load)
    }

    #[test]
    fn old_new_and_cached_match_on_regular_inputs() {
        let (names, required, join_classpath, load) = create_inputs(48);
        let old = old_stream_summary(256, &names, &required, &join_classpath, &load);
        let new = new_loop_summary(256, &names, &required, &join_classpath, &load);
        let cached = cached_summary(256, &names, &required, &join_classpath, &load);

        assert_eq!(old, new);
        assert_eq!(new, cached);
    }

    #[test]
    fn empty_inputs_are_stable() {
        let names = Vec::new();
        let required = Vec::new();
        let join_classpath = Vec::new();
        let load = Vec::new();

        let old = old_stream_summary(64, &names, &required, &join_classpath, &load);
        let new = new_loop_summary(64, &names, &required, &join_classpath, &load);
        let cached = cached_summary(64, &names, &required, &join_classpath, &load);

        assert_eq!(old, new);
        assert_eq!(new, cached);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (names, required, join_classpath, load) = create_inputs(16);
        assert_eq!(
            old_stream_summary(0, &names, &required, &join_classpath, &load),
            PluginMetaDependencySummary::default()
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (names, required, join_classpath, load) = create_inputs(24);
        let first = cached_summary(128, &names, &required, &join_classpath, &load);
        let second = cached_summary(128, &names, &required, &join_classpath, &load);

        assert_eq!(first, second);
    }
}
