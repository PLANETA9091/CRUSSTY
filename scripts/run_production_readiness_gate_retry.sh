#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

MAX_RETRIES="${PRODUCTION_READINESS_GATE_RETRY_COUNT:-3}"
RETRY_DELAY_SECONDS="${PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS:-60}"
INNER_GATE="${PRODUCTION_READINESS_GATE_INNER:-$ROOT/scripts/run_production_readiness_gate.sh}"
REPORT_ROOT="${PRODUCTION_READINESS_GATE_REPORT_ROOT:-$ROOT}"
SOAK_REPORT="$REPORT_ROOT/reports/production-500-soak-gate.txt"
RELEASE_REPORT="$REPORT_ROOT/reports/production-500-release-gate.txt"
GO_NOGO_REPORT="$REPORT_ROOT/reports/production-500-go-nogo-current.txt"
PREFLIGHT_ENABLED="${PRODUCTION_READINESS_PREFLIGHT:-true}"
DIAGNOSTIC_MODE="${PRODUCTION_READINESS_DIAGNOSTIC_MODE:-false}"
PREFLIGHT_COMMAND="${PRODUCTION_READINESS_PREFLIGHT_COMMAND:-$ROOT/scripts/check_production_500_go_nogo.sh}"
REFRESH_SOAK="${PRODUCTION_READINESS_REFRESH_SOAK:-true}"
REFRESH_REPEAT="${PRODUCTION_READINESS_REFRESH_REPEAT:-true}"
REFRESH_COMPAT="${PRODUCTION_READINESS_REFRESH_COMPAT:-true}"
RELEASE_REPEAT_COUNT="${PRODUCTION_RELEASE_REPEAT_COUNT:-3}"
PREP_CLEANUP_COMMAND="${PRODUCTION_READINESS_RETRY_CLEANUP_COMMAND:-}"
STAMP="${PRODUCTION_READINESS_GATE_RETRY_STAMP:-$(date +%Y%m%d-%H%M%S)}"
RETRY_REPORT="$REPORT_ROOT/reports/production-500-readiness-gate-retry-${STAMP}.txt"

if ! [[ "$MAX_RETRIES" =~ ^[0-9]+$ ]]; then
  echo "PRODUCTION_READINESS_GATE_RETRY_COUNT must be a non-negative integer." >&2
  exit 64
fi
if ! [[ "$RETRY_DELAY_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "PRODUCTION_READINESS_GATE_RETRY_DELAY_SECONDS must be a non-negative integer." >&2
  exit 64
fi
case "$REFRESH_SOAK" in true|false) ;; *) echo "PRODUCTION_READINESS_REFRESH_SOAK must be true or false." >&2; exit 64 ;; esac
case "$REFRESH_REPEAT" in true|false) ;; *) echo "PRODUCTION_READINESS_REFRESH_REPEAT must be true or false." >&2; exit 64 ;; esac
case "$REFRESH_COMPAT" in true|false) ;; *) echo "PRODUCTION_READINESS_REFRESH_COMPAT must be true or false." >&2; exit 64 ;; esac
case "$PREFLIGHT_ENABLED" in true|false) ;; *) echo "PRODUCTION_READINESS_PREFLIGHT must be true or false." >&2; exit 64 ;; esac
case "$DIAGNOSTIC_MODE" in true|false) ;; *) echo "PRODUCTION_READINESS_DIAGNOSTIC_MODE must be true or false." >&2; exit 64 ;; esac
if ! [[ "$RELEASE_REPEAT_COUNT" =~ ^[0-9]+$ ]] || (( RELEASE_REPEAT_COUNT < 1 )); then
  echo "PRODUCTION_RELEASE_REPEAT_COUNT must be a positive integer." >&2
  exit 64
fi
export LOAD_TEST_DIAGNOSTIC_MODE="$DIAGNOSTIC_MODE"

mkdir -p "$ROOT/reports" "$REPORT_ROOT/reports"

read_report_value() {
  local key="$1"
  local path="$2"
  awk -F= -v key="$key" '$1 == key { sub(/^[^=]*=/, "", $0); print; exit }' "$path" 2>/dev/null || true
}

