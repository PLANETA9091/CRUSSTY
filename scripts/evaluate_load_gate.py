#!/usr/bin/env python3
"""Evaluate a run_load_test summary against a claim gate."""

from __future__ import annotations

import argparse
import hashlib
import pathlib
import re
import sys
from dataclasses import dataclass
from typing import Callable


SUMMARY_TOKEN_RE = re.compile(r"([A-Za-z0-9_]+)=([^\s]+)")


@dataclass(frozen=True)
class GateProfile:
    name: str
    min_bots: int | None
    required_scenario: str
    required_gamemode: str
    min_view_distance: int
    min_simulation_distance: int
    min_metrics_samples: int
    min_tps1_avg: float
    min_tps1_min: float
    max_avg_tick_ms_avg: float
    max_avg_tick_ms_max: float
    max_rss_mib: float
    min_loaded_chunks_per_bot: float
    require_bukkit_connection_throttle_zero: bool
    require_block_actions: bool
    max_host_load_per_cpu: float = 0.750
    max_host_steal_percent: float = 10.0
    max_host_iowait_percent: float = 10.0
    require_host_metrics: bool = True
    min_server_block_places: int = 0
    min_server_block_breaks: int = 0
    require_stress_corpus: bool = False
    min_stress_plugin_jars: int = 0
    min_stress_datapack_zips: int = 0
    min_mob_storm_spawned: int = 0
    require_warm_world_source: bool = False
    reject_warm_world_source: bool = False
    reject_spark_background_profiler: bool = True
    require_mixed_gameplay_workload: bool = False
    min_mixed_packets_per_bot: int = 1
    required_action_start_mode: str | None = None
    min_action_gate_settle_ms: int = 0
    require_action_gate_block_armed: bool = False


PROFILES: dict[str, GateProfile] = {
    "production-500": GateProfile(
        name="production-500",
        min_bots=500,
        required_scenario="block",
        required_gamemode="creative",
        min_view_distance=32,
        min_simulation_distance=32,
        min_metrics_samples=10,
        min_tps1_avg=19.5,
        min_tps1_min=18.0,
        max_avg_tick_ms_avg=50.0,
        max_avg_tick_ms_max=100.0,
        max_rss_mib=28_672.0,
        min_loaded_chunks_per_bot=8.0,
        require_bukkit_connection_throttle_zero=True,
        require_block_actions=True,
        min_server_block_places=500,
        min_server_block_breaks=500,
        reject_warm_world_source=True,
        required_action_start_mode="all-ready",
        min_action_gate_settle_ms=15000,
        require_action_gate_block_armed=True,
    ),
    "production-500-warm": GateProfile(
        name="production-500-warm",
        min_bots=500,
        required_scenario="block",
        required_gamemode="creative",
        min_view_distance=32,
        min_simulation_distance=32,
        min_metrics_samples=10,
        min_tps1_avg=19.5,
        min_tps1_min=18.0,
        max_avg_tick_ms_avg=50.0,
        max_avg_tick_ms_max=100.0,
        max_rss_mib=28_672.0,
        min_loaded_chunks_per_bot=8.0,
        require_bukkit_connection_throttle_zero=True,
        require_block_actions=True,
        min_server_block_places=500,
        min_server_block_breaks=500,
        require_warm_world_source=True,
        required_action_start_mode="all-ready",
        min_action_gate_settle_ms=15000,
        require_action_gate_block_armed=True,
    ),
    "strict-block": GateProfile(
        name="strict-block",
        min_bots=None,
        required_scenario="block",
        required_gamemode="creative",
        min_view_distance=32,
        min_simulation_distance=32,
        min_metrics_samples=6,
        min_tps1_avg=19.0,
        min_tps1_min=17.5,
        max_avg_tick_ms_avg=55.0,
        max_avg_tick_ms_max=125.0,
        max_rss_mib=28_672.0,
        min_loaded_chunks_per_bot=8.0,
        require_bukkit_connection_throttle_zero=True,
        require_block_actions=True,
    ),
    "stress-mixed": GateProfile(
        name="stress-mixed",
        min_bots=None,
        required_scenario="mixed",
        required_gamemode="survival",
        min_view_distance=16,
        min_simulation_distance=16,
        min_metrics_samples=6,
        min_tps1_avg=18.0,
        min_tps1_min=15.0,
        max_avg_tick_ms_avg=75.0,
        max_avg_tick_ms_max=150.0,
        max_rss_mib=28_672.0,
        min_loaded_chunks_per_bot=6.0,
        require_bukkit_connection_throttle_zero=True,
        require_block_actions=False,
        require_stress_corpus=True,
        min_stress_plugin_jars=20,
        min_stress_datapack_zips=10,
        min_mob_storm_spawned=100,
    ),
    "stress-mixed-gameplay": GateProfile(
        name="stress-mixed-gameplay",
        min_bots=None,
        required_scenario="mixed-gameplay",
        required_gamemode="creative",
        min_view_distance=16,
        min_simulation_distance=16,
        min_metrics_samples=6,
        min_tps1_avg=18.0,
        min_tps1_min=15.0,
        max_avg_tick_ms_avg=75.0,
        max_avg_tick_ms_max=150.0,
        max_rss_mib=28_672.0,
        min_loaded_chunks_per_bot=6.0,
        require_bukkit_connection_throttle_zero=True,
        require_block_actions=True,
        require_stress_corpus=True,
        min_stress_plugin_jars=20,
        min_stress_datapack_zips=10,
        min_mob_storm_spawned=100,
        require_mixed_gameplay_workload=True,
        required_action_start_mode="all-ready",
        min_action_gate_settle_ms=15000,
        require_action_gate_block_armed=True,
    ),
}


