#!/usr/bin/env python3
"""Rank measured P500 CPU/RSS evidence and thread-stack hotspots."""

from __future__ import annotations

import argparse
import json
import pathlib
import re
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from typing import Any

from summarize_memory_peak_snapshot import parse_snapshot as parse_memory_snapshot


ROOT = pathlib.Path(__file__).resolve().parents[1]

TOKEN_RE = re.compile(r"([A-Za-z0-9_]+)=([^\s]+)")
STATUS_PATH_RE = re.compile(r"^([A-Za-z0-9_]+)_status=\S+\s+path=(\S+)")
STAMP_RE = re.compile(r"(\d{8}-\d{6}(?:-[A-Za-z0-9]+)*)")
THREAD_HEADER_RE = re.compile(r'^"(?P<name>(?:\\.|[^"])*)".*\bnid=(?P<nid>\S+)')
THREAD_CPU_RE = re.compile(r"\bcpu=(?P<cpu>[0-9.]+)ms\b")
THREAD_STATE_RE = re.compile(r"java\.lang\.Thread\.State:\s+([A-Z_]+)")
FRAME_RE = re.compile(r"^\s+at\s+(.+)$")

STAMP_SUFFIXES = (
    "-thread-samples",
    "-thread-prints",
    "-summary",
    "-gate",
    "-preflight",
)

CONTEXT_KEYS = (
    ("stamp", ("p500_contended_diagnostic_stamp",)),
    ("label", ("p500_contended_diagnostic_label",)),
    ("generated_at_utc", ("p500_contended_diagnostic_generated_at_utc",)),
    ("profile", ("p500_contended_diagnostic_profile", "load_test_gate_profile")),
    ("target", ("p500_contended_diagnostic_target", "load_test_scenario")),
    ("production_claim_eligible", ("p500_contended_diagnostic_production_claim_eligible",)),
    ("non_claim_reason", ("p500_contended_diagnostic_non_claim_reason",)),
    ("exit_code", ("p500_contended_diagnostic_exit_code",)),
    ("gate_pass", ("observed_gate_pass", "gate_pass")),
    ("failure_count", ("observed_failure_count", "failure_count")),
)

METRIC_KEYS = (
    ("bot_count", ("bot_count", "bots")),
    ("duration_seconds", ("duration_seconds",)),
    ("view_distance", ("view_distance",)),
    ("simulation_distance", ("simulation_distance",)),
    ("load_window_metrics_samples", ("load_window_metrics_samples",)),
    ("load_window_online_max", ("load_window_online_max",)),
    ("load_window_loaded_chunks_max", ("load_window_loaded_chunks_max",)),
    ("load_window_tps1_avg", ("load_window_tps1_avg",)),
    ("load_window_tps1_min", ("load_window_tps1_min",)),
    ("load_window_avg_tick_ms_avg", ("load_window_avg_tick_ms_avg",)),
    ("load_window_avg_tick_ms_max", ("load_window_avg_tick_ms_max",)),
    ("process_cpu_max", ("process_cpu_max",)),
    ("process_rss_mib_max", ("process_rss_mib_max",)),
    ("host_cpu_count", ("host_cpu_count",)),
    ("host_system_load1_per_cpu_max", ("host_system_load1_per_cpu_max",)),
    ("host_cpu_steal_percent_max", ("host_cpu_steal_percent_max",)),
    ("host_cpu_steal_percent_avg", ("host_cpu_steal_percent_avg",)),
    ("host_cpu_iowait_percent_max", ("host_cpu_iowait_percent_max",)),
    ("watchdog_thread_dumps", ("watchdog_thread_dumps",)),
    ("external_thread_prints", ("external_thread_prints",)),
    ("bot_connected_max", ("bot_connected_max",)),
    ("bot_ready_max", ("bot_ready_max",)),
    ("bot_active_max", ("bot_active_max",)),
    ("bot_kicked_max", ("bot_kicked_max",)),
    ("bot_errors_max", ("bot_errors_max",)),
    ("bot_block_place_packets_max", ("bot_block_place_packets_max",)),
    ("bot_block_dig_packets_max", ("bot_block_dig_packets_max",)),
    ("bot_loadgen_loop_delay_p95_ms_max", ("bot_loadgen_loop_delay_p95_ms_max",)),
    ("bot_loadgen_elu_pct_max", ("bot_loadgen_elu_pct_max",)),
)


