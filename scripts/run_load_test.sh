#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/launcher_utils.sh"
source "$ROOT/scripts/stress_plugin_overrides.sh"
source "$ROOT/scripts/load_test_defaults.sh"
TASKSET_CMD=()
if [[ -n "${BENCHMARK_CPUSET:-}" ]]; then
  if ! command -v taskset >/dev/null 2>&1; then
    echo "BENCHMARK_CPUSET is set but taskset is unavailable." >&2
    exit 1
  fi
  TASKSET_CMD=(taskset -c "$BENCHMARK_CPUSET")
fi
BOT_COUNT="${BOT_COUNT:-${1:-50}}"
DURATION_SECONDS="${DURATION_SECONDS:-${2:-120}}"
export DURATION_SECONDS
VIEW_DISTANCE="${VIEW_DISTANCE:-32}"
SIMULATION_DISTANCE="${SIMULATION_DISTANCE:-32}"
BOT_SPEED="${BOT_SPEED:-48}"
BOT_MOVE_INTERVAL_MS="${BOT_MOVE_INTERVAL_MS:-100}"
BOT_SEND_STATIONARY_POSITIONS="${BOT_SEND_STATIONARY_POSITIONS:-true}"
BOT_MIXED_ACTION_INTERVAL_MS="${BOT_MIXED_ACTION_INTERVAL_MS:-250}"
BOT_MIXED_COMMAND_INTERVAL_MS="${BOT_MIXED_COMMAND_INTERVAL_MS:-10000}"
BOT_MIXED_USE_ENTITY_ATTACKS="${BOT_MIXED_USE_ENTITY_ATTACKS:-false}"
BOT_DEPHASE_ACTIONS="${BOT_DEPHASE_ACTIONS:-true}"
BOT_MOVE_SCHEDULER_TICK_MS="${BOT_MOVE_SCHEDULER_TICK_MS:-}"
BOT_BLOCK_SCHEDULER_TICK_MS="${BOT_BLOCK_SCHEDULER_TICK_MS:-}"
BOT_MIXED_SCHEDULER_TICK_MS="${BOT_MIXED_SCHEDULER_TICK_MS:-}"
BOT_RAMP_SECONDS="${BOT_RAMP_SECONDS:-60}"
BOT_BLOCK_RAMP_SECONDS="${BOT_BLOCK_RAMP_SECONDS:-}"
BOT_ALTITUDE="${BOT_ALTITUDE:-160}"
BOT_BLOCK_ACTION_INTERVAL_MS="${BOT_BLOCK_ACTION_INTERVAL_MS:-100}"
BOT_GLOBAL_ACTION_START_AFTER_MS="${BOT_GLOBAL_ACTION_START_AFTER_MS:-}"
PAPER_MISC_MAX_JOINS_PER_TICK="${PAPER_MISC_MAX_JOINS_PER_TICK:-}"
BOT_START_MOVING_AFTER_MS_EXPLICIT="${BOT_START_MOVING_AFTER_MS+x}"
BOT_START_MOVING_AFTER_MS="${BOT_START_MOVING_AFTER_MS:-$(( (BOT_RAMP_SECONDS + 15) * 1000 ))}"
LOAD_TEST_SCENARIO="${LOAD_TEST_SCENARIO:-movement}"
case "$LOAD_TEST_SCENARIO" in
  stress-mixed-gameplay)
    LOAD_TEST_SCENARIO="mixed-gameplay"
    LOAD_TEST_GATE_PROFILE="${LOAD_TEST_GATE_PROFILE:-stress-mixed-gameplay}"
    ;;
esac
if [[ -z "${LOAD_TEST_GAMEMODE:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "block" || "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    LOAD_TEST_GAMEMODE="creative"
  else
    LOAD_TEST_GAMEMODE="survival"
  fi
fi
if [[ -z "${BOT_ACTION_START_MODE:-}" ]]; then
  if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_SCENARIO" == "block" ]]; then
    BOT_ACTION_START_MODE="all-ready"
  elif [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    BOT_ACTION_START_MODE="all-ready"
  else
    BOT_ACTION_START_MODE="timer"
  fi
fi
if [[ -z "${BOT_BLOCK_MOVEMENT_MODE:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "block" || "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    BOT_BLOCK_MOVEMENT_MODE="wait-for-teleport"
  else
    BOT_BLOCK_MOVEMENT_MODE="walk"
  fi
fi
if [[ -z "${BOT_ACTION_READY_SETTLE_MS:-}" ]]; then
  if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_SCENARIO" == "block" ]]; then
    BOT_ACTION_READY_SETTLE_MS=15000
  elif [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    BOT_ACTION_READY_SETTLE_MS=15000
  else
    BOT_ACTION_READY_SETTLE_MS=0
  fi
fi
if [[ -z "${BOT_ACTION_READY_REQUIRES_BLOCK_ARMED:-}" ]]; then
  if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_SCENARIO" == "block" ]]; then
    BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true
  elif [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true
  else
    BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=false
  fi
fi
BOT_ACTION_READY_MIN_COUNT="${BOT_ACTION_READY_MIN_COUNT:-$BOT_COUNT}"
BOT_ACTION_READY_MIN_FRACTION="${BOT_ACTION_READY_MIN_FRACTION:-1}"
if [[ -z "${BOT_CHECK_TIMEOUT_INTERVAL_MS:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    BOT_CHECK_TIMEOUT_INTERVAL_MS=180000
  else
    BOT_CHECK_TIMEOUT_INTERVAL_MS=60000
  fi
fi
BOT_STRICT_FAILURES="${BOT_STRICT_FAILURES:-}"
if [[ -z "$BOT_STRICT_FAILURES" ]]; then
  if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* ]]; then
    BOT_STRICT_FAILURES=true
  else
    BOT_STRICT_FAILURES=false
  fi
fi
BOT_MIN_RUN_AFTER_ACTION_OPEN_MS="${BOT_MIN_RUN_AFTER_ACTION_OPEN_MS:-0}"
BOT_SWARM_NODE_MAX_OLD_SPACE_MB="${BOT_SWARM_NODE_MAX_OLD_SPACE_MB:-512}"
BOT_SWARM_NODE_MAX_SEMI_SPACE_MB="${BOT_SWARM_NODE_MAX_SEMI_SPACE_MB:-16}"
if [[ -z "${BOT_MAX_DURATION_MS:-}" ]]; then
  if [[ "$BOT_MIN_RUN_AFTER_ACTION_OPEN_MS" =~ ^[0-9]+$ ]] && (( BOT_MIN_RUN_AFTER_ACTION_OPEN_MS > 0 )); then
    BOT_MAX_DURATION_MS="$(( DURATION_SECONDS * 1000 + BOT_MIN_RUN_AFTER_ACTION_OPEN_MS ))"
  else
    BOT_MAX_DURATION_MS="$(( DURATION_SECONDS * 1000 ))"
  fi
fi
BOT_ACTION_START_MODE="${BOT_ACTION_START_MODE//_/-}"
case "$BOT_ACTION_START_MODE" in
  allready|swarm-ready) BOT_ACTION_START_MODE="all-ready" ;;
esac
BOT_BLOCK_MOVEMENT_MODE="${BOT_BLOCK_MOVEMENT_MODE//_/-}"
default_java_opts_load() {
  local bot_count="$1"
  local xmx_g=2

  if [[ "$bot_count" =~ ^[0-9]+$ ]]; then
    if (( bot_count > 500 )); then
      xmx_g=8
    elif (( bot_count > 200 )); then
      xmx_g=6
    elif (( bot_count > 50 )); then
      xmx_g=4
    fi
  fi

  printf -- '-Xms1G -Xmx%sG -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100\n' "$xmx_g"
}
JAVA_OPTS_LOAD="${JAVA_OPTS_LOAD:-$(default_java_opts_load "$BOT_COUNT")}"
LOAD_TEST_NATIVE_MEMORY_TRACKING="${LOAD_TEST_NATIVE_MEMORY_TRACKING:-false}"
LAUNCHER="${LAUNCHER:-$ROOT/artifacts/optimized-runtime/run.sh}"
LAUNCHER="$(resolve_launcher_path "$LAUNCHER")"
server_proc_cmdline=""
server_jcmd_vm_command_line_available=false
server_jcmd_vm_command_line=""
bot_child_process_count=0
bot_child_process_missing_count=0
bot_child_process_rss_kb_total=0
bot_child_process_rss_kb_max=0
bot_child_process_pss_kb_total=0
bot_child_process_pss_kb_max=0
if [[ -z "${LOAD_TEST_LABEL:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "block" ]]; then
    LOAD_TEST_LABEL="${BOT_COUNT}bots-block-vd${VIEW_DISTANCE}-sd${SIMULATION_DISTANCE}"
  else
    LOAD_TEST_LABEL="${BOT_COUNT}bots-vd${VIEW_DISTANCE}-sd${SIMULATION_DISTANCE}"
  fi
fi
if [[ -n "${BOT_ACTION_START_AFTER_MS:-}" ]]; then
  :
elif [[ -n "$BOT_START_MOVING_AFTER_MS_EXPLICIT" ]]; then
  BOT_ACTION_START_AFTER_MS="$BOT_START_MOVING_AFTER_MS"
elif [[ "$LOAD_TEST_SCENARIO" == "block" ]]; then
  BOT_ACTION_START_AFTER_MS=5000
elif [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" && "$BOT_ACTION_START_MODE" != "timer" ]]; then
  BOT_ACTION_START_AFTER_MS=0
else
  BOT_ACTION_START_AFTER_MS="$BOT_START_MOVING_AFTER_MS"
fi
if [[ "$LOAD_TEST_SCENARIO" == "block" && -n "$BOT_BLOCK_RAMP_SECONDS" ]]; then
  BOT_EFFECTIVE_RAMP_SECONDS="$BOT_BLOCK_RAMP_SECONDS"
else
  BOT_EFFECTIVE_RAMP_SECONDS="$BOT_RAMP_SECONDS"
fi
if [[ "$LOAD_TEST_SCENARIO" == "block" && -z "$BOT_BLOCK_RAMP_SECONDS" ]]; then
  BOT_BLOCK_RAMP_SECONDS=$(( BOT_COUNT * 12 / 10 ))
  if (( BOT_BLOCK_RAMP_SECONDS < BOT_RAMP_SECONDS )); then
    BOT_BLOCK_RAMP_SECONDS="$BOT_RAMP_SECONDS"
  fi
  if (( BOT_BLOCK_RAMP_SECONDS < 60 )); then
    BOT_BLOCK_RAMP_SECONDS=60
  fi
  BOT_EFFECTIVE_RAMP_SECONDS="$BOT_BLOCK_RAMP_SECONDS"
fi
if [[ -z "$BOT_GLOBAL_ACTION_START_AFTER_MS" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "block" ]]; then
    BOT_GLOBAL_ACTION_START_AFTER_MS="$(( (BOT_EFFECTIVE_RAMP_SECONDS + 15) * 1000 ))"
  else
    BOT_GLOBAL_ACTION_START_AFTER_MS=0
  fi
fi
BLOCK_ARENA_CENTER_X="${BLOCK_ARENA_CENTER_X:-0}"
BLOCK_ARENA_CENTER_Z="${BLOCK_ARENA_CENTER_Z:-0}"
BLOCK_ARENA_TARGET_Y="${BLOCK_ARENA_TARGET_Y:-$BOT_ALTITUDE}"
if [[ -z "${BLOCK_ARENA_SPACING:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    BLOCK_ARENA_SPACING="$(( VIEW_DISTANCE * 4 ))"
  else
    BLOCK_ARENA_SPACING=4
  fi
fi
BLOCK_ARENA_COLUMNS="${BLOCK_ARENA_COLUMNS:-$(python3 - "$BOT_COUNT" <<'PY'
import math
import sys
count = int(sys.argv[1])
print(max(1, math.ceil(math.sqrt(count))))
PY
)}"
BLOCK_ARENA_ITEM="${BLOCK_ARENA_ITEM:-stone}"
if [[ -z "${LOAD_TEST_ARENA_PRELOAD:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "block" || "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    LOAD_TEST_ARENA_PRELOAD=true
  else
    LOAD_TEST_ARENA_PRELOAD=false
  fi
fi
LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS="${LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS:-$(( BOT_COUNT * 8 + 600 ))}"
LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS="${LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS:-60}"
if [[ -z "${LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS="$VIEW_DISTANCE"
  else
    LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS=0
  fi
fi
if [[ -z "${LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT:-}" ]]; then
  if [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT="${PAPER_CHUNK_WORKER_THREADS:-12}"
  else
    LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT=4
  fi
fi
PAPER_PLAYER_MAX_CONCURRENT_LOADS="${PAPER_PLAYER_MAX_CONCURRENT_LOADS:-}"
PAPER_PLAYER_MAX_CONCURRENT_GENS="${PAPER_PLAYER_MAX_CONCURRENT_GENS:-}"
PAPER_PLAYER_MAX_LOAD_RATE="${PAPER_PLAYER_MAX_LOAD_RATE:-}"
PAPER_PLAYER_MAX_GEN_RATE="${PAPER_PLAYER_MAX_GEN_RATE:-}"
PAPER_PLAYER_MAX_SEND_RATE="${PAPER_PLAYER_MAX_SEND_RATE:-}"
PAPER_CHUNK_WORKER_THREADS="${PAPER_CHUNK_WORKER_THREADS:-}"
PAPER_CHUNK_IO_THREADS="${PAPER_CHUNK_IO_THREADS:-}"
PAPER_PREVENT_MOVING_INTO_UNLOADED_CHUNKS="${PAPER_PREVENT_MOVING_INTO_UNLOADED_CHUNKS:-}"
PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE="${PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE:-}"
PAPER_TICK_RATE_CONTAINER_UPDATE="${PAPER_TICK_RATE_CONTAINER_UPDATE:-}"
PAPER_OPTIMIZE_NON_FLUSH_PACKET_SENDING="${PAPER_OPTIMIZE_NON_FLUSH_PACKET_SENDING:-}"
PAPER_RECEIVE_BACKPRESSURE_ENABLED="${PAPER_RECEIVE_BACKPRESSURE_ENABLED:-}"
PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS="${PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS:-}"
PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS="${PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS:-}"
if [[ -z "$PAPER_RECEIVE_BACKPRESSURE_ENABLED$PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS$PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS" ]]; then
  if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_SCENARIO" == "block" ]]; then
    PAPER_RECEIVE_BACKPRESSURE_ENABLED=true
  fi
fi
BUKKIT_CONNECTION_THROTTLE="${BUKKIT_CONNECTION_THROTTLE:-0}"
LOAD_TEST_WORLD_SOURCE_ORIGINAL="${LOAD_TEST_WORLD_SOURCE:-}"
LOAD_TEST_WORLD_SOURCE_RESOLVED=""
LOAD_TEST_WORLD_SOURCE_KIND="fresh"
LOAD_TEST_WORLD_COPY_METHOD="none"
LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA="${LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA:-false}"
LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS="${LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS:-false}"
LOAD_TEST_SPARK_BACKGROUND_PROFILER="${LOAD_TEST_SPARK_BACKGROUND_PROFILER:-false}"
LOAD_TEST_STRESS_CORPUS="${LOAD_TEST_STRESS_CORPUS:-false}"
LOAD_TEST_STRESS_PLUGINS="${LOAD_TEST_STRESS_PLUGINS:-$LOAD_TEST_STRESS_CORPUS}"
LOAD_TEST_STRESS_DATAPACKS="${LOAD_TEST_STRESS_DATAPACKS:-$LOAD_TEST_STRESS_CORPUS}"
LOAD_TEST_MOB_STORM_COUNT="${LOAD_TEST_MOB_STORM_COUNT:-0}"
LOAD_TEST_MOB_STORM_ENTITY="${LOAD_TEST_MOB_STORM_ENTITY:-zombie}"
LOAD_TEST_MOB_STORM_CENTER_X="${LOAD_TEST_MOB_STORM_CENTER_X:-0}"
LOAD_TEST_MOB_STORM_CENTER_Z="${LOAD_TEST_MOB_STORM_CENTER_Z:-0}"
LOAD_TEST_MOB_STORM_Y="${LOAD_TEST_MOB_STORM_Y:-surface}"
LOAD_TEST_MOB_STORM_SPACING="${LOAD_TEST_MOB_STORM_SPACING:-2}"
LOAD_TEST_MOB_STORM_COLUMNS="${LOAD_TEST_MOB_STORM_COLUMNS:-$(python3 - "$LOAD_TEST_MOB_STORM_COUNT" <<'PY'
import math
import sys
count = int(sys.argv[1])
print(max(1, math.ceil(math.sqrt(max(1, count)))))
PY
)}"
LOAD_TEST_THREAD_SAMPLES="${LOAD_TEST_THREAD_SAMPLES:-0}"
LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS="${LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS:-2}"
LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS="${LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS:-0}"
LOAD_TEST_ABORT_ON_TPS1_BELOW="${LOAD_TEST_ABORT_ON_TPS1_BELOW:-}"
LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE="${LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE:-}"
LOAD_TEST_ALLOW_FOREIGN_PROCESSES="${LOAD_TEST_ALLOW_FOREIGN_PROCESSES:-false}"
LOAD_TEST_DIAGNOSTIC_MODE="${LOAD_TEST_DIAGNOSTIC_MODE:-false}"
LOAD_TEST_DIAGNOSTIC_FOREIGN_PRESENT=false
LOAD_TEST_ABORT_ON_HOST_CONTENTION="${LOAD_TEST_ABORT_ON_HOST_CONTENTION:-true}"
LOAD_TEST_HOST_CONTENTION_SAMPLE_INTERVAL_SECONDS="${LOAD_TEST_HOST_CONTENTION_SAMPLE_INTERVAL_SECONDS:-5}"
LOAD_TEST_HOST_CONTENTION_BAD_SAMPLES="${LOAD_TEST_HOST_CONTENTION_BAD_SAMPLES:-3}"
LOAD_TEST_HOST_CONTENTION_MAX_LOAD_PER_CPU="${LOAD_TEST_HOST_CONTENTION_MAX_LOAD_PER_CPU:-${LOAD_TEST_MAX_LOAD_PER_CPU:-0.75}}"
LOAD_TEST_HOST_CONTENTION_MAX_STEAL_PERCENT="${LOAD_TEST_HOST_CONTENTION_MAX_STEAL_PERCENT:-${LOAD_TEST_MAX_HOST_STEAL_PERCENT:-10}}"
LOAD_TEST_HOST_CONTENTION_MAX_IOWAIT_PERCENT="${LOAD_TEST_HOST_CONTENTION_MAX_IOWAIT_PERCENT:-${LOAD_TEST_MAX_HOST_IOWAIT_PERCENT:-10}}"
LOAD_TEST_WAIT_FOR_HOST_READY="${LOAD_TEST_WAIT_FOR_HOST_READY:-}"
if [[ -z "$LOAD_TEST_WAIT_FOR_HOST_READY" ]]; then
  case "${LOAD_TEST_GATE_PROFILE:-}" in
    production-*) LOAD_TEST_WAIT_FOR_HOST_READY=true ;;
    *) LOAD_TEST_WAIT_FOR_HOST_READY=false ;;
  esac
fi
LOAD_TEST_HOST_READY_TIMEOUT_SECONDS="${LOAD_TEST_HOST_READY_TIMEOUT_SECONDS:-900}"
LOAD_TEST_HOST_READY_POLL_SECONDS="${LOAD_TEST_HOST_READY_POLL_SECONDS:-15}"
LOAD_TEST_HOST_READY_STABLE_SECONDS="${LOAD_TEST_HOST_READY_STABLE_SECONDS:-}"
if [[ -z "$LOAD_TEST_HOST_READY_STABLE_SECONDS" ]]; then
  case "${LOAD_TEST_GATE_PROFILE:-}" in
    production-*) LOAD_TEST_HOST_READY_STABLE_SECONDS=60 ;;
    *) LOAD_TEST_HOST_READY_STABLE_SECONDS=0 ;;
  esac
fi
LOAD_TEST_HOST_SYNTHETIC_CANARY="${LOAD_TEST_HOST_SYNTHETIC_CANARY:-}"
if [[ -z "$LOAD_TEST_HOST_SYNTHETIC_CANARY" ]]; then
  case "${LOAD_TEST_GATE_PROFILE:-}" in
    production-*) LOAD_TEST_HOST_SYNTHETIC_CANARY=true ;;
    *) LOAD_TEST_HOST_SYNTHETIC_CANARY=false ;;
  esac
fi
LOAD_TEST_HOST_SYNTHETIC_CANARY_SECONDS="${LOAD_TEST_HOST_SYNTHETIC_CANARY_SECONDS:-15}"
LOAD_TEST_HOST_SYNTHETIC_CANARY_SAMPLE_INTERVAL_SECONDS="${LOAD_TEST_HOST_SYNTHETIC_CANARY_SAMPLE_INTERVAL_SECONDS:-1}"
LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_STEAL_PERCENT="${LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_STEAL_PERCENT:-${LOAD_TEST_MAX_HOST_STEAL_PERCENT:-10}}"
LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_IOWAIT_PERCENT="${LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_IOWAIT_PERCENT:-${LOAD_TEST_MAX_HOST_IOWAIT_PERCENT:-10}}"
LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS="${LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS:-}"
LOAD_TEST_HOST_SYNTHETIC_CANARY_STAT_PATH="${LOAD_TEST_HOST_SYNTHETIC_CANARY_STAT_PATH:-/proc/stat}"
LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS="${LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS:-360}"
LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS="${LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS:-5}"
RUN="$ROOT/runs/load-${LOAD_TEST_LABEL}"
LOG="$ROOT/logs/load-${LOAD_TEST_LABEL}.log"
BOT_LOG="$ROOT/logs/load-${LOAD_TEST_LABEL}-bots.log"
BOT_LOG_DIR="$ROOT/logs/load-${LOAD_TEST_LABEL}-bots"
RESOURCE_LOG="$ROOT/reports/load-${LOAD_TEST_LABEL}-resources.csv"
MEMORY_SNAPSHOT_DIR="$ROOT/reports/load-${LOAD_TEST_LABEL}-memory"
SUMMARY="$ROOT/reports/load-${LOAD_TEST_LABEL}-summary.txt"
STATUS_JSON="$ROOT/reports/load-${LOAD_TEST_LABEL}-status.json"
JSTACK_DIR="$ROOT/logs/load-${LOAD_TEST_LABEL}-jstacks"
PREFLIGHT="$ROOT/reports/load-${LOAD_TEST_LABEL}-preflight.txt"
HARNESS_EXIT="$ROOT/reports/load-${LOAD_TEST_LABEL}-harness-exit.txt"
EARLY_ABORT_REASON="$RUN/early-abort.reason"
DIAGNOSTIC_REASON="$ROOT/reports/load-${LOAD_TEST_LABEL}-diagnostic.reason"
FIFO="$RUN/console.in"
bot_pid_file="$RUN/bot-pids.txt"
PORT="${SERVER_PORT:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
  sock.bind(("127.0.0.1", 0))
  print(sock.getsockname()[1])
PY
)}"
free_tcp_port() {
  python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}
SQUAREMAP_PORT="${SQUAREMAP_PORT:-$(free_tcp_port)}"
BLUEMAP_PORT="${BLUEMAP_PORT:-$(free_tcp_port)}"
VOICECHAT_PORT="${VOICECHAT_PORT:-$(free_tcp_port)}"
SERVERPULSE_INFLUX_PORT="${SERVERPULSE_INFLUX_PORT:-$(free_tcp_port)}"
SERVERPULSE_INFLUX_LOG="$ROOT/logs/load-${LOAD_TEST_LABEL}-serverpulse-influx-mock.log"
SERVERPULSE_INFLUX_PID=""
export SQUAREMAP_PORT BLUEMAP_PORT VOICECHAT_PORT SERVERPULSE_INFLUX_PORT SERVERPULSE_INFLUX_LOG

if [[ ! -x "$LAUNCHER" && ! -f "$LAUNCHER" ]]; then
  echo "Missing launcher: $LAUNCHER" >&2
  exit 1
fi
file_sha256_or_none() {
  local path="$1"
  if [[ -f "$path" ]]; then
    sha256sum "$path" | awk '{print $1}'
  else
    echo "none"
  fi
}
read_first_hash_or_none() {
  local path="$1"
  if [[ -f "$path" ]]; then
    awk 'NF {print $1; exit}' "$path"
  else
    echo "none"
  fi
}
LAUNCHER_SHA256="$(file_sha256_or_none "$LAUNCHER")"
OPTIMIZED_ARTIFACT_PATH="$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"
OPTIMIZED_RUNTIME_RUN_SH_PATH="$ROOT/artifacts/optimized-runtime/run.sh"
OPTIMIZED_RUNTIME_JAR_SHA256_PATH="$ROOT/artifacts/optimized-runtime/runtime.jar.sha256"
OPTIMIZED_RUNTIME_NATIVE_LIBRARY_PATH="$ROOT/artifacts/optimized-runtime/native/libpaper_native_jni.so"
OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_PATH="$ROOT/artifacts/optimized-runtime/native/libpaper_native_chunk_encode_jni.so"
OPTIMIZED_ARTIFACT_SHA256="$(file_sha256_or_none "$OPTIMIZED_ARTIFACT_PATH")"
OPTIMIZED_RUNTIME_RUN_SH_SHA256="$(file_sha256_or_none "$OPTIMIZED_RUNTIME_RUN_SH_PATH")"
OPTIMIZED_RUNTIME_JAR_SHA256="$(read_first_hash_or_none "$OPTIMIZED_RUNTIME_JAR_SHA256_PATH")"
OPTIMIZED_RUNTIME_NATIVE_LIBRARY_SHA256="$(file_sha256_or_none "$OPTIMIZED_RUNTIME_NATIVE_LIBRARY_PATH")"
OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_SHA256="$(file_sha256_or_none "$OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_PATH")"
export LAUNCHER LAUNCHER_SHA256
export OPTIMIZED_ARTIFACT_PATH OPTIMIZED_ARTIFACT_SHA256
export OPTIMIZED_RUNTIME_RUN_SH_PATH OPTIMIZED_RUNTIME_RUN_SH_SHA256 OPTIMIZED_RUNTIME_JAR_SHA256
export OPTIMIZED_RUNTIME_NATIVE_LIBRARY_PATH OPTIMIZED_RUNTIME_NATIVE_LIBRARY_SHA256
export OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_PATH OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_SHA256
export LOAD_TEST_RUN_DIR="$RUN"
if [[ ! "$BUKKIT_CONNECTION_THROTTLE" =~ ^[0-9]+$ ]]; then
  echo "BUKKIT_CONNECTION_THROTTLE must be a non-negative integer." >&2
  exit 1
fi
if [[ "$LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA" != "true" && "$LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA" != "false" ]]; then
  echo "LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS" != "true" && "$LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS" != "false" ]]; then
  echo "LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_ARENA_PRELOAD" != "true" && "$LOAD_TEST_ARENA_PRELOAD" != "false" ]]; then
  echo "LOAD_TEST_ARENA_PRELOAD must be true or false." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS must be a non-negative integer." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS < 1 )); then
  echo "LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS must be a positive integer." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS" =~ ^[0-9]+$ ]]; then
  echo "LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS must be a non-negative integer." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT" =~ ^[0-9]+$ ]] || (( LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT < 1 )); then
  echo "LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT must be a positive integer." >&2
  exit 1
fi
if [[ "$LOAD_TEST_SPARK_BACKGROUND_PROFILER" != "true" && "$LOAD_TEST_SPARK_BACKGROUND_PROFILER" != "false" ]]; then
  echo "LOAD_TEST_SPARK_BACKGROUND_PROFILER must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_STRESS_CORPUS" != "true" && "$LOAD_TEST_STRESS_CORPUS" != "false" ]]; then
  echo "LOAD_TEST_STRESS_CORPUS must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_STRESS_PLUGINS" != "true" && "$LOAD_TEST_STRESS_PLUGINS" != "false" ]]; then
  echo "LOAD_TEST_STRESS_PLUGINS must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_STRESS_DATAPACKS" != "true" && "$LOAD_TEST_STRESS_DATAPACKS" != "false" ]]; then
  echo "LOAD_TEST_STRESS_DATAPACKS must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_ALLOW_FOREIGN_PROCESSES" != "true" && "$LOAD_TEST_ALLOW_FOREIGN_PROCESSES" != "false" ]]; then
  echo "LOAD_TEST_ALLOW_FOREIGN_PROCESSES must be true or false." >&2
  exit 1
fi
if [[ "$LOAD_TEST_DIAGNOSTIC_MODE" != "true" && "$LOAD_TEST_DIAGNOSTIC_MODE" != "false" ]]; then
  echo "LOAD_TEST_DIAGNOSTIC_MODE must be true or false." >&2
  exit 1
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_ALLOW_FOREIGN_PROCESSES" == "true" ]]; then
  echo "LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true is not allowed with production claim profiles." >&2
  echo "Use a non-production profile for explicitly contaminated/non-claim evidence." >&2
  exit 75
