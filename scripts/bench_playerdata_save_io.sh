#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="${ROOT}/bench/playerdata-save-io"
SRC="${SRC_DIR}/PlayerDataSaveIoBench.java"
OUT="${SRC_DIR}/.classes"
REPORT="${ROOT}/reports/playerdata-save-io-bench.txt"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"
SERVER_CLASSES="${ROOT}/upstream/Paper/paper-server/build/classes/java/main"
SERVER_RESOURCES="${ROOT}/upstream/Paper/paper-server/build/resources/main"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/prepare_fast_runtime.sh first" >&2
  exit 1
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"
COMPILE_CP="${SERVER_CLASSES}:${SERVER_RESOURCES}:${RUNTIME_CP}"

javac -proc:none -cp "${COMPILE_CP}" -d "${OUT}" "${SRC}"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m -cp \"${OUT}:${COMPILE_CP}\" PlayerDataSaveIoBench"
  java -Xms512m -Xmx512m -cp "${OUT}:${COMPILE_CP}" PlayerDataSaveIoBench
  echo "script_status=PASS"
} | tee "${REPORT}"
