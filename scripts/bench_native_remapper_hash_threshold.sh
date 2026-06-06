#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-remapper-hash-threshold/.classes"
REPORT="${ROOT}/reports/remapper-hash-threshold-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"
GUAVA="$(find "${HOME}/.gradle/caches/modules-2/files-2.1/com.google.guava/guava" -name 'guava-*.jar' | sort | tail -n 1)"

if [[ -z "${GUAVA}" ]]; then
  echo "guava jar not found under ~/.gradle/caches" >&2
  exit 1
fi

mapfile -t JARS < <(find "${ROOT}/plugins/matrix" "${ROOT}/plugins/matrix-libraries" -name '*.jar' -type f | sort)

if [[ ! -f "${LIB_DIR}/libpaper_native_jni.so" ]]; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -cp "${GUAVA}" -d "${OUT}" \
  "${ROOT}/bench/remapper-hash/RemapperHashThresholdBench.java" \
  "${ROOT}/bench/native-remapper-hash-threshold/PaperNativeRemapperHashThreshold.java" \
  "${ROOT}/bench/native-remapper-hash-threshold/NativeRemapperHashThresholdBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "guava=${GUAVA}"
  echo "native_lib=${LIB_DIR}/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -Dhash.bench.iterations=\"${HASH_BENCH_ITERATIONS:-200}\" -Dhash.bench.rounds=\"${HASH_BENCH_ROUNDS:-8}\" -Dhash.bench.warmup=\"${HASH_BENCH_WARMUP:-4}\" -cp \"${OUT}:${GUAVA}\" NativeRemapperHashThresholdBench <${#JARS[@]} jars>"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" \
    -Dhash.bench.iterations="${HASH_BENCH_ITERATIONS:-200}" \
    -Dhash.bench.rounds="${HASH_BENCH_ROUNDS:-8}" \
    -Dhash.bench.warmup="${HASH_BENCH_WARMUP:-4}" \
    -cp "${OUT}:${GUAVA}" NativeRemapperHashThresholdBench "${JARS[@]}"
} | tee "${REPORT}"
