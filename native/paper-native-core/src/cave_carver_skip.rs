pub const SUMMARY_FIELDS: usize = 2;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CaveCarverSkipSummary {
    pub count: u64,
    pub guard: u64,
}

pub fn old_lambda_summary(
    carves: usize,
    floor_levels: &[f64],
    relative_x: &[f64],
    relative_y: &[f64],
    relative_z: &[f64],
) -> CaveCarverSkipSummary {
    run_summary(carves, floor_levels, relative_x, relative_y, relative_z)
}

pub fn reused_checker_summary(
    carves: usize,
    floor_levels: &[f64],
    relative_x: &[f64],
    relative_y: &[f64],
    relative_z: &[f64],
) -> CaveCarverSkipSummary {
    run_summary(carves, floor_levels, relative_x, relative_y, relative_z)
}

pub fn direct_helper_summary(
    carves: usize,
    floor_levels: &[f64],
    relative_x: &[f64],
    relative_y: &[f64],
    relative_z: &[f64],
) -> CaveCarverSkipSummary {
    run_summary(carves, floor_levels, relative_x, relative_y, relative_z)
}

fn run_summary(
    carves: usize,
    floor_levels: &[f64],
    relative_x: &[f64],
    relative_y: &[f64],
    relative_z: &[f64],
) -> CaveCarverSkipSummary {
    if carves == 0 || floor_levels.is_empty() || relative_x.is_empty() {
        return CaveCarverSkipSummary::default();
    }
    debug_assert_eq!(relative_y.len(), relative_x.len());
    debug_assert_eq!(relative_z.len(), relative_x.len());

    let cave_count = floor_levels.len();
    let sample_count = relative_x.len();
    let mut guard = 0u64;

    for carve in 0..carves {
        for cave in 0..cave_count {
            let floor_level = floor_levels[(cave + carve) % cave_count];
            let seed = carve + cave;
            guard = guard.wrapping_add(consume_direct(
                floor_level,
                seed,
                relative_x,
                relative_y,
                relative_z,
                sample_count,
            ));
        }
    }

    CaveCarverSkipSummary {
        count: carves as u64,
        guard,
    }
}

fn consume_direct(
    floor_level: f64,
    seed: usize,
    relative_x: &[f64],
    relative_y: &[f64],
    relative_z: &[f64],
    sample_count: usize,
) -> u64 {
    let mut guard = 0u64;
    for i in 0..sample_count {
        let index = (i + seed) % sample_count;
        if should_skip(relative_x[index], relative_y[index], relative_z[index], floor_level) {
            guard = guard.wrapping_add(31 + i as u64);
        } else {
            guard = guard.wrapping_add(7 + i as u64);
        }
    }
    guard
}

#[inline]
fn should_skip(relative_x: f64, relative_y: f64, relative_z: f64, floor_level: f64) -> bool {
    relative_y <= floor_level
        || relative_x * relative_x + relative_y * relative_y + relative_z * relative_z >= 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_reused_and_direct_match_on_regular_inputs() {
        let samples = build_samples(6, 48);
        let old = old_lambda_summary(
            2_048,
            &samples.floor_levels,
            &samples.relative_x,
            &samples.relative_y,
            &samples.relative_z,
        );
        let reused = reused_checker_summary(
            2_048,
            &samples.floor_levels,
            &samples.relative_x,
            &samples.relative_y,
            &samples.relative_z,
        );
        let direct = direct_helper_summary(
            2_048,
            &samples.floor_levels,
            &samples.relative_x,
            &samples.relative_y,
            &samples.relative_z,
        );

        assert_eq!(old, reused);
        assert_eq!(old, direct);
        assert_eq!(old.count, 2_048);
    }

    #[test]
    fn zero_carves_are_empty() {
        let samples = build_samples(2, 4);
        let summary = direct_helper_summary(
            0,
            &samples.floor_levels,
            &samples.relative_x,
            &samples.relative_y,
            &samples.relative_z,
        );

        assert_eq!(summary, CaveCarverSkipSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let samples = build_samples(3, 11);
        let first = direct_helper_summary(
            512,
            &samples.floor_levels,
            &samples.relative_x,
            &samples.relative_y,
            &samples.relative_z,
        );
        let second = direct_helper_summary(
            512,
            &samples.floor_levels,
            &samples.relative_x,
            &samples.relative_y,
            &samples.relative_z,
        );

        assert_eq!(first, second);
    }

    #[test]
    fn empty_samples_are_empty() {
        let summary = old_lambda_summary(128, &[0.0], &[], &[], &[]);

        assert_eq!(summary, CaveCarverSkipSummary::default());
    }

    struct Samples {
        floor_levels: Vec<f64>,
        relative_x: Vec<f64>,
        relative_y: Vec<f64>,
        relative_z: Vec<f64>,
    }

    fn build_samples(caves: usize, samples: usize) -> Samples {
        let mut floor_levels = Vec::with_capacity(caves);
        for i in 0..caves {
            floor_levels.push(-0.85 + i as f64 * 0.21);
        }

        let mut relative_x = Vec::with_capacity(samples);
        let mut relative_y = Vec::with_capacity(samples);
        let mut relative_z = Vec::with_capacity(samples);
        for i in 0..samples {
            relative_x.push(((i * 37) % 97) as f64 / 48.5 - 1.0);
            relative_y.push(((i * 53 + 11) % 101) as f64 / 50.5 - 1.0);
            relative_z.push(((i * 71 + 19) % 103) as f64 / 51.5 - 1.0);
        }

        Samples {
            floor_levels,
            relative_x,
            relative_y,
            relative_z,
        }
    }
}
