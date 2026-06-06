#!/usr/bin/env python3
"""Compare two run_load_test summary files."""

from __future__ import annotations

import argparse
import pathlib
import re
from collections.abc import Iterable


TOKEN_RE = re.compile(r"([A-Za-z0-9_]+)=([^\s]+)")

DEFAULT_KEYS = (
    "bots",
    "view_distance",
    "simulation_distance",
    "world_mode",
    "bot_speed_blocks_per_second",
    "bot_move_interval_ms",
    "bot_send_stationary_positions",
    "bot_action_start_after_ms",
    "bot_ramp_seconds",
    "stress_corpus",
    "stress_plugins_enabled",
    "stress_datapacks_enabled",
    "plugin_jars_total",
    "datapack_zips_total",
    "stress_plugin_jars",
    "stress_datapack_zips",
    "mob_storm_requested",
    "load_window_reached_full_online",
    "load_window_online_max",
    "bot_connected_max",
    "bot_ready_max",
    "bot_active_max",
    "bot_position_packets_max",
    "bot_positions_per_sec_max",
    "bot_chunk_packets_window_max",
    "bot_chunks_per_sec_max",
    "compat_probe_send_pressure_samples",
    "compat_probe_send_pressure_players_max",
    "compat_probe_send_pressure_connections_max",
    "compat_probe_send_pressure_chunk_senders_max",
    "compat_probe_send_pending_actions_max",
    "compat_probe_send_pending_outbound_bytes_max",
    "compat_probe_send_bytes_before_writable_max",
    "compat_probe_send_bytes_before_unwritable_min",
    "compat_probe_send_non_writable_connections_max",
    "compat_probe_chunk_send_pending_chunks_max",
    "compat_probe_chunk_send_unacknowledged_batches_max",
    "compat_probe_chunk_send_batch_quota_max",
    "compat_probe_chunk_send_desired_chunks_per_tick_max",
    "compat_probe_chunk_send_max_unacknowledged_batches_max",
    "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak",
    "compat_probe_chunk_send_channel_not_writable_skips_max",
    "compat_probe_chunk_send_near_unwritable_pending_chunks_peak",
    "compat_probe_chunk_send_near_unwritable_skips_max",
    "load_window_loaded_chunks_max",
    "load_window_tps1_avg",
    "load_window_tps1_min",
    "load_window_avg_tick_ms_avg",
    "load_window_avg_tick_ms_max",
    "process_rss_mib_max",
    "watchdog_thread_dumps",
    "sync_load_stack_hits",
    "nearby_players_stack_hits",
    "stability_failures",
    "moved_too_quickly_warnings",
)

LOWER_IS_BETTER = {
    "load_window_avg_tick_ms_avg",
    "load_window_avg_tick_ms_max",
    "process_rss_mib_max",
    "watchdog_thread_dumps",
    "sync_load_stack_hits",
    "nearby_players_stack_hits",
    "stability_failures",
    "moved_too_quickly_warnings",
    "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak",
    "compat_probe_chunk_send_channel_not_writable_skips_max",
    "compat_probe_chunk_send_near_unwritable_pending_chunks_peak",
    "compat_probe_chunk_send_near_unwritable_skips_max",
}

HIGHER_IS_BETTER = {
    "load_window_online_max",
    "bot_connected_max",
    "bot_ready_max",
    "bot_active_max",
    "load_window_loaded_chunks_max",
    "load_window_tps1_avg",
    "load_window_tps1_min",
}


def parse_summary(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            if raw.startswith("bot_log_tail:"):
                break
            for key, value in TOKEN_RE.findall(raw):
                values[key] = value
    return values


def as_float(value: str | None) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except ValueError:
        return None


def compare_key(key: str, left: dict[str, str], right: dict[str, str]) -> str:
    left_value = left.get(key, "missing")
    right_value = right.get(key, "missing")
    left_float = as_float(left.get(key))
    right_float = as_float(right.get(key))
    if left_float is None or right_float is None:
        return f"{key}: left={left_value} right={right_value}"

    delta = right_float - left_float
    if key in HIGHER_IS_BETTER:
        verdict = "better" if delta > 0 else "worse" if delta < 0 else "same"
    elif key in LOWER_IS_BETTER:
        verdict = "better" if delta < 0 else "worse" if delta > 0 else "same"
    else:
        verdict = "delta"
    return (
        f"{key}: left={left_value} right={right_value} "
        f"delta={delta:.3f} verdict={verdict}"
    )


def iter_keys(extra: Iterable[str]) -> list[str]:
    keys = list(DEFAULT_KEYS)
    for key in extra:
        if key not in keys:
            keys.append(key)
    return keys


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", required=True, type=pathlib.Path)
    parser.add_argument("--right", required=True, type=pathlib.Path)
    parser.add_argument("--left-label", default="left")
    parser.add_argument("--right-label", default="right")
    parser.add_argument("--key", action="append", default=[])
    parser.add_argument("--report", type=pathlib.Path)
    args = parser.parse_args()

    if not args.left.is_file():
        raise SystemExit(f"Missing left summary: {args.left}")
    if not args.right.is_file():
        raise SystemExit(f"Missing right summary: {args.right}")

    left = parse_summary(args.left)
    right = parse_summary(args.right)
    lines = [
        "load_summary_comparison=true",
        f"left_label={args.left_label}",
        f"left_summary={args.left}",
        f"right_label={args.right_label}",
        f"right_summary={args.right}",
        "",
    ]
    lines.extend(compare_key(key, left, right) for key in iter_keys(args.key))
    report = "\n".join(lines) + "\n"

    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(report, encoding="utf-8")
    print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
