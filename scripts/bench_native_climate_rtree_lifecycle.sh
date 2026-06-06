#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/climate-rtree-lifecycle/.classes"
REPORT="$ROOT/reports/native-climate-rtree-lifecycle-bench.txt"
LIB_DIR="$ROOT/native/target/release"
JAVA_ARGS=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_ARGS=(${JAVA_PROPS})
fi

if [[ -n "${LEAVES:-}" ]]; then
  JAVA_ARGS+=("-Dleaves=$LEAVES")
fi
if [[ -n "${QUERIES:-}" ]]; then
  JAVA_ARGS+=("-Dqueries=$QUERIES")
fi
if [[ -n "${WARMUP:-}" ]]; then
  JAVA_ARGS+=("-Dwarmup=$WARMUP")
fi
if [[ -n "${ROUNDS:-}" ]]; then
  JAVA_ARGS+=("-Drounds=$ROUNDS")
fi

if [[ "${SKIP_NATIVE_BUILD:-0}" != "1" ]]; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

mkdir -p "$OUT" "$ROOT/reports"
javac -d "$OUT" \
  "$ROOT/bench/climate-rtree-search/PaperNativeClimateRTree.java" \
  "$ROOT/bench/climate-rtree-lifecycle/NativeClimateRTreeLifecycleBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$LIB_DIR/libpaper_native_jni.so"
  echo "command=java ${JAVA_ARGS[*]} -Djava.library.path=\"$LIB_DIR\" -cp \"$OUT\" NativeClimateRTreeLifecycleBench"
  java "${JAVA_ARGS[@]}" -Djava.library.path="$LIB_DIR" -cp "$OUT" NativeClimateRTreeLifecycleBench
} | tee "$REPORT"
