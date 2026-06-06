#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT="$ROOT/reports/native-climate-rtree-arena-bench.txt"

if [[ "${SKIP_NATIVE_BUILD:-0}" != "1" ]]; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

mkdir -p "$ROOT/reports"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "command=LEAVES=${LEAVES:-1400} QUERIES=${QUERIES:-120000} WARMUP=${WARMUP:-2} ROUNDS=${ROUNDS:-4} cargo run --release --manifest-path \"$ROOT/native/Cargo.toml\" --bin climate_rtree_arena_bench"
  LEAVES=${LEAVES:-1400} QUERIES=${QUERIES:-120000} WARMUP=${WARMUP:-2} ROUNDS=${ROUNDS:-4} \
    cargo run --release --manifest-path "$ROOT/native/Cargo.toml" --bin climate_rtree_arena_bench
} | tee "$REPORT"
