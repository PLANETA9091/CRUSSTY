#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-density-ap2-minmax/.classes"
REPORT="$ROOT/reports/native-density-ap2-minmax-fill-bench.txt"
LIB_DIR="$ROOT/native/target/release"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Diterations=${NATIVE_DENSITY_AP2_MINMAX_FILL_ITERATIONS:-2000}"
    "-Dwarmup=${NATIVE_DENSITY_AP2_MINMAX_FILL_WARMUP:-1}"
    "-Drounds=${NATIVE_DENSITY_AP2_MINMAX_FILL_ROUNDS:-3}"
  )
fi

if [[ ! -f "$LIB_DIR/libpaper_native_jni.so" ]]; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

mkdir -p "$OUT" "$ROOT/reports"
javac -d "$OUT" \
  "$ROOT/bench/native-density-ap2-minmax/PaperNativeDensityAp2MinMaxFill.java" \
  "$ROOT/bench/native-density-ap2-minmax/NativeDensityAp2MinMaxFillBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$LIB_DIR/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"$LIB_DIR\" ${JAVA_PROPS_ARRAY[*]} -cp \"$OUT\" NativeDensityAp2MinMaxFillBench"
  java -Xms512m -Xmx512m -Djava.library.path="$LIB_DIR" "${JAVA_PROPS_ARRAY[@]}" -cp "$OUT" NativeDensityAp2MinMaxFillBench
} | tee "$REPORT"
