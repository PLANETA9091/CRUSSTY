#!/usr/bin/env python3
"""Analyze a failed load-test run into a compact breakdown."""

from __future__ import annotations

import argparse
import csv
import pathlib
import re
import sys
from collections import Counter
from dataclasses import dataclass


TOKEN_RE = re.compile(r"\b([A-Za-z0-9_]+)=([^\s]*)")


@dataclass(frozen=True)
class Series:
    count: int | None
    minimum: float | None
    average: float | None
    maximum: float | None
    source: str


@dataclass(frozen=True)
class Inputs:
    run_id: str
    summary: pathlib.Path | None
    gate: pathlib.Path | None
    log: pathlib.Path | None
    bot_log: pathlib.Path | None
    resources: pathlib.Path | None
    preflight: pathlib.Path | None


def parse_tokens(line: str) -> dict[str, str]:
    return dict(TOKEN_RE.findall(line))


def parse_kv_file(path: pathlib.Path | None, *, stop_at_bot_tail: bool = False) -> dict[str, str]:
    values: dict[str, str] = {}
    if path is None or not path.is_file():
        return values
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if stop_at_bot_tail and line.startswith("bot_log_tail:"):
                break
            for key, value in TOKEN_RE.findall(line):
                values[key] = value
    return values


def parse_gate(path: pathlib.Path | None) -> tuple[dict[str, str], list[str]]:
    values: dict[str, str] = {}
    failures: list[str] = []
    if path is None or not path.is_file():
        return values, failures
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line:
                continue
            if line.startswith("failure="):
                failures.append(line.removeprefix("failure="))
            elif "=" in line:
                key, value = line.split("=", 1)
                values[key] = value
    return values, failures


def to_float(value: str | None) -> float | None:
    if value is None or value == "":
        return None
    try:
        return float(value)
    except ValueError:
        return None


def to_int(value: str | None) -> int | None:
    parsed = to_float(value)
    if parsed is None:
        return None
    return int(parsed)


def first_int(*values: int | str | None) -> int | None:
    for value in values:
        if isinstance(value, int):
            return value
        parsed = to_int(value)
        if parsed is not None:
            return parsed
    return None


def first_float(*values: float | str | None) -> float | None:
    for value in values:
        if isinstance(value, float):
            return value
        parsed = to_float(value)
        if parsed is not None:
            return parsed
    return None


def stats(values: list[float], source: str) -> Series | None:
    if not values:
        return None
    return Series(
        count=len(values),
        minimum=min(values),
        average=sum(values) / len(values),
        maximum=max(values),
        source=source,
    )


def series_from_summary(
    values: dict[str, str],
    *,
    minimum_key: str,
    average_key: str,
    maximum_key: str,
    count_key: str,
    source: str,
) -> Series | None:
    count = to_int(values.get(count_key))
    minimum = to_float(values.get(minimum_key))
    average = to_float(values.get(average_key))
    maximum = to_float(values.get(maximum_key))
    if count is None and minimum is None and average is None and maximum is None:
        return None
    return Series(count=count, minimum=minimum, average=average, maximum=maximum, source=source)


def max_avg_series_from_summary(values: dict[str, str], *, prefix: str, count_key: str, source: str) -> Series | None:
    count = to_int(values.get(count_key))
    average = to_float(values.get(f"{prefix}_avg"))
    maximum = to_float(values.get(f"{prefix}_max"))
    if count is None and average is None and maximum is None:
        return None
    return Series(count=count, minimum=None, average=average, maximum=maximum, source=source)


def fmt_value(value: float | int | None, digits: int = 2) -> str:
    if value is None:
        return "n/a"
    if isinstance(value, int):
        return str(value)
    return f"{value:.{digits}f}"


def fmt_series(series: Series | None, *, digits: int = 2) -> str:
    if series is None:
        return "samples=n/a min=n/a avg=n/a max=n/a source=missing"
    count = "n/a" if series.count is None else str(series.count)
    return (
        f"samples={count} min={fmt_value(series.minimum, digits)} "
        f"avg={fmt_value(series.average, digits)} max={fmt_value(series.maximum, digits)} "
        f"source={series.source}"
    )


