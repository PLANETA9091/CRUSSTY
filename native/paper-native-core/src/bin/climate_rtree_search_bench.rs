use std::env;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

use paper_native_core::climate_rtree::{
    build, checksum_tree, create_leaf, leaf_value, parameter_space, search_bounded,
    search_current, NodeRef, Parameter, PARAMETER_COUNT,
};

static SINK: AtomicI64 = AtomicI64::new(0);

fn main() {
    let leaves_count = setting("LEAVES", 1400);
    let queries_count = setting("QUERIES", 120_000);
    let warmup = setting("WARMUP", 4);
    let rounds = setting("ROUNDS", 8);

    let leaves = create_leaves(leaves_count);
    let input_leaves_checksum = checksum_inputs(&leaves);
    let current_root = build(PARAMETER_COUNT, leaves.clone()).expect("non-empty leaves");
    let bounded_root = build(PARAMETER_COUNT, leaves).expect("non-empty leaves");
    let random_queries = create_random_queries(queries_count);
    let walk_queries = create_walk_queries(queries_count);

    verify_equivalence(&current_root, &bounded_root, &random_queries);
    verify_equivalence(&current_root, &bounded_root, &walk_queries);

    for _ in 0..warmup {
        SINK.fetch_xor(run_current(&current_root, &random_queries), Ordering::Relaxed);
        SINK.fetch_xor(run_bounded(&bounded_root, &random_queries), Ordering::Relaxed);
        SINK.fetch_xor(run_current(&current_root, &walk_queries), Ordering::Relaxed);
        SINK.fetch_xor(run_bounded(&bounded_root, &walk_queries), Ordering::Relaxed);
    }

    let mut current_random_best = u128::MAX;
    let mut bounded_random_best = u128::MAX;
    let mut current_walk_best = u128::MAX;
    let mut bounded_walk_best = u128::MAX;
    for _ in 0..rounds {
        current_random_best =
            current_random_best.min(time(|| run_current(&current_root, &random_queries)));
        bounded_random_best =
            bounded_random_best.min(time(|| run_bounded(&bounded_root, &random_queries)));
        current_walk_best = current_walk_best.min(time(|| run_current(&current_root, &walk_queries)));
        bounded_walk_best = bounded_walk_best.min(time(|| run_bounded(&bounded_root, &walk_queries)));
    }

    let random_checksum = run_current(&current_root, &random_queries);
    let bounded_random_checksum = run_bounded(&bounded_root, &random_queries);
    let walk_checksum = run_current(&current_root, &walk_queries);
    let bounded_walk_checksum = run_bounded(&bounded_root, &walk_queries);
    if random_checksum != bounded_random_checksum || walk_checksum != bounded_walk_checksum {
        panic!("checksum mismatch");
    }

    println!("leaves={}", leaves_count);
    println!("queries={}", queries_count);
    println!("warmup={} rounds={}", warmup, rounds);
    println!("input_leaves_checksum={}", input_leaves_checksum);
    println!("current_tree_checksum={}", checksum_tree(&current_root));
    println!("random_queries_checksum={}", checksum_queries(&random_queries));
    println!("walk_queries_checksum={}", checksum_queries(&walk_queries));
    println!(
        "native_current_random_best_ms={:.3}",
        millis(current_random_best)
    );
    println!(
        "native_bounded_random_best_ms={:.3}",
        millis(bounded_random_best)
    );
    println!(
        "native_bounded_random_speedup={:.3}x",
        current_random_best as f64 / bounded_random_best as f64
    );
    println!("native_current_walk_best_ms={:.3}", millis(current_walk_best));
    println!("native_bounded_walk_best_ms={:.3}", millis(bounded_walk_best));
    println!(
        "native_bounded_walk_speedup={:.3}x",
        current_walk_best as f64 / bounded_walk_best as f64
    );
    println!("random_checksum={}", random_checksum);
    println!("walk_checksum={}", walk_checksum);
    println!("equivalence=PASS");
    println!("sink={}", SINK.load(Ordering::Relaxed));
}

fn setting(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn create_leaves(leaves_count: usize) -> Vec<NodeRef> {
    let mut random = JavaRandom::new(0xC11A7E5EED);
    let mut leaves = Vec::with_capacity(leaves_count);
    for value in 0..leaves_count {
        let mut parameters = [Parameter { min: 0, max: 0 }; PARAMETER_COUNT];
        for parameter in &mut parameters {
            let center = random.next_int(80_001) as i64 - 40_000;
            let radius = random.next_int(7_001) as i64;
            *parameter = Parameter {
                min: center - radius,
                max: center + radius,
            };
        }
        leaves.push(create_leaf(parameters, value as i32));
    }
    leaves
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

fn verify_equivalence(
    current_root: &NodeRef,
    bounded_root: &NodeRef,
    queries: &[[i64; PARAMETER_COUNT]],
) {
    let mut current_last = None;
    let mut bounded_last = None;
    for query in queries {
        let (current, current_distance) =
            search_current(current_root, query, current_last.as_ref());
        let (bounded, bounded_distance) =
            search_bounded(bounded_root, query, bounded_last.as_ref());
        if leaf_value(&current) != leaf_value(&bounded) || current_distance != bounded_distance {
            panic!(
                "search mismatch current={} bounded={}",
                leaf_value(&current),
                leaf_value(&bounded)
            );
        }
        current_last = Some(current);
        bounded_last = Some(bounded);
    }
}

fn run_current(root: &NodeRef, queries: &[[i64; PARAMETER_COUNT]]) -> i64 {
    let mut checksum = 0i64;
    let mut last = None;
    for query in queries {
        let (leaf, _) = search_current(root, query, last.as_ref());
        checksum = checksum
            .wrapping_mul(31)
            .wrapping_add(leaf_value(&leaf) as i64);
        last = Some(leaf);
    }
    checksum
}

fn run_bounded(root: &NodeRef, queries: &[[i64; PARAMETER_COUNT]]) -> i64 {
    let mut checksum = 0i64;
    let mut last = None;
    for query in queries {
        let (leaf, _) = search_bounded(root, query, last.as_ref());
        checksum = checksum
            .wrapping_mul(31)
            .wrapping_add(leaf_value(&leaf) as i64);
        last = Some(leaf);
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
