#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT/bench/remapper-index/out"
mkdir -p "$OUT_DIR"

javac -d "$OUT_DIR" "$ROOT/bench/remapper-index/RemapperIndexCleanupBench.java"
java -cp "$OUT_DIR" \
  -Dinputs="${REMAP_INDEX_BENCH_INPUTS:-12}" \
  -Dremapped="${REMAP_INDEX_BENCH_REMAPPED:-4}" \
  -Diterations="${REMAP_INDEX_BENCH_ITERATIONS:-5000000}" \
  -Drounds="${REMAP_INDEX_BENCH_ROUNDS:-7}" \
  RemapperIndexCleanupBench
