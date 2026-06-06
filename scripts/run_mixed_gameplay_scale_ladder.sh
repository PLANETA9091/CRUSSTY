#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"

MIXED_GAMEPLAY_SCALE_COUNTS="${MIXED_GAMEPLAY_SCALE_COUNTS:-100 250 500}"
MIXED_GAMEPLAY_SCALE_DURATION_SECONDS="${MIXED_GAMEPLAY_SCALE_DURATION_SECONDS:-180}"
MIXED_GAMEPLAY_SCALE_VIEW_DISTANCE="${MIXED_GAMEPLAY_SCALE_VIEW_DISTANCE:-16}"
MIXED_GAMEPLAY_SCALE_SIMULATION_DISTANCE="${MIXED_GAMEPLAY_SCALE_SIMULATION_DISTANCE:-16}"
MIXED_GAMEPLAY_SCALE_MOB_STORM_COUNT="${MIXED_GAMEPLAY_SCALE_MOB_STORM_COUNT:-150}"
MIXED_GAMEPLAY_SCALE_THREAD_SAMPLES="${MIXED_GAMEPLAY_SCALE_THREAD_SAMPLES:-10}"
MIXED_GAMEPLAY_SCALE_THREAD_SAMPLE_INTERVAL_SECONDS="${MIXED_GAMEPLAY_SCALE_THREAD_SAMPLE_INTERVAL_SECONDS:-2}"
MIXED_GAMEPLAY_SCALE_THREAD_SAMPLE_START_AFTER_SECONDS="${MIXED_GAMEPLAY_SCALE_THREAD_SAMPLE_START_AFTER_SECONDS:-20}"
MIXED_GAMEPLAY_SCALE_STOP_ON_FAILURE="${MIXED_GAMEPLAY_SCALE_STOP_ON_FAILURE:-false}"
MIXED_GAMEPLAY_SCALE_LABEL_PREFIX="${MIXED_GAMEPLAY_SCALE_LABEL_PREFIX:-mixed-gameplay-scale}"
MIXED_GAMEPLAY_SCALE_WAIT_FOR_HOST_READY="${MIXED_GAMEPLAY_SCALE_WAIT_FOR_HOST_READY:-true}"
MIXED_GAMEPLAY_SCALE_HOST_READY_TIMEOUT_SECONDS="${MIXED_GAMEPLAY_SCALE_HOST_READY_TIMEOUT_SECONDS:-900}"
MIXED_GAMEPLAY_SCALE_HOST_READY_POLL_SECONDS="${MIXED_GAMEPLAY_SCALE_HOST_READY_POLL_SECONDS:-15}"
MIXED_GAMEPLAY_SCALE_MIN_HOST_IDLE_PERCENT="${MIXED_GAMEPLAY_SCALE_MIN_HOST_IDLE_PERCENT:-${LOAD_TEST_MIN_HOST_IDLE_PERCENT:-40}}"
MIXED_GAMEPLAY_SCALE_MAX_LOAD_PER_CPU="${MIXED_GAMEPLAY_SCALE_MAX_LOAD_PER_CPU:-${LOAD_TEST_MAX_LOAD_PER_CPU:-0.75}}"
MIXED_GAMEPLAY_SCALE_MAX_HOST_STEAL_PERCENT="${MIXED_GAMEPLAY_SCALE_MAX_HOST_STEAL_PERCENT:-${LOAD_TEST_MAX_HOST_STEAL_PERCENT:-10}}"
MIXED_GAMEPLAY_SCALE_MAX_HOST_IOWAIT_PERCENT="${MIXED_GAMEPLAY_SCALE_MAX_HOST_IOWAIT_PERCENT:-${LOAD_TEST_MAX_HOST_IOWAIT_PERCENT:-10}}"

mkdir -p "$ROOT/reports"
LADDER_REPORT="$ROOT/reports/mixed-gameplay-scale-ladder-$STAMP.txt"
CURRENT_REPORT="$ROOT/reports/mixed-gameplay-scale-ladder-current.txt"

read_gate_value() {
  local key="$1"
  local path="$2"
  if [[ -f "$path" ]]; then
    awk -F= -v key="$key" '$1 == key {print $2; exit}' "$path"
  fi
}

read_first_gate_value() {
  local path="$1"
  shift
  local key value
  for key in "$@"; do
    value="$(read_gate_value "$key" "$path")"
    if [[ -n "$value" ]]; then
      echo "$value"
      return
    fi
  done
}

