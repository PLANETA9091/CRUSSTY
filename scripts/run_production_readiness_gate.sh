#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

STAMP="${PRODUCTION_READINESS_GATE_STAMP:-$(date +%Y%m%d-%H%M%S)}"
RUN_REPORT="${PRODUCTION_READINESS_GATE_RUN_REPORT:-$ROOT/reports/production-500-readiness-run-${STAMP}.txt}"
REFRESH_SOAK="${PRODUCTION_READINESS_REFRESH_SOAK:-true}"
REFRESH_REPEAT="${PRODUCTION_READINESS_REFRESH_REPEAT:-true}"
REFRESH_COMPAT="${PRODUCTION_READINESS_REFRESH_COMPAT:-true}"
PREFLIGHT_ENABLED="${PRODUCTION_READINESS_PREFLIGHT:-true}"
DIAGNOSTIC_MODE="${PRODUCTION_READINESS_DIAGNOSTIC_MODE:-false}"
OUTER_PREFLIGHT_PASSED="${PRODUCTION_READINESS_OUTER_PREFLIGHT_PASSED:-false}"
PREFLIGHT_COMMAND="${PRODUCTION_READINESS_PREFLIGHT_COMMAND:-$ROOT/scripts/check_production_500_go_nogo.sh}"
PREFLIGHT_REPORT="${PRODUCTION_READINESS_PREFLIGHT_REPORT:-$ROOT/reports/production-500-go-nogo-current.txt}"
SOAK_COMMAND="${PRODUCTION_READINESS_SOAK_COMMAND:-$ROOT/scripts/run_production_soak_gate.sh}"
DEGRADED_DIAGNOSTIC_COMMAND="${PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_COMMAND:-$ROOT/scripts/run_p500_contended_diagnostic.sh}"
SOAK_REFRESH_OUTPUT="$ROOT/reports/production-500-soak-refresh-${STAMP}.txt"
DEGRADED_PREFLIGHT_REASON="${PRODUCTION_READINESS_DEGRADED_PREFLIGHT_REASON:-}"
DEGRADED_PREFLIGHT_REPORT="${PRODUCTION_READINESS_DEGRADED_PREFLIGHT_REPORT:-$PREFLIGHT_REPORT}"

case "$REFRESH_SOAK" in true|false) ;; *) echo "PRODUCTION_READINESS_REFRESH_SOAK must be true or false." >&2; exit 64 ;; esac
case "$REFRESH_REPEAT" in true|false) ;; *) echo "PRODUCTION_READINESS_REFRESH_REPEAT must be true or false." >&2; exit 64 ;; esac
case "$REFRESH_COMPAT" in true|false) ;; *) echo "PRODUCTION_READINESS_REFRESH_COMPAT must be true or false." >&2; exit 64 ;; esac
case "$PREFLIGHT_ENABLED" in true|false) ;; *) echo "PRODUCTION_READINESS_PREFLIGHT must be true or false." >&2; exit 64 ;; esac
case "$DIAGNOSTIC_MODE" in true|false) ;; *) echo "PRODUCTION_READINESS_DIAGNOSTIC_MODE must be true or false." >&2; exit 64 ;; esac
case "$OUTER_PREFLIGHT_PASSED" in true|false) ;; *) echo "PRODUCTION_READINESS_OUTER_PREFLIGHT_PASSED must be true or false." >&2; exit 64 ;; esac
export LOAD_TEST_DIAGNOSTIC_MODE="$DIAGNOSTIC_MODE"

mkdir -p "$ROOT/reports"

read_report_value() {
  local key="$1"
  local path="$2"
  awk -F= -v key="$key" '$1 == key { sub(/^[^=]*=/, "", $0); print; exit }' "$path" 2>/dev/null || true
}

