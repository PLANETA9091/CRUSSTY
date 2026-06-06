#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT="${REPORT:-/dev/stdout}"

mkdir -p "$(dirname "$REPORT")" 2>/dev/null || true

log_line() {
  echo "$1"
  case "$REPORT" in
    /dev/stdout|/proc/self/fd/1) ;;
    *) echo "$1" >> "$REPORT" ;;
  esac
}

mapfile -t pgids < <(
  ps -eo pgid=,args= \
    | awk '/\/root\/\.cache\/puppeteer\/chrome\// && /--user-data-dir=\/tmp\/puppeteer_dev_chrome_profile-/ {print $1}' \
    | sort -nu
)

if [[ "${#pgids[@]}" -eq 0 ]]; then
  log_line "cleanup_viper_puppeteer_chrome=none"
  exit 0
fi

log_line "cleanup_viper_puppeteer_chrome_root=$ROOT"
log_line "cleanup_viper_puppeteer_chrome_groups=${#pgids[@]}"

for pgid in "${pgids[@]}"; do
  log_line "cleanup_viper_puppeteer_chrome_terminate_pgid=$pgid"
  kill -TERM -- "-$pgid" 2>/dev/null || true
done

sleep 2

for pgid in "${pgids[@]}"; do
  if ps -eo pgid= | awk -v pgid="$pgid" '$1 == pgid { found=1 } END { exit(found ? 0 : 1) }'; then
    log_line "cleanup_viper_puppeteer_chrome_kill_pgid=$pgid"
    kill -KILL -- "-$pgid" 2>/dev/null || true
  fi
done

remaining="$(ps -eo pgid=,args= | awk '/\/root\/\.cache\/puppeteer\/chrome\// && /--user-data-dir=\/tmp\/puppeteer_dev_chrome_profile-/ {print $1}' | sort -nu | wc -l | tr -d ' ')"
log_line "cleanup_viper_puppeteer_chrome_remaining_groups=$remaining"
