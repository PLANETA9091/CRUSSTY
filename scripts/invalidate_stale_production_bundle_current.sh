#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CURRENT_BUNDLE="${CURRENT_BUNDLE:-$ROOT/reports/production-500-readiness-bundle-current}"
REPORTS_DIR="${REPORTS_DIR:-$ROOT/reports}"
STAMP="${STAMP:-$(date +%Y%m%d-%H%M%S)}"
REASON="${REASON:-current_bundle_failed_freshness_validation}"
REPORT="${REPORT:-$ROOT/reports/production-500-readiness-bundle-current-invalidation-${STAMP}.txt}"
STALE_BASE="${STALE_BASE:-$ROOT/reports/production-500-readiness-bundle-stale-${STAMP}}"

mkdir -p "$ROOT/reports" "$(dirname "$REPORT")"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

{
  printf 'current_bundle_invalidation_generated_at_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'current_bundle_invalidation_reason=%s\n' "$REASON"
  printf 'current_bundle_path=%s\n' "$CURRENT_BUNDLE"
  printf 'reports_dir=%s\n' "$REPORTS_DIR"
} > "$REPORT"

if [[ ! -e "$CURRENT_BUNDLE" && ! -L "$CURRENT_BUNDLE" ]]; then
  {
    printf 'current_bundle_invalidation_action=already_missing\n'
    printf 'current_bundle_invalidation_pass=true\n'
  } >> "$REPORT"
  cat "$REPORT"
  exit 0
fi

set +e
python3 "$ROOT/scripts/validate_production_readiness_bundle.py" \
  "$CURRENT_BUNDLE" \
  --reports-dir "$REPORTS_DIR" > "$TMP/current-validation.txt" 2>&1
validation_status=$?
set -e

cat "$TMP/current-validation.txt" >> "$REPORT"
if [[ "$validation_status" -eq 0 ]]; then
  {
    printf 'current_bundle_invalidation_action=kept_valid_current_bundle\n'
    printf 'current_bundle_invalidation_pass=true\n'
  } >> "$REPORT"
  cat "$REPORT"
  exit 0
fi

stale_path="$STALE_BASE"
suffix=0
while [[ -e "$stale_path" || -L "$stale_path" ]]; do
  suffix=$((suffix + 1))
  stale_path="${STALE_BASE}-${suffix}"
done

if [[ -L "$CURRENT_BUNDLE" ]]; then
  link_target="$(readlink "$CURRENT_BUNDLE" || true)"
  rm -f "$CURRENT_BUNDLE"
  archived_kind="symlink_removed"
else
  mv -T "$CURRENT_BUNDLE" "$stale_path"
  link_target=""
  archived_kind="moved"
fi

mkdir -p "$CURRENT_BUNDLE"
{
  printf 'production_ready_500_claim=false\n'
  printf 'current_bundle_valid=false\n'
  printf 'current_bundle_invalidated_at_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'current_bundle_invalidation_reason=%s\n' "$REASON"
  printf 'previous_current_bundle_archive=%s\n' "$stale_path"
  printf 'previous_current_bundle_link_target=%s\n' "$link_target"
  printf 'validation_status=%s\n' "$validation_status"
} > "$CURRENT_BUNDLE/INVALIDATED_CURRENT_BUNDLE.txt"
cp "$TMP/current-validation.txt" "$CURRENT_BUNDLE/previous-current-validation.txt"
cat > "$CURRENT_BUNDLE/README.txt" <<'TXT'
This directory is an invalidation tombstone, not a production-readiness bundle.
Run the full production readiness gate to publish a new valid current bundle.
TXT

{
  printf 'current_bundle_invalidation_action=invalidated_stale_current_bundle\n'
  printf 'current_bundle_invalidation_pass=true\n'
  printf 'current_bundle_validation_status=%s\n' "$validation_status"
  printf 'current_bundle_archive_kind=%s\n' "$archived_kind"
  printf 'current_bundle_archive_path=%s\n' "$stale_path"
  printf 'current_bundle_link_target=%s\n' "$link_target"
  printf 'current_bundle_tombstone=%s\n' "$CURRENT_BUNDLE"
} >> "$REPORT"
cat "$REPORT"
