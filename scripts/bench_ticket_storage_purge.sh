#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/ticket-storage-purge/TicketStoragePurgeBench.java"
OUT="$ROOT/bench/ticket-storage-purge/.classes"
REPORT="$ROOT/reports/ticket-storage-purge-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -proc:none -cp "$RUNTIME_CP" -d "$OUT" \
  "$ROOT/upstream/Paper/paper-server/src/minecraft/java/net/minecraft/world/level/TicketStorage.java" \
  "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "source_override=$ROOT/upstream/Paper/paper-server/src/minecraft/java/net/minecraft/world/level/TicketStorage.java"
  echo "command=java \${JAVA_PROPS:-} -cp \"$OUT:\$(cat $RUNTIME_CP_FILE)\" TicketStoragePurgeBench"
  # shellcheck disable=SC2086
  java ${JAVA_PROPS:-} -cp "$OUT:$RUNTIME_CP" TicketStoragePurgeBench
} | tee "$REPORT"
