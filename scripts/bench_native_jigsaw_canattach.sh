#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-jigsaw-canattach/.classes"
REPORT="$ROOT/reports/native-jigsaw-canattach-bench.txt"
NATIVE_LIB_DIR="$ROOT/native/target/release"

if [[ ! -f "$NATIVE_LIB_DIR/libpaper_native_jni.so" ]]; then
  echo "native library not found; run scripts/build_native.sh first" >&2
  exit 1
fi

RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"
if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$RUNTIME_CP" -d "$OUT" \
  "$ROOT/bench/native-jigsaw-canattach/PaperNativeJigsawCanAttach.java" \
  "$ROOT/bench/native-jigsaw-canattach/NativeJigsawCanAttachBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$NATIVE_LIB_DIR/libpaper_native_jni.so"
  echo "command=java ${JAVA_PROPS:-} -Djava.library.path=\"$NATIVE_LIB_DIR\" -cp \"$OUT:$RUNTIME_CP\" NativeJigsawCanAttachBench"
  java ${JAVA_PROPS:-} -Djava.library.path="$NATIVE_LIB_DIR" -cp "$OUT:$RUNTIME_CP" NativeJigsawCanAttachBench
} | tee "$REPORT"
