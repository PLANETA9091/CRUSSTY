#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/remapper-hash/RemapperHashThresholdBench.java"
OUT="$ROOT/bench/remapper-hash/.classes"
REPORT="$ROOT/reports/remapper-hash-threshold-bench.txt"
GUAVA="$(find "$HOME/.gradle/caches/modules-2/files-2.1/com.google.guava/guava" -name 'guava-*.jar' | sort | tail -n 1)"

mapfile -t JARS < <(find "$ROOT/plugins/matrix" "$ROOT/plugins/matrix-libraries" -name '*.jar' -type f | sort)

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$GUAVA" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "guava=$GUAVA"
  echo "command=java -cp \"$OUT:$GUAVA\" RemapperHashThresholdBench <${#JARS[@]} jars>"
  java -cp "$OUT:$GUAVA" \
    -Dhash.bench.iterations="${ITERATIONS:-200}" \
    -Dhash.bench.rounds="${ROUNDS:-8}" \
    -Dhash.bench.warmup="${WARMUP:-4}" \
    RemapperHashThresholdBench "${JARS[@]}"
} | tee "$REPORT"
