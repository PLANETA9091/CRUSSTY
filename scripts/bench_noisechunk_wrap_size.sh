#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/noisechunk-wrap-size/NoiseChunkWrapSizeBench.java"
OUT="$ROOT/bench/noisechunk-wrap-size/.classes"
REPORT="$ROOT/reports/noisechunk-wrap-size-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$RUNTIME_CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java ${JAVA_PROPS:-} --add-opens java.base/java.util=ALL-UNNAMED -cp \"$OUT:\$(cat $RUNTIME_CP_FILE)\" net.minecraft.world.level.levelgen.NoiseChunkWrapSizeBench"
  java ${JAVA_PROPS:-} --add-opens java.base/java.util=ALL-UNNAMED -Xms1G -Xmx3G -cp "$OUT:$RUNTIME_CP" net.minecraft.world.level.levelgen.NoiseChunkWrapSizeBench
} | tee "$REPORT"
