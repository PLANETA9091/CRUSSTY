#!/usr/bin/env python3
"""Summarize a load-test memory peak snapshot.

The script reads a `peak-latest.txt` snapshot produced by
`scripts/run_load_test.sh` and prints the memory metrics that are most useful
for quick inspection or downstream automation.
"""

from __future__ import annotations

import argparse
import pathlib
import re


ROOT = pathlib.Path(__file__).resolve().parents[1]
SECTION_RE = re.compile(r"^\[(.+)\]$")
KV_RE = re.compile(r"^([^=]+)=(.*)$")
PAIR_RE = re.compile(r"^\s*([^:]+):\s*(.*)$")
INT_RE = re.compile(r"(-?\d+)")
HEAP_RE = re.compile(r"total\s+(\d+)K,\s+used\s+(\d+)K")
METASPACE_RE = re.compile(r"used\s+(\d+)K")


def resolve_snapshot(path: pathlib.Path) -> pathlib.Path:
    if path.is_dir():
        path = path / "peak-latest.txt"
    if not path.is_file():
        raise SystemExit(f"Missing snapshot file: {path}")
    return path


def relative_path(path: pathlib.Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT.resolve()))
    except ValueError:
        return str(path)


def first_int(value: str | None) -> str | None:
    if value is None:
        return None
    match = INT_RE.search(value)
    return match.group(1) if match else None


def parse_snapshot(path: pathlib.Path) -> dict[str, str]:
    path = resolve_snapshot(path)
    top: dict[str, str] = {}
    proc_status: dict[str, str] = {}
    smaps_rollup: dict[str, str] = {}
    heap_total_kb: str | None = None
    heap_used_kb: str | None = None
    metaspace_used_kb: str | None = None
    nmt_enabled = "unknown"
    section = "top"

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
                kv_match = KV_RE.match(stripped)
                if kv_match:
                    top[kv_match.group(1).strip()] = kv_match.group(2).strip()
                continue

            if section == "proc_status":
                pair_match = PAIR_RE.match(line)
                if pair_match:
                    proc_status[pair_match.group(1).strip()] = pair_match.group(2).strip()
                continue

            if section == "smaps_rollup":
                pair_match = PAIR_RE.match(line)
                if pair_match:
                    smaps_rollup[pair_match.group(1).strip()] = pair_match.group(2).strip()
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
                if lowered == "jcmd_unavailable=true":
                    nmt_enabled = "unknown"
                elif "native memory tracking is not enabled" in lowered:
                    nmt_enabled = "false"
                elif lowered.startswith("native memory tracking"):
                    nmt_enabled = "true"

    rss_kb = top.get("rss_kb")
    rss_mib = top.get("rss_mib")
    if rss_mib is None and rss_kb is not None:
        try:
            rss_mib = f"{int(rss_kb) / 1024:.1f}"
        except ValueError:
            rss_mib = None
    if rss_kb is None and rss_mib is not None:
        try:
            rss_kb = str(int(float(rss_mib) * 1024))
        except ValueError:
            rss_kb = None

    def value(source: dict[str, str], key: str) -> str:
        parsed = first_int(source.get(key))
        return parsed if parsed is not None else "missing"

    return {
        "snapshot": relative_path(path),
        "rss_kb": rss_kb or "missing",
        "rss_mib": rss_mib or "missing",
        "proc_status_VmRSS": value(proc_status, "VmRSS"),
        "proc_status_RssAnon": value(proc_status, "RssAnon"),
        "proc_status_RssFile": value(proc_status, "RssFile"),
        "proc_status_VmData": value(proc_status, "VmData"),
        "proc_status_Threads": value(proc_status, "Threads"),
        "smaps_rollup_Rss": value(smaps_rollup, "Rss"),
        "smaps_rollup_Pss": value(smaps_rollup, "Pss"),
        "smaps_rollup_Pss_Anon": value(smaps_rollup, "Pss_Anon"),
        "smaps_rollup_Pss_File": value(smaps_rollup, "Pss_File"),
        "smaps_rollup_Private_Dirty": value(smaps_rollup, "Private_Dirty"),
        "smaps_rollup_Shared_Clean": value(smaps_rollup, "Shared_Clean"),
        "jcmd_heap_used_kb": heap_used_kb or "missing",
        "jcmd_heap_total_kb": heap_total_kb or "missing",
        "jcmd_metaspace_used_kb": metaspace_used_kb or "missing",
        "nmt_enabled": nmt_enabled,
    }


def format_report(fields: dict[str, str]) -> str:
    lines = [
        "memory_peak_snapshot=true",
        f"snapshot={fields['snapshot']}",
        f"rss_kb={fields['rss_kb']}",
        f"rss_mib={fields['rss_mib']}",
        "",
        "[proc_status]",
        f"VmRSS={fields['proc_status_VmRSS']}",
        f"RssAnon={fields['proc_status_RssAnon']}",
        f"RssFile={fields['proc_status_RssFile']}",
        f"VmData={fields['proc_status_VmData']}",
        f"Threads={fields['proc_status_Threads']}",
        "",
        "[smaps_rollup]",
        f"Rss={fields['smaps_rollup_Rss']}",
        f"Pss={fields['smaps_rollup_Pss']}",
        f"Pss_Anon={fields['smaps_rollup_Pss_Anon']}",
        f"Pss_File={fields['smaps_rollup_Pss_File']}",
        f"Private_Dirty={fields['smaps_rollup_Private_Dirty']}",
        f"Shared_Clean={fields['smaps_rollup_Shared_Clean']}",
        "",
        "[jcmd_gc_heap_info]",
        f"heap_used_kb={fields['jcmd_heap_used_kb']}",
        f"heap_total_kb={fields['jcmd_heap_total_kb']}",
        f"metaspace_used_kb={fields['jcmd_metaspace_used_kb']}",
        f"nmt_enabled={fields['nmt_enabled']}",
    ]
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "snapshot",
        type=pathlib.Path,
        help="Path to a peak snapshot file or the directory containing peak-latest.txt",
    )
    args = parser.parse_args()

    fields = parse_snapshot(args.snapshot)
    print(format_report(fields), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
