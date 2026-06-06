#!/usr/bin/env python3
"""Show the newest P500 contended diagnostic summary/gate metrics."""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
TOKEN_RE = re.compile(r"([A-Za-z0-9_]+)=([^\s]+)")
REPORT_RE = re.compile(r"^p500-contended-diagnostic-(\d{8}-\d{6}(?:-[A-Za-z0-9]+)*)\.txt$")


def parse_tokens(path: pathlib.Path | None, *, stop_at_bot_tail: bool = False) -> dict[str, str]:
    values: dict[str, str] = {}
    if path is None or not path.is_file():
        return values

    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
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
        for raw in handle:
            line = raw.strip()
            if not line:
                continue
            if line.startswith("failure="):
                failures.append(line.removeprefix("failure="))
            elif "=" in line:
                key, value = line.split("=", 1)
                values[key] = value
    return values, failures


def latest_report(reports_dir: pathlib.Path) -> pathlib.Path | None:
    reports = list(reports_dir.glob("p500-contended-diagnostic-*.txt"))
    if not reports:
        return None

    def report_companion_exists(run: dict[str, str], key: str, suffix: str) -> int:
        value = run.get(key)
        if value:
            candidate = pathlib.Path(value)
            if not candidate.is_absolute():
                candidate = ROOT / candidate
            if candidate.is_file():
                return 1
        label = run.get("p500_contended_diagnostic_label")
        if label and (reports_dir / f"load-{label}-{suffix}.txt").is_file():
            return 1
        return 0

    def sort_key(path: pathlib.Path) -> tuple[int, int, str, int]:
        match = REPORT_RE.match(path.name)
        run = parse_tokens(path)
        if match:
            return (
                report_companion_exists(run, "summary_report", "summary"),
                report_companion_exists(run, "gate_report", "gate"),
                match.group(1),
                path.stat().st_mtime_ns,
            )
        return (0, 0, "", path.stat().st_mtime_ns)

    return max(reports, key=sort_key)


def resolve_path(value: str | None) -> pathlib.Path | None:
    if not value:
        return None
    path = pathlib.Path(value)
    return path if path.is_absolute() else ROOT / path


def companion_path(run: dict[str, str], reports_dir: pathlib.Path, suffix: str) -> pathlib.Path | None:
    label = run.get("p500_contended_diagnostic_label")
    if not label:
        return None
    return reports_dir / f"load-{label}-{suffix}.txt"


def companion_json_path(run: dict[str, str], reports_dir: pathlib.Path, suffix: str) -> pathlib.Path | None:
    label = run.get("p500_contended_diagnostic_label")
    if not label:
        return None
    return reports_dir / f"load-{label}-{suffix}.json"


def rank_sections(path: pathlib.Path | None) -> dict[str, list[tuple[int, str]]]:
    sections: dict[str, list[tuple[int, str]]] = {}
    if path is None or not path.is_file():
        return sections

    section: str | None = None
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line:
                continue
            if line.startswith("[") and line.endswith("]"):
                section = line[1:-1]
                sections.setdefault(section, [])
                continue
            if section is None:
                continue
            count, sep, item = line.partition("\t")
            if not sep:
                parts = line.split(None, 1)
                if len(parts) != 2:
                    continue
                count, item = parts
            try:
                sections[section].append((int(float(count)), item))
            except ValueError:
                continue
    return sections


def hotspot_priority(item: str) -> int | None:
    if any(
        ignored in item
        for ignored in (
            "Thread.run(",
            "Thread.runWith(",
            "ThreadPoolExecutor",
            "ForkJoinWorkerThread.run",
            "LockSupport.park",
            "Unsafe.park",
            "SynchronousQueue",
            "epollWait",
            "LinuxWatchService",
            "Reference.waitForReferencePendingList",
            "FileInputStream.readBytes",
            "CompilerThread",
            "WatchdogThread",
            "BackgroundWriterThread",
        )
    ):
        return None

    prefixes = (
        ("net.minecraft.network.", 0),
        ("net.minecraft.", 1),
        ("ca.spottedleaf.", 2),
        ("io.netty.channel.", 3),
        ("io.netty.", 4),
        ("org.spigotmc.", 5),
        ("com.mojang.", 6),
        ("java.", 7),
        ("sun.", 8),
        ("jdk.", 9),
    )
    for prefix, priority in prefixes:
        if item.startswith(prefix):
            return priority
    return None


