#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/bench/plugin-load-order/out"
mkdir -p "$OUT_DIR" "$ROOT/reports"

javac -d "$OUT_DIR" "$ROOT/bench/plugin-load-order/LegacyProvidedAliasRemovalBench.java"
java -cp "$OUT_DIR" \
  -Dproviders="${PROVIDERS:-512}" \
  -Daliases="${ALIASES_PER_PROVIDER:-4}" \
  -Diterations="${ITERATIONS:-200}" \
  -Drounds="${ROUNDS:-6}" \
  -Dwarmup="${WARMUP:-3}" \
  LegacyProvidedAliasRemovalBench | tee "$ROOT/reports/legacy-provided-alias-removal-bench.txt"
