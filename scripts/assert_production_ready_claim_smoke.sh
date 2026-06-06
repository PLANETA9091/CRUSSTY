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

python3 "$ROOT/scripts/assert_production_ready_claim.py" \
  "$TMP/bundle" \
  --reports-dir "$REPORTS_DIR" \
  --report "$TMP/claim-verdict.txt" > "$TMP/assert.out"
rg -q '^claim_assertion_pass=true$' "$TMP/assert.out"
rg -q '^claim_assertion_pass=true$' "$TMP/claim-verdict.txt"
rg -q '^claim_ru=production-ready для измеренного 500 bots / 32 view / 32 simulation / creative block' "$TMP/claim-verdict.txt"
rg -q '^non_claim_ru=Это не claim про полный Rust Paper runtime, unlimited plugins, real-player gameplay или multi-hour soak\.$' "$TMP/claim-verdict.txt"
rg -q '^optimized_runtime_run_sh_sha256=[0-9a-f]{64}$' "$TMP/claim-verdict.txt"
rg -q '^optimized_runtime_native_library_sha256=[0-9a-f]{64}$' "$TMP/claim-verdict.txt"
rg -q '^optimized_runtime_chunk_encode_native_library_sha256=[0-9a-f]{64}$' "$TMP/claim-verdict.txt"
rg -q '^current_optimized_runtime_run_sh_sha256=[0-9a-f]{64}$' "$TMP/claim-verdict.txt"
rg -q '^current_optimized_runtime_native_library_sha256=[0-9a-f]{64}$' "$TMP/claim-verdict.txt"
rg -q '^current_optimized_runtime_chunk_encode_native_library_sha256=[0-9a-f]{64}$' "$TMP/claim-verdict.txt"

python3 "$ROOT/scripts/assert_production_ready_claim.py" \
  "$TMP/bundle" \
  --reports-dir "$REPORTS_DIR" \
  --report "$TMP/claim-only-verdict.txt" \
  --claim-only > "$TMP/claim-only.out"
test "$(wc -l < "$TMP/claim-only.out")" -eq 2
sed -n '1p' "$TMP/claim-only.out" | rg -q '^production-ready для измеренного 500 bots / 32 view / 32 simulation / creative block'
sed -n '2p' "$TMP/claim-only.out" | rg -q '^Это не claim про полный Rust Paper runtime, unlimited plugins, real-player gameplay или multi-hour soak\.$'
rg -q '^claim_assertion_pass=true$' "$TMP/claim-only-verdict.txt"

cp -a "$TMP/bundle" "$TMP/bad-claim"
python3 - "$TMP/bad-claim/bundle.json" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
data["allowed_claim"] = "Production-ready for all plugins and real-player gameplay."
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if python3 "$ROOT/scripts/assert_production_ready_claim.py" \
  "$TMP/bad-claim" \
  --reports-dir "$REPORTS_DIR" \
  --report "$TMP/bad-claim-verdict.txt" > "$TMP/bad-claim.out" 2>&1; then
  echo "Expected broadened allowed_claim to fail claim assertion." >&2
  exit 1
fi
rg -q '^claim_assertion_pass=false$' "$TMP/bad-claim.out"
rg -q 'claim_assertion_failure=allowed_claim is not exact' "$TMP/bad-claim.out"

cp -a "$TMP/bundle" "$TMP/bad-surface"
python3 - "$TMP/bad-surface/bundle.json" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
data["measured_load_surface"]["cold"]["tps1_min"] = 17.99
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if python3 "$ROOT/scripts/assert_production_ready_claim.py" \
  "$TMP/bad-surface" \
  --reports-dir "$REPORTS_DIR" \
  --report "$TMP/bad-surface-verdict.txt" > "$TMP/bad-surface.out" 2>&1; then
  echo "Expected weakened measured surface to fail claim assertion." >&2
  exit 1
fi
rg -q '^claim_assertion_pass=false$' "$TMP/bad-surface.out"
rg -q 'claim_assertion_failure=cold.tps1_min=17.99 < 18.0' "$TMP/bad-surface.out"

echo "assert_production_ready_claim_smoke=PASS"
