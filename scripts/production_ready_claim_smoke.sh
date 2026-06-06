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

PRODUCTION_READY_CLAIM_REPORTS_DIR="$REPORTS_DIR" \
PRODUCTION_READY_CLAIM_REPORT="$TMP/claim-verdict.txt" \
  "$ROOT/scripts/production_ready_claim.sh" "$TMP/bundle" > "$TMP/claim.out"
test "$(wc -l < "$TMP/claim.out")" -eq 2
sed -n '1p' "$TMP/claim.out" | rg -q '^production-ready для измеренного 500 bots / 32 view / 32 simulation / creative block'
sed -n '2p' "$TMP/claim.out" | rg -q '^Это не claim про полный Rust Paper runtime, unlimited plugins, real-player gameplay или multi-hour soak\.$'
rg -q '^claim_assertion_pass=true$' "$TMP/claim-verdict.txt"
rg -q '^current_artifact_freshness_pass=true$' "$TMP/claim-verdict.txt"

echo "production_ready_claim_smoke=PASS"
