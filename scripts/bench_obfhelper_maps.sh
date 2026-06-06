#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRGUTILS_JAR="$(find "${HOME}/.gradle/caches/modules-2/files-2.1/net.neoforged/srgutils/1.0.9" -name 'srgutils-1.0.9.jar' | head -n 1)"
OUT="${ROOT}/bench/obfhelper-maps/.classes"
MAPPINGS_JAR="${1:-${ROOT}/artifacts/optimized-runtime/bundler/versions/1.21.10/paper-1.21.10.jar}"

if [[ -z "${SRGUTILS_JAR}" ]]; then
  echo "required srgutils jar not found" >&2
  exit 1
fi

mkdir -p "${OUT}" "${ROOT}/reports"
javac -cp "${SRGUTILS_JAR}" -d "${OUT}" "${ROOT}/bench/obfhelper-maps/ObfHelperMapsBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "cpu_count=$(nproc)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "mappings_jar=${MAPPINGS_JAR}"
  echo "command=java -cp \"${OUT}:${SRGUTILS_JAR}\" ObfHelperMapsBench \"${MAPPINGS_JAR}\""
  java -cp "${OUT}:${SRGUTILS_JAR}" ObfHelperMapsBench "${MAPPINGS_JAR}"
} | tee "${ROOT}/reports/obfhelper-maps-bench.txt"
