#!/usr/bin/env bash
# Compile the c-menus Java bridge (dev.crussty.cmenus.Bridge) into
# bridge-build/ as a committed artifact (same convention as the platform's
# cplug-sdk/asm-build). Requires: javac (JDK 21+). The bridge is compiled
# against the minimal stub API in bridge/stubs (descriptors identical to the
# real Bukkit/JOML API), so NO external jars are needed — fully offline and
# reproducible.
#
#   JAVAC  path to javac  (default: $JAVA_HOME/bin/javac or PATH)
#
# The classfile is embedded into the Rust .so via include_bytes!, so the
# committed artifact MUST stay in sync with bridge/src — CI recompiles and
# fails when it drifts (git diff --exit-code bridge-build).
set -euo pipefail

DIR="$(cd "$(dirname "$0")/.." && pwd)"
if [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/javac" ]; then
    JAVAC="$JAVA_HOME/bin/javac"
else
    JAVAC="${JAVAC:-javac}"
fi
OUT="$DIR/bridge-build"
STUBS="$DIR/bridge/stubs"
SRC="$DIR/bridge/src/dev/crussty/cmenus/Bridge.java"

mkdir -p "$OUT"
rm -rf "$OUT/dev"
"$JAVAC" --release 21 -nowarn -implicit:none \
    -sourcepath "$STUBS" \
    -d "$OUT" \
    "$SRC"

echo "bridge compiled -> $OUT/dev/crussty/cmenus/Bridge.class"
