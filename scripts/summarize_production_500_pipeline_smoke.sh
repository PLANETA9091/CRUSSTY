#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

REPORT="$TMP/production-500-pipeline-summary.txt"

python3 "$ROOT/scripts/summarize_production_500_pipeline.py" \
  --reports-dir "$ROOT/reports" \
  --bundle-dir "$ROOT/reports/production-500-readiness-bundle-current" \
  --report "$REPORT" \
  >/dev/null

rg -q '^p500_pipeline_summary_profile=production-500-control-plane$' "$REPORT"
rg -q '^p500_pipeline_summary_complete=true$' "$REPORT"
rg -q '^refresh_artifacts=false$' "$REPORT"
rg -q '^\[go/nogo\]$' "$REPORT"
rg -q '^\[artifact freshness\]$' "$REPORT"
rg -q '^\[current artifact hashes\]$' "$REPORT"
rg -q '^\[evidence presence\]$' "$REPORT"
rg -q '^\[bundle validation\]$' "$REPORT"
rg -q '^\[claim assertion\]$' "$REPORT"
rg -q '^\[control plane verdict\]$' "$REPORT"
rg -q '^go_nogo_pass=false$' "$REPORT"
rg -q '^go_nogo_reason=strict_foreign_process_present$' "$REPORT"
rg -q '^production_readiness_next_action=stop_foreign_process$' "$REPORT"
rg -q '^artifact_source_freshness_pass=true$' "$REPORT"
rg -q '^sha256sum_pass=true$' "$REPORT"
rg -q '^readiness_gate_readiness_gate_pass=' "$REPORT"
rg -q '^readiness_gate_plugin_matrix_pass=' "$REPORT"
rg -q '^readiness_gate_restart_recovery_pass=' "$REPORT"
rg -q '^readiness_gate_forced_ticket_persistence_pass=' "$REPORT"
rg -q '^repeat_quorum_repeat_quorum_pass=true$' "$REPORT"
rg -q '^plugin_matrix_plugin_matrix_log=' "$REPORT"
rg -q '^restart_recovery_restart_recovery_log=' "$REPORT"
rg -q '^forced_ticket_forced_ticket_persistence=PASS$' "$REPORT"
rg -q '^bundle_validation_pass=' "$REPORT"
rg -q '^claim_assertion_pass=' "$REPORT"

python3 - "$REPORT" "$ROOT/reports/artifacts.json" <<'PY'
from __future__ import annotations

import json
from pathlib import Path
import sys

report = Path(sys.argv[1]).read_text(encoding="utf-8")
artifacts = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))

expected = {
    "current_optimized_artifact_sha256": artifacts["optimized"]["sha256"],
    "current_optimized_runtime_run_sh_sha256": artifacts["optimized_runtime"]["run_sh"]["sha256"],
    "current_optimized_runtime_native_library_sha256": artifacts["optimized_runtime"]["native_library"]["sha256"],
}
for key, value in expected.items():
    needle = f"{key}={value}"
    if needle not in report:
        raise SystemExit(f"missing {needle}")
PY

echo "summarize_production_500_pipeline_smoke=PASS"