run_preflight() {
  local started_epoch tmp_report report_mtime
  started_epoch="$(date +%s)"
  tmp_report="${GO_NOGO_REPORT}.${STAMP}.$$.tmp"
  rm -f "$tmp_report"
  set +e
  {
    REPORT="$tmp_report" \
      PRODUCTION_READINESS_DIAGNOSTIC_MODE="$DIAGNOSTIC_MODE" \
      "$PREFLIGHT_COMMAND"
  } 2>&1 | tee -a "$RETRY_REPORT"
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
  mv -f "$tmp_report" "$GO_NOGO_REPORT"
  PREFLIGHT_PASS="$(read_report_value production_500_go_nogo_pass "$GO_NOGO_REPORT")"
  PREFLIGHT_REASON="$(read_report_value production_500_go_nogo_reason "$GO_NOGO_REPORT")"
  PREFLIGHT_EXIT_CODE="$(read_report_value production_500_go_nogo_exit_code "$GO_NOGO_REPORT")"
}

run_preflight_cleanup() {
  if [[ -z "$PREP_CLEANUP_COMMAND" ]]; then
    return 0
  fi
  set +e
  {
    REPORT="$RETRY_REPORT" "$PREP_CLEANUP_COMMAND"
  } 2>&1 | tee -a "$RETRY_REPORT"
  cleanup_status=${PIPESTATUS[0]}
  set -e
  echo "production_readiness_retry_cleanup_status=$cleanup_status" | tee -a "$RETRY_REPORT"
  return 0
}

DEGRADED_PREFLIGHT_REASON=""
DEGRADED_PREFLIGHT_REPORT=""

is_retryable_host_contention() {
  python3 - "$1" "$SOAK_REPORT" "$RELEASE_REPORT" <<'PY'
from __future__ import annotations

from pathlib import Path
import sys

min_mtime = float(sys.argv[1])
for raw_path in sys.argv[2:]:
    path = Path(raw_path)
    if not path.exists():
        continue
    try:
        if path.stat().st_mtime < min_mtime:
            continue
    except OSError:
        continue
    text = path.read_text(encoding="utf-8", errors="replace")
    if "host_contention" in text and (
        "environment_invalid=true" in text or "run_class=environment-invalid" in text
    ):
        print(raw_path)
        raise SystemExit(0)

raise SystemExit(1)
PY
}

