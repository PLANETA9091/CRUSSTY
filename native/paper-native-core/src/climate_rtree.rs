use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

pub const PARAMETER_COUNT: usize = 7;
pub const CHILDREN_PER_NODE: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub min: i64,
    pub max: i64,
}

impl Parameter {
    #[inline]
    pub fn distance(self, point_value: i64) -> i64 {
        let above = point_value.wrapping_sub(self.max);
        if above > 0 {
            above
        } else {
            let below = self.min.wrapping_sub(point_value);
            if below > 0 {
                below
            } else {
                0
            }
        }
    }

    #[inline]
    pub fn span(self, other: Parameter) -> Parameter {
        Parameter {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    parameter_space: [Parameter; PARAMETER_COUNT],
    kind: NodeKind,
}

#[derive(Debug)]
enum NodeKind {
    Leaf { value: i32 },
    SubTree { children: Vec<Arc<Node>> },
}

pub type NodeRef = Arc<Node>;

#[derive(Debug)]
pub struct ArenaTree {
    nodes: Vec<ArenaNode>,
    root: usize,
}

#[derive(Debug)]
struct ArenaNode {
    parameter_space: [Parameter; PARAMETER_COUNT],
    kind: ArenaNodeKind,
}

#[derive(Debug)]
enum ArenaNodeKind {
    Leaf { value: i32 },
    SubTree { children: Vec<usize> },
}

#[derive(Debug)]
struct ArenaBucket {
    parameter_space: [Parameter; PARAMETER_COUNT],
    children: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClimateRTreeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClimateRTreeBatchError {
    InvalidInputLength,
    OutputTooSmall(usize),
}

#[inline]
pub fn create_leaf(parameters: [Parameter; PARAMETER_COUNT], value: i32) -> NodeRef {
    Arc::new(Node {
        parameter_space: parameters,
        kind: NodeKind::Leaf { value },
    })
}

#[inline]
pub fn build(param_space_size: usize, mut children: Vec<NodeRef>) -> Result<NodeRef, ClimateRTreeError> {
    if children.is_empty() {
        return Err(ClimateRTreeError);
    }
    Ok(build_inner(param_space_size, &mut children))
}

#[inline]
pub fn build_from_flat(node_mins: &[i64], node_maxs: &[i64]) -> Result<NodeRef, ClimateRTreeBatchError> {
    build_from_flat_with_leaves(node_mins, node_maxs).map(|(root, _)| root)
}

#[inline]
pub fn build_from_flat_with_leaves(
    node_mins: &[i64],
    node_maxs: &[i64],
) -> Result<(NodeRef, Vec<NodeRef>), ClimateRTreeBatchError> {
    if node_mins.len() != node_maxs.len()
        || node_mins.is_empty()
        || node_mins.len() % PARAMETER_COUNT != 0
    {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let mut leaves = Vec::with_capacity(node_mins.len() / PARAMETER_COUNT);
    for (index, (node_min, node_max)) in node_mins
        .chunks_exact(PARAMETER_COUNT)
        .zip(node_maxs.chunks_exact(PARAMETER_COUNT))
        .enumerate()
    {
        let mut parameters = [Parameter { min: 0, max: 0 }; PARAMETER_COUNT];
        for parameter in 0..PARAMETER_COUNT {
            parameters[parameter] = Parameter {
                min: node_min[parameter],
                max: node_max[parameter],
            };
        }
        leaves.push(create_leaf(parameters, index as i32));
    }

    let root = build(PARAMETER_COUNT, leaves.clone()).map_err(|_| ClimateRTreeBatchError::InvalidInputLength)?;
    Ok((root, leaves))
}

#[inline]
pub fn build_arena_from_flat(
    node_mins: &[i64],
    node_maxs: &[i64],
) -> Result<ArenaTree, ClimateRTreeBatchError> {
    if node_mins.len() != node_maxs.len()
        || node_mins.is_empty()
        || node_mins.len() % PARAMETER_COUNT != 0
    {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let leaf_count = node_mins.len() / PARAMETER_COUNT;
    let mut nodes = Vec::with_capacity(leaf_count * 2);
    let mut leaves = Vec::with_capacity(leaf_count);
    for (index, (node_min, node_max)) in node_mins
        .chunks_exact(PARAMETER_COUNT)
        .zip(node_maxs.chunks_exact(PARAMETER_COUNT))
        .enumerate()
    {
        let mut parameters = [Parameter { min: 0, max: 0 }; PARAMETER_COUNT];
        for parameter in 0..PARAMETER_COUNT {
            parameters[parameter] = Parameter {
                min: node_min[parameter],
                max: node_max[parameter],
            };
        }

        let node_index = nodes.len();
        nodes.push(ArenaNode {
            parameter_space: parameters,
            kind: ArenaNodeKind::Leaf {
                value: index as i32,
            },
        });
        leaves.push(node_index);
    }

    let root = build_arena_inner(PARAMETER_COUNT, &mut leaves, &mut nodes);
    Ok(ArenaTree { nodes, root })
}

#[inline]
pub fn exact_distance(node: &NodeRef, values: &[i64; PARAMETER_COUNT]) -> i64 {
    let mut distance = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let component = node.parameter_space[parameter].distance(values[parameter]);
        distance = distance.wrapping_add(component.wrapping_mul(component));
    }
    distance
}

#[inline]
pub fn bounded_distance(node: &NodeRef, values: &[i64; PARAMETER_COUNT], limit: i64) -> i64 {
    let mut distance = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let component = node.parameter_space[parameter].distance(values[parameter]);
        distance = distance.wrapping_add(component.wrapping_mul(component));
        if distance >= limit {
            return limit;
        }
    }
    distance
}

#[inline]
pub fn search_current(
    node: &NodeRef,
    searched_values: &[i64; PARAMETER_COUNT],
    leaf: Option<&NodeRef>,
) -> (NodeRef, i64) {
    search_current_slice(node, searched_values, leaf)
}

#[inline]
pub fn search_bounded(
    node: &NodeRef,
    searched_values: &[i64; PARAMETER_COUNT],
    leaf: Option<&NodeRef>,
) -> (NodeRef, i64) {
    search_bounded_slice(node, searched_values, leaf)
}

#[inline]
pub fn search_current_index(
    node: &NodeRef,
    leaves: &[NodeRef],
    searched_values: &[i64; PARAMETER_COUNT],
    previous_index: i32,
) -> Result<(i32, i64), ClimateRTreeBatchError> {
    let leaf = leaf_from_index(leaves, previous_index);
    let (leaf, score) = search_current(node, searched_values, leaf);
    Ok((leaf_value(&leaf), score))
}

#[inline]
pub fn search_bounded_index(
    node: &NodeRef,
    leaves: &[NodeRef],
    searched_values: &[i64; PARAMETER_COUNT],
    previous_index: i32,
) -> Result<(i32, i64), ClimateRTreeBatchError> {
    let leaf = leaf_from_index(leaves, previous_index);
    let (leaf, score) = search_bounded(node, searched_values, leaf);
    Ok((leaf_value(&leaf), score))
}

#[inline]
pub fn search_current_index_borrowed(
    node: &NodeRef,
    leaves: &[NodeRef],
    searched_values: &[i64; PARAMETER_COUNT],
    previous_index: i32,
) -> Result<(i32, i64), ClimateRTreeBatchError> {
    search_index_borrowed(
        node,
        leaves,
        searched_values,
        previous_index,
        search_current_borrowed_node,
    )
}

#[inline]
pub fn search_bounded_index_borrowed(
    node: &NodeRef,
    leaves: &[NodeRef],
    searched_values: &[i64; PARAMETER_COUNT],
    previous_index: i32,
) -> Result<(i32, i64), ClimateRTreeBatchError> {
    search_index_borrowed(
        node,
        leaves,
        searched_values,
        previous_index,
        search_bounded_borrowed_node,
    )
}

#[inline]
pub fn search_bounded_index_batch_borrowed(
    node: &NodeRef,
    leaves: &[NodeRef],
    queries: &[i64],
    previous_index: i32,
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    if queries.len() % PARAMETER_COUNT != 0 {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateRTreeBatchError::OutputTooSmall(query_count));
    }

    let mut last_index = previous_index;
    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let query = query.try_into().map_err(|_| ClimateRTreeBatchError::InvalidInputLength)?;
        let (leaf_index, score) = search_bounded_index_borrowed(node, leaves, query, last_index)?;
        best_indices[query_index] = leaf_index;
        best_scores[query_index] = score;
        last_index = leaf_index;
    }

    Ok(query_count)
}

#[inline]
pub fn search_current_batch(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_cloned_batch(node, queries, best_indices, best_scores, search_current_slice)
}

#[inline]
pub fn search_current_batch_direct(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    if queries.len() % PARAMETER_COUNT != 0 {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateRTreeBatchError::OutputTooSmall(query_count));
    }

