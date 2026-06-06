#!/usr/bin/env python3
"""Report focused P500 metric deltas from two load summary files."""

from __future__ import annotations

import argparse
import pathlib
import re


TOKEN_RE = re.compile(r"([A-Za-z0-9_]+)=([^ \t\r\n]+)")

GROUPS: tuple[tuple[str, tuple[str, ...]], ...] = (
    (
        "core",
        (
            "online_max",
            "load_window_online_max",
            "load_window_tps1_avg",
            "load_window_tps1_min",
            "avg_tick_ms_avg",
            "avg_tick_ms_max",
            "load_window_avg_tick_ms_avg",
            "load_window_avg_tick_ms_max",
            "loaded_chunks_max",
            "load_window_loaded_chunks_max",
        ),
    ),
    (
        "watchdog",
        (
            "watchdog_thread_dumps",
            "external_thread_prints",
            "diagnostic_thread_samples",
            "sync_load_stack_hits",
            "nearby_players_stack_hits",
            "stability_failures",
        ),
    ),
    (
        "host_cpu",
        (
            "host_cpu_iowait_percent_max",
            "host_cpu_iowait_percent_avg",
            "host_cpu_steal_percent_max",
            "host_cpu_steal_percent_avg",
        ),
    ),
    (
        "send_pressure",
        (
            "compat_probe_send_pressure_samples",
            "compat_probe_send_pressure_players_max",
            "compat_probe_send_pressure_connections_max",
            "compat_probe_send_pressure_chunk_senders_max",
            "compat_probe_send_pending_actions_max",
            "compat_probe_send_pending_outbound_bytes_max",
            "compat_probe_send_bytes_before_writable_max",
            "compat_probe_send_bytes_before_unwritable_min",
            "compat_probe_send_non_writable_connections_max",
        ),
    ),
    (
        "chunk_sender_pressure",
        (
            "compat_probe_chunk_send_pending_chunks_max",
            "compat_probe_chunk_send_unacknowledged_batches_max",
            "compat_probe_chunk_send_batch_quota_max",
            "compat_probe_chunk_send_desired_chunks_per_tick_max",
            "compat_probe_chunk_send_max_unacknowledged_batches_max",
            "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak",
            "compat_probe_chunk_send_channel_not_writable_skips_max",
            "compat_probe_chunk_send_near_unwritable_pending_chunks_peak",
            "compat_probe_chunk_send_near_unwritable_skips_max",
        ),
    ),
    (
        "bot_rss_elu",
        (
            "bot_child_process_rss_kb_total",
            "bot_child_process_rss_kb_max",
            "bot_rss_mib_current",
            "bot_rss_mib_max",
            "bot_rss_mib_aggregate_current",
            "bot_rss_mib_aggregate_max",
            "bot_loadgen_elu_pct_max",
            "bot_loadgen_elu_pct_avg",
        ),
    ),
)


def parse_summary(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw_line in handle:
            if raw_line.startswith("bot_log_tail:"):
                break
            for key, value in TOKEN_RE.findall(raw_line):
                values[key] = value
    return values


def as_float(value: str | None) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except ValueError:
        return None


def format_delta(left_value: str | None, right_value: str | None) -> str:
    left_float = as_float(left_value)
    right_float = as_float(right_value)
    if left_float is None or right_float is None:
        return "n/a"
    return f"{right_float - left_float:+.3f}"


def report_lines(
    left_path: pathlib.Path,
    right_path: pathlib.Path,
    left_values: dict[str, str],
    right_values: dict[str, str],
) -> list[str]:
    lines = [
        "p500_metric_delta_report=true",
        f"left_summary={left_path}",
        f"right_summary={right_path}",
        "delta=right-left",
        "",
    ]

    for group, keys in GROUPS:
        lines.append(f"[{group}]")
        width = max(len(key) for key in keys)
        for key in keys:
            left = left_values.get(key, "missing")
            right = right_values.get(key, "missing")
            delta = format_delta(left_values.get(key), right_values.get(key))
            lines.append(f"{key:<{width}} left={left} right={right} delta={delta}")
        lines.append("")

    if lines[-1] == "":
        lines.pop()
    return lines


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("left", type=pathlib.Path)
    parser.add_argument("right", type=pathlib.Path)
    args = parser.parse_args()

    if not args.left.is_file():
        raise SystemExit(f"missing left summary: {args.left}")
    if not args.right.is_file():
        raise SystemExit(f"missing right summary: {args.right}")

    left_values = parse_summary(args.left)
    right_values = parse_summary(args.right)
    print("\n".join(report_lines(args.left, args.right, left_values, right_values)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
