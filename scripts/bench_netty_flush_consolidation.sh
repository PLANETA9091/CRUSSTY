#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/netty-flush-consolidation/NettyFlushConsolidationBench.java"
OUT="$ROOT/bench/netty-flush-consolidation/.classes"
REPORT="$ROOT/reports/netty-flush-consolidation-bench.txt"
RUNTIME_CLASSPATH_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"
RUNTIME_CLASSPATH="$(cat "$RUNTIME_CLASSPATH_FILE")"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$RUNTIME_CLASSPATH" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java \${JAVA_PROPS:-} -cp \"$OUT:$RUNTIME_CLASSPATH\" NettyFlushConsolidationBench"
  # shellcheck disable=SC2086
  java ${JAVA_PROPS:-} -cp "$OUT:$RUNTIME_CLASSPATH" NettyFlushConsolidationBench
} | tee "$REPORT"
