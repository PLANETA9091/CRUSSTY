#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/repeat/run-1"
cat > "$TMP/repeat/run-1/production-500-release-gate.txt" <<'REPORT'
claim_text=500-bots-production-ready-for-measured-32-32-creative-block-profile
claim_scope=cold-fresh-and-warm-source-500-bots-32-view-32-simulation-creative-block-workload
production_ready_claim_eligible=true
release_gate_pass=true
failure_count=0
artifact_hashes_pass=true
cold_gate_pass=true
cold_failure_count=0
cold_bots=500
cold_view_distance=32
cold_simulation_distance=32
cold_load_test_scenario=block
cold_load_test_gamemode=creative
cold_load_window_reached_full_online=true
cold_load_window_online_max=500
cold_watchdog_thread_dumps=0
cold_sync_load_stack_hits=0
cold_stability_failures=0
warm_gate_pass=true
warm_failure_count=0
warm_bots=500
warm_view_distance=32
warm_simulation_distance=32
warm_load_test_scenario=block
warm_load_test_gamemode=creative
warm_load_window_reached_full_online=true
warm_load_window_online_max=500
warm_watchdog_thread_dumps=0
warm_sync_load_stack_hits=0
warm_stability_failures=0
REPORT
touch "$TMP/repeat/run-1/cold-summary.txt" "$TMP/repeat/run-1/warm-summary.txt"

python3 "$ROOT/scripts/evaluate_production_release_repeat.py" \
  --repeat-dir "$TMP/repeat" \
  --min-passes 1 \
  --report "$TMP/quorum-pass.txt" >/dev/null

sed -i 's/release_gate_pass=true/release_gate_pass=false/' \
  "$TMP/repeat/run-1/production-500-release-gate.txt"

if python3 "$ROOT/scripts/evaluate_production_release_repeat.py" \
  --repeat-dir "$TMP/repeat" \
  --min-passes 1 \
  --report "$TMP/quorum-fail.txt" >/dev/null 2>&1; then
  echo "expected repeat quorum smoke failure did not happen" >&2
  exit 1
fi

echo "evaluate_production_release_repeat_smoke=PASS"
