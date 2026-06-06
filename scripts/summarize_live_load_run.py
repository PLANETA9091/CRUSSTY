#!/usr/bin/env python3
"""Summarize a live load run without confusing benign counters for failures."""

from __future__ import annotations

import argparse
import csv
import gzip
import json
import pathlib
import re
from dataclasses import dataclass


TOKEN_RE = re.compile(r"([A-Za-z0-9_]+)=([^\s]+)")
BOT_JOIN_RE = re.compile(r"\b(?:LoadBot|username=LoadBot)(\d+)\b")

SEND_PRESSURE_KEYS = (
    "compat_probe_send_pressure_samples",
    "compat_probe_send_pressure_players_max",
    "compat_probe_send_pressure_connections_max",
    "compat_probe_send_pressure_chunk_senders_max",
    "compat_probe_send_pending_actions_max",
    "compat_probe_send_pending_outbound_bytes_max",
    "compat_probe_send_pending_outbound_bytes_read_count_max",
    "compat_probe_send_pending_outbound_bytes_unavailable_count_max",
    "compat_probe_send_bytes_before_writable_max",
    "compat_probe_send_bytes_before_writable_read_count_max",
    "compat_probe_send_bytes_before_writable_unavailable_count_max",
    "compat_probe_send_bytes_before_unwritable_min",
    "compat_probe_send_bytes_before_unwritable_read_count_max",
    "compat_probe_send_bytes_before_unwritable_unavailable_count_max",
    "compat_probe_send_non_writable_connections_max",
    "compat_probe_chunk_send_pending_chunks_max",
    "compat_probe_chunk_send_pending_chunks_read_count_max",
    "compat_probe_chunk_send_pending_chunks_unavailable_count_max",
    "compat_probe_chunk_send_unacknowledged_batches_max",
    "compat_probe_chunk_send_batch_quota_max",
    "compat_probe_chunk_send_desired_chunks_per_tick_max",
    "compat_probe_chunk_send_max_unacknowledged_batches_max",
    "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak",
    "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_read_count_max",
    "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_unavailable_count_max",
    "compat_probe_chunk_send_channel_not_writable_skips_max",
    "compat_probe_chunk_send_channel_not_writable_skips_read_count_max",
    "compat_probe_chunk_send_channel_not_writable_skips_unavailable_count_max",
    "compat_probe_chunk_send_near_unwritable_pending_chunks_peak",
    "compat_probe_chunk_send_near_unwritable_pending_chunks_peak_read_count_max",
    "compat_probe_chunk_send_near_unwritable_pending_chunks_peak_unavailable_count_max",
    "compat_probe_chunk_send_near_unwritable_skips_max",
    "compat_probe_chunk_send_near_unwritable_skips_read_count_max",
    "compat_probe_chunk_send_near_unwritable_skips_unavailable_count_max",
)

