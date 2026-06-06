#!/usr/bin/env python3
"""Validate a stress-corpus boot/join self-contained evidence bundle."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import pathlib
import re
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = "stress-corpus-boot-join-bundle/v1"
ALLOWED_CLAIM = (
    "Stress-corpus boot/join compatibility gate passed for the measured "
    "Paper 1.21.10 artifact with the recorded plugin/datapack corpus."
)
REQUIRED_LOGICALS = [
    "stress-corpus-summary.txt",
    "stress-corpus-status.json",
    "stress-corpus-join.txt",
    "stress-corpus-hard-errors.txt",
    "stress-corpus-inspection.txt",
    "stress-corpus-inspection.json",
    "stress-corpus-artifacts.csv",
    "stress-corpus-manifest.json",
    "artifacts.json",
]
MIN_MATRIX_PLUGINS = 12
MIN_STRESS_PLUGINS = 22
MIN_TOTAL_PLUGINS = 34
MIN_DATAPACKS = 10
EXPECTED_STATUS_VERSION = "Paper 1.21.10"
EXPECTED_PROTOCOL = 773
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
SERVER_READY_RE = re.compile(
    r"^(?:(?:\x1b\[[0-9;?]*[ -/]*[@-~])|[>\t\r ])*"
    r"\[[0-9]{2}:[0-9]{2}:[0-9]{2} INFO\]: "
    r"Done \([0-9.]+s\)! For help, type \"help\""
    r"(?:(?:\x1b\[[0-9;?]*[ -/]*[@-~])|[>\t\r ])*$",
    re.MULTILINE,
)


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def is_sha256(value: Any) -> bool:
    return isinstance(value, str) and SHA256_RE.fullmatch(value) is not None


def parse_kv(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, str]:
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
            if key in values and values[key] != value:
                failures.append(f"{label}:{line_no}: duplicate key {key}")
            values[key] = value
    return values


def read_json(path: pathlib.Path, label: str, failures: list[str]) -> dict[str, Any]:
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


def bundle_path(bundle_dir: pathlib.Path, raw: Any, label: str, failures: list[str]) -> pathlib.Path | None:
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


def require_int(
    values: dict[str, str],
    key: str,
    label: str,
    failures: list[str],
    *,
    minimum: int | None = None,
    expected: int | None = None,
) -> int | None:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{label}.{key} is missing")
        return None
    try:
        parsed = int(raw)
    except ValueError:
        failures.append(f"{label}.{key}={raw!r} is not an int")
        return None
    if minimum is not None and parsed < minimum:
        failures.append(f"{label}.{key}={parsed} expected>={minimum}")
    if expected is not None and parsed != expected:
        failures.append(f"{label}.{key}={parsed} expected={expected}")
    return parsed


def require_float(
    values: dict[str, str],
    key: str,
    label: str,
    failures: list[str],
    *,
    minimum: float | None = None,
) -> float | None:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{label}.{key} is missing")
        return None
    try:
        parsed = float(raw)
    except ValueError:
        failures.append(f"{label}.{key}={raw!r} is not numeric")
        return None
    if minimum is not None and parsed < minimum:
        failures.append(f"{label}.{key}={parsed} expected>={minimum}")
    return parsed


def validate_evidence_files(
    bundle_dir: pathlib.Path,
    data: dict[str, Any],
    manifest: dict[str, dict[str, str]],
    failures: list[str],
) -> dict[str, pathlib.Path]:
    raw_records = data.get("evidence_files")
    paths: dict[str, pathlib.Path] = {}
    if not isinstance(raw_records, list):
        failures.append("bundle.evidence_files is missing or not a list")
        return paths
    for index, record in enumerate(raw_records):
        label = f"evidence_files[{index}]"
        if not isinstance(record, dict):
            failures.append(f"{label}: record is not an object")
            continue
        logical = record.get("logical_name")
        if not isinstance(logical, str) or not logical:
            failures.append(f"{label}.logical_name is missing")
            continue
        if logical in paths:
            failures.append(f"{label}.logical_name duplicates {logical}")
            continue
        expected_sha = record.get("sha256")
        expected_bytes = record.get("bytes")
        if not is_sha256(expected_sha):
            failures.append(f"{logical}.sha256 is missing or invalid")
        if not isinstance(expected_bytes, int) or expected_bytes < 0:
            failures.append(f"{logical}.bytes is missing or invalid")
        path = bundle_path(bundle_dir, record.get("relative_path"), logical, failures)
        if path is None:
            continue
        if not path.is_file():
            failures.append(f"{logical}: {path} is missing")
        else:
            observed_sha = sha256(path)
            observed_bytes = path.stat().st_size
            if is_sha256(expected_sha) and observed_sha != expected_sha:
                failures.append(f"{logical}: sha256={observed_sha} expected={expected_sha}")
            if isinstance(expected_bytes, int) and observed_bytes != expected_bytes:
                failures.append(f"{logical}: bytes={observed_bytes} expected={expected_bytes}")
            if observed_bytes == 0 and logical != "stress-corpus-hard-errors.txt":
                failures.append(f"{logical}: file is empty")
        manifest_record = manifest.get(logical)
        if manifest_record is None:
            failures.append(f"{logical}: missing from MANIFEST.txt")
        else:
            if is_sha256(expected_sha) and manifest_record.get("sha256") != expected_sha:
                failures.append(f"{logical}: manifest sha256 mismatch")
            if isinstance(expected_bytes, int) and manifest_record.get("bytes") != str(expected_bytes):
                failures.append(f"{logical}: manifest bytes mismatch")
        paths[logical] = path
    for logical in REQUIRED_LOGICALS:
        if logical not in paths:
            failures.append(f"required_evidence: missing {logical}")
    if "stress-corpus-gate.log" not in paths:
        failures.append("required_evidence: missing stress-corpus-gate.log")
    if "stress-corpus-influx-mock.log" not in paths:
        failures.append("required_evidence: missing stress-corpus-influx-mock.log")
    return paths


def validate_claim(data: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    if data.get("schema") != SCHEMA:
        failures.append(f"bundle.schema={data.get('schema')!r} expected={SCHEMA!r}")
    if data.get("allowed_claim") != ALLOWED_CLAIM:
        failures.append("bundle.allowed_claim is not the stress-corpus boot/join claim")
    claim = data.get("claim")
    if not isinstance(claim, dict):
        failures.append("bundle.claim is missing or not an object")
        return {}
    if claim.get("stress_corpus_boot_join_claim") is not True:
        failures.append("claim.stress_corpus_boot_join_claim expected=True")
    if claim.get("stress_corpus_gate") is not True:
        failures.append("claim.stress_corpus_gate expected=True")
    if claim.get("failure_reason") != "none":
        failures.append(f"claim.failure_reason={claim.get('failure_reason')!r} expected='none'")
    numeric_minimums = {
        "matrix_plugin_count": MIN_MATRIX_PLUGINS,
        "stress_plugin_count": MIN_STRESS_PLUGINS,
        "plugin_count": MIN_TOTAL_PLUGINS,
        "datapack_count": MIN_DATAPACKS,
    }
    for key, minimum in numeric_minimums.items():
        value = claim.get(key)
        if not isinstance(value, int):
            failures.append(f"claim.{key} is missing or not an int")
        elif value < minimum:
            failures.append(f"claim.{key}={value} expected>={minimum}")
    if claim.get("plugin_count") != claim.get("matrix_plugin_count", 0) + claim.get("stress_plugin_count", 0):
        failures.append("claim.plugin_count does not equal matrix_plugin_count + stress_plugin_count")
    if not isinstance(claim.get("done_seconds"), (int, float)) or claim.get("done_seconds") <= 0:
        failures.append("claim.done_seconds must be positive")
    if claim.get("status_version_name") != EXPECTED_STATUS_VERSION:
        failures.append(f"claim.status_version_name={claim.get('status_version_name')!r} expected={EXPECTED_STATUS_VERSION!r}")
    if claim.get("status_protocol") != EXPECTED_PROTOCOL:
        failures.append(f"claim.status_protocol={claim.get('status_protocol')!r} expected={EXPECTED_PROTOCOL}")
    for key in (
        "optimized_artifact_sha256",
        "optimized_runtime_run_sh_sha256",
        "optimized_runtime_native_library_sha256",
    ):
        if not is_sha256(claim.get(key)):
            failures.append(f"claim.{key} must be a 64-char sha256")
    limits = claim.get("claim_limits")
    if not isinstance(limits, list) or "not-player-capacity-or-tps-claim" not in limits:
        failures.append("claim.claim_limits missing not-player-capacity-or-tps-claim")
    return claim


def validate_summary(paths: dict[str, pathlib.Path], claim: dict[str, Any], failures: list[str]) -> dict[str, str]:
    values = parse_kv(paths["stress-corpus-summary.txt"], "summary", failures)
    if values.get("stress_corpus_gate") != "PASS":
        failures.append(f"summary.stress_corpus_gate={values.get('stress_corpus_gate')!r} expected=PASS")
    if values.get("failure_reason") != "none":
        failures.append(f"summary.failure_reason={values.get('failure_reason')!r} expected=none")
    matrix = require_int(values, "matrix_plugin_count", "summary", failures, minimum=MIN_MATRIX_PLUGINS)
    stress = require_int(values, "stress_plugin_count", "summary", failures, minimum=MIN_STRESS_PLUGINS)
    total = require_int(values, "plugin_count", "summary", failures, minimum=MIN_TOTAL_PLUGINS)
    require_int(values, "datapack_count", "summary", failures, minimum=MIN_DATAPACKS)
    require_float(values, "done_seconds", "summary", failures, minimum=1.0)
    if matrix is not None and stress is not None and total is not None and total != matrix + stress:
        failures.append("summary.plugin_count does not equal matrix_plugin_count + stress_plugin_count")
    for key in ("optimized_artifact_sha256", "optimized_runtime_run_sh_sha256", "optimized_runtime_native_library_sha256"):
        if values.get(key) != claim.get(key):
            failures.append(f"summary.{key} does not match bundle claim")
    return values


def validate_status(paths: dict[str, pathlib.Path], claim: dict[str, Any], failures: list[str]) -> None:
    status = read_json(paths["stress-corpus-status.json"], "status", failures)
    version = status.get("version") if status else None
    if not isinstance(version, dict):
        failures.append("status.version is missing or not an object")
        return
    if version.get("name") != EXPECTED_STATUS_VERSION:
        failures.append(f"status.version.name={version.get('name')!r} expected={EXPECTED_STATUS_VERSION!r}")
    if version.get("protocol") != EXPECTED_PROTOCOL:
        failures.append(f"status.version.protocol={version.get('protocol')!r} expected={EXPECTED_PROTOCOL}")
    if version.get("name") != claim.get("status_version_name"):
        failures.append("status.version.name does not match bundle claim")
    if version.get("protocol") != claim.get("status_protocol"):
        failures.append("status.version.protocol does not match bundle claim")


def validate_join_and_logs(paths: dict[str, pathlib.Path], failures: list[str]) -> None:
    join_lines = []
    join_attempt = None
    with paths["stress-corpus-join.txt"].open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line:
                continue
            join_lines.append(line)
            if line.startswith("join_attempt="):
                if join_attempt is not None:
                    failures.append("join.join_attempt appears more than once")
                join_attempt = line.split("=", 1)[1]

    if join_attempt != "1":
        failures.append(f"join.join_attempt={join_attempt!r} expected=1")

    join_client_lines = [line for line in join_lines if line.startswith("join_client=")]
    if not join_client_lines:
        failures.append("join.join_client is missing")
    if not any("username=StressProbe" in line for line in join_client_lines):
        failures.append("join.join_client missing username=StressProbe")
    if not any("reason=join-check-complete" in line for line in join_client_lines):
        failures.append("join.join_client missing reason=join-check-complete")
    hard_errors = paths["stress-corpus-hard-errors.txt"]
    if hard_errors.stat().st_size != 0:
        failures.append("stress-corpus-hard-errors.txt must be empty")

    log = paths["stress-corpus-gate.log"].read_text(encoding="utf-8", errors="replace")
    if SERVER_READY_RE.search(log) is None:
        failures.append("stress-corpus-gate.log missing Paper server-ready Done line")
    for snippet in [
        "StressProbe joined the game",
        "StressProbe left the game",
        "There are 13 data pack(s) enabled",
    ]:
        if snippet not in log:
            failures.append(f"stress-corpus-gate.log missing {snippet!r}")


def validate_corpus_manifests(paths: dict[str, pathlib.Path], claim: dict[str, Any], failures: list[str]) -> None:
    inspection: dict[str, str] = {}
    plugin_status_count = 0
    datapack_status_count = 0
    with paths["stress-corpus-inspection.txt"].open(encoding="utf-8", errors="replace") as handle:
        for line_no, raw in enumerate(handle, start=1):
            line = raw.strip()
            if not line:
                continue
            if line.startswith("plugin status="):
                plugin_status_count += 1
                continue
            if line.startswith("datapack status="):
                datapack_status_count += 1
                continue
            if "=" not in line:
                continue
            key, value = line.split("=", 1)
            if key in inspection and inspection[key] != value:
                failures.append(f"inspection:{line_no}: duplicate key {key}")
            inspection[key] = value
    if inspection.get("stress_corpus_inspection_pass") != "true":
        failures.append("inspection.stress_corpus_inspection_pass expected=true")
    require_int(inspection, "plugin_count", "inspection", failures, expected=claim.get("stress_plugin_count"))
    require_int(inspection, "datapack_count", "inspection", failures, expected=claim.get("datapack_count"))
    require_int(inspection, "failure_count", "inspection", failures, expected=0)
    if plugin_status_count != claim.get("stress_plugin_count"):
        failures.append("inspection plugin status row count does not match claim")
    if datapack_status_count != claim.get("datapack_count"):
        failures.append("inspection datapack status row count does not match claim")

    inspection_json = read_json(paths["stress-corpus-inspection.json"], "inspection_json", failures)
    if inspection_json:
        if inspection_json.get("failure_count") != 0:
            failures.append("inspection_json.failure_count expected=0")
        if inspection_json.get("plugin_count") != claim.get("stress_plugin_count"):
            failures.append("inspection_json.plugin_count does not match claim")
        if inspection_json.get("datapack_count") != claim.get("datapack_count"):
            failures.append("inspection_json.datapack_count does not match claim")

    corpus_manifest = read_json(paths["stress-corpus-manifest.json"], "stress_manifest", failures)
    if corpus_manifest:
        if corpus_manifest.get("plugin_count") != claim.get("stress_plugin_count"):
            failures.append("stress_manifest.plugin_count does not match claim")
        if corpus_manifest.get("datapack_count") != claim.get("datapack_count"):
            failures.append("stress_manifest.datapack_count does not match claim")
        rows = corpus_manifest.get("rows")
        if not isinstance(rows, list):
            failures.append("stress_manifest.rows is missing or not a list")
        else:
            plugin_rows = [row for row in rows if isinstance(row, dict) and row.get("side") == "plugin"]
            datapack_rows = [row for row in rows if isinstance(row, dict) and row.get("side") == "datapack"]
            if len(plugin_rows) != claim.get("stress_plugin_count"):
                failures.append("stress_manifest plugin row count does not match claim")
            if len(datapack_rows) != claim.get("datapack_count"):
                failures.append("stress_manifest datapack row count does not match claim")
            for row in rows:
                if not isinstance(row, dict):
                    failures.append("stress_manifest row is not an object")
                    continue
                if row.get("status") != "downloaded":
                    failures.append(f"stress_manifest row {row.get('filename')} status is not downloaded")
                if not is_sha256(row.get("sha256")):
                    failures.append(f"stress_manifest row {row.get('filename')} has invalid sha256")

    with paths["stress-corpus-artifacts.csv"].open(encoding="utf-8", errors="replace", newline="") as handle:
        reader = csv.DictReader(handle)
        rows = list(reader)
    plugin_rows = [row for row in rows if row.get("side") == "plugin"]
    datapack_rows = [row for row in rows if row.get("side") == "datapack"]
    if len(plugin_rows) != claim.get("stress_plugin_count"):
        failures.append("stress-corpus-artifacts.csv plugin row count does not match claim")
    if len(datapack_rows) != claim.get("datapack_count"):
        failures.append("stress-corpus-artifacts.csv datapack row count does not match claim")


def validate_artifacts_json(paths: dict[str, pathlib.Path], claim: dict[str, Any], failures: list[str]) -> None:
    artifacts = read_json(paths["artifacts.json"], "artifacts", failures)
    optimized = artifacts.get("optimized") if artifacts else None
    if not isinstance(optimized, dict) or optimized.get("sha256") != claim.get("optimized_artifact_sha256"):
        failures.append("artifacts.optimized.sha256 does not match claim")
    runtime = artifacts.get("optimized_runtime") if artifacts else None
    if not isinstance(runtime, dict):
        failures.append("artifacts.optimized_runtime is missing or not an object")
        return
    run_sh = runtime.get("run_sh")
    if not isinstance(run_sh, dict) or run_sh.get("sha256") != claim.get("optimized_runtime_run_sh_sha256"):
        failures.append("artifacts.optimized_runtime.run_sh.sha256 does not match claim")
    native = runtime.get("native_library")
    if not isinstance(native, dict) or native.get("sha256") != claim.get("optimized_runtime_native_library_sha256"):
        failures.append("artifacts.optimized_runtime.native_library.sha256 does not match claim")


def validate_claim_markdown(bundle_dir: pathlib.Path, failures: list[str]) -> None:
    claim_md = bundle_dir / "CLAIM.md"
    if not claim_md.is_file():
        failures.append("CLAIM.md is missing")
        return
    text = claim_md.read_text(encoding="utf-8", errors="replace")
    for snippet in [
        "Stress Corpus Boot/Join Evidence Bundle",
        "Not a 500-bot",
        "Not proof for unlimited",
        "MC_EULA_AGREE=true ./scripts/run_stress_corpus_gate.sh",
    ]:
        if snippet not in text:
            failures.append(f"CLAIM.md missing {snippet!r}")


def validate_bundle(bundle_dir: pathlib.Path) -> dict[str, Any]:
    bundle_dir = bundle_dir.expanduser().resolve()
    failures: list[str] = []
    if not bundle_dir.is_dir():
        failures.append(f"bundle_dir: {bundle_dir} is missing")
    data = read_json(bundle_dir / "bundle.json", "bundle", failures)
    manifest = parse_manifest(bundle_dir / "MANIFEST.txt", failures)
    claim = validate_claim(data, failures) if data else {}
    paths = validate_evidence_files(bundle_dir, data, manifest, failures) if data else {}
    if claim and paths:
        validate_summary(paths, claim, failures)
        validate_status(paths, claim, failures)
        validate_join_and_logs(paths, failures)
        validate_corpus_manifests(paths, claim, failures)
        validate_artifacts_json(paths, claim, failures)
    validate_claim_markdown(bundle_dir, failures)
    return {
        "passed": not failures,
        "failures": failures,
        "bundle_dir": bundle_dir,
        "data": data,
        "claim": claim,
        "evidence_paths": paths,
    }


def print_result(result: dict[str, Any]) -> None:
    claim = result["claim"]
    print(f"stress_corpus_bundle_validation_pass={str(result['passed']).lower()}")
    print(f"failure_count={len(result['failures'])}")
    print(f"bundle_dir={result['bundle_dir']}")
    print(f"schema={result['data'].get('schema') if result['data'] else None}")
    print(f"evidence_file_count={len(result['evidence_paths'])}")
    print(f"plugin_count={claim.get('plugin_count') if claim else None}")
    print(f"datapack_count={claim.get('datapack_count') if claim else None}")
    print(f"done_seconds={claim.get('done_seconds') if claim else None}")
    print(f"optimized_artifact_sha256={claim.get('optimized_artifact_sha256') if claim else None}")
    for failure in result["failures"]:
        print(f"stress_corpus_bundle_validation_failure={failure}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "bundle_dir",
        nargs="?",
        default=ROOT / "reports" / "stress-corpus-bundle-current",
        help="Path to a stress corpus evidence bundle.",
    )
    args = parser.parse_args()
    result = validate_bundle(pathlib.Path(args.bundle_dir))
    print_result(result)
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
