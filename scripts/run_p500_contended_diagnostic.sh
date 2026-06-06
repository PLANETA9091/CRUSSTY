#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

STAMP="${P500_CONTENDED_DIAGNOSTIC_STAMP:-$(date +%Y%m%d-%H%M%S)}"
LABEL="${P500_CONTENDED_DIAGNOSTIC_LABEL:-p500-contended-diagnostic-current-artifact-${STAMP}}"
RUN_REPORT="${P500_CONTENDED_DIAGNOSTIC_REPORT:-$ROOT/reports/p500-contended-diagnostic-${STAMP}.txt}"
GATE_PROFILE="${P500_CONTENDED_DIAGNOSTIC_GATE_PROFILE:-strict-block}"
REFRESH_ARTIFACTS="${P500_CONTENDED_DIAGNOSTIC_REFRESH_ARTIFACTS:-true}"
DRY_RUN="${P500_CONTENDED_DIAGNOSTIC_DRY_RUN:-false}"
SERVER_READY_TIMEOUT_SECONDS="${P500_CONTENDED_DIAGNOSTIC_SERVER_READY_TIMEOUT_SECONDS:-900}"

BOT_COUNT="${BOT_COUNT:-500}"
DURATION_SECONDS="${DURATION_SECONDS:-900}"
VIEW_DISTANCE="${VIEW_DISTANCE:-32}"
SIMULATION_DISTANCE="${SIMULATION_DISTANCE:-32}"
BOT_BLOCK_RAMP_SECONDS="${BOT_BLOCK_RAMP_SECONDS:-600}"
BOT_BLOCK_ACTION_INTERVAL_MS="${BOT_BLOCK_ACTION_INTERVAL_MS:-1000}"
BOT_ACTION_START_MODE="${BOT_ACTION_START_MODE:-all-ready}"
BOT_ACTION_READY_SETTLE_MS="${BOT_ACTION_READY_SETTLE_MS:-15000}"
BOT_ACTION_READY_REQUIRES_BLOCK_ARMED="${BOT_ACTION_READY_REQUIRES_BLOCK_ARMED:-true}"
BOT_ACTION_READY_MIN_COUNT="${BOT_ACTION_READY_MIN_COUNT:-$BOT_COUNT}"
BOT_ACTION_READY_MIN_FRACTION="${BOT_ACTION_READY_MIN_FRACTION:-1}"
BOT_SWARM_NODE_MAX_OLD_SPACE_MB="${BOT_SWARM_NODE_MAX_OLD_SPACE_MB:-512}"
BOT_SWARM_NODE_MAX_SEMI_SPACE_MB="${BOT_SWARM_NODE_MAX_SEMI_SPACE_MB:-16}"
BUKKIT_CONNECTION_THROTTLE="${BUKKIT_CONNECTION_THROTTLE:-0}"
PAPER_DISABLE_METHOD_PROFILER="${PAPER_DISABLE_METHOD_PROFILER:-true}"
PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE="${PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE:-false}"
PAPER_CHUNK_WORKER_THREADS="${PAPER_CHUNK_WORKER_THREADS:-10}"
PAPER_PLAYER_MAX_SEND_RATE="${PAPER_PLAYER_MAX_SEND_RATE:-60}"
PAPER_PLAYER_MAX_GEN_RATE="${PAPER_PLAYER_MAX_GEN_RATE:-20}"
PAPER_RECEIVE_BACKPRESSURE_ENABLED="${PAPER_RECEIVE_BACKPRESSURE_ENABLED:-true}"
PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS="${PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS:-4096}"
PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS="${PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS:-2048}"
THREAD_SAMPLES="${P500_CONTENDED_DIAGNOSTIC_THREAD_SAMPLES:-${LOAD_TEST_THREAD_SAMPLES:-120}}"
THREAD_SAMPLE_INTERVAL_SECONDS="${P500_CONTENDED_DIAGNOSTIC_THREAD_SAMPLE_INTERVAL_SECONDS:-${LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS:-5}}"
THREAD_SAMPLE_START_AFTER_SECONDS="${P500_CONTENDED_DIAGNOSTIC_THREAD_SAMPLE_START_AFTER_SECONDS:-${LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS:-120}}"