{
  echo "production_readiness_retry_stamp=$STAMP"
  echo "production_readiness_retry_count=$MAX_RETRIES"
  echo "production_readiness_retry_delay_seconds=$RETRY_DELAY_SECONDS"
  echo "production_readiness_retry_inner=$INNER_GATE"
  echo "production_readiness_retry_refresh_soak=$REFRESH_SOAK"
  echo "production_readiness_retry_refresh_repeat=$REFRESH_REPEAT"
  echo "production_readiness_retry_refresh_compat=$REFRESH_COMPAT"
  echo "production_readiness_retry_repeat_count=$RELEASE_REPEAT_COUNT"
  echo "production_readiness_retry_cleanup_command=$PREP_CLEANUP_COMMAND"
  echo "production_readiness_preflight_enabled=$PREFLIGHT_ENABLED"
  echo "production_readiness_diagnostic_mode=$DIAGNOSTIC_MODE"
  if [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
    echo "production_readiness_evidence_mode=diagnostic_non_claim"
    echo "production_readiness_claim_eligible=false"
  else
    echo "production_readiness_evidence_mode=strict_claim_candidate"
    echo "production_readiness_claim_eligible=true"
  fi
  echo "production_readiness_preflight_command=$PREFLIGHT_COMMAND"
  echo "production_readiness_preflight_report=$GO_NOGO_REPORT"
  echo "production_readiness_retry_report=$RETRY_REPORT"
} | tee "$RETRY_REPORT"

attempt=1
while :; do
  echo "production_readiness_retry_attempt=$attempt" | tee -a "$RETRY_REPORT"
  attempt_started_epoch="$(date +%s)"
  run_preflight_cleanup

  if [[ "$PREFLIGHT_ENABLED" == "true" ]]; then
    run_preflight
    PREFLIGHT_REASON="${PREFLIGHT_REASON:-unknown}"
    PREFLIGHT_PASS="${PREFLIGHT_PASS:-false}"
    PREFLIGHT_EXIT_CODE="${PREFLIGHT_EXIT_CODE:-$PREFLIGHT_STATUS}"
    if [[ "$PREFLIGHT_STATUS" -eq 0 && "$PREFLIGHT_PASS" == "true" ]]; then
      echo "production_readiness_preflight_pass=true attempt=$attempt reason=none exit_code=0 report=$GO_NOGO_REPORT" | tee -a "$RETRY_REPORT"
      echo "production_readiness_heavy_gate_allowed=true" | tee -a "$RETRY_REPORT"
      DEGRADED_PREFLIGHT_REASON=""
      DEGRADED_PREFLIGHT_REPORT=""
    elif [[ "$DIAGNOSTIC_MODE" == "true" && "$PREFLIGHT_STATUS" -eq 0 && "$PREFLIGHT_REASON" == diagnostic_* ]]; then
      echo "production_readiness_preflight_pass=false attempt=$attempt reason=$PREFLIGHT_REASON exit_code=0 report=$GO_NOGO_REPORT" | tee -a "$RETRY_REPORT"
      echo "production_readiness_heavy_gate_allowed=false" | tee -a "$RETRY_REPORT"
      echo "production_readiness_next_action=run_degraded_p500_diagnostic_without_soak" | tee -a "$RETRY_REPORT"
      DEGRADED_PREFLIGHT_REASON="$PREFLIGHT_REASON"
      DEGRADED_PREFLIGHT_REPORT="$GO_NOGO_REPORT"
    elif [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
      echo "production_readiness_preflight_pass=false attempt=$attempt reason=$PREFLIGHT_REASON exit_code=$PREFLIGHT_EXIT_CODE report=$GO_NOGO_REPORT" | tee -a "$RETRY_REPORT"
      echo "production_readiness_heavy_gate_allowed=false" | tee -a "$RETRY_REPORT"
      echo "production_readiness_next_action=run_degraded_p500_diagnostic_without_soak" | tee -a "$RETRY_REPORT"
      DEGRADED_PREFLIGHT_REASON="$PREFLIGHT_REASON"
      DEGRADED_PREFLIGHT_REPORT="$GO_NOGO_REPORT"
    else
      echo "production_readiness_preflight_pass=false attempt=$attempt reason=$PREFLIGHT_REASON exit_code=$PREFLIGHT_EXIT_CODE report=$GO_NOGO_REPORT" | tee -a "$RETRY_REPORT"
      echo "production_readiness_heavy_gate_allowed=false" | tee -a "$RETRY_REPORT"
      case "$PREFLIGHT_REASON" in
        strict_foreign_process_present)
          echo "production_readiness_next_action=stop_foreign_process" | tee -a "$RETRY_REPORT"
          exit "${PREFLIGHT_EXIT_CODE:-75}"
          ;;
        host_synthetic_canary_failed)
          echo "production_readiness_next_action=wait_for_clean_host" | tee -a "$RETRY_REPORT"
          if (( attempt > MAX_RETRIES )); then
            echo "production_readiness_retry_exhausted=true attempts=$attempt last_status=${PREFLIGHT_EXIT_CODE:-1}" | tee -a "$RETRY_REPORT"
            exit "${PREFLIGHT_EXIT_CODE:-1}"
          fi
          echo "production_readiness_retry_retrying=true attempt=$attempt delay_seconds=$RETRY_DELAY_SECONDS reason=host_synthetic_canary_failed phase=preflight report=$GO_NOGO_REPORT" | tee -a "$RETRY_REPORT"
          if (( RETRY_DELAY_SECONDS > 0 )); then
            sleep "$RETRY_DELAY_SECONDS"
          fi
          attempt=$((attempt + 1))
          continue
          ;;
        *)
          echo "production_readiness_next_action=retry_preflight" | tee -a "$RETRY_REPORT"
          exit "${PREFLIGHT_EXIT_CODE:-${PREFLIGHT_STATUS:-1}}"
          ;;
        esac
    fi
  else
    echo "production_readiness_preflight_pass=skipped attempt=$attempt reason=disabled exit_code=0 report=$GO_NOGO_REPORT" | tee -a "$RETRY_REPORT"
    echo "production_readiness_heavy_gate_allowed=true" | tee -a "$RETRY_REPORT"
  fi

  if [[ -n "$DEGRADED_PREFLIGHT_REASON" ]]; then
    set +e
    {
      MC_EULA_AGREE=true \
        PRODUCTION_READINESS_PREFLIGHT=false \
        PRODUCTION_READINESS_DIAGNOSTIC_MODE="$DIAGNOSTIC_MODE" \
        PRODUCTION_READINESS_OUTER_PREFLIGHT_PASSED=true \
        PRODUCTION_READINESS_DEGRADED_PREFLIGHT_REASON="$DEGRADED_PREFLIGHT_REASON" \
        PRODUCTION_READINESS_DEGRADED_PREFLIGHT_REPORT="$DEGRADED_PREFLIGHT_REPORT" \
        PRODUCTION_READINESS_REFRESH_SOAK="$REFRESH_SOAK" \
        PRODUCTION_READINESS_REFRESH_REPEAT="$REFRESH_REPEAT" \
        PRODUCTION_READINESS_REFRESH_COMPAT="$REFRESH_COMPAT" \
        PRODUCTION_RELEASE_REPEAT_COUNT="$RELEASE_REPEAT_COUNT" \
        PRODUCTION_READINESS_GATE_REPORT_ROOT="$REPORT_ROOT" \
        "$INNER_GATE"
    } 2>&1 | tee -a "$RETRY_REPORT"
    status=${PIPESTATUS[0]}
    set -e

    if (( status == 0 )); then
      echo "production_readiness_retry_pass=false attempts=$attempt reason=diagnostic_non_claim_status_zero" | tee -a "$RETRY_REPORT"
      echo "production_readiness_retry_exit_reason=diagnostic_non_claim_not_publishable" | tee -a "$RETRY_REPORT"
      exit 75
    fi
    echo "production_readiness_retry_pass=false attempts=$attempt reason=diagnostic_non_claim_status_$status" | tee -a "$RETRY_REPORT"
    echo "production_readiness_retry_exit_reason=diagnostic_non_claim_not_publishable" | tee -a "$RETRY_REPORT"
    exit 75
  fi

  set +e
  {
    MC_EULA_AGREE=true \
      PRODUCTION_READINESS_PREFLIGHT=false \
      PRODUCTION_READINESS_DIAGNOSTIC_MODE="$DIAGNOSTIC_MODE" \
      PRODUCTION_READINESS_OUTER_PREFLIGHT_PASSED=true \
      PRODUCTION_READINESS_DEGRADED_PREFLIGHT_REASON="" \
      PRODUCTION_READINESS_DEGRADED_PREFLIGHT_REPORT="" \
      PRODUCTION_READINESS_REFRESH_SOAK="$REFRESH_SOAK" \
      PRODUCTION_READINESS_REFRESH_REPEAT="$REFRESH_REPEAT" \
      PRODUCTION_READINESS_REFRESH_COMPAT="$REFRESH_COMPAT" \
      PRODUCTION_RELEASE_REPEAT_COUNT="$RELEASE_REPEAT_COUNT" \
      PRODUCTION_READINESS_GATE_REPORT_ROOT="$REPORT_ROOT" \
      "$INNER_GATE"
  } 2>&1 | tee -a "$RETRY_REPORT"
  status=${PIPESTATUS[0]}
  set -e

  if (( status == 0 )); then
    if [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
      echo "production_readiness_retry_pass=false attempts=$attempt reason=diagnostic_non_claim_status_zero" | tee -a "$RETRY_REPORT"
      echo "production_readiness_retry_exit_reason=diagnostic_non_claim_not_publishable" | tee -a "$RETRY_REPORT"
      exit 75
    fi
    echo "production_readiness_retry_pass=true attempts=$attempt" | tee -a "$RETRY_REPORT"
    exit 0
  fi

  if (( attempt > MAX_RETRIES )); then
    echo "production_readiness_retry_exhausted=true attempts=$attempt last_status=$status" | tee -a "$RETRY_REPORT"
    exit "$status"
  fi

  if retryable_report="$(is_retryable_host_contention "$attempt_started_epoch")"; then
    echo "production_readiness_retry_retrying=true attempt=$attempt delay_seconds=$RETRY_DELAY_SECONDS reason=host_contention report=$retryable_report" | tee -a "$RETRY_REPORT"
    if (( RETRY_DELAY_SECONDS > 0 )); then
      sleep "$RETRY_DELAY_SECONDS"
    fi
    attempt=$((attempt + 1))
    continue
  fi

  echo "production_readiness_retry_retryable=false attempt=$attempt last_status=$status" | tee -a "$RETRY_REPORT"
  exit "$status"
done
