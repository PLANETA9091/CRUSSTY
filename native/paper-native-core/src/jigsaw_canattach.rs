pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const DIRECTION_OPPOSITES: [i32; 6] = [1, 0, 3, 2, 5, 4];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JigsawCanAttachSummary {
    pub count: u64,
    pub success_count: u64,
    pub checksum: u64,
    pub last_decision: u64,
}

pub fn old_batch_summary(
    iterations: usize,
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientations: &[i32],
    child_orientations: &[i32],
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
) -> JigsawCanAttachSummary {
    run_batch_summary(
        iterations,
        orientation_fronts,
        orientation_tops,
        parent_orientations,
        child_orientations,
        parent_rollables,
        parent_targets,
        child_names,
        old_can_attach,
    )
}

pub fn optimized_batch_summary(
    iterations: usize,
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientations: &[i32],
    child_orientations: &[i32],
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
) -> JigsawCanAttachSummary {
    run_batch_summary(
        iterations,
        orientation_fronts,
        orientation_tops,
        parent_orientations,
        child_orientations,
        parent_rollables,
        parent_targets,
        child_names,
        optimized_can_attach,
    )
}

pub fn target_first_batch_summary(
    iterations: usize,
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientations: &[i32],
    child_orientations: &[i32],
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
) -> JigsawCanAttachSummary {
    run_batch_summary(
        iterations,
        orientation_fronts,
        orientation_tops,
        parent_orientations,
        child_orientations,
        parent_rollables,
        parent_targets,
        child_names,
        target_first_can_attach,
    )
}