    let mut last: Option<NodeRef> = None;
    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let (leaf, score) = search_current_slice(node, query, last.as_ref());
        best_indices[query_index] = leaf_value(&leaf);
        best_scores[query_index] = score;
        last = Some(leaf);
    }

    Ok(query_count)
}

#[inline]
pub fn search_bounded_batch(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_bounded_batch_cloned(node, queries, best_indices, best_scores)
}

#[inline]
pub fn search_current_batch_borrowed(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_borrowed_batch(
        node.as_ref(),
        queries,
        best_indices,
        best_scores,
        search_current_borrowed_node,
    )
}

#[inline]
pub fn search_bounded_batch_borrowed(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_borrowed_batch(
        node.as_ref(),
        queries,
        best_indices,
        best_scores,
        search_bounded_borrowed_node,
    )
}

#[inline]
pub fn search_bounded_batch_cloned(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_cloned_batch(node, queries, best_indices, best_scores, search_bounded_slice)
}

#[inline]
pub fn search_arena_current_batch(
    tree: &ArenaTree,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_arena_batch(
        tree,
        queries,
        best_indices,
        best_scores,
        search_arena_current_node,
    )
}

#[inline]
pub fn search_arena_bounded_batch(
    tree: &ArenaTree,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateRTreeBatchError> {
    search_arena_batch(
        tree,
        queries,
        best_indices,
        best_scores,
        search_arena_bounded_node,
    )
}

pub fn parameter_space(node: &NodeRef) -> &[Parameter; PARAMETER_COUNT] {
    &node.parameter_space
}

pub fn leaf_value(node: &NodeRef) -> i32 {
    leaf_value_node(node.as_ref())
}

fn leaf_from_index<'a>(leaves: &'a [NodeRef], previous_index: i32) -> Option<&'a NodeRef> {
    usize::try_from(previous_index).ok().and_then(|index| leaves.get(index))
}

#[inline]
fn search_index_borrowed(
    node: &NodeRef,
    leaves: &[NodeRef],
    searched_values: &[i64; PARAMETER_COUNT],
    previous_index: i32,
    search: for<'a> fn(&'a Node, &[i64], Option<&'a Node>) -> (&'a Node, i64),
) -> Result<(i32, i64), ClimateRTreeBatchError> {
    let leaf = leaf_from_index(leaves, previous_index).map(|leaf| leaf.as_ref());
    let (leaf, score) = search(node.as_ref(), searched_values, leaf);
    Ok((leaf_value_node(leaf), score))
}

fn leaf_value_node(node: &Node) -> i32 {
    match &node.kind {
        NodeKind::Leaf { value } => *value,
        NodeKind::SubTree { .. } => panic!("expected leaf"),
    }
}

pub fn checksum_leaves(root: &NodeRef) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    checksum_leaves_inner(root, &mut checksum);
    checksum
}

pub fn checksum_tree(root: &NodeRef) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    checksum_tree_inner(root, &mut checksum);
    checksum
}

pub fn checksum_arena_tree(tree: &ArenaTree) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    checksum_arena_tree_inner(tree, tree.root, &mut checksum);
    checksum
}

pub fn arena_node_count(tree: &ArenaTree) -> usize {
    tree.nodes.len()
}

