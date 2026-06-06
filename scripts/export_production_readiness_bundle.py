#!/usr/bin/env python3
"""Export a self-contained evidence bundle for the 500-bot readiness claim."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import pathlib
import shutil
import sys
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]


EVIDENCE_KEYS = [
    "go_nogo_report",
    "soak_report",
    "repeat_report",
    "plugin_matrix_summary",
    "restart_recovery_summary",
    "forced_ticket_summary",
    "artifact_hash_manifest",
]

BASE_REQUIRED_EVIDENCE = [
    "production-500-readiness-gate.txt",
    "production-500-go-nogo-current.txt",
    "production-500-soak-gate.txt",
    "production-500-repeat-quorum.txt",
    "plugin-matrix-summary.txt",
    "restart-recovery-summary.txt",
    "forced-ticket-persistence-summary.txt",
    "artifact-hashes.txt",
]

REFERENCED_FILE_SUFFIXES = (".txt", ".csv", ".json", ".sha256", ".log")


def resolve_path(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


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


def reject_duplicate_json_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate json key {key!r}")
        result[key] = value
    return result


def load_json_object(
    path: pathlib.Path,
    failures: list[str] | None = None,
    label: str | None = None,
) -> dict[str, object]:
    try:
        data = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_json_pairs,
        )
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        if failures is not None:
            failures.append(f"{label or path}: invalid json: {exc}")
        return {}
    return data if isinstance(data, dict) else {}


def nested_dict(data: dict[str, object], key: str) -> dict[str, object]:
    value = data.get(key)
    return value if isinstance(value, dict) else {}


def nested_str(data: dict[str, object], keys: tuple[str, ...]) -> str | None:
    current: object = data
    for key in keys:
        if not isinstance(current, dict):
            return None
        current = current.get(key)
    return current if isinstance(current, str) else None


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require(values: dict[str, str], key: str, expected: str, failures: list[str]) -> None:
    observed = values.get(key)
    if observed != expected:
        failures.append(f"{key}={observed} expected={expected}")


def copy_with_manifest(
    src: pathlib.Path,
    dst: pathlib.Path,
    logical_name: str,
    manifest: list[str],
) -> dict[str, object]:
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    digest = sha256(dst)
    size = dst.stat().st_size
    manifest.append(
        f"{logical_name}\tsha256={digest}\tbytes={size}\tsource={src}"
    )
    return {
        "logical_name": logical_name,
        "relative_path": f"evidence/{dst.name}",
        "sha256": digest,
        "bytes": size,
        "source": str(src),
    }


def add_evidence(
    evidence_records: list[dict[str, object]],
    manifest_lines: list[str],
    copied_sources: set[pathlib.Path],
    evidence_dir: pathlib.Path,
    src: pathlib.Path,
    logical_name: str | None = None,
) -> None:
    try:
        resolved = src.resolve()
    except OSError:
        resolved = src
    if resolved in copied_sources:
        return
    if not src.is_file():
        return
    name = logical_name or src.name
    evidence_records.append(
        copy_with_manifest(src, evidence_dir / name, name, manifest_lines)
    )
    copied_sources.add(resolved)


def replace_summary_suffix(path: pathlib.Path, suffix: str) -> pathlib.Path | None:
    name = path.name
    if not name.endswith("-summary.txt"):
        return None
    return path.with_name(f"{name[:-len('-summary.txt')]}{suffix}")


def readiness_artifacts_json(values: dict[str, str]) -> pathlib.Path:
    raw = values.get("artifacts_json")
    if raw:
        return resolve_path(raw)
    return ROOT / "reports" / "artifacts.json"


def append_referenced_values(referenced: list[pathlib.Path], values: list[str]) -> None:
    for raw in values:
        if raw.endswith(REFERENCED_FILE_SUFFIXES):
            referenced.append(resolve_path(raw))


def referenced_bundle_files(
    readiness_report: pathlib.Path,
    values: dict[str, str],
    artifacts_json: pathlib.Path,
    failures: list[str],
) -> list[pathlib.Path]:
    referenced: list[pathlib.Path] = []
    for key in EVIDENCE_KEYS:
        raw = values.get(key)
        if raw:
            path = resolve_path(raw)
            referenced.append(path)
            if path.is_file():
                append_referenced_values(
                    referenced,
                    list(parse_kv(path, failures, str(path)).values()),
                )

    append_referenced_values(referenced, list(values.values()))

    soak_report_raw = values.get("soak_report")
    if soak_report_raw:
        soak_report = resolve_path(soak_report_raw)
        if soak_report.is_file():
            soak_values = parse_kv(soak_report, failures, str(soak_report))
            append_referenced_values(referenced, list(soak_values.values()))
            for side in ("cold", "warm"):
                raw_summary = soak_values.get(f"{side}_summary_path")
                if not raw_summary:
                    continue
                summary = resolve_path(raw_summary)
                referenced.append(summary)
                for derived in (
                    replace_summary_suffix(summary, "-preflight.txt"),
                    replace_summary_suffix(summary, "-resources.csv"),
                ):
                    if derived is not None:
                        referenced.append(derived)
                if summary.is_file():
                    summary_values = parse_kv(summary, failures, str(summary))
                    append_referenced_values(referenced, list(summary_values.values()))

    native_hash_file = None
    chunk_encode_native_hash_file = None
    if artifacts_json.is_file():
        referenced.append(artifacts_json)
        artifacts = load_json_object(artifacts_json, failures, str(artifacts_json))
        native_hash_file = nested_str(
            artifacts,
            ("optimized_runtime", "native_library_sha256_file", "path"),
        )
        if native_hash_file:
            referenced.append(resolve_path(native_hash_file))
        chunk_encode_native_hash_file = nested_str(
            artifacts,
            ("optimized_runtime", "chunk_encode_native_library_sha256_file", "path"),
        )
        if chunk_encode_native_hash_file:
            referenced.append(resolve_path(chunk_encode_native_hash_file))

    if not native_hash_file:
        for candidate in (
            ROOT / "artifacts" / "optimized-runtime" / "native" / "libpaper_native_jni.so.sha256",
            ROOT / "reports" / "paper-native-jni.sha256",
        ):
            if candidate.is_file():
                referenced.append(candidate)

    if not chunk_encode_native_hash_file:
        for candidate in (
            ROOT / "artifacts" / "optimized-runtime" / "native" / "libpaper_native_chunk_encode_jni.so.sha256",
            ROOT / "reports" / "paper-native-chunk-encode-jni.sha256",
        ):
            if candidate.is_file():
                referenced.append(candidate)

    return referenced


def dedupe_paths(paths: list[pathlib.Path]) -> list[pathlib.Path]:
    deduped: list[pathlib.Path] = []
    seen: set[pathlib.Path] = set()
    for path in paths:
        try:
            resolved = path.resolve()
        except OSError:
            resolved = path
        if resolved in seen:
            continue
        seen.add(resolved)
        deduped.append(path)
    return deduped


def parse_bool(values: dict[str, str], key: str) -> bool | None:
    raw = values.get(key)
    if raw == "true":
        return True
    if raw == "false":
        return False
    return None


def parse_int(values: dict[str, str], key: str) -> int | None:
    raw = values.get(key)
    if raw is None:
        return None
    try:
        return int(raw)
    except ValueError:
        return None


def parse_float(values: dict[str, str], key: str) -> float | None:
    raw = values.get(key)
    if raw is None:
        return None
    try:
        return float(raw)
    except ValueError:
        return None


def bundle_index(
    values: dict[str, str],
    evidence_records: list[dict[str, object]],
    artifacts: dict[str, object],
) -> dict[str, object]:
    optimized_runtime_run_sh_sha256 = (
        values.get("current_optimized_runtime_run_sh_sha256")
        or values.get("optimized_runtime_run_sh_sha256")
    )
    optimized_runtime_chunk_encode_native_library_sha256 = nested_str(
        artifacts,
        ("optimized_runtime", "chunk_encode_native_library", "sha256"),
    )
    return {
        "schema": "production-500-readiness-bundle/v1",
        "generated_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "allowed_claim": (
            "Production-ready for the measured 500-bot, 32 view-distance, "
            "32 simulation-distance, creative block workload profile on the "
            "verified current optimized artifact."
        ),
        "claim": {
            "claim_text": values.get("claim_text"),
            "claim_scope": values.get("claim_scope"),
            "claim_limits": [
                item for item in values.get("claim_limits", "").split(";") if item
            ],
            "production_ready_500_claim": parse_bool(values, "production_ready_500_claim"),
            "readiness_gate_pass": parse_bool(values, "readiness_gate_pass"),
            "failure_count": parse_int(values, "failure_count"),
            "soak_gate_pass": parse_bool(values, "soak_gate_pass"),
            "repeat_quorum_pass": parse_bool(values, "repeat_quorum_pass"),
            "plugin_matrix_pass": parse_bool(values, "plugin_matrix_pass"),
            "restart_recovery_pass": parse_bool(values, "restart_recovery_pass"),
            "forced_ticket_persistence_pass": parse_bool(
                values, "forced_ticket_persistence_pass"
            ),
            "artifact_hashes_pass": parse_bool(values, "artifact_hashes_pass"),
            "current_artifact_consistency_pass": parse_bool(
                values, "current_artifact_consistency_pass"
            ),
            "artifact_hash_count": parse_int(values, "artifact_hash_count"),
            "optimized_artifact_sha256": values.get("optimized_artifact_sha256"),
            "optimized_runtime_run_sh_sha256": optimized_runtime_run_sh_sha256,
            "optimized_runtime_native_library_sha256": nested_str(
                artifacts,
                ("optimized_runtime", "native_library", "sha256"),
            ),
            "optimized_runtime_chunk_encode_native_library_sha256": (
                optimized_runtime_chunk_encode_native_library_sha256
            ),
            "repeat_passes": parse_int(values, "repeat_passes"),
        },
        "measured_load_surface": {
            "cold": {
                "tps1_avg": parse_float(values, "cold_load_window_tps1_avg"),
                "tps1_min": parse_float(values, "cold_load_window_tps1_min"),
                "avg_tick_ms_max": parse_float(
                    values, "cold_load_window_avg_tick_ms_max"
                ),
                "block_place_packets": parse_int(
                    values, "cold_bot_block_place_packets_max"
                ),
                "block_dig_packets": parse_int(values, "cold_bot_block_dig_packets_max"),
            },
            "warm": {
                "tps1_avg": parse_float(values, "warm_load_window_tps1_avg"),
                "tps1_min": parse_float(values, "warm_load_window_tps1_min"),
                "avg_tick_ms_max": parse_float(
                    values, "warm_load_window_avg_tick_ms_max"
                ),
                "block_place_packets": parse_int(
                    values, "warm_bot_block_place_packets_max"
                ),
                "block_dig_packets": parse_int(values, "warm_bot_block_dig_packets_max"),
            },
        },
        "explicit_non_claims": [
            "not a full Paper runtime rewrite to Rust",
            "not unlimited plugin compatibility",
            "not proof for unmeasured real-player gameplay",
            "not a multi-hour soak claim",
        ],
        "required_evidence": [
            *BASE_REQUIRED_EVIDENCE,
            *[
                str(record["logical_name"])
                for record in evidence_records
                if record.get("logical_name") not in BASE_REQUIRED_EVIDENCE
            ],
        ],
        "evidence_files": evidence_records,
        "reproduce": ["MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh"],
    }


def claim_markdown(
    values: dict[str, str],
    manifest_lines: list[str],
    artifacts: dict[str, object],
) -> str:
    return "\n".join(
        [
            "# Production 500 Readiness Claim",
            "",
            "Allowed claim:",
            "",
            "> Production-ready for the measured 500-bot, 32 view-distance,",
            "> 32 simulation-distance, creative block workload profile on the",
            "> verified current optimized artifact.",
            "",
            "This claim is allowed only with the exact evidence bundle generated here.",
            "",
            "Gate summary:",
            "",
            f"- production_ready_500_claim={values.get('production_ready_500_claim')}",
            f"- readiness_gate_pass={values.get('readiness_gate_pass')}",
            f"- failure_count={values.get('failure_count')}",
            f"- soak_gate_pass={values.get('soak_gate_pass')}",
            f"- repeat_quorum_pass={values.get('repeat_quorum_pass')}",
            f"- plugin_matrix_pass={values.get('plugin_matrix_pass')}",
            f"- restart_recovery_pass={values.get('restart_recovery_pass')}",
            f"- forced_ticket_persistence_pass={values.get('forced_ticket_persistence_pass')}",
            f"- artifact_hashes_pass={values.get('artifact_hashes_pass')}",
            f"- optimized_artifact_sha256={values.get('optimized_artifact_sha256')}",
            f"- optimized_runtime_run_sh_sha256={values.get('current_optimized_runtime_run_sh_sha256') or values.get('optimized_runtime_run_sh_sha256')}",
            f"- optimized_runtime_native_library_sha256={nested_str(artifacts, ('optimized_runtime', 'native_library', 'sha256'))}",
            f"- optimized_runtime_chunk_encode_native_library_sha256={nested_str(artifacts, ('optimized_runtime', 'chunk_encode_native_library', 'sha256'))}",
            "",
            "Measured load surface:",
            "",
            f"- cold TPS avg/min/MSPT max: {values.get('cold_load_window_tps1_avg')} / {values.get('cold_load_window_tps1_min')} / {values.get('cold_load_window_avg_tick_ms_max')}",
            f"- warm TPS avg/min/MSPT max: {values.get('warm_load_window_tps1_avg')} / {values.get('warm_load_window_tps1_min')} / {values.get('warm_load_window_avg_tick_ms_max')}",
            f"- cold block place/dig packets: {values.get('cold_bot_block_place_packets_max')} / {values.get('cold_bot_block_dig_packets_max')}",
            f"- warm block place/dig packets: {values.get('warm_bot_block_place_packets_max')} / {values.get('warm_bot_block_dig_packets_max')}",
            f"- repeat_passes={values.get('repeat_passes')}",
            "",
            "Explicit non-claims:",
            "",
            "- Not a full Paper runtime rewrite to Rust.",
            "- Not unlimited plugin compatibility.",
            "- Not proof for unmeasured real-player gameplay.",
            "- Not a multi-hour soak claim.",
            "",
            "Reproduce:",
            "",
            "```bash",
            "MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh",
            "```",
            "",
            "Evidence files:",
            "",
            "```text",
            *manifest_lines,
            "```",
            "",
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--readiness-report",
        default="reports/production-500-readiness-gate.txt",
        help="Input readiness gate report.",
    )
    parser.add_argument(
        "--out-dir",
        default=None,
        help="Output bundle directory. Default: reports/production-500-readiness-bundle-<timestamp>",
    )
    args = parser.parse_args()

    readiness_report = resolve_path(args.readiness_report)
    if not readiness_report.is_file():
        raise SystemExit(f"{readiness_report}: missing readiness report")

    failures: list[str] = []
    values = parse_kv(readiness_report, failures, str(readiness_report))
    require(values, "production_ready_500_claim", "true", failures)
    require(values, "readiness_gate_pass", "true", failures)
    require(values, "failure_count", "0", failures)

    for key in EVIDENCE_KEYS:
        raw = values.get(key)
        if raw is None:
            failures.append(f"{key} is missing from readiness report")
            continue
        path = resolve_path(raw)
        if not path.is_file():
            failures.append(f"{key}: {path} is missing")
            continue
        expected = values.get(f"{key}_sha256")
        observed = sha256(path)
        if expected and observed != expected:
            failures.append(f"{key}: sha256={observed} expected={expected}")

    if failures:
        for failure in failures:
            print(f"bundle_failure={failure}", file=sys.stderr)
        return 1

    if args.out_dir:
        out_dir = resolve_path(args.out_dir)
    else:
        stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%d-%H%M%S")
        out_dir = ROOT / "reports" / f"production-500-readiness-bundle-{stamp}"
    if out_dir.exists():
        raise SystemExit(f"{out_dir}: output bundle already exists")

    evidence_dir = out_dir / "evidence"
    manifest_lines: list[str] = []
    evidence_records: list[dict[str, object]] = []
    copied_sources: set[pathlib.Path] = set()
    add_evidence(
        evidence_records,
        manifest_lines,
        copied_sources,
        evidence_dir,
        readiness_report,
        "production-500-readiness-gate.txt",
    )
    for key in EVIDENCE_KEYS:
        src = resolve_path(values[key])
        add_evidence(evidence_records, manifest_lines, copied_sources, evidence_dir, src)

    artifacts_json = readiness_artifacts_json(values)
    if not artifacts_json.is_file():
        print(f"bundle_failure=artifacts_json: {artifacts_json} is missing", file=sys.stderr)
        return 1
    artifacts = load_json_object(artifacts_json, failures, str(artifacts_json))
    referenced_files = dedupe_paths(
        referenced_bundle_files(readiness_report, values, artifacts_json, failures)
    )
    if failures:
        for failure in failures:
            print(f"bundle_failure={failure}", file=sys.stderr)
        return 1
    missing_referenced = [src for src in referenced_files if not src.is_file()]
    if missing_referenced:
        for src in missing_referenced:
            print(f"bundle_failure=referenced_evidence_missing: {src}", file=sys.stderr)
        return 1
    for src in referenced_files:
        add_evidence(
            evidence_records,
            manifest_lines,
            copied_sources,
            evidence_dir,
            src,
        )

    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "bundle.json").write_text(
        json.dumps(bundle_index(values, evidence_records, artifacts), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (out_dir / "MANIFEST.txt").write_text("\n".join(manifest_lines) + "\n", encoding="utf-8")
    (out_dir / "CLAIM.md").write_text(
        claim_markdown(values, manifest_lines, artifacts),
        encoding="utf-8",
    )

    print(f"claim_bundle={out_dir}")
    print(f"claim_bundle_index={out_dir / 'bundle.json'}")
    print(f"claim_bundle_manifest={out_dir / 'MANIFEST.txt'}")
    print(f"claim_bundle_claim={out_dir / 'CLAIM.md'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
