#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCH_DIR="$ROOT/bench/aquifer-positional-location"
CLASSES="$BENCH_DIR/.classes"
REPORT="$ROOT/reports/aquifer-positional-location-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

mkdir -p "$CLASSES" "$(dirname "$REPORT")"
javac -cp "$RUNTIME_CP" -d "$CLASSES" "$BENCH_DIR/AquiferPositionalLocationBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$CLASSES:$RUNTIME_CP\" AquiferPositionalLocationBench"
  java -cp "$CLASSES:$RUNTIME_CP" AquiferPositionalLocationBench
} | tee "$REPORT"
