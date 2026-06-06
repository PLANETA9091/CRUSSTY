#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/bench/plugin-startup/out"
mkdir -p "$OUT_DIR" "$ROOT/reports"

javac -d "$OUT_DIR" "$ROOT/bench/plugin-startup/PluginNameJoinBench.java"
java -cp "$OUT_DIR" \
  -Dplugins="${PLUGINS:-512}" \
  -Diterations="${ITERATIONS:-5000}" \
  -Drounds="${ROUNDS:-6}" \
  -Dwarmup="${WARMUP:-3}" \
  PluginNameJoinBench | tee "$ROOT/reports/plugin-name-join-bench.txt"
