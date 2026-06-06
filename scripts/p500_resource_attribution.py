#!/usr/bin/env python3
"""Summarize resource attribution for a P500 contended diagnostic artifact."""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import os
import pathlib
import re
import statistics
from dataclasses import dataclass
from typing import Iterable


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_STAMP = "20260530-105204"
DEFAULT_LABEL = "p500-contended-diagnostic-current-artifact"
TOKEN_RE = re.compile(r"\b([A-Za-z][A-Za-z0-9_]*)=([^\s]+)")
SECTION_RE = re.compile(r"^\[(.+)\]$")
KV_RE = re.compile(r"^([^=]+)=(.*)$")
PAIR_RE = re.compile(r"^\s*([^:]+):\s*(.*)$")
HEAP_RE = re.compile(r"total\s+(\d+)K,\s+used\s+(\d+)K")
METASPACE_RE = re.compile(r"used\s+(\d+)K")
INT_RE = re.compile(r"(-?\d+)")


@dataclass(frozen=True)
class Inputs:
    stamp: str
    label: str
    stem: str
    resources: pathlib.Path
    server_log: pathlib.Path
    preflight: pathlib.Path
    memory_snapshot: pathlib.Path
    summary: pathlib.Path
    report: pathlib.Path


@dataclass
class NumberBucket:
    values: list[float]
    current: float | None = None

    def add(self, value: float | None) -> None:
        if value is None:
            return
        self.values.append(value)
        self.current = value

    @property
    def count(self) -> int:
        return len(self.values)

    @property
    def minimum(self) -> float | None:
        return min(self.values) if self.values else None

    @property
    def average(self) -> float | None:
        return statistics.fmean(self.values) if self.values else None

    @property
    def maximum(self) -> float | None:
        return max(self.values) if self.values else None


def to_float(value: str | int | float | None) -> float | None:
    if value is None or value == "":
        return None
    if isinstance(value, (int, float)):
        return float(value)
    try:
        return float(value)
    except ValueError:
        return None


def to_int(value: str | int | float | None) -> int | None:
    parsed = to_float(value)
    if parsed is None:
        return None
    return int(parsed)


def mib(kb: float | int | None) -> float | None:
    return None if kb is None else float(kb) / 1024.0


def fmt(value: float | int | None, digits: int = 2) -> str:
    if value is None:
        return "missing"
    if isinstance(value, int):
        return str(value)
    return f"{value:.{digits}f}"


def fmt_mib_from_kb(kb: float | int | None) -> str:
    return fmt(mib(kb), 1)


def boolish(value: str | None) -> bool:
    return (value or "").strip().lower() == "true"


