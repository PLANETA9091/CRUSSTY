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

RUN="$ROOT/runs/native-climate-rtree-fallback-smoke"
LOG="$ROOT/logs/native-climate-rtree-fallback-smoke.log"
CLEAN="$ROOT/reports/native-climate-rtree-fallback-log-clean.txt"
SUMMARY="$ROOT/reports/native-climate-rtree-fallback-smoke.txt"
FIFO="$RUN/console.in"
PORT="${SERVER_PORT:-$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"

rm -rf "$RUN"
mkdir -p "$RUN/native" "$ROOT/logs" "$ROOT/reports"

if ! printf 'void paper_native_jni_dummy(void) {}\n' \
  | cc -shared -x c -fPIC -o "$RUN/native/libpaper_native_jni.so" -; then
  echo "Failed to build dummy native library for fallback smoke." >&2
  exit 1
fi

cat > "$RUN/eula.txt" <<'EULA'
# Explicitly enabled by scripts/native_climate_rtree_fallback_smoke.sh only when MC_EULA_AGREE=true.
eula=true
EULA
cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
level-seed=paper-rust-native-climate-fallback-12110
view-distance=4
simulation-distance=4
enable-rcon=false
motd=Paper Rust Native Climate Fallback Smoke
PROPS

mkfifo "$FIFO"
if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("$LAUNCHER" --nogui)
else
  CMD=(java -Xms256m -Xmx1G -jar "$LAUNCHER" --nogui)
fi
(
  cd "$RUN"
  env \
    PAPER_NATIVE_DIR="$RUN/native" \
    PAPER_NATIVE_CLIMATE_RTREE=true \
    PAPER_NATIVE_AREA_MAP=false \
    PAPER_NATIVE_IMPROVED_NOISE=false \
    PAPER_NATIVE_PERLIN_NOISE=false \
    PAPER_NATIVE_PERLIN_NOISE_GENERIC=false \
    PAPER_NATIVE_PERLIN_NOISE_NO_Y_SCALE=false \
    "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
  echo $! > server.pid
)

exec 3>"$FIFO"

deadline=$((SECONDS + 180))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if rg -q "Failed to bind to port|Encountered an unexpected exception|Exception in server tick loop|Failed to start" "$LOG"; then
    tail -n 180 "$LOG" >&2
    exit 1
  fi
  sleep 1
done

if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

if ! rg -q "\\[optimized-runtime\\].*native_climate_rtree=true.*native_climate_rtree_hook_present=true" "$LOG"; then
  tail -n 120 "$LOG" >&2
  echo "Runtime did not request native Climate RTree in fallback smoke." >&2
  exit 1
fi

if ! rg -q "Paper: Native Climate RTree unavailable.*using Java fallback" "$LOG"; then
  tail -n 180 "$LOG" >&2
  echo "Climate RTree fallback log line was not observed." >&2
  exit 1
fi

if rg -q "Paper: Using native Climate RTree from paper_native_jni\\." "$LOG"; then
  tail -n 180 "$LOG" >&2
  echo "Fallback smoke unexpectedly loaded native Climate RTree." >&2
  exit 1
fi

printf 'stop\n' >&3
exec 3>&-

pid="$(cat "$RUN/server.pid")"
deadline=$((SECONDS + 90))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

"$ROOT/scripts/check_runtime_log_clean.sh" "$LOG" native-climate-rtree-fallback > "$CLEAN"

{
  echo "native_climate_rtree_fallback=PASS"
  echo "fallback_log=$LOG"
  echo "fallback_port=$PORT"
  echo "dummy_native_lib=$RUN/native/libpaper_native_jni.so"
  cat "$CLEAN"
  rg -n \
    -e "$SERVER_READY_REGEX" \
    -e "\\[optimized-runtime\\]|Paper: Native Climate RTree unavailable|Paper: Using native Climate RTree|ERROR|Exception" \
    "$LOG" || true
} > "$SUMMARY"

cat "$SUMMARY"
