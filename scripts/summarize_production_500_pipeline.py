#!/usr/bin/env python3
"""Print the current P500 pipeline state without running load gates."""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import io
import json
import pathlib
import subprocess
import sys
from typing import Any

import assert_production_ready_claim as claim_assertion
import validate_production_readiness_bundle as bundle_validator


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_REPORTS_DIR = ROOT / "reports"
DEFAULT_BUNDLE_DIR = DEFAULT_REPORTS_DIR / "production-500-readiness-bundle-current"


GO_NOGO_KEYS = [
    "production_500_go_nogo_pass",
    "production_500_go_nogo_exit_code",
    "production_500_go_nogo_reason",
    "production_500_go_nogo_foreign_pattern",
    "production_500_go_nogo_canary_duration_seconds",
    "production_500_go_nogo_canary_sample_interval_seconds",
    "production_500_go_nogo_canary_max_steal_percent",
    "production_500_go_nogo_canary_max_iowait_percent",
]


EVIDENCE_REPORTS = [
    (
        "go_nogo",
        "production-500-go-nogo-current.txt",
        GO_NOGO_KEYS,
    ),
    (
        "readiness_gate",
        "production-500-readiness-gate.txt",
        [
            "production_ready_500_claim",
            "readiness_gate_pass",
            "failure_count",
            "soak_gate_pass",
            "repeat_quorum_pass",
            "plugin_matrix_pass",
            "restart_recovery_pass",
            "forced_ticket_persistence_pass",
            "artifact_hashes_pass",
            "current_artifact_consistency_pass",
            "optimized_artifact_sha256",
            "current_optimized_artifact_sha256",
            "current_optimized_runtime_run_sh_sha256",
            "current_optimized_runtime_native_library_sha256",
            "current_optimized_runtime_chunk_encode_native_library_sha256",
        ],
    ),
    (
        "cold_warm_soak",
        "production-500-soak-gate.txt",
        [
            "production_ready_soak_claim_eligible",
            "soak_gate_pass",
            "base_cold_gate_pass",
            "base_warm_gate_pass",
            "artifact_hashes_pass",
            "artifact_hash_count",
            "optimized_artifact_sha256",
            "optimized_runtime_chunk_encode_native_library_sha256",
            "cold_summary_path",
            "warm_summary_path",
            "cold_gate_pass",
            "warm_gate_pass",
            "cold_failure_count",
            "warm_failure_count",
        ],
    ),
    (
        "repeat_quorum",
        "production-500-repeat-quorum.txt",
        [
            "repeat_quorum_pass",
            "repeat_passes",
            "repeat_failures",
            "repeat_run_count",
            "run_1_pass",
            "run_2_pass",
            "run_3_pass",
        ],
    ),
    (
        "plugin_matrix",
        "plugin-matrix-summary.txt",
        [
            "plugin_matrix_pass",
            "plugin_matrix_log",
            "status_json",
        ],
    ),
    (
        "restart_recovery",
        "restart-recovery-summary.txt",
        [
            "restart_recovery_pass",
            "restart_recovery_log",
            "status_json",
        ],
    ),
    (
        "forced_ticket",
        "forced-ticket-persistence-summary.txt",
        [
            "forced_ticket_persistence",
            "first_log",
            "restart_log",
            "runtime_log_clean",
        ],
    ),
]


def bool_text(value: bool) -> str:
    return str(value).lower()


def resolve_path(raw: str | pathlib.Path) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def format_epoch(seconds: float) -> str:
    return dt.datetime.fromtimestamp(seconds, dt.timezone.utc).isoformat()


def run_command(argv: list[str]) -> tuple[int, list[str]]:
    try:
        completed = subprocess.run(
            argv,
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
        )
    except OSError as exc:
        return 127, [f"command_error={exc}"]
    return completed.returncode, completed.stdout.splitlines()


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


def file_status(prefix: str, path: pathlib.Path) -> list[str]:
    lines = [
        f"{prefix}_path={path}",
        f"{prefix}_present={bool_text(path.is_file())}",
    ]
    if not path.exists():
        return lines
    try:
        stat_result = path.stat()
    except OSError as exc:
        lines.append(f"{prefix}_stat_error={exc}")
        return lines
    lines.extend(
        [
            f"{prefix}_size_bytes={stat_result.st_size}",
            f"{prefix}_mtime_utc={format_epoch(stat_result.st_mtime)}",
        ]
    )
    return lines


def capture_print(func: Any, *args: Any, **kwargs: Any) -> list[str]:
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        func(*args, **kwargs)
    return output.getvalue().splitlines()


def append_section(output: list[str], title: str, body: list[str]) -> None:
    output.append(f"[{title}]")
    output.extend(body or ["empty=true"])
    output.append("")


