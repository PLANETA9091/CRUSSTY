#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/paletted/remap-cache-out"
REPORT="$ROOT/reports/paletted-reencode-remap-cache-bench.txt"
mkdir -p "$OUT" "$ROOT/reports"

javac -d "$OUT" "$ROOT/bench/paletted/PalettedReencodeRemapCacheBench.java"
java -Xms512m -Xmx512m -cp "$OUT" PalettedReencodeRemapCacheBench | tee "$REPORT"
