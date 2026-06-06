#!/usr/bin/env python3
"""Validate a self-contained 500-bot production-readiness evidence bundle."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import pathlib
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = "production-500-readiness-bundle/v1"

REQUIRED_TRUE_CLAIM_KEYS = [
    "production_ready_500_claim",
    "readiness_gate_pass",
    "soak_gate_pass",
    "repeat_quorum_pass",
    "plugin_matrix_pass",
    "restart_recovery_pass",
    "forced_ticket_persistence_pass",
    "artifact_hashes_pass",
    "current_artifact_consistency_pass",
]

REQUIRED_EVIDENCE = [
    "production-500-readiness-gate.txt",
    "production-500-go-nogo-current.txt",
    "production-500-soak-gate.txt",
    "production-500-repeat-quorum.txt",
    "plugin-matrix-summary.txt",
    "restart-recovery-summary.txt",
    "forced-ticket-persistence-summary.txt",
    "artifact-hashes.txt",
]

REQUIRED_RAW_LOG_EVIDENCE = [
    "plugin-matrix.log",
    "restart-recovery.log",
    "forced-ticket-persistence-first.log",
    "forced-ticket-persistence-restart.log",
]

SUMMARY_LOG_REFERENCES = {
    "plugin-matrix-summary.txt": ["plugin_matrix_log"],
    "restart-recovery-summary.txt": ["restart_recovery_log"],
    "forced-ticket-persistence-summary.txt": ["first_log", "restart_log"],
}

NATIVE_PROOF_EVIDENCE = [
    "libpaper_native_jni.so.sha256",
    "paper-native-jni.sha256",
]

CHUNK_ENCODE_NATIVE_PROOF_EVIDENCE = [
    "libpaper_native_chunk_encode_jni.so.sha256",
    "paper-native-chunk-encode-jni.sha256",
]

READINESS_HASH_KEYS = {
    "go_nogo_report_sha256": "production-500-go-nogo-current.txt",
    "soak_report_sha256": "production-500-soak-gate.txt",
    "repeat_report_sha256": "production-500-repeat-quorum.txt",
    "plugin_matrix_summary_sha256": "plugin-matrix-summary.txt",
    "restart_recovery_summary_sha256": "restart-recovery-summary.txt",
    "forced_ticket_summary_sha256": "forced-ticket-persistence-summary.txt",
    "artifact_hash_manifest_sha256": "artifact-hashes.txt",
}

CURRENT_ARTIFACT_EVIDENCE_PATTERNS = {
    "cold": [
        "load-production-500-cold*current-artifact*-gate.txt",
        "load-production-500-cold*current-artifact*-summary.txt",
    ],
    "warm": [
        "load-production-500-warm*current-artifact*-gate.txt",
        "load-production-500-warm*current-artifact*-summary.txt",
    ],
}

REQUIRED_NON_CLAIMS = [
    "not a full Paper runtime rewrite to Rust",
    "not unlimited plugin compatibility",
    "not proof for unmeasured real-player gameplay",
    "not a multi-hour soak claim",
]

MEASURED_SURFACE_KEYS = {
    "cold": {
        "tps1_avg": ("cold_load_window_tps1_avg", float),
        "tps1_min": ("cold_load_window_tps1_min", float),
        "avg_tick_ms_max": ("cold_load_window_avg_tick_ms_max", float),
        "block_place_packets": ("cold_bot_block_place_packets_max", int),
        "block_dig_packets": ("cold_bot_block_dig_packets_max", int),
    },
    "warm": {
        "tps1_avg": ("warm_load_window_tps1_avg", float),
        "tps1_min": ("warm_load_window_tps1_min", float),
        "avg_tick_ms_max": ("warm_load_window_avg_tick_ms_max", float),
        "block_place_packets": ("warm_bot_block_place_packets_max", int),
        "block_dig_packets": ("warm_bot_block_dig_packets_max", int),
    },
}

CURRENT_EVIDENCE_GATE_CONTEXT_KEYS = [
    "gate_pass",
    "failure_count",
    "environment_invalid",
    "environment_invalid_kind",
    "environment_invalid_reason",
    "run_class",
    "observed_early_abort_reason",
    "observed_bots",
    "observed_view_distance",
    "observed_simulation_distance",
    "observed_load_test_scenario",
    "observed_load_test_gamemode",
    "observed_load_window_reached_full_online",
    "observed_load_window_metrics_samples",
    "observed_online_max",
    "observed_loaded_chunks_max",
    "observed_tps1_avg",
    "observed_tps1_min",
    "observed_avg_tick_ms_avg",
    "observed_avg_tick_ms_max",
    "observed_process_rss_mib_max",
    "observed_host_cpu_iowait_percent_max",
    "observed_host_cpu_steal_percent_max",
    "observed_watchdog_thread_dumps",
    "observed_sync_load_stack_hits",
]

CURRENT_EVIDENCE_SUMMARY_CONTEXT_KEYS = [
    "optimized_artifact_path",
    "optimized_artifact_sha256",
    "optimized_runtime_run_sh_path",
    "optimized_runtime_run_sh_sha256",
    "optimized_runtime_native_library_path",
    "optimized_runtime_native_library_sha256",
    "optimized_runtime_chunk_encode_native_library_path",
    "optimized_runtime_chunk_encode_native_library_sha256",
    "load_window_reached_full_online",
    "load_window_metrics_samples",
    "online_max",
    "loaded_chunks_max",
    "tps1_avg",
    "tps1_min",
    "avg_tick_ms_avg",
    "avg_tick_ms_max",
    "process_rss_mib_max",
    "host_system_load1_per_cpu_max",
    "host_cpu_iowait_percent_max",
    "host_cpu_steal_percent_max",
    "early_abort_reason",
]


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def reject_duplicate_json_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate json key {key!r}")
        result[key] = value
    return result


def parse_kv(
    path: pathlib.Path,
    failures: list[str] | None = None,
    label: str | None = None,
) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line or "=" not in line:
                continue
            key, value = line.split("=", 1)
            if failures is not None and key in values:
                failures.append(f"{label or path}:{line_no}: duplicate key {key}")
            values[key] = value
    return values


def is_sha256(raw: Any) -> bool:
    if not isinstance(raw, str) or len(raw) != 64:
        return False
    return all(char in "0123456789abcdef" for char in raw)


def resolve_path(raw: str | pathlib.Path) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def format_epoch(seconds: float) -> str:
    return dt.datetime.fromtimestamp(seconds, dt.timezone.utc).isoformat()


def format_age(seconds: float) -> str:
    sign = "-" if seconds < 0 else ""
    seconds = abs(int(seconds))
    days, remainder = divmod(seconds, 24 * 60 * 60)
    hours, remainder = divmod(remainder, 60 * 60)
    minutes, seconds = divmod(remainder, 60)
    if days:
        return f"{sign}{days}d{hours}h{minutes}m{seconds}s"
    if hours:
        return f"{sign}{hours}h{minutes}m{seconds}s"
    if minutes:
        return f"{sign}{minutes}m{seconds}s"
    return f"{sign}{seconds}s"


def path_diagnostics(raw_path: pathlib.Path, now_epoch: float) -> dict[str, Any]:
    alias_path = raw_path.expanduser()
    if not alias_path.is_absolute():
        alias_path = ROOT / alias_path

    diagnostics: dict[str, Any] = {
        "alias_path": str(alias_path),
        "alias_is_symlink": alias_path.is_symlink(),
    }
    if alias_path.is_symlink():
        try:
            diagnostics["symlink_target"] = str(alias_path.readlink())
        except OSError as exc:
            diagnostics["symlink_target_error"] = str(exc)

    try:
        real_path = alias_path.resolve()
    except (OSError, RuntimeError) as exc:
        diagnostics["real_path_error"] = str(exc)
        real_path = alias_path.absolute()
    diagnostics["real_path"] = str(real_path)

    try:
        stat_result = alias_path.stat()
    except OSError as exc:
        diagnostics["stat_error"] = str(exc)
        return diagnostics

    if alias_path.is_dir():
        path_type = "directory"
    elif alias_path.is_file():
        path_type = "file"
    else:
        path_type = "other"
    age_seconds = now_epoch - stat_result.st_mtime
    diagnostics.update(
        {
            "type": path_type,
            "mtime_utc": format_epoch(stat_result.st_mtime),
            "age_seconds": f"{age_seconds:.3f}",
            "age": format_age(age_seconds),
        }
    )

    if alias_path.is_symlink():
        try:
            link_stat = alias_path.lstat()
        except OSError as exc:
            diagnostics["alias_lstat_error"] = str(exc)
        else:
            link_age_seconds = now_epoch - link_stat.st_mtime
            diagnostics["alias_lstat_mtime_utc"] = format_epoch(link_stat.st_mtime)
            diagnostics["alias_lstat_age_seconds"] = f"{link_age_seconds:.3f}"
            diagnostics["alias_lstat_age"] = format_age(link_age_seconds)
    return diagnostics


def parse_generated_at_utc(data: dict[str, Any], failures: list[str]) -> tuple[str | None, float | None]:
    raw = data.get("generated_at_utc")
    if not isinstance(raw, str) or not raw:
        failures.append("current_artifact_freshness: bundle_index.generated_at_utc is missing")
        return None, None
    try:
        parsed = dt.datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError as exc:
        failures.append(f"current_artifact_freshness: invalid bundle_index.generated_at_utc={raw!r}: {exc}")
        return raw, None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=dt.timezone.utc)
    else:
        parsed = parsed.astimezone(dt.timezone.utc)
    return parsed.isoformat(), parsed.timestamp()


def nested_str(data: dict[str, Any], keys: tuple[str, ...]) -> str | None:
    current: Any = data
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current if isinstance(current, str) else None


def iter_path_sha_pairs(data: Any, prefix: str = "artifacts_json") -> list[tuple[str, str, str]]:
    pairs: list[tuple[str, str, str]] = []
    if isinstance(data, dict):
        raw_path = data.get("path")
        raw_sha = data.get("sha256")
        if isinstance(raw_path, str) and isinstance(raw_sha, str):
            pairs.append((prefix, raw_path, raw_sha))
        for key, value in data.items():
            child_prefix = f"{prefix}.{key}"
            pairs.extend(iter_path_sha_pairs(value, child_prefix))
    elif isinstance(data, list):
        for index, value in enumerate(data):
            pairs.extend(iter_path_sha_pairs(value, f"{prefix}[{index}]"))
    return pairs


def validate_live_path_sha_pairs(
    data: dict[str, Any],
    failures: list[str],
    *,
    label: str,
) -> None:
    for key_path, raw_path, expected_sha in iter_path_sha_pairs(data, label):
        if not is_sha256(expected_sha):
            failures.append(f"{key_path}.sha256 is not a lowercase sha256")
            continue
        path = resolve_path(raw_path)
        if not path.is_file():
            failures.append(f"{key_path}.path={path} is missing")
            continue
        observed_sha = sha256(path)
        if observed_sha != expected_sha:
            failures.append(
                f"{key_path}.path live sha256={observed_sha} expected={expected_sha}"
            )


def parse_manifest(path: pathlib.Path, failures: list[str]) -> dict[str, dict[str, str]]:
    records: dict[str, dict[str, str]] = {}
    if not path.is_file():
        failures.append(f"manifest: {path} is missing")
        return records
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.rstrip("\n")
            if not line:
                continue
            parts = line.split("\t")
            logical_name = parts[0]
            fields: dict[str, str] = {}
            for part in parts[1:]:
                if "=" not in part:
                    failures.append(f"manifest:{line_no}: malformed field {part!r}")
                    continue
                key, value = part.split("=", 1)
                if key in fields:
                    failures.append(f"manifest:{line_no}: duplicate field {key} for {logical_name}")
                fields[key] = value
            if logical_name in records:
                failures.append(f"manifest:{line_no}: duplicate {logical_name}")
            records[logical_name] = fields
    return records


def load_current_artifacts(path: pathlib.Path, failures: list[str]) -> dict[str, Any]:
    if not path.is_file():
        failures.append(f"current_artifact_freshness: {path} is missing")
        return {}
    try:
        data = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_json_pairs,
        )
    except (json.JSONDecodeError, ValueError) as exc:
        failures.append(f"current_artifact_freshness: invalid artifacts json {path}: {exc}")
        return {}
    if not isinstance(data, dict):
        failures.append(f"current_artifact_freshness: {path} top-level json is not an object")
        return {}
    return data


def read_json_object(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, Any]:
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return {}
    try:
        data = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_json_pairs,
        )
    except (json.JSONDecodeError, ValueError) as exc:
        failures.append(f"{label}: invalid json: {exc}")
        return {}
    if not isinstance(data, dict):
        failures.append(f"{label}: top-level json is not an object")
        return {}
    return data


def read_optional_json_object(path: pathlib.Path) -> dict[str, Any]:
    if not path.is_file():
        return {}
    try:
        data = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_json_pairs,
        )
    except (json.JSONDecodeError, ValueError, OSError):
        return {}
    return data if isinstance(data, dict) else {}


def parse_sha256_rows(path: pathlib.Path, label: str, failures: list[str]) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return rows
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line:
                continue
            parts = line.split(None, 1)
            if len(parts) != 2:
                failures.append(f"{label}:{line_no}: expected sha256 and path")
                continue
            digest, artifact_path = parts
            if not is_sha256(digest):
                failures.append(f"{label}:{line_no}: invalid sha256 {digest!r}")
            rows.append((digest, artifact_path))
    if not rows:
        failures.append(f"{label}: no hash rows found")
    return rows


def same_artifact_path(left: str, right: str) -> bool:
    left_path = pathlib.Path(left)
    right_path = pathlib.Path(right)
    try:
        return left_path.resolve() == right_path.resolve()
    except OSError:
        return left_path == right_path


def has_sha_path_row(rows: list[tuple[str, str]], digest: str, artifact_path: str) -> bool:
    return any(row_digest == digest and same_artifact_path(row_path, artifact_path) for row_digest, row_path in rows)


def bundled_artifacts_json(
    evidence: dict[str, dict[str, Any]],
    failures: list[str],
) -> dict[str, Any]:
    record = evidence.get("artifacts.json")
    path = record.get("path") if record else None
    if not isinstance(path, pathlib.Path):
        return {}
    return read_json_object(path, "artifacts_json", failures)


def latest_current_artifact_evidence(
    reports_dir: pathlib.Path,
    failures: list[str],
) -> tuple[pathlib.Path | None, float | None, dict[str, tuple[pathlib.Path, float]]]:
    if not reports_dir.is_dir():
        failures.append(f"current_artifact_freshness: reports dir {reports_dir} is missing")
        return None, None, {}

    candidates: dict[pathlib.Path, float] = {}
    by_surface: dict[str, tuple[pathlib.Path, float]] = {}
    for surface, patterns in CURRENT_ARTIFACT_EVIDENCE_PATTERNS.items():
        surface_candidates: dict[pathlib.Path, float] = {}
        for pattern in patterns:
            for path in reports_dir.glob(pattern):
                if not path.is_file():
                    continue
                try:
                    surface_candidates[path] = path.stat().st_mtime
                except OSError as exc:
                    failures.append(
                        f"current_artifact_freshness: could not stat current-artifact evidence {path}: {exc}"
                    )
        if not surface_candidates:
            failures.append(
                f"current_artifact_freshness: no {surface} current-artifact 500 gate/summary evidence found under {reports_dir}"
            )
            continue
        latest_surface = max(surface_candidates.items(), key=lambda item: (item[1], str(item[0])))
        by_surface[surface] = latest_surface
        candidates.update(surface_candidates)

    if not candidates:
        failures.append(
            f"current_artifact_freshness: no current-artifact 500 gate/summary evidence found under {reports_dir}"
        )
        return None, None, by_surface

    latest = max(candidates.items(), key=lambda item: (item[1], str(item[0])))
    return latest[0], latest[1], by_surface


def freshness_subjects(bundle_dir: pathlib.Path) -> dict[str, pathlib.Path]:
    return {
        "bundle_index": bundle_dir / "bundle.json",
        "bundle_claim_markdown": bundle_dir / "CLAIM.md",
        "bundle_readiness_report": bundle_dir / "evidence" / "production-500-readiness-gate.txt",
    }


def resolve_report_reference(raw: str, reports_dir: pathlib.Path) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if path.is_absolute():
        return path
    reports_relative = reports_dir / path
    if reports_relative.exists():
        return reports_relative
    return resolve_path(path)


def summary_sibling(path: pathlib.Path) -> pathlib.Path | None:
    if path.name.endswith("-gate.txt"):
        return path.with_name(f"{path.name[:-len('-gate.txt')]}-summary.txt")
    return None


def gate_sibling(path: pathlib.Path) -> pathlib.Path | None:
    if path.name.endswith("-summary.txt"):
        return path.with_name(f"{path.name[:-len('-summary.txt')]}-gate.txt")
    return None


def safe_parse_kv(path: pathlib.Path) -> tuple[dict[str, str], str | None]:
    try:
        return parse_kv(path), None
    except OSError as exc:
        return {}, str(exc)


def add_file_time_context(
    context: dict[str, str],
    prefix: str,
    path: pathlib.Path,
    now_epoch: float,
) -> None:
    context[f"{prefix}_path"] = str(path)
    try:
        mtime = path.stat().st_mtime
    except OSError as exc:
        context[f"{prefix}_stat_error"] = str(exc)
        return
    age_seconds = now_epoch - mtime
    context[f"{prefix}_mtime_utc"] = format_epoch(mtime)
    context[f"{prefix}_age_seconds"] = f"{age_seconds:.3f}"
    context[f"{prefix}_age"] = format_age(age_seconds)


def latest_current_evidence_context(
    evidence_path: pathlib.Path | None,
    reports_dir: pathlib.Path,
    now_epoch: float,
) -> dict[str, str]:
    if evidence_path is None:
        return {}

    context: dict[str, str] = {}
    add_file_time_context(context, "evidence", evidence_path, now_epoch)
    evidence_values, evidence_error = safe_parse_kv(evidence_path)
    if evidence_error:
        context["evidence_read_error"] = evidence_error
        return context

    gate_path: pathlib.Path | None = None
    summary_path: pathlib.Path | None = None
    if evidence_path.name.endswith("-gate.txt"):
        gate_path = evidence_path
        raw_summary_path = evidence_values.get("summary_path")
        summary_path = (
            resolve_report_reference(raw_summary_path, reports_dir)
            if raw_summary_path
            else summary_sibling(evidence_path)
        )
    elif evidence_path.name.endswith("-summary.txt"):
        summary_path = evidence_path
        gate_path = gate_sibling(evidence_path)

    gate_values: dict[str, str] = {}
    if gate_path is not None and gate_path.is_file():
        if gate_path != evidence_path:
            add_file_time_context(context, "gate", gate_path, now_epoch)
        gate_values, gate_error = safe_parse_kv(gate_path)
        if gate_error:
            context["gate_read_error"] = gate_error
    elif gate_path is not None:
        context["gate_path"] = str(gate_path)
        context["gate_missing"] = "true"

    summary_values: dict[str, str] = {}
    if summary_path is not None and summary_path.is_file():
        if summary_path != evidence_path:
            add_file_time_context(context, "summary", summary_path, now_epoch)
        summary_values, summary_error = safe_parse_kv(summary_path)
        if summary_error:
            context["summary_read_error"] = summary_error
    elif summary_path is not None:
        context["summary_path"] = str(summary_path)
        context["summary_missing"] = "true"

    for key in CURRENT_EVIDENCE_GATE_CONTEXT_KEYS:
        value = gate_values.get(key)
        if value is not None:
            context[f"gate_{key}"] = value
    for key in CURRENT_EVIDENCE_SUMMARY_CONTEXT_KEYS:
        value = summary_values.get(key)
        if value is not None:
            context[f"summary_{key}"] = value
    return context


def validate_current_artifact_freshness(
    bundle_dir: pathlib.Path,
    data: dict[str, Any],
    claim: dict[str, Any],
    reports_dir: pathlib.Path,
    failures: list[str],
    *,
    bundled_artifacts: dict[str, Any] | None = None,
) -> dict[str, Any]:
    reports_dir = resolve_path(reports_dir)
    artifacts_json = reports_dir / "artifacts.json"
    local_failures: list[str] = []
    now_epoch = dt.datetime.now(dt.timezone.utc).timestamp()
    bundle_generated_at_utc, bundle_generated_epoch = parse_generated_at_utc(data, local_failures)
    bundle_artifacts = bundled_artifacts or read_optional_json_object(
        bundle_dir / "evidence" / "artifacts.json"
    )

    artifacts = load_current_artifacts(artifacts_json, local_failures)
    if artifacts:
        validate_live_path_sha_pairs(
            artifacts,
            local_failures,
            label="current_artifact_freshness.artifacts_json",
        )
    current_path = nested_str(artifacts, ("optimized", "path")) if artifacts else None
    current_sha = nested_str(artifacts, ("optimized", "sha256")) if artifacts else None
    current_run_sh_path = nested_str(artifacts, ("optimized_runtime", "run_sh", "path")) if artifacts else None
    current_run_sh_sha = nested_str(artifacts, ("optimized_runtime", "run_sh", "sha256")) if artifacts else None
    current_native_path = nested_str(artifacts, ("optimized_runtime", "native_library", "path")) if artifacts else None
    current_native_sha = nested_str(artifacts, ("optimized_runtime", "native_library", "sha256")) if artifacts else None
    current_native_proof_sha = nested_str(
        artifacts,
        ("optimized_runtime", "native_library_sha256_file", "native_library_sha256"),
    ) if artifacts else None
    current_chunk_encode_native_path = nested_str(
        artifacts,
        ("optimized_runtime", "chunk_encode_native_library", "path"),
    ) if artifacts else None
    current_chunk_encode_native_sha = nested_str(
        artifacts,
        ("optimized_runtime", "chunk_encode_native_library", "sha256"),
    ) if artifacts else None
    current_chunk_encode_native_proof_sha = nested_str(
        artifacts,
        (
            "optimized_runtime",
            "chunk_encode_native_library_sha256_file",
            "chunk_encode_native_library_sha256",
        ),
    ) if artifacts else None
    bundle_artifact_path = nested_str(bundle_artifacts, ("optimized", "path")) if bundle_artifacts else None
    bundle_artifact_sha = nested_str(bundle_artifacts, ("optimized", "sha256")) if bundle_artifacts else None
    bundle_run_sh_path = nested_str(bundle_artifacts, ("optimized_runtime", "run_sh", "path")) if bundle_artifacts else None
    bundle_run_sh_sha = nested_str(bundle_artifacts, ("optimized_runtime", "run_sh", "sha256")) if bundle_artifacts else None
    bundle_native_path = nested_str(bundle_artifacts, ("optimized_runtime", "native_library", "path")) if bundle_artifacts else None
    bundle_native_sha = nested_str(bundle_artifacts, ("optimized_runtime", "native_library", "sha256")) if bundle_artifacts else None
    bundle_chunk_encode_native_path = nested_str(
        bundle_artifacts,
        ("optimized_runtime", "chunk_encode_native_library", "path"),
    ) if bundle_artifacts else None
    bundle_chunk_encode_native_sha = nested_str(
        bundle_artifacts,
        ("optimized_runtime", "chunk_encode_native_library", "sha256"),
    ) if bundle_artifacts else None
    claim_sha = claim.get("optimized_artifact_sha256")
    claim_run_sh_sha = claim.get("optimized_runtime_run_sh_sha256")
    claim_native_sha = claim.get("optimized_runtime_native_library_sha256")
    claim_chunk_encode_native_sha = claim.get("optimized_runtime_chunk_encode_native_library_sha256")
    if current_sha is None:
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} missing optimized.sha256"
        )
    elif not is_sha256(current_sha):
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} optimized.sha256 is not a lowercase sha256"
        )
    elif claim_sha != current_sha:
        local_failures.append(
            "current_artifact_freshness: "
            f"bundle optimized_artifact_sha256={claim_sha} does not match "
            f"current artifacts.json optimized.sha256={current_sha}"
        )
    if current_run_sh_sha is None:
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} missing optimized_runtime.run_sh.sha256"
        )
    elif not is_sha256(current_run_sh_sha):
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} optimized_runtime.run_sh.sha256 is not a lowercase sha256"
        )
    elif claim_run_sh_sha != current_run_sh_sha:
        local_failures.append(
            "current_artifact_freshness: "
            f"bundle optimized_runtime_run_sh_sha256={claim_run_sh_sha} does not match "
            f"current artifacts.json optimized_runtime.run_sh.sha256={current_run_sh_sha}"
        )
    if current_native_sha is None:
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} missing optimized_runtime.native_library.sha256"
        )
    elif not is_sha256(current_native_sha):
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} optimized_runtime.native_library.sha256 is not a lowercase sha256"
        )
    elif claim_native_sha != current_native_sha:
        local_failures.append(
            "current_artifact_freshness: "
            f"bundle optimized_runtime_native_library_sha256={claim_native_sha} does not match "
            f"current artifacts.json optimized_runtime.native_library.sha256={current_native_sha}"
        )
    if current_native_proof_sha is None:
        local_failures.append(
            f"current_artifact_freshness: {artifacts_json} missing optimized_runtime.native_library_sha256_file.native_library_sha256"
        )
    elif current_native_proof_sha != current_native_sha:
        local_failures.append(
            "current_artifact_freshness: "
            "artifacts.json optimized_runtime.native_library_sha256_file.native_library_sha256 "
            f"{current_native_proof_sha} does not match native_library.sha256={current_native_sha}"
        )
    chunk_encode_present = (
        current_chunk_encode_native_path is not None
        or current_chunk_encode_native_sha is not None
        or claim_chunk_encode_native_sha is not None
    )
    if chunk_encode_present:
        if current_chunk_encode_native_sha is None:
            local_failures.append(
                f"current_artifact_freshness: {artifacts_json} missing optimized_runtime.chunk_encode_native_library.sha256"
            )
        elif not is_sha256(current_chunk_encode_native_sha):
            local_failures.append(
                f"current_artifact_freshness: {artifacts_json} optimized_runtime.chunk_encode_native_library.sha256 is not a lowercase sha256"
            )
        elif claim_chunk_encode_native_sha != current_chunk_encode_native_sha:
            local_failures.append(
                "current_artifact_freshness: "
                "bundle optimized_runtime_chunk_encode_native_library_sha256="
                f"{claim_chunk_encode_native_sha} does not match current artifacts.json "
                f"optimized_runtime.chunk_encode_native_library.sha256={current_chunk_encode_native_sha}"
            )
        if current_chunk_encode_native_proof_sha is None:
            local_failures.append(
                f"current_artifact_freshness: {artifacts_json} missing optimized_runtime.chunk_encode_native_library_sha256_file.chunk_encode_native_library_sha256"
            )
        elif current_chunk_encode_native_proof_sha != current_chunk_encode_native_sha:
            local_failures.append(
                "current_artifact_freshness: "
                "artifacts.json optimized_runtime.chunk_encode_native_library_sha256_file.chunk_encode_native_library_sha256 "
                f"{current_chunk_encode_native_proof_sha} does not match "
                f"chunk_encode_native_library.sha256={current_chunk_encode_native_sha}"
            )

    latest_path, latest_mtime, latest_by_surface = latest_current_artifact_evidence(
        reports_dir,
        local_failures,
    )
    latest_mtime_utc = format_epoch(latest_mtime) if latest_mtime is not None else None
    if bundle_generated_epoch is not None and latest_mtime is not None:
        if bundle_generated_epoch < latest_mtime:
            local_failures.append(
                "current_artifact_freshness: "
                f"bundle generated_at_utc={bundle_generated_at_utc} is older than "
                f"latest current-artifact 500 gate/summary evidence {latest_path} "
                f"mtime={latest_mtime_utc}; rerun the production readiness bundle"
            )
    if latest_mtime is not None:
        for label, path in freshness_subjects(bundle_dir).items():
            if not path.is_file():
                local_failures.append(f"current_artifact_freshness: {label} {path} is missing")
                continue
            try:
                subject_mtime = path.stat().st_mtime
            except OSError as exc:
                local_failures.append(
                    f"current_artifact_freshness: could not stat {label} {path}: {exc}"
                )
                continue
            if subject_mtime < latest_mtime:
                local_failures.append(
                    "current_artifact_freshness: "
                    f"{label} mtime={format_epoch(subject_mtime)} is older than "
                    f"latest current-artifact 500 gate/summary evidence {latest_path} "
                    f"mtime={latest_mtime_utc}; rerun the production readiness bundle"
                )

    failures.extend(local_failures)
    result = {
        "passed": not local_failures,
        "reports_dir": str(reports_dir),
        "artifacts_json": str(artifacts_json),
        "bundle_generated_at_utc": bundle_generated_at_utc,
        "bundle_artifacts_json_optimized_artifact_path": bundle_artifact_path,
        "bundle_artifacts_json_optimized_artifact_sha256": bundle_artifact_sha,
        "bundle_artifacts_json_optimized_runtime_run_sh_path": bundle_run_sh_path,
        "bundle_artifacts_json_optimized_runtime_run_sh_sha256": bundle_run_sh_sha,
        "bundle_artifacts_json_optimized_runtime_native_library_path": bundle_native_path,
        "bundle_artifacts_json_optimized_runtime_native_library_sha256": bundle_native_sha,
        "bundle_artifacts_json_optimized_runtime_chunk_encode_native_library_path": bundle_chunk_encode_native_path,
        "bundle_artifacts_json_optimized_runtime_chunk_encode_native_library_sha256": bundle_chunk_encode_native_sha,
        "current_optimized_artifact_path": current_path,
        "current_optimized_artifact_sha256": current_sha,
        "current_optimized_runtime_run_sh_path": current_run_sh_path,
        "current_optimized_runtime_run_sh_sha256": current_run_sh_sha,
        "current_optimized_runtime_native_library_path": current_native_path,
        "current_optimized_runtime_native_library_sha256": current_native_sha,
        "current_optimized_runtime_chunk_encode_native_library_path": current_chunk_encode_native_path,
        "current_optimized_runtime_chunk_encode_native_library_sha256": current_chunk_encode_native_sha,
        "latest_current_artifact_evidence": str(latest_path) if latest_path else None,
        "latest_current_artifact_evidence_mtime_utc": latest_mtime_utc,
        "latest_current_artifact_evidence_context": latest_current_evidence_context(
            latest_path,
            reports_dir,
            now_epoch,
        ),
        "failure_count": len(local_failures),
    }
    for surface, (path, mtime) in latest_by_surface.items():
        result[f"latest_{surface}_current_artifact_evidence"] = str(path)
        result[f"latest_{surface}_current_artifact_evidence_mtime_utc"] = format_epoch(mtime)
    for label, path in freshness_subjects(bundle_dir).items():
        if path.is_file():
            try:
                result[f"{label}_mtime_utc"] = format_epoch(path.stat().st_mtime)
            except OSError:
                pass
    return result


def load_bundle_index(path: pathlib.Path, failures: list[str]) -> dict[str, Any]:
    if not path.is_file():
        failures.append(f"bundle_index: {path} is missing")
        return {}
    try:
        data = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_json_pairs,
        )
    except (json.JSONDecodeError, ValueError) as exc:
        failures.append(f"bundle_index: invalid json: {exc}")
        return {}
    if not isinstance(data, dict):
        failures.append("bundle_index: top-level json is not an object")
        return {}
    return data


def bundle_relative_path(bundle_dir: pathlib.Path, raw: Any, failures: list[str]) -> pathlib.Path | None:
    if not isinstance(raw, str) or not raw:
        failures.append(f"evidence: invalid relative_path={raw!r}")
        return None
    relative = pathlib.PurePosixPath(raw)
    if relative.is_absolute() or ".." in relative.parts:
        failures.append(f"evidence: unsafe relative_path={raw!r}")
        return None
    return bundle_dir / pathlib.Path(*relative.parts)


def validate_evidence_records(
    bundle_dir: pathlib.Path,
    data: dict[str, Any],
    manifest_records: dict[str, dict[str, str]],
    failures: list[str],
) -> dict[str, dict[str, Any]]:
    raw_records = data.get("evidence_files")
    if not isinstance(raw_records, list) or not raw_records:
        failures.append("bundle_index: evidence_files must be a non-empty list")
        return {}

    by_logical: dict[str, dict[str, Any]] = {}
    for index, raw_record in enumerate(raw_records):
        if not isinstance(raw_record, dict):
            failures.append(f"evidence_files[{index}]: record is not an object")
            continue
        logical = raw_record.get("logical_name")
        if not isinstance(logical, str) or not logical:
            failures.append(f"evidence_files[{index}]: missing logical_name")
            continue
        if logical in by_logical:
            failures.append(f"evidence_files[{index}]: duplicate {logical}")
            continue

        path = bundle_relative_path(bundle_dir, raw_record.get("relative_path"), failures)
        if path is None:
            continue
        if not path.is_file():
            failures.append(f"{logical}: {path} is missing")
            continue

        expected_sha = raw_record.get("sha256")
        expected_bytes = raw_record.get("bytes")
        observed_sha = sha256(path)
        observed_bytes = path.stat().st_size
        if expected_sha != observed_sha:
            failures.append(f"{logical}: sha256={observed_sha} expected={expected_sha}")
        if expected_bytes != observed_bytes:
            failures.append(f"{logical}: bytes={observed_bytes} expected={expected_bytes}")

        manifest = manifest_records.get(logical)
        if manifest is None:
            failures.append(f"{logical}: missing from MANIFEST.txt")
        else:
            if manifest.get("sha256") != expected_sha:
                failures.append(
                    f"{logical}: manifest sha256={manifest.get('sha256')} expected={expected_sha}"
                )
            if manifest.get("bytes") != str(expected_bytes):
                failures.append(
                    f"{logical}: manifest bytes={manifest.get('bytes')} expected={expected_bytes}"
                )

        by_logical[logical] = {
            **raw_record,
            "path": path,
            "observed_sha256": observed_sha,
            "observed_bytes": observed_bytes,
        }

    for logical in [*REQUIRED_EVIDENCE, *REQUIRED_RAW_LOG_EVIDENCE]:
        if logical not in by_logical:
            failures.append(f"required_evidence: missing {logical}")
    return by_logical


def validate_summary_log_references(
    evidence: dict[str, dict[str, Any]],
    failures: list[str],
) -> None:
    for summary_logical, required_keys in SUMMARY_LOG_REFERENCES.items():
        record = evidence.get(summary_logical)
        path = record.get("path") if record else None
        if not isinstance(path, pathlib.Path):
            failures.append(f"{summary_logical}: evidence missing")
            continue
        values = parse_kv(path, failures, summary_logical)
        for key in required_keys:
            raw = values.get(key)
            if not raw:
                failures.append(f"{summary_logical}.{key} is missing")
                continue
            log_logical = pathlib.PurePath(raw).name
            if log_logical not in evidence:
                failures.append(
                    f"{summary_logical}: referenced log {log_logical} is missing from bundle evidence"
                )


def validate_claim(data: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    if data.get("schema") != SCHEMA:
        failures.append(f"bundle_index: schema={data.get('schema')} expected={SCHEMA}")

    claim = data.get("claim")
    if not isinstance(claim, dict):
        failures.append("bundle_index: claim is missing or not an object")
        return {}

    for key in REQUIRED_TRUE_CLAIM_KEYS:
        if claim.get(key) is not True:
            failures.append(f"claim.{key}={claim.get(key)!r} expected=True")
    if claim.get("failure_count") != 0:
        failures.append(f"claim.failure_count={claim.get('failure_count')!r} expected=0")
    if not isinstance(claim.get("artifact_hash_count"), int) or claim["artifact_hash_count"] < 1:
        failures.append("claim.artifact_hash_count must be a positive integer")
    if not isinstance(claim.get("repeat_passes"), int) or claim["repeat_passes"] < 3:
        failures.append("claim.repeat_passes must be >= 3")
    if not isinstance(claim.get("optimized_artifact_sha256"), str):
        failures.append("claim.optimized_artifact_sha256 is missing")
    if not is_sha256(claim.get("optimized_runtime_run_sh_sha256")):
        failures.append("claim.optimized_runtime_run_sh_sha256 is missing")
    native_sha = claim.get("optimized_runtime_native_library_sha256")
    if native_sha is not None and not is_sha256(native_sha):
        failures.append("claim.optimized_runtime_native_library_sha256 must be a 64-char sha256 when present")
    chunk_encode_native_sha = claim.get("optimized_runtime_chunk_encode_native_library_sha256")
    if chunk_encode_native_sha is not None and not is_sha256(chunk_encode_native_sha):
        failures.append(
            "claim.optimized_runtime_chunk_encode_native_library_sha256 must be a 64-char sha256 when present"
        )

    non_claims = data.get("explicit_non_claims")
    if not isinstance(non_claims, list):
        failures.append("bundle_index: explicit_non_claims must be a list")
    else:
        for required in REQUIRED_NON_CLAIMS:
            if required not in non_claims:
                failures.append(f"explicit_non_claims: missing {required}")
    return claim


def validate_artifacts_json_against_claim(
    artifacts: dict[str, Any],
    claim: dict[str, Any],
    failures: list[str],
    *,
    require_native: bool,
) -> None:
    if not artifacts:
        if require_native:
            failures.append("artifacts_json: artifacts.json evidence is required for current native proof")
        return
    validate_live_path_sha_pairs(artifacts, failures, label="artifacts_json")
    checks = [
        (
            "optimized.sha256",
            nested_str(artifacts, ("optimized", "sha256")),
            claim.get("optimized_artifact_sha256"),
            True,
        ),
        (
            "optimized_runtime.run_sh.sha256",
            nested_str(artifacts, ("optimized_runtime", "run_sh", "sha256")),
            claim.get("optimized_runtime_run_sh_sha256"),
            True,
        ),
        (
            "optimized_runtime.native_library.sha256",
            nested_str(artifacts, ("optimized_runtime", "native_library", "sha256")),
            claim.get("optimized_runtime_native_library_sha256"),
            require_native,
        ),
    ]
    chunk_encode_sha = nested_str(artifacts, ("optimized_runtime", "chunk_encode_native_library", "sha256"))
    claim_chunk_encode_sha = claim.get("optimized_runtime_chunk_encode_native_library_sha256")
    chunk_encode_required = chunk_encode_sha is not None or claim_chunk_encode_sha is not None
    checks.append(
        (
            "optimized_runtime.chunk_encode_native_library.sha256",
            chunk_encode_sha,
            claim_chunk_encode_sha,
            chunk_encode_required,
        )
    )
    for key, observed, expected, required in checks:
        if observed is None:
            if required:
                failures.append(f"artifacts_json.{key} is missing")
            continue
        if not is_sha256(observed):
            failures.append(f"artifacts_json.{key} is not a lowercase sha256")
        elif expected != observed:
            failures.append(f"artifacts_json.{key} does not match bundle claim")

    native_sha = nested_str(artifacts, ("optimized_runtime", "native_library", "sha256"))
    native_proof_sha = nested_str(
        artifacts,
        ("optimized_runtime", "native_library_sha256_file", "native_library_sha256"),
    )
    if require_native:
        if native_proof_sha is None:
            failures.append("artifacts_json.optimized_runtime.native_library_sha256_file.native_library_sha256 is missing")
        elif native_proof_sha != native_sha:
            failures.append(
                "artifacts_json.optimized_runtime.native_library_sha256_file.native_library_sha256 "
                "does not match optimized_runtime.native_library.sha256"
            )
    if chunk_encode_required:
        chunk_encode_proof_sha = nested_str(
            artifacts,
            (
                "optimized_runtime",
                "chunk_encode_native_library_sha256_file",
                "chunk_encode_native_library_sha256",
            ),
        )
        if chunk_encode_proof_sha is None:
            failures.append(
                "artifacts_json.optimized_runtime.chunk_encode_native_library_sha256_file.chunk_encode_native_library_sha256 is missing"
            )
        elif chunk_encode_proof_sha != chunk_encode_sha:
            failures.append(
                "artifacts_json.optimized_runtime.chunk_encode_native_library_sha256_file.chunk_encode_native_library_sha256 "
                "does not match optimized_runtime.chunk_encode_native_library.sha256"
            )


def validate_artifact_hash_manifest(
    evidence: dict[str, dict[str, Any]],
    artifacts: dict[str, Any],
    claim: dict[str, Any],
    failures: list[str],
    *,
    require_live_hashes: bool,
) -> list[tuple[str, str]]:
    artifact_hash_record = evidence.get("artifact-hashes.txt")
    artifact_hash_path = artifact_hash_record.get("path") if artifact_hash_record else None
    rows: list[tuple[str, str]] = []
    if isinstance(artifact_hash_path, pathlib.Path):
        rows = parse_sha256_rows(artifact_hash_path, "artifact_hashes", failures)
    else:
        failures.append("artifact_hashes: artifact-hashes.txt evidence missing")
        return rows

    claim_count = claim.get("artifact_hash_count")
    if isinstance(claim_count, int) and rows and claim_count != len(rows):
        failures.append(
            f"artifact_hashes: claim.artifact_hash_count={claim_count} actual_rows={len(rows)}"
        )

    seen_rows: set[tuple[str, str]] = set()
    for index, row in enumerate(rows, start=1):
        if row in seen_rows:
            failures.append(f"artifact_hashes:{index}: duplicate hash/path row")
        seen_rows.add(row)
        if require_live_hashes:
            digest, raw_path = row
            path = resolve_path(raw_path)
            if not path.is_file():
                failures.append(f"artifact_hashes:{index}: path={path} is missing")
            elif is_sha256(digest):
                observed_sha = sha256(path)
                if observed_sha != digest:
                    failures.append(
                        f"artifact_hashes:{index}: path={path} live sha256={observed_sha} expected={digest}"
                    )

    for key_path, raw_path, expected_sha in iter_path_sha_pairs(artifacts, "artifacts_json"):
        if not is_sha256(expected_sha):
            continue
        if not has_sha_path_row(rows, expected_sha, raw_path):
            failures.append(
                f"artifact_hashes: missing {key_path} row sha256={expected_sha} path={raw_path}"
            )

    return rows


def validate_native_library_bundle_proof(
    evidence: dict[str, dict[str, Any]],
    claim: dict[str, Any],
    failures: list[str],
    *,
    require_native: bool,
) -> None:
    native_sha = claim.get("optimized_runtime_native_library_sha256")
    if not is_sha256(native_sha):
        if require_native:
            failures.append("claim.optimized_runtime_native_library_sha256 is required for current bundle native proof")
        return

    artifact_hash_record = evidence.get("artifact-hashes.txt")
    artifact_hash_path = artifact_hash_record.get("path") if artifact_hash_record else None
    hash_rows: list[tuple[str, str]] = []
    if isinstance(artifact_hash_path, pathlib.Path):
        hash_rows = parse_sha256_rows(artifact_hash_path, "artifact_hashes", failures)
    native_rows = [
        row for row in hash_rows
        if row[0] == native_sha and row[1].endswith("libpaper_native_jni.so")
    ]
    if require_native and not native_rows:
        failures.append("artifact_hashes: optimized_runtime_native_library_sha256 is not present for libpaper_native_jni.so")

    proof_found = False
    for logical in NATIVE_PROOF_EVIDENCE:
        record = evidence.get(logical)
        path = record.get("path") if record else None
        if not isinstance(path, pathlib.Path):
            continue
        proof_found = True
        rows = parse_sha256_rows(path, logical, failures)
        if not any(row[0] == native_sha for row in rows):
            failures.append(f"{logical}: does not contain optimized_runtime_native_library_sha256")
    if require_native and not proof_found:
        failures.append(
            "native_library_sha256_proof: missing libpaper_native_jni.so.sha256 or paper-native-jni.sha256 evidence"
        )


def validate_chunk_encode_native_library_bundle_proof(
    evidence: dict[str, dict[str, Any]],
    claim: dict[str, Any],
    failures: list[str],
) -> None:
    chunk_encode_native_sha = claim.get("optimized_runtime_chunk_encode_native_library_sha256")
    if chunk_encode_native_sha is None:
        return
    if not is_sha256(chunk_encode_native_sha):
        return

    artifact_hash_record = evidence.get("artifact-hashes.txt")
    artifact_hash_path = artifact_hash_record.get("path") if artifact_hash_record else None
    hash_rows: list[tuple[str, str]] = []
    if isinstance(artifact_hash_path, pathlib.Path):
        hash_rows = parse_sha256_rows(artifact_hash_path, "artifact_hashes", failures)
    native_rows = [
        row for row in hash_rows
        if row[0] == chunk_encode_native_sha
        and row[1].endswith("libpaper_native_chunk_encode_jni.so")
    ]
    if not native_rows:
        failures.append(
            "artifact_hashes: optimized_runtime_chunk_encode_native_library_sha256 is not present for libpaper_native_chunk_encode_jni.so"
        )

    proof_found = False
    for logical in CHUNK_ENCODE_NATIVE_PROOF_EVIDENCE:
        record = evidence.get(logical)
        path = record.get("path") if record else None
        if not isinstance(path, pathlib.Path):
            continue
        proof_found = True
        rows = parse_sha256_rows(path, logical, failures)
        if not any(row[0] == chunk_encode_native_sha for row in rows):
            failures.append(
                f"{logical}: does not contain optimized_runtime_chunk_encode_native_library_sha256"
            )
    if not proof_found:
        failures.append(
            "chunk_encode_native_library_sha256_proof: missing libpaper_native_chunk_encode_jni.so.sha256 or paper-native-chunk-encode-jni.sha256 evidence"
        )


def validate_readiness_report(
    readiness_path: pathlib.Path,
    data: dict[str, Any],
    claim: dict[str, Any],
    evidence: dict[str, dict[str, Any]],
    failures: list[str],
) -> None:
    if not readiness_path.is_file():
        failures.append(f"readiness: {readiness_path} is missing")
        return

    values = parse_kv(readiness_path, failures, "readiness")
    for key in REQUIRED_TRUE_CLAIM_KEYS:
        if values.get(key) != "true":
            failures.append(f"readiness.{key}={values.get(key)} expected=true")
    if values.get("failure_count") != "0":
        failures.append(f"readiness.failure_count={values.get('failure_count')} expected=0")
    if values.get("optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append(
            "readiness.optimized_artifact_sha256 does not match bundle claim"
        )
    if values.get("current_optimized_runtime_run_sh_sha256") != claim.get("optimized_runtime_run_sh_sha256"):
        failures.append(
            "readiness.current_optimized_runtime_run_sh_sha256 does not match bundle claim"
        )
    readiness_native = (
        values.get("current_optimized_runtime_native_library_sha256")
        or values.get("optimized_runtime_native_library_sha256")
    )
    if readiness_native is not None and readiness_native != claim.get("optimized_runtime_native_library_sha256"):
        failures.append(
            "readiness.current_optimized_runtime_native_library_sha256 does not match bundle claim"
        )
    readiness_chunk_encode_native = (
        values.get("current_optimized_runtime_chunk_encode_native_library_sha256")
        or values.get("optimized_runtime_chunk_encode_native_library_sha256")
    )
    claim_chunk_encode_native = claim.get("optimized_runtime_chunk_encode_native_library_sha256")
    if readiness_chunk_encode_native is not None and readiness_chunk_encode_native != claim_chunk_encode_native:
        failures.append(
            "readiness.current_optimized_runtime_chunk_encode_native_library_sha256 does not match bundle claim"
        )
    if claim_chunk_encode_native is not None and readiness_chunk_encode_native is None:
        failures.append(
            "readiness.current_optimized_runtime_chunk_encode_native_library_sha256 is missing"
        )
    if values.get("go_nogo_present") != "true":
        failures.append("readiness.go_nogo_present=true expected")
    if values.get("production_500_go_nogo_pass") != "true":
        failures.append(
            f"readiness.production_500_go_nogo_pass={values.get('production_500_go_nogo_pass')} expected=true"
        )
    if values.get("production_500_go_nogo_reason") != "none":
        failures.append(
            f"readiness.production_500_go_nogo_reason={values.get('production_500_go_nogo_reason')} expected=none"
        )
    go_nogo_record = evidence.get("production-500-go-nogo-current.txt")
    go_nogo_path = go_nogo_record.get("path") if go_nogo_record else None
    if isinstance(go_nogo_path, pathlib.Path):
        go_nogo_values = parse_kv(go_nogo_path, failures, "go_nogo")
        if go_nogo_values.get("production_500_go_nogo_pass") != "true":
            failures.append(
                f"go_nogo.production_500_go_nogo_pass={go_nogo_values.get('production_500_go_nogo_pass')} expected=true"
            )
        if go_nogo_values.get("production_500_go_nogo_reason") != "none":
            failures.append(
                f"go_nogo.production_500_go_nogo_reason={go_nogo_values.get('production_500_go_nogo_reason')} expected=none"
            )
    else:
        failures.append("go_nogo: production-500-go-nogo-current.txt evidence missing")
    if values.get("repeat_passes") != str(claim.get("repeat_passes")):
        failures.append("readiness.repeat_passes does not match bundle claim")
    if values.get("artifact_hash_count") != str(claim.get("artifact_hash_count")):
        failures.append("readiness.artifact_hash_count does not match bundle claim")

    surface = data.get("measured_load_surface")
    if not isinstance(surface, dict):
        failures.append("readiness.measured_load_surface comparison missing bundle surface")
    else:
        for side, key_map in MEASURED_SURFACE_KEYS.items():
            bundle_side = surface.get(side)
            if not isinstance(bundle_side, dict):
                failures.append(f"readiness.measured_load_surface.{side} missing from bundle")
                continue
            for bundle_key, (readiness_key, parser) in key_map.items():
                raw = values.get(readiness_key)
                if raw is None:
                    failures.append(f"readiness.{readiness_key} is missing")
                    continue
                try:
                    expected = parser(raw)
                except ValueError:
                    failures.append(f"readiness.{readiness_key}={raw!r} is not numeric")
                    continue
                observed = bundle_side.get(bundle_key)
                if not isinstance(observed, (int, float)):
                    failures.append(f"bundle.measured_load_surface.{side}.{bundle_key}={observed!r} is not numeric")
                    continue
                if parser is int:
                    if not isinstance(observed, int) or observed != expected:
                        failures.append(
                            f"bundle.measured_load_surface.{side}.{bundle_key}={observed} "
                            f"readiness.{readiness_key}={expected}"
                        )
                elif abs(float(observed) - float(expected)) > 1e-9:
                    failures.append(
                        f"bundle.measured_load_surface.{side}.{bundle_key}={observed} "
                        f"readiness.{readiness_key}={expected}"
                    )

    for hash_key, logical in READINESS_HASH_KEYS.items():
        expected = values.get(hash_key)
        observed = evidence.get(logical, {}).get("observed_sha256")
        if expected != observed:
            failures.append(f"readiness.{hash_key}={expected} observed_bundle_sha256={observed}")


def validate_claim_markdown(path: pathlib.Path, failures: list[str]) -> None:
    if not path.is_file():
        failures.append(f"claim_markdown: {path} is missing")
        return
    text = path.read_text(encoding="utf-8", errors="replace")
    required_snippets = [
        "Production-ready for the measured 500-bot",
        "Not unlimited plugin compatibility",
        "MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh",
    ]
    for snippet in required_snippets:
        if snippet not in text:
            failures.append(f"claim_markdown: missing {snippet!r}")


def validate_bundle(
    bundle_dir: pathlib.Path,
    *,
    require_current_freshness: bool = True,
    allow_stale_freshness: bool = False,
    reports_dir: pathlib.Path | None = None,
) -> dict[str, Any]:
    bundle_alias_path = bundle_dir.expanduser()
    if not bundle_alias_path.is_absolute():
        bundle_alias_path = ROOT / bundle_alias_path
    now_epoch = dt.datetime.now(dt.timezone.utc).timestamp()
    bundle_path_info = path_diagnostics(bundle_alias_path, now_epoch)
    try:
        bundle_dir = bundle_alias_path.resolve()
    except (OSError, RuntimeError):
        bundle_dir = bundle_alias_path.absolute()
    failures: list[str] = []
    if not bundle_dir.is_dir():
        failures.append(f"bundle_dir: {bundle_dir} is missing")

    data = load_bundle_index(bundle_dir / "bundle.json", failures)
    claim = validate_claim(data, failures) if data else {}
    manifest_records = parse_manifest(bundle_dir / "MANIFEST.txt", failures)
    evidence = validate_evidence_records(bundle_dir, data, manifest_records, failures) if data else {}
    if evidence:
        validate_summary_log_references(evidence, failures)
    is_current_bundle = bundle_alias_path.name.endswith("-current") or bundle_dir.name.endswith("-current")
    # Current bundles are publication targets, so freshness is mandatory even
    # if a caller asks for stale-freshness debug mode.
    should_require_current_freshness = is_current_bundle or (
        require_current_freshness and not allow_stale_freshness
    )
    artifacts = bundled_artifacts_json(evidence, failures) if evidence else {}
    validate_artifacts_json_against_claim(
        artifacts,
        claim,
        failures,
        require_native=should_require_current_freshness,
    )
    validate_artifact_hash_manifest(
        evidence,
        artifacts,
        claim,
        failures,
        require_live_hashes=should_require_current_freshness,
    )
    validate_native_library_bundle_proof(
        evidence,
        claim,
        failures,
        require_native=should_require_current_freshness,
    )
    validate_chunk_encode_native_library_bundle_proof(evidence, claim, failures)

    readiness_record = evidence.get("production-500-readiness-gate.txt")
    readiness_path = readiness_record.get("path") if readiness_record else None
    if isinstance(readiness_path, pathlib.Path):
        validate_readiness_report(readiness_path, data, claim, evidence, failures)
    else:
        failures.append("readiness: production-500-readiness-gate.txt evidence missing")
    validate_claim_markdown(bundle_dir / "CLAIM.md", failures)
    freshness: dict[str, Any] = {}
    if should_require_current_freshness:
        freshness = validate_current_artifact_freshness(
            bundle_dir,
            data,
            claim,
            reports_dir if reports_dir is not None else ROOT / "reports",
            failures,
            bundled_artifacts=artifacts,
        )

    return {
        "passed": not failures,
        "failures": failures,
        "bundle_dir": bundle_dir,
        "bundle_path_diagnostics": bundle_path_info,
        "data": data,
        "claim": claim,
        "evidence": evidence,
        "freshness": freshness,
        "current_freshness_required": should_require_current_freshness,
    }


def print_validation_result(result: dict[str, Any]) -> None:
    data = result["data"]
    claim = result["claim"]
    evidence = result["evidence"]
    freshness = result.get("freshness") or {}
    bundle_path_info = result.get("bundle_path_diagnostics") or {}
    print(f"bundle_validation_pass={str(result['passed']).lower()}")
    print(f"failure_count={len(result['failures'])}")
    print(f"bundle_dir={result['bundle_dir']}")
    print(f"bundle_dir_alias_path={bundle_path_info.get('alias_path')}")
    print(f"bundle_dir_alias_is_symlink={str(bundle_path_info.get('alias_is_symlink')).lower()}")
    print(f"bundle_dir_symlink_target={bundle_path_info.get('symlink_target')}")
    print(f"bundle_dir_real_path={bundle_path_info.get('real_path')}")
    print(f"bundle_dir_type={bundle_path_info.get('type')}")
    print(f"bundle_dir_mtime_utc={bundle_path_info.get('mtime_utc')}")
    print(f"bundle_dir_age_seconds={bundle_path_info.get('age_seconds')}")
    print(f"bundle_dir_age={bundle_path_info.get('age')}")
    if bundle_path_info.get("alias_lstat_mtime_utc") is not None:
        print(f"bundle_dir_alias_lstat_mtime_utc={bundle_path_info.get('alias_lstat_mtime_utc')}")
        print(f"bundle_dir_alias_lstat_age_seconds={bundle_path_info.get('alias_lstat_age_seconds')}")
        print(f"bundle_dir_alias_lstat_age={bundle_path_info.get('alias_lstat_age')}")
    if bundle_path_info.get("stat_error") is not None:
        print(f"bundle_dir_stat_error={bundle_path_info.get('stat_error')}")
    if bundle_path_info.get("symlink_target_error") is not None:
        print(f"bundle_dir_symlink_target_error={bundle_path_info.get('symlink_target_error')}")
    print(f"schema={data.get('schema') if data else None}")
    print(f"evidence_file_count={len(evidence)}")
    print(f"optimized_artifact_sha256={claim.get('optimized_artifact_sha256')}")
    print(
        "optimized_runtime_run_sh_sha256="
        f"{claim.get('optimized_runtime_run_sh_sha256')}"
    )
    print(
        "optimized_runtime_native_library_sha256="
        f"{claim.get('optimized_runtime_native_library_sha256')}"
    )
    print(
        "optimized_runtime_chunk_encode_native_library_sha256="
        f"{claim.get('optimized_runtime_chunk_encode_native_library_sha256')}"
    )
    if freshness:
        print(f"current_freshness_required={str(result.get('current_freshness_required')).lower()}")
        print(f"current_artifact_freshness_pass={str(freshness.get('passed')).lower()}")
        print(f"current_artifacts_json={freshness.get('artifacts_json')}")
        print(
            "bundle_artifacts_json_optimized_artifact_path="
            f"{freshness.get('bundle_artifacts_json_optimized_artifact_path')}"
        )
        print(
            "bundle_artifacts_json_optimized_artifact_sha256="
            f"{freshness.get('bundle_artifacts_json_optimized_artifact_sha256')}"
        )
        print(
            "bundle_artifacts_json_optimized_runtime_run_sh_path="
            f"{freshness.get('bundle_artifacts_json_optimized_runtime_run_sh_path')}"
        )
        print(
            "bundle_artifacts_json_optimized_runtime_run_sh_sha256="
            f"{freshness.get('bundle_artifacts_json_optimized_runtime_run_sh_sha256')}"
        )
        print(
            "bundle_artifacts_json_optimized_runtime_native_library_path="
            f"{freshness.get('bundle_artifacts_json_optimized_runtime_native_library_path')}"
        )
        print(
            "bundle_artifacts_json_optimized_runtime_native_library_sha256="
            f"{freshness.get('bundle_artifacts_json_optimized_runtime_native_library_sha256')}"
        )
        print(
            "bundle_artifacts_json_optimized_runtime_chunk_encode_native_library_path="
            f"{freshness.get('bundle_artifacts_json_optimized_runtime_chunk_encode_native_library_path')}"
        )
        print(
            "bundle_artifacts_json_optimized_runtime_chunk_encode_native_library_sha256="
            f"{freshness.get('bundle_artifacts_json_optimized_runtime_chunk_encode_native_library_sha256')}"
        )
        print(f"current_optimized_artifact_path={freshness.get('current_optimized_artifact_path')}")
        print(f"current_optimized_artifact_sha256={freshness.get('current_optimized_artifact_sha256')}")
        print(
            "current_optimized_runtime_run_sh_path="
            f"{freshness.get('current_optimized_runtime_run_sh_path')}"
        )
        print(
            "current_optimized_runtime_run_sh_sha256="
            f"{freshness.get('current_optimized_runtime_run_sh_sha256')}"
        )
        print(
            "current_optimized_runtime_native_library_path="
            f"{freshness.get('current_optimized_runtime_native_library_path')}"
        )
        print(
            "current_optimized_runtime_native_library_sha256="
            f"{freshness.get('current_optimized_runtime_native_library_sha256')}"
        )
        print(
            "current_optimized_runtime_chunk_encode_native_library_path="
            f"{freshness.get('current_optimized_runtime_chunk_encode_native_library_path')}"
        )
        print(
            "current_optimized_runtime_chunk_encode_native_library_sha256="
            f"{freshness.get('current_optimized_runtime_chunk_encode_native_library_sha256')}"
        )
        print(f"bundle_generated_at_utc={freshness.get('bundle_generated_at_utc')}")
        print(f"latest_current_artifact_evidence={freshness.get('latest_current_artifact_evidence')}")
        print(
            "latest_current_artifact_evidence_mtime_utc="
            f"{freshness.get('latest_current_artifact_evidence_mtime_utc')}"
        )
        print(f"latest_cold_current_artifact_evidence={freshness.get('latest_cold_current_artifact_evidence')}")
        print(
            "latest_cold_current_artifact_evidence_mtime_utc="
            f"{freshness.get('latest_cold_current_artifact_evidence_mtime_utc')}"
        )
        print(f"latest_warm_current_artifact_evidence={freshness.get('latest_warm_current_artifact_evidence')}")
        print(
            "latest_warm_current_artifact_evidence_mtime_utc="
            f"{freshness.get('latest_warm_current_artifact_evidence_mtime_utc')}"
        )
        for key, value in (freshness.get("latest_current_artifact_evidence_context") or {}).items():
            print(f"latest_current_artifact_evidence_context_{key}={value}")
    for failure in result["failures"]:
        print(f"bundle_validation_failure={failure}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle_dir", help="Path to production-500 readiness bundle.")
    parser.add_argument(
        "--require-current-freshness",
        action="store_true",
        help="Require the bundle claim to match live reports/artifacts.json and postdate latest current-artifact 500 evidence. This is the default.",
    )
    parser.add_argument(
        "--allow-stale-freshness",
        action="store_true",
        help="Legacy/debug escape hatch: validate only bundle internals without comparing to current artifacts.",
    )
    parser.add_argument(
        "--reports-dir",
        default="reports",
        help="Reports directory containing artifacts.json and current-artifact 500 evidence.",
    )
    args = parser.parse_args()

    result = validate_bundle(
        pathlib.Path(args.bundle_dir),
        require_current_freshness=True,
        allow_stale_freshness=args.allow_stale_freshness,
        reports_dir=resolve_path(args.reports_dir),
    )
    print_validation_result(result)
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
