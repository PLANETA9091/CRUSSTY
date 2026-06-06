#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/plugin-scan/PluginDirectoryScanBench.java"
OUT="$ROOT/bench/plugin-scan/.classes"
REPORT="$ROOT/reports/plugin-scan-bench.txt"
PLUGIN_DIR="${1:-$ROOT/plugins/matrix}"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -cp \"$OUT\" PluginDirectoryScanBench \"$PLUGIN_DIR\""
  java -cp "$OUT" PluginDirectoryScanBench "$PLUGIN_DIR"
} | tee "$REPORT"
