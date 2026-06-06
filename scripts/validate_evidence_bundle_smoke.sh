#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
BUNDLE="$TMP/bundle-current"
source "$ROOT/scripts/production_ready_claim_smoke_fixture.sh"

create_production_ready_claim_smoke_fixture "$TMP" READINESS_REPORT REPORTS_DIR

python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$READINESS_REPORT" \
  --out-dir "$BUNDLE" > "$TMP/export.out"

python3 "$ROOT/scripts/validate_evidence_bundle.py" \
  "$BUNDLE" \
  --reports-dir "$REPORTS_DIR" > "$TMP/good.out"
rg -q '^evidence_bundle_validation_pass=true$' "$TMP/good.out"
rg -q '^cold_summary_path=' "$TMP/good.out"
rg -q '^warm_summary_path=' "$TMP/good.out"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/good.out"
rg -q '^plugin-matrix.log\t' "$BUNDLE/MANIFEST.txt"
rg -q '^restart-recovery.log\t' "$BUNDLE/MANIFEST.txt"
rg -q '^forced-ticket-persistence-first.log\t' "$BUNDLE/MANIFEST.txt"
rg -q '^forced-ticket-persistence-restart.log\t' "$BUNDLE/MANIFEST.txt"

cp -a "$BUNDLE" "$TMP/missing-plugin-log"
rm -f "$TMP/missing-plugin-log/evidence/plugin-matrix.log"
if python3 "$ROOT/scripts/validate_evidence_bundle.py" \
  "$TMP/missing-plugin-log" \
  --reports-dir "$REPORTS_DIR" > "$TMP/missing-plugin-log.out" 2>&1; then
  echo "Expected missing plugin matrix log to fail validation." >&2
  exit 1
fi
rg -q '^evidence_bundle_validation_pass=false$' "$TMP/missing-plugin-log.out"
rg -q 'plugin-matrix\.log: .*/evidence/plugin-matrix\.log is missing' \
  "$TMP/missing-plugin-log.out"

cp -a "$BUNDLE" "$TMP/missing-readiness-status"
sed -i '/^soak_gate_pass=/d' \
  "$TMP/missing-readiness-status/evidence/production-500-readiness-gate.txt"
if python3 "$ROOT/scripts/validate_evidence_bundle.py" \
  "$TMP/missing-readiness-status" \
  --reports-dir "$REPORTS_DIR" > "$TMP/missing-readiness-status.out" 2>&1; then
  echo "Expected missing readiness gate status to fail validation." >&2
  exit 1
fi
rg -q '^evidence_bundle_validation_pass=false$' "$TMP/missing-readiness-status.out"
rg -q 'readiness\.soak_gate_pass is missing' "$TMP/missing-readiness-status.out"

cp -a "$BUNDLE" "$TMP/missing-cold-summary"
sed -i '/^cold_summary_path=/d' \
  "$TMP/missing-cold-summary/evidence/production-500-soak-gate.txt"
if python3 "$ROOT/scripts/validate_evidence_bundle.py" \
  "$TMP/missing-cold-summary" \
  --reports-dir "$REPORTS_DIR" > "$TMP/missing-cold-summary.out" 2>&1; then
  echo "Expected missing cold summary reference to fail validation." >&2
  exit 1
fi
rg -q '^evidence_bundle_validation_pass=false$' "$TMP/missing-cold-summary.out"
rg -q 'soak\.cold_summary_path is missing' "$TMP/missing-cold-summary.out"

cp -a "$BUNDLE" "$TMP/missing-artifact-hashes"
: > "$TMP/missing-artifact-hashes/evidence/artifact-hashes.txt"
if python3 "$ROOT/scripts/validate_evidence_bundle.py" \
  "$TMP/missing-artifact-hashes" \
  --reports-dir "$REPORTS_DIR" > "$TMP/missing-artifact-hashes.out" 2>&1; then
  echo "Expected missing artifact hash rows to fail validation." >&2
  exit 1
fi
rg -q '^evidence_bundle_validation_pass=false$' "$TMP/missing-artifact-hashes.out"
rg -q 'artifact_hashes: no hash rows found' "$TMP/missing-artifact-hashes.out"

python3 - "$BUNDLE" "$REPORTS_DIR" <<'PY'
from __future__ import annotations

import datetime as dt
import json
from pathlib import Path
import os
import sys

bundle = Path(sys.argv[1])
reports_dir = Path(sys.argv[2])
data = json.loads((bundle / "bundle.json").read_text(encoding="utf-8"))
generated = dt.datetime.fromisoformat(data["generated_at_utc"].replace("Z", "+00:00"))
if generated.tzinfo is None:
    generated = generated.replace(tzinfo=dt.timezone.utc)
target = reports_dir / "load-production-500-warm-current-artifact-smoke-summary.txt"
stale_epoch = int(generated.timestamp()) + 100
os.utime(target, (stale_epoch, stale_epoch))
PY
if python3 "$ROOT/scripts/validate_evidence_bundle.py" \
  "$BUNDLE" \
  --reports-dir "$REPORTS_DIR" > "$TMP/stale-current.out" 2>&1; then
  echo "Expected stale current bundle freshness to fail validation." >&2
  exit 1
fi
rg -q '^evidence_bundle_validation_pass=false$' "$TMP/stale-current.out"
rg -q '^current_artifact_freshness_pass=false$' "$TMP/stale-current.out"
rg -q 'current_artifact_freshness: bundle generated_at_utc=' "$TMP/stale-current.out"

echo "validate_evidence_bundle_smoke=PASS"
