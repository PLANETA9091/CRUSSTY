#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

window_pass="$TMP_DIR/window-pass-summary.txt"
warm_pass="$TMP_DIR/warm-pass-summary.txt"
fallback_fail="$TMP_DIR/fallback-fail-summary.txt"
missing_block_probe="$TMP_DIR/missing-block-probe-summary.txt"
low_block_probe="$TMP_DIR/low-block-probe-summary.txt"
missing_window="$TMP_DIR/missing-window-summary.txt"
contention_invalid="$TMP_DIR/contention-invalid-summary.txt"
stress_pass="$TMP_DIR/stress-pass-summary.txt"
stress_missing_manifest="$TMP_DIR/stress-missing-manifest-summary.txt"
stress_bad_manifest_sha="$TMP_DIR/stress-bad-manifest-sha-summary.txt"
stress_count_mismatch="$TMP_DIR/stress-count-mismatch-summary.txt"
stress_file_drift="$TMP_DIR/stress-file-drift-summary.txt"

create_stress_manifest_fixture() {
  local run_dir="$1"
  python3 - "$run_dir" <<'PY'
from __future__ import annotations

import hashlib
import pathlib
import sys

run_dir = pathlib.Path(sys.argv[1]).resolve()


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


rows: list[tuple[str, str, str, int]] = []
for index in range(1, 21):
    path = run_dir / "plugins" / f"stress-plugin-{index:02d}.jar"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(f"plugin-{index:02d}\n".encode("utf-8"))
    rows.append(("stress_plugin_jar", path.relative_to(run_dir).as_posix(), sha256(path), path.stat().st_size))

for index in range(1, 11):
    path = run_dir / "world" / "datapacks" / f"stress-datapack-{index:02d}.zip"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(f"datapack-{index:02d}\n".encode("utf-8"))
    rows.append(("stress_datapack_zip", path.relative_to(run_dir).as_posix(), sha256(path), path.stat().st_size))

manifest = run_dir / "stress-corpus-manifest.txt"
with manifest.open("w", encoding="utf-8") as handle:
    handle.write("kind\trelative_path\tsha256\tbytes\n")
    for kind, relative_path, digest, size in rows:
        handle.write(f"{kind}\t{relative_path}\t{digest}\t{size}\n")

print(manifest, sha256(manifest))
PY
}

read -r STRESS_MANIFEST_PATH STRESS_MANIFEST_SHA256 < <(
  create_stress_manifest_fixture "$TMP_DIR/stress-run"
)

cat > "$window_pass" <<'SUMMARY'
bots=500 view_distance=32 simulation_distance=32 bot_exit=0
world_mode=fresh
claim_surface=cold-fresh
world_warm_source_present=false
spark_background_profiler=false
load_test_scenario=block
load_test_gamemode=creative
bukkit_connection_throttle=0
metrics_samples=20
online_max=500
loaded_chunks_max=5476
tps1_avg=1.00
tps1_min=1.00
avg_tick_ms_avg=999.00
avg_tick_ms_max=999.00
load_window_policy=until_first_online_drop_after_reaching_bots
load_window_reached_full_online=true
load_window_ended_by_online_drop=true
load_window_metrics_samples=20
load_window_online_max=500
load_window_loaded_chunks_max=5476
load_window_tps1_avg=19.55
load_window_tps1_min=18.07
load_window_avg_tick_ms_avg=42.61
load_window_avg_tick_ms_max=65.09
process_rss_mib_max=12044.1
host_cpu_count=12
host_system_load1_per_cpu_max=0.620
host_cpu_windows=20
host_cpu_iowait_percent_max=1.00
host_cpu_iowait_percent_avg=0.25
host_cpu_steal_percent_max=1.00
host_cpu_steal_percent_avg=0.10
bot_created_max=500
bot_connected_max=500
bot_ready_max=500
bot_active_max=500
bot_kicked_max=0
bot_errors_max=0
server_join_events=500
server_quit_events=500
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_creative_slot_packets_max=500
bot_block_place_packets_max=59000
bot_block_dig_packets_max=59000
bot_block_action_errors_max=0
bot_action_start_mode=all-ready
bot_action_ready_settle_ms=15000
bot_action_ready_requires_block_armed=true
bot_action_ready_min_count=500
bot_action_ready_min_fraction=1
bot_action_gate_opened=true
bot_action_gate_open_mode=all-ready
bot_action_gate_opened_after_ms=180000
bot_action_gate_open_ready=500
bot_action_gate_open_active=500
bot_action_gate_open_settled=500
bot_action_gate_open_required=500
bot_action_gate_open_block_armed=500
compat_probe_block_places_max=59000
compat_probe_block_breaks_max=59000
compat_probe_arena_prepared_max=500
compat_probe_block_evidence_accepted=true
compat_probe_direct_block_loadbot_event_lines=59000
compat_probe_direct_block_loadbot_place_event_lines=29500
compat_probe_direct_block_loadbot_break_event_lines=29500
compat_probe_direct_block_loadbot_cancelled_false_lines=59000
compat_probe_direct_block_loadbot_players=500
moved_too_quickly_warnings=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
external_thread_prints=0
SUMMARY

