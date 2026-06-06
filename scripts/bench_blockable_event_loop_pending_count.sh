#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/blockable-event-loop-pending-count/BlockableEventLoopPendingCountBench.java"
OUT="$ROOT/bench/blockable-event-loop-pending-count/.classes"
REPORT="$ROOT/reports/blockable-event-loop-pending-count-bench.txt"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$OUT\" BlockableEventLoopPendingCountBench"
  java -cp "$OUT" BlockableEventLoopPendingCountBench
} | tee "$REPORT"
