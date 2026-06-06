#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/placed-feature/PlacedFeatureTraversalBench.java"
OUT="$ROOT/bench/placed-feature/.classes"
REPORT="$ROOT/reports/placed-feature-traversal-bench.txt"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m -cp \"$OUT\" PlacedFeatureTraversalBench"
  java -Xms512m -Xmx512m -cp "$OUT" PlacedFeatureTraversalBench
} | tee "$REPORT"
