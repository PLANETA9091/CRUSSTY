#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/launcher_utils.sh"
source "$ROOT/scripts/load_test_defaults.sh"
SERVER_READY_REGEX="$(load_test_server_ready_regex)"
if [[ $# -gt 0 ]]; then
  LAUNCHER="$1"
elif [[ -x "$ROOT/artifacts/optimized-runtime/run.sh" ]]; then
  LAUNCHER="$ROOT/artifacts/optimized-runtime/run.sh"
else
  LAUNCHER="$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"
fi
LAUNCHER="$(resolve_launcher_path "$LAUNCHER")"
RUN="$ROOT/runs/plugin-matrix"
LOG="$ROOT/logs/restart-recovery.log"
FIFO="$RUN/restart-console.in"
PORT="${SERVER_PORT:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"

if [[ ! -f "$RUN/eula.txt" || ! -d "$RUN/world" ]]; then
  echo "Missing prior plugin-matrix world. Run scripts/run_plugin_matrix.sh first." >&2
  exit 1
fi

cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
level-seed=paper-rust-compat-12110
view-distance=6
simulation-distance=4
enable-rcon=false
motd=Paper Rust Compat Matrix Restart
PROPS

rm -f "$FIFO"
mkfifo "$FIFO"
if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("$LAUNCHER" --nogui)
else
  CMD=(java -Xms1G -Xmx2G -jar "$LAUNCHER" --nogui)
fi
(
  cd "$RUN"
  "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
  echo $! > restart.pid
)

exec 3>"$FIFO"
deadline=$((SECONDS + 180))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if rg -q "Failed to bind to port|Encountered an unexpected exception|UnsupportedClassVersionError|Could not load plugin|Failed to start" "$LOG"; then
    tail -n 140 "$LOG" >&2
    exit 1
  fi
  sleep 1
done

if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$PORT" > "$ROOT/reports/restart-recovery-status.json"
printf 'compatprobe\n' >&3
printf 'save-all flush\n' >&3
sleep 8
printf 'stop\n' >&3
exec 3>&-

pid="$(cat "$RUN/restart.pid")"
deadline=$((SECONDS + 90))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

if ! find "$RUN/world" -type f -name '*.mca' | rg -q .; then
  echo "No region files found after restart/save." >&2
  exit 1
fi

{
  echo "restart_recovery_log=$LOG"
  echo "restart_recovery_port=$PORT"
  echo "status_json=$ROOT/reports/restart-recovery-status.json"
  rg -n \
    -e "$SERVER_READY_REGEX" \
    -e "COMPAT_PROBE|Enabling|Disabling|ERROR|WARN|Exception|Saved the game" \
    "$LOG" || true
} > "$ROOT/reports/restart-recovery-summary.txt"
cat "$ROOT/reports/restart-recovery-summary.txt"