case "$GATE_PROFILE" in
  production-*)
    echo "P500 contended diagnostics must not use production claim profiles." >&2
    echo "Use scripts/run_production_readiness_gate.sh for a strict production claim attempt." >&2
    exit 64
    ;;
esac
case "$REFRESH_ARTIFACTS" in true|false) ;; *) echo "P500_CONTENDED_DIAGNOSTIC_REFRESH_ARTIFACTS must be true or false." >&2; exit 64 ;; esac
case "$DRY_RUN" in true|false) ;; *) echo "P500_CONTENDED_DIAGNOSTIC_DRY_RUN must be true or false." >&2; exit 64 ;; esac
if [[ ! "$SERVER_READY_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( SERVER_READY_TIMEOUT_SECONDS < 1 )); then
  echo "P500_CONTENDED_DIAGNOSTIC_SERVER_READY_TIMEOUT_SECONDS must be a positive integer." >&2
  exit 64
fi

for numeric_var in BOT_COUNT DURATION_SECONDS VIEW_DISTANCE SIMULATION_DISTANCE BOT_BLOCK_RAMP_SECONDS BOT_BLOCK_ACTION_INTERVAL_MS BOT_ACTION_READY_SETTLE_MS BOT_ACTION_READY_MIN_COUNT BOT_SWARM_NODE_MAX_OLD_SPACE_MB BOT_SWARM_NODE_MAX_SEMI_SPACE_MB THREAD_SAMPLES THREAD_SAMPLE_INTERVAL_SECONDS THREAD_SAMPLE_START_AFTER_SECONDS; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+$ ]]; then
    echo "$numeric_var must be a non-negative integer." >&2
    exit 64
  fi
done
if (( BOT_COUNT < 1 || DURATION_SECONDS < 1 || VIEW_DISTANCE < 1 || SIMULATION_DISTANCE < 1 )); then
  echo "BOT_COUNT, DURATION_SECONDS, VIEW_DISTANCE, and SIMULATION_DISTANCE must be positive." >&2
  exit 64
fi

SUMMARY="$ROOT/reports/load-${LABEL}-summary.txt"
GATE="$ROOT/reports/load-${LABEL}-gate.txt"
PREFLIGHT="$ROOT/reports/load-${LABEL}-preflight.txt"
RUN_DIR="$ROOT/runs/load-${LABEL}"
SERVER_LOG="$ROOT/logs/load-${LABEL}.log"
BOT_LOG="$ROOT/logs/load-${LABEL}-bots.log"
JSTACK_DIR="$ROOT/logs/load-${LABEL}-jstacks"
THREAD_SAMPLE_REPORT="$ROOT/reports/load-${LABEL}-thread-samples.txt"
THREAD_SAMPLE_JSON="$ROOT/reports/load-${LABEL}-thread-samples.json"
THREAD_PRINT_REPORT="$ROOT/reports/load-${LABEL}-thread-prints.txt"
THREAD_PRINT_JSON="$ROOT/reports/load-${LABEL}-thread-prints.json"
HOTSPOT_RANK_REPORT="$ROOT/reports/load-${LABEL}-hotspot-rank.txt"

mkdir -p "$ROOT/reports"
p500_diagnostic_phase=initializing
p500_diagnostic_report_finalized=false

