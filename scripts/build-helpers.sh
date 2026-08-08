#!/usr/bin/env bash
# Build the Java helper classes that the Rust crates embed via include_bytes!.
#
# The .class artifacts are COMMITTED (see .gitignore exceptions) so the
# workspace builds with zero Java toolchain; this script is for regenerating
# them after a helper source change and for CI verification.
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

echo "== modules/crussty area-map helpers (area-map/build/) =="
rm -rf modules/crussty/area-map/build
javac "${JAVAC_FLAGS[@]}" -d modules/crussty/area-map/build \
  modules/crussty/area-map/ca/spottedleaf/moonrise/common/misc/RuntimeStubs.java \
  modules/crussty/area-map/ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps.java

echo "== modules/crussty improved-noise bridge (noise/build/) =="
rm -rf modules/crussty/noise/build
javac "${JAVAC_FLAGS[@]}" -d modules/crussty/noise/build \
  modules/crussty/noise/stubs/net/minecraft/world/level/levelgen/synth/ImprovedNoise.java \
  modules/crussty/noise/stubs/net/minecraft/world/level/levelgen/synth/PaperNativeImprovedNoise.java \
  modules/crussty/noise/net/minecraft/world/level/levelgen/synth/ImprovedNoiseNativeOps.java

echo "== modules/dist DistKernel (build/) =="
rm -rf modules/dist/build
javac "${JAVAC_FLAGS[@]}" -d modules/dist/build \
  modules/dist/helper/stubs/org/bukkit/BlockData.java \
  modules/dist/helper/stubs/org/bukkit/Bukkit.java \
  modules/dist/helper/stubs/org/bukkit/Chunk.java \
  modules/dist/helper/stubs/org/bukkit/ChunkSnapshot.java \
  modules/dist/helper/stubs/org/bukkit/World.java \
  modules/dist/helper/DistKernel.java

echo
echo "Generated artifacts:"
find cplug-sdk/asm-build modules/crussty/area-map/build modules/crussty/noise/build modules/dist/build -name "*.class" | sort
