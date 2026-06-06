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
COLD_LABEL="${PRODUCTION_RELEASE_COLD_LABEL:-production-500-cold-current-artifact-${STAMP}}"
WARM_LABEL="${PRODUCTION_RELEASE_WARM_LABEL:-production-500-warm-current-artifact-${STAMP}}"
WARM_SOURCE="$(resolve_default_world_source "${LOAD_TEST_WORLD_SOURCE:-}")"
CHUNK_WORKER_THREADS="${PRODUCTION_RELEASE_CHUNK_WORKER_THREADS:-${PAPER_CHUNK_WORKER_THREADS:-10}}"
PLAYER_MAX_SEND_RATE="${PRODUCTION_RELEASE_PLAYER_MAX_SEND_RATE:-${PAPER_PLAYER_MAX_SEND_RATE:-60}}"
PLAYER_MAX_GEN_RATE="${PRODUCTION_RELEASE_PLAYER_MAX_GEN_RATE:-${PAPER_PLAYER_MAX_GEN_RATE:-20}}"
COLD_SUMMARY="$ROOT/reports/load-${COLD_LABEL}-summary.txt"
WARM_SUMMARY="$ROOT/reports/load-${WARM_LABEL}-summary.txt"
RELEASE_REPORT="$ROOT/reports/production-500-release-gate.txt"

echo "production_release_stamp=$STAMP"
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
set -e

if (( COLD_EXIT != 0 )); then
  python3 "$ROOT/scripts/update_artifact_reports.py"
  sha256sum -c "$ROOT/reports/artifact-hashes.txt"
  "$ROOT/scripts/check_artifact_source_freshness.sh"

  COLD_SUMMARY_ARG="$COLD_SUMMARY"
  if [[ ! -f "$COLD_SUMMARY" ]]; then
    COLD_SUMMARY_ARG=skipped
  fi

  set +e
  python3 "$ROOT/scripts/evaluate_production_release.py" \
    --cold-summary "$COLD_SUMMARY_ARG" \
    --warm-summary skipped \
    --artifact-hashes "$ROOT/reports/artifact-hashes.txt" \
    --artifacts-json "$ROOT/reports/artifacts.json" \
    --require-current-artifacts \
    --report "$RELEASE_REPORT"
  RELEASE_EXIT=$?
  set -e

  {
    echo "production_release_cold_exit=$COLD_EXIT"
    echo "production_release_warm_exit=skipped"
    echo "production_release_gate_exit=$RELEASE_EXIT"
    echo "warm_skipped_due_to_cold_failure=true"
  } >> "$RELEASE_REPORT"

  echo "production_release_cold_exit=$COLD_EXIT"
  echo "production_release_warm_exit=skipped"
  echo "production_release_gate_exit=$RELEASE_EXIT"
  exit 1
fi

set +e

(
  cd "$ROOT"
  MC_EULA_AGREE=true \
    PAPER_CHUNK_WORKER_THREADS="$CHUNK_WORKER_THREADS" \
    PAPER_PLAYER_MAX_SEND_RATE="$PLAYER_MAX_SEND_RATE" \
    PAPER_PLAYER_MAX_GEN_RATE="$PLAYER_MAX_GEN_RATE" \
    LOAD_TEST_WORLD_SOURCE="$WARM_SOURCE" \
    LOAD_TEST_LABEL="$WARM_LABEL" \
    ./scripts/run_production_warm_claim_gate.sh
)
WARM_EXIT=$?
set -e

python3 "$ROOT/scripts/update_artifact_reports.py"
sha256sum -c "$ROOT/reports/artifact-hashes.txt"
"$ROOT/scripts/check_artifact_source_freshness.sh"

set +e
python3 "$ROOT/scripts/evaluate_production_release.py" \
  --cold-summary "$COLD_SUMMARY" \
  --warm-summary "$WARM_SUMMARY" \
  --artifact-hashes "$ROOT/reports/artifact-hashes.txt" \
  --artifacts-json "$ROOT/reports/artifacts.json" \
  --require-current-artifacts \
  --report "$RELEASE_REPORT"
RELEASE_EXIT=$?
set -e

if (( COLD_EXIT != 0 || WARM_EXIT != 0 || RELEASE_EXIT != 0 )); then
  echo "production_release_cold_exit=$COLD_EXIT"
  echo "production_release_warm_exit=$WARM_EXIT"
  echo "production_release_gate_exit=$RELEASE_EXIT"
  exit 1
fi
