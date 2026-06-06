#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-hash-path/.classes"
REPORT="$ROOT/reports/native-hash-path-bench.txt"
LIB_DIR="$ROOT/native/target/release"
JAVA_ARGS=()

if [[ "${SKIP_NATIVE_BUILD:-0}" != "1" ]]; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

mapfile -t JARS < <(find "$ROOT/plugins/matrix" "$ROOT/plugins/matrix-libraries" -name '*.jar' -type f | sort)
if [[ "${#JARS[@]}" -eq 0 ]]; then
  echo "no plugin/library jars found" >&2
  exit 1
fi

mkdir -p "$OUT" "$ROOT/reports"

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_ARGS=(${JAVA_PROPS})
fi

javac -d "$OUT" \
  "$ROOT/bench/native-hash-path/PaperNativeHashPath.java" \
  "$ROOT/bench/native-hash-path/NativeHashPathBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$LIB_DIR/libpaper_native_jni.so"
  echo "inputs=${#JARS[@]}"
  echo "command=java ${JAVA_ARGS[*]} -Djava.library.path=\"$LIB_DIR\" -cp \"$OUT\" NativeHashPathBench <${#JARS[@]} jars>"
  java "${JAVA_ARGS[@]}" -Djava.library.path="$LIB_DIR" -cp "$OUT" NativeHashPathBench "${JARS[@]}"
} | tee "$REPORT"