fn build_inner(param_space_size: usize, children: &mut Vec<NodeRef>) -> NodeRef {
    if children.len() == 1 {
        return Arc::clone(&children[0]);
    } else if children.len() <= CHILDREN_PER_NODE {
        children.sort_by(|a, b| center_sum(a, param_space_size).cmp(&center_sum(b, param_space_size)));
        return subtree(children.clone());
    }

    let mut best_cost = i64::MAX;
    let mut best_dimension = 0usize;
    let mut best_buckets: Vec<NodeRef> = Vec::new();

    for dimension in 0..param_space_size {
        sort(children, param_space_size, dimension, false);
        let buckets = bucketize(children);
        let mut cost = 0i64;
        for bucket in &buckets {
            cost = cost.wrapping_add(cost_of(parameter_space(bucket)));
        }
        if best_cost > cost {
            best_cost = cost;
            best_dimension = dimension;
            best_buckets = buckets;
        }
    }

    sort(&mut best_buckets, param_space_size, best_dimension, true);
    let mut built_children = Vec::with_capacity(best_buckets.len());
    for bucket in &best_buckets {
        let mut children = match &bucket.kind {
            NodeKind::Leaf { .. } => panic!("bucket cannot be leaf"),
            NodeKind::SubTree { children } => children.clone(),
        };
        built_children.push(build_inner(param_space_size, &mut children));
    }
    subtree(built_children)
}

fn sort(children: &mut [NodeRef], param_space_size: usize, size: usize, absolute: bool) {
    children.sort_by(|a, b| compare_nodes(a, b, param_space_size, size, absolute));
}

fn compare_nodes(
    a: &NodeRef,
    b: &NodeRef,
    param_space_size: usize,
    size: usize,
    absolute: bool,
) -> Ordering {
    compare_parameter_spaces(
        parameter_space(a),
        parameter_space(b),
        param_space_size,
        size,
        absolute,
    )
}

