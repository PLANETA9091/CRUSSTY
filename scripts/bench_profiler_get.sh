#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/profiler-get/ProfilerGetBench.java"
OUT="$ROOT/bench/profiler-get/.classes"
REPORT="$ROOT/reports/profiler-get-bench.txt"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$OUT\" ProfilerGetBench"
  java -cp "$OUT" ProfilerGetBench
  echo "command=java -Dpaper.disableMethodProfiler=true -cp \"$OUT\" ProfilerGetBench"
  java -Dpaper.disableMethodProfiler=true -cp "$OUT" ProfilerGetBench
} | tee "$REPORT"
