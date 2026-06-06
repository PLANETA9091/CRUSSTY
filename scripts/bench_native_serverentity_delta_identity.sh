#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-serverentity-delta-identity/.classes"
REPORT="$ROOT/reports/native-serverentity-delta-identity-bench.txt"
NATIVE_LIB_DIR="$ROOT/native/target/release"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"

if [[ ! -f "$NATIVE_LIB_DIR/libpaper_native_jni.so" ]]; then
  echo "native library not found; run scripts/build_native.sh first" >&2
  exit 1
fi

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$(cat "$RUNTIME_CP_FILE")" -d "$OUT" \
  "$ROOT/bench/native-serverentity-delta-identity/PaperNativeServerEntityDeltaIdentity.java" \
  "$ROOT/bench/native-serverentity-delta-identity/NativeServerEntityDeltaIdentityBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$NATIVE_LIB_DIR/libpaper_native_jni.so"
  echo "command=java ${JAVA_PROPS:-} -Djava.library.path=\"$NATIVE_LIB_DIR\" -cp \"$OUT:$(cat "$RUNTIME_CP_FILE")\" NativeServerEntityDeltaIdentityBench"
  java ${JAVA_PROPS:-} -Djava.library.path="$NATIVE_LIB_DIR" -cp "$OUT:$(cat "$RUNTIME_CP_FILE")" NativeServerEntityDeltaIdentityBench
} | tee "$REPORT"
