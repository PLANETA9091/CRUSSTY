#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"

EXTREME_AXIS_BOT_COUNT="${EXTREME_AXIS_BOT_COUNT:-100}"
EXTREME_AXIS_DURATION_SECONDS="${EXTREME_AXIS_DURATION_SECONDS:-180}"
EXTREME_AXIS_VIEW_DISTANCE="${EXTREME_AXIS_VIEW_DISTANCE:-16}"
EXTREME_AXIS_SIMULATION_DISTANCE="${EXTREME_AXIS_SIMULATION_DISTANCE:-16}"
EXTREME_AXIS_MOB_STORM_COUNT="${EXTREME_AXIS_MOB_STORM_COUNT:-150}"
EXTREME_AXIS_THREAD_SAMPLES="${EXTREME_AXIS_THREAD_SAMPLES:-10}"
EXTREME_AXIS_THREAD_SAMPLE_INTERVAL_SECONDS="${EXTREME_AXIS_THREAD_SAMPLE_INTERVAL_SECONDS:-2}"
EXTREME_AXIS_THREAD_SAMPLE_START_AFTER_SECONDS="${EXTREME_AXIS_THREAD_SAMPLE_START_AFTER_SECONDS:-20}"
EXTREME_AXIS_VARIANTS="${EXTREME_AXIS_VARIANTS:-slow-move parked no-mobs}"
EXTREME_AXIS_LABEL_PREFIX="${EXTREME_AXIS_LABEL_PREFIX:-extreme-axis}"
EXTREME_AXIS_REFERENCE_SUMMARY="${EXTREME_AXIS_REFERENCE_SUMMARY:-$ROOT/reports/load-extreme-stress-mixed-100-warm-from-p100-20260517-123222-summary.txt}"
EXTREME_AXIS_WORLD_SOURCE="${EXTREME_AXIS_WORLD_SOURCE:-}"

if [[ -z "$EXTREME_AXIS_WORLD_SOURCE" ]]; then
  EXTREME_AXIS_WORLD_SOURCE="$(
    find "$ROOT/runs" -maxdepth 1 -type d \
      -name 'load-extreme-stress-mixed-100-*' \
      ! -name '*warm*' \
      | sort \
      | tail -1
  )"
fi

if [[ -z "$EXTREME_AXIS_WORLD_SOURCE" || ! -f "$EXTREME_AXIS_WORLD_SOURCE/world/level.dat" ]]; then
  echo "Missing EXTREME_AXIS_WORLD_SOURCE server run with world/level.dat: ${EXTREME_AXIS_WORLD_SOURCE:-none}" >&2
  exit 66
fi

mkdir -p "$ROOT/reports"
MATRIX_REPORT="$ROOT/reports/extreme-axis-matrix-$STAMP.txt"
CURRENT_REPORT="$ROOT/reports/extreme-axis-matrix-current.txt"

read_gate_value() {
  local key="$1"
  local path="$2"
  if [[ -f "$path" ]]; then
    awk -F= -v key="$key" '$1 == key {print $2; exit}' "$path"
  fi
}

write_header() {
  {
    echo "extreme_axis_matrix=true"
    echo "stamp=$STAMP"
    echo "bot_count=$EXTREME_AXIS_BOT_COUNT"
    echo "duration_seconds=$EXTREME_AXIS_DURATION_SECONDS"
    echo "view_distance=$EXTREME_AXIS_VIEW_DISTANCE"
    echo "simulation_distance=$EXTREME_AXIS_SIMULATION_DISTANCE"
    echo "world_source=$EXTREME_AXIS_WORLD_SOURCE"
    echo "reference_summary=$EXTREME_AXIS_REFERENCE_SUMMARY"
    echo "variants=$EXTREME_AXIS_VARIANTS"
    echo
  } > "$MATRIX_REPORT"
}

