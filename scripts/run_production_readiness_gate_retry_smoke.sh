#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fake_success_after_retry="$TMP/fake-success-after-retry.sh"
cat > "$fake_success_after_retry" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

state_file="${PRODUCTION_READINESS_GATE_FAKE_STATE:?}"
report_root="${PRODUCTION_READINESS_GATE_REPORT_ROOT:?}"
soak_report="$report_root/reports/production-500-soak-gate.txt"

mkdir -p "$report_root/reports"

attempt=0
if [[ -f "$state_file" ]]; then
  attempt="$(cat "$state_file")"
fi
attempt=$((attempt + 1))
printf '%s\n' "$attempt" > "$state_file"

if (( attempt == 1 )); then
  cat > "$soak_report" <<'EOF'
gate_pass=false
environment_invalid=true
environment_invalid_kind=host_contention
environment_invalid_reason=host_contention_bad_samples=3_load_per_cpu=0.877_max_load_per_cpu=0.750_steal_percent=49.80_max_steal_percent=10.00_iowait_percent=0.03_max_iowait_percent=10.00
run_class=environment-invalid
failure=environment_invalid=true; kind=host_contention; early_abort_reason=host_contention_bad_samples=3
EOF
  exit 1
fi

cat > "$soak_report" <<'EOF'
gate_pass=true
environment_invalid=false
run_class=success
EOF
exit 0
SH
chmod +x "$fake_success_after_retry"

success_out="$TMP/success.out"
success_err="$TMP/success.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_PREFLIGHT=false \
    PRODUCTION_READINESS_GATE_INNER="$fake_success_after_retry" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=2 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000000 \
    PRODUCTION_READINESS_GATE_FAKE_STATE="$TMP/state-success" \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$success_out" 2>"$success_err"
status=$?
set -e

if [[ "$status" -ne 0 ]]; then
  echo "expected host-contention retry smoke to succeed, got $status" >&2
  cat "$success_out" >&2
  cat "$success_err" >&2
  exit 1
fi
rg -q '^production_readiness_retry_attempt=1$' "$success_out"
rg -q '^production_readiness_retry_refresh_soak=true$' "$success_out"
rg -q '^production_readiness_retry_refresh_repeat=true$' "$success_out"
rg -q '^production_readiness_retry_refresh_compat=true$' "$success_out"
rg -q '^production_readiness_retry_repeat_count=3$' "$success_out"
rg -q '^production_readiness_retry_retrying=true attempt=1 delay_seconds=0 reason=host_contention report=.+' "$success_out"
rg -q '^production_readiness_retry_attempt=2$' "$success_out"
rg -q '^production_readiness_retry_pass=true attempts=2$' "$success_out"
rg -q '^gate_pass=true$' "$TMP/reports/production-500-soak-gate.txt"

fake_foreign_preflight="$TMP/fake-foreign-preflight.sh"
cat > "$fake_foreign_preflight" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

report="${REPORT:?}"
mkdir -p "$(dirname "$report")"
cat > "$report" <<'EOF'
production_500_go_nogo_pass=false
production_500_go_nogo_exit_code=75
production_500_go_nogo_reason=strict_foreign_process_present
EOF
exit 75
SH
chmod +x "$fake_foreign_preflight"

fake_noop_inner="$TMP/fake-noop-inner.sh"
cat > "$fake_noop_inner" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

marker="${PRODUCTION_READINESS_GATE_FAKE_INNER_MARKER:?}"
printf 'load_test_diagnostic_mode=%s\n' "${LOAD_TEST_DIAGNOSTIC_MODE:-unset}" > "$marker"
exit 0
SH
chmod +x "$fake_noop_inner"

fake_degraded_diagnostic="$TMP/fake-degraded-diagnostic.sh"
cat > "$fake_degraded_diagnostic" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

report="${P500_CONTENDED_DIAGNOSTIC_REPORT:?}"
marker="${PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER:?}"
mkdir -p "$(dirname "$report")"
{
  printf 'p500_contended_diagnostic_report=%s\n' "$report"
  printf 'p500_contended_diagnostic_label=%s\n' "${P500_CONTENDED_DIAGNOSTIC_LABEL:?}"
  printf 'p500_contended_diagnostic_non_claim=true\n'
  printf 'p500_contended_diagnostic_production_claim_eligible=false\n'
  printf 'p500_contended_diagnostic_exit_code=0\n'
} > "$report"
{
  printf 'mc_eula=%s\n' "${MC_EULA_AGREE:-unset}"
  printf 'refresh_artifacts=%s\n' "${P500_CONTENDED_DIAGNOSTIC_REFRESH_ARTIFACTS:-unset}"
  printf 'label=%s\n' "${P500_CONTENDED_DIAGNOSTIC_LABEL:-unset}"
  printf 'report=%s\n' "$report"
} > "$marker"
exit 0
SH
chmod +x "$fake_degraded_diagnostic"

