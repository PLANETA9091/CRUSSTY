#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCH_DIR="$ROOT/bench/aquifer-surface-sampling"
CLASSES="$BENCH_DIR/.classes"
REPORT="$ROOT/reports/aquifer-surface-sampling-bench.txt"

mkdir -p "$CLASSES" "$ROOT/reports"
javac -d "$CLASSES" "$BENCH_DIR/AquiferSurfaceSamplingBench.java"

{
  printf 'timestamp_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'host=%s\n' "$(hostname)"
  printf 'kernel=%s\n' "$(uname -srm)"
  printf 'cpu_count=%s\n' "$(nproc)"
  java -version 2>&1 | sed '1!d;s/^/java=/'
  printf 'command=java %s -cp %s AquiferSurfaceSamplingBench\n' "${BENCH_JAVA_OPTS:-}" "$CLASSES"
  java ${BENCH_JAVA_OPTS:-} -cp "$CLASSES" AquiferSurfaceSamplingBench
} | tee "$REPORT"
