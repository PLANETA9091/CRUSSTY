#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/reference-list/ReferenceListKnownAbsentBench.java"
OUT="$ROOT/bench/reference-list/.classes-known-absent"
REPORT="$ROOT/reports/reference-list-known-absent-bench.txt"
FASTUTIL="/root/.gradle/caches/modules-2/files-2.1/it.unimi.dsi/fastutil/8.5.15/1e885b40c9563ab0d3899b871fd0b30e958705dc/fastutil-8.5.15.jar"

mkdir -p "$OUT" "$(dirname "$REPORT")"

javac -cp "$FASTUTIL" -d "$OUT" \
  "$ROOT/upstream/Paper/paper-server/src/main/java/ca/spottedleaf/moonrise/common/list/ReferenceList.java" \
  "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$OUT:$FASTUTIL\" ReferenceListKnownAbsentBench"
  java -cp "$OUT:$FASTUTIL" ReferenceListKnownAbsentBench
} | tee "$REPORT"
