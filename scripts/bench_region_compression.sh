#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/region-compression/RegionCompressionBench.java"
NATIVE_SRC="$ROOT/bench/region-compression/PaperNativeLz4.java"
OUT="$ROOT/target/bench-classes"
REPORT="$ROOT/reports/region-compression.csv"

"$ROOT/scripts/build_native.sh" >/dev/null

lz4_jar="$(find "$HOME/.gradle/caches/modules-2/files-2.1/org.lz4/lz4-java" -type f -name 'lz4-java-*.jar' | sort | tail -n 1)"
if [[ -z "${lz4_jar:-}" ]]; then
  echo "lz4-java jar not found; run Paper Gradle build first" >&2
  exit 1
fi

mkdir -p "$OUT" "$ROOT/reports"
javac -cp "$lz4_jar" -d "$OUT" "$NATIVE_SRC" "$SRC"
java -Djava.library.path="$ROOT/native/target/release" -cp "$OUT:$lz4_jar" RegionCompressionBench | tee "$REPORT"
