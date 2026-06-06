#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/local-mob-cap-counts/LocalMobCapCountsBench.java"
REPORT="$ROOT/reports/local-mob-cap-counts-bench.txt"
OUT="$(mktemp -d "${TMPDIR:-/tmp}/local-mob-cap-counts.XXXXXX")"
trap 'rm -rf "$OUT"' EXIT

FASTUTIL_JAR="$(
  {
    find \
      "$HOME/.gradle/caches/modules-2/files-2.1/it.unimi.dsi/fastutil" \
      "$HOME/.gradle/caches/paperweight" \
      "$ROOT/.gradle/caches/paperweight" \
      -type f -name 'fastutil-*.jar' 2>/dev/null || true
  } | sort -V | tail -n 1
)"

if [[ -z "$FASTUTIL_JAR" ]]; then
  echo "fastutil jar not found in Gradle or Paperweight cache" >&2
  exit 1
fi

mkdir -p "$(dirname "$REPORT")"
javac -cp "$FASTUTIL_JAR" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "fastutil_jar=$FASTUTIL_JAR"
  echo "command=java -cp \"$OUT:$FASTUTIL_JAR\" LocalMobCapCountsBench"
  java -cp "$OUT:$FASTUTIL_JAR" LocalMobCapCountsBench
} | tee "$REPORT"