probe_host_ready() {
  python3 - "$MIXED_GAMEPLAY_SCALE_MIN_HOST_IDLE_PERCENT" "$MIXED_GAMEPLAY_SCALE_MAX_LOAD_PER_CPU" "$MIXED_GAMEPLAY_SCALE_MAX_HOST_STEAL_PERCENT" "$MIXED_GAMEPLAY_SCALE_MAX_HOST_IOWAIT_PERCENT" <<'PY'
import os
import sys
import time

min_idle = float(sys.argv[1])
max_load_per_cpu = float(sys.argv[2])
max_steal_percent = float(sys.argv[3])
max_iowait_percent = float(sys.argv[4])
cpu_count = os.cpu_count() or 1

def read_cpu():
    with open("/proc/stat", "r", encoding="utf-8") as handle:
        fields = handle.readline().split()[1:]
    values = [int(value) for value in fields]
    iowait = values[4] if len(values) > 4 else 0
    steal = values[7] if len(values) > 7 else 0
    idle = values[3] + (values[4] if len(values) > 4 else 0)
    total = sum(values)
    return idle, total, iowait, steal

idle0, total0, iowait0, steal0 = read_cpu()
time.sleep(1)
idle1, total1, iowait1, steal1 = read_cpu()
idle_delta = idle1 - idle0
total_delta = max(1, total1 - total0)
idle_percent = idle_delta * 100.0 / total_delta
iowait_percent = (iowait1 - iowait0) * 100.0 / total_delta
steal_percent = (steal1 - steal0) * 100.0 / total_delta
load1, load5, load15 = os.getloadavg()
load_per_cpu = load1 / cpu_count
ok = (
    idle_percent >= min_idle
    and load_per_cpu <= max_load_per_cpu
    and steal_percent <= max_steal_percent
    and iowait_percent <= max_iowait_percent
)
print(
    f"host_ready={str(ok).lower()} "
    f"cpu_count={cpu_count} "
    f"load1={load1:.2f} "
    f"load5={load5:.2f} "
    f"load15={load15:.2f} "
    f"load_per_cpu={load_per_cpu:.3f} "
    f"idle_percent_1s={idle_percent:.2f} "
    f"steal_percent={steal_percent:.2f} "
    f"iowait_percent={iowait_percent:.2f} "
    f"min_idle_percent={min_idle:.2f} "
    f"max_load_per_cpu={max_load_per_cpu:.3f} "
    f"max_steal_percent={max_steal_percent:.2f} "
    f"max_iowait_percent={max_iowait_percent:.2f}"
)
raise SystemExit(0 if ok else 1)
PY
}

wait_for_host_ready() {
  local tier="$1"
  if [[ "$MIXED_GAMEPLAY_SCALE_WAIT_FOR_HOST_READY" != "true" ]]; then
    echo "host_ready_wait=false" >> "$LADDER_REPORT"
    return 0
  fi

  local start now elapsed probe_status probe_output
  start="$(date +%s)"
  while true; do
    set +e
    probe_output="$(probe_host_ready)"
    probe_status=$?
    set -e
    now="$(date +%s)"
    elapsed=$(( now - start ))
    echo "host_ready_probe tier=$tier elapsed_seconds=$elapsed $probe_output" >> "$LADDER_REPORT"
    if (( probe_status == 0 )); then
      echo "host_ready_wait_result tier=$tier result=ready elapsed_seconds=$elapsed" >> "$LADDER_REPORT"
      return 0
    fi
    if (( elapsed >= MIXED_GAMEPLAY_SCALE_HOST_READY_TIMEOUT_SECONDS )); then
      echo "host_ready_wait_result tier=$tier result=timeout elapsed_seconds=$elapsed" >> "$LADDER_REPORT"
      return 0
    fi
    sleep "$MIXED_GAMEPLAY_SCALE_HOST_READY_POLL_SECONDS"
  done
}

{
  echo "mixed_gameplay_scale_ladder=true"
  echo "stamp=$STAMP"
  echo "counts=$MIXED_GAMEPLAY_SCALE_COUNTS"
  echo "duration_seconds=$MIXED_GAMEPLAY_SCALE_DURATION_SECONDS"
  echo "view_distance=$MIXED_GAMEPLAY_SCALE_VIEW_DISTANCE"
  echo "simulation_distance=$MIXED_GAMEPLAY_SCALE_SIMULATION_DISTANCE"
  echo "mob_storm_count=$MIXED_GAMEPLAY_SCALE_MOB_STORM_COUNT"
  echo "thread_samples=$MIXED_GAMEPLAY_SCALE_THREAD_SAMPLES"
  echo "stop_on_failure=$MIXED_GAMEPLAY_SCALE_STOP_ON_FAILURE"
  echo "wait_for_host_ready=$MIXED_GAMEPLAY_SCALE_WAIT_FOR_HOST_READY"
  echo "host_ready_timeout_seconds=$MIXED_GAMEPLAY_SCALE_HOST_READY_TIMEOUT_SECONDS"
  echo "host_ready_poll_seconds=$MIXED_GAMEPLAY_SCALE_HOST_READY_POLL_SECONDS"
  echo "host_ready_min_idle_percent=$MIXED_GAMEPLAY_SCALE_MIN_HOST_IDLE_PERCENT"
  echo "host_ready_max_load_per_cpu=$MIXED_GAMEPLAY_SCALE_MAX_LOAD_PER_CPU"
  echo "host_ready_max_steal_percent=$MIXED_GAMEPLAY_SCALE_MAX_HOST_STEAL_PERCENT"
  echo "host_ready_max_iowait_percent=$MIXED_GAMEPLAY_SCALE_MAX_HOST_IOWAIT_PERCENT"
  echo "runner=scripts/run_stress_mixed_gameplay_gate.sh"
  echo
} > "$LADDER_REPORT"

