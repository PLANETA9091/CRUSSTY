#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-entity-chunk-transient/.classes"
REPORT="${ROOT}/reports/entity-chunk-transient-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"

if [[ ! -f "${LIB_DIR}/libpaper_native_jni.so" ]] || find "${ROOT}/native" -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -newer "${LIB_DIR}/libpaper_native_jni.so" -print -quit | grep -q .; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -d "${OUT}" \
  "${ROOT}/bench/native-entity-chunk-transient/PaperNativeEntityChunkTransient.java" \
  "${ROOT}/bench/native-entity-chunk-transient/NativeEntityChunkTransientBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=${LIB_DIR}/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -cp \"${OUT}\" NativeEntityChunkTransientBench"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" -cp "${OUT}" NativeEntityChunkTransientBench
} | tee "${REPORT}"