def parse_summary(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    in_bot_tail = False
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith("bot_log_tail:"):
                in_bot_tail = True
                continue
            if in_bot_tail and not line.startswith("early_abort_reason="):
                continue
            for key, value in SUMMARY_TOKEN_RE.findall(line):
                values[key] = value
    return values


def resolve_path(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = pathlib.Path(__file__).resolve().parents[1] / path
    return path


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def is_sha256(raw: str | None) -> bool:
    if raw is None or len(raw) != 64:
        return False
    return all(char in "0123456789abcdef" for char in raw)


def validate_stress_corpus_manifest(
    values: dict[str, str],
    failures: list[str],
) -> tuple[int, int]:
    manifest_raw = values.get("stress_corpus_manifest_path")
    expected_manifest_sha = values.get("stress_corpus_manifest_sha256")
    if not manifest_raw or manifest_raw == "none":
        failures.append("stress_corpus_manifest_path is missing")
        return 0, 0

    manifest = resolve_path(manifest_raw)
    if not manifest.is_file():
        failures.append(f"stress_corpus_manifest_path={manifest} is missing")
        return 0, 0

    if not is_sha256(expected_manifest_sha):
        failures.append("stress_corpus_manifest_sha256 is missing or not a lowercase sha256")
    else:
        observed_manifest_sha = sha256(manifest)
        if observed_manifest_sha != expected_manifest_sha:
            failures.append(
                "stress_corpus_manifest_sha256="
                f"{expected_manifest_sha} != actual {observed_manifest_sha}"
            )

    plugin_rows = 0
    datapack_rows = 0
    seen_relative_paths: set[str] = set()
    with manifest.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.rstrip("\n")
            if not line:
                continue
            if line_no == 1 and line == "kind\trelative_path\tsha256\tbytes":
                continue
            parts = line.split("\t")
            if len(parts) != 4:
                failures.append(f"stress_corpus_manifest:{line_no}: expected 4 tab-separated fields")
                continue
            kind, relative_path, expected_sha, expected_bytes_raw = parts
            if kind not in ("stress_plugin_jar", "stress_datapack_zip"):
                failures.append(f"stress_corpus_manifest:{line_no}: unknown kind {kind}")
                continue
            if relative_path in seen_relative_paths:
                failures.append(f"stress_corpus_manifest:{line_no}: duplicate relative_path {relative_path}")
                continue
            seen_relative_paths.add(relative_path)
            if relative_path.startswith("/") or ".." in pathlib.PurePosixPath(relative_path).parts:
                failures.append(f"stress_corpus_manifest:{line_no}: unsafe relative_path {relative_path}")
                continue
            artifact = manifest.parent / relative_path
            if not artifact.is_file():
                failures.append(f"stress_corpus_manifest:{line_no}: {artifact} is missing")
                continue
            if not is_sha256(expected_sha):
                failures.append(f"stress_corpus_manifest:{line_no}: invalid sha256 {expected_sha}")
            else:
                observed_sha = sha256(artifact)
                if observed_sha != expected_sha:
                    failures.append(
                        f"stress_corpus_manifest:{line_no}: {relative_path} "
                        f"sha256={observed_sha} expected={expected_sha}"
                    )
            try:
                expected_bytes = int(expected_bytes_raw)
            except ValueError:
                failures.append(f"stress_corpus_manifest:{line_no}: bytes is not numeric")
            else:
                observed_bytes = artifact.stat().st_size
                if observed_bytes != expected_bytes:
                    failures.append(
                        f"stress_corpus_manifest:{line_no}: {relative_path} "
                        f"bytes={observed_bytes} expected={expected_bytes}"
                    )
            if kind == "stress_plugin_jar":
                plugin_rows += 1
            else:
                datapack_rows += 1

    def compare_count(summary_key: str, manifest_count: int) -> None:
        raw = values.get(summary_key)
        if raw is None:
            failures.append(f"{summary_key} is missing, expected manifest count {manifest_count}")
            return
        try:
            observed = int(float(raw))
        except ValueError:
            failures.append(f"{summary_key}={raw} is not numeric")
            return
        if observed != manifest_count:
            failures.append(f"{summary_key}={observed} != manifest count {manifest_count}")

    compare_count("stress_plugin_jars", plugin_rows)
    compare_count("stress_datapack_zips", datapack_rows)
    return plugin_rows, datapack_rows


def make_reader(
    values: dict[str, str],
    failures: list[str],
) -> tuple[Callable[[str], str | None], Callable[[str], int | None], Callable[[str], float | None]]:
    parse_failures: set[str] = set()

    def raw(key: str) -> str | None:
        return values.get(key)

    def integer(key: str) -> int | None:
        value = raw(key)
        if value is None:
            return None
        try:
            return int(float(value))
        except ValueError:
            if key not in parse_failures:
                failures.append(f"{key}={value} is not numeric")
                parse_failures.add(key)
            return None

    def floating(key: str) -> float | None:
        value = raw(key)
        if value is None:
            return None
        try:
            return float(value)
        except ValueError:
            if key not in parse_failures:
                failures.append(f"{key}={value} is not numeric")
                parse_failures.add(key)
            return None

    return raw, integer, floating


def evaluate(
    values: dict[str, str],
    profile: GateProfile,
    *,
    min_bots_override: int | None,
    min_loaded_chunks_override: int | None,
    min_tps1_avg_override: float | None,
    min_tps1_min_override: float | None,
    max_avg_tick_ms_avg_override: float | None,
    max_avg_tick_ms_max_override: float | None,
    max_rss_mib_override: float | None,
) -> tuple[bool, list[str], dict[str, str]]:
    failures: list[str] = []
    raw, integer, floating = make_reader(values, failures)

    observed_bots = integer("bots")
    required_bots = min_bots_override if min_bots_override is not None else profile.min_bots
    if required_bots is None:
        required_bots = observed_bots
    if required_bots is None:
        failures.append("bots is missing")
        required_bots = 0

    min_tps1_avg = min_tps1_avg_override if min_tps1_avg_override is not None else profile.min_tps1_avg
    min_tps1_min = min_tps1_min_override if min_tps1_min_override is not None else profile.min_tps1_min
    max_avg_tick_ms_avg = (
        max_avg_tick_ms_avg_override
        if max_avg_tick_ms_avg_override is not None
        else profile.max_avg_tick_ms_avg
    )
    max_avg_tick_ms_max = (
        max_avg_tick_ms_max_override
        if max_avg_tick_ms_max_override is not None
        else profile.max_avg_tick_ms_max
    )
    max_rss_mib = max_rss_mib_override if max_rss_mib_override is not None else profile.max_rss_mib
    min_loaded_chunks = (
        min_loaded_chunks_override
        if min_loaded_chunks_override is not None
        else int(required_bots * profile.min_loaded_chunks_per_bot)
    )
    required_stress_plugin_jars = profile.min_stress_plugin_jars
    required_stress_datapack_zips = profile.min_stress_datapack_zips
    stress_required_source = "manifest" if profile.require_stress_corpus else "profile"

    checks = 0
    run_class = "failed"
    environment_invalid = False
    environment_invalid_kind = "none"
    environment_invalid_reason = raw("early_abort_reason") or ""

    if environment_invalid_reason.startswith("host_contention"):
        checks += 1
        run_class = "environment-invalid"
        environment_invalid = True
        environment_invalid_kind = "host_contention"
        failures.append(
            "environment_invalid=true; "
            f"kind={environment_invalid_kind}; early_abort_reason={environment_invalid_reason}"
        )

    if (raw("bot_action_gate_softened") or "").lower() == "true":
        checks += 1
        if not environment_invalid:
            run_class = "softened-invalid"
        reason = raw("bot_action_gate_softened_reason") or "unknown"
        original_required = raw("bot_action_gate_softened_original_required") or "unknown"
        live_required = raw("bot_action_gate_softened_live_required") or "unknown"
        failures.append(
            "bot_action_gate_softened=true; "
            f"reason={reason}; originalRequired={original_required}; liveRequired={live_required}"
        )

    def require_str(key: str, expected: str) -> None:
        nonlocal checks
        checks += 1
        value = raw(key)
        if value is None:
            failures.append(f"{key} is missing, expected {expected}")
        elif value != expected:
            failures.append(f"{key}={value} != required {expected}")

    def require_bool_str(key: str, expected: bool) -> None:
        nonlocal checks
        checks += 1
        value = raw(key)
        expected_value = str(expected).lower()
        if value is None:
            failures.append(f"{key} is missing, expected {expected_value}")
        elif value.lower() != expected_value:
            failures.append(f"{key}={value} != required {expected_value}")

    def require_int_at_least(key: str, minimum: int) -> None:
        nonlocal checks
        checks += 1
        value = integer(key)
        if value is None:
            failures.append(f"{key} is missing, required >= {minimum}")
        elif value < minimum:
            failures.append(f"{key}={value} < required {minimum}")

    def require_float_at_least(key: str, minimum: float) -> None:
        nonlocal checks
        checks += 1
        value = floating(key)
        if value is None:
            failures.append(f"{key} is missing, required >= {minimum:.2f}")
        elif value < minimum:
            failures.append(f"{key}={value:.2f} < required {minimum:.2f}")

    def require_int_at_most(key: str, maximum: int) -> None:
        nonlocal checks
        checks += 1
        value = integer(key)
        if value is None:
            failures.append(f"{key} is missing, required <= {maximum}")
        elif value > maximum:
            failures.append(f"{key}={value} > allowed {maximum}")

    def require_float_at_most(key: str, maximum: float) -> None:
        nonlocal checks
        checks += 1
        value = floating(key)
        if value is None:
            failures.append(f"{key} is missing, required <= {maximum:.2f}")
        elif value > maximum:
            failures.append(f"{key}={value:.2f} > allowed {maximum:.2f}")

    def metric_key(key: str) -> str:
        load_window_key = f"load_window_{key}"
        if raw("load_window_policy") is not None:
            return load_window_key
        if load_window_key in values:
            return load_window_key
        return key

    def require_metric_int_at_least(key: str, minimum: int) -> None:
        require_int_at_least(metric_key(key), minimum)

    def require_metric_float_at_least(key: str, minimum: float) -> None:
        require_float_at_least(metric_key(key), minimum)

    def require_metric_float_at_most(key: str, maximum: float) -> None:
        require_float_at_most(metric_key(key), maximum)

    def require_optional_float_at_most(key: str, maximum: float) -> None:
        nonlocal checks
        checks += 1
        value = floating(key)
        if value is None:
            if profile.require_host_metrics:
                failures.append(f"{key} is missing, required <= {maximum:.2f}")
            return
        if value > maximum:
            failures.append(f"{key}={value:.2f} > allowed {maximum:.2f}")

    def require_optional_int_at_least(key: str, minimum: int) -> None:
        nonlocal checks
        checks += 1
        value = integer(key)
        if value is None:
            if profile.require_host_metrics:
                failures.append(f"{key} is missing, required >= {minimum}")
            return
        if value < minimum:
            failures.append(f"{key}={value} < required {minimum}")

    require_int_at_least("bots", required_bots)
    require_str("load_test_scenario", profile.required_scenario)
    require_str("load_test_gamemode", profile.required_gamemode)
    require_int_at_least("view_distance", profile.min_view_distance)
    require_int_at_least("simulation_distance", profile.min_simulation_distance)
    require_int_at_most("bot_exit", 0)
    require_metric_int_at_least("metrics_samples", profile.min_metrics_samples)
    if profile.require_bukkit_connection_throttle_zero:
        require_int_at_most("bukkit_connection_throttle", 0)
    if profile.require_warm_world_source:
        require_bool_str("world_warm_source_present", True)
        require_str("world_mode", "warm-source")
    if profile.reject_warm_world_source and raw("world_warm_source_present") is not None:
        require_bool_str("world_warm_source_present", False)
    if profile.reject_spark_background_profiler and raw("spark_background_profiler") is not None:
        require_bool_str("spark_background_profiler", False)
    if profile.require_stress_corpus:
        require_bool_str("stress_corpus", True)
        validate_stress_corpus_manifest(values, failures)
        require_int_at_least("stress_plugin_jars", required_stress_plugin_jars)
        require_int_at_least("stress_datapack_zips", required_stress_datapack_zips)
    if profile.min_mob_storm_spawned > 0:
        require_int_at_least("mob_storm_requested", profile.min_mob_storm_spawned)
        require_int_at_least("compat_probe_mobstorm_spawned_max", profile.min_mob_storm_spawned)
        require_int_at_least("compat_probe_mobstorm_spawned_total", profile.min_mob_storm_spawned)
        require_int_at_least("compat_probe_living_entities_max", profile.min_mob_storm_spawned)
    if raw("load_window_policy") is not None:
        require_bool_str("load_window_reached_full_online", True)

    require_metric_int_at_least("online_max", required_bots)
    require_int_at_least("bot_created_max", required_bots)
    require_int_at_least("bot_connected_max", required_bots)
    require_int_at_least("bot_ready_max", required_bots)
    require_int_at_least("bot_active_max", required_bots)
    require_int_at_most("bot_kicked_max", 0)
    require_int_at_most("bot_errors_max", 0)
    require_int_at_least("server_join_events", required_bots)
    require_int_at_least("server_quit_events", required_bots)

    require_metric_float_at_least("tps1_avg", min_tps1_avg)
    require_metric_float_at_least("tps1_min", min_tps1_min)
    require_metric_float_at_most("avg_tick_ms_avg", max_avg_tick_ms_avg)
    require_metric_float_at_most("avg_tick_ms_max", max_avg_tick_ms_max)
    require_float_at_most("process_rss_mib_max", max_rss_mib)
    require_metric_int_at_least("loaded_chunks_max", min_loaded_chunks)
    require_optional_int_at_least("host_cpu_windows", 1)
    require_optional_float_at_most("host_system_load1_per_cpu_max", profile.max_host_load_per_cpu)
    require_optional_float_at_most("host_cpu_steal_percent_max", profile.max_host_steal_percent)
    require_optional_float_at_most("host_cpu_iowait_percent_max", profile.max_host_iowait_percent)

    if profile.require_block_actions:
        if profile.required_action_start_mode is not None:
            require_str("bot_action_start_mode", profile.required_action_start_mode)
            require_str("bot_action_gate_open_mode", profile.required_action_start_mode)
        if profile.min_action_gate_settle_ms > 0:
            require_int_at_least("bot_action_ready_settle_ms", profile.min_action_gate_settle_ms)
        if profile.require_action_gate_block_armed:
            require_bool_str("bot_action_ready_requires_block_armed", True)
        if profile.required_action_start_mode is not None or profile.min_action_gate_settle_ms > 0:
            require_bool_str("bot_action_gate_opened", True)
            require_int_at_least("bot_action_ready_min_count", required_bots)
            require_float_at_least("bot_action_ready_min_fraction", 1.0)
            require_int_at_least("bot_action_gate_open_ready", required_bots)
            require_int_at_least("bot_action_gate_open_active", required_bots)
            require_int_at_least("bot_action_gate_open_settled", required_bots)
            require_int_at_least("bot_action_gate_open_required", required_bots)
        if profile.require_action_gate_block_armed:
            require_int_at_least("bot_action_gate_open_block_armed", required_bots)
        require_int_at_least("bot_block_armed_max", required_bots)
        require_int_at_least("bot_block_primed_max", required_bots)
        require_int_at_least("bot_block_creative_slot_packets_max", required_bots)
        require_int_at_least("bot_block_place_packets_max", required_bots)
        require_int_at_least("bot_block_dig_packets_max", required_bots)
        require_int_at_most("bot_block_action_errors_max", 0)
        require_int_at_least("compat_probe_arena_prepared_max", required_bots)
        require_bool_str("compat_probe_block_evidence_accepted", True)
        require_int_at_least("compat_probe_direct_block_loadbot_event_lines", 1)
        require_int_at_least("compat_probe_direct_block_loadbot_place_event_lines", 1)
        require_int_at_least("compat_probe_direct_block_loadbot_break_event_lines", 1)
        require_int_at_least("compat_probe_direct_block_loadbot_cancelled_false_lines", 1)
        require_int_at_least("compat_probe_direct_block_loadbot_players", 1)
    if profile.min_server_block_places > 0:
        require_int_at_least("compat_probe_block_places_max", profile.min_server_block_places)
    if profile.min_server_block_breaks > 0:
        require_int_at_least("compat_probe_block_breaks_max", profile.min_server_block_breaks)

    if profile.require_mixed_gameplay_workload:
        mixed_packet_min = max(1, required_bots * profile.min_mixed_packets_per_bot)
        if raw("bot_action_start_mode") is not None and raw("bot_action_start_mode") != "timer":
            require_bool_str("bot_action_gate_opened", True)
            require_int_at_least("bot_action_gate_open_active", required_bots)
            require_int_at_least("bot_action_gate_open_settled", required_bots)
        require_int_at_least("bot_mixed_action_ticks_max", 1)
        require_int_at_least("bot_mixed_held_item_packets_max", mixed_packet_min)
        require_int_at_least("bot_mixed_arm_animation_packets_max", mixed_packet_min)
        require_int_at_least("bot_mixed_player_input_packets_max", mixed_packet_min)
        require_int_at_least("bot_mixed_use_item_packets_max", mixed_packet_min)
        require_int_at_least("bot_mixed_command_packets_max", required_bots)
        require_int_at_least("bot_mixed_block_place_packets_max", required_bots)
        require_int_at_least("bot_mixed_block_dig_packets_max", required_bots)
        require_int_at_most("bot_mixed_action_errors_max", 0)
        require_int_at_least("compat_probe_commands_max", 1)

    require_int_at_most("moved_too_quickly_warnings", 0)
    require_int_at_most("watchdog_thread_dumps", 0)
    require_int_at_most("sync_load_stack_hits", 0)
    require_int_at_most("nearby_players_stack_hits", 0)
    require_int_at_most("thread_check_failures", 0)
    require_int_at_most("chunk_system_errors", 0)
    require_int_at_most("feature_placement_errors", 0)
    require_int_at_most("off_main_poi_hits", 0)
    require_int_at_most("stability_failures", 0)
    require_int_at_most("external_thread_prints", 0)

    observed_keys = [
        "bots",
        "view_distance",
        "simulation_distance",
        "load_test_scenario",
        "load_test_gamemode",
        "early_abort_reason",
        "bot_speed_blocks_per_second",
        "bot_move_interval_ms",
        "bot_send_stationary_positions",
        "bot_mixed_use_entity_attacks",
        "bot_action_start_mode",
        "bot_action_start_after_ms",
        "bot_action_ready_settle_ms",
        "bot_action_ready_requires_block_armed",
        "bot_action_ready_min_count",
        "bot_action_ready_min_fraction",
        "bot_check_timeout_interval_ms",
        "bot_action_gate_ready_events",
        "bot_action_gate_reset_events",
        "bot_action_gate_softened",
        "bot_action_gate_softened_events",
        "bot_action_gate_softened_reason",
        "bot_action_gate_softened_original_required",
        "bot_action_gate_softened_live_required",
        "bot_action_gate_softened_missing",
        "bot_action_gate_softened_active",
        "bot_action_gate_softened_settled",
        "bot_action_gate_softened_block_armed",
        "bot_action_gate_opened",
        "bot_action_gate_open_mode",
        "bot_action_gate_opened_after_ms",
        "bot_action_gate_open_ready",
        "bot_action_gate_open_active",
        "bot_action_gate_open_settled",
        "bot_action_gate_open_required",
        "bot_action_gate_open_block_armed",
        "load_window_start_mode",
        "bot_ramp_seconds",
        "world_mode",
        "claim_surface",
        "world_warm_source_present",
        "world_warm_source",
        "spark_background_profiler",
        "stress_corpus",
        "stress_plugins_enabled",
        "stress_datapacks_enabled",
        "stress_corpus_manifest_path",
        "stress_corpus_manifest_sha256",
        "plugin_jars_total",
        "datapack_zips_total",
        "stress_plugin_jars",
        "stress_datapack_zips",
        "mob_storm_requested",
        "compat_probe_mobstorm_spawned_max",
        "compat_probe_mobstorm_spawned_total",
        "compat_probe_living_entities_max",
        "load_window_policy",
        "load_window_reached_full_online",
        "load_window_ended_by_online_drop",
        "load_window_metrics_samples",
        "load_window_online_max",
        "load_window_loaded_chunks_max",
        "load_window_tps1_avg",
        "load_window_tps1_min",
        "load_window_avg_tick_ms_avg",
        "load_window_avg_tick_ms_max",
        "teardown_metrics_samples",
        "teardown_online_max",
        "teardown_loaded_chunks_max",
        "teardown_tps1_avg",
        "teardown_tps1_min",
        "teardown_avg_tick_ms_avg",
        "teardown_avg_tick_ms_max",
        "online_max",
        "bot_created_max",
        "bot_connected_max",
        "bot_ready_max",
        "bot_active_max",
        "bot_kicked_max",
        "bot_errors_max",
        "bot_block_armed_max",
        "bot_block_primed_max",
        "bot_block_creative_slot_packets_max",
        "bot_block_place_packets_max",
        "bot_block_dig_packets_max",
        "bot_block_action_errors_max",
        "compat_probe_block_evidence_accepted",
        "compat_probe_block_metrics_loadbot_direct_evidence",
        "bot_mixed_action_ticks_max",
        "bot_mixed_held_item_packets_max",
        "bot_mixed_arm_animation_packets_max",
        "bot_mixed_player_input_packets_max",
        "bot_mixed_use_item_packets_max",
        "bot_mixed_command_packets_max",
        "bot_mixed_block_place_packets_max",
        "bot_mixed_block_dig_packets_max",
        "bot_mixed_attack_packets_max",
        "bot_mixed_action_errors_max",
        "compat_probe_arena_commands_max",
        "compat_probe_arena_prepared_max",
        "compat_probe_arena_skipped_max",
        "compat_probe_block_places_max",
        "compat_probe_block_breaks_max",
        "compat_probe_block_event_loadbot_places_max",
        "compat_probe_block_event_loadbot_breaks_max",
        "compat_probe_commands_max",
        "compat_probe_player_commands_max",
        "compat_probe_item_held_events_max",
        "compat_probe_animation_events_max",
        "compat_probe_interact_events_max",
        "compat_probe_entity_damage_events_max",
        "compat_probe_direct_block_loadbot_event_lines",
        "compat_probe_direct_block_loadbot_place_event_lines",
        "compat_probe_direct_block_loadbot_break_event_lines",
        "compat_probe_direct_block_loadbot_cancelled_true_lines",
        "compat_probe_direct_block_loadbot_cancelled_false_lines",
        "compat_probe_direct_block_loadbot_players",
        "server_join_events",
        "server_quit_events",
        "bot_position_packets_max",
        "bot_positions_per_sec_max",
        "bot_chunk_packets_window_max",
        "bot_chunks_per_sec_max",
        "loaded_chunks_max",
        "tps1_avg",
        "tps1_min",
        "avg_tick_ms_avg",
        "avg_tick_ms_max",
        "process_rss_mib_max",
        "host_cpu_windows",
        "host_cpu_idle_percent_min",
        "host_cpu_iowait_percent_max",
        "host_cpu_iowait_percent_avg",
        "host_cpu_steal_percent_max",
        "host_cpu_steal_percent_avg",
        "moved_too_quickly_warnings",
        "watchdog_thread_dumps",
        "sync_load_stack_hits",
        "nearby_players_stack_hits",
        "stability_failures",
    ]
    metadata = {
        "requirement_count": str(checks),
        "required_bots": str(required_bots),
        "required_loaded_chunks_min": str(min_loaded_chunks),
        "required_tps1_avg_min": f"{min_tps1_avg:.2f}",
        "required_tps1_min_min": f"{min_tps1_min:.2f}",
        "required_avg_tick_ms_avg_max": f"{max_avg_tick_ms_avg:.2f}",
        "required_avg_tick_ms_max_max": f"{max_avg_tick_ms_max:.2f}",
        "required_process_rss_mib_max": f"{max_rss_mib:.1f}",
        "required_host_load_per_cpu_max": f"{profile.max_host_load_per_cpu:.3f}",
        "required_host_steal_percent_max": f"{profile.max_host_steal_percent:.2f}",
        "required_host_iowait_percent_max": f"{profile.max_host_iowait_percent:.2f}",
        "requires_host_metrics": str(profile.require_host_metrics).lower(),
        "requires_warm_world_source": str(profile.require_warm_world_source).lower(),
        "rejects_warm_world_source": str(profile.reject_warm_world_source).lower(),
        "rejects_spark_background_profiler": str(profile.reject_spark_background_profiler).lower(),
        "requires_stress_corpus": str(profile.require_stress_corpus).lower(),
        "required_stress_plugin_jars_min": str(required_stress_plugin_jars),
        "required_stress_datapack_zips_min": str(required_stress_datapack_zips),
        "required_stress_corpus_source": stress_required_source,
        "requires_server_block_workload": str(
            profile.min_server_block_places > 0 or profile.min_server_block_breaks > 0
        ).lower(),
        "required_compat_probe_block_places_min": str(profile.min_server_block_places),
        "required_compat_probe_block_breaks_min": str(profile.min_server_block_breaks),
        "required_mob_storm_spawned_min": str(profile.min_mob_storm_spawned),
        "requires_mixed_gameplay_workload": str(profile.require_mixed_gameplay_workload).lower(),
        "required_mixed_packets_per_bot_min": str(profile.min_mixed_packets_per_bot),
        "required_action_start_mode": str(profile.required_action_start_mode or ""),
        "required_action_gate_settle_ms_min": str(profile.min_action_gate_settle_ms),
        "requires_action_gate_block_armed": str(profile.require_action_gate_block_armed).lower(),
        "environment_invalid": str(environment_invalid).lower(),
        "environment_invalid_kind": environment_invalid_kind,
    }
    if environment_invalid_reason:
        metadata["environment_invalid_reason"] = environment_invalid_reason
    metadata["run_class"] = "passed" if not failures else run_class
    for key in observed_keys:
        if key in values:
            metadata[f"observed_{key}"] = values[key]

    return not failures, failures, metadata


def build_report(
    *,
    summary: pathlib.Path,
    profile: GateProfile,
    passed: bool,
    failures: list[str],
    metadata: dict[str, str],
) -> str:
    lines = [
        f"gate_profile={profile.name}",
        f"summary_path={summary}",
        f"claim_eligible={str(passed).lower()}",
        f"gate_pass={str(passed).lower()}",
        f"failure_count={len(failures)}",
    ]
    for key, value in metadata.items():
        lines.append(f"{key}={value}")
    for failure in failures:
        lines.append(f"failure={failure}")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("summary", type=pathlib.Path, help="run_load_test summary file")
    parser.add_argument("--profile", choices=sorted(PROFILES), default="production-500")
    parser.add_argument("--report", type=pathlib.Path, help="optional gate report output path")
    parser.add_argument("--min-bots", type=int)
    parser.add_argument("--min-loaded-chunks", type=int)
    parser.add_argument("--min-tps1-avg", type=float)
    parser.add_argument("--min-tps1-min", type=float)
    parser.add_argument("--max-avg-tick-ms-avg", type=float)
    parser.add_argument("--max-avg-tick-ms-max", type=float)
    parser.add_argument("--max-rss-mib", type=float)
    args = parser.parse_args()

    if not args.summary.is_file():
        print(f"Missing summary file: {args.summary}", file=sys.stderr)
        return 66

    profile = PROFILES[args.profile]
    values = parse_summary(args.summary)
    passed, failures, metadata = evaluate(
        values,
        profile,
        min_bots_override=args.min_bots,
        min_loaded_chunks_override=args.min_loaded_chunks,
        min_tps1_avg_override=args.min_tps1_avg,
        min_tps1_min_override=args.min_tps1_min,
        max_avg_tick_ms_avg_override=args.max_avg_tick_ms_avg,
        max_avg_tick_ms_max_override=args.max_avg_tick_ms_max,
        max_rss_mib_override=args.max_rss_mib,
    )
    report = build_report(
        summary=args.summary,
        profile=profile,
        passed=passed,
        failures=failures,
        metadata=metadata,
    )
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(report, encoding="utf-8")
    print(report, end="")
    if not passed:
        print(
            f"Load claim gate failed for profile {profile.name}: {len(failures)} failure(s).",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
