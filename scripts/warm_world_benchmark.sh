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

TASKSET_CMD=()
if [[ -n "${BENCHMARK_CPUSET:-}" ]]; then
  if ! command -v taskset >/dev/null 2>&1; then
    echo "BENCHMARK_CPUSET is set but taskset is unavailable." >&2
    exit 1
  fi
  TASKSET_CMD=(taskset -c "$BENCHMARK_CPUSET")
fi

WARM_WORLD_SOURCE="${WARM_WORLD_SOURCE:-$ROOT/runs/plugin-matrix}"
if [[ ! -d "$WARM_WORLD_SOURCE/world" ]]; then
  echo "Missing warm-world source: $WARM_WORLD_SOURCE/world" >&2
  echo "Run scripts/run_plugin_matrix.sh first." >&2
  exit 1
fi

REPORT="$ROOT/reports/warm-world-benchmark.csv"
mkdir -p "$ROOT/runs/warm-world-benchmark" "$ROOT/logs" "$ROOT/reports"

port_for_run() {
  python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

now_ms() {
  python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
}

copy_warm_world() {
  local run="$1"
  rm -rf "$run"
  mkdir -p "$run"
  if command -v rsync >/dev/null 2>&1; then
    rsync -a --delete \
      --exclude '/logs/' \
      --exclude '/server.pid' \
      --exclude '/restart.pid' \
      --exclude '/console.in' \
      --exclude '/restart-console.in' \
      "$WARM_WORLD_SOURCE"/ "$run"/
  else
    cp -a "$WARM_WORLD_SOURCE"/. "$run"/
    rm -rf "$run/logs" "$run/server.pid" "$run/restart.pid" "$run/console.in" "$run/restart-console.in"
  fi
}

run_one() {
  local name="$1"
  shift
  local -a cmd=("${TASKSET_CMD[@]}" "$@")
  local run="$ROOT/runs/warm-world-benchmark/$name"
  local log="$ROOT/logs/warm-world-$name.log"
  local fifo="$run/console.in"
  local port
  port="$(port_for_run)"

  copy_warm_world "$run"
  cat > "$run/eula.txt" <<'EULA'
eula=true
EULA
  cat > "$run/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$port
view-distance=6
simulation-distance=4
enable-rcon=false
motd=Paper Rust Warm-World Benchmark
PROPS

  mkfifo "$fifo"

  local start status_ms done_ms rss_kb stop_ms
  local status_file="$ROOT/reports/warm-world-$name-status.json"
  local status_tmp="$status_file.tmp"
  rm -f "$status_file" "$status_tmp"
  start="$(now_ms)"
  (
    cd "$run"
    "${cmd[@]}" < "$fifo" > "$log" 2>&1 &
    echo $! > server.pid
  )
  exec 3>"$fifo"
  local pid
  pid="$(cat "$run/server.pid")"

  status_ms=""
  local deadline=$((SECONDS + 240))
  while (( SECONDS < deadline )); do
    if [[ -z "$status_ms" ]] && python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$port" > "$status_tmp" 2>/dev/null; then
      mv "$status_tmp" "$status_file"
      status_ms=$(( $(now_ms) - start ))
    else
      rm -f "$status_tmp"
    fi
    if rg -q "$SERVER_READY_REGEX" "$log"; then
      for _ in 1 2 3 4 5; do
        if [[ -n "$status_ms" ]]; then
          break
        fi
        if python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$port" > "$status_tmp" 2>/dev/null; then
          mv "$status_tmp" "$status_file"
          status_ms=$(( $(now_ms) - start ))
          break
        fi
        rm -f "$status_tmp"
        sleep 0.2
      done
      done_ms=$(( $(now_ms) - start ))
      break
    fi
    if rg -q "Failed to bind to port|Encountered an unexpected exception|Failed to start|OutOfMemoryError" "$log"; then
      tail -n 140 "$log" >&2
      exit 1
    fi
    sleep 1
  done
  if [[ -z "${done_ms:-}" ]]; then
    tail -n 180 "$log" >&2
    exit 1
  fi
  rss_kb="$(ps -o rss= -p "$pid" | awk '{print $1}')"
  printf 'save-all flush\n' >&3
  sleep 3
  printf 'stop\n' >&3
  exec 3>&-

  deadline=$((SECONDS + 90))
  while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
    sleep 1
  done
  if kill -0 "$pid" 2>/dev/null; then
    tail -n 180 "$log" >&2
    exit 1
  fi
  stop_ms=$(( $(now_ms) - start ))
  printf '%s,%s,%s,%s,%s,%s,%s,%s\n' "$name" "$port" "${status_ms:-}" "$done_ms" "${rss_kb:-}" "$stop_ms" "$log" "$WARM_WORLD_SOURCE" >> "$REPORT"
}

printf 'name,port,status_ms,done_ms,rss_kb,stop_ms,log,source\n' > "$REPORT"
run_one "stock-paper-1.21.10" java -Xms1G -Xmx2G -jar "$ROOT/artifacts/paper-1.21.10-130.jar" --nogui
run_one "optimized-paper-1.21.10" java -Xms1G -Xmx2G -jar "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" --nogui
run_one "optimized-runtime-1.21.10" "$ROOT/artifacts/optimized-runtime/run.sh" --nogui
cat "$REPORT"
