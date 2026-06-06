#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"

"${ROOT}/scripts/build_native.sh" >/dev/null

OUT="${ROOT}/bench/lz4-stream/.classes"
mkdir -p "${OUT}" "${ROOT}/reports"
javac -cp "${RUNTIME_CP}" -d "${OUT}" \
  "${ROOT}/bench/lz4-stream/NativeLz4BlockOutputStream.java" \
  "${ROOT}/bench/lz4-stream/Lz4StreamBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "cpu_count=$(nproc)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Djava.library.path=\"${ROOT}/native/target/release\" -cp \"${OUT}:\$(cat ${RUNTIME_CP_FILE})\" Lz4StreamBench"
  java -Djava.library.path="${ROOT}/native/target/release" -cp "${OUT}:${RUNTIME_CP}" Lz4StreamBench
} | tee "${ROOT}/reports/lz4-stream-bench.txt"
