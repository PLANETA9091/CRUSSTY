#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/clean.log" <<'LOG'
[00:00:00 INFO]: Starting minecraft server version 1.21.10
[00:00:01 INFO]: Done (1.234s)! For help, type "help"
LOG
"$ROOT/scripts/check_runtime_log_clean.sh" "$TMP/clean.log" clean > "$TMP/clean.out"
rg -q '^runtime_log_clean=PASS label=clean ' "$TMP/clean.out"

cat > "$TMP/pre-done-exception.log" <<'LOG'
[00:00:00 INFO]: Starting minecraft server version 1.21.10
Exception in thread "Server thread" java.lang.RuntimeException: synthetic failure
[00:00:01 INFO]: Done (1.234s)! For help, type "help"
LOG
if "$ROOT/scripts/check_runtime_log_clean.sh" "$TMP/pre-done-exception.log" pre-done > "$TMP/pre-done.out" 2>&1; then
  echo "Expected pre-Done exception log to fail runtime log validation." >&2
  exit 1
fi
rg -q '^runtime_log_clean=FAIL label=pre-done reason=watchdog_or_unexpected_exception ' "$TMP/pre-done.out"

echo "check_runtime_log_clean_smoke=PASS"
