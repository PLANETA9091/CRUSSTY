#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
LABEL="host-synthetic-canary-smoke"
PROD_LABEL="host-synthetic-canary-production-guard-smoke"

cleanup() {
  rm -rf "$TMP"
  rm -rf "$ROOT/runs/load-$LABEL" "$ROOT/runs/load-$PROD_LABEL"
  rm -f \
    "$ROOT/logs/load-$LABEL.log" \
    "$ROOT/logs/load-$LABEL-bots.log" \
    "$ROOT/logs/load-$PROD_LABEL.log" \
    "$ROOT/logs/load-$PROD_LABEL-bots.log" \
    "$ROOT/reports/load-$LABEL-resources.csv" \
    "$ROOT/reports/load-$LABEL-summary.txt" \
    "$ROOT/reports/load-$LABEL-status.json" \
    "$ROOT/reports/load-$LABEL-preflight.txt" \
    "$ROOT/reports/load-$LABEL-harness-exit.txt" \
    "$ROOT/reports/load-$LABEL-gate.txt" \
    "$ROOT/reports/load-$PROD_LABEL-resources.csv" \
    "$ROOT/reports/load-$PROD_LABEL-summary.txt" \
    "$ROOT/reports/load-$PROD_LABEL-status.json" \
    "$ROOT/reports/load-$PROD_LABEL-preflight.txt" \
    "$ROOT/reports/load-$PROD_LABEL-harness-exit.txt" \
    "$ROOT/reports/load-$PROD_LABEL-gate.txt"
  rm -rf \
    "$ROOT/logs/load-$LABEL-bots" \
    "$ROOT/logs/load-$LABEL-jstacks" \
    "$ROOT/logs/load-$PROD_LABEL-bots" \
    "$ROOT/logs/load-$PROD_LABEL-jstacks"
}
trap cleanup EXIT

write_stat() {
  local path="$1"
  local user="$2"
  local idle="$3"
  local iowait="$4"
  local steal="$5"
  local next="$path.next"
  printf 'cpu  %s 0 0 %s %s 0 0 %s 0 0\n' "$user" "$idle" "$iowait" "$steal" > "$next"
  mv "$next" "$path"
}

feed_stat_sequence() {
  local path="$1"
  local user_step="$2"
  local idle_step="$3"
  local iowait_step="$4"
  local steal_step="$5"
  local count="$6"
  local interval="$7"
  local user=1000
  local idle=1000
  local iowait=0
  local steal=0
  local i

  sleep 0.01
  for ((i = 0; i < count; i++)); do
    user=$((user + user_step))
    idle=$((idle + idle_step))
    iowait=$((iowait + iowait_step))
    steal=$((steal + steal_step))
    write_stat "$path" "$user" "$idle" "$iowait" "$steal"
    sleep "$interval"
  done
}

feed_stat_forever() {
  local path="$1"
  local user_step="$2"
  local idle_step="$3"
  local iowait_step="$4"
  local steal_step="$5"
  local interval="$6"
  local user=1000
  local idle=1000
  local iowait=0
  local steal=0

  sleep 0.01
  while true; do
    user=$((user + user_step))
    idle=$((idle + idle_step))
    iowait=$((iowait + iowait_step))
    steal=$((steal + steal_step))
    write_stat "$path" "$user" "$idle" "$iowait" "$steal"
    sleep "$interval"
  done
}

