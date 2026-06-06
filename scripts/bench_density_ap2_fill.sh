#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/density-ap2-fill/DensityAp2FillBench.java"
OUT_DIR="$ROOT/bench/density-ap2-fill/.classes"
REPORT="$ROOT/reports/density-ap2-fill-bench.txt"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Diterations=${DENSITY_AP2_FILL_ITERATIONS:-10000}"
    "-Dwarmup=${DENSITY_AP2_FILL_WARMUP:-1}"
    "-Drounds=${DENSITY_AP2_FILL_ROUNDS:-3}"
  )
fi

mkdir -p "$OUT_DIR" "$ROOT/reports"
javac -d "$OUT_DIR" "$SRC"
{
  echo "command=java -Xms512m -Xmx512m ${JAVA_PROPS_ARRAY[*]} -cp \"$OUT_DIR\" DensityAp2FillBench"
  java -Xms512m -Xmx512m "${JAVA_PROPS_ARRAY[@]}" -cp "$OUT_DIR" DensityAp2FillBench
} | tee "$REPORT"