python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$window_pass" > "$TMP_DIR/window-pass.out"
grep -q '^claim_eligible=true$' "$TMP_DIR/window-pass.out"
grep -q '^required_compat_probe_block_places_min=500$' "$TMP_DIR/window-pass.out"
grep -q '^required_compat_probe_block_breaks_min=500$' "$TMP_DIR/window-pass.out"
grep -q '^required_action_start_mode=all-ready$' "$TMP_DIR/window-pass.out"
grep -q '^required_action_gate_settle_ms_min=15000$' "$TMP_DIR/window-pass.out"
grep -q '^requires_action_gate_block_armed=true$' "$TMP_DIR/window-pass.out"
grep -q '^required_host_steal_percent_max=10.00$' "$TMP_DIR/window-pass.out"
grep -q '^requires_host_metrics=true$' "$TMP_DIR/window-pass.out"
grep -q '^observed_compat_probe_block_places_max=59000$' "$TMP_DIR/window-pass.out"
grep -q '^observed_compat_probe_block_breaks_max=59000$' "$TMP_DIR/window-pass.out"
grep -q '^observed_host_cpu_steal_percent_max=1.00$' "$TMP_DIR/window-pass.out"

cat > "$stress_pass" <<SUMMARY
bots=50 view_distance=16 simulation_distance=16 bot_exit=0
world_mode=fresh
claim_surface=cold-fresh
world_warm_source_present=false
spark_background_profiler=false
stress_corpus=true
stress_plugins_enabled=true
stress_datapacks_enabled=true
stress_corpus_manifest_path=$STRESS_MANIFEST_PATH
stress_corpus_manifest_sha256=$STRESS_MANIFEST_SHA256
stress_plugin_jars=20
stress_datapack_zips=10
load_test_scenario=mixed
load_test_gamemode=survival
bukkit_connection_throttle=0
metrics_samples=12
online_max=50
loaded_chunks_max=420
tps1_avg=18.50
tps1_min=16.50
avg_tick_ms_avg=60.00
avg_tick_ms_max=120.00
process_rss_mib_max=4096.0
host_cpu_count=12
host_system_load1_per_cpu_max=0.620
host_cpu_windows=12
host_cpu_iowait_percent_max=1.00
host_cpu_iowait_percent_avg=0.25
host_cpu_steal_percent_max=1.00
host_cpu_steal_percent_avg=0.10
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
mob_storm_requested=150
compat_probe_mobstorm_spawned_max=150
compat_probe_mobstorm_spawned_total=150
compat_probe_living_entities_max=150
moved_too_quickly_warnings=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
external_thread_prints=0
SUMMARY

python3 "$ROOT/scripts/evaluate_load_gate.py" --profile stress-mixed "$stress_pass" > "$TMP_DIR/stress-pass.out"
grep -q '^claim_eligible=true$' "$TMP_DIR/stress-pass.out"
grep -q '^required_stress_corpus_source=manifest$' "$TMP_DIR/stress-pass.out"
grep -q '^observed_stress_plugin_jars=20$' "$TMP_DIR/stress-pass.out"
grep -q '^observed_stress_datapack_zips=10$' "$TMP_DIR/stress-pass.out"

sed -e '/^stress_corpus_manifest_path=/d' "$stress_pass" > "$stress_missing_manifest"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile stress-mixed "$stress_missing_manifest" > "$TMP_DIR/stress-missing-manifest.out" 2>&1; then
  echo "Expected stress-mixed summary without manifest path to fail, but it passed." >&2
  cat "$TMP_DIR/stress-missing-manifest.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/stress-missing-manifest.out"
grep -q 'stress_corpus_manifest_path is missing' "$TMP_DIR/stress-missing-manifest.out"

