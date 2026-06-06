#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="p500-contended-diagnostic-smoke-$$"
REPORT="$ROOT/reports/p500-contended-diagnostic-${STAMP}.txt"
TMP="$(mktemp -d)"
LABEL="p500-contended-diagnostic-smoke-$STAMP"
RUN_DIR="$ROOT/runs/load-$LABEL"
PREFLIGHT="$ROOT/reports/load-$LABEL-preflight.txt"
GATE="$ROOT/reports/load-$LABEL-gate.txt"
SUMMARY="$ROOT/reports/load-$LABEL-summary.txt"
SERVER_LOG="$ROOT/logs/load-$LABEL.log"
BOT_LOG="$ROOT/logs/load-$LABEL-bots.log"
MARKER="$TMP/fake-launcher.executed"
FAKE_LAUNCHER="$TMP/fake-launcher.sh"
foreign_pid=""

cleanup() {
  if [[ -n "$foreign_pid" ]]; then
    kill "$foreign_pid" 2>/dev/null || true
    wait "$foreign_pid" 2>/dev/null || true
  fi
  rm -rf "$TMP"
  rm -rf "$RUN_DIR"
  rm -rf "$ROOT/logs/load-$LABEL-bots" "$ROOT/logs/load-$LABEL-jstacks"
  rm -f "$REPORT" \
    "$PREFLIGHT" \
    "$ROOT/reports/load-$LABEL-harness-exit.txt" \
    "$GATE" \
    "$SUMMARY" \
    "$SERVER_LOG" \
    "$BOT_LOG" \
    "$ROOT/logs/load-$LABEL-bots.log" \
    "$ROOT/reports/load-$LABEL-resources.csv" \
    "$ROOT/reports/load-$LABEL-status.json"
}
trap cleanup EXIT

cat > "$FAKE_LAUNCHER" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'executed\n' > "$FAKE_LAUNCHER_MARKER"
exit 99
SH
chmod +x "$FAKE_LAUNCHER"

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
  P500_CONTENDED_DIAGNOSTIC_STAMP="$STAMP" \
  P500_CONTENDED_DIAGNOSTIC_LABEL="$LABEL" \
  P500_CONTENDED_DIAGNOSTIC_REPORT="$REPORT" \
  P500_CONTENDED_DIAGNOSTIC_REFRESH_ARTIFACTS=false \
  LOAD_TEST_MIN_HOST_IDLE_PERCENT=101 \
  LOAD_TEST_MAX_LOAD_PER_CPU=0 \
  LOAD_TEST_MAX_HOST_STEAL_PERCENT=0 \
  LOAD_TEST_MAX_HOST_IOWAIT_PERCENT=0 \
  P500_CONTENDED_DIAGNOSTIC_SERVER_READY_TIMEOUT_SECONDS=1 \
  LAUNCHER="$FAKE_LAUNCHER" \
  FAKE_LAUNCHER_MARKER="$MARKER" \
  "$ROOT/scripts/run_p500_contended_diagnostic.sh" 1 1 > "$TMP/run.out" 2> "$TMP/run.err"
status=$?
set -e

if [[ "$status" -ne 1 ]]; then
  echo "expected fake-launcher contended diagnostic to exit 1 after server-ready timeout, got $status" >&2
  cat "$TMP/run.out" >&2
  cat "$TMP/run.err" >&2
  exit 1
fi

test -f "$REPORT"
test -e "$MARKER"
rg -q '^p500_contended_diagnostic_non_claim=true$' "$REPORT"
rg -q '^p500_contended_diagnostic_production_claim_eligible=false$' "$REPORT"
rg -q '^p500_contended_diagnostic_no_bundle_publication=true$' "$REPORT"
rg -q '^p500_contended_diagnostic_dry_run=false$' "$REPORT"
rg -q '^p500_contended_diagnostic_exit_code=1$' "$REPORT"
rg -q '^p500_contended_diagnostic_server_ready_timeout_seconds=1$' "$REPORT"
rg -q '^load_test_server_ready_timeout_seconds=1$' "$REPORT"
rg -q '^load_test_gate_profile=strict-block$' "$REPORT"
rg -q '^load_test_allow_foreign_processes=true$' "$REPORT"
rg -q '^load_test_allow_busy_host=true$' "$REPORT"
rg -q '^load_test_abort_on_host_contention=false$' "$REPORT"
rg -q '^load_test_wait_for_host_ready=false$' "$REPORT"
rg -q '^load_test_host_synthetic_canary=false$' "$REPORT"
rg -q '^load_test_scenario=block$' "$REPORT"
rg -q '^load_test_gamemode=creative$' "$REPORT"
rg -q '^bot_count=500$' "$REPORT"
rg -q '^view_distance=32$' "$REPORT"
rg -q '^simulation_distance=32$' "$REPORT"
rg -q '^bot_action_start_mode=all-ready$' "$REPORT"
rg -q '^bot_action_ready_settle_ms=15000$' "$REPORT"
rg -q '^bot_action_ready_requires_block_armed=true$' "$REPORT"
rg -q '^bot_swarm_node_max_old_space_mb=512$' "$REPORT"
rg -q '^bot_swarm_node_max_semi_space_mb=16$' "$REPORT"
rg -q '^summary_report=.*load-p500-contended-diagnostic-smoke-' "$REPORT"
rg -q '^gate_report=.*load-p500-contended-diagnostic-smoke-' "$REPORT"
rg -q '^preflight_report=.*load-p500-contended-diagnostic-smoke-' "$REPORT"
rg -q '^summary_report_status=missing ' "$REPORT"
rg -q '^gate_report_status=missing ' "$REPORT"
rg -q '^preflight_report_status=present ' "$REPORT"
test -f "$PREFLIGHT"
rg -q '^host_preflight_host_checked=false$' "$PREFLIGHT"
rg -q '^host_preflight_host_ok=true$' "$PREFLIGHT"
rg -q '^strict_foreign_process_gate_allowed=true$' "$PREFLIGHT"
rg -q '^strict_foreign_process_gate_pass=true$' "$PREFLIGHT"
rg -q '^strict_foreign_process_count=[1-9][0-9]*$' "$PREFLIGHT"
rg -q 'foreign_process_pattern=server\.jar' "$PREFLIGHT"