def parse_server_log(path: pathlib.Path | None) -> dict[str, object]:
    tps1: list[float] = []
    avg_tick_ms: list[float] = []
    online: list[float] = []
    loaded_chunks: list[float] = []
    disconnects: Counter[str] = Counter()
    disconnect_bot_names: set[str] = set()
    disconnect_line_count = 0
    error_lines = 0

    if path is None or not path.is_file():
        return {
            "tps1": None,
            "avg_tick_ms": None,
            "online_max": None,
            "loaded_chunks_max": None,
            "disconnects": disconnects,
            "disconnect_unique_bots": 0,
            "disconnect_line_count": 0,
            "error_lines": error_lines,
        }

    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if " ERROR]:" in line or " ERROR " in line:
                error_lines += 1
            if "COMPAT_PROBE metrics" in line:
                tokens = parse_tokens(line)
                for key, bucket in (
                    ("tps1", tps1),
                    ("avgTickMs", avg_tick_ms),
                    ("online", online),
                    ("loadedChunks", loaded_chunks),
                ):
                    value = to_float(tokens.get(key))
                    if value is not None:
                        bucket.append(value)
            if "lost connection:" in line:
                match = re.search(r"lost connection: (.+)$", line)
                if match:
                    disconnect_line_count += 1
                    disconnects[match.group(1).strip()] += 1
                    bot_match = re.search(r"\b(LoadBot\d+)\b", line)
                    if bot_match:
                        disconnect_bot_names.add(bot_match.group(1))
            elif "was kicked due to" in line:
                match = re.search(r"was kicked due to (.+)!$", line)
                if match:
                    disconnect_line_count += 1
                    disconnects[f"kicked:{match.group(1).strip()}"] += 1
                    bot_match = re.search(r"\b(LoadBot\d+)\b", line)
                    if bot_match:
                        disconnect_bot_names.add(bot_match.group(1))

    return {
        "tps1": stats(tps1, "server_log"),
        "avg_tick_ms": stats(avg_tick_ms, "server_log"),
        "online_max": int(max(online)) if online else None,
        "loaded_chunks_max": int(max(loaded_chunks)) if loaded_chunks else None,
        "disconnects": disconnects,
        "disconnect_unique_bots": len(disconnect_bot_names),
        "disconnect_line_count": disconnect_line_count,
        "error_lines": error_lines,
    }


def classify_kick_reason(raw_reason: str) -> str:
    if "disconnect.timeout" in raw_reason:
        return "disconnect.timeout"
    if raw_reason:
        return raw_reason
    return "unknown"


def parse_bot_log(path: pathlib.Path | None) -> dict[str, object]:
    error_counts: Counter[str] = Counter()
    kick_counts: Counter[str] = Counter()
    end_counts: Counter[str] = Counter()
    metrics_max: dict[str, int] = {}
    gate_ready_events = 0
    gate_open_events = 0

    if path is None or not path.is_file():
        return {
            "error_counts": error_counts,
            "kick_counts": kick_counts,
            "end_counts": end_counts,
            "metrics_max": metrics_max,
            "gate_ready_events": gate_ready_events,
            "gate_open_events": gate_open_events,
        }

    metric_keys = (
        "created",
        "connected",
        "ready",
        "active",
        "ended",
        "kicked",
        "errors",
        "actionGateReady",
        "actionGateActive",
        "actionGateSettled",
        "blockArmed",
        "blockPrimed",
        "blockCreativeSlotPackets",
        "blockPlacePackets",
        "blockDigPackets",
        "blockActionErrors",
    )

    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if "bot_error " in line:
                match = re.search(r"\berror=(.*)$", line)
                error_counts[(match.group(1).strip() if match else "unknown")] += 1
            elif "bot_kick " in line:
                match = re.search(r"\breason=(.*)$", line)
                kick_counts[classify_kick_reason(match.group(1).strip() if match else "")] += 1
            elif "bot_end " in line:
                reason = parse_tokens(line).get("reason", "unknown")
                end_counts[reason] += 1

            if "swarm_metrics" in line:
                tokens = parse_tokens(line)
                for key in metric_keys:
                    value = to_int(tokens.get(key))
                    if value is not None:
                        metrics_max[key] = max(metrics_max.get(key, value), value)
            elif "swarm_action_gate_ready" in line:
                gate_ready_events += 1
            elif "swarm_action_gate_open" in line:
                gate_open_events += 1

    return {
        "error_counts": error_counts,
        "kick_counts": kick_counts,
        "end_counts": end_counts,
        "metrics_max": metrics_max,
        "gate_ready_events": gate_ready_events,
        "gate_open_events": gate_open_events,
    }


