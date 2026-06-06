#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="$ROOT/bench/native-obfhelper-maps"
OUT="$SRC_DIR/.classes"
REPORT="$ROOT/reports/obfhelper-maps-native-bench.txt"

"$ROOT/scripts/build_native.sh" >/dev/null

srgutils_jar="$(find "$HOME/.gradle/caches/modules-2/files-2.1/net.neoforged/srgutils" -type f -name 'srgutils-*.jar' | sort | tail -n 1)"
if [[ -z "${srgutils_jar:-}" ]]; then
  echo "srgutils jar not found; run a Paper build first" >&2
  exit 1
fi

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$srgutils_jar" -d "$OUT" \
  "$SRC_DIR/PaperNativeObfHelperMaps.java" \
  "$SRC_DIR/NativeObfHelperMapsBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Djava.library.path=\"$ROOT/native/target/release\" -cp \"$OUT:$srgutils_jar\" NativeObfHelperMapsBench"
  java -Djava.library.path="$ROOT/native/target/release" -cp "$OUT:$srgutils_jar" NativeObfHelperMapsBench
} | tee "$REPORT"
