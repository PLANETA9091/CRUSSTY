#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/cubic-spline-create/CubicSplineCreateBench.java"
OUT="$ROOT/bench/cubic-spline-create/.classes"
REPORT="$ROOT/reports/cubic-spline-create-bench.txt"
CLASSPATH="$OUT:$(cat "$ROOT/artifacts/optimized-runtime/classpath.txt")"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$(cat "$ROOT/artifacts/optimized-runtime/classpath.txt")" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$CLASSPATH\" CubicSplineCreateBench"
  java -cp "$CLASSPATH" CubicSplineCreateBench
} | tee "$REPORT"