def parse_resources(path: pathlib.Path | None) -> dict[str, object]:
    if path is None or not path.is_file():
        return {
            "windows": None,
            "steal": None,
            "iowait": None,
            "idle": None,
            "load1": None,
            "pid_cpu_max": None,
            "pid_rss_mib_max": None,
        }

    with path.open(encoding="utf-8", errors="replace", newline="") as handle:
        rows = list(csv.DictReader(handle))

    steal: list[float] = []
    iowait: list[float] = []
    idle: list[float] = []
    load1: list[float] = []
    pid_cpu: list[float] = []
    pid_rss_mib: list[float] = []

    for row in rows:
        load = to_float(row.get("system_load1"))
        if load is not None:
            load1.append(load)
        cpu = to_float(row.get("pid_cpu"))
        if cpu is not None:
            pid_cpu.append(cpu)
        rss_kb = to_float(row.get("pid_rss_kb"))
        if rss_kb is not None:
            pid_rss_mib.append(rss_kb / 1024.0)

    for previous, current in zip(rows, rows[1:]):
        total_before = to_float(previous.get("host_cpu_total"))
        total_after = to_float(current.get("host_cpu_total"))
        if total_before is None or total_after is None:
            continue
        total_delta = total_after - total_before
        if total_delta <= 0:
            continue
        for field, bucket in (
            ("host_cpu_steal", steal),
            ("host_cpu_iowait", iowait),
            ("host_cpu_idle", idle),
        ):
            before = to_float(previous.get(field))
            after = to_float(current.get(field))
            if before is not None and after is not None:
                bucket.append((after - before) / total_delta * 100.0)

    return {
        "windows": len(steal),
        "steal": stats(steal, "resources_csv"),
        "iowait": stats(iowait, "resources_csv"),
        "idle": stats(idle, "resources_csv"),
        "load1": stats(load1, "resources_csv"),
        "pid_cpu_max": max(pid_cpu) if pid_cpu else None,
        "pid_rss_mib_max": max(pid_rss_mib) if pid_rss_mib else None,
    }


def classify_gate_failure(failure: str) -> str:
    key = failure.split("=", 1)[0].split(" ", 1)[0]
    if key.startswith("environment_invalid"):
        return "environment"
    if key.startswith("load_window_"):
        return "load_window"
    if key.startswith("bot_"):
        return "bot_swarm"
    if key.startswith("server_"):
        return "server_presence"
    if key.startswith("compat_probe_"):
        return "compat_probe"
    if key.startswith(("tps", "avg_tick", "loaded_chunks")):
        return "performance"
    return "other"


def append_counter(lines: list[str], title: str, counts: Counter[str], *, limit: int = 8) -> None:
    lines.append(f"{title}:")
    if not counts:
        lines.append("- none")
        return
    for label, count in counts.most_common(limit):
        lines.append(f"- {label}: {count}")
    omitted = len(counts) - limit
    if omitted > 0:
        lines.append(f"- ... {omitted} more classes omitted")


def bool_is_false(value: str | None) -> bool:
    return value is not None and value.lower() == "false"


def bool_is_true(value: str | None) -> bool:
    return value is not None and value.lower() == "true"


