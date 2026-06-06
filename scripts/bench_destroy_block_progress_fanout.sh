#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/destroy-block-progress-fanout/DestroyBlockProgressFanoutBench.java"
OUT="${OUT:-$ROOT/bench/destroy-block-progress-fanout/.classes}"
REPORT="${REPORT:-$ROOT/reports/destroy-block-progress-fanout-bench.txt}"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Dplayers=${PLAYERS:-500}"
    "-Dlevels=${LEVELS:-4}"
    "-Dbroadcasts=${BROADCASTS:-128}"
    "-Diterations=${ITERATIONS:-96}"
    "-Dwarmup=${WARMUP:-4}"
    "-Drounds=${ROUNDS:-8}"
  )
fi

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m ${JAVA_PROPS_ARRAY[*]} -cp \"$OUT\" DestroyBlockProgressFanoutBench"
  java -Xms512m -Xmx512m "${JAVA_PROPS_ARRAY[@]}" -cp "$OUT" DestroyBlockProgressFanoutBench
} | tee "$REPORT"
