#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT="${REPORT:-$ROOT/reports/production-500-go-nogo-current.txt}"
FOREIGN_PATTERN="${LOAD_TEST_STRICT_FOREIGN_PROCESS_PATTERN:-java --add-modules|server\\.jar|mc_bot|probe\\.js}"
DIAGNOSTIC_MODE="${PRODUCTION_READINESS_DIAGNOSTIC_MODE:-false}"
CANARY_DURATION_SECONDS="${CANARY_DURATION_SECONDS:-15}"
CANARY_SAMPLE_INTERVAL_SECONDS="${CANARY_SAMPLE_INTERVAL_SECONDS:-1}"
CANARY_MAX_STEAL_PERCENT="${CANARY_MAX_STEAL_PERCENT:-10}"
CANARY_MAX_IOWAIT_PERCENT="${CANARY_MAX_IOWAIT_PERCENT:-10}"
CANARY_WORKERS="${CANARY_WORKERS:-}"

mkdir -p "$(dirname "$REPORT")"
TMP="$(mktemp -d)"
DIAGNOSTIC_FOREIGN_PRESENT=false
trap 'rm -rf "$TMP"' EXIT

case "$DIAGNOSTIC_MODE" in
  true|false) ;;
  *) echo "PRODUCTION_READINESS_DIAGNOSTIC_MODE must be true or false." >&2; exit 64 ;;
esac

sanitize_process_lines() {
  python3 -c '
from __future__ import annotations

import os
import sys

MAX_ARGS_PREFIX = 96
MODE = os.environ.get("GO_NOGO_SANITIZE_MODE", "top")

for raw in sys.stdin:
    line = raw.rstrip("\n")
    if not line:
        print()
        continue
    parts = line.split(None, 7)
    if len(parts) < 8:
        print(line)
        continue
    pid, ppid, user, stat, pcpu, pmem, comm, args = parts
    args = args.strip()
    if len(args) > MAX_ARGS_PREFIX:
        if MODE == "foreign":
            args = f"{args[:MAX_ARGS_PREFIX]} ...[redacted len={len(args)}]"
        else:
            args = f"[redacted len={len(args)}]"
    print(f"{pid} {ppid} {user} {stat} {pcpu} {pmem} {comm} {args}")
' 
}

write_header() {
  {
    printf 'production_500_go_nogo_profile=production-500\n'
    printf 'production_500_go_nogo_generated_at_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'production_500_go_nogo_diagnostic_mode=%s\n' "$DIAGNOSTIC_MODE"
    printf 'production_500_go_nogo_foreign_pattern=%s\n' "$FOREIGN_PATTERN"
    printf 'production_500_go_nogo_canary_duration_seconds=%s\n' "$CANARY_DURATION_SECONDS"
    printf 'production_500_go_nogo_canary_sample_interval_seconds=%s\n' "$CANARY_SAMPLE_INTERVAL_SECONDS"
    printf 'production_500_go_nogo_canary_max_steal_percent=%s\n' "$CANARY_MAX_STEAL_PERCENT"
    printf 'production_500_go_nogo_canary_max_iowait_percent=%s\n' "$CANARY_MAX_IOWAIT_PERCENT"
  } > "$REPORT"
}

write_top_processes() {
  {
    printf 'top_processes_begin\n'
    ps -eo pid=,ppid=,user=,stat=,pcpu=,pmem=,comm=,args= --sort=-pcpu \
      | rg -v 'rg |grep |ps -eo|check_production_500_go_nogo|python3 -c|bash -c set \+e; REPORT=' \
      | sed -n '1,20p' \
      | GO_NOGO_SANITIZE_MODE=top sanitize_process_lines
    printf 'top_processes_end\n'
  } >> "$REPORT"
}

write_header
write_top_processes

ps -eo pid=,ppid=,user=,stat=,pcpu=,pmem=,args= \
  | rg "$FOREIGN_PATTERN" \
  | rg -v 'rg |grep |ps -eo|check_production_500_go_nogo|python3 -c|bash -c set \+e; REPORT=' \
  > "$TMP/foreign.txt" || true

if [[ -s "$TMP/foreign.txt" ]]; then
  if [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
    DIAGNOSTIC_FOREIGN_PRESENT=true
    {
      printf 'production_500_go_nogo_foreign_processes_present=true\n'
      printf 'foreign_processes_begin\n'
      GO_NOGO_SANITIZE_MODE=foreign sanitize_process_lines < "$TMP/foreign.txt"
      printf 'foreign_processes_end\n'
    } >> "$REPORT"
  else
    {
      printf 'production_500_go_nogo_pass=false\n'
      printf 'production_500_go_nogo_exit_code=75\n'
      printf 'production_500_go_nogo_reason=strict_foreign_process_present\n'
      printf 'foreign_processes_begin\n'
      GO_NOGO_SANITIZE_MODE=foreign sanitize_process_lines < "$TMP/foreign.txt"
      printf 'foreign_processes_end\n'
    } >> "$REPORT"
    cat "$REPORT"
    exit 75
  fi
fi

canary_args=(
  --duration-seconds "$CANARY_DURATION_SECONDS"
  --sample-interval-seconds "$CANARY_SAMPLE_INTERVAL_SECONDS"
  --max-steal-percent "$CANARY_MAX_STEAL_PERCENT"
  --max-iowait-percent "$CANARY_MAX_IOWAIT_PERCENT"
  --report "$TMP/canary.txt"
)
if [[ -n "$CANARY_WORKERS" ]]; then
  canary_args+=(--workers "$CANARY_WORKERS")
fi

set +e
python3 "$ROOT/scripts/probe_host_synthetic_contention.py" "${canary_args[@]}" > "$TMP/canary.stdout"
canary_status=$?
set -e
cat "$TMP/canary.txt" >> "$REPORT"

if [[ "$canary_status" -ne 0 ]]; then
  if [[ "$DIAGNOSTIC_MODE" == "true" ]]; then
    {
      printf 'production_500_go_nogo_pass=false\n'
      printf 'production_500_go_nogo_exit_code=0\n'
      printf 'production_500_go_nogo_reason=diagnostic_host_synthetic_canary_failed\n'
    } >> "$REPORT"
    cat "$REPORT"
    exit 0
  else
    {
      printf 'production_500_go_nogo_pass=false\n'
      printf 'production_500_go_nogo_exit_code=%s\n' "$canary_status"
      printf 'production_500_go_nogo_reason=host_synthetic_canary_failed\n'
    } >> "$REPORT"
    cat "$REPORT"
    exit "$canary_status"
  fi
fi

{
  if [[ "$DIAGNOSTIC_FOREIGN_PRESENT" == "true" ]]; then
    printf 'production_500_go_nogo_pass=false\n'
    printf 'production_500_go_nogo_exit_code=0\n'
    printf 'production_500_go_nogo_reason=diagnostic_foreign_process_present\n'
  else
    printf 'production_500_go_nogo_pass=true\n'
    printf 'production_500_go_nogo_exit_code=0\n'
    printf 'production_500_go_nogo_reason=none\n'
  fi
} >> "$REPORT"
cat "$REPORT"