run_case() {
  local name="$1"
  local expected_status="$2"
  local expected_ok="$3"
  local user_step="$4"
  local idle_step="$5"
  local iowait_step="$6"
  local steal_step="$7"
  local expected_reason_prefix="${8:-}"
  local stat="$TMP/$name.stat"
  local reason="$TMP/$name.reason"
  local out="$TMP/$name.out"
  local err="$TMP/$name.err"

  write_stat "$stat" 1000 1000 0 0
  feed_stat_forever "$stat" "$user_step" "$idle_step" "$iowait_step" "$steal_step" 0.01 &
  local feeder_pid=$!
  sleep 0.08

  set +e
  python3 "$ROOT/scripts/probe_host_synthetic_contention.py" \
    --duration-seconds 1 \
    --sample-interval-seconds 0.1 \
    --workers 1 \
    --max-steal-percent 10 \
    --max-iowait-percent 10 \
    --stat-path "$stat" \
    --reason-path "$reason" \
    >"$out" 2>"$err"
  local status=$?
  set -e

  kill "$feeder_pid" 2>/dev/null || true
  wait "$feeder_pid" 2>/dev/null || true

  if [[ "$status" -ne "$expected_status" ]]; then
    echo "expected $name to exit $expected_status, got $status" >&2
    echo "--- stdout ---" >&2
    cat "$out" >&2
    echo "--- stderr ---" >&2
    cat "$err" >&2
    return 1
  fi

  rg -q '^host_synthetic_canary_enabled=true$' "$out"
  rg -q '^host_synthetic_canary_duration_seconds=1$' "$out"
  rg -q '^host_synthetic_canary_sample_interval_seconds=0\.1$' "$out"
  rg -q '^host_synthetic_canary_workers=1$' "$out"
  rg -q '^host_synthetic_canary_samples=[0-9]+$' "$out"
  rg -q "^host_synthetic_canary_ok=${expected_ok}$" "$out"
  rg -q "^host_synthetic_canary_exit_code=${expected_status}$" "$out"

  if [[ "$expected_ok" == "true" ]]; then
    rg -q '^host_synthetic_canary_reason=none$' "$out"
    if [[ -e "$reason" ]]; then
      echo "unexpected reason file for pass case" >&2
      cat "$reason" >&2
      return 1
    fi
  else
    test -f "$reason"
    if [[ -n "$expected_reason_prefix" ]]; then
      rg -q "^${expected_reason_prefix}" "$reason"
    fi
    rg -q '^host_synthetic_canary_reason=host_contention_prelaunch_canary_' "$out"
    rg -q '^host_synthetic_canary_steal_percent_max=[0-9]+\.[0-9]{2}$' "$out"
    rg -q '^host_synthetic_canary_iowait_percent_max=[0-9]+\.[0-9]{2}$' "$out"
  fi
}

run_case \
  pass \
  0 \
  true \
  700 \
  250 \
  25 \
  25

run_case \
  contention \
  75 \
  false \
  100 \
  100 \
  400 \
  400 \
  'host_contention_prelaunch_canary '

FAKE_LAUNCHER="$TMP/fake-launcher.sh"
cat > "$FAKE_LAUNCHER" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'executed\n' > "$FAKE_LAUNCHER_MARKER"
exit 99
SH
chmod +x "$FAKE_LAUNCHER"

PROD_MARKER="$TMP/production-guard-launcher.marker"
set +e
(
  cd "$ROOT"
  MC_EULA_AGREE=true \
    LOAD_TEST_GATE_PROFILE=production-500 \
    LOAD_TEST_SCENARIO=block \
    LOAD_TEST_LABEL="$PROD_LABEL" \
    LOAD_TEST_HOST_SYNTHETIC_CANARY=false \
    BOT_COUNT=500 \
    DURATION_SECONDS=700 \
    VIEW_DISTANCE=32 \
    SIMULATION_DISTANCE=32 \
    LAUNCHER="$FAKE_LAUNCHER" \
    FAKE_LAUNCHER_MARKER="$PROD_MARKER" \
    "$ROOT/scripts/run_load_test.sh"
) > "$TMP/production-guard.out" 2> "$TMP/production-guard.err"
guard_status=$?
set -e
if [[ "$guard_status" -ne 75 ]]; then
  echo "Expected production synthetic canary guard to exit 75, got $guard_status." >&2
  cat "$TMP/production-guard.out" >&2
  cat "$TMP/production-guard.err" >&2
  exit 1
fi
if [[ -e "$PROD_MARKER" ]]; then
  echo "Production canary guard must reject before launcher execution." >&2
  exit 1
fi
rg -q 'Production claim profiles require LOAD_TEST_HOST_SYNTHETIC_CANARY=true\.' "$TMP/production-guard.err"

