#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASM_UTILS_JAR="$(find "${HOME}/.gradle/caches/modules-2/files-2.1/io.papermc/asm-utils/0.0.3" -name 'asm-utils-0.0.3.jar' | head -n 1)"
ASM_JAR="$(find "${HOME}/.gradle/caches/modules-2/files-2.1/org.ow2.asm/asm/9.8" -name 'asm-9.8.jar' | head -n 1)"

if [[ -z "${ASM_UTILS_JAR}" || -z "${ASM_JAR}" ]]; then
  echo "required asm-utils/asm jars not found" >&2
  exit 1
fi

OUT="${ROOT}/bench/ownable-rule/.classes"
mkdir -p "${OUT}" "${ROOT}/reports"
javac -cp "${ASM_UTILS_JAR}:${ASM_JAR}" -d "${OUT}" "${ROOT}/bench/ownable-rule/OwnableRuleBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "cpu_count=$(nproc)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"${OUT}:${ASM_UTILS_JAR}:${ASM_JAR}\" OwnableRuleBench"
  java -cp "${OUT}:${ASM_UTILS_JAR}:${ASM_JAR}" OwnableRuleBench
} | tee "${ROOT}/reports/ownable-rule-bench.txt"