cp -a "$TMP_DIR/stress-run" "$TMP_DIR/stress-run-manifest-drift"
bad_manifest="$TMP_DIR/stress-run-manifest-drift/stress-corpus-manifest.txt"
sed \
  -e "s|^stress_corpus_manifest_path=.*$|stress_corpus_manifest_path=$bad_manifest|" \
  "$stress_pass" > "$stress_bad_manifest_sha"
printf '# drift\n' >> "$bad_manifest"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile stress-mixed "$stress_bad_manifest_sha" > "$TMP_DIR/stress-bad-manifest-sha.out" 2>&1; then
  echo "Expected tampered stress manifest sha summary to fail, but it passed." >&2
  cat "$TMP_DIR/stress-bad-manifest-sha.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/stress-bad-manifest-sha.out"
grep -q 'stress_corpus_manifest_sha256=' "$TMP_DIR/stress-bad-manifest-sha.out"

sed \
  -e 's/^stress_plugin_jars=20$/stress_plugin_jars=21/' \
  "$stress_pass" > "$stress_count_mismatch"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile stress-mixed "$stress_count_mismatch" > "$TMP_DIR/stress-count-mismatch.out" 2>&1; then
  echo "Expected stress count mismatch summary to fail, but it passed." >&2
  cat "$TMP_DIR/stress-count-mismatch.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/stress-count-mismatch.out"
grep -q 'stress_plugin_jars=21 != manifest count 20' "$TMP_DIR/stress-count-mismatch.out"

cp -a "$TMP_DIR/stress-run" "$TMP_DIR/stress-run-file-drift"
file_drift_manifest="$TMP_DIR/stress-run-file-drift/stress-corpus-manifest.txt"
sed \
  -e "s|^stress_corpus_manifest_path=.*$|stress_corpus_manifest_path=$file_drift_manifest|" \
  "$stress_pass" > "$stress_file_drift"
printf 'mutated\n' > "$TMP_DIR/stress-run-file-drift/plugins/stress-plugin-01.jar"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile stress-mixed "$stress_file_drift" > "$TMP_DIR/stress-file-drift.out" 2>&1; then
  echo "Expected stress file drift summary to fail, but it passed." >&2
  cat "$TMP_DIR/stress-file-drift.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/stress-file-drift.out"
grep -q 'stress_corpus_manifest:2: plugins/stress-plugin-01.jar sha256=' "$TMP_DIR/stress-file-drift.out"

sed \
  -e 's/^bot_action_start_mode=all-ready$/bot_action_start_mode=timer/' \
  -e 's/^bot_action_ready_settle_ms=15000$/bot_action_ready_settle_ms=0/' \
  -e 's/^bot_action_ready_requires_block_armed=true$/bot_action_ready_requires_block_armed=false/' \
  -e 's/^bot_action_gate_open_mode=all-ready$/bot_action_gate_open_mode=timer/' \
  -e 's/^bot_action_gate_open_ready=500$/bot_action_gate_open_ready=4/' \
  -e 's/^bot_action_gate_open_active=500$/bot_action_gate_open_active=4/' \
  -e 's/^bot_action_gate_open_settled=500$/bot_action_gate_open_settled=0/' \
  -e 's/^bot_action_gate_open_required=500$/bot_action_gate_open_required=0/' \
  -e 's/^bot_action_gate_open_block_armed=500$/bot_action_gate_open_block_armed=0/' \
  "$window_pass" > "$TMP_DIR/timer-open-summary.txt"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$TMP_DIR/timer-open-summary.txt" > "$TMP_DIR/timer-open.out" 2>&1; then
  echo "Expected timer-open production action gate summary to fail, but it passed." >&2
  cat "$TMP_DIR/timer-open.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/timer-open.out"
grep -q 'bot_action_start_mode=timer != required all-ready' "$TMP_DIR/timer-open.out"
grep -q 'bot_action_gate_open_mode=timer != required all-ready' "$TMP_DIR/timer-open.out"
grep -q 'bot_action_gate_open_ready=4 < required 500' "$TMP_DIR/timer-open.out"

sed \
  -e 's/^world_mode=fresh$/world_mode=warm-source/' \
  -e 's/^claim_surface=cold-fresh$/claim_surface=warm-source/' \
  -e 's/^world_warm_source_present=false$/world_warm_source_present=true/' \
  "$window_pass" > "$warm_pass"