def artifact_freshness_section() -> list[str]:
    command = [str(ROOT / "scripts" / "check_artifact_source_freshness.sh")]
    exit_code, lines = run_command(command)
    return [
        f"command={' '.join(command)}",
        f"exit_code={exit_code}",
        *lines,
    ]


def validate_bundle_result(bundle_dir: pathlib.Path, reports_dir: pathlib.Path) -> dict[str, Any]:
    return bundle_validator.validate_bundle(
        bundle_dir,
        require_current_freshness=True,
        reports_dir=reports_dir,
    )


def bundle_validation_section(result: dict[str, Any]) -> list[str]:
    return capture_print(bundle_validator.print_validation_result, result)


def claim_assertion_result(bundle_dir: pathlib.Path, reports_dir: pathlib.Path) -> tuple[bool, list[str], list[str]]:
    return claim_assertion.build_report(bundle_dir, reports_dir=reports_dir)


def claim_assertion_section(result: tuple[bool, list[str], list[str]]) -> list[str]:
    passed, _, lines = result
    return [
        *lines,
        f"claim_assertion_section_pass={bool_text(passed)}",
    ]


def current_artifact_hash_section(reports_dir: pathlib.Path, *, refresh_artifacts: bool) -> list[str]:
    artifacts_json = reports_dir / "artifacts.json"
    artifact_hashes = reports_dir / "artifact-hashes.txt"
    lines = [f"refresh_artifacts={bool_text(refresh_artifacts)}"]
    if refresh_artifacts:
        update_command = ["python3", str(ROOT / "scripts" / "update_artifact_reports.py")]
        update_exit, update_lines = run_command(update_command)
        lines.extend(
            [
                f"update_command={' '.join(update_command)}",
                f"update_exit_code={update_exit}",
                *[f"update_output={line}" for line in update_lines],
            ]
        )

    hash_command = ["sha256sum", "-c", str(artifact_hashes)]
    hash_exit, _ = run_command(hash_command)

    lines.extend(
        [
            *file_status("artifacts_json", artifacts_json),
            *file_status("artifact_hash_manifest", artifact_hashes),
            f"sha256sum_command={' '.join(hash_command)}",
            f"sha256sum_exit_code={hash_exit}",
            f"sha256sum_pass={bool_text(hash_exit == 0)}",
        ]
    )

    data: dict[str, Any] = {}
    if artifacts_json.is_file():
        try:
            loaded = json.loads(artifacts_json.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as exc:
            lines.append(f"artifacts_json_read_error={exc}")
        else:
            if isinstance(loaded, dict):
                data = loaded
            else:
                lines.append("artifacts_json_read_error=top-level json is not an object")

    optimized = data.get("optimized") if isinstance(data.get("optimized"), dict) else {}
    runtime = data.get("optimized_runtime") if isinstance(data.get("optimized_runtime"), dict) else {}
    run_sh = runtime.get("run_sh") if isinstance(runtime.get("run_sh"), dict) else {}
    native = runtime.get("native_library") if isinstance(runtime.get("native_library"), dict) else {}
    chunk_encode_native = (
        runtime.get("chunk_encode_native_library")
        if isinstance(runtime.get("chunk_encode_native_library"), dict)
        else {}
    )
    runtime_jar_sha = (
        runtime.get("runtime_jar_sha256_file")
        if isinstance(runtime.get("runtime_jar_sha256_file"), dict)
        else {}
    )

    lines.extend(
        [
            f"current_optimized_artifact_path={optimized.get('path')}",
            f"current_optimized_artifact_sha256={optimized.get('sha256')}",
            f"current_optimized_runtime_run_sh_path={run_sh.get('path')}",
            f"current_optimized_runtime_run_sh_sha256={run_sh.get('sha256')}",
            f"current_optimized_runtime_jar_sha256={runtime_jar_sha.get('runtime_jar_sha256')}",
            f"current_optimized_runtime_native_library_path={native.get('path')}",
            f"current_optimized_runtime_native_library_sha256={native.get('sha256')}",
            f"current_optimized_runtime_chunk_encode_native_library_path={chunk_encode_native.get('path')}",
            f"current_optimized_runtime_chunk_encode_native_library_sha256={chunk_encode_native.get('sha256')}",
        ]
    )
    if artifact_hashes.is_file():
        rows = [
            line
            for line in artifact_hashes.read_text(encoding="utf-8", errors="replace").splitlines()
            if line.strip()
        ]
        lines.append(f"artifact_hash_count={len(rows)}")
    return lines


def evidence_presence_section(reports_dir: pathlib.Path) -> list[str]:
    lines: list[str] = []
    for name, filename, keys in EVIDENCE_REPORTS:
        path = reports_dir / filename
        lines.extend(file_status(name, path))
        values = parse_kv(path)
        for key in keys:
            if key in values:
                lines.append(f"{name}_{key}={values[key]}")
        lines.append("")
    return lines


def go_nogo_section(reports_dir: pathlib.Path) -> list[str]:
    path = reports_dir / "production-500-go-nogo-current.txt"
    lines = file_status("go_nogo", path)
    values = parse_kv(path)
    for key in GO_NOGO_KEYS:
        if key in values:
            lines.append(f"{key}={values[key]}")
    return lines


def control_plane_verdict_section(
    reports_dir: pathlib.Path,
    bundle_result: dict[str, Any],
    claim_result: tuple[bool, list[str], list[str]],
) -> list[str]:
    go_nogo_path = reports_dir / "production-500-go-nogo-current.txt"
    readiness_path = reports_dir / "production-500-readiness-gate.txt"
    go_nogo = parse_kv(go_nogo_path)
    readiness = parse_kv(readiness_path)
    go_nogo_pass = go_nogo.get("production_500_go_nogo_pass") == "true"
    go_nogo_reason = go_nogo.get("production_500_go_nogo_reason") or "missing"
    claim_pass = claim_result[0]
    if not go_nogo:
        next_action = "run_go_nogo_preflight"
    elif not go_nogo_pass:
        if go_nogo_reason == "strict_foreign_process_present":
            next_action = "stop_foreign_process"
        elif go_nogo_reason == "host_synthetic_canary_failed":
            next_action = "wait_for_clean_host"
        elif go_nogo_reason.startswith("diagnostic_"):
            next_action = "run_diagnostics_with_degraded_host_evidence"
        else:
            next_action = "rerun_go_nogo_preflight"
    elif not bundle_result["passed"]:
        next_action = "regenerate_bundle"
    elif not claim_pass:
        next_action = "repair_bundle_or_claim"
    elif readiness.get("readiness_gate_pass") == "true":
        next_action = "publish_production_ready_claim"
    else:
        next_action = "rerun_p500_gate"
    return [
        f"go_nogo_present={bool_text(go_nogo_path.is_file())}",
        f"go_nogo_pass={bool_text(go_nogo_pass)}",
        f"go_nogo_reason={go_nogo_reason}",
        f"bundle_validation_pass={bool_text(bundle_result['passed'])}",
        f"claim_assertion_pass={bool_text(claim_pass)}",
        f"readiness_gate_pass={readiness.get('readiness_gate_pass')}",
        f"production_readiness_next_action={next_action}",
    ]


def build_summary(args: argparse.Namespace) -> list[str]:
    reports_dir = resolve_path(args.reports_dir)
    bundle_dir = resolve_path(args.bundle_dir)
    bundle_result = validate_bundle_result(bundle_dir, reports_dir)
    claim_result = claim_assertion_result(bundle_dir, reports_dir)
    output = [
        "p500_pipeline_summary_profile=production-500-control-plane",
        f"generated_at_utc={dt.datetime.now(dt.timezone.utc).isoformat()}",
        f"root={ROOT}",
        f"reports_dir={reports_dir}",
        f"bundle_dir={bundle_dir}",
        f"report={resolve_path(args.report) if args.report else 'disabled'}",
        "",
    ]
    append_section(output, "artifact freshness", artifact_freshness_section())
    append_section(
        output,
        "current artifact hashes",
        current_artifact_hash_section(reports_dir, refresh_artifacts=args.refresh_artifacts),
    )
    append_section(output, "go/nogo", go_nogo_section(reports_dir))
    append_section(output, "evidence presence", evidence_presence_section(reports_dir))
    append_section(output, "bundle validation", bundle_validation_section(bundle_result))
    append_section(output, "claim assertion", claim_assertion_section(claim_result))
    append_section(output, "control plane verdict", control_plane_verdict_section(reports_dir, bundle_result, claim_result))
    output.append("p500_pipeline_summary_complete=true")
    return output


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--reports-dir",
        default=str(DEFAULT_REPORTS_DIR),
        help="Reports directory containing artifacts and P500 evidence reports.",
    )
    parser.add_argument(
        "--bundle-dir",
        default=str(DEFAULT_BUNDLE_DIR),
        help="Production-500 readiness bundle to validate and assert.",
    )
    parser.add_argument(
        "--report",
        default="",
        help="Combined summary report path. Leave empty to print only to stdout.",
    )
    parser.add_argument(
        "--refresh-artifacts",
        action="store_true",
        help="Run update_artifact_reports.py before reading artifact hashes.",
    )
    args = parser.parse_args()

    lines = build_summary(args)
    text = "\n".join(lines) + "\n"
    report = resolve_path(args.report) if args.report else None
    if report is not None:
        report.parent.mkdir(parents=True, exist_ok=True)
        report.write_text(text, encoding="utf-8")
    print(text, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main())
