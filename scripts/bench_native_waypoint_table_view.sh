#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-waypoint-table/.classes"
REPORT="${ROOT}/reports/waypoint-table-view-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"
LIB="${LIB_DIR}/libpaper_native_jni.so"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

if [[ ! -f "${LIB}" ]] || find "${ROOT}/native" -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -newer "${LIB}" -print -quit | grep -q .; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -cp "${RUNTIME_CP}" -d "${OUT}" \
  "${ROOT}/bench/waypoint-table/WaypointTableViewBench.java" \
  "${ROOT}/bench/native-waypoint-table/PaperNativeWaypointTableView.java" \
  "${ROOT}/bench/native-waypoint-table/NativeWaypointTableViewBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -cp \"${OUT}:${RUNTIME_CP}\" NativeWaypointTableViewBench"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" -cp "${OUT}:${RUNTIME_CP}" NativeWaypointTableViewBench
} | tee "${REPORT}"