def build_blockers(
    *,
    summary: dict[str, str],
    gate_values: dict[str, str],
    gate_failures: list[str],
    bot_data: dict[str, object],
    tps1: Series | None,
    avg_tick_ms: Series | None,
    cpu_data: dict[str, object],
    preflight: dict[str, str],
) -> list[str]:
    blockers: list[str] = []
    gate_pass_raw = gate_values.get("gate_pass")
    gate_failed = bool_is_false(gate_pass_raw) or bool(gate_failures)
    required_bots = first_int(gate_values.get("required_bots"), summary.get("bots"))

    if gate_failed:
        blockers.append(f"FAIL gate: {len(gate_failures)} gate failure(s)")
    else:
        blockers.append("PASS gate: no gate failures found")

    load_window_samples = first_int(summary.get("load_window_metrics_samples"), gate_values.get("observed_load_window_metrics_samples"))
    full_online = summary.get("load_window_reached_full_online", gate_values.get("observed_load_window_reached_full_online"))
    if load_window_samples == 0 or bool_is_false(full_online):
        blockers.append(
            "FAIL load_window: full-online window never opened "
            f"(reached_full_online={full_online or 'n/a'} samples={fmt_value(load_window_samples, 0)})"
        )

    bot_metrics = bot_data["metrics_max"]
    assert isinstance(bot_metrics, dict)
    online_max = first_int(summary.get("online_max"), gate_values.get("observed_online_max"))
    connected_max = first_int(summary.get("bot_connected_max"), gate_values.get("observed_bot_connected_max"), bot_metrics.get("connected"))
    active_max = first_int(summary.get("bot_active_max"), gate_values.get("observed_bot_active_max"), bot_metrics.get("active"))
    if required_bots is not None and (
        (online_max is not None and online_max < required_bots)
        or (connected_max is not None and connected_max < required_bots)
        or (active_max is not None and active_max < required_bots)
    ):
        blockers.append(
            "FAIL population: "
            f"online_max={fmt_value(online_max, 0)} connected_max={fmt_value(connected_max, 0)} "
            f"active_max={fmt_value(active_max, 0)} required={required_bots}"
        )

    error_counts = bot_data["error_counts"]
    kick_counts = bot_data["kick_counts"]
    assert isinstance(error_counts, Counter)
    assert isinstance(kick_counts, Counter)
    bot_errors_max = first_int(summary.get("bot_errors_max"), gate_values.get("observed_bot_errors_max"), bot_metrics.get("errors"))
    bot_kicked_max = first_int(summary.get("bot_kicked_max"), gate_values.get("observed_bot_kicked_max"), bot_metrics.get("kicked"))
    if sum(error_counts.values()) > 0 or sum(kick_counts.values()) > 0 or (bot_errors_max or 0) > 0 or (bot_kicked_max or 0) > 0:
        blockers.append(
            "FAIL bot_errors: "
            f"summary_errors_max={fmt_value(bot_errors_max, 0)} summary_kicked_max={fmt_value(bot_kicked_max, 0)} "
            f"log_error_events={sum(error_counts.values())} log_kicks={sum(kick_counts.values())}"
        )

    required_tps_avg = to_float(gate_values.get("required_tps1_avg_min"))
    required_tps_min = to_float(gate_values.get("required_tps1_min_min"))
    required_tick_avg_max = to_float(gate_values.get("required_avg_tick_ms_avg_max"))
    required_tick_max_max = to_float(gate_values.get("required_avg_tick_ms_max_max"))
    tps_failed = (
        tps1 is not None
        and (
            (required_tps_avg is not None and tps1.average is not None and tps1.average < required_tps_avg)
            or (required_tps_min is not None and tps1.minimum is not None and tps1.minimum < required_tps_min)
        )
    )
    tick_failed = (
        avg_tick_ms is not None
        and (
            required_tick_avg_max is not None
            and avg_tick_ms.average is not None
            and avg_tick_ms.average > required_tick_avg_max
            or required_tick_max_max is not None
            and avg_tick_ms.maximum is not None
            and avg_tick_ms.maximum > required_tick_max_max
        )
    )
    if tps_failed or tick_failed:
        blockers.append(
            "FAIL whole_run_tps_tick: "
            f"tps_avg={fmt_value(tps1.average if tps1 else None)} min={fmt_value(tps1.minimum if tps1 else None)} "
            f"tick_avg_ms={fmt_value(avg_tick_ms.average if avg_tick_ms else None)} "
            f"tick_max_ms={fmt_value(avg_tick_ms.maximum if avg_tick_ms else None)}"
        )

    steal = cpu_data.get("steal")
    iowait = cpu_data.get("iowait")
    assert steal is None or isinstance(steal, Series)
    assert iowait is None or isinstance(iowait, Series)
    steal_limit = first_float(preflight.get("max_steal_percent"), "10")
    iowait_limit = first_float(preflight.get("max_iowait_percent"), "10")
    cpu_failed = (
        steal is not None
        and steal.maximum is not None
        and steal_limit is not None
        and steal.maximum > steal_limit
    ) or (
        iowait is not None
        and iowait.maximum is not None
        and iowait_limit is not None
        and iowait.maximum > iowait_limit
    )
    if cpu_failed:
        blockers.append(
            "FAIL host_cpu: "
            f"steal_max={fmt_value(steal.maximum if steal else None)} limit={fmt_value(steal_limit)} "
            f"iowait_max={fmt_value(iowait.maximum if iowait else None)} limit={fmt_value(iowait_limit)}"
        )

    process_rss_limit = first_float(gate_values.get("required_process_rss_mib_max"))
    process_rss_max = first_float(summary.get("process_rss_mib_max"), cpu_data.get("pid_rss_mib_max"))
    if process_rss_limit is not None and process_rss_max is not None:
        if process_rss_max > process_rss_limit:
            blockers.append(
                "FAIL process_rss: "
                f"process_rss_max={fmt_value(process_rss_max)} limit={fmt_value(process_rss_limit)}"
            )
        else:
            blockers.append(
                "PASS process_rss: "
                f"process_rss_max={fmt_value(process_rss_max)} limit={fmt_value(process_rss_limit)}"
            )

    stability_keys = (
        "moved_too_quickly_warnings",
        "watchdog_thread_dumps",
        "sync_load_stack_hits",
        "nearby_players_stack_hits",
        "thread_check_failures",
        "chunk_system_errors",
        "feature_placement_errors",
        "off_main_poi_hits",
        "stability_failures",
        "external_thread_prints",
    )
    if all((to_int(summary.get(key)) or 0) == 0 for key in stability_keys):
        blockers.append("PASS stability_counters: no watchdog/thread/chunk/stability hits")

    return blockers