fn run_batch_summary<F>(
    iterations: usize,
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientations: &[i32],
    child_orientations: &[i32],
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
    mut can_attach: F,
) -> JigsawCanAttachSummary
where
    F: FnMut(
        &[i32],
        &[i32],
        usize,
        usize,
        usize,
        usize,
        &[bool],
        &[i32],
        &[i32],
    ) -> bool,
{
    let positions = parent_orientations.len();
    debug_assert_eq!(positions, child_orientations.len());
    debug_assert_eq!(positions, parent_rollables.len());
    debug_assert_eq!(positions, parent_targets.len());
    debug_assert_eq!(positions, child_names.len());

    if iterations == 0 || positions == 0 {
        return JigsawCanAttachSummary::default();
    }

    debug_assert!(positions.is_power_of_two());
    let mask = positions - 1;
    let mut success_count = 0u64;
    let mut checksum = 0u64;
    let mut last_decision = 0u64;

    for i in 0..iterations {
        let parent_index = ((i as u32).wrapping_mul(17) as usize) & mask;
        let child_index = ((i as u32).wrapping_mul(31) as usize) & mask;
        let parent_orientation = parent_orientations[parent_index] as usize;
        let child_orientation = child_orientations[child_index] as usize;
        let decision = if can_attach(
            orientation_fronts,
            orientation_tops,
            parent_orientation,
            child_orientation,
            parent_index,
            child_index,
            parent_rollables,
            parent_targets,
            child_names,
        ) {
            1u64
        } else {
            0u64
        };
        success_count = success_count.wrapping_add(decision);
        last_decision = decision;
        checksum = mix64(
            checksum
                ^ decision
                ^ ((parent_index as u64) << 1)
                ^ ((child_index as u64) << 33)
                ^ ((i as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    JigsawCanAttachSummary {
        count: iterations as u64,
        success_count,
        checksum,
        last_decision,
    }
}

fn old_can_attach(
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientation: usize,
    child_orientation: usize,
    parent_index: usize,
    child_index: usize,
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
) -> bool {
    let parent_front = front_facing(orientation_fronts, parent_orientation);
    let child_front = front_facing(orientation_fronts, child_orientation);
    let parent_top = top_facing(orientation_tops, parent_orientation);
    let child_top = top_facing(orientation_tops, child_orientation);
    parent_front == opposite_direction(child_front)
        && (parent_rollables[parent_index] || parent_top == child_top)
        && parent_targets[parent_index] == child_names[child_index]
}

fn optimized_can_attach(
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientation: usize,
    child_orientation: usize,
    parent_index: usize,
    child_index: usize,
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
) -> bool {
    let parent_front = orientation_fronts[parent_orientation];
    let child_front = orientation_fronts[child_orientation];
    let parent_top = orientation_tops[parent_orientation];
    let child_top = orientation_tops[child_orientation];
    parent_front == opposite_direction(child_front)
        && (parent_rollables[parent_index] || parent_top == child_top)
        && parent_targets[parent_index] == child_names[child_index]
}

fn target_first_can_attach(
    orientation_fronts: &[i32],
    orientation_tops: &[i32],
    parent_orientation: usize,
    child_orientation: usize,
    parent_index: usize,
    child_index: usize,
    parent_rollables: &[bool],
    parent_targets: &[i32],
    child_names: &[i32],
) -> bool {
    if parent_targets[parent_index] != child_names[child_index] {
        return false;
    }

    let parent_front = orientation_fronts[parent_orientation];
    let child_front = orientation_fronts[child_orientation];
    let parent_top = orientation_tops[parent_orientation];
    let child_top = orientation_tops[child_orientation];
    parent_front == opposite_direction(child_front)
        && (parent_rollables[parent_index] || parent_top == child_top)
}

#[inline]
fn front_facing(orientation_fronts: &[i32], orientation_index: usize) -> i32 {
    orientation_fronts[orientation_index]
}

#[inline]
fn top_facing(orientation_tops: &[i32], orientation_index: usize) -> i32 {
    orientation_tops[orientation_index]
}

#[inline]
fn opposite_direction(direction: i32) -> i32 {
    DIRECTION_OPPOSITES[direction as usize]
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
    fn old_optimized_and_target_first_match_on_regular_inputs() {
        let orientation_fronts = [0, 1, 2, 3];
        let orientation_tops = [1, 0, 3, 2];
        let parent_orientations = [0, 1, 2, 3, 1, 2, 3, 0];
        let child_orientations = [1, 0, 3, 2, 0, 3, 2, 1];
        let parent_rollables = [true, false, true, false, true, false, true, false];
        let parent_targets = [0, 1, 2, 3, 0, 1, 2, 3];
        let child_names = [0, 1, 2, 3, 1, 0, 3, 2];
        let iterations = 1024;

        let old = old_batch_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );
        let optimized = optimized_batch_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );
        let target_first = target_first_batch_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );

        assert_eq!(old, optimized);
        assert_eq!(old, target_first);
        assert_eq!(old.count, iterations as u64);
    }

    #[test]
    fn orientation_ordinals_are_resolved_before_lookup() {
        let orientation_fronts = [2, 3, 4, 5, 0, 1];
        let orientation_tops = [1, 0, 3, 2, 5, 4];
        let parent_orientations = [4, 5, 1, 0];
        let child_orientations = [5, 4, 0, 1];
        let parent_rollables = [false, true, false, true];
        let parent_targets = [7, 8, 7, 8];
        let child_names = [7, 8, 8, 7];
        let iterations = 512;

        let expected = reference_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );
        let actual = old_batch_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let empty_i32: [i32; 0] = [];
        let empty_bool: [bool; 0] = [];
        let summary = old_batch_summary(
            0,
            &empty_i32,
            &empty_i32,
            &empty_i32,
            &empty_i32,
            &empty_bool,
            &empty_i32,
            &empty_i32,
        );
        assert_eq!(summary, JigsawCanAttachSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let orientation_fronts = [0, 1, 2, 3, 4, 5];
        let orientation_tops = [1, 0, 3, 2, 5, 4];
        let parent_orientations = [0, 1, 2, 3];
        let child_orientations = [1, 0, 3, 2];
        let parent_rollables = [true, false, true, false];
        let parent_targets = [0, 1, 2, 3];
        let child_names = [0, 1, 2, 3];
        let iterations = 256;

        let first = optimized_batch_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );
        let second = optimized_batch_summary(
            iterations,
            &orientation_fronts,
            &orientation_tops,
            &parent_orientations,
            &child_orientations,
            &parent_rollables,
            &parent_targets,
            &child_names,
        );

        assert_eq!(first, second);
    }

    fn reference_summary(
        iterations: usize,
        orientation_fronts: &[i32],
        orientation_tops: &[i32],
        parent_orientations: &[i32],
        child_orientations: &[i32],
        parent_rollables: &[bool],
        parent_targets: &[i32],
        child_names: &[i32],
    ) -> JigsawCanAttachSummary {
        let positions = parent_orientations.len();
        if iterations == 0 || positions == 0 {
            return JigsawCanAttachSummary::default();
        }

        debug_assert!(positions.is_power_of_two());
        let mask = positions - 1;
        let mut success_count = 0u64;
        let mut checksum = 0u64;
        let mut last_decision = 0u64;

        for i in 0..iterations {
            let parent_index = ((i as u32).wrapping_mul(17) as usize) & mask;
            let child_index = ((i as u32).wrapping_mul(31) as usize) & mask;
            let parent_orientation = parent_orientations[parent_index] as usize;
            let child_orientation = child_orientations[child_index] as usize;
            let decision = if orientation_fronts[parent_orientation]
                == opposite_direction(orientation_fronts[child_orientation])
                && (parent_rollables[parent_index]
                    || orientation_tops[parent_orientation] == orientation_tops[child_orientation])
                && parent_targets[parent_index] == child_names[child_index]
            {
                1u64
            } else {
                0u64
            };
            success_count = success_count.wrapping_add(decision);
            last_decision = decision;
            checksum = mix64(
                checksum
                    ^ decision
                    ^ ((parent_index as u64) << 1)
                    ^ ((child_index as u64) << 33)
                    ^ ((i as u64).wrapping_mul(MIX_GAMMA))
                    ^ ((iterations as u64).rotate_left(13)),
            );
        }

        JigsawCanAttachSummary {
            count: iterations as u64,
            success_count,
            checksum,
            last_decision,
        }
    }
}