run_preflight() {
  local started_epoch tmp_report report_mtime
  started_epoch="$(date +%s)"
  tmp_report="${PREFLIGHT_REPORT}.${STAMP}.$$.tmp"
  rm -f "$tmp_report"
  set +e
  {
    REPORT="$tmp_report" \
      PRODUCTION_READINESS_DIAGNOSTIC_MODE="$DIAGNOSTIC_MODE" \
      "$PREFLIGHT_COMMAND"
  } 2>&1 | tee -a "$RUN_REPORT"
  PREFLIGHT_STATUS=${PIPESTATUS[0]}
  set -e
  if [[ ! -f "$tmp_report" ]]; then
    PREFLIGHT_PASS=false
    PREFLIGHT_REASON=missing_preflight_report
    PREFLIGHT_EXIT_CODE="${PREFLIGHT_STATUS:-1}"
    return 0
  fi
  report_mtime="$(stat -c %Y "$tmp_report")"
  if (( report_mtime < started_epoch )); then
    PREFLIGHT_PASS=false
    PREFLIGHT_REASON=stale_preflight_report
    PREFLIGHT_EXIT_CODE="${PREFLIGHT_STATUS:-1}"
    rm -f "$tmp_report"
    return 0
  fi
  mv -f "$tmp_report" "$PREFLIGHT_REPORT"
  PREFLIGHT_PASS="$(read_report_value production_500_go_nogo_pass "$PREFLIGHT_REPORT")"
  PREFLIGHT_REASON="$(read_report_value production_500_go_nogo_reason "$PREFLIGHT_REPORT")"
  PREFLIGHT_EXIT_CODE="$(read_report_value production_500_go_nogo_exit_code "$PREFLIGHT_REPORT")"
}

