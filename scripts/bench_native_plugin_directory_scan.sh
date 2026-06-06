#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-plugin-directory-scan/.classes"
REPORT="${ROOT}/reports/plugin-directory-scan-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"
PLUGIN_DIR="${1:-${ROOT}/plugins/matrix}"

if [[ ! -f "${LIB_DIR}/libpaper_native_jni.so" ]]; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "${ROOT}/reports"
javac -proc:none -d "${OUT}" \
  "${ROOT}/bench/native-plugin-directory-scan/PaperNativePluginDirectoryScan.java" \
  "${ROOT}/bench/native-plugin-directory-scan/NativePluginDirectoryScanBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=${LIB_DIR}/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -cp \"${OUT}\" NativePluginDirectoryScanBench \"${PLUGIN_DIR}\""
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" -cp "${OUT}" NativePluginDirectoryScanBench "${PLUGIN_DIR}"
} | tee "${REPORT}"
