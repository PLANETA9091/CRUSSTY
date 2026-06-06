#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fake_preflight="$TMP/fake-preflight.sh"
cat > "$fake_preflight" <<'SH'
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
chmod +x "$fake_preflight"

run_report="$TMP/readiness-run.txt"
go_nogo_report="$TMP/go-nogo.txt"
out="$TMP/out.txt"
err="$TMP/err.txt"

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_PREFLIGHT=true \
    PRODUCTION_READINESS_PREFLIGHT_COMMAND="$fake_preflight" \
    PRODUCTION_READINESS_PREFLIGHT_REPORT="$go_nogo_report" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$run_report" \
    PRODUCTION_READINESS_GATE_STAMP=preflight-smoke \
    ./scripts/run_production_readiness_gate.sh
) >"$out" 2>"$err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected readiness gate preflight to stop with 75, got $status" >&2
  cat "$out" >&2
  cat "$err" >&2
  exit 1
fi

test -f "$run_report"
test -f "$go_nogo_report"
rg -q '^readiness_preflight_enabled=true$' "$run_report"
rg -q '^readiness_preflight_pass=false reason=strict_foreign_process_present exit_code=75 report=.+' "$run_report"
rg -q '^readiness_heavy_gate_allowed=false$' "$run_report"
rg -q '^readiness_next_action=stop_foreign_process$' "$run_report"
if rg -q '^running_soak_gate=' "$run_report"; then
  echo "direct readiness gate must not reach heavy stages after failed preflight" >&2
  cat "$run_report" >&2
  exit 1
fi

fake_soak_failure="$TMP/fake-soak-failure.sh"
cat > "$fake_soak_failure" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake_soak_started=true\n'
printf 'load_window_policy=prelaunch-abort\n'
printf 'environment_invalid_kind=host_contention\n'
exit 75
SH
chmod +x "$fake_soak_failure"

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

strict_diagnostic_marker="$TMP/strict-diagnostic-marker.txt"
strict_diagnostic_run_report="$TMP/readiness-strict-diagnostic-run.txt"
strict_diagnostic_out="$TMP/strict-diagnostic-out.txt"
strict_diagnostic_err="$TMP/strict-diagnostic-err.txt"

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_DIAGNOSTIC_MODE=true \
    PRODUCTION_READINESS_PREFLIGHT=true \
    PRODUCTION_READINESS_PREFLIGHT_COMMAND="$fake_preflight" \
    PRODUCTION_READINESS_PREFLIGHT_REPORT="$TMP/strict-diagnostic-go-nogo.txt" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND="$fake_degraded_diagnostic" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT="$TMP/strict-diagnostic-p500-report.txt" \
    PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER="$strict_diagnostic_marker" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$strict_diagnostic_run_report" \
    PRODUCTION_READINESS_GATE_STAMP=strict-diagnostic-preflight-smoke \
    ./scripts/run_production_readiness_gate.sh
) >"$strict_diagnostic_out" 2>"$strict_diagnostic_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected diagnostic readiness gate strict preflight to exit 75 after degraded diagnostic, got $status" >&2
  cat "$strict_diagnostic_out" >&2
  cat "$strict_diagnostic_err" >&2
  exit 1
fi

test -f "$strict_diagnostic_run_report"
test -f "$strict_diagnostic_marker"
rg -q '^readiness_diagnostic_mode=true$' "$strict_diagnostic_run_report"
rg -q '^readiness_evidence_mode=diagnostic_non_claim$' "$strict_diagnostic_run_report"
rg -q '^readiness_claim_eligible=false$' "$strict_diagnostic_run_report"
rg -q '^readiness_preflight_pass=false reason=strict_foreign_process_present exit_code=75 report=' "$strict_diagnostic_run_report"
rg -q '^readiness_heavy_gate_allowed=false$' "$strict_diagnostic_run_report"
rg -q '^readiness_next_action=run_degraded_p500_diagnostic_without_soak$' "$strict_diagnostic_run_report"
rg -q '^running_soak_gate=false$' "$strict_diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic=true$' "$strict_diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_reason=diagnostic_preflight_failed_strict_foreign_process_present$' "$strict_diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_trigger_status=75$' "$strict_diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_non_claim=true$' "$strict_diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_production_claim_eligible=false$' "$strict_diagnostic_run_report"
rg -q '^readiness_p500_contended_diagnostic_phase=preflight$' "$strict_diagnostic_run_report"
rg -q '^readiness_exit_reason=diagnostic_preflight_failed_after_degraded_diagnostic$' "$strict_diagnostic_run_report"
rg -q '^refresh_artifacts=false$' "$strict_diagnostic_marker"
if rg -q '^running_repeat_gate=' "$strict_diagnostic_run_report"; then
  echo "diagnostic preflight fallback must not continue to later claim gates" >&2
  cat "$strict_diagnostic_run_report" >&2
  exit 1
fi

