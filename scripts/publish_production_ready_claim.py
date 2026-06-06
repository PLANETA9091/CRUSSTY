#!/usr/bin/env python3
"""Publish the exact production-ready claim as stable txt/md/json artifacts."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import pathlib
import sys
from typing import Any

import assert_production_ready_claim as claim_assertion


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = "production-500-claim-publication/v1"


def resolve_path(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def parse_lines(lines: list[str]) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in lines:
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key] = value
    return values


def number_or_none(value: str | None) -> int | float | None:
    if value is None:
        return None
    try:
        as_int = int(value)
    except ValueError:
        try:
            return float(value)
        except ValueError:
            return None
    return as_int


def publication(values: dict[str, str]) -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "generated_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "claim_assertion_pass": values.get("claim_assertion_pass") == "true",
        "bundle_validation_pass": values.get("bundle_validation_pass") == "true",
        "current_freshness_required": values.get("current_freshness_required") == "true",
        "current_artifact_freshness_pass": values.get("current_artifact_freshness_pass") == "true",
        "claim": values.get("claim_ru"),
        "non_claim": values.get("non_claim_ru"),
        "allowed_claim_en": values.get("allowed_claim_en"),
        "claim_scope": values.get("claim_scope"),
        "claim_limits": [
            item for item in values.get("claim_limits", "").split(";") if item
        ],
        "bundle_dir": values.get("bundle_dir"),
        "bundle_schema": values.get("bundle_schema"),
        "bundle_evidence_file_count": number_or_none(values.get("bundle_evidence_file_count")),
        "bundle_generated_at_utc": values.get("bundle_generated_at_utc"),
        "current_artifacts_json": values.get("current_artifacts_json"),
        "current_optimized_artifact_sha256": values.get("current_optimized_artifact_sha256"),
        "current_optimized_runtime_run_sh_sha256": values.get(
            "current_optimized_runtime_run_sh_sha256"
        ),
        "current_optimized_runtime_native_library_sha256": values.get(
            "current_optimized_runtime_native_library_sha256"
        ),
        "current_optimized_runtime_chunk_encode_native_library_sha256": values.get(
            "current_optimized_runtime_chunk_encode_native_library_sha256"
        ),
        "latest_current_artifact_evidence": values.get("latest_current_artifact_evidence"),
        "latest_current_artifact_evidence_mtime_utc": values.get(
            "latest_current_artifact_evidence_mtime_utc"
        ),
        "latest_cold_current_artifact_evidence": values.get("latest_cold_current_artifact_evidence"),
        "latest_cold_current_artifact_evidence_mtime_utc": values.get(
            "latest_cold_current_artifact_evidence_mtime_utc"
        ),
        "latest_warm_current_artifact_evidence": values.get("latest_warm_current_artifact_evidence"),
        "latest_warm_current_artifact_evidence_mtime_utc": values.get(
            "latest_warm_current_artifact_evidence_mtime_utc"
        ),
        "optimized_artifact_sha256": values.get("optimized_artifact_sha256"),
        "optimized_runtime_run_sh_sha256": values.get("optimized_runtime_run_sh_sha256"),
        "optimized_runtime_native_library_sha256": values.get(
            "optimized_runtime_native_library_sha256"
        ),
        "optimized_runtime_chunk_encode_native_library_sha256": values.get(
            "optimized_runtime_chunk_encode_native_library_sha256"
        ),
        "repeat_passes": number_or_none(values.get("repeat_passes")),
        "artifact_hash_count": number_or_none(values.get("artifact_hash_count")),
        "measured_load_surface": {
            "cold": {
                "tps1_avg": number_or_none(values.get("cold_tps1_avg")),
                "tps1_min": number_or_none(values.get("cold_tps1_min")),
                "avg_tick_ms_max": number_or_none(values.get("cold_avg_tick_ms_max")),
                "block_place_packets": number_or_none(values.get("cold_block_place_packets")),
                "block_dig_packets": number_or_none(values.get("cold_block_dig_packets")),
            },
            "warm": {
                "tps1_avg": number_or_none(values.get("warm_tps1_avg")),
                "tps1_min": number_or_none(values.get("warm_tps1_min")),
                "avg_tick_ms_max": number_or_none(values.get("warm_avg_tick_ms_max")),
                "block_place_packets": number_or_none(values.get("warm_block_place_packets")),
                "block_dig_packets": number_or_none(values.get("warm_block_dig_packets")),
            },
        },
        "reproduce": [
            "MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh",
            "./scripts/production_ready_claim.sh",
        ],
    }


def remove_stale_outputs(out_prefix: pathlib.Path) -> None:
    for suffix in (".txt", ".md", ".json"):
        path = out_prefix.with_suffix(suffix)
        try:
            path.unlink()
        except FileNotFoundError:
            pass


def markdown(data: dict[str, Any]) -> str:
    cold = data["measured_load_surface"]["cold"]
    warm = data["measured_load_surface"]["warm"]
    return "\n".join(
        [
            "# Production 500 Claim",
            "",
            "## Claim",
            "",
            data["claim"],
            "",
            "## Non-Claim",
            "",
            data["non_claim"],
            "",
            "## Evidence",
            "",
            f"- claim_assertion_pass={str(data['claim_assertion_pass']).lower()}",
            f"- bundle_validation_pass={str(data['bundle_validation_pass']).lower()}",
            f"- current_freshness_required={str(data['current_freshness_required']).lower()}",
            f"- current_artifact_freshness_pass={str(data['current_artifact_freshness_pass']).lower()}",
            f"- bundle_evidence_file_count={data['bundle_evidence_file_count']}",
            f"- optimized_artifact_sha256={data['optimized_artifact_sha256']}",
            f"- optimized_runtime_run_sh_sha256={data['optimized_runtime_run_sh_sha256']}",
            f"- optimized_runtime_native_library_sha256={data['optimized_runtime_native_library_sha256']}",
            f"- optimized_runtime_chunk_encode_native_library_sha256={data['optimized_runtime_chunk_encode_native_library_sha256']}",
            f"- current_optimized_artifact_sha256={data['current_optimized_artifact_sha256']}",
            f"- current_optimized_runtime_run_sh_sha256={data['current_optimized_runtime_run_sh_sha256']}",
            f"- current_optimized_runtime_native_library_sha256={data['current_optimized_runtime_native_library_sha256']}",
            f"- current_optimized_runtime_chunk_encode_native_library_sha256={data['current_optimized_runtime_chunk_encode_native_library_sha256']}",
            f"- latest_current_artifact_evidence={data['latest_current_artifact_evidence']}",
            f"- latest_cold_current_artifact_evidence={data['latest_cold_current_artifact_evidence']}",
            f"- latest_warm_current_artifact_evidence={data['latest_warm_current_artifact_evidence']}",
            f"- repeat_passes={data['repeat_passes']}",
            f"- artifact_hash_count={data['artifact_hash_count']}",
            f"- bundle_dir={data['bundle_dir']}",
            "",
            "## Measured Load Surface",
            "",
            "| surface | TPS avg/min | MSPT max | block place/dig |",
            "| --- | --- | ---: | --- |",
            (
                f"| cold/fresh | `{cold['tps1_avg']} / {cold['tps1_min']}` | "
                f"`{cold['avg_tick_ms_max']}` | "
                f"`{cold['block_place_packets']} / {cold['block_dig_packets']}` |"
            ),
            (
                f"| warm-source | `{warm['tps1_avg']} / {warm['tps1_min']}` | "
                f"`{warm['avg_tick_ms_max']}` | "
                f"`{warm['block_place_packets']} / {warm['block_dig_packets']}` |"
            ),
            "",
            "## Reproduce",
            "",
            "```bash",
            *data["reproduce"],
            "```",
            "",
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "bundle_dir",
        nargs="?",
        default="reports/production-500-readiness-bundle-current",
        help="Validated production readiness bundle.",
    )
    parser.add_argument(
        "--reports-dir",
        default="reports",
        help="Reports directory containing artifacts.json and current-artifact 500 evidence.",
    )
    parser.add_argument(
        "--out-prefix",
        default="reports/production-500-claim-current",
        help="Output path prefix for .txt, .md, and .json files.",
    )
    parser.add_argument(
        "--verdict-report",
        default="reports/production-500-claim-current-verdict.txt",
        help="Output claim assertion verdict report.",
    )
    args = parser.parse_args()

    passed, _, lines = claim_assertion.build_report(
        pathlib.Path(args.bundle_dir),
        reports_dir=resolve_path(args.reports_dir),
    )
    verdict_report = resolve_path(args.verdict_report)
    verdict_report.parent.mkdir(parents=True, exist_ok=True)
    verdict_report.write_text("\n".join(lines) + "\n", encoding="utf-8")
    if not passed:
        remove_stale_outputs(resolve_path(args.out_prefix))
        print("\n".join(lines))
        print(f"claim_publication_failure=claim assertion failed")
        return 1

    values = parse_lines(lines)
    data = publication(values)
    out_prefix = resolve_path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    txt_path = out_prefix.with_suffix(".txt")
    md_path = out_prefix.with_suffix(".md")
    json_path = out_prefix.with_suffix(".json")
    txt_path.write_text(f"{data['claim']}\n{data['non_claim']}\n", encoding="utf-8")
    md_path.write_text(markdown(data), encoding="utf-8")
    json_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("claim_publication_pass=true")
    print(f"claim_publication_txt={txt_path}")
    print(f"claim_publication_md={md_path}")
    print(f"claim_publication_json={json_path}")
    print(f"claim_publication_verdict={verdict_report}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
