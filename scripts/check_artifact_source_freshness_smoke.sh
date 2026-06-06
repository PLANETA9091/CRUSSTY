#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PROJECT="$TMP/project"
PAPER_SRC="$PROJECT/paper/src"
NATIVE_SRC="$PROJECT/native"
PAPER_ART="$PROJECT/artifacts/optimized-paper.jar"
NATIVE_ART="$PROJECT/artifacts/native/libpaper_native_jni.so"
CHUNK_ENCODE_NATIVE_ART="$PROJECT/artifacts/native/libpaper_native_chunk_encode_jni.so"

reset_fixture() {
  rm -rf "$PROJECT"
  mkdir -p "$PAPER_SRC" "$NATIVE_SRC/src" "$(dirname "$PAPER_ART")" "$(dirname "$NATIVE_ART")"
  printf 'class Smoke {}\n' > "$PAPER_SRC/Smoke.java"
  printf '[package]\nname = "smoke"\nversion = "0.1.0"\n' > "$NATIVE_SRC/Cargo.toml"
  printf 'pub fn smoke() {}\n' > "$NATIVE_SRC/src/lib.rs"
  printf 'paper artifact\n' > "$PAPER_ART"
  printf 'native artifact\n' > "$NATIVE_ART"
  printf 'native chunk encode artifact\n' > "$CHUNK_ENCODE_NATIVE_ART"
  touch -d '2026-01-01 00:00:00 UTC' "$PAPER_SRC/Smoke.java" "$NATIVE_SRC/Cargo.toml" "$NATIVE_SRC/src/lib.rs"
  touch -d '2026-01-01 00:00:10 UTC' "$PAPER_ART" "$NATIVE_ART" "$CHUNK_ENCODE_NATIVE_ART"
}

run_check() {
  local output="${1:?output is required}"
  shift || true
  env \
    ARTIFACT_SOURCE_FRESHNESS_ROOT="$PROJECT" \
    PAPER_ARTIFACT="$PAPER_ART" \
    NATIVE_ARTIFACTS="${NATIVE_ARTIFACTS_OVERRIDE:-$NATIVE_ART:$CHUNK_ENCODE_NATIVE_ART}" \
    PAPER_SOURCE_PATHS="${PAPER_SOURCE_PATHS_OVERRIDE:-$PAPER_SRC}" \
    NATIVE_SOURCE_PATHS="${NATIVE_SOURCE_PATHS_OVERRIDE:-$NATIVE_SRC}" \
    "$@" \
    "$ROOT/scripts/check_artifact_source_freshness.sh" > "$output" 2>&1
}

reset_fixture
run_check "$TMP/pass.out"
rg -q '^artifact_source_freshness=PASS$' "$TMP/pass.out"
rg -q '^artifact_source_freshness_paper_pass=true$' "$TMP/pass.out"
rg -q '^artifact_source_freshness_native_pass=true$' "$TMP/pass.out"
rg -q '^artifact_source_freshness_native_2_pass=true$' "$TMP/pass.out"

reset_fixture
touch -d '2026-01-01 00:00:20 UTC' "$PAPER_SRC/Smoke.java"
if run_check "$TMP/stale-paper.out"; then
  echo "Expected stale paper source to fail freshness check." >&2
  exit 1
fi
rg -q '^artifact_source_freshness=FAIL$' "$TMP/stale-paper.out"
rg -q 'artifact_source_freshness_failure=paper source newer than artifact' "$TMP/stale-paper.out"

reset_fixture
touch -d '2026-01-01 00:00:20 UTC' "$NATIVE_SRC/src/lib.rs"
if run_check "$TMP/stale-native.out"; then
  echo "Expected stale native source to fail freshness check." >&2
  exit 1
fi
rg -q '^artifact_source_freshness=FAIL$' "$TMP/stale-native.out"
rg -q 'artifact_source_freshness_failure=native source newer than artifact' "$TMP/stale-native.out"

reset_fixture
PAPER_SOURCE_PATHS_OVERRIDE="$PROJECT/missing-src"
if run_check "$TMP/missing-source.out"; then
  echo "Expected missing source path to fail freshness check." >&2
  exit 1
fi
rg -q 'artifact_source_freshness_failure=paper source path missing' "$TMP/missing-source.out"
unset PAPER_SOURCE_PATHS_OVERRIDE

reset_fixture
: > "$PAPER_ART"
if run_check "$TMP/empty-artifact.out"; then
  echo "Expected empty artifact to fail freshness check." >&2
  exit 1
fi
rg -q 'artifact_source_freshness_failure=paper artifact is empty' "$TMP/empty-artifact.out"

reset_fixture
mkdir -p "$TMP/outside"
printf 'class Escape {}\n' > "$TMP/outside/Escape.java"
ln -s "$TMP/outside" "$PROJECT/escaped-src"
PAPER_SOURCE_PATHS_OVERRIDE="$PROJECT/escaped-src"
if run_check "$TMP/escaped-source.out"; then
  echo "Expected escaped source path to fail freshness check." >&2
  exit 1
fi
rg -q 'artifact_source_freshness_failure=paper source path escapes root' "$TMP/escaped-source.out"
unset PAPER_SOURCE_PATHS_OVERRIDE

reset_fixture
REPORT="$TMP/freshness.report" run_check "$TMP/report.out"
rg -q '^artifact_source_freshness=PASS$' "$TMP/report.out"
rg -q '^artifact_source_freshness=PASS$' "$TMP/freshness.report"

echo "check_artifact_source_freshness_smoke=PASS"
