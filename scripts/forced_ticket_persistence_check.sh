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
LOG_CLEAN_HELPER="$ROOT/scripts/check_runtime_log_clean.sh"
if [[ $# -gt 0 ]]; then
  LAUNCHER="$1"
elif [[ -x "$ROOT/artifacts/optimized-runtime/run.sh" ]]; then
  LAUNCHER="$ROOT/artifacts/optimized-runtime/run.sh"
else
  LAUNCHER="$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"
fi
LAUNCHER="$(resolve_launcher_path "$LAUNCHER")"

RUN="$ROOT/runs/forced-ticket-persistence"
LOG1="$ROOT/logs/forced-ticket-persistence-first.log"
LOG2="$ROOT/logs/forced-ticket-persistence-restart.log"
CLEAN1="$ROOT/reports/forced-ticket-persistence-first-log-clean.txt"
CLEAN2="$ROOT/reports/forced-ticket-persistence-restart-log-clean.txt"
PORT1="${SERVER_PORT_FIRST:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"
PORT2="${SERVER_PORT_RESTART:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"

rm -rf "$RUN"
mkdir -p "$RUN" "$ROOT/logs" "$ROOT/reports"
cat > "$RUN/eula.txt" <<'EULA'
# Explicitly enabled by scripts/forced_ticket_persistence_check.sh only when MC_EULA_AGREE=true.
eula=true
EULA

write_properties() {
  local port="$1"
  cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$port
level-seed=paper-rust-forced-ticket-12110
view-distance=4
simulation-distance=4
enable-rcon=false
motd=Paper Rust Forced Ticket Persistence
PROPS
}

start_server() {
  local fifo="$1"
  local log="$2"
  rm -f "$fifo"
  mkfifo "$fifo"
  if [[ "$LAUNCHER" == *.sh ]]; then
    CMD=("$LAUNCHER" --nogui)
  else
    CMD=(java -Xms1G -Xmx2G -jar "$LAUNCHER" --nogui)
  fi
  (
    cd "$RUN"
    "${CMD[@]}" < "$fifo" > "$log" 2>&1 &
    echo $! > server.pid
  )
}

wait_done() {
  local log="$1"
  local deadline=$((SECONDS + 180))
  while (( SECONDS < deadline )); do
    if rg -q "$SERVER_READY_REGEX" "$log"; then
      return 0
    fi
    if rg -q "Failed to bind to port|Encountered an unexpected exception|UnsupportedClassVersionError|Failed to start" "$log"; then
      tail -n 160 "$log" >&2
      return 1
    fi
    sleep 1
  done
  tail -n 180 "$log" >&2
  return 1
}

wait_status_ping() {
  local port="$1"
  local report="$2"
  local tmp="${report}.tmp"
  for _ in $(seq 1 12); do
    if python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$port" 773 5 > "$tmp" 2>/dev/null; then
      mv "$tmp" "$report"
      return 0
    fi
    sleep 1
  done
  rm -f "$tmp"
  return 1
}

stop_server() {
  local pid
  pid="$(cat "$RUN/server.pid")"
  local deadline=$((SECONDS + 90))
  while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
    sleep 1
  done
  if kill -0 "$pid" 2>/dev/null; then
    return 1
  fi
}

write_properties "$PORT1"
FIFO1="$RUN/console-first.in"
start_server "$FIFO1" "$LOG1"
exec 3>"$FIFO1"
wait_done "$LOG1"
wait_status_ping "$PORT1" "$ROOT/reports/forced-ticket-persistence-first-status.json"
printf 'forceload add 0 0\n' >&3
printf 'forceload query 0 0\n' >&3
printf 'save-all flush\n' >&3
sleep 6
printf 'stop\n' >&3
exec 3>&-
stop_server
"$LOG_CLEAN_HELPER" "$LOG1" first > "$CLEAN1"

write_properties "$PORT2"
FIFO2="$RUN/console-restart.in"
start_server "$FIFO2" "$LOG2"
exec 4>"$FIFO2"
wait_done "$LOG2"
wait_status_ping "$PORT2" "$ROOT/reports/forced-ticket-persistence-restart-status.json"
printf 'forceload query 0 0\n' >&4
sleep 4
printf 'stop\n' >&4
exec 4>&-
stop_server
"$LOG_CLEAN_HELPER" "$LOG2" restart > "$CLEAN2"

if rg -q "No force loaded chunks|not marked for force loading|not force loaded" "$LOG2"; then
  tail -n 180 "$LOG2" >&2
  exit 1
fi

if ! rg -qi "0, 0|0 0|chunk.*0.*0|force.?loaded" "$LOG2"; then
  tail -n 180 "$LOG2" >&2
  exit 1
fi

{
  echo "forced_ticket_persistence=PASS"
  echo "first_log=$LOG1"
  echo "restart_log=$LOG2"
  echo "first_port=$PORT1"
  echo "restart_port=$PORT2"
  cat "$CLEAN1"
  cat "$CLEAN2"
  rg -n \
    -e "$SERVER_READY_REGEX" \
    -e "forceload|force load|force loading|marked for force|Saved the game|ERROR|Exception" \
    "$LOG1" "$LOG2" || true
} > "$ROOT/reports/forced-ticket-persistence-summary.txt"
cat "$ROOT/reports/forced-ticket-persistence-summary.txt"
