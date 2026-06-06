#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
source "$ROOT/scripts/production_ready_claim_smoke_fixture.sh"

create_production_ready_claim_smoke_fixture "$TMP_DIR" READINESS_REPORT REPORTS_DIR
COLD="$REPORTS_DIR/load-production-500-cold-current-artifact-smoke-summary.txt"
WARM="$REPORTS_DIR/load-production-500-warm-current-artifact-smoke-summary.txt"
ARTIFACT_HASHES="$REPORTS_DIR/artifact-hashes.txt"
ARTIFACTS_JSON="$REPORTS_DIR/artifacts.json"

python3 "$ROOT/scripts/evaluate_production_soak.py" \
  --cold-summary "$COLD" \
  --warm-summary "$WARM" \
  --artifact-hashes "$ARTIFACT_HASHES" \
  --artifacts-json "$ARTIFACTS_JSON" \
  --min-load-window-metrics-samples 300 \
  --min-block-place-packets 120000 \
  --min-block-dig-packets 120000 \
  --report "$TMP_DIR/soak-pass.txt" >/dev/null
grep -q '^production_ready_soak_claim_eligible=true$' "$TMP_DIR/soak-pass.txt"
grep -q '^soak_gate_pass=true$' "$TMP_DIR/soak-pass.txt"

bad_cold="$TMP_DIR/bad-cold-summary.txt"
sed 's/load_window_metrics_samples=300/load_window_metrics_samples=100/' "$COLD" > "$bad_cold"
if python3 "$ROOT/scripts/evaluate_production_soak.py" \
  --cold-summary "$bad_cold" \
  --warm-summary "$WARM" \
  --artifact-hashes "$ARTIFACT_HASHES" \
  --artifacts-json "$ARTIFACTS_JSON" \
  --min-load-window-metrics-samples 300 \
  --min-block-place-packets 120000 \
  --min-block-dig-packets 120000 \
  --report "$TMP_DIR/soak-fail.txt" >/dev/null 2>&1; then
  echo "expected soak smoke failure did not happen" >&2
  exit 1
fi
grep -q '^production_ready_soak_claim_eligible=false$' "$TMP_DIR/soak-fail.txt"
grep -q 'cold: load_window_metrics_samples=100 < required 300' "$TMP_DIR/soak-fail.txt"

if python3 "$ROOT/scripts/evaluate_production_soak.py" \
  --cold-summary "$COLD" \
  --warm-summary skipped \
  --artifact-hashes "$ARTIFACT_HASHES" \
  --artifacts-json "$ARTIFACTS_JSON" \
  --min-load-window-metrics-samples 300 \
  --min-block-place-packets 120000 \
  --min-block-dig-packets 120000 \
  --report "$TMP_DIR/soak-warm-skipped.txt" >/dev/null 2>&1; then
  echo "expected warm skipped soak smoke failure did not happen" >&2
  exit 1
fi
grep -q '^warm_skipped=true$' "$TMP_DIR/soak-warm-skipped.txt"
grep -q '^failure=warm production-500-warm summary skipped$' "$TMP_DIR/soak-warm-skipped.txt"

echo "evaluate_production_soak_smoke=PASS"
