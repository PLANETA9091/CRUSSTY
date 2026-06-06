#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
source "$ROOT/scripts/production_ready_claim_smoke_fixture.sh"

refresh_bundle_evidence_record() {
  local bundle="${1:?bundle is required}"
  local logical="${2:?logical name is required}"
  python3 - "$bundle" "$logical" <<'PY'
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

bundle = Path(sys.argv[1])
logical = sys.argv[2]
index_path = bundle / "bundle.json"
data = json.loads(index_path.read_text(encoding="utf-8"))

target_record = None
for record in data["evidence_files"]:
    if record["logical_name"] == logical:
        target_record = record
        break
if target_record is None:
    raise SystemExit(f"{logical}: evidence record missing")

path = bundle / target_record["relative_path"]
raw = path.read_bytes()
digest = hashlib.sha256(raw).hexdigest()
size = len(raw)
target_record["sha256"] = digest
target_record["bytes"] = size
index_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

manifest = bundle / "MANIFEST.txt"
lines = manifest.read_text(encoding="utf-8").splitlines()
for index, line in enumerate(lines):
    if not line.startswith(f"{logical}\t"):
        continue
    fields = line.split("\t")
    source = next((field for field in fields[1:] if field.startswith("source=")), "source=unknown")
    lines[index] = f"{logical}\tsha256={digest}\tbytes={size}\t{source}"
    break
else:
    raise SystemExit(f"{logical}: manifest row missing")
manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
}

create_production_ready_claim_smoke_fixture "$TMP" READINESS_REPORT REPORTS_DIR

python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$READINESS_REPORT" \
  --out-dir "$TMP/bundle" > "$TMP/export.out"

python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/bundle" \
  --reports-dir "$REPORTS_DIR" > "$TMP/validate.out"
rg -q '^bundle_validation_pass=true$' "$TMP/validate.out"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/validate.out"
rg -q '^plugin-matrix.log\t' "$TMP/bundle/MANIFEST.txt"
rg -q '^restart-recovery.log\t' "$TMP/bundle/MANIFEST.txt"
rg -q '^forced-ticket-persistence-first.log\t' "$TMP/bundle/MANIFEST.txt"
rg -q '^forced-ticket-persistence-restart.log\t' "$TMP/bundle/MANIFEST.txt"
python3 - "$TMP/bundle/bundle.json" "$REPORTS_DIR/artifacts.json" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

bundle = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
artifacts = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
claim = bundle["claim"]
assert claim["optimized_artifact_sha256"] == artifacts["optimized"]["sha256"]
assert claim["optimized_runtime_run_sh_sha256"] == artifacts["optimized_runtime"]["run_sh"]["sha256"]
assert claim["optimized_runtime_native_library_sha256"] == artifacts["optimized_runtime"]["native_library"]["sha256"]
assert claim["optimized_runtime_chunk_encode_native_library_sha256"] == artifacts["optimized_runtime"]["chunk_encode_native_library"]["sha256"]
PY

cp -a "$TMP/bundle" "$TMP/duplicate-readiness-key"
printf 'production_ready_500_claim=true\n' >> \
  "$TMP/duplicate-readiness-key/evidence/production-500-readiness-gate.txt"
refresh_bundle_evidence_record "$TMP/duplicate-readiness-key" "production-500-readiness-gate.txt"
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/duplicate-readiness-key" \
  --reports-dir "$REPORTS_DIR" > "$TMP/duplicate-readiness-key.out" 2>&1; then
  echo "Expected duplicate readiness keys to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/duplicate-readiness-key.out"
rg -q 'readiness:[0-9]+: duplicate key production_ready_500_claim' \
  "$TMP/duplicate-readiness-key.out"

cp -a "$TMP/bundle" "$TMP/missing-runtime-hashes"
python3 - "$TMP/missing-runtime-hashes/bundle.json" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
del data["claim"]["optimized_runtime_run_sh_sha256"]
del data["claim"]["optimized_runtime_native_library_sha256"]
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/missing-runtime-hashes" \
  --reports-dir "$REPORTS_DIR" > "$TMP/missing-runtime-hashes.out" 2>&1; then
  echo "Expected missing runtime/native hashes in bundle.json to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/missing-runtime-hashes.out"
rg -q 'claim.optimized_runtime_run_sh_sha256 is missing' \
  "$TMP/missing-runtime-hashes.out"
rg -q 'claim.optimized_runtime_native_library_sha256 is required for current bundle native proof' \
  "$TMP/missing-runtime-hashes.out"

cp -a "$TMP/bundle" "$TMP/missing-plugin-log"
rm -f "$TMP/missing-plugin-log/evidence/plugin-matrix.log"
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/missing-plugin-log" \
  --reports-dir "$REPORTS_DIR" > "$TMP/missing-plugin-log.out" 2>&1; then
  echo "Expected missing raw log evidence to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/missing-plugin-log.out"
rg -q 'plugin-matrix.log: .* is missing' "$TMP/missing-plugin-log.out"

cp -a "$TMP/bundle" "$TMP/forged-surface"
python3 - "$TMP/forged-surface/bundle.json" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
data["measured_load_surface"]["cold"]["tps1_min"] = 20.0
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/forged-surface" \
  --reports-dir "$REPORTS_DIR" > "$TMP/forged-surface.out" 2>&1; then
  echo "Expected forged measured_load_surface to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/forged-surface.out"