def relative(path: pathlib.Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        return str(path)


def parse_tokens(line: str) -> dict[str, str]:
    return dict(TOKEN_RE.findall(line))


def parse_kv_file(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line or line.endswith(":"):
                continue
            match = KV_RE.match(line)
            if match:
                values[match.group(1).strip()] = match.group(2).strip()
    return values


def parse_ps_row(line: str) -> dict[str, str] | None:
    parts = line.strip().split(None, 6)
    if len(parts) < 7:
        return None
    return {
        "pid": parts[0],
        "ppid": parts[1],
        "stat": parts[2],
        "etime": parts[3],
        "cpu": parts[4],
        "mem": parts[5],
        "cmd": parts[6],
        "raw": line.strip(),
    }


def parse_foreign_row(line: str) -> dict[str, str]:
    row: dict[str, str] = {}
    before_cmd, separator, cmd = line.partition(" cmd=")
    row.update(parse_tokens(before_cmd))
    if separator:
        row["cmd"] = cmd.strip()
    row["raw"] = line.strip()
    return row


def parse_preflight(path: pathlib.Path) -> dict[str, object]:
    values: dict[str, str] = {}
    foreign: list[dict[str, str]] = []
    interesting: list[dict[str, str]] = []
    section = "top"

    if not path.is_file():
        return {"values": values, "foreign": foreign, "interesting": interesting}

    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            stripped = line.strip()
            if not stripped:
                continue
            if stripped == "foreign_processes:":
                section = "foreign"
                continue
            if stripped == "interesting_processes:":
                section = "interesting"
                continue
            if section == "foreign":
                foreign.append(parse_foreign_row(stripped))
                continue
            if section == "interesting":
                parsed = parse_ps_row(stripped)
                if parsed is not None:
                    interesting.append(parsed)
                else:
                    interesting.append({"raw": stripped, "cmd": stripped})
                continue
            match = KV_RE.match(stripped)
            if match:
                values[match.group(1).strip()] = match.group(2).strip()

    return {"values": values, "foreign": foreign, "interesting": interesting}


def process_rows_summary(rows: Iterable[dict[str, str]]) -> dict[str, object]:
    rows = list(rows)
    cpu_values = [value for value in (to_float(row.get("cpu")) for row in rows) if value is not None]
    mem_values = [value for value in (to_float(row.get("mem")) for row in rows) if value is not None]
    return {
        "count": len(rows),
        "cpu_max": max(cpu_values) if cpu_values else None,
        "cpu_total": sum(cpu_values) if cpu_values else 0.0,
        "mem_percent_max": max(mem_values) if mem_values else None,
        "mem_percent_total": sum(mem_values) if mem_values else 0.0,
    }


def parse_memory_snapshot(path: pathlib.Path) -> dict[str, str]:
    top: dict[str, str] = {}
    proc_status: dict[str, str] = {}
    smaps_rollup: dict[str, str] = {}
    heap_total_kb: str | None = None
    heap_used_kb: str | None = None
    metaspace_used_kb: str | None = None
    nmt_enabled = "missing"
    section = "top"

    if not path.is_file():
        return {
            "present": "false",
            "rss_kb": "missing",
            "rss_mib": "missing",
            "heap_used_kb": "missing",
            "heap_total_kb": "missing",
            "metaspace_used_kb": "missing",
            "nmt_enabled": "missing",
        }

    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            stripped = line.strip()
            if not stripped:
                continue
            section_match = SECTION_RE.match(stripped)
            if section_match:
                section = section_match.group(1)
                continue
            if section == "top":
                match = KV_RE.match(stripped)
                if match:
                    top[match.group(1).strip()] = match.group(2).strip()
                continue
            if section == "proc_status":
                match = PAIR_RE.match(line)
                if match:
                    proc_status[match.group(1).strip()] = match.group(2).strip()
                continue
            if section == "smaps_rollup":
                match = PAIR_RE.match(line)
                if match:
                    smaps_rollup[match.group(1).strip()] = match.group(2).strip()
                continue
            if section == "jcmd_gc_heap_info":
                if heap_total_kb is None or heap_used_kb is None:
                    heap_match = HEAP_RE.search(stripped)
                    if heap_match:
                        heap_total_kb = heap_match.group(1)
                        heap_used_kb = heap_match.group(2)
                        continue
                if metaspace_used_kb is None and stripped.startswith("Metaspace"):
                    metaspace_match = METASPACE_RE.search(stripped)
                    if metaspace_match:
                        metaspace_used_kb = metaspace_match.group(1)
                continue
            if section == "jcmd_vm_native_memory_summary":
                lowered = stripped.lower()
                if "native memory tracking is not enabled" in lowered:
                    nmt_enabled = "false"
                elif lowered.startswith("native memory tracking"):
                    nmt_enabled = "true"

    def first_int(value: str | None) -> str:
        if value is None:
            return "missing"
        match = INT_RE.search(value)
        return match.group(1) if match else "missing"

    rss_kb = top.get("rss_kb") or first_int(proc_status.get("VmRSS"))
    rss_mib = top.get("rss_mib")
    if rss_mib is None and rss_kb != "missing":
        rss_mib = fmt_mib_from_kb(to_int(rss_kb))

    return {
        "present": "true",
        "rss_kb": rss_kb,
        "rss_mib": rss_mib or "missing",
        "proc_status_vmrss_kb": first_int(proc_status.get("VmRSS")),
        "smaps_rollup_rss_kb": first_int(smaps_rollup.get("Rss")),
        "smaps_rollup_pss_kb": first_int(smaps_rollup.get("Pss")),
        "heap_used_kb": heap_used_kb or "missing",
        "heap_total_kb": heap_total_kb or "missing",
        "metaspace_used_kb": metaspace_used_kb or "missing",
        "nmt_enabled": nmt_enabled,
    }


def parse_resources(path: pathlib.Path, cpu_count: int | None) -> dict[str, object]:
    buckets = {
        "pid_cpu": NumberBucket([]),
        "pid_rss_kb": NumberBucket([]),
        "system_load1": NumberBucket([]),
        "system_mem_available_kb": NumberBucket([]),
        "bot_process_count": NumberBucket([]),
        "bot_rss_kb_max": NumberBucket([]),
        "bot_rss_kb_total": NumberBucket([]),
        "bot_pss_kb_max": NumberBucket([]),
        "bot_pss_kb_total": NumberBucket([]),
    }
    host_idle = NumberBucket([])
    host_iowait = NumberBucket([])
    host_steal = NumberBucket([])
    combined_rss_kb = NumberBucket([])
    combined_peak_server_rss_kb: float | None = None
    combined_peak_bot_rss_kb: float | None = None
    first_ts: int | None = None
    last_ts: int | None = None
    samples = 0
    pss_available_samples = 0
    previous: dict[str, str] | None = None

    if not path.is_file():
        return {
            "present": False,
            "samples": 0,
            "buckets": buckets,
            "host_idle": host_idle,
            "host_iowait": host_iowait,
            "host_steal": host_steal,
            "combined_rss_kb": combined_rss_kb,
            "combined_peak_server_rss_kb": combined_peak_server_rss_kb,
            "combined_peak_bot_rss_kb": combined_peak_bot_rss_kb,
            "pss_available_samples": 0,
            "span_seconds": None,
        }

    with path.open(encoding="utf-8", errors="replace", newline="") as handle:
        for row in csv.DictReader(handle):
            samples += 1
            ts = to_int(row.get("ts_ms"))
            if ts is not None:
                if first_ts is None:
                    first_ts = ts
                last_ts = ts

            for key, bucket in buckets.items():
                bucket.add(to_float(row.get(key)))

            if boolish(row.get("bot_pss_available")):
                pss_available_samples += 1

            server_rss = to_float(row.get("pid_rss_kb"))
            bot_total = to_float(row.get("bot_rss_kb_total"))
            if server_rss is not None and bot_total is not None:
                combined_value = server_rss + bot_total
                if combined_rss_kb.maximum is None or combined_value > combined_rss_kb.maximum:
                    combined_peak_server_rss_kb = server_rss
                    combined_peak_bot_rss_kb = bot_total
                combined_rss_kb.add(combined_value)

            if previous is not None:
                total_before = to_float(previous.get("host_cpu_total"))
                total_after = to_float(row.get("host_cpu_total"))
                if total_before is not None and total_after is not None:
                    total_delta = total_after - total_before
                    if total_delta > 0:
                        for field, bucket in (
                            ("host_cpu_idle", host_idle),
                            ("host_cpu_iowait", host_iowait),
                            ("host_cpu_steal", host_steal),
                        ):
                            before = to_float(previous.get(field))
                            after = to_float(row.get(field))
                            if before is not None and after is not None:
                                bucket.add((after - before) * 100.0 / total_delta)
            previous = row

    load1_per_cpu_max = None
    if cpu_count and buckets["system_load1"].maximum is not None:
        load1_per_cpu_max = buckets["system_load1"].maximum / cpu_count

    span_seconds = None
    if first_ts is not None and last_ts is not None:
        span_seconds = (last_ts - first_ts) / 1000.0

    return {
        "present": True,
        "samples": samples,
        "first_ts_ms": first_ts,
        "last_ts_ms": last_ts,
        "span_seconds": span_seconds,
        "buckets": buckets,
        "host_idle": host_idle,
        "host_iowait": host_iowait,
        "host_steal": host_steal,
        "combined_rss_kb": combined_rss_kb,
        "combined_peak_server_rss_kb": combined_peak_server_rss_kb,
        "combined_peak_bot_rss_kb": combined_peak_bot_rss_kb,
        "pss_available_samples": pss_available_samples,
        "load1_per_cpu_max": load1_per_cpu_max,
    }


def parse_server_log(path: pathlib.Path, target_online: int) -> dict[str, object]:
    metrics: list[dict[str, float]] = []
    if path.is_file():
        with path.open(encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if "COMPAT_PROBE metrics" not in line:
                    continue
                tokens = parse_tokens(line)
                metric = {
                    "online": to_float(tokens.get("online")),
                    "loadedChunks": to_float(tokens.get("loadedChunks")),
                    "tps1": to_float(tokens.get("tps1")),
                    "avgTickMs": to_float(tokens.get("avgTickMs")),
                    "usedMemMiB": to_float(tokens.get("usedMemMiB")),
                }
                if any(value is not None for value in metric.values()):
                    metrics.append(metric)

    def bucket_for(key: str, samples: list[dict[str, float]]) -> NumberBucket:
        bucket = NumberBucket([])
        for sample in samples:
            bucket.add(sample.get(key))
        return bucket

    load_window: list[dict[str, float]] = []
    reached_full_online = False
    ended_by_online_drop = False
    started = False
    end_index: int | None = None
    start_index: int | None = None

    for idx, metric in enumerate(metrics):
        online = metric.get("online")
        if online is None:
            continue
        if not started:
            if online >= target_online:
                started = True
                reached_full_online = True
                start_index = idx
                load_window.append(metric)
            continue
        if online < target_online:
            ended_by_online_drop = True
            end_index = idx
            break
        load_window.append(metric)
        if online >= target_online:
            reached_full_online = True
    if started and end_index is None:
        end_index = (start_index or 0) + len(load_window)

    return {
        "metrics_samples": len(metrics),
        "online": bucket_for("online", metrics),
        "loaded_chunks": bucket_for("loadedChunks", metrics),
        "tps1": bucket_for("tps1", metrics),
        "avg_tick_ms": bucket_for("avgTickMs", metrics),
        "used_mem_mib": bucket_for("usedMemMiB", metrics),
        "load_window_samples": len(load_window),
        "load_window_reached_full_online": reached_full_online,
        "load_window_ended_by_online_drop": ended_by_online_drop,
        "load_window_online": bucket_for("online", load_window),
        "load_window_loaded_chunks": bucket_for("loadedChunks", load_window),
        "load_window_tps1": bucket_for("tps1", load_window),
        "load_window_avg_tick_ms": bucket_for("avgTickMs", load_window),
        "load_window_used_mem_mib": bucket_for("usedMemMiB", load_window),
    }


def resolve_inputs(args: argparse.Namespace) -> Inputs:
    stamp = args.stamp
    label = args.label or f"{DEFAULT_LABEL}-{stamp}"
    stem = label if label.startswith("load-") else f"load-{label}"

    resources = args.resources or ROOT / "reports" / f"{stem}-resources.csv"
    server_log = args.server_log or ROOT / "logs" / f"{stem}.log"
    preflight = args.preflight or ROOT / "reports" / f"{stem}-preflight.txt"
    memory_snapshot = args.memory_snapshot or ROOT / "reports" / f"{stem}-memory" / "peak-latest.txt"
    summary = args.summary or ROOT / "reports" / f"{stem}-summary.txt"
    report = args.out or ROOT / "reports" / f"p500-resource-attribution-{stamp}.txt"

    return Inputs(
        stamp=stamp,
        label=label,
        stem=stem,
        resources=resources,
        server_log=server_log,
        preflight=preflight,
        memory_snapshot=memory_snapshot,
        summary=summary,
        report=report,
    )


def append_bucket(
    lines: list[str],
    prefix: str,
    bucket: NumberBucket,
    *,
    digits: int = 2,
    as_mib: bool = False,
) -> None:
    def format_value(value: float | None) -> str:
        if as_mib:
            return fmt_mib_from_kb(value)
        return fmt(value, digits)

    lines.append(f"{prefix}_samples={bucket.count}")
    lines.append(f"{prefix}_current={format_value(bucket.current)}")
    lines.append(f"{prefix}_min={format_value(bucket.minimum)}")
    lines.append(f"{prefix}_avg={format_value(bucket.average)}")
    lines.append(f"{prefix}_max={format_value(bucket.maximum)}")


def build_report(inputs: Inputs, target_online: int) -> str:
    preflight = parse_preflight(inputs.preflight)
    preflight_values = preflight["values"]
    assert isinstance(preflight_values, dict)
    cpu_count = to_int(preflight_values.get("cpu_count")) or os.cpu_count() or 1
    resources = parse_resources(inputs.resources, cpu_count)
    log = parse_server_log(inputs.server_log, target_online)
    memory = parse_memory_snapshot(inputs.memory_snapshot)
    summary_values = parse_kv_file(inputs.summary)

    buckets = resources["buckets"]
    assert isinstance(buckets, dict)
    foreign_rows = preflight["foreign"]
    interesting_rows = preflight["interesting"]
    assert isinstance(foreign_rows, list)
    assert isinstance(interesting_rows, list)

    gradle_rows = [
        row for row in foreign_rows + interesting_rows
        if "gradle" in row.get("cmd", row.get("raw", "")).lower()
        or "gradlew" in row.get("cmd", row.get("raw", "")).lower()
    ]
    foreign_summary = process_rows_summary(foreign_rows)
    gradle_summary = process_rows_summary(gradle_rows)
    interesting_summary = process_rows_summary(interesting_rows)

    server_rss_max = buckets["pid_rss_kb"].maximum
    bot_total_max = buckets["bot_rss_kb_total"].maximum
    combined_max = resources["combined_rss_kb"].maximum
    combined_peak_server = resources.get("combined_peak_server_rss_kb")
    combined_peak_bot = resources.get("combined_peak_bot_rss_kb")
    server_share = (combined_peak_server / combined_max * 100.0) if combined_peak_server and combined_max else None
    bot_share = (combined_peak_bot / combined_max * 100.0) if combined_peak_bot and combined_max else None

    generated_at = dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    lines: list[str] = [
        "p500_resource_attribution_report=true",
        f"stamp={inputs.stamp}",
        f"label={inputs.label}",
        f"target_online={target_online}",
        f"generated_at_utc={generated_at}",
        "",
        "[inputs]",
        f"resources_csv={relative(inputs.resources)}",
        f"server_log={relative(inputs.server_log)}",
        f"preflight_report={relative(inputs.preflight)}",
        f"memory_snapshot={relative(inputs.memory_snapshot)}",
        f"summary_report={relative(inputs.summary)}",
        "",
        "[key_facts]",
        f"resource_samples={resources['samples']}",
        f"resource_span_seconds={fmt(resources['span_seconds'], 1)}",
        f"server_rss_mib_max={fmt_mib_from_kb(server_rss_max)}",
        f"server_cpu_percent_max={fmt(buckets['pid_cpu'].maximum, 2)}",
        f"bot_process_count_max={fmt(buckets['bot_process_count'].maximum, 0)}",
        f"bot_shard_rss_mib_max={fmt_mib_from_kb(buckets['bot_rss_kb_max'].maximum)}",
        f"bot_aggregate_rss_mib_max={fmt_mib_from_kb(bot_total_max)}",
        f"bot_aggregate_pss_mib_max={fmt_mib_from_kb(buckets['bot_pss_kb_total'].maximum)}",
        f"combined_server_plus_bot_rss_mib_max={fmt_mib_from_kb(combined_max)}",
        f"peak_online={fmt(log['online'].maximum, 0)}",
        f"peak_loaded_chunks={fmt(log['loaded_chunks'].maximum, 0)}",
        f"tps1_avg={fmt(log['tps1'].average, 2)}",
        f"tps1_min={fmt(log['tps1'].minimum, 2)}",
        f"avg_tick_ms_avg={fmt(log['avg_tick_ms'].average, 2)}",
        f"avg_tick_ms_max={fmt(log['avg_tick_ms'].maximum, 2)}",
        f"compat_probe_used_mem_mib_max={fmt(log['used_mem_mib'].maximum, 0)}",
        f"snapshot_heap_used_mib={fmt_mib_from_kb(to_int(memory.get('heap_used_kb')))}",
        f"snapshot_heap_total_mib={fmt_mib_from_kb(to_int(memory.get('heap_total_kb')))}",
        f"host_load1_max={fmt(buckets['system_load1'].maximum, 2)}",
        f"host_load1_per_cpu_max={fmt(resources['load1_per_cpu_max'], 3)}",
        f"host_steal_percent_max={fmt(resources['host_steal'].maximum, 2)}",
        f"host_iowait_percent_max={fmt(resources['host_iowait'].maximum, 2)}",
        f"gradle_processes_observed={gradle_summary['count']}",
        f"foreign_processes_observed={foreign_summary['count']}",
        "",
        "[server_process_from_resources]",
    ]

    append_bucket(lines, "server_cpu_percent", buckets["pid_cpu"], digits=2)
    append_bucket(lines, "server_rss_mib", buckets["pid_rss_kb"], digits=1, as_mib=True)
    append_bucket(lines, "combined_server_plus_bot_rss_mib", resources["combined_rss_kb"], digits=1, as_mib=True)
    lines.extend([
        f"server_rss_at_combined_peak_mib={fmt_mib_from_kb(combined_peak_server)}",
        f"bot_aggregate_rss_at_combined_peak_mib={fmt_mib_from_kb(combined_peak_bot)}",
        f"server_rss_share_of_combined_peak_percent={fmt(server_share, 1)}",
        f"bot_rss_share_of_combined_peak_percent={fmt(bot_share, 1)}",
        "",
        "[bot_shards_from_resources]",
    ])
    append_bucket(lines, "bot_process_count", buckets["bot_process_count"], digits=0)
    append_bucket(lines, "bot_shard_rss_mib", buckets["bot_rss_kb_max"], digits=1, as_mib=True)
    append_bucket(lines, "bot_aggregate_rss_mib", buckets["bot_rss_kb_total"], digits=1, as_mib=True)
    lines.append(f"bot_pss_available_samples={resources['pss_available_samples']}")
    append_bucket(lines, "bot_shard_pss_mib", buckets["bot_pss_kb_max"], digits=1, as_mib=True)
    append_bucket(lines, "bot_aggregate_pss_mib", buckets["bot_pss_kb_total"], digits=1, as_mib=True)

    lines.extend([
        "",
        "[host_from_resources]",
        f"cpu_count={cpu_count}",
    ])
    append_bucket(lines, "host_load1", buckets["system_load1"], digits=2)
    append_bucket(lines, "host_mem_available_mib", buckets["system_mem_available_kb"], digits=1, as_mib=True)
    append_bucket(lines, "host_cpu_idle_percent", resources["host_idle"], digits=2)
    append_bucket(lines, "host_cpu_iowait_percent", resources["host_iowait"], digits=2)
    append_bucket(lines, "host_cpu_steal_percent", resources["host_steal"], digits=2)

    lines.extend([
        "",
        "[compat_probe_from_server_log]",
        f"metrics_samples={log['metrics_samples']}",
    ])
    append_bucket(lines, "online", log["online"], digits=0)
    append_bucket(lines, "loaded_chunks", log["loaded_chunks"], digits=0)
    append_bucket(lines, "tps1", log["tps1"], digits=2)
    append_bucket(lines, "avg_tick_ms", log["avg_tick_ms"], digits=2)
    append_bucket(lines, "used_mem_mib", log["used_mem_mib"], digits=0)
    lines.extend([
        f"load_window_policy=first_full_online_until_online_drop",
        f"load_window_reached_full_online={str(log['load_window_reached_full_online']).lower()}",
        f"load_window_ended_by_online_drop={str(log['load_window_ended_by_online_drop']).lower()}",
        f"load_window_metrics_samples={log['load_window_samples']}",
    ])
    append_bucket(lines, "load_window_online", log["load_window_online"], digits=0)
    append_bucket(lines, "load_window_loaded_chunks", log["load_window_loaded_chunks"], digits=0)
    append_bucket(lines, "load_window_tps1", log["load_window_tps1"], digits=2)
    append_bucket(lines, "load_window_avg_tick_ms", log["load_window_avg_tick_ms"], digits=2)
    append_bucket(lines, "load_window_used_mem_mib", log["load_window_used_mem_mib"], digits=0)

    lines.extend([
        "",
        "[heap_snapshot]",
        f"snapshot_present={memory['present']}",
        f"snapshot_rss_mib={memory['rss_mib']}",
        f"snapshot_rss_kb={memory['rss_kb']}",
        f"snapshot_smaps_pss_mib={fmt_mib_from_kb(to_int(memory.get('smaps_rollup_pss_kb')))}",
        f"heap_used_kb={memory['heap_used_kb']}",
        f"heap_used_mib={fmt_mib_from_kb(to_int(memory.get('heap_used_kb')))}",
        f"heap_total_kb={memory['heap_total_kb']}",
        f"heap_total_mib={fmt_mib_from_kb(to_int(memory.get('heap_total_kb')))}",
        f"metaspace_used_kb={memory['metaspace_used_kb']}",
        f"metaspace_used_mib={fmt_mib_from_kb(to_int(memory.get('metaspace_used_kb')))}",
        f"nmt_enabled={memory['nmt_enabled']}",
        "",
        "[gradle_foreign_from_preflight]",
        f"preflight_present={str(inputs.preflight.is_file()).lower()}",
        f"strict_foreign_process_count={preflight_values.get('strict_foreign_process_count', 'missing')}",
        f"strict_foreign_process_gate_pass={preflight_values.get('strict_foreign_process_gate_pass', 'missing')}",
        f"foreign_processes_observed={foreign_summary['count']}",
        f"foreign_cpu_percent_total={fmt(foreign_summary['cpu_total'], 2)}",
        f"foreign_cpu_percent_max={fmt(foreign_summary['cpu_max'], 2)}",
        f"foreign_mem_percent_total={fmt(foreign_summary['mem_percent_total'], 2)}",
        f"foreign_mem_percent_max={fmt(foreign_summary['mem_percent_max'], 2)}",
        "foreign_rss_mib=not_captured_by_preflight_artifact",
        f"interesting_processes_observed={interesting_summary['count']}",
        f"interesting_cpu_percent_total={fmt(interesting_summary['cpu_total'], 2)}",
        f"interesting_cpu_percent_max={fmt(interesting_summary['cpu_max'], 2)}",
        f"interesting_mem_percent_total={fmt(interesting_summary['mem_percent_total'], 2)}",
        f"interesting_mem_percent_max={fmt(interesting_summary['mem_percent_max'], 2)}",
        "interesting_rss_mib=not_captured_by_preflight_artifact",
        f"gradle_processes_observed={gradle_summary['count']}",
        f"gradle_cpu_percent_total={fmt(gradle_summary['cpu_total'], 2)}",
        f"gradle_cpu_percent_max={fmt(gradle_summary['cpu_max'], 2)}",
        f"gradle_mem_percent_total={fmt(gradle_summary['mem_percent_total'], 2)}",
        f"gradle_mem_percent_max={fmt(gradle_summary['mem_percent_max'], 2)}",
        "gradle_rss_mib=not_captured_by_preflight_artifact",
    ])
    for index, row in enumerate(foreign_rows[:10], start=1):
        lines.append(f"foreign_process_{index}={row.get('raw', '')}")
    for index, row in enumerate(interesting_rows[:10], start=1):
        lines.append(f"interesting_process_{index}={row.get('raw', '')}")

    lines.extend([
        "",
        "[summary_crosscheck]",
        f"summary_online_max={summary_values.get('online_max', 'missing')}",
        f"summary_loaded_chunks_max={summary_values.get('loaded_chunks_max', 'missing')}",
        f"summary_tps1_avg={summary_values.get('tps1_avg', 'missing')}",
        f"summary_tps1_min={summary_values.get('tps1_min', 'missing')}",
        f"summary_avg_tick_ms_avg={summary_values.get('avg_tick_ms_avg', 'missing')}",
        f"summary_avg_tick_ms_max={summary_values.get('avg_tick_ms_max', 'missing')}",
        f"summary_process_rss_mib_max={summary_values.get('process_rss_mib_max', 'missing')}",
        f"summary_bot_rss_mib_aggregate_max={summary_values.get('bot_rss_mib_aggregate_max', 'missing')}",
        f"summary_host_steal_percent_max={summary_values.get('host_cpu_steal_percent_max', 'missing')}",
        f"summary_host_iowait_percent_max={summary_values.get('host_cpu_iowait_percent_max', 'missing')}",
    ])

    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stamp", default=DEFAULT_STAMP, help="Diagnostic stamp, e.g. 20260530-105204")
    parser.add_argument("--label", help="Diagnostic label without or with load- prefix")
    parser.add_argument("--target-online", type=int, default=500)
    parser.add_argument("--resources", type=pathlib.Path)
    parser.add_argument("--server-log", type=pathlib.Path)
    parser.add_argument("--preflight", type=pathlib.Path)
    parser.add_argument("--memory-snapshot", type=pathlib.Path)
    parser.add_argument("--summary", type=pathlib.Path)
    parser.add_argument("--out", type=pathlib.Path)
    parser.add_argument("--write-report", action="store_true", help="Write the report to --out/default path")
    args = parser.parse_args()

    inputs = resolve_inputs(args)
    report = build_report(inputs, args.target_online)
    if args.write_report:
        inputs.report.parent.mkdir(parents=True, exist_ok=True)
        inputs.report.write_text(report, encoding="utf-8")
        print(str(inputs.report))
    else:
        print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
