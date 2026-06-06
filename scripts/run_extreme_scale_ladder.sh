#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"

EXTREME_SCALE_COUNTS="${EXTREME_SCALE_COUNTS:-100 250 500}"
EXTREME_SCALE_DURATION_SECONDS="${EXTREME_SCALE_DURATION_SECONDS:-180}"
EXTREME_SCALE_VIEW_DISTANCE="${EXTREME_SCALE_VIEW_DISTANCE:-16}"
EXTREME_SCALE_SIMULATION_DISTANCE="${EXTREME_SCALE_SIMULATION_DISTANCE:-16}"
EXTREME_SCALE_MOB_STORM_COUNT="${EXTREME_SCALE_MOB_STORM_COUNT:-150}"
EXTREME_SCALE_THREAD_SAMPLES="${EXTREME_SCALE_THREAD_SAMPLES:-10}"
EXTREME_SCALE_THREAD_SAMPLE_INTERVAL_SECONDS="${EXTREME_SCALE_THREAD_SAMPLE_INTERVAL_SECONDS:-2}"
EXTREME_SCALE_THREAD_SAMPLE_START_AFTER_SECONDS="${EXTREME_SCALE_THREAD_SAMPLE_START_AFTER_SECONDS:-20}"
EXTREME_SCALE_STOP_ON_FAILURE="${EXTREME_SCALE_STOP_ON_FAILURE:-false}"
EXTREME_SCALE_LABEL_PREFIX="${EXTREME_SCALE_LABEL_PREFIX:-extreme-stress-mixed}"

mkdir -p "$ROOT/reports"
LADDER_REPORT="$ROOT/reports/extreme-scale-ladder-$STAMP.txt"
CURRENT_REPORT="$ROOT/reports/extreme-scale-ladder-current.txt"

read_gate_value() {
  local key="$1"
  local path="$2"
  if [[ -f "$path" ]]; then
    awk -F= -v key="$key" '$1 == key {print $2; exit}' "$path"
  fi
}

{
  echo "extreme_scale_ladder=true"
  echo "stamp=$STAMP"
  echo "counts=$EXTREME_SCALE_COUNTS"
  echo "duration_seconds=$EXTREME_SCALE_DURATION_SECONDS"
  echo "view_distance=$EXTREME_SCALE_VIEW_DISTANCE"
  echo "simulation_distance=$EXTREME_SCALE_SIMULATION_DISTANCE"
  echo "mob_storm_count=$EXTREME_SCALE_MOB_STORM_COUNT"
  echo "thread_samples=$EXTREME_SCALE_THREAD_SAMPLES"
  echo "stop_on_failure=$EXTREME_SCALE_STOP_ON_FAILURE"
  echo
} > "$LADDER_REPORT"

overall_status=0
for count in $EXTREME_SCALE_COUNTS; do
  label="$EXTREME_SCALE_LABEL_PREFIX-${count}-${STAMP}"
  run_output="$ROOT/reports/${label}-runner.txt"
  gate_copy="$ROOT/reports/${label}-gate.txt"
  per_label_gate="$ROOT/reports/load-${label}-gate.txt"
  summary="$ROOT/reports/load-${label}-summary.txt"
  sample_dir="$ROOT/logs/load-${label}-jstacks"
  sample_report="$ROOT/reports/${label}-thread-samples.txt"
  sample_json="$ROOT/reports/${label}-thread-samples.json"

  {
    echo "[tier]"
    echo "bot_count=$count"
    echo "label=$label"
    echo "runner_output=$run_output"
    echo "summary=$summary"
    echo "gate=$gate_copy"
    echo "thread_sample_report=$sample_report"
  } >> "$LADDER_REPORT"

  set +e
  MC_EULA_AGREE=true \
  BOT_COUNT="$count" \
  DURATION_SECONDS="$EXTREME_SCALE_DURATION_SECONDS" \
  VIEW_DISTANCE="$EXTREME_SCALE_VIEW_DISTANCE" \
  SIMULATION_DISTANCE="$EXTREME_SCALE_SIMULATION_DISTANCE" \
  LOAD_TEST_LABEL="$label" \
  LOAD_TEST_MOB_STORM_COUNT="$EXTREME_SCALE_MOB_STORM_COUNT" \
  LOAD_TEST_THREAD_SAMPLES="$EXTREME_SCALE_THREAD_SAMPLES" \
  LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS="$EXTREME_SCALE_THREAD_SAMPLE_INTERVAL_SECONDS" \
  LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS="$EXTREME_SCALE_THREAD_SAMPLE_START_AFTER_SECONDS" \
  ./scripts/run_stress_mixed_load_gate.sh > "$run_output" 2>&1
  status=$?
  set -e

  if [[ -f "$per_label_gate" ]]; then
    cp "$per_label_gate" "$gate_copy"
  elif [[ -f "$ROOT/reports/stress-mixed-load-gate.txt" ]]; then
    cp "$ROOT/reports/stress-mixed-load-gate.txt" "$gate_copy"
  elif [[ -f "$summary" ]]; then
    python3 "$ROOT/scripts/evaluate_load_gate.py" \
      --profile stress-mixed \
      --report "$gate_copy" \
      "$summary" > /dev/null || true
  fi

  if [[ -d "$sample_dir" ]]; then
    python3 "$ROOT/scripts/summarize_thread_samples.py" \
      "$sample_dir" \
      --top 30 \
      --report "$sample_report" \
      --json-report "$sample_json" > /dev/null
  fi

  gate_pass="$(read_gate_value gate_pass "$gate_copy")"
  tps_avg="$(read_gate_value observed_load_window_tps1_avg "$gate_copy")"
  tps_min="$(read_gate_value observed_load_window_tps1_min "$gate_copy")"
  mspt_avg="$(read_gate_value observed_load_window_avg_tick_ms_avg "$gate_copy")"
  mspt_max="$(read_gate_value observed_load_window_avg_tick_ms_max "$gate_copy")"
  online_max="$(read_gate_value observed_load_window_online_max "$gate_copy")"
  rss_max="$(read_gate_value observed_process_rss_mib_max "$gate_copy")"
  failure_count="$(read_gate_value failure_count "$gate_copy")"

  {
    echo "exit_code=$status"
    echo "gate_pass=${gate_pass:-missing}"
    echo "failure_count=${failure_count:-missing}"
    echo "online_max=${online_max:-missing}"
    echo "load_window_tps1_avg=${tps_avg:-missing}"
    echo "load_window_tps1_min=${tps_min:-missing}"
    echo "load_window_avg_tick_ms_avg=${mspt_avg:-missing}"
    echo "load_window_avg_tick_ms_max=${mspt_max:-missing}"
    echo "process_rss_mib_max=${rss_max:-missing}"
    echo
  } >> "$LADDER_REPORT"

  if (( status != 0 )); then
    overall_status=1
    if [[ "$EXTREME_SCALE_STOP_ON_FAILURE" == "true" ]]; then
      break
    fi
  fi
done

cp "$LADDER_REPORT" "$CURRENT_REPORT"
cat "$LADDER_REPORT"
exit "$overall_status"