rg -q 'bundle\.measured_load_surface\.cold\.tps1_min=20\.0 readiness\.cold_load_window_tps1_min=18\.86' \
  "$TMP/forged-surface.out"

cp -a "$TMP/bundle" "$TMP/partial-artifact-hashes"
sed -n '1p' "$TMP/partial-artifact-hashes/evidence/artifact-hashes.txt" > \
  "$TMP/partial-artifact-hashes/evidence/artifact-hashes.txt.tmp"
mv "$TMP/partial-artifact-hashes/evidence/artifact-hashes.txt.tmp" \
  "$TMP/partial-artifact-hashes/evidence/artifact-hashes.txt"
refresh_bundle_evidence_record "$TMP/partial-artifact-hashes" "artifact-hashes.txt"
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/partial-artifact-hashes" \
  --reports-dir "$REPORTS_DIR" > "$TMP/partial-artifact-hashes.out" 2>&1; then
  echo "Expected partial artifact hash manifest to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/partial-artifact-hashes.out"
rg -q 'artifact_hashes: claim\.artifact_hash_count=7 actual_rows=1' \
  "$TMP/partial-artifact-hashes.out"

cp -a "$TMP/bundle" "$TMP/bad-evidence"
python3 - "$TMP/bad-evidence/evidence/production-500-readiness-gate.txt" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace("production_ready_500_claim=true", "production_ready_500_claim=false", 1)
path.write_text(text, encoding="utf-8")
PY
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/bad-evidence" \
  --reports-dir "$REPORTS_DIR" > "$TMP/bad-evidence.out" 2>&1; then
  echo "Expected tampered readiness evidence to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/bad-evidence.out"
rg -q 'production-500-readiness-gate.txt: sha256=' "$TMP/bad-evidence.out"

cp -a "$TMP/bundle" "$TMP/bad-index"
python3 - "$TMP/bad-index/bundle.json" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
data["claim"]["production_ready_500_claim"] = False
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/bad-index" \
  --reports-dir "$REPORTS_DIR" > "$TMP/bad-index.out" 2>&1; then
  echo "Expected tampered bundle index to fail bundle validation." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/bad-index.out"
rg -q 'claim.production_ready_500_claim=False expected=True' "$TMP/bad-index.out"

cp -a "$TMP/bundle" "$TMP/live-artifact-drift"
python3 - "$REPORTS_DIR/artifacts.json" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

artifacts_json = Path(sys.argv[1])
data = json.loads(artifacts_json.read_text(encoding="utf-8"))
Path(data["optimized"]["path"]).write_bytes(b"mutated optimized artifact\n")
PY
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/live-artifact-drift" \
  --reports-dir "$REPORTS_DIR" > "$TMP/live-artifact-drift.out" 2>&1; then
  echo "Expected live optimized artifact byte drift to fail bundle validation." >&2
  exit 1
fi
rg -q '^current_artifact_freshness_pass=false$' "$TMP/live-artifact-drift.out"
rg -q 'current_artifact_freshness\.artifacts_json\.optimized\.path live sha256=' \
  "$TMP/live-artifact-drift.out"

create_production_ready_claim_smoke_fixture "$TMP/restored-current" READINESS_REPORT RESTORED_REPORTS_DIR
REPORTS_DIR="$RESTORED_REPORTS_DIR"
python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$READINESS_REPORT" \
  --out-dir "$TMP/restored-bundle" > "$TMP/restored-export.out"

python3 - "$REPORTS_DIR" <<'PY'
import json
import sys
from pathlib import Path

reports = Path(sys.argv[1])
(reports / "artifacts.json").write_text(
    json.dumps({"optimized": {"sha256": "c" * 64}}, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/bundle" \
  --reports-dir "$REPORTS_DIR" > "$TMP/stale.out" 2>&1; then
  echo "Expected default bundle validation to fail closed on current artifact drift." >&2
  exit 1
fi
rg -q '^current_artifact_freshness_pass=false$' "$TMP/stale.out"
rg -q 'does not match current artifacts\.json optimized\.sha256' "$TMP/stale.out"

python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/restored-bundle" \
  --allow-stale-freshness \
  --reports-dir "$REPORTS_DIR" > "$TMP/stale-allowed.out"
rg -q '^bundle_validation_pass=true$' "$TMP/stale-allowed.out"
if rg -q '^current_artifact_freshness_pass=' "$TMP/stale-allowed.out"; then
  echo "Expected --allow-stale-freshness to skip current artifact freshness output." >&2
  exit 1
fi

cp -a "$TMP/restored-bundle" "$TMP/restored-bundle-current"
if python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/restored-bundle-current" \
  --allow-stale-freshness \
  --reports-dir "$REPORTS_DIR" > "$TMP/current-stale-disallowed.out" 2>&1; then
  echo "Expected *-current bundle validation to require freshness even with --allow-stale-freshness." >&2
  exit 1
fi
rg -q '^bundle_validation_pass=false$' "$TMP/current-stale-disallowed.out"
rg -q '^current_freshness_required=true$' "$TMP/current-stale-disallowed.out"
rg -q '^current_artifact_freshness_pass=false$' "$TMP/current-stale-disallowed.out"
rg -q 'does not match current artifacts\.json optimized\.sha256' \
  "$TMP/current-stale-disallowed.out"

echo "validate_production_readiness_bundle_smoke=PASS"
