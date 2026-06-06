#!/usr/bin/env python3
from __future__ import annotations

import csv
import math
import re
import statistics
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORTS = ROOT / "reports"
LABEL = "p500-profiled-20260529-094738"


def percentile(values: list[float], p: float) -> float:
    ordered = sorted(values)
    k = (len(ordered) - 1) * p / 100
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return ordered[int(k)]
    return ordered[f] * (c - k) + ordered[c] * (k - f)


def run_jfr_views() -> list[Path]:
    jfr = REPORTS / f"{LABEL}.jfr"
    outputs: list[tuple[list[str], Path]] = [
        (["jfr", "summary", str(jfr)], REPORTS / f"{LABEL}-jfr-summary.txt"),
        (
            ["jfr", "view", "--width", "180", "--cell-height", "8", "hot-methods", str(jfr)],
            REPORTS / f"{LABEL}-jfr-hot-methods.txt",
        ),
        (
            ["jfr", "view", "--width", "180", "allocation-by-class", str(jfr)],
            REPORTS / f"{LABEL}-jfr-allocation-by-class.txt",
        ),
        (
            ["jfr", "view", "--width", "180", "--cell-height", "8", "allocation-by-site", str(jfr)],
            REPORTS / f"{LABEL}-jfr-allocation-by-site.txt",
        ),
        (
            ["jfr", "view", "--width", "180", "gc-pauses", str(jfr)],
            REPORTS / f"{LABEL}-jfr-gc-pauses.txt",
        ),
    ]
    written: list[Path] = []
    for command, output in outputs:
        result = subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True)
        output.write_text(result.stdout, encoding="utf-8")
        written.append(output)
    return written


def write_gc_summary() -> tuple[Path, int, int]:
    gc_log = REPORTS / f"gc-{LABEL}.log"
    output = REPORTS / f"{LABEL}-gc-pause-summary.txt"
    pause_re = re.compile(
        r"^\[(?P<wall>[^\]]+)\]\[(?P<uptime>[0-9.]+)s\]\[info\]\[gc\s*\] "
        r"GC\((?P<id>\d+)\) (?P<kind>Pause .*) "
        r"(?P<before>\d+)M->(?P<after>\d+)M\((?P<heap>\d+)M\) (?P<ms>[0-9.]+)ms$"
    )
    safe_re = re.compile(r'Safepoint "(?P<name>[^"]+)", .* Total: (?P<ns>\d+) ns')
    pauses: list[dict[str, str | float | int]] = []
    safepoints: list[tuple[str, float]] = []
    for line in gc_log.read_text(encoding="utf-8", errors="replace").splitlines():
        if match := pause_re.match(line):
            row: dict[str, str | float | int] = match.groupdict()
            row["ms"] = float(row["ms"])
            row["before"] = int(row["before"])
            row["after"] = int(row["after"])
            row["heap"] = int(row["heap"])
            pauses.append(row)
        elif match := safe_re.search(line):
            safepoints.append((match.group("name"), int(match.group("ns")) / 1_000_000))

    by_kind: dict[str, list[float]] = {}
    for pause in pauses:
        by_kind.setdefault(str(pause["kind"]), []).append(float(pause["ms"]))

    with output.open("w", encoding="utf-8") as handle:
        handle.write(f"GC pause summary for {LABEL}\n")
        handle.write(f"source={gc_log.relative_to(ROOT)}\n")
        handle.write("note=derived from existing GC log only; no load was run\n\n")
        handle.write(f"pause_count={len(pauses)}\n")
        if pauses:
            values = [float(pause["ms"]) for pause in pauses]
            worst = max(pauses, key=lambda pause: float(pause["ms"]))
            handle.write(f"total_pause_ms={sum(values):.3f}\n")
            handle.write(f"mean_pause_ms={statistics.mean(values):.3f}\n")
            handle.write(f"median_pause_ms={statistics.median(values):.3f}\n")
            handle.write(f"p95_pause_ms={percentile(values, 95):.3f}\n")
            handle.write(f"p99_pause_ms={percentile(values, 99):.3f}\n")
            handle.write(f"max_pause_ms={max(values):.3f}\n")
            handle.write(
                "worst_pause="
                f"GC({worst['id']}) {worst['kind']} {worst['before']}M->{worst['after']}M"
                f"({worst['heap']}M) {float(worst['ms']):.3f}ms at {worst['wall']}\n"
            )
            handle.write("\n[by_kind]\n")
            for kind, values_for_kind in sorted(by_kind.items(), key=lambda item: (-len(item[1]), item[0])):
                handle.write(
                    f"{len(values_for_kind):4d} count  total={sum(values_for_kind):9.3f}ms  "
                    f"mean={statistics.mean(values_for_kind):7.3f}ms  "
                    f"max={max(values_for_kind):7.3f}ms  {kind}\n"
                )
            handle.write("\n[top_10_pauses]\n")
            for pause in sorted(pauses, key=lambda item: float(item["ms"]), reverse=True)[:10]:
                handle.write(
                    f"GC({pause['id']}) {float(pause['ms']):8.3f}ms "
                    f"{pause['before']}M->{pause['after']}M({pause['heap']}M) "
                    f"{pause['kind']} {pause['wall']}\n"
                )
        handle.write("\n[safepoints]\n")
        handle.write(f"safepoint_count={len(safepoints)}\n")
        if safepoints:
            values = [duration for _, duration in safepoints]
            handle.write(f"total_safepoint_ms={sum(values):.3f}\n")
            handle.write(f"mean_safepoint_ms={statistics.mean(values):.3f}\n")
            handle.write(f"p99_safepoint_ms={percentile(values, 99):.3f}\n")
            handle.write(f"max_safepoint_ms={max(values):.3f}\n")
            handle.write("top_10_safepoints_ms:\n")
            for name, duration in sorted(safepoints, key=lambda item: item[1], reverse=True)[:10]:
                handle.write(f"{duration:10.3f} {name}\n")
    return output, len(pauses), len(safepoints)