python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500-warm "$warm_pass" > "$TMP_DIR/warm-pass.out"
grep -q '^claim_eligible=true$' "$TMP_DIR/warm-pass.out"
grep -q '^required_compat_probe_block_places_min=500$' "$TMP_DIR/warm-pass.out"
grep -q '^required_compat_probe_block_breaks_min=500$' "$TMP_DIR/warm-pass.out"

{
  cat "$window_pass"
  printf 'bot_log_tail:\n'
  printf 'bot-side diagnostic ignored_key=999\n'
  printf 'early_abort_reason=host_contention_bad_samples=3_load_per_cpu=0.785_max_load_per_cpu=0.750_steal_percent=47.63_max_steal_percent=10.00\n'
} > "$contention_invalid"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$contention_invalid" > "$TMP_DIR/contention-invalid.out" 2>&1; then
  echo "Expected host-contention-contaminated summary to fail, but it passed." >&2
  cat "$TMP_DIR/contention-invalid.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/contention-invalid.out"
grep -q '^run_class=environment-invalid$' "$TMP_DIR/contention-invalid.out"
grep -q '^environment_invalid=true$' "$TMP_DIR/contention-invalid.out"
grep -q '^environment_invalid_kind=host_contention$' "$TMP_DIR/contention-invalid.out"
grep -q '^observed_early_abort_reason=host_contention_bad_samples=3_' "$TMP_DIR/contention-invalid.out"
grep -q 'failure=environment_invalid=true; kind=host_contention;' "$TMP_DIR/contention-invalid.out"

sed -e 's/^host_cpu_steal_percent_max=1.00$/host_cpu_steal_percent_max=47.63/' "$window_pass" > "$TMP_DIR/high-steal-summary.txt"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$TMP_DIR/high-steal-summary.txt" > "$TMP_DIR/high-steal.out" 2>&1; then
  echo "Expected high host steal summary to fail, but it passed." >&2
  cat "$TMP_DIR/high-steal.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/high-steal.out"
grep -q 'host_cpu_steal_percent_max=47.63 > allowed 10.00' "$TMP_DIR/high-steal.out"

grep -Ev '^(load_window_|teardown_)' "$window_pass" > "$fallback_fail"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$fallback_fail" > "$TMP_DIR/fallback-fail.out" 2>&1; then
  echo "Expected teardown-inclusive fallback summary to fail, but it passed." >&2
  cat "$TMP_DIR/fallback-fail.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/fallback-fail.out"

grep -Ev '^compat_probe_block_(places|breaks)_max=' "$window_pass" > "$missing_block_probe"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$missing_block_probe" > "$TMP_DIR/missing-block-probe.out" 2>&1; then
  echo "Expected summary without server-side block workload to fail, but it passed." >&2
  cat "$TMP_DIR/missing-block-probe.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/missing-block-probe.out"
grep -q 'compat_probe_block_places_max is missing, required >= 500' "$TMP_DIR/missing-block-probe.out"
grep -q 'compat_probe_block_breaks_max is missing, required >= 500' "$TMP_DIR/missing-block-probe.out"

sed \
  -e 's/^compat_probe_block_places_max=59000$/compat_probe_block_places_max=499/' \
  -e 's/^compat_probe_block_breaks_max=59000$/compat_probe_block_breaks_max=499/' \
  "$window_pass" > "$low_block_probe"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$low_block_probe" > "$TMP_DIR/low-block-probe.out" 2>&1; then
  echo "Expected summary with low server-side block workload to fail, but it passed." >&2
  cat "$TMP_DIR/low-block-probe.out" >&2
  exit 1
fi
grep -q '^claim_eligible=false$' "$TMP_DIR/low-block-probe.out"
grep -q 'compat_probe_block_places_max=499 < required 500' "$TMP_DIR/low-block-probe.out"
grep -q 'compat_probe_block_breaks_max=499 < required 500' "$TMP_DIR/low-block-probe.out"

grep -Ev '^load_window_tps1_min=' "$window_pass" > "$missing_window"
if python3 "$ROOT/scripts/evaluate_load_gate.py" --profile production-500 "$missing_window" > "$TMP_DIR/missing-window.out" 2>&1; then
  echo "Expected incomplete load-window summary to fail, but it passed." >&2
  cat "$TMP_DIR/missing-window.out" >&2
  exit 1
fi
grep -q 'load_window_tps1_min is missing' "$TMP_DIR/missing-window.out"

echo "evaluate_load_gate_window_smoke=PASS"