fi
if [[ "$LOAD_TEST_ABORT_ON_HOST_CONTENTION" != "true" && "$LOAD_TEST_ABORT_ON_HOST_CONTENTION" != "false" ]]; then
  echo "LOAD_TEST_ABORT_ON_HOST_CONTENTION must be true or false." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS < 1 )); then
  echo "LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS must be a positive integer." >&2
  exit 1
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "${LOAD_TEST_ALLOW_BUSY_HOST:-false}" == "true" ]]; then
  echo "LOAD_TEST_ALLOW_BUSY_HOST=true is not allowed with production claim profiles." >&2
  echo "Use a non-production profile for explicitly noisy/non-claim evidence." >&2
  exit 75
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_ABORT_ON_HOST_CONTENTION" != "true" ]]; then
  echo "LOAD_TEST_ABORT_ON_HOST_CONTENTION=false is not allowed with production claim profiles." >&2
  echo "Production claim profiles must abort contaminated runs on sustained host contention." >&2
  exit 75
fi
if [[ ! "$LOAD_TEST_HOST_CONTENTION_SAMPLE_INTERVAL_SECONDS" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "LOAD_TEST_HOST_CONTENTION_SAMPLE_INTERVAL_SECONDS must be numeric." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_HOST_CONTENTION_BAD_SAMPLES" =~ ^[0-9]+$ ]] || (( LOAD_TEST_HOST_CONTENTION_BAD_SAMPLES < 1 )); then
  echo "LOAD_TEST_HOST_CONTENTION_BAD_SAMPLES must be a positive integer." >&2
  exit 1
fi
for numeric_var in LOAD_TEST_HOST_CONTENTION_MAX_LOAD_PER_CPU LOAD_TEST_HOST_CONTENTION_MAX_STEAL_PERCENT LOAD_TEST_HOST_CONTENTION_MAX_IOWAIT_PERCENT; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "$numeric_var must be numeric." >&2
    exit 1
  fi
done
if [[ "$LOAD_TEST_WAIT_FOR_HOST_READY" != "true" && "$LOAD_TEST_WAIT_FOR_HOST_READY" != "false" ]]; then
  echo "LOAD_TEST_WAIT_FOR_HOST_READY must be true or false." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_HOST_READY_STABLE_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "LOAD_TEST_HOST_READY_STABLE_SECONDS must be a non-negative integer." >&2
  exit 1
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_WAIT_FOR_HOST_READY" != "true" ]]; then
  echo "Production claim profiles require LOAD_TEST_WAIT_FOR_HOST_READY=true." >&2
  exit 75
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_HOST_READY_STABLE_SECONDS" -lt 60 ]]; then
  echo "Production claim profiles require LOAD_TEST_HOST_READY_STABLE_SECONDS >= 60." >&2
  exit 75
fi
if [[ "$LOAD_TEST_HOST_SYNTHETIC_CANARY" != "true" && "$LOAD_TEST_HOST_SYNTHETIC_CANARY" != "false" ]]; then
  echo "LOAD_TEST_HOST_SYNTHETIC_CANARY must be true or false." >&2
  exit 1
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_HOST_SYNTHETIC_CANARY" != "true" ]]; then
  echo "Production claim profiles require LOAD_TEST_HOST_SYNTHETIC_CANARY=true." >&2
  exit 75
fi
for numeric_var in LOAD_TEST_HOST_SYNTHETIC_CANARY_SECONDS LOAD_TEST_HOST_SYNTHETIC_CANARY_SAMPLE_INTERVAL_SECONDS LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_STEAL_PERCENT LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_IOWAIT_PERCENT; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "$numeric_var must be numeric." >&2
    exit 1
  fi
done
if [[ -n "$LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS" ]] && { [[ ! "$LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS" =~ ^[0-9]+$ ]] || (( LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS < 1 )); }; then
  echo "LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS must be a positive integer when set." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_HOST_READY_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( LOAD_TEST_HOST_READY_TIMEOUT_SECONDS < 1 )); then
  echo "LOAD_TEST_HOST_READY_TIMEOUT_SECONDS must be a positive integer." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_HOST_READY_POLL_SECONDS" =~ ^[0-9]+$ ]] || (( LOAD_TEST_HOST_READY_POLL_SECONDS < 1 )); then
  echo "LOAD_TEST_HOST_READY_POLL_SECONDS must be a positive integer." >&2
  exit 1
fi
if [[ ! "$LOAD_TEST_MOB_STORM_COUNT" =~ ^[0-9]+$ ]]; then
  echo "LOAD_TEST_MOB_STORM_COUNT must be a non-negative integer." >&2
  exit 1
fi
if [[ "$LOAD_TEST_MOB_STORM_Y" != "surface" && ! "$LOAD_TEST_MOB_STORM_Y" =~ ^-?[0-9]+$ ]]; then
  echo "LOAD_TEST_MOB_STORM_Y must be 'surface' or an integer." >&2
  exit 1
fi
if [[ -n "$PAPER_TICK_RATE_CONTAINER_UPDATE" ]]; then
  if [[ ! "$PAPER_TICK_RATE_CONTAINER_UPDATE" =~ ^[0-9]+$ ]] || (( 10#$PAPER_TICK_RATE_CONTAINER_UPDATE < 1 )); then
    echo "PAPER_TICK_RATE_CONTAINER_UPDATE must be a positive integer when set." >&2
    exit 1
  fi
fi
for numeric_var in LOAD_TEST_MOB_STORM_CENTER_X LOAD_TEST_MOB_STORM_CENTER_Z LOAD_TEST_MOB_STORM_SPACING LOAD_TEST_MOB_STORM_COLUMNS; do
  if [[ ! "${!numeric_var}" =~ ^-?[0-9]+$ ]]; then
    echo "$numeric_var must be an integer." >&2
    exit 1
  fi
done
if [[ -n "$LOAD_TEST_ABORT_ON_TPS1_BELOW" && ! "$LOAD_TEST_ABORT_ON_TPS1_BELOW" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "LOAD_TEST_ABORT_ON_TPS1_BELOW must be numeric when set." >&2
  exit 1
fi
if [[ -n "$LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE" && ! "$LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE must be numeric when set." >&2
  exit 1
fi
if [[ ! "$BOT_MOVE_INTERVAL_MS" =~ ^[0-9]+$ ]] || (( BOT_MOVE_INTERVAL_MS < 1 )); then
  echo "BOT_MOVE_INTERVAL_MS must be a positive integer." >&2
  exit 1
fi
if [[ ! "$BOT_MIXED_ACTION_INTERVAL_MS" =~ ^[0-9]+$ ]] || (( BOT_MIXED_ACTION_INTERVAL_MS < 1 )); then
  echo "BOT_MIXED_ACTION_INTERVAL_MS must be a positive integer." >&2
  exit 1
fi
if [[ ! "$BOT_MIXED_COMMAND_INTERVAL_MS" =~ ^[0-9]+$ ]] || (( BOT_MIXED_COMMAND_INTERVAL_MS < 1 )); then
  echo "BOT_MIXED_COMMAND_INTERVAL_MS must be a positive integer." >&2
  exit 1
fi
if [[ "$BOT_DEPHASE_ACTIONS" != "true" && "$BOT_DEPHASE_ACTIONS" != "false" ]]; then
  echo "BOT_DEPHASE_ACTIONS must be true or false." >&2
  exit 1
fi
default_bot_scheduler_tick_ms() {
  local interval="$1"
  if [[ "$BOT_DEPHASE_ACTIONS" == "false" ]]; then
    printf '%s\n' "$interval"
    return
  fi
  local tick=$(( interval / 10 ))
  if (( tick < 5 )); then
    tick=5
  elif (( tick > 25 )); then
    tick=25
  fi
  printf '%s\n' "$tick"
}
BOT_MOVE_SCHEDULER_TICK_MS="${BOT_MOVE_SCHEDULER_TICK_MS:-$(default_bot_scheduler_tick_ms "$BOT_MOVE_INTERVAL_MS")}"
BOT_BLOCK_SCHEDULER_TICK_MS="${BOT_BLOCK_SCHEDULER_TICK_MS:-$(default_bot_scheduler_tick_ms "$BOT_BLOCK_ACTION_INTERVAL_MS")}"
BOT_MIXED_SCHEDULER_TICK_MS="${BOT_MIXED_SCHEDULER_TICK_MS:-$(default_bot_scheduler_tick_ms "$BOT_MIXED_ACTION_INTERVAL_MS")}"
for numeric_var in BOT_MOVE_SCHEDULER_TICK_MS BOT_BLOCK_SCHEDULER_TICK_MS BOT_MIXED_SCHEDULER_TICK_MS; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+$ ]] || (( ${!numeric_var} < 1 )); then
    echo "$numeric_var must be a positive integer." >&2
    exit 1
  fi
done
if [[ "$BOT_SEND_STATIONARY_POSITIONS" != "true" && "$BOT_SEND_STATIONARY_POSITIONS" != "false" ]]; then
  echo "BOT_SEND_STATIONARY_POSITIONS must be true or false." >&2
  exit 1
fi
if [[ ! "$BOT_BLOCK_MOVEMENT_MODE" =~ ^(walk|wait-for-teleport)$ ]]; then
  echo "BOT_BLOCK_MOVEMENT_MODE must be walk or wait-for-teleport." >&2
  exit 1
fi
if [[ "$BOT_MIXED_USE_ENTITY_ATTACKS" != "true" && "$BOT_MIXED_USE_ENTITY_ATTACKS" != "false" ]]; then
  echo "BOT_MIXED_USE_ENTITY_ATTACKS must be true or false." >&2
  exit 1
fi
if [[ ! "$BOT_ACTION_START_MODE" =~ ^(timer|all-ready|ready-count|ready-fraction)$ ]]; then
  echo "BOT_ACTION_START_MODE must be timer, all-ready, ready-count, or ready-fraction." >&2
  exit 1
fi
if [[ ! "$BOT_ACTION_READY_SETTLE_MS" =~ ^[0-9]+$ ]]; then
  echo "BOT_ACTION_READY_SETTLE_MS must be a non-negative integer." >&2
  exit 1
fi
if [[ "$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" != "true" && "$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" != "false" ]]; then
  echo "BOT_ACTION_READY_REQUIRES_BLOCK_ARMED must be true or false." >&2
  exit 1
fi
if [[ ! "$BOT_ACTION_READY_MIN_COUNT" =~ ^[0-9]+$ ]]; then
  echo "BOT_ACTION_READY_MIN_COUNT must be a non-negative integer." >&2
  exit 1
fi
if [[ ! "$BOT_ACTION_READY_MIN_FRACTION" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "BOT_ACTION_READY_MIN_FRACTION must be numeric." >&2
  exit 1
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* && "$LOAD_TEST_SCENARIO" == "block" ]]; then
  if [[ "$BOT_ACTION_START_MODE" != "all-ready" ]]; then
    echo "Production block claim profiles require BOT_ACTION_START_MODE=all-ready." >&2
    exit 75
  fi
  if (( BOT_ACTION_READY_SETTLE_MS < 15000 )); then
    echo "Production block claim profiles require BOT_ACTION_READY_SETTLE_MS >= 15000." >&2
    exit 75
  fi
  if [[ "$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" != "true" ]]; then
    echo "Production block claim profiles require BOT_ACTION_READY_REQUIRES_BLOCK_ARMED=true." >&2
    exit 75
  fi
  if (( BOT_ACTION_READY_MIN_COUNT < BOT_COUNT )); then
    echo "Production block claim profiles require BOT_ACTION_READY_MIN_COUNT >= BOT_COUNT." >&2
    exit 75
  fi
fi
if [[ ! "$BOT_CHECK_TIMEOUT_INTERVAL_MS" =~ ^[0-9]+$ ]] || (( BOT_CHECK_TIMEOUT_INTERVAL_MS < 1000 )); then
  echo "BOT_CHECK_TIMEOUT_INTERVAL_MS must be an integer >= 1000." >&2
  exit 1
fi
if [[ "$BOT_STRICT_FAILURES" != "true" && "$BOT_STRICT_FAILURES" != "false" ]]; then
  echo "BOT_STRICT_FAILURES must be true or false." >&2
  exit 1
fi
for numeric_var in BOT_SWARM_NODE_MAX_OLD_SPACE_MB BOT_SWARM_NODE_MAX_SEMI_SPACE_MB; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+$ ]]; then
    echo "$numeric_var must be a non-negative integer; use 0 to disable this Node heap flag." >&2
    exit 1
  fi
done
if [[ -n "$PAPER_MISC_MAX_JOINS_PER_TICK" ]]; then
  if [[ ! "$PAPER_MISC_MAX_JOINS_PER_TICK" =~ ^[0-9]+$ ]] || (( PAPER_MISC_MAX_JOINS_PER_TICK < 1 )); then
    echo "PAPER_MISC_MAX_JOINS_PER_TICK must be a positive integer." >&2
    exit 1
  fi
fi
PAPER_RECEIVE_BACKPRESSURE_CONFIGURED=false
if [[ -n "$PAPER_RECEIVE_BACKPRESSURE_ENABLED$PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS$PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS" ]]; then
  PAPER_RECEIVE_BACKPRESSURE_CONFIGURED=true
  if [[ -n "$PAPER_RECEIVE_BACKPRESSURE_ENABLED" && "$PAPER_RECEIVE_BACKPRESSURE_ENABLED" != "true" && "$PAPER_RECEIVE_BACKPRESSURE_ENABLED" != "false" ]]; then
    echo "PAPER_RECEIVE_BACKPRESSURE_ENABLED must be true or false when set." >&2
    exit 1
  fi
  for numeric_var in PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS; do
    if [[ -n "${!numeric_var}" && ! "${!numeric_var}" =~ ^[0-9]+$ ]]; then
      echo "$numeric_var must be a non-negative integer when set." >&2
      exit 1
    fi
  done
  PAPER_RECEIVE_BACKPRESSURE_ENABLED="${PAPER_RECEIVE_BACKPRESSURE_ENABLED:-false}"
  PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS="${PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS:-4096}"
  PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS="${PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS:-2048}"
fi
if [[ ! "$BOT_MIN_RUN_AFTER_ACTION_OPEN_MS" =~ ^[0-9]+$ ]]; then
  echo "BOT_MIN_RUN_AFTER_ACTION_OPEN_MS must be a non-negative integer." >&2
  exit 1
fi
if [[ ! "$BOT_MAX_DURATION_MS" =~ ^[0-9]+$ ]] || (( BOT_MAX_DURATION_MS < DURATION_SECONDS * 1000 )); then
  echo "BOT_MAX_DURATION_MS must be an integer >= DURATION_SECONDS * 1000." >&2
  exit 1
fi
if [[ "$BOT_ACTION_START_MODE" == "all-ready" ]]; then
  all_ready_gate_floor_ms=$(( BOT_EFFECTIVE_RAMP_SECONDS * 1000 + BOT_ACTION_READY_SETTLE_MS ))
  if (( BOT_MIN_RUN_AFTER_ACTION_OPEN_MS > 0 )); then
    all_ready_required_lifetime_ms=$(( all_ready_gate_floor_ms + BOT_MIN_RUN_AFTER_ACTION_OPEN_MS ))
    all_ready_configured_lifetime_ms="$BOT_MAX_DURATION_MS"
    all_ready_limit_name="BOT_MAX_DURATION_MS"
  else
    all_ready_required_lifetime_ms="$all_ready_gate_floor_ms"
    all_ready_configured_lifetime_ms=$(( DURATION_SECONDS * 1000 ))
    all_ready_limit_name="DURATION_SECONDS"
  fi
  if (( all_ready_configured_lifetime_ms < all_ready_required_lifetime_ms )); then
    echo "$all_ready_limit_name leaves no possible all-ready gate window: configured=${all_ready_configured_lifetime_ms}ms required>=${all_ready_required_lifetime_ms}ms ramp=${BOT_EFFECTIVE_RAMP_SECONDS}s settle=${BOT_ACTION_READY_SETTLE_MS}ms post_open=${BOT_MIN_RUN_AFTER_ACTION_OPEN_MS}ms." >&2
    exit 75
  fi
fi
export BOT_MIN_RUN_AFTER_ACTION_OPEN_MS BOT_MAX_DURATION_MS BOT_STRICT_FAILURES
export BOT_SWARM_NODE_MAX_OLD_SPACE_MB BOT_SWARM_NODE_MAX_SEMI_SPACE_MB
export LOAD_TEST_NATIVE_MEMORY_TRACKING
export LOAD_TEST_ARENA_PRELOAD LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS
if [[ -z "${BOT_SWARM_SHARDS:-}" ]]; then
  BOT_SWARM_SHARDS="$(load_test_default_bot_swarm_shards "$LOAD_TEST_SCENARIO" "$BOT_COUNT")"
fi
if [[ ! "$BOT_SWARM_SHARDS" =~ ^[0-9]+$ ]] || (( BOT_SWARM_SHARDS < 1 )); then
  echo "BOT_SWARM_SHARDS must be a positive integer." >&2
  exit 1
fi
if (( BOT_SWARM_SHARDS > BOT_COUNT )); then
  BOT_SWARM_SHARDS="$BOT_COUNT"
fi
export BOT_SWARM_SHARDS
export BOT_LOG_DIR
if load_test_should_use_shared_action_gate "$BOT_SWARM_SHARDS" "$BOT_ACTION_START_MODE"; then
  BOT_SWARM_SHARED_GATE_FILE="$RUN/bot-action-gate.json"
else
  BOT_SWARM_SHARED_GATE_FILE=""
fi
export BOT_SWARM_SHARED_GATE_FILE
if [[ ! "$LOAD_TEST_THREAD_SAMPLES" =~ ^[0-9]+$ ]]; then
  echo "LOAD_TEST_THREAD_SAMPLES must be a non-negative integer." >&2
  exit 1
fi
case "$LOAD_TEST_NATIVE_MEMORY_TRACKING" in
  true|false) ;;
  *) echo "LOAD_TEST_NATIVE_MEMORY_TRACKING must be true or false." >&2; exit 1 ;;
esac
for numeric_var in LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "$numeric_var must be numeric." >&2
    exit 1
  fi
done
if [[ ! "$LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS must be a non-negative integer." >&2
  exit 1
fi
if (( LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS < 1 )); then
  echo "LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS must be at least 1." >&2
  exit 1
fi
LOAD_TEST_MEMORY_SNAPSHOTS="${LOAD_TEST_MEMORY_SNAPSHOTS:-true}"
LOAD_TEST_MEMORY_SNAPSHOT_INTERVAL_SECONDS="${LOAD_TEST_MEMORY_SNAPSHOT_INTERVAL_SECONDS:-30}"
LOAD_TEST_MEMORY_SNAPSHOT_MIN_RSS_DELTA_KB="${LOAD_TEST_MEMORY_SNAPSHOT_MIN_RSS_DELTA_KB:-65536}"
case "$LOAD_TEST_MEMORY_SNAPSHOTS" in
  true|false) ;;
  *) echo "LOAD_TEST_MEMORY_SNAPSHOTS must be true or false." >&2; exit 1 ;;
esac
for numeric_var in LOAD_TEST_MEMORY_SNAPSHOT_INTERVAL_SECONDS LOAD_TEST_MEMORY_SNAPSHOT_MIN_RSS_DELTA_KB; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+$ ]]; then
    echo "$numeric_var must be a non-negative integer." >&2
    exit 1
  fi
done
if (( LOAD_TEST_MEMORY_SNAPSHOT_INTERVAL_SECONDS < 1 )); then
  echo "LOAD_TEST_MEMORY_SNAPSHOT_INTERVAL_SECONDS must be at least 1." >&2
  exit 1
fi
if [[ -n "$LOAD_TEST_WORLD_SOURCE_ORIGINAL" ]]; then
  LOAD_TEST_WORLD_SOURCE_RESOLVED="$(python3 - "$ROOT" "$LOAD_TEST_WORLD_SOURCE_ORIGINAL" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
path = pathlib.Path(sys.argv[2])
if not path.is_absolute():
    path = root / path
print(path.resolve())
PY
)"
  if [[ -f "$LOAD_TEST_WORLD_SOURCE_RESOLVED/level.dat" ]]; then
    LOAD_TEST_WORLD_SOURCE_KIND="single-world"
  elif [[ -f "$LOAD_TEST_WORLD_SOURCE_RESOLVED/world/level.dat" ]]; then
    LOAD_TEST_WORLD_SOURCE_KIND="server-run"
  else
    echo "LOAD_TEST_WORLD_SOURCE must point to a world directory or a server run containing world/level.dat: $LOAD_TEST_WORLD_SOURCE_RESOLVED" >&2
    exit 1
  fi
fi

mkdir -p "$ROOT/logs" "$ROOT/reports"
rm -f "$LOG" "$BOT_LOG" "$RESOURCE_LOG" "$SUMMARY" "$STATUS_JSON" "$HARNESS_EXIT" "$SERVERPULSE_INFLUX_LOG" "$DIAGNOSTIC_REASON" "$bot_pid_file"
rm -rf "$BOT_LOG_DIR" "$JSTACK_DIR" "$MEMORY_SNAPSHOT_DIR"
BOT_PID_FILE="$bot_pid_file"
export BOT_PID_FILE

run_host_preflight_once() {
  local status=0
  python3 - "$PREFLIGHT" \
  "${LOAD_TEST_MIN_HOST_IDLE_PERCENT:-40}" \
  "${LOAD_TEST_MAX_LOAD_PER_CPU:-0.75}" \
  "${LOAD_TEST_MAX_HOST_STEAL_PERCENT:-10}" \
  "${LOAD_TEST_MAX_HOST_IOWAIT_PERCENT:-10}" \
  "${LOAD_TEST_ALLOW_BUSY_HOST:-false}" \
  "$LOAD_TEST_ALLOW_FOREIGN_PROCESSES" \
  "$LOAD_TEST_DIAGNOSTIC_MODE" <<'PY' || status=$?
import os
import subprocess
import sys
import time

report_path = sys.argv[1]
min_idle = float(sys.argv[2])
max_load_per_cpu = float(sys.argv[3])
max_steal_percent = float(sys.argv[4])
max_iowait_percent = float(sys.argv[5])
allow_busy_host = sys.argv[6] == "true"
allow_foreign_processes = sys.argv[7] == "true"
diagnostic_mode = sys.argv[8] == "true"

def parse_cpu_list(value):
    cpus = set()
    for raw_part in value.split(","):
        part = raw_part.strip()
        if not part:
            continue
        if "-" in part:
            start_raw, end_raw = part.split("-", 1)
            start = int(start_raw)
            end = int(end_raw)
            if end < start:
                raise ValueError(value)
            cpus.update(range(start, end + 1))
        else:
            cpus.add(int(part))
    return cpus

def effective_cpu_count():
    cpuset = os.environ.get("BENCHMARK_CPUSET", "").strip()
    if cpuset:
        try:
            parsed = parse_cpu_list(cpuset)
            if parsed:
                return len(parsed)
        except ValueError:
            pass
    try:
        return max(1, len(os.sched_getaffinity(0)))
    except (AttributeError, OSError):
        return os.cpu_count() or 1

cpu_count = effective_cpu_count()

def read_cpu():
    with open("/proc/stat", "r", encoding="utf-8") as handle:
        fields = handle.readline().split()[1:]
    values = [int(value) for value in fields]
    iowait = values[4] if len(values) > 4 else 0
    steal = values[7] if len(values) > 7 else 0
    idle = values[3] + iowait
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

patterns = (
    "java --add-modules",
    "server.jar",
    "velocity",
    "run_load_test",
    "mc_bot",
    "node _codex",
    "probe.js",
)
hard_block_patterns = (
    "java --add-modules",
    "server.jar",
    "mc_bot",
    "probe.js",
)
hard_block_override = os.environ.get("LOAD_TEST_STRICT_FOREIGN_PROCESS_HARD_BLOCK_PATTERNS", "").strip()
if hard_block_override:
    hard_block_patterns = tuple(
        pattern.strip()
        for pattern in hard_block_override.split(",")
        if pattern.strip()
    )
    if not hard_block_patterns:
        hard_block_patterns = ("__load_test_no_empty_foreign_pattern__",)
scanner_noise_patterns = (
    "pgrep -af",
    "ps -eo",
    "rg ",
    "grep ",
    "sed -n",
    "awk ",
)
try:
    ps_lines = subprocess.check_output(
        ["ps", "-eo", "pid=,ppid=,stat=,etime=,%cpu=,%mem=,args="],
        text=True,
        stderr=subprocess.DEVNULL,
    ).splitlines()
except Exception:
    ps_lines = []

rows = []
ppid_by_pid = {}
for raw_line in ps_lines:
    parts = raw_line.strip().split(None, 6)
    if len(parts) < 7:
        continue
    try:
        row_pid = int(parts[0])
        row_ppid = int(parts[1])
    except ValueError:
        continue
    row = {
        "pid": row_pid,
        "ppid": row_ppid,
        "stat": parts[2],
        "etime": parts[3],
        "cpu": parts[4],
        "mem": parts[5],
        "cmd": parts[6],
        "raw": raw_line.strip(),
    }
    rows.append(row)
    ppid_by_pid[row_pid] = row_ppid

self_and_ancestors = set()
cursor = os.getpid()
while cursor and cursor not in self_and_ancestors:
    self_and_ancestors.add(cursor)
    cursor = ppid_by_pid.get(cursor, 0)

foreign_rows = [row for row in rows if row["pid"] not in self_and_ancestors]
foreign_rows = [
    row for row in foreign_rows
    if not any(pattern in row["cmd"] for pattern in scanner_noise_patterns)
]
interesting = [
    row for row in foreign_rows
    if any(pattern in row["cmd"] for pattern in patterns)
]
foreign_blockers = []
for row in foreign_rows:
    matched = [pattern for pattern in hard_block_patterns if pattern in row["cmd"]]
    if matched:
        foreign_blockers.append((row, matched[0]))

host_ok = (
    idle_percent >= min_idle
    and load_per_cpu <= max_load_per_cpu
    and steal_percent <= max_steal_percent
    and iowait_percent <= max_iowait_percent
)
host_checked = not allow_busy_host
if not host_checked:
    host_ok = True
foreign_ok = allow_foreign_processes or not foreign_blockers
diagnostic_foreign_present = diagnostic_mode and bool(foreign_blockers)
ok = host_ok and (foreign_ok or diagnostic_mode)
with open(report_path, "w", encoding="utf-8") as report:
    report.write(f"host_preflight_ok={str(ok).lower()}\n")
    report.write(f"host_preflight_host_checked={str(host_checked).lower()}\n")
    report.write(f"host_preflight_host_ok={str(host_ok).lower()}\n")
    report.write(f"host_preflight_diagnostic_mode={str(diagnostic_mode).lower()}\n")
    report.write(f"cpu_count={cpu_count}\n")
    report.write(f"load1={load1:.2f}\n")
    report.write(f"load5={load5:.2f}\n")
    report.write(f"load15={load15:.2f}\n")
    report.write(f"load_per_cpu={load_per_cpu:.3f}\n")
    report.write(f"idle_percent_1s={idle_percent:.2f}\n")
    report.write(f"iowait_percent_1s={iowait_percent:.2f}\n")
    report.write(f"steal_percent_1s={steal_percent:.2f}\n")
    report.write(f"min_idle_percent={min_idle:.2f}\n")
    report.write(f"max_load_per_cpu={max_load_per_cpu:.3f}\n")
    report.write(f"max_steal_percent={max_steal_percent:.2f}\n")
    report.write(f"max_iowait_percent={max_iowait_percent:.2f}\n")
    report.write("strict_foreign_process_gate_enabled=true\n")
    report.write(f"strict_foreign_process_gate_allowed={str(allow_foreign_processes).lower()}\n")
    report.write(f"strict_foreign_process_gate_diagnostic_mode={str(diagnostic_mode).lower()}\n")
    report.write(f"strict_foreign_process_gate_diagnostic_foreign_present={str(diagnostic_foreign_present).lower()}\n")
    report.write(f"strict_foreign_process_hard_block_patterns={','.join(hard_block_patterns)}\n")
    report.write(f"strict_foreign_process_gate_pass={str(foreign_ok).lower()}\n")
    report.write(f"strict_foreign_process_count={len(foreign_blockers)}\n")
    report.write("foreign_processes:\n")
    for row, pattern in foreign_blockers[:30]:
        report.write(
            f"foreign_process_pattern={pattern} pid={row['pid']} ppid={row['ppid']} "
            f"stat={row['stat']} etime={row['etime']} cpu={row['cpu']} mem={row['mem']} "
            f"cmd={row['cmd']}\n"
        )
    report.write("interesting_processes:\n")
    for row in interesting[:30]:
        report.write(row["raw"] + "\n")

if not ok:
    print(open(report_path, encoding="utf-8").read(), end="", file=sys.stderr)
    raise SystemExit(75)
PY
  LOAD_TEST_DIAGNOSTIC_FOREIGN_PRESENT=false
  if [[ "$LOAD_TEST_DIAGNOSTIC_MODE" == "true" ]]; then
    if [[ "$(awk -F= '$1 == "strict_foreign_process_gate_diagnostic_foreign_present" { print $2; exit }' "$PREFLIGHT" 2>/dev/null || true)" == "true" ]]; then
      LOAD_TEST_DIAGNOSTIC_FOREIGN_PRESENT=true
    fi
  fi
  return "$status"
}

