#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${ROOT}/bench/waypoint-distance-guard/WaypointDistanceGuardBench.java"
OUT="${ROOT}/bench/waypoint-distance-guard/.classes"
REPORT="${ROOT}/reports/waypoint-distance-guard-bench.txt"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"
JAVA_PROPS="${WAYPOINT_DISTANCE_JAVA_PROPS:--Dwaypoint.distance.iterations=8000000 -Dwaypoint.distance.warmup=3 -Dwaypoint.distance.rounds=5}"

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -cp "${RUNTIME_CP}" -d "${OUT}" "${SRC}"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java ${JAVA_PROPS} -cp \"${OUT}:\$(cat ${RUNTIME_CP_FILE})\" WaypointDistanceGuardBench"
  java ${JAVA_PROPS} -cp "${OUT}:${RUNTIME_CP}" WaypointDistanceGuardBench
} | tee "${REPORT}"
