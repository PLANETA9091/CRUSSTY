#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-paletted-reencode-remap-cache/.classes"
REPORT="${ROOT}/reports/paletted-reencode-remap-cache-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"

if [[ ! -f "${LIB_DIR}/libpaper_native_jni.so" ]] || find "${ROOT}/native" -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -newer "${LIB_DIR}/libpaper_native_jni.so" -print -quit | grep -q .; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -d "${OUT}" \
  "${ROOT}/bench/native-paletted-reencode-remap-cache/PaperNativePalettedReencodeRemapCache.java" \
  "${ROOT}/bench/native-paletted-reencode-remap-cache/NativePalettedReencodeRemapCacheBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=${LIB_DIR}/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -Diterations=\"${PALETTED_REMAP_ITERATIONS:-80000}\" -Dwarmup=\"${PALETTED_REMAP_WARMUP:-4}\" -Drounds=\"${PALETTED_REMAP_ROUNDS:-6}\" -cp \"${OUT}\" NativePalettedReencodeRemapCacheBench"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" \
    -Diterations="${PALETTED_REMAP_ITERATIONS:-80000}" \
    -Dwarmup="${PALETTED_REMAP_WARMUP:-4}" \
    -Drounds="${PALETTED_REMAP_ROUNDS:-6}" \
    -cp "${OUT}" NativePalettedReencodeRemapCacheBench
} | tee "${REPORT}"
