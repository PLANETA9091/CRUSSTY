#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/cansee/CraftPlayerCanSeeBench.java"
OUT="$ROOT/bench/cansee/.classes"
REPORT="$ROOT/reports/craftplayer-cansee-bench.txt"
FASTUTIL_JAR="$(find "$HOME/.gradle/caches/modules-2/files-2.1/it.unimi.dsi/fastutil" -name 'fastutil-*.jar' | sort | tail -n 1)"

if [[ ! -f "$FASTUTIL_JAR" ]]; then
  echo "fastutil jar not found" >&2
  exit 1
fi

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$FASTUTIL_JAR" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$OUT:$FASTUTIL_JAR\" CraftPlayerCanSeeBench"
  java -cp "$OUT:$FASTUTIL_JAR" CraftPlayerCanSeeBench
} | tee "$REPORT"
