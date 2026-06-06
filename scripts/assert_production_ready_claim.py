#!/usr/bin/env python3
"""Emit the exact allowed production-ready claim only after bundle validation."""

from __future__ import annotations

import argparse
import datetime as dt
import pathlib
import sys
from typing import Any

import validate_production_readiness_bundle as bundle_validator


ROOT = pathlib.Path(__file__).resolve().parents[1]

ALLOWED_CLAIM_EN = (
    "Production-ready for the measured 500-bot, 32 view-distance, "
    "32 simulation-distance, creative block workload profile on the "
    "verified current optimized artifact."
)

ALLOWED_CLAIM_RU = (
    "production-ready для измеренного 500 bots / 32 view / 32 simulation / "
    "creative block профиля на проверенном artifact, с cold+warm soak, "
    "repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence "
    "и валидируемым self-contained evidence bundle."
)

NON_CLAIM_RU = (
    "Это не claim про полный Rust Paper runtime, unlimited plugins, "
    "real-player gameplay или multi-hour soak."
)

CLAIM_SCOPE = (
    "500-bots-32-view-32-simulation-creative-block-cold-warm-soak-repeat-"
    "plugin-restart-forced-ticket"
)

CLAIM_LIMITS = [
    "not-full-paper-runtime-rust-rewrite",
    "not-unbounded-plugin-compatibility",
    "not-unmeasured-real-player-gameplay",
    "not-multi-hour-soak",
]

MIN_TPS_AVG = 19.50
MIN_TPS_MIN = 18.00
MAX_MSPT_MAX = 100.00
MIN_BLOCK_PACKETS = 120_000


def require(condition: bool, failures: list[str], message: str) -> None:
    if not condition:
        failures.append(message)


def get_number(data: dict[str, Any], key: str, failures: list[str], label: str) -> float:
    value = data.get(key)
    if isinstance(value, (int, float)):
        return float(value)
    failures.append(f"{label}.{key}={value!r} is not numeric")
    return 0.0


def get_int(data: dict[str, Any], key: str, failures: list[str]) -> int:
    value = data.get(key)
    if isinstance(value, int):
        return value
    failures.append(f"{key}={value!r} is not an int")
    return 0


def validate_measured_surface(data: dict[str, Any], failures: list[str]) -> None:
    surface = data.get("measured_load_surface")
    if not isinstance(surface, dict):
        failures.append("measured_load_surface is missing")
        return

    for side in ["cold", "warm"]:
        values = surface.get(side)
        if not isinstance(values, dict):
            failures.append(f"measured_load_surface.{side} is missing")
            continue
        tps_avg = get_number(values, "tps1_avg", failures, side)
        tps_min = get_number(values, "tps1_min", failures, side)
        mspt_max = get_number(values, "avg_tick_ms_max", failures, side)
        place_packets = get_number(values, "block_place_packets", failures, side)
        dig_packets = get_number(values, "block_dig_packets", failures, side)

        require(tps_avg >= MIN_TPS_AVG, failures, f"{side}.tps1_avg={tps_avg} < {MIN_TPS_AVG}")
        require(tps_min >= MIN_TPS_MIN, failures, f"{side}.tps1_min={tps_min} < {MIN_TPS_MIN}")
        require(mspt_max <= MAX_MSPT_MAX, failures, f"{side}.avg_tick_ms_max={mspt_max} > {MAX_MSPT_MAX}")
        require(
            place_packets >= MIN_BLOCK_PACKETS,
            failures,
            f"{side}.block_place_packets={place_packets} < {MIN_BLOCK_PACKETS}",
        )
        require(
            dig_packets >= MIN_BLOCK_PACKETS,
            failures,
            f"{side}.block_dig_packets={dig_packets} < {MIN_BLOCK_PACKETS}",
        )


