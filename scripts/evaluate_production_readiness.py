#!/usr/bin/env python3
"""Evaluate the top-level 500-bot production-readiness evidence bundle."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]


HARD_FAILURE_PATTERNS = [
    r"Could not load plugin",
    r"Encountered an unexpected exception",
    r"Failed to bind to port",
    r"Failed to start",
    r"UnsupportedClassVersionError",
]

SERVER_READY_PATTERN = (
    r"^(?:(?:\x1b\[[0-9;?]*[ -/]*[@-~])|[>\t\r ])*"
    r"\[[0-9]{2}:[0-9]{2}:[0-9]{2} INFO\]: "
    r"Done \([0-9.]+s\)! For help, type \"help\""
    r"(?:(?:\x1b\[[0-9;?]*[ -/]*[@-~])|[>\t\r ])*$"
)


def bool_text(value: bool) -> str:
    return str(value).lower()


def resolve_path(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def parse_kv(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line or "=" not in line:
                continue
            key, value = line.split("=", 1)
            values[key] = value
    return values


def read_text(path: pathlib.Path, failures: list[str], label: str) -> str:
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return ""
    return path.read_text(encoding="utf-8", errors="replace")


def require_value(
    values: dict[str, str],
    key: str,
    expected: str,
    failures: list[str],
    label: str,
) -> None:
    observed = values.get(key)
    if observed != expected:
        failures.append(f"{label}: {key}={observed} expected={expected}")


def require_int_min(
    values: dict[str, str],
    key: str,
    minimum: int,
    failures: list[str],
    label: str,
) -> None:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{label}: {key} is missing")
        return
    try:
        observed = int(float(raw))
    except ValueError:
        failures.append(f"{label}: {key}={raw} is not numeric")
        return
    if observed < minimum:
        failures.append(f"{label}: {key}={observed} < required {minimum}")


def require_float_min(
    values: dict[str, str],
    key: str,
    minimum: float,
    failures: list[str],
    label: str,
) -> None:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{label}: {key} is missing")
        return
    try:
        observed = float(raw)
    except ValueError:
        failures.append(f"{label}: {key}={raw} is not numeric")
        return
    if observed < minimum:
        failures.append(f"{label}: {key}={observed:.2f} < required {minimum:.2f}")


def require_float_max(
    values: dict[str, str],
    key: str,
    maximum: float,
    failures: list[str],
    label: str,
) -> None:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{label}: {key} is missing")
        return
    try:
        observed = float(raw)
    except ValueError:
        failures.append(f"{label}: {key}={raw} is not numeric")
        return
    if observed > maximum:
        failures.append(f"{label}: {key}={observed:.2f} > allowed {maximum:.2f}")


def require_pattern(
    text: str,
    pattern: str,
    failures: list[str],
    label: str,
    description: str,
) -> None:
    if not re.search(pattern, text, re.MULTILINE):
        failures.append(f"{label}: missing {description}")


def reject_pattern(
    text: str,
    pattern: str,
    failures: list[str],
    label: str,
    description: str,
) -> None:
    if re.search(pattern, text, re.MULTILINE):
        failures.append(f"{label}: found forbidden {description}")


def verify_hash_manifest(manifest: pathlib.Path) -> tuple[bool, int, list[str]]:
    failures: list[str] = []
    count = 0
    if not manifest.is_file():
        return False, 0, [f"{manifest}: missing artifact hash manifest"]
    with manifest.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line:
                continue
            parts = line.split(None, 1)
            if len(parts) != 2:
                failures.append(f"{manifest}:{line_no}: invalid sha256 line")
                continue
            expected, raw_path = parts
            file_text = raw_path.lstrip("*")
            file_path = pathlib.Path(file_text)
            if not file_path.is_absolute():
                file_path = ROOT / file_path
            count += 1
            if not file_path.is_file():
                failures.append(f"{file_path}: missing hashed artifact")
                continue
            digest = hashlib.sha256(file_path.read_bytes()).hexdigest()
            if digest != expected:
                failures.append(
                    f"{file_path}: sha256={digest} expected={expected}"
                )
    return not failures and count > 0, count, failures


def parse_hash_manifest(manifest: pathlib.Path) -> tuple[dict[pathlib.Path, str], list[str]]:
    hashes: dict[pathlib.Path, str] = {}
    failures: list[str] = []
    if not manifest.is_file():
        return hashes, [f"{manifest}: missing artifact hash manifest"]
    with manifest.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line:
                continue
            parts = line.split(None, 1)
            if len(parts) != 2:
                failures.append(f"{manifest}:{line_no}: invalid sha256 line")
                continue
            digest, raw_path = parts
            file_path = pathlib.Path(raw_path.lstrip("*"))
            if not file_path.is_absolute():
                file_path = ROOT / file_path
            hashes[file_path.resolve()] = digest
    return hashes, failures


def file_sha256(path: pathlib.Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_sha_match(
    values: dict[str, str],
    key: str,
    expected: str,
    failures: list[str],
    label: str,
) -> None:
    observed = values.get(key)
    if observed is None:
        failures.append(f"{label}: {key} is missing")
    elif observed != expected:
        failures.append(f"{label}: {key}={observed} expected_current={expected}")


def evaluate_current_artifact_consistency(
    soak_values: dict[str, str],
    repeat_values: dict[str, str],
    artifact_hashes: pathlib.Path,
) -> tuple[bool, list[str], dict[str, str]]:
    failures: list[str] = []
    details: dict[str, str] = {}
    manifest_hashes, parse_failures = parse_hash_manifest(artifact_hashes)
    failures.extend(parse_failures)
    if parse_failures:
        return False, failures, details

    optimized_path = resolve_path(
        soak_values.get(
            "optimized_artifact_path",
            "artifacts/optimized-paper-1.21.10-mojmap.jar",
        )
    ).resolve()
    optimized_sha = manifest_hashes.get(optimized_path)
    if optimized_sha is None:
        failures.append(f"current_artifact: {optimized_path} is missing from hash manifest")
    else:
        details["current_optimized_artifact_sha256"] = optimized_sha
        for key in [
            "optimized_artifact_sha256",
            "cold_optimized_artifact_sha256",
            "cold_optimized_runtime_jar_sha256",
            "warm_optimized_artifact_sha256",
            "warm_optimized_runtime_jar_sha256",
        ]:
            require_sha_match(soak_values, key, optimized_sha, failures, "soak")

    run_sh_raw = soak_values.get("optimized_runtime_run_sh")
    if run_sh_raw:
        run_sh_path = resolve_path(run_sh_raw).resolve()
        run_sh_sha = manifest_hashes.get(run_sh_path)
        if run_sh_sha is None:
            failures.append(f"current_artifact: {run_sh_path} is missing from hash manifest")
        else:
            details["current_optimized_runtime_run_sh_sha256"] = run_sh_sha
            for key in [
                "cold_optimized_runtime_run_sh_sha256",
                "warm_optimized_runtime_run_sh_sha256",
            ]:
                require_sha_match(soak_values, key, run_sh_sha, failures, "soak")
    else:
        failures.append("soak: optimized_runtime_run_sh is missing")

    native_raw = (
        soak_values.get("optimized_runtime_native_library")
        or soak_values.get("optimized_runtime_native_library_path")
        or "artifacts/optimized-runtime/native/libpaper_native_jni.so"
    )
    native_path = resolve_path(native_raw).resolve()
    native_sha = manifest_hashes.get(native_path)
    if native_sha is None:
        failures.append(f"current_artifact: {native_path} is missing from hash manifest")
    else:
        details["current_optimized_runtime_native_library_sha256"] = native_sha
        for key in [
            "optimized_runtime_native_library_sha256",
            "cold_optimized_runtime_native_library_sha256",
            "warm_optimized_runtime_native_library_sha256",
        ]:
            if key in soak_values:
                require_sha_match(soak_values, key, native_sha, failures, "soak")

    chunk_encode_native_raw = (
        soak_values.get("optimized_runtime_chunk_encode_native_library")
        or soak_values.get("optimized_runtime_chunk_encode_native_library_path")
    )
    if chunk_encode_native_raw:
        chunk_encode_native_path = resolve_path(chunk_encode_native_raw).resolve()
        chunk_encode_native_sha = manifest_hashes.get(chunk_encode_native_path)
        if chunk_encode_native_sha is None:
            failures.append(f"current_artifact: {chunk_encode_native_path} is missing from hash manifest")
        else:
            details["current_optimized_runtime_chunk_encode_native_library_path"] = str(chunk_encode_native_path)
            details["current_optimized_runtime_chunk_encode_native_library_sha256"] = chunk_encode_native_sha
            for key in [
                "optimized_runtime_chunk_encode_native_library_sha256",
                "cold_optimized_runtime_chunk_encode_native_library_sha256",
                "warm_optimized_runtime_chunk_encode_native_library_sha256",
            ]:
                if key in soak_values:
                    require_sha_match(soak_values, key, chunk_encode_native_sha, failures, "soak")

    if optimized_sha is not None:
        raw_repeat_count = repeat_values.get("repeat_run_count")
        try:
            repeat_count = int(float(raw_repeat_count)) if raw_repeat_count is not None else 0
        except ValueError:
            failures.append(f"repeat: repeat_run_count={raw_repeat_count} is not numeric")
            repeat_count = 0
        for index in range(1, repeat_count + 1):
            require_sha_match(
                repeat_values,
                f"run_{index}_optimized_artifact_sha256",
                optimized_sha,
                failures,
                "repeat",
            )

    return not failures, failures, details


def collect_go_nogo(report: pathlib.Path) -> dict[str, str]:
    go_nogo: dict[str, str] = {
        "go_nogo_report": str(report),
        "go_nogo_present": bool_text(report.is_file()),
    }
    parsed = parse_kv(report)
    for key in [
        "production_500_go_nogo_pass",
        "production_500_go_nogo_exit_code",
        "production_500_go_nogo_reason",
        "production_500_go_nogo_foreign_pattern",
        "production_500_go_nogo_canary_duration_seconds",
        "production_500_go_nogo_canary_sample_interval_seconds",
        "production_500_go_nogo_canary_max_steal_percent",
        "production_500_go_nogo_canary_max_iowait_percent",
    ]:
        if key in parsed:
            go_nogo[key] = parsed[key]
    digest = file_sha256(report)
    if digest is not None:
        go_nogo["go_nogo_report_sha256"] = digest
    return go_nogo


def evaluate_soak(values: dict[str, str], args: argparse.Namespace) -> tuple[bool, list[str]]:
    failures: list[str] = []
    label = "soak"
    for key in [
        "production_ready_soak_claim_eligible",
        "soak_gate_pass",
        "base_cold_gate_pass",
        "base_warm_gate_pass",
        "artifact_hashes_pass",
    ]:
        require_value(values, key, "true", failures, label)
    require_value(values, "failure_count", "0", failures, label)
    require_value(
        values,
        "claim_scope",
        "cold-fresh-and-warm-source-500-bots-32-view-32-simulation-creative-block-soak",
        failures,
        label,
    )

    for prefix in ["cold", "warm"]:
        side = f"{label}.{prefix}"
        require_value(values, f"{prefix}_gate_pass", "true", failures, side)
        require_value(values, f"{prefix}_failure_count", "0", failures, side)
        require_value(values, f"{prefix}_bots", "500", failures, side)
        require_value(values, f"{prefix}_view_distance", "32", failures, side)
        require_value(values, f"{prefix}_simulation_distance", "32", failures, side)
        require_value(values, f"{prefix}_load_test_scenario", "block", failures, side)
        require_value(values, f"{prefix}_load_test_gamemode", "creative", failures, side)
        require_value(values, f"{prefix}_spark_background_profiler", "false", failures, side)
        require_value(values, f"{prefix}_load_window_reached_full_online", "true", failures, side)
        require_int_min(values, f"{prefix}_load_window_metrics_samples", args.min_soak_samples, failures, side)
        require_int_min(values, f"{prefix}_load_window_online_max", 500, failures, side)
        require_int_min(values, f"{prefix}_load_window_loaded_chunks_max", args.min_loaded_chunks, failures, side)
        require_float_min(values, f"{prefix}_load_window_tps1_avg", args.min_tps_avg, failures, side)
        require_float_min(values, f"{prefix}_load_window_tps1_min", args.min_tps_min, failures, side)
        require_float_max(values, f"{prefix}_load_window_avg_tick_ms_avg", args.max_mspt_avg, failures, side)
        require_float_max(values, f"{prefix}_load_window_avg_tick_ms_max", args.max_mspt_max, failures, side)
        require_int_min(values, f"{prefix}_bot_block_place_packets_max", args.min_block_packets, failures, side)
        require_int_min(values, f"{prefix}_bot_block_dig_packets_max", args.min_block_packets, failures, side)
        require_value(values, f"{prefix}_bot_block_action_errors_max", "0", failures, side)
        require_value(values, f"{prefix}_watchdog_thread_dumps", "0", failures, side)
        require_value(values, f"{prefix}_sync_load_stack_hits", "0", failures, side)
        require_value(values, f"{prefix}_stability_failures", "0", failures, side)
    return not failures, failures


def evaluate_repeat(values: dict[str, str], min_repeat_passes: int) -> tuple[bool, list[str]]:
    failures: list[str] = []
    require_value(values, "repeat_quorum_pass", "true", failures, "repeat")
    require_value(values, "repeat_failures", "0", failures, "repeat")
    require_int_min(values, "repeat_passes", min_repeat_passes, failures, "repeat")
    require_int_min(values, "repeat_run_count", min_repeat_passes, failures, "repeat")
    return not failures, failures


def evaluate_plugin_matrix(text: str) -> tuple[bool, list[str]]:
    failures: list[str] = []
    label = "plugin_matrix"
    required = [
        (r"Initialized 11 plugins", "11 plugin initialization"),
        (r"\[LibraryProbe\] Enabling LibraryProbe", "LibraryProbe load"),
        (r"\[CompatProbe\] Enabling CompatProbe", "CompatProbe load"),
        (r"COMPAT_PROBE scheduler=async ticked=true", "async scheduler tick"),
        (r"COMPAT_PROBE scheduler=sync ticked=true", "sync scheduler tick"),
        (r"COMPAT_PROBE event=PlayerJoinEvent .*detail=CodexJoinProbe", "join event"),
        (r"COMPAT_PROBE event=PlayerQuitEvent .*detail=CodexJoinProbe", "quit event"),
        (r"COMPAT_PROBE command=ok events=4", "CompatProbe command with join/quit coverage"),
        (SERVER_READY_PATTERN, "server startup Done line"),
    ]
    for pattern, description in required:
        require_pattern(text, pattern, failures, label, description)
    for pattern in HARD_FAILURE_PATTERNS:
        reject_pattern(text, pattern, failures, label, pattern)
    return not failures, failures


def evaluate_restart_recovery(text: str) -> tuple[bool, list[str]]:
    failures: list[str] = []
    label = "restart_recovery"
    required = [
        (SERVER_READY_PATTERN, "server startup Done line"),
        (r"COMPAT_PROBE scheduler=async ticked=true", "async scheduler tick"),
        (r"COMPAT_PROBE scheduler=sync ticked=true", "sync scheduler tick"),
        (r"COMPAT_PROBE command=ok events=2", "CompatProbe command"),
        (r"Saved the game", "save-all evidence"),
    ]
    for pattern, description in required:
        require_pattern(text, pattern, failures, label, description)
    for pattern in HARD_FAILURE_PATTERNS:
        reject_pattern(text, pattern, failures, label, pattern)
    return not failures, failures


def evaluate_forced_ticket(values: dict[str, str], text: str) -> tuple[bool, list[str]]:
    failures: list[str] = []
    require_value(values, "forced_ticket_persistence", "PASS", failures, "forced_ticket")
    require_pattern(text, SERVER_READY_PATTERN, failures, "forced_ticket", "startup Done lines")
    require_pattern(text, r"Saved the game", failures, "forced_ticket", "save-all evidence")
    require_pattern(text, r"marked for force loading", failures, "forced_ticket", "force-load persistence evidence")
    for pattern in HARD_FAILURE_PATTERNS:
        reject_pattern(text, pattern, failures, "forced_ticket", pattern)
    return not failures, failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--soak-report", default="reports/production-500-soak-gate.txt")
    parser.add_argument("--repeat-report", default="reports/production-500-repeat-quorum.txt")
    parser.add_argument("--plugin-matrix-summary", default="reports/plugin-matrix-summary.txt")
    parser.add_argument("--restart-recovery-summary", default="reports/restart-recovery-summary.txt")
    parser.add_argument("--forced-ticket-summary", default="reports/forced-ticket-persistence-summary.txt")
    parser.add_argument("--artifact-hashes", default="reports/artifact-hashes.txt")
    parser.add_argument("--go-nogo-report", default="reports/production-500-go-nogo-current.txt")
    parser.add_argument("--report", default="reports/production-500-readiness-gate.txt")
    parser.add_argument("--min-repeat-passes", type=int, default=3)
    parser.add_argument("--min-soak-samples", type=int, default=300)
    parser.add_argument("--min-block-packets", type=int, default=120_000)
    parser.add_argument("--min-loaded-chunks", type=int, default=4_000)
    parser.add_argument("--min-tps-avg", type=float, default=19.50)
    parser.add_argument("--min-tps-min", type=float, default=18.00)
    parser.add_argument("--max-mspt-avg", type=float, default=50.00)
    parser.add_argument("--max-mspt-max", type=float, default=100.00)
    parser.add_argument("--skip-artifact-hash-check", action="store_true")
    args = parser.parse_args()

    if args.min_repeat_passes < 1:
        raise SystemExit("--min-repeat-passes must be >= 1")
    if args.min_soak_samples < 1 or args.min_block_packets < 1:
        raise SystemExit("soak sample and block packet minima must be >= 1")

    soak_report = resolve_path(args.soak_report)
    repeat_report = resolve_path(args.repeat_report)
    plugin_summary = resolve_path(args.plugin_matrix_summary)
    restart_summary = resolve_path(args.restart_recovery_summary)
    forced_summary = resolve_path(args.forced_ticket_summary)
    artifact_hashes = resolve_path(args.artifact_hashes)
    go_nogo_report = resolve_path(args.go_nogo_report)
    output_report = resolve_path(args.report)

    failures: list[str] = []
    soak_values = parse_kv(soak_report)
    repeat_values = parse_kv(repeat_report)
    forced_values = parse_kv(forced_summary)
    go_nogo_values = collect_go_nogo(go_nogo_report)

    if not soak_values:
        failures.append(f"soak: {soak_report} is missing or empty")
    if not repeat_values:
        failures.append(f"repeat: {repeat_report} is missing or empty")
    if not forced_values:
        failures.append(f"forced_ticket: {forced_summary} is missing or empty")

    plugin_text = read_text(plugin_summary, failures, "plugin_matrix")
    restart_text = read_text(restart_summary, failures, "restart_recovery")
    forced_text = read_text(forced_summary, failures, "forced_ticket")

    soak_pass, soak_failures = evaluate_soak(soak_values, args) if soak_values else (False, [])
    repeat_pass, repeat_failures = (
        evaluate_repeat(repeat_values, args.min_repeat_passes) if repeat_values else (False, [])
    )
    plugin_pass, plugin_failures = evaluate_plugin_matrix(plugin_text) if plugin_text else (False, [])
    restart_pass, restart_failures = evaluate_restart_recovery(restart_text) if restart_text else (False, [])
    forced_pass, forced_failures = (
        evaluate_forced_ticket(forced_values, forced_text) if forced_values and forced_text else (False, [])
    )
    artifact_consistency_pass = True
    artifact_consistency_failures: list[str] = []
    artifact_consistency_details: dict[str, str] = {}
    claim_disabled_by_unverified_artifacts = False
    if args.skip_artifact_hash_check:
        artifact_consistency_pass = False
        artifact_consistency_failures.append(
            "artifact_consistency: --skip-artifact-hash-check disables production-ready claims"
        )
        artifact_consistency_details["claim_disabled_by_unverified_artifacts"] = "true"
    else:
        artifact_consistency_pass, artifact_consistency_failures, artifact_consistency_details = (
            evaluate_current_artifact_consistency(soak_values, repeat_values, artifact_hashes)
        )

    failures.extend(soak_failures)
    failures.extend(repeat_failures)
    failures.extend(plugin_failures)
    failures.extend(restart_failures)
    failures.extend(forced_failures)
    failures.extend(artifact_consistency_failures)
    go_nogo_failures: list[str] = []
    if go_nogo_values.get("go_nogo_present") != "true":
        go_nogo_failures.append("go_nogo: production-500-go-nogo-current.txt is missing")
    if go_nogo_values.get("production_500_go_nogo_pass") != "true":
        go_nogo_failures.append(
            f"go_nogo: production_500_go_nogo_pass={go_nogo_values.get('production_500_go_nogo_pass')} expected=true"
        )
    if go_nogo_values.get("production_500_go_nogo_reason") not in {"none", None}:
        go_nogo_failures.append(
            f"go_nogo: production_500_go_nogo_reason={go_nogo_values.get('production_500_go_nogo_reason')} expected=none"
        )
    failures.extend(go_nogo_failures)

    if args.skip_artifact_hash_check:
        claim_disabled_by_unverified_artifacts = True
        hash_pass = False
        hash_count = 0
        hash_failures: list[str] = [
            "artifact_hashes: --skip-artifact-hash-check disables production-ready claims"
        ]
    else:
        hash_pass, hash_count, hash_failures = verify_hash_manifest(artifact_hashes)
    failures.extend(hash_failures)

    ready = (
        soak_pass
        and repeat_pass
        and plugin_pass
        and restart_pass
        and forced_pass
        and hash_pass
        and artifact_consistency_pass
        and not failures
    )

    lines = [
        "readiness_profile=production-500-production-ready-certification",
        f"generated_at_utc={dt.datetime.now(dt.timezone.utc).isoformat()}",
        "claim_text=500-bots-production-ready-for-measured-32-32-creative-block-profile",
        "claim_scope=500-bots-32-view-32-simulation-creative-block-cold-warm-soak-repeat-plugin-restart-forced-ticket",
        "claim_limits=not-full-paper-runtime-rust-rewrite;not-unbounded-plugin-compatibility;not-unmeasured-real-player-gameplay;not-multi-hour-soak",
        f"production_ready_500_claim={bool_text(ready)}",
        f"readiness_gate_pass={bool_text(ready)}",
        f"failure_count={len(failures)}",
        f"soak_gate_pass={bool_text(soak_pass)}",
        f"repeat_quorum_pass={bool_text(repeat_pass)}",
        f"plugin_matrix_pass={bool_text(plugin_pass)}",
        f"restart_recovery_pass={bool_text(restart_pass)}",
        f"forced_ticket_persistence_pass={bool_text(forced_pass)}",
        f"artifact_hashes_pass={bool_text(hash_pass)}",
        f"current_artifact_consistency_pass={bool_text(artifact_consistency_pass)}",
        f"claim_disabled_by_unverified_artifacts={bool_text(claim_disabled_by_unverified_artifacts)}",
        f"artifact_hash_count={hash_count}",
        f"soak_report={soak_report}",
        f"repeat_report={repeat_report}",
        f"plugin_matrix_summary={plugin_summary}",
        f"restart_recovery_summary={restart_summary}",
        f"forced_ticket_summary={forced_summary}",
        f"artifact_hash_manifest={artifact_hashes}",
        f"min_repeat_passes={args.min_repeat_passes}",
        f"min_soak_samples={args.min_soak_samples}",
        f"min_block_packets={args.min_block_packets}",
    ]

    for label, path in [
        ("soak_report", soak_report),
        ("repeat_report", repeat_report),
        ("plugin_matrix_summary", plugin_summary),
        ("restart_recovery_summary", restart_summary),
        ("forced_ticket_summary", forced_summary),
        ("artifact_hash_manifest", artifact_hashes),
    ]:
        digest = file_sha256(path)
        if digest is not None:
            lines.append(f"{label}_sha256={digest}")

    for key in [
        "optimized_artifact_sha256",
        "optimized_runtime_run_sh_sha256",
        "optimized_runtime_native_library_sha256",
        "optimized_runtime_chunk_encode_native_library_sha256",
        "cold_load_window_tps1_avg",
        "cold_load_window_tps1_min",
        "cold_load_window_avg_tick_ms_max",
        "cold_bot_block_place_packets_max",
        "cold_bot_block_dig_packets_max",
        "warm_load_window_tps1_avg",
        "warm_load_window_tps1_min",
        "warm_load_window_avg_tick_ms_max",
        "warm_bot_block_place_packets_max",
        "warm_bot_block_dig_packets_max",
    ]:
        if key in soak_values:
            lines.append(f"{key}={soak_values[key]}")
    if "repeat_passes" in repeat_values:
        lines.append(f"repeat_passes={repeat_values['repeat_passes']}")
    for key, value in artifact_consistency_details.items():
        lines.append(f"{key}={value}")
    for key, value in go_nogo_values.items():
        lines.append(f"{key}={value}")

    for failure in failures:
        lines.append(f"readiness_failure={failure}")

    output_report.parent.mkdir(parents=True, exist_ok=True)
    output_report.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines))
    return 0 if ready else 1


if __name__ == "__main__":
    sys.exit(main())
