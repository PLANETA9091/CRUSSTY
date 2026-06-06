#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
LABEL="host-ready-stable-smoke-$$"
MARKER="$TMP/fake-launcher.executed"
RUN_DIR="$ROOT/runs/load-$LABEL"
PREFLIGHT="$ROOT/reports/load-$LABEL-preflight.txt"
SUMMARY="$ROOT/reports/load-$LABEL-summary.txt"
GATE="$ROOT/reports/load-$LABEL-gate.txt"
foreign_log="$TMP/foreign.out"
foreign_err="$TMP/foreign.err"
timeout_log="$TMP/timeout.out"
timeout_err="$TMP/timeout.err"

cleanup() {
  rm -rf "$TMP" "$RUN_DIR"
  rm -f "$ROOT/logs/load-$LABEL.log" \
    "$ROOT/logs/load-$LABEL-bots.log" \
    "$ROOT/reports/load-$LABEL-preflight.txt" \
    "$ROOT/reports/load-$LABEL-resources.csv" \
    "$ROOT/reports/load-$LABEL-summary.txt" \
    "$ROOT/reports/load-$LABEL-status.json" \
    "$ROOT/reports/load-$LABEL-gate.txt"
}
trap cleanup EXIT

fake_launcher="$TMP/fake-launcher.sh"
cat > "$fake_launcher" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'fake launcher executed\n' > "$FAKE_LAUNCHER_MARKER"
exit 99
SH
chmod +x "$fake_launcher"

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    LOAD_TEST_GATE_PROFILE=production-500 \
    LOAD_TEST_LABEL="$LABEL" \
    LOAD_TEST_SCENARIO=movement \
    LOAD_TEST_WAIT_FOR_HOST_READY=false \
    BOT_COUNT=1 \
    DURATION_SECONDS=1 \
    LAUNCHER="$fake_launcher" \
    FAKE_LAUNCHER_MARKER="$MARKER" \
    "$ROOT/scripts/run_load_test.sh"
) >"$foreign_log" 2>"$foreign_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected production host-ready override rejection to exit 75, got $status" >&2
  cat "$foreign_log" >&2
  cat "$foreign_err" >&2
  exit 1
fi
if [[ -e "$MARKER" ]]; then
  echo "launcher must not execute when production host readiness is disabled" >&2
  exit 1
fi
rg -q 'Production claim profiles require LOAD_TEST_WAIT_FOR_HOST_READY=true\.' "$foreign_err"

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    LOAD_TEST_GATE_PROFILE=strict-block \
    LOAD_TEST_LABEL="$LABEL" \
    LOAD_TEST_SCENARIO=movement \
    LOAD_TEST_WAIT_FOR_HOST_READY=true \
    LOAD_TEST_HOST_READY_STABLE_SECONDS=60 \
    LOAD_TEST_MIN_HOST_IDLE_PERCENT=0 \
    LOAD_TEST_MAX_LOAD_PER_CPU=999 \
    LOAD_TEST_MAX_HOST_STEAL_PERCENT=100 \
    LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=100 \
    LOAD_TEST_HOST_READY_TIMEOUT_SECONDS=2 \
    LOAD_TEST_HOST_READY_POLL_SECONDS=1 \
    LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
    BOT_COUNT=1 \
    DURATION_SECONDS=1 \
    LAUNCHER="$fake_launcher" \
    FAKE_LAUNCHER_MARKER="$MARKER" \
    "$ROOT/scripts/run_load_test.sh"
) >"$timeout_log" 2>"$timeout_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected production host-ready stable window timeout to exit 75, got $status" >&2
  cat "$timeout_log" >&2
  cat "$timeout_err" >&2
  exit 1
fi
if [[ -e "$MARKER" ]]; then
  echo "launcher must not execute before the stable host window is met" >&2
  exit 1
fi
test -f "$PREFLIGHT"
test -f "$SUMMARY"
test -f "$GATE"
rg -q '^host_preflight_ok=true$' "$PREFLIGHT"
rg -q '^host_ready_wait_enabled=true$' "$timeout_log"
rg -q '^host_ready_stable_seconds=60$' "$timeout_log"
rg -q '^host_ready_wait_attempt=1 .*host_preflight_host_ok=true stable_window_seconds=0 stable_required_seconds=60 stable_samples=1$' "$timeout_log"
rg -q '^host_ready_wait_result=timeout attempts=2 elapsed_seconds=[0-9]+ stable_window_seconds=[0-9]+ stable_samples=2$' "$timeout_log"
rg -q 'Host did not provide a stable 60s ready window before benchmark start\.' "$timeout_err"
rg -q '(^| )bot_exit=75($| )' "$SUMMARY"
rg -q '^early_abort_reason=host_contention_prelaunch_host_ready_timeout' "$SUMMARY"
rg -q '^environment_invalid=true$' "$GATE"
rg -q '^environment_invalid_kind=host_contention$' "$GATE"
rg -q '^run_class=environment-invalid$' "$GATE"

echo "run_load_test_host_ready_stable_smoke=PASS"
