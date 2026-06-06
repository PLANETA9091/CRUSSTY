#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/direction-plane-iteration/.classes"
REPORT="$ROOT/reports/direction-plane-iteration-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -f "$RUNTIME_CP_FILE" ]]; then
  echo "missing optimized runtime classpath; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(cat "$RUNTIME_CP_FILE")"
mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$RUNTIME_CP" -d "$OUT" \
  "$ROOT/bench/direction-plane-iteration/DirectionPlaneIterationBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m -cp \"$OUT:$RUNTIME_CP\" DirectionPlaneIterationBench"
  java -Xms512m -Xmx512m -cp "$OUT:$RUNTIME_CP" DirectionPlaneIterationBench
} | tee "$REPORT"
