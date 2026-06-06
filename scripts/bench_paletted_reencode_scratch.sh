#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/paletted/out"
mkdir -p "$OUT" "$ROOT/reports"

javac -d "$OUT" "$ROOT/bench/paletted/PalettedReencodeScratchBench.java"
java -Xms512m -Xmx512m -cp "$OUT" PalettedReencodeScratchBench | tee "$ROOT/reports/paletted-reencode-scratch-bench.txt"
