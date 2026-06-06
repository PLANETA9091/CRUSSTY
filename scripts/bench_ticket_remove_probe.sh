#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${ROOT}/bench/ticket-remove-probe/TicketRemoveProbeBench.java"
TICKET_SET_SRC="${ROOT}/upstream/Paper/paper-server/src/minecraft/java/ca/spottedleaf/moonrise/patches/chunk_system/util/stream/TicketSet.java"
OUT="${ROOT}/bench/ticket-remove-probe/.classes"
REPORT="${ROOT}/reports/ticket-remove-probe-bench.txt"
RUNTIME_CP_FILE="${ROOT}/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "${RUNTIME_CP_FILE}" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"${RUNTIME_CP_FILE}")"

mkdir -p "${OUT}" "$(dirname "${REPORT}")"
javac -proc:none -cp "${RUNTIME_CP}" -d "${OUT}" "${TICKET_SET_SRC}" "${SRC}"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "source_override=${TICKET_SET_SRC}"
  echo "command=java -cp \"${OUT}:\$(cat ${RUNTIME_CP_FILE})\" TicketRemoveProbeBench"
  java -cp "${OUT}:${RUNTIME_CP}" TicketRemoveProbeBench
} | tee "${REPORT}"
