#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/upstream/Paper/paper-api/build/libs/paper-api-1.21.10-R0.1-SNAPSHOT.jar"
DEPS="$(find "$HOME/.gradle/caches/modules-2/files-2.1" -type f -name '*.jar' | sort | paste -sd ':' -)"

LIB_SRC="$ROOT/test-plugins/library-probe-lib/src/main/java"
LIB_OUT="$ROOT/target/library-probe-lib-classes"
LIB_JAR="$ROOT/plugins/matrix-libraries/library-probe-dep.jar"

PLUGIN_SRC="$ROOT/test-plugins/library-probe/src/main/java"
PLUGIN_OUT="$ROOT/target/library-probe-classes"
PLUGIN_JAR="$ROOT/plugins/matrix/LibraryProbe-0.1.0.jar"

if [[ ! -f "$API" ]]; then
  echo "Paper API jar missing: $API" >&2
  echo "Run scripts/build_optimized.sh first." >&2
  exit 1
fi

mkdir -p "$LIB_OUT" "$PLUGIN_OUT" "$(dirname "$LIB_JAR")" "$(dirname "$PLUGIN_JAR")"
rm -rf "$LIB_OUT"/* "$PLUGIN_OUT"/*

javac -d "$LIB_OUT" $(find "$LIB_SRC" -name '*.java' | sort)
(cd "$LIB_OUT" && jar cf "$LIB_JAR" .)

javac -cp "$API:$LIB_JAR:$DEPS" -d "$PLUGIN_OUT" $(find "$PLUGIN_SRC" -name '*.java' | sort)
cp "$ROOT/test-plugins/library-probe/paper-plugin.yml" "$PLUGIN_OUT/paper-plugin.yml"
(cd "$PLUGIN_OUT" && jar cf "$PLUGIN_JAR" .)

sha256sum "$LIB_JAR" "$PLUGIN_JAR"