fn compare_parameter_spaces(
    a: &[Parameter; PARAMETER_COUNT],
    b: &[Parameter; PARAMETER_COUNT],
    param_space_size: usize,
    size: usize,
    absolute: bool,
) -> Ordering {
    for offset in 0..param_space_size {
        let dimension = (size + offset) % param_space_size;
        let ordering = compare_space_by_center(a, b, dimension, absolute);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn compare_space_by_center(
    a: &[Parameter; PARAMETER_COUNT],
    b: &[Parameter; PARAMETER_COUNT],
    size: usize,
    absolute: bool,
) -> Ordering {
    let a_value = center_value(a[size]);
    let b_value = center_value(b[size]);
    let a_key = if absolute { a_value.wrapping_abs() } else { a_value };
    let b_key = if absolute { b_value.wrapping_abs() } else { b_value };
    a_key.cmp(&b_key)
}

fn center_sum(node: &NodeRef, param_space_size: usize) -> i64 {
    let mut sum = 0i64;
    for parameter in 0..param_space_size {
        sum = sum.wrapping_add(center_value(parameter_space(node)[parameter]).wrapping_abs());
    }
    sum
}

fn center_value(parameter: Parameter) -> i64 {
    parameter.min.wrapping_add(parameter.max) / 2
}

fn bucketize(nodes: &[NodeRef]) -> Vec<NodeRef> {
    let exponent = (((nodes.len() as f64) - 0.01).ln() / (CHILDREN_PER_NODE as f64).ln()).floor();
    let bucket_size = (CHILDREN_PER_NODE as f64).powf(exponent) as usize;
    let mut buckets = Vec::with_capacity((nodes.len() + bucket_size - 1) / bucket_size);
    let mut bucket = Vec::with_capacity(bucket_size);
    for node in nodes {
        bucket.push(Arc::clone(node));
        if bucket.len() >= bucket_size {
            buckets.push(subtree(bucket));
            bucket = Vec::with_capacity(bucket_size);
        }
    }
    if !bucket.is_empty() {
        buckets.push(subtree(bucket));
    }
    buckets
}

fn subtree(children: Vec<NodeRef>) -> NodeRef {
    Arc::new(Node {
        parameter_space: build_parameter_space(&children),
        kind: NodeKind::SubTree { children },
    })
}

fn build_parameter_space(children: &[NodeRef]) -> [Parameter; PARAMETER_COUNT] {
    let mut parameters = *parameter_space(&children[0]);
    for child in &children[1..] {
        let child_space = parameter_space(child);
        for parameter in 0..PARAMETER_COUNT {
            parameters[parameter] = parameters[parameter].span(child_space[parameter]);
        }
    }
    parameters
}

fn build_arena_inner(
    param_space_size: usize,
    children: &mut Vec<usize>,
    nodes: &mut Vec<ArenaNode>,
) -> usize {
    if children.len() == 1 {
        return children[0];
    } else if children.len() <= CHILDREN_PER_NODE {
        children.sort_by(|a, b| {
            center_sum_arena(nodes, *a, param_space_size)
                .cmp(&center_sum_arena(nodes, *b, param_space_size))
        });
        return arena_subtree(nodes, children.clone());
    }

    let mut best_cost = i64::MAX;
    let mut best_dimension = 0usize;
    let mut best_buckets: Vec<ArenaBucket> = Vec::new();

    for dimension in 0..param_space_size {
        sort_arena_indices(children, nodes, param_space_size, dimension, false);
        let buckets = bucketize_arena(children, nodes);
        let mut cost = 0i64;
        for bucket in &buckets {
            cost = cost.wrapping_add(cost_of(&bucket.parameter_space));
        }
        if best_cost > cost {
            best_cost = cost;
            best_dimension = dimension;
            best_buckets = buckets;
        }
    }

    sort_arena_buckets(&mut best_buckets, param_space_size, best_dimension, true);
    let mut built_children = Vec::with_capacity(best_buckets.len());
    for mut bucket in best_buckets {
        built_children.push(build_arena_inner(param_space_size, &mut bucket.children, nodes));
    }
    arena_subtree(nodes, built_children)
}

fn sort_arena_indices(
    children: &mut [usize],
    nodes: &[ArenaNode],
    param_space_size: usize,
    size: usize,
    absolute: bool,
) {
    children.sort_by(|a, b| {
        compare_parameter_spaces(
            &nodes[*a].parameter_space,
            &nodes[*b].parameter_space,
            param_space_size,
            size,
            absolute,
        )
    });
}

fn sort_arena_buckets(
    buckets: &mut [ArenaBucket],
    param_space_size: usize,
    size: usize,
    absolute: bool,
) {
    buckets.sort_by(|a, b| {
        compare_parameter_spaces(
            &a.parameter_space,
            &b.parameter_space,
            param_space_size,
            size,
            absolute,
        )
    });
}

fn center_sum_arena(nodes: &[ArenaNode], node: usize, param_space_size: usize) -> i64 {
    let mut sum = 0i64;
    for parameter in 0..param_space_size {
        sum = sum.wrapping_add(nodes[node].parameter_space[parameter].wrapping_center_abs());
    }
    sum
}

fn bucketize_arena(children: &[usize], nodes: &[ArenaNode]) -> Vec<ArenaBucket> {
    let exponent = (((children.len() as f64) - 0.01).ln() / (CHILDREN_PER_NODE as f64).ln()).floor();
    let bucket_size = (CHILDREN_PER_NODE as f64).powf(exponent) as usize;
    let mut buckets = Vec::with_capacity((children.len() + bucket_size - 1) / bucket_size);
    let mut bucket = Vec::with_capacity(bucket_size);
    for node in children {
        bucket.push(*node);
        if bucket.len() >= bucket_size {
            buckets.push(arena_bucket(nodes, bucket));
            bucket = Vec::with_capacity(bucket_size);
        }
    }
    if !bucket.is_empty() {
        buckets.push(arena_bucket(nodes, bucket));
    }
    buckets
}

fn arena_bucket(nodes: &[ArenaNode], children: Vec<usize>) -> ArenaBucket {
    ArenaBucket {
        parameter_space: build_arena_parameter_space(nodes, &children),
        children,
    }
}

fn arena_subtree(nodes: &mut Vec<ArenaNode>, children: Vec<usize>) -> usize {
    let parameter_space = build_arena_parameter_space(nodes, &children);
    let node_index = nodes.len();
    nodes.push(ArenaNode {
        parameter_space,
        kind: ArenaNodeKind::SubTree { children },
    });
    node_index
}

fn build_arena_parameter_space(
    nodes: &[ArenaNode],
    children: &[usize],
) -> [Parameter; PARAMETER_COUNT] {
    let mut parameters = nodes[children[0]].parameter_space;
    for child in &children[1..] {
        let child_space = &nodes[*child].parameter_space;
        for parameter in 0..PARAMETER_COUNT {
            parameters[parameter] = parameters[parameter].span(child_space[parameter]);
        }
    }
    parameters
}

fn search_current_slice(
    node: &NodeRef,
    searched_values: &[i64],
    leaf: Option<&NodeRef>,
) -> (NodeRef, i64) {
    let best_distance = leaf.map_or(i64::MAX, |best| exact_distance_slice(best, searched_values));
    search_current_slice_with_best(node, searched_values, leaf, best_distance)
}

fn search_current_slice_with_best(
    node: &NodeRef,
    searched_values: &[i64],
    leaf: Option<&NodeRef>,
    best_distance: i64,
) -> (NodeRef, i64) {
    match &node.kind {
        NodeKind::Leaf { .. } => (Arc::clone(node), exact_distance_slice(node, searched_values)),
        NodeKind::SubTree { children } => {
            let mut best_distance = best_distance;
            let mut best_leaf = leaf.cloned();

            for child in children {
                match &child.kind {
                    NodeKind::Leaf { .. } => {
                        let node_distance = exact_distance_slice(child, searched_values);
                        if best_distance > node_distance {
                            best_distance = node_distance;
                            best_leaf = Some(Arc::clone(child));
                        }
                    }
                    NodeKind::SubTree { .. } => {
                        let node_distance = exact_distance_slice(child, searched_values);
                        if best_distance > node_distance {
                            let (candidate, candidate_distance) = search_current_slice_with_best(
                                child,
                                searched_values,
                                best_leaf.as_ref(),
                                best_distance,
                            );
                            if best_distance > candidate_distance {
                                best_distance = candidate_distance;
                                best_leaf = Some(candidate);
                            }
                        }
                    }
                }
            }

            (best_leaf.expect("non-empty subtree"), best_distance)
        }
    }
}

fn search_bounded_slice(
    node: &NodeRef,
    searched_values: &[i64],
    leaf: Option<&NodeRef>,
) -> (NodeRef, i64) {
    let best_distance = leaf.map_or(i64::MAX, |best| exact_distance_slice(best, searched_values));
    search_bounded_slice_with_best(node, searched_values, leaf, best_distance)
}

fn search_bounded_slice_with_best(
    node: &NodeRef,
    searched_values: &[i64],
    leaf: Option<&NodeRef>,
    best_distance: i64,
) -> (NodeRef, i64) {
    match &node.kind {
        NodeKind::Leaf { .. } => (Arc::clone(node), exact_distance_slice(node, searched_values)),
        NodeKind::SubTree { children } => {
            let mut best_distance = best_distance;
            let mut best_leaf = leaf.cloned();

            for child in children {
                let node_distance = bounded_distance_slice(child, searched_values, best_distance);
                if best_distance > node_distance {
                    match &child.kind {
                        NodeKind::Leaf { .. } => {
                            best_distance = node_distance;
                            best_leaf = Some(Arc::clone(child));
                        }
                        NodeKind::SubTree { .. } => {
                            let (candidate, candidate_distance) = search_bounded_slice_with_best(
                                child,
                                searched_values,
                                best_leaf.as_ref(),
                                best_distance,
                            );
                            if best_distance > candidate_distance {
                                best_distance = candidate_distance;
                                best_leaf = Some(candidate);
                            }
                        }
                    }
                }
            }

            (best_leaf.expect("non-empty subtree"), best_distance)
        }
    }
}

fn search_current_borrowed_node<'a>(
    node: &'a Node,
    searched_values: &[i64],
    leaf: Option<&'a Node>,
) -> (&'a Node, i64) {
    let best_distance =
        leaf.map_or(i64::MAX, |best| exact_distance_node_slice(best, searched_values));
    search_current_borrowed_node_with_best(node, searched_values, leaf, best_distance)
}

fn search_current_borrowed_node_with_best<'a>(
    node: &'a Node,
    searched_values: &[i64],
    leaf: Option<&'a Node>,
    best_distance: i64,
) -> (&'a Node, i64) {
    match &node.kind {
        NodeKind::Leaf { .. } => (node, exact_distance_node_slice(node, searched_values)),
        NodeKind::SubTree { children } => {
            let mut best_distance = best_distance;
            let mut best_leaf = leaf;

            for child in children {
                let child_node = child.as_ref();
                match &child_node.kind {
                    NodeKind::Leaf { .. } => {
                        let node_distance = exact_distance_node_slice(child_node, searched_values);
                        if best_distance > node_distance {
                            best_distance = node_distance;
                            best_leaf = Some(child_node);
                        }
                    }
                    NodeKind::SubTree { .. } => {
                        let node_distance = exact_distance_node_slice(child_node, searched_values);
                        if best_distance > node_distance {
                            let (candidate, candidate_distance) =
                                search_current_borrowed_node_with_best(
                                    child_node,
                                    searched_values,
                                    best_leaf,
                                    best_distance,
                                );
                            if best_distance > candidate_distance {
                                best_distance = candidate_distance;
                                best_leaf = Some(candidate);
                            }
                        }
                    }
                }
            }

            (best_leaf.expect("non-empty subtree"), best_distance)
        }
    }
}

