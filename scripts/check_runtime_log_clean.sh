#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <runtime-log> [label]" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/load_test_defaults.sh"
LOG="$1"
LABEL="${2:-$(basename "$LOG")}"
SUCCESS_PATTERN="${RUNTIME_LOG_SUCCESS_PATTERN:-$(load_test_server_ready_regex)}"

if [[ ! -f "$LOG" ]]; then
  echo "runtime_log_clean=FAIL label=$LABEL reason=missing_log log=$LOG" >&2
  exit 1
fi

success_line="$(rg -n -m1 "$SUCCESS_PATTERN" "$LOG" | cut -d: -f1 || true)"
if [[ -z "$success_line" ]]; then
  echo "runtime_log_clean=FAIL label=$LABEL reason=missing_success_marker log=$LOG" >&2
  exit 1
fi

MATCHES="$(mktemp)"
trap 'rm -f "$MATCHES"' EXIT

fail_with_matches() {
  local reason="$1"
  echo "runtime_log_clean=FAIL label=$LABEL reason=$reason log=$LOG" >&2
  sed -n '1,12p' "$MATCHES" >&2 || true
  exit 1
}

if rg -n -i \
  -e 'DO NOT REPORT THIS TO PAPER' \
  -e 'The server has (not responded|stopped responding)' \
  -e 'Server thread dump' \
  -e 'Entire Thread Dump' \
  -e 'Current Thread: Paper Watchdog Thread' \
  -e 'org\.spigotmc\.WatchdogThread\.run' \
  -e 'Encountered an unexpected exception' \
  -e 'Exception in thread' \
  -e 'Caused by: .*(Exception|Error)\b' \
  -e '^[0-9]+:.*\b(java|javax|jdk|sun|net\.minecraft|org\.bukkit|io\.papermc|org\.spigotmc)\..*(Exception|Error)\b' \
  "$LOG" > "$MATCHES"; then
  fail_with_matches "watchdog_or_unexpected_exception"
fi

echo "runtime_log_clean=PASS label=$LABEL log=$LOG"
