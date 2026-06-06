#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

python3 - "$ROOT" "$TMP" <<'PY'
from __future__ import annotations

import json
import os
import sys
from pathlib import Path

root = Path(sys.argv[1])
tmp = Path(sys.argv[2])
sys.path.insert(0, str(root / "scripts"))

import evaluate_production_release as release

release.ROOT = tmp

reports = tmp / "reports"
reports.mkdir(parents=True, exist_ok=True)

opt_sha = "a" * 64
run_sha = "b" * 64
jar_sha = "c" * 64
native_sha = "e" * 64
stale_sha = "d" * 64

artifacts = {
    "optimized": {"sha256": opt_sha},
    "optimized_runtime": {
        "run_sh": {"sha256": run_sha},
        "runtime_jar_sha256_file": {"runtime_jar_sha256": jar_sha},
        "native_library": {"sha256": native_sha},
    },
}
artifacts_json = reports / "artifacts.json"
artifacts_json.write_text(json.dumps(artifacts, indent=2, sort_keys=True) + "\n", encoding="utf-8")

source_cold = root / "reports" / "load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-summary.txt"
source_warm = root / "reports" / "load-production-500-warm-block-500bots-post0097-20260516-194812-summary.txt"


def inject_hashes(src: Path, dst: Path, opt: str, run: str, jar: str, native: str) -> None:
    lines = src.read_text(encoding="utf-8", errors="replace").splitlines()
    out: list[str] = []
    inserted = False
    for line in lines:
        if line == "bot_log_tail:" and not inserted:
            out.append(f"optimized_artifact_sha256={opt}")
            out.append(f"optimized_runtime_run_sh_sha256={run}")
            out.append(f"optimized_runtime_jar_sha256={jar}")
            out.append(f"optimized_runtime_native_library_sha256={native}")
            out.append("host_cpu_windows=20")
            out.append("host_system_load1_per_cpu_max=0.650")
            out.append("host_cpu_iowait_percent_max=1.00")
            out.append("host_cpu_iowait_percent_avg=0.20")
            out.append("host_cpu_steal_percent_max=1.00")
            out.append("host_cpu_steal_percent_avg=0.10")
            out.append("bot_action_start_mode=all-ready")
            out.append("bot_action_gate_open_mode=all-ready")
            out.append("bot_action_ready_settle_ms=15000")
            out.append("bot_action_ready_requires_block_armed=true")
            out.append("bot_action_gate_opened=true")
            out.append("bot_action_ready_min_count=500")
            out.append("bot_action_ready_min_fraction=1")
            out.append("bot_action_gate_open_ready=500")
            out.append("bot_action_gate_open_active=500")
            out.append("bot_action_gate_open_settled=500")
            out.append("bot_action_gate_open_required=500")
            out.append("bot_action_gate_open_block_armed=500")
            out.append("compat_probe_block_evidence_accepted=true")
            out.append("compat_probe_direct_block_loadbot_event_lines=59000")
            out.append("compat_probe_direct_block_loadbot_place_event_lines=29500")
            out.append("compat_probe_direct_block_loadbot_break_event_lines=29500")
            out.append("compat_probe_direct_block_loadbot_cancelled_false_lines=59000")
            out.append("compat_probe_direct_block_loadbot_players=500")
            inserted = True
        out.append(line)
    if not inserted:
        out.extend(
            [
                f"optimized_artifact_sha256={opt}",
                f"optimized_runtime_run_sh_sha256={run}",
                f"optimized_runtime_jar_sha256={jar}",
                f"optimized_runtime_native_library_sha256={native}",
                "host_cpu_windows=20",
                "host_system_load1_per_cpu_max=0.650",
                "host_cpu_iowait_percent_max=1.00",
                "host_cpu_iowait_percent_avg=0.20",
                "host_cpu_steal_percent_max=1.00",
                "host_cpu_steal_percent_avg=0.10",
                "bot_action_start_mode=all-ready",
                "bot_action_gate_open_mode=all-ready",
                "bot_action_ready_settle_ms=15000",
                "bot_action_ready_requires_block_armed=true",
                "bot_action_gate_opened=true",
                "bot_action_ready_min_count=500",
                "bot_action_ready_min_fraction=1",
                "bot_action_gate_open_ready=500",
                "bot_action_gate_open_active=500",
                "bot_action_gate_open_settled=500",
                "bot_action_gate_open_required=500",
                "bot_action_gate_open_block_armed=500",
                "compat_probe_block_evidence_accepted=true",
                "compat_probe_direct_block_loadbot_event_lines=59000",
                "compat_probe_direct_block_loadbot_place_event_lines=29500",
                "compat_probe_direct_block_loadbot_break_event_lines=29500",
                "compat_probe_direct_block_loadbot_cancelled_false_lines=59000",
                "compat_probe_direct_block_loadbot_players=500",
            ]
        )
    dst.write_text("\n".join(out) + "\n", encoding="utf-8")


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def set_mtime(path: Path, seconds: int) -> None:
    os.utime(path, (seconds, seconds))