RUN_STAT="$TMP/run-load-test.stat"
write_stat "$RUN_STAT" 1000 1000 0 0
feed_stat_forever "$RUN_STAT" 100 100 400 400 0.005 &
feeder_pid=$!
FAKE_BIN="$TMP/fake-bin"
FAKE_DATE_STATE="$TMP/fake-date.state"
mkdir -p "$FAKE_BIN"
cat > "$FAKE_BIN/date" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "+%s" ]]; then
  state="${FAKE_DATE_STATE:?}"
  if [[ -f "$state" ]]; then
    count="$(cat "$state")"
  else
    count=0
  fi
  case "$count" in
    0|1) printf '1000\n' ;;
    *) printf '1061\n' ;;
  esac
  printf '%s\n' "$((count + 1))" > "$state"
  exit 0
fi
exec /usr/bin/date "$@"
SH
chmod +x "$FAKE_BIN/date"
LOAD_MARKER="$TMP/run-load-test-launcher.marker"
set +e
(
  cd "$ROOT"
  MC_EULA_AGREE=true \
    PATH="$FAKE_BIN:$PATH" \
    FAKE_DATE_STATE="$FAKE_DATE_STATE" \
    LOAD_TEST_GATE_PROFILE=stress-mixed-gameplay \
    LOAD_TEST_SCENARIO=block \
    LOAD_TEST_LABEL="$LABEL" \
    BOT_COUNT=500 \
    DURATION_SECONDS=700 \
    VIEW_DISTANCE=32 \
    SIMULATION_DISTANCE=32 \
    LAUNCHER="$FAKE_LAUNCHER" \
    LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
    LOAD_TEST_MIN_HOST_IDLE_PERCENT=0 \
    LOAD_TEST_MAX_LOAD_PER_CPU=999 \
    LOAD_TEST_MAX_HOST_STEAL_PERCENT=100 \
    LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=100 \
    LOAD_TEST_WAIT_FOR_HOST_READY=false \
    LOAD_TEST_HOST_READY_TIMEOUT_SECONDS=120 \
    LOAD_TEST_HOST_READY_POLL_SECONDS=1 \
    LOAD_TEST_HOST_SYNTHETIC_CANARY=true \
    LOAD_TEST_HOST_SYNTHETIC_CANARY_SECONDS=1 \
    LOAD_TEST_HOST_SYNTHETIC_CANARY_SAMPLE_INTERVAL_SECONDS=0.05 \
    LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_STEAL_PERCENT=10 \
    LOAD_TEST_HOST_SYNTHETIC_CANARY_MAX_IOWAIT_PERCENT=10 \
    LOAD_TEST_HOST_SYNTHETIC_CANARY_WORKERS=1 \
    LOAD_TEST_HOST_SYNTHETIC_CANARY_STAT_PATH="$RUN_STAT" \
    FAKE_LAUNCHER_MARKER="$LOAD_MARKER" \
    "$ROOT/scripts/run_load_test.sh"
) > "$TMP/run-load-test.out" 2> "$TMP/run-load-test.err"
load_status=$?
set -e
kill "$feeder_pid" 2>/dev/null || true
wait "$feeder_pid" 2>/dev/null || true
if [[ "$load_status" -ne 75 ]]; then
  echo "Expected run_load_test synthetic canary abort exit 75, got $load_status." >&2
  cat "$TMP/run-load-test.out" >&2
  cat "$TMP/run-load-test.err" >&2
  exit 1
fi
if [[ -e "$LOAD_MARKER" ]]; then
  echo "Synthetic canary abort must happen before launcher execution." >&2
  exit 1
fi

SUMMARY="$ROOT/reports/load-$LABEL-summary.txt"
GATE="$ROOT/reports/load-$LABEL-gate.txt"
PREFLIGHT="$ROOT/reports/load-$LABEL-preflight.txt"
test -f "$SUMMARY"
test -f "$GATE"
test -f "$PREFLIGHT"
rg -q '(^| )bot_exit=75($| )' "$SUMMARY"
rg -q '^early_abort_reason=host_contention_prelaunch_canary' "$SUMMARY"
rg -q '^host_synthetic_canary_enabled=true$' "$SUMMARY"
rg -q '^host_synthetic_canary_ok=false$' "$SUMMARY"
rg -q '^host_synthetic_canary_ok=false$' "$PREFLIGHT"
rg -q '^environment_invalid=true$' "$GATE"
rg -q '^environment_invalid_kind=host_contention$' "$GATE"
rg -q '^run_class=environment-invalid$' "$GATE"

echo "run_load_test_host_synthetic_canary_smoke=PASS"
