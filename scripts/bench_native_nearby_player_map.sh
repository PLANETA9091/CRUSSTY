#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-nearby-player-map/.classes"
REPORT="${ROOT}/reports/nearby-player-map-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"
LIB="${LIB_DIR}/libpaper_native_jni.so"
FASTUTIL_JAR="$(find "${HOME}/.gradle/caches/modules-2/files-2.1/it.unimi.dsi/fastutil" -name 'fastutil-*.jar' | sort -V | tail -n 1)"

if [[ -z "${FASTUTIL_JAR}" ]]; then
  echo "fastutil jar not found in Gradle cache" >&2
  exit 1
fi

if [[ ! -f "${LIB}" ]] || find "${ROOT}/native" -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -newer "${LIB}" -print -quit | grep -q .; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "${ROOT}/reports"
javac -proc:none -cp "${FASTUTIL_JAR}" -d "${OUT}" \
  "${ROOT}/bench/native-nearby-player-map/PaperNativeNearbyPlayerMap.java" \
  "${ROOT}/bench/native-nearby-player-map/NativeNearbyPlayerMapBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "fastutil_jar=${FASTUTIL_JAR}"
  echo "native_lib=${LIB}"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -cp \"${OUT}:${FASTUTIL_JAR}\" NativeNearbyPlayerMapBench"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" -cp "${OUT}:${FASTUTIL_JAR}" NativeNearbyPlayerMapBench
} | tee "${REPORT}"