def select_cold() -> release.SummaryVerdict:
    return release.select_summary(
        "auto",
        "production-500",
        ["load-production-500-*-summary.txt"],
        artifacts,
        artifacts_json,
    )


def select_warm() -> release.SummaryVerdict:
    return release.select_summary(
        "auto",
        "production-500-warm",
        ["load-production-500-warm-*-summary.txt"],
        artifacts,
        artifacts_json,
    )


# Positive current-artifact matching.
set_mtime(artifacts_json, 1_000_000_000)
good_cold = reports / "load-production-500-cold-current-ok-summary.txt"
good_warm = reports / "load-production-500-warm-current-ok-summary.txt"
inject_hashes(source_cold, good_cold, opt_sha, run_sha, jar_sha, native_sha)
inject_hashes(source_warm, good_warm, opt_sha, run_sha, jar_sha, native_sha)
set_mtime(good_cold, 1_000_000_060)
set_mtime(good_warm, 1_000_000_070)

cold = select_cold()
warm = select_warm()
ensure(cold.passed, f"expected current-bound cold summary to pass, got: {cold.failures}")
ensure(warm.passed, f"expected current-bound warm summary to pass, got: {warm.failures}")
ensure(cold.path == good_cold, f"expected cold selector to choose {good_cold}, got {cold.path}")
ensure(warm.path == good_warm, f"expected warm selector to choose {good_warm}, got {warm.path}")

# Newer stale current summary must fail instead of falling back to the older pass.
stale_cold = reports / "load-production-500-cold-current-stale-summary.txt"
older_good_cold = reports / "load-production-500-cold-current-older-good-summary.txt"
inject_hashes(source_cold, stale_cold, stale_sha, run_sha, jar_sha, native_sha)
inject_hashes(source_cold, older_good_cold, opt_sha, run_sha, jar_sha, native_sha)
set_mtime(older_good_cold, 1_000_000_040)
set_mtime(stale_cold, 1_000_000_080)

stale_verdict = select_cold()
ensure(not stale_verdict.passed, "expected stale current cold summary to fail")
ensure(
    stale_verdict.path == stale_cold,
    f"expected stale cold selector to stop at {stale_cold}, got {stale_verdict.path}",
)
ensure(
    any("does not match current artifact" in failure for failure in stale_verdict.failures),
    f"expected current-artifact mismatch failure, got: {stale_verdict.failures}",
)

# Timestamp stale current summary must fail even when hashes match.
timestamp_stale_cold = reports / "load-production-500-cold-current-timestamp-stale-summary.txt"
older_good_timestamp = reports / "load-production-500-cold-current-timestamp-good-summary.txt"
inject_hashes(source_cold, timestamp_stale_cold, opt_sha, run_sha, jar_sha, native_sha)
inject_hashes(source_cold, older_good_timestamp, opt_sha, run_sha, jar_sha, native_sha)
set_mtime(older_good_timestamp, 1_000_000_050)
set_mtime(timestamp_stale_cold, 1_000_000_090)
set_mtime(artifacts_json, 1_000_000_120)

timestamp_verdict = select_cold()
ensure(not timestamp_verdict.passed, "expected timestamp-stale current cold summary to fail")
ensure(
    timestamp_verdict.path == timestamp_stale_cold,
    f"expected timestamp-stale selector to stop at {timestamp_stale_cold}, got {timestamp_verdict.path}",
)
ensure(
    any("summary mtime" in failure for failure in timestamp_verdict.failures),
    f"expected timestamp staleness failure, got: {timestamp_verdict.failures}",
)

print("evaluate_production_release_current_binding_smoke=PASS")
PY
