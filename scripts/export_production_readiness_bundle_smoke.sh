#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
source "$ROOT/scripts/production_ready_claim_smoke_fixture.sh"

create_production_ready_claim_smoke_fixture "$TMP" READINESS_REPORT REPORTS_DIR

python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$READINESS_REPORT" \
  --out-dir "$TMP/bundle" > "$TMP/export.out"

rg -q '^claim_bundle=' "$TMP/export.out"
rg -q '^claim_bundle_index=' "$TMP/export.out"
rg -q 'production_ready_500_claim=true' "$TMP/bundle/CLAIM.md"
rg -qi 'not unlimited plugin compatibility' "$TMP/bundle/CLAIM.md"
rg -q 'production-500-readiness-gate.txt' "$TMP/bundle/MANIFEST.txt"
rg -q 'production-500-go-nogo-current.txt' "$TMP/bundle/MANIFEST.txt"
rg -q '^plugin-matrix.log\t' "$TMP/bundle/MANIFEST.txt"
rg -q '^restart-recovery.log\t' "$TMP/bundle/MANIFEST.txt"
rg -q '^forced-ticket-persistence-first.log\t' "$TMP/bundle/MANIFEST.txt"
rg -q '^forced-ticket-persistence-restart.log\t' "$TMP/bundle/MANIFEST.txt"
test -f "$TMP/bundle/bundle.json"
test -f "$TMP/bundle/evidence/production-500-readiness-gate.txt"
test -f "$TMP/bundle/evidence/production-500-go-nogo-current.txt"
python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$TMP/bundle" \
  --require-current-freshness \
  --reports-dir "$REPORTS_DIR" > "$TMP/validate.out"
rg -q '^bundle_validation_pass=true$' "$TMP/validate.out"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/validate.out"

create_production_ready_claim_smoke_fixture "$TMP/missing-ref" MISSING_READINESS_REPORT MISSING_REPORTS_DIR
missing_plugin_summary="$(awk -F= '$1 == "plugin_matrix_summary" { print $2 }' "$MISSING_READINESS_REPORT")"
missing_plugin_log="$(awk -F= '$1 == "plugin_matrix_log" { print $2 }' "$missing_plugin_summary")"
rm -f "$missing_plugin_log"
if python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$MISSING_READINESS_REPORT" \
  --out-dir "$TMP/missing-ref-bundle" > "$TMP/missing-ref.out" 2>&1; then
  echo "Expected missing referenced evidence to fail bundle export." >&2
  exit 1
fi
rg -q 'bundle_failure=referenced_evidence_missing: .*plugin-matrix.log' "$TMP/missing-ref.out"

cp "$READINESS_REPORT" "$TMP/bad-readiness.txt"
sed -i 's/^production_ready_500_claim=true$/production_ready_500_claim=false/' "$TMP/bad-readiness.txt"
if python3 "$ROOT/scripts/export_production_readiness_bundle.py" \
  --readiness-report "$TMP/bad-readiness.txt" \
  --out-dir "$TMP/bad-bundle" > "$TMP/bad.out" 2>&1; then
  echo "Expected bad readiness report to fail bundle export." >&2
  exit 1
fi
rg -q 'bundle_failure=production_ready_500_claim=false expected=true' "$TMP/bad.out"

echo "export_production_readiness_bundle_smoke=PASS"
