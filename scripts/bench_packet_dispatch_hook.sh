#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/packet-dispatch-hook/PacketDispatchHookBench.java"
OUT="$ROOT/bench/packet-dispatch-hook/.classes"
REPORT="$ROOT/reports/packet-dispatch-hook-bench.txt"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Diterations=${PACKET_DISPATCH_HOOK_ITERATIONS:-20000000}"
    "-Dwarmup=${PACKET_DISPATCH_HOOK_WARMUP:-4}"
    "-Drounds=${PACKET_DISPATCH_HOOK_ROUNDS:-8}"
  )
fi

mkdir -p "$OUT" "$ROOT/reports"
javac -d "$OUT" "$SRC"
{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m ${JAVA_PROPS_ARRAY[*]} -cp \"$OUT\" PacketDispatchHookBench"
  java -Xms512m -Xmx512m "${JAVA_PROPS_ARRAY[@]}" -cp "$OUT" PacketDispatchHookBench
} | tee "$REPORT"
