use std::collections::{HashMap, VecDeque};

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SORT_TAG: u64 = 0xF2F2_B4C6_DA4E_2E31;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TopographicGraphSortCapacitySummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_default_capacity_summary(
    iterations: usize,
    successor_offsets: &[usize],
    successors: &[usize],
    in_degree: &[usize],
) -> TopographicGraphSortCapacitySummary {
    run_summary(
        iterations,
        successor_offsets,
        successors,
        in_degree,
        Mode::OldDefaultCapacity,
    )
}

pub fn new_presized_summary(
    iterations: usize,
    successor_offsets: &[usize],
    successors: &[usize],
    in_degree: &[usize],
) -> TopographicGraphSortCapacitySummary {
    run_summary(
        iterations,
        successor_offsets,
        successors,
        in_degree,
        Mode::NewPresized,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    OldDefaultCapacity,
    NewPresized,
}

fn run_summary(
    iterations: usize,
    successor_offsets: &[usize],
    successors: &[usize],
    in_degree: &[usize],
    mode: Mode,
) -> TopographicGraphSortCapacitySummary {
    if iterations == 0 {
        return TopographicGraphSortCapacitySummary::default();
    }

    let node_count = in_degree.len();
    debug_assert_eq!(successor_offsets.len(), node_count + 1);
    debug_assert_eq!(successors.len(), successor_offsets[node_count]);

    let shape_digest = input_digest(
        SORT_TAG,
        successor_offsets,
        successors,
        in_degree,
    );
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let value = match mode {
            Mode::OldDefaultCapacity => old_sort_once(successor_offsets, successors, in_degree),
            Mode::NewPresized => new_sort_once(successor_offsets, successors, in_degree),
        } as u64;

        total += value;
        last_total = value;
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((node_count as u64) << 7)
                ^ ((successors.len() as u64) << 23),
        );
    }

    TopographicGraphSortCapacitySummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn old_sort_once(
    successor_offsets: &[usize],
    successors: &[usize],
    in_degree: &[usize],
) -> usize {
    let mut sorted = Vec::new();
    let mut roots = VecDeque::new();
    let mut non_roots = HashMap::new();
    sort_graph(
        successor_offsets,
        successors,
        in_degree,
        &mut sorted,
        &mut roots,
        &mut non_roots,
    )
}

fn new_sort_once(
    successor_offsets: &[usize],
    successors: &[usize],
    in_degree: &[usize],
) -> usize {
    let node_count = in_degree.len();
    let mut sorted = Vec::with_capacity(node_count);
    let mut roots = VecDeque::with_capacity(node_count);
    let mut non_roots = HashMap::with_capacity(expected_collection_capacity(node_count));
    sort_graph(
        successor_offsets,
        successors,
        in_degree,
        &mut sorted,
        &mut roots,
        &mut non_roots,
    )
}

fn sort_graph(
    successor_offsets: &[usize],
    successors: &[usize],
    in_degree: &[usize],
    sorted: &mut Vec<usize>,
    roots: &mut VecDeque<usize>,
    non_roots: &mut HashMap<usize, usize>,
) -> usize {
    for (node, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            roots.push_back(node);
        } else {
            non_roots.insert(node, degree);
        }
    }

    while let Some(next) = roots.pop_front() {
        let start = successor_offsets[next];
        let end = successor_offsets[next + 1];
        for &successor in &successors[start..end] {
            let previous_in_degree = non_roots
                .remove(&successor)
                .expect("successor must be tracked");
            let new_in_degree = previous_in_degree - 1;
            if new_in_degree == 0 {
                roots.push_back(successor);
            } else {
                non_roots.insert(successor, new_in_degree);
            }
        }
        sorted.push(next);
    }

    assert!(non_roots.is_empty(), "cycle");
    sorted.len()
}

fn input_digest(tag: u64, successor_offsets: &[usize], successors: &[usize], in_degree: &[usize]) -> u64 {
    mix64(
        tag
            ^ usize_digest(successor_offsets, 0x1656_67B1_9E37_79F9)
            ^ usize_digest(successors, 0x85EB_CA77_C2B2_AE63)
            ^ usize_digest(in_degree, 0x27D4_EB2F_1656_67C5),
    )
}

fn usize_digest(values: &[usize], tag: u64) -> u64 {
    let mut digest = mix64(tag ^ (values.len() as u64));
    for (index, value) in values.iter().enumerate() {
        digest = mix64(
            digest
                ^ (*value as u64)
                ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
        );
    }
    digest
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

    fn create_graph(node_count: usize, edges_per_node: usize) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        let mut offsets = Vec::with_capacity(node_count + 1);
        let mut successors = Vec::with_capacity(node_count * edges_per_node);
        let mut in_degree = vec![0usize; node_count];
        offsets.push(0);
        for node in 0..node_count {
            for edge in 1..=edges_per_node {
                let successor = node + edge;
                if successor >= node_count {
                    break;
                }
                successors.push(successor);
                in_degree[successor] += 1;
            }
            offsets.push(successors.len());
        }
        (offsets, successors, in_degree)
    }

    #[test]
    fn old_and_new_match_on_regular_inputs() {
        let (offsets, successors, in_degree) = create_graph(64, 4);
        assert_eq!(
            old_default_capacity_summary(128, &offsets, &successors, &in_degree),
            new_presized_summary(128, &offsets, &successors, &in_degree),
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        let (offsets, successors, in_degree) = create_graph(8, 2);
        assert_eq!(
            old_default_capacity_summary(0, &offsets, &successors, &in_degree),
            TopographicGraphSortCapacitySummary::default(),
        );
    }

    #[test]
    fn repeated_runs_are_stable() {
        let (offsets, successors, in_degree) = create_graph(24, 3);
        let first = new_presized_summary(64, &offsets, &successors, &in_degree);
        let second = new_presized_summary(64, &offsets, &successors, &in_degree);
        assert_eq!(first, second);
    }
}
