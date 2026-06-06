#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/upstream/Paper/paper-api/build/libs/paper-api-1.21.10-R0.1-SNAPSHOT.jar"
DEPS="$(find "$HOME/.gradle/caches/modules-2/files-2.1" -type f -name '*.jar' | sort | paste -sd ':' -)"
SRC_DIR="$ROOT/test-plugins/compat-probe/src/main/java"
OUT_DIR="$ROOT/target/compat-probe-classes"
JAR="$ROOT/plugins/matrix/CompatProbe-0.1.0.jar"
JAVAC_ARGS="$(mktemp)"
trap 'rm -f "$JAVAC_ARGS"' EXIT

if [[ ! -f "$API" ]]; then
  echo "Paper API jar missing: $API" >&2
  echo "Run scripts/build_optimized.sh first." >&2
  exit 1
fi

mkdir -p "$OUT_DIR" "$(dirname "$JAR")"
rm -rf "$OUT_DIR"/*
{
  printf -- '-cp\n%s\n' "$API:$DEPS"
  printf -- '-d\n%s\n' "$OUT_DIR"
  find "$SRC_DIR" -name '*.java' | sort
} >"$JAVAC_ARGS"
javac @"$JAVAC_ARGS"
cp "$ROOT/test-plugins/compat-probe/plugin.yml" "$OUT_DIR/plugin.yml"
(cd "$OUT_DIR" && jar cf "$JAR" .)
sha256sum "$JAR"