foreign_marker="$TMP/fake-inner-marker"
foreign_out="$TMP/foreign.out"
foreign_err="$TMP/foreign.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_PREFLIGHT=true \
    PRODUCTION_READINESS_PREFLIGHT_COMMAND="$fake_foreign_preflight" \
    PRODUCTION_READINESS_GATE_INNER="$fake_noop_inner" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP/foreign" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=2 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000000-foreign \
    PRODUCTION_READINESS_GATE_FAKE_INNER_MARKER="$foreign_marker" \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$foreign_out" 2>"$foreign_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected strict foreign preflight to stop the run with 75, got $status" >&2
  cat "$foreign_out" >&2
  cat "$foreign_err" >&2
  exit 1
fi
test ! -e "$foreign_marker"
rg -q '^production_readiness_preflight_enabled=true$' "$foreign_out"
rg -q '^production_readiness_preflight_pass=false attempt=1 reason=strict_foreign_process_present exit_code=75 report=.+' "$foreign_out"
rg -q '^production_readiness_heavy_gate_allowed=false$' "$foreign_out"
rg -q '^production_readiness_next_action=stop_foreign_process$' "$foreign_out"

strict_diagnostic_degraded_marker="$TMP/fake-degraded-strict-diagnostic-marker"
strict_diagnostic_inner_report="$TMP/strict-diagnostic-inner-run.txt"
strict_diagnostic_out="$TMP/strict-diagnostic.out"
strict_diagnostic_err="$TMP/strict-diagnostic.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_DIAGNOSTIC_MODE=true \
    PRODUCTION_READINESS_PREFLIGHT=true \
    PRODUCTION_READINESS_PREFLIGHT_COMMAND="$fake_foreign_preflight" \
    PRODUCTION_READINESS_GATE_INNER="$ROOT/scripts/run_production_readiness_gate.sh" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND="$fake_degraded_diagnostic" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT="$TMP/strict-diagnostic-p500-report.txt" \
    PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER="$strict_diagnostic_degraded_marker" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$strict_diagnostic_inner_report" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP/strict-diagnostic" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=0 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000000-strict-diagnostic \
    PRODUCTION_READINESS_GATE_STAMP=20260523-000000-strict-diagnostic-inner \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$strict_diagnostic_out" 2>"$strict_diagnostic_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected strict diagnostic preflight to run degraded diagnostic and exit 75, got $status" >&2
  cat "$strict_diagnostic_out" >&2
  cat "$strict_diagnostic_err" >&2
  exit 1
fi
test -e "$strict_diagnostic_degraded_marker"
test -f "$strict_diagnostic_inner_report"
rg -q '^production_readiness_diagnostic_mode=true$' "$strict_diagnostic_out"
rg -q '^production_readiness_evidence_mode=diagnostic_non_claim$' "$strict_diagnostic_out"
rg -q '^production_readiness_claim_eligible=false$' "$strict_diagnostic_out"
rg -q '^production_readiness_preflight_pass=false attempt=1 reason=strict_foreign_process_present exit_code=75 report=.+' "$strict_diagnostic_out"
rg -q '^production_readiness_heavy_gate_allowed=false$' "$strict_diagnostic_out"
rg -q '^production_readiness_next_action=run_degraded_p500_diagnostic_without_soak$' "$strict_diagnostic_out"
rg -q '^readiness_degraded_p500_diagnostic_reason=diagnostic_preflight_failed_strict_foreign_process_present$' "$strict_diagnostic_inner_report"
rg -q '^readiness_degraded_p500_diagnostic_trigger_status=75$' "$strict_diagnostic_inner_report"
rg -q '^readiness_degraded_p500_diagnostic_non_claim=true$' "$strict_diagnostic_inner_report"
rg -q '^readiness_degraded_p500_diagnostic_production_claim_eligible=false$' "$strict_diagnostic_inner_report"
rg -q '^readiness_p500_contended_diagnostic_phase=preflight$' "$strict_diagnostic_inner_report"
rg -q '^production_readiness_retry_exit_reason=diagnostic_non_claim_not_publishable$' "$strict_diagnostic_out"
rg -q '^refresh_artifacts=false$' "$strict_diagnostic_degraded_marker"

fake_diagnostic_preflight="$TMP/fake-diagnostic-foreign-preflight.sh"
cat > "$fake_diagnostic_preflight" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

