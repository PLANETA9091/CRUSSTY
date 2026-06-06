#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
STAT="$TMP/stat"
REASON="$TMP/early-abort.reason"
PID_FILE="$TMP/bot-pids.txt"
EXISTING_REASON="$TMP/existing.reason"
bot_pid=""

cleanup() {
  if [[ -n "$bot_pid" ]]; then
    kill "$bot_pid" 2>/dev/null || true
    wait "$bot_pid" 2>/dev/null || true
  fi
  rm -rf "$TMP"
}
trap cleanup EXIT

write_stat() {
  local content="$1"
  local tmp_stat="$TMP/stat.next"
  printf '%s\n' "$content" > "$tmp_stat"
  mv "$tmp_stat" "$STAT"
}

feed_contention_samples() {
  local iowait
  sleep 0.02
  for iowait in 1000 2000 3000 4000 5000 6000; do
    write_stat "cpu  0 0 0 1000 $iowait 0 0 0 0 0"
    sleep 0.03
  done
}

feed_invalid_cpu_samples() {
  local idle
  sleep 0.02
  for idle in 900 800 700 600; do
    write_stat "cpu  0 0 0 $idle 900 0 0 900 0 0"
    sleep 0.03
  done
}

write_stat "cpu  0 0 0 1000 0 0 0 0 0 0"
sleep 120 &
bot_pid=$!
printf '%s\n' "$bot_pid" > "$PID_FILE"

set +e
timeout 5s python3 "$ROOT/scripts/watch_load_host_contention.py" \
  --reason "$REASON" \
  --bot-pid "$bot_pid" \
  --bot-pid-file "$PID_FILE" \
  --max-load-per-cpu 999 \
  --max-steal-percent 999 \
  --max-iowait-percent 1 \
  --interval 0.05 \
  --bad-samples 2 \
  --stat-path "$STAT" &
watcher_pid=$!
feed_contention_samples &
feeder_pid=$!
wait "$watcher_pid"
status=$?
wait "$feeder_pid" 2>/dev/null || true
set -e

if [[ "$status" -ne 42 ]]; then
  echo "expected host contention watcher to exit 42, got $status" >&2
  exit 1
fi
test -f "$REASON"
rg -q '^host_contention ' "$REASON"
rg -q '^host_contention bad_samples=2 ' "$REASON"
rg -q 'iowait_percent=100\.00 max_iowait_percent=1\.00' "$REASON"
wait "$bot_pid" 2>/dev/null || true

printf 'preexisting_reason\n' > "$EXISTING_REASON"
sleep 120 &
bot_pid=$!
printf '%s\n' "$bot_pid" > "$PID_FILE"
write_stat "cpu  0 0 0 1000 0 0 0 0 0 0"
set +e
timeout 5s python3 "$ROOT/scripts/watch_load_host_contention.py" \
  --reason "$EXISTING_REASON" \
  --bot-pid "$bot_pid" \
  --bot-pid-file "$PID_FILE" \
  --max-load-per-cpu 999 \
  --max-steal-percent 999 \
  --max-iowait-percent 1 \
  --interval 0.05 \
  --bad-samples 2 \
  --stat-path "$STAT" &
watcher_pid=$!
feed_contention_samples &
feeder_pid=$!
wait "$watcher_pid"
status=$?
wait "$feeder_pid" 2>/dev/null || true
set -e
if [[ "$status" -ne 42 ]]; then
  echo "expected host contention watcher overwrite check to exit 42, got $status" >&2
  exit 1
fi
test "$(cat "$EXISTING_REASON")" = "preexisting_reason"

INVALID_REASON="$TMP/invalid.reason"
sleep 120 &
bot_pid=$!
printf '%s\n' "$bot_pid" > "$PID_FILE"
write_stat "cpu  0 0 0 1000 1000 0 0 1000 0 0"
set +e
timeout 5s python3 "$ROOT/scripts/watch_load_host_contention.py" \
  --reason "$INVALID_REASON" \
  --bot-pid "$bot_pid" \
  --bot-pid-file "$PID_FILE" \
  --max-load-per-cpu 999 \
  --max-steal-percent 999 \
  --max-iowait-percent 999 \
  --interval 0.05 \
  --bad-samples 2 \
  --stat-path "$STAT" &
watcher_pid=$!
feed_invalid_cpu_samples &
feeder_pid=$!
wait "$watcher_pid"
status=$?
wait "$feeder_pid" 2>/dev/null || true
set -e
if [[ "$status" -ne 42 ]]; then
  echo "expected host contention watcher invalid-delta check to exit 42, got $status" >&2
  exit 1
fi
test -f "$INVALID_REASON"
rg -q '^host_contention invalid_cpu_delta bad_samples=2 ' "$INVALID_REASON"
wait "$bot_pid" 2>/dev/null || true

MISSING_STAT_REASON="$TMP/missing-stat.reason"
MISSING_STAT="$TMP/missing-stat"
sleep 120 &
bot_pid=$!
printf '%s\n' "$bot_pid" > "$PID_FILE"
set +e
timeout 5s python3 "$ROOT/scripts/watch_load_host_contention.py" \
  --reason "$MISSING_STAT_REASON" \
  --bot-pid "$bot_pid" \
  --bot-pid-file "$PID_FILE" \
  --max-load-per-cpu 999 \
  --max-steal-percent 999 \
  --max-iowait-percent 999 \
  --interval 0.05 \
  --bad-samples 2 \
  --stat-path "$MISSING_STAT"
status=$?
set -e
if [[ "$status" -ne 42 ]]; then
  echo "expected host contention watcher missing-stat check to exit 42, got $status" >&2
  exit 1
fi
test -f "$MISSING_STAT_REASON"
rg -q '^host_contention watcher_exception bad_samples=1 exception=FileNotFoundError$' "$MISSING_STAT_REASON"
wait "$bot_pid" 2>/dev/null || true

echo "run_load_test_host_contention_watcher_smoke=PASS"
