#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-density-ap2-fill/.classes"
REPORT="$ROOT/reports/native-density-ap2-fill-bench.txt"
LIB_DIR="$ROOT/native/target/release"

if [[ ! -f "$LIB_DIR/libpaper_native_jni.so" ]]; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

mkdir -p "$OUT" "$ROOT/reports"
javac -d "$OUT" \
  "$ROOT/bench/native-density-ap2-fill/PaperNativeDensityAp2Fill.java" \
  "$ROOT/bench/native-density-ap2-fill/NativeDensityAp2FillBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$LIB_DIR/libpaper_native_jni.so"
  echo "command=java -Xms512m -Xmx512m -Djava.library.path=\"$LIB_DIR\" -cp \"$OUT\" NativeDensityAp2FillBench"
  java -Xms512m -Xmx512m -Djava.library.path="$LIB_DIR" -cp "$OUT" NativeDensityAp2FillBench
} | tee "$REPORT"