write_report_header() {
  {
    echo "p500_contended_diagnostic_stamp=$STAMP"
    echo "p500_contended_diagnostic_label=$LABEL"
    echo "p500_contended_diagnostic_report=$RUN_REPORT"
    echo "p500_contended_diagnostic_generated_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "p500_contended_diagnostic_profile=$GATE_PROFILE"
    echo "p500_contended_diagnostic_non_claim=true"
    echo "p500_contended_diagnostic_production_claim_eligible=false"
    echo "p500_contended_diagnostic_non_claim_reason=foreign_processes_and_busy_host_are_explicitly_allowed"
    echo "p500_contended_diagnostic_target=500_bots_32_view_32_simulation_creative_block"
    echo "p500_contended_diagnostic_no_bundle_publication=true"
    echo "load_test_gate_profile=$GATE_PROFILE"
    echo "load_test_allow_foreign_processes=true"
    echo "load_test_allow_busy_host=true"
    echo "load_test_abort_on_host_contention=false"
    echo "load_test_wait_for_host_ready=false"
    echo "load_test_host_synthetic_canary=false"
    echo "load_test_scenario=block"
    echo "load_test_gamemode=creative"
    echo "bot_count=$BOT_COUNT"
    echo "duration_seconds=$DURATION_SECONDS"
    echo "view_distance=$VIEW_DISTANCE"
    echo "simulation_distance=$SIMULATION_DISTANCE"
    echo "bot_block_ramp_seconds=$BOT_BLOCK_RAMP_SECONDS"
    echo "bot_block_action_interval_ms=$BOT_BLOCK_ACTION_INTERVAL_MS"
    echo "bot_action_start_mode=$BOT_ACTION_START_MODE"
    echo "bot_action_ready_settle_ms=$BOT_ACTION_READY_SETTLE_MS"
    echo "bot_action_ready_requires_block_armed=$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED"
    echo "bot_action_ready_min_count=$BOT_ACTION_READY_MIN_COUNT"
    echo "bot_swarm_node_max_old_space_mb=$BOT_SWARM_NODE_MAX_OLD_SPACE_MB"
    echo "bot_swarm_node_max_semi_space_mb=$BOT_SWARM_NODE_MAX_SEMI_SPACE_MB"
    echo "bukkit_connection_throttle=$BUKKIT_CONNECTION_THROTTLE"
    echo "paper_chunk_worker_threads=$PAPER_CHUNK_WORKER_THREADS"
    echo "paper_player_max_send_rate=$PAPER_PLAYER_MAX_SEND_RATE"
    echo "paper_player_max_gen_rate=$PAPER_PLAYER_MAX_GEN_RATE"
    echo "paper_receive_backpressure_enabled=$PAPER_RECEIVE_BACKPRESSURE_ENABLED"
    echo "paper_receive_backpressure_pause_at_queued_packets=$PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS"
    echo "paper_receive_backpressure_resume_at_queued_packets=$PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS"
    echo "load_test_thread_samples=$THREAD_SAMPLES"
    echo "load_test_thread_sample_interval_seconds=$THREAD_SAMPLE_INTERVAL_SECONDS"
    echo "load_test_thread_sample_start_after_seconds=$THREAD_SAMPLE_START_AFTER_SECONDS"
    echo "p500_contended_diagnostic_server_ready_timeout_seconds=$SERVER_READY_TIMEOUT_SECONDS"
    echo "load_test_server_ready_timeout_seconds=$SERVER_READY_TIMEOUT_SECONDS"
    echo "summary_report=$SUMMARY"
    echo "gate_report=$GATE"
    echo "preflight_report=$PREFLIGHT"
    echo "run_dir=$RUN_DIR"
    echo "server_log=$SERVER_LOG"
    echo "bot_log=$BOT_LOG"
    echo "diagnostic_thread_sample_dir=$JSTACK_DIR"
    echo "thread_sample_report=$THREAD_SAMPLE_REPORT"
    echo "thread_sample_json=$THREAD_SAMPLE_JSON"
    echo "thread_print_report=$THREAD_PRINT_REPORT"
    echo "thread_print_json=$THREAD_PRINT_JSON"
    echo "hotspot_rank_report=$HOTSPOT_RANK_REPORT"
    echo "artifact_hashes=$ROOT/reports/artifact-hashes.txt"
    echo "artifacts_json=$ROOT/reports/artifacts.json"
  } | tee "$RUN_REPORT"
}