if "$ROOT/scripts/check_artifact_source_freshness.sh" >/dev/null 2>&1; then
diagnostic_marker="$TMP/degraded-diagnostic-marker.txt"
diagnostic_run_report="$TMP/readiness-diagnostic-run.txt"
diagnostic_out="$TMP/diagnostic-out.txt"
diagnostic_err="$TMP/diagnostic-err.txt"

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_DIAGNOSTIC_MODE=true \
    PRODUCTION_READINESS_PREFLIGHT=false \
    PRODUCTION_READINESS_REFRESH_SOAK=true \
    PRODUCTION_READINESS_REFRESH_REPEAT=false \
    PRODUCTION_READINESS_REFRESH_COMPAT=false \
    PRODUCTION_READINESS_SOAK_COMMAND="$fake_soak_failure" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND="$fake_degraded_diagnostic" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT="$TMP/diagnostic-p500-report.txt" \
    PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER="$diagnostic_marker" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$diagnostic_run_report" \
    PRODUCTION_READINESS_GATE_STAMP=diagnostic-fallback-smoke \
    ./scripts/run_production_readiness_gate.sh
) >"$diagnostic_out" 2>"$diagnostic_err"
status=$?
set -e

if [[ "$status" -ne 75 ]]; then
  echo "expected diagnostic readiness gate to preserve failed soak status 75, got $status" >&2
  cat "$diagnostic_out" >&2
  cat "$diagnostic_err" >&2
  exit 1
fi

test -f "$diagnostic_run_report"
test -f "$diagnostic_marker"
rg -q '^readiness_diagnostic_mode=true$' "$diagnostic_run_report"
rg -q '^readiness_preflight_pass=skipped reason=disabled exit_code=0 report=' "$diagnostic_run_report"
rg -q '^running_soak_gate=true$' "$diagnostic_run_report"
rg -q '^fake_soak_started=true$' "$diagnostic_run_report"
rg -q '^readiness_soak_gate_exit=75$' "$diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic=true$' "$diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_reason=production_soak_failed_under_diagnostic_mode$' "$diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_trigger_status=75$' "$diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_non_claim=true$' "$diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_production_claim_eligible=false$' "$diagnostic_run_report"
rg -q '^readiness_degraded_p500_diagnostic_exit=0$' "$diagnostic_run_report"
rg -q '^readiness_p500_contended_diagnostic_launched=true$' "$diagnostic_run_report"
rg -q '^readiness_p500_contended_diagnostic_phase=soak$' "$diagnostic_run_report"
rg -q '^readiness_p500_contended_diagnostic_exit_code=0$' "$diagnostic_run_report"
rg -q '^readiness_exit_reason=soak_gate_failed_after_degraded_diagnostic$' "$diagnostic_run_report"
rg -q '^refresh_artifacts=false$' "$diagnostic_marker"
if rg -q '^running_repeat_gate=' "$diagnostic_run_report"; then
  echo "diagnostic fallback should exit after failed soak instead of continuing to later claim gates" >&2
  cat "$diagnostic_run_report" >&2
  exit 1
fi

fake_soak_non_host_failure="$TMP/fake-soak-non-host-failure.sh"
cat > "$fake_soak_non_host_failure" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake_soak_non_host_started=true\n'
printf 'gate_pass=false\n'
printf 'failure=compat_probe_block_evidence_accepted is missing, expected true\n'
exit 1
SH
chmod +x "$fake_soak_non_host_failure"

non_host_marker="$TMP/non-host-diagnostic-marker.txt"
non_host_run_report="$TMP/readiness-non-host-run.txt"
non_host_out="$TMP/non-host-out.txt"
non_host_err="$TMP/non-host-err.txt"

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    PRODUCTION_READINESS_DIAGNOSTIC_MODE=true \
    PRODUCTION_READINESS_PREFLIGHT=false \
    PRODUCTION_READINESS_REFRESH_SOAK=true \
    PRODUCTION_READINESS_REFRESH_REPEAT=false \
    PRODUCTION_READINESS_REFRESH_COMPAT=false \
    PRODUCTION_READINESS_SOAK_COMMAND="$fake_soak_non_host_failure" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND="$fake_degraded_diagnostic" \
    PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT="$TMP/non-host-p500-report.txt" \
    PRODUCTION_READINESS_GATE_FAKE_DIAGNOSTIC_MARKER="$non_host_marker" \
    PRODUCTION_READINESS_GATE_RUN_REPORT="$non_host_run_report" \
    PRODUCTION_READINESS_GATE_STAMP=diagnostic-non-host-smoke \
    ./scripts/run_production_readiness_gate.sh
) >"$non_host_out" 2>"$non_host_err"
status=$?
set -e

if [[ "$status" -ne 1 ]]; then
  echo "expected non-host diagnostic readiness gate to preserve failed soak status 1, got $status" >&2
  cat "$non_host_out" >&2
  cat "$non_host_err" >&2
  exit 1
fi

test -f "$non_host_run_report"
test ! -e "$non_host_marker"
rg -q '^fake_soak_non_host_started=true$' "$non_host_run_report"
rg -q '^readiness_soak_gate_exit=1$' "$non_host_run_report"
rg -q '^readiness_degraded_p500_diagnostic=false$' "$non_host_run_report"
rg -q '^readiness_degraded_p500_diagnostic_reason=no_host_contention_prelaunch_evidence$' "$non_host_run_report"
rg -q '^readiness_p500_contended_diagnostic_launched=false$' "$non_host_run_report"
rg -q '^readiness_exit_reason=soak_gate_failed_without_degraded_diagnostic$' "$non_host_run_report"
else
  echo "run_production_readiness_gate_preflight_smoke_soak_fallback=SKIP reason=artifact_source_freshness_failed"
fi

echo "run_production_readiness_gate_preflight_smoke=PASS"
