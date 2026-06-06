#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
source "$ROOT/scripts/production_ready_claim_smoke_fixture.sh"

create_production_ready_claim_smoke_fixture "$TMP" READINESS_REPORT REPORTS_DIR

SOAK="$TMP/soak.txt"
REPEAT="$TMP/repeat.txt"
PLUGIN="$TMP/plugin.txt"
RESTART="$TMP/restart.txt"
FORCED="$TMP/forced.txt"
ARTIFACT_HASHES="$TMP/artifact-hashes.txt"
GO_NOGO="$REPORTS_DIR/production-500-go-nogo-current.txt"

cp "$REPORTS_DIR/production-500-soak-gate.txt" "$SOAK"
cp "$REPORTS_DIR/production-500-repeat-quorum.txt" "$REPEAT"
cp "$REPORTS_DIR/plugin-matrix-summary.txt" "$PLUGIN"
cp "$REPORTS_DIR/restart-recovery-summary.txt" "$RESTART"
cp "$REPORTS_DIR/forced-ticket-persistence-summary.txt" "$FORCED"
cp "$REPORTS_DIR/artifact-hashes.txt" "$ARTIFACT_HASHES"

if python3 "$ROOT/scripts/evaluate_production_readiness.py" \
  --soak-report "$SOAK" \
  --repeat-report "$REPEAT" \
  --plugin-matrix-summary "$PLUGIN" \
  --restart-recovery-summary "$RESTART" \
  --forced-ticket-summary "$FORCED" \
  --go-nogo-report "$GO_NOGO" \
  --skip-artifact-hash-check \
  --report "$TMP/readiness.txt" >/dev/null; then
  echo "readiness smoke expected skipped artifact hash check to disable production claim" >&2
  exit 1
fi

if ! rg -q "production_ready_500_claim=false" "$TMP/readiness.txt"; then
  echo "readiness smoke expected skipped artifact hash claim to stay false" >&2
  exit 1
fi
if ! rg -q "claim_disabled_by_unverified_artifacts=true" "$TMP/readiness.txt"; then
  echo "readiness smoke expected skipped artifact hash claim-disabled marker" >&2
  exit 1
fi
if ! rg -q "artifact_hashes_pass=false" "$TMP/readiness.txt"; then
  echo "readiness smoke expected skipped artifact hash pass to stay false" >&2
  exit 1
fi
if ! rg -q "current_artifact_consistency_pass=false" "$TMP/readiness.txt"; then
  echo "readiness smoke expected skipped current artifact consistency to stay false" >&2
  exit 1
fi

python3 - "$SOAK" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
text = text.replace(
    "production_ready_soak_claim_eligible=true",
    "production_ready_soak_claim_eligible=false",
    1,
)
path.write_text(text)
PY

if python3 "$ROOT/scripts/evaluate_production_readiness.py" \
  --soak-report "$SOAK" \
  --repeat-report "$REPEAT" \
  --plugin-matrix-summary "$PLUGIN" \
  --restart-recovery-summary "$RESTART" \
  --forced-ticket-summary "$FORCED" \
  --go-nogo-report "$GO_NOGO" \
  --skip-artifact-hash-check \
  --report "$TMP/readiness-bad.txt" >/dev/null; then
  echo "readiness smoke expected failure did not fail" >&2
  exit 1
fi

python3 - "$TMP" "$SOAK" "$REPEAT" "$ARTIFACT_HASHES" <<'PY'
from pathlib import Path
import hashlib
import re
import sys

tmp = Path(sys.argv[1])
soak_path = Path(sys.argv[2])
repeat_path = Path(sys.argv[3])
artifact_hashes_path = Path(sys.argv[4])

optimized = tmp / "optimized-artifact-smoke.bin"
optimized.write_bytes(b"smoke optimized artifact\n")
run_sh = tmp / "optimized-run-smoke.sh"
run_sh.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
native_lib = tmp / "libpaper_native_jni.so"
native_lib.write_bytes(b"native smoke library\n")
chunk_encode_native_lib = tmp / "libpaper_native_chunk_encode_jni.so"
chunk_encode_native_lib.write_bytes(b"native chunk encode smoke library\n")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


opt_sha = sha256(optimized)
run_sha = sha256(run_sh)
native_sha = sha256(native_lib)
chunk_encode_native_sha = sha256(chunk_encode_native_lib)


