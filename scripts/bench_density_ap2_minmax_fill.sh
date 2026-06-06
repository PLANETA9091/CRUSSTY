#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/density-ap2-minmax/DensityAp2MinMaxFillBench.java"
OUT_DIR="$ROOT/bench/density-ap2-minmax/.classes"
REPORT="$ROOT/reports/density-ap2-minmax-fill-bench.txt"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Diterations=${DENSITY_AP2_MINMAX_FILL_ITERATIONS:-2000}"
    "-Dwarmup=${DENSITY_AP2_MINMAX_FILL_WARMUP:-1}"
    "-Drounds=${DENSITY_AP2_MINMAX_FILL_ROUNDS:-3}"
  )
fi

mkdir -p "$OUT_DIR" "$ROOT/reports"
javac -d "$OUT_DIR" "$SRC"
{
  echo "command=java -Xms512m -Xmx512m ${JAVA_PROPS_ARRAY[*]} -cp \"$OUT_DIR\" DensityAp2MinMaxFillBench"
  java -Xms512m -Xmx512m "${JAVA_PROPS_ARRAY[@]}" -cp "$OUT_DIR" DensityAp2MinMaxFillBench
} | tee "$REPORT"