if [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
  READINESS_EVIDENCE_MODE=diagnostic_non_claim
  READINESS_CLAIM_ELIGIBLE=false
elif [[ "$PREFLIGHT_ENABLED" == "true" || "$OUTER_PREFLIGHT_PASSED" == "true" ]]; then
  READINESS_EVIDENCE_MODE=strict_claim_candidate
  READINESS_CLAIM_ELIGIBLE=true
else
  READINESS_EVIDENCE_MODE=preflight_skipped_non_claim
  READINESS_CLAIM_ELIGIBLE=false
fi

{
  echo "readiness_run_stamp=$STAMP"
  echo "refresh_soak=$REFRESH_SOAK"
  echo "refresh_repeat=$REFRESH_REPEAT"
  echo "refresh_compat=$REFRESH_COMPAT"
  echo "readiness_preflight_enabled=$PREFLIGHT_ENABLED"
  echo "readiness_diagnostic_mode=$DIAGNOSTIC_MODE"
  echo "readiness_outer_preflight_passed=$OUTER_PREFLIGHT_PASSED"
  echo "readiness_evidence_mode=$READINESS_EVIDENCE_MODE"
  echo "readiness_claim_eligible=$READINESS_CLAIM_ELIGIBLE"
  echo "readiness_preflight_command=$PREFLIGHT_COMMAND"
  echo "readiness_preflight_report=$PREFLIGHT_REPORT"
  echo "readiness_soak_command=$SOAK_COMMAND"
  echo "readiness_soak_refresh_output=$SOAK_REFRESH_OUTPUT"
  echo "readiness_degraded_diagnostic_command=$DEGRADED_DIAGNOSTIC_COMMAND"
  if [[ -n "$DEGRADED_PREFLIGHT_REASON" ]]; then
    echo "readiness_degraded_preflight_reason=$DEGRADED_PREFLIGHT_REASON"
    echo "readiness_degraded_preflight_report=$DEGRADED_PREFLIGHT_REPORT"
  fi
} | tee "$RUN_REPORT"

should_run_degraded_p500_diagnostic() {
  local evidence_path
  for evidence_path in "$@"; do
    [[ -f "$evidence_path" ]] || continue
    if grep -Eq '^(load_window_policy=prelaunch-abort|environment_invalid_kind=host_contention|environment_invalid_reason=.*host_contention|failure=environment_invalid=true; kind=host_contention|.*host_contention_prelaunch_)' "$evidence_path"; then
      echo "$evidence_path"
      return 0
    fi
  done
  return 1
}

run_degraded_p500_diagnostic() {
  local triggering_status="$1"
  local triggering_evidence="$2"
  local diagnostic_reason="${3:-production_soak_failed_under_diagnostic_mode}"
  local diagnostic_phase="${4:-soak}"
  local diagnostic_stamp="${PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_STAMP:-${STAMP}-degraded-host}"
  local diagnostic_label="${PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_LABEL:-p500-contended-diagnostic-readiness-${diagnostic_stamp}}"
  local diagnostic_report="${PRODUCTION_READINESS_DEGRADED_DIAGNOSTIC_REPORT:-$ROOT/reports/p500-contended-diagnostic-${diagnostic_stamp}.txt}"
  local diagnostic_status=0

  {
    echo "readiness_degraded_p500_diagnostic=true"
    echo "readiness_degraded_p500_diagnostic_reason=$diagnostic_reason"
    echo "readiness_degraded_p500_diagnostic_trigger_status=$triggering_status"
    echo "readiness_degraded_p500_diagnostic_trigger_evidence=$triggering_evidence"
    echo "readiness_degraded_p500_diagnostic_non_claim=true"
    echo "readiness_degraded_p500_diagnostic_production_claim_eligible=false"
    echo "readiness_degraded_p500_diagnostic_stamp=$diagnostic_stamp"
    echo "readiness_degraded_p500_diagnostic_label=$diagnostic_label"
    echo "readiness_degraded_p500_diagnostic_report=$diagnostic_report"
    echo "readiness_p500_contended_diagnostic_launched=true"
    echo "readiness_p500_contended_diagnostic_phase=$diagnostic_phase"
    echo "readiness_p500_contended_diagnostic_report=$diagnostic_report"
  } | tee -a "$RUN_REPORT"

  set +e
  {
    MC_EULA_AGREE=true \
      P500_CONTENDED_DIAGNOSTIC_STAMP="$diagnostic_stamp" \
      P500_CONTENDED_DIAGNOSTIC_LABEL="$diagnostic_label" \
      P500_CONTENDED_DIAGNOSTIC_REPORT="$diagnostic_report" \
      P500_CONTENDED_DIAGNOSTIC_REFRESH_ARTIFACTS=false \
      "$DEGRADED_DIAGNOSTIC_COMMAND"
  } 2>&1 | tee -a "$RUN_REPORT"
  diagnostic_status=${PIPESTATUS[0]}
  set -e

  echo "readiness_degraded_p500_diagnostic_exit=$diagnostic_status" | tee -a "$RUN_REPORT"
  echo "readiness_p500_contended_diagnostic_exit_code=$diagnostic_status" | tee -a "$RUN_REPORT"
}

if [[ "$PREFLIGHT_ENABLED" == "true" ]]; then
  run_preflight
  PREFLIGHT_REASON="${PREFLIGHT_REASON:-unknown}"
  PREFLIGHT_PASS="${PREFLIGHT_PASS:-false}"
  PREFLIGHT_EXIT_CODE="${PREFLIGHT_EXIT_CODE:-$PREFLIGHT_STATUS}"
  if [[ "$PREFLIGHT_STATUS" -eq 0 && "$PREFLIGHT_PASS" == "true" ]]; then
    echo "readiness_preflight_pass=true reason=none exit_code=0 report=$PREFLIGHT_REPORT" | tee -a "$RUN_REPORT"
    echo "readiness_heavy_gate_allowed=true" | tee -a "$RUN_REPORT"
  elif [[ "$DIAGNOSTIC_MODE" == "true" && "$PREFLIGHT_STATUS" -eq 0 && "$PREFLIGHT_REASON" == diagnostic_* ]]; then
    echo "readiness_preflight_pass=false reason=$PREFLIGHT_REASON exit_code=0 report=$PREFLIGHT_REPORT" | tee -a "$RUN_REPORT"
    echo "readiness_heavy_gate_allowed=false" | tee -a "$RUN_REPORT"
    echo "readiness_next_action=run_degraded_p500_diagnostic_without_soak" | tee -a "$RUN_REPORT"
    DEGRADED_PREFLIGHT_REASON="$PREFLIGHT_REASON"
    DEGRADED_PREFLIGHT_REPORT="$PREFLIGHT_REPORT"
  elif [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
    echo "readiness_preflight_pass=false reason=$PREFLIGHT_REASON exit_code=$PREFLIGHT_EXIT_CODE report=$PREFLIGHT_REPORT" | tee -a "$RUN_REPORT"
    echo "readiness_heavy_gate_allowed=false" | tee -a "$RUN_REPORT"
    echo "readiness_next_action=run_degraded_p500_diagnostic_without_soak" | tee -a "$RUN_REPORT"
    DEGRADED_PREFLIGHT_REASON="$PREFLIGHT_REASON"
    DEGRADED_PREFLIGHT_REPORT="$PREFLIGHT_REPORT"
  else
    echo "readiness_preflight_pass=false reason=$PREFLIGHT_REASON exit_code=$PREFLIGHT_EXIT_CODE report=$PREFLIGHT_REPORT" | tee -a "$RUN_REPORT"
    echo "readiness_heavy_gate_allowed=false" | tee -a "$RUN_REPORT"
    case "$PREFLIGHT_REASON" in
      strict_foreign_process_present)
        echo "readiness_next_action=stop_foreign_process" | tee -a "$RUN_REPORT"
        ;;
      host_synthetic_canary_failed)
        echo "readiness_next_action=wait_for_clean_host" | tee -a "$RUN_REPORT"
        ;;
      *)
        echo "readiness_next_action=rerun_go_nogo_preflight" | tee -a "$RUN_REPORT"
        ;;
    esac
    exit "${PREFLIGHT_EXIT_CODE:-${PREFLIGHT_STATUS:-1}}"
  fi