fn search_bounded_borrowed_node<'a>(
    node: &'a Node,
    searched_values: &[i64],
    leaf: Option<&'a Node>,
) -> (&'a Node, i64) {
    let best_distance =
        leaf.map_or(i64::MAX, |best| exact_distance_node_slice(best, searched_values));
    search_bounded_borrowed_node_with_best(node, searched_values, leaf, best_distance)
}

fn search_bounded_borrowed_node_with_best<'a>(
    node: &'a Node,
    searched_values: &[i64],
    leaf: Option<&'a Node>,
    best_distance: i64,
) -> (&'a Node, i64) {
    match &node.kind {
        NodeKind::Leaf { .. } => (node, exact_distance_node_slice(node, searched_values)),
        NodeKind::SubTree { children } => {
            let mut best_distance = best_distance;
            let mut best_leaf = leaf;

            for child in children {
                let child_node = child.as_ref();
                let node_distance =
                    bounded_distance_node_slice(child_node, searched_values, best_distance);
                if best_distance > node_distance {
                    match &child_node.kind {
                        NodeKind::Leaf { .. } => {
                            best_distance = node_distance;
                            best_leaf = Some(child_node);
                        }
                        NodeKind::SubTree { .. } => {
                            let (candidate, candidate_distance) =
                                search_bounded_borrowed_node_with_best(
                                    child_node,
                                    searched_values,
                                    best_leaf,
                                    best_distance,
                                );
                            if best_distance > candidate_distance {
                                best_distance = candidate_distance;
                                best_leaf = Some(candidate);
                            }
                        }
                    }
                }
            }

            (best_leaf.expect("non-empty subtree"), best_distance)
        }
    }
}

fn search_arena_current_node(
    tree: &ArenaTree,
    node_index: usize,
    searched_values: &[i64],
    leaf: Option<usize>,
) -> (usize, i64) {
    let best_distance = leaf.map_or(i64::MAX, |best| {
        exact_distance_arena_slice(tree, best, searched_values)
    });
    search_arena_current_node_with_best(tree, node_index, searched_values, leaf, best_distance)
}

fn search_arena_current_node_with_best(
    tree: &ArenaTree,
    node_index: usize,
    searched_values: &[i64],
    leaf: Option<usize>,
    best_distance: i64,
) -> (usize, i64) {
    match &tree.nodes[node_index].kind {
        ArenaNodeKind::Leaf { .. } => (
            node_index,
            exact_distance_arena_slice(tree, node_index, searched_values),
        ),
        ArenaNodeKind::SubTree { children } => {
            let mut best_distance = best_distance;
            let mut best_leaf = leaf;

            for child in children {
                match &tree.nodes[*child].kind {
                    ArenaNodeKind::Leaf { .. } => {
                        let node_distance = exact_distance_arena_slice(tree, *child, searched_values);
                        if best_distance > node_distance {
                            best_distance = node_distance;
                            best_leaf = Some(*child);
                        }
                    }
                    ArenaNodeKind::SubTree { .. } => {
                        let node_distance =
                            bounded_distance_arena_slice(tree, *child, searched_values, best_distance);
                        if best_distance > node_distance {
                            let (candidate, candidate_distance) =
                                search_arena_current_node_with_best(
                                    tree,
                                    *child,
                                    searched_values,
                                    best_leaf,
                                    best_distance,
                                );
                            if best_distance > candidate_distance {
                                best_distance = candidate_distance;
                                best_leaf = Some(candidate);
                            }
                        }
                    }
                }
            }

            (best_leaf.expect("non-empty subtree"), best_distance)
        }
    }
}

fn search_arena_bounded_node(
    tree: &ArenaTree,
    node_index: usize,
    searched_values: &[i64],
    leaf: Option<usize>,
) -> (usize, i64) {
    let best_distance = leaf.map_or(i64::MAX, |best| {
        exact_distance_arena_slice(tree, best, searched_values)
    });
    search_arena_bounded_node_with_best(tree, node_index, searched_values, leaf, best_distance)
}

fn search_arena_bounded_node_with_best(
    tree: &ArenaTree,
    node_index: usize,
    searched_values: &[i64],
    leaf: Option<usize>,
    best_distance: i64,
) -> (usize, i64) {
    match &tree.nodes[node_index].kind {
        ArenaNodeKind::Leaf { .. } => (
            node_index,
            exact_distance_arena_slice(tree, node_index, searched_values),
        ),
        ArenaNodeKind::SubTree { children } => {
            let mut best_distance = best_distance;
            let mut best_leaf = leaf;

            for child in children {
                let node_distance =
                    bounded_distance_arena_slice(tree, *child, searched_values, best_distance);
                if best_distance > node_distance {
                    match &tree.nodes[*child].kind {
                        ArenaNodeKind::Leaf { .. } => {
                            best_distance = node_distance;
                            best_leaf = Some(*child);
                        }
                        ArenaNodeKind::SubTree { .. } => {
                            let (candidate, candidate_distance) =
                                search_arena_bounded_node_with_best(
                                    tree,
                                    *child,
                                    searched_values,
                                    best_leaf,
                                    best_distance,
                                );
                            if best_distance > candidate_distance {
                                best_distance = candidate_distance;
                                best_leaf = Some(candidate);
                            }
                        }
                    }
                }
            }

            (best_leaf.expect("non-empty subtree"), best_distance)
        }
    }
}

