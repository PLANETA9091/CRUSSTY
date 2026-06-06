#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

mapfile -t samples < <(
  if [[ "$#" -gt 0 ]]; then
    printf '%s\n' "$@"
  else
    find "${ROOT}/runs" -path '*/world/region/*.mca' -type f -printf '%s %T@ %p\n' \
      | sort -nr \
      | head -n "${NBT_REGION_SAMPLE_LIMIT:-16}" \
      | awk '{print $3}'
  fi
)

if [[ "${#samples[@]}" -eq 0 ]]; then
  echo "no region samples found" >&2
  exit 1
fi

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"
OUT="${ROOT}/bench/native-nbt-compound-map-capacity/.classes"
mkdir -p "${OUT}" "${ROOT}/reports"

javac -proc:none -cp "${RUNTIME_CP}" -d "${OUT}" \
  "${ROOT}/bench/native-nbt-compound-map-capacity/PaperNativeNbtCompoundMapCapacity.java" \
  "${ROOT}/bench/native-nbt-compound-map-capacity/NativeNbtCompoundMapCapacityBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "cpu_count=$(nproc)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "region_sample_limit=${NBT_REGION_SAMPLE_LIMIT:-16}"
  echo "chunk_sample_limit=${NBT_CHUNK_SAMPLE_LIMIT:-256}"
  printf 'samples='
  printf '%s;' "${samples[@]}"
  printf '\n'
  java -Xms1g -Xmx1g \
    -Dnative.nbtCapacity.maxChunks="${NBT_CHUNK_SAMPLE_LIMIT:-256}" \
    -Djava.library.path="${ROOT}/native/target/release" \
    -cp "${OUT}:${RUNTIME_CP}" \
    NativeNbtCompoundMapCapacityBench "${samples[@]}"
} | tee "${ROOT}/reports/native-nbt-compound-map-capacity-bench.txt"
