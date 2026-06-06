#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to set eula=true without explicit MC_EULA_AGREE=true." >&2
  echo "This is a license gate, not a technical failure." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/launcher_utils.sh"
source "$ROOT/scripts/load_test_defaults.sh"
SERVER_READY_REGEX="$(load_test_server_ready_regex)"
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
RUN="$ROOT/runs/plugin-matrix"
LOG="$ROOT/logs/plugin-matrix.log"
FIFO="$RUN/console.in"
PORT="${SERVER_PORT:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"

rm -rf "$RUN"
mkdir -p "$RUN/plugins" "$ROOT/logs" "$ROOT/reports"
cp "$ROOT/plugins/matrix"/*.jar "$RUN/plugins/"
if compgen -G "$ROOT/plugins/matrix-libraries/*" > /dev/null; then
  cp -R "$ROOT/plugins/matrix-libraries"/. "$RUN/"
fi
cat > "$RUN/eula.txt" <<'EULA'
# Explicitly enabled by scripts/run_plugin_matrix.sh only when MC_EULA_AGREE=true.
eula=true
EULA
cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
level-seed=paper-rust-compat-12110
view-distance=6
simulation-distance=4
enable-rcon=false
motd=Paper Rust Compat Matrix
PROPS
echo "$PORT" > "$ROOT/reports/plugin-matrix-port.txt"

mkfifo "$FIFO"
if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("${TASKSET_CMD[@]}" "$LAUNCHER" --nogui)
else
  CMD=("${TASKSET_CMD[@]}" java -Xms1G -Xmx2G -jar "$LAUNCHER" --nogui)
fi
(
  cd "$RUN"
  "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
  echo $! > server.pid
)

exec 3>"$FIFO"

deadline=$((SECONDS + 180))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if rg -q "Failed to bind to port|Encountered an unexpected exception|UnsupportedClassVersionError|Could not load plugin|Failed to start" "$LOG"; then
    tail -n 120 "$LOG" >&2
    exit 1
  fi
  sleep 1
done

if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  tail -n 160 "$LOG" >&2
  exit 1
fi

status_tmp="$ROOT/reports/plugin-matrix-status.json.tmp"
rm -f "$status_tmp"
for attempt in $(seq 1 12); do
  if python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$PORT" 773 10 > "$status_tmp"; then
    mv "$status_tmp" "$ROOT/reports/plugin-matrix-status.json"
    break
  fi
  if (( attempt == 12 )); then
    tail -n 180 "$LOG" >&2
    exit 1
  fi
  sleep 2
done
"$ROOT/scripts/mc_join_client.sh" 127.0.0.1 "$PORT" CodexJoinProbe 45000 \
  > "$ROOT/reports/plugin-matrix-join.txt" 2>&1
deadline=$((SECONDS + 20))
while (( SECONDS < deadline )); do
  if rg -q "COMPAT_PROBE event=PlayerJoinEvent .*detail=CodexJoinProbe|CodexJoinProbe joined the game" "$LOG"; then
    break
  fi
  sleep 1
done
if ! rg -q "COMPAT_PROBE event=PlayerJoinEvent .*detail=CodexJoinProbe|CodexJoinProbe joined the game" "$LOG"; then
  cat "$ROOT/reports/plugin-matrix-join.txt" >&2 || true
  tail -n 180 "$LOG" >&2
  exit 1
fi
printf 'plugins\n' >&3
printf 'version\n' >&3
printf 'compatprobe\n' >&3
printf 'save-all flush\n' >&3
sleep 8
printf 'stop\n' >&3
exec 3>&-

pid="$(cat "$RUN/server.pid")"
deadline=$((SECONDS + 90))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  tail -n 160 "$LOG" >&2
  exit 1
fi

{
  echo "plugin_matrix_log=$LOG"
  echo "plugin_matrix_port=$PORT"
  echo "status_json=$ROOT/reports/plugin-matrix-status.json"
  rg -n \
    -e "$SERVER_READY_REGEX" \
    -e "COMPAT_PROBE|Enabling|Disabling|ERROR|WARN|Exception|plugins" \
    "$LOG" || true
} > "$ROOT/reports/plugin-matrix-summary.txt"
cat "$ROOT/reports/plugin-matrix-summary.txt"
