#!/usr/bin/env python3
"""Run a short CPU canary and report host steal/iowait pressure."""

from __future__ import annotations

import argparse
import math
import multiprocessing as mp
import os
import pathlib
import statistics
import sys
import time


EXIT_PASS = 0
EXIT_FAIL = 75
EXIT_TOOL_ERROR = 70


def parse_cpu_list(value: str) -> set[int]:
    cpus: set[int] = set()
    for raw_part in value.split(","):
        part = raw_part.strip()
        if not part:
            continue
        if "-" in part:
            start_raw, end_raw = part.split("-", 1)
            start = int(start_raw)
            end = int(end_raw)
            if end < start:
                raise ValueError(value)
            cpus.update(range(start, end + 1))
        else:
            cpus.add(int(part))
    return cpus


def effective_cpu_count() -> int:
    cpuset = os.environ.get("BENCHMARK_CPUSET", "").strip()
    if cpuset:
        try:
            parsed = parse_cpu_list(cpuset)
        except ValueError:
            parsed = set()
        if parsed:
            return len(parsed)
    try:
        return max(1, len(os.sched_getaffinity(0)))
    except (AttributeError, OSError):
        return os.cpu_count() or 1


def default_workers() -> int:
    return max(1, min(8, effective_cpu_count()))


def read_cpu(path: pathlib.Path) -> tuple[int, int, int]:
    with path.open("r", encoding="utf-8") as handle:
        first = handle.readline().split()
    if not first or first[0] != "cpu" or len(first) < 5:
        raise ValueError(f"{path}: missing aggregate cpu row")
    values = [int(value) for value in first[1:]]
    total = sum(values)
    iowait = values[4] if len(values) > 4 else 0
    steal = values[7] if len(values) > 7 else 0
    return total, iowait, steal


def burn_cpu(stop: mp.Event) -> None:
    value = 0x1234_5678_9ABC_DEF0
    while not stop.is_set():
        value ^= (value << 13) & 0xFFFFFFFFFFFFFFFF
        value ^= value >> 7
        value ^= (value << 17) & 0xFFFFFFFFFFFFFFFF


def sample_windows(
    *,
    stat_path: pathlib.Path,
    duration_seconds: float,
    sample_interval_seconds: float,
) -> tuple[list[float], list[float]]:
    deadline = time.monotonic() + duration_seconds
    previous_total, previous_iowait, previous_steal = read_cpu(stat_path)
    iowait_values: list[float] = []
    steal_values: list[float] = []

    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0 and iowait_values:
            break
        time.sleep(min(sample_interval_seconds, max(0.001, remaining)))
        current_total, current_iowait, current_steal = read_cpu(stat_path)
        total_delta = current_total - previous_total
        if total_delta <= 0:
            if time.monotonic() >= deadline:
                if iowait_values:
                    break
                raise ValueError(f"{stat_path}: no cpu windows sampled")
            continue
        iowait_values.append((current_iowait - previous_iowait) * 100.0 / total_delta)
        steal_values.append((current_steal - previous_steal) * 100.0 / total_delta)
        previous_total, previous_iowait, previous_steal = current_total, current_iowait, current_steal
        if time.monotonic() >= deadline:
            break

    return iowait_values, steal_values


def write_lines(lines: list[str], report: pathlib.Path | None, append: bool) -> None:
    text = "\n".join(lines) + "\n"
    print(text, end="")
    if report is None:
        return
    report.parent.mkdir(parents=True, exist_ok=True)
    mode = "a" if append else "w"
    with report.open(mode, encoding="utf-8") as handle:
        handle.write(text)


def write_reason(path: pathlib.Path | None, reason: str) -> None:
    if path is None:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(reason.rstrip() + "\n", encoding="utf-8")


def format_float(value: float) -> str:
    if not math.isfinite(value):
        return "nan"
    return f"{value:.2f}"


