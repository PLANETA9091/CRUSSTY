#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run_rejected_case() {
  local name="$1"
  local expected="$2"
  shift 2

  local stdout="$TMP/$name.out"
  local stderr="$TMP/$name.err"
  set +e
  (
    cd "$ROOT"
    env \
      MC_EULA_AGREE=true \
      LOAD_TEST_GATE_PROFILE=production-500 \
      LOAD_TEST_SCENARIO=block \
      LOAD_TEST_LABEL="production-action-gate-smoke-$name" \
      BOT_COUNT=500 \
      DURATION_SECONDS=1 \
      VIEW_DISTANCE=32 \
      SIMULATION_DISTANCE=32 \
      "$@" \
      "$ROOT/scripts/run_load_test.sh"
  ) >"$stdout" 2>"$stderr"
  local status=$?
  set -e

  if [[ "$status" != "75" ]]; then
    echo "Expected $name to exit 75, got $status." >&2
    echo "--- stdout ---" >&2
    cat "$stdout" >&2
    echo "--- stderr ---" >&2
    cat "$stderr" >&2
    return 1
  fi
  if ! grep -Fq "$expected" "$stderr"; then
    echo "Expected $name stderr to contain: $expected" >&2
    echo "--- stderr ---" >&2
    cat "$stderr" >&2
    return 1
  fi
}

run_rejected_case \
  timer-mode \
  "Production block claim profiles require BOT_ACTION_START_MODE=all-ready." \
  BOT_ACTION_START_MODE=timer \
  BOT_ACTION_READY_SETTLE_MS=15000 \
  BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true \
  BOT_ACTION_READY_MIN_COUNT=500

run_rejected_case \
  short-settle \
  "Production block claim profiles require BOT_ACTION_READY_SETTLE_MS >= 15000." \
  BOT_ACTION_START_MODE=all-ready \
  BOT_ACTION_READY_SETTLE_MS=14999 \
  BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true \
  BOT_ACTION_READY_MIN_COUNT=500

run_rejected_case \
  no-block-armed \
  "Production block claim profiles require BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true." \
  BOT_ACTION_START_MODE=all-ready \
  BOT_ACTION_READY_SETTLE_MS=15000 \
  BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=false \
  BOT_ACTION_READY_MIN_COUNT=500

run_rejected_case \
  partial-ready-count \
  "Production block claim profiles require BOT_ACTION_READY_MIN_COUNT >= BOT_COUNT." \
  BOT_ACTION_START_MODE=all-ready \
  BOT_ACTION_READY_SETTLE_MS=15000 \
  BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true \
  BOT_ACTION_READY_MIN_COUNT=499

run_rejected_case \
  impossible-all-ready-window \
  "BOT_MAX_DURATION_MS leaves no possible all-ready gate window" \
  BOT_ACTION_START_MODE=all-ready \
  BOT_ACTION_READY_SETTLE_MS=15000 \
  BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true \
  BOT_ACTION_READY_MIN_COUNT=500 \
  BOT_MIN_RUN_AFTER_ACTION_OPEN_MS=15000 \
  BOT_MAX_DURATION_MS=1000

echo "run_load_test_production_action_gate_smoke=PASS"
