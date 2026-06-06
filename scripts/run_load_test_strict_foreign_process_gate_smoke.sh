#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
LABEL="strict-foreign-process-gate-smoke-$$"
SCANNER_LABEL="strict-foreign-process-scanner-smoke-$$"
DIAGNOSTIC_LABEL="strict-foreign-process-diagnostic-smoke-$$"
RUN_DIR="$ROOT/runs/load-$LABEL"
SCANNER_RUN_DIR="$ROOT/runs/load-$SCANNER_LABEL"
DIAGNOSTIC_RUN_DIR="$ROOT/runs/load-$DIAGNOSTIC_LABEL"
PREFLIGHT="$ROOT/reports/load-$LABEL-preflight.txt"
SCANNER_PREFLIGHT="$ROOT/reports/load-$SCANNER_LABEL-preflight.txt"
DIAGNOSTIC_PREFLIGHT="$ROOT/reports/load-$DIAGNOSTIC_LABEL-preflight.txt"
DIAGNOSTIC_REASON="$ROOT/reports/load-$DIAGNOSTIC_LABEL-diagnostic.reason"
MARKER="$TMP/fake-launcher.executed"
SCANNER_MARKER="$TMP/fake-launcher-scanner.executed"
DIAGNOSTIC_MARKER="$TMP/fake-launcher-diagnostic.executed"
foreign_pid=""
scanner_pid=""

cleanup() {
  if [[ -n "$foreign_pid" ]]; then
    kill "$foreign_pid" 2>/dev/null || true
    wait "$foreign_pid" 2>/dev/null || true
  fi
  if [[ -n "$scanner_pid" ]]; then
    kill "$scanner_pid" 2>/dev/null || true
    wait "$scanner_pid" 2>/dev/null || true
  fi
  rm -rf "$TMP" "$RUN_DIR" "$SCANNER_RUN_DIR" "$DIAGNOSTIC_RUN_DIR"
  rm -f "$ROOT/logs/load-$LABEL.log" \
    "$ROOT/logs/load-$LABEL-bots.log" \
    "$ROOT/logs/load-$SCANNER_LABEL.log" \
    "$ROOT/logs/load-$SCANNER_LABEL-bots.log" \
    "$ROOT/logs/load-$DIAGNOSTIC_LABEL.log" \
    "$ROOT/logs/load-$DIAGNOSTIC_LABEL-bots.log" \
    "$ROOT/reports/load-$LABEL-preflight.txt" \
    "$ROOT/reports/load-$LABEL-harness-exit.txt" \
    "$ROOT/reports/load-$LABEL-resources.csv" \
    "$ROOT/reports/load-$LABEL-summary.txt" \
    "$ROOT/reports/load-$LABEL-status.json" \
    "$ROOT/reports/load-$LABEL-gate.txt" \
    "$ROOT/reports/load-$SCANNER_LABEL-preflight.txt" \
    "$ROOT/reports/load-$SCANNER_LABEL-harness-exit.txt" \
    "$ROOT/reports/load-$SCANNER_LABEL-resources.csv" \
    "$ROOT/reports/load-$SCANNER_LABEL-summary.txt" \
    "$ROOT/reports/load-$SCANNER_LABEL-status.json" \
    "$ROOT/reports/load-$SCANNER_LABEL-gate.txt" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-preflight.txt" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-harness-exit.txt" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-resources.csv" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-summary.txt" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-status.json" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-gate.txt" \
    "$ROOT/reports/load-$DIAGNOSTIC_LABEL-diagnostic.reason"
}
trap cleanup EXIT

fake_launcher="$TMP/fake-launcher.sh"
cat > "$fake_launcher" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'fake launcher executed\n' > "$FAKE_LAUNCHER_MARKER"
sleep 120
SH
chmod +x "$fake_launcher"

mkdir -p "$RUN_DIR"
printf 'must-survive-preflight\n' > "$RUN_DIR/preflight-sentinel"

bash -c 'exec -a server.jar sleep 120' &
foreign_pid=$!
for _ in $(seq 1 50); do
  if ps -p "$foreign_pid" -o args= | rg -q '^server\.jar\b'; then
    break
  fi
  sleep 0.1
done
if ! ps -p "$foreign_pid" -o args= | rg -q '^server\.jar\b'; then
  echo "synthetic foreign server.jar process did not start" >&2
  exit 1
fi

set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=false \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=false \
LOAD_TEST_MIN_HOST_IDLE_PERCENT=0 \
LOAD_TEST_MAX_LOAD_PER_CPU=999 \
LOAD_TEST_MAX_HOST_STEAL_PERCENT=100 \
LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=100 \
LOAD_TEST_LABEL="$LABEL" \
LAUNCHER="$fake_launcher" \
FAKE_LAUNCHER_MARKER="$MARKER" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/run.out" 2> "$TMP/run.err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected run_load_test.sh to exit 75, got $status" >&2
  cat "$TMP/run.out" >&2
  cat "$TMP/run.err" >&2
  exit 1
