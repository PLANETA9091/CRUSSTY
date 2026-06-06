#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COUNT="${PRODUCTION_RELEASE_REPEAT_COUNT:-3}"

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

if ! [[ "$COUNT" =~ ^[0-9]+$ ]] || (( COUNT < 1 )); then
  echo "PRODUCTION_RELEASE_REPEAT_COUNT must be a positive integer." >&2
  exit 64
fi

STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="$ROOT/reports/release-repeat-$STAMP"
mkdir -p "$OUT_DIR"

echo "production_release_repeat_stamp=$STAMP"
echo "production_release_repeat_count=$COUNT"
echo "production_release_repeat_out_dir=$OUT_DIR"

passes=0
for run_index in $(seq 1 "$COUNT"); do
  run_stamp="$(date +%Y%m%d-%H%M%S)"
  cold_label="production-500-cold-repeat-${STAMP}-run${run_index}-${run_stamp}"
  warm_label="production-500-warm-repeat-${STAMP}-run${run_index}-${run_stamp}"
  run_dir="$OUT_DIR/run-${run_index}"
  mkdir -p "$run_dir"

  echo "repeat_run_index=$run_index"
  echo "repeat_cold_label=$cold_label"
  echo "repeat_warm_label=$warm_label"

  if ! (
    cd "$ROOT"
    MC_EULA_AGREE=true \
      PAPER_CHUNK_WORKER_THREADS="${PRODUCTION_RELEASE_CHUNK_WORKER_THREADS:-${PAPER_CHUNK_WORKER_THREADS:-10}}" \
      PAPER_PLAYER_MAX_SEND_RATE="${PRODUCTION_RELEASE_PLAYER_MAX_SEND_RATE:-${PAPER_PLAYER_MAX_SEND_RATE:-60}}" \
      PAPER_PLAYER_MAX_GEN_RATE="${PRODUCTION_RELEASE_PLAYER_MAX_GEN_RATE:-${PAPER_PLAYER_MAX_GEN_RATE:-20}}" \
      PRODUCTION_RELEASE_COLD_LABEL="$cold_label" \
      PRODUCTION_RELEASE_WARM_LABEL="$warm_label" \
      ./scripts/run_production_release_gate.sh
  ); then
    echo "repeat_run_index=$run_index status=FAIL"
    cp "$ROOT/reports/production-500-release-gate.txt" "$run_dir/production-500-release-gate.txt" || true
    cp "$ROOT/reports/load-${cold_label}-summary.txt" "$run_dir/cold-summary.txt" || true
    cp "$ROOT/reports/load-${warm_label}-summary.txt" "$run_dir/warm-summary.txt" || true
    exit 1
  fi

  cp "$ROOT/reports/production-500-release-gate.txt" "$run_dir/production-500-release-gate.txt"
  cp "$ROOT/reports/load-${cold_label}-summary.txt" "$run_dir/cold-summary.txt"
  cp "$ROOT/reports/load-${warm_label}-summary.txt" "$run_dir/warm-summary.txt"
  passes=$((passes + 1))
  echo "repeat_run_index=$run_index status=PASS"
done

echo "repeat_passes=$passes"
