#!/usr/bin/env bash
# Build the Java helper classes that the Rust crates embed via include_bytes!.
#
# The .class artifacts are COMMITTED (see .gitignore exceptions) so the
# workspace builds with zero Java toolchain; this script is for regenerating
# them after a helper source change and for CI verification.
#
# Module Java helpers (crussty area-map/noise, dist DistKernel) now live in
# the modules' own repositories (PLANETA9091/crussty-module, PLANETA9091/dist)
# with their .class artifacts committed there — rebuild them from those repos.
#
# Requirements: javac (JDK 8+; --release 8 target works on any modern JDK).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

JAVAC_FLAGS=(-encoding UTF-8 --release 8 -g -nowarn)

ASM_JAR="cplug-sdk/asm-lib/asm-9.7.1.jar"
if [[ ! -f "$ASM_JAR" ]]; then
    echo "== downloading ASM (SdkAsmHelper dependency) =="
    mkdir -p cplug-sdk/asm-lib
    curl -fsSL -o "$ASM_JAR" \
      https://repo1.maven.org/maven2/org/ow2/asm/asm/9.7.1/asm-9.7.1.jar
fi

echo "== cplug-sdk SdkAsmHelper (asm-build/) =="
mkdir -p cplug-sdk/asm-build
javac "${JAVAC_FLAGS[@]}" -cp "$ASM_JAR" -d cplug-sdk/asm-build \
  cplug-sdk/asm-src/dev/dist/SdkAsmHelper.java

echo "== runtime transform hook classes (runtime/build/hooks/) =="
rm -rf runtime/build/hooks
javac "${JAVAC_FLAGS[@]}" -d runtime/build/hooks \
  runtime/hook-src/dev/crussty/hooks/SchedulerHooks.java \
  runtime/hook-src/dev/crussty/hooks/NetHooks.java \
  runtime/hook-src/dev/crussty/hooks/TickHook.java \
  runtime/hook-src/dev/crussty/hooks/StorageHooks.java

echo
echo "Generated artifacts:"
find cplug-sdk/asm-build runtime/build/hooks -name "*.class" | sort
