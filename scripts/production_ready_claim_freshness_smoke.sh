#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
source "$ROOT/scripts/production_ready_claim_smoke_fixture.sh"

create_production_ready_claim_smoke_fixture "$TMP" READINESS_REPORT FIXTURE_REPORTS_DIR
python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$READINESS_REPORT" \
  --out-dir "$TMP/production-500-readiness-bundle-current" > "$TMP/export-current.out"

python3 - "$ROOT" "$TMP" <<'PY'
from __future__ import annotations

import datetime as dt
import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

root = Path(sys.argv[1])
tmp = Path(sys.argv[2])

bundle = tmp / "production-500-readiness-bundle-current"
reports = tmp / "reports"
reports.mkdir(parents=True, exist_ok=True)

current_artifacts = json.loads((root / "reports" / "artifacts.json").read_text(encoding="utf-8"))
opt_sha = current_artifacts["optimized"]["sha256"]
run_sha = current_artifacts["optimized_runtime"]["run_sh"]["sha256"]
native_sha = current_artifacts["optimized_runtime"]["native_library"]["sha256"]
chunk_encode_native_sha = current_artifacts["optimized_runtime"].get("chunk_encode_native_library", {}).get("sha256")
generated_epoch = 1_700_000_100
evidence_epoch = 1_700_000_000


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def replace_kv(path: Path, key: str, value: str) -> None:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    replaced = False
    out: list[str] = []
    for line in lines:
        if line.startswith(f"{key}="):
            out.append(f"{key}={value}")
            replaced = True
        else:
            out.append(line)
    if not replaced:
        out.append(f"{key}={value}")
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def delete_kv(path: Path, key: str) -> None:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    path.write_text(
        "\n".join(line for line in lines if not line.startswith(f"{key}=")) + "\n",
        encoding="utf-8",
    )


def set_mtime(path: Path, seconds: int) -> None:
    os.utime(path, (seconds, seconds))


def patch_manifest(path: Path, logical_name: str, digest: str, size: int, source: str = "smoke") -> None:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    out: list[str] = []
    replaced = False
    for line in lines:
        parts = line.split("\t")
        if parts and parts[0] == logical_name:
            fields = {}
            for part in parts[1:]:
                if "=" in part:
                    key, value = part.split("=", 1)
                    fields[key] = value
            source = fields.get("source", source)
            out.append(f"{logical_name}\tsha256={digest}\tbytes={size}\tsource={source}")
            replaced = True
        else:
            out.append(line)
    if not replaced:
        out.append(f"{logical_name}\tsha256={digest}\tbytes={size}\tsource={source}")
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def upsert_evidence_record(data: dict, logical_name: str, relative_path: str, digest: str, size: int) -> None:
    for record in data["evidence_files"]:
        if record.get("logical_name") == logical_name:
            record["sha256"] = digest
            record["bytes"] = size
            record["relative_path"] = relative_path
            return
    data["evidence_files"].append(
        {
            "logical_name": logical_name,
            "relative_path": relative_path,
            "sha256": digest,
            "bytes": size,
            "source": "smoke",
        }
    )


def write_bundle_log(logical_name: str, text: str) -> None:
    path = bundle / "evidence" / logical_name
    path.write_text(text, encoding="utf-8")
    digest = sha256(path)
    size = path.stat().st_size
    upsert_evidence_record(data, logical_name, f"evidence/{logical_name}", digest, size)
    patch_manifest(bundle / "MANIFEST.txt", logical_name, digest, size)