fi
test ! -e "$MARKER"
test -f "$RUN_DIR/preflight-sentinel"
test -f "$PREFLIGHT"
rg -q '^host_preflight_host_checked=true$' "$PREFLIGHT"
rg -q '^strict_foreign_process_gate_pass=false$' "$PREFLIGHT"
rg -q '^strict_foreign_process_count=[1-9][0-9]*$' "$PREFLIGHT"
rg -q 'foreign_process_pattern=server\.jar' "$PREFLIGHT"
rg -q 'Foreign Minecraft/load-test process detected before benchmark start' "$TMP/run.err"

diagnostic_launcher="$TMP/fake-diagnostic-launcher.sh"
cat > "$diagnostic_launcher" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'diagnostic launcher executed\n' > "$FAKE_LAUNCHER_MARKER"
exit 0
SH
chmod +x "$diagnostic_launcher"

set +e
MC_EULA_AGREE=true \
LOAD_TEST_DIAGNOSTIC_MODE=true \
LOAD_TEST_ALLOW_BUSY_HOST=false \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=false \
LOAD_TEST_MIN_HOST_IDLE_PERCENT=0 \
LOAD_TEST_MAX_LOAD_PER_CPU=999 \
LOAD_TEST_MAX_HOST_STEAL_PERCENT=100 \
LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=100 \
LOAD_TEST_LABEL="$DIAGNOSTIC_LABEL" \
LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS=1 \
LAUNCHER="$diagnostic_launcher" \
FAKE_LAUNCHER_MARKER="$DIAGNOSTIC_MARKER" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/diagnostic.out" 2> "$TMP/diagnostic.err"
diagnostic_status=$?
set -e

if [[ "$diagnostic_status" -eq 75 ]]; then
  echo "diagnostic mode must not treat unrelated foreign servers as a hard blocker" >&2
  cat "$TMP/diagnostic.out" >&2
  cat "$TMP/diagnostic.err" >&2
  exit 1
fi
test -e "$DIAGNOSTIC_MARKER"
test -f "$DIAGNOSTIC_PREFLIGHT"
test -f "$DIAGNOSTIC_REASON"
rg -q '^host_preflight_diagnostic_mode=true$' "$DIAGNOSTIC_PREFLIGHT"
rg -q '^strict_foreign_process_gate_diagnostic_mode=true$' "$DIAGNOSTIC_PREFLIGHT"
rg -q '^strict_foreign_process_gate_diagnostic_foreign_present=true$' "$DIAGNOSTIC_PREFLIGHT"
rg -q '^strict_foreign_process_gate_pass=false$' "$DIAGNOSTIC_PREFLIGHT"
rg -q '^host_contention_diagnostic_foreign_process_present$' "$DIAGNOSTIC_REASON"

kill "$foreign_pid" 2>/dev/null || true
wait "$foreign_pid" 2>/dev/null || true
foreign_pid=""

env -i bash -lc 'exec -a "pgrep -af run_load_test|mc_bot_swarm|optimized-runtime/run.sh|java .*paper" sleep 120' &
scanner_pid=$!
for _ in $(seq 1 50); do
  if ps -p "$scanner_pid" -o args= | rg -q 'pgrep -af.*mc_bot_swarm'; then
    break
  fi
  sleep 0.1
done
if ! ps -p "$scanner_pid" -o args= | rg -q 'pgrep -af.*mc_bot_swarm'; then
  echo "synthetic scanner process did not start" >&2
  exit 1
fi

set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=true \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=false \
LOAD_TEST_STRICT_FOREIGN_PROCESS_HARD_BLOCK_PATTERNS=mc_bot_swarm \
LOAD_TEST_WAIT_FOR_HOST_READY=false \
LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS=1 \
LOAD_TEST_LABEL="$SCANNER_LABEL" \
LAUNCHER="$fake_launcher" \
FAKE_LAUNCHER_MARKER="$SCANNER_MARKER" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/scanner.out" 2> "$TMP/scanner.err"
status=$?
set -e

if [[ "$status" -eq 75 ]]; then
  echo "scanner-only process must not trip the foreign-process gate" >&2
  cat "$TMP/scanner.out" >&2
  cat "$TMP/scanner.err" >&2
  exit 1
fi
test -f "$SCANNER_PREFLIGHT"
rg -q '^strict_foreign_process_gate_pass=true$' "$SCANNER_PREFLIGHT"
rg -q '^strict_foreign_process_count=0$' "$SCANNER_PREFLIGHT"
test -e "$SCANNER_MARKER"
if rg -q 'Foreign Minecraft/load-test process detected before benchmark start' "$TMP/scanner.err"; then
  echo "scanner-only process was reported as a foreign process" >&2
  cat "$SCANNER_PREFLIGHT" >&2
  exit 1
fi

echo "run_load_test_strict_foreign_process_gate_smoke=PASS"