fn search_cloned_batch(
    node: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
    search: fn(&NodeRef, &[i64], Option<&NodeRef>) -> (NodeRef, i64),
) -> Result<usize, ClimateRTreeBatchError> {
    if queries.len() % PARAMETER_COUNT != 0 {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateRTreeBatchError::OutputTooSmall(query_count));
    }

    let mut last: Option<NodeRef> = None;
    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let (leaf, score) = search(node, query, last.as_ref());
        best_indices[query_index] = leaf_value(&leaf);
        best_scores[query_index] = score;
        last = Some(leaf);
    }

    Ok(query_count)
}

fn search_borrowed_batch(
    node: &Node,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
    search: for<'a> fn(&'a Node, &[i64], Option<&'a Node>) -> (&'a Node, i64),
) -> Result<usize, ClimateRTreeBatchError> {
    if queries.len() % PARAMETER_COUNT != 0 {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateRTreeBatchError::OutputTooSmall(query_count));
    }

    let mut last = None;
    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let (leaf, score) = search(node, query, last);
        best_indices[query_index] = leaf_value_node(leaf);
        best_scores[query_index] = score;
        last = Some(leaf);
    }

    Ok(query_count)
}

fn search_arena_batch(
    tree: &ArenaTree,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
    search: fn(&ArenaTree, usize, &[i64], Option<usize>) -> (usize, i64),
) -> Result<usize, ClimateRTreeBatchError> {
    if queries.len() % PARAMETER_COUNT != 0 {
        return Err(ClimateRTreeBatchError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateRTreeBatchError::OutputTooSmall(query_count));
    }

    let mut last = None;
    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let (leaf, score) = search(tree, tree.root, query, last);
        best_indices[query_index] = arena_leaf_value(tree, leaf);
        best_scores[query_index] = score;
        last = Some(leaf);
    }

    Ok(query_count)
}

fn cost_of(parameters: &[Parameter; PARAMETER_COUNT]) -> i64 {
    let mut cost = 0i64;
    for parameter in parameters {
        cost = cost.wrapping_add(parameter.max.wrapping_sub(parameter.min).wrapping_abs());
    }
    cost
}

trait ParameterCenter {
    fn wrapping_center_abs(self) -> i64;
}

impl ParameterCenter for Parameter {
    #[inline]
    fn wrapping_center_abs(self) -> i64 {
        center_value(self).wrapping_abs()
    }
}

fn exact_distance_slice(node: &NodeRef, values: &[i64]) -> i64 {
    exact_distance_node_slice(node.as_ref(), values)
}

fn exact_distance_node_slice(node: &Node, values: &[i64]) -> i64 {
    let mut distance = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let component = node.parameter_space[parameter].distance(values[parameter]);
        distance = distance.wrapping_add(component.wrapping_mul(component));
    }
    distance
}

fn bounded_distance_slice(node: &NodeRef, values: &[i64], limit: i64) -> i64 {
    bounded_distance_node_slice(node.as_ref(), values, limit)
}

fn bounded_distance_node_slice(node: &Node, values: &[i64], limit: i64) -> i64 {
    let mut distance = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let component = node.parameter_space[parameter].distance(values[parameter]);
        distance = distance.wrapping_add(component.wrapping_mul(component));
        if distance >= limit {
            return limit;
        }
    }
    distance
}

fn exact_distance_arena_slice(tree: &ArenaTree, node_index: usize, values: &[i64]) -> i64 {
    let mut distance = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let component = tree.nodes[node_index].parameter_space[parameter].distance(values[parameter]);
        distance = distance.wrapping_add(component.wrapping_mul(component));
    }
    distance
}

fn bounded_distance_arena_slice(
    tree: &ArenaTree,
    node_index: usize,
    values: &[i64],
    limit: i64,
) -> i64 {
    let mut distance = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let component = tree.nodes[node_index].parameter_space[parameter].distance(values[parameter]);
        distance = distance.wrapping_add(component.wrapping_mul(component));
        if distance >= limit {
            return limit;
        }
    }
    distance
}

fn arena_leaf_value(tree: &ArenaTree, node_index: usize) -> i32 {
    match &tree.nodes[node_index].kind {
        ArenaNodeKind::Leaf { value } => *value,
        ArenaNodeKind::SubTree { .. } => panic!("expected leaf"),
    }
}

fn checksum_leaves_inner(node: &NodeRef, checksum: &mut i64) {
    match &node.kind {
        NodeKind::Leaf { value } => {
            for parameter in &node.parameter_space {
                *checksum = checksum.wrapping_mul(31).wrapping_add(parameter.min);
                *checksum = checksum.wrapping_mul(31).wrapping_add(parameter.max);
            }
            *checksum = checksum.wrapping_mul(31).wrapping_add(*value as i64);
        }
        NodeKind::SubTree { children } => {
            for child in children {
                checksum_leaves_inner(child, checksum);
            }
        }
    }
}

fn checksum_tree_inner(node: &NodeRef, checksum: &mut i64) {
    for parameter in &node.parameter_space {
        *checksum = checksum.wrapping_mul(31).wrapping_add(parameter.min);
        *checksum = checksum.wrapping_mul(31).wrapping_add(parameter.max);
    }
    match &node.kind {
        NodeKind::Leaf { value } => {
            *checksum = checksum.wrapping_mul(31).wrapping_add(*value as i64);
        }
        NodeKind::SubTree { children } => {
            *checksum = checksum.wrapping_mul(31).wrapping_add(children.len() as i64);
            for child in children {
                *checksum ^= checksum_tree_inner_value(child);
            }
        }
    }
}

fn checksum_tree_inner_value(node: &NodeRef) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    checksum_tree_inner(node, &mut checksum);
    checksum
}

fn checksum_arena_tree_inner(tree: &ArenaTree, node_index: usize, checksum: &mut i64) {
    for parameter in &tree.nodes[node_index].parameter_space {
        *checksum = checksum.wrapping_mul(31).wrapping_add(parameter.min);
        *checksum = checksum.wrapping_mul(31).wrapping_add(parameter.max);
    }
    match &tree.nodes[node_index].kind {
        ArenaNodeKind::Leaf { value } => {
            *checksum = checksum.wrapping_mul(31).wrapping_add(*value as i64);
        }
        ArenaNodeKind::SubTree { children } => {
            *checksum = checksum.wrapping_mul(31).wrapping_add(children.len() as i64);
            for child in children {
                *checksum ^= checksum_arena_tree_inner_value(tree, *child);
            }
        }
    }
}

