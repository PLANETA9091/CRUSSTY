#!/usr/bin/env python3
"""Evaluate preserved repeat runs for the measured 500-bot release claim."""

from __future__ import annotations

import argparse
import datetime as dt
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]


REQUIRED_TRUE_KEYS = [
    "production_ready_claim_eligible",
    "release_gate_pass",
    "artifact_hashes_pass",
    "cold_gate_pass",
    "warm_gate_pass",
]

REQUIRED_ZERO_KEYS = [
    "failure_count",
    "cold_failure_count",
    "warm_failure_count",
    "cold_watchdog_thread_dumps",
    "cold_sync_load_stack_hits",
    "cold_stability_failures",
    "warm_watchdog_thread_dumps",
    "warm_sync_load_stack_hits",
    "warm_stability_failures",
]

REQUIRED_MATCHES = {
    "claim_text": "500-bots-production-ready-for-measured-32-32-creative-block-profile",
    "claim_scope": "cold-fresh-and-warm-source-500-bots-32-view-32-simulation-creative-block-workload",
    "cold_bots": "500",
    "warm_bots": "500",
    "cold_view_distance": "32",
    "warm_view_distance": "32",
    "cold_simulation_distance": "32",
    "warm_simulation_distance": "32",
    "cold_load_test_scenario": "block",
    "warm_load_test_scenario": "block",
    "cold_load_test_gamemode": "creative",
    "warm_load_test_gamemode": "creative",
    "cold_load_window_reached_full_online": "true",
    "warm_load_window_reached_full_online": "true",
    "cold_load_window_online_max": "500",
    "warm_load_window_online_max": "500",
}


def parse_kv(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line or "=" not in line:
                continue
            key, value = line.split("=", 1)
            values[key] = value
    return values


def resolve_repeat_dirs(raw_dirs: list[str]) -> list[pathlib.Path]:
    if raw_dirs != ["auto"]:
        dirs = []
        for raw_dir in raw_dirs:
            path = pathlib.Path(raw_dir).expanduser()
            if not path.is_absolute():
                path = ROOT / path
            dirs.append(path)
        return dirs
    return sorted((ROOT / "reports").glob("release-repeat-*"))


def run_sort_key(path: pathlib.Path) -> tuple[int, str]:
    match = re.search(r"run-(\d+)$", path.name)
    if match:
        return int(match.group(1)), path.name
    return 10**9, path.name


def discover_runs(repeat_dirs: list[pathlib.Path]) -> list[pathlib.Path]:
    runs: list[pathlib.Path] = []
    for repeat_dir in repeat_dirs:
        if not repeat_dir.is_dir():
            continue
        runs.extend(path for path in repeat_dir.glob("run-*") if path.is_dir())
    return sorted(runs, key=lambda path: (path.parent.name, run_sort_key(path)))


def is_int_zero(value: str | None) -> bool:
    if value is None:
        return False
    try:
        return int(value) == 0
    except ValueError:
        return False


def evaluate_run(run_dir: pathlib.Path) -> tuple[bool, dict[str, str], list[str]]:
    report = run_dir / "production-500-release-gate.txt"
    cold_summary = run_dir / "cold-summary.txt"
    warm_summary = run_dir / "warm-summary.txt"
    failures: list[str] = []

    if not report.is_file():
        return False, {}, [f"{report}: missing release report"]
    if not cold_summary.is_file():
        failures.append(f"{cold_summary}: missing copied cold summary")
    if not warm_summary.is_file():
        failures.append(f"{warm_summary}: missing copied warm summary")

    values = parse_kv(report)

    for key in REQUIRED_TRUE_KEYS:
        if values.get(key) != "true":
            failures.append(f"{report}: {key}={values.get(key)} expected=true")

    for key in REQUIRED_ZERO_KEYS:
        if not is_int_zero(values.get(key)):
            failures.append(f"{report}: {key}={values.get(key)} expected=0")

    for key, expected in REQUIRED_MATCHES.items():
        if values.get(key) != expected:
            failures.append(f"{report}: {key}={values.get(key)} expected={expected}")

    return not failures, values, failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repeat-dir",
        action="append",
        default=None,
        help="Repeat directory to evaluate, or auto for all reports/release-repeat-* dirs.",
    )
    parser.add_argument(
        "--min-passes",
        type=int,
        default=1,
        help="Minimum number of passing repeat runs required.",
    )
    parser.add_argument(
        "--report",
        default="reports/production-500-repeat-quorum.txt",
        help="Where to write the quorum report.",
    )
    args = parser.parse_args()

    if args.min_passes < 1:
        raise SystemExit("--min-passes must be >= 1")

    repeat_dirs = resolve_repeat_dirs(args.repeat_dir or ["auto"])
    runs = discover_runs(repeat_dirs)
    lines = [
        "repeat_profile=production-500-release-repeat",
        f"generated_at_utc={dt.datetime.now(dt.UTC).isoformat()}",
        f"required_min_passes={args.min_passes}",
        f"repeat_dir_count={len(repeat_dirs)}",
        f"repeat_run_count={len(runs)}",
    ]

    passes = 0
    failures: list[str] = []
    for index, run_dir in enumerate(runs, start=1):
        passed, values, run_failures = evaluate_run(run_dir)
        if passed:
            passes += 1
        else:
            failures.extend(run_failures)

        prefix = f"run_{index}"
        lines.append(f"{prefix}_dir={run_dir}")
        lines.append(f"{prefix}_pass={str(passed).lower()}")
        for key in [
            "production_ready_claim_eligible",
            "release_gate_pass",
            "failure_count",
            "cold_load_window_tps1_avg",
            "cold_load_window_tps1_min",
            "cold_load_window_avg_tick_ms_max",
            "cold_load_window_loaded_chunks_max",
            "warm_load_window_tps1_avg",
            "warm_load_window_tps1_min",
            "warm_load_window_avg_tick_ms_max",
            "warm_load_window_loaded_chunks_max",
            "optimized_artifact_sha256",
            "optimized_runtime_run_sh",
        ]:
            if key in values:
                lines.append(f"{prefix}_{key}={values[key]}")

    quorum_pass = passes >= args.min_passes and not failures
    lines.extend(
        [
            f"repeat_passes={passes}",
            f"repeat_failures={len(failures)}",
            f"repeat_quorum_pass={str(quorum_pass).lower()}",
        ]
    )
    for failure in failures:
        lines.append(f"repeat_failure={failure}")

    report = pathlib.Path(args.report).expanduser()
    if not report.is_absolute():
        report = ROOT / report
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines))
    return 0 if quorum_pass else 1


if __name__ == "__main__":
    sys.exit(main())
