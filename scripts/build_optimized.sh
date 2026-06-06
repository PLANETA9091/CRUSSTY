#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PAPER="$ROOT/upstream/Paper"

cd "$PAPER"
GRADLE_ARGS=(--no-daemon --no-parallel --no-configuration-cache "-Dorg.gradle.jvmargs=-Xms1g -Xmx4g -XX:MaxMetaspaceSize=1g -XX:+HeapDumpOnOutOfMemoryError")

# Codebook refuses to overwrite its intermediate jar after an interrupted or
# failed paperweight run, even with --rerun-tasks.
rm -rf "$PAPER/paper-server/build/tmp/macheRemapJar"
rm -f "$PAPER/paper-server/.gradle/caches/paperweight/taskCache/codebook-minecraft.jar"
DECOMPILE_JAR="$PAPER/paper-server/.gradle/caches/paperweight/taskCache/decompileJar.jar"
if [[ -f "$DECOMPILE_JAR" ]] && command -v zip >/dev/null 2>&1 && ! zip -T "$DECOMPILE_JAR" >/dev/null 2>&1; then
  rm -f "$DECOMPILE_JAR"
fi

# Paperweight can leave stale compiled Minecraft classes when source patches are
# reapplied through Gradle's cached task graph. Force the compile and bundler
# tasks so the exported artifact always matches the current patch layer.
./gradlew :paper-server:compileJava :paper-server:createMojmapBundlerJar --rerun-tasks -x :paper-server:indexLibraryFiles "${GRADLE_ARGS[@]}"

mkdir -p "$ROOT/artifacts"
cp "$PAPER/paper-server/build/libs/paper-bundler-1.21.10-R0.1-SNAPSHOT-mojmap.jar" \
  "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"
sha256sum "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" \
  > "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar.sha256"

if [[ "${BUILD_NATIVE:-true}" != "false" && "${BUILD_NATIVE:-true}" != "0" ]]; then
  "$ROOT/scripts/build_native.sh"
fi

"$ROOT/scripts/prepare_fast_runtime.sh" \
  "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" \
  "$ROOT/artifacts/optimized-runtime"

if [[ "${MC_EULA_AGREE:-}" == "true" && -f "$ROOT/plugins/matrix/CompatProbe-0.1.0.jar" ]]; then
  "$ROOT/scripts/precompute_server_remap.sh" \
    "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" \
    "$ROOT/artifacts/optimized-runtime/remap-classpath"
  "$ROOT/scripts/precompute_plugin_remaps.sh" \
    "$ROOT/artifacts/optimized-runtime/run.sh" \
    "$ROOT/artifacts/optimized-runtime/plugin-remaps"
  "$ROOT/scripts/generate_app_cds.sh" \
    "$ROOT/artifacts/optimized-runtime/run.sh" \
    "$ROOT/artifacts/optimized-runtime"
fi

python3 "$ROOT/scripts/update_artifact_reports.py"
REPORT="${REPORT:-$ROOT/reports/artifact-source-freshness-current.txt}" \
  "$ROOT/scripts/check_artifact_source_freshness.sh"