fn checksum_arena_tree_inner_value(tree: &ArenaTree, node_index: usize) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    checksum_arena_tree_inner(tree, node_index, &mut checksum);
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn parameter_distance_matches_reference() {
        let parameter = Parameter { min: 10, max: 20 };
        assert_eq!(parameter.distance(5), 5);
        assert_eq!(parameter.distance(10), 0);
        assert_eq!(parameter.distance(15), 0);
        assert_eq!(parameter.distance(25), 5);
    }

    #[test]
    fn build_and_search_picks_same_leaf() {
        let leaf_a = create_leaf([Parameter { min: 0, max: 0 }; PARAMETER_COUNT], 1);
        let leaf_b = create_leaf([Parameter { min: 10, max: 10 }; PARAMETER_COUNT], 2);
        let root = build(PARAMETER_COUNT, vec![leaf_a, leaf_b]).unwrap();
        let query = [5, 5, 5, 5, 5, 5, 5];
        let (current, current_score) = search_current(&root, &query, None);
        let (bounded, bounded_score) = search_bounded(&root, &query, None);
        assert_eq!(leaf_value(&current), 1);
        assert_eq!(leaf_value(&bounded), 1);
        assert_eq!(current_score, 175);
        assert_eq!(bounded_score, 175);
    }

    #[test]
    fn build_from_flat_and_batch_search_match_scalar_search() {
        let node_mins = vec![
            0, 0, 0, 0, 0, 0, 0, //
            10, 10, 10, 10, 10, 10, 10, //
            20, 20, 20, 20, 20, 20, 20,
        ];
        let node_maxs = vec![
            0, 0, 0, 0, 0, 0, 0, //
            10, 10, 10, 10, 10, 10, 10, //
            20, 20, 20, 20, 20, 20, 20,
        ];
        let root = build_from_flat(&node_mins, &node_maxs).unwrap();

        let queries = vec![
            1, 1, 1, 1, 1, 1, 1, //
            8, 8, 8, 8, 8, 8, 8, //
            18, 18, 18, 18, 18, 18, 18,
        ];
        let mut current_indices = [0i32; 3];
        let mut current_scores = [0i64; 3];
        let mut bounded_indices = [0i32; 3];
        let mut bounded_scores = [0i64; 3];

        assert_eq!(
            search_current_batch(&root, &queries, &mut current_indices, &mut current_scores).unwrap(),
            3
        );
        assert_eq!(
            search_bounded_batch(&root, &queries, &mut bounded_indices, &mut bounded_scores).unwrap(),
            3
        );

        let expected_queries = [
            [1, 1, 1, 1, 1, 1, 1],
            [8, 8, 8, 8, 8, 8, 8],
            [18, 18, 18, 18, 18, 18, 18],
        ];
        let mut current_last = None;
        let mut bounded_last = None;
        for (index, query) in expected_queries.iter().enumerate() {
            let (current, current_score) = search_current(&root, query, current_last.as_ref());
            let (bounded, bounded_score) = search_bounded(&root, query, bounded_last.as_ref());
            assert_eq!(leaf_value(&current), current_indices[index]);
            assert_eq!(current_score, current_scores[index]);
            assert_eq!(leaf_value(&bounded), bounded_indices[index]);
            assert_eq!(bounded_score, bounded_scores[index]);
            current_last = Some(current);
            bounded_last = Some(bounded);
        }

        assert_eq!(current_indices, bounded_indices);
        assert_eq!(current_scores, bounded_scores);
    }

    #[test]
    fn build_from_flat_with_leaves_and_index_search_match_batch_search() {
        let node_mins = vec![
            0, 0, 0, 0, 0, 0, 0, //
            10, 10, 10, 10, 10, 10, 10, //
            20, 20, 20, 20, 20, 20, 20,
        ];
        let node_maxs = node_mins.clone();
        let (root, leaves) = build_from_flat_with_leaves(&node_mins, &node_maxs).unwrap();
        let queries = [
            [1, 1, 1, 1, 1, 1, 1],
            [8, 8, 8, 8, 8, 8, 8],
            [18, 18, 18, 18, 18, 18, 18],
        ];

        let mut current_last = None;
        let mut bounded_last = None;
        let mut current_previous = -1;
        let mut bounded_previous = -1;
        let mut borrowed_previous = -1;
        for query in &queries {
            let (current, current_score) = search_current(&root, query, current_last.as_ref());
            let (bounded, bounded_score) = search_bounded(&root, query, bounded_last.as_ref());
            let (current_index, current_index_score) =
                search_current_index(&root, &leaves, query, current_previous).unwrap();
            let (bounded_index, bounded_index_score) =
                search_bounded_index(&root, &leaves, query, bounded_previous).unwrap();
            let (borrowed_index, borrowed_index_score) =
                search_bounded_index_borrowed(&root, &leaves, query, borrowed_previous).unwrap();
            assert_eq!(leaf_value(&current), current_index);
            assert_eq!(current_score, current_index_score);
            assert_eq!(leaf_value(&bounded), bounded_index);
            assert_eq!(bounded_score, bounded_index_score);
            assert_eq!(bounded_index, borrowed_index);
            assert_eq!(bounded_score, borrowed_index_score);
            current_last = Some(current);
            bounded_last = Some(bounded);
            current_previous = current_index;
            bounded_previous = bounded_index;
            borrowed_previous = borrowed_index;
        }
    }

    #[test]
    fn arena_build_from_flat_matches_node_ref_tree() {
        let node_mins = vec![
            -5, -4, -3, -2, -1, 0, 1, //
            10, 10, 10, 10, 10, 10, 10, //
            15, 14, 13, 12, 11, 10, 9, //
            30, 30, 30, 30, 30, 30, 30, //
            -20, -19, -18, -17, -16, -15, -14, //
            40, 41, 42, 43, 44, 45, 46, //
            50, 50, 50, 50, 50, 50, 50,
        ];
        let node_maxs = vec![
            5, 4, 3, 2, 1, 0, 1, //
            11, 11, 11, 11, 11, 11, 11, //
            16, 15, 14, 13, 12, 11, 10, //
            31, 31, 31, 31, 31, 31, 31, //
            -10, -9, -8, -7, -6, -5, -4, //
            41, 42, 43, 44, 45, 46, 47, //
            51, 51, 51, 51, 51, 51, 51,
        ];
        let node_ref_tree = build_from_flat(&node_mins, &node_maxs).unwrap();
        let arena_tree = build_arena_from_flat(&node_mins, &node_maxs).unwrap();

        assert_eq!(checksum_tree(&node_ref_tree), checksum_arena_tree(&arena_tree));
        assert!(arena_node_count(&arena_tree) >= node_mins.len() / PARAMETER_COUNT);
    }

    #[test]
    fn arena_batch_search_matches_node_ref_batch_search() {
        let node_mins = vec![
            0, 0, 0, 0, 0, 0, 0, //
            10, 10, 10, 10, 10, 10, 10, //
            20, 20, 20, 20, 20, 20, 20, //
            30, 30, 30, 30, 30, 30, 30, //
            40, 40, 40, 40, 40, 40, 40, //
            50, 50, 50, 50, 50, 50, 50, //
            60, 60, 60, 60, 60, 60, 60,
        ];
        let node_maxs = node_mins.clone();
        let queries = vec![
            1, 1, 1, 1, 1, 1, 1, //
            18, 18, 18, 18, 18, 18, 18, //
            55, 55, 55, 55, 55, 55, 55, //
            100, 100, 100, 100, 100, 100, 100,
        ];

        let node_ref_tree = build_from_flat(&node_mins, &node_maxs).unwrap();
        let arena_tree = build_arena_from_flat(&node_mins, &node_maxs).unwrap();
        let mut node_ref_indices = [0i32; 4];
        let mut node_ref_scores = [0i64; 4];
        let mut arena_indices = [0i32; 4];
        let mut arena_scores = [0i64; 4];

        assert_eq!(
            search_current_batch(&node_ref_tree, &queries, &mut node_ref_indices, &mut node_ref_scores)
                .unwrap(),
            4
        );
        assert_eq!(
            search_arena_current_batch(&arena_tree, &queries, &mut arena_indices, &mut arena_scores)
                .unwrap(),
            4
        );
        assert_eq!(node_ref_indices, arena_indices);
        assert_eq!(node_ref_scores, arena_scores);

        assert_eq!(
            search_bounded_batch(&node_ref_tree, &queries, &mut node_ref_indices, &mut node_ref_scores)
                .unwrap(),
            4
        );
        assert_eq!(
            search_arena_bounded_batch(&arena_tree, &queries, &mut arena_indices, &mut arena_scores)
                .unwrap(),
            4
        );
        assert_eq!(node_ref_indices, arena_indices);
        assert_eq!(node_ref_scores, arena_scores);
    }

    #[test]
    fn batch_search_variants_match() {
        let node_mins = vec![
            0, 0, 0, 0, 0, 0, 0, //
            10, 10, 10, 10, 10, 10, 10, //
            20, 20, 20, 20, 20, 20, 20, //
            30, 30, 30, 30, 30, 30, 30, //
            40, 40, 40, 40, 40, 40, 40, //
            50, 50, 50, 50, 50, 50, 50, //
            60, 60, 60, 60, 60, 60, 60,
        ];
        let node_maxs = node_mins.clone();
        let queries = vec![
            1, 1, 1, 1, 1, 1, 1, //
            18, 18, 18, 18, 18, 18, 18, //
            55, 55, 55, 55, 55, 55, 55, //
            100, 100, 100, 100, 100, 100, 100,
        ];

        let root = build_from_flat(&node_mins, &node_maxs).unwrap();
        let mut borrowed_indices = [0i32; 4];
        let mut borrowed_scores = [0i64; 4];
        let mut cloned_indices = [0i32; 4];
        let mut cloned_scores = [0i64; 4];

        assert_eq!(
            search_current_batch(&root, &queries, &mut cloned_indices, &mut cloned_scores)
                .unwrap(),
            4
        );
        assert_eq!(
            search_current_batch_borrowed(
                &root,
                &queries,
                &mut borrowed_indices,
                &mut borrowed_scores,
            )
            .unwrap(),
            4
        );
        assert_eq!(
            search_bounded_batch(&root, &queries, &mut borrowed_indices, &mut borrowed_scores)
                .unwrap(),
            4
        );
        assert_eq!(
            search_bounded_batch_cloned(&root, &queries, &mut cloned_indices, &mut cloned_scores)
                .unwrap(),
            4
        );
        assert_eq!(borrowed_indices, cloned_indices);
        assert_eq!(borrowed_scores, cloned_scores);
    }

    #[test]
    fn shared_handle_search_is_thread_safe() {
        let node_mins = vec![
            -30, -30, -30, -30, -30, -30, -30, //
            -10, -10, -10, -10, -10, -10, -10, //
            0, 0, 0, 0, 0, 0, 0, //
            10, 10, 10, 10, 10, 10, 10, //
            20, 20, 20, 20, 20, 20, 20, //
            30, 30, 30, 30, 30, 30, 30, //
            40, 40, 40, 40, 40, 40, 40,
        ];
        let node_maxs = node_mins.clone();
        let (root, leaves) = build_from_flat_with_leaves(&node_mins, &node_maxs).unwrap();

        let queries = [
            [1, 1, 1, 1, 1, 1, 1],
            [18, 18, 18, 18, 18, 18, 18],
            [-20, -20, -20, -20, -20, -20, -20],
            [55, 55, 55, 55, 55, 55, 55],
        ];

        let mut expected_indices = [0i32; 4];
        let mut expected_scores = [0i64; 4];
        let mut previous_index = -1;
        for (query_index, query) in queries.iter().enumerate() {
            let (index, score) =
                search_bounded_index(&root, &leaves, query, previous_index).unwrap();
            expected_indices[query_index] = index;
            expected_scores[query_index] = score;
            previous_index = index;
        }

        let mut workers = Vec::new();
        for _ in 0..8 {
            let root = Arc::clone(&root);
            let leaves = leaves.clone();
            workers.push(std::thread::spawn(move || {
                let mut previous_index = -1;
                for (query_index, query) in queries.iter().enumerate() {
                    let (index, score) =
                        search_bounded_index(&root, &leaves, query, previous_index).unwrap();
                    assert_eq!(index, expected_indices[query_index]);
                    assert_eq!(score, expected_scores[query_index]);
                    previous_index = index;
                }
            }));
        }

        for worker in workers {
            worker.join().unwrap();
        }
    }
}