variant_env() {
  local variant="$1"
  VARIANT_STRESS_CORPUS=true
  VARIANT_STRESS_PLUGINS=true
  VARIANT_STRESS_DATAPACKS=true
  VARIANT_WORLD_SOURCE="$EXTREME_AXIS_WORLD_SOURCE"
  VARIANT_WORLD_PRUNE_DATAPACKS=false
  VARIANT_GATE_PROFILE=stress-mixed
  case "$variant" in
    baseline)
      BOT_SPEED=48
      BOT_MOVE_INTERVAL_MS=100
      BOT_SEND_STATIONARY_POSITIONS=true
      LOAD_TEST_MOB_STORM_COUNT="$EXTREME_AXIS_MOB_STORM_COUNT"
      ;;
    slow-move)
      BOT_SPEED="${EXTREME_AXIS_SLOW_MOVE_SPEED:-12}"
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_SLOW_MOVE_INTERVAL_MS:-500}"
      BOT_SEND_STATIONARY_POSITIONS=true
      LOAD_TEST_MOB_STORM_COUNT="$EXTREME_AXIS_MOB_STORM_COUNT"
      ;;
    parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT="$EXTREME_AXIS_MOB_STORM_COUNT"
      ;;
    parked-position-spam)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=true
      LOAD_TEST_MOB_STORM_COUNT="$EXTREME_AXIS_MOB_STORM_COUNT"
      ;;
    no-mobs)
      BOT_SPEED=48
      BOT_MOVE_INTERVAL_MS=100
      BOT_SEND_STATIONARY_POSITIONS=true
      LOAD_TEST_MOB_STORM_COUNT=0
      ;;
    no-mobs-slow-move)
      BOT_SPEED="${EXTREME_AXIS_SLOW_MOVE_SPEED:-12}"
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_SLOW_MOVE_INTERVAL_MS:-500}"
      BOT_SEND_STATIONARY_POSITIONS=true
      LOAD_TEST_MOB_STORM_COUNT=0
      ;;
    no-mobs-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      ;;
    warm-datapacks-matrix-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      VARIANT_STRESS_CORPUS=false
      VARIANT_STRESS_PLUGINS=false
      VARIANT_STRESS_DATAPACKS=false
      VARIANT_WORLD_SOURCE="$EXTREME_AXIS_WORLD_SOURCE"
      ;;
    warm-plugins-only-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      VARIANT_STRESS_CORPUS=false
      VARIANT_STRESS_PLUGINS=true
      VARIANT_STRESS_DATAPACKS=false
      VARIANT_WORLD_SOURCE="$EXTREME_AXIS_WORLD_SOURCE"
      VARIANT_WORLD_PRUNE_DATAPACKS=true
      ;;
    fresh-plugins-only-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      VARIANT_STRESS_CORPUS=false
      VARIANT_STRESS_PLUGINS=true
      VARIANT_STRESS_DATAPACKS=false
      VARIANT_WORLD_SOURCE=""
      VARIANT_WORLD_PRUNE_DATAPACKS=false
      ;;
    fresh-datapacks-matrix-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      VARIANT_STRESS_CORPUS=false
      VARIANT_STRESS_PLUGINS=false
      VARIANT_STRESS_DATAPACKS=true
      VARIANT_WORLD_SOURCE=""
      VARIANT_WORLD_PRUNE_DATAPACKS=false
      ;;
    fresh-full-corpus-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      VARIANT_STRESS_CORPUS=true
      VARIANT_STRESS_PLUGINS=true
      VARIANT_STRESS_DATAPACKS=true
      VARIANT_WORLD_SOURCE=""
      VARIANT_WORLD_PRUNE_DATAPACKS=false
      ;;
    matrix-only-parked)
      BOT_SPEED=0
      BOT_MOVE_INTERVAL_MS="${EXTREME_AXIS_PARKED_MOVE_INTERVAL_MS:-1000}"
      BOT_SEND_STATIONARY_POSITIONS=false
      LOAD_TEST_MOB_STORM_COUNT=0
      VARIANT_STRESS_CORPUS=false
      VARIANT_STRESS_PLUGINS=false
      VARIANT_STRESS_DATAPACKS=false
      VARIANT_WORLD_SOURCE=""
      ;;
    *)
      echo "Unknown variant: $variant" >&2
      exit 64
      ;;
  esac
}

write_header
overall_status=0