append_if_exists() {
  local key="$1"
  local path="$2"
  if [[ -e "$path" ]]; then
    echo "${key}=present path=${path}" | tee -a "$RUN_REPORT"
  else
    echo "${key}=missing path=${path}" | tee -a "$RUN_REPORT"
  fi
}

finalize_diagnostic_report() {
  local status="$1"
  local finish_kind="$2"
  if [[ "$p500_diagnostic_report_finalized" == "true" ]]; then
    return
  fi
  p500_diagnostic_report_finalized=true
  {
    echo "p500_contended_diagnostic_finish_kind=$finish_kind"
    echo "p500_contended_diagnostic_phase=${p500_diagnostic_phase:-unknown}"
    echo "p500_contended_diagnostic_exit_code=$status"
  } | tee -a "$RUN_REPORT"
  append_if_exists summary_report_status "$SUMMARY"
  append_if_exists gate_report_status "$GATE"
  append_if_exists preflight_report_status "$PREFLIGHT"
  append_if_exists server_log_status "$SERVER_LOG"
  append_if_exists bot_log_status "$BOT_LOG"
  append_if_exists diagnostic_thread_sample_dir_status "$JSTACK_DIR"
}

finalize_diagnostic_on_exit() {
  local status=$?
  trap - EXIT
  finalize_diagnostic_report "$status" interrupted
  exit "$status"
}

trap finalize_diagnostic_on_exit EXIT

write_report_header

if [[ "$REFRESH_ARTIFACTS" == "true" ]]; then
  p500_diagnostic_phase=artifact-refresh
  echo "artifact_report_refresh=true" | tee -a "$RUN_REPORT"
  python3 "$ROOT/scripts/update_artifact_reports.py" | tee -a "$RUN_REPORT"
  sha256sum -c "$ROOT/reports/artifact-hashes.txt" | tee -a "$RUN_REPORT"
else
  echo "artifact_report_refresh=false" | tee -a "$RUN_REPORT"
fi

if [[ "$DRY_RUN" == "true" ]]; then
  {
    echo "p500_contended_diagnostic_dry_run=true"
  } | tee -a "$RUN_REPORT"
  finalize_diagnostic_report 0 complete
  exit 0
fi

