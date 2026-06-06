use std::collections::HashSet;

pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const MARKER_CACHE_TAG: u64 = 0xD91C_45A7_3B6E_28F1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MarkerCacheSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_marker_cache_summary(
    iterations: usize,
    roots: usize,
    depth: usize,
) -> MarkerCacheSummary {
    run_summary(iterations, roots, depth, Mode::Old)
}

pub fn cached_marker_cache_summary(
    iterations: usize,
    roots: usize,
    depth: usize,
) -> MarkerCacheSummary {
    run_summary(iterations, roots, depth, Mode::Cached)
}

#[derive(Clone, Copy)]
enum Mode {
    Old,
    Cached,
}

fn run_summary(iterations: usize, roots: usize, depth: usize, mode: Mode) -> MarkerCacheSummary {
    if iterations == 0 {
        return MarkerCacheSummary::default();
    }

    let graph = build_graph(roots, depth);
    let shape_digest = input_digest(roots, depth, graph.nodes.len(), graph.roots.len());
    let mut total = 0u64;
    let mut checksum = 0u64;
    let mut last_total = 0u64;

    for iteration in 0..iterations {
        let result = run_once(&graph, matches!(mode, Mode::Cached));
        total += result.guard;
        last_total = result.marker_allocations;
        checksum = mix64(
            checksum
                ^ result.guard
                ^ result.marker_allocations
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((roots as u64) << 7)
                ^ ((depth as u64) << 19),
        );
    }

    MarkerCacheSummary {
        count: iterations as u64,
        total,
        checksum,
        last_total,
    }
}

fn run_once(graph: &Graph, cached: bool) -> EvalResult {
    let mut seen_markers = HashSet::with_capacity(expected_collection_capacity(graph.marker_count()));
    let mut guard = 0u64;
    let mut marker_allocations = 0u64;

    for &root in &graph.roots {
        let result = eval_node(graph, root, cached, &mut seen_markers);
        guard += result.guard;
        marker_allocations += result.marker_allocations;
    }

    EvalResult {
        guard,
        marker_allocations,
    }
}

fn eval_node(graph: &Graph, node_index: usize, cached: bool, seen_markers: &mut HashSet<usize>) -> EvalResult {
    match graph.nodes[node_index] {
        Node::Leaf(id) => EvalResult {
            guard: id as u64,
            marker_allocations: 0,
        },
        Node::Marker { wrapped } => {
            let child = eval_node(graph, wrapped, cached, seen_markers);
            let mut marker_allocations = child.marker_allocations;
            if cached {
                if seen_markers.insert(node_index) {
                    marker_allocations += 1;
                }
            } else {
                marker_allocations += 1;
            }
            EvalResult {
                guard: 31 * child.guard + 1,
                marker_allocations,
            }
        }
        Node::Pair { left, right } => {
            let left = eval_node(graph, left, cached, seen_markers);
            let right = eval_node(graph, right, cached, seen_markers);
            EvalResult {
                guard: 17 * left.guard + 13 * right.guard + 2,
                marker_allocations: left.marker_allocations + right.marker_allocations,
            }
        }
    }
}

fn build_graph(roots: usize, depth: usize) -> Graph {
    let mut nodes = Vec::with_capacity(1 + depth + roots * 3);
    nodes.push(Node::Leaf(0));
    let mut shared = 0usize;
    for _ in 0..depth {
        let next = nodes.len();
        nodes.push(Node::Marker { wrapped: shared });
        shared = next;
    }

    let mut root_nodes = Vec::with_capacity(roots);
    for index in 0..roots {
        let leaf = nodes.len();
        nodes.push(Node::Leaf(index + 1));
        let marker = nodes.len();
        nodes.push(Node::Marker { wrapped: leaf });
        let pair = nodes.len();
        nodes.push(Node::Pair { left: shared, right: marker });
        root_nodes.push(pair);
    }

    Graph {
        nodes,
        roots: root_nodes,
    }
}

fn input_digest(roots: usize, depth: usize, node_count: usize, root_count: usize) -> u64 {
    mix64(
        MARKER_CACHE_TAG
            ^ (roots as u64)
            ^ ((depth as u64) << 16)
            ^ ((node_count as u64) << 32)
            ^ ((root_count as u64) << 48),
    )
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

struct Graph {
    nodes: Vec<Node>,
    roots: Vec<usize>,
}

impl Graph {
    fn marker_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| matches!(node, Node::Marker { .. }))
            .count()
    }
}

#[derive(Clone, Copy)]
enum Node {
    Leaf(usize),
    Marker { wrapped: usize },
    Pair { left: usize, right: usize },
}

#[derive(Clone, Copy)]
struct EvalResult {
    guard: u64,
    marker_allocations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_cached_match_on_regular_inputs() {
        let old = old_marker_cache_summary(8, 16, 6);
        let cached = cached_marker_cache_summary(8, 16, 6);
        assert_eq!(old.total, cached.total);
        assert!(old.last_total > cached.last_total);
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(old_marker_cache_summary(0, 16, 6), MarkerCacheSummary::default());
    }
}
