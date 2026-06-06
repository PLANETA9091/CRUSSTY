#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="$ROOT/bench/native-noisechunk-wrap-capacity"
OUT="$SRC_DIR/.classes"
REPORT="$ROOT/reports/native-noisechunk-wrap-capacity-bench.txt"
NATIVE_LIB_DIR="$ROOT/native/target/release"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

if [[ "${SKIP_NATIVE_BUILD:-0}" != "1" ]]; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$RUNTIME_CP" -d "$OUT" \
  "$SRC_DIR/PaperNativeNoiseChunkWrapCapacity.java" \
  "$SRC_DIR/NativeNoiseChunkWrapCapacityBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$NATIVE_LIB_DIR/libpaper_native_jni.so"
  echo "command=java ${JAVA_PROPS:-} -Xms1G -Xmx3G -Djava.library.path=\"$NATIVE_LIB_DIR\" -cp \"$OUT:\$(cat $RUNTIME_CP_FILE)\" net.minecraft.world.level.levelgen.NativeNoiseChunkWrapCapacityBench"
  java ${JAVA_PROPS:-} -Xms1G -Xmx3G -Djava.library.path="$NATIVE_LIB_DIR" -cp "$OUT:$RUNTIME_CP" net.minecraft.world.level.levelgen.NativeNoiseChunkWrapCapacityBench
  echo "script_status=PASS"
} | tee "$REPORT"
