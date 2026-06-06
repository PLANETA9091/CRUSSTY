#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="$ROOT/bench/native-nbt-gzip-buffer-shape"
OUT="$SRC_DIR/.classes"
REPORT="$ROOT/reports/native-nbt-gzip-buffer-shape-bench.txt"
NATIVE_LIB_DIR="$ROOT/native/target/release"

"$ROOT/scripts/build_native.sh" >/dev/null

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" \
  "$SRC_DIR/PaperNativeNbtGzipBufferShape.java" \
  "$SRC_DIR/NativeNbtGzipBufferShapeBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$NATIVE_LIB_DIR/libpaper_native_jni.so"
  echo "command=java ${JAVA_PROPS:-} -Djava.library.path=\"$NATIVE_LIB_DIR\" -cp \"$OUT\" NativeNbtGzipBufferShapeBench"
  java ${JAVA_PROPS:-} -Djava.library.path="$NATIVE_LIB_DIR" -cp "$OUT" NativeNbtGzipBufferShapeBench
  echo "script_status=PASS"
} | tee "$REPORT"
