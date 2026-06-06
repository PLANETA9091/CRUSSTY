#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fake_launcher="$TMP/fake-plugin-done-launcher.sh"
cat > "$fake_launcher" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo '[00:00:00 INFO]: [Geyser-Spigot] Done (1.000s)! Run /geyser help for help!'
while IFS= read -r line; do
  if [[ "$line" == "stop" ]]; then
    echo '[00:00:01 INFO]: Stopping server'
    exit 0
  fi
done
SH
chmod +x "$fake_launcher"

label="server-ready-smoke-$$"
stdout="$TMP/stdout.txt"
stderr="$TMP/stderr.txt"
expected_java_opts_load='-Xms1G -Xmx2G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100'

set +e
(
  cd "$ROOT"
  env \
    MC_EULA_AGREE=true \
    LOAD_TEST_LABEL="$label" \
    LOAD_TEST_SCENARIO=movement \
    BOT_COUNT=1 \
    DURATION_SECONDS=1 \
    LOAD_TEST_ALLOW_BUSY_HOST=true \
    LOAD_TEST_ALLOW_FOREIGN_PROCESSES=true \
    LOAD_TEST_SERVER_READY_TIMEOUT_SECONDS=2 \
    LAUNCHER="$fake_launcher" \
    "$ROOT/scripts/run_load_test.sh"
) >"$stdout" 2>"$stderr"
status=$?
set -e

if [[ "$status" == "0" ]]; then
  echo "Expected fake plugin done launcher to fail readiness, got success." >&2
  exit 1
fi

harness_exit="$ROOT/reports/load-${label}-harness-exit.txt"
if [[ ! -f "$harness_exit" ]]; then
  echo "Expected harness exit report: $harness_exit" >&2
  exit 1
fi
if ! grep -Fqx "java_opts_load=$expected_java_opts_load" "$harness_exit"; then
  echo "Expected java_opts_load recorded in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^server_cmdline_effective=' "$harness_exit"; then
  echo "Expected effective server cmdline field in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^server_jvm_flags_effective=' "$harness_exit"; then
  echo "Expected effective JVM flags field in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^server_proc_cmdline=' "$harness_exit"; then
  echo "Expected server_proc_cmdline recorded in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^server_jcmd_vm_command_line_available=false$' "$harness_exit"; then
  echo "Expected server_jcmd_vm_command_line_available=false in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^bot_child_process_count=0$' "$harness_exit"; then
  echo "Expected bot_child_process_count=0 in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^bot_child_process_missing_count=0$' "$harness_exit"; then
  echo "Expected bot_child_process_missing_count=0 in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^bot_resource_samples=0$' "$harness_exit"; then
  echo "Expected zero bot resource samples before bot phase." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^bot_pss_mib_available=false$' "$harness_exit"; then
  echo "Expected bot PSS availability false before bot phase." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^phase=waiting-for-server-ready$' "$harness_exit"; then
  echo "Expected waiting-for-server-ready phase in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if grep -q '^rc=0$' "$harness_exit"; then
  echo "Harness exit report must not claim rc=0 before summary generation." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if ! grep -q '^server_ready_seen=false$' "$harness_exit"; then
  echo "Expected server_ready_seen=false in harness exit report." >&2
  cat "$harness_exit" >&2
  exit 1
fi
if [[ -e "$ROOT/reports/load-${label}-status.json" ]]; then
  echo "Status ping must not run after a plugin-specific done line." >&2
  exit 1
fi
if ! grep -q '^\[00:00:00 INFO\]: \[Geyser-Spigot\] Done (1.000s)! Run /geyser help for help!$' "$ROOT/logs/load-${label}.log"; then
  echo "Expected launcher log to contain the fake startup line." >&2
  exit 1
fi
if [[ -e "$ROOT/logs/load-${label}-bots.log" ]]; then
  echo "Bot phase must not start after a plugin-specific done line." >&2
  exit 1
fi

rm -rf "$ROOT/runs/load-${label}" \
  "$ROOT/logs/load-${label}.log" \
  "$ROOT/logs/load-${label}-bots.log" \
  "$ROOT/logs/load-${label}-bots" \
  "$ROOT/logs/load-${label}-jstacks" \
  "$ROOT/reports/load-${label}-preflight.txt" \
  "$ROOT/reports/load-${label}-harness-exit.txt" \
  "$ROOT/reports/load-${label}-summary.txt" \
  "$ROOT/reports/load-${label}-resources.csv" \
  "$ROOT/reports/load-${label}-status.json" \
  "$ROOT/logs/load-${label}.log"

printf 'java_opts_load=%s\n' "$expected_java_opts_load"
echo "run_load_test_server_ready_smoke=PASS"