SEND_PRESSURE_TOKEN_FIELDS = {
    "sendPressurePlayers": "compat_probe_send_pressure_players_max",
    "sendPressureConnections": "compat_probe_send_pressure_connections_max",
    "sendPressureChunkSenders": "compat_probe_send_pressure_chunk_senders_max",
    "connectionPendingActionsMax": "compat_probe_send_pending_actions_max",
    "connectionPendingOutboundBytesMax": "compat_probe_send_pending_outbound_bytes_max",
    "connectionPendingOutboundBytesReadCount": "compat_probe_send_pending_outbound_bytes_read_count_max",
    "connectionPendingOutboundBytesUnavailableCount": "compat_probe_send_pending_outbound_bytes_unavailable_count_max",
    "connectionBytesBeforeWritableMax": "compat_probe_send_bytes_before_writable_max",
    "connectionBytesBeforeWritableReadCount": "compat_probe_send_bytes_before_writable_read_count_max",
    "connectionBytesBeforeWritableUnavailableCount": "compat_probe_send_bytes_before_writable_unavailable_count_max",
    "connectionBytesBeforeUnwritableMin": "compat_probe_send_bytes_before_unwritable_min",
    "connectionBytesBeforeUnwritableReadCount": "compat_probe_send_bytes_before_unwritable_read_count_max",
    "connectionBytesBeforeUnwritableUnavailableCount": "compat_probe_send_bytes_before_unwritable_unavailable_count_max",
    "connectionNonWritable": "compat_probe_send_non_writable_connections_max",
    "chunkSenderPendingChunksMax": "compat_probe_chunk_send_pending_chunks_max",
    "chunkSenderPendingChunksReadCount": "compat_probe_chunk_send_pending_chunks_read_count_max",
    "chunkSenderPendingChunksUnavailableCount": "compat_probe_chunk_send_pending_chunks_unavailable_count_max",
    "chunkSenderUnacknowledgedBatchesMax": "compat_probe_chunk_send_unacknowledged_batches_max",
    "chunkSenderBatchQuotaMax": "compat_probe_chunk_send_batch_quota_max",
    "chunkSenderDesiredChunksPerTickMax": "compat_probe_chunk_send_desired_chunks_per_tick_max",
    "chunkSenderMaxUnacknowledgedBatchesMax": "compat_probe_chunk_send_max_unacknowledged_batches_max",
    "chunkSenderChannelNotWritablePendingChunksPeak": "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak",
    "chunkSenderChannelNotWritablePendingChunksPeakReadCount": "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_read_count_max",
    "chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount": "compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_unavailable_count_max",
    "chunkSenderChannelNotWritableSkipsMax": "compat_probe_chunk_send_channel_not_writable_skips_max",
    "chunkSenderChannelNotWritableSkipsReadCount": "compat_probe_chunk_send_channel_not_writable_skips_read_count_max",
    "chunkSenderChannelNotWritableSkipsUnavailableCount": "compat_probe_chunk_send_channel_not_writable_skips_unavailable_count_max",
    "chunkSenderNearUnwritablePendingChunksPeak": "compat_probe_chunk_send_near_unwritable_pending_chunks_peak",
    "chunkSenderNearUnwritablePendingChunksPeakReadCount": "compat_probe_chunk_send_near_unwritable_pending_chunks_peak_read_count_max",
    "chunkSenderNearUnwritablePendingChunksPeakUnavailableCount": "compat_probe_chunk_send_near_unwritable_pending_chunks_peak_unavailable_count_max",
    "chunkSenderNearUnwritableSkipsMax": "compat_probe_chunk_send_near_unwritable_skips_max",
    "chunkSenderNearUnwritableSkipsReadCount": "compat_probe_chunk_send_near_unwritable_skips_read_count_max",
    "chunkSenderNearUnwritableSkipsUnavailableCount": "compat_probe_chunk_send_near_unwritable_skips_unavailable_count_max",
}


@dataclass(frozen=True)
class ResolvedRun:
    label: str
    stem: str
    summary: pathlib.Path
    gate: pathlib.Path
    status: pathlib.Path
    resources: pathlib.Path
    server_log: pathlib.Path
    bot_log: pathlib.Path


def normalize_label(value: str) -> str:
    value = value.strip()
    return value if value.startswith("load-") else f"load-{value}"


def strip_known_suffix(name: str) -> str | None:
    suffixes = (
        "-summary.txt",
        "-gate.txt",
        "-status.json",
        "-resources.csv",
        "-bots.log",
        ".log.gz",
        ".log",
    )
    for suffix in suffixes:
        if name.endswith(suffix):
            return name[: -len(suffix)]
    return None


def stem_from_path(path: pathlib.Path) -> str | None:
    return strip_known_suffix(path.name)


def path_kind(path: pathlib.Path) -> str | None:
    name = path.name
    if name.endswith("-summary.txt"):
        return "summary"
    if name.endswith("-gate.txt"):
        return "gate"
    if name.endswith("-status.json"):
        return "status"
    if name.endswith("-resources.csv"):
        return "resources"
    if name.endswith("-bots.log"):
        return "bot_log"
    if name.endswith(".log") or name.endswith(".log.gz"):
        return "server_log"
    return None


def open_text(path: pathlib.Path):
    if path.suffix == ".gz":
        return gzip.open(path, "rt", encoding="utf-8", errors="replace")
    return path.open(encoding="utf-8", errors="replace")


