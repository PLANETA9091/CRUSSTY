#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-remapper-index-cleanup/.classes"
REPORT="${ROOT}/reports/remapper-index-cleanup-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"
WARMUP="${REMAP_INDEX_CLEANUP_BENCH_WARMUP:-2}"
ROUNDS="${REMAP_INDEX_CLEANUP_BENCH_ROUNDS:-5}"
ITERATIONS="${REMAP_INDEX_CLEANUP_BENCH_ITERATIONS:-250000}"
INPUTS="${REMAP_INDEX_CLEANUP_BENCH_INPUTS:-12}"
REMAPPED="${REMAP_INDEX_CLEANUP_BENCH_REMAPPED:-4}"

if [[ ! -f "${LIB_DIR}/libpaper_native_jni.so" ]]; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "${ROOT}/reports"
javac -proc:none -d "${OUT}" \
  "${ROOT}/bench/native-remapper-index-cleanup/PaperNativeRemapperIndexCleanup.java" \
  "${ROOT}/bench/native-remapper-index-cleanup/NativeRemapperIndexCleanupBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=${LIB_DIR}/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -Dwarmup=\"${WARMUP}\" -Drounds=\"${ROUNDS}\" -Diterations=\"${ITERATIONS}\" -Dinputs=\"${INPUTS}\" -Dremapped=\"${REMAPPED}\" -cp \"${OUT}\" NativeRemapperIndexCleanupBench"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" \
    -Dwarmup="${WARMUP}" \
    -Drounds="${ROUNDS}" \
    -Diterations="${ITERATIONS}" \
    -Dinputs="${INPUTS}" \
    -Dremapped="${REMAPPED}" \
    -cp "${OUT}" NativeRemapperIndexCleanupBench
} | tee "${REPORT}"
