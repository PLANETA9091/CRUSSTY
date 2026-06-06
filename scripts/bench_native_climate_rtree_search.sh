#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT="$ROOT/reports/native-climate-rtree-search-bench.txt"

mkdir -p "$(dirname "$REPORT")"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "command=LEAVES=${LEAVES:-1400} QUERIES=${QUERIES:-120000} WARMUP=${WARMUP:-4} ROUNDS=${ROUNDS:-8} cargo run --release --manifest-path \"$ROOT/native/Cargo.toml\" --bin climate_rtree_search_bench"
  LEAVES="${LEAVES:-1400}" \
    QUERIES="${QUERIES:-120000}" \
    WARMUP="${WARMUP:-4}" \
    ROUNDS="${ROUNDS:-8}" \
    cargo run --release --manifest-path "$ROOT/native/Cargo.toml" --bin climate_rtree_search_bench
} | tee "$REPORT"