def write_artifacts(hash_value: str) -> None:
    data = json.loads(json.dumps(current_artifacts))
    data["optimized"]["sha256"] = hash_value
    (reports / "artifacts.json").write_text(
        json.dumps(data, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_current_artifact_evidence(name: str, seconds: int) -> None:
    path = reports / name
    path.write_text("gate_profile=production-500\ngate_pass=true\n", encoding="utf-8")
    os.utime(path, (seconds, seconds))


generated_at = dt.datetime.fromtimestamp(
    generated_epoch,
    dt.timezone.utc,
).isoformat()

bundle_index = bundle / "bundle.json"
data = json.loads(bundle_index.read_text(encoding="utf-8"))
data["generated_at_utc"] = generated_at
data["claim"]["optimized_artifact_sha256"] = opt_sha
data["claim"]["optimized_runtime_run_sh_sha256"] = run_sha
data["claim"]["optimized_runtime_native_library_sha256"] = native_sha
if chunk_encode_native_sha:
    data["claim"]["optimized_runtime_chunk_encode_native_library_sha256"] = chunk_encode_native_sha
else:
    data["claim"].pop("optimized_runtime_chunk_encode_native_library_sha256", None)

readiness = bundle / "evidence" / "production-500-readiness-gate.txt"
replace_kv(readiness, "optimized_artifact_sha256", opt_sha)
replace_kv(readiness, "current_optimized_artifact_sha256", opt_sha)
replace_kv(readiness, "current_optimized_runtime_run_sh_sha256", run_sha)
replace_kv(readiness, "current_optimized_runtime_native_library_sha256", native_sha)
if chunk_encode_native_sha:
    replace_kv(
        readiness,
        "current_optimized_runtime_chunk_encode_native_library_sha256",
        chunk_encode_native_sha,
    )
else:
    delete_kv(readiness, "current_optimized_runtime_chunk_encode_native_library_sha256")
    delete_kv(readiness, "optimized_runtime_chunk_encode_native_library_sha256")
readiness_sha = sha256(readiness)
readiness_size = readiness.stat().st_size

bundled_artifacts = bundle / "evidence" / "artifacts.json"
bundled_artifacts.write_text(
    json.dumps(current_artifacts, indent=2, sort_keys=True)
    + "\n",
    encoding="utf-8",
)
artifacts_sha = sha256(bundled_artifacts)
artifacts_size = bundled_artifacts.stat().st_size

native_proof = bundle / "evidence" / "libpaper_native_jni.so.sha256"
native_proof.write_text(
    f"{native_sha}  /root/rust/artifacts/optimized-runtime/native/libpaper_native_jni.so\n",
    encoding="utf-8",
)
native_proof_sha = sha256(native_proof)
native_proof_size = native_proof.stat().st_size
if chunk_encode_native_sha:
    chunk_encode_native_proof = bundle / "evidence" / "libpaper_native_chunk_encode_jni.so.sha256"
    chunk_encode_native_proof.write_text(
        f"{chunk_encode_native_sha}  /root/rust/artifacts/optimized-runtime/native/libpaper_native_chunk_encode_jni.so\n",
        encoding="utf-8",
    )
    chunk_encode_native_proof_sha = sha256(chunk_encode_native_proof)
    chunk_encode_native_proof_size = chunk_encode_native_proof.stat().st_size

upsert_evidence_record(
    data,
    "production-500-readiness-gate.txt",
    "evidence/production-500-readiness-gate.txt",
    readiness_sha,
    readiness_size,
)
upsert_evidence_record(data, "artifacts.json", "evidence/artifacts.json", artifacts_sha, artifacts_size)
upsert_evidence_record(
    data,
    "libpaper_native_jni.so.sha256",
    "evidence/libpaper_native_jni.so.sha256",
    native_proof_sha,
    native_proof_size,
)
if chunk_encode_native_sha:
    upsert_evidence_record(
        data,
        "libpaper_native_chunk_encode_jni.so.sha256",
        "evidence/libpaper_native_chunk_encode_jni.so.sha256",
        chunk_encode_native_proof_sha,
        chunk_encode_native_proof_size,
    )

bundle_index.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
patch_manifest(
    bundle / "MANIFEST.txt",
    "production-500-readiness-gate.txt",
    readiness_sha,
    readiness_size,
)
patch_manifest(bundle / "MANIFEST.txt", "artifacts.json", artifacts_sha, artifacts_size)
patch_manifest(
    bundle / "MANIFEST.txt",
    "libpaper_native_jni.so.sha256",
    native_proof_sha,
    native_proof_size,
)
if chunk_encode_native_sha:
    patch_manifest(
        bundle / "MANIFEST.txt",
        "libpaper_native_chunk_encode_jni.so.sha256",
        chunk_encode_native_proof_sha,
        chunk_encode_native_proof_size,
    )
for logical_name, text in [
    ("plugin-matrix.log", "smoke plugin matrix log\n"),
    ("restart-recovery.log", "smoke restart recovery log\n"),
    ("forced-ticket-persistence-first.log", "smoke forced ticket first log\n"),
    ("forced-ticket-persistence-restart.log", "smoke forced ticket restart log\n"),
]:
    write_bundle_log(logical_name, text)

artifact_hashes = bundle / "evidence" / "artifact-hashes.txt"
shutil.copy2(root / "reports" / "artifact-hashes.txt", artifact_hashes)
artifact_lines = artifact_hashes.read_text(encoding="utf-8").splitlines()
artifact_hashes_sha = sha256(artifact_hashes)
artifact_hashes_size = artifact_hashes.stat().st_size
artifact_hash_count = len(artifact_lines)
upsert_evidence_record(
    data,
    "artifact-hashes.txt",
    "evidence/artifact-hashes.txt",
    artifact_hashes_sha,
    artifact_hashes_size,
)
patch_manifest(bundle / "MANIFEST.txt", "artifact-hashes.txt", artifact_hashes_sha, artifact_hashes_size)
replace_kv(readiness, "artifact_hash_manifest_sha256", artifact_hashes_sha)
replace_kv(readiness, "artifact_hash_count", str(artifact_hash_count))
readiness_sha = sha256(readiness)
readiness_size = readiness.stat().st_size
data["claim"]["artifact_hash_count"] = artifact_hash_count
upsert_evidence_record(
    data,
    "production-500-readiness-gate.txt",
    "evidence/production-500-readiness-gate.txt",
    readiness_sha,
    readiness_size,
)
patch_manifest(
    bundle / "MANIFEST.txt",
    "production-500-readiness-gate.txt",
    readiness_sha,
    readiness_size,
)

bundle_index.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

set_mtime(bundle_index, generated_epoch)
set_mtime(bundle / "CLAIM.md", generated_epoch)
set_mtime(readiness, generated_epoch)

write_artifacts(opt_sha)
write_current_artifact_evidence(
    "load-production-500-cold-current-artifact-smoke-gate.txt",
    evidence_epoch,
)
write_current_artifact_evidence(
    "load-production-500-cold-current-artifact-smoke-summary.txt",
    evidence_epoch,
)
write_current_artifact_evidence(
    "load-production-500-warm-current-artifact-smoke-gate.txt",
    evidence_epoch,
)
write_current_artifact_evidence(
    "load-production-500-warm-current-artifact-smoke-summary.txt",
    evidence_epoch,
)
PY

BUNDLE="$TMP/production-500-readiness-bundle-current"
REPORTS="$TMP/reports"

if ! python3 "$ROOT/scripts/validate_production_readiness_bundle.py" "$BUNDLE" \
  --reports-dir "$REPORTS" \
  > "$TMP/current-validate.out" 2>&1; then
  cat "$TMP/current-validate.out" >&2
  exit 1
fi
rg -q '^bundle_validation_pass=true$' "$TMP/current-validate.out"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/current-validate.out"

PRODUCTION_READY_CLAIM_REPORTS_DIR="$REPORTS" \
PRODUCTION_READY_CLAIM_REPORT="$TMP/current-wrapper-verdict.txt" \
  "$ROOT/scripts/production_ready_claim.sh" "$BUNDLE" > "$TMP/current-wrapper.out"
rg -q '^production-ready ' "$TMP/current-wrapper.out"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/current-wrapper-verdict.txt"

python3 "$ROOT/scripts/publish_production_ready_claim.py" \
  "$BUNDLE" \
  --reports-dir "$REPORTS" \
  --out-prefix "$TMP/current-publication" \
  --verdict-report "$TMP/current-publication-verdict.txt" \
  > "$TMP/current-publication.out"
rg -q '^claim_publication_pass=true$' "$TMP/current-publication.out"
test -f "$TMP/current-publication.txt"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/current-publication-verdict.txt"

python3 - "$REPORTS" <<'PY'
import json
import sys
from pathlib import Path

reports = Path(sys.argv[1])
artifacts_json = reports / "artifacts.json"
data = json.loads(artifacts_json.read_text(encoding="utf-8"))
data["optimized"]["sha256"] = "b" * 64
artifacts_json.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" "$BUNDLE" \
  --reports-dir "$REPORTS" \
  > "$TMP/hash-drift-validate.out" 2>&1; then
  echo "expected validate_production_readiness_bundle.py to fail after artifact hash drift" >&2
  exit 1
fi
rg -q 'does not match current artifacts\.json optimized\.sha256' "$TMP/hash-drift-validate.out"

if PRODUCTION_READY_CLAIM_REPORTS_DIR="$REPORTS" \
  PRODUCTION_READY_CLAIM_REPORT="$TMP/hash-drift-wrapper-verdict.txt" \
  "$ROOT/scripts/production_ready_claim.sh" "$BUNDLE" > "$TMP/hash-drift-wrapper.out" 2>&1; then
  echo "expected production_ready_claim.sh to fail after artifact hash drift" >&2
  exit 1
fi
rg -q 'does not match current artifacts\.json optimized\.sha256' "$TMP/hash-drift-wrapper.out"

if python3 "$ROOT/scripts/publish_production_ready_claim.py" \
  "$BUNDLE" \
  --reports-dir "$REPORTS" \
  --out-prefix "$TMP/hash-drift-publication" \
  --verdict-report "$TMP/hash-drift-publication-verdict.txt" \
  > "$TMP/hash-drift-publication.out" 2>&1; then
  echo "expected publish_production_ready_claim.py to fail after artifact hash drift" >&2
  exit 1
fi
rg -q '^claim_publication_failure=claim assertion failed$' "$TMP/hash-drift-publication.out"
test ! -e "$TMP/hash-drift-publication.txt"

python3 - "$ROOT" "$REPORTS" <<'PY'
import json
import os
import sys
from pathlib import Path

root = Path(sys.argv[1])
reports = Path(sys.argv[2])
(reports / "artifacts.json").write_text(
    (root / "reports" / "artifacts.json").read_text(encoding="utf-8"),
    encoding="utf-8",
)
newer = reports / "load-production-500-cold-current-artifact-smoke-newer-summary.txt"
newer.write_text("gate_profile=production-500\ngate_pass=false\n", encoding="utf-8")
os.utime(newer, (1_700_000_200, 1_700_000_200))
PY

if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" "$BUNDLE" \
  --reports-dir "$REPORTS" \
  > "$TMP/newer-evidence-validate.out" 2>&1; then
  echo "expected validate_production_readiness_bundle.py to fail after newer current evidence" >&2
  exit 1
fi
rg -q 'older than latest current-artifact 500 gate/summary evidence' "$TMP/newer-evidence-validate.out"

if PRODUCTION_READY_CLAIM_REPORTS_DIR="$REPORTS" \
  PRODUCTION_READY_CLAIM_REPORT="$TMP/newer-evidence-wrapper-verdict.txt" \
  "$ROOT/scripts/production_ready_claim.sh" "$BUNDLE" > "$TMP/newer-evidence-wrapper.out" 2>&1; then
  echo "expected production_ready_claim.sh to fail after newer current-artifact evidence" >&2
  exit 1
fi
rg -q 'older than latest current-artifact 500 gate/summary evidence' "$TMP/newer-evidence-wrapper.out"

echo "production_ready_claim_freshness_smoke=PASS"