def replace(path: Path, mapping: dict[str, str]) -> None:
    text = path.read_text(encoding="utf-8")
    for key, value in mapping.items():
        pattern = rf"^{re.escape(key)}=.*$"
        if re.search(pattern, text, flags=re.MULTILINE):
            text = re.sub(pattern, f"{key}={value}", text, flags=re.MULTILINE)
        else:
            text += f"{key}={value}\n"
    path.write_text(text, encoding="utf-8")


replace(
    soak_path,
    {
        "production_ready_soak_claim_eligible": "true",
        "optimized_artifact_path": str(optimized),
        "optimized_runtime_run_sh": str(run_sh),
        "optimized_runtime_native_library": str(native_lib),
        "optimized_runtime_chunk_encode_native_library": str(chunk_encode_native_lib),
        "optimized_artifact_sha256": opt_sha,
        "optimized_runtime_native_library_sha256": native_sha,
        "optimized_runtime_chunk_encode_native_library_sha256": chunk_encode_native_sha,
        "cold_optimized_artifact_sha256": opt_sha,
        "cold_optimized_runtime_jar_sha256": opt_sha,
        "warm_optimized_artifact_sha256": opt_sha,
        "warm_optimized_runtime_jar_sha256": opt_sha,
        "cold_optimized_runtime_run_sh_sha256": run_sha,
        "warm_optimized_runtime_run_sh_sha256": run_sha,
        "cold_optimized_runtime_native_library_sha256": native_sha,
        "warm_optimized_runtime_native_library_sha256": native_sha,
        "cold_optimized_runtime_chunk_encode_native_library_sha256": chunk_encode_native_sha,
        "warm_optimized_runtime_chunk_encode_native_library_sha256": chunk_encode_native_sha,
    },
)
replace(
    repeat_path,
    {
        "run_1_optimized_artifact_sha256": opt_sha,
        "run_2_optimized_artifact_sha256": opt_sha,
        "run_3_optimized_artifact_sha256": opt_sha,
    },
)
artifact_hashes_path.write_text(
    f"{opt_sha}  {optimized}\n{run_sha}  {run_sh}\n{native_sha}  {native_lib}\n{chunk_encode_native_sha}  {chunk_encode_native_lib}\n",
    encoding="utf-8",
)
PY

python3 "$ROOT/scripts/evaluate_production_readiness.py" \
  --soak-report "$SOAK" \
  --repeat-report "$REPEAT" \
  --plugin-matrix-summary "$PLUGIN" \
  --restart-recovery-summary "$RESTART" \
  --forced-ticket-summary "$FORCED" \
  --artifact-hashes "$ARTIFACT_HASHES" \
  --go-nogo-report "$GO_NOGO" \
  --report "$TMP/readiness-current.txt" >/dev/null

if ! rg -q "current_artifact_consistency_pass=true" "$TMP/readiness-current.txt"; then
  echo "readiness smoke failed to validate current artifact consistency" >&2
  exit 1
fi
if ! rg -q "current_optimized_runtime_native_library_sha256=" "$TMP/readiness-current.txt"; then
  echo "readiness smoke failed to emit current native runtime sha" >&2
  exit 1
fi
if ! rg -q "current_optimized_runtime_chunk_encode_native_library_sha256=" "$TMP/readiness-current.txt"; then
  echo "readiness smoke failed to emit current chunk encode native runtime sha" >&2
  exit 1
fi

python3 - "$REPEAT" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(
    r"^run_2_optimized_artifact_sha256=.*$",
    "run_2_optimized_artifact_sha256=0000000000000000000000000000000000000000000000000000000000000000",
    text,
    flags=re.MULTILINE,
)
path.write_text(text, encoding="utf-8")
PY

if python3 "$ROOT/scripts/evaluate_production_readiness.py" \
  --soak-report "$SOAK" \
  --repeat-report "$REPEAT" \
  --plugin-matrix-summary "$PLUGIN" \
  --restart-recovery-summary "$RESTART" \
  --forced-ticket-summary "$FORCED" \
  --artifact-hashes "$ARTIFACT_HASHES" \
  --go-nogo-report "$GO_NOGO" \
  --report "$TMP/readiness-artifact-bad.txt" >/dev/null; then
  echo "readiness smoke expected current-artifact mismatch did not fail" >&2
  exit 1
fi

echo "evaluate_production_readiness_smoke=PASS"
