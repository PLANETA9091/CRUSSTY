#!/usr/bin/env bash

create_production_ready_claim_smoke_fixture() {
  local tmp_root="${1:?tmp root is required}"
  local -n readiness_report_ref="${2:?readiness report variable name is required}"
  local -n reports_dir_ref="${3:?reports dir variable name is required}"

  local fixture_root="$tmp_root/production-ready-claim-fixture"
  local reports_dir="$fixture_root/reports"
  local readiness_report="$fixture_root/production-500-readiness-gate.txt"

  python3 - "$fixture_root" "$reports_dir" "$readiness_report" <<'PY'
from __future__ import annotations

import hashlib
import json
import os
import sys
from pathlib import Path

fixture_root = Path(sys.argv[1]).resolve()
reports_dir = Path(sys.argv[2]).resolve()
readiness_report = Path(sys.argv[3]).resolve()
reports_dir.mkdir(parents=True, exist_ok=True)
readiness_report.parent.mkdir(parents=True, exist_ok=True)

optimized_sha = "a" * 64
runtime_sha = "b" * 64
current_evidence_epoch = 1_700_000_000
readiness_epoch = 1_700_000_100
def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write(path: Path, content: str | bytes, *, executable: bool = False) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(content, bytes):
        path.write_bytes(content)
    else:
        path.write_text(content, encoding="utf-8")
    if executable:
        path.chmod(0o755)
    os.utime(path, (readiness_epoch, readiness_epoch))
    return path


artifact_root = fixture_root / "artifacts"
optimized_artifact = write(
    artifact_root / "optimized-paper-1.21.10-mojmap.jar",
    b"optimized smoke artifact\n",
)
run_sh = write(
    artifact_root / "optimized-runtime" / "run.sh",
    "#!/usr/bin/env sh\nexec java -jar runtime.jar \"$@\"\n",
    executable=True,
)
native_library = write(
    artifact_root / "optimized-runtime" / "native" / "libpaper_native_jni.so",
    b"native smoke library\n",
)
chunk_encode_native_library = write(
    artifact_root / "optimized-runtime" / "native" / "libpaper_native_chunk_encode_jni.so",
    b"native chunk encode smoke library\n",
)
optimized_sha = sha256(optimized_artifact)
runtime_sha = sha256(run_sh)
native_sha = sha256(native_library)
chunk_encode_native_sha = sha256(chunk_encode_native_library)
runtime_jar_sha_file = write(
    artifact_root / "optimized-runtime" / "runtime.jar.sha256",
    f"{optimized_sha}  runtime.jar\n",
)
native_sha_file = write(
    artifact_root / "optimized-runtime" / "native" / "libpaper_native_jni.so.sha256",
    f"{native_sha}  {native_library}\n",
)
chunk_encode_native_sha_file = write(
    artifact_root / "optimized-runtime" / "native" / "libpaper_native_chunk_encode_jni.so.sha256",
    f"{chunk_encode_native_sha}  {chunk_encode_native_library}\n",
)
runtime_jar_sha_file_sha = sha256(runtime_jar_sha_file)
native_sha_file_sha = sha256(native_sha_file)
chunk_encode_native_sha_file_sha = sha256(chunk_encode_native_sha_file)

artifacts_json = write(
    reports_dir / "artifacts.json",
    json.dumps(
        {
            "optimized": {
                "path": str(optimized_artifact),
                "sha256": optimized_sha,
            },
            "optimized_runtime": {
                "run_sh": {
                    "path": str(run_sh),
                    "sha256": runtime_sha,
                },
                "runtime_jar_sha256_file": {
                    "path": str(runtime_jar_sha_file),
                    "sha256": runtime_jar_sha_file_sha,
                    "runtime_jar_sha256": optimized_sha,
                },
                "native_library": {
                    "path": str(native_library),
                    "sha256": native_sha,
                },
                "native_library_sha256_file": {
                    "path": str(native_sha_file),
                    "sha256": native_sha_file_sha,
                    "native_library_sha256": native_sha,
                },
                "chunk_encode_native_library": {
                    "path": str(chunk_encode_native_library),
                    "sha256": chunk_encode_native_sha,
                },
                "chunk_encode_native_library_sha256_file": {
                    "path": str(chunk_encode_native_sha_file),
                    "sha256": chunk_encode_native_sha_file_sha,
                    "chunk_encode_native_library_sha256": chunk_encode_native_sha,
                },
            },
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
)

artifact_rows = [
    (optimized_sha, optimized_artifact),
    (runtime_sha, run_sh),
    (runtime_jar_sha_file_sha, runtime_jar_sha_file),
    (native_sha, native_library),
    (native_sha_file_sha, native_sha_file),
    (chunk_encode_native_sha, chunk_encode_native_library),
    (chunk_encode_native_sha_file_sha, chunk_encode_native_sha_file),
]
artifact_hash_manifest = write(
    reports_dir / "artifact-hashes.txt",
    "".join(f"{digest}  {path}\n" for digest, path in artifact_rows),
)
artifact_hash_count = len(artifact_rows)

summary_template = """\
bots=500 view_distance=32 simulation_distance=32 bot_exit=0
duration_seconds=2400
optimized_artifact_sha256={optimized_sha}
optimized_runtime_run_sh_sha256={runtime_sha}
optimized_runtime_jar_sha256={optimized_sha}
optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_sha}
world_mode={world_mode}
claim_surface={claim_surface}
world_warm_source_present={world_warm_source_present}
load_test_scenario=block
load_test_gamemode=creative
native_runtime_line=[optimized-runtime] native_lib_available=true native_area_map=true native_improved_noise=true native_normal_noise=true native_perlin_noise=false
native_area_map_loaded=true
native_improved_noise_loaded=true
native_normal_noise_loaded=true
native_perlin_noise_loaded=false
bukkit_connection_throttle=0
compat_probe_send_pressure_samples=300
compat_probe_send_pressure_players_max=500
compat_probe_send_pressure_connections_max=500
compat_probe_send_pressure_chunk_senders_max=32
compat_probe_send_pending_actions_max=0
compat_probe_send_pending_outbound_bytes_max=0
compat_probe_send_bytes_before_writable_max=0
compat_probe_send_bytes_before_unwritable_min=0
compat_probe_send_non_writable_connections_max=0
compat_probe_chunk_send_pending_chunks_max=0
compat_probe_chunk_send_unacknowledged_batches_max=0
compat_probe_chunk_send_max_unacknowledged_batches_max=0
compat_probe_chunk_send_channel_not_writable_skips_max=0
compat_probe_chunk_send_batch_quota_max=0.00
compat_probe_chunk_send_desired_chunks_per_tick_max=0.00
load_window_reached_full_online=true
load_window_metrics_samples=300
load_window_online_max=500
load_window_loaded_chunks_max=5476
load_window_tps1_avg={tps_avg}
load_window_tps1_min={tps_min}
load_window_avg_tick_ms_avg=38.00
load_window_avg_tick_ms_max={mspt_max}
online_max=500
loaded_chunks_max=5476
tps1_avg={tps_avg}
tps1_min={tps_min}
avg_tick_ms_avg=38.00
avg_tick_ms_max={mspt_max}
server_join_events=500
server_quit_events=500
bot_created_max=500
bot_connected_max=500
bot_ready_max=500
bot_active_max=500
bot_kicked_max=0
bot_errors_max=0
bot_action_start_mode=all-ready
bot_action_gate_open_mode=all-ready
bot_action_ready_settle_ms=15000
bot_action_ready_requires_block_armed=true
bot_action_gate_opened=true
bot_action_ready_min_count=500
bot_action_ready_min_fraction=1
bot_action_gate_open_ready=500
bot_action_gate_open_active=500
bot_action_gate_open_settled=500
bot_action_gate_open_required=500
bot_action_gate_open_block_armed=500
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_creative_slot_packets_max=500
bot_block_place_packets_max={place_packets}
bot_block_dig_packets_max={dig_packets}
bot_block_action_errors_max=0
compat_probe_arena_prepared_max=500
compat_probe_block_evidence_accepted=true
compat_probe_direct_block_loadbot_event_lines=1
compat_probe_direct_block_loadbot_place_event_lines=1
compat_probe_direct_block_loadbot_break_event_lines=1
compat_probe_direct_block_loadbot_cancelled_false_lines=1
compat_probe_direct_block_loadbot_players=500
compat_probe_block_places_max=500
compat_probe_block_breaks_max=500
resource_samples=300
process_cpu_max=100.0
process_rss_mib_max=1024.0
host_cpu_count=12
host_system_load1_max=1.0
host_system_load1_per_cpu_max=0.10
host_cpu_windows=300
host_cpu_steal_percent_max=0.10
host_cpu_iowait_percent_max=0.10
host_mem_available_kb_min=1000000
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
bot_log_tail:
2024-01-01T00:00:00.000Z bot_end username=LoadBot499 reason=swarm-shutdown
"""

cold_summary = write(
    reports_dir / "load-production-500-cold-current-artifact-smoke-summary.txt",
    summary_template.format(
        optimized_sha=optimized_sha,
        runtime_sha=runtime_sha,
        chunk_encode_native_sha=chunk_encode_native_sha,
        world_mode="fresh",
        claim_surface="cold-fresh",
        world_warm_source_present="false",
        tps_avg="19.77",
        tps_min="18.86",
        mspt_max="66.82",
        place_packets="399000",
        dig_packets="398500",
    ),
)
warm_summary = write(
    reports_dir / "load-production-500-warm-current-artifact-smoke-summary.txt",
    summary_template.format(
        optimized_sha=optimized_sha,
        runtime_sha=runtime_sha,
        chunk_encode_native_sha=chunk_encode_native_sha,
        world_mode="warm-source",
        claim_surface="warm-world",
        world_warm_source_present="true",
        tps_avg="19.92",
        tps_min="19.38",
        mspt_max="56.90",
        place_packets="407000",
        dig_packets="407000",
    ),
)

preflight_content = """\
host_preflight_ok=true
host_preflight_host_checked=true
host_preflight_host_ok=true
cpu_count=12
load1=0.10
load5=0.10
load15=0.10
load_per_cpu=0.01
idle_percent_1s=99.00
min_idle_percent=40.00
max_load_per_cpu=0.75
"""
resources_content = """\
ts_ms,pid_cpu,pid_rss_kb,system_load1,system_mem_available_kb
0,0.0,1048576,0.10,1000000
"""
for summary in (cold_summary, warm_summary):
    base = summary.name[:-len("-summary.txt")]
    write(summary.with_name(base + "-preflight.txt"), preflight_content)
    write(summary.with_name(base + "-resources.csv"), resources_content)
    write(summary.with_name(base + "-gate.txt"), "gate_pass=true\n")

logs_dir = fixture_root / "logs"
plugin_matrix_log = write(
    logs_dir / "plugin-matrix.log",
    "\n".join(
        [
            "[00:00:00 INFO]: Done (12.345s)! For help, type \"help\"",
            "[00:00:00 INFO]: [LibraryProbe] Enabling LibraryProbe",
            "[00:00:00 INFO]: [CompatProbe] Enabling CompatProbe",
            "[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE lifecycle=enable",
            "[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE scheduler=async ticked=true",
            "[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE scheduler=sync ticked=true",
            "[00:00:02 INFO]: Initialized 11 plugins",
            "[00:00:03 INFO]: CodexJoinProbe joined the game",
            "[00:00:04 INFO]: [CompatProbe] COMPAT_PROBE event=PlayerJoinEvent detail=CodexJoinProbe",
            "[00:00:05 INFO]: [CompatProbe] COMPAT_PROBE event=PlayerQuitEvent detail=CodexJoinProbe",
            "[00:00:06 INFO]: [CompatProbe] COMPAT_PROBE command=ok events=4",
        ]
    )
    + "\n",
)
restart_recovery_log = write(
    logs_dir / "restart-recovery.log",
    "\n".join(
        [
            "[00:00:00 INFO]: Done (10.111s)! For help, type \"help\"",
            "[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE scheduler=async ticked=true",
            "[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE scheduler=sync ticked=true",
            "[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE command=ok events=2",
            "[00:00:01 INFO]: Saved the game",
            "[00:00:02 INFO]: [CompatProbe] COMPAT_PROBE lifecycle=disable",
        ]
    )
    + "\n",
)
forced_ticket_first_log = write(
    logs_dir / "forced-ticket-persistence-first.log",
    "\n".join(
        [
            "[00:00:00 INFO]: Done (9.001s)! For help, type \"help\"",
            "[00:00:01 INFO]: Marked chunk [0, 0] in minecraft:overworld for force loading",
            "[00:00:01 INFO]: chunk [0, 0] marked for force loading",
            "[00:00:02 INFO]: Saved the game",
        ]
    )
    + "\n",
)
forced_ticket_restart_log = write(
    logs_dir / "forced-ticket-persistence-restart.log",
    "\n".join(
        [
            "[00:00:00 INFO]: Done (9.500s)! For help, type \"help\"",
            "[00:00:01 INFO]: Force loaded chunks in minecraft:overworld: 0, 0",
        ]
    )
    + "\n",
)

go_nogo_report = write(
    reports_dir / "production-500-go-nogo-current.txt",
    "\n".join(
        [
            "production_500_go_nogo_profile=production-500-control-plane",
            "production_500_go_nogo_generated_at_utc=2024-01-01T00:00:00+00:00",
            "production_500_go_nogo_foreign_pattern=java --add-modules|server\\.jar|mc_bot|probe\\.js",
            "production_500_go_nogo_canary_duration_seconds=15",
            "production_500_go_nogo_canary_sample_interval_seconds=1",
            "production_500_go_nogo_canary_max_steal_percent=10",
            "production_500_go_nogo_canary_max_iowait_percent=10",
            "production_500_go_nogo_pass=true",
            "production_500_go_nogo_exit_code=0",
            "production_500_go_nogo_reason=none",
        ]
    )
    + "\n",
)

soak_report = write(
    reports_dir / "production-500-soak-gate.txt",
    "\n".join(
        [
            "claim_text=500-bots-production-ready-for-measured-32-32-creative-block-profile",
            "claim_scope=cold-fresh-and-warm-source-500-bots-32-view-32-simulation-creative-block-soak",
            "production_ready_soak_claim_eligible=true",
            "soak_gate_pass=true",
            "base_cold_gate_pass=true",
            "base_warm_gate_pass=true",
            "artifact_hashes_pass=true",
            "cold_gate_pass=true",
            "warm_gate_pass=true",
            "cold_load_window_reached_full_online=true",
            "warm_load_window_reached_full_online=true",
            "failure_count=0",
            f"artifact_hash_count={artifact_hash_count}",
            f"artifact_hash_manifest={artifact_hash_manifest}",
            f"artifacts_json={artifacts_json}",
            f"optimized_artifact_path={optimized_artifact}",
            f"optimized_artifact_sha256={optimized_sha}",
            f"optimized_runtime_run_sh={run_sh}",
            f"optimized_runtime_chunk_encode_native_library={chunk_encode_native_library}",
            f"optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_sha}",
            f"cold_summary_path={cold_summary}",
            f"warm_summary_path={warm_summary}",
            "cold_failure_count=0",
            "warm_failure_count=0",
            "cold_bots=500",
            "warm_bots=500",
            "cold_view_distance=32",
            "warm_view_distance=32",
            "cold_simulation_distance=32",
            "warm_simulation_distance=32",
            "cold_duration_seconds=2400",
            "warm_duration_seconds=2400",
            "cold_world_mode=fresh",
            "warm_world_mode=warm-source",
            "cold_claim_surface=cold-fresh",
            "warm_claim_surface=warm-world",
            "cold_load_test_scenario=block",
            "warm_load_test_scenario=block",
            "cold_load_test_gamemode=creative",
            "warm_load_test_gamemode=creative",
            "cold_spark_background_profiler=false",
            "warm_spark_background_profiler=false",
            "cold_compat_probe_send_pressure_samples=300",
            "warm_compat_probe_send_pressure_samples=300",
            "cold_compat_probe_send_pressure_players_max=500",
            "warm_compat_probe_send_pressure_players_max=500",
            "cold_compat_probe_send_pressure_connections_max=500",
            "warm_compat_probe_send_pressure_connections_max=500",
            "cold_compat_probe_send_pressure_chunk_senders_max=32",
            "warm_compat_probe_send_pressure_chunk_senders_max=32",
            "cold_compat_probe_send_pending_actions_max=0",
            "warm_compat_probe_send_pending_actions_max=0",
            "cold_compat_probe_send_pending_outbound_bytes_max=0",
            "warm_compat_probe_send_pending_outbound_bytes_max=0",
            "cold_compat_probe_send_bytes_before_writable_max=0",
            "warm_compat_probe_send_bytes_before_writable_max=0",
            "cold_compat_probe_send_bytes_before_unwritable_min=0",
            "warm_compat_probe_send_bytes_before_unwritable_min=0",
            "cold_compat_probe_send_non_writable_connections_max=0",
            "warm_compat_probe_send_non_writable_connections_max=0",
            "cold_compat_probe_chunk_send_pending_chunks_max=0",
            "warm_compat_probe_chunk_send_pending_chunks_max=0",
            "cold_compat_probe_chunk_send_unacknowledged_batches_max=0",
            "warm_compat_probe_chunk_send_unacknowledged_batches_max=0",
            "cold_compat_probe_chunk_send_max_unacknowledged_batches_max=0",
            "warm_compat_probe_chunk_send_max_unacknowledged_batches_max=0",
            "cold_compat_probe_chunk_send_channel_not_writable_skips_max=0",
            "warm_compat_probe_chunk_send_channel_not_writable_skips_max=0",
            "cold_compat_probe_chunk_send_batch_quota_max=0.00",
            "warm_compat_probe_chunk_send_batch_quota_max=0.00",
            "cold_compat_probe_chunk_send_desired_chunks_per_tick_max=0.00",
            "warm_compat_probe_chunk_send_desired_chunks_per_tick_max=0.00",
            "cold_load_window_metrics_samples=300",
            "warm_load_window_metrics_samples=300",
            "cold_load_window_online_max=500",
            "warm_load_window_online_max=500",
            "cold_load_window_loaded_chunks_max=5476",
            "warm_load_window_loaded_chunks_max=5476",
            "cold_load_window_tps1_avg=19.77",
            "warm_load_window_tps1_avg=19.92",
            "cold_load_window_tps1_min=18.86",
            "warm_load_window_tps1_min=19.38",
            "cold_load_window_avg_tick_ms_avg=38.00",
            "warm_load_window_avg_tick_ms_avg=38.00",
            "cold_load_window_avg_tick_ms_max=66.82",
            "warm_load_window_avg_tick_ms_max=56.90",
            "cold_bot_block_place_packets_max=399000",
            "warm_bot_block_place_packets_max=407000",
            "cold_bot_block_dig_packets_max=398500",
            "warm_bot_block_dig_packets_max=407000",
            "cold_bot_block_action_errors_max=0",
            "warm_bot_block_action_errors_max=0",
            "cold_stability_failures=0",
            "warm_stability_failures=0",
            "cold_watchdog_thread_dumps=0",
            "warm_watchdog_thread_dumps=0",
            "cold_sync_load_stack_hits=0",
            "warm_sync_load_stack_hits=0",
            f"cold_optimized_artifact_sha256={optimized_sha}",
            f"warm_optimized_artifact_sha256={optimized_sha}",
            f"cold_optimized_runtime_run_sh_sha256={runtime_sha}",
            f"warm_optimized_runtime_run_sh_sha256={runtime_sha}",
            f"cold_optimized_runtime_jar_sha256={optimized_sha}",
            f"warm_optimized_runtime_jar_sha256={optimized_sha}",
            f"cold_optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_sha}",
            f"warm_optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_sha}",
        ]
    )
    + "\n",
)

repeat_report = write(
    reports_dir / "production-500-repeat-quorum.txt",
    "\n".join(
        [
            "required_min_passes=3",
            "repeat_run_count=3",
            "repeat_passes=3",
            "repeat_failures=0",
    "repeat_quorum_pass=true",
    *[
        item
        for index in range(1, 4)
        for item in [
            f"run_{index}_dir={reports_dir / f'repeat-{index}'}",
            f"run_{index}_pass=true",
            f"run_{index}_production_ready_claim_eligible=true",
            f"run_{index}_release_gate_pass=true",
            f"run_{index}_failure_count=0",
            f"run_{index}_cold_load_window_tps1_avg=19.77",
            f"run_{index}_cold_load_window_tps1_min=18.86",
            f"run_{index}_cold_load_window_avg_tick_ms_max=66.82",
            f"run_{index}_warm_load_window_tps1_avg=19.92",
            f"run_{index}_warm_load_window_tps1_min=19.38",
            f"run_{index}_warm_load_window_avg_tick_ms_max=56.90",
            f"run_{index}_optimized_artifact_sha256={optimized_sha}",
        ]
    ],
        ]
    )
    + "\n",
)

plugin_matrix_summary = write(
    reports_dir / "plugin-matrix-summary.txt",
    "\n".join(
        [
            f"plugin_matrix_log={plugin_matrix_log}",
            "plugin_matrix_profile=production-500-smoke",
            "plugin_matrix_pass=true",
            "status_json={}",
            "[00:00:00 INFO]: Done (12.345s)! For help, type \"help\"",
            "Initialized 11 plugins",
            "plugin_matrix_detail_1=[LibraryProbe] Enabling LibraryProbe",
            "plugin_matrix_detail_2=[CompatProbe] Enabling CompatProbe",
            "plugin_matrix_detail_3=COMPAT_PROBE lifecycle=enable",
            "plugin_matrix_detail_4=COMPAT_PROBE scheduler=async ticked=true",
            "plugin_matrix_detail_5=COMPAT_PROBE scheduler=sync ticked=true",
            "plugin_matrix_detail_6=COMPAT_PROBE event=PlayerJoinEvent detail=CodexJoinProbe",
            "plugin_matrix_detail_7=COMPAT_PROBE event=PlayerQuitEvent detail=CodexJoinProbe",
            "plugin_matrix_detail_8=COMPAT_PROBE command=ok events=4",
        ]
    )
    + "\n",
)
restart_recovery_summary = write(
    reports_dir / "restart-recovery-summary.txt",
    "\n".join(
        [
            f"restart_recovery_log={restart_recovery_log}",
            "restart_recovery_profile=production-500-smoke",
            "restart_recovery_pass=true",
            "status_json={}",
            "[00:00:00 INFO]: Done (10.111s)! For help, type \"help\"",
            "restart_recovery_detail_1=COMPAT_PROBE scheduler=async ticked=true",
            "restart_recovery_detail_2=COMPAT_PROBE scheduler=sync ticked=true",
            "restart_recovery_detail_3=COMPAT_PROBE command=ok events=2",
            "Saved the game",
            "restart_recovery_detail_4=COMPAT_PROBE lifecycle=disable",
        ]
    )
    + "\n",
)
forced_ticket_summary = write(
    reports_dir / "forced-ticket-persistence-summary.txt",
    "\n".join(
        [
            "forced_ticket_profile=production-500-smoke",
            "forced_ticket_persistence_pass=true",
            "forced_ticket_persistence=PASS",
            f"first_log={forced_ticket_first_log}",
            f"restart_log={forced_ticket_restart_log}",
            "[00:00:00 INFO]: Done (9.001s)! For help, type \"help\"",
            "Saved the game",
            "chunk [0, 0] marked for force loading",
        ]
    )
    + "\n",
)

evidence_paths: dict[str, Path] = {
    "go_nogo_report": go_nogo_report,
    "soak_report": soak_report,
    "repeat_report": repeat_report,
    "plugin_matrix_summary": plugin_matrix_summary,
    "restart_recovery_summary": restart_recovery_summary,
    "forced_ticket_summary": forced_ticket_summary,
    "artifact_hash_manifest": artifact_hash_manifest,
}
evidence_hashes: dict[str, str] = {
    f"{key}_sha256": sha256(path) for key, path in evidence_paths.items()
}

current_bundle_evidence = reports_dir / "load-production-500-cold-current-artifact-smoke-gate.txt"
for surface in ("cold", "warm"):
    for kind in ("gate", "summary"):
        path = reports_dir / f"load-production-500-{surface}-current-artifact-smoke-{kind}.txt"
        if not path.exists():
            write(path, f"gate_profile=production-500-smoke\nsurface={surface}\n{kind}_pass=true\n")
        os.utime(path, (current_evidence_epoch, current_evidence_epoch))

lines = [
    "readiness_profile=production-500-production-ready-certification",
    "generated_at_utc=2024-01-01T00:00:00+00:00",
    "claim_text=500-bots-production-ready-for-measured-32-32-creative-block-profile",
    (
        "claim_scope=500-bots-32-view-32-simulation-creative-block-cold-warm-"
        "soak-repeat-plugin-restart-forced-ticket"
    ),
    (
        "claim_limits=not-full-paper-runtime-rust-rewrite;not-unbounded-plugin-"
        "compatibility;not-unmeasured-real-player-gameplay;not-multi-hour-soak"
    ),
    "production_ready_500_claim=true",
    "readiness_gate_pass=true",
    "failure_count=0",
    f"go_nogo_report={evidence_paths['go_nogo_report']}",
    "go_nogo_present=true",
    "production_500_go_nogo_pass=true",
    "production_500_go_nogo_exit_code=0",
    "production_500_go_nogo_reason=none",
    "production_500_go_nogo_foreign_pattern=java --add-modules|server\\.jar|mc_bot|probe\\.js",
    "production_500_go_nogo_canary_duration_seconds=15",
    "production_500_go_nogo_canary_sample_interval_seconds=1",
    "production_500_go_nogo_canary_max_steal_percent=10",
    "production_500_go_nogo_canary_max_iowait_percent=10",
    f"go_nogo_report_sha256={evidence_hashes['go_nogo_report_sha256']}",
    "soak_gate_pass=true",
    "repeat_quorum_pass=true",
    "plugin_matrix_pass=true",
    "restart_recovery_pass=true",
    "forced_ticket_persistence_pass=true",
    "artifact_hashes_pass=true",
    "current_artifact_consistency_pass=true",
    f"artifact_hash_count={artifact_hash_count}",
    f"soak_report={evidence_paths['soak_report']}",
    f"repeat_report={evidence_paths['repeat_report']}",
    f"plugin_matrix_summary={evidence_paths['plugin_matrix_summary']}",
    f"restart_recovery_summary={evidence_paths['restart_recovery_summary']}",
    f"forced_ticket_summary={evidence_paths['forced_ticket_summary']}",
    f"artifact_hash_manifest={evidence_paths['artifact_hash_manifest']}",
    f"artifacts_json={artifacts_json}",
    "min_repeat_passes=3",
    "min_soak_samples=300",
    "min_block_packets=120000",
    f"soak_report_sha256={evidence_hashes['soak_report_sha256']}",
    f"repeat_report_sha256={evidence_hashes['repeat_report_sha256']}",
    f"plugin_matrix_summary_sha256={evidence_hashes['plugin_matrix_summary_sha256']}",
    f"restart_recovery_summary_sha256={evidence_hashes['restart_recovery_summary_sha256']}",
    f"forced_ticket_summary_sha256={evidence_hashes['forced_ticket_summary_sha256']}",
    f"artifact_hash_manifest_sha256={evidence_hashes['artifact_hash_manifest_sha256']}",
    f"optimized_artifact_sha256={optimized_sha}",
    f"optimized_runtime_run_sh_sha256={runtime_sha}",
    f"optimized_runtime_native_library_sha256={native_sha}",
    f"optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_sha}",
    "cold_load_window_tps1_avg=19.77",
    "cold_load_window_tps1_min=18.86",
    "cold_load_window_avg_tick_ms_max=66.82",
    "cold_bot_block_place_packets_max=399000",
    "cold_bot_block_dig_packets_max=398500",
    "warm_load_window_tps1_avg=19.92",
    "warm_load_window_tps1_min=19.38",
    "warm_load_window_avg_tick_ms_max=56.90",
    "warm_bot_block_place_packets_max=407000",
    "warm_bot_block_dig_packets_max=407000",
    "repeat_passes=3",
    f"current_optimized_artifact_sha256={optimized_sha}",
    f"current_optimized_runtime_run_sh_sha256={runtime_sha}",
    f"current_optimized_runtime_native_library_sha256={native_sha}",
    f"current_optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_sha}",
]
readiness_report.write_text("\n".join(lines) + "\n", encoding="utf-8")
os.utime(readiness_report, (readiness_epoch, readiness_epoch))
PY

  readiness_report_ref="$readiness_report"
  reports_dir_ref="$reports_dir"
}