def load_resource_rows() -> list[dict[str, str]]:
    with (REPORTS / f"load-{LABEL}-resources.csv").open(encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def numeric(rows: list[dict[str, str]], column: str) -> list[float]:
    values: list[float] = []
    for row in rows:
        value = row.get(column, "")
        if value:
            values.append(float(value))
    return values


def host_cpu_intervals(rows: list[dict[str, str]]) -> list[dict[str, float]]:
    intervals: list[dict[str, float]] = []
    for before, after in zip(rows, rows[1:]):
        total = float(after["host_cpu_total"]) - float(before["host_cpu_total"])
        if total <= 0:
            continue
        idle = float(after["host_cpu_idle"]) - float(before["host_cpu_idle"])
        iowait = float(after["host_cpu_iowait"]) - float(before["host_cpu_iowait"])
        steal = float(after["host_cpu_steal"]) - float(before["host_cpu_steal"])
        intervals.append(
            {
                "busy_pct": 100 * (total - idle) / total,
                "iowait_pct": 100 * iowait / total,
                "steal_pct": 100 * steal / total,
            }
        )
    return intervals


def write_resource_summary() -> tuple[Path, int, list[dict[str, float]]]:
    rows = load_resource_rows()
    intervals = host_cpu_intervals(rows)
    output = REPORTS / f"{LABEL}-resources-summary.txt"
    source = REPORTS / f"load-{LABEL}-resources.csv"
    with output.open("w", encoding="utf-8") as handle:
        handle.write(f"Resource CSV summary for {LABEL}\n")
        handle.write(f"source={source.relative_to(ROOT)}\n")
        handle.write("note=derived from existing CSV only; no load was run\n\n")
        handle.write(f"rows={len(rows)}\n")
        if rows:
            handle.write(f"first_ts_ms={rows[0].get('ts_ms')}\nlast_ts_ms={rows[-1].get('ts_ms')}\n")
        for column in [
            "pid_cpu",
            "pid_rss_kb",
            "system_load1",
            "system_mem_available_kb",
            "bot_process_count",
            "bot_rss_kb_total",
            "bot_pss_kb_total",
        ]:
            values = numeric(rows, column)
            if values:
                handle.write(
                    f"{column}: min={min(values):.2f} avg={statistics.mean(values):.2f} "
                    f"p95={percentile(values, 95):.2f} max={max(values):.2f}\n"
                )
        if rss := numeric(rows, "pid_rss_kb"):
            handle.write(f"pid_rss_mib_max={max(rss) / 1024:.1f}\n")
        for column in ["busy_pct", "iowait_pct", "steal_pct"]:
            values = [interval[column] for interval in intervals]
            if values:
                handle.write(
                    f"host_cpu_{column}: min={min(values):.2f} avg={statistics.mean(values):.2f} "
                    f"p95={percentile(values, 95):.2f} max={max(values):.2f}\n"
                )
    return output, len(rows), intervals


def write_nmt_summary() -> tuple[Path, list[tuple[str, int, int]]]:
    peak = REPORTS / f"load-{LABEL}-memory" / "peak-latest.txt"
    output = REPORTS / f"{LABEL}-nmt-summary.txt"
    text = peak.read_text(encoding="utf-8", errors="replace")
    category_re = re.compile(r"^-\s+(?P<name>[^\(]+)\(reserved=(?P<reserved>\d+)MB, committed=(?P<committed>\d+)MB\)")
    categories: list[tuple[str, int, int]] = []
    keys: dict[str, str] = {}
    for line in text.splitlines():
        if match := category_re.match(line.strip()):
            categories.append((match.group("name").strip(), int(match.group("reserved")), int(match.group("committed"))))
        if "=" in line and not line.startswith(" "):
            key, value = line.split("=", 1)
            keys[key] = value

    with output.open("w", encoding="utf-8") as handle:
        handle.write(f"NMT/memory peak summary for {LABEL}\n")
        handle.write(f"source={peak.relative_to(ROOT)}\n")
        handle.write("note=derived from existing peak snapshot only; no live jcmd was run\n\n")
        for key in ["ts_utc", "pid", "rss_kb", "rss_mib"]:
            if key in keys:
                handle.write(f"{key}={keys[key]}\n")
        if match := re.search(r"Total: reserved=(\d+)MB, committed=(\d+)MB", text):
            handle.write(f"nmt_total_reserved_mb={match.group(1)}\nnmt_total_committed_mb={match.group(2)}\n")
        if match := re.search(r"garbage-first heap\s+total (\d+)K, used (\d+)K", text):
            handle.write(f"jcmd_heap_total_kb={match.group(1)}\njcmd_heap_used_kb={match.group(2)}\n")
        if match := re.search(r"Metaspace\s+used (\d+)K, committed (\d+)K, reserved (\d+)K", text):
            handle.write(
                f"jcmd_metaspace_used_kb={match.group(1)}\n"
                f"jcmd_metaspace_committed_kb={match.group(2)}\n"
                f"jcmd_metaspace_reserved_kb={match.group(3)}\n"
            )
        handle.write("\n[nmt_top_committed_mb]\n")
        for name, reserved, committed in sorted(categories, key=lambda item: item[2], reverse=True)[:12]:
            handle.write(f"{committed:5d} committed_mb  {reserved:5d} reserved_mb  {name}\n")
    return output, categories


def write_overall_summary(
    generated: list[Path],
    pause_count: int,
    intervals: list[dict[str, float]],
    categories: list[tuple[str, int, int]],
) -> Path:
    output = REPORTS / f"{LABEL}-postprocess-summary.txt"
    gc_summary = (REPORTS / f"{LABEL}-gc-pause-summary.txt").read_text(encoding="utf-8")
    pause_line = re.search(
        r"total_pause_ms=([0-9.]+).*?p95_pause_ms=([0-9.]+).*?p99_pause_ms=([0-9.]+).*?max_pause_ms=([0-9.]+)",
        gc_summary,
        re.S,
    )
    rows = load_resource_rows()
    rss = numeric(rows, "pid_rss_kb")
    pid_cpu = numeric(rows, "pid_cpu")
    mem_dir = REPORTS / f"load-{LABEL}-memory"
    peaks: list[tuple[str, int, str]] = []
    for path in sorted(mem_dir.glob("peak-[0-9]*.txt")):
        text = path.read_text(encoding="utf-8", errors="replace")
        rss_match = re.search(r"rss_kb=(\d+)", text)
        ts_match = re.search(r"ts_utc=(\S+)", text)
        if rss_match:
            peaks.append((path.name, int(rss_match.group(1)), ts_match.group(1) if ts_match else ""))

    with output.open("w", encoding="utf-8") as handle:
        handle.write(f"P500 postprocess summary for {LABEL}\n")
        handle.write("inputs=existing reports/JFR/GC/NMT/resource artifacts only\n")
        handle.write("load_run=false\nproduction_claim=false\n\n")
        handle.write("[generated_reports]\n")
        for path in generated:
            handle.write(f"{path.relative_to(ROOT)}\n")
        handle.write("\n[key_gc]\n")
        if pause_line:
            handle.write(
                f"pause_count={pause_count} total_pause_ms={float(pause_line.group(1)):.3f} "
                f"p95_ms={float(pause_line.group(2)):.3f} "
                f"p99_ms={float(pause_line.group(3)):.3f} "
                f"max_ms={float(pause_line.group(4)):.3f}\n"
            )
        handle.write("\n[key_resources]\n")
        if pid_cpu:
            handle.write(f"pid_cpu_max={max(pid_cpu):.2f}\n")
        if rss:
            handle.write(f"pid_rss_mib_max={max(rss) / 1024:.1f}\n")
        if intervals:
            handle.write(f"host_cpu_steal_percent_max={max(item['steal_pct'] for item in intervals):.2f}\n")
            handle.write(f"host_cpu_iowait_percent_max={max(item['iowait_pct'] for item in intervals):.2f}\n")
        handle.write("\n[key_memory]\n")
        for name, rss_kb, ts_utc in peaks:
            handle.write(f"{name} rss_mib={rss_kb / 1024:.1f} ts_utc={ts_utc}\n")
        handle.write("\n[key_nmt_top_committed_mb]\n")
        for name, _, committed in sorted(categories, key=lambda item: item[2], reverse=True)[:8]:
            handle.write(f"{committed} {name}\n")
    return output


def main() -> None:
    generated = run_jfr_views()
    gc_output, pause_count, safepoint_count = write_gc_summary()
    resource_output, row_count, intervals = write_resource_summary()
    nmt_output, categories = write_nmt_summary()
    generated.extend([gc_output, resource_output, nmt_output])
    overall = write_overall_summary(generated, pause_count, intervals, categories)
    generated.append(overall)
    for path in generated:
        print(f"wrote {path.relative_to(ROOT)}")
    print(f"gc_pauses={pause_count}")
    print(f"safepoints={safepoint_count}")
    print(f"resource_rows={row_count}")
    print(f"resource_cpu_intervals={len(intervals)}")
    print(f"nmt_categories={len(categories)}")


if __name__ == "__main__":
    main()