explain_host_preflight_failure() {
  if rg -q '^host_preflight_host_ok=false$' "$PREFLIGHT" 2>/dev/null; then
    echo "Host is too busy for a trustworthy load benchmark. See $PREFLIGHT" >&2
    echo "Set LOAD_TEST_ALLOW_BUSY_HOST=true only for an explicitly noisy/non-comparable run." >&2
  elif rg -q '^strict_foreign_process_gate_pass=false$' "$PREFLIGHT" 2>/dev/null; then
    echo "Foreign Minecraft/load-test process detected before benchmark start. See $PREFLIGHT" >&2
    echo "Set LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true only for an explicitly contaminated/non-claim run." >&2
  else
    echo "Host is too busy for a trustworthy load benchmark. See $PREFLIGHT" >&2
    echo "Set LOAD_TEST_ALLOW_BUSY_HOST=true only for an explicitly noisy/non-comparable run." >&2
  fi
}

explain_host_ready_timeout() {
  if (( LOAD_TEST_HOST_READY_STABLE_SECONDS > 0 )); then
    echo "Host did not provide a stable ${LOAD_TEST_HOST_READY_STABLE_SECONDS}s ready window before benchmark start. See $PREFLIGHT" >&2
  else
    explain_host_preflight_failure
  fi
}

write_host_ready_abort_reason() {
  mkdir -p "$RUN"
  printf '%s\n' "$1" > "$EARLY_ABORT_REASON"
}

write_diagnostic_reason() {
  mkdir -p "$RUN"
  printf '%s\n' "$1" > "$DIAGNOSTIC_REASON"
}

wait_for_host_ready() {
  if [[ "$LOAD_TEST_WAIT_FOR_HOST_READY" != "true" ]]; then
    if ! run_host_preflight_once; then
      write_host_ready_abort_reason "host_contention_prelaunch_host_ready_preflight_failure"
      explain_host_preflight_failure
      return 75
    fi
    if [[ "$LOAD_TEST_DIAGNOSTIC_FOREIGN_PRESENT" == "true" ]]; then
      write_diagnostic_reason "host_contention_diagnostic_foreign_process_present"
    fi
    return 0
  fi

  local attempt=0
  local elapsed=0
  local host_ok=""
  local foreign_ok=""
  local start
  local status
  local now
  local stable_since=""
  local stable_elapsed=0
  local stable_samples=0
  start="$(date +%s)"

  echo "host_ready_wait_enabled=true"
  echo "host_ready_timeout_seconds=$LOAD_TEST_HOST_READY_TIMEOUT_SECONDS"
  echo "host_ready_poll_seconds=$LOAD_TEST_HOST_READY_POLL_SECONDS"
  if (( LOAD_TEST_HOST_READY_STABLE_SECONDS > 0 )); then
    echo "host_ready_stable_seconds=$LOAD_TEST_HOST_READY_STABLE_SECONDS"
  fi

  while true; do
    set +e
    run_host_preflight_once >/dev/null 2>&1
    status=$?
    set -e
    now="$(date +%s)"
    elapsed=$(( now - start ))
    if [[ "$LOAD_TEST_DIAGNOSTIC_FOREIGN_PRESENT" == "true" ]]; then
      write_diagnostic_reason "host_contention_diagnostic_foreign_process_present"
    fi

    if (( status == 0 )); then
      if (( LOAD_TEST_HOST_READY_STABLE_SECONDS <= 0 )); then
        echo "host_ready_wait_result=ready attempts=$(( attempt + 1 )) elapsed_seconds=$elapsed"
        return 0
      fi
      if [[ -z "$stable_since" ]]; then
        stable_since="$now"
        stable_samples=1
        stable_elapsed=0
      else
        stable_samples=$(( stable_samples + 1 ))
        stable_elapsed=$(( now - stable_since ))
      fi
      if (( stable_elapsed >= LOAD_TEST_HOST_READY_STABLE_SECONDS )); then
        echo "host_ready_wait_result=ready attempts=$(( attempt + 1 )) elapsed_seconds=$elapsed stable_window_seconds=$stable_elapsed stable_samples=$stable_samples"
        return 0
      fi
      if (( elapsed >= LOAD_TEST_HOST_READY_TIMEOUT_SECONDS )); then
        echo "host_ready_wait_result=timeout attempts=$(( attempt + 1 )) elapsed_seconds=$elapsed stable_window_seconds=$stable_elapsed stable_samples=$stable_samples"
        write_host_ready_abort_reason "host_contention_prelaunch_host_ready_timeout attempts=$(( attempt + 1 )) elapsed_seconds=$elapsed stable_window_seconds=$stable_elapsed stable_samples=$stable_samples"
        explain_host_ready_timeout
        return 75
      fi
      attempt=$(( attempt + 1 ))
      echo "host_ready_wait_attempt=$attempt elapsed_seconds=$elapsed host_preflight_host_ok=true stable_window_seconds=$stable_elapsed stable_required_seconds=$LOAD_TEST_HOST_READY_STABLE_SECONDS stable_samples=$stable_samples"
      sleep "$LOAD_TEST_HOST_READY_POLL_SECONDS"
      continue
    fi

    stable_since=""
    stable_elapsed=0
    stable_samples=0

    host_ok="$(awk -F= '$1 == "host_preflight_host_ok" { print $2; exit }' "$PREFLIGHT" 2>/dev/null || true)"
    diagnostic_mode_report="$(awk -F= '$1 == "host_preflight_diagnostic_mode" { print $2; exit }' "$PREFLIGHT" 2>/dev/null || true)"
    foreign_ok="$(awk -F= '$1 == "strict_foreign_process_gate_pass" { print $2; exit }' "$PREFLIGHT" 2>/dev/null || true)"
    if [[ "$host_ok" == "false" ]]; then
      echo "host_ready_wait_result=host_busy elapsed_seconds=$elapsed"
      write_host_ready_abort_reason "host_contention_prelaunch_host_ready_failure elapsed_seconds=$elapsed"
      explain_host_preflight_failure
      return 75
    fi

    if [[ "$diagnostic_mode_report" != "true" && "$foreign_ok" == "false" ]]; then
      echo "host_ready_wait_result=foreign_process elapsed_seconds=$elapsed"
      write_host_ready_abort_reason "host_contention_prelaunch_host_ready_foreign_process elapsed_seconds=$elapsed"
      explain_host_preflight_failure
      return 75
    fi

    if [[ "$host_ok" != "false" ]]; then
      echo "host_ready_wait_result=unexpected_preflight_failure elapsed_seconds=$elapsed"
      write_host_ready_abort_reason "host_contention_prelaunch_host_ready_unexpected_preflight_failure elapsed_seconds=$elapsed"
      explain_host_preflight_failure
      return 75
    fi

    if (( elapsed >= LOAD_TEST_HOST_READY_TIMEOUT_SECONDS )); then
      echo "host_ready_wait_result=timeout attempts=$(( attempt + 1 )) elapsed_seconds=$elapsed"
      write_host_ready_abort_reason "host_contention_prelaunch_host_ready_timeout attempts=$(( attempt + 1 )) elapsed_seconds=$elapsed"
      explain_host_ready_timeout
      return 75
    fi

    attempt=$(( attempt + 1 ))
    echo "host_ready_wait_attempt=$attempt elapsed_seconds=$elapsed host_preflight_host_ok=false"
    sleep "$LOAD_TEST_HOST_READY_POLL_SECONDS"
  done
}

write_prelaunch_abort_summary_and_gate() {
  local reason="$1"
  local exit_code="$2"
  local reason_token
  local host_canary_ok
  local host_canary_samples
  local host_canary_steal_max
  local host_canary_steal_avg
  local host_canary_iowait_max
  local host_canary_iowait_avg
  local gate_report

  mkdir -p "$ROOT/reports" "$RUN"
  reason_token="$(printf '%s' "$reason" | tr '\n ' '__' | sed 's/_*$//')"
  printf '%s\n' "$reason" > "$EARLY_ABORT_REASON"
  host_canary_ok="$(awk -F= '$1 == "host_synthetic_canary_ok" { value = $2 } END { print value }' "$PREFLIGHT" 2>/dev/null || true)"
  host_canary_samples="$(awk -F= '$1 == "host_synthetic_canary_samples" { value = $2 } END { print value }' "$PREFLIGHT" 2>/dev/null || true)"
  host_canary_steal_max="$(awk -F= '$1 == "host_synthetic_canary_steal_percent_max" { value = $2 } END { print value }' "$PREFLIGHT" 2>/dev/null || true)"
  host_canary_steal_avg="$(awk -F= '$1 == "host_synthetic_canary_steal_percent_avg" { value = $2 } END { print value }' "$PREFLIGHT" 2>/dev/null || true)"
  host_canary_iowait_max="$(awk -F= '$1 == "host_synthetic_canary_iowait_percent_max" { value = $2 } END { print value }' "$PREFLIGHT" 2>/dev/null || true)"
  host_canary_iowait_avg="$(awk -F= '$1 == "host_synthetic_canary_iowait_percent_avg" { value = $2 } END { print value }' "$PREFLIGHT" 2>/dev/null || true)"
  {
    printf 'bots=%s view_distance=%s simulation_distance=%s bot_exit=%s\n' "$BOT_COUNT" "$VIEW_DISTANCE" "$SIMULATION_DISTANCE" "$exit_code"
    printf 'duration_seconds=%s\n' "$DURATION_SECONDS"
    printf 'java_opts_load=%s\n' "$JAVA_OPTS_LOAD"
    printf 'server_effective_pid=%s\n' "${SERVER_EFFECTIVE_PID:-}"
    printf 'server_cmdline_effective=%s\n' "${SERVER_CMDLINE_EFFECTIVE:-}"
    printf 'server_jvm_flags_effective=%s\n' "${SERVER_JVM_FLAGS_EFFECTIVE:-}"
    printf 'server_proc_cmdline=%s\n' "${server_proc_cmdline:-}"
    printf 'server_jcmd_vm_command_line_available=%s\n' "${server_jcmd_vm_command_line_available:-false}"
    printf 'server_jcmd_vm_command_line=%s\n' "${server_jcmd_vm_command_line:-}"
    printf 'resource_log=%s\n' "$RESOURCE_LOG"
    printf 'bot_pid_file=%s\n' "$bot_pid_file"
    printf 'launcher_path=%s\n' "$LAUNCHER"
    printf 'launcher_sha256=%s\n' "$LAUNCHER_SHA256"
    printf 'optimized_artifact_path=%s\n' "$OPTIMIZED_ARTIFACT_PATH"
    printf 'optimized_artifact_sha256=%s\n' "$OPTIMIZED_ARTIFACT_SHA256"
    printf 'optimized_runtime_run_sh_path=%s\n' "$OPTIMIZED_RUNTIME_RUN_SH_PATH"
    printf 'optimized_runtime_run_sh_sha256=%s\n' "$OPTIMIZED_RUNTIME_RUN_SH_SHA256"
    printf 'optimized_runtime_jar_sha256=%s\n' "$OPTIMIZED_RUNTIME_JAR_SHA256"
    printf 'optimized_runtime_native_library_path=%s\n' "$OPTIMIZED_RUNTIME_NATIVE_LIBRARY_PATH"
    printf 'optimized_runtime_native_library_sha256=%s\n' "$OPTIMIZED_RUNTIME_NATIVE_LIBRARY_SHA256"
    printf 'optimized_runtime_chunk_encode_native_library_path=%s\n' "$OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_PATH"
    printf 'optimized_runtime_chunk_encode_native_library_sha256=%s\n' "$OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_SHA256"
    printf 'world_mode=%s\n' "fresh"
    printf 'claim_surface=%s\n' "cold-fresh"
    printf 'world_warm_source_present=false\n'
    printf 'world_warm_source=none\n'
    printf 'spark_background_profiler=%s\n' "$LOAD_TEST_SPARK_BACKGROUND_PROFILER"
    printf 'stress_corpus=%s\n' "$LOAD_TEST_STRESS_CORPUS"
    printf 'stress_plugins_enabled=%s\n' "$LOAD_TEST_STRESS_PLUGINS"
    printf 'stress_datapacks_enabled=%s\n' "$LOAD_TEST_STRESS_DATAPACKS"
    printf 'load_test_scenario=%s\n' "$LOAD_TEST_SCENARIO"
    printf 'load_test_gamemode=%s\n' "$LOAD_TEST_GAMEMODE"
    printf 'arena_preload_enabled=%s\n' "$LOAD_TEST_ARENA_PRELOAD"
    printf 'arena_preload_radius_chunks=%s\n' "$LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS"
    printf 'arena_preload_max_in_flight=%s\n' "$LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT"
    printf 'arena_preload_timeout_seconds=%s\n' "$LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS"
    printf 'arena_preload_ack_timeout_seconds=%s\n' "$LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS"
    printf 'bukkit_connection_throttle=%s\n' "$BUKKIT_CONNECTION_THROTTLE"
    printf 'bot_speed_blocks_per_second=%s\n' "$BOT_SPEED"
    printf 'bot_move_interval_ms=%s\n' "$BOT_MOVE_INTERVAL_MS"
    printf 'bot_dephase_actions=%s\n' "$BOT_DEPHASE_ACTIONS"
    printf 'bot_send_stationary_positions=%s\n' "$BOT_SEND_STATIONARY_POSITIONS"
    printf 'bot_block_movement_mode=%s\n' "$BOT_BLOCK_MOVEMENT_MODE"
    printf 'bot_action_start_mode=%s\n' "$BOT_ACTION_START_MODE"
    printf 'bot_action_start_after_ms=%s\n' "$BOT_ACTION_START_AFTER_MS"
    printf 'bot_action_ready_settle_ms=%s\n' "$BOT_ACTION_READY_SETTLE_MS"
    printf 'bot_action_ready_requires_block_armed=%s\n' "$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED"
    printf 'bot_action_ready_min_count=%s\n' "$BOT_ACTION_READY_MIN_COUNT"
    printf 'bot_action_ready_min_fraction=%s\n' "$BOT_ACTION_READY_MIN_FRACTION"
    printf 'bot_check_timeout_interval_ms=%s\n' "$BOT_CHECK_TIMEOUT_INTERVAL_MS"
    printf 'bot_strict_failures=%s\n' "$BOT_STRICT_FAILURES"
    printf 'bot_swarm_shards=%s\n' "${BOT_SWARM_SHARDS:-1}"
    printf 'bot_swarm_node_max_old_space_mb=%s\n' "$BOT_SWARM_NODE_MAX_OLD_SPACE_MB"
    printf 'bot_swarm_node_max_semi_space_mb=%s\n' "$BOT_SWARM_NODE_MAX_SEMI_SPACE_MB"
    printf 'load_test_native_memory_tracking=%s\n' "$LOAD_TEST_NATIVE_MEMORY_TRACKING"
    printf 'bot_resource_samples=0\n'
    printf 'bot_process_count_current=0\n'
    printf 'bot_process_count_max=0\n'
    printf 'bot_rss_mib_current=0.0\n'
    printf 'bot_rss_mib_max=0.0\n'
    printf 'bot_rss_mib_aggregate_current=0.0\n'
    printf 'bot_rss_mib_aggregate_max=0.0\n'
    printf 'bot_pss_mib_available=false\n'
    printf 'bot_child_process_count=0\n'
    printf 'bot_child_process_rss_kb_total=0\n'
    printf 'bot_child_process_rss_kb_max=0\n'
    printf 'bot_child_process_pss_kb_total=0\n'
    printf 'bot_child_process_pss_kb_max=0\n'
    printf 'bot_child_process_missing_count=0\n'
    printf 'bot_action_gate_opened=false\n'
    printf 'load_window_policy=prelaunch-abort\n'
    printf 'load_window_reached_full_online=false\n'
    printf 'load_window_metrics_samples=0\n'
    printf 'online_max=0\n'
    printf 'loaded_chunks_max=0\n'
    printf 'resource_samples=0\n'
    printf 'host_cpu_windows=0\n'
    printf 'host_synthetic_canary_enabled=%s\n' "$LOAD_TEST_HOST_SYNTHETIC_CANARY"
    printf 'host_synthetic_canary_ok=%s\n' "${host_canary_ok:-false}"
    printf 'host_synthetic_canary_samples=%s\n' "${host_canary_samples:-0}"
    printf 'host_synthetic_canary_steal_percent_max=%s\n' "${host_canary_steal_max:-0.00}"
    printf 'host_synthetic_canary_steal_percent_avg=%s\n' "${host_canary_steal_avg:-0.00}"
    printf 'host_synthetic_canary_iowait_percent_max=%s\n' "${host_canary_iowait_max:-0.00}"
    printf 'host_synthetic_canary_iowait_percent_avg=%s\n' "${host_canary_iowait_avg:-0.00}"
    printf 'moved_too_quickly_warnings=0\n'
    printf 'watchdog_thread_dumps=0\n'
    printf 'sync_load_stack_hits=0\n'
    printf 'nearby_players_stack_hits=0\n'
    printf 'thread_check_failures=0\n'
    printf 'chunk_system_errors=0\n'
    printf 'feature_placement_errors=0\n'
    printf 'off_main_poi_hits=0\n'
    printf 'stability_failures=0\n'
    printf 'external_thread_prints=0\n'
    printf 'early_abort_reason=%s\n' "$reason_token"
    printf 'bot_log_tail:\n'
  } > "$SUMMARY"

  cat "$SUMMARY"
  if [[ -n "${LOAD_TEST_GATE_PROFILE:-}" ]]; then
    gate_report="${LOAD_TEST_GATE_REPORT:-$ROOT/reports/load-${LOAD_TEST_LABEL}-gate.txt}"
    set +e
    python3 "$ROOT/scripts/evaluate_load_gate.py" \
      --profile "$LOAD_TEST_GATE_PROFILE" \
      --report "$gate_report" \
      "$SUMMARY"
    set -e
  fi
}

run_host_synthetic_canary() {
  local args=()
  local status
  if [[ "$LOAD_TEST_HOST_SYNTHETIC_CANARY" != "true" ]]; then
    return 0
  fi
  mkdir -p "$RUN"
  rm -f "$EARLY_ABORT_REASON"
  args=(
    --duration-seconds "$LOAD_TEST_HOST_SYNTHETIC_CANARY_SECONDS"
    --sample-interval-seconds "$LOAD_TEST_HOST_SYNTHETIC_CANARY_SAMPLE_INTERVAL_SECONDS"
    --max-steal-percent "$LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_STEAL_PERCENT"
    --max-iowait-percent "$LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_IOWAIT_PERCENT"
    --stat-path "$LOAD_TEST_HOST_SYNTHETIC_CANARY_STAT_PATH"
    --report "$PREFLIGHT"
    --append-report
    --reason-path "$EARLY_ABORT_REASON"
  )
  if [[ -n "$LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS" ]]; then
    args+=(--workers "$LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS")
  fi
  set +e
  python3 "$ROOT/scripts/probe_host_synthetic_contention.py" "${args[@]}"
  status=$?
  set -e
  if (( status == 0 )); then
    return 0
  fi
  if [[ -s "$EARLY_ABORT_REASON" ]]; then
    write_prelaunch_abort_summary_and_gate "$(tr '\n' ' ' < "$EARLY_ABORT_REASON" | sed 's/[[:space:]]*$//')" "$status"
  else
    write_prelaunch_abort_summary_and_gate "host_synthetic_canary_tool_error exit_code=$status" "$status"
  fi
  return "$status"
}

if ! wait_for_host_ready; then
  if [[ -s "$EARLY_ABORT_REASON" ]]; then
    write_prelaunch_abort_summary_and_gate "$(tr '\n' ' ' < "$EARLY_ABORT_REASON" | sed 's/[[:space:]]*$//')" 75
  else
    write_prelaunch_abort_summary_and_gate "host_contention_prelaunch_host_ready_failure" 75
  fi
  exit 75
fi

if run_host_synthetic_canary; then
  :
else
  canary_status=$?
  exit "$canary_status"
fi

