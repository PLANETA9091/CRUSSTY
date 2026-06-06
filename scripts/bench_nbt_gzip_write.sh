#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

sample="${1:-}"
if [[ -z "${sample}" ]]; then
  sample="$(find "${ROOT}/runs" -path '*/level.dat' -type f -printf '%s %p\n' | sort -nr | awk 'NR == 1 { print $2 }')"
fi

if [[ -z "${sample}" || ! -s "${sample}" ]]; then
  echo "level.dat sample not found" >&2
  exit 1
fi

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"
OUT="${ROOT}/bench/nbt-gzip/.classes"
mkdir -p "${OUT}" "${ROOT}/reports"

javac -cp "${RUNTIME_CP}" -d "${OUT}" "${ROOT}/bench/nbt-gzip/NbtGzipWriteBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "cpu_count=$(nproc)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"${OUT}:\$(cat ${RUNTIME_CP_FILE})\" NbtGzipWriteBench \"${sample}\""
  java -Xms512m -Xmx512m -cp "${OUT}:${RUNTIME_CP}" NbtGzipWriteBench "${sample}"
} | tee "${ROOT}/reports/nbt-gzip-write-bench.txt"
