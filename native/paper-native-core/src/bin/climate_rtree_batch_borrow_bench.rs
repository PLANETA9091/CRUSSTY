use std::env;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

use paper_native_core::climate_rtree::{
    build_from_flat, checksum_tree, search_bounded_batch_borrowed, search_bounded_batch_cloned,
    search_current_batch, search_current_batch_borrowed, search_current_batch_direct,
    PARAMETER_COUNT,
};

static SINK: AtomicI64 = AtomicI64::new(0);

fn main() {
    let leaves_count = setting("LEAVES", 1400);
    let queries_count = setting("QUERIES", 120_000);
    let warmup = setting("WARMUP", 4);
    let rounds = setting("ROUNDS", 8);

    let (node_mins, node_maxs) = create_inputs(leaves_count);
    let input_checksum = checksum_input(&node_mins, &node_maxs);
    let root = build_from_flat(&node_mins, &node_maxs).expect("non-empty leaves");
    let tree_checksum = checksum_tree(&root);
    let random_queries = create_random_queries(queries_count);
    let walk_queries = create_walk_queries(queries_count);
    let random_queries_flat = flatten_queries(&random_queries);
    let walk_queries_flat = flatten_queries(&walk_queries);
    let mut clone_indices = vec![0i32; queries_count];
    let mut clone_scores = vec![0i64; queries_count];
    let mut direct_indices = vec![0i32; queries_count];
    let mut direct_scores = vec![0i64; queries_count];
    let mut borrowed_indices = vec![0i32; queries_count];
    let mut borrowed_scores = vec![0i64; queries_count];

    verify_equivalence(
        &root,
        &random_queries_flat,
        &mut clone_indices,
        &mut clone_scores,
        &mut direct_indices,
        &mut direct_scores,
        &mut borrowed_indices,
        &mut borrowed_scores,
    );
    verify_equivalence(
        &root,
        &walk_queries_flat,
        &mut clone_indices,
        &mut clone_scores,
        &mut direct_indices,
        &mut direct_scores,
        &mut borrowed_indices,
        &mut borrowed_scores,
    );

    for _ in 0..warmup {
        SINK.fetch_xor(
            run_cloned_current(&root, &random_queries_flat, &mut clone_indices, &mut clone_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_direct_current(&root, &random_queries_flat, &mut direct_indices, &mut direct_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_borrowed_current(&root, &random_queries_flat, &mut borrowed_indices, &mut borrowed_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_cloned_bounded(&root, &random_queries_flat, &mut clone_indices, &mut clone_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_borrowed_bounded(&root, &random_queries_flat, &mut borrowed_indices, &mut borrowed_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_cloned_current(&root, &walk_queries_flat, &mut clone_indices, &mut clone_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_direct_current(&root, &walk_queries_flat, &mut direct_indices, &mut direct_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_borrowed_current(&root, &walk_queries_flat, &mut borrowed_indices, &mut borrowed_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_cloned_bounded(&root, &walk_queries_flat, &mut clone_indices, &mut clone_scores),
            Ordering::Relaxed,
        );
        SINK.fetch_xor(
            run_borrowed_bounded(&root, &walk_queries_flat, &mut borrowed_indices, &mut borrowed_scores),
            Ordering::Relaxed,
        );
    }

    let mut cloned_current_random_best = u128::MAX;
    let mut direct_current_random_best = u128::MAX;
    let mut borrowed_current_random_best = u128::MAX;
    let mut cloned_bounded_random_best = u128::MAX;
    let mut borrowed_bounded_random_best = u128::MAX;
    let mut cloned_current_walk_best = u128::MAX;
    let mut direct_current_walk_best = u128::MAX;
    let mut borrowed_current_walk_best = u128::MAX;
    let mut cloned_bounded_walk_best = u128::MAX;
    let mut borrowed_bounded_walk_best = u128::MAX;

    for _ in 0..rounds {
        cloned_current_random_best = cloned_current_random_best.min(time(|| {
            run_cloned_current(&root, &random_queries_flat, &mut clone_indices, &mut clone_scores)
        }));
        direct_current_random_best = direct_current_random_best.min(time(|| {
            run_direct_current(&root, &random_queries_flat, &mut direct_indices, &mut direct_scores)
        }));
        borrowed_current_random_best = borrowed_current_random_best.min(time(|| {
            run_borrowed_current(&root, &random_queries_flat, &mut borrowed_indices, &mut borrowed_scores)
        }));
        cloned_bounded_random_best = cloned_bounded_random_best.min(time(|| {
            run_cloned_bounded(&root, &random_queries_flat, &mut clone_indices, &mut clone_scores)
        }));
        borrowed_bounded_random_best = borrowed_bounded_random_best.min(time(|| {
            run_borrowed_bounded(&root, &random_queries_flat, &mut borrowed_indices, &mut borrowed_scores)
        }));

        cloned_current_walk_best = cloned_current_walk_best.min(time(|| {
            run_cloned_current(&root, &walk_queries_flat, &mut clone_indices, &mut clone_scores)
        }));
        direct_current_walk_best = direct_current_walk_best.min(time(|| {
            run_direct_current(&root, &walk_queries_flat, &mut direct_indices, &mut direct_scores)
        }));
        borrowed_current_walk_best = borrowed_current_walk_best.min(time(|| {
            run_borrowed_current(&root, &walk_queries_flat, &mut borrowed_indices, &mut borrowed_scores)
        }));
        cloned_bounded_walk_best = cloned_bounded_walk_best.min(time(|| {
            run_cloned_bounded(&root, &walk_queries_flat, &mut clone_indices, &mut clone_scores)
        }));
        borrowed_bounded_walk_best = borrowed_bounded_walk_best.min(time(|| {
            run_borrowed_bounded(&root, &walk_queries_flat, &mut borrowed_indices, &mut borrowed_scores)
        }));
    }

    let cloned_current_random_checksum =
        run_cloned_current(&root, &random_queries_flat, &mut clone_indices, &mut clone_scores);
    let direct_current_random_checksum =
        run_direct_current(&root, &random_queries_flat, &mut direct_indices, &mut direct_scores);
    let borrowed_current_random_checksum = run_borrowed_current(
        &root,
        &random_queries_flat,
        &mut borrowed_indices,
        &mut borrowed_scores,
    );
    let cloned_bounded_random_checksum =
        run_cloned_bounded(&root, &random_queries_flat, &mut clone_indices, &mut clone_scores);
    let borrowed_bounded_random_checksum = run_borrowed_bounded(
        &root,
        &random_queries_flat,
        &mut borrowed_indices,
        &mut borrowed_scores,
    );
    let cloned_current_walk_checksum =
        run_cloned_current(&root, &walk_queries_flat, &mut clone_indices, &mut clone_scores);
    let direct_current_walk_checksum =
        run_direct_current(&root, &walk_queries_flat, &mut direct_indices, &mut direct_scores);
    let borrowed_current_walk_checksum = run_borrowed_current(
        &root,
        &walk_queries_flat,
        &mut borrowed_indices,
        &mut borrowed_scores,
    );
    let cloned_bounded_walk_checksum =
        run_cloned_bounded(&root, &walk_queries_flat, &mut clone_indices, &mut clone_scores);
    let borrowed_bounded_walk_checksum = run_borrowed_bounded(
        &root,
        &walk_queries_flat,
        &mut borrowed_indices,
        &mut borrowed_scores,
    );

    if cloned_current_random_checksum != borrowed_current_random_checksum
        || cloned_current_random_checksum != direct_current_random_checksum
        || cloned_current_random_checksum != cloned_bounded_random_checksum
        || cloned_current_random_checksum != borrowed_bounded_random_checksum
        || cloned_current_walk_checksum != borrowed_current_walk_checksum
        || cloned_current_walk_checksum != direct_current_walk_checksum
        || cloned_current_walk_checksum != cloned_bounded_walk_checksum
        || cloned_current_walk_checksum != borrowed_bounded_walk_checksum
        || clone_indices != borrowed_indices
        || clone_scores != borrowed_scores
        || clone_indices != direct_indices
        || clone_scores != direct_scores
    {
        panic!("checksum mismatch");
    }

    println!("leaves={}", leaves_count);
    println!("queries={}", queries_count);
    println!("warmup={} rounds={}", warmup, rounds);
    println!("input_checksum={}", input_checksum);
    println!("tree_checksum={}", tree_checksum);
    println!("random_queries_checksum={}", checksum_queries(&random_queries));
    println!("walk_queries_checksum={}", checksum_queries(&walk_queries));
    println!("cloned_current_random_best_ms={:.3}", millis(cloned_current_random_best));
    println!("direct_current_random_best_ms={:.3}", millis(direct_current_random_best));
    println!(
        "direct_current_random_speedup_vs_cloned={:.3}x",
        cloned_current_random_best as f64 / direct_current_random_best as f64
    );
    println!("borrowed_current_random_best_ms={:.3}", millis(borrowed_current_random_best));
    println!(
        "borrowed_current_random_speedup_vs_cloned={:.3}x",
        cloned_current_random_best as f64 / borrowed_current_random_best as f64
    );
    println!("cloned_bounded_random_best_ms={:.3}", millis(cloned_bounded_random_best));
    println!("borrowed_bounded_random_best_ms={:.3}", millis(borrowed_bounded_random_best));
    println!(
        "borrowed_bounded_random_speedup_vs_cloned={:.3}x",
        cloned_bounded_random_best as f64 / borrowed_bounded_random_best as f64
    );
    println!("cloned_current_walk_best_ms={:.3}", millis(cloned_current_walk_best));
    println!("direct_current_walk_best_ms={:.3}", millis(direct_current_walk_best));
    println!(
        "direct_current_walk_speedup_vs_cloned={:.3}x",
        cloned_current_walk_best as f64 / direct_current_walk_best as f64
    );
    println!("borrowed_current_walk_best_ms={:.3}", millis(borrowed_current_walk_best));
    println!(
        "borrowed_current_walk_speedup_vs_cloned={:.3}x",
        cloned_current_walk_best as f64 / borrowed_current_walk_best as f64
    );
    println!("cloned_bounded_walk_best_ms={:.3}", millis(cloned_bounded_walk_best));
    println!("borrowed_bounded_walk_best_ms={:.3}", millis(borrowed_bounded_walk_best));
    println!(
        "borrowed_bounded_walk_speedup_vs_cloned={:.3}x",
        cloned_bounded_walk_best as f64 / borrowed_bounded_walk_best as f64
    );
    println!("borrowed_batch_equivalence=PASS");
    println!("random_checksum={}", borrowed_current_random_checksum);
    println!("walk_checksum={}", borrowed_current_walk_checksum);
    println!("sink={}", SINK.load(Ordering::Relaxed));
}

fn setting(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn create_inputs(leaves_count: usize) -> (Vec<i64>, Vec<i64>) {
    let mut random = JavaRandom::new(0xC11A7E5EED);
    let mut node_mins = Vec::with_capacity(leaves_count * PARAMETER_COUNT);
    let mut node_maxs = Vec::with_capacity(leaves_count * PARAMETER_COUNT);
    for _ in 0..leaves_count {
        for _ in 0..PARAMETER_COUNT {
            let center = random.next_int(80_001) as i64 - 40_000;
            let radius = random.next_int(7_001) as i64;
            node_mins.push(center - radius);
            node_maxs.push(center + radius);
        }
    }
    (node_mins, node_maxs)
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
    root: &paper_native_core::climate_rtree::NodeRef,
    queries_flat: &[i64],
    clone_indices: &mut [i32],
    clone_scores: &mut [i64],
    direct_indices: &mut [i32],
    direct_scores: &mut [i64],
    borrowed_indices: &mut [i32],
    borrowed_scores: &mut [i64],
) {
    let cloned_current = run_cloned_current(root, queries_flat, clone_indices, clone_scores);
    let direct_current = run_direct_current(root, queries_flat, direct_indices, direct_scores);
    let borrowed_current = run_borrowed_current(root, queries_flat, borrowed_indices, borrowed_scores);
    if cloned_current != direct_current
        || cloned_current != borrowed_current
        || clone_indices != borrowed_indices
        || clone_scores != borrowed_scores
        || clone_indices != direct_indices
        || clone_scores != direct_scores
    {
        panic!("current batch equivalence mismatch");
    }

    let cloned_bounded = run_cloned_bounded(root, queries_flat, clone_indices, clone_scores);
    let borrowed_bounded = run_borrowed_bounded(root, queries_flat, borrowed_indices, borrowed_scores);
    if cloned_current != cloned_bounded
        || cloned_current != borrowed_bounded
        || clone_indices != borrowed_indices
        || clone_scores != borrowed_scores
    {
        panic!("bounded batch equivalence mismatch");
    }
}

fn run_cloned_current(
    root: &paper_native_core::climate_rtree::NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_current_batch(root, queries, best_indices, best_scores)
        .expect("cloned current batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_direct_current(
    root: &paper_native_core::climate_rtree::NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_current_batch_direct(root, queries, best_indices, best_scores)
        .expect("direct current batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_borrowed_current(
    root: &paper_native_core::climate_rtree::NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_current_batch_borrowed(root, queries, best_indices, best_scores)
        .expect("borrowed current batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_cloned_bounded(
    root: &paper_native_core::climate_rtree::NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_bounded_batch_cloned(root, queries, best_indices, best_scores)
        .expect("cloned bounded batch");
    assert_eq!(written, best_indices.len());
    checksum(best_indices)
}

fn run_borrowed_bounded(
    root: &paper_native_core::climate_rtree::NodeRef,
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> i64 {
    let written = search_bounded_batch_borrowed(root, queries, best_indices, best_scores)
        .expect("borrowed bounded batch");
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

fn checksum_input(node_mins: &[i64], node_maxs: &[i64]) -> i64 {
    let mut checksum = 0x9E3779B97F4A7C15u64 as i64;
    for (min, max) in node_mins.iter().zip(node_maxs) {
        checksum = checksum.wrapping_mul(31).wrapping_add(*min);
        checksum = checksum.wrapping_mul(31).wrapping_add(*max);
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
