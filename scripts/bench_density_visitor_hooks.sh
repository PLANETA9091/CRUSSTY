#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/density-mapall/DensityVisitorHookBench.java"
OUT_DIR="$ROOT/bench/density-mapall/.classes"
REPORT="$ROOT/reports/density-visitor-hooks-bench.txt"

mkdir -p "$OUT_DIR" "$ROOT/reports"
javac -d "$OUT_DIR" "$SRC"
java -Xms512m -Xmx512m -cp "$OUT_DIR" DensityVisitorHookBench | tee "$REPORT"
