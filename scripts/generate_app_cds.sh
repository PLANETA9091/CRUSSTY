#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to generate CDS without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/launcher_utils.sh"
source "$ROOT/scripts/load_test_defaults.sh"
SERVER_READY_REGEX="$(load_test_server_ready_regex)"
LAUNCHER="${1:-$ROOT/artifacts/optimized-runtime/run.sh}"
LAUNCHER="$(resolve_launcher_path "$LAUNCHER")"
OUT="${2:-$ROOT/artifacts/optimized-runtime}"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"

if [[ ! -x "$LAUNCHER" && ! -f "$LAUNCHER" ]]; then
  echo "Missing launcher: $LAUNCHER" >&2
  exit 1
fi

rm -f "$OUT/app-cds.jsa"

RUN="$ROOT/runs/app-cds"
LOG="$ROOT/logs/app-cds.log"
FIFO="$RUN/console.in"
PORT="${SERVER_PORT:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"

rm -rf "$RUN"
mkdir -p "$RUN" "$ROOT/logs" "$ROOT/reports"
cat > "$RUN/eula.txt" <<'EULA'
eula=true
EULA
cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
level-seed=paper-rust-cds-12110
view-distance=6
simulation-distance=4
enable-rcon=false
motd=Paper Rust CDS Warmup
PROPS

rm -f "$FIFO"
mkfifo "$FIFO"
if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("$LAUNCHER" --nogui)
else
  CMD=(java -jar "$LAUNCHER" --nogui)
fi

(
  cd "$RUN"
  JAVA_OPTS="${JAVA_OPTS:--Xms1G -Xmx2G} -XX:ArchiveClassesAtExit=$OUT/app-cds.jsa" "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
  echo $! > server.pid
)
pid="$(cat "$RUN/server.pid")"
exec 3>"$FIFO"

cleanup() {
  if kill -0 "$pid" 2>/dev/null; then
    printf 'stop\n' >&3 2>/dev/null || true
    sleep 5
    kill "$pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

deadline=$((SECONDS + 240))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if rg -q "Failed to bind to port|Encountered an unexpected exception|UnsupportedClassVersionError|Could not load plugin|Failed to start|OutOfMemoryError" "$LOG"; then
    tail -n 200 "$LOG" >&2
    exit 1
  fi
  sleep 1
done
if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  tail -n 220 "$LOG" >&2
  exit 1
fi

printf 'stop\n' >&3
exec 3>&-

deadline=$((SECONDS + 90))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  tail -n 200 "$LOG" >&2
  exit 1
fi

if [[ ! -s "$OUT/app-cds.jsa" ]]; then
  echo "CDS archive was not created: $OUT/app-cds.jsa" >&2
  tail -n 200 "$LOG" >&2
  exit 1
fi

echo "$OUT/app-cds.jsa"
