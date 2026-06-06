#!/usr/bin/env python3
import argparse
import os
import pathlib
import signal
import sys
import time


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Abort a load-test bot process after sustained host contention."
    )
    parser.add_argument("--reason", required=True, type=pathlib.Path)
    parser.add_argument("--bot-pid", required=True, type=int)
    parser.add_argument("--bot-pid-file", required=True, type=pathlib.Path)
    parser.add_argument("--max-load-per-cpu", required=True, type=float)
    parser.add_argument("--max-steal-percent", required=True, type=float)
    parser.add_argument("--max-iowait-percent", required=True, type=float)
    parser.add_argument("--interval", required=True, type=float)
    parser.add_argument("--bad-samples", required=True, type=int)
    parser.add_argument("--stat-path", default="/proc/stat", type=pathlib.Path)
    return parser.parse_args()


def read_cpu(stat_path: pathlib.Path) -> tuple[int, int, int]:
    with stat_path.open("r", encoding="utf-8") as handle:
        fields = handle.readline().split()[1:]
    values = [int(value) for value in fields]
    iowait = values[4] if len(values) > 4 else 0
    steal = values[7] if len(values) > 7 else 0
    total = sum(values)
    return total, iowait, steal


def terminate_bots(bot_pid: int, bot_pid_file: pathlib.Path) -> None:
    pids = [bot_pid]
    try:
        pids.extend(
            int(line.strip())
            for line in bot_pid_file.read_text(encoding="utf-8").splitlines()
            if line.strip()
        )
    except Exception:
        pass
    for pid in dict.fromkeys(pids):
        try:
            os.kill(pid, signal.SIGTERM)
        except Exception:
            pass


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except OSError:
        return False


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
            if parsed:
                return len(parsed)
        except ValueError:
            pass
    try:
        return max(1, len(os.sched_getaffinity(0)))
    except (AttributeError, OSError):
        return os.cpu_count() or 1


def main() -> int:
    args = parse_args()
    if args.interval < 0:
        print("--interval must be non-negative.", file=sys.stderr)
        return 64
    if args.bad_samples < 1:
        print("--bad-samples must be a positive integer.", file=sys.stderr)
        return 64

    cpu_count = effective_cpu_count()
    bad_samples = 0
    error_samples = 0
    try:
        previous_total, previous_iowait, previous_steal = read_cpu(args.stat_path)
    except Exception as exc:
        if not args.reason.exists():
            args.reason.write_text(
                "host_contention watcher_exception "
                "bad_samples=1 "
                f"exception={type(exc).__name__}\n",
                encoding="utf-8",
            )
        terminate_bots(args.bot_pid, args.bot_pid_file)
        return 42
    while pid_alive(args.bot_pid):
        time.sleep(args.interval)
        try:
            current_total, current_iowait, current_steal = read_cpu(args.stat_path)
            total_delta = current_total - previous_total
            iowait_delta = current_iowait - previous_iowait
            steal_delta = current_steal - previous_steal
            if total_delta <= 0 or iowait_delta < 0 or steal_delta < 0:
                bad_samples += 1
                previous_total, previous_iowait, previous_steal = (
                    current_total,
                    current_iowait,
                    current_steal,
                )
                if bad_samples >= args.bad_samples:
                    if not args.reason.exists():
                        args.reason.write_text(
                            "host_contention invalid_cpu_delta "
                            f"bad_samples={bad_samples} "
                            f"total_delta={total_delta} "
                            f"iowait_delta={iowait_delta} "
                            f"steal_delta={steal_delta}\n",
                            encoding="utf-8",
                        )
                    terminate_bots(args.bot_pid, args.bot_pid_file)
                    return 42
                continue
            iowait_percent = iowait_delta * 100.0 / total_delta
            steal_percent = steal_delta * 100.0 / total_delta
            load_per_cpu = os.getloadavg()[0] / cpu_count
            previous_total, previous_iowait, previous_steal = (
                current_total,
                current_iowait,
                current_steal,
            )
            error_samples = 0
        except Exception as exc:
            error_samples += 1
            if error_samples < args.bad_samples:
                continue
            if not args.reason.exists():
                args.reason.write_text(
                    "host_contention watcher_exception "
                    f"bad_samples={error_samples} "
                    f"exception={type(exc).__name__}\n",
                    encoding="utf-8",
                )
            terminate_bots(args.bot_pid, args.bot_pid_file)
            return 42

        bad = (
            load_per_cpu > args.max_load_per_cpu
            or steal_percent > args.max_steal_percent
            or iowait_percent > args.max_iowait_percent
        )
        bad_samples = bad_samples + 1 if bad else 0
        if bad_samples < args.bad_samples:
            continue
        if not args.reason.exists():
            args.reason.write_text(
                "host_contention "
                f"bad_samples={bad_samples} "
                f"load_per_cpu={load_per_cpu:.3f} "
                f"max_load_per_cpu={args.max_load_per_cpu:.3f} "
                f"steal_percent={steal_percent:.2f} "
                f"max_steal_percent={args.max_steal_percent:.2f} "
                f"iowait_percent={iowait_percent:.2f} "
                f"max_iowait_percent={args.max_iowait_percent:.2f}\n",
                encoding="utf-8",
            )
        terminate_bots(args.bot_pid, args.bot_pid_file)
        return 42
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