overall_status=0
for count in $MIXED_GAMEPLAY_SCALE_COUNTS; do
  label="$MIXED_GAMEPLAY_SCALE_LABEL_PREFIX-${count}-${STAMP}"
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

  wait_for_host_ready "$count"

  set +e
  MC_EULA_AGREE=true \
  BOT_COUNT="$count" \
  DURATION_SECONDS="$MIXED_GAMEPLAY_SCALE_DURATION_SECONDS" \
  VIEW_DISTANCE="$MIXED_GAMEPLAY_SCALE_VIEW_DISTANCE" \
  SIMULATION_DISTANCE="$MIXED_GAMEPLAY_SCALE_SIMULATION_DISTANCE" \
  LOAD_TEST_LABEL="$label" \
  LOAD_TEST_MOB_STORM_COUNT="$MIXED_GAMEPLAY_SCALE_MOB_STORM_COUNT" \
  LOAD_TEST_THREAD_SAMPLES="$MIXED_GAMEPLAY_SCALE_THREAD_SAMPLES" \
  LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS="$MIXED_GAMEPLAY_SCALE_THREAD_SAMPLE_INTERVAL_SECONDS" \
  LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS="$MIXED_GAMEPLAY_SCALE_THREAD_SAMPLE_START_AFTER_SECONDS" \
  "$ROOT/scripts/run_stress_mixed_gameplay_gate.sh" > "$run_output" 2>&1
  status=$?
  set -e

  if [[ -f "$per_label_gate" ]]; then
    cp "$per_label_gate" "$gate_copy"
  elif [[ -f "$summary" ]]; then
    python3 "$ROOT/scripts/evaluate_load_gate.py" \
      --profile stress-mixed-gameplay \
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
  failure_count="$(read_gate_value failure_count "$gate_copy")"
  online_max="$(read_first_gate_value "$gate_copy" observed_load_window_online_max observed_online_max)"
  loaded_chunks="$(read_first_gate_value "$gate_copy" observed_load_window_loaded_chunks_max observed_loaded_chunks_max)"
  tps_avg="$(read_first_gate_value "$gate_copy" observed_load_window_tps1_avg observed_tps1_avg)"
  tps_min="$(read_first_gate_value "$gate_copy" observed_load_window_tps1_min observed_tps1_min)"
  mspt_avg="$(read_first_gate_value "$gate_copy" observed_load_window_avg_tick_ms_avg observed_avg_tick_ms_avg)"
  mspt_max="$(read_first_gate_value "$gate_copy" observed_load_window_avg_tick_ms_max observed_avg_tick_ms_max)"
  bot_errors="$(read_gate_value observed_bot_errors_max "$gate_copy")"
  bot_connected="$(read_gate_value observed_bot_connected_max "$gate_copy")"
  bot_ready="$(read_gate_value observed_bot_ready_max "$gate_copy")"
  bot_active="$(read_gate_value observed_bot_active_max "$gate_copy")"
  moved_too_quickly="$(read_gate_value observed_moved_too_quickly_warnings "$gate_copy")"
  watchdogs="$(read_gate_value observed_watchdog_thread_dumps "$gate_copy")"
  sync_loads="$(read_gate_value observed_sync_load_stack_hits "$gate_copy")"
  rss_max="$(read_gate_value observed_process_rss_mib_max "$gate_copy")"

  {
    echo "exit_code=$status"
    echo "gate_pass=${gate_pass:-missing}"
    echo "failure_count=${failure_count:-missing}"
    echo "online_max=${online_max:-missing}"
    echo "loaded_chunks_max=${loaded_chunks:-missing}"
    echo "bot_connected_max=${bot_connected:-missing}"
    echo "bot_ready_max=${bot_ready:-missing}"
    echo "bot_active_max=${bot_active:-missing}"
    echo "bot_errors_max=${bot_errors:-missing}"
    echo "load_window_tps1_avg=${tps_avg:-missing}"
    echo "load_window_tps1_min=${tps_min:-missing}"
    echo "load_window_avg_tick_ms_avg=${mspt_avg:-missing}"
    echo "load_window_avg_tick_ms_max=${mspt_max:-missing}"
    echo "moved_too_quickly_warnings=${moved_too_quickly:-missing}"
    echo "watchdog_thread_dumps=${watchdogs:-missing}"
    echo "sync_load_stack_hits=${sync_loads:-missing}"
    echo "process_rss_mib_max=${rss_max:-missing}"
    echo
  } >> "$LADDER_REPORT"

  if (( status != 0 )); then
    overall_status=1
    if [[ "$MIXED_GAMEPLAY_SCALE_STOP_ON_FAILURE" == "true" ]]; then
      break
    fi
  fi
done

cp "$LADDER_REPORT" "$CURRENT_REPORT"
cat "$LADDER_REPORT"
exit "$overall_status"
