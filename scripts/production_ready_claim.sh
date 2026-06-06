#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUNDLE="${1:-$ROOT/reports/production-500-readiness-bundle-current}"
REPORTS_DIR="${PRODUCTION_READY_CLAIM_REPORTS_DIR:-$ROOT/reports}"
REPORT="${PRODUCTION_READY_CLAIM_REPORT:-$ROOT/reports/production-500-claim-verdict.txt}"

python3 "$ROOT/scripts/assert_production_ready_claim.py" \
  "$BUNDLE" \
  --reports-dir "$REPORTS_DIR" \
  --report "$REPORT" \
  --claim-only
