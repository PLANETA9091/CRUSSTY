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

python3 "$ROOT/scripts/publish_production_ready_claim.py" \
  "$TMP/bundle" \
  --reports-dir "$REPORTS_DIR" \
  --out-prefix "$TMP/claim" \
  --verdict-report "$TMP/verdict.txt" > "$TMP/publish.out"

rg -q '^claim_publication_pass=true$' "$TMP/publish.out"
test "$(wc -l < "$TMP/claim.txt")" -eq 2
sed -n '1p' "$TMP/claim.txt" | rg -q '^production-ready для измеренного 500 bots / 32 view / 32 simulation / creative block'
sed -n '2p' "$TMP/claim.txt" | rg -q '^Это не claim про полный Rust Paper runtime, unlimited plugins, real-player gameplay или multi-hour soak\.$'
rg -q '^# Production 500 Claim$' "$TMP/claim.md"
rg -q '^claim_assertion_pass=true$' "$TMP/verdict.txt"
python3 - "$TMP/claim.json" <<'PY'
from pathlib import Path
import json
import sys

data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert data["schema"] == "production-500-claim-publication/v1"
assert data["claim_assertion_pass"] is True
assert data["bundle_validation_pass"] is True
assert len(data["optimized_runtime_run_sh_sha256"]) == 64
assert len(data["optimized_runtime_native_library_sha256"]) == 64
assert len(data["optimized_runtime_chunk_encode_native_library_sha256"]) == 64
assert data["current_optimized_runtime_run_sh_sha256"] == data["optimized_runtime_run_sh_sha256"]
assert data["current_optimized_runtime_native_library_sha256"] == data["optimized_runtime_native_library_sha256"]
assert data["current_optimized_runtime_chunk_encode_native_library_sha256"] == data["optimized_runtime_chunk_encode_native_library_sha256"]
assert data["repeat_passes"] >= 3
assert data["measured_load_surface"]["cold"]["tps1_min"] >= 18.0
assert data["measured_load_surface"]["warm"]["tps1_min"] >= 18.0
PY

cp -a "$TMP/bundle" "$TMP/bad-bundle"
python3 - "$TMP/bad-bundle/bundle.json" <<'PY'
from pathlib import Path
import json
import sys

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
data["allowed_claim"] = "Production-ready for everything."
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if python3 "$ROOT/scripts/publish_production_ready_claim.py" \
  "$TMP/bad-bundle" \
  --reports-dir "$REPORTS_DIR" \
  --out-prefix "$TMP/bad-claim" \
  --verdict-report "$TMP/bad-verdict.txt" > "$TMP/bad-publish.out" 2>&1; then
  echo "Expected broadened allowed_claim to fail publication." >&2
  exit 1
fi
rg -q '^claim_assertion_pass=false$' "$TMP/bad-publish.out"
rg -q 'claim_publication_failure=claim assertion failed' "$TMP/bad-publish.out"
test ! -e "$TMP/bad-claim.txt"
test ! -e "$TMP/bad-claim.md"
test ! -e "$TMP/bad-claim.json"

printf 'stale txt\n' > "$TMP/stale-claim.txt"
printf 'stale md\n' > "$TMP/stale-claim.md"
printf '{"stale": true}\n' > "$TMP/stale-claim.json"
if python3 "$ROOT/scripts/publish_production_ready_claim.py" \
  "$TMP/bad-bundle" \
  --reports-dir "$REPORTS_DIR" \
  --out-prefix "$TMP/stale-claim" \
  --verdict-report "$TMP/stale-verdict.txt" > "$TMP/stale-publish.out" 2>&1; then
  echo "Expected stale output cleanup publication to fail." >&2
  exit 1
fi
rg -q '^claim_assertion_pass=false$' "$TMP/stale-publish.out"
test -f "$TMP/stale-verdict.txt"
test ! -e "$TMP/stale-claim.txt"
test ! -e "$TMP/stale-claim.md"
test ! -e "$TMP/stale-claim.json"

echo "publish_production_ready_claim_smoke=PASS"
