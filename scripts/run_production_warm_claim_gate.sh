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

if [[ -z "${LOAD_TEST_WORLD_SOURCE:-}" ]]; then
  LOAD_TEST_WORLD_SOURCE="$(resolve_default_world_source "")"
fi

if [[ -z "$LOAD_TEST_WORLD_SOURCE" ]]; then
  echo "LOAD_TEST_WORLD_SOURCE is required for the production-500-warm gate." >&2
  echo "Point it at a saved server run containing world/level.dat, or at a single world directory." >&2
  exit 64
fi

"$ROOT/scripts/check_artifact_source_freshness.sh"

export MC_EULA_AGREE="${MC_EULA_AGREE:-true}"
export LOAD_TEST_SCENARIO="${LOAD_TEST_SCENARIO:-block}"
export LOAD_TEST_GAMEMODE="${LOAD_TEST_GAMEMODE:-creative}"
export BOT_COUNT="${BOT_COUNT:-500}"
export DURATION_SECONDS="${DURATION_SECONDS:-900}"
export VIEW_DISTANCE="${VIEW_DISTANCE:-32}"
export SIMULATION_DISTANCE="${SIMULATION_DISTANCE:-32}"
export BOT_BLOCK_RAMP_SECONDS="${BOT_BLOCK_RAMP_SECONDS:-600}"
export BOT_BLOCK_ACTION_INTERVAL_MS="${BOT_BLOCK_ACTION_INTERVAL_MS:-1000}"
export BOT_ACTION_START_MODE="${BOT_ACTION_START_MODE:-all-ready}"
export BOT_ACTION_READY_SETTLE_MS="${BOT_ACTION_READY_SETTLE_MS:-15000}"
export BOT_ACTION_READY_REQUIRES_BLOCK_ARMED="${BOT_ACTION_READY_REQUIRES_BLOCK_ARMED:-true}"
export BOT_ACTION_READY_MIN_COUNT="${BOT_ACTION_READY_MIN_COUNT:-$BOT_COUNT}"
export BOT_ACTION_READY_MIN_FRACTION="${BOT_ACTION_READY_MIN_FRACTION:-1}"
export BUKKIT_CONNECTION_THROTTLE="${BUKKIT_CONNECTION_THROTTLE:-0}"
export PAPER_DISABLE_METHOD_PROFILER="${PAPER_DISABLE_METHOD_PROFILER:-true}"
export PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE="${PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE:-false}"
export LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA="${LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA:-false}"
export LOAD_TEST_GATE_PROFILE="${LOAD_TEST_GATE_PROFILE:-production-500-warm}"
export LOAD_TEST_LABEL="${LOAD_TEST_LABEL:-production-500-warm-block-${BOT_COUNT}bots-$(date +%Y%m%d-%H%M%S)}"

exec "$ROOT/scripts/run_load_test.sh" "$@"
