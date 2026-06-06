#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/network-compression/VelocityByteBufCompressionBench.java"
OUT="$ROOT/bench/network-compression/.classes"
REPORT="$ROOT/reports/velocity-bytebuf-compression-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

require_class() {
  local class_name="$1"
  if ! javap -classpath "$RUNTIME_CP" "$class_name" >/dev/null 2>&1; then
    echo "required runtime class '$class_name' not found in $RUNTIME_CP_FILE; run scripts/build_optimized.sh first" >&2
    exit 1
  fi
}

require_class io.netty.buffer.ByteBuf
require_class io.netty.buffer.UnpooledByteBufAllocator
require_class com.google.common.base.Preconditions
require_class com.google.common.collect.ImmutableList
require_class com.velocitypowered.natives.compression.VelocityCompressor
require_class com.velocitypowered.natives.util.Natives

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -proc:none -cp "$RUNTIME_CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java ${JAVA_PROPS:-} -cp \"$OUT:\$(cat $RUNTIME_CP_FILE)\" VelocityByteBufCompressionBench"
  java ${JAVA_PROPS:-} -cp "$OUT:$RUNTIME_CP" VelocityByteBufCompressionBench
  echo "script_status=PASS"
} | tee "$REPORT"