def existing_or_none(path: pathlib.Path | None) -> pathlib.Path | None:
    if path is not None and path.is_file():
        return path
    return None


def derive_run_id(paths: list[pathlib.Path | None]) -> str | None:
    suffixes = (
        "-summary.txt",
        "-gate.txt",
        "-preflight.txt",
        "-resources.csv",
        "-bots.log",
        ".log",
    )
    for path in paths:
        if path is None:
            continue
        name = path.name
        for suffix in suffixes:
            if name.endswith(suffix):
                return name.removesuffix(suffix)
    return None


def resolve_inputs(args: argparse.Namespace) -> Inputs:
    explicit_paths = [args.summary, args.gate, args.log, args.bot_log, args.resources, args.preflight]
    run_id = args.run_id or derive_run_id(explicit_paths)
    if not run_id:
        raise ValueError("provide --run-id or at least one explicit artifact path")

    def choose(explicit: pathlib.Path | None, inferred: pathlib.Path) -> pathlib.Path | None:
        return existing_or_none(explicit) if explicit is not None else existing_or_none(inferred)

    return Inputs(
        run_id=run_id,
        summary=choose(args.summary, args.reports_dir / f"{run_id}-summary.txt"),
        gate=choose(args.gate, args.reports_dir / f"{run_id}-gate.txt"),
        log=choose(args.log, args.logs_dir / f"{run_id}.log"),
        bot_log=choose(args.bot_log, args.logs_dir / f"{run_id}-bots.log"),
        resources=choose(args.resources, args.reports_dir / f"{run_id}-resources.csv"),
        preflight=choose(args.preflight, args.reports_dir / f"{run_id}-preflight.txt"),
    )


def path_status(label: str, path: pathlib.Path | None) -> str:
    return f"{label}={path if path is not None else 'missing'}"