rm -rf "$RUN"
mkdir -p "$RUN/plugins" "$ROOT/logs" "$ROOT/reports"
cp "$ROOT/plugins/matrix"/*.jar "$RUN/plugins/"
LOAD_TEST_MATRIX_PLUGIN_JAR_COUNT="$(find "$ROOT/plugins/matrix" -maxdepth 1 -type f -name '*.jar' | wc -l | tr -d ' ')"
LOAD_TEST_STRESS_PLUGIN_JAR_COUNT=0
LOAD_TEST_STRESS_DATAPACK_ZIP_COUNT=0
LOAD_TEST_STRESS_CORPUS_MANIFEST="$RUN/stress-corpus-manifest.txt"
LOAD_TEST_STRESS_CORPUS_MANIFEST_SHA256="none"
STRESS_PLUGIN_RUN_FILES=()
STRESS_DATAPACK_RUN_FILES=()
append_stress_manifest_rows() {
  local kind="$1"
  shift
  local path rel sha bytes
  for path in "$@"; do
    if [[ -f "$path" ]]; then
      rel="${path#"$RUN"/}"
      sha="$(file_sha256_or_none "$path")"
      bytes="$(wc -c < "$path" | tr -d ' ')"
      printf '%s\t%s\t%s\t%s\n' "$kind" "$rel" "$sha" "$bytes"
    fi
  done
}
write_stress_corpus_manifest() {
  local manifest="$1"
  {
    printf 'kind\trelative_path\tsha256\tbytes\n'
    append_stress_manifest_rows "stress_plugin_jar" "${STRESS_PLUGIN_RUN_FILES[@]}"
    append_stress_manifest_rows "stress_datapack_zip" "${STRESS_DATAPACK_RUN_FILES[@]}"
  } > "$manifest"
}
count_stress_manifest_rows() {
  local manifest="$1"
  local kind="$2"
  awk -F '\t' -v kind="$kind" '$1 == kind { count++ } END { print count + 0 }' "$manifest"
}
if compgen -G "$ROOT/plugins/matrix-libraries/*" > /dev/null; then
  cp -R "$ROOT/plugins/matrix-libraries"/. "$RUN/"
fi
if [[ "$LOAD_TEST_STRESS_PLUGINS" == "true" ]]; then
  if ! compgen -G "$ROOT/plugins/stress/*.jar" > /dev/null; then
    echo "LOAD_TEST_STRESS_PLUGINS=true but plugins/stress has no jars. Run scripts/fetch_stress_corpus.py first." >&2
    exit 1
  fi
  stress_plugin_sources=("$ROOT/plugins/stress"/*.jar)
  cp "${stress_plugin_sources[@]}" "$RUN/plugins/"
  for stress_plugin_source in "${stress_plugin_sources[@]}"; do
    STRESS_PLUGIN_RUN_FILES+=("$RUN/plugins/$(basename "$stress_plugin_source")")
  done
  mkdir -p "$RUN/plugins/squaremap" "$RUN/plugins/BlueMap" "$RUN/plugins/voicechat"
  cat > "$RUN/plugins/squaremap/config.yml" <<SQUAREMAP
config-version: 2
settings:
    language-file: lang-en.yml
    debug-mode: false
    update-checker: false
    web-address: http://127.0.0.1:$SQUAREMAP_PORT
    internal-webserver:
        enabled: true
        bind: 127.0.0.1
        port: $SQUAREMAP_PORT
world-settings:
    default:
        map:
            enabled: true
SQUAREMAP
  cat > "$RUN/plugins/BlueMap/core.conf" <<BLUEMAPCORE
accept-download: false
data: "bluemap"
render-thread-count: 1
scan-for-mod-resources: true
metrics: false
BLUEMAPCORE
  cat > "$RUN/plugins/BlueMap/webserver.conf" <<BLUEMAPWEB
enabled: true
webroot: "bluemap/web"
port: $BLUEMAP_PORT
BLUEMAPWEB
  cat > "$RUN/plugins/voicechat/voicechat-server.properties" <<VOICECHAT
port=$VOICECHAT_PORT
bind_address=127.0.0.1
max_voice_distance=48.0
whisper_distance=24.0
codec=VOIP
mtu_size=1024
tcp_rate_limit=16
keep_alive=1000
enable_groups=true
voice_host=
allow_recording=true
spectator_interaction=false
spectator_player_possession=false
force_voice_chat=false
login_timeout=10000
broadcast_range=-1.0
allow_pings=false
VOICECHAT
  mkdir -p "$RUN/plugins/ServerPulse"
  cat > "$RUN/plugins/ServerPulse/config.yml" <<SERVERPULSE
metrics:
  interval: 5
  influxdb:
    url: http://127.0.0.1:$SERVERPULSE_INFLUX_PORT
    org: load-test
    bucket: load_test_metrics
    token: load-test-local-token
    table: minecraft_stats
  tags:
    server: "$LOAD_TEST_LABEL"
messages:
  noPerms: "&7[&bServer&7Pulse] &7No permission."
  reloadConfig: "&7[&bServer&7Pulse] &7Configuration reloaded."
  reloadConfigError: "&7[&bServer&7Pulse] &7Error reloading configuration."
  noArgs: "&7[&bServer&7Pulse] &7Use reload or status."
  playerOnly: "&7[&bServer&7Pulse] &7Players only."
  noCommand: "&7[&bServer&7Pulse] &7Command unavailable."
  reloadConfigUsage: "&7[&bServer&7Pulse] &7Usage: /serverpulse reload."
  statusConfigUsage: "&7[&bServer&7Pulse] &7Usage: /serverpulse status."
  statusConnected: "&7[&bServer&7Pulse] &7Connected to InfluxDB."
  statusNotConnected: "&7[&bServer&7Pulse] &7Not connected to InfluxDB."
SERVERPULSE
  mkdir -p "$RUN/plugins/SkinsRestorer"
  cat > "$RUN/plugins/SkinsRestorer/config.yml" <<SKINSRESTORER
storage:
    defaultSkins:
        enabled: false
    disallowAutoUpdateSkin: true
    skinExpiresAfter: 10080
    uuidExpiresAfter: 10080
api:
    mineskinAPIKey: ''
    fetchRecommendedSkins: false
    mojangBatchWindowSeconds: 1
    elyByEnabled: false
advanced:
    disableOnJoinSkins: true
    enablePaperJoinListener: false
    noConnections: true
SKINSRESTORER
  touch "$RUN/plugins/SkinsRestorer/noautoupdate.txt"
fi
if [[ "$LOAD_TEST_STRESS_DATAPACKS" == "true" ]]; then
  if ! compgen -G "$ROOT/datapacks/stress/*.zip" > /dev/null; then
    echo "LOAD_TEST_STRESS_DATAPACKS=true but datapacks/stress has no zips. Run scripts/fetch_stress_corpus.py first." >&2
    exit 1
  fi
  stress_datapack_sources=("$ROOT/datapacks/stress"/*.zip)
  mkdir -p "$RUN/world/datapacks"
  cp "${stress_datapack_sources[@]}" "$RUN/world/datapacks/"
  for stress_datapack_source in "${stress_datapack_sources[@]}"; do
    STRESS_DATAPACK_RUN_FILES+=("$RUN/world/datapacks/$(basename "$stress_datapack_source")")
  done
fi
write_stress_corpus_manifest "$LOAD_TEST_STRESS_CORPUS_MANIFEST"
LOAD_TEST_STRESS_PLUGIN_JAR_COUNT="$(count_stress_manifest_rows "$LOAD_TEST_STRESS_CORPUS_MANIFEST" "stress_plugin_jar")"
LOAD_TEST_STRESS_DATAPACK_ZIP_COUNT="$(count_stress_manifest_rows "$LOAD_TEST_STRESS_CORPUS_MANIFEST" "stress_datapack_zip")"
LOAD_TEST_STRESS_CORPUS_MANIFEST_SHA256="$(file_sha256_or_none "$LOAD_TEST_STRESS_CORPUS_MANIFEST")"
export LOAD_TEST_MATRIX_PLUGIN_JAR_COUNT LOAD_TEST_STRESS_PLUGIN_JAR_COUNT LOAD_TEST_STRESS_DATAPACK_ZIP_COUNT
export LOAD_TEST_STRESS_CORPUS_MANIFEST LOAD_TEST_STRESS_CORPUS_MANIFEST_SHA256
seed_luckperms_vault_unsafe_lookups "$RUN" "$ROOT"
copy_warm_world_dir() {
  local src="$1"
  local dst="$2"
  rm -rf "$dst"
  mkdir -p "$(dirname "$dst")"
  if command -v rsync >/dev/null 2>&1; then
    LOAD_TEST_WORLD_COPY_METHOD="rsync"
    rsync -a --delete "$src"/ "$dst"/
  else
    LOAD_TEST_WORLD_COPY_METHOD="cp"
    cp -a "$src" "$dst"
  fi
}

if [[ -n "$LOAD_TEST_WORLD_SOURCE_RESOLVED" ]]; then
  if [[ "$LOAD_TEST_WORLD_SOURCE_KIND" == "single-world" ]]; then
    copy_warm_world_dir "$LOAD_TEST_WORLD_SOURCE_RESOLVED" "$RUN/world"
  else
    for world_dir in world world_nether world_the_end; do
      if [[ -d "$LOAD_TEST_WORLD_SOURCE_RESOLVED/$world_dir" ]]; then
        copy_warm_world_dir "$LOAD_TEST_WORLD_SOURCE_RESOLVED/$world_dir" "$RUN/$world_dir"
      fi
    done
  fi
  if [[ "$LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA" != "true" ]]; then
    rm -rf "$RUN/world/playerdata" "$RUN/world/advancements" "$RUN/world/stats"
  fi
  if [[ "$LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS" == "true" ]]; then
    rm -rf "$RUN/world/datapacks"
  fi
fi
mkdir -p "$RUN/plugins/spark"
cat > "$RUN/plugins/spark/config.json" <<SPARKCFG
{
  "_header": "spark configuration file - https://spark.lucko.me/docs/Configuration",
  "backgroundProfiler": $LOAD_TEST_SPARK_BACKGROUND_PROFILER
}
SPARKCFG
cat > "$RUN/eula.txt" <<'EULA'
eula=true
EULA
cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
max-players=$((BOT_COUNT + 20))
level-seed=paper-rust-load-12110
gamemode=$LOAD_TEST_GAMEMODE
force-gamemode=true
view-distance=$VIEW_DISTANCE
simulation-distance=$SIMULATION_DISTANCE
allow-flight=true
enable-rcon=false
motd=Paper Rust Load Test
PROPS
cat > "$RUN/bukkit.yml" <<BUKKIT
# Generated by scripts/run_load_test.sh so localhost bot scale is not limited by the default join throttle.
settings:
  allow-end: true
  warn-on-overload: true
  permissions-file: permissions.yml
  update-folder: update
  plugin-profiling: false
  connection-throttle: $BUKKIT_CONNECTION_THROTTLE
  query-plugins: true
  deprecated-verbose: default
  shutdown-message: Server closed
  minimum-api: none
  use-map-color-cache: true
spawn-limits:
  monsters: 70
  animals: 10
  water-animals: 5
  water-ambient: 20
  water-underground-creature: 5
  axolotls: 5
  ambient: 15
chunk-gc:
  period-in-ticks: 600
ticks-per:
  animal-spawns: 400
  monster-spawns: 1
  water-spawns: 1
  water-ambient-spawns: 1
  water-underground-creature-spawns: 1
  axolotl-spawns: 1
  ambient-spawns: 1
  autosave: 6000
aliases: now-in-commands.yml
BUKKIT

mkdir -p "$RUN/config"
if [[ -n "$PAPER_PLAYER_MAX_CONCURRENT_LOADS$PAPER_PLAYER_MAX_CONCURRENT_GENS$PAPER_PLAYER_MAX_LOAD_RATE$PAPER_PLAYER_MAX_GEN_RATE$PAPER_PLAYER_MAX_SEND_RATE$PAPER_CHUNK_WORKER_THREADS$PAPER_CHUNK_IO_THREADS$PAPER_OPTIMIZE_NON_FLUSH_PACKET_SENDING$PAPER_MISC_MAX_JOINS_PER_TICK" || "$PAPER_RECEIVE_BACKPRESSURE_CONFIGURED" == "true" ]]; then
  cat > "$RUN/config/paper-global.yml" <<PAPERCFG
_version: 31
chunk-loading-advanced:
  auto-config-send-distance: true
  player-max-concurrent-chunk-generates: ${PAPER_PLAYER_MAX_CONCURRENT_GENS:-0}
  player-max-concurrent-chunk-loads: ${PAPER_PLAYER_MAX_CONCURRENT_LOADS:-0}
chunk-loading-basic:
  player-max-chunk-generate-rate: ${PAPER_PLAYER_MAX_GEN_RATE:--1.0}
  player-max-chunk-load-rate: ${PAPER_PLAYER_MAX_LOAD_RATE:-100.0}
  player-max-chunk-send-rate: ${PAPER_PLAYER_MAX_SEND_RATE:-75.0}
chunk-system:
  io-threads: ${PAPER_CHUNK_IO_THREADS:--1}
  worker-threads: ${PAPER_CHUNK_WORKER_THREADS:--1}
network:
  optimize-non-flush-packet-sending: ${PAPER_OPTIMIZE_NON_FLUSH_PACKET_SENDING:-false}
PAPERCFG
  if [[ "$PAPER_RECEIVE_BACKPRESSURE_CONFIGURED" == "true" ]]; then
    cat >> "$RUN/config/paper-global.yml" <<PAPERCFG
  receive-backpressure:
    enabled: $PAPER_RECEIVE_BACKPRESSURE_ENABLED
    pause-at-queued-packets: $PAPER_RECEIVE_BACKPRESSURE_PAUSE_AT_QUEUED_PACKETS
    resume-at-queued-packets: $PAPER_RECEIVE_BACKPRESSURE_RESUME_AT_QUEUED_PACKETS
PAPERCFG
  fi
  cat >> "$RUN/config/paper-global.yml" <<PAPERCFG
misc:
  max-joins-per-tick: ${PAPER_MISC_MAX_JOINS_PER_TICK:-5}
PAPERCFG
fi
if [[ "$PAPER_PREVENT_MOVING_INTO_UNLOADED_CHUNKS" == "true" || -n "$PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE" || -n "$PAPER_TICK_RATE_CONTAINER_UPDATE" ]]; then
  cat > "$RUN/config/paper-world-defaults.yml" <<PAPERWORLD
_version: 31
PAPERWORLD
  if [[ "$PAPER_PREVENT_MOVING_INTO_UNLOADED_CHUNKS" == "true" ]]; then
    cat >> "$RUN/config/paper-world-defaults.yml" <<PAPERWORLD
chunks:
  prevent-moving-into-unloaded-chunks: true
PAPERWORLD
  fi
  if [[ -n "$PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE" ]]; then
    if [[ "$PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE" != "true" && "$PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE" != "false" ]]; then
      echo "PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE must be true or false when set." >&2
      exit 1
    fi
    cat >> "$RUN/config/paper-world-defaults.yml" <<PAPERWORLD
misc:
  update-pathfinding-on-block-update: $PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE
PAPERWORLD
  fi
  if [[ -n "$PAPER_TICK_RATE_CONTAINER_UPDATE" ]]; then
    cat >> "$RUN/config/paper-world-defaults.yml" <<PAPERWORLD
tick-rates:
  container-update: $PAPER_TICK_RATE_CONTAINER_UPDATE
PAPERWORLD
  fi
fi

rm -f "$FIFO"
mkfifo "$FIFO"
stop_serverpulse_influx_mock() {
  if [[ -n "${SERVERPULSE_INFLUX_PID:-}" ]] && kill -0 "$SERVERPULSE_INFLUX_PID" 2>/dev/null; then
    kill "$SERVERPULSE_INFLUX_PID" 2>/dev/null || true
    wait "$SERVERPULSE_INFLUX_PID" 2>/dev/null || true
  fi
}
start_serverpulse_influx_mock() {
  python3 - "$SERVERPULSE_INFLUX_PORT" > "$SERVERPULSE_INFLUX_LOG" 2>&1 <<'PY' &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import sys


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _no_content(self):
        self.send_response(204)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def do_GET(self):
        self._no_content()

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0") or "0")
        if length:
            self.rfile.read(length)
        self._no_content()

    def log_message(self, fmt, *args):
        return


port = int(sys.argv[1])
print(f"load_test_serverpulse_influx_mock=listening host=127.0.0.1 port={port}", flush=True)
server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
server.serve_forever()
PY
  SERVERPULSE_INFLUX_PID=$!
  trap stop_serverpulse_influx_mock EXIT
  local deadline=$((SECONDS + 10))
  while (( SECONDS < deadline )); do
    if rg -q '^load_test_serverpulse_influx_mock=listening ' "$SERVERPULSE_INFLUX_LOG"; then
      return 0
    fi
    if ! kill -0 "$SERVERPULSE_INFLUX_PID" 2>/dev/null; then
      cat "$SERVERPULSE_INFLUX_LOG" >&2 || true
      return 1
    fi
    sleep 0.1
  done
  cat "$SERVERPULSE_INFLUX_LOG" >&2 || true
  return 1
}

SERVER_EFFECTIVE_PID=""
SERVER_CMDLINE_EFFECTIVE=""
SERVER_JVM_FLAGS_EFFECTIVE=""
capture_server_process_metadata() {
  local root_pid="$1"
  local metadata

  if ! metadata="$(python3 - "$root_pid" 2>/dev/null <<'PY'
import os
import pathlib
import subprocess
import sys

root_pid = int(sys.argv[1])

def read_cmdline(process_pid):
    try:
        raw = pathlib.Path(f"/proc/{process_pid}/cmdline").read_bytes()
    except Exception:
        return []
    return [
        part.decode("utf-8", errors="replace").replace("\n", " ")
        for part in raw.split(b"\0")
        if part
    ]

def read_children(process_pid):
    try:
        raw = pathlib.Path(f"/proc/{process_pid}/task/{process_pid}/children").read_text(encoding="utf-8")
    except Exception:
        return []
    return [int(part) for part in raw.split() if part.isdigit()]

def looks_like_java(parts):
    if not parts:
        return False
    basename = os.path.basename(parts[0])
    if basename.startswith("java"):
        return True
    return any(part.startswith("-X") or part.startswith("-XX") for part in parts) or "-jar" in parts

candidate_pid = root_pid
candidate_parts = read_cmdline(root_pid)
queue = [root_pid]
seen = set()
while queue:
    process_pid = queue.pop(0)
    if process_pid in seen:
        continue
    seen.add(process_pid)
    parts = read_cmdline(process_pid)
    if looks_like_java(parts):
        candidate_pid = process_pid
        candidate_parts = parts
        break
    queue.extend(child for child in read_children(process_pid) if child not in seen)

if not candidate_parts:
    try:
        args = subprocess.check_output(
            ["ps", "-o", "args=", "-p", str(root_pid)],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
        candidate_parts = [args] if args else []
    except Exception:
        candidate_parts = []

def extract_jvm_flags(parts):
    if not looks_like_java(parts):
        return ""
    flags = []
    for part in parts[1:]:
        if part == "-jar":
            break
        if not part.startswith("-"):
            break
        flags.append(part)
    return " ".join(flags)

print(f"server_effective_pid={candidate_pid}")
print(f"server_cmdline_effective={' '.join(candidate_parts)}")
print(f"server_jvm_flags_effective={extract_jvm_flags(candidate_parts)}")
PY
)"; then
    metadata=""
  fi

  while IFS= read -r line; do
    case "$line" in
      server_effective_pid=*) SERVER_EFFECTIVE_PID="${line#*=}" ;;
      server_cmdline_effective=*) SERVER_CMDLINE_EFFECTIVE="${line#*=}" ;;
      server_jvm_flags_effective=*) SERVER_JVM_FLAGS_EFFECTIVE="${line#*=}" ;;
    esac
  done <<< "$metadata"
  export SERVER_EFFECTIVE_PID SERVER_CMDLINE_EFFECTIVE SERVER_JVM_FLAGS_EFFECTIVE
}

BOT_RESOURCE_SAMPLES=0
BOT_PROCESS_COUNT_CURRENT=0
BOT_PROCESS_COUNT_MAX=0
BOT_RSS_MIB_CURRENT=0.0
BOT_RSS_MIB_MAX=0.0
BOT_RSS_MIB_AGGREGATE_CURRENT=0.0
BOT_RSS_MIB_AGGREGATE_MAX=0.0
BOT_PSS_MIB_AVAILABLE=false
BOT_PSS_MIB_CURRENT=""
BOT_PSS_MIB_MAX=""
BOT_PSS_MIB_AGGREGATE_CURRENT=""
BOT_PSS_MIB_AGGREGATE_MAX=""
load_bot_resource_state_from_resource_log() {
  local metadata

  if [[ ! -f "$RESOURCE_LOG" ]]; then
    BOT_RESOURCE_SAMPLES=0
    BOT_PROCESS_COUNT_CURRENT=0
    BOT_PROCESS_COUNT_MAX=0
    BOT_RSS_MIB_CURRENT=0.0
    BOT_RSS_MIB_MAX=0.0
    BOT_RSS_MIB_AGGREGATE_CURRENT=0.0
    BOT_RSS_MIB_AGGREGATE_MAX=0.0
    BOT_PSS_MIB_AVAILABLE=false
    BOT_PSS_MIB_CURRENT=""
    BOT_PSS_MIB_MAX=""
    BOT_PSS_MIB_AGGREGATE_CURRENT=""
    BOT_PSS_MIB_AGGREGATE_MAX=""
    return 0
  fi

  if ! metadata="$(python3 - "$RESOURCE_LOG" 2>/dev/null <<'PY'
import csv
import pathlib
import sys

resource_log = pathlib.Path(sys.argv[1])

def int_field(row, key):
    value = (row.get(key) or "").strip()
    if not value:
        return None
    try:
        return int(value)
    except ValueError:
        return None

def mib(kb):
    return f"{kb / 1024:.1f}"

bot_samples = []
try:
    with resource_log.open(newline="", encoding="utf-8", errors="replace") as handle:
        for row in csv.DictReader(handle):
            count = int_field(row, "bot_process_count")
            rss_max = int_field(row, "bot_rss_kb_max")
            rss_total = int_field(row, "bot_rss_kb_total")
            if count is None or count <= 0 or rss_max is None or rss_total is None:
                continue
            pss_max = int_field(row, "bot_pss_kb_max")
            pss_total = int_field(row, "bot_pss_kb_total")
            bot_samples.append({
                "count": count,
                "rss_max": rss_max,
                "rss_total": rss_total,
                "pss_max": pss_max,
                "pss_total": pss_total,
                "pss_available": (row.get("bot_pss_available") or "").lower() == "true"
                and pss_max is not None
                and pss_total is not None,
            })
except FileNotFoundError:
    pass

print(f"bot_resource_samples={len(bot_samples)}")
if bot_samples:
    current = bot_samples[-1]
    print(f"bot_process_count_current={current['count']}")
    print(f"bot_process_count_max={max(sample['count'] for sample in bot_samples)}")
    print(f"bot_rss_mib_current={mib(current['rss_max'])}")
    print(f"bot_rss_mib_max={mib(max(sample['rss_max'] for sample in bot_samples))}")
    print(f"bot_rss_mib_aggregate_current={mib(current['rss_total'])}")
    print(f"bot_rss_mib_aggregate_max={mib(max(sample['rss_total'] for sample in bot_samples))}")
else:
    print("bot_process_count_current=0")
    print("bot_process_count_max=0")
    print("bot_rss_mib_current=0.0")
    print("bot_rss_mib_max=0.0")
    print("bot_rss_mib_aggregate_current=0.0")
    print("bot_rss_mib_aggregate_max=0.0")

pss_samples = [sample for sample in bot_samples if sample["pss_available"]]
print(f"bot_pss_mib_available={str(bool(pss_samples)).lower()}")
if pss_samples:
    if bot_samples and bot_samples[-1]["pss_available"]:
        current = bot_samples[-1]
        print(f"bot_pss_mib_current={mib(current['pss_max'])}")
        print(f"bot_pss_mib_aggregate_current={mib(current['pss_total'])}")
    print(f"bot_pss_mib_max={mib(max(sample['pss_max'] for sample in pss_samples))}")
    print(f"bot_pss_mib_aggregate_max={mib(max(sample['pss_total'] for sample in pss_samples))}")
PY
)"; then
    metadata=""
  fi

  while IFS= read -r line; do
    case "$line" in
      bot_resource_samples=*) BOT_RESOURCE_SAMPLES="${line#*=}" ;;
      bot_process_count_current=*) BOT_PROCESS_COUNT_CURRENT="${line#*=}" ;;
      bot_process_count_max=*) BOT_PROCESS_COUNT_MAX="${line#*=}" ;;
      bot_rss_mib_current=*) BOT_RSS_MIB_CURRENT="${line#*=}" ;;
      bot_rss_mib_max=*) BOT_RSS_MIB_MAX="${line#*=}" ;;
      bot_rss_mib_aggregate_current=*) BOT_RSS_MIB_AGGREGATE_CURRENT="${line#*=}" ;;
      bot_rss_mib_aggregate_max=*) BOT_RSS_MIB_AGGREGATE_MAX="${line#*=}" ;;
      bot_pss_mib_available=*) BOT_PSS_MIB_AVAILABLE="${line#*=}" ;;
      bot_pss_mib_current=*) BOT_PSS_MIB_CURRENT="${line#*=}" ;;
      bot_pss_mib_max=*) BOT_PSS_MIB_MAX="${line#*=}" ;;
      bot_pss_mib_aggregate_current=*) BOT_PSS_MIB_AGGREGATE_CURRENT="${line#*=}" ;;
      bot_pss_mib_aggregate_max=*) BOT_PSS_MIB_AGGREGATE_MAX="${line#*=}" ;;
    esac
  done <<< "$metadata"
}
if [[ "$LOAD_TEST_STRESS_PLUGINS" == "true" ]]; then
  start_serverpulse_influx_mock
  trap stop_serverpulse_influx_mock EXIT
fi
if [[ "$LOAD_TEST_NATIVE_MEMORY_TRACKING" == "true" ]]; then
  if [[ "$JAVA_OPTS_LOAD" != *"-XX:NativeMemoryTracking="* ]]; then
    JAVA_OPTS_LOAD="$JAVA_OPTS_LOAD -XX:+UnlockDiagnosticVMOptions -XX:NativeMemoryTracking=summary"
  fi
fi
if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("${TASKSET_CMD[@]}" "$LAUNCHER" --nogui)
else
  CMD=("${TASKSET_CMD[@]}" java $JAVA_OPTS_LOAD -jar "$LAUNCHER" --nogui)
fi

console_fd_open=false
(
  cd "$RUN"
  JAVA_OPTS="$JAVA_OPTS_LOAD" "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
  echo $! > server.pid
)
pid="$(cat "$RUN/server.pid")"
exec 3>"$FIFO"
console_fd_open=true
trap '' PIPE
for _ in $(seq 1 10); do
  capture_server_process_metadata "$pid"
  if [[ "${SERVER_CMDLINE_EFFECTIVE:-}" != *"run_load_test.sh"* ]]; then
    break
  fi
  sleep 0.1
done
run_phase="waiting-for-server-ready"
server_ready_seen=false
bot_phase_started=false
harness_exit_reason=""

send_console_command() {
  # Best-effort writes must not abort the harness if the server is already
  # going down or the FIFO is closed. The load run still needs to reach summary
  # generation in that case.
  if [[ "${console_fd_open:-false}" != "true" || ! -e "/proc/$$/fd/3" ]]; then
    console_fd_open=false
    return 0
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    return 0
  fi
  printf '%s\n' "$*" >&3 2>/dev/null || console_fd_open=false
}

single_line_evidence() {
  tr '\0\n\r' '   ' | sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//'
}

capture_server_runtime_evidence() {
  capture_server_process_metadata "${pid:-}"
  if [[ -n "${pid:-}" && -r "/proc/$pid/cmdline" ]]; then
    server_proc_cmdline="$(single_line_evidence < "/proc/$pid/cmdline" || true)"
  fi
  if [[ -n "${pid:-}" ]] && command -v jcmd >/dev/null 2>&1 && kill -0 "$pid" 2>/dev/null; then
    server_jcmd_vm_command_line="$(jcmd "$pid" VM.command_line 2>&1 | single_line_evidence || true)"
    if [[ -n "$server_jcmd_vm_command_line" ]]; then
      server_jcmd_vm_command_line_available=true
    fi
  fi
}

capture_bot_child_memory_snapshot() {
  bot_child_process_count=0
  bot_child_process_missing_count=0
  bot_child_process_rss_kb_total=0
  bot_child_process_rss_kb_max=0
  bot_child_process_pss_kb_total=0
  bot_child_process_pss_kb_max=0

  if [[ -z "${bot_pid_file:-}" || ! -f "${bot_pid_file:-}" ]]; then
    return 0
  fi

  local child_pid rss_kb pss_kb
  while IFS= read -r child_pid; do
    if [[ ! "$child_pid" =~ ^[0-9]+$ ]]; then
      continue
    fi
    bot_child_process_count=$((bot_child_process_count + 1))
    if [[ ! -d "/proc/$child_pid" ]]; then
      bot_child_process_missing_count=$((bot_child_process_missing_count + 1))
      continue
    fi

    if [[ -r "/proc/$child_pid/smaps_rollup" ]]; then
      rss_kb="$(awk '/^Rss:/ {print $2; found=1; exit} END {if (!found) print ""}' "/proc/$child_pid/smaps_rollup" 2>/dev/null || true)"
    else
      rss_kb=""
    fi
    if [[ "$rss_kb" =~ ^[0-9]+$ ]]; then
      bot_child_process_rss_kb_total=$((bot_child_process_rss_kb_total + rss_kb))
      if (( rss_kb > bot_child_process_rss_kb_max )); then
        bot_child_process_rss_kb_max="$rss_kb"
      fi
    else
      rss_kb="$(awk '/^VmRSS:/ {print $2; found=1; exit} END {if (!found) print ""}' "/proc/$child_pid/status" 2>/dev/null || true)"
      if [[ "$rss_kb" =~ ^[0-9]+$ ]]; then
        bot_child_process_rss_kb_total=$((bot_child_process_rss_kb_total + rss_kb))
        if (( rss_kb > bot_child_process_rss_kb_max )); then
          bot_child_process_rss_kb_max="$rss_kb"
        fi
      else
        rss_kb="$(ps -o rss= -p "$child_pid" 2>/dev/null | tr -d '[:space:]' || true)"
        if [[ "$rss_kb" =~ ^[0-9]+$ ]]; then
          bot_child_process_rss_kb_total=$((bot_child_process_rss_kb_total + rss_kb))
          if (( rss_kb > bot_child_process_rss_kb_max )); then
            bot_child_process_rss_kb_max="$rss_kb"
          fi
        else
          bot_child_process_missing_count=$((bot_child_process_missing_count + 1))
          continue
        fi
      fi
    fi

    pss_kb="$(awk '/^Pss:/ {print $2; found=1; exit} END {if (!found) print ""}' "/proc/$child_pid/smaps_rollup" 2>/dev/null || true)"
    if [[ "$pss_kb" =~ ^[0-9]+$ ]]; then
      bot_child_process_pss_kb_total=$((bot_child_process_pss_kb_total + pss_kb))
      if (( pss_kb > bot_child_process_pss_kb_max )); then
        bot_child_process_pss_kb_max="$pss_kb"
      fi
    fi
  done < "$bot_pid_file"
}

load_test_effective_exit_code() {
  local rc="$1"
  local phase="${run_phase:-unknown}"
  local expected_gate="${LOAD_TEST_GATE_REPORT:-$ROOT/reports/load-${LOAD_TEST_LABEL}-gate.txt}"

  harness_exit_reason=none
  if (( rc == 0 )); then
    if [[ "$phase" != "summary" ]]; then
      harness_exit_reason="incomplete_before_summary_phase_${phase//[^A-Za-z0-9_.-]/_}"
      printf '%s\n' 70
      return
    fi
    if [[ ! -s "$SUMMARY" ]]; then
      harness_exit_reason=missing_summary_after_zero_exit
      printf '%s\n' 70
      return
    fi
    if [[ -n "${LOAD_TEST_GATE_PROFILE:-}" && ! -s "$expected_gate" ]]; then
      harness_exit_reason=missing_gate_after_zero_exit
      printf '%s\n' 70
      return
    fi
  fi

  printf '%s\n' "$rc"
}

write_harness_exit() {
  local rc="$1"
  local server_alive_at_cleanup="$2"
  local cleanup_sent_stop="$3"
  mkdir -p "$ROOT/reports" 2>/dev/null || true
  load_bot_resource_state_from_resource_log
  {
    echo "rc=$rc"
    echo "phase=${run_phase:-unknown}"
    echo "harness_exit_reason=${harness_exit_reason:-none}"
    echo "label=$LOAD_TEST_LABEL"
    echo "server_pid=${pid:-}"
    echo "server_effective_pid=${SERVER_EFFECTIVE_PID:-}"
    echo "server_alive_at_cleanup=$server_alive_at_cleanup"
    echo "cleanup_sent_stop=$cleanup_sent_stop"
    echo "server_cmdline_effective=${SERVER_CMDLINE_EFFECTIVE:-}"
    echo "server_jvm_flags_effective=${SERVER_JVM_FLAGS_EFFECTIVE:-}"
    echo "server_proc_cmdline=${server_proc_cmdline:-}"
    echo "server_jcmd_vm_command_line_available=${server_jcmd_vm_command_line_available:-false}"
    echo "server_jcmd_vm_command_line=${server_jcmd_vm_command_line:-}"
    echo "server_ready_seen=${server_ready_seen:-false}"
    echo "bot_phase_started=${bot_phase_started:-false}"
    echo "java_opts_load=$JAVA_OPTS_LOAD"
    echo "bot_pid=${bot_pid:-}"
    echo "bot_pid_file=${bot_pid_file:-}"
    echo "resource_log=$RESOURCE_LOG"
    echo "bot_resource_samples=${BOT_RESOURCE_SAMPLES:-0}"
    echo "bot_process_count_current=${BOT_PROCESS_COUNT_CURRENT:-0}"
    echo "bot_process_count_max=${BOT_PROCESS_COUNT_MAX:-0}"
    echo "bot_rss_mib_current=${BOT_RSS_MIB_CURRENT:-0.0}"
    echo "bot_rss_mib_max=${BOT_RSS_MIB_MAX:-0.0}"
    echo "bot_rss_mib_aggregate_current=${BOT_RSS_MIB_AGGREGATE_CURRENT:-0.0}"
    echo "bot_rss_mib_aggregate_max=${BOT_RSS_MIB_AGGREGATE_MAX:-0.0}"
    echo "bot_pss_mib_available=${BOT_PSS_MIB_AVAILABLE:-false}"
    if [[ -n "${BOT_PSS_MIB_CURRENT:-}" ]]; then
      echo "bot_pss_mib_current=$BOT_PSS_MIB_CURRENT"
    fi
    if [[ -n "${BOT_PSS_MIB_MAX:-}" ]]; then
      echo "bot_pss_mib_max=$BOT_PSS_MIB_MAX"
    fi
    if [[ -n "${BOT_PSS_MIB_AGGREGATE_CURRENT:-}" ]]; then
      echo "bot_pss_mib_aggregate_current=$BOT_PSS_MIB_AGGREGATE_CURRENT"
    fi
    if [[ -n "${BOT_PSS_MIB_AGGREGATE_MAX:-}" ]]; then
      echo "bot_pss_mib_aggregate_max=$BOT_PSS_MIB_AGGREGATE_MAX"
    fi
    echo "bot_child_process_count=${bot_child_process_count:-0}"
    echo "bot_child_process_rss_kb_total=${bot_child_process_rss_kb_total:-0}"
    echo "bot_child_process_rss_kb_max=${bot_child_process_rss_kb_max:-0}"
    echo "bot_child_process_pss_kb_total=${bot_child_process_pss_kb_total:-0}"
    echo "bot_child_process_pss_kb_max=${bot_child_process_pss_kb_max:-0}"
    echo "bot_child_process_missing_count=${bot_child_process_missing_count:-0}"
    echo "serverpulse_influx_pid=${SERVERPULSE_INFLUX_PID:-}"
    echo "serverpulse_influx_log=$SERVERPULSE_INFLUX_LOG"
    echo "log=$LOG"
    echo "bot_log=$BOT_LOG"
    echo "status_json=$STATUS_JSON"
    echo "summary=$SUMMARY"
  } > "$HARNESS_EXIT" 2>/dev/null || true
}

cleanup() {
  local rc=$?
  trap - EXIT
  rc="$(load_test_effective_exit_code "$rc")"
  local server_alive_at_cleanup=false
  local cleanup_sent_stop=false
  if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
    server_alive_at_cleanup=true
  fi
  if [[ -n "${sample_pid:-}" ]] && kill -0 "$sample_pid" 2>/dev/null; then
    kill "$sample_pid" 2>/dev/null || true
    wait "$sample_pid" 2>/dev/null || true
  fi
  if [[ -n "${memory_pid:-}" ]] && kill -0 "$memory_pid" 2>/dev/null; then
    kill "$memory_pid" 2>/dev/null || true
    wait "$memory_pid" 2>/dev/null || true
  fi
  if [[ -n "${resource_pid:-}" ]] && kill -0 "$resource_pid" 2>/dev/null; then
    kill "$resource_pid" 2>/dev/null || true
    wait "$resource_pid" 2>/dev/null || true
  fi
  if [[ -n "${gate_abort_pid:-}" ]] && kill -0 "$gate_abort_pid" 2>/dev/null; then
    kill "$gate_abort_pid" 2>/dev/null || true
    wait "$gate_abort_pid" 2>/dev/null || true
  fi
  if [[ -n "${host_abort_pid:-}" ]] && kill -0 "$host_abort_pid" 2>/dev/null; then
    kill "$host_abort_pid" 2>/dev/null || true
    wait "$host_abort_pid" 2>/dev/null || true
  fi
  if [[ -n "${watchdog_pid:-}" ]] && kill -0 "$watchdog_pid" 2>/dev/null; then
    kill "$watchdog_pid" 2>/dev/null || true
    wait "$watchdog_pid" 2>/dev/null || true
  fi
  if [[ -n "${bot_pid:-}" ]] && kill -0 "$bot_pid" 2>/dev/null; then
    kill "$bot_pid" 2>/dev/null || true
    wait "$bot_pid" 2>/dev/null || true
  fi
  if [[ -n "${bot_gate_pid:-}" ]] && kill -0 "$bot_gate_pid" 2>/dev/null; then
    kill "$bot_gate_pid" 2>/dev/null || true
    wait "$bot_gate_pid" 2>/dev/null || true
  fi
  stop_serverpulse_influx_mock
  if [[ -n "${bot_pid_file:-}" && -f "${bot_pid_file:-}" ]]; then
    while IFS= read -r child_pid; do
      if [[ "$child_pid" =~ ^[0-9]+$ ]]; then
        kill "$child_pid" 2>/dev/null || true
      fi
    done < "$bot_pid_file"
  fi
  if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
    cleanup_sent_stop=true
    send_console_command stop
    sleep 5
    kill "$pid" 2>/dev/null || true
  fi
  write_harness_exit "$rc" "$server_alive_at_cleanup" "$cleanup_sent_stop"
  exit "$rc"
}

finalize_harness_exit() {
  local rc=$?
  trap - EXIT
  rc="$(load_test_effective_exit_code "$rc")"
  write_harness_exit "$rc" false false
  exit "$rc"
}
trap cleanup EXIT

SERVER_READY_REGEX="$(load_test_server_ready_regex)"
server_ready_start_seconds="$SECONDS"
server_ready_log_grace_seconds="${LOAD_TEST_SERVER_READY_LOG_GRACE_SECONDS:-15}"
deadline=$((SECONDS + LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if rg -q "Failed to bind to port|Encountered an unexpected exception|UnsupportedClassVersionError|Could not load plugin|Failed to start|OutOfMemoryError" "$LOG"; then
    tail -n 200 "$LOG" >&2
    exit 1
  fi
  if [[ -n "${pid:-}" ]] && ! kill -0 "$pid" 2>/dev/null; then
    printf 'server_ready_wait_server_exited\n' > "$DIAGNOSTIC_REASON"
    tail -n 220 "$LOG" >&2 || true
    exit 1
  fi
  if (( SECONDS - server_ready_start_seconds >= server_ready_log_grace_seconds )) && [[ ! -s "$LOG" ]]; then
    printf 'server_ready_wait_log_missing\n' > "$DIAGNOSTIC_REASON"
    exit 1
  fi
  sleep 1
done
if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  printf 'server_ready_wait_timeout\n' > "$DIAGNOSTIC_REASON"
  tail -n 220 "$LOG" >&2
  exit 1
fi

server_ready_seen=true
capture_server_runtime_evidence
run_phase="status-ping"
status_tmp="$STATUS_JSON.tmp"
rm -f "$status_tmp"
for attempt in $(seq 1 12); do
  if python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$PORT" 773 10 > "$status_tmp"; then
    mv "$status_tmp" "$STATUS_JSON"
    break
  fi
  if (( attempt == 12 )); then
    tail -n 220 "$LOG" >&2
    exit 1
  fi
  sleep 2
done
if [[ "$LOAD_TEST_SCENARIO" == "block" || "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
  run_phase="server-setup"
  send_console_command gamerule doMobSpawning false
  send_console_command gamerule doDaylightCycle false
  send_console_command gamerule randomTickSpeed 0
  send_console_command gamerule sendCommandFeedback false
  if [[ "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
    send_console_command gamerule fallDamage false
    send_console_command gamerule keepInventory true
    send_console_command gamerule doImmediateRespawn true
  fi
  if [[ "$LOAD_TEST_ARENA_PRELOAD" == "true" ]]; then
    run_phase="arena-preload"
    send_console_command compatprobe arena-preload \
      "$BOT_COUNT" \
      "$BLOCK_ARENA_CENTER_X" \
      "$BLOCK_ARENA_TARGET_Y" \
      "$BLOCK_ARENA_CENTER_Z" \
      "$BLOCK_ARENA_SPACING" \
      "$BLOCK_ARENA_COLUMNS" \
      "$BLOCK_ARENA_ITEM" \
      "$LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS" \
      "$LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT"
    arena_preload_ack_deadline=$((SECONDS + LOAD_TEST_ARENA_PRELOAD_ACK_TIMEOUT_SECONDS))
    while (( SECONDS < arena_preload_ack_deadline )); do
      if rg -q "COMPAT_PROBE arena_preload scheduled .* expectedPlayers=$BOT_COUNT " "$LOG"; then
        break
      fi
      if rg -q "COMPAT_PROBE arena_preload error=|Unknown or incomplete command" "$LOG"; then
        tail -n 220 "$LOG" >&2
        exit 1
      fi
      sleep 1
    done
    if ! rg -q "COMPAT_PROBE arena_preload scheduled .* expectedPlayers=$BOT_COUNT " "$LOG"; then
      tail -n 220 "$LOG" >&2
      exit 1
    fi
    arena_preload_deadline=$((SECONDS + LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS))
    while (( SECONDS < arena_preload_deadline )); do
      if rg -q "COMPAT_PROBE arena_preload complete .* expectedPlayers=$BOT_COUNT .* failed=0" "$LOG"; then
        break
      fi
      if rg -q "COMPAT_PROBE arena_preload error=|COMPAT_PROBE arena_preload complete .* failed=[1-9][0-9]*|Unknown or incomplete command" "$LOG"; then
        tail -n 220 "$LOG" >&2
        exit 1
      fi
      sleep 2
    done
    if ! rg -q "COMPAT_PROBE arena_preload complete .* expectedPlayers=$BOT_COUNT .* failed=0" "$LOG"; then
      tail -n 220 "$LOG" >&2
      exit 1
    fi
  else
    send_console_command compatprobe arena LoadBot \
      "$BOT_COUNT" \
      "$BLOCK_ARENA_CENTER_X" \
      "$BLOCK_ARENA_TARGET_Y" \
      "$BLOCK_ARENA_CENTER_Z" \
      "$BLOCK_ARENA_SPACING" \
      "$BLOCK_ARENA_COLUMNS" \
      "$BLOCK_ARENA_ITEM"
  fi
else
  run_phase="server-setup"
  send_console_command gamerule spectatorsGenerateChunks true
fi
send_console_command difficulty normal
if (( LOAD_TEST_MOB_STORM_COUNT > 0 )); then
  send_console_command compatprobe mobstorm \
    "$LOAD_TEST_MOB_STORM_ENTITY" \
    "$LOAD_TEST_MOB_STORM_COUNT" \
    "$LOAD_TEST_MOB_STORM_CENTER_X" \
    "$LOAD_TEST_MOB_STORM_Y" \
    "$LOAD_TEST_MOB_STORM_CENTER_Z" \
    "$LOAD_TEST_MOB_STORM_SPACING" \
    "$LOAD_TEST_MOB_STORM_COLUMNS"
fi

rm -rf "$JSTACK_DIR"
mkdir -p "$JSTACK_DIR"
(
  last_watchdog_count=0
  while kill -0 "$pid" 2>/dev/null; do
    watchdog_count="$(rg -c "DO NOT REPORT THIS TO PAPER" "$LOG" 2>/dev/null || true)"
    if [[ "$watchdog_count" =~ ^[0-9]+$ ]] && (( watchdog_count > last_watchdog_count )); then
      dump="$JSTACK_DIR/thread-print-${watchdog_count}.txt"
      if command -v jcmd >/dev/null 2>&1; then
        jcmd "$pid" Thread.print > "$dump" 2>&1 || true
      elif command -v jstack >/dev/null 2>&1; then
        jstack "$pid" > "$dump" 2>&1 || true
      else
        printf 'No jcmd or jstack available\n' > "$dump"
      fi
      last_watchdog_count="$watchdog_count"
    fi
    sleep 2
  done
) &
watchdog_pid=$!

sample_pid=""
if (( LOAD_TEST_THREAD_SAMPLES > 0 )); then
  (
    sleep "$LOAD_TEST_THREAD_SAMPLE_START_AFTER_SECONDS"
    sample=0
    while kill -0 "$pid" 2>/dev/null && (( sample < LOAD_TEST_THREAD_SAMPLES )); do
      sample=$((sample + 1))
      dump="$JSTACK_DIR/thread-sample-$(printf '%04d' "$sample").txt"
      if command -v jcmd >/dev/null 2>&1; then
        jcmd "$pid" Thread.print > "$dump" 2>&1 || true
      elif command -v jstack >/dev/null 2>&1; then
        jstack "$pid" > "$dump" 2>&1 || true
      else
        printf 'No jcmd or jstack available\n' > "$dump"
      fi
      sleep "$LOAD_TEST_THREAD_SAMPLE_INTERVAL_SECONDS"
    done
  ) &
  sample_pid=$!
fi

{
  echo "ts_ms,pid_cpu,pid_rss_kb,system_load1,system_mem_available_kb,host_cpu_total,host_cpu_idle,host_cpu_iowait,host_cpu_steal,bot_process_count,bot_rss_kb_max,bot_rss_kb_total,bot_pss_kb_max,bot_pss_kb_total,bot_pss_available"
  while kill -0 "$pid" 2>/dev/null; do
    python3 - "$pid" "$bot_pid_file" <<'PY'
import os
import pathlib
import subprocess
import sys
import time

pid = sys.argv[1]
bot_pid_file = pathlib.Path(sys.argv[2])
try:
    ps = subprocess.check_output(["ps", "-o", "%cpu=,rss=", "-p", pid], text=True).strip().split()
    cpu, rss = ps[0], ps[1]
except Exception:
    cpu, rss = "", ""
load1 = os.getloadavg()[0]
available = ""
with open("/proc/meminfo", "r", encoding="utf-8") as handle:
    for line in handle:
        if line.startswith("MemAvailable:"):
            available = line.split()[1]
            break
host_total = host_idle = host_iowait = host_steal = ""
try:
    with open("/proc/stat", "r", encoding="utf-8") as handle:
        fields = handle.readline().split()
    if fields and fields[0] == "cpu":
        values = [int(value) for value in fields[1:]]
        if len(values) >= 8:
            host_total = str(sum(values))
            host_idle = str(values[3])
            host_iowait = str(values[4])
            host_steal = str(values[7])
except Exception:
    pass

def read_bot_pids():
    try:
        lines = bot_pid_file.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return []
    except Exception:
        return []
    pids = []
    seen = set()
    for line in lines:
        value = line.strip()
        if not value.isdigit():
            continue
        bot_pid = int(value)
        if bot_pid in seen:
            continue
        seen.add(bot_pid)
        try:
            os.kill(bot_pid, 0)
        except OSError:
            continue
        pids.append(bot_pid)
    return pids

def read_smaps_rollup(bot_pid):
    rss_kb = None
    pss_kb = None
    try:
        with open(f"/proc/{bot_pid}/smaps_rollup", "r", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if line.startswith("Rss:"):
                    rss_kb = int(line.split()[1])
                elif line.startswith("Pss:"):
                    pss_kb = int(line.split()[1])
                if rss_kb is not None and pss_kb is not None:
                    break
    except Exception:
        pass
    return rss_kb, pss_kb

def read_ps_rss(bot_pid):
    try:
        parts = subprocess.check_output(
            ["ps", "-o", "rss=", "-p", str(bot_pid)],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip().split()
        return int(parts[0]) if parts else None
    except Exception:
        return None

bot_rss_values = []
bot_pss_values = []
bot_pss_complete = True
for bot_child_pid in read_bot_pids():
    bot_rss_kb, bot_pss_kb = read_smaps_rollup(bot_child_pid)
    if bot_rss_kb is None:
        bot_rss_kb = read_ps_rss(bot_child_pid)
    if bot_rss_kb is None:
        continue
    bot_rss_values.append(bot_rss_kb)
    if bot_pss_kb is None:
        bot_pss_complete = False
    else:
        bot_pss_values.append(bot_pss_kb)

bot_process_count = len(bot_rss_values)
bot_rss_kb_max = bot_rss_kb_total = ""
bot_pss_kb_max = bot_pss_kb_total = ""
bot_pss_available = "false"
if bot_rss_values:
    bot_rss_kb_max = str(max(bot_rss_values))
    bot_rss_kb_total = str(sum(bot_rss_values))
    if bot_pss_complete and len(bot_pss_values) == len(bot_rss_values):
        bot_pss_kb_max = str(max(bot_pss_values))
        bot_pss_kb_total = str(sum(bot_pss_values))
        bot_pss_available = "true"

print(
    f"{int(time.time() * 1000)},{cpu},{rss},{load1:.2f},{available},"
    f"{host_total},{host_idle},{host_iowait},{host_steal},"
    f"{bot_process_count},{bot_rss_kb_max},{bot_rss_kb_total},"
    f"{bot_pss_kb_max},{bot_pss_kb_total},{bot_pss_available}",
    flush=True,
)
PY
    sleep "$LOAD_TEST_BOT_RESOURCE_SAMPLE_INTERVAL_SECONDS"
  done
} > "$RESOURCE_LOG" &
resource_pid=$!

memory_pid=""
if [[ "$LOAD_TEST_MEMORY_SNAPSHOTS" == "true" ]]; then
  mkdir -p "$MEMORY_SNAPSHOT_DIR"
  (
    peak_rss_kb=0
    snapshot_index=0
    while kill -0 "$pid" 2>/dev/null; do
      rss_kb="$(ps -o rss= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)"
      if [[ "$rss_kb" =~ ^[0-9]+$ ]]; then
        next_threshold=$((peak_rss_kb + LOAD_TEST_MEMORY_SNAPSHOT_MIN_RSS_DELTA_KB))
        if (( peak_rss_kb == 0 || rss_kb >= next_threshold )); then
          peak_rss_kb="$rss_kb"
          snapshot_index=$((snapshot_index + 1))
          stamp="$(date -u +%Y%m%dT%H%M%SZ)"
          snapshot="$MEMORY_SNAPSHOT_DIR/peak-$(printf '%04d' "$snapshot_index")-${rss_kb}kb-${stamp}.txt"
          tmp_snapshot="${snapshot}.tmp"
          {
            echo "memory_snapshot=true"
            echo "label=$LOAD_TEST_LABEL"
            echo "ts_utc=$stamp"
            echo "pid=$pid"
            echo "rss_kb=$rss_kb"
            echo "rss_mib=$(python3 - "$rss_kb" <<'PY'
import sys
print(f"{int(sys.argv[1]) / 1024:.1f}")
PY
)"
            echo
            echo "[proc_status]"
            cat "/proc/$pid/status" 2>/dev/null || true
            echo
            echo "[smaps_rollup]"
            cat "/proc/$pid/smaps_rollup" 2>/dev/null || true
            echo
            echo "[jcmd_gc_heap_info]"
            if command -v jcmd >/dev/null 2>&1; then
              jcmd "$pid" GC.heap_info 2>&1 || true
            else
              echo "jcmd_unavailable=true"
            fi
            echo
            echo "[jcmd_vm_native_memory_summary]"
            if command -v jcmd >/dev/null 2>&1; then
              jcmd "$pid" VM.native_memory summary scale=MB 2>&1 || true
            else
              echo "jcmd_unavailable=true"
            fi
          } > "$tmp_snapshot" 2>&1
          mv "$tmp_snapshot" "$snapshot"
          cp "$snapshot" "$MEMORY_SNAPSHOT_DIR/peak-latest.txt"
        fi
      fi
      sleep "$LOAD_TEST_MEMORY_SNAPSHOT_INTERVAL_SECONDS"
    done
  ) &
  memory_pid=$!
fi

bot_pids=()
bot_shard_logs=()
bot_pid_file="$RUN/bot-pids.txt"
bot_gate_pid=""
rm -f "$bot_pid_file"
if [[ -n "$BOT_SWARM_SHARED_GATE_FILE" ]]; then
  rm -f "$BOT_SWARM_SHARED_GATE_FILE"
fi
run_phase="bot-launch"
if (( BOT_SWARM_SHARDS > 1 )); then
  rm -rf "$BOT_LOG_DIR"
  mkdir -p "$BOT_LOG_DIR"
  shard_base=$(( BOT_COUNT / BOT_SWARM_SHARDS ))
  shard_extra=$(( BOT_COUNT % BOT_SWARM_SHARDS ))
  start_index=0
  for (( shard = 0; shard < BOT_SWARM_SHARDS; shard++ )); do
    shard_count="$shard_base"
    if (( shard < shard_extra )); then
      shard_count=$(( shard_count + 1 ))
    fi
    shard_log="$BOT_LOG_DIR/shard-${shard}.log"
    shard_ready_min_count="$BOT_ACTION_READY_MIN_COUNT"
    if [[ "$BOT_ACTION_START_MODE" == "all-ready" ]]; then
      shard_ready_min_count="$shard_count"
    elif [[ "$BOT_ACTION_START_MODE" == "ready-count" ]]; then
      shard_ready_min_count=$(( (BOT_ACTION_READY_MIN_COUNT * shard_count + BOT_COUNT - 1) / BOT_COUNT ))
      if (( shard_ready_min_count < 1 )); then
        shard_ready_min_count=1
      fi
    fi
    bot_shard_logs+=("$shard_log")
    "$ROOT/scripts/mc_bot_swarm.sh" \
      --host=127.0.0.1 \
      --port="$PORT" \
      --count="$shard_count" \
      --total-count="$BOT_COUNT" \
      --start-index="$start_index" \
      --duration-ms="$((DURATION_SECONDS * 1000))" \
      --min-run-after-action-open-ms="$BOT_MIN_RUN_AFTER_ACTION_OPEN_MS" \
      --max-duration-ms="$BOT_MAX_DURATION_MS" \
      --ramp-ms="$((BOT_EFFECTIVE_RAMP_SECONDS * 1000))" \
      --ramp-shard-index="$shard" \
      --ramp-shard-count="$BOT_SWARM_SHARDS" \
      --mode="$LOAD_TEST_SCENARIO" \
      --action-start-mode="$BOT_ACTION_START_MODE" \
      --action-start-after-ms="$BOT_ACTION_START_AFTER_MS" \
      --action-ready-settle-ms="$BOT_ACTION_READY_SETTLE_MS" \
      --action-ready-requires-block-armed="$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" \
      --action-ready-min-count="$shard_ready_min_count" \
      --action-ready-min-fraction="$BOT_ACTION_READY_MIN_FRACTION" \
      --check-timeout-interval-ms="$BOT_CHECK_TIMEOUT_INTERVAL_MS" \
      --strict-failures="$BOT_STRICT_FAILURES" \
      --global-action-start-after-ms="$BOT_GLOBAL_ACTION_START_AFTER_MS" \
      --shared-action-gate-file="$BOT_SWARM_SHARED_GATE_FILE" \
      --move-interval-ms="$BOT_MOVE_INTERVAL_MS" \
      --dephase-actions="$BOT_DEPHASE_ACTIONS" \
      --move-scheduler-tick-ms="$BOT_MOVE_SCHEDULER_TICK_MS" \
      --block-scheduler-tick-ms="$BOT_BLOCK_SCHEDULER_TICK_MS" \
      --mixed-scheduler-tick-ms="$BOT_MIXED_SCHEDULER_TICK_MS" \
      --send-stationary-positions="$BOT_SEND_STATIONARY_POSITIONS" \
      --block-movement-mode="$BOT_BLOCK_MOVEMENT_MODE" \
      --mixed-action-interval-ms="$BOT_MIXED_ACTION_INTERVAL_MS" \
      --mixed-command-interval-ms="$BOT_MIXED_COMMAND_INTERVAL_MS" \
      --mixed-use-entity-attacks="$BOT_MIXED_USE_ENTITY_ATTACKS" \
      --block-action-interval-ms="$BOT_BLOCK_ACTION_INTERVAL_MS" \
      --speed="$BOT_SPEED" \
      --altitude="$BOT_ALTITUDE" \
      --block-item="$BLOCK_ARENA_ITEM" \
      --block-arena-center-x="$BLOCK_ARENA_CENTER_X" \
      --block-arena-center-z="$BLOCK_ARENA_CENTER_Z" \
      --block-arena-target-y="$BLOCK_ARENA_TARGET_Y" \
      --block-arena-spacing="$BLOCK_ARENA_SPACING" \
      --block-arena-columns="$BLOCK_ARENA_COLUMNS" \
      --name=LoadBot \
      > "$shard_log" 2>&1 &
    bot_pids+=("$!")
    start_index=$(( start_index + shard_count ))
  done
  bot_phase_started=true
  printf '%s\n' "${bot_pids[@]}" > "$bot_pid_file"
  if [[ -n "$BOT_SWARM_SHARED_GATE_FILE" ]]; then
    python3 "$ROOT/scripts/coordinate_swarm_gate.py" \
      --count "$BOT_COUNT" \
      --action-mode "$BOT_ACTION_START_MODE" \
      --ready-min-count "$BOT_ACTION_READY_MIN_COUNT" \
      --ready-min-fraction "$BOT_ACTION_READY_MIN_FRACTION" \
      --settle-ms "$BOT_ACTION_READY_SETTLE_MS" \
      --requires-block-armed "$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" \
      --gate-file "$BOT_SWARM_SHARED_GATE_FILE" \
      --timeout-ms "$BOT_MAX_DURATION_MS" \
      "${bot_shard_logs[@]}" \
      > "$BOT_LOG_DIR/gate-coordinator.log" 2>&1 &
    bot_gate_pid=$!
  fi
  (
    mapfile -t monitor_pids < "$bot_pid_file"
    while :; do
      alive=false
      for child in "${monitor_pids[@]}"; do
        if kill -0 "$child" 2>/dev/null; then
          alive=true
          break
        fi
      done
      if [[ "$alive" == "false" ]]; then
        break
      fi
      sleep 1
    done
    exit 0
  ) &
  bot_pid=$!
else
  "$ROOT/scripts/mc_bot_swarm.sh" \
    --host=127.0.0.1 \
    --port="$PORT" \
    --count="$BOT_COUNT" \
    --total-count="$BOT_COUNT" \
    --start-index=0 \
    --duration-ms="$((DURATION_SECONDS * 1000))" \
    --min-run-after-action-open-ms="$BOT_MIN_RUN_AFTER_ACTION_OPEN_MS" \
    --max-duration-ms="$BOT_MAX_DURATION_MS" \
    --ramp-ms="$((BOT_EFFECTIVE_RAMP_SECONDS * 1000))" \
    --ramp-shard-index=0 \
    --ramp-shard-count=1 \
    --mode="$LOAD_TEST_SCENARIO" \
    --action-start-mode="$BOT_ACTION_START_MODE" \
    --action-start-after-ms="$BOT_ACTION_START_AFTER_MS" \
    --action-ready-settle-ms="$BOT_ACTION_READY_SETTLE_MS" \
    --action-ready-requires-block-armed="$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" \
    --action-ready-min-count="$BOT_ACTION_READY_MIN_COUNT" \
    --action-ready-min-fraction="$BOT_ACTION_READY_MIN_FRACTION" \
    --check-timeout-interval-ms="$BOT_CHECK_TIMEOUT_INTERVAL_MS" \
    --strict-failures="$BOT_STRICT_FAILURES" \
    --global-action-start-after-ms="$BOT_GLOBAL_ACTION_START_AFTER_MS" \
    --shared-action-gate-file="$BOT_SWARM_SHARED_GATE_FILE" \
    --move-interval-ms="$BOT_MOVE_INTERVAL_MS" \
    --dephase-actions="$BOT_DEPHASE_ACTIONS" \
    --move-scheduler-tick-ms="$BOT_MOVE_SCHEDULER_TICK_MS" \
    --block-scheduler-tick-ms="$BOT_BLOCK_SCHEDULER_TICK_MS" \
    --mixed-scheduler-tick-ms="$BOT_MIXED_SCHEDULER_TICK_MS" \
    --send-stationary-positions="$BOT_SEND_STATIONARY_POSITIONS" \
    --block-movement-mode="$BOT_BLOCK_MOVEMENT_MODE" \
    --mixed-action-interval-ms="$BOT_MIXED_ACTION_INTERVAL_MS" \
    --mixed-command-interval-ms="$BOT_MIXED_COMMAND_INTERVAL_MS" \
    --mixed-use-entity-attacks="$BOT_MIXED_USE_ENTITY_ATTACKS" \
    --block-action-interval-ms="$BOT_BLOCK_ACTION_INTERVAL_MS" \
    --speed="$BOT_SPEED" \
    --altitude="$BOT_ALTITUDE" \
    --block-item="$BLOCK_ARENA_ITEM" \
    --block-arena-center-x="$BLOCK_ARENA_CENTER_X" \
    --block-arena-center-z="$BLOCK_ARENA_CENTER_Z" \
    --block-arena-target-y="$BLOCK_ARENA_TARGET_Y" \
    --block-arena-spacing="$BLOCK_ARENA_SPACING" \
    --block-arena-columns="$BLOCK_ARENA_COLUMNS" \
    --name=LoadBot \
    > "$BOT_LOG" 2>&1 &
  bot_pid=$!
  bot_pids=("$bot_pid")
  bot_shard_logs=("$BOT_LOG")
  bot_phase_started=true
  printf '%s\n' "$bot_pid" > "$bot_pid_file"
fi

run_phase="bot-running"
gate_abort_pid=""
host_abort_pid=""
rm -f "$EARLY_ABORT_REASON"
if [[ -n "$LOAD_TEST_ABORT_ON_TPS1_BELOW" || -n "$LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE" ]]; then
  (
    set +e
    python3 - "$LOG" "$EARLY_ABORT_REASON" "$bot_pid" "$bot_pid_file" "$LOAD_TEST_ABORT_ON_TPS1_BELOW" "$LOAD_TEST_ABORT_ON_AVG_TICK_MS_ABOVE" <<'PY'
import pathlib
import os
import signal
import re
import sys
import time

log_path = pathlib.Path(sys.argv[1])
reason_path = pathlib.Path(sys.argv[2])
bot_pid = int(sys.argv[3])
bot_pid_file = pathlib.Path(sys.argv[4])
tps_min = float(sys.argv[5]) if sys.argv[5] else None
tick_max = float(sys.argv[6]) if sys.argv[6] else None
metric_re = re.compile(
    r"COMPAT_PROBE metrics online=(\d+) loadedChunks=(\d+) tps1=([0-9.]+).* avgTickMs=([0-9.]+)"
)

def terminate_bots():
    pids = [bot_pid]
    try:
        pids.extend(int(line.strip()) for line in bot_pid_file.read_text().splitlines() if line.strip())
    except Exception:
        pass
    for pid in dict.fromkeys(pids):
        try:
            os.kill(pid, signal.SIGTERM)
        except Exception:
            pass

position = 0
while True:
    try:
        with log_path.open(encoding="utf-8", errors="replace") as handle:
            handle.seek(position)
            for line in handle:
                match = metric_re.search(line)
                if not match:
                    continue
                online = int(match.group(1))
                loaded = int(match.group(2))
                tps1 = float(match.group(3))
                avg_tick = float(match.group(4))
                if tps_min is not None and tps1 < tps_min:
                    reason_path.write_text(
                        f"tps1_below_threshold online={online} loadedChunks={loaded} "
                        f"tps1={tps1:.2f} threshold={tps_min:.2f}\n",
                        encoding="utf-8",
                    )
                    terminate_bots()
                    raise SystemExit(42)
                if tick_max is not None and avg_tick > tick_max:
                    reason_path.write_text(
                        f"avg_tick_above_threshold online={online} loadedChunks={loaded} "
                        f"avgTickMs={avg_tick:.2f} threshold={tick_max:.2f}\n",
                        encoding="utf-8",
                    )
                    terminate_bots()
                    raise SystemExit(42)
            position = handle.tell()
    except FileNotFoundError:
        pass
    time.sleep(1.0)
PY
  ) &
  gate_abort_pid=$!
fi
if [[ "${LOAD_TEST_ALLOW_BUSY_HOST:-false}" != "true" && "$LOAD_TEST_ABORT_ON_HOST_CONTENTION" == "true" ]]; then
  (
    set +e
    python3 "$ROOT/scripts/watch_load_host_contention.py" \
      --reason "$EARLY_ABORT_REASON" \
      --bot-pid "$bot_pid" \
      --bot-pid-file "$bot_pid_file" \
      --max-load-per-cpu "$LOAD_TEST_HOST_CONTENTION_MAX_LOAD_PER_CPU" \
      --max-steal-percent "$LOAD_TEST_HOST_CONTENTION_MAX_STEAL_PERCENT" \
      --max-iowait-percent "$LOAD_TEST_HOST_CONTENTION_MAX_IOWAIT_PERCENT" \
      --interval "$LOAD_TEST_HOST_CONTENTION_SAMPLE_INTERVAL_SECONDS" \
      --bad-samples "$LOAD_TEST_HOST_CONTENTION_BAD_SAMPLES"
  ) &
  host_abort_pid=$!
fi

if [[ "$LOAD_TEST_SCENARIO" == "block" || "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
  # Block and mixed-gameplay runs need to seat late joiners until every bot observes its arena target.
  arena_deadline=$((SECONDS + DURATION_SECONDS + 15))
else
  arena_deadline=$((SECONDS + BOT_RAMP_SECONDS + 12))
fi

read_block_progress() {
  local log_paths=("$BOT_LOG")
  if (( BOT_SWARM_SHARDS > 1 )); then
    shopt -s nullglob
    log_paths=("$BOT_LOG_DIR"/shard-*.log)
    shopt -u nullglob
  fi
  python3 - "${log_paths[@]}" <<'PY'
import re
import sys

armed_total = 0
primed_total = 0
found = False
for path in sys.argv[1:]:
    line = None
    try:
        handle = open(path, encoding="utf-8", errors="replace")
    except FileNotFoundError:
        continue
    with handle:
        for raw in handle:
            if "swarm_metrics mode=" in raw and "blockArmed=" in raw:
                line = raw
    if not line:
        continue
    match = re.search(r"blockArmed=(\d+).*blockPrimed=(\d+)", line)
    if not match:
        continue
    armed_total += int(match.group(1))
    primed_total += int(match.group(2))
    found = True

if not found:
    print("-1 -1")
    raise SystemExit(0)
print(f"{armed_total} {primed_total}")
PY
}

while kill -0 "$bot_pid" 2>/dev/null && (( SECONDS < arena_deadline )); do
    if [[ "$LOAD_TEST_SCENARIO" == "block" || "$LOAD_TEST_SCENARIO" == "mixed-gameplay" ]]; then
      read -r block_armed block_primed < <(read_block_progress)
      if [[ "$block_armed" =~ ^[0-9]+$ ]] && (( block_armed >= BOT_COUNT )); then
        break
      fi
      send_console_command compatprobe arena LoadBot \
        "$BOT_COUNT" \
        "$BLOCK_ARENA_CENTER_X" \
        "$BLOCK_ARENA_TARGET_Y" \
        "$BLOCK_ARENA_CENTER_Z" \
        "$BLOCK_ARENA_SPACING" \
        "$BLOCK_ARENA_COLUMNS" \
        "$BLOCK_ARENA_ITEM"
    else
      send_console_command compatprobe spectator LoadBot "$BOT_ALTITUDE"
    fi
    sleep 5
  done

capture_bot_child_memory_snapshot

set +e
if (( BOT_SWARM_SHARDS > 1 )); then
  bot_exit=0
  for child_pid in "${bot_pids[@]}"; do
    wait "$child_pid"
    child_exit=$?
    if (( child_exit != 0 && bot_exit == 0 )); then
      bot_exit="$child_exit"
    fi
  done
  wait "$bot_pid" 2>/dev/null || true
  python3 "$ROOT/scripts/aggregate_swarm_logs.py" \
    --count "$BOT_COUNT" \
    --mode "$LOAD_TEST_SCENARIO" \
    --action-mode "$BOT_ACTION_START_MODE" \
    --settle-ms "$BOT_ACTION_READY_SETTLE_MS" \
    "${bot_shard_logs[@]}" > "$BOT_LOG"
  aggregate_exit=$?
  if (( aggregate_exit != 0 && bot_exit == 0 )); then
    bot_exit="$aggregate_exit"
  fi
else
  wait "$bot_pid"
  bot_exit=$?
fi
set -e
if [[ -n "$gate_abort_pid" ]] && kill -0 "$gate_abort_pid" 2>/dev/null; then
  kill "$gate_abort_pid" 2>/dev/null || true
  wait "$gate_abort_pid" 2>/dev/null || true
fi
if [[ -n "$host_abort_pid" ]] && kill -0 "$host_abort_pid" 2>/dev/null; then
  kill "$host_abort_pid" 2>/dev/null || true
  wait "$host_abort_pid" 2>/dev/null || true
fi
if [[ -n "${bot_gate_pid:-}" ]] && kill -0 "$bot_gate_pid" 2>/dev/null; then
  kill "$bot_gate_pid" 2>/dev/null || true
  wait "$bot_gate_pid" 2>/dev/null || true
fi
if [[ -s "$EARLY_ABORT_REASON" ]]; then
  bot_exit=75
fi
kill "$watchdog_pid" 2>/dev/null || true
wait "$watchdog_pid" 2>/dev/null || true
if [[ -n "$sample_pid" ]]; then
  kill "$sample_pid" 2>/dev/null || true
  wait "$sample_pid" 2>/dev/null || true
fi
kill "$resource_pid" 2>/dev/null || true
wait "$resource_pid" 2>/dev/null || true
if [[ -n "$memory_pid" ]]; then
  kill "$memory_pid" 2>/dev/null || true
  wait "$memory_pid" 2>/dev/null || true
fi

run_phase="server-shutdown"
send_console_command tps
send_console_command mspt
send_console_command compatprobe
send_console_command save-all
sleep 10
send_console_command stop
console_fd_open=false
exec 3>&-

deadline=$((SECONDS + 180))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  tail -n 240 "$LOG" >&2
  exit 1
fi
stop_serverpulse_influx_mock
trap finalize_harness_exit EXIT

run_phase="summary"
export SERVER_EFFECTIVE_PID
export SERVER_CMDLINE_EFFECTIVE
export SERVER_JVM_FLAGS_EFFECTIVE
export server_proc_cmdline
export server_jcmd_vm_command_line_available
export server_jcmd_vm_command_line
export BOT_PID_FILE
export BOT_RESOURCE_SAMPLES
export BOT_PROCESS_COUNT_CURRENT
export BOT_PROCESS_COUNT_MAX
export BOT_RSS_MIB_CURRENT
export BOT_RSS_MIB_MAX
export BOT_RSS_MIB_AGGREGATE_CURRENT
export BOT_RSS_MIB_AGGREGATE_MAX
export BOT_PSS_MIB_AVAILABLE
export BOT_PSS_MIB_CURRENT
export BOT_PSS_MIB_MAX
export BOT_PSS_MIB_AGGREGATE_CURRENT
export BOT_PSS_MIB_AGGREGATE_MAX
export bot_child_process_count
export bot_child_process_rss_kb_total
export bot_child_process_rss_kb_max
export bot_child_process_pss_kb_total
export bot_child_process_pss_kb_max
export bot_child_process_missing_count
python3 - "$LOG" "$BOT_LOG" "$RESOURCE_LOG" "$SUMMARY" "$BOT_COUNT" "$VIEW_DISTANCE" "$SIMULATION_DISTANCE" "$bot_exit" "$PAPER_PLAYER_MAX_CONCURRENT_LOADS" "$PAPER_PLAYER_MAX_CONCURRENT_GENS" "$PAPER_PLAYER_MAX_LOAD_RATE" "$PAPER_PLAYER_MAX_GEN_RATE" "$PAPER_PLAYER_MAX_SEND_RATE" "$PAPER_CHUNK_WORKER_THREADS" "$PAPER_CHUNK_IO_THREADS" "$PAPER_MISC_MAX_JOINS_PER_TICK" "$PAPER_PREVENT_MOVING_INTO_UNLOADED_CHUNKS" "$PAPER_UPDATE_PATHFINDING_ON_BLOCK_UPDATE" "$PAPER_TICK_RATE_CONTAINER_UPDATE" "$JSTACK_DIR" "$LOAD_TEST_SCENARIO" "$BLOCK_ARENA_CENTER_X" "$BLOCK_ARENA_CENTER_Z" "$BLOCK_ARENA_TARGET_Y" "$BLOCK_ARENA_SPACING" "$BLOCK_ARENA_COLUMNS" "$BLOCK_ARENA_ITEM" "$LOAD_TEST_ARENA_PRELOAD" "$LOAD_TEST_ARENA_PRELOAD_RADIUS_CHUNKS" "$LOAD_TEST_ARENA_PRELOAD_MAX_IN_FLIGHT" "$LOAD_TEST_ARENA_PRELOAD_TIMEOUT_SECONDS" "$LOAD_TEST_GAMEMODE" "$BOT_SPEED" "$BOT_MOVE_INTERVAL_MS" "$BOT_DEPHASE_ACTIONS" "$BOT_MOVE_SCHEDULER_TICK_MS" "$BOT_BLOCK_SCHEDULER_TICK_MS" "$BOT_MIXED_SCHEDULER_TICK_MS" "$BOT_SEND_STATIONARY_POSITIONS" "$BOT_BLOCK_MOVEMENT_MODE" "$BOT_MIXED_ACTION_INTERVAL_MS" "$BOT_MIXED_COMMAND_INTERVAL_MS" "$BOT_MIXED_USE_ENTITY_ATTACKS" "$BOT_ACTION_START_MODE" "$BOT_ACTION_START_AFTER_MS" "$BOT_ACTION_READY_SETTLE_MS" "$BOT_ACTION_READY_REQUIRES_BLOCK_ARMED" "$BOT_ACTION_READY_MIN_COUNT" "$BOT_ACTION_READY_MIN_FRACTION" "$BOT_CHECK_TIMEOUT_INTERVAL_MS" "$BOT_BLOCK_ACTION_INTERVAL_MS" "$BOT_EFFECTIVE_RAMP_SECONDS" "$BOT_GLOBAL_ACTION_START_AFTER_MS" "$BUKKIT_CONNECTION_THROTTLE" "$LOAD_TEST_WORLD_SOURCE_ORIGINAL" "$LOAD_TEST_WORLD_SOURCE_RESOLVED" "$LOAD_TEST_WORLD_SOURCE_KIND" "$LOAD_TEST_WORLD_COPY_METHOD" "$LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA" "$LOAD_TEST_WORLD_SOURCE_PRUNE_DATAPACKS" "$LOAD_TEST_SPARK_BACKGROUND_PROFILER" "$LOAD_TEST_STRESS_CORPUS" "$LOAD_TEST_STRESS_PLUGINS" "$LOAD_TEST_STRESS_DATAPACKS" "$LOAD_TEST_MOB_STORM_COUNT" "$LOAD_TEST_MOB_STORM_ENTITY" "$LOAD_TEST_MOB_STORM_CENTER_X" "$LOAD_TEST_MOB_STORM_Y" "$LOAD_TEST_MOB_STORM_CENTER_Z" "$LOAD_TEST_MOB_STORM_SPACING" "$LOAD_TEST_MOB_STORM_COLUMNS" <<'PY'
import pathlib
import os
import re
import statistics
import sys

(
    log,
    bot_log,
    resource_log,
    summary,
    bots,
    vd,
    sd,
    bot_exit,
    max_loads,
    max_gens,
    load_rate,
    gen_rate,
    send_rate,
    worker_threads,
    io_threads,
    paper_misc_max_joins_per_tick,
    prevent_moving_into_unloaded_chunks,
    update_pathfinding_on_block_update,
    container_update_rate,
    jstack_dir,
    load_test_scenario,
    block_arena_center_x,
    block_arena_center_z,
    block_arena_target_y,
    block_arena_spacing,
    block_arena_columns,
    block_arena_item,
    block_arena_preload_enabled,
    block_arena_preload_radius_chunks,
    block_arena_preload_max_in_flight,
    block_arena_preload_timeout_seconds,
    load_test_gamemode,
    bot_speed,
    bot_move_interval_ms,
    bot_dephase_actions,
    bot_move_scheduler_tick_ms,
    bot_block_scheduler_tick_ms,
    bot_mixed_scheduler_tick_ms,
    bot_send_stationary_positions,
    bot_block_movement_mode,
    bot_mixed_action_interval_ms,
    bot_mixed_command_interval_ms,
    bot_mixed_use_entity_attacks,
    bot_action_start_mode,
    bot_action_start_after_ms,
    bot_action_ready_settle_ms,
    bot_action_ready_requires_block_armed,
    bot_action_ready_min_count,
    bot_action_ready_min_fraction,
    bot_check_timeout_interval_ms,
    block_action_interval_ms,
    block_ramp_seconds,
    global_action_start_after_ms,
    bukkit_connection_throttle,
    world_source_original,
    world_source_resolved,
    world_source_kind,
    world_copy_method,
    world_keep_playerdata,
    world_prune_datapacks,
    spark_background_profiler,
    stress_corpus,
    stress_plugins_enabled,
    stress_datapacks_enabled,
    mob_storm_count,
    mob_storm_entity,
    mob_storm_center_x,
    mob_storm_y,
    mob_storm_center_z,
    mob_storm_spacing,
    mob_storm_columns,
) = sys.argv[1:]
metric_re = re.compile(r"COMPAT_PROBE metrics online=(\d+) loadedChunks=(\d+) tps1=([0-9.]+) tps5=([0-9.]+) tps15=([0-9.]+) avgTickMs=([0-9.]+) usedMemMiB=(\d+)")
token_re = re.compile(r"([A-Za-z][A-Za-z0-9]*)=([^ ]+)")
compat_block_metric_re = re.compile(
    r"COMPAT_PROBE metrics .* blockPlaces=(\d+) blockBreaks=(\d+) arenaCommands=(\d+)(?: arenaPrepared=(\d+) arenaSkipped=(\d+))?"
)
compat_direct_block_event_re = re.compile(
    r"COMPAT_PROBE block_event type=(place|break) places=(\d+) breaks=(\d+) cancelled=(true|false) player=([^ ]+)"
)
compat_arena_chunk_metric_re = re.compile(
    r"COMPAT_PROBE metrics .* arenaChunkLoadsQueued=(\d+) arenaChunkLoadsStarted=(\d+) arenaChunkLoadsCompleted=(\d+) arenaChunkLoadsFailed=(\d+) arenaChunkLoadsInFlight=(\d+)"
)
compat_mob_metric_re = re.compile(
    r"COMPAT_PROBE metrics .* mobStormCommands=(\d+) mobStormSpawned=(\d+) livingEntities=(\d+)"
)
compat_mixed_metric_re = re.compile(
    r"COMPAT_PROBE metrics .* compatProbeCommands=(\d+) playerCommands=(\d+) itemHeldEvents=(\d+) animationEvents=(\d+) interactEvents=(\d+) entityDamageEvents=(\d+) toggleSprintEvents=(\d+) toggleSneakEvents=(\d+)"
)
mobstorm_command_re = re.compile(
    r"COMPAT_PROBE mobstorm entity=([^ ]+) requested=(\d+) spawned=(\d+)"
)
arena_command_re = re.compile(
    r"COMPAT_PROBE arena .* matched=(\d+) prepared=(\d+) teleported=(\d+) skipped=(\d+)"
)
arena_preload_complete_re = re.compile(
    r"COMPAT_PROBE arena_preload complete .* expectedPlayers=(\d+)(?: radiusChunks=(\d+))?(?: maxInFlight=(\d+))? chunks=(\d+) loaded=(\d+) failed=(\d+) tickets=(\d+)"
)
bot_metric_re = re.compile(
    r"swarm_metrics(?: mode=[^ ]+)? created=(\d+) connected=(\d+) ready=(\d+) active=(\d+) ended=(\d+) kicked=(\d+) errors=(\d+) positions=(\d+) positionsPerSec=([0-9.]+) chunks=(\d+) chunksPerSec=([0-9.]+)"
)
action_gate_ready_re = re.compile(
    r"swarm_action_gate_ready mode=([^ ]+) created=(\d+) connected=(\d+) ready=(\d+) active=(\d+) settled=(\d+) required=(\d+) settleMs=(\d+)(?: blockArmed=(\d+))?"
)
action_gate_open_re = re.compile(
    r"swarm_action_gate_open mode=([^ ]+) openedAfterMs=(\d+) .* created=(\d+) connected=(\d+) ready=(\d+) active=(\d+) settled=(\d+) required=(\d+) settleMs=(\d+)(?: minDelayMs=(\d+))?(?: blockArmed=(\d+))?"
)
action_gate_reset_re = re.compile(
    r"swarm_action_gate_reset mode=([^ ]+) reason=(.+?) readySinceMs=(\d+)"
)
action_gate_softened_re = re.compile(
    r"swarm_action_gate_softened mode=([^ ]+) reason=([^ ]+) originalRequired=(\d+) liveRequired=(\d+) missing=(\d+) "
    r"created=(\d+) connected=(\d+) ready=(\d+) active=(\d+) settled=(\d+) blockArmed=(\d+) ended=(\d+) kicked=(\d+) errors=(\d+)"
)
block_metric_re = re.compile(
    r"swarm_metrics .* blockArmed=(\d+) blockPrimed=(\d+) blockCreativeSlotPackets=(\d+) blockPlacePackets=(\d+) blockDigPackets=(\d+) blockActionErrors=(\d+) blockActionsPerSec=([0-9.]+)"
)
mixed_metric_re = re.compile(
    r"swarm_metrics .* mixedActionTicks=(\d+) mixedHeldItemPackets=(\d+) mixedArmAnimationPackets=(\d+) mixedPlayerInputPackets=(\d+) mixedUseItemPackets=(\d+) mixedCommandPackets=(\d+) mixedBlockPlacePackets=(\d+) mixedBlockDigPackets=(\d+) mixedAttackPackets=(\d+) mixedActionErrors=(\d+) mixedActionsPerSec=([0-9.]+)"
)
metrics = []
compat_block_metrics = []
compat_direct_block_events = []
compat_direct_loadbot_block_events = []
compat_arena_chunk_metrics = []
compat_mob_metrics = []
compat_mixed_metrics = []
compat_send_pressure_metrics = []
mobstorm_command_metrics = []
arena_command_metrics = []
arena_preload_metrics = []
bot_metrics = []
block_metrics = []
mixed_metrics = []
loadgen_metrics = []
loadgen_metric_source = "bot_log"
moved_too_quickly = 0
watchdog_dumps = 0
sync_load_stack_hits = 0
nearby_players_stack_hits = 0
thread_check_failures = 0
chunk_system_errors = 0
feature_placement_errors = 0
off_main_poi_hits = 0
server_join_events = 0
server_quit_events = 0
startup_done_seconds = None
worker_line = ""
native_runtime_line = ""
native_climate_rtree_loaded = False
native_area_map_loaded = False
native_improved_noise_loaded = False
native_normal_noise_loaded = False
native_perlin_noise_loaded = False
native_perlin_noise_generic_loaded = False
native_perlin_noise_no_y_scale_loaded = False
native_chunk_packet_encode_loaded = False
native_chunk_packet_encode_sections_enabled = False
native_chunk_packet_encode_light_enabled = False
native_climate_rtree_unavailable = False
native_area_map_unavailable = False
native_improved_noise_unavailable = False
native_normal_noise_unavailable = False
native_perlin_noise_unavailable = False
native_perlin_noise_generic_unavailable = False
native_perlin_noise_no_y_scale_unavailable = False
native_chunk_packet_encode_unavailable = False
server_ready_re = re.compile(
    r"^(?:(?:\x1b\[[0-9;?]*[ -/]*[@-~])|[>\t\r ])*"
    r"\[[0-9]{2}:[0-9]{2}:[0-9]{2} INFO\]: "
    r"Done \(([0-9.]+)s\)! For help, type \"help\""
    r"(?:(?:\x1b\[[0-9;?]*[ -/]*[@-~])|[>\t\r ])*$"
)

def parse_tokens(line):
    return dict(token_re.findall(line))

def int_token(tokens, key):
    try:
        return int(tokens[key])
    except (KeyError, ValueError):
        return None

def float_token(tokens, key):
    try:
        return float(tokens[key])
    except (KeyError, ValueError):
        return None

for line in open(log, encoding="utf-8", errors="replace"):
    if "LoadBot" in line and " joined the game" in line:
        server_join_events += 1
    if "LoadBot" in line and " left the game" in line:
        server_quit_events += 1
    done_match = server_ready_re.search(line)
    if done_match:
        startup_done_seconds = float(done_match.group(1))
    if "moved too quickly" in line:
        moved_too_quickly += 1
    if "Server thread dump" in line:
        watchdog_dumps += 1
    if "ServerChunkCache.syncLoad" in line:
        sync_load_stack_hits += 1
    if "NearbyPlayers" in line:
        nearby_players_stack_hits += 1
    if "Thread failed main thread check" in line:
        thread_check_failures += 1
    if "Chunk system error" in line:
        chunk_system_errors += 1
    if "ReportedException: Feature placement" in line:
        feature_placement_errors += 1
    if "Accessing poi chunk off-main" in line:
        off_main_poi_hits += 1
    if "worker threads" in line and "I/O threads" in line:
        worker_line = line.strip()
    if "[optimized-runtime] native_lib_available=" in line:
        native_runtime_line = line.strip()
        native_tokens = parse_tokens(line)
        native_chunk_packet_encode_sections_enabled = native_tokens.get("native_chunk_packet_encode_sections") == "true"
        native_chunk_packet_encode_light_enabled = native_tokens.get("native_chunk_packet_encode_light") == "true"
    if "Paper: Using native Climate RTree from paper_native_jni." in line:
        native_climate_rtree_loaded = True
    if "Paper: Using native AreaMap from paper_native_jni." in line:
        native_area_map_loaded = True
    if "Paper: Using native ImprovedNoise from paper_native_jni." in line:
        native_improved_noise_loaded = True
    if "Paper: Using native NormalNoise from paper_native_jni." in line:
        native_normal_noise_loaded = True
    if "Paper: Using native PerlinNoise from paper_native_jni." in line:
        native_perlin_noise_loaded = True
    if "Paper: Using native PerlinNoise generic from paper_native_jni." in line:
        native_perlin_noise_loaded = True
        native_perlin_noise_generic_loaded = True
    if "Paper: Using native PerlinNoise no-y-scale from paper_native_jni." in line:
        native_perlin_noise_loaded = True
        native_perlin_noise_no_y_scale_loaded = True
    if (
        "Paper: Using native chunk packet encode from paper_native_chunk_encode_jni." in line
        or "Paper: Using native chunk packet light encode from paper_native_chunk_encode_jni." in line
    ):
        native_chunk_packet_encode_loaded = True
    if "Paper: Native Climate RTree unavailable" in line:
        native_climate_rtree_unavailable = True
    if "Paper: Native AreaMap unavailable" in line:
        native_area_map_unavailable = True
    if "Paper: Native ImprovedNoise unavailable" in line:
        native_improved_noise_unavailable = True
    if "Paper: Native NormalNoise unavailable" in line:
        native_normal_noise_unavailable = True
    if "Paper: Native PerlinNoise unavailable" in line:
        native_perlin_noise_unavailable = True
    if "Paper: Native PerlinNoise generic unavailable" in line:
        native_perlin_noise_unavailable = True
        native_perlin_noise_generic_unavailable = True
    if "Paper: Native PerlinNoise no-y-scale unavailable" in line:
        native_perlin_noise_unavailable = True
        native_perlin_noise_no_y_scale_unavailable = True
    if "Paper: Native chunk packet encode unavailable" in line:
        native_chunk_packet_encode_unavailable = True
    match = metric_re.search(line)
    if match:
        metrics.append({
            "online": int(match.group(1)),
            "chunks": int(match.group(2)),
            "tps1": float(match.group(3)),
            "mspt": float(match.group(6)),
            "mem": int(match.group(7)),
        })
    match = compat_block_metric_re.search(line)
    if match:
        compat_block_metrics.append({
            "places": int(match.group(1)),
            "breaks": int(match.group(2)),
            "arena": int(match.group(3)),
            "arena_prepared": int(match.group(4) or 0),
            "arena_skipped": int(match.group(5) or 0),
        })
    match = compat_direct_block_event_re.search(line)
    if match:
        event = {
            "type": match.group(1),
            "places": int(match.group(2)),
            "breaks": int(match.group(3)),
            "cancelled": match.group(4) == "true",
            "player": match.group(5),
        }
        compat_direct_block_events.append(event)
        if event["player"].startswith("LoadBot"):
            compat_direct_loadbot_block_events.append(event)
    match = compat_arena_chunk_metric_re.search(line)
    if match:
        compat_arena_chunk_metrics.append({
            "queued": int(match.group(1)),
            "started": int(match.group(2)),
            "completed": int(match.group(3)),
            "failed": int(match.group(4)),
            "in_flight": int(match.group(5)),
        })
    match = compat_mob_metric_re.search(line)
    if match:
        compat_mob_metrics.append({
            "commands": int(match.group(1)),
            "spawned": int(match.group(2)),
            "living": int(match.group(3)),
        })
    match = compat_mixed_metric_re.search(line)
    if match:
        compat_mixed_metrics.append({
            "commands": int(match.group(1)),
            "player_commands": int(match.group(2)),
            "held": int(match.group(3)),
            "animation": int(match.group(4)),
            "interact": int(match.group(5)),
            "damage": int(match.group(6)),
            "sprint": int(match.group(7)),
            "sneak": int(match.group(8)),
        })
    if "COMPAT_PROBE metrics" in line and "sendPressurePlayers=" in line:
        tokens = parse_tokens(line)
        sample = {
            "players": int_token(tokens, "sendPressurePlayers"),
            "connections": int_token(tokens, "sendPressureConnections"),
            "chunk_senders": int_token(tokens, "sendPressureChunkSenders"),
            "pending_actions": int_token(tokens, "connectionPendingActionsMax"),
            "pending_outbound_bytes": int_token(tokens, "connectionPendingOutboundBytesMax"),
            "pending_outbound_bytes_read_count": int_token(tokens, "connectionPendingOutboundBytesReadCount"),
            "pending_outbound_bytes_unavailable_count": int_token(tokens, "connectionPendingOutboundBytesUnavailableCount"),
            "bytes_before_writable": int_token(tokens, "connectionBytesBeforeWritableMax"),
            "bytes_before_writable_read_count": int_token(tokens, "connectionBytesBeforeWritableReadCount"),
            "bytes_before_writable_unavailable_count": int_token(tokens, "connectionBytesBeforeWritableUnavailableCount"),
            "bytes_before_unwritable": int_token(tokens, "connectionBytesBeforeUnwritableMin"),
            "bytes_before_unwritable_read_count": int_token(tokens, "connectionBytesBeforeUnwritableReadCount"),
            "bytes_before_unwritable_unavailable_count": int_token(tokens, "connectionBytesBeforeUnwritableUnavailableCount"),
            "non_writable": int_token(tokens, "connectionNonWritable"),
            "pending_chunks": int_token(tokens, "chunkSenderPendingChunksMax"),
            "pending_chunks_read_count": int_token(tokens, "chunkSenderPendingChunksReadCount"),
            "pending_chunks_unavailable_count": int_token(tokens, "chunkSenderPendingChunksUnavailableCount"),
            "unack_batches": int_token(tokens, "chunkSenderUnacknowledgedBatchesMax"),
            "batch_quota": float_token(tokens, "chunkSenderBatchQuotaMax"),
            "desired_chunks_per_tick": float_token(tokens, "chunkSenderDesiredChunksPerTickMax"),
            "max_unack_batches": int_token(tokens, "chunkSenderMaxUnacknowledgedBatchesMax"),
            "channel_not_writable_pending_chunks_peak": int_token(tokens, "chunkSenderChannelNotWritablePendingChunksPeak"),
            "channel_not_writable_pending_chunks_peak_read_count": int_token(tokens, "chunkSenderChannelNotWritablePendingChunksPeakReadCount"),
            "channel_not_writable_pending_chunks_peak_unavailable_count": int_token(tokens, "chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount"),
            "channel_not_writable_skips": int_token(tokens, "chunkSenderChannelNotWritableSkipsMax"),
            "channel_not_writable_skips_read_count": int_token(tokens, "chunkSenderChannelNotWritableSkipsReadCount"),
            "channel_not_writable_skips_unavailable_count": int_token(tokens, "chunkSenderChannelNotWritableSkipsUnavailableCount"),
            "channel_near_unwritable_pending_chunks_peak": int_token(tokens, "chunkSenderNearUnwritablePendingChunksPeak"),
            "channel_near_unwritable_pending_chunks_peak_read_count": int_token(tokens, "chunkSenderNearUnwritablePendingChunksPeakReadCount"),
            "channel_near_unwritable_pending_chunks_peak_unavailable_count": int_token(tokens, "chunkSenderNearUnwritablePendingChunksPeakUnavailableCount"),
            "channel_near_unwritable_skips": int_token(tokens, "chunkSenderNearUnwritableSkipsMax"),
            "channel_near_unwritable_skips_read_count": int_token(tokens, "chunkSenderNearUnwritableSkipsReadCount"),
            "channel_near_unwritable_skips_unavailable_count": int_token(tokens, "chunkSenderNearUnwritableSkipsUnavailableCount"),
        }
        if any(value is not None for value in sample.values()):
            compat_send_pressure_metrics.append(sample)
    match = mobstorm_command_re.search(line)
    if match:
        mobstorm_command_metrics.append({
            "entity": match.group(1),
            "requested": int(match.group(2)),
            "spawned": int(match.group(3)),
        })
    match = arena_command_re.search(line)
    if match:
        arena_command_metrics.append({
            "matched": int(match.group(1)),
            "prepared": int(match.group(2)),
            "teleported": int(match.group(3)),
            "skipped": int(match.group(4)),
        })
    match = arena_preload_complete_re.search(line)
    if match:
        arena_preload_metrics.append({
            "expected": int(match.group(1)),
            "radius_chunks": int(match.group(2) or 0),
            "max_in_flight": int(match.group(3) or 0),
            "chunks": int(match.group(4)),
            "loaded": int(match.group(5)),
            "failed": int(match.group(6)),
            "tickets": int(match.group(7)),
        })
bot_lines = open(bot_log, encoding="utf-8", errors="replace").read().splitlines()

def read_log_lines(path):
    try:
        return path.read_text(encoding="utf-8", errors="replace").splitlines()
    except FileNotFoundError:
        return []

def env_int(name, default):
    try:
        return int(os.environ.get(name, str(default)))
    except ValueError:
        return default

def parse_cpu_list(value):
    cpus = set()
    for raw_part in value.split(","):
        part = raw_part.strip()
        if not part:
            continue
        if "-" in part:
            start_raw, end_raw = part.split("-", 1)
            start = int(start_raw)
            end = int(end_raw)
            if end < start:
                raise ValueError(value)
            cpus.update(range(start, end + 1))
        else:
            cpus.add(int(part))
    return cpus

def effective_cpu_count():
    cpuset = os.environ.get("BENCHMARK_CPUSET", "").strip()
    if cpuset:
        try:
            parsed = parse_cpu_list(cpuset)
            if parsed:
                return len(parsed)
        except ValueError:
            pass
    try:
        return max(1, len(os.sched_getaffinity(0)))
    except (AttributeError, OSError):
        return os.cpu_count() or 1

loadgen_lines = bot_lines
bot_log_dir = os.environ.get("BOT_LOG_DIR", "")
if bot_log_dir and env_int("BOT_SWARM_SHARDS", 1) > 1:
    shard_paths = sorted(pathlib.Path(bot_log_dir).glob("shard-*.log"))
    if shard_paths:
        shard_lines = []
        for shard_path in shard_paths:
            shard_lines.extend(read_log_lines(shard_path))
        if shard_lines:
            loadgen_lines = shard_lines
            loadgen_metric_source = "shard_logs"

for line in loadgen_lines:
    if "swarm_metrics" not in line or "loadGenLoopDelayP95Ms=" not in line:
        continue
    tokens = parse_tokens(line)
    sample = {
        "loop_delay_p95_ms": float_token(tokens, "loadGenLoopDelayP95Ms"),
        "loop_delay_max_ms": float_token(tokens, "loadGenLoopDelayMaxMs"),
        "loop_delay_mean_ms": float_token(tokens, "loadGenLoopDelayMeanMs"),
        "timer_drift_max_ms": float_token(tokens, "loadGenTimerDriftMaxMs"),
        "timer_drift_avg_ms": float_token(tokens, "loadGenTimerDriftAvgMs"),
        "elu_pct": float_token(tokens, "loadGenEluPct"),
    }
    if all(value is not None for value in sample.values()):
        loadgen_metrics.append(sample)

action_gate_ready_events = []
action_gate_open_events = []
action_gate_reset_events = []
action_gate_softened_events = []
for line in bot_lines:
    match = action_gate_ready_re.search(line)
    if match:
        action_gate_ready_events.append({
            "mode": match.group(1),
            "created": int(match.group(2)),
            "connected": int(match.group(3)),
            "ready": int(match.group(4)),
            "active": int(match.group(5)),
            "settled": int(match.group(6)),
            "required": int(match.group(7)),
            "settle_ms": int(match.group(8)),
            "block_armed": int(match.group(9) or 0),
        })
    match = action_gate_open_re.search(line)
    if match:
        action_gate_open_events.append({
            "mode": match.group(1),
            "opened_after_ms": int(match.group(2)),
            "created": int(match.group(3)),
            "connected": int(match.group(4)),
            "ready": int(match.group(5)),
            "active": int(match.group(6)),
            "settled": int(match.group(7)),
            "required": int(match.group(8)),
            "settle_ms": int(match.group(9)),
            "min_delay_ms": int(match.group(10) or 0),
            "block_armed": int(match.group(11) or 0),
        })
    match = action_gate_reset_re.search(line)
    if match:
        action_gate_reset_events.append({
            "mode": match.group(1),
            "reason": match.group(2),
            "ready_since_ms": int(match.group(3)),
        })
    match = action_gate_softened_re.search(line)
    if match:
        action_gate_softened_events.append({
            "mode": match.group(1),
            "reason": match.group(2),
            "original_required": int(match.group(3)),
            "live_required": int(match.group(4)),
            "missing": int(match.group(5)),
            "created": int(match.group(6)),
            "connected": int(match.group(7)),
            "ready": int(match.group(8)),
            "active": int(match.group(9)),
            "settled": int(match.group(10)),
            "block_armed": int(match.group(11)),
            "ended": int(match.group(12)),
            "kicked": int(match.group(13)),
            "errors": int(match.group(14)),
        })
    match = bot_metric_re.search(line)
    if match:
        bot_metrics.append({
            "created": int(match.group(1)),
            "connected": int(match.group(2)),
            "ready": int(match.group(3)),
            "active": int(match.group(4)),
            "ended": int(match.group(5)),
            "kicked": int(match.group(6)),
            "errors": int(match.group(7)),
            "positions": int(match.group(8)),
            "positions_per_sec": float(match.group(9)),
            "chunks": int(match.group(10)),
            "chunks_per_sec": float(match.group(11)),
        })
    match = block_metric_re.search(line)
    if match:
        block_metrics.append({
            "armed": int(match.group(1)),
            "primed": int(match.group(2)),
            "creative": int(match.group(3)),
            "place": int(match.group(4)),
            "dig": int(match.group(5)),
            "errors": int(match.group(6)),
            "actions_per_sec": float(match.group(7)),
        })
    match = mixed_metric_re.search(line)
    if match:
        mixed_metrics.append({
            "ticks": int(match.group(1)),
            "held": int(match.group(2)),
            "arm": int(match.group(3)),
            "input": int(match.group(4)),
            "use": int(match.group(5)),
            "command": int(match.group(6)),
            "place": int(match.group(7)),
            "dig": int(match.group(8)),
            "attack": int(match.group(9)),
            "errors": int(match.group(10)),
            "actions_per_sec": float(match.group(11)),
        })
bot_tail = bot_lines[-20:]
resources = []
bot_resources = []
for line in open(resource_log, encoding="utf-8", errors="replace"):
    if line.startswith("ts_ms,"):
        continue
    parts = line.strip().split(",")
    if len(parts) >= 3 and parts[1] and parts[2]:
        try:
            cpu = float(parts[1])
            rss_kb = int(parts[2])
        except ValueError:
            continue
        resource = {"cpu": cpu, "rss_kb": rss_kb}
        if len(parts) >= 4 and parts[3]:
            try:
                resource["system_load1"] = float(parts[3])
            except ValueError:
                pass
        if len(parts) >= 5 and parts[4]:
            try:
                resource["system_mem_available_kb"] = int(parts[4])
            except ValueError:
                pass
        if len(parts) >= 9 and all(parts[i] for i in range(5, 9)):
            try:
                resource["host_cpu_total"] = int(parts[5])
                resource["host_cpu_idle"] = int(parts[6])
                resource["host_cpu_iowait"] = int(parts[7])
                resource["host_cpu_steal"] = int(parts[8])
            except ValueError:
                pass
        resources.append(resource)
        if len(parts) >= 15 and parts[9]:
            try:
                bot_process_count = int(parts[9])
            except ValueError:
                bot_process_count = None
            if bot_process_count is not None and bot_process_count > 0 and parts[10] and parts[11]:
                try:
                    bot_resource = {
                        "process_count": bot_process_count,
                        "rss_kb_max": int(parts[10]),
                        "rss_kb_total": int(parts[11]),
                        "pss_available": parts[14].strip().lower() == "true" if len(parts) >= 15 else False,
                    }
                    if bot_resource["pss_available"] and parts[12] and parts[13]:
                        bot_resource["pss_kb_max"] = int(parts[12])
                        bot_resource["pss_kb_total"] = int(parts[13])
                    else:
                        bot_resource["pss_available"] = False
                    bot_resources.append(bot_resource)
                except ValueError:
                    pass

def values(name):
    return [m[name] for m in metrics]

def metric_values(samples, name):
    return [m[name] for m in samples]

def summarize_metric_window(out, prefix, samples):
    out.write(f"{prefix}metrics_samples={len(samples)}\n")
    if not samples:
        return
    out.write(f"{prefix}online_max={max(metric_values(samples, 'online'))}\n")
    out.write(f"{prefix}loaded_chunks_max={max(metric_values(samples, 'chunks'))}\n")
    out.write(f"{prefix}tps1_min={min(metric_values(samples, 'tps1')):.2f}\n")
    out.write(f"{prefix}tps1_avg={statistics.fmean(metric_values(samples, 'tps1')):.2f}\n")
    out.write(f"{prefix}avg_tick_ms_max={max(metric_values(samples, 'mspt')):.2f}\n")
    out.write(f"{prefix}avg_tick_ms_avg={statistics.fmean(metric_values(samples, 'mspt')):.2f}\n")
    out.write(f"{prefix}used_mem_mib_max={max(metric_values(samples, 'mem'))}\n")

def fmt(v):
    return "n/a" if v is None else f"{v:.2f}" if isinstance(v, float) else str(v)

def max_sample(samples, key):
    return max((sample[key] for sample in samples), default=0)

bot_resource_samples = len(bot_resources)
bot_resource_pss_samples = [sample for sample in bot_resources if sample["pss_available"]]
bot_resource_current = bot_resources[-1] if bot_resources else None
bot_resource_pss_current = bot_resource_current if bot_resource_current and bot_resource_current["pss_available"] else None

bot_count_int = int(float(bots))
load_window_metrics = []
load_window_reached_full_online = False
load_window_ended_by_online_drop = False
load_window_started = False
load_window_start_index = None
load_window_end_index = None
for idx, metric in enumerate(metrics):
    if not load_window_started:
        if metric["online"] >= bot_count_int:
            load_window_started = True
            load_window_reached_full_online = True
            load_window_start_index = idx
            load_window_metrics.append(metric)
        continue
    if metric["online"] < bot_count_int:
        load_window_ended_by_online_drop = True
        load_window_end_index = idx
        break
    load_window_metrics.append(metric)
    if metric["online"] >= bot_count_int:
        load_window_reached_full_online = True
if load_window_started and load_window_end_index is None:
    load_window_end_index = (load_window_start_index or 0) + len(load_window_metrics)
teardown_metrics = metrics[load_window_end_index:] if load_window_end_index is not None else []

bot_block_place_packets_max = max_sample(block_metrics, "place")
bot_block_dig_packets_max = max_sample(block_metrics, "dig")
bot_mixed_block_place_packets_max = max_sample(mixed_metrics, "place")
bot_mixed_block_dig_packets_max = max_sample(mixed_metrics, "dig")
compat_probe_periodic_block_places_max = max_sample(compat_block_metrics, "places")
compat_probe_periodic_block_breaks_max = max_sample(compat_block_metrics, "breaks")
compat_probe_block_event_places_max = max_sample(compat_direct_block_events, "places")
compat_probe_block_event_breaks_max = max_sample(compat_direct_block_events, "breaks")
compat_probe_block_event_loadbot_places_max = max_sample(compat_direct_loadbot_block_events, "places")
compat_probe_block_event_loadbot_breaks_max = max_sample(compat_direct_loadbot_block_events, "breaks")
compat_probe_block_places_max = max(
    compat_probe_periodic_block_places_max,
    compat_probe_block_event_loadbot_places_max,
)
compat_probe_block_breaks_max = max(
    compat_probe_periodic_block_breaks_max,
    compat_probe_block_event_loadbot_breaks_max,
)
compat_probe_block_actions_observed = any((
    bot_block_place_packets_max > 0,
    bot_block_dig_packets_max > 0,
    bot_mixed_block_place_packets_max > 0,
    bot_mixed_block_dig_packets_max > 0,
))
compat_probe_periodic_block_counts_seen = (
    compat_probe_periodic_block_places_max > 0 or compat_probe_periodic_block_breaks_max > 0
)
compat_probe_direct_block_event_evidence = bool(compat_direct_block_events)
compat_probe_block_metrics_loadbot_direct_evidence = bool(compat_direct_loadbot_block_events)
compat_probe_direct_block_loadbot_place_event_lines = sum(
    1 for m in compat_direct_loadbot_block_events if m["type"] == "place"
)
compat_probe_direct_block_loadbot_break_event_lines = sum(
    1 for m in compat_direct_loadbot_block_events if m["type"] == "break"
)
compat_probe_direct_block_loadbot_cancelled_true_lines = sum(
    1 for m in compat_direct_loadbot_block_events if m["cancelled"]
)
compat_probe_direct_block_loadbot_cancelled_false_lines = sum(
    1 for m in compat_direct_loadbot_block_events if not m["cancelled"]
)
compat_probe_direct_block_loadbot_players = len({m["player"] for m in compat_direct_loadbot_block_events})
compat_probe_block_evidence_accepted = (
    compat_probe_block_actions_observed
    and compat_probe_block_metrics_loadbot_direct_evidence
    and compat_probe_direct_block_loadbot_cancelled_false_lines > 0
    and compat_probe_block_event_loadbot_places_max > 0
    and compat_probe_block_event_loadbot_breaks_max > 0
)
compat_probe_block_metrics_after_actions = (
    compat_probe_block_actions_observed
    and (
        compat_probe_periodic_block_counts_seen
        or compat_probe_block_metrics_loadbot_direct_evidence
    )
)
compat_probe_block_metrics_stale_after_actions = (
    compat_probe_block_actions_observed
    and bool(compat_block_metrics)
    and not compat_probe_periodic_block_counts_seen
    and not compat_probe_block_metrics_loadbot_direct_evidence
)
compat_probe_block_metrics_ambiguous_after_actions = (
    compat_probe_block_actions_observed
    and not compat_probe_periodic_block_counts_seen
    and not compat_probe_block_metrics_loadbot_direct_evidence
)

with open(summary, "w", encoding="utf-8") as out:
    out.write(f"bots={bots} view_distance={vd} simulation_distance={sd} bot_exit={bot_exit}\n")
    out.write(f"duration_seconds={os.environ.get('DURATION_SECONDS', '')}\n")
    out.write(f"java_opts_load={os.environ.get('JAVA_OPTS_LOAD', '')}\n")
    out.write(f"server_effective_pid={os.environ.get('SERVER_EFFECTIVE_PID', '')}\n")
    out.write(f"server_cmdline_effective={os.environ.get('SERVER_CMDLINE_EFFECTIVE', '')}\n")
    out.write(f"server_jvm_flags_effective={os.environ.get('SERVER_JVM_FLAGS_EFFECTIVE', '')}\n")
    out.write(f"server_proc_cmdline={os.environ.get('server_proc_cmdline', '')}\n")
    out.write(f"server_jcmd_vm_command_line_available={os.environ.get('server_jcmd_vm_command_line_available', 'false')}\n")
    out.write(f"server_jcmd_vm_command_line={os.environ.get('server_jcmd_vm_command_line', '')}\n")
    out.write(f"resource_log={resource_log}\n")
    out.write(f"bot_pid_file={os.environ.get('BOT_PID_FILE', '')}\n")
    out.write(f"launcher_path={os.environ.get('LAUNCHER', '')}\n")
    out.write(f"launcher_sha256={os.environ.get('LAUNCHER_SHA256', 'none')}\n")
    out.write(f"optimized_artifact_path={os.environ.get('OPTIMIZED_ARTIFACT_PATH', '')}\n")
    out.write(f"optimized_artifact_sha256={os.environ.get('OPTIMIZED_ARTIFACT_SHA256', 'none')}\n")
    out.write(f"optimized_runtime_run_sh_path={os.environ.get('OPTIMIZED_RUNTIME_RUN_SH_PATH', '')}\n")
    out.write(f"optimized_runtime_run_sh_sha256={os.environ.get('OPTIMIZED_RUNTIME_RUN_SH_SHA256', 'none')}\n")
    out.write(f"optimized_runtime_jar_sha256={os.environ.get('OPTIMIZED_RUNTIME_JAR_SHA256', 'none')}\n")
    out.write(f"optimized_runtime_native_library_path={os.environ.get('OPTIMIZED_RUNTIME_NATIVE_LIBRARY_PATH', '')}\n")
    out.write(f"optimized_runtime_native_library_sha256={os.environ.get('OPTIMIZED_RUNTIME_NATIVE_LIBRARY_SHA256', 'none')}\n")
    out.write(f"optimized_runtime_chunk_encode_native_library_path={os.environ.get('OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_PATH', '')}\n")
    out.write(f"optimized_runtime_chunk_encode_native_library_sha256={os.environ.get('OPTIMIZED_RUNTIME_CHUNK_ENCODE_NATIVE_LIBRARY_SHA256', 'none')}\n")
    world_mode = "warm-source" if world_source_resolved else "fresh"
    claim_surface = "warm-world" if world_source_resolved else "cold-fresh"
    out.write(f"world_mode={world_mode}\n")
    out.write(f"claim_surface={claim_surface}\n")
    out.write(f"world_warm_source_present={str(bool(world_source_resolved)).lower()}\n")
    out.write(f"world_warm_source={world_source_resolved or 'none'}\n")
    out.write(f"world_warm_source_original={world_source_original or 'none'}\n")
    out.write(f"world_warm_source_kind={world_source_kind}\n")
    out.write(f"world_warm_copy_method={world_copy_method}\n")
    out.write(f"world_warm_keep_playerdata={world_keep_playerdata}\n")
    out.write(f"world_warm_prune_datapacks={world_prune_datapacks}\n")
    out.write(f"spark_background_profiler={spark_background_profiler}\n")
    out.write(f"stress_corpus={stress_corpus}\n")
    out.write(f"stress_plugins_enabled={stress_plugins_enabled}\n")
    out.write(f"stress_datapacks_enabled={stress_datapacks_enabled}\n")
    out.write(f"stress_corpus_manifest_path={os.environ.get('LOAD_TEST_STRESS_CORPUS_MANIFEST', '')}\n")
    out.write(f"stress_corpus_manifest_sha256={os.environ.get('LOAD_TEST_STRESS_CORPUS_MANIFEST_SHA256', 'none')}\n")
    run_dir = pathlib.Path(os.environ.get("LOAD_TEST_RUN_DIR", ""))
    out.write(f"plugin_jars_total={len(list((run_dir / 'plugins').glob('*.jar')))}\n")
    out.write(f"datapack_zips_total={len(list((run_dir / 'world/datapacks').glob('*.zip')))}\n")
    out.write(f"stress_plugin_jars={os.environ.get('LOAD_TEST_STRESS_PLUGIN_JAR_COUNT', '0')}\n")
    out.write(f"stress_datapack_zips={os.environ.get('LOAD_TEST_STRESS_DATAPACK_ZIP_COUNT', '0')}\n")
    if stress_plugins_enabled == "true":
        out.write(f"matrix_plugin_jars={os.environ.get('LOAD_TEST_MATRIX_PLUGIN_JAR_COUNT', '0')}\n")
    if stress_plugins_enabled == "true":
        out.write(f"stress_squaremap_port={os.environ.get('SQUAREMAP_PORT', '')}\n")
        out.write(f"stress_bluemap_port={os.environ.get('BLUEMAP_PORT', '')}\n")
        out.write(f"stress_voicechat_port={os.environ.get('VOICECHAT_PORT', '')}\n")
        out.write(f"stress_serverpulse_influx_port={os.environ.get('SERVERPULSE_INFLUX_PORT', '')}\n")
        out.write(f"stress_serverpulse_influx_log={os.environ.get('SERVERPULSE_INFLUX_LOG', '')}\n")
    out.write(f"mob_storm_requested={mob_storm_count}\n")
    out.write(f"mob_storm_entity={mob_storm_entity}\n")
    out.write(
        "mob_storm="
        f"center_x={mob_storm_center_x} "
        f"center_z={mob_storm_center_z} "
        f"target_y={mob_storm_y} "
        f"spacing={mob_storm_spacing} "
        f"columns={mob_storm_columns}\n"
    )
    out.write(f"load_test_scenario={load_test_scenario}\n")
    out.write(f"load_test_gamemode={load_test_gamemode}\n")
    out.write(f"arena_preload_enabled={block_arena_preload_enabled}\n")
    out.write(f"arena_preload_radius_chunks={block_arena_preload_radius_chunks}\n")
    out.write(f"arena_preload_max_in_flight={block_arena_preload_max_in_flight}\n")
    out.write(f"arena_preload_timeout_seconds={block_arena_preload_timeout_seconds}\n")
    out.write(f"bukkit_connection_throttle={bukkit_connection_throttle}\n")
    out.write(f"paper_misc_max_joins_per_tick={paper_misc_max_joins_per_tick}\n")
    out.write(f"bot_speed_blocks_per_second={bot_speed}\n")
    out.write(f"bot_move_interval_ms={bot_move_interval_ms}\n")
    out.write(f"bot_dephase_actions={bot_dephase_actions}\n")
    out.write(f"bot_move_scheduler_tick_ms={bot_move_scheduler_tick_ms}\n")
    out.write(f"bot_block_scheduler_tick_ms={bot_block_scheduler_tick_ms}\n")
    out.write(f"bot_mixed_scheduler_tick_ms={bot_mixed_scheduler_tick_ms}\n")
    out.write(f"bot_send_stationary_positions={bot_send_stationary_positions}\n")
    out.write(f"bot_block_movement_mode={bot_block_movement_mode}\n")
    out.write(f"bot_mixed_action_interval_ms={bot_mixed_action_interval_ms}\n")
    out.write(f"bot_mixed_command_interval_ms={bot_mixed_command_interval_ms}\n")
    out.write(f"bot_mixed_use_entity_attacks={bot_mixed_use_entity_attacks}\n")
    out.write(f"bot_action_start_mode={bot_action_start_mode}\n")
    out.write(f"bot_action_start_after_ms={bot_action_start_after_ms}\n")
    out.write(f"bot_action_ready_settle_ms={bot_action_ready_settle_ms}\n")
    out.write(f"bot_action_ready_requires_block_armed={bot_action_ready_requires_block_armed}\n")
    out.write(f"bot_action_ready_min_count={bot_action_ready_min_count}\n")
    out.write(f"bot_action_ready_min_fraction={bot_action_ready_min_fraction}\n")
    out.write(f"bot_check_timeout_interval_ms={bot_check_timeout_interval_ms}\n")
    out.write(f"bot_strict_failures={os.environ.get('BOT_STRICT_FAILURES', '')}\n")
    out.write(f"bot_min_run_after_action_open_ms={os.environ.get('BOT_MIN_RUN_AFTER_ACTION_OPEN_MS', '')}\n")
    out.write(f"bot_max_duration_ms={os.environ.get('BOT_MAX_DURATION_MS', '')}\n")
    out.write(f"bot_swarm_shards={os.environ.get('BOT_SWARM_SHARDS', '1')}\n")
    out.write(f"bot_swarm_node_max_old_space_mb={os.environ.get('BOT_SWARM_NODE_MAX_OLD_SPACE_MB', '')}\n")
    out.write(f"bot_swarm_node_max_semi_space_mb={os.environ.get('BOT_SWARM_NODE_MAX_SEMI_SPACE_MB', '')}\n")
    out.write(f"load_test_native_memory_tracking={os.environ.get('LOAD_TEST_NATIVE_MEMORY_TRACKING', '')}\n")
    out.write(f"bot_shared_action_gate_file={os.environ.get('BOT_SWARM_SHARED_GATE_FILE', '')}\n")
    out.write(f"bot_shard_log_dir={os.environ.get('BOT_LOG_DIR', '')}\n")
    out.write(f"bot_child_process_count={os.environ.get('bot_child_process_count', '0')}\n")
    out.write(f"bot_child_process_rss_kb_total={os.environ.get('bot_child_process_rss_kb_total', '0')}\n")
    out.write(f"bot_child_process_rss_kb_max={os.environ.get('bot_child_process_rss_kb_max', '0')}\n")
    out.write(f"bot_child_process_pss_kb_total={os.environ.get('bot_child_process_pss_kb_total', '0')}\n")
    out.write(f"bot_child_process_pss_kb_max={os.environ.get('bot_child_process_pss_kb_max', '0')}\n")
    out.write(f"bot_child_process_missing_count={os.environ.get('bot_child_process_missing_count', '0')}\n")
    out.write(f"bot_resource_samples={bot_resource_samples}\n")
    if bot_resource_current:
        out.write(f"bot_process_count_current={bot_resource_current['process_count']}\n")
        out.write(f"bot_process_count_max={max(sample['process_count'] for sample in bot_resources)}\n")
        out.write(f"bot_rss_mib_current={bot_resource_current['rss_kb_max'] / 1024:.1f}\n")
        out.write(f"bot_rss_mib_max={max(sample['rss_kb_max'] for sample in bot_resources) / 1024:.1f}\n")
        out.write(f"bot_rss_mib_aggregate_current={bot_resource_current['rss_kb_total'] / 1024:.1f}\n")
        out.write(f"bot_rss_mib_aggregate_max={max(sample['rss_kb_total'] for sample in bot_resources) / 1024:.1f}\n")
    else:
        out.write("bot_process_count_current=0\n")
        out.write("bot_process_count_max=0\n")
        out.write("bot_rss_mib_current=0.0\n")
        out.write("bot_rss_mib_max=0.0\n")
        out.write("bot_rss_mib_aggregate_current=0.0\n")
        out.write("bot_rss_mib_aggregate_max=0.0\n")
    out.write(f"bot_pss_mib_available={str(bool(bot_resource_pss_samples)).lower()}\n")
    if bot_resource_pss_current:
        out.write(f"bot_pss_mib_current={bot_resource_pss_current['pss_kb_max'] / 1024:.1f}\n")
        out.write(f"bot_pss_mib_max={max(sample['pss_kb_max'] for sample in bot_resource_pss_samples) / 1024:.1f}\n")
        out.write(f"bot_pss_mib_aggregate_current={bot_resource_pss_current['pss_kb_total'] / 1024:.1f}\n")
        out.write(f"bot_pss_mib_aggregate_max={max(sample['pss_kb_total'] for sample in bot_resource_pss_samples) / 1024:.1f}\n")
    out.write(f"bot_action_gate_ready_events={len(action_gate_ready_events)}\n")
    out.write(f"bot_action_gate_reset_events={len(action_gate_reset_events)}\n")
    out.write(f"bot_action_gate_softened={str(bool(action_gate_softened_events)).lower()}\n")
    out.write(f"bot_action_gate_softened_events={len(action_gate_softened_events)}\n")
    if action_gate_softened_events:
        action_gate_softened = action_gate_softened_events[-1]
        out.write(f"bot_action_gate_softened_mode={action_gate_softened['mode']}\n")
        out.write(f"bot_action_gate_softened_reason={action_gate_softened['reason']}\n")
        out.write(f"bot_action_gate_softened_original_required={action_gate_softened['original_required']}\n")
        out.write(f"bot_action_gate_softened_live_required={action_gate_softened['live_required']}\n")
        out.write(f"bot_action_gate_softened_missing={action_gate_softened['missing']}\n")
        out.write(f"bot_action_gate_softened_active={action_gate_softened['active']}\n")
        out.write(f"bot_action_gate_softened_settled={action_gate_softened['settled']}\n")
        out.write(f"bot_action_gate_softened_block_armed={action_gate_softened['block_armed']}\n")
        out.write(f"bot_action_gate_softened_ended={action_gate_softened['ended']}\n")
        out.write(f"bot_action_gate_softened_kicked={action_gate_softened['kicked']}\n")
        out.write(f"bot_action_gate_softened_errors={action_gate_softened['errors']}\n")
    action_gate_open_mode = "none"
    out.write(f"bot_action_gate_opened={str(bool(action_gate_open_events)).lower()}\n")
    if action_gate_open_events:
        action_gate_open = action_gate_open_events[-1]
        action_gate_open_mode = action_gate_open["mode"]
        out.write(f"bot_action_gate_open_mode={action_gate_open_mode}\n")
        out.write(f"bot_action_gate_opened_after_ms={action_gate_open['opened_after_ms']}\n")
        out.write(f"bot_action_gate_open_ready={action_gate_open['ready']}\n")
        out.write(f"bot_action_gate_open_active={action_gate_open['active']}\n")
        out.write(f"bot_action_gate_open_settled={action_gate_open['settled']}\n")
        out.write(f"bot_action_gate_open_required={action_gate_open['required']}\n")
        out.write(f"bot_action_gate_open_block_armed={action_gate_open['block_armed']}\n")
    else:
        out.write(f"bot_action_gate_open_mode={action_gate_open_mode}\n")
    out.write(f"bot_ramp_seconds={block_ramp_seconds}\n")
    if startup_done_seconds is not None:
        out.write(f"startup_done_seconds={startup_done_seconds:.3f}\n")
    if load_test_scenario in ("block", "mixed-gameplay"):
        out.write(f"block_action_interval_ms={block_action_interval_ms}\n")
        out.write(f"block_ramp_seconds={block_ramp_seconds}\n")
        out.write(f"global_action_start_after_ms={global_action_start_after_ms}\n")
        out.write(
            "block_arena="
            f"center_x={block_arena_center_x} "
            f"center_z={block_arena_center_z} "
            f"target_y={block_arena_target_y} "
            f"spacing={block_arena_spacing} "
            f"columns={block_arena_columns} "
            f"item={block_arena_item}\n"
        )
    if load_test_scenario == "mixed-gameplay":
        out.write(f"mixed_action_interval_ms={bot_mixed_action_interval_ms}\n")
        out.write(f"mixed_command_interval_ms={bot_mixed_command_interval_ms}\n")
        out.write(f"mixed_use_entity_attacks={bot_mixed_use_entity_attacks}\n")
    if any((max_loads, max_gens, load_rate, gen_rate, send_rate, worker_threads, io_threads, prevent_moving_into_unloaded_chunks, update_pathfinding_on_block_update, container_update_rate)):
        out.write(
            "paper_chunk_config="
            f"max_loads={max_loads or 'auto'} "
            f"max_gens={max_gens or 'auto'} "
            f"load_rate={load_rate or 'default'} "
            f"gen_rate={gen_rate or 'default'} "
            f"send_rate={send_rate or 'default'} "
            f"workers={worker_threads or 'auto'} "
            f"io={io_threads or 'auto'} "
            f"prevent_unloaded_move={prevent_moving_into_unloaded_chunks or 'false'} "
            f"update_pathfinding_on_block_update={update_pathfinding_on_block_update or 'default'} "
            f"container_update={container_update_rate or 'default'}\n"
        )
    if worker_line:
        out.write(f"worker_line={worker_line}\n")
    if native_runtime_line:
        out.write(f"native_runtime_line={native_runtime_line}\n")
    out.write(f"native_climate_rtree_loaded={str(native_climate_rtree_loaded).lower()}\n")
    out.write(f"native_area_map_loaded={str(native_area_map_loaded).lower()}\n")
    out.write(f"native_improved_noise_loaded={str(native_improved_noise_loaded).lower()}\n")
    out.write(f"native_normal_noise_loaded={str(native_normal_noise_loaded).lower()}\n")
    out.write(f"native_perlin_noise_loaded={str(native_perlin_noise_loaded).lower()}\n")
    out.write(f"native_perlin_noise_generic_loaded={str(native_perlin_noise_generic_loaded).lower()}\n")
    out.write(f"native_perlin_noise_no_y_scale_loaded={str(native_perlin_noise_no_y_scale_loaded).lower()}\n")
    out.write(f"native_chunk_packet_encode_loaded={str(native_chunk_packet_encode_loaded).lower()}\n")
    out.write(f"native_chunk_packet_encode_sections_enabled={str(native_chunk_packet_encode_sections_enabled).lower()}\n")
    out.write(f"native_chunk_packet_encode_light_enabled={str(native_chunk_packet_encode_light_enabled).lower()}\n")
    out.write(f"native_climate_rtree_unavailable={str(native_climate_rtree_unavailable).lower()}\n")
    out.write(f"native_area_map_unavailable={str(native_area_map_unavailable).lower()}\n")
    out.write(f"native_improved_noise_unavailable={str(native_improved_noise_unavailable).lower()}\n")
    out.write(f"native_normal_noise_unavailable={str(native_normal_noise_unavailable).lower()}\n")
    out.write(f"native_perlin_noise_unavailable={str(native_perlin_noise_unavailable).lower()}\n")
    out.write(f"native_perlin_noise_generic_unavailable={str(native_perlin_noise_generic_unavailable).lower()}\n")
    out.write(f"native_perlin_noise_no_y_scale_unavailable={str(native_perlin_noise_no_y_scale_unavailable).lower()}\n")
    out.write(f"native_chunk_packet_encode_unavailable={str(native_chunk_packet_encode_unavailable).lower()}\n")
    summarize_metric_window(out, "", metrics)
    out.write("load_window_policy=until_first_online_drop_after_reaching_bots\n")
    out.write("load_window_start_mode=first_full_online_sample\n")
    out.write(f"load_window_reached_full_online={str(load_window_reached_full_online).lower()}\n")
    out.write(f"load_window_ended_by_online_drop={str(load_window_ended_by_online_drop).lower()}\n")
    summarize_metric_window(out, "load_window_", load_window_metrics)
    summarize_metric_window(out, "teardown_", teardown_metrics)
    if bot_metrics:
        out.write(f"bot_created_max={max(m['created'] for m in bot_metrics)}\n")
        out.write(f"bot_connected_max={max(m['connected'] for m in bot_metrics)}\n")
        out.write(f"bot_ready_max={max(m['ready'] for m in bot_metrics)}\n")
        out.write("bot_connected_source=protocol_login_packet\n")
        out.write("bot_ready_source=client_playerJoin_signal\n")
        out.write(f"bot_login_packet_max={max(m['connected'] for m in bot_metrics)}\n")
        out.write(f"bot_player_join_ready_max={max(m['ready'] for m in bot_metrics)}\n")
        out.write(f"bot_active_max={max(m['active'] for m in bot_metrics)}\n")
        out.write(f"bot_kicked_max={max(m['kicked'] for m in bot_metrics)}\n")
        out.write(f"bot_errors_max={max(m['errors'] for m in bot_metrics)}\n")
        out.write(f"bot_position_packets_max={max(m['positions'] for m in bot_metrics)}\n")
        out.write(f"bot_positions_per_sec_max={max(m['positions_per_sec'] for m in bot_metrics):.1f}\n")
        out.write(f"bot_chunk_packets_window_max={max(m['chunks'] for m in bot_metrics)}\n")
        out.write(f"bot_chunks_per_sec_max={max(m['chunks_per_sec'] for m in bot_metrics):.1f}\n")
    if loadgen_metrics:
        def write_loadgen_summary(key, output_key):
            values = [m[key] for m in loadgen_metrics]
            out.write(f"{output_key}_max={max(values):.2f}\n")
            out.write(f"{output_key}_avg={statistics.fmean(values):.2f}\n")

        out.write(f"bot_loadgen_telemetry_source={loadgen_metric_source}\n")
        out.write(f"bot_loadgen_telemetry_samples={len(loadgen_metrics)}\n")
        write_loadgen_summary("loop_delay_p95_ms", "bot_loadgen_loop_delay_p95_ms")
        write_loadgen_summary("loop_delay_max_ms", "bot_loadgen_loop_delay_max_ms")
        write_loadgen_summary("loop_delay_mean_ms", "bot_loadgen_loop_delay_mean_ms")
        write_loadgen_summary("timer_drift_max_ms", "bot_loadgen_timer_drift_max_ms")
        write_loadgen_summary("timer_drift_avg_ms", "bot_loadgen_timer_drift_avg_ms")
        write_loadgen_summary("elu_pct", "bot_loadgen_elu_pct")
    if compat_send_pressure_metrics:
        def send_pressure_values(key):
            return [m[key] for m in compat_send_pressure_metrics if m[key] is not None]

        def write_send_pressure_max(key, output_key):
            values = send_pressure_values(key)
            if values:
                out.write(f"{output_key}={max(values)}\n")

        def write_send_pressure_float_max(key, output_key):
            values = send_pressure_values(key)
            if values:
                out.write(f"{output_key}={max(values):.2f}\n")

        def write_send_pressure_nonnegative_min(key, output_key):
            values = [value for value in send_pressure_values(key) if value >= 0]
            if values:
                out.write(f"{output_key}={min(values)}\n")
            else:
                out.write(f"{output_key}=-1\n")

        out.write(f"compat_probe_send_pressure_samples={len(compat_send_pressure_metrics)}\n")
        write_send_pressure_max("players", "compat_probe_send_pressure_players_max")
        write_send_pressure_max("connections", "compat_probe_send_pressure_connections_max")
        write_send_pressure_max("chunk_senders", "compat_probe_send_pressure_chunk_senders_max")
        write_send_pressure_max("pending_actions", "compat_probe_send_pending_actions_max")
        write_send_pressure_max("pending_outbound_bytes", "compat_probe_send_pending_outbound_bytes_max")
        write_send_pressure_max("pending_outbound_bytes_read_count", "compat_probe_send_pending_outbound_bytes_read_count_max")
        write_send_pressure_max("pending_outbound_bytes_unavailable_count", "compat_probe_send_pending_outbound_bytes_unavailable_count_max")
        write_send_pressure_max("bytes_before_writable", "compat_probe_send_bytes_before_writable_max")
        write_send_pressure_max("bytes_before_writable_read_count", "compat_probe_send_bytes_before_writable_read_count_max")
        write_send_pressure_max("bytes_before_writable_unavailable_count", "compat_probe_send_bytes_before_writable_unavailable_count_max")
        write_send_pressure_nonnegative_min("bytes_before_unwritable", "compat_probe_send_bytes_before_unwritable_min")
        write_send_pressure_max("bytes_before_unwritable_read_count", "compat_probe_send_bytes_before_unwritable_read_count_max")
        write_send_pressure_max("bytes_before_unwritable_unavailable_count", "compat_probe_send_bytes_before_unwritable_unavailable_count_max")
        write_send_pressure_max("non_writable", "compat_probe_send_non_writable_connections_max")
        write_send_pressure_max("pending_chunks", "compat_probe_chunk_send_pending_chunks_max")
        write_send_pressure_max("pending_chunks_read_count", "compat_probe_chunk_send_pending_chunks_read_count_max")
        write_send_pressure_max("pending_chunks_unavailable_count", "compat_probe_chunk_send_pending_chunks_unavailable_count_max")
        write_send_pressure_max("unack_batches", "compat_probe_chunk_send_unacknowledged_batches_max")
        write_send_pressure_float_max("batch_quota", "compat_probe_chunk_send_batch_quota_max")
        write_send_pressure_float_max("desired_chunks_per_tick", "compat_probe_chunk_send_desired_chunks_per_tick_max")
        write_send_pressure_max("max_unack_batches", "compat_probe_chunk_send_max_unacknowledged_batches_max")
        write_send_pressure_max("channel_not_writable_pending_chunks_peak", "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak")
        write_send_pressure_max("channel_not_writable_pending_chunks_peak_read_count", "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_read_count_max")
        write_send_pressure_max("channel_not_writable_pending_chunks_peak_unavailable_count", "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_unavailable_count_max")
        write_send_pressure_max("channel_not_writable_skips", "compat_probe_chunk_send_channel_not_writable_skips_max")
        write_send_pressure_max("channel_not_writable_skips_read_count", "compat_probe_chunk_send_channel_not_writable_skips_read_count_max")
        write_send_pressure_max("channel_not_writable_skips_unavailable_count", "compat_probe_chunk_send_channel_not_writable_skips_unavailable_count_max")
        write_send_pressure_max("channel_near_unwritable_pending_chunks_peak", "compat_probe_chunk_send_near_unwritable_pending_chunks_peak")
        write_send_pressure_max("channel_near_unwritable_pending_chunks_peak_read_count", "compat_probe_chunk_send_near_unwritable_pending_chunks_peak_read_count_max")
        write_send_pressure_max("channel_near_unwritable_pending_chunks_peak_unavailable_count", "compat_probe_chunk_send_near_unwritable_pending_chunks_peak_unavailable_count_max")
        write_send_pressure_max("channel_near_unwritable_skips", "compat_probe_chunk_send_near_unwritable_skips_max")
        write_send_pressure_max("channel_near_unwritable_skips_read_count", "compat_probe_chunk_send_near_unwritable_skips_read_count_max")
        write_send_pressure_max("channel_near_unwritable_skips_unavailable_count", "compat_probe_chunk_send_near_unwritable_skips_unavailable_count_max")
    if block_metrics:
        out.write(f"bot_block_armed_max={max(m['armed'] for m in block_metrics)}\n")
        out.write(f"bot_block_primed_max={max(m['primed'] for m in block_metrics)}\n")
        out.write(f"bot_block_creative_slot_packets_max={max(m['creative'] for m in block_metrics)}\n")
        out.write(f"bot_block_place_packets_max={bot_block_place_packets_max}\n")
        out.write(f"bot_block_dig_packets_max={bot_block_dig_packets_max}\n")
        out.write(f"bot_block_action_errors_max={max(m['errors'] for m in block_metrics)}\n")
        out.write(f"bot_block_actions_per_sec_max={max(m['actions_per_sec'] for m in block_metrics):.1f}\n")
    if mixed_metrics:
        out.write(f"bot_mixed_action_ticks_max={max(m['ticks'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_held_item_packets_max={max(m['held'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_arm_animation_packets_max={max(m['arm'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_player_input_packets_max={max(m['input'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_use_item_packets_max={max(m['use'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_command_packets_max={max(m['command'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_block_place_packets_max={bot_mixed_block_place_packets_max}\n")
        out.write(f"bot_mixed_block_dig_packets_max={bot_mixed_block_dig_packets_max}\n")
        out.write(f"bot_mixed_attack_packets_max={max(m['attack'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_action_errors_max={max(m['errors'] for m in mixed_metrics)}\n")
        out.write(f"bot_mixed_actions_per_sec_max={max(m['actions_per_sec'] for m in mixed_metrics):.1f}\n")
    out.write(f"compat_probe_block_actions_observed={str(compat_probe_block_actions_observed).lower()}\n")
    out.write(f"compat_probe_block_metrics_seen={str(bool(compat_block_metrics)).lower()}\n")
    out.write(f"compat_probe_block_metrics_periodic_counts_seen={str(compat_probe_periodic_block_counts_seen).lower()}\n")
    out.write(f"compat_probe_block_metrics_direct_evidence={str(compat_probe_direct_block_event_evidence).lower()}\n")
    out.write(f"compat_probe_block_metrics_loadbot_direct_evidence={str(compat_probe_block_metrics_loadbot_direct_evidence).lower()}\n")
    out.write(f"compat_probe_block_evidence_accepted={str(compat_probe_block_evidence_accepted).lower()}\n")
    out.write(f"compat_probe_block_metrics_after_actions={str(compat_probe_block_metrics_after_actions).lower()}\n")
    out.write(f"compat_probe_block_metrics_stale_after_actions={str(compat_probe_block_metrics_stale_after_actions).lower()}\n")
    out.write(f"compat_probe_block_metrics_ambiguous_after_actions={str(compat_probe_block_metrics_ambiguous_after_actions).lower()}\n")
    out.write(f"compat_probe_periodic_block_places_max={compat_probe_periodic_block_places_max}\n")
    out.write(f"compat_probe_periodic_block_breaks_max={compat_probe_periodic_block_breaks_max}\n")
    out.write(f"compat_probe_block_event_samples={len(compat_direct_block_events)}\n")
    out.write(f"compat_probe_block_event_loadbot_samples={len(compat_direct_loadbot_block_events)}\n")
    out.write(f"compat_probe_block_event_places_max={compat_probe_block_event_places_max}\n")
    out.write(f"compat_probe_block_event_breaks_max={compat_probe_block_event_breaks_max}\n")
    out.write(f"compat_probe_block_event_loadbot_places_max={compat_probe_block_event_loadbot_places_max}\n")
    out.write(f"compat_probe_block_event_loadbot_breaks_max={compat_probe_block_event_loadbot_breaks_max}\n")
    out.write(f"compat_probe_direct_block_event_lines={len(compat_direct_block_events)}\n")
    out.write(f"compat_probe_direct_block_loadbot_event_lines={len(compat_direct_loadbot_block_events)}\n")
    out.write(f"compat_probe_direct_block_places_max={compat_probe_block_event_places_max}\n")
    out.write(f"compat_probe_direct_block_breaks_max={compat_probe_block_event_breaks_max}\n")
    if compat_direct_block_events:
        out.write(f"compat_probe_direct_block_place_event_lines={sum(1 for m in compat_direct_block_events if m['type'] == 'place')}\n")
        out.write(f"compat_probe_direct_block_break_event_lines={sum(1 for m in compat_direct_block_events if m['type'] == 'break')}\n")
        out.write(f"compat_probe_direct_block_cancelled_true_lines={sum(1 for m in compat_direct_block_events if m['cancelled'])}\n")
        out.write(f"compat_probe_direct_block_cancelled_false_lines={sum(1 for m in compat_direct_block_events if not m['cancelled'])}\n")
        out.write(f"compat_probe_direct_block_players={len({m['player'] for m in compat_direct_block_events})}\n")
    out.write(f"compat_probe_direct_block_loadbot_place_event_lines={compat_probe_direct_block_loadbot_place_event_lines}\n")
    out.write(f"compat_probe_direct_block_loadbot_break_event_lines={compat_probe_direct_block_loadbot_break_event_lines}\n")
    out.write(f"compat_probe_direct_block_loadbot_cancelled_true_lines={compat_probe_direct_block_loadbot_cancelled_true_lines}\n")
    out.write(f"compat_probe_direct_block_loadbot_cancelled_false_lines={compat_probe_direct_block_loadbot_cancelled_false_lines}\n")
    out.write(f"compat_probe_direct_block_loadbot_players={compat_probe_direct_block_loadbot_players}\n")
    if compat_block_metrics or compat_direct_block_events:
        out.write(f"compat_probe_block_places_max={compat_probe_block_places_max}\n")
        out.write(f"compat_probe_block_breaks_max={compat_probe_block_breaks_max}\n")
    if compat_block_metrics:
        out.write(f"compat_probe_arena_commands_max={max(m['arena'] for m in compat_block_metrics)}\n")
        out.write(f"compat_probe_arena_prepared_max={max(m['arena_prepared'] for m in compat_block_metrics)}\n")
        out.write(f"compat_probe_arena_skipped_max={max(m['arena_skipped'] for m in compat_block_metrics)}\n")
    if compat_arena_chunk_metrics:
        out.write(f"compat_probe_arena_chunk_loads_queued_max={max(m['queued'] for m in compat_arena_chunk_metrics)}\n")
        out.write(f"compat_probe_arena_chunk_loads_started_max={max(m['started'] for m in compat_arena_chunk_metrics)}\n")
        out.write(f"compat_probe_arena_chunk_loads_completed_max={max(m['completed'] for m in compat_arena_chunk_metrics)}\n")
        out.write(f"compat_probe_arena_chunk_loads_failed_max={max(m['failed'] for m in compat_arena_chunk_metrics)}\n")
        out.write(f"compat_probe_arena_chunk_loads_in_flight_max={max(m['in_flight'] for m in compat_arena_chunk_metrics)}\n")
    if arena_preload_metrics:
        out.write(f"compat_probe_arena_preload_complete_lines={len(arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_expected_max={max(m['expected'] for m in arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_radius_chunks_max={max(m['radius_chunks'] for m in arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_max_in_flight_max={max(m['max_in_flight'] for m in arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_chunks_max={max(m['chunks'] for m in arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_loaded_max={max(m['loaded'] for m in arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_failed_max={max(m['failed'] for m in arena_preload_metrics)}\n")
        out.write(f"compat_probe_arena_preload_tickets_max={max(m['tickets'] for m in arena_preload_metrics)}\n")
    if arena_command_metrics:
        out.write(f"compat_probe_arena_command_lines={len(arena_command_metrics)}\n")
        out.write(f"compat_probe_arena_matched_max={max(m['matched'] for m in arena_command_metrics)}\n")
        out.write(f"compat_probe_arena_prepared_total={sum(m['prepared'] for m in arena_command_metrics)}\n")
        out.write(f"compat_probe_arena_teleported_total={sum(m['teleported'] for m in arena_command_metrics)}\n")
        out.write(f"compat_probe_arena_skipped_total={sum(m['skipped'] for m in arena_command_metrics)}\n")
    if compat_mob_metrics:
        out.write(f"compat_probe_mobstorm_commands_max={max(m['commands'] for m in compat_mob_metrics)}\n")
        out.write(f"compat_probe_mobstorm_spawned_max={max(m['spawned'] for m in compat_mob_metrics)}\n")
        out.write(f"compat_probe_living_entities_max={max(m['living'] for m in compat_mob_metrics)}\n")
    if compat_mixed_metrics:
        out.write(f"compat_probe_commands_max={max(m['commands'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_player_commands_max={max(m['player_commands'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_item_held_events_max={max(m['held'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_animation_events_max={max(m['animation'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_interact_events_max={max(m['interact'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_entity_damage_events_max={max(m['damage'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_toggle_sprint_events_max={max(m['sprint'] for m in compat_mixed_metrics)}\n")
        out.write(f"compat_probe_toggle_sneak_events_max={max(m['sneak'] for m in compat_mixed_metrics)}\n")
    if mobstorm_command_metrics:
        out.write(f"compat_probe_mobstorm_command_lines={len(mobstorm_command_metrics)}\n")
        out.write(f"compat_probe_mobstorm_requested_total={sum(m['requested'] for m in mobstorm_command_metrics)}\n")
        out.write(f"compat_probe_mobstorm_spawned_total={sum(m['spawned'] for m in mobstorm_command_metrics)}\n")
    out.write(f"server_join_events={server_join_events}\n")
    out.write(f"server_quit_events={server_quit_events}\n")
    if resources:
        out.write(f"resource_samples={len(resources)}\n")
        out.write(f"process_cpu_max={max(resource['cpu'] for resource in resources):.2f}\n")
        out.write(f"process_rss_mib_max={max(resource['rss_kb'] for resource in resources) / 1024:.1f}\n")
        load1_values = [resource["system_load1"] for resource in resources if "system_load1" in resource]
        mem_values = [
            resource["system_mem_available_kb"]
            for resource in resources
            if "system_mem_available_kb" in resource
        ]
        if load1_values:
            cpu_count = effective_cpu_count()
            out.write(f"host_cpu_count={cpu_count}\n")
            out.write(f"host_system_load1_max={max(load1_values):.2f}\n")
            out.write(f"host_system_load1_per_cpu_max={max(load1_values) / cpu_count:.3f}\n")
        if mem_values:
            out.write(f"host_mem_available_kb_min={min(mem_values)}\n")
        host_cpu_windows = []
        for previous, current in zip(resources, resources[1:]):
            required = (
                "host_cpu_total",
                "host_cpu_idle",
                "host_cpu_iowait",
                "host_cpu_steal",
            )
            if not all(key in previous and key in current for key in required):
                continue
            total_delta = current["host_cpu_total"] - previous["host_cpu_total"]
            if total_delta <= 0:
                continue
            host_cpu_windows.append({
                "idle_percent": 100.0 * (current["host_cpu_idle"] - previous["host_cpu_idle"]) / total_delta,
                "iowait_percent": 100.0 * (current["host_cpu_iowait"] - previous["host_cpu_iowait"]) / total_delta,
                "steal_percent": 100.0 * (current["host_cpu_steal"] - previous["host_cpu_steal"]) / total_delta,
            })
        if host_cpu_windows:
            out.write(f"host_cpu_windows={len(host_cpu_windows)}\n")
            out.write(f"host_cpu_idle_percent_min={min(m['idle_percent'] for m in host_cpu_windows):.2f}\n")
            out.write(f"host_cpu_iowait_percent_max={max(m['iowait_percent'] for m in host_cpu_windows):.2f}\n")
            out.write(f"host_cpu_iowait_percent_avg={statistics.fmean(m['iowait_percent'] for m in host_cpu_windows):.2f}\n")
            out.write(f"host_cpu_steal_percent_max={max(m['steal_percent'] for m in host_cpu_windows):.2f}\n")
            out.write(f"host_cpu_steal_percent_avg={statistics.fmean(m['steal_percent'] for m in host_cpu_windows):.2f}\n")
    out.write(f"moved_too_quickly_warnings={moved_too_quickly}\n")
    out.write(f"watchdog_thread_dumps={watchdog_dumps}\n")
    out.write(f"sync_load_stack_hits={sync_load_stack_hits}\n")
    out.write(f"nearby_players_stack_hits={nearby_players_stack_hits}\n")
    out.write(f"thread_check_failures={thread_check_failures}\n")
    out.write(f"chunk_system_errors={chunk_system_errors}\n")
    out.write(f"feature_placement_errors={feature_placement_errors}\n")
    out.write(f"off_main_poi_hits={off_main_poi_hits}\n")
    stability_failures = thread_check_failures + chunk_system_errors + feature_placement_errors + off_main_poi_hits
    if block_metrics:
        stability_failures += max(m['errors'] for m in block_metrics)
    if mixed_metrics:
        stability_failures += max(m['errors'] for m in mixed_metrics)
    out.write(f"stability_failures={stability_failures}\n")
    dumps = sorted(pathlib.Path(jstack_dir).glob("thread-print-*.txt"))
    out.write(f"external_thread_prints={len(dumps)}\n")
    if dumps:
        out.write(f"external_thread_print_dir={jstack_dir}\n")
    samples = sorted(pathlib.Path(jstack_dir).glob("thread-sample-*.txt"))
    out.write(f"diagnostic_thread_samples={len(samples)}\n")
    if samples:
        out.write(f"diagnostic_thread_sample_dir={jstack_dir}\n")
    out.write("bot_log_tail:\n")
    for line in bot_tail:
        out.write(line + "\n")
print(open(summary, encoding="utf-8").read(), end="")
PY

if [[ -d "$MEMORY_SNAPSHOT_DIR" ]]; then
  memory_snapshot_count="$(find "$MEMORY_SNAPSHOT_DIR" -maxdepth 1 -type f -name 'peak-*.txt' ! -name 'peak-latest.txt' | wc -l | tr -d '[:space:]')"
  {
    echo "memory_snapshot_dir=$MEMORY_SNAPSHOT_DIR"
    echo "memory_snapshot_count=$memory_snapshot_count"
    if [[ -f "$MEMORY_SNAPSHOT_DIR/peak-latest.txt" ]]; then
      echo "memory_peak_snapshot=$MEMORY_SNAPSHOT_DIR/peak-latest.txt"
      awk -F= '/^rss_kb=/{print "memory_peak_snapshot_rss_kb=" $2; found=1; exit} END {if (!found) print "memory_peak_snapshot_rss_kb=unknown"}' "$MEMORY_SNAPSHOT_DIR/peak-latest.txt"
      awk -F= '/^rss_mib=/{print "memory_peak_snapshot_rss_mib=" $2; found=1; exit} END {if (!found) print "memory_peak_snapshot_rss_mib=unknown"}' "$MEMORY_SNAPSHOT_DIR/peak-latest.txt"
    fi
  } >> "$SUMMARY"
fi

native_gate_exit=0
set +e
python3 "$ROOT/scripts/evaluate_load_native_gate.py" \
  --profile "${LOAD_TEST_GATE_PROFILE:-}" \
  "$SUMMARY"
native_gate_exit=$?
set -e
if (( native_gate_exit != 0 )); then
  printf 'native_gate_evaluation_exit=%s\n' "$native_gate_exit" >> "$SUMMARY"
fi

early_abort_reason=""
if [[ -s "$EARLY_ABORT_REASON" ]]; then
  early_abort_reason="$(tr '\n ' '__' < "$EARLY_ABORT_REASON" | sed 's/_*$//')"
elif [[ -s "$DIAGNOSTIC_REASON" ]]; then
  early_abort_reason="$(tr '\n ' '__' < "$DIAGNOSTIC_REASON" | sed 's/_*$//')"
fi
if [[ -n "$early_abort_reason" ]]; then
  tmp_summary="$SUMMARY.tmp"
  awk -v early_abort_reason="$early_abort_reason" '
    BEGIN { inserted = 0 }
    /^early_abort_reason=/ { next }
    /^bot_log_tail:/ && inserted == 0 {
      print "early_abort_reason=" early_abort_reason
      inserted = 1
    }
    { print }
    END {
      if (inserted == 0) {
        print "early_abort_reason=" early_abort_reason
      }
    }
  ' "$SUMMARY" > "$tmp_summary"
  mv "$tmp_summary" "$SUMMARY"
fi
cat "$SUMMARY"
gate_exit=0
if [[ -n "${LOAD_TEST_GATE_PROFILE:-}" ]]; then
  LOAD_TEST_GATE_REPORT="${LOAD_TEST_GATE_REPORT:-$ROOT/reports/load-${LOAD_TEST_LABEL}-gate.txt}"
  if python3 "$ROOT/scripts/evaluate_load_gate.py" \
    --profile "$LOAD_TEST_GATE_PROFILE" \
    --report "$LOAD_TEST_GATE_REPORT" \
    "$SUMMARY"; then
    gate_exit=0
  else
    gate_exit=$?
  fi
fi
if [[ "$bot_exit" -ne 0 ]]; then
  exit "$bot_exit"
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" == production-* ]] && awk -F= '$1 == "stability_failures" && ($2 + 0) > 0 { found = 1 } END { exit found ? 0 : 1 }' "$SUMMARY"; then
  echo "Production load test failed stability gate; see $SUMMARY" >&2
  exit 1
fi
if [[ "${LOAD_TEST_GATE_PROFILE:-}" != production-* && "${LOAD_TEST_ALLOW_BUSY_HOST:-false}" != "true" ]] && awk -F= '$1 == "stability_failures" && ($2 + 0) > 0 { found = 1 } END { exit found ? 0 : 1 }' "$SUMMARY"; then
  echo "Load test failed stability gate; see $SUMMARY" >&2
  exit 1
fi
if [[ "$gate_exit" -ne 0 ]]; then
  exit "$gate_exit"
fi
exit 0
