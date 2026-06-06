use std::env;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

use paper_native_core::climate_rtree::{
    arena_node_count, build_arena_from_flat, build_from_flat, checksum_arena_tree, checksum_tree,
    create_leaf, leaf_value, parameter_space, search_arena_bounded_batch,
    search_arena_current_batch, search_bounded_batch, search_current_batch, ArenaTree, NodeRef,
    Parameter, PARAMETER_COUNT, search_bounded, search_current,
};

static SINK: AtomicI64 = AtomicI64::new(0);

fn main() {
    let leaves_count = setting("LEAVES", 1400);
    let queries_count = setting("QUERIES", 120_000);
    let warmup = setting("WARMUP", 2);
    let rounds = setting("ROUNDS", 4);

    let (leaves, node_mins, node_maxs) = create_inputs(leaves_count);
    let input_leaves_checksum = checksum_inputs(&leaves);
    let random_queries = create_random_queries(queries_count);
    let walk_queries = create_walk_queries(queries_count);
    let random_queries_flat = flatten_queries(&random_queries);
    let walk_queries_flat = flatten_queries(&walk_queries);
    let mut rc_indices = vec![0i32; queries_count];
    let mut rc_scores = vec![0i64; queries_count];
    let mut arena_indices = vec![0i32; queries_count];
    let mut arena_scores = vec![0i64; queries_count];

    let rc_current_root = build_from_flat(&node_mins, &node_maxs).expect("non-empty leaves");
    let rc_bounded_root = build_from_flat(&node_mins, &node_maxs).expect("non-empty leaves");
    let arena_root = build_arena_from_flat(&node_mins, &node_maxs).expect("non-empty leaves");
    let rc_tree_checksum = checksum_tree(&rc_current_root);
    let arena_tree_checksum = checksum_arena_tree(&arena_root);
    if rc_tree_checksum != arena_tree_checksum {
        panic!(
            "tree checksum mismatch rc={} arena={}",
            rc_tree_checksum, arena_tree_checksum
        );
    }

    verify_equivalence(&rc_current_root, &rc_bounded_root, &arena_root, &random_queries, &random_queries_flat);
    verify_equivalence(&rc_current_root, &rc_bounded_root, &arena_root, &walk_queries, &walk_queries_flat);

    for _ in 0..warmup {
        SINK.fetch_xor(
            run_rc_lifecycle_current(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_rc_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_arena_lifecycle_current(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_arena_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_rc_lifecycle_current(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_rc_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_arena_lifecycle_current(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            ),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_arena_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            ),
            Ordering::Relaxed,
        );
    }

    let mut rc_current_random_best = u128::MAX;
    let mut rc_bounded_random_best = u128::MAX;
    let mut arena_current_random_best = u128::MAX;
    let mut arena_bounded_random_best = u128::MAX;
    let mut rc_current_walk_best = u128::MAX;
    let mut rc_bounded_walk_best = u128::MAX;
    let mut arena_current_walk_best = u128::MAX;
    let mut arena_bounded_walk_best = u128::MAX;

    for _ in 0..rounds {
        rc_current_random_best = rc_current_random_best.min(time(|| {
            run_rc_lifecycle_current(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            )
        }));
        rc_bounded_random_best = rc_bounded_random_best.min(time(|| {
            run_rc_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            )
        }));
        arena_current_random_best = arena_current_random_best.min(time(|| {
            run_arena_lifecycle_current(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            )
        }));
        arena_bounded_random_best = arena_bounded_random_best.min(time(|| {
            run_arena_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &random_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            )
        }));

        rc_current_walk_best = rc_current_walk_best.min(time(|| {
            run_rc_lifecycle_current(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            )
        }));
        rc_bounded_walk_best = rc_bounded_walk_best.min(time(|| {
            run_rc_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut rc_indices,
                &mut rc_scores,
            )
        }));
        arena_current_walk_best = arena_current_walk_best.min(time(|| {
            run_arena_lifecycle_current(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            )
        }));
        arena_bounded_walk_best = arena_bounded_walk_best.min(time(|| {
            run_arena_lifecycle_bounded(
                &node_mins,
                &node_maxs,
                &walk_queries_flat,
                &mut arena_indices,
                &mut arena_scores,
            )
        }));
    }

    let rc_current_random_checksum = run_rc_lifecycle_current(
        &node_mins,
        &node_maxs,
        &random_queries_flat,
        &mut rc_indices,
        &mut rc_scores,
    );
    let rc_bounded_random_checksum = run_rc_lifecycle_bounded(
        &node_mins,
        &node_maxs,
        &random_queries_flat,
        &mut rc_indices,
        &mut rc_scores,
    );
    let arena_current_random_checksum = run_arena_lifecycle_current(
        &node_mins,
        &node_maxs,
        &random_queries_flat,
        &mut arena_indices,
        &mut arena_scores,
    );
    let arena_bounded_random_checksum = run_arena_lifecycle_bounded(
        &node_mins,
        &node_maxs,
        &random_queries_flat,
        &mut arena_indices,
        &mut arena_scores,
    );
    let rc_current_walk_checksum = run_rc_lifecycle_current(
        &node_mins,
        &node_maxs,
        &walk_queries_flat,
        &mut rc_indices,
        &mut rc_scores,
    );
    let rc_bounded_walk_checksum = run_rc_lifecycle_bounded(
        &node_mins,
        &node_maxs,
        &walk_queries_flat,
        &mut rc_indices,
        &mut rc_scores,
    );
    let arena_current_walk_checksum = run_arena_lifecycle_current(
        &node_mins,
        &node_maxs,
        &walk_queries_flat,
        &mut arena_indices,
        &mut arena_scores,
    );
    let arena_bounded_walk_checksum = run_arena_lifecycle_bounded(
        &node_mins,
        &node_maxs,
        &walk_queries_flat,
        &mut arena_indices,
        &mut arena_scores,
    );

    if rc_current_random_checksum != rc_bounded_random_checksum
        || rc_current_random_checksum != arena_current_random_checksum
        || rc_current_random_checksum != arena_bounded_random_checksum
        || rc_current_walk_checksum != rc_bounded_walk_checksum
        || rc_current_walk_checksum != arena_current_walk_checksum
        || rc_current_walk_checksum != arena_bounded_walk_checksum
    {
        panic!("checksum mismatch");
    }

    println!("leaves={}", leaves_count);
    println!("queries={}", queries_count);
    println!("warmup={} rounds={}", warmup, rounds);
    println!("arena_node_count={}", arena_node_count(&arena_root));
    println!("input_leaves_checksum={}", input_leaves_checksum);
    println!("rc_tree_checksum={}", rc_tree_checksum);
    println!("arena_tree_checksum={}", arena_tree_checksum);
    println!("random_queries_checksum={}", checksum_queries(&random_queries));
    println!("walk_queries_checksum={}", checksum_queries(&walk_queries));
    println!("rc_batch_current_random_lifecycle_best_ms={:.3}", millis(rc_current_random_best));
    println!("rc_batch_bounded_random_lifecycle_best_ms={:.3}", millis(rc_bounded_random_best));
    println!(
        "rc_batch_bounded_random_lifecycle_speedup={:.3}x",
        rc_current_random_best as f64 / rc_bounded_random_best as f64
    );
    println!("arena_current_random_lifecycle_best_ms={:.3}", millis(arena_current_random_best));
    println!("arena_bounded_random_lifecycle_best_ms={:.3}", millis(arena_bounded_random_best));
    println!(
        "arena_current_random_lifecycle_speedup_vs_rc={:.3}x",
        rc_current_random_best as f64 / arena_current_random_best as f64
    );
    println!(
        "arena_bounded_random_lifecycle_speedup_vs_rc={:.3}x",
        rc_bounded_random_best as f64 / arena_bounded_random_best as f64
    );
    println!("rc_batch_current_walk_lifecycle_best_ms={:.3}", millis(rc_current_walk_best));
    println!("rc_batch_bounded_walk_lifecycle_best_ms={:.3}", millis(rc_bounded_walk_best));
    println!(
        "rc_batch_bounded_walk_lifecycle_speedup={:.3}x",
        rc_current_walk_best as f64 / rc_bounded_walk_best as f64
    );
    println!("arena_current_walk_lifecycle_best_ms={:.3}", millis(arena_current_walk_best));
    println!("arena_bounded_walk_lifecycle_best_ms={:.3}", millis(arena_bounded_walk_best));
    println!(
        "arena_current_walk_lifecycle_speedup_vs_rc={:.3}x",
        rc_current_walk_best as f64 / arena_current_walk_best as f64
    );
    println!(
        "arena_bounded_walk_lifecycle_speedup_vs_rc={:.3}x",
        rc_bounded_walk_best as f64 / arena_bounded_walk_best as f64
    );
    println!("rc_arena_lifecycle_equivalence=PASS");
    println!("random_checksum={}", rc_current_random_checksum);
    println!("walk_checksum={}", rc_current_walk_checksum);
    println!("sink={}", SINK.load(Ordering::Relaxed));
}

