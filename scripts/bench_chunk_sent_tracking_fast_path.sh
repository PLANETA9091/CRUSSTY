#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/chunk-sent-tracking/ChunkSentTrackingBench.java"
OUT="$ROOT/bench/chunk-sent-tracking/.classes"
REPORT="$ROOT/reports/chunk-sent-tracking-fast-path-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

mkdir -p "$OUT" "$ROOT/reports"
if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"
javac -cp "$RUNTIME_CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$OUT:$RUNTIME_CP\" ChunkSentTrackingBench"
  java -cp "$OUT:$RUNTIME_CP" ChunkSentTrackingBench
} | tee "$REPORT"
