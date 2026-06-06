#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

resolve_default_world_source() {
  local explicit="$1"
  local candidate=""

  if [[ -n "$explicit" ]]; then
    printf '%s\n' "$explicit"
    return 0
  fi

  candidate="$(
    find "$ROOT/runs" -maxdepth 1 -type d \
      -name 'load-production-500-block-500bots-current-*' \
      | sort \
      | tail -1
  )"
  if [[ -z "$candidate" ]]; then
    candidate="$(
      find "$ROOT/runs" -maxdepth 1 -type d \
        -name 'load-production-500-*' \
        | sort \
        | tail -1
    )"
  fi

  if [[ -n "$candidate" && -f "$candidate/world/level.dat" ]]; then
    printf '%s\n' "$candidate"
    return 0
  fi
  if [[ -n "$candidate" && -f "$candidate/level.dat" ]]; then
    printf '%s\n' "$candidate"
    return 0
  fi

  printf '%s\n' ""
}

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

STAMP="$(date +%Y%m%d-%H%M%S)"
DURATION_SECONDS_SOAK="${PRODUCTION_SOAK_DURATION_SECONDS:-}"
MIN_SAMPLES="${PRODUCTION_SOAK_MIN_LOAD_WINDOW_SAMPLES:-300}"
MIN_BLOCK_PACKETS="${PRODUCTION_SOAK_MIN_BLOCK_PACKETS:-120000}"
METRICS_SAMPLE_INTERVAL_SECONDS="${PRODUCTION_SOAK_METRICS_SAMPLE_INTERVAL_SECONDS:-5}"
DURATION_BUFFER_SECONDS="${PRODUCTION_SOAK_DURATION_BUFFER_SECONDS:-300}"
BOT_BLOCK_RAMP_SECONDS_SOAK="${BOT_BLOCK_RAMP_SECONDS:-600}"
COLD_LABEL="${PRODUCTION_SOAK_COLD_LABEL:-production-500-cold-soak-current-artifact-${STAMP}}"
WARM_LABEL="${PRODUCTION_SOAK_WARM_LABEL:-production-500-warm-soak-current-artifact-${STAMP}}"
WARM_SOURCE="$(resolve_default_world_source "${LOAD_TEST_WORLD_SOURCE:-}")"
CHUNK_WORKER_THREADS="${PRODUCTION_RELEASE_CHUNK_WORKER_THREADS:-${PAPER_CHUNK_WORKER_THREADS:-10}}"
PLAYER_MAX_SEND_RATE="${PRODUCTION_RELEASE_PLAYER_MAX_SEND_RATE:-${PAPER_PLAYER_MAX_SEND_RATE:-60}}"
PLAYER_MAX_GEN_RATE="${PRODUCTION_RELEASE_PLAYER_MAX_GEN_RATE:-${PAPER_PLAYER_MAX_GEN_RATE:-20}}"
COLD_SUMMARY="$ROOT/reports/load-${COLD_LABEL}-summary.txt"
WARM_SUMMARY="$ROOT/reports/load-${WARM_LABEL}-summary.txt"
WARM_SUMMARY_ARG="$WARM_SUMMARY"

if ! [[ "$MIN_SAMPLES" =~ ^[0-9]+$ ]] || (( MIN_SAMPLES < 1 )); then
  echo "PRODUCTION_SOAK_MIN_LOAD_WINDOW_SAMPLES must be a positive integer." >&2
  exit 64
fi
if ! [[ "$MIN_BLOCK_PACKETS" =~ ^[0-9]+$ ]] || (( MIN_BLOCK_PACKETS < 1 )); then
  echo "PRODUCTION_SOAK_MIN_BLOCK_PACKETS must be a positive integer." >&2
  exit 64
fi
if ! [[ "$METRICS_SAMPLE_INTERVAL_SECONDS" =~ ^[0-9]+$ ]] || (( METRICS_SAMPLE_INTERVAL_SECONDS < 1 )); then
  echo "PRODUCTION_SOAK_METRICS_SAMPLE_INTERVAL_SECONDS must be a positive integer." >&2
  exit 64
fi
if ! [[ "$DURATION_BUFFER_SECONDS" =~ ^[0-9]+$ ]] || (( DURATION_BUFFER_SECONDS < 0 )); then
  echo "PRODUCTION_SOAK_DURATION_BUFFER_SECONDS must be a non-negative integer." >&2
  exit 64
fi
if ! [[ "$BOT_BLOCK_RAMP_SECONDS_SOAK" =~ ^[0-9]+$ ]] || (( BOT_BLOCK_RAMP_SECONDS_SOAK < 0 )); then
  echo "BOT_BLOCK_RAMP_SECONDS must be a non-negative integer." >&2
  exit 64
fi

SOAK_MIN_FULL_LOAD_SECONDS=$(( MIN_SAMPLES * METRICS_SAMPLE_INTERVAL_SECONDS ))
SOAK_MIN_DURATION_FLOOR_SECONDS=$(( BOT_BLOCK_RAMP_SECONDS_SOAK + SOAK_MIN_FULL_LOAD_SECONDS + DURATION_BUFFER_SECONDS ))

if [[ -z "$DURATION_SECONDS_SOAK" ]]; then
  DURATION_SECONDS_SOAK="$SOAK_MIN_DURATION_FLOOR_SECONDS"