report="${REPORT:?}"
mkdir -p "$(dirname "$report")"
cat > "$report" <<'EOF'
production_500_go_nogo_pass=false
production_500_go_nogo_exit_code=0
production_500_go_nogo_reason=diagnostic_foreign_process_present
production_500_go_nogo_diagnostic_mode=true
EOF
exit 0
SH
chmod +x "$fake_diagnostic_preflight"

diagnostic_marker="$TMP/fake-inner-diagnostic-marker"
diagnostic_degraded_marker="$TMP/fake-degraded-diagnostic-marker"
diagnostic_inner_report="$TMP/diagnostic-inner-run.txt"
diagnostic_out="$TMP/diagnostic.out"
diagnostic_err="$TMP/diagnostic.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_DIAGNOSTIC_MODE=true \
    PRODUCTION_READINESS_PREFLIGHT=true \
    PRODUCTION_READINESS_PREFLIGHT_COMMAND="$fake_diagnostic_preflight" \
    PRODUCTION_READINESS_GATE_INNER="$ROOT/scripts/run_production_readiness_gate.sh" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND="$fake_degraded_diagnostic" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT="$TMP/diagnostic-p500-report.txt" \
    PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER="$diagnostic_degraded_marker" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$diagnostic_inner_report" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP/diagnostic" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=0 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000000-diagnostic \
    PRODUCTION_READINESS_GATE_STAMP=20260523-000000-diagnostic-inner \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$diagnostic_out" 2>"$diagnostic_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected diagnostic foreign preflight to run degraded diagnostic and exit 75, got $status" >&2
  cat "$diagnostic_out" >&2
  cat "$diagnostic_err" >&2
  exit 1
fi
test ! -e "$diagnostic_marker"
test -e "$diagnostic_degraded_marker"
test -f "$diagnostic_inner_report"
rg -q '^production_readiness_diagnostic_mode=true$' "$diagnostic_out"
rg -q '^production_readiness_preflight_pass=false attempt=1 reason=diagnostic_foreign_process_present exit_code=0 report=.+' "$diagnostic_out"
rg -q '^production_readiness_heavy_gate_allowed=false$' "$diagnostic_out"
rg -q '^production_readiness_next_action=run_degraded_p500_diagnostic_without_soak$' "$diagnostic_out"
rg -q '^readiness_degraded_p500_diagnostic_reason=diagnostic_preflight_failed_diagnostic_foreign_process_present$' "$diagnostic_inner_report"
rg -q '^readiness_degraded_p500_diagnostic_non_claim=true$' "$diagnostic_inner_report"
rg -q '^readiness_degraded_p500_diagnostic_production_claim_eligible=false$' "$diagnostic_inner_report"
rg -q '^readiness_p500_contended_diagnostic_phase=preflight$' "$diagnostic_inner_report"
rg -q '^production_readiness_retry_exit_reason=diagnostic_non_claim_not_publishable$' "$diagnostic_out"
rg -q '^refresh_artifacts=false$' "$diagnostic_degraded_marker"

fake_diagnostic_canary_preflight="$TMP/fake-diagnostic-canary-preflight.sh"
cat > "$fake_diagnostic_canary_preflight" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

report="${REPORT:?}"
mkdir -p "$(dirname "$report")"
cat > "$report" <<'EOF'
production_500_go_nogo_pass=false
production_500_go_nogo_exit_code=0
production_500_go_nogo_reason=diagnostic_host_synthetic_canary_failed
production_500_go_nogo_diagnostic_mode=true
EOF
exit 0
SH
chmod +x "$fake_diagnostic_canary_preflight"

canary_marker="$TMP/fake-inner-canary-marker"
canary_degraded_marker="$TMP/fake-degraded-canary-marker"
canary_inner_report="$TMP/canary-inner-run.txt"
canary_out="$TMP/canary.out"
canary_err="$TMP/canary.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_DIAGNOSTIC_MODE=true \
    PRODUCTION_READINESS_PREFLIGHT=true \
    PRODUCTION_READINESS_PREFLIGHT_COMMAND="$fake_diagnostic_canary_preflight" \
    PRODUCTION_READINESS_GATE_INNER="$ROOT/scripts/run_production_readiness_gate.sh" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND="$fake_degraded_diagnostic" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT="$TMP/canary-p500-report.txt" \
    PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER="$canary_degraded_marker" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$canary_inner_report" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP/canary" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=0 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000000-canary \
    PRODUCTION_READINESS_GATE_STAMP=20260523-000000-canary-inner \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$canary_out" 2>"$canary_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected diagnostic canary preflight to run degraded diagnostic and exit 75, got $status" >&2
  cat "$canary_out" >&2
  cat "$canary_err" >&2
  exit 1
