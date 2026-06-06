#!/usr/bin/env python3
"""Summarize native pack runner reports."""

from __future__ import annotations

import argparse
from collections import Counter
import re
import sys
from pathlib import Path


RESULT_RE = re.compile(r"^PACK_RESULT script=(?P<script>\S+) status=(?P<status>\S+)(?: duration_ms=(?P<duration>\d+))?")
START_RE = re.compile(r"^PACK_START script=(?P<script>\S+)$")
SPEEDUP_RE = re.compile(r"^[A-Za-z0-9_]+=(?P<value>[0-9]+(?:\.[0-9]+)?)x$")
HEADER_RE = re.compile(r"^(?P<key>[A-Za-z0-9_]+)=(?P<value>.*)$")
MANIFEST_RE = re.compile(r"^PACK_MANIFEST group=(?P<group>\S+) script=(?P<script>\S+)$")


def parse_int(value: str | None) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def summarize(report: Path) -> int:
    if not report.is_file():
        print(f"summary_status=FAIL reason=missing_report report={report}")
        return 1

    results: list[tuple[str, str, int]] = []
    equivalence_pass = 0
    equivalence_fail = 0
    speedup_lines = 0
    speedup_ge_one = 0
    speedup_lt_one = 0
    pack_status = ""
    headers: dict[str, str] = {}
    manifest_entries: list[tuple[str, str]] = []
    starts: list[str] = []

    for raw_line in report.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw_line.strip()
        manifest_match = MANIFEST_RE.match(line)
        if manifest_match:
            manifest_entries.append((manifest_match.group("group"), manifest_match.group("script")))
            continue

        start_match = START_RE.match(line)
        if start_match:
            starts.append(start_match.group("script"))
            continue

        match = RESULT_RE.match(line)
        if match:
            results.append(
                (
                    match.group("script"),
                    match.group("status"),
                    int(match.group("duration") or 0),
                )
            )
            continue

        if line == "equivalence=PASS":
            equivalence_pass += 1
        elif line.startswith("equivalence=FAIL"):
            equivalence_fail += 1
        elif line.startswith("pack_status="):
            pack_status = line

        header_match = HEADER_RE.match(line)
        if header_match and header_match.group("key") not in headers:
            headers[header_match.group("key")] = header_match.group("value")

        speedup_match = SPEEDUP_RE.match(line)
        if speedup_match:
            speedup_lines += 1
            if float(speedup_match.group("value")) >= 1.0:
                speedup_ge_one += 1
            else:
                speedup_lt_one += 1

    pass_count = sum(1 for _, status, _ in results if status == "PASS")
    fail_count = sum(1 for _, status, _ in results if status != "PASS")
    total_duration_ms = sum(duration for _, _, duration in results)
    slowest = max(results, key=lambda item: item[2], default=("", "NONE", 0))
    script_counter = Counter(script for script, _, _ in results)
    duplicate_scripts = sorted(script for script, count in script_counter.items() if count > 1)
    start_counter = Counter(starts)
    duplicate_starts = sorted(script for script, count in start_counter.items() if count > 1)
    declared_script_count = parse_int(headers.get("script_count"))
    all_real_expected = parse_int(headers.get("all_real_scripts_expected"))
    all_real_covered = parse_int(headers.get("all_real_scripts_covered"))
    leaf_group_count = parse_int(headers.get("leaf_group_count"))
    leaf_group_memberships = parse_int(headers.get("leaf_group_memberships"))
    manifest_counter = Counter(script for _, script in manifest_entries)
    manifest_duplicate_scripts = sorted(script for script, count in manifest_counter.items() if count > 1)
    manifest_script_to_group = {script: group for group, script in manifest_entries}
    result_scripts = [script for script, _, _ in results]
    result_script_set = set(result_scripts)
    start_script_set = set(starts)
    result_missing_start = sorted(result_script_set - start_script_set) if starts else []
    start_missing_result = sorted(start_script_set - result_script_set) if starts else []
    manifest_script_set = set(manifest_script_to_group)
    result_missing_manifest = sorted(result_script_set - manifest_script_set) if manifest_entries else []
    manifest_missing_result = sorted(manifest_script_set - result_script_set) if manifest_entries else []
    manifest_groups = sorted({group for group, _ in manifest_entries})

    print(f"summary_report={report}")
    print(f"summary_scripts={len(results)}")
    print(f"summary_pack_status_present={str(bool(pack_status)).upper()}")
    if starts:
        print(f"summary_started_scripts={len(starts)}")
        print(f"summary_duplicate_starts={len(duplicate_starts)}")
        print(f"summary_results_missing_start={len(result_missing_start)}")
        print(f"summary_started_scripts_missing_result={len(start_missing_result)}")
        print(f"summary_start_result_sets_match={str(not duplicate_starts and not result_missing_start and not start_missing_result).upper()}")
    if declared_script_count is not None:
        print(f"summary_declared_script_count={declared_script_count}")
        print(f"summary_script_count_matches_declared={str(declared_script_count == len(results)).upper()}")
    if all_real_expected is not None:
        print(f"summary_all_real_expected={all_real_expected}")
    if all_real_covered is not None:
        print(f"summary_all_real_covered={all_real_covered}")
    if all_real_expected is not None and all_real_covered is not None:
        print(f"summary_all_real_coverage_matches={str(all_real_expected == all_real_covered == len(results)).upper()}")
    if leaf_group_count is not None:
        print(f"summary_leaf_group_count={leaf_group_count}")
    if leaf_group_memberships is not None:
        print(f"summary_leaf_group_memberships={leaf_group_memberships}")
        print(f"summary_leaf_group_memberships_match_scripts={str(leaf_group_memberships == len(results)).upper()}")
    if manifest_entries:
        print(f"summary_manifest_entries={len(manifest_entries)}")
        print(f"summary_manifest_groups={len(manifest_groups)}")
        print(f"summary_manifest_duplicate_scripts={len(manifest_duplicate_scripts)}")
        print(f"summary_result_scripts_missing_manifest={len(result_missing_manifest)}")
        print(f"summary_manifest_scripts_missing_result={len(manifest_missing_result)}")
        print(f"summary_manifest_entries_match_scripts={str(len(manifest_entries) == len(results)).upper()}")
        if leaf_group_count is not None:
            print(f"summary_manifest_groups_match_leaf_count={str(len(manifest_groups) == leaf_group_count).upper()}")
    print(f"summary_duplicate_scripts={len(duplicate_scripts)}")
    for duplicate_script in duplicate_starts:
        print(f"summary_duplicate_start={duplicate_script}")
    for duplicate_script in duplicate_scripts:
        print(f"summary_duplicate_script={duplicate_script}")
    for script in result_missing_start:
        print(f"summary_result_missing_start={script}")
    for script in start_missing_result:
        print(f"summary_start_missing_result={script}")
    for duplicate_script in manifest_duplicate_scripts:
        print(f"summary_manifest_duplicate_script={duplicate_script}")
    for script in result_missing_manifest:
        print(f"summary_result_missing_manifest={script}")
    for script in manifest_missing_result:
        print(f"summary_manifest_missing_result={script}")
    print(f"summary_pass={pass_count}")
    print(f"summary_fail={fail_count}")
    print(f"summary_total_duration_ms={total_duration_ms}")
    print(f"summary_slowest_script={slowest[0]}")
    print(f"summary_slowest_duration_ms={slowest[2]}")
    print(f"summary_equivalence_pass_lines={equivalence_pass}")
    print(f"summary_equivalence_fail_lines={equivalence_fail}")
    print(f"summary_speedup_lines={speedup_lines}")
    print(f"summary_speedup_ge_1x={speedup_ge_one}")
    print(f"summary_speedup_lt_1x={speedup_lt_one}")
    if pack_status:
        print(f"summary_pack_status_line={pack_status}")

    contract_failed = False
    if not pack_status:
        contract_failed = True
    if starts and (duplicate_starts or result_missing_start or start_missing_result):
        contract_failed = True
    if declared_script_count is not None and declared_script_count != len(results):
        contract_failed = True
    if all_real_expected is not None and all_real_covered is not None:
        if all_real_expected != all_real_covered or all_real_covered != len(results):
            contract_failed = True
    if leaf_group_memberships is not None and leaf_group_memberships != len(results):
        contract_failed = True
    if manifest_entries:
        if len(manifest_entries) != len(results):
            contract_failed = True
        if leaf_group_count is not None and len(manifest_groups) != leaf_group_count:
            contract_failed = True
        if manifest_duplicate_scripts or result_missing_manifest or manifest_missing_result:
            contract_failed = True
    if duplicate_scripts:
        contract_failed = True

    if manifest_entries:
        grouped_results: dict[str, list[tuple[str, str, int]]] = {group: [] for group in manifest_groups}
        for script, status, duration in results:
            group = manifest_script_to_group.get(script)
            if group is not None:
                grouped_results.setdefault(group, []).append((script, status, duration))

        for group in manifest_groups:
            group_results = grouped_results.get(group, [])
            group_pass = sum(1 for _, status, _ in group_results if status == "PASS")
            group_fail = sum(1 for _, status, _ in group_results if status != "PASS")
            group_duration = sum(duration for _, _, duration in group_results)
            group_slowest = max(group_results, key=lambda item: item[2], default=("", "NONE", 0))
            key = group.replace("-", "_")
            print(f"summary_group_{key}_scripts={len(group_results)}")
            print(f"summary_group_{key}_pass={group_pass}")
            print(f"summary_group_{key}_fail={group_fail}")
            print(f"summary_group_{key}_duration_ms={group_duration}")
            print(f"summary_group_{key}_slowest_script={group_slowest[0]}")
            print(f"summary_group_{key}_slowest_duration_ms={group_slowest[2]}")

    if fail_count != 0 or equivalence_fail != 0 or "pack_status=PASS" not in pack_status or contract_failed:
        print("summary_status=FAIL")
        return 1

    print("summary_status=PASS")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    args = parser.parse_args(argv)
    return summarize(args.report)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