p500_diagnostic_phase=load-test
set +e
(
  cd "$ROOT"
  MC_EULA_AGREE=true \
    LOAD_TEST_GATE_PROFILE="$GATE_PROFILE" \
    LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
    LOAD_TEST_ALLOW_BUSY_HOST=true \
    LOAD_TEST_ABORT_ON_HOST_CONTENTION=false \
    LOAD_TEST_WAIT_FOR_HOST_READY=false \
    LOAD_TEST_HOST_SYNTHETIC_CANARY=false \
    LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS="$SERVER_READY_TIMEOUT_SECONDS" \
    LOAD_TEST_THREAD_SAMPLES="$THREAD_SAMPLES" \
    LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS="$THREAD_SAMPLE_INTERVAL_SECONDS" \
    LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS="$THREAD_SAMPLE_START_AFTER_SECONDS" \
    LOAD_TEST_SCENARIO=block \
    LOAD_TEST_GAMEMODE=creative \
    LOAD_TEST_LABEL="$LABEL" \
    BOT_COUNT="$BOT_COUNT" \
    DURATION_SECONDS="$DURATION_SECONDS" \
    VIEW_DISTANCE="$VIEW_DISTANCE" \
    SIMULATION_DISTANCE="$SIMULATION_DISTANCE" \
    BOT_BLOCK_RAMP_SECONDS="$BOT_BLOCK_RAMP_SECONDS" \
    BOT_BLOCK_ACTION_INTERVAL_MS="$BOT_BLOCK_ACTION_INTERVAL_MS" \
    BOT_ACTION_START_MODE="$BOT_ACTION_START_MODE" \
    BOT_ACTION_READY_SETTLE_MS="$BOT_ACTION_READY_SETTLE_MS" \
    BOT_ACTION_READY_REQUIRES_BLOCK_ARMED="$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" \
    BOT_ACTION_READY_MIN_COUNT="$BOT_ACTION_READY_MIN_COUNT" \
    BOT_ACTION_READY_MIN_FRACTION="$BOT_ACTION_READY_MIN_FRACTION" \
    BOT_SWARM_NODE_MAX_OLD_SPACE_MB="$BOT_SWARM_NODE_MAX_OLD_SPACE_MB" \
    BOT_SWARM_NODE_MAX_SEMI_SPACE_MB="$BOT_SWARM_NODE_MAX_SEMI_SPACE_MB" \
    BUKKIT_CONNECTION_THROTTLE="$BUKKIT_CONNECTION_THROTTLE" \
    PAPER_DISABLE_METHOD_PROFILER="$PAPER_DISABLE_METHOD_PROFILER" \
    PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE="$PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE" \
    PAPER_CHUNK_WORKER_THREADS="$PAPER_CHUNK_WORKER_THREADS" \
    PAPER_PLAYER_MAX_SEND_RATE="$PAPER_PLAYER_MAX_SEND_RATE" \
    PAPER_PLAYER_MAX_GEN_RATE="$PAPER_PLAYER_MAX_GEN_RATE" \
    PAPER_RECEIVE_BACKPRESSURE_ENABLED="$PAPER_RECEIVE_BACKPRESSURE_ENABLED" \
    PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS="$PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS" \
    PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS="$PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS" \
    "$ROOT/scripts/run_load_test.sh" "$@"
)
status=$?
set -e

p500_diagnostic_phase=postprocess
{
  echo "p500_contended_diagnostic_dry_run=false"
} | tee -a "$RUN_REPORT"

summarize_threads() {
  local pattern="$1"
  local report="$2"
  local json_report="$3"
  local status_key="$4"
  local json_status_key="$5"
  local count
  if [[ ! -d "$JSTACK_DIR" ]]; then
    return
  fi
  count="$(find "$JSTACK_DIR" -maxdepth 1 -type f -name "$pattern" | wc -l | tr -d ' ')"
  if [[ "$count" == "0" ]]; then
    return
  fi
  python3 "$ROOT/scripts/summarize_thread_samples.py" \
    "$JSTACK_DIR" \
    --pattern "$pattern" \
    --top 50 \
    --report "$report" \
    --json-report "$json_report" > /dev/null
  append_if_exists "$status_key" "$report"
  append_if_exists "$json_status_key" "$json_report"
}

summarize_threads "thread-sample-*.txt" "$THREAD_SAMPLE_REPORT" "$THREAD_SAMPLE_JSON" thread_sample_report_status thread_sample_json_status
summarize_threads "thread-print-*.txt" "$THREAD_PRINT_REPORT" "$THREAD_PRINT_JSON" thread_print_report_status thread_print_json_status
python3 "$ROOT/scripts/p500_hotspot_rank.py" --report "$RUN_REPORT" > "$HOTSPOT_RANK_REPORT"
append_if_exists hotspot_rank_report_status "$HOTSPOT_RANK_REPORT"

if [[ -f "$GATE" ]]; then
  awk -F= '
    $1 == "gate_pass" { print "observed_gate_pass=" $2 }
    $1 == "claim_eligible" { print "observed_claim_eligible=" $2 }
    $1 == "failure_count" { print "observed_failure_count=" $2 }
    $1 == "environment_invalid" { print "observed_environment_invalid=" $2 }
    $1 == "environment_invalid_kind" { print "observed_environment_invalid_kind=" $2 }
  ' "$GATE" | tee -a "$RUN_REPORT"
fi

finalize_diagnostic_report "$status" complete
exit "$status"
