#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/bench/native-waypoint-chunk-update/.classes"
REPORT="${ROOT}/reports/waypoint-chunk-update-native-bench.txt"
LIB_DIR="${ROOT}/native/target/release"
LIB="${LIB_DIR}/libpaper_native_jni.so"

if [[ ! -f "${LIB}" ]] || find "${ROOT}/native" -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -newer "${LIB}" -print -quit | grep -q .; then
  "${ROOT}/scripts/build_native.sh" >/dev/null
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -d "${OUT}" \
  "${ROOT}/bench/waypoint-chunk-update/WaypointChunkUpdateBench.java" \
  "${ROOT}/bench/native-waypoint-chunk-update/PaperNativeWaypointChunkUpdate.java" \
  "${ROOT}/bench/native-waypoint-chunk-update/NativeWaypointChunkUpdateBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=${LIB}"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"${LIB_DIR}\" -Dwaypoint.chunk.iterations=\"${WAYPOINT_CHUNK_ITERATIONS:-16000000}\" -Dwaypoint.chunk.warmup=\"${WAYPOINT_CHUNK_WARMUP:-4}\" -Dwaypoint.chunk.rounds=\"${WAYPOINT_CHUNK_ROUNDS:-7}\" -cp \"${OUT}\" NativeWaypointChunkUpdateBench"
  java -Xms512m -Xmx512m -Djava.library.path="${LIB_DIR}" \
    -Dwaypoint.chunk.iterations="${WAYPOINT_CHUNK_ITERATIONS:-16000000}" \
    -Dwaypoint.chunk.warmup="${WAYPOINT_CHUNK_WARMUP:-4}" \
    -Dwaypoint.chunk.rounds="${WAYPOINT_CHUNK_ROUNDS:-7}" \
    -cp "${OUT}" NativeWaypointChunkUpdateBench
} | tee "${REPORT}"