def build_report(inputs: Inputs) -> str:
    summary = parse_kv_file(inputs.summary, stop_at_bot_tail=True)
    gate_values, gate_failures = parse_gate(inputs.gate)
    preflight = parse_kv_file(inputs.preflight)
    server_data = parse_server_log(inputs.log)
    bot_data = parse_bot_log(inputs.bot_log)
    cpu_data = parse_resources(inputs.resources)

    tps1 = server_data["tps1"]
    avg_tick_ms = server_data["avg_tick_ms"]
    assert tps1 is None or isinstance(tps1, Series)
    assert avg_tick_ms is None or isinstance(avg_tick_ms, Series)
    if tps1 is None:
        tps1 = series_from_summary(
            summary,
            minimum_key="tps1_min",
            average_key="tps1_avg",
            maximum_key="tps1_max",
            count_key="metrics_samples",
            source="summary",
        )
    if avg_tick_ms is None:
        avg_tick_ms = series_from_summary(
            summary,
            minimum_key="avg_tick_ms_min",
            average_key="avg_tick_ms_avg",
            maximum_key="avg_tick_ms_max",
            count_key="metrics_samples",
            source="summary",
        )

    required_bots = first_int(gate_values.get("required_bots"), summary.get("bots"))
    bot_metrics = bot_data["metrics_max"]
    assert isinstance(bot_metrics, dict)
    online_max = first_int(summary.get("online_max"), gate_values.get("observed_online_max"), server_data["online_max"])
    connected_max = first_int(summary.get("bot_connected_max"), gate_values.get("observed_bot_connected_max"), bot_metrics.get("connected"))
    active_max = first_int(summary.get("bot_active_max"), gate_values.get("observed_bot_active_max"), bot_metrics.get("active"))
    ready_max = first_int(summary.get("bot_ready_max"), gate_values.get("observed_bot_ready_max"), bot_metrics.get("ready"))
    created_max = first_int(summary.get("bot_created_max"), gate_values.get("observed_bot_created_max"), bot_metrics.get("created"))
    login_packet_max = first_int(summary.get("bot_login_packet_max"), connected_max)
    player_join_ready_max = first_int(summary.get("bot_player_join_ready_max"), ready_max)
    loadgen_source = summary.get("bot_loadgen_telemetry_source", "missing")
    loadgen_loop_delay_p95 = max_avg_series_from_summary(
        summary,
        prefix="bot_loadgen_loop_delay_p95_ms",
        count_key="bot_loadgen_telemetry_samples",
        source=loadgen_source,
    )
    loadgen_loop_delay_max = max_avg_series_from_summary(
        summary,
        prefix="bot_loadgen_loop_delay_max_ms",
        count_key="bot_loadgen_telemetry_samples",
        source=loadgen_source,
    )
    loadgen_timer_drift_max = max_avg_series_from_summary(
        summary,
        prefix="bot_loadgen_timer_drift_max_ms",
        count_key="bot_loadgen_telemetry_samples",
        source=loadgen_source,
    )
    loadgen_elu = max_avg_series_from_summary(
        summary,
        prefix="bot_loadgen_elu_pct",
        count_key="bot_loadgen_telemetry_samples",
        source=loadgen_source,
    )

    gate_failure_classes = Counter(classify_gate_failure(failure) for failure in gate_failures)
    gate_pass = gate_values.get("gate_pass", "unknown")
    profile = gate_values.get("gate_profile", "unknown")
    failure_count = first_int(gate_values.get("failure_count"), len(gate_failures)) or 0
    requirement_count = gate_values.get("requirement_count", "n/a")
    claim_eligible = gate_values.get("claim_eligible", "n/a")
    run_class = gate_values.get("run_class", "unknown")
    environment_invalid = gate_values.get("environment_invalid", "unknown")
    environment_invalid_kind = gate_values.get("environment_invalid_kind", "unknown")
    environment_invalid_reason = gate_values.get("environment_invalid_reason", gate_values.get("early_abort_reason", "missing"))
    world_mode = summary.get("world_mode", gate_values.get("observed_world_mode", "n/a"))
    claim_surface = summary.get("claim_surface", gate_values.get("observed_claim_surface", "n/a"))
    warm_source_present = summary.get("world_warm_source_present", gate_values.get("observed_world_warm_source_present", "n/a"))
    warm_source = summary.get("world_warm_source", gate_values.get("observed_world_warm_source", "n/a"))
    load_window_samples = first_int(summary.get("load_window_metrics_samples"), gate_values.get("observed_load_window_metrics_samples"))
    load_window_reached_full_online = summary.get("load_window_reached_full_online", gate_values.get("observed_load_window_reached_full_online"))
    load_window_ended_by_drop = summary.get("load_window_ended_by_online_drop", gate_values.get("observed_load_window_ended_by_online_drop"))
    load_window_online_max = first_int(summary.get("load_window_online_max"), gate_values.get("observed_load_window_online_max"))
    load_window_loaded_chunks_max = first_int(summary.get("load_window_loaded_chunks_max"), gate_values.get("observed_load_window_loaded_chunks_max"))
    load_window_tps1_avg = to_float(summary.get("load_window_tps1_avg"))
    load_window_tps1_min = to_float(summary.get("load_window_tps1_min"))
    load_window_avg_tick_ms_avg = to_float(summary.get("load_window_avg_tick_ms_avg"))
    load_window_avg_tick_ms_max = to_float(summary.get("load_window_avg_tick_ms_max"))
    process_rss_limit = to_float(gate_values.get("required_process_rss_mib_max"))
    process_rss_max = first_float(summary.get("process_rss_mib_max"), cpu_data.get("pid_rss_mib_max"))

    lines: list[str] = [
        f"run_id={inputs.run_id}",
        "inputs:",
        f"- {path_status('summary', inputs.summary)}",
        f"- {path_status('gate', inputs.gate)}",
        f"- {path_status('log', inputs.log)}",
        f"- {path_status('bot_log', inputs.bot_log)}",
        f"- {path_status('resources', inputs.resources)}",
        f"- {path_status('preflight', inputs.preflight)}",
        "",
        f"gate: status={gate_pass} profile={profile} failures={failure_count} requirements={requirement_count}",
        "gate_failure_classes:",
    ]

    if gate_failure_classes:
        for label, count in sorted(gate_failure_classes.items()):
            lines.append(f"- {label}: {count}")
    else:
        lines.append("- none")

    lines.append("gate_failures:")
    if gate_failures:
        for failure in gate_failures:
            lines.append(f"- {failure}")
    else:
        lines.append("- none")

    error_counts = bot_data["error_counts"]
    kick_counts = bot_data["kick_counts"]
    end_counts = bot_data["end_counts"]
    assert isinstance(error_counts, Counter)
    assert isinstance(kick_counts, Counter)
    assert isinstance(end_counts, Counter)

    lines.extend(
        [
            "",
            "population:",
            (
                f"- online_max={fmt_value(online_max, 0)} connected_max={fmt_value(connected_max, 0)} "
                f"active_max={fmt_value(active_max, 0)} ready_max={fmt_value(ready_max, 0)} "
                f"created_max={fmt_value(created_max, 0)} required={fmt_value(required_bots, 0)}"
            ),
            (
                f"- bot_signals connected_source={summary.get('bot_connected_source', 'protocol_login_packet')} "
                f"ready_source={summary.get('bot_ready_source', 'client_playerJoin_signal')} "
                f"login_packet_max={fmt_value(login_packet_max, 0)} "
                f"player_join_ready_max={fmt_value(player_join_ready_max, 0)}"
            ),
            (
                f"- server_join_events={summary.get('server_join_events', 'n/a')} "
                f"server_quit_events={summary.get('server_quit_events', 'n/a')} "
                f"loaded_chunks_max={summary.get('loaded_chunks_max', fmt_value(server_data['loaded_chunks_max'], 0))}"
            ),
            "",
            "claim_state:",
            f"- claim_eligible={claim_eligible} gate_pass={gate_pass} profile={profile}",
            f"- run_class={run_class} environment_invalid={environment_invalid} kind={environment_invalid_kind} reason={environment_invalid_reason}",
            f"- world_mode={world_mode} claim_surface={claim_surface} warm_source_present={warm_source_present} warm_source={warm_source}",
            "",
            "strict_load_window:",
            f"- samples={fmt_value(load_window_samples, 0)} reached_full_online={load_window_reached_full_online} ended_by_online_drop={load_window_ended_by_drop}",
            f"- online_max={fmt_value(load_window_online_max, 0)} loaded_chunks_max={fmt_value(load_window_loaded_chunks_max, 0)}",
            f"- tps1_avg={fmt_value(load_window_tps1_avg)} tps1_min={fmt_value(load_window_tps1_min)}",
            f"- avg_tick_ms_avg={fmt_value(load_window_avg_tick_ms_avg)} avg_tick_ms_max={fmt_value(load_window_avg_tick_ms_max)}",
            "",
            "tps_tick:",
            f"- whole_run_tps1 {fmt_series(tps1)} required_avg_min={gate_values.get('required_tps1_avg_min', 'n/a')} required_min_min={gate_values.get('required_tps1_min_min', 'n/a')}",
            f"- whole_run_avg_tick_ms {fmt_series(avg_tick_ms)} required_avg_max={gate_values.get('required_avg_tick_ms_avg_max', 'n/a')} required_max_max={gate_values.get('required_avg_tick_ms_max_max', 'n/a')}",
            "",
            "host_cpu:",
            f"- windows={cpu_data.get('windows', 'n/a')} host_cpu_count={summary.get('host_cpu_count', 'n/a')}",
            f"- steal_pct {fmt_series(cpu_data.get('steal') if isinstance(cpu_data.get('steal'), Series) else None)} limit={preflight.get('max_steal_percent', '10')}",
            f"- iowait_pct {fmt_series(cpu_data.get('iowait') if isinstance(cpu_data.get('iowait'), Series) else None)} limit={preflight.get('max_iowait_percent', '10')}",
            f"- idle_pct {fmt_series(cpu_data.get('idle') if isinstance(cpu_data.get('idle'), Series) else None)}",
            f"- system_load1 {fmt_series(cpu_data.get('load1') if isinstance(cpu_data.get('load1'), Series) else None)}",
            f"- process_rss_mib_max={fmt_value(process_rss_max)} limit={fmt_value(process_rss_limit)}",
            "",
            "load_generator:",
            f"- loop_delay_p95_ms {fmt_series(loadgen_loop_delay_p95)}",
            f"- loop_delay_max_ms {fmt_series(loadgen_loop_delay_max)}",
            f"- timer_drift_max_ms {fmt_series(loadgen_timer_drift_max)}",
            f"- event_loop_utilization_pct {fmt_series(loadgen_elu)}",
            "",
        ]
    )

    append_counter(lines, "bot_error_classes", error_counts)
    append_counter(lines, "bot_kick_reasons", kick_counts)
    append_counter(lines, "bot_end_reasons_top", end_counts, limit=5)

    disconnects = server_data["disconnects"]
    assert isinstance(disconnects, Counter)
    lines.append("")
    append_counter(lines, "server_disconnect_reasons_top", disconnects, limit=5)
    lines.append(f"server_disconnect_events_total={server_data.get('disconnect_line_count', 0)}")
    lines.append(f"server_disconnect_unique_bots={server_data.get('disconnect_unique_bots', 0)}")

    lines.append("")
    lines.append("pass_fail_blockers:")
    for blocker in build_blockers(
        summary=summary,
        gate_values=gate_values,
        gate_failures=gate_failures,
        bot_data=bot_data,
        tps1=tps1,
        avg_tick_ms=avg_tick_ms,
        cpu_data=cpu_data,
        preflight=preflight,
    ):
        lines.append(f"- {blocker}")

    if server_data["error_lines"]:
        lines.append(f"- NOTE server_error_lines={server_data['error_lines']}")

    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-id", help="run id prefix, without -summary/-resources suffix")
    parser.add_argument("--summary", type=pathlib.Path, help="explicit summary path")
    parser.add_argument("--gate", type=pathlib.Path, help="explicit gate report path")
    parser.add_argument("--log", type=pathlib.Path, help="explicit server log path")
    parser.add_argument("--bot-log", type=pathlib.Path, help="explicit bot log path")
    parser.add_argument("--resources", type=pathlib.Path, help="explicit resources CSV path")
    parser.add_argument("--preflight", type=pathlib.Path, help="explicit host preflight path")
    parser.add_argument("--report", type=pathlib.Path, help="optional report output path")
    parser.add_argument("--reports-dir", type=pathlib.Path, default=pathlib.Path("reports"))
    parser.add_argument("--logs-dir", type=pathlib.Path, default=pathlib.Path("logs"))
    args = parser.parse_args()

    try:
        inputs = resolve_inputs(args)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 64

    if not any((inputs.summary, inputs.gate, inputs.log, inputs.bot_log, inputs.resources, inputs.preflight)):
        print(f"no artifacts found for run id {inputs.run_id}", file=sys.stderr)
        return 66

    report = build_report(inputs)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(report, encoding="utf-8")
    print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