for variant in $EXTREME_AXIS_VARIANTS; do
  variant_env "$variant"
  label="$EXTREME_AXIS_LABEL_PREFIX-${variant}-${EXTREME_AXIS_BOT_COUNT}-${STAMP}"
  run_output="$ROOT/reports/${label}-runner.txt"
  summary="$ROOT/reports/load-${label}-summary.txt"
  gate="$ROOT/reports/${label}-gate.txt"
  sample_dir="$ROOT/logs/load-${label}-jstacks"
  sample_report="$ROOT/reports/${label}-thread-samples.txt"
  sample_json="$ROOT/reports/${label}-thread-samples.json"
  compare_report="$ROOT/reports/${label}-vs-reference.txt"

  {
    echo "[variant]"
    echo "variant=$variant"
    echo "label=$label"
    echo "bot_speed=$BOT_SPEED"
    echo "bot_move_interval_ms=$BOT_MOVE_INTERVAL_MS"
    echo "bot_send_stationary_positions=$BOT_SEND_STATIONARY_POSITIONS"
    echo "mob_storm_count=$LOAD_TEST_MOB_STORM_COUNT"
    echo "stress_corpus=$VARIANT_STRESS_CORPUS"
    echo "stress_plugins=$VARIANT_STRESS_PLUGINS"
    echo "stress_datapacks=$VARIANT_STRESS_DATAPACKS"
    echo "world_source=${VARIANT_WORLD_SOURCE:-fresh}"
    echo "world_prune_datapacks=${VARIANT_WORLD_PRUNE_DATAPACKS:-false}"
    echo "runner_output=$run_output"
    echo "summary=$summary"
    echo "gate=$gate"
    echo "thread_sample_report=$sample_report"
    echo "compare_report=$compare_report"
  } >> "$MATRIX_REPORT"

  set +e
  MC_EULA_AGREE=true \
  BOT_COUNT="$EXTREME_AXIS_BOT_COUNT" \
  DURATION_SECONDS="$EXTREME_AXIS_DURATION_SECONDS" \
  VIEW_DISTANCE="$EXTREME_AXIS_VIEW_DISTANCE" \
  SIMULATION_DISTANCE="$EXTREME_AXIS_SIMULATION_DISTANCE" \
  LOAD_TEST_LABEL="$label" \
  LOAD_TEST_SCENARIO=mixed \
  LOAD_TEST_GAMEMODE=survival \
  LOAD_TEST_STRESS_CORPUS="$VARIANT_STRESS_CORPUS" \
  LOAD_TEST_STRESS_PLUGINS="$VARIANT_STRESS_PLUGINS" \
  LOAD_TEST_STRESS_DATAPACKS="$VARIANT_STRESS_DATAPACKS" \
  LOAD_TEST_WORLD_SOURCE="$VARIANT_WORLD_SOURCE" \
  LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS="${VARIANT_WORLD_PRUNE_DATAPACKS:-false}" \
  LOAD_TEST_MOB_STORM_COUNT="$LOAD_TEST_MOB_STORM_COUNT" \
  BOT_SPEED="$BOT_SPEED" \
  BOT_MOVE_INTERVAL_MS="$BOT_MOVE_INTERVAL_MS" \
  BOT_SEND_STATIONARY_POSITIONS="$BOT_SEND_STATIONARY_POSITIONS" \
  LOAD_TEST_THREAD_SAMPLES="$EXTREME_AXIS_THREAD_SAMPLES" \
  LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS="$EXTREME_AXIS_THREAD_SAMPLE_INTERVAL_SECONDS" \
  LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS="$EXTREME_AXIS_THREAD_SAMPLE_START_AFTER_SECONDS" \
  ./scripts/run_load_test.sh > "$run_output" 2>&1
  status=$?
  set -e

  if [[ -f "$summary" ]]; then
    python3 "$ROOT/scripts/evaluate_load_gate.py" \
      --profile "$VARIANT_GATE_PROFILE" \
      --report "$gate" \
      "$summary" > /dev/null || true
    if [[ -f "$EXTREME_AXIS_REFERENCE_SUMMARY" ]]; then
      python3 "$ROOT/scripts/compare_load_summaries.py" \
        --left "$EXTREME_AXIS_REFERENCE_SUMMARY" \
        --right "$summary" \
        --left-label reference-warm-p100 \
        --right-label "$variant" \
        --key bot_speed_blocks_per_second \
        --key bot_move_interval_ms \
        --key bot_send_stationary_positions \
        --key bot_positions_per_sec_max \
        --key bot_chunks_per_sec_max \
        --key plugin_jars_total \
        --key datapack_zips_total \
        --report "$compare_report" > /dev/null
    fi
  fi

  if [[ -d "$sample_dir" ]]; then
    python3 "$ROOT/scripts/summarize_thread_samples.py" \
      "$sample_dir" \
      --top 30 \
      --report "$sample_report" \
      --json-report "$sample_json" > /dev/null
  fi

  gate_pass="$(read_gate_value gate_pass "$gate")"
  failure_count="$(read_gate_value failure_count "$gate")"
  online_max="$(read_gate_value observed_load_window_online_max "$gate")"
  tps_avg="$(read_gate_value observed_load_window_tps1_avg "$gate")"
  tps_min="$(read_gate_value observed_load_window_tps1_min "$gate")"
  mspt_avg="$(read_gate_value observed_load_window_avg_tick_ms_avg "$gate")"
  mspt_max="$(read_gate_value observed_load_window_avg_tick_ms_max "$gate")"
  watchdog="$(read_gate_value observed_watchdog_thread_dumps "$gate")"
  sync_load="$(read_gate_value observed_sync_load_stack_hits "$gate")"

  {
    echo "exit_code=$status"
    echo "gate_pass=${gate_pass:-missing}"
    echo "failure_count=${failure_count:-missing}"
    echo "online_max=${online_max:-missing}"
    echo "load_window_tps1_avg=${tps_avg:-missing}"
    echo "load_window_tps1_min=${tps_min:-missing}"
    echo "load_window_avg_tick_ms_avg=${mspt_avg:-missing}"
    echo "load_window_avg_tick_ms_max=${mspt_max:-missing}"
    echo "watchdog_thread_dumps=${watchdog:-missing}"
    echo "sync_load_stack_hits=${sync_load:-missing}"
    echo
  } >> "$MATRIX_REPORT"

  if (( status != 0 )); then
    overall_status=1
    if [[ "${EXTREME_AXIS_STOP_ON_RUN_FAILURE:-false}" == "true" ]]; then
      break
    fi
  fi
done

cp "$MATRIX_REPORT" "$CURRENT_REPORT"
cat "$MATRIX_REPORT"
exit "$overall_status"