fn setting(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn create_inputs(leaves_count: usize) -> (Vec<NodeRef>, Vec<i64>, Vec<i64>) {
    let mut random = JavaRandom::new(0xC11A7E5EED);
    let mut leaves = Vec::with_capacity(leaves_count);
    let mut node_mins = Vec::with_capacity(leaves_count * PARAMETER_COUNT);
    let mut node_maxs = Vec::with_capacity(leaves_count * PARAMETER_COUNT);
    for value in 0..leaves_count {
        let mut parameters = [Parameter { min: 0, max: 0 }; PARAMETER_COUNT];
        for parameter in &mut parameters {
            let center = random.next_int(80_001) as i64 - 40_000;
            let radius = random.next_int(7_001) as i64;
            let min = center - radius;
            let max = center + radius;
            node_mins.push(min);
            node_maxs.push(max);
            *parameter = Parameter { min, max };
        }
        leaves.push(create_leaf(parameters, value as i32));
    }
    (leaves, node_mins, node_maxs)
}

fn create_random_queries(queries_count: usize) -> Vec<[i64; PARAMETER_COUNT]> {
    let mut random = JavaRandom::new(0x57A7E5EED);
    let mut queries = Vec::with_capacity(queries_count);
    for _ in 0..queries_count {
        let mut query = [0i64; PARAMETER_COUNT];
        for parameter in &mut query {
            *parameter = random.next_int(100_001) as i64 - 50_000;
        }
        queries.push(query);
    }
    queries
}

fn create_walk_queries(queries_count: usize) -> Vec<[i64; PARAMETER_COUNT]> {
    let mut random = JavaRandom::new(0xB10C1A7E5);
    let mut queries = Vec::with_capacity(queries_count);
    let mut current = [0i64; PARAMETER_COUNT];
    for _ in 0..queries_count {
        for parameter in &mut current {
            *parameter += random.next_int(2001) as i64 - 1000;
            if *parameter > 50_000 {
                *parameter = 50_000;
            } else if *parameter < -50_000 {
                *parameter = -50_000;
            }
        }
        queries.push(current);
    }
    queries
}

fn flatten_queries(queries: &[[i64; PARAMETER_COUNT]]) -> Vec<i64> {
    let mut flat = Vec::with_capacity(queries.len() * PARAMETER_COUNT);
    for query in queries {
        flat.extend_from_slice(query);
    }
    flat
}

fn verify_equivalence(
    rc_current_root: &NodeRef,
    rc_bounded_root: &NodeRef,
    arena_root: &ArenaTree,
    queries: &[[i64; PARAMETER_COUNT]],
    queries_flat: &[i64],
) {
    let mut rc_current_indices = vec![0i32; queries.len()];
    let mut rc_current_scores = vec![0i64; queries.len()];
    let mut rc_bounded_indices = vec![0i32; queries.len()];
    let mut rc_bounded_scores = vec![0i64; queries.len()];
    let mut arena_current_indices = vec![0i32; queries.len()];
    let mut arena_current_scores = vec![0i64; queries.len()];
    let mut arena_bounded_indices = vec![0i32; queries.len()];
    let mut arena_bounded_scores = vec![0i64; queries.len()];

    let rc_current_checksum = fill_current_arrays(
        rc_current_root,
        queries,
        &mut rc_current_indices,
        &mut rc_current_scores,
    );
    let rc_bounded_checksum = fill_bounded_arrays(
        rc_bounded_root,
        queries,
        &mut rc_bounded_indices,
        &mut rc_bounded_scores,
    );
    let arena_current_checksum = run_arena_current(
        arena_root,
        queries_flat,
        &mut arena_current_indices,
        &mut arena_current_scores,
    );
    let arena_bounded_checksum = run_arena_bounded(
        arena_root,
        queries_flat,
        &mut arena_bounded_indices,
        &mut arena_bounded_scores,
    );

    if rc_current_checksum != rc_bounded_checksum
        || rc_current_checksum != arena_current_checksum
        || rc_current_checksum != arena_bounded_checksum
        || rc_current_indices != rc_bounded_indices
        || rc_current_indices != arena_current_indices
        || rc_current_indices != arena_bounded_indices
        || rc_current_scores != rc_bounded_scores
        || rc_current_scores != arena_current_scores
        || rc_current_scores != arena_bounded_scores
    {
        panic!("equivalence mismatch");
    }
}

fn run_rc_lifecycle_current(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let root = build_from_flat(node_mins, node_maxs).expect("non-empty leaves");
    run_rc_current_batch(&root, queries, best_indices, best_scores)
}

fn run_rc_lifecycle_bounded(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let root = build_from_flat(node_mins, node_maxs).expect("non-empty leaves");
    run_rc_bounded_batch(&root, queries, best_indices, best_scores)
}

fn run_arena_lifecycle_current(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let tree = build_arena_from_flat(node_mins, node_maxs).expect("non-empty leaves");
    run_arena_current(&tree, queries, best_indices, best_scores)
}

fn run_arena_lifecycle_bounded(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let tree = build_arena_from_flat(node_mins, node_maxs).expect("non-empty leaves");
    run_arena_bounded(&tree, queries, best_indices, best_scores)
}

fn fill_current_arrays(
    root: &NodeRef,
    queries: &[[i64; PARAMETER_COUNT]],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let mut checksum = 0i64;
    let mut last = None;
    for (query_index, query) in queries.iter().enumerate() {
        let (leaf, distance) = search_current(root, query, last.as_ref());
        best_indices[query_index] = leaf_value(&leaf);
        best_scores[query_index] = distance;
        checksum = checksum.wrapping_mul(31).wrapping_add(best_indices[query_index] as i64);
        last = Some(leaf);
    }
    checksum
}

fn fill_bounded_arrays(
    root: &NodeRef,
    queries: &[[i64; PARAMETER_COUNT]],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let mut checksum = 0i64;
    let mut last = None;
    for (query_index, query) in queries.iter().enumerate() {
        let (leaf, distance) = search_bounded(root, query, last.as_ref());
        best_indices[query_index] = leaf_value(&leaf);
        best_scores[query_index] = distance;
        checksum = checksum.wrapping_mul(31).wrapping_add(best_indices[query_index] as i64);
        last = Some(leaf);
    }
    checksum
}

fn run_arena_current(
    tree: &ArenaTree,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_arena_current_batch(tree, queries, best_indices, best_scores)
        .expect("arena current batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_arena_bounded(
    tree: &ArenaTree,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_arena_bounded_batch(tree, queries, best_indices, best_scores)
        .expect("arena bounded batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_rc_current_batch(
    root: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_current_batch(root, queries, best_indices, best_scores)
        .expect("rc current batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_rc_bounded_batch(
    root: &NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_bounded_batch(root, queries, best_indices, best_scores)
        .expect("rc bounded batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn checksum(values: &[i32]) -> i64 {
    let mut checksum = 0i64;
    for value in values {
        checksum = checksum.wrapping_mul(31).wrapping_add(*value as i64);
    }
    checksum
}

fn time(run: impl FnOnce() -> i64) -> u128 {
    let start = Instant::now();
    let checksum = run();
    SINK.fetch_xor(checksum, Ordering::Relaxed);
    start.elapsed().as_nanos()
}

fn millis(nanos: u128) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn checksum_inputs(leaves: &[NodeRef]) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    for node in leaves {
        for parameter in parameter_space(node) {
            checksum = checksum.wrapping_mul(31).wrapping_add(parameter.min);
            checksum = checksum.wrapping_mul(31).wrapping_add(parameter.max);
        }
        checksum = checksum.wrapping_mul(31).wrapping_add(leaf_value(node) as i64);
    }
    checksum
}

fn checksum_queries(queries: &[[i64; PARAMETER_COUNT]]) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    for query in queries {
        for value in query {
            checksum = checksum.wrapping_mul(31).wrapping_add(*value);
        }
    }
    checksum
}

struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    fn new(seed: u64) -> JavaRandom {
        JavaRandom {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("bound must be positive");
        }
        if (bound & -bound) == bound {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
}
