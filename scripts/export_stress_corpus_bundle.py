#!/usr/bin/env python3
"""Export a self-contained evidence bundle for the stress-corpus boot/join gate."""

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


def resolve_path(raw: str) -> pathlib.Path:
    path = pathlib.Path(raw).expanduser()
    if not path.is_absolute():
        path = ROOT / path
    return path


def report_evidence_path(values: dict[str, str], summary: pathlib.Path, key: str, default_name: str) -> pathlib.Path:
    raw = values.get(key)
    if raw:
        return resolve_path(raw)
    return summary.parent / default_name


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_kv(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.strip()
            if not line or "=" not in line:
                continue
            key, value = line.split("=", 1)
            values[key] = value
    return values


def read_json(path: pathlib.Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def int_value(values: dict[str, str], key: str, failures: list[str]) -> int:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{key} is missing")
        return 0
    try:
        return int(raw)
    except ValueError:
        failures.append(f"{key}={raw!r} is not an int")
        return 0


def float_value(values: dict[str, str], key: str, failures: list[str]) -> float:
    raw = values.get(key)
    if raw is None:
        failures.append(f"{key} is missing")
        return 0.0
    try:
        return float(raw)
    except ValueError:
        failures.append(f"{key}={raw!r} is not numeric")
        return 0.0


def validate_source_evidence(summary: pathlib.Path, values: dict[str, str]) -> list[str]:
    failures: list[str] = []
    require(values.get("stress_corpus_gate") == "PASS", "stress_corpus_gate must be PASS", failures)
    require(values.get("failure_reason") == "none", "failure_reason must be none", failures)
    require(int_value(values, "matrix_plugin_count", failures) >= 1, "matrix_plugin_count must be positive", failures)
    require(int_value(values, "stress_plugin_count", failures) >= 1, "stress_plugin_count must be positive", failures)
    require(int_value(values, "plugin_count", failures) >= 1, "plugin_count must be positive", failures)
    require(int_value(values, "datapack_count", failures) >= 1, "datapack_count must be positive", failures)
    require(float_value(values, "done_seconds", failures) > 0.0, "done_seconds must be positive", failures)
    require(bool(values.get("launcher")), "launcher is missing", failures)
    require(bool(values.get("artifacts_json")), "artifacts_json is missing", failures)
    require(bool(values.get("optimized_artifact_sha256")), "optimized_artifact_sha256 is missing", failures)
    require(bool(values.get("optimized_runtime_run_sh_sha256")), "optimized_runtime_run_sh_sha256 is missing", failures)
    require(bool(values.get("optimized_runtime_native_library_sha256")), "optimized_runtime_native_library_sha256 is missing", failures)

    hard_errors = resolve_path(values.get("hard_errors", "reports/stress-corpus-hard-errors.txt"))
    if not hard_errors.is_file():
        failures.append(f"hard_errors evidence missing: {hard_errors}")
    elif hard_errors.stat().st_size != 0:
        failures.append(f"hard_errors is not empty: {hard_errors}")

    for key in ("status_json", "log", "serverpulse_influx_log"):
        raw = values.get(key)
        if not raw:
            failures.append(f"{key} is missing")
            continue
        path = resolve_path(raw)
        if not path.is_file():
            failures.append(f"{key} evidence missing: {path}")

    for path in [
        summary,
        report_evidence_path(values, summary, "join_report", "stress-corpus-join.txt"),
        report_evidence_path(values, summary, "inspection_txt", "stress-corpus-inspection.txt"),
        report_evidence_path(values, summary, "inspection_json", "stress-corpus-inspection.json"),
        report_evidence_path(values, summary, "artifacts_csv", "stress-corpus-artifacts.csv"),
        report_evidence_path(values, summary, "stress_manifest", "stress-corpus-manifest.json"),
    ]:
        if not path.is_file():
            failures.append(f"required evidence missing: {path}")
    return failures


def copy_evidence(
    src: pathlib.Path,
    dst_name: str,
    evidence_dir: pathlib.Path,
    manifest: list[str],
    records: list[dict[str, object]],
    seen: set[pathlib.Path],
) -> None:
    resolved = src.resolve()
    if resolved in seen:
        return
    if not src.is_file():
        raise FileNotFoundError(src)
    dst = evidence_dir / dst_name
    shutil.copy2(src, dst)
    digest = sha256(dst)
    size = dst.stat().st_size
    manifest.append(f"{dst_name}\tsha256={digest}\tbytes={size}\tsource={src}")
    records.append(
        {
            "logical_name": dst_name,
            "relative_path": f"evidence/{dst_name}",
            "sha256": digest,
            "bytes": size,
            "source": str(src),
        }
    )
    seen.add(resolved)


def write_claim_markdown(bundle_dir: pathlib.Path, data: dict[str, Any]) -> None:
    claim = data["claim"]
    text = f"""# Stress Corpus Boot/Join Evidence Bundle

Allowed claim:

> {ALLOWED_CLAIM}

Measured facts:

- stress_corpus_gate=true
- matrix_plugin_count={claim["matrix_plugin_count"]}
- stress_plugin_count={claim["stress_plugin_count"]}
- plugin_count={claim["plugin_count"]}
- datapack_count={claim["datapack_count"]}
- done_seconds={claim["done_seconds"]}
- status_version={claim["status_version_name"]}
- status_protocol={claim["status_protocol"]}
- join_probe=StressProbe join/check/quit completed

Explicit non-claims:

- Not a 500-bot, P500 mixed-gameplay, TPS, or player-capacity claim.
- Not proof for unlimited plugins, datapacks, players, mobs, chunks, or ticks.
- Not a full Rust Paper runtime claim.
- Not real-player gameplay parity.
- Not a multi-hour soak claim.

Reproduce:

```bash
MC_EULA_AGREE=true ./scripts/run_stress_corpus_gate.sh
python3 ./scripts/export_stress_corpus_bundle.py --out-dir reports/stress-corpus-bundle-current --force
python3 ./scripts/validate_stress_corpus_bundle.py reports/stress-corpus-bundle-current
```
"""
    (bundle_dir / "CLAIM.md").write_text(text, encoding="utf-8")


def build_bundle(summary: pathlib.Path, out_dir: pathlib.Path, *, force: bool) -> dict[str, Any]:
    summary = summary.resolve()
    values = parse_kv(summary)
    failures = validate_source_evidence(summary, values)
    if failures:
        return {"passed": False, "failures": failures, "out_dir": out_dir}

    out_dir = out_dir.resolve()
    if out_dir.exists():
        if not force:
            return {"passed": False, "failures": [f"out_dir already exists: {out_dir}"], "out_dir": out_dir}
        shutil.rmtree(out_dir)
    evidence_dir = out_dir / "evidence"
    evidence_dir.mkdir(parents=True)

    status_json = resolve_path(values["status_json"])
    status = read_json(status_json)
    version = status.get("version") if isinstance(status, dict) else {}
    if not isinstance(version, dict):
        version = {}

    manifest_lines: list[str] = []
    evidence_records: list[dict[str, object]] = []
    seen: set[pathlib.Path] = set()

    fixed_evidence = [
        (summary, "stress-corpus-summary.txt"),
        (status_json, "stress-corpus-status.json"),
        (report_evidence_path(values, summary, "join_report", "stress-corpus-join.txt"), "stress-corpus-join.txt"),
        (resolve_path(values["hard_errors"]), "stress-corpus-hard-errors.txt"),
        (report_evidence_path(values, summary, "inspection_txt", "stress-corpus-inspection.txt"), "stress-corpus-inspection.txt"),
        (report_evidence_path(values, summary, "inspection_json", "stress-corpus-inspection.json"), "stress-corpus-inspection.json"),
        (report_evidence_path(values, summary, "artifacts_csv", "stress-corpus-artifacts.csv"), "stress-corpus-artifacts.csv"),
        (report_evidence_path(values, summary, "stress_manifest", "stress-corpus-manifest.json"), "stress-corpus-manifest.json"),
        (resolve_path(values["artifacts_json"]), "artifacts.json"),
        (resolve_path(values["log"]), pathlib.Path(values["log"]).name),
        (resolve_path(values["serverpulse_influx_log"]), pathlib.Path(values["serverpulse_influx_log"]).name),
    ]
    for src, logical in fixed_evidence:
        copy_evidence(src, logical, evidence_dir, manifest_lines, evidence_records, seen)

    claim = {
        "stress_corpus_boot_join_claim": True,
        "stress_corpus_gate": True,
        "failure_reason": values["failure_reason"],
        "matrix_plugin_count": int(values["matrix_plugin_count"]),
        "stress_plugin_count": int(values["stress_plugin_count"]),
        "plugin_count": int(values["plugin_count"]),
        "datapack_count": int(values["datapack_count"]),
        "done_seconds": float(values["done_seconds"]),
        "launcher": values["launcher"],
        "launcher_kind": values.get("launcher_kind", "unknown"),
        "optimized_artifact_sha256": values["optimized_artifact_sha256"],
        "optimized_runtime_run_sh_sha256": values["optimized_runtime_run_sh_sha256"],
        "optimized_runtime_native_library_sha256": values["optimized_runtime_native_library_sha256"],
        "status_version_name": version.get("name"),
        "status_protocol": version.get("protocol"),
        "claim_limits": [
            "not-player-capacity-or-tps-claim",
            "not-unlimited-players-mobs-chunks-ticks-plugins-or-datapacks",
            "not-full-paper-runtime-rust-rewrite",
            "not-real-player-gameplay-parity",
            "not-multi-hour-soak",
        ],
    }

    data: dict[str, Any] = {
        "schema": SCHEMA,
        "generated_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "allowed_claim": ALLOWED_CLAIM,
        "claim": claim,
        "required_evidence": REQUIRED_LOGICALS,
        "evidence_files": evidence_records,
        "explicit_non_claims": [
            "not a 500-bot, P500 mixed-gameplay, TPS, or player-capacity claim",
            "not proof for unlimited plugins, datapacks, players, mobs, chunks, or ticks",
            "not a full Rust Paper runtime claim",
            "not real-player gameplay parity",
            "not a multi-hour soak claim",
        ],
        "reproduce": [
            "MC_EULA_AGREE=true ./scripts/run_stress_corpus_gate.sh",
            "python3 ./scripts/export_stress_corpus_bundle.py --out-dir reports/stress-corpus-bundle-current --force",
            "python3 ./scripts/validate_stress_corpus_bundle.py reports/stress-corpus-bundle-current",
        ],
    }
    (out_dir / "bundle.json").write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (out_dir / "MANIFEST.txt").write_text("\n".join(manifest_lines) + "\n", encoding="utf-8")
    write_claim_markdown(out_dir, data)
    return {"passed": True, "failures": [], "out_dir": out_dir, "data": data}


def main() -> int:
    timestamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%d-%H%M%S")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--summary",
        type=pathlib.Path,
        default=ROOT / "reports" / "stress-corpus-summary.txt",
        help="stress-corpus-summary.txt path.",
    )
    parser.add_argument(
        "--out-dir",
        type=pathlib.Path,
        default=ROOT / "reports" / f"stress-corpus-bundle-{timestamp}",
        help="Bundle output directory.",
    )
    parser.add_argument("--force", action="store_true", help="Replace an existing output directory.")
    args = parser.parse_args()

    result = build_bundle(args.summary, args.out_dir, force=args.force)
    print(f"stress_corpus_bundle_export_pass={str(result['passed']).lower()}")
    print(f"bundle_dir={result['out_dir']}")
    print(f"failure_count={len(result['failures'])}")
    if result["passed"]:
        print(f"bundle_index={result['out_dir'] / 'bundle.json'}")
        print(f"bundle_claim={result['out_dir'] / 'CLAIM.md'}")
        print(f"evidence_file_count={len(result['data']['evidence_files'])}")
    for failure in result["failures"]:
        print(f"stress_corpus_bundle_export_failure={failure}")
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