set +e
MC_EULA_AGREE=true \
  P500_CONTENDED_DIAGNOSTIC_STAMP="$STAMP-prod-profile" \
  P500_CONTENDED_DIAGNOSTIC_REFRESH_ARTIFACTS=false \
  P500_CONTENDED_DIAGNOSTIC_GATE_PROFILE=production-500 \
  "$ROOT/scripts/run_p500_contended_diagnostic.sh" > "$TMP/production-profile.out" 2> "$TMP/production-profile.err"
status=$?
set -e
if [[ "$status" -ne 64 ]]; then
  echo "expected production profile rejection to exit 64, got $status" >&2
  cat "$TMP/production-profile.out" >&2
  cat "$TMP/production-profile.err" >&2
  exit 1
fi
rg -q 'must not use production claim profiles' "$TMP/production-profile.err"

set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL-prod-foreign" \
LAUNCHER="$FAKE_LAUNCHER" \
FAKE_LAUNCHER_MARKER="$TMP/production-foreign.marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/production-foreign.out" 2> "$TMP/production-foreign.err"
status=$?
set -e
if [[ "$status" -ne 75 ]]; then
  echo "expected production foreign-process override to exit 75, got $status" >&2
  cat "$TMP/production-foreign.out" >&2
  cat "$TMP/production-foreign.err" >&2
  exit 1
fi
rg -q 'LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true is not allowed with production claim profiles' "$TMP/production-foreign.err"
rg -q 'Use a non-production profile for explicitly contaminated/non-claim evidence.' "$TMP/production-foreign.err"

set +e
MC_EULA_AGREE=true \
LOAD_TEST_ALLOW_BUSY_HOST=true \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL-prod-busy" \
LAUNCHER="$FAKE_LAUNCHER" \
FAKE_LAUNCHER_MARKER="$TMP/production-busy.marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/production-busy.out" 2> "$TMP/production-busy.err"
status=$?
set -e
if [[ "$status" -ne 75 ]]; then
  echo "expected production busy-host override to exit 75, got $status" >&2
  cat "$TMP/production-busy.out" >&2
  cat "$TMP/production-busy.err" >&2
  exit 1
fi
rg -q 'LOAD_TEST_ALLOW_BUSY_HOST=true is not allowed with production claim profiles' "$TMP/production-busy.err"
rg -q 'Use a non-production profile for explicitly noisy/non-claim evidence.' "$TMP/production-busy.err"

set +e
MC_EULA_AGREE=true \
LOAD_TEST_ABORT_ON_HOST_CONTENTION=false \
LOAD_TEST_GATE_PROFILE=production-500 \
LOAD_TEST_LABEL="$LABEL-prod-abort" \
LAUNCHER="$FAKE_LAUNCHER" \
FAKE_LAUNCHER_MARKER="$TMP/production-abort.marker" \
  "$ROOT/scripts/run_load_test.sh" 1 1 > "$TMP/production-abort.out" 2> "$TMP/production-abort.err"
status=$?
set -e
if [[ "$status" -ne 75 ]]; then
  echo "expected production abort-policy override to exit 75, got $status" >&2
  cat "$TMP/production-abort.out" >&2
  cat "$TMP/production-abort.err" >&2
  exit 1
fi
rg -q 'LOAD_TEST_ABORT_ON_HOST_CONTENTION=false is not allowed with production claim profiles' "$TMP/production-abort.err"
rg -q 'Production claim profiles must abort contaminated runs on sustained host contention.' "$TMP/production-abort.err"

echo "run_p500_contended_diagnostic_smoke=PASS"