fi
if ! [[ "$DURATION_SECONDS_SOAK" =~ ^[0-9]+$ ]] || (( DURATION_SECONDS_SOAK < 1 )); then
  echo "PRODUCTION_SOAK_DURATION_SECONDS must be a positive integer." >&2
  exit 64
fi
if (( DURATION_SECONDS_SOAK < SOAK_MIN_DURATION_FLOOR_SECONDS )); then
  echo "PRODUCTION_SOAK_DURATION_SECONDS=$DURATION_SECONDS_SOAK is below the soak floor $SOAK_MIN_DURATION_FLOOR_SECONDS (ramp=$BOT_BLOCK_RAMP_SECONDS_SOAK, samples=$MIN_SAMPLES, sample_interval=$METRICS_SAMPLE_INTERVAL_SECONDS, buffer=$DURATION_BUFFER_SECONDS)." >&2
  exit 64
fi

echo "production_soak_stamp=$STAMP"
echo "soak_duration_seconds=$DURATION_SECONDS_SOAK"
echo "soak_min_load_window_samples=$MIN_SAMPLES"
echo "soak_min_block_packets=$MIN_BLOCK_PACKETS"
echo "soak_metrics_sample_interval_seconds=$METRICS_SAMPLE_INTERVAL_SECONDS"
echo "soak_duration_floor_buffer_seconds=$DURATION_BUFFER_SECONDS"
echo "soak_bot_block_ramp_seconds=$BOT_BLOCK_RAMP_SECONDS_SOAK"
echo "soak_min_full_load_seconds=$SOAK_MIN_FULL_LOAD_SECONDS"
echo "soak_min_duration_floor_seconds=$SOAK_MIN_DURATION_FLOOR_SECONDS"
echo "cold_label=$COLD_LABEL"
echo "warm_label=$WARM_LABEL"
echo "warm_source=$WARM_SOURCE"
echo "chunk_worker_threads=$CHUNK_WORKER_THREADS"
echo "player_max_send_rate=$PLAYER_MAX_SEND_RATE"
echo "player_max_gen_rate=$PLAYER_MAX_GEN_RATE"

python3 "$ROOT/scripts/update_artifact_reports.py"
sha256sum -c "$ROOT/reports/artifact-hashes.txt"
"$ROOT/scripts/check_artifact_source_freshness.sh"

set +e
(
  cd "$ROOT"
  MC_EULA_AGREE=true \
    DURATION_SECONDS="$DURATION_SECONDS_SOAK" \
    BOT_BLOCK_RAMP_SECONDS="$BOT_BLOCK_RAMP_SECONDS_SOAK" \
    PAPER_CHUNK_WORKER_THREADS="$CHUNK_WORKER_THREADS" \
    PAPER_PLAYER_MAX_SEND_RATE="$PLAYER_MAX_SEND_RATE" \
    PAPER_PLAYER_MAX_GEN_RATE="$PLAYER_MAX_GEN_RATE" \
    LOAD_TEST_WORLD_SOURCE= \
    LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA=false \
    LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS=false \
    LOAD_TEST_LABEL="$COLD_LABEL" \
    ./scripts/run_production_claim_gate.sh
)
COLD_EXIT=$?

if (( COLD_EXIT != 0 )); then
  WARM_EXIT=75
  WARM_SUMMARY_ARG=skipped
else
  (
    cd "$ROOT"
    MC_EULA_AGREE=true \
      DURATION_SECONDS="$DURATION_SECONDS_SOAK" \
      BOT_BLOCK_RAMP_SECONDS="$BOT_BLOCK_RAMP_SECONDS_SOAK" \
      PAPER_CHUNK_WORKER_THREADS="$CHUNK_WORKER_THREADS" \
      PAPER_PLAYER_MAX_SEND_RATE="$PLAYER_MAX_SEND_RATE" \
      PAPER_PLAYER_MAX_GEN_RATE="$PLAYER_MAX_GEN_RATE" \
      LOAD_TEST_WORLD_SOURCE="$WARM_SOURCE" \
      LOAD_TEST_LABEL="$WARM_LABEL" \
      ./scripts/run_production_warm_claim_gate.sh
  )
  WARM_EXIT=$?
fi
set -e

python3 "$ROOT/scripts/update_artifact_reports.py"
sha256sum -c "$ROOT/reports/artifact-hashes.txt"
"$ROOT/scripts/check_artifact_source_freshness.sh"

set +e
python3 "$ROOT/scripts/evaluate_production_soak.py" \
  --cold-summary "$COLD_SUMMARY" \
  --warm-summary "$WARM_SUMMARY_ARG" \
  --artifact-hashes "$ROOT/reports/artifact-hashes.txt" \
  --artifacts-json "$ROOT/reports/artifacts.json" \
  --require-current-artifacts \
  --min-load-window-metrics-samples "$MIN_SAMPLES" \
  --min-block-place-packets "$MIN_BLOCK_PACKETS" \
  --min-block-dig-packets "$MIN_BLOCK_PACKETS" \
  --report "$ROOT/reports/production-500-soak-gate.txt"
SOAK_EXIT=$?
set -e

if (( COLD_EXIT != 0 || WARM_EXIT != 0 || SOAK_EXIT != 0 )); then
  echo "production_soak_cold_exit=$COLD_EXIT"
  echo "production_soak_warm_exit=$WARM_EXIT"
  echo "production_soak_gate_exit=$SOAK_EXIT"
  if (( COLD_EXIT != 0 )); then
    echo "warm_skipped_due_to_cold_failure=true"
  fi
  exit 1
fi