def next_hotspot(path: pathlib.Path | None) -> tuple[str, int, str] | None:
    sections = rank_sections(path)
    candidates: list[tuple[int, int, int, str, int, str]] = []
    for section_index, section in enumerate(
        (
            "thread_prints.runnable_frame_counts",
            "thread_samples.runnable_frame_counts",
            "thread_prints.runnable_stack_counts",
            "thread_samples.runnable_stack_counts",
            "thread_prints.top_stack_counts",
            "thread_samples.top_stack_counts",
        )
    ):
        for rank_index, (count, item) in enumerate(sections.get(section, [])):
            priority = hotspot_priority(item)
            if priority is None:
                continue
            candidates.append((section_index, priority, rank_index, section, count, item))
    if not candidates:
        return None
    _, _, _, section, count, item = min(candidates)
    return section, count, item


def shown_path(path: pathlib.Path | None) -> str:
    if path is None:
        return "n/a"
    try:
        return str(path.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        return str(path)


def presence(path: pathlib.Path | None) -> str:
    if path is None:
        return "n/a"
    return "present" if path.is_file() else "missing"


def first(*values: str | None) -> str:
    for value in values:
        if value not in (None, ""):
            return value
    return "n/a"


def observed(key: str, gate: dict[str, str], summary: dict[str, str]) -> str:
    return first(gate.get(f"observed_{key}"), summary.get(key))


def lower_limit(value: str, required: str | None) -> str:
    return f"{value} (required >= {required})" if required else value


def upper_limit(value: str, allowed: str | None) -> str:
    return f"{value} (allowed <= {allowed})" if allowed else value


def millis(value: str) -> str:
    if value == "n/a":
        return value
    try:
        return f"{value} ms ({float(value) / 1000:.1f} s)"
    except ValueError:
        return value


def pass_label(value: str) -> str:
    if value == "true":
        return "PASS"
    if value == "false":
        return "FAIL"
    return "UNKNOWN"


def load_json(path: pathlib.Path | None) -> dict[str, object]:
    if path is None or not path.is_file():
        return {}
    try:
        with path.open(encoding="utf-8", errors="replace") as handle:
            value = json.load(handle)
    except (OSError, json.JSONDecodeError):
        return {}
    return value if isinstance(value, dict) else {}


def sorted_counts(summary: dict[str, object], key: str, limit: int) -> list[tuple[str, int]]:
    value = summary.get(key)
    if not isinstance(value, dict):
        return []
    items: list[tuple[str, int]] = []
    for count_key, count_value in value.items():
        try:
            items.append((str(count_key), int(count_value)))
        except (TypeError, ValueError):
            continue
    return sorted(items, key=lambda item: (-item[1], item[0]))[:limit]


def compact_counts(summary: dict[str, object], key: str, limit: int = 4) -> str:
    items = sorted_counts(summary, key, limit)
    if not items:
        return "n/a"
    return ", ".join(f"{count} {name}" for name, count in items)


def report_text(
    *,
    report_path: pathlib.Path,
    summary_path: pathlib.Path | None,
    gate_path: pathlib.Path | None,
    preflight_path: pathlib.Path | None,
    run: dict[str, str],
    summary: dict[str, str],
    gate: dict[str, str],
    failures: list[str],
    thread_sample_report_path: pathlib.Path | None,
    thread_sample_json_path: pathlib.Path | None,
    thread_print_report_path: pathlib.Path | None,
    thread_print_json_path: pathlib.Path | None,
    hotspot_rank_path: pathlib.Path | None,
    thread_samples: dict[str, object],
    thread_prints: dict[str, object],
    max_failures: int,
) -> str:
    lines: list[str] = []

    def section(name: str) -> None:
        lines.extend(["", name])

    def row(label: str, value: str) -> None:
        lines.append(f"  {label}: {value}")

    def o(key: str) -> str:
        return observed(key, gate, summary)

    gate_pass = first(gate.get("gate_pass"), run.get("observed_gate_pass"))
    claim_eligible = first(
        gate.get("claim_eligible"),
        run.get("observed_claim_eligible"),
        run.get("p500_contended_diagnostic_production_claim_eligible"),
    )
    failure_count = first(gate.get("failure_count"), run.get("observed_failure_count"))

    lines.append(f"P500 contended diagnostic: {pass_label(gate_pass)}")

    section("Run")
    row("stamp", run.get("p500_contended_diagnostic_stamp", "n/a"))
    row("generated", run.get("p500_contended_diagnostic_generated_at_utc", "n/a"))
    row("profile", first(gate.get("gate_profile"), run.get("p500_contended_diagnostic_profile")))
    row("exit_code", run.get("p500_contended_diagnostic_exit_code", "n/a"))
    row("claim_eligible", claim_eligible)
    row("non_claim", run.get("p500_contended_diagnostic_non_claim", "n/a"))

    section("Files")
    for label, path in (
        ("report", report_path),
        ("summary", summary_path),
        ("gate", gate_path),
        ("preflight", preflight_path),
        ("server_log", resolve_path(run.get("server_log"))),
        ("bot_log", resolve_path(run.get("bot_log"))),
        ("thread_samples", thread_sample_report_path),
        ("thread_sample_json", thread_sample_json_path),
        ("thread_prints", thread_print_report_path),
        ("thread_print_json", thread_print_json_path),
        ("hotspot_rank", hotspot_rank_path),
    ):
        row(label, f"{shown_path(path)} [{presence(path)}]")

    section("Gate")
    row("gate_pass", gate_pass)
    row("run_class", gate.get("run_class", "n/a"))
    row("failures", f"{failure_count} / {gate.get('requirement_count', 'n/a')} requirements")
    row("environment", f"invalid={gate.get('environment_invalid', 'n/a')} kind={gate.get('environment_invalid_kind', 'n/a')}")

    section("Load Window")
    row("online", f"{o('load_window_online_max')} / {first(gate.get('required_bots'), run.get('bot_count'))}")
    row("full_online", f"reached={o('load_window_reached_full_online')} ended_by_drop={o('load_window_ended_by_online_drop')}")
    row("loaded_chunks", lower_limit(o("load_window_loaded_chunks_max"), gate.get("required_loaded_chunks_min")))
    row(
        "tps avg/min",
        f"{lower_limit(o('load_window_tps1_avg'), gate.get('required_tps1_avg_min'))} / "
        f"{lower_limit(o('load_window_tps1_min'), gate.get('required_tps1_min_min'))}",
    )
    row(
        "tick_ms avg/max",
        f"{upper_limit(o('load_window_avg_tick_ms_avg'), gate.get('required_avg_tick_ms_avg_max'))} / "
        f"{upper_limit(o('load_window_avg_tick_ms_max'), gate.get('required_avg_tick_ms_max_max'))}",
    )
    row("samples", o("load_window_metrics_samples"))

    section("Bots And Workload")
    row(
        "created/connected/ready/active",
        " / ".join(o(key) for key in ("bot_created_max", "bot_connected_max", "bot_ready_max", "bot_active_max")),
    )
    row("kicked/errors/bot_exit", " / ".join([o("bot_kicked_max"), o("bot_errors_max"), summary.get("bot_exit", "n/a")]))
    row(
        "action_gate",
        f"opened={o('bot_action_gate_opened')} after={millis(o('bot_action_gate_opened_after_ms'))} "
        f"ready={o('bot_action_gate_open_ready')} active={o('bot_action_gate_open_active')} "
        f"resets={o('bot_action_gate_reset_events')}",
    )
    row(
        "block_packets",
        f"place={o('bot_block_place_packets_max')} dig={o('bot_block_dig_packets_max')} errors={o('bot_block_action_errors_max')}",
    )
    row(
        "block_probe",
        f"accepted={o('compat_probe_block_evidence_accepted')} places={o('compat_probe_block_places_max')} "
        f"breaks={o('compat_probe_block_breaks_max')} loadbot_lines={o('compat_probe_direct_block_loadbot_event_lines')}",
    )

    section("Send Pressure")
    row(
        "connections",
        f"players={o('compat_probe_send_pressure_players_max')} conns={o('compat_probe_send_pressure_connections_max')} "
        f"non_writable={o('compat_probe_send_non_writable_connections_max')} outbound_bytes={o('compat_probe_send_pending_outbound_bytes_max')}",
    )
    row(
        "chunk_sender",
        f"pending={o('compat_probe_chunk_send_pending_chunks_max')} unack={o('compat_probe_chunk_send_unacknowledged_batches_max')} "
        f"quota={o('compat_probe_chunk_send_batch_quota_max')} desired={o('compat_probe_chunk_send_desired_chunks_per_tick_max')} "
        f"not_writable_skips={o('compat_probe_chunk_send_channel_not_writable_skips_max')} "
        f"not_writable_pending_peak={o('compat_probe_chunk_send_channel_not_writable_pending_chunks_peak')}",
    )

    section("Host And Stability")
    row("process_cpu_max", o("process_cpu_max"))
    row("rss_mib", upper_limit(o("process_rss_mib_max"), gate.get("required_process_rss_mib_max")))
    row("host_load_per_cpu", upper_limit(o("host_system_load1_per_cpu_max"), gate.get("required_host_load_per_cpu_max")))
    row("steal max/avg", f"{upper_limit(o('host_cpu_steal_percent_max'), gate.get('required_host_steal_percent_max'))} / {o('host_cpu_steal_percent_avg')}")
    row("iowait max/avg", f"{upper_limit(o('host_cpu_iowait_percent_max'), gate.get('required_host_iowait_percent_max'))} / {o('host_cpu_iowait_percent_avg')}")
    row(
        "warnings",
        f"moved_too_quickly={o('moved_too_quickly_warnings')} watchdog={o('watchdog_thread_dumps')} "
        f"sync_load={o('sync_load_stack_hits')} nearby_players={o('nearby_players_stack_hits')} stability={o('stability_failures')}",
    )

    section("Loadgen And Thread Evidence")
    row(
        "loadgen",
        f"elu_max={o('bot_loadgen_elu_pct_max')} loop_delay_p95_max_ms={o('bot_loadgen_loop_delay_p95_ms_max')} "
        f"timer_drift_max_ms={o('bot_loadgen_timer_drift_max_ms_max')}",
    )
    row(
        "capture_config",
        f"samples={run.get('load_test_thread_samples', 'n/a')} interval_s={run.get('load_test_thread_sample_interval_seconds', 'n/a')} "
        f"start_after_s={run.get('load_test_thread_sample_start_after_seconds', 'n/a')}",
    )
    row("captured", f"thread_samples={o('diagnostic_thread_samples')} watchdog_thread_prints={o('external_thread_prints')}")
    if thread_samples:
        row(
            "sample_summary",
            f"files={thread_samples.get('sample_count', 'n/a')} stacks={thread_samples.get('thread_stack_count', 'n/a')} "
            f"states={compact_counts(thread_samples, 'state_counts')}",
        )
        for frame, count in sorted_counts(thread_samples, "top_frame_counts", 5):
            lines.append(f"  sample_top_frame: {count} {frame}")
    if thread_prints:
        row(
            "watchdog_summary",
            f"files={thread_prints.get('sample_count', 'n/a')} stacks={thread_prints.get('thread_stack_count', 'n/a')} "
            f"states={compact_counts(thread_prints, 'state_counts')}",
        )
        for frame, count in sorted_counts(thread_prints, "top_frame_counts", 5):
            lines.append(f"  watchdog_top_frame: {count} {frame}")

    hotspot = next_hotspot(hotspot_rank_path)
    if hotspot is not None:
        section("Next Hotspot")
        hotspot_section, hotspot_count, hotspot_item = hotspot
        row("artifact", f"{shown_path(hotspot_rank_path)} [{presence(hotspot_rank_path)}]")
        row("bottleneck", f"{hotspot_item} [{hotspot_section}]")
        row("ranked_count", str(hotspot_count))

    section("Failures")
    if failures:
        for failure in failures[:max_failures]:
            lines.append(f"  - {failure}")
        if len(failures) > max_failures:
            lines.append(f"  ... {len(failures) - max_failures} more")
    else:
        lines.append("  none")

    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports-dir", type=pathlib.Path, default=ROOT / "reports")
    parser.add_argument("--report", type=pathlib.Path, help="Specific p500-contended-diagnostic report.")
    parser.add_argument("--summary", type=pathlib.Path, help="Override summary file.")
    parser.add_argument("--gate", type=pathlib.Path, help="Override gate file.")
    parser.add_argument("--max-failures", type=int, default=20)
    args = parser.parse_args(argv)

    if args.max_failures < 1:
        parser.error("--max-failures must be positive")

    report_path = args.report or latest_report(args.reports_dir)
    if report_path is None:
        print(f"No P500 contended diagnostic reports found in {args.reports_dir}", file=sys.stderr)
        return 1

    run = parse_tokens(report_path)
    summary_path = args.summary or resolve_path(run.get("summary_report")) or companion_path(run, args.reports_dir, "summary")
    gate_path = args.gate or resolve_path(run.get("gate_report")) or companion_path(run, args.reports_dir, "gate")
    preflight_path = resolve_path(run.get("preflight_report")) or companion_path(run, args.reports_dir, "preflight")
    thread_sample_report_path = resolve_path(run.get("thread_sample_report")) or companion_path(run, args.reports_dir, "thread-samples")
    thread_sample_json_path = resolve_path(run.get("thread_sample_json")) or companion_json_path(run, args.reports_dir, "thread-samples")
    thread_print_report_path = resolve_path(run.get("thread_print_report")) or companion_path(run, args.reports_dir, "thread-prints")
    thread_print_json_path = resolve_path(run.get("thread_print_json")) or companion_json_path(run, args.reports_dir, "thread-prints")
    hotspot_rank_path = resolve_path(run.get("hotspot_rank_report")) or companion_path(run, args.reports_dir, "hotspot-rank")

    summary = parse_tokens(summary_path, stop_at_bot_tail=True)
    gate, failures = parse_gate(gate_path)
    thread_samples = load_json(thread_sample_json_path)
    thread_prints = load_json(thread_print_json_path)
    print(
        report_text(
            report_path=report_path,
            summary_path=summary_path,
            gate_path=gate_path,
            preflight_path=preflight_path,
            run=run,
            summary=summary,
            gate=gate,
            failures=failures,
            thread_sample_report_path=thread_sample_report_path,
            thread_sample_json_path=thread_sample_json_path,
            thread_print_report_path=thread_print_report_path,
            thread_print_json_path=thread_print_json_path,
            hotspot_rank_path=hotspot_rank_path,
            thread_samples=thread_samples,
            thread_prints=thread_prints,
            max_failures=args.max_failures,
        ),
        end="",
    )

    if not summary:
        print(f"Missing or empty summary file: {shown_path(summary_path)}", file=sys.stderr)
        return 1
    if not gate:
        print(f"Missing or empty gate file: {shown_path(gate_path)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