def parse_tokens(line: str) -> dict[str, str]:
    return dict(TOKEN_RE.findall(line))


def parse_token_files(paths: list[pathlib.Path]) -> dict[str, str]:
    values: dict[str, str] = {}
    for path in paths:
        if not path.is_file() or path.suffix in {".csv", ".json"}:
            continue
        in_bot_tail = False
        with open_text(path) as handle:
            for line in handle:
                if line.startswith("bot_log_tail:"):
                    in_bot_tail = True
                    continue
                if in_bot_tail and not line.startswith("early_abort_reason="):
                    continue
                for key, value in TOKEN_RE.findall(line):
                    values[key] = value
    return values


def parse_status_file(path: pathlib.Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    try:
        with path.open(encoding="utf-8", errors="replace") as handle:
            payload = json.load(handle)
    except (json.JSONDecodeError, OSError):
        return {}
    values: dict[str, str] = {}
    players = payload.get("players")
    if isinstance(players, dict):
        for key in ("max", "online"):
            value = players.get(key)
            if value is not None:
                values[f"status_players_{key}"] = str(value)
    version = payload.get("version")
    if isinstance(version, dict):
        for key in ("name", "protocol"):
            value = version.get(key)
            if value is not None:
                values[f"status_version_{key}"] = str(value)
    description = payload.get("description")
    if description is not None:
        values["status_description"] = str(description)
    return values


def parse_csv_tail(path: pathlib.Path, tail_rows: int = 5) -> dict[str, str]:
    if not path.is_file():
        return {}
    with path.open(encoding="utf-8", errors="replace", newline="") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        return {}

    tail = rows[-tail_rows:]
    last = tail[-1]
    summary: dict[str, str] = {
        "resources_tail_rows": str(len(tail)),
        "resources_tail_last_ts_ms": last.get("ts_ms", ""),
        "resources_tail_last_pid_cpu": last.get("pid_cpu", ""),
        "resources_tail_last_pid_rss_mib": _kb_to_mib(last.get("pid_rss_kb")),
        "resources_tail_last_system_load1": last.get("system_load1", ""),
        "resources_tail_last_system_mem_available_kb": last.get("system_mem_available_kb", ""),
    }

    loads: list[float] = []
    cpus: list[float] = []
    rss_mib: list[float] = []
    for row in tail:
        load = _to_float(row.get("system_load1"))
        if load is not None:
            loads.append(load)
        cpu = _to_float(row.get("pid_cpu"))
        if cpu is not None:
            cpus.append(cpu)
        rss = _to_float(row.get("pid_rss_kb"))
        if rss is not None:
            rss_mib.append(rss / 1024.0)

    if loads:
        summary["resources_tail_system_load1_max"] = f"{max(loads):.2f}"
    if cpus:
        summary["resources_tail_pid_cpu_max"] = f"{max(cpus):.0f}"
    if rss_mib:
        summary["resources_tail_pid_rss_mib_max"] = f"{max(rss_mib):.1f}"
    return summary


def _to_float(value: str | None) -> float | None:
    if value is None or value == "":
        return None
    try:
        return float(value)
    except ValueError:
        return None


def _kb_to_mib(value: str | None) -> str:
    parsed = _to_float(value)
    if parsed is None:
        return ""
    return f"{parsed / 1024.0:.1f}"


def parse_join_stats(paths: list[pathlib.Path]) -> dict[str, str]:
    join_names: set[str] = set()
    join_max = 0
    for path in paths:
        if not path.is_file() or path.suffix == ".csv":
            continue
        with open_text(path) as handle:
            for line in handle:
                if "bot_player_join" in line or "joined the game" in line:
                    match = BOT_JOIN_RE.search(line)
                    if match:
                        join_names.add(f"LoadBot{int(match.group(1)):03d}")
                        join_max = max(join_max, int(match.group(1)) + 1)
                    else:
                        join_names.add(line.strip())
    values = {
        "join_events": str(len(join_names)),
        "join_max": str(join_max if join_max else len(join_names)),
    }
    return values


def parse_latest_compat_probe(paths: list[pathlib.Path]) -> dict[str, str]:
    latest: tuple[float, pathlib.Path, dict[str, str], list[dict[str, str]]] | None = None
    for path in paths:
        if not path.is_file() or path.suffix == ".csv" or path.suffix == ".json":
            continue
        try:
            mtime = path.stat().st_mtime
        except OSError:
            mtime = 0.0
        last_line_tokens: dict[str, str] | None = None
        metrics_tokens: list[dict[str, str]] = []
        with open_text(path) as handle:
            for line in handle:
                if "COMPAT_PROBE metrics" in line:
                    last_line_tokens = parse_tokens(line)
                    metrics_tokens.append(last_line_tokens)
        if last_line_tokens and (latest is None or mtime >= latest[0]):
            latest = (mtime, path, last_line_tokens, metrics_tokens)
    if latest is None:
        return {}
    tokens = latest[2]
    fields = [
        "online",
        "loadedChunks",
        "tps1",
        "tps5",
        "tps15",
        "avgTickMs",
        "usedMemMiB",
        "blockPlaces",
        "blockBreaks",
        "arenaCommands",
        "arenaPrepared",
        "arenaSkipped",
    ]
    values: dict[str, str] = {}
    for field in fields:
        if field in tokens:
            values[f"compat_probe_{field}"] = tokens[field]

    latest_metrics = latest[3]
    if latest_metrics:
        values["compat_probe_send_pressure_samples"] = str(len(latest_metrics))

        def int_max(raw_key: str) -> str | None:
            observed: list[int] = []
            for sample in latest_metrics:
                raw_value = sample.get(raw_key)
                if raw_value is None:
                    continue
                try:
                    observed.append(int(float(raw_value)))
                except ValueError:
                    continue
            return str(max(observed)) if observed else None

        def float_max(raw_key: str) -> str | None:
            observed: list[float] = []
            for sample in latest_metrics:
                raw_value = sample.get(raw_key)
                if raw_value is None:
                    continue
                try:
                    observed.append(float(raw_value))
                except ValueError:
                    continue
            return f"{max(observed):.2f}" if observed else None

        def nonnegative_min(raw_key: str) -> str | None:
            observed: list[int] = []
            for sample in latest_metrics:
                raw_value = sample.get(raw_key)
                if raw_value is None:
                    continue
                try:
                    parsed = int(float(raw_value))
                except ValueError:
                    continue
                if parsed >= 0:
                    observed.append(parsed)
            return str(min(observed)) if observed else None

        for raw_key, output_key in SEND_PRESSURE_TOKEN_FIELDS.items():
            if output_key == "compat_probe_send_bytes_before_unwritable_min":
                value = nonnegative_min(raw_key)
            elif output_key in {
                "compat_probe_chunk_send_batch_quota_max",
                "compat_probe_chunk_send_desired_chunks_per_tick_max",
            }:
                value = float_max(raw_key)
            else:
                value = int_max(raw_key)
            if value is not None:
                values[output_key] = value
    return values


def parse_strict_failures(paths: list[pathlib.Path], shard_log_dir: pathlib.Path | None = None) -> dict[str, str]:
    strict_lines = 0
    bot_errors = 0
    bot_kicks = 0
    metrics_errors_max = 0
    metrics_kicked_max = 0
    failure_reasons: list[str] = []
    seen_strict_lines: set[str] = set()

    scan_paths = list(paths)
    if shard_log_dir is not None and shard_log_dir.is_dir():
        scan_paths.extend(sorted(shard_log_dir.glob("shard-*.log")))

    unique_paths: list[pathlib.Path] = []
    seen_paths: set[pathlib.Path] = set()
    for path in scan_paths:
        resolved = path.resolve()
        if resolved in seen_paths:
            continue
        seen_paths.add(resolved)
        unique_paths.append(path)

    for path in unique_paths:
        if not path.is_file() or path.suffix == ".csv" or path.suffix == ".json":
            continue
        with open_text(path) as handle:
            for line in handle:
                if "swarm_strict_failure" in line:
                    line_key = line.strip()
                    if line_key in seen_strict_lines:
                        continue
                    seen_strict_lines.add(line_key)
                    strict_lines += 1
                    tokens = parse_tokens(line)
                    reason = tokens.get("reason") or tokens.get("detail") or tokens.get("kind")
                    if reason:
                        failure_reasons.append(reason)
                elif "bot_error " in line:
                    bot_errors += 1
                elif "bot_kick " in line:
                    bot_kicks += 1
                elif "swarm_metrics" in line:
                    tokens = parse_tokens(line)
                    metrics_errors_max = max(metrics_errors_max, int(_safe_int(tokens.get("errors"))))
                    metrics_kicked_max = max(metrics_kicked_max, int(_safe_int(tokens.get("kicked"))))
                    strict_fields = (
                        "blockActionErrors",
                        "botActionErrors",
                    )
                    for field in strict_fields:
                        metrics_errors_max = max(metrics_errors_max, int(_safe_int(tokens.get(field))))

    values = {
        "strict_failure_lines": str(strict_lines),
        "bot_error_events": str(bot_errors),
        "bot_kick_events": str(bot_kicks),
        "metrics_errors_max": str(metrics_errors_max),
        "metrics_kicked_max": str(metrics_kicked_max),
    }
    if failure_reasons:
        values["strict_failure_reasons"] = " | ".join(failure_reasons[:5])
    return values


def _safe_int(value: str | None) -> int:
    if value is None:
        return 0
    try:
        return int(float(value))
    except ValueError:
        return 0


def resolve_run(args: argparse.Namespace) -> ResolvedRun:
    explicit_paths = [pathlib.Path(item) for item in args.inputs]
    existing_paths = [path for path in explicit_paths if path.exists()]
    explicit_by_kind: dict[str, list[pathlib.Path]] = {}
    for path in existing_paths:
        kind = path_kind(path)
        if kind is not None:
            explicit_by_kind.setdefault(kind, []).append(path)

    label = normalize_label(args.label) if args.label else ""
    stems = {stem_from_path(path) for path in existing_paths if stem_from_path(path)}
    if args.label:
        if stems and (len(stems) != 1 or next(iter(stems)) != label):
            raise SystemExit(f"explicit paths do not match label {label}")
        stem = label
    elif len(stems) == 1:
        stem = next(iter(stems))
    elif len(args.inputs) == 1 and not explicit_paths[0].exists():
        stem = normalize_label(args.inputs[0])
    else:
        raise SystemExit("provide one label or one stem-consistent set of log/report paths")

    if not label:
        label = stem

    base_reports = pathlib.Path(args.reports_dir)
    base_logs = pathlib.Path(args.logs_dir)
    if args.reports_dir == "reports":
        for kind in ("summary", "gate", "status", "resources"):
            if explicit_by_kind.get(kind):
                base_reports = explicit_by_kind[kind][0].parent
                break
    if args.logs_dir == "logs":
        for kind in ("server_log", "bot_log"):
            if explicit_by_kind.get(kind):
                base_logs = explicit_by_kind[kind][0].parent
                break
    expected_summary = base_reports / f"{stem}-summary.txt"
    expected_gate = base_reports / f"{stem}-gate.txt"
    expected_status = base_reports / f"{stem}-status.json"
    expected_resources = base_reports / f"{stem}-resources.csv"
    expected_server_log = base_logs / f"{stem}.log"
    expected_bot_log = base_logs / f"{stem}-bots.log"

    def choose_path(kind: str, expected: pathlib.Path) -> pathlib.Path:
        for candidate in explicit_by_kind.get(kind, []):
            return candidate
        return expected

    summary = choose_path("summary", expected_summary)
    gate = choose_path("gate", expected_gate)
    status = choose_path("status", expected_status)
    resources = choose_path("resources", expected_resources)
    server_log = choose_path("server_log", expected_server_log)
    bot_log = choose_path("bot_log", expected_bot_log)

    return ResolvedRun(
        label=label,
        stem=stem,
        summary=summary,
        gate=gate,
        status=status,
        resources=resources,
        server_log=server_log,
        bot_log=bot_log,
    )


def emit(key: str, value: str | None) -> None:
    print(f"{key}={value if value is not None and value != '' else 'missing'}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("inputs", nargs="*", help="one label or a set of existing log/report paths")
    parser.add_argument("--label", help="explicit run label")
    parser.add_argument("--reports-dir", default="reports")
    parser.add_argument("--logs-dir", default="logs")
    args = parser.parse_args(argv)

    run = resolve_run(args)
    discovered_paths = [run.summary, run.gate, run.status, run.resources, run.server_log, run.bot_log]
    discovered_paths = [path for path in discovered_paths if path.exists()]

    token_values = parse_token_files(discovered_paths)
    token_values.update(parse_status_file(run.status))
    token_values.update(parse_join_stats(discovered_paths))
    for key, value in parse_latest_compat_probe(discovered_paths).items():
        token_values.setdefault(key, value)
    token_values.update(parse_strict_failures(discovered_paths, run.bot_log.with_suffix("")))
    token_values.update(parse_csv_tail(run.resources))

    emit("label", run.label)
    emit("stem", run.stem)
    emit("expected_summary_path", str(run.summary.resolve()))
    emit("expected_gate_path", str(run.gate.resolve()))
    emit("expected_status_path", str(run.status.resolve()))
    emit("summary_path", str(run.summary.resolve()) if run.summary.exists() else None)
    emit("gate_path", str(run.gate.resolve()) if run.gate.exists() else None)
    emit("status_path", str(run.status.resolve()) if run.status.exists() else None)
    emit("server_log_path", str(run.server_log.resolve()) if run.server_log.exists() else None)
    emit("bot_log_path", str(run.bot_log.resolve()) if run.bot_log.exists() else None)
    emit("resources_csv_path", str(run.resources.resolve()) if run.resources.exists() else None)
    emit("claim_eligible", token_values.get("claim_eligible"))
    emit("gate_pass", token_values.get("gate_pass"))
    emit("run_class", token_values.get("run_class"))
    emit("environment_invalid", token_values.get("environment_invalid"))
    emit("environment_invalid_kind", token_values.get("environment_invalid_kind"))
    emit("environment_invalid_reason", token_values.get("environment_invalid_reason"))
    emit("early_abort_reason", token_values.get("early_abort_reason"))
    emit("failure_count", token_values.get("failure_count"))

    online_max = (
        token_values.get("compat_probe_online")
        or token_values.get("status_players_online")
        or token_values.get("online_max")
        or token_values.get("load_window_online_max")
    )
    join_max = token_values.get("join_max")
    join_events = token_values.get("join_events")
    emit("online_max", online_max)
    emit("join_max", join_max)
    emit("join_events", join_events)

    compat_keys = [
        "compat_probe_online",
        "compat_probe_loadedChunks",
        "compat_probe_tps1",
        "compat_probe_tps5",
        "compat_probe_tps15",
        "compat_probe_avgTickMs",
        "compat_probe_usedMemMiB",
        "compat_probe_blockPlaces",
        "compat_probe_blockBreaks",
        "compat_probe_arenaCommands",
        "compat_probe_arenaPrepared",
        "compat_probe_arenaSkipped",
    ]
    for key in compat_keys:
        emit(key, token_values.get(key))

    for key in SEND_PRESSURE_KEYS:
        emit(key, token_values.get(key))

    emit("strict_failure_lines", token_values.get("strict_failure_lines"))
    emit("bot_error_events", token_values.get("bot_error_events"))
    emit("bot_kick_events", token_values.get("bot_kick_events"))
    emit("metrics_errors_max", token_values.get("metrics_errors_max"))
    emit("metrics_kicked_max", token_values.get("metrics_kicked_max"))
    emit("strict_failure_reasons", token_values.get("strict_failure_reasons"))

    resource_keys = [
        "resources_tail_rows",
        "resources_tail_last_ts_ms",
        "resources_tail_last_pid_cpu",
        "resources_tail_last_pid_rss_mib",
        "resources_tail_last_system_load1",
        "resources_tail_last_system_mem_available_kb",
        "resources_tail_system_load1_max",
        "resources_tail_pid_cpu_max",
        "resources_tail_pid_rss_mib_max",
    ]
    if any(key in token_values for key in resource_keys):
        for key in resource_keys:
            emit(key, token_values.get(key))
    else:
        emit("resources_tail_rows", None)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
