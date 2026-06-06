#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/density-weird-scaled-sampler/DensityWeirdScaledSamplerBench.java"
OUT_DIR="$ROOT/bench/density-weird-scaled-sampler/.classes"
REPORT="$ROOT/reports/density-weird-scaled-sampler-bench.txt"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Dlength=${DENSITY_WEIRD_SCALED_SAMPLER_LENGTH:-1024}"
    "-Diterations=${DENSITY_WEIRD_SCALED_SAMPLER_ITERATIONS:-1000}"
    "-Dwarmup=${DENSITY_WEIRD_SCALED_SAMPLER_WARMUP:-1}"
    "-Drounds=${DENSITY_WEIRD_SCALED_SAMPLER_ROUNDS:-3}"
  )
fi

mkdir -p "$OUT_DIR" "$ROOT/reports"
javac -d "$OUT_DIR" "$SRC"
{
  echo "command=java -Xms512m -Xmx512m ${JAVA_PROPS_ARRAY[*]} -cp \"$OUT_DIR\" DensityWeirdScaledSamplerBench"
  java -Xms512m -Xmx512m "${JAVA_PROPS_ARRAY[@]}" -cp "$OUT_DIR" DensityWeirdScaledSamplerBench
} | tee "$REPORT"
