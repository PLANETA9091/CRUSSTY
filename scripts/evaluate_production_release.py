#!/usr/bin/env python3
"""Build a single release verdict for the measured 500-bot production claim."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import pathlib
import sys
from dataclasses import dataclass

import evaluate_load_gate


ROOT = pathlib.Path(__file__).resolve().parents[1]

SEND_PRESSURE_KEYS = [
    "compat_probe_send_pressure_samples",
    "compat_probe_send_pressure_players_max",
    "compat_probe_send_pressure_connections_max",
    "compat_probe_send_pressure_chunk_senders_max",
    "compat_probe_send_pending_actions_max",
    "compat_probe_send_pending_outbound_bytes_max",
    "compat_probe_send_bytes_before_writable_max",
    "compat_probe_send_bytes_before_unwritable_min",
    "compat_probe_send_non_writable_connections_max",
    "compat_probe_chunk_send_pending_chunks_max",
    "compat_probe_chunk_send_unacknowledged_batches_max",
    "compat_probe_chunk_send_batch_quota_max",
    "compat_probe_chunk_send_desired_chunks_per_tick_max",
    "compat_probe_chunk_send_max_unacknowledged_batches_max",
    "compat_probe_chunk_send_channel_not_writable_skips_max",
]


@dataclass(frozen=True)
class SummaryVerdict:
    path: pathlib.Path
    profile: str
    passed: bool
    failures: list[str]
    values: dict[str, str]
    metadata: dict[str, str]


def bool_text(value: bool) -> str:
    return str(value).lower()


def resolve_path(value: str) -> pathlib.Path:
    path = pathlib.Path(value).expanduser()
    if path.is_absolute():
        return path
    return ROOT / path


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def evaluate_summary(path: pathlib.Path, profile_name: str) -> SummaryVerdict:
    profile = evaluate_load_gate.PROFILES[profile_name]
    values = evaluate_load_gate.parse_summary(path)
    passed, failures, metadata = evaluate_load_gate.evaluate(
        values,
        profile,
        min_bots_override=None,
        min_loaded_chunks_override=None,
        min_tps1_avg_override=None,
        min_tps1_min_override=None,
        max_avg_tick_ms_avg_override=None,
        max_avg_tick_ms_max_override=None,
        max_rss_mib_override=None,
    )
    return SummaryVerdict(path, profile_name, passed, failures, values, metadata)


def latest_matching(patterns: list[str]) -> list[pathlib.Path]:
    paths: set[pathlib.Path] = set()
    for pattern in patterns:
        paths.update((ROOT / "reports").glob(pattern))
    return sorted(paths, key=lambda path: path.stat().st_mtime, reverse=True)


def profile_candidates(candidates: list[pathlib.Path], profile_name: str) -> list[pathlib.Path]:
    if profile_name == "production-500":
        return [path for path in candidates if "-warm-" not in path.name]
    return candidates


def format_epoch(seconds: float) -> str:
    return dt.datetime.fromtimestamp(seconds, dt.timezone.utc).isoformat()


def summary_with_failures(verdict: SummaryVerdict, failures: list[str]) -> SummaryVerdict:
    return SummaryVerdict(
        verdict.path,
        verdict.profile,
        False,
        failures,
        verdict.values,
        verdict.metadata,
    )


def evaluate_or_failure(candidate: pathlib.Path, profile_name: str) -> SummaryVerdict:
    try:
        return evaluate_summary(candidate, profile_name)
    except Exception as exc:  # Keep auto-discovery robust against partial logs.
        return SummaryVerdict(
            candidate,
            profile_name,
            False,
            [f"could not evaluate summary: {exc}"],
            {},
            {},
        )


def select_summary(
    requested: str,
    profile_name: str,
    patterns: list[str],
    current_artifacts: dict[str, object] | None = None,
    current_artifacts_path: pathlib.Path | None = None,
) -> SummaryVerdict:
    if requested in {"skip", "skipped"}:
        return SummaryVerdict(
            pathlib.Path("skipped"),
            profile_name,
            False,
            [f"{profile_name} summary skipped by release wrapper"],
            {},
            {"skipped": "true"},
        )

    if requested != "auto":
        path = resolve_path(requested)
        if not path.is_file():
            raise SystemExit(f"Missing {profile_name} summary: {path}")
        return evaluate_summary(path, profile_name)

    candidates = latest_matching(patterns)
    candidates = profile_candidates(candidates, profile_name)
    if not candidates:
        raise SystemExit(
            f"No candidate summaries found for {profile_name}; searched: {', '.join(patterns)}"
        )

    if current_artifacts is not None:
        newest = evaluate_or_failure(candidates[0], profile_name)
        binding_failures = check_current_artifact_binding(
            newest,
            current_artifacts,
            current_artifacts_path,
        )
        if binding_failures:
            return summary_with_failures(
                newest,
                [f"newest {profile_name} summary is not bound to current artifacts"]
                + binding_failures
                + newest.failures,
            )
        if newest.passed:
            return newest
        return summary_with_failures(
            newest,
            [f"newest current-artifact {profile_name} summary did not pass"]
            + newest.failures,
        )

    evaluated: list[SummaryVerdict] = []
    for candidate in candidates:
        verdict = evaluate_or_failure(candidate, profile_name)
        if not verdict.values:
            evaluated.append(verdict)
            continue
        evaluated.append(verdict)
        if verdict.passed:
            return verdict

    newest = evaluated[0]
    return SummaryVerdict(
        newest.path,
        newest.profile,
        False,
        [f"no passing {profile_name} summary found"] + newest.failures,
        newest.values,
        newest.metadata,
    )


def verify_artifact_hashes(path: pathlib.Path) -> tuple[bool, int, list[str]]:
    failures: list[str] = []
    count = 0
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_number, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped:
                continue
            parts = stripped.split(maxsplit=1)
            if len(parts) != 2:
                failures.append(f"{path}:{line_number}: malformed hash manifest line")
                continue
            expected, raw_file = parts
            expected = expected.lower()
            file_path = pathlib.Path(raw_file.strip()).expanduser()
            if not file_path.is_absolute():
                file_path = ROOT / file_path
            count += 1
            if not file_path.is_file():
                failures.append(f"{file_path}: missing artifact")
                continue
            actual = sha256(file_path)
            if actual != expected:
                failures.append(f"{file_path}: sha256 mismatch expected={expected} actual={actual}")
    return not failures, count, failures


def load_artifacts(path: pathlib.Path) -> tuple[dict[str, object], list[str]]:
    if not path.is_file():
        return {}, [f"{path}: missing artifacts JSON"]
    try:
        return json.loads(path.read_text(encoding="utf-8")), []
    except json.JSONDecodeError as exc:
        return {}, [f"{path}: invalid JSON: {exc}"]


def value(values: dict[str, str], key: str) -> str | None:
    return values.get(key)


def nested_str(data: dict[str, object], keys: tuple[str, ...]) -> str | None:
    current: object = data
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    if isinstance(current, str):
        return current
    return None


def check_current_artifact_binding(
    verdict: SummaryVerdict,
    artifacts: dict[str, object],
    current_artifacts_path: pathlib.Path | None = None,
) -> list[str]:
    if verdict.metadata.get("skipped") == "true":
        return []

    failures: list[str] = []
    expected_pairs = [
        (
            "optimized_artifact_sha256",
            nested_str(artifacts, ("optimized", "sha256")),
        ),
        (
            "optimized_runtime_run_sh_sha256",
            nested_str(artifacts, ("optimized_runtime", "run_sh", "sha256")),
        ),
        (
            "optimized_runtime_jar_sha256",
            nested_str(artifacts, ("optimized_runtime", "runtime_jar_sha256_file", "runtime_jar_sha256")),
        ),
        (
            "optimized_runtime_native_library_sha256",
            nested_str(artifacts, ("optimized_runtime", "native_library", "sha256")),
        ),
    ]
    chunk_encode_native_sha = nested_str(
        artifacts,
        ("optimized_runtime", "chunk_encode_native_library", "sha256"),
    )
    if chunk_encode_native_sha is not None:
        expected_pairs.append(
            (
                "optimized_runtime_chunk_encode_native_library_sha256",
                chunk_encode_native_sha,
            )
        )
    for key, expected in expected_pairs:
        observed = verdict.values.get(key)
        if expected is None:
            failures.append(f"{verdict.path}: current artifact metadata missing expected {key}")
        elif observed is None:
            failures.append(f"{verdict.path}: summary missing {key}; rerun load gate with current harness")
        elif observed != expected:
            failures.append(
                f"{verdict.path}: {key}={observed} does not match current artifact {expected}"
            )
    if current_artifacts_path is not None and verdict.path.is_file():
        try:
            summary_mtime = verdict.path.stat().st_mtime
            metadata_mtime = current_artifacts_path.stat().st_mtime
        except OSError as exc:
            failures.append(
                f"{verdict.path}: could not compare current artifact metadata timestamp: {exc}"
            )
        else:
            if summary_mtime < metadata_mtime:
                failures.append(
                    f"{verdict.path}: summary mtime {format_epoch(summary_mtime)} "
                    f"is older than current artifact metadata {current_artifacts_path} "
                    f"mtime {format_epoch(metadata_mtime)}; rerun load gate with current artifacts"
                )
    return failures


def is_skipped(verdict: SummaryVerdict) -> bool:
    return verdict.metadata.get("skipped") == "true"


def add_summary_lines(lines: list[str], prefix: str, verdict: SummaryVerdict) -> None:
    lines.append(f"{prefix}_summary_path={verdict.path}")
    lines.append(f"{prefix}_gate_profile={verdict.profile}")
    lines.append(f"{prefix}_skipped={bool_text(is_skipped(verdict))}")
    lines.append(f"{prefix}_gate_pass={bool_text(verdict.passed)}")
    lines.append(f"{prefix}_failure_count={len(verdict.failures)}")
    selected_keys = [
        "bots",
        "view_distance",
        "simulation_distance",
        "duration_seconds",
        "world_mode",
        "claim_surface",
        "world_warm_source_present",
        "load_test_scenario",
        "load_test_gamemode",
        "spark_background_profiler",
        "load_window_policy",
        "load_window_reached_full_online",
        "load_window_ended_by_online_drop",
        "load_window_metrics_samples",
        "load_window_online_max",
        "load_window_loaded_chunks_max",
        "load_window_tps1_avg",
        "load_window_tps1_min",
        "load_window_avg_tick_ms_avg",
        "load_window_avg_tick_ms_max",
        "online_max",
        "loaded_chunks_max",
        "tps1_avg",
        "tps1_min",
        "avg_tick_ms_avg",
        "avg_tick_ms_max",
        "resource_samples",
        "process_cpu_max",
        "process_rss_mib_max",
        "host_cpu_count",
        "host_system_load1_max",
        "host_system_load1_per_cpu_max",
        "host_mem_available_kb_min",
        "host_cpu_windows",
        "host_cpu_idle_percent_min",
        "host_cpu_iowait_percent_max",
        "host_cpu_iowait_percent_avg",
        "host_cpu_steal_percent_max",
        "host_cpu_steal_percent_avg",
        "bot_swarm_shards",
        "bot_created_max",
        "bot_connected_max",
        "bot_connected_source",
        "bot_ready_max",
        "bot_ready_source",
        "bot_login_packet_max",
        "bot_player_join_ready_max",
        "bot_active_max",
        "bot_kicked_max",
        "bot_errors_max",
        "bot_loadgen_telemetry_source",
        "bot_loadgen_telemetry_samples",
        "bot_loadgen_loop_delay_p95_ms_max",
        "bot_loadgen_loop_delay_p95_ms_avg",
        "bot_loadgen_loop_delay_max_ms_max",
        "bot_loadgen_loop_delay_max_ms_avg",
        "bot_loadgen_loop_delay_mean_ms_max",
        "bot_loadgen_loop_delay_mean_ms_avg",
        "bot_loadgen_timer_drift_max_ms_max",
        "bot_loadgen_timer_drift_max_ms_avg",
        "bot_loadgen_elu_pct_max",
        "bot_loadgen_elu_pct_avg",
        *SEND_PRESSURE_KEYS,
        "bot_block_armed_max",
        "bot_block_primed_max",
        "bot_block_place_packets_max",
        "bot_block_dig_packets_max",
        "bot_block_action_errors_max",
        "compat_probe_block_evidence_accepted",
        "compat_probe_block_metrics_loadbot_direct_evidence",
        "compat_probe_block_event_loadbot_places_max",
        "compat_probe_block_event_loadbot_breaks_max",
        "compat_probe_direct_block_loadbot_event_lines",
        "compat_probe_direct_block_loadbot_place_event_lines",
        "compat_probe_direct_block_loadbot_break_event_lines",
        "compat_probe_direct_block_loadbot_cancelled_true_lines",
        "compat_probe_direct_block_loadbot_cancelled_false_lines",
        "compat_probe_direct_block_loadbot_players",
        "watchdog_thread_dumps",
        "sync_load_stack_hits",
        "nearby_players_stack_hits",
        "stability_failures",
        "launcher_sha256",
        "optimized_artifact_sha256",
        "optimized_runtime_run_sh_sha256",
        "optimized_runtime_jar_sha256",
        "optimized_runtime_native_library_sha256",
        "optimized_runtime_chunk_encode_native_library_sha256",
    ]
    for key in selected_keys:
        current = value(verdict.values, key)
        if current is not None:
            lines.append(f"{prefix}_{key}={current}")
    for failure in verdict.failures:
        lines.append(f"{prefix}_failure={failure}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cold-summary",
        default="auto",
        help="production-500 cold/fresh summary path, or auto",
    )
    parser.add_argument(
        "--warm-summary",
        default="auto",
        help="production-500-warm saved-world summary path, or auto",
    )
    parser.add_argument(
        "--artifact-hashes",
        default="reports/artifact-hashes.txt",
        help="sha256sum-style artifact hash manifest",
    )
    parser.add_argument(
        "--artifacts-json",
        default="reports/artifacts.json",
        help="artifact metadata JSON generated by update_artifact_reports.py",
    )
    parser.add_argument(
        "--report",
        default="reports/production-500-release-gate.txt",
        help="output release gate report path",
    )
    parser.add_argument(
        "--require-current-artifacts",
        action="store_true",
        default=True,
        help="require load summaries to include hashes matching reports/artifacts.json (default)",
    )
    parser.add_argument(
        "--allow-stale-artifacts",
        action="store_true",
        help="legacy debug mode: do not require load summaries to match reports/artifacts.json (requires PRODUCTION_RELEASE_ALLOW_STALE_ARTIFACTS=true)",
    )
    args = parser.parse_args()

    if args.allow_stale_artifacts and os.environ.get("PRODUCTION_RELEASE_ALLOW_STALE_ARTIFACTS") != "true":
        print(
            "Refusing stale-artifacts mode without PRODUCTION_RELEASE_ALLOW_STALE_ARTIFACTS=true",
            file=sys.stderr,
        )
        return 78

    require_current_artifacts = not args.allow_stale_artifacts

    artifact_hashes = resolve_path(args.artifact_hashes)
    artifacts_json = resolve_path(args.artifacts_json)
    report_path = resolve_path(args.report)

    failures: list[str] = []

    if not artifact_hashes.is_file():
        hash_passed = False
        hash_count = 0
        hash_failures = [f"{artifact_hashes}: missing artifact hash manifest"]
    else:
        hash_passed, hash_count, hash_failures = verify_artifact_hashes(artifact_hashes)
    if not hash_passed:
        failures.append("artifact hash verification failed")
    failures.extend(hash_failures)

    artifacts, artifact_json_failures = load_artifacts(artifacts_json)
    if artifact_json_failures:
        failures.append("artifact metadata verification failed")
        failures.extend(artifact_json_failures)

    current_artifacts = artifacts if require_current_artifacts and artifacts else None
    cold = select_summary(
        args.cold_summary,
        "production-500",
        ["load-production-500-*-summary.txt"],
        current_artifacts,
        artifacts_json,
    )
    warm = select_summary(
        args.warm_summary,
        "production-500-warm",
        ["load-production-500-warm-*-summary.txt"],
        current_artifacts,
        artifacts_json,
    )

    if not cold.passed:
        if is_skipped(cold):
            failures.append("cold production-500 summary skipped")
        else:
            failures.append("cold production-500 gate failed")
    if not warm.passed:
        if is_skipped(warm):
            if cold.passed:
                failures.append("warm production-500-warm summary skipped")
        else:
            failures.append("warm production-500-warm gate failed")
    if require_current_artifacts:
        if artifacts:
            current_artifact_failures = []
            current_artifact_failures.extend(
                check_current_artifact_binding(cold, artifacts, artifacts_json)
            )
            current_artifact_failures.extend(
                check_current_artifact_binding(warm, artifacts, artifacts_json)
            )
        else:
            current_artifact_failures = ["current artifact binding requested but artifacts JSON is unavailable"]
        if current_artifact_failures:
            failures.append("load summaries are not bound to current artifacts")
            failures.extend(current_artifact_failures)

    optimized = artifacts.get("optimized", {}) if isinstance(artifacts, dict) else {}
    runtime = artifacts.get("optimized_runtime", {}) if isinstance(artifacts, dict) else {}
    ready = not failures

    lines = [
        "release_profile=production-500-release",
        f"generated_at_utc={dt.datetime.now(dt.timezone.utc).isoformat()}",
        "claim_text=500-bots-production-ready-for-measured-32-32-creative-block-profile",
        "claim_scope=cold-fresh-and-warm-source-500-bots-32-view-32-simulation-creative-block-workload",
        "claim_limits=not-full-paper-runtime-rust-rewrite;not-unbounded-plugin-compatibility;not-unmeasured-real-player-gameplay",
        f"production_ready_claim_eligible={bool_text(ready)}",
        f"release_gate_pass={bool_text(ready)}",
        f"failure_count={len(failures)}",
        f"artifact_hash_manifest={artifact_hashes}",
        f"artifact_hashes_pass={bool_text(hash_passed)}",
        f"artifact_hash_count={hash_count}",
        f"artifacts_json={artifacts_json}",
        f"requires_current_artifacts={bool_text(require_current_artifacts)}",
    ]
    if isinstance(optimized, dict):
        if "path" in optimized:
            lines.append(f"optimized_artifact_path={optimized['path']}")
        if "sha256" in optimized:
            lines.append(f"optimized_artifact_sha256={optimized['sha256']}")
    if isinstance(runtime, dict):
        run_sh = runtime.get("run_sh", {})
        app_cds = runtime.get("app_cds", {})
        native_library = runtime.get("native_library", {})
        chunk_encode_native_library = runtime.get("chunk_encode_native_library", {})
        if isinstance(run_sh, dict) and "path" in run_sh:
            lines.append(f"optimized_runtime_run_sh={run_sh['path']}")
        if isinstance(app_cds, dict) and "sha256" in app_cds:
            lines.append(f"optimized_runtime_app_cds_sha256={app_cds['sha256']}")
        if isinstance(native_library, dict):
            if "path" in native_library:
                lines.append(f"optimized_runtime_native_library={native_library['path']}")
            if "sha256" in native_library:
                lines.append(f"optimized_runtime_native_library_sha256={native_library['sha256']}")
        if isinstance(chunk_encode_native_library, dict):
            if "path" in chunk_encode_native_library:
                lines.append(f"optimized_runtime_chunk_encode_native_library={chunk_encode_native_library['path']}")
            if "sha256" in chunk_encode_native_library:
                lines.append(f"optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native_library['sha256']}")

    add_summary_lines(lines, "cold", cold)
    add_summary_lines(lines, "warm", warm)
    for failure in failures:
        lines.append(f"failure={failure}")

    report = "\n".join(lines) + "\n"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(report, encoding="utf-8")
    print(report, end="")
    return 0 if ready else 1


if __name__ == "__main__":
    raise SystemExit(main())