def run(args: argparse.Namespace) -> int:
    lines: list[str] = [
        "host_synthetic_canary_enabled=true",
        f"host_synthetic_canary_duration_seconds={args.duration_seconds:g}",
        f"host_synthetic_canary_sample_interval_seconds={args.sample_interval_seconds:g}",
        f"host_synthetic_canary_workers={args.workers}",
        f"host_synthetic_canary_stat_path={args.stat_path}",
        f"host_synthetic_canary_max_steal_percent={args.max_steal_percent:.2f}",
        f"host_synthetic_canary_max_iowait_percent={args.max_iowait_percent:.2f}",
    ]

    stop = mp.Event()
    workers: list[mp.Process] = []
    try:
        for index in range(args.workers):
            process = mp.Process(target=burn_cpu, args=(stop,), name=f"host-canary-{index}")
            process.start()
            workers.append(process)

        iowait_values, steal_values = sample_windows(
            stat_path=args.stat_path,
            duration_seconds=args.duration_seconds,
            sample_interval_seconds=args.sample_interval_seconds,
        )
        if not iowait_values or not steal_values:
            raise ValueError("no cpu windows sampled")

        iowait_max = max(iowait_values)
        iowait_avg = statistics.fmean(iowait_values)
        steal_max = max(steal_values)
        steal_avg = statistics.fmean(steal_values)
        ok = steal_max <= args.max_steal_percent and iowait_max <= args.max_iowait_percent

        reason = (
            "host_contention_prelaunch_canary "
            f"samples={len(iowait_values)} "
            f"steal_percent_max={steal_max:.2f} max_steal_percent={args.max_steal_percent:.2f} "
            f"iowait_percent_max={iowait_max:.2f} max_iowait_percent={args.max_iowait_percent:.2f}"
        )
        lines.extend(
            [
                f"host_synthetic_canary_ok={str(ok).lower()}",
                f"host_synthetic_canary_exit_code={EXIT_PASS if ok else EXIT_FAIL}",
                f"host_synthetic_canary_samples={len(iowait_values)}",
                f"host_synthetic_canary_steal_percent_max={format_float(steal_max)}",
                f"host_synthetic_canary_steal_percent_avg={format_float(steal_avg)}",
                f"host_synthetic_canary_iowait_percent_max={format_float(iowait_max)}",
                f"host_synthetic_canary_iowait_percent_avg={format_float(iowait_avg)}",
                f"host_synthetic_canary_reason={reason.replace(' ', '_') if not ok else 'none'}",
            ]
        )
        if not ok:
            write_reason(args.reason_path, reason)
        write_lines(lines, args.report, args.append_report)
        return EXIT_PASS if ok else EXIT_FAIL
    except Exception as exc:
        reason = f"host_synthetic_canary_tool_error exception={type(exc).__name__} detail={exc}"
        lines.extend(
            [
                "host_synthetic_canary_ok=false",
                f"host_synthetic_canary_exit_code={EXIT_TOOL_ERROR}",
                f"host_synthetic_canary_tool_error={type(exc).__name__}",
                f"host_synthetic_canary_reason={reason.replace(' ', '_')}",
            ]
        )
        write_reason(args.reason_path, reason)
        write_lines(lines, args.report, args.append_report)
        return EXIT_TOOL_ERROR
    finally:
        stop.set()
        for process in workers:
            process.join(timeout=2.0)
            if process.is_alive():
                process.terminate()
                process.join(timeout=2.0)


def positive_float(raw: str) -> float:
    value = float(raw)
    if value <= 0:
        raise argparse.ArgumentTypeError("must be > 0")
    return value


def non_negative_float(raw: str) -> float:
    value = float(raw)
    if value < 0:
        raise argparse.ArgumentTypeError("must be >= 0")
    return value


def positive_int(raw: str) -> int:
    value = int(raw)
    if value < 1:
        raise argparse.ArgumentTypeError("must be >= 1")
    return value


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--duration-seconds", type=positive_float, default=15.0)
    parser.add_argument("--sample-interval-seconds", type=positive_float, default=1.0)
    parser.add_argument("--max-steal-percent", type=non_negative_float, default=10.0)
    parser.add_argument("--max-iowait-percent", type=non_negative_float, default=10.0)
    parser.add_argument("--workers", type=positive_int, default=default_workers())
    parser.add_argument("--stat-path", type=pathlib.Path, default=pathlib.Path("/proc/stat"))
    parser.add_argument("--report", type=pathlib.Path)
    parser.add_argument("--append-report", action="store_true")
    parser.add_argument("--reason-path", type=pathlib.Path)
    args = parser.parse_args()
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())