fi
test ! -e "$canary_marker"
test -e "$canary_degraded_marker"
test -f "$canary_inner_report"
rg -q '^production_readiness_preflight_pass=false attempt=1 reason=diagnostic_host_synthetic_canary_failed exit_code=0 report=.+' "$canary_out"
rg -q '^production_readiness_heavy_gate_allowed=false$' "$canary_out"
rg -q '^production_readiness_next_action=run_degraded_p500_diagnostic_without_soak$' "$canary_out"
rg -q '^readiness_degraded_p500_diagnostic_reason=diagnostic_preflight_failed_diagnostic_host_synthetic_canary_failed$' "$canary_inner_report"
rg -q '^readiness_p500_contended_diagnostic_phase=preflight$' "$canary_inner_report"
rg -q '^production_readiness_retry_exit_reason=diagnostic_non_claim_not_publishable$' "$canary_out"
rg -q '^refresh_artifacts=false$' "$canary_degraded_marker"

fake_stale_report="$TMP/fake-stale-report.sh"
cat > "$fake_stale_report" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

exit 1
SH
chmod +x "$fake_stale_report"

mkdir -p "$TMP/stale/reports"
cat > "$TMP/stale/reports/production-500-soak-gate.txt" <<'EOF'
gate_pass=false
environment_invalid=true
environment_invalid_kind=host_contention
environment_invalid_reason=host_contention_bad_samples=3_load_per_cpu=0.877_max_load_per_cpu=0.750_steal_percent=49.80_max_steal_percent=10.00_iowait_percent=0.03_max_iowait_percent=10.00
run_class=environment-invalid
failure=environment_invalid=true; kind=host_contention; early_abort_reason=host_contention_bad_samples=3
EOF
python3 - "$TMP/stale/reports/production-500-soak-gate.txt" <<'PY'
from __future__ import annotations

from pathlib import Path
import os
import sys

path = Path(sys.argv[1])
stale_epoch = 1_700_000_000
os.utime(path, (stale_epoch, stale_epoch))
PY

stale_out="$TMP/stale.out"
stale_err="$TMP/stale.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_PREFLIGHT=false \
    PRODUCTION_READINESS_GATE_INNER="$fake_stale_report" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP/stale" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=2 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000000-stale \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$stale_out" 2>"$stale_err"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
  echo "expected stale host-contention evidence to stay non-retryable" >&2
  cat "$stale_out" >&2
  cat "$stale_err" >&2
  exit 1
fi
rg -q '^production_readiness_retry_attempt=1$' "$stale_out"
rg -q '^production_readiness_retry_retryable=false attempt=1 last_status=1$' "$stale_out"
if rg -q '^production_readiness_retry_retrying=true' "$stale_out"; then
  echo "stale host-contention evidence must not trigger a retry" >&2
  cat "$stale_out" >&2
  exit 1
fi

fake_non_retryable="$TMP/fake-non-retryable.sh"
cat > "$fake_non_retryable" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

report_root="${PRODUCTION_READINESS_GATE_REPORT_ROOT:?}"
soak_report="$report_root/reports/production-500-soak-gate.txt"

mkdir -p "$report_root/reports"
cat > "$soak_report" <<'EOF'
gate_pass=false
environment_invalid=true
environment_invalid_kind=foreign_process
run_class=environment-invalid
failure=environment_invalid=true; kind=foreign_process
EOF
exit 1
SH
chmod +x "$fake_non_retryable"

failure_out="$TMP/failure.out"
failure_err="$TMP/failure.err"
set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_PREFLIGHT=false \
    PRODUCTION_READINESS_GATE_INNER="$fake_non_retryable" \
    PRODUCTION_READINESS_GATE_REPORT_ROOT="$TMP/non-retryable" \
    PRODUCTION_READINESS_GATE_RETRY_COUNT=2 \
    PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS=0 \
    PRODUCTION_READINESS_GATE_RETRY_STAMP=20260523-000001 \
    "$ROOT/scripts/run_production_readiness_gate_retry.sh"
) >"$failure_out" 2>"$failure_err"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
  echo "expected non-host-contention failure to remain failed" >&2
  cat "$failure_out" >&2
  cat "$failure_err" >&2
  exit 1
fi
rg -q '^production_readiness_retry_attempt=1$' "$failure_out"
rg -q '^production_readiness_retry_retryable=false attempt=1 last_status=1$' "$failure_out"
if rg -q '^production_readiness_retry_retrying=true' "$failure_out"; then
  echo "non-host-contention failure must not retry" >&2
  cat "$failure_out" >&2
  exit 1
fi

echo "run_production_readiness_gate_retry_smoke=PASS"