@dataclass
class CountArtifact:
    path: pathlib.Path
    sample_dir: pathlib.Path | None = None
    file_pattern: str | None = None
    sample_count: int | None = None
    thread_stack_count: int | None = None
    state_counts: Counter[str] = field(default_factory=Counter)
    top_frame_counts: Counter[str] = field(default_factory=Counter)
    all_frame_counts: Counter[str] = field(default_factory=Counter)
    thread_counts: Counter[str] = field(default_factory=Counter)


@dataclass
class ThreadSnapshot:
    name: str
    nid: str
    state: str | None
    cpu_ms: float | None
    frames: list[str]


@dataclass
class RawHotspots:
    file_count: int
    thread_snapshot_count: int
    state_counts: Counter[str]
    runnable_top_frame_counts: Counter[str]
    runnable_all_frame_counts: Counter[str]
    thread_cpu_ms_delta: Counter[str]


def resolve_path(raw: str | pathlib.Path) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def parse_key_values(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line:
                continue
            status_match = STATUS_PATH_RE.match(line)
            if status_match:
                values[f"{status_match.group(1)}_status_path"] = status_match.group(2)
            for key, value in TOKEN_RE.findall(line):
                values[key] = value
    return values


def first_value(maps: list[dict[str, str]], aliases: tuple[str, ...]) -> str | None:
    for values in maps:
        for alias in aliases:
            if alias in values:
                return values[alias]
            observed = f"observed_{alias}"
            if observed in values:
                return values[observed]
    return None


def trim_stamp(raw: str) -> str:
    changed = True
    while changed:
        changed = False
        for suffix in STAMP_SUFFIXES:
            if raw.endswith(suffix):
                raw = raw[: -len(suffix)]
                changed = True
    return raw


def infer_stamp(report_path: pathlib.Path, values: dict[str, str]) -> str | None:
    explicit = values.get("p500_contended_diagnostic_stamp")
    if explicit:
        return explicit
    match = STAMP_RE.search(report_path.stem)
    if not match:
        return None
    return trim_stamp(match.group(1))


def value_path(values: dict[str, str], key: str) -> pathlib.Path | None:
    raw = values.get(key) or values.get(f"{key}_status_path")
    if not raw:
        return None
    return resolve_path(raw)


def first_existing(paths: list[pathlib.Path | None]) -> pathlib.Path | None:
    for path in paths:
        if path and path.is_file():
            return path
    return None


def glob_one(directory: pathlib.Path, pattern: str) -> pathlib.Path | None:
    matches = sorted(directory.glob(pattern), key=lambda item: item.name)
    return matches[0] if matches else None


def find_related_paths(report_path: pathlib.Path, values: dict[str, str]) -> dict[str, pathlib.Path | None]:
    stamp = infer_stamp(report_path, values)
    report_dir = report_path.parent

    def related(key: str, suffix: str) -> pathlib.Path | None:
        declared = value_path(values, key)
        if declared and declared.is_file():
            return declared
        if not stamp:
            return None
        return glob_one(report_dir, f"*{stamp}*{suffix}")

    return {
        "summary_report": related("summary_report", "-summary.txt"),
        "gate_report": related("gate_report", "-gate.txt"),
        "thread_sample_json": related("thread_sample_json", "-thread-samples.json"),
        "thread_sample_report": related("thread_sample_report", "-thread-samples.txt"),
        "thread_print_json": related("thread_print_json", "-thread-prints.json"),
        "thread_print_report": related("thread_print_report", "-thread-prints.txt"),
    }


def find_memory_snapshot_path(report_path: pathlib.Path, values_maps: list[dict[str, str]]) -> pathlib.Path | None:
    candidates: list[pathlib.Path | None] = []
    for values in values_maps:
        candidates.append(value_path(values, "memory_peak_snapshot"))
        memory_dir = value_path(values, "memory_snapshot_dir")
        if memory_dir is not None:
            candidates.append(memory_dir / "peak-latest.txt")

    label = first_value(values_maps, ("p500_contended_diagnostic_label",))
    if label:
        candidates.append(report_path.parent / f"load-{label}-memory" / "peak-latest.txt")

    stamp = first_value(values_maps, ("p500_contended_diagnostic_stamp",))
    if stamp:
        candidates.extend(sorted(report_path.parent.glob(f"*{stamp}*-memory/peak-latest.txt"), key=lambda path: path.name))

    return first_existing(candidates)


def int_or_none(value: Any) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def counter_from_json(raw: Any) -> Counter[str]:
    counts: Counter[str] = Counter()
    if not isinstance(raw, dict):
        return counts
    for key, value in raw.items():
        parsed = int_or_none(value)
        if parsed is not None:
            counts[str(key)] = parsed
    return counts


def parse_count_section(path: pathlib.Path) -> CountArtifact:
    artifact = CountArtifact(path=path)
    section: str | None = None
    section_counts = {
        "states": artifact.state_counts,
        "top_frames": artifact.top_frame_counts,
        "all_frames": artifact.all_frame_counts,
        "threads": artifact.thread_counts,
    }
    values: dict[str, str] = {}

    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            stripped = line.strip()
            if not stripped:
                continue
            if stripped.startswith("[") and stripped.endswith("]"):
                section = stripped[1:-1]
                continue
            if section in section_counts:
                parts = stripped.split(None, 1)
                if len(parts) == 2:
                    count = int_or_none(parts[0])
                    if count is not None:
                        section_counts[section][parts[1]] += count
                continue
            for key, value in TOKEN_RE.findall(stripped):
                values[key] = value

    artifact.sample_count = int_or_none(values.get("sample_count"))
    artifact.thread_stack_count = int_or_none(values.get("thread_stack_count"))
    sample_dir = values.get("sample_dir")
    if sample_dir:
        artifact.sample_dir = resolve_path(sample_dir)
    artifact.file_pattern = values.get("file_pattern")
    return artifact


def load_count_artifact(json_path: pathlib.Path | None, text_path: pathlib.Path | None) -> CountArtifact | None:
    if json_path and json_path.is_file():
        data = json.loads(json_path.read_text(encoding="utf-8"))
        artifact = CountArtifact(path=json_path)
        artifact.sample_count = int_or_none(data.get("sample_count"))
        artifact.thread_stack_count = int_or_none(data.get("thread_stack_count"))
        sample_dir = data.get("sample_dir")
        if isinstance(sample_dir, str):
            artifact.sample_dir = resolve_path(sample_dir)
        file_pattern = data.get("file_pattern")
        if isinstance(file_pattern, str):
            artifact.file_pattern = file_pattern
        artifact.state_counts = counter_from_json(data.get("state_counts"))
        artifact.top_frame_counts = counter_from_json(data.get("top_frame_counts"))
        artifact.all_frame_counts = counter_from_json(data.get("all_frame_counts"))
        artifact.thread_counts = counter_from_json(data.get("thread_counts"))
        return artifact
    if text_path and text_path.is_file():
        return parse_count_section(text_path)
    return None


def natural_key(path: pathlib.Path) -> list[int | str]:
    key: list[int | str] = []
    for part in re.split(r"(\d+)", path.name):
        key.append(int(part) if part.isdigit() else part)
    return key


def parse_thread_file(path: pathlib.Path) -> list[ThreadSnapshot]:
    snapshots: list[ThreadSnapshot] = []
    current: ThreadSnapshot | None = None

    def flush() -> None:
        nonlocal current
        if current is not None:
            snapshots.append(current)
            current = None

    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            header = THREAD_HEADER_RE.match(raw)
            if header:
                flush()
                cpu_match = THREAD_CPU_RE.search(raw)
                cpu_ms = float(cpu_match.group("cpu")) if cpu_match else None
                current = ThreadSnapshot(
                    name=header.group("name").replace('\\"', '"'),
                    nid=header.group("nid"),
                    state=None,
                    cpu_ms=cpu_ms,
                    frames=[],
                )
                continue
            if current is None:
                continue
            state_match = THREAD_STATE_RE.search(raw)
            if state_match:
                current.state = state_match.group(1)
                continue
            frame_match = FRAME_RE.match(raw)
            if frame_match:
                current.frames.append(frame_match.group(1))
    flush()
    return snapshots


def raw_hotspots(sample_dir: pathlib.Path | None, file_pattern: str | None) -> RawHotspots | None:
    if not sample_dir or not sample_dir.is_dir():
        return None
    pattern = file_pattern or "thread-sample-*.txt"
    files = sorted(sample_dir.glob(pattern), key=natural_key)
    if not files:
        return None

    state_counts: Counter[str] = Counter()
    runnable_top_frame_counts: Counter[str] = Counter()
    runnable_all_frame_counts: Counter[str] = Counter()
    cpu_bounds: dict[tuple[str, str], list[float]] = {}
    thread_snapshot_count = 0

    for file_path in files:
        for snapshot in parse_thread_file(file_path):
            thread_snapshot_count += 1
            state = snapshot.state or "UNKNOWN"
            state_counts[state] += 1
            if snapshot.cpu_ms is not None:
                key = (snapshot.name, snapshot.nid)
                if key not in cpu_bounds:
                    cpu_bounds[key] = [snapshot.cpu_ms, snapshot.cpu_ms]
                else:
                    cpu_bounds[key][1] = snapshot.cpu_ms
            if state == "RUNNABLE" and snapshot.frames:
                runnable_top_frame_counts[snapshot.frames[0]] += 1
                runnable_all_frame_counts.update(snapshot.frames)

    thread_cpu_ms_delta: Counter[str] = Counter()
    for (name, _nid), (first_cpu_ms, last_cpu_ms) in cpu_bounds.items():
        delta = max(0.0, last_cpu_ms - first_cpu_ms)
        if delta > 0:
            thread_cpu_ms_delta[name] += round(delta, 2)

    return RawHotspots(
        file_count=len(files),
        thread_snapshot_count=thread_snapshot_count,
        state_counts=state_counts,
        runnable_top_frame_counts=runnable_top_frame_counts,
        runnable_all_frame_counts=runnable_all_frame_counts,
        thread_cpu_ms_delta=thread_cpu_ms_delta,
    )


def append_counts(lines: list[str], title: str, counts: Counter[str], limit: int) -> None:
    lines.append(f"[{title}]")
    if not counts:
        lines.append("missing=true")
        lines.append("")
        return
    for item, count in counts.most_common(limit):
        lines.append(f"{count}\t{item}")
    lines.append("")


def append_memory_section(lines: list[str], snapshot_path: pathlib.Path | None) -> None:
    lines.append("[memory]")
    if snapshot_path is None:
        lines.append("missing=true")
        lines.append("")
        return

    try:
        fields = parse_memory_snapshot(snapshot_path)
    except Exception as exc:  # pragma: no cover - defensive for damaged snapshots
        lines.append(f"path={snapshot_path}")
        lines.append(f"parse_error={exc.__class__.__name__}")
        lines.append("")
        return

    for key in (
        "snapshot",
        "rss_kb",
        "rss_mib",
        "proc_status_VmRSS",
        "proc_status_RssAnon",
        "proc_status_RssFile",
        "proc_status_VmData",
        "proc_status_Threads",
        "smaps_rollup_Rss",
        "smaps_rollup_Pss",
        "smaps_rollup_Pss_Anon",
        "smaps_rollup_Pss_File",
        "smaps_rollup_Private_Dirty",
        "jcmd_heap_used_kb",
        "jcmd_heap_total_kb",
        "jcmd_metaspace_used_kb",
        "nmt_enabled",
    ):
        value = fields.get(key)
        if value is not None:
            lines.append(f"{key}={value}")
    lines.append("")


def build_report(report_path: pathlib.Path, limit: int) -> str:
    primary_values = parse_key_values(report_path)
    related = find_related_paths(report_path, primary_values)

    summary_values = parse_key_values(related["summary_report"]) if related["summary_report"] else {}
    gate_values = parse_key_values(related["gate_report"]) if related["gate_report"] else {}
    context_maps = [primary_values, gate_values, summary_values]
    metric_maps = [summary_values, gate_values, primary_values]

    sample_artifact = load_count_artifact(related["thread_sample_json"], related["thread_sample_report"])
    print_artifact = load_count_artifact(related["thread_print_json"], related["thread_print_report"])
    memory_snapshot_path = find_memory_snapshot_path(report_path, [primary_values, summary_values, gate_values])

    lines = [
        "p500_hotspot_rank=true",
        f"report={report_path}",
        f"summary_report={related['summary_report'] or 'missing'}",
        f"gate_report={related['gate_report'] or 'missing'}",
        f"thread_sample_report={related['thread_sample_report'] or 'missing'}",
        f"thread_sample_json={related['thread_sample_json'] or 'missing'}",
        f"thread_print_report={related['thread_print_report'] or 'missing'}",
        f"thread_print_json={related['thread_print_json'] or 'missing'}",
        f"memory_peak_snapshot={memory_snapshot_path or 'missing'}",
        "",
        "[context]",
    ]
    for output_key, aliases in CONTEXT_KEYS:
        value = first_value(context_maps, aliases)
        if value is not None:
            lines.append(f"{output_key}={value}")
    lines.append("")

    lines.append("[metrics]")
    for output_key, aliases in METRIC_KEYS:
        value = first_value(metric_maps, aliases)
        if value is not None:
            lines.append(f"{output_key}={value}")
    lines.append("")

    append_memory_section(lines, memory_snapshot_path)

    for label, artifact, fallback_pattern in (
        ("thread_samples", sample_artifact, "thread-sample-*.txt"),
        ("thread_prints", print_artifact, "thread-print-*.txt"),
    ):
        lines.append(f"[{label}]")
        if artifact is None:
            lines.append("missing=true")
            lines.append("")
            continue
        lines.append(f"path={artifact.path}")
        if artifact.sample_dir:
            lines.append(f"sample_dir={artifact.sample_dir}")
        lines.append(f"file_pattern={artifact.file_pattern or fallback_pattern}")
        if artifact.sample_count is not None:
            lines.append(f"sample_count={artifact.sample_count}")
        if artifact.thread_stack_count is not None:
            lines.append(f"thread_stack_count={artifact.thread_stack_count}")
        if artifact.state_counts:
            state_line = " ".join(f"{state}={count}" for state, count in artifact.state_counts.most_common())
            lines.append(f"states={state_line}")
        lines.append("")

        raw = raw_hotspots(artifact.sample_dir, artifact.file_pattern or fallback_pattern)
        if raw is not None:
            lines.append(f"[{label}.raw_jstack]")
            lines.append(f"file_count={raw.file_count}")
            lines.append(f"thread_snapshot_count={raw.thread_snapshot_count}")
            state_line = " ".join(f"{state}={count}" for state, count in raw.state_counts.most_common())
            lines.append(f"states={state_line}")
            lines.append("")
            append_counts(lines, f"{label}.thread_cpu_ms_delta", raw.thread_cpu_ms_delta, limit)
            append_counts(lines, f"{label}.runnable_stack_counts", raw.runnable_top_frame_counts, limit)
            append_counts(lines, f"{label}.runnable_frame_counts", raw.runnable_all_frame_counts, limit)

        append_counts(lines, f"{label}.top_stack_counts", artifact.top_frame_counts, limit)
        append_counts(lines, f"{label}.top_frame_counts", artifact.all_frame_counts, limit)

    return "\n".join(lines).rstrip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True, type=pathlib.Path, help="P500 diagnostic report path")
    parser.add_argument("--limit", type=int, default=10, help="Rows per ranked count section")
    args = parser.parse_args()

    if args.limit < 1:
        raise SystemExit("--limit must be >= 1")

    report_path = resolve_path(args.report)
    if not report_path.is_file():
        raise SystemExit(f"Missing report: {report_path}")

    print(build_report(report_path, args.limit), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
