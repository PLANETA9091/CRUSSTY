#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
REPORT="$ROOT/reports/boot-benchmark.csv"
mkdir -p "$ROOT/runs/boot-benchmark" "$ROOT/logs" "$ROOT/reports"

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

run_one() {
  local name="$1"
  shift
  local -a cmd=("${TASKSET_CMD[@]}" "$@")
  local run="$ROOT/runs/boot-benchmark/$name"
  local log="$ROOT/logs/boot-$name.log"
  local fifo="$run/console.in"
  local port
  port="$(port_for_run)"

  rm -rf "$run"
  mkdir -p "$run"
  cat > "$run/eula.txt" <<'EULA'
eula=true
EULA
  cat > "$run/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$port
level-seed=paper-rust-compat-12110
view-distance=6
simulation-distance=4
enable-rcon=false
motd=Paper Rust Boot Benchmark
PROPS
  mkfifo "$fifo"

  local start status_ms done_ms rss_kb stop_ms
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
    if [[ -z "$status_ms" ]] && python3 "$ROOT/scripts/mc_status_ping.py" 127.0.0.1 "$port" > "$ROOT/reports/boot-$name-status.json" 2>/dev/null; then
      status_ms=$(( $(now_ms) - start ))
    fi
    if rg -q "$SERVER_READY_REGEX" "$log"; then
      done_ms=$(( $(now_ms) - start ))
      break
    fi
    if rg -q "Failed to bind to port|Encountered an unexpected exception|Failed to start" "$log"; then
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
  printf '%s,%s,%s,%s,%s,%s,%s\n' "$name" "$port" "${status_ms:-}" "$done_ms" "${rss_kb:-}" "$stop_ms" "$log" >> "$REPORT"
}

printf 'name,port,status_ms,done_ms,rss_kb,stop_ms,log\n' > "$REPORT"
run_one "vanilla-1.21.10" java -Xms1G -Xmx2G -jar "$ROOT/artifacts/vanilla-1.21.10.jar" --nogui
run_one "stock-paper-1.21.10" java -Xms1G -Xmx2G -jar "$ROOT/artifacts/paper-1.21.10-130.jar" --nogui
run_one "optimized-paper-1.21.10" java -Xms1G -Xmx2G -jar "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" --nogui
run_one "optimized-runtime-1.21.10" "$ROOT/artifacts/optimized-runtime/run.sh" --nogui
cat "$REPORT"
