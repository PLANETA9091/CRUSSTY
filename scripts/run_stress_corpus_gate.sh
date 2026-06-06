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
JAVA_OPTS_STRESS_CORPUS="${JAVA_OPTS_STRESS_CORPUS:--Xms4G -Xmx16G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100}"
TASKSET_CMD=()
if [[ -n "${BENCHMARK_CPUSET:-}" ]]; then
  if ! command -v taskset >/dev/null 2>&1; then
    echo "BENCHMARK_CPUSET is set but taskset is unavailable." >&2
    exit 1
  fi
  TASKSET_CMD=(taskset -c "$BENCHMARK_CPUSET")
fi

if [[ $# -gt 0 ]]; then
  LAUNCHER="$1"
elif [[ -x "$ROOT/artifacts/optimized-runtime/run.sh" ]]; then
  LAUNCHER="$ROOT/artifacts/optimized-runtime/run.sh"
else
  LAUNCHER="$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"
fi
LAUNCHER="$(resolve_launcher_path "$LAUNCHER")"

RUN="$ROOT/runs/stress-corpus-gate"
LOG="$ROOT/logs/stress-corpus-gate.log"
FIFO="$RUN/console.in"
SERVER_READY_REGEX="$(load_test_server_ready_regex)"
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
SERVERPULSE_INFLUX_LOG="$ROOT/logs/stress-corpus-influx-mock.log"
SERVERPULSE_INFLUX_PID=""

ALLOWED_STRESS_ERROR_REGEX='^[0-9]+:\[[0-9]{2}:[0-9]{2}:[0-9]{2} ERROR\]: \[Essentials\] You are running an unsupported server version!$'
HARD_LOG_PATTERN="Could not load plugin|Could not find library|Failed to load data pack|Couldn't load data pack|Exception in server tick loop|OutOfMemoryError|NoClassDefFoundError|UnsupportedClassVersionError|ServerChunkCache\\.syncLoad|DO NOT REPORT THIS TO PAPER|InfluxDB connection data is missing or invalid|Missing, empty, or default 'metrics\\.influxdb\\.token'|ERROR"

count_glob() {
  local pattern="$1"
  compgen -G "$pattern" | wc -l | tr -d ' '
}

last_done_seconds() {
  if [[ ! -f "$LOG" ]]; then
    return 0
  fi
  rg "$SERVER_READY_REGEX" "$LOG" | sed -E 's/.*Done \(([0-9.]+)s\)!.*/\1/' | tail -n 1 || true
}

artifact_summary_lines() {
  python3 - "$ROOT" "$LAUNCHER" <<'PY' || true
from __future__ import annotations

import json
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
launcher = Path(sys.argv[2]).resolve()
artifacts_json = root / "reports" / "artifacts.json"

print(f"launcher={launcher}")
print(f"artifacts_json={artifacts_json}")

if not artifacts_json.is_file():
    print("artifact_summary_error=artifacts_json_missing")
    raise SystemExit(0)

try:
    data = json.loads(artifacts_json.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    print(f"artifact_summary_error=artifacts_json_invalid:{exc}")
    raise SystemExit(0)

optimized = data.get("optimized") if isinstance(data, dict) else {}
runtime = data.get("optimized_runtime") if isinstance(data, dict) else {}
if not isinstance(optimized, dict):
    optimized = {}
if not isinstance(runtime, dict):
    runtime = {}

optimized_path = optimized.get("path")
if optimized_path:
    print(f"optimized_artifact_path={optimized_path}")
print(f"optimized_artifact_sha256={optimized.get('sha256', '')}")

run_sh = runtime.get("run_sh")
if not isinstance(run_sh, dict):
    run_sh = {}
run_sh_path = run_sh.get("path")
if run_sh_path:
    print(f"optimized_runtime_run_sh={run_sh_path}")
print(f"optimized_runtime_run_sh_sha256={run_sh.get('sha256', '')}")

runtime_jar = runtime.get("runtime_jar_sha256_file")
if isinstance(runtime_jar, dict):
    print(f"optimized_runtime_jar_sha256={runtime_jar.get('runtime_jar_sha256', '')}")

native_library = runtime.get("native_library")
if isinstance(native_library, dict):
    print(f"optimized_runtime_native_library={native_library.get('path', '')}")
    print(f"optimized_runtime_native_library_sha256={native_library.get('sha256', '')}")

launcher_kind = "unknown"
try:
    if run_sh_path and launcher == Path(str(run_sh_path)).resolve():
        launcher_kind = "optimized-runtime"
    elif optimized_path and launcher == Path(str(optimized_path)).resolve():
        launcher_kind = "optimized-paper"
except OSError:
    pass
print(f"launcher_kind={launcher_kind}")
PY
}

hard_log_lines() {
  if [[ ! -f "$LOG" ]]; then
    return 0
  fi
  rg -n "$HARD_LOG_PATTERN" "$LOG" | rg -v "$ALLOWED_STRESS_ERROR_REGEX" || true
}

has_hard_log_lines() {
  [[ -f "$LOG" ]] || return 1
  hard_log_lines | rg -q '.'
}

write_summary() {
  local result="$1"
  local reason="${2:-none}"
  {
    echo "stress_corpus_gate=$result"
    echo "failure_reason=$reason"
    echo "matrix_plugin_count=$(count_glob "$ROOT/plugins/matrix/*.jar")"
    echo "stress_plugin_count=$(count_glob "$ROOT/plugins/stress/*.jar")"
    echo "plugin_count=$(( $(count_glob "$ROOT/plugins/matrix/*.jar") + $(count_glob "$ROOT/plugins/stress/*.jar") ))"
    echo "datapack_count=$(count_glob "$ROOT/datapacks/stress/*.zip")"
    echo "done_seconds=$(last_done_seconds)"
    echo "java_opts=$JAVA_OPTS_STRESS_CORPUS"
    artifact_summary_lines
    echo "server_port=$PORT"
    echo "squaremap_port=$SQUAREMAP_PORT"
    echo "bluemap_port=$BLUEMAP_PORT"
    echo "voicechat_port=$VOICECHAT_PORT"
    echo "serverpulse_influx_port=$SERVERPULSE_INFLUX_PORT"
    echo "log=$LOG"
    echo "serverpulse_influx_log=$SERVERPULSE_INFLUX_LOG"
    echo "status_json=$ROOT/reports/stress-corpus-status.json"
    echo "join_report=$ROOT/reports/stress-corpus-join.txt"
    echo "hard_errors=$ROOT/reports/stress-corpus-hard-errors.txt"
    echo "inspection_txt=$ROOT/reports/stress-corpus-inspection.txt"
    echo "inspection_json=$ROOT/reports/stress-corpus-inspection.json"
    echo "artifacts_csv=$ROOT/reports/stress-corpus-artifacts.csv"
    echo "stress_manifest=$ROOT/reports/stress-corpus-manifest.json"
    if [[ -f "$LOG" ]]; then
      rg -n \
        -e "$SERVER_READY_REGEX" \
        -e "Could not load plugin|Could not find library|Failed to load data pack|Couldn't load data pack|StressProbe|plugins\\]|DataPacks|datapack|BindException|OutOfMemoryError|ServerChunkCache\\.syncLoad|DO NOT REPORT THIS TO PAPER|ERROR|Exception" \
        "$LOG" || true
    fi
  } > "$ROOT/reports/stress-corpus-summary.txt"
}

fail_gate() {
  local reason="$1"
  hard_log_lines > "$ROOT/reports/stress-corpus-hard-errors.txt"
  write_summary "FAIL" "$reason"
  tail -n 240 "$LOG" >&2 || true
  exit 1
}

mkdir -p "$ROOT/logs" "$ROOT/reports"

if [[ ! -d "$ROOT/plugins/matrix" ]] || ! compgen -G "$ROOT/plugins/matrix/*.jar" > /dev/null; then
  echo "Missing matrix plugins. Run scripts/fetch_matrix_plugins.py or restore plugins/matrix first." >&2
  write_summary "FAIL" "missing_matrix_plugins"
  exit 1
fi
if [[ ! -d "$ROOT/plugins/stress" ]] || ! compgen -G "$ROOT/plugins/stress/*.jar" > /dev/null; then
  echo "Missing stress plugins. Run scripts/fetch_stress_corpus.py first." >&2
  write_summary "FAIL" "missing_stress_plugins"
  exit 1
fi
if [[ ! -d "$ROOT/datapacks/stress" ]] || ! compgen -G "$ROOT/datapacks/stress/*.zip" > /dev/null; then
  echo "Missing stress datapacks. Run scripts/fetch_stress_corpus.py first." >&2
  write_summary "FAIL" "missing_stress_datapacks"
  exit 1
fi

rm -rf "$RUN"
mkdir -p "$RUN/plugins" "$RUN/world/datapacks"
rm -f \
  "$LOG" \
  "$SERVERPULSE_INFLUX_LOG" \
  "$ROOT/reports/stress-corpus-summary.txt" \
  "$ROOT/reports/stress-corpus-status.json" \
  "$ROOT/reports/stress-corpus-status.json.tmp" \
  "$ROOT/reports/stress-corpus-join.txt" \
  "$ROOT/reports/stress-corpus-hard-errors.txt"
cp "$ROOT/plugins/matrix"/*.jar "$RUN/plugins/"
if compgen -G "$ROOT/plugins/matrix-libraries/*" > /dev/null; then
  cp -R "$ROOT/plugins/matrix-libraries"/. "$RUN/"
fi
cp "$ROOT/plugins/stress"/*.jar "$RUN/plugins/"
cp "$ROOT/datapacks/stress"/*.zip "$RUN/world/datapacks/"

mkdir -p "$RUN/plugins/spark" "$RUN/plugins/squaremap" "$RUN/plugins/BlueMap" "$RUN/plugins/voicechat"
cat > "$RUN/plugins/spark/config.json" <<SPARKCFG
{
  "_header": "stress corpus gate keeps spark background profiler disabled for comparable boot/load diagnostics",
  "backgroundProfiler": false
}
SPARKCFG
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
    org: stress-corpus
    bucket: stress_metrics
    token: stress-corpus-local-token
    table: minecraft_stats
  tags:
    server: "stress-corpus-gate"
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
seed_luckperms_vault_unsafe_lookups "$RUN" "$ROOT"

cat > "$RUN/eula.txt" <<'EULA'
# Explicitly enabled by scripts/run_stress_corpus_gate.sh only when MC_EULA_AGREE=true.
eula=true
EULA
cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
max-players=40
level-seed=paper-rust-stress-corpus-12110
gamemode=survival
force-gamemode=false
view-distance=6
simulation-distance=4
allow-flight=true
enable-rcon=false
generate-structures=true
motd=Paper Rust Stress Corpus Gate
PROPS
cat > "$RUN/bukkit.yml" <<BUKKIT
settings:
  allow-end: true
  warn-on-overload: true
  permissions-file: permissions.yml
  update-folder: update
  plugin-profiling: false
  connection-throttle: 0
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

rm -f "$FIFO"
mkfifo "$FIFO"
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
print(f"stress_corpus_influx_mock=listening host=127.0.0.1 port={port}", flush=True)
server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
server.serve_forever()
PY
SERVERPULSE_INFLUX_PID=$!

if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("${TASKSET_CMD[@]}" "$LAUNCHER" --nogui)
else
  CMD=("${TASKSET_CMD[@]}" java $JAVA_OPTS_STRESS_CORPUS -jar "$LAUNCHER" --nogui)
fi
(
  cd "$RUN"
  JAVA_OPTS="$JAVA_OPTS_STRESS_CORPUS" "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
  echo $! > server.pid
)
pid="$(cat "$RUN/server.pid")"
exec 3>"$FIFO"
trap '' PIPE

server_is_running() {
  kill -0 "$pid" 2>/dev/null
}

send_console_command() {
  if ! server_is_running; then
    return 0
  fi
  printf '%s\n' "$*" >&3 2>/dev/null || true
}

ensure_server_running() {
  local reason="$1"
  if ! server_is_running; then
    fail_gate "$reason"
  fi
}

cleanup() {
  if server_is_running; then
    send_console_command stop
    sleep 5
    kill "$pid" 2>/dev/null || true
  fi
  if [[ -n "${SERVERPULSE_INFLUX_PID:-}" ]] && kill -0 "$SERVERPULSE_INFLUX_PID" 2>/dev/null; then
    kill "$SERVERPULSE_INFLUX_PID" 2>/dev/null || true
    wait "$SERVERPULSE_INFLUX_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

deadline=$((SECONDS + 240))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if has_hard_log_lines; then
    fail_gate "startup_hard_error"
  fi
  ensure_server_running "server_exited_before_ready"
  sleep 1
done
if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  fail_gate "startup_timeout"
fi
hard_log_lines > "$ROOT/reports/stress-corpus-hard-errors.txt"
if [[ -s "$ROOT/reports/stress-corpus-hard-errors.txt" ]]; then
  fail_gate "startup_hard_error"
fi

status_tmp="$ROOT/reports/stress-corpus-status.json.tmp"
rm -f "$status_tmp"
for attempt in $(seq 1 12); do
  ensure_server_running "server_exited_before_status"
  if timeout 15s python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$PORT" 773 10 > "$status_tmp"; then
    mv "$status_tmp" "$ROOT/reports/stress-corpus-status.json"
    break
  fi
  ensure_server_running "server_exited_during_status"
  if (( attempt == 12 )); then
    fail_gate "status_ping_failed"
  fi
  sleep 2
done

join_report="$ROOT/reports/stress-corpus-join.txt"
rm -f "$join_report"
for attempt in $(seq 1 8); do
  ensure_server_running "server_exited_before_join"
  {
    echo "join_attempt=$attempt"
    "$ROOT/scripts/mc_join_client.sh" 127.0.0.1 "$PORT" StressProbe 45000
  } > "$join_report.tmp" 2>&1 && {
    mv "$join_report.tmp" "$join_report"
    break
  }
  cat "$join_report.tmp" >> "$join_report"
  rm -f "$join_report.tmp"
  ensure_server_running "server_exited_during_join_probe"
  if (( attempt == 8 )); then
    cat "$join_report" >&2 || true
    fail_gate "join_probe_failed"
  fi
  sleep 3
done

deadline=$((SECONDS + 20))
while (( SECONDS < deadline )); do
  if rg -q "StressProbe joined the game|COMPAT_PROBE event=PlayerJoinEvent .*detail=StressProbe" "$LOG"; then
    break
  fi
  sleep 1
done
if ! rg -q "StressProbe joined the game|COMPAT_PROBE event=PlayerJoinEvent .*detail=StressProbe" "$LOG"; then
  cat "$ROOT/reports/stress-corpus-join.txt" >&2 || true
  fail_gate "join_probe_timeout"
fi

send_console_command plugins
send_console_command "datapack list"
send_console_command "save-all flush"
sleep 8

hard_log_lines > "$ROOT/reports/stress-corpus-hard-errors.txt"
if [[ -s "$ROOT/reports/stress-corpus-hard-errors.txt" ]]; then
  fail_gate "post_ready_hard_error"
fi

send_console_command stop
exec 3>&-

deadline=$((SECONDS + 180))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  fail_gate "shutdown_timeout"
fi
if [[ -n "${SERVERPULSE_INFLUX_PID:-}" ]] && kill -0 "$SERVERPULSE_INFLUX_PID" 2>/dev/null; then
  kill "$SERVERPULSE_INFLUX_PID" 2>/dev/null || true
  wait "$SERVERPULSE_INFLUX_PID" 2>/dev/null || true
fi
trap - EXIT

write_summary "PASS" "none"
cat "$ROOT/reports/stress-corpus-summary.txt"
