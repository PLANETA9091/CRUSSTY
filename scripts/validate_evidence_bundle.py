#!/usr/bin/env python3
"""Validate a measured 500-bot production evidence bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import sys
from typing import Any

import validate_production_readiness_bundle as production_readiness_bundle


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = "production-500-readiness-bundle/v1"
CLAIM_TEXT = "500-bots-production-ready-for-measured-32-32-creative-block-profile"
CLAIM_SCOPE = (
    "500-bots-32-view-32-simulation-creative-block-cold-warm-soak-repeat-"
    "plugin-restart-forced-ticket"
)
ALLOWED_CLAIM = (
    "Production-ready for the measured 500-bot, 32 view-distance, "
    "32 simulation-distance, creative block workload profile on the "
    "verified current optimized artifact."
)

REQUIRED_EVIDENCE = [
    "production-500-readiness-gate.txt",
    "production-500-soak-gate.txt",
    "production-500-repeat-quorum.txt",
    "plugin-matrix-summary.txt",
    "restart-recovery-summary.txt",
    "forced-ticket-persistence-summary.txt",
    "artifact-hashes.txt",
    "artifacts.json",
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

REQUIRED_INDEX_EVIDENCE = [
    "production-500-readiness-gate.txt",
    "production-500-soak-gate.txt",
    "production-500-repeat-quorum.txt",
    "plugin-matrix-summary.txt",
    "restart-recovery-summary.txt",
    "forced-ticket-persistence-summary.txt",
    "artifact-hashes.txt",
]

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

READINESS_TRUE_KEYS = REQUIRED_TRUE_CLAIM_KEYS

SEND_PRESSURE_INT_KEYS = [
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
    "compat_probe_chunk_send_max_unacknowledged_batches_max",
    "compat_probe_chunk_send_channel_not_writable_skips_max",
]

SEND_PRESSURE_FLOAT_KEYS = [
    "compat_probe_chunk_send_batch_quota_max",
    "compat_probe_chunk_send_desired_chunks_per_tick_max",
]

SEND_PRESSURE_KEYS = SEND_PRESSURE_INT_KEYS + SEND_PRESSURE_FLOAT_KEYS

READINESS_REQUIRED_KEYS = [
    "claim_text",
    "claim_scope",
    "claim_limits",
    "failure_count",
    "artifact_hash_count",
    "soak_report",
    "repeat_report",
    "plugin_matrix_summary",
    "restart_recovery_summary",
    "forced_ticket_summary",
    "artifact_hash_manifest",
    "soak_report_sha256",
    "repeat_report_sha256",
    "plugin_matrix_summary_sha256",
    "restart_recovery_summary_sha256",
    "forced_ticket_summary_sha256",
    "artifact_hash_manifest_sha256",
    "optimized_artifact_sha256",
    "current_optimized_artifact_sha256",
    "current_optimized_runtime_run_sh_sha256",
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
    "repeat_passes",
]

SOAK_TRUE_KEYS = [
    "production_ready_soak_claim_eligible",
    "soak_gate_pass",
    "base_cold_gate_pass",
    "base_warm_gate_pass",
    "artifact_hashes_pass",
    "cold_gate_pass",
    "warm_gate_pass",
    "cold_load_window_reached_full_online",
    "warm_load_window_reached_full_online",
]

SOAK_REQUIRED_KEYS = [
    "claim_text",
    "claim_scope",
    "failure_count",
    "artifact_hash_count",
    "artifact_hash_manifest",
    "artifacts_json",
    "optimized_artifact_path",
    "optimized_artifact_sha256",
    "optimized_runtime_run_sh",
    "cold_summary_path",
    "warm_summary_path",
    "cold_failure_count",
    "warm_failure_count",
    "cold_bots",
    "warm_bots",
    "cold_view_distance",
    "warm_view_distance",
    "cold_simulation_distance",
    "warm_simulation_distance",
    "cold_duration_seconds",
    "warm_duration_seconds",
    "cold_world_mode",
    "warm_world_mode",
    "cold_claim_surface",
    "warm_claim_surface",
    "cold_load_test_scenario",
    "warm_load_test_scenario",
    "cold_load_test_gamemode",
    "warm_load_test_gamemode",
    "cold_load_window_metrics_samples",
    "warm_load_window_metrics_samples",
    "cold_load_window_online_max",
    "warm_load_window_online_max",
    "cold_load_window_tps1_avg",
    "warm_load_window_tps1_avg",
    "cold_load_window_tps1_min",
    "warm_load_window_tps1_min",
    "cold_load_window_avg_tick_ms_max",
    "warm_load_window_avg_tick_ms_max",
    "cold_bot_block_place_packets_max",
    "warm_bot_block_place_packets_max",
    "cold_bot_block_dig_packets_max",
    "warm_bot_block_dig_packets_max",
    "cold_bot_block_action_errors_max",
    "warm_bot_block_action_errors_max",
    "cold_stability_failures",
    "warm_stability_failures",
    "cold_watchdog_thread_dumps",
    "warm_watchdog_thread_dumps",
    "cold_sync_load_stack_hits",
    "warm_sync_load_stack_hits",
    "cold_optimized_artifact_sha256",
    "warm_optimized_artifact_sha256",
    "cold_optimized_runtime_run_sh_sha256",
    "warm_optimized_runtime_run_sh_sha256",
    "cold_optimized_runtime_jar_sha256",
    "warm_optimized_runtime_jar_sha256",
] + [f"{side}_{key}" for side in ("cold", "warm") for key in SEND_PRESSURE_KEYS]

REPEAT_REQUIRED_KEYS = [
    "required_min_passes",
    "repeat_run_count",
    "repeat_passes",
    "repeat_failures",
    "repeat_quorum_pass",
]

SUMMARY_KEYS = [
    "bots",
    "view_distance",
    "simulation_distance",
    "duration_seconds",
    "optimized_artifact_sha256",
    "optimized_runtime_run_sh_sha256",
    "optimized_runtime_jar_sha256",
    "world_mode",
    "claim_surface",
    "load_test_scenario",
    "load_test_gamemode",
    "load_window_reached_full_online",
    "load_window_metrics_samples",
    "load_window_online_max",
    "load_window_tps1_avg",
    "load_window_tps1_min",
    "load_window_avg_tick_ms_max",
    "bot_block_place_packets_max",
    "bot_block_dig_packets_max",
    "bot_block_action_errors_max",
    *SEND_PRESSURE_KEYS,
]

SUMMARY_HOST_KEYS = [
    "resource_samples",
    "process_cpu_max",
    "process_rss_mib_max",
    "host_cpu_count",
    "host_system_load1_max",
    "host_system_load1_per_cpu_max",
    "host_mem_available_kb_min",
]

PREFLIGHT_TRUE_KEYS = ["host_preflight_ok"]
PREFLIGHT_REQUIRED_KEYS = [
    "cpu_count",
    "load1",
    "load5",
    "load15",
    "load_per_cpu",
    "idle_percent_1s",
    "min_idle_percent",
    "max_load_per_cpu",
]

RESOURCES_REQUIRED_COLUMNS = {
    "ts_ms",
    "pid_cpu",
    "pid_rss_kb",
    "system_load1",
    "system_mem_available_kb",
}

READINESS_HASH_KEYS = {
    "soak_report_sha256": "production-500-soak-gate.txt",
    "repeat_report_sha256": "production-500-repeat-quorum.txt",
    "plugin_matrix_summary_sha256": "plugin-matrix-summary.txt",
    "restart_recovery_summary_sha256": "restart-recovery-summary.txt",
    "forced_ticket_summary_sha256": "forced-ticket-persistence-summary.txt",
    "artifact_hash_manifest_sha256": "artifact-hashes.txt",
}

MIN_TPS_AVG = 19.50
MIN_TPS_MIN = 18.00
MAX_MSPT_MAX = 100.00
MIN_BLOCK_PACKETS = 120_000
MIN_LOAD_WINDOW_SAMPLES = 300
EXPECTED_BOTS = 500
EXPECTED_DISTANCE = 32
EXPECTED_DURATION_SECONDS = 2400

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
KV_TOKEN_RE = re.compile(r"(?<!\S)([A-Za-z0-9_]+)=([^ \t\r\n]+)")


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def is_sha256(value: Any) -> bool:
    return isinstance(value, str) and SHA256_RE.fullmatch(value) is not None


def read_json_object(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, Any]:
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        failures.append(f"{label}: invalid json: {exc}")
        return {}
    if not isinstance(data, dict):
        failures.append(f"{label}: top-level json is not an object")
        return {}
    return data


def parse_kv_lines(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return values
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line or "=" not in line:
                continue
            key, value = line.split("=", 1)
            if not key:
                failures.append(f"{label}:{line_no}: empty key")
                continue
            if key in values and values[key] != value:
                failures.append(
                    f"{label}:{line_no}: duplicate key {key} has conflicting values "
                    f"{values[key]!r} vs {value!r}"
                )
            values[key] = value
    return values


def parse_kv_tokens(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return values
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            for match in KV_TOKEN_RE.finditer(raw):
                values[match.group(1)] = match.group(2)
    return values


def parse_summary_kv_lines(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return values
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if line == "bot_log_tail:":
                break
            if not line or "=" not in line:
                continue
            key, value = line.split("=", 1)
            if not key:
                failures.append(f"{label}:{line_no}: empty key")
                continue
            values[key] = value
    return values


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
            logical = parts[0]
            if not logical:
                failures.append(f"manifest:{line_no}: empty logical name")
                continue
            fields: dict[str, str] = {}
            for part in parts[1:]:
                if "=" not in part:
                    failures.append(f"manifest:{line_no}: malformed field {part!r}")
                    continue
                key, value = part.split("=", 1)
                fields[key] = value
            if logical in records:
                failures.append(f"manifest:{line_no}: duplicate {logical}")
            records[logical] = fields
    return records


def bundle_relative_path(
    bundle_dir: pathlib.Path, raw: Any, label: str, failures: list[str]
) -> pathlib.Path | None:
    if not isinstance(raw, str) or not raw:
        failures.append(f"{label}: invalid relative_path={raw!r}")
        return None
    relative = pathlib.PurePosixPath(raw)
    if relative.is_absolute() or ".." in relative.parts:
        failures.append(f"{label}: unsafe relative_path={raw!r}")
        return None
    candidate = (bundle_dir / pathlib.Path(*relative.parts)).resolve()
    try:
        candidate.relative_to(bundle_dir)
    except ValueError:
        failures.append(f"{label}: relative_path escapes bundle: {raw!r}")
        return None
    return candidate


def replace_summary_suffix(path: pathlib.PurePath, suffix: str) -> str | None:
    name = path.name
    if not name.endswith("-summary.txt"):
        return None
    return f"{name[:-len('-summary.txt')]}{suffix}"


def evidence_path_by_logical(
    evidence: dict[str, dict[str, Any]],
    logical: str,
) -> pathlib.Path | None:
    record = evidence.get(logical)
    path = record.get("path") if record else None
    return path if isinstance(path, pathlib.Path) else None


def require_bundled_log_reference(
    evidence: dict[str, dict[str, Any]],
    values: dict[str, str],
    label: str,
    key: str,
    failures: list[str],
) -> None:
    raw = values.get(key)
    if not raw:
        failures.append(f"{label}.{key} is missing")
        return
    logical = pathlib.PurePath(raw).name
    if evidence_path_by_logical(evidence, logical) is None:
        failures.append(
            f"{label}: referenced log {logical} is missing from bundle evidence"
        )


def resolve_repo_path(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def parse_int(value: Any) -> int | None:
    try:
        return int(str(value))
    except (TypeError, ValueError):
        return None


def parse_float(value: Any) -> float | None:
    try:
        return float(str(value))
    except (TypeError, ValueError):
        return None


def require_json_true(data: dict[str, Any], key: str, label: str, failures: list[str]) -> None:
    if key not in data:
        failures.append(f"{label}.{key} is missing")
    elif data.get(key) is not True:
        failures.append(f"{label}.{key}={data.get(key)!r} expected=True")


def require_kv_present(values: dict[str, str], keys: list[str], label: str, failures: list[str]) -> None:
    for key in keys:
        if key not in values:
            failures.append(f"{label}.{key} is missing")


def require_kv_true(values: dict[str, str], keys: list[str], label: str, failures: list[str]) -> None:
    for key in keys:
        if key not in values:
            failures.append(f"{label}.{key} is missing")
        elif values[key] != "true":
            failures.append(f"{label}.{key}={values[key]!r} expected=true")


def require_kv_zero(values: dict[str, str], keys: list[str], label: str, failures: list[str]) -> None:
    for key in keys:
        if key not in values:
            failures.append(f"{label}.{key} is missing")
        elif values[key] != "0":
            failures.append(f"{label}.{key}={values[key]!r} expected=0")


def require_kv_int(
    values: dict[str, str],
    key: str,
    label: str,
    failures: list[str],
    *,
    expected: int | None = None,
    minimum: int | None = None,
) -> int | None:
    if key not in values:
        failures.append(f"{label}.{key} is missing")
        return None
    parsed = parse_int(values[key])
    if parsed is None:
        failures.append(f"{label}.{key}={values[key]!r} is not an int")
        return None
    if expected is not None and parsed != expected:
        failures.append(f"{label}.{key}={parsed} expected={expected}")
    if minimum is not None and parsed < minimum:
        failures.append(f"{label}.{key}={parsed} expected>={minimum}")
    return parsed


def require_kv_float(
    values: dict[str, str],
    key: str,
    label: str,
    failures: list[str],
    *,
    minimum: float | None = None,
    maximum: float | None = None,
) -> float | None:
    if key not in values:
        failures.append(f"{label}.{key} is missing")
        return None
    parsed = parse_float(values[key])
    if parsed is None:
        failures.append(f"{label}.{key}={values[key]!r} is not numeric")
        return None
    if minimum is not None and parsed < minimum:
        failures.append(f"{label}.{key}={parsed} expected>={minimum}")
    if maximum is not None and parsed > maximum:
        failures.append(f"{label}.{key}={parsed} expected<={maximum}")
    return parsed


def read_csv_header(path: pathlib.Path, label: str, failures: list[str]) -> set[str]:
    if not path.is_file():
        failures.append(f"{label}: {path} is missing")
        return set()
    try:
        first = path.read_text(encoding="utf-8", errors="replace").splitlines()[0]
    except IndexError:
        failures.append(f"{label}: file is empty")
        return set()
    return {item.strip() for item in first.split(",") if item.strip()}


def advertised_native_modules(native_runtime_line: str) -> list[str]:
    modules: list[str] = []
    for match in KV_TOKEN_RE.finditer(native_runtime_line):
        key = match.group(1)
        value = match.group(2)
        if (
            key.startswith("native_")
            and key not in {"native_lib_available", "native_library_available", "native_dir"}
            and value == "true"
        ):
            modules.append(key.removeprefix("native_"))
    return modules


def validate_native_summary_fields(
    summary: dict[str, str],
    label: str,
    failures: list[str],
) -> None:
    native_runtime_line = summary.get("native_runtime_line")
    if native_runtime_line is None:
        failures.append(f"{label}.native_runtime_line is missing")
        return
    has_available_marker = (
        "native_lib_available=true" in native_runtime_line
        or "native_library_available=true" in native_runtime_line
        or summary.get("native_lib_available") == "true"
        or summary.get("native_library_available") == "true"
    )
    if not has_available_marker:
        failures.append(f"{label}.native_runtime_line does not advertise native library availability")
    modules = advertised_native_modules(native_runtime_line)
    if not modules:
        failures.append(f"{label}.native_runtime_line does not advertise any native modules")
    for module in modules:
        key = f"native_{module}_loaded"
        if key not in summary:
            failures.append(f"{label}.{key} is missing for advertised native_{module}=true")
        elif summary[key] != "true":
            failures.append(f"{label}.{key}={summary[key]!r} expected=true for advertised native_{module}=true")


def validate_preflight_evidence(
    path: pathlib.Path,
    label: str,
    failures: list[str],
) -> None:
    values = parse_kv_lines(path, label, failures)
    require_kv_true(values, PREFLIGHT_TRUE_KEYS, label, failures)
    require_kv_present(values, PREFLIGHT_REQUIRED_KEYS, label, failures)
    require_kv_int(values, "cpu_count", label, failures, minimum=1)
    require_kv_float(values, "load_per_cpu", label, failures, maximum=0.75)
    require_kv_float(values, "idle_percent_1s", label, failures, minimum=40.0)
    if values.get("host_preflight_host_checked") == "false":
        failures.append(f"{label}.host_preflight_host_checked=false expected=true for production evidence")
    if values.get("host_preflight_host_ok") == "false":
        failures.append(f"{label}.host_preflight_host_ok=false expected=true for production evidence")


def validate_resources_evidence(
    path: pathlib.Path,
    label: str,
    failures: list[str],
) -> None:
    columns = read_csv_header(path, label, failures)
    missing = sorted(RESOURCES_REQUIRED_COLUMNS - columns)
    for column in missing:
        failures.append(f"{label}: missing column {column}")


def require_send_pressure_metrics(
    values: dict[str, str],
    label: str,
    failures: list[str],
    *,
    prefix: str = "",
) -> None:
    for key in SEND_PRESSURE_INT_KEYS:
        if key == "compat_probe_send_pressure_samples":
            minimum = 1
        elif key == "compat_probe_send_bytes_before_unwritable_min":
            minimum = None
        else:
            minimum = 0
        require_kv_int(values, f"{prefix}{key}", label, failures, minimum=minimum)
    for key in SEND_PRESSURE_FLOAT_KEYS:
        require_kv_float(values, f"{prefix}{key}", label, failures, minimum=0.0)


def validate_claim(data: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    if data.get("schema") != SCHEMA:
        failures.append(f"bundle.schema={data.get('schema')!r} expected={SCHEMA!r}")
    if data.get("allowed_claim") != ALLOWED_CLAIM:
        failures.append("bundle.allowed_claim is not the measured 500-bot production claim")

    required_index_evidence = data.get("required_evidence")
    if not isinstance(required_index_evidence, list):
        failures.append("bundle.required_evidence is missing or not a list")
    else:
        for logical in REQUIRED_INDEX_EVIDENCE:
            if logical not in required_index_evidence:
                failures.append(f"bundle.required_evidence missing {logical}")

    claim = data.get("claim")
    if not isinstance(claim, dict):
        failures.append("bundle.claim is missing or not an object")
        return {}
    if claim.get("claim_text") != CLAIM_TEXT:
        failures.append(f"claim.claim_text={claim.get('claim_text')!r} expected={CLAIM_TEXT!r}")
    if claim.get("claim_scope") != CLAIM_SCOPE:
        failures.append(f"claim.claim_scope={claim.get('claim_scope')!r} expected={CLAIM_SCOPE!r}")
    for key in REQUIRED_TRUE_CLAIM_KEYS:
        require_json_true(claim, key, "claim", failures)
    if claim.get("failure_count") != 0:
        failures.append(f"claim.failure_count={claim.get('failure_count')!r} expected=0")
    if not isinstance(claim.get("artifact_hash_count"), int) or claim["artifact_hash_count"] < 1:
        failures.append("claim.artifact_hash_count must be a positive int")
    if not isinstance(claim.get("repeat_passes"), int) or claim["repeat_passes"] < 3:
        failures.append("claim.repeat_passes must be >= 3")
    if not is_sha256(claim.get("optimized_artifact_sha256")):
        failures.append("claim.optimized_artifact_sha256 must be a 64-char sha256")
    if not is_sha256(claim.get("optimized_runtime_run_sh_sha256")):
        failures.append("claim.optimized_runtime_run_sh_sha256 must be a 64-char sha256")
    if not is_sha256(claim.get("optimized_runtime_native_library_sha256")):
        failures.append("claim.optimized_runtime_native_library_sha256 must be a 64-char sha256")
    validate_measured_surface(data, failures)
    return claim


def validate_measured_surface(data: dict[str, Any], failures: list[str]) -> None:
    surface = data.get("measured_load_surface")
    if not isinstance(surface, dict):
        failures.append("bundle.measured_load_surface is missing or not an object")
        return
    for side in ("cold", "warm"):
        values = surface.get(side)
        if not isinstance(values, dict):
            failures.append(f"bundle.measured_load_surface.{side} is missing or not an object")
            continue
        tps_avg = values.get("tps1_avg")
        tps_min = values.get("tps1_min")
        mspt_max = values.get("avg_tick_ms_max")
        place_packets = values.get("block_place_packets")
        dig_packets = values.get("block_dig_packets")
        for key, value in [
            ("tps1_avg", tps_avg),
            ("tps1_min", tps_min),
            ("avg_tick_ms_max", mspt_max),
            ("block_place_packets", place_packets),
            ("block_dig_packets", dig_packets),
        ]:
            if not isinstance(value, (int, float)):
                failures.append(f"bundle.measured_load_surface.{side}.{key} is missing or not numeric")
        if isinstance(tps_avg, (int, float)) and tps_avg < MIN_TPS_AVG:
            failures.append(f"bundle.measured_load_surface.{side}.tps1_avg={tps_avg} expected>={MIN_TPS_AVG}")
        if isinstance(tps_min, (int, float)) and tps_min < MIN_TPS_MIN:
            failures.append(f"bundle.measured_load_surface.{side}.tps1_min={tps_min} expected>={MIN_TPS_MIN}")
        if isinstance(mspt_max, (int, float)) and mspt_max > MAX_MSPT_MAX:
            failures.append(f"bundle.measured_load_surface.{side}.avg_tick_ms_max={mspt_max} expected<={MAX_MSPT_MAX}")
        if isinstance(place_packets, (int, float)) and place_packets < MIN_BLOCK_PACKETS:
            failures.append(
                f"bundle.measured_load_surface.{side}.block_place_packets={place_packets} expected>={MIN_BLOCK_PACKETS}"
            )
        if isinstance(dig_packets, (int, float)) and dig_packets < MIN_BLOCK_PACKETS:
            failures.append(
                f"bundle.measured_load_surface.{side}.block_dig_packets={dig_packets} expected>={MIN_BLOCK_PACKETS}"
            )


def validate_evidence_files(
    bundle_dir: pathlib.Path,
    data: dict[str, Any],
    manifest_records: dict[str, dict[str, str]],
    failures: list[str],
) -> dict[str, dict[str, Any]]:
    evidence_files = data.get("evidence_files")
    if not isinstance(evidence_files, list) or not evidence_files:
        failures.append("bundle.evidence_files must be a non-empty list")
        return {}

    records: dict[str, dict[str, Any]] = {}
    for index, raw_record in enumerate(evidence_files):
        label = f"evidence_files[{index}]"
        if not isinstance(raw_record, dict):
            failures.append(f"{label}: record is not an object")
            continue
        logical = raw_record.get("logical_name")
        if not isinstance(logical, str) or not logical:
            failures.append(f"{label}.logical_name is missing")
            continue
        if logical in records:
            failures.append(f"{label}.logical_name duplicates {logical}")
            continue
        expected_sha = raw_record.get("sha256")
        expected_bytes = raw_record.get("bytes")
        if not is_sha256(expected_sha):
            failures.append(f"{logical}.sha256 is missing or not a 64-char sha256")
        if not isinstance(expected_bytes, int) or expected_bytes < 0:
            failures.append(f"{logical}.bytes is missing or not a non-negative int")
        path = bundle_relative_path(bundle_dir, raw_record.get("relative_path"), logical, failures)
        if path is None:
            continue
        observed_sha = None
        observed_bytes = None
        if not path.is_file():
            failures.append(f"{logical}: {path} is missing")
        else:
            observed_sha = sha256(path)
            observed_bytes = path.stat().st_size
            if is_sha256(expected_sha) and observed_sha != expected_sha:
                failures.append(f"{logical}: sha256={observed_sha} expected={expected_sha}")
            if isinstance(expected_bytes, int) and observed_bytes != expected_bytes:
                failures.append(f"{logical}: bytes={observed_bytes} expected={expected_bytes}")
            if observed_bytes == 0:
                failures.append(f"{logical}: file is empty")

        manifest = manifest_records.get(logical)
        if manifest is None:
            failures.append(f"{logical}: missing from MANIFEST.txt")
        else:
            if is_sha256(expected_sha) and manifest.get("sha256") != expected_sha:
                failures.append(f"{logical}: manifest sha256={manifest.get('sha256')} expected={expected_sha}")
            if isinstance(expected_bytes, int) and manifest.get("bytes") != str(expected_bytes):
                failures.append(f"{logical}: manifest bytes={manifest.get('bytes')} expected={expected_bytes}")

        records[logical] = {
            **raw_record,
            "path": path,
            "observed_sha256": observed_sha,
            "observed_bytes": observed_bytes,
        }

    for logical in REQUIRED_EVIDENCE:
        if logical not in records:
            failures.append(f"required_evidence: missing {logical}")
    return records


def validate_readiness_report(
    evidence: dict[str, dict[str, Any]], claim: dict[str, Any], failures: list[str]
) -> dict[str, str]:
    record = evidence.get("production-500-readiness-gate.txt")
    path = record.get("path") if record else None
    if not isinstance(path, pathlib.Path):
        failures.append("readiness: production-500-readiness-gate.txt evidence missing")
        return {}
    values = parse_kv_lines(path, "readiness", failures)
    require_kv_present(values, READINESS_REQUIRED_KEYS, "readiness", failures)
    require_kv_true(values, READINESS_TRUE_KEYS, "readiness", failures)
    require_kv_zero(values, ["failure_count"], "readiness", failures)
    if values.get("claim_text") != CLAIM_TEXT:
        failures.append(f"readiness.claim_text={values.get('claim_text')!r} expected={CLAIM_TEXT!r}")
    if values.get("claim_scope") != CLAIM_SCOPE:
        failures.append(f"readiness.claim_scope={values.get('claim_scope')!r} expected={CLAIM_SCOPE!r}")
    if values.get("optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append("readiness.optimized_artifact_sha256 does not match bundle claim")
    if values.get("current_optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append("readiness.current_optimized_artifact_sha256 does not match bundle claim")
    if values.get("current_optimized_runtime_run_sh_sha256") != claim.get("optimized_runtime_run_sh_sha256"):
        failures.append("readiness.current_optimized_runtime_run_sh_sha256 does not match bundle claim")
    if values.get("current_optimized_runtime_native_library_sha256") not in {None, claim.get("optimized_runtime_native_library_sha256")}:
        failures.append("readiness.current_optimized_runtime_native_library_sha256 does not match bundle claim")
    if values.get("artifact_hash_count") != str(claim.get("artifact_hash_count")):
        failures.append("readiness.artifact_hash_count does not match bundle claim")
    if values.get("repeat_passes") != str(claim.get("repeat_passes")):
        failures.append("readiness.repeat_passes does not match bundle claim")
    for hash_key, logical in READINESS_HASH_KEYS.items():
        expected = values.get(hash_key)
        observed = evidence.get(logical, {}).get("observed_sha256")
        if expected != observed:
            failures.append(f"readiness.{hash_key}={expected} observed_bundle_sha256={observed}")
    for side in ("cold", "warm"):
        require_kv_float(values, f"{side}_load_window_tps1_avg", "readiness", failures, minimum=MIN_TPS_AVG)
        require_kv_float(values, f"{side}_load_window_tps1_min", "readiness", failures, minimum=MIN_TPS_MIN)
        require_kv_float(values, f"{side}_load_window_avg_tick_ms_max", "readiness", failures, maximum=MAX_MSPT_MAX)
        require_kv_int(values, f"{side}_bot_block_place_packets_max", "readiness", failures, minimum=MIN_BLOCK_PACKETS)
        require_kv_int(values, f"{side}_bot_block_dig_packets_max", "readiness", failures, minimum=MIN_BLOCK_PACKETS)
    return values


def validate_soak_report(
    evidence: dict[str, dict[str, Any]],
    claim: dict[str, Any],
    failures: list[str],
    *,
    check_referenced_summaries: bool,
) -> dict[str, str]:
    record = evidence.get("production-500-soak-gate.txt")
    path = record.get("path") if record else None
    if not isinstance(path, pathlib.Path):
        failures.append("soak: production-500-soak-gate.txt evidence missing")
        return {}
    values = parse_kv_lines(path, "soak", failures)
    require_kv_present(values, SOAK_REQUIRED_KEYS, "soak", failures)
    require_kv_true(values, SOAK_TRUE_KEYS, "soak", failures)
    require_kv_zero(
        values,
        [
            "failure_count",
            "cold_failure_count",
            "warm_failure_count",
            "cold_bot_block_action_errors_max",
            "warm_bot_block_action_errors_max",
            "cold_stability_failures",
            "warm_stability_failures",
            "cold_watchdog_thread_dumps",
            "warm_watchdog_thread_dumps",
            "cold_sync_load_stack_hits",
            "warm_sync_load_stack_hits",
        ],
        "soak",
        failures,
    )
    if values.get("optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append("soak.optimized_artifact_sha256 does not match bundle claim")
    if values.get("cold_optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append("soak.cold_optimized_artifact_sha256 does not match bundle claim")
    if values.get("warm_optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append("soak.warm_optimized_artifact_sha256 does not match bundle claim")
    if values.get("cold_optimized_runtime_run_sh_sha256") != claim.get("optimized_runtime_run_sh_sha256"):
        failures.append("soak.cold_optimized_runtime_run_sh_sha256 does not match bundle claim")
    if values.get("warm_optimized_runtime_run_sh_sha256") != claim.get("optimized_runtime_run_sh_sha256"):
        failures.append("soak.warm_optimized_runtime_run_sh_sha256 does not match bundle claim")
    if values.get("artifact_hash_count") != str(claim.get("artifact_hash_count")):
        failures.append("soak.artifact_hash_count does not match bundle claim")
    if values.get("cold_summary_path") == values.get("warm_summary_path") and values.get("cold_summary_path"):
        failures.append("soak.cold_summary_path and soak.warm_summary_path must differ")

    expected_world_modes = {"cold": "fresh", "warm": "warm-source"}
    expected_surfaces = {"cold": "cold-fresh", "warm": "warm-world"}
    for side in ("cold", "warm"):
        require_kv_int(values, f"{side}_bots", "soak", failures, expected=EXPECTED_BOTS)
        require_kv_int(values, f"{side}_view_distance", "soak", failures, expected=EXPECTED_DISTANCE)
        require_kv_int(values, f"{side}_simulation_distance", "soak", failures, expected=EXPECTED_DISTANCE)
        require_kv_int(values, f"{side}_duration_seconds", "soak", failures, expected=EXPECTED_DURATION_SECONDS)
        if values.get(f"{side}_world_mode") != expected_world_modes[side]:
            failures.append(
                f"soak.{side}_world_mode={values.get(f'{side}_world_mode')!r} expected={expected_world_modes[side]!r}"
            )
        if values.get(f"{side}_claim_surface") != expected_surfaces[side]:
            failures.append(
                f"soak.{side}_claim_surface={values.get(f'{side}_claim_surface')!r} expected={expected_surfaces[side]!r}"
            )
        if values.get(f"{side}_load_test_scenario") != "block":
            failures.append(f"soak.{side}_load_test_scenario={values.get(f'{side}_load_test_scenario')!r} expected='block'")
        if values.get(f"{side}_load_test_gamemode") != "creative":
            failures.append(f"soak.{side}_load_test_gamemode={values.get(f'{side}_load_test_gamemode')!r} expected='creative'")
        require_kv_int(values, f"{side}_load_window_metrics_samples", "soak", failures, minimum=MIN_LOAD_WINDOW_SAMPLES)
        require_kv_int(values, f"{side}_load_window_online_max", "soak", failures, expected=EXPECTED_BOTS)
        require_kv_float(values, f"{side}_load_window_tps1_avg", "soak", failures, minimum=MIN_TPS_AVG)
        require_kv_float(values, f"{side}_load_window_tps1_min", "soak", failures, minimum=MIN_TPS_MIN)
        require_kv_float(values, f"{side}_load_window_avg_tick_ms_max", "soak", failures, maximum=MAX_MSPT_MAX)
        require_kv_int(values, f"{side}_bot_block_place_packets_max", "soak", failures, minimum=MIN_BLOCK_PACKETS)
        require_kv_int(values, f"{side}_bot_block_dig_packets_max", "soak", failures, minimum=MIN_BLOCK_PACKETS)
        require_send_pressure_metrics(values, "soak", failures, prefix=f"{side}_")
        validate_summary_reference(
            evidence,
            values,
            side,
            claim,
            failures,
            check_referenced_summaries=check_referenced_summaries,
        )
    return values


def validate_summary_reference(
    evidence: dict[str, dict[str, Any]],
    soak_values: dict[str, str],
    side: str,
    claim: dict[str, Any],
    failures: list[str],
    *,
    check_referenced_summaries: bool,
) -> None:
    key = f"{side}_summary_path"
    raw = soak_values.get(key)
    if not raw:
        failures.append(f"soak.{key} is missing")
        return
    name = pathlib.PurePath(raw).name
    if "summary" not in name:
        failures.append(f"soak.{key}={raw!r} does not look like a summary file")
    if side not in name:
        failures.append(f"soak.{key}={raw!r} does not contain {side!r}")
    if not check_referenced_summaries:
        return

    label = f"{side}_summary"
    bundled_summary = evidence_path_by_logical(evidence, name)
    if bundled_summary is None:
        failures.append(f"{label}: referenced summary {name} is missing from bundle evidence")
    summary_path = bundled_summary if bundled_summary is not None else resolve_repo_path(raw)
    summary_tokens = parse_kv_tokens(summary_path, label, failures)
    summary_lines = parse_summary_kv_lines(summary_path, label, failures)
    if not summary_tokens:
        return
    require_kv_present(summary_tokens, SUMMARY_KEYS, label, failures)
    require_kv_present(summary_tokens, SUMMARY_HOST_KEYS, label, failures)
    validate_native_summary_fields(summary_lines, label, failures)
    require_kv_int(summary_tokens, "bots", label, failures, expected=EXPECTED_BOTS)
    require_kv_int(summary_tokens, "view_distance", label, failures, expected=EXPECTED_DISTANCE)
    require_kv_int(summary_tokens, "simulation_distance", label, failures, expected=EXPECTED_DISTANCE)
    require_kv_int(summary_tokens, "duration_seconds", label, failures, expected=EXPECTED_DURATION_SECONDS)
    require_kv_int(summary_tokens, "load_window_metrics_samples", label, failures, minimum=MIN_LOAD_WINDOW_SAMPLES)
    require_kv_int(summary_tokens, "load_window_online_max", label, failures, expected=EXPECTED_BOTS)
    require_kv_float(summary_tokens, "load_window_tps1_avg", label, failures, minimum=MIN_TPS_AVG)
    require_kv_float(summary_tokens, "load_window_tps1_min", label, failures, minimum=MIN_TPS_MIN)
    require_kv_float(summary_tokens, "load_window_avg_tick_ms_max", label, failures, maximum=MAX_MSPT_MAX)
    require_kv_int(summary_tokens, "bot_block_place_packets_max", label, failures, minimum=MIN_BLOCK_PACKETS)
    require_kv_int(summary_tokens, "bot_block_dig_packets_max", label, failures, minimum=MIN_BLOCK_PACKETS)
    require_kv_zero(summary_tokens, ["bot_block_action_errors_max"], label, failures)
    require_send_pressure_metrics(summary_tokens, label, failures)
    require_kv_int(summary_tokens, "resource_samples", label, failures, minimum=MIN_LOAD_WINDOW_SAMPLES)
    require_kv_int(summary_tokens, "host_cpu_count", label, failures, minimum=1)
    require_kv_float(summary_tokens, "host_system_load1_per_cpu_max", label, failures)
    if summary_tokens.get("optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
        failures.append(f"{label}.optimized_artifact_sha256 does not match bundle claim")
    if summary_tokens.get("optimized_runtime_run_sh_sha256") != claim.get("optimized_runtime_run_sh_sha256"):
        failures.append(f"{label}.optimized_runtime_run_sh_sha256 does not match bundle claim")
    if summary_tokens.get("load_window_reached_full_online") != "true":
        failures.append(f"{label}.load_window_reached_full_online={summary_tokens.get('load_window_reached_full_online')!r} expected=true")
    expected_world_mode = "fresh" if side == "cold" else "warm-source"
    expected_surface = "cold-fresh" if side == "cold" else "warm-world"
    if summary_tokens.get("world_mode") != expected_world_mode:
        failures.append(f"{label}.world_mode={summary_tokens.get('world_mode')!r} expected={expected_world_mode!r}")
    if summary_tokens.get("claim_surface") != expected_surface:
        failures.append(f"{label}.claim_surface={summary_tokens.get('claim_surface')!r} expected={expected_surface!r}")
    if summary_tokens.get("load_test_scenario") != "block":
        failures.append(f"{label}.load_test_scenario={summary_tokens.get('load_test_scenario')!r} expected='block'")
    if summary_tokens.get("load_test_gamemode") != "creative":
        failures.append(f"{label}.load_test_gamemode={summary_tokens.get('load_test_gamemode')!r} expected='creative'")

    pure = pathlib.PurePath(raw)
    preflight_logical = replace_summary_suffix(pure, "-preflight.txt")
    resources_logical = replace_summary_suffix(pure, "-resources.csv")
    if preflight_logical is None:
        failures.append(f"{label}: cannot derive preflight path from {raw!r}")
    else:
        preflight_path = evidence_path_by_logical(evidence, preflight_logical)
        if preflight_path is None:
            failures.append(f"{label}: referenced preflight {preflight_logical} is missing from bundle evidence")
        else:
            validate_preflight_evidence(preflight_path, f"{label}_preflight", failures)
    if resources_logical is None:
        failures.append(f"{label}: cannot derive resources path from {raw!r}")
    else:
        resources_path = evidence_path_by_logical(evidence, resources_logical)
        if resources_path is None:
            failures.append(f"{label}: referenced resources {resources_logical} is missing from bundle evidence")
        else:
            validate_resources_evidence(resources_path, f"{label}_resources", failures)


def validate_repeat_report(
    evidence: dict[str, dict[str, Any]], claim: dict[str, Any], failures: list[str]
) -> dict[str, str]:
    record = evidence.get("production-500-repeat-quorum.txt")
    path = record.get("path") if record else None
    if not isinstance(path, pathlib.Path):
        failures.append("repeat: production-500-repeat-quorum.txt evidence missing")
        return {}
    values = parse_kv_lines(path, "repeat", failures)
    require_kv_present(values, REPEAT_REQUIRED_KEYS, "repeat", failures)
    require_kv_true(values, ["repeat_quorum_pass"], "repeat", failures)
    require_kv_zero(values, ["repeat_failures"], "repeat", failures)
    repeat_passes = require_kv_int(values, "repeat_passes", "repeat", failures, minimum=3)
    require_kv_int(values, "required_min_passes", "repeat", failures, minimum=3)
    require_kv_int(values, "repeat_run_count", "repeat", failures, minimum=3)
    if repeat_passes is not None and repeat_passes != claim.get("repeat_passes"):
        failures.append("repeat.repeat_passes does not match bundle claim")
    for run_number in range(1, 4):
        prefix = f"run_{run_number}"
        require_kv_present(
            values,
            [
                f"{prefix}_dir",
                f"{prefix}_pass",
                f"{prefix}_production_ready_claim_eligible",
                f"{prefix}_release_gate_pass",
                f"{prefix}_failure_count",
                f"{prefix}_cold_load_window_tps1_avg",
                f"{prefix}_cold_load_window_tps1_min",
                f"{prefix}_cold_load_window_avg_tick_ms_max",
                f"{prefix}_warm_load_window_tps1_avg",
                f"{prefix}_warm_load_window_tps1_min",
                f"{prefix}_warm_load_window_avg_tick_ms_max",
                f"{prefix}_optimized_artifact_sha256",
            ],
            "repeat",
            failures,
        )
        require_kv_true(
            values,
            [
                f"{prefix}_pass",
                f"{prefix}_production_ready_claim_eligible",
                f"{prefix}_release_gate_pass",
            ],
            "repeat",
            failures,
        )
        require_kv_zero(values, [f"{prefix}_failure_count"], "repeat", failures)
        if values.get(f"{prefix}_optimized_artifact_sha256") != claim.get("optimized_artifact_sha256"):
            failures.append(f"repeat.{prefix}_optimized_artifact_sha256 does not match bundle claim")
        for side in ("cold", "warm"):
            require_kv_float(values, f"{prefix}_{side}_load_window_tps1_avg", "repeat", failures, minimum=MIN_TPS_AVG)
            require_kv_float(values, f"{prefix}_{side}_load_window_tps1_min", "repeat", failures, minimum=MIN_TPS_MIN)
            require_kv_float(values, f"{prefix}_{side}_load_window_avg_tick_ms_max", "repeat", failures, maximum=MAX_MSPT_MAX)
    return values


def validate_artifact_data(
    evidence: dict[str, dict[str, Any]], claim: dict[str, Any], failures: list[str]
) -> None:
    artifact_record = evidence.get("artifact-hashes.txt")
    artifact_path = artifact_record.get("path") if artifact_record else None
    if not isinstance(artifact_path, pathlib.Path):
        failures.append("artifact_hashes: artifact-hashes.txt evidence missing")
        return
    hash_rows = parse_artifact_hashes(artifact_path, failures)
    claim_hash_count = claim.get("artifact_hash_count")
    if isinstance(claim_hash_count, int) and len(hash_rows) < claim_hash_count:
        failures.append(f"artifact_hashes: {len(hash_rows)} rows expected>={claim_hash_count}")
    claim_sha = claim.get("optimized_artifact_sha256")
    if is_sha256(claim_sha) and claim_sha not in {row[0] for row in hash_rows}:
        failures.append("artifact_hashes: optimized_artifact_sha256 is not present")
    claim_run_sh_sha = claim.get("optimized_runtime_run_sh_sha256")
    if is_sha256(claim_run_sh_sha) and claim_run_sh_sha not in {row[0] for row in hash_rows}:
        failures.append("artifact_hashes: optimized_runtime_run_sh_sha256 is not present")
    claim_native_sha = claim.get("optimized_runtime_native_library_sha256")
    if is_sha256(claim_native_sha):
        if not any(row[0] == claim_native_sha and row[1].endswith("libpaper_native_jni.so") for row in hash_rows):
            failures.append("artifact_hashes: optimized_runtime_native_library_sha256 is not present for libpaper_native_jni.so")

    artifacts_record = evidence.get("artifacts.json")
    artifacts_path = artifacts_record.get("path") if artifacts_record else None
    if not isinstance(artifacts_path, pathlib.Path):
        failures.append("artifacts_json: artifacts.json evidence missing")
        return
    artifacts = read_json_object(artifacts_path, "artifacts_json", failures)
    optimized = artifacts.get("optimized") if artifacts else None
    if not isinstance(optimized, dict):
        failures.append("artifacts_json.optimized is missing or not an object")
    elif optimized.get("sha256") != claim_sha:
        failures.append("artifacts_json.optimized.sha256 does not match bundle claim")
    runtime = artifacts.get("optimized_runtime") if artifacts else None
    if not isinstance(runtime, dict):
        failures.append("artifacts_json.optimized_runtime is missing or not an object")
    else:
        run_sh = runtime.get("run_sh")
        if not isinstance(run_sh, dict) or not is_sha256(run_sh.get("sha256")):
            failures.append("artifacts_json.optimized_runtime.run_sh.sha256 is missing")
        elif run_sh.get("sha256") != claim.get("optimized_runtime_run_sh_sha256"):
            failures.append("artifacts_json.optimized_runtime.run_sh.sha256 does not match bundle claim")
        runtime_jar = runtime.get("runtime_jar_sha256_file")
        if not isinstance(runtime_jar, dict) or runtime_jar.get("runtime_jar_sha256") != claim_sha:
            failures.append("artifacts_json.optimized_runtime.runtime_jar_sha256_file.runtime_jar_sha256 does not match bundle claim")
        native_library = runtime.get("native_library")
        if not isinstance(native_library, dict) or not is_sha256(native_library.get("sha256")):
            failures.append("artifacts_json.optimized_runtime.native_library.sha256 is missing")
        elif native_library.get("sha256") != claim_native_sha:
            failures.append("artifacts_json.optimized_runtime.native_library.sha256 does not match bundle claim")
        native_library_hash_file = runtime.get("native_library_sha256_file")
        if not isinstance(native_library_hash_file, dict):
            failures.append("artifacts_json.optimized_runtime.native_library_sha256_file is missing")
        elif native_library_hash_file.get("native_library_sha256") != claim_native_sha:
            failures.append(
                "artifacts_json.optimized_runtime.native_library_sha256_file.native_library_sha256 "
                "does not match bundle claim"
            )

    native_proof_found = False
    for logical in NATIVE_PROOF_EVIDENCE:
        proof_path = evidence_path_by_logical(evidence, logical)
        if proof_path is None:
            continue
        native_proof_found = True
        proof_rows = parse_artifact_hashes(proof_path, failures)
        if is_sha256(claim_native_sha) and not any(row[0] == claim_native_sha for row in proof_rows):
            failures.append(f"{logical}: does not contain optimized_runtime_native_library_sha256")
    if not native_proof_found:
        failures.append(
            "native_library_sha256_proof: missing libpaper_native_jni.so.sha256 or paper-native-jni.sha256 evidence"
        )


def parse_artifact_hashes(path: pathlib.Path, failures: list[str]) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line:
                continue
            parts = line.split(None, 1)
            if len(parts) != 2:
                failures.append(f"artifact_hashes:{line_no}: expected sha256 and path")
                continue
            digest, artifact_path = parts
            if not is_sha256(digest):
                failures.append(f"artifact_hashes:{line_no}: invalid sha256 {digest!r}")
            if not artifact_path:
                failures.append(f"artifact_hashes:{line_no}: artifact path is empty")
            rows.append((digest, artifact_path))
    if not rows:
        failures.append("artifact_hashes: no hash rows found")
    return rows


def validate_summary_evidence(evidence: dict[str, dict[str, Any]], failures: list[str]) -> None:
    plugin_path = evidence.get("plugin-matrix-summary.txt", {}).get("path")
    if isinstance(plugin_path, pathlib.Path) and plugin_path.is_file():
        text = plugin_path.read_text(encoding="utf-8", errors="replace")
        values = parse_kv_lines(plugin_path, "plugin_matrix", failures)
        require_bundled_log_reference(
            evidence,
            values,
            "plugin_matrix",
            "plugin_matrix_log",
            failures,
        )
        for snippet in ["status_json=", "Initialized 11 plugins", "COMPAT_PROBE lifecycle=enable"]:
            if snippet not in text:
                failures.append(f"plugin-matrix-summary.txt: missing {snippet!r}")
    restart_path = evidence.get("restart-recovery-summary.txt", {}).get("path")
    if isinstance(restart_path, pathlib.Path) and restart_path.is_file():
        text = restart_path.read_text(encoding="utf-8", errors="replace")
        values = parse_kv_lines(restart_path, "restart_recovery", failures)
        require_bundled_log_reference(
            evidence,
            values,
            "restart_recovery",
            "restart_recovery_log",
            failures,
        )
        for snippet in ["status_json=", "Saved the game", "COMPAT_PROBE lifecycle=disable"]:
            if snippet not in text:
                failures.append(f"restart-recovery-summary.txt: missing {snippet!r}")
    forced_path = evidence.get("forced-ticket-persistence-summary.txt", {}).get("path")
    if isinstance(forced_path, pathlib.Path) and forced_path.is_file():
        values = parse_kv_lines(forced_path, "forced_ticket", failures)
        require_bundled_log_reference(
            evidence,
            values,
            "forced_ticket",
            "first_log",
            failures,
        )
        require_bundled_log_reference(
            evidence,
            values,
            "forced_ticket",
            "restart_log",
            failures,
        )
        if values.get("forced_ticket_persistence") != "PASS":
            failures.append(
                f"forced_ticket.forced_ticket_persistence={values.get('forced_ticket_persistence')!r} expected=PASS"
            )


def validate_claim_markdown(bundle_dir: pathlib.Path, failures: list[str]) -> None:
    path = bundle_dir / "CLAIM.md"
    if not path.is_file():
        failures.append(f"claim_markdown: {path} is missing")
        return
    text = path.read_text(encoding="utf-8", errors="replace")
    for snippet in [
        "Production-ready for the measured 500-bot",
        "This claim is allowed only with the exact evidence bundle generated here.",
        "Not unlimited plugin compatibility",
        "MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh",
    ]:
        if snippet not in text:
            failures.append(f"claim_markdown: missing {snippet!r}")


def validate_bundle(
    bundle_dir: pathlib.Path,
    *,
    check_referenced_summaries: bool,
    reports_dir: pathlib.Path | None = None,
) -> dict[str, Any]:
    bundle_dir = bundle_dir.expanduser().resolve()
    failures: list[str] = []
    if not bundle_dir.is_dir():
        failures.append(f"bundle_dir: {bundle_dir} is missing")

    data = read_json_object(bundle_dir / "bundle.json", "bundle_index", failures)
    claim = validate_claim(data, failures) if data else {}
    manifest_records = parse_manifest(bundle_dir / "MANIFEST.txt", failures)
    evidence = validate_evidence_files(bundle_dir, data, manifest_records, failures) if data else {}

    readiness = validate_readiness_report(evidence, claim, failures) if evidence else {}
    soak = validate_soak_report(
        evidence,
        claim,
        failures,
        check_referenced_summaries=check_referenced_summaries,
    ) if evidence else {}
    repeat = validate_repeat_report(evidence, claim, failures) if evidence else {}
    if evidence:
        validate_artifact_data(evidence, claim, failures)
        validate_summary_evidence(evidence, failures)
    freshness: dict[str, Any] = {}
    if bundle_dir.name.endswith("-current") and data and claim:
        freshness = production_readiness_bundle.validate_current_artifact_freshness(
            bundle_dir,
            data,
            claim,
            reports_dir if reports_dir is not None else ROOT / "reports",
            failures,
        )
    validate_claim_markdown(bundle_dir, failures)

    return {
        "passed": not failures,
        "failures": failures,
        "bundle_dir": bundle_dir,
        "data": data,
        "claim": claim,
        "evidence": evidence,
        "readiness": readiness,
        "soak": soak,
        "repeat": repeat,
        "freshness": freshness,
    }


def print_result(result: dict[str, Any]) -> None:
    data = result["data"]
    claim = result["claim"]
    evidence = result["evidence"]
    soak = result["soak"]
    freshness = result.get("freshness") or {}
    print(f"evidence_bundle_validation_pass={str(result['passed']).lower()}")
    print(f"failure_count={len(result['failures'])}")
    print(f"bundle_dir={result['bundle_dir']}")
    print(f"schema={data.get('schema') if data else None}")
    print(f"evidence_file_count={len(evidence)}")
    print(f"artifact_hash_count={claim.get('artifact_hash_count') if claim else None}")
    print(f"optimized_artifact_sha256={claim.get('optimized_artifact_sha256') if claim else None}")
    print(
        "optimized_runtime_run_sh_sha256="
        f"{claim.get('optimized_runtime_run_sh_sha256') if claim else None}"
    )
    print(f"cold_summary_path={soak.get('cold_summary_path') if soak else None}")
    print(f"warm_summary_path={soak.get('warm_summary_path') if soak else None}")
    if freshness:
        print(f"current_freshness_required={str(True).lower()}")
        print(f"current_artifact_freshness_pass={str(freshness.get('passed')).lower()}")
        print(f"current_artifacts_json={freshness.get('artifacts_json')}")
        print(f"current_optimized_artifact_sha256={freshness.get('current_optimized_artifact_sha256')}")
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
    for failure in result["failures"]:
        print(f"evidence_bundle_validation_failure={failure}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "bundle_dir",
        nargs="?",
        default="reports/production-500-readiness-bundle-current",
        help="Path to a production 500 evidence bundle.",
    )
    parser.add_argument(
        "--skip-referenced-summary-file-check",
        action="store_true",
        help="Only require cold/warm summary references, not local referenced files.",
    )
    parser.add_argument(
        "--reports-dir",
        type=pathlib.Path,
        default=None,
        help="Directory containing current artifact evidence and artifacts.json.",
    )
    args = parser.parse_args()

    result = validate_bundle(
        pathlib.Path(args.bundle_dir),
        check_referenced_summaries=not args.skip_referenced_summary_file_check,
        reports_dir=args.reports_dir,
    )
    print_result(result)
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
