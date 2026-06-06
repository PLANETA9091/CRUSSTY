#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-varint/.classes"
REPORT="$ROOT/reports/native-varint-bench.txt"
LIB_DIR="$ROOT/native/target/release"

"$ROOT/scripts/build_native.sh" >/dev/null

mkdir -p "$OUT" "$ROOT/reports"
javac -d "$OUT" "$ROOT/bench/native-varint/PaperNativeVarInt.java" "$ROOT/bench/native-varint/NativeVarIntBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$LIB_DIR/libpaper_native_jni.so"
  echo "command=java ${JAVA_PROPS:-} -Djava.library.path=\"$LIB_DIR\" -cp \"$OUT\" NativeVarIntBench"
  java ${JAVA_PROPS:-} -Djava.library.path="$LIB_DIR" -cp "$OUT" NativeVarIntBench
} | tee "$REPORT"