else
  echo "readiness_preflight_pass=skipped reason=disabled exit_code=0 report=$PREFLIGHT_REPORT" | tee -a "$RUN_REPORT"
  echo "readiness_heavy_gate_allowed=true" | tee -a "$RUN_REPORT"
fi

if [[ -n "$DEGRADED_PREFLIGHT_REASON" ]]; then
  echo "running_soak_gate=false" | tee -a "$RUN_REPORT"
  run_degraded_p500_diagnostic "${PREFLIGHT_EXIT_CODE:-${PREFLIGHT_STATUS:-75}}" "$DEGRADED_PREFLIGHT_REPORT" "diagnostic_preflight_failed_${DEGRADED_PREFLIGHT_REASON}" preflight
  echo "readiness_exit_reason=diagnostic_preflight_failed_after_degraded_diagnostic" | tee -a "$RUN_REPORT"
  exit 75
fi

python3 "$ROOT/scripts/update_artifact_reports.py" | tee -a "$RUN_REPORT"
sha256sum -c "$ROOT/reports/artifact-hashes.txt" | tee -a "$RUN_REPORT"
"$ROOT/scripts/check_artifact_source_freshness.sh" | tee -a "$RUN_REPORT"
REASON=pre_run_current_bundle_freshness_check \
  "$ROOT/scripts/invalidate_stale_production_bundle_current.sh" | tee -a "$RUN_REPORT"

