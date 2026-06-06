#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${ROOT}/bench/chunk-dependencies/ChunkDependenciesArrayBench.java"
OUT="${ROOT}/bench/chunk-dependencies/.classes"
REPORT="${ROOT}/reports/chunk-dependencies-array-bench.txt"

GUAVA_JAR="${GUAVA_JAR:-$(find "${HOME}/.gradle/caches/modules-2/files-2.1/com.google.guava/guava" -name 'guava-*.jar' 2>/dev/null | sort -V | tail -1)}"
if [[ -z "${GUAVA_JAR}" ]]; then
  echo "guava jar not found" >&2
  exit 1
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -cp "${GUAVA_JAR}" -d "${OUT}" "${SRC}"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "guava_jar=${GUAVA_JAR}"
  echo "command=java -cp \"${OUT}:${GUAVA_JAR}\" ChunkDependenciesArrayBench"
  java -Xms512m -Xmx512m -cp "${OUT}:${GUAVA_JAR}" ChunkDependenciesArrayBench
} | tee "${REPORT}"
