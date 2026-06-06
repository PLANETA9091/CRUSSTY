#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

REPORT="$TMP_DIR/release-gate.txt"
read_artifact_values() {
  python3 - "$ROOT/reports/artifacts.json" <<'PY'
import json
import sys
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(data["optimized"]["sha256"])
print(data["optimized_runtime"]["run_sh"]["sha256"])
print(data["optimized_runtime"]["runtime_jar_sha256_file"]["runtime_jar_sha256"])
print(data["optimized_runtime"]["native_library"]["sha256"])
PY
}
readarray -t ARTIFACT_VALUES < <(read_artifact_values)
OPT_SHA="${ARTIFACT_VALUES[0]}"
RUN_SH_SHA="${ARTIFACT_VALUES[1]}"
RUNTIME_JAR_SHA="${ARTIFACT_VALUES[2]}"
NATIVE_LIB_SHA="${ARTIFACT_VALUES[3]}"

append_current_artifacts() {
  local src="$1"
  local dst="$2"
  awk '
    BEGIN { inserted = 0 }
    /^bot_log_tail:$/ && !inserted {
      print "optimized_artifact_sha256='"$OPT_SHA"'"
      print "optimized_runtime_run_sh_sha256='"$RUN_SH_SHA"'"
      print "optimized_runtime_jar_sha256='"$RUNTIME_JAR_SHA"'"
      print "optimized_runtime_native_library_sha256='"$NATIVE_LIB_SHA"'"
      print "host_cpu_windows=1"
      print "host_system_load1_per_cpu_max=0.01"
      print "host_cpu_steal_percent_max=0.00"
      print "host_cpu_iowait_percent_max=0.00"
      print "bot_action_start_mode=all-ready"
      print "bot_action_gate_open_mode=all-ready"
      print "bot_action_ready_settle_ms=15000"
      print "bot_action_ready_requires_block_armed=true"
      print "bot_action_gate_opened=true"
      print "bot_action_ready_min_count=500"
      print "bot_action_ready_min_fraction=1.00"
      print "bot_action_gate_open_ready=500"
      print "bot_action_gate_open_active=500"
      print "bot_action_gate_open_settled=500"
      print "bot_action_gate_open_required=500"
      print "bot_action_gate_open_block_armed=500"
      print "compat_probe_block_evidence_accepted=true"
      print "compat_probe_direct_block_loadbot_event_lines=1"
      print "compat_probe_direct_block_loadbot_place_event_lines=1"
      print "compat_probe_direct_block_loadbot_break_event_lines=1"
      print "compat_probe_direct_block_loadbot_cancelled_false_lines=1"
      print "compat_probe_direct_block_loadbot_players=1"
      inserted = 1
    }
    { print }
  ' "$src" > "$dst"
}

positive_cold="$TMP_DIR/positive-cold-summary.txt"
positive_warm="$TMP_DIR/positive-warm-summary.txt"
append_current_artifacts \
  "$ROOT/reports/load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-summary.txt" \
  "$positive_cold"
append_current_artifacts \
  "$ROOT/reports/load-production-500-warm-block-500bots-post0097-20260516-194812-summary.txt" \
  "$positive_warm"

python3 "$ROOT/scripts/evaluate_production_release.py" \
  --cold-summary "$positive_cold" \
  --warm-summary "$positive_warm" \
  --artifact-hashes "$ROOT/reports/artifact-hashes.txt" \
  --artifacts-json "$ROOT/reports/artifacts.json" \
  --require-current-artifacts \
  --report "$REPORT" > "$TMP_DIR/pass.out"
grep -q '^production_ready_claim_eligible=true$' "$REPORT"
grep -q '^release_gate_pass=true$' "$REPORT"
grep -q '^cold_gate_pass=true$' "$REPORT"
grep -q '^warm_gate_pass=true$' "$REPORT"
grep -q '^artifact_hashes_pass=true$' "$REPORT"
grep -q '^requires_current_artifacts=true$' "$REPORT"

bad_cold="$TMP_DIR/bad-cold-summary.txt"
sed 's/load_window_tps1_min=18.07/load_window_tps1_min=17.99/' \
  "$positive_cold" \
  > "$bad_cold"
if python3 "$ROOT/scripts/evaluate_production_release.py" \
  --cold-summary "$bad_cold" \
  --warm-summary "$positive_warm" \
  --artifact-hashes "$ROOT/reports/artifact-hashes.txt" \
  --artifacts-json "$ROOT/reports/artifacts.json" \
  --require-current-artifacts \
  --report "$TMP_DIR/fail-gate.txt" > "$TMP_DIR/fail.out" 2>&1; then
  echo "Expected mutated cold summary to fail the release gate." >&2
  cat "$TMP_DIR/fail.out" >&2
  exit 1
fi
grep -q '^production_ready_claim_eligible=false$' "$TMP_DIR/fail-gate.txt"
grep -q 'cold production-500 gate failed' "$TMP_DIR/fail-gate.txt"

if python3 "$ROOT/scripts/evaluate_production_release.py" \
  --cold-summary "$positive_cold" \
  --warm-summary "$positive_warm" \
  --artifact-hashes "$ROOT/reports/artifact-hashes.txt" \
  --artifacts-json "$ROOT/reports/artifacts.json" \
  --allow-stale-artifacts \
  --report "$TMP_DIR/stale-mode.txt" > "$TMP_DIR/stale-mode.out" 2>&1; then
  echo "Expected stale-artifacts mode to require an explicit debug env guard." >&2
  exit 1
fi
grep -q 'Refusing stale-artifacts mode without PRODUCTION_RELEASE_ALLOW_STALE_ARTIFACTS=true' "$TMP_DIR/stale-mode.out"

echo "evaluate_production_release_smoke=PASS"