if [[ "$REFRESH_SOAK" == "true" ]]; then
  echo "running_soak_gate=true" | tee -a "$RUN_REPORT"
  set +e
  {
    MC_EULA_AGREE=true "$SOAK_COMMAND"
  } 2>&1 | tee -a "$RUN_REPORT" "$SOAK_REFRESH_OUTPUT"
  SOAK_STATUS=${PIPESTATUS[0]}
  set -e
  echo "readiness_soak_gate_exit=$SOAK_STATUS" | tee -a "$RUN_REPORT"
  if (( SOAK_STATUS != 0 )); then
    if [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
      if triggering_evidence="$(should_run_degraded_p500_diagnostic "$SOAK_REFRESH_OUTPUT")"; then
        run_degraded_p500_diagnostic "$SOAK_STATUS" "$triggering_evidence"
        echo "readiness_exit_reason=soak_gate_failed_after_degraded_diagnostic" | tee -a "$RUN_REPORT"
      else
        echo "readiness_degraded_p500_diagnostic=false" | tee -a "$RUN_REPORT"
        echo "readiness_degraded_p500_diagnostic_reason=no_host_contention_prelaunch_evidence" | tee -a "$RUN_REPORT"
        echo "readiness_p500_contended_diagnostic_launched=false" | tee -a "$RUN_REPORT"
        echo "readiness_exit_reason=soak_gate_failed_without_degraded_diagnostic" | tee -a "$RUN_REPORT"
      fi
    fi
    exit "$SOAK_STATUS"
  fi
else
  echo "running_soak_gate=false" | tee -a "$RUN_REPORT"
fi

if [[ "$REFRESH_REPEAT" == "true" ]]; then
  echo "running_repeat_gate=true" | tee -a "$RUN_REPORT"
  REPEAT_REFRESH_OUTPUT="$ROOT/reports/production-500-repeat-refresh-${STAMP}.txt"
  MC_EULA_AGREE=true \
    PRODUCTION_RELEASE_REPEAT_COUNT="${PRODUCTION_RELEASE_REPEAT_COUNT:-3}" \
    "$ROOT/scripts/run_production_release_repeat_gate.sh" | tee -a "$RUN_REPORT" "$REPEAT_REFRESH_OUTPUT"
  REPEAT_OUT_DIR="$(awk -F= '$1 == "production_release_repeat_out_dir" { print $2 }' "$REPEAT_REFRESH_OUTPUT" | tail -n 1)"
  if [[ -z "$REPEAT_OUT_DIR" || ! -d "$REPEAT_OUT_DIR" ]]; then
    echo "repeat_refresh_failure=missing_production_release_repeat_out_dir" | tee -a "$RUN_REPORT"
    exit 1
  fi
  python3 "$ROOT/scripts/evaluate_production_release_repeat.py" \
    --repeat-dir "$REPEAT_OUT_DIR" \
    --min-passes 3 \
    --report "$ROOT/reports/production-500-repeat-quorum.txt" | tee -a "$RUN_REPORT"
else
  echo "running_repeat_gate=false" | tee -a "$RUN_REPORT"
fi

if [[ "$REFRESH_COMPAT" == "true" ]]; then
  echo "running_plugin_matrix=true" | tee -a "$RUN_REPORT"
  MC_EULA_AGREE=true "$ROOT/scripts/run_plugin_matrix.sh" | tee -a "$RUN_REPORT"
  echo "running_restart_recovery=true" | tee -a "$RUN_REPORT"
  MC_EULA_AGREE=true "$ROOT/scripts/restart_recovery_check.sh" | tee -a "$RUN_REPORT"
  echo "running_forced_ticket_persistence=true" | tee -a "$RUN_REPORT"
  MC_EULA_AGREE=true "$ROOT/scripts/forced_ticket_persistence_check.sh" | tee -a "$RUN_REPORT"
else
  echo "running_plugin_matrix=false" | tee -a "$RUN_REPORT"
  echo "running_restart_recovery=false" | tee -a "$RUN_REPORT"
  echo "running_forced_ticket_persistence=false" | tee -a "$RUN_REPORT"
fi

python3 -m py_compile \
  "$ROOT/scripts/evaluate_production_readiness.py" \
  "$ROOT/scripts/export_production_readiness_bundle.py" \
  "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$ROOT/scripts/assert_production_ready_claim.py" \
  "$ROOT/scripts/publish_production_ready_claim.py" \
  "$ROOT/scripts/evaluate_production_soak.py" \
  "$ROOT/scripts/evaluate_production_release.py" \
  "$ROOT/scripts/evaluate_production_release_repeat.py" \
  "$ROOT/scripts/watch_load_host_contention.py" \
  "$ROOT/scripts/probe_host_synthetic_contention.py" | tee -a "$RUN_REPORT"

"$ROOT/scripts/evaluate_production_readiness_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/evaluate_production_release_current_binding_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/check_runtime_log_clean_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/run_load_test_production_action_gate_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/run_load_test_strict_foreign_process_gate_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/run_load_test_host_ready_stable_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/run_load_test_host_contention_watcher_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/run_load_test_host_synthetic_canary_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/run_load_test_sharding_defaults_smoke.sh" | tee -a "$RUN_REPORT"

python3 "$ROOT/scripts/evaluate_production_readiness.py" \
  --report "$ROOT/reports/production-500-readiness-gate.txt" | tee -a "$RUN_REPORT"

if [[ "$READINESS_CLAIM_ELIGIBLE" != "true" ]]; then
  echo "readiness_publication_guard=blocked" | tee -a "$RUN_REPORT"
  echo "readiness_exit_reason=non_claim_evidence_not_publishable mode=$READINESS_EVIDENCE_MODE" | tee -a "$RUN_REPORT"
  exit 75
fi

"$ROOT/scripts/export_production_readiness_bundle_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/validate_evidence_bundle_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/validate_production_readiness_bundle_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/assert_production_ready_claim_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/production_ready_claim_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/production_ready_claim_freshness_smoke.sh" | tee -a "$RUN_REPORT"
"$ROOT/scripts/publish_production_ready_claim_smoke.sh" | tee -a "$RUN_REPORT"

BUNDLE_DIR="$ROOT/reports/production-500-readiness-bundle-${STAMP}"
CLAIM_VERDICT="$ROOT/reports/production-500-claim-verdict-${STAMP}.txt"
CURRENT_BUNDLE="$ROOT/reports/production-500-readiness-bundle-current"
PUBLISH_STAGE="$ROOT/reports/.production-500-claim-current-${STAMP}.stage"
CURRENT_LINK_TMP="$ROOT/reports/.production-500-readiness-bundle-current.${STAMP}.tmp"
CURRENT_BACKUP=""
cleanup_publish_stage() {
  rm -rf "$PUBLISH_STAGE" "$CURRENT_LINK_TMP"
  if [[ -n "$CURRENT_BACKUP" && -e "$CURRENT_BACKUP" ]]; then
    if [[ ! -e "$CURRENT_BUNDLE" ]]; then
      mv -T "$CURRENT_BACKUP" "$CURRENT_BUNDLE"
    else
      rm -rf "$CURRENT_BACKUP"
    fi
  fi
}
trap cleanup_publish_stage EXIT
mkdir -p "$PUBLISH_STAGE"
python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$ROOT/reports/production-500-readiness-gate.txt" \
  --out-dir "$BUNDLE_DIR" | tee -a "$RUN_REPORT"

python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$BUNDLE_DIR" \
  --require-current-freshness | tee -a "$RUN_REPORT"

python3 "$ROOT/scripts/assert_production_ready_claim.py" \
  "$BUNDLE_DIR" \
  --report "$CLAIM_VERDICT" | tee -a "$RUN_REPORT"

PRODUCTION_READY_CLAIM_REPORTS_DIR="$ROOT/reports" \
PRODUCTION_READY_CLAIM_REPORT="$PUBLISH_STAGE/production-500-claim-verdict.txt" \
  "$ROOT/scripts/production_ready_claim.sh" "$BUNDLE_DIR" | tee -a "$RUN_REPORT"

python3 "$ROOT/scripts/publish_production_ready_claim.py" \
  "$BUNDLE_DIR" \
  --reports-dir "$ROOT/reports" \
  --out-prefix "$PUBLISH_STAGE/production-500-claim-current" \
  --verdict-report "$PUBLISH_STAGE/production-500-claim-current-verdict.txt" | tee -a "$RUN_REPORT"

ln -sfn "$BUNDLE_DIR" "$CURRENT_LINK_TMP"
if [[ -e "$CURRENT_BUNDLE" || -L "$CURRENT_BUNDLE" ]]; then
  CURRENT_BACKUP="$ROOT/reports/.production-500-readiness-bundle-current.previous-${STAMP}"
  rm -rf "$CURRENT_BACKUP"
  mv -T "$CURRENT_BUNDLE" "$CURRENT_BACKUP"
fi
mv -Tf "$CURRENT_LINK_TMP" "$CURRENT_BUNDLE"
rm -rf "$CURRENT_BACKUP"
CURRENT_BACKUP=""

mv -f "$PUBLISH_STAGE/production-500-claim-current.txt" \
  "$ROOT/reports/production-500-claim-current.txt"
mv -f "$PUBLISH_STAGE/production-500-claim-current.md" \
  "$ROOT/reports/production-500-claim-current.md"
mv -f "$PUBLISH_STAGE/production-500-claim-current.json" \
  "$ROOT/reports/production-500-claim-current.json"
mv -f "$PUBLISH_STAGE/production-500-claim-current-verdict.txt" \
  "$ROOT/reports/production-500-claim-current-verdict.txt"
cp "$CLAIM_VERDICT" "$ROOT/reports/production-500-claim-verdict.txt"
rm -rf "$PUBLISH_STAGE"

{
  echo "readiness_run_report=$RUN_REPORT"
  echo "readiness_gate_report=$ROOT/reports/production-500-readiness-gate.txt"
  echo "readiness_claim_bundle=$BUNDLE_DIR"
  echo "readiness_claim_bundle_current=$CURRENT_BUNDLE"
  echo "readiness_claim_verdict=$CLAIM_VERDICT"
  echo "readiness_claim_publication=$ROOT/reports/production-500-claim-current"
} | tee -a "$RUN_REPORT"
