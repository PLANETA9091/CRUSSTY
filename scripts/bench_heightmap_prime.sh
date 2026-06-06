#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/heightmap-prime/HeightmapPrimeBench.java"
OUT="$ROOT/bench/heightmap-prime/.classes"
REPORT="$ROOT/reports/heightmap-prime-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -f "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -proc:none -cp "$(cat "$RUNTIME_CP_FILE")" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m -cp \"$OUT:\$(cat $RUNTIME_CP_FILE)\" HeightmapPrimeBench"
  java -Xms512m -Xmx512m -cp "$OUT:$(cat "$RUNTIME_CP_FILE")" HeightmapPrimeBench
} | tee "$REPORT"
