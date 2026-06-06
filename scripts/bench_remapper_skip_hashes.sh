#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${ROOT}/bench/remapper-skip-hashes/.classes"
mkdir -p "${OUT}" "${ROOT}/reports"

javac -d "${OUT}" "${ROOT}/bench/remapper-skip-hashes/SkipHashesBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "cpu_count=$(nproc)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"${OUT}\" SkipHashesBench"
  java -cp "${OUT}" SkipHashesBench
} | tee "${ROOT}/reports/remapper-skip-hashes-bench.txt"