def build_report(
    bundle_dir: pathlib.Path,
    reports_dir: pathlib.Path | None = None,
) -> tuple[bool, list[str], list[str]]:
    validation = bundle_validator.validate_bundle(
        bundle_dir,
        require_current_freshness=True,
        reports_dir=reports_dir if reports_dir is not None else ROOT / "reports",
    )
    data = validation["data"]
    claim = validation["claim"]
    failures = list(validation["failures"])
    freshness = validation.get("freshness") or {}
    repeat_passes = get_int(claim, "repeat_passes", failures)
    artifact_hash_count = get_int(claim, "artifact_hash_count", failures)

    require(validation["passed"], failures, "bundle_validation_pass is not true")
    require(data.get("allowed_claim") == ALLOWED_CLAIM_EN, failures, "allowed_claim is not exact")
    require(claim.get("claim_scope") == CLAIM_SCOPE, failures, "claim_scope is not exact")
    require(claim.get("claim_limits") == CLAIM_LIMITS, failures, "claim_limits are not exact")
    require(repeat_passes >= 3, failures, "repeat_passes must be >= 3")
    require(artifact_hash_count >= 1, failures, "artifact_hash_count must be >= 1")
    require(
        isinstance(claim.get("optimized_artifact_sha256"), str)
        and len(claim["optimized_artifact_sha256"]) == 64,
        failures,
        "optimized_artifact_sha256 must be a 64-char sha256",
    )
    validate_measured_surface(data, failures)

    surface = data.get("measured_load_surface", {})
    cold = surface.get("cold", {}) if isinstance(surface, dict) else {}
    warm = surface.get("warm", {}) if isinstance(surface, dict) else {}

    passed = not failures
    lines = [
        "claim_profile=production-ready-measured-500-bots-32-32-creative-block",
        f"generated_at_utc={dt.datetime.now(dt.timezone.utc).isoformat()}",
        f"claim_assertion_pass={str(passed).lower()}",
        f"failure_count={len(failures)}",
        f"bundle_validation_pass={str(validation['passed']).lower()}",
        f"bundle_dir={validation['bundle_dir']}",
        f"bundle_schema={data.get('schema')}",
        f"bundle_evidence_file_count={len(validation['evidence'])}",
        f"current_freshness_required={str(validation.get('current_freshness_required')).lower()}",
        f"current_artifact_freshness_pass={str(freshness.get('passed')).lower() if freshness else 'false'}",
        f"current_artifacts_json={freshness.get('artifacts_json')}",
        f"current_optimized_artifact_sha256={freshness.get('current_optimized_artifact_sha256')}",
        f"current_optimized_runtime_run_sh_sha256={freshness.get('current_optimized_runtime_run_sh_sha256')}",
        f"current_optimized_runtime_native_library_sha256={freshness.get('current_optimized_runtime_native_library_sha256')}",
        f"current_optimized_runtime_chunk_encode_native_library_sha256={freshness.get('current_optimized_runtime_chunk_encode_native_library_sha256')}",
        f"bundle_generated_at_utc={freshness.get('bundle_generated_at_utc')}",
        f"latest_current_artifact_evidence={freshness.get('latest_current_artifact_evidence')}",
        f"latest_current_artifact_evidence_mtime_utc={freshness.get('latest_current_artifact_evidence_mtime_utc')}",
        f"latest_cold_current_artifact_evidence={freshness.get('latest_cold_current_artifact_evidence')}",
        f"latest_cold_current_artifact_evidence_mtime_utc={freshness.get('latest_cold_current_artifact_evidence_mtime_utc')}",
        f"latest_warm_current_artifact_evidence={freshness.get('latest_warm_current_artifact_evidence')}",
        f"latest_warm_current_artifact_evidence_mtime_utc={freshness.get('latest_warm_current_artifact_evidence_mtime_utc')}",
        f"claim_ru={ALLOWED_CLAIM_RU}",
        f"non_claim_ru={NON_CLAIM_RU}",
        f"allowed_claim_en={ALLOWED_CLAIM_EN}",
        f"claim_scope={claim.get('claim_scope')}",
        f"claim_limits={';'.join(claim.get('claim_limits', [])) if isinstance(claim.get('claim_limits'), list) else None}",
        f"optimized_artifact_sha256={claim.get('optimized_artifact_sha256')}",
        f"optimized_runtime_run_sh_sha256={claim.get('optimized_runtime_run_sh_sha256')}",
        f"optimized_runtime_native_library_sha256={claim.get('optimized_runtime_native_library_sha256')}",
        f"optimized_runtime_chunk_encode_native_library_sha256={claim.get('optimized_runtime_chunk_encode_native_library_sha256')}",
        f"repeat_passes={repeat_passes}",
        f"artifact_hash_count={artifact_hash_count}",
        f"cold_tps1_avg={cold.get('tps1_avg')}",
        f"cold_tps1_min={cold.get('tps1_min')}",
        f"cold_avg_tick_ms_max={cold.get('avg_tick_ms_max')}",
        f"cold_block_place_packets={cold.get('block_place_packets')}",
        f"cold_block_dig_packets={cold.get('block_dig_packets')}",
        f"warm_tps1_avg={warm.get('tps1_avg')}",
        f"warm_tps1_min={warm.get('tps1_min')}",
        f"warm_avg_tick_ms_max={warm.get('avg_tick_ms_max')}",
        f"warm_block_place_packets={warm.get('block_place_packets')}",
        f"warm_block_dig_packets={warm.get('block_dig_packets')}",
    ]
    for failure in failures:
        lines.append(f"claim_assertion_failure={failure}")
    return passed, failures, lines


def resolve_report(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle_dir", help="Path to a validated production readiness bundle.")
    parser.add_argument(
        "--reports-dir",
        default="reports",
        help="Reports directory containing artifacts.json and current-artifact 500 evidence.",
    )
    parser.add_argument(
        "--report",
        default="reports/production-500-claim-verdict.txt",
        help="Output claim verdict report.",
    )
    parser.add_argument(
        "--claim-only",
        action="store_true",
        help="Print only the exact allowed claim text after assertion passes.",
    )
    args = parser.parse_args()

    passed, _, lines = build_report(
        pathlib.Path(args.bundle_dir),
        reports_dir=pathlib.Path(args.reports_dir),
    )
    report = resolve_report(args.report)
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text("\n".join(lines) + "\n", encoding="utf-8")
    if args.claim_only and passed:
        print(ALLOWED_CLAIM_RU)
        print(NON_CLAIM_RU)
    else:
        print("\n".join(lines))
        print(f"claim_verdict_report={report}")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
