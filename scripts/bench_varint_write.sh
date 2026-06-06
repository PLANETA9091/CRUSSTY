#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/network-varint/VarIntWriteBench.java"
OUT="$ROOT/bench/network-varint/.classes"
REPORT="$ROOT/reports/varint-write-bench.txt"

NETTY_BUFFER="$(find "$HOME/.gradle/caches/modules-2/files-2.1/io.netty/netty-buffer" -name 'netty-buffer-*.jar' | sort | tail -n 1)"
NETTY_COMMON="$(find "$HOME/.gradle/caches/modules-2/files-2.1/io.netty/netty-common" -name 'netty-common-*.jar' | sort | tail -n 1)"
CP="$NETTY_BUFFER:$NETTY_COMMON"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "classpath=$CP"
  echo "command=java -cp \"$OUT:$CP\" VarIntWriteBench"
  java -cp "$OUT:$CP" VarIntWriteBench
} | tee "$REPORT"
