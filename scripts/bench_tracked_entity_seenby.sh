#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${ROOT}/bench/tracked-entity-seenby/TrackedEntitySeenByBench.java"
OUT="${ROOT}/bench/tracked-entity-seenby/.classes"
REPORT="${ROOT}/reports/tracked-entity-seenby-bench.txt"

FASTUTIL_JAR="$(find "${HOME}/.gradle/caches/modules-2/files-2.1/it.unimi.dsi/fastutil" -name 'fastutil-*.jar' | sort -V | tail -n 1)"
if [[ -z "${FASTUTIL_JAR}" ]]; then
  echo "fastutil jar not found in Gradle cache" >&2
  exit 1
fi

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -cp "${FASTUTIL_JAR}" -d "${OUT}" "${SRC}"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "fastutil_jar=${FASTUTIL_JAR}"
  echo "command=java -Xms512m -Xmx512m -cp \"${OUT}:${FASTUTIL_JAR}\" TrackedEntitySeenByBench"
  java -Xms512m -Xmx512m -cp "${OUT}:${FASTUTIL_JAR}" TrackedEntitySeenByBench
} | tee "${REPORT}"
