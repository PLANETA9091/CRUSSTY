#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JAR="${1:-$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar}"
OUT="${2:-$ROOT/artifacts/optimized-runtime}"
NATIVE_TARGET_DIR="${PAPER_NATIVE_TARGET_DIR:-$ROOT/native/target/release}"
NATIVE_LIB="$NATIVE_TARGET_DIR/libpaper_native_jni.so"
CHUNK_ENCODE_NATIVE_LIB="$NATIVE_TARGET_DIR/libpaper_native_chunk_encode_jni.so"

if [[ ! -f "$JAR" ]]; then
  echo "Missing server jar: $JAR" >&2
  exit 1
fi

JAR="$(readlink -f "$JAR")"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
BUNDLER="$OUT/bundler"
LOG="$OUT/prepare-fast-runtime.log"
JAR_HASH="$(sha256sum "$JAR" | awk '{print $1}')"
if [[ ! -f "$OUT/runtime.jar.sha256" || "$(cat "$OUT/runtime.jar.sha256")" != "$JAR_HASH" ]]; then
  rm -f "$OUT/app-cds.jsa"
  rm -rf "$OUT/remap-classpath" "$OUT/plugin-remaps" "$OUT/reversed-mappings"
fi

rm -rf "$BUNDLER"
rm -rf "$OUT/native"
mkdir -p "$OUT" "$BUNDLER" "$OUT/remap-classpath" "$OUT/plugin-remaps" "$OUT/reversed-mappings"

python3 - "$JAR" "$BUNDLER" "$OUT/classpath.txt" "$LOG" <<'PY'
import hashlib
import pathlib
import sys
import zipfile

jar = pathlib.Path(sys.argv[1])
bundler = pathlib.Path(sys.argv[2])
out = pathlib.Path(sys.argv[3])
log = pathlib.Path(sys.argv[4])

classpath = []
extracted = 0
with zipfile.ZipFile(jar) as zf:
    for subdir in ("versions", "libraries"):
        lines = zf.read(f"META-INF/{subdir}.list").decode("utf-8").splitlines()
        for line in lines:
            if not line.strip():
                continue
            fields = line.split()
            if len(fields) not in (2, 3):
                raise SystemExit(f"Malformed {subdir}.list entry: {line!r}")
            expected_hash = fields[0].lower()
            rel = fields[-1]
            if rel.startswith("/"):
                rel = rel[1:]
            if rel.startswith("minecraft-server"):
                continue
            path = bundler / subdir / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            source_name = f"META-INF/{subdir}/{rel}"
            digest = hashlib.sha256()
            with zf.open(source_name) as source, path.open("wb") as target:
                while True:
                    chunk = source.read(1024 * 1024)
                    if not chunk:
                        break
                    digest.update(chunk)
                    target.write(chunk)
            actual_hash = digest.hexdigest()
            if actual_hash != expected_hash:
                raise SystemExit(f"Hash mismatch for {source_name}: expected {expected_hash}, got {actual_hash}")
            extracted += 1
            classpath.append(str(path))

out.write_text(":".join(classpath) + "\n", encoding="utf-8")
log.write_text(f"Extracted {extracted} runtime jars from {jar}\n", encoding="utf-8")
PY

if [[ -s "$NATIVE_LIB" ]]; then
  mkdir -p "$OUT/native"
  cp "$NATIVE_LIB" "$OUT/native/libpaper_native_jni.so"
  sha256sum "$OUT/native/libpaper_native_jni.so" > "$OUT/native/libpaper_native_jni.so.sha256"
fi
if [[ -s "$CHUNK_ENCODE_NATIVE_LIB" ]]; then
  mkdir -p "$OUT/native"
  cp "$CHUNK_ENCODE_NATIVE_LIB" "$OUT/native/libpaper_native_chunk_encode_jni.so"
  sha256sum "$OUT/native/libpaper_native_chunk_encode_jni.so" > "$OUT/native/libpaper_native_chunk_encode_jni.so.sha256"
fi

SERVER_RUNTIME_JAR="$(tr ':' '\n' < "$OUT/classpath.txt" | head -n 1)"
jar_has_class() {
  [[ -n "$SERVER_RUNTIME_JAR" && -f "$SERVER_RUNTIME_JAR" ]] && jar tf "$SERVER_RUNTIME_JAR" | grep -qx "$1"
}

PAPER_NATIVE_CLIMATE_RTREE_DEFAULT=false
PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT=false
if jar_has_class 'net/minecraft/world/level/biome/PaperNativeClimateRTree.class'; then
  PAPER_NATIVE_CLIMATE_RTREE_DEFAULT=auto
  PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT=true
fi
PAPER_NATIVE_AREA_MAP_HOOK_PRESENT=false
if jar_has_class 'ca/spottedleaf/moonrise/common/misc/PaperNativeAreaMap.class'; then
  PAPER_NATIVE_AREA_MAP_HOOK_PRESENT=true
fi
PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT=false
if jar_has_class 'net/minecraft/world/level/levelgen/synth/PaperNativeImprovedNoise.class'; then
  PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT=true
fi
PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT=false
if jar_has_class 'net/minecraft/world/level/levelgen/synth/PaperNativeNormalNoise.class'; then
  PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT=true
fi
PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT=false
if jar_has_class 'net/minecraft/world/level/levelgen/synth/PaperNativePerlinNoise.class'; then
  PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT=true
fi
PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT=false
if jar_has_class 'net/minecraft/network/protocol/game/PaperNativeChunkPacketEncode.class'; then
  PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT=true
fi

cat > "$OUT/run.sh" <<'RUN'
#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JAVA_BIN="${JAVA_BIN:-java}"
JAVA_OPTS_RAW="${JAVA_OPTS:-}"
NATIVE_DIR="${PAPER_NATIVE_DIR:-$HERE/native}"
PAPER_NATIVE_CLIMATE_RTREE_DEFAULT="__PAPER_NATIVE_CLIMATE_RTREE_DEFAULT__"
PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT="__PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT__"
PAPER_NATIVE_AREA_MAP_HOOK_PRESENT="__PAPER_NATIVE_AREA_MAP_HOOK_PRESENT__"
PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT="__PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT__"
PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT="__PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT__"
PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT="__PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT__"
PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT="__PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT__"
PAPER_NATIVE_CLIMATE_RTREE="${PAPER_NATIVE_CLIMATE_RTREE:-$PAPER_NATIVE_CLIMATE_RTREE_DEFAULT}"
PAPER_NATIVE_AREA_MAP="${PAPER_NATIVE_AREA_MAP:-auto}"
PAPER_NATIVE_IMPROVED_NOISE="${PAPER_NATIVE_IMPROVED_NOISE:-auto}"
PAPER_NATIVE_NORMAL_NOISE="${PAPER_NATIVE_NORMAL_NOISE:-auto}"
PAPER_NATIVE_CHUNK_PACKET_ENCODE="${PAPER_NATIVE_CHUNK_PACKET_ENCODE:-auto}"
PAPER_NATIVE_CHUNK_PACKET_ENCODE_SECTIONS="${PAPER_NATIVE_CHUNK_PACKET_ENCODE_SECTIONS:-$PAPER_NATIVE_CHUNK_PACKET_ENCODE}"
PAPER_NATIVE_CHUNK_PACKET_ENCODE_LIGHT="${PAPER_NATIVE_CHUNK_PACKET_ENCODE_LIGHT:-false}"
PAPER_NATIVE_PERLIN_NOISE_LEGACY_EXPLICIT=false
if [[ -n "${PAPER_NATIVE_PERLIN_NOISE+x}" ]]; then
  PAPER_NATIVE_PERLIN_NOISE_LEGACY_EXPLICIT=true
fi
PAPER_NATIVE_PERLIN_NOISE="${PAPER_NATIVE_PERLIN_NOISE:-false}"
if [[ -z "${PAPER_NATIVE_PERLIN_NOISE_GENERIC+x}" ]]; then
  if [[ "$PAPER_NATIVE_PERLIN_NOISE_LEGACY_EXPLICIT" == "true" ]]; then
    PAPER_NATIVE_PERLIN_NOISE_GENERIC="$PAPER_NATIVE_PERLIN_NOISE"
  else
    PAPER_NATIVE_PERLIN_NOISE_GENERIC=false
  fi
fi
if [[ -z "${PAPER_NATIVE_PERLIN_NOISE_NO_Y_SCALE+x}" ]]; then
  if [[ "$PAPER_NATIVE_PERLIN_NOISE_LEGACY_EXPLICIT" == "true" ]]; then
    PAPER_NATIVE_PERLIN_NOISE_NO_Y_SCALE="$PAPER_NATIVE_PERLIN_NOISE"
  else
    PAPER_NATIVE_PERLIN_NOISE_NO_Y_SCALE=auto
  fi
fi
PAPER_DISABLE_METHOD_PROFILER="${PAPER_DISABLE_METHOD_PROFILER:-false}"
JAVA_OPTS=()
if [[ -n "$JAVA_OPTS_RAW" ]]; then
  # shellcheck disable=SC2206
  JAVA_OPTS=($JAVA_OPTS_RAW)
else
  JAVA_OPTS=(-Xms256m -Xmx2G)
fi
NATIVE_OPTS=()
native_climate_enabled=false
native_area_map_enabled=false
native_improved_noise_enabled=false
native_normal_noise_enabled=false
native_perlin_noise_enabled=false
native_perlin_noise_generic_enabled=false
native_perlin_noise_no_y_scale_enabled=false
native_chunk_packet_encode_enabled=false
native_chunk_packet_encode_sections_enabled=false
native_chunk_packet_encode_light_enabled=false
native_library_available=false
native_chunk_packet_encode_library_available=false
PROFILER_OPTS=()
method_profiler_disabled=false
native_setting_is_true() {
  local value="${1,,}"
  [[ "$value" == "true" || "$value" == "1" || "$value" == "yes" || "$value" == "on" ]]
}
native_setting_enabled() {
  local value="${1,,}"
  local hook_present="$2"
  native_setting_is_true "$value" || [[ "$value" == "auto" && "$hook_present" == "true" ]]
}
require_native_hook() {
  local value="$1"
  local hook_present="$2"
  local class_name="$3"
  local module_name="$4"
  if native_setting_is_true "$value" && [[ "$hook_present" != "true" ]]; then
    echo "[optimized-runtime] requested native $module_name but this artifact does not contain $class_name" >&2
    exit 1
  fi
}
disable_method_profiler_value="${PAPER_DISABLE_METHOD_PROFILER,,}"
if [[ "$disable_method_profiler_value" == "true" || "$disable_method_profiler_value" == "1" || "$disable_method_profiler_value" == "yes" || "$disable_method_profiler_value" == "on" ]]; then
  PROFILER_OPTS+=("-Dpaper.disableMethodProfiler=true")
  method_profiler_disabled=true
fi
if [[ -s "$NATIVE_DIR/libpaper_native_jni.so" ]]; then
  native_library_available=true
  require_native_hook "$PAPER_NATIVE_CLIMATE_RTREE" "$PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT" "net.minecraft.world.level.biome.PaperNativeClimateRTree" "Climate RTree"
  if native_setting_enabled "$PAPER_NATIVE_CLIMATE_RTREE" "$PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT"; then
    NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR" "-Dpaper.nativeClimateRTree=true")
    native_climate_enabled=true
  fi
  require_native_hook "$PAPER_NATIVE_AREA_MAP" "$PAPER_NATIVE_AREA_MAP_HOOK_PRESENT" "ca.spottedleaf.moonrise.common.misc.PaperNativeAreaMap" "AreaMap"
  if native_setting_enabled "$PAPER_NATIVE_AREA_MAP" "$PAPER_NATIVE_AREA_MAP_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativeAreaMap=true")
    native_area_map_enabled=true
  fi
  improved_noise_value="${PAPER_NATIVE_IMPROVED_NOISE:-auto}"
  require_native_hook "$improved_noise_value" "$PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT" "net.minecraft.world.level.levelgen.synth.PaperNativeImprovedNoise" "ImprovedNoise"
  if native_setting_enabled "$improved_noise_value" "$PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativeImprovedNoise=true")
    native_improved_noise_enabled=true
  fi
  normal_noise_value="${PAPER_NATIVE_NORMAL_NOISE:-auto}"
  require_native_hook "$normal_noise_value" "$PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT" "net.minecraft.world.level.levelgen.synth.PaperNativeNormalNoise" "NormalNoise"
  if native_setting_enabled "$normal_noise_value" "$PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativeNormalNoise=true")
    native_normal_noise_enabled=true
  fi
  perlin_noise_generic_value="${PAPER_NATIVE_PERLIN_NOISE_GENERIC,,}"
  require_native_hook "$perlin_noise_generic_value" "$PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT" "net.minecraft.world.level.levelgen.synth.PaperNativePerlinNoise" "PerlinNoise generic"
  if native_setting_enabled "$perlin_noise_generic_value" "$PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativePerlinNoiseGeneric=true")
    native_perlin_noise_enabled=true
    native_perlin_noise_generic_enabled=true
  fi
  perlin_noise_no_y_scale_value="${PAPER_NATIVE_PERLIN_NOISE_NO_Y_SCALE,,}"
  require_native_hook "$perlin_noise_no_y_scale_value" "$PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT" "net.minecraft.world.level.levelgen.synth.PaperNativePerlinNoise" "PerlinNoise no-y-scale"
  if native_setting_enabled "$perlin_noise_no_y_scale_value" "$PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativePerlinNoiseNoYScale=true")
    native_perlin_noise_enabled=true
    native_perlin_noise_no_y_scale_enabled=true
  fi
elif native_setting_is_true "$PAPER_NATIVE_CLIMATE_RTREE" || native_setting_is_true "$PAPER_NATIVE_AREA_MAP" || native_setting_is_true "$PAPER_NATIVE_IMPROVED_NOISE" || native_setting_is_true "$PAPER_NATIVE_NORMAL_NOISE" || native_setting_is_true "$PAPER_NATIVE_PERLIN_NOISE" || native_setting_is_true "$PAPER_NATIVE_PERLIN_NOISE_GENERIC" || native_setting_is_true "$PAPER_NATIVE_PERLIN_NOISE_NO_Y_SCALE"; then
  echo "[optimized-runtime] requested native paper modules but $NATIVE_DIR/libpaper_native_jni.so is missing" >&2
  exit 1
fi
chunk_packet_encode_value="${PAPER_NATIVE_CHUNK_PACKET_ENCODE:-auto}"
chunk_packet_encode_sections_value="${PAPER_NATIVE_CHUNK_PACKET_ENCODE_SECTIONS:-auto}"
chunk_packet_encode_light_value="${PAPER_NATIVE_CHUNK_PACKET_ENCODE_LIGHT:-false}"
if [[ -s "$NATIVE_DIR/libpaper_native_chunk_encode_jni.so" ]]; then
  native_chunk_packet_encode_library_available=true
  require_native_hook "$chunk_packet_encode_value" "$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT" "net.minecraft.network.protocol.game.PaperNativeChunkPacketEncode" "Chunk packet encode"
  require_native_hook "$chunk_packet_encode_sections_value" "$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT" "net.minecraft.network.protocol.game.PaperNativeChunkPacketEncode" "Chunk packet encode sections"
  require_native_hook "$chunk_packet_encode_light_value" "$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT" "net.minecraft.network.protocol.game.PaperNativeChunkPacketEncode" "Chunk packet encode light"
  if native_setting_enabled "$chunk_packet_encode_sections_value" "$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativeChunkPacketEncode.sections=true")
    native_chunk_packet_encode_enabled=true
    native_chunk_packet_encode_sections_enabled=true
  fi
  if native_setting_enabled "$chunk_packet_encode_light_value" "$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT"; then
    if [[ "${#NATIVE_OPTS[@]}" -eq 0 ]]; then
      NATIVE_OPTS+=("-Djava.library.path=$NATIVE_DIR")
    fi
    NATIVE_OPTS+=("-Dpaper.nativeChunkPacketEncode=true")
    native_chunk_packet_encode_enabled=true
    native_chunk_packet_encode_light_enabled=true
  fi
elif native_setting_is_true "$chunk_packet_encode_value" || native_setting_is_true "$chunk_packet_encode_sections_value" || native_setting_is_true "$chunk_packet_encode_light_value"; then
  echo "[optimized-runtime] requested native chunk packet encode but $NATIVE_DIR/libpaper_native_chunk_encode_jni.so is missing" >&2
  exit 1
fi
echo "[optimized-runtime] native_lib_available=$native_library_available native_dir=$NATIVE_DIR native_climate_rtree=$native_climate_enabled native_climate_rtree_hook_present=$PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT native_area_map=$native_area_map_enabled native_area_map_hook_present=$PAPER_NATIVE_AREA_MAP_HOOK_PRESENT native_improved_noise=$native_improved_noise_enabled native_improved_noise_hook_present=$PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT native_normal_noise=$native_normal_noise_enabled native_normal_noise_hook_present=$PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT native_perlin_noise=$native_perlin_noise_enabled native_perlin_noise_hook_present=$PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT native_perlin_noise_generic=$native_perlin_noise_generic_enabled native_perlin_noise_no_y_scale=$native_perlin_noise_no_y_scale_enabled native_chunk_packet_encode=$native_chunk_packet_encode_enabled native_chunk_packet_encode_sections=$native_chunk_packet_encode_sections_enabled native_chunk_packet_encode_light=$native_chunk_packet_encode_light_enabled native_chunk_packet_encode_hook_present=$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT native_chunk_packet_encode_lib_available=$native_chunk_packet_encode_library_available method_profiler_disabled=$method_profiler_disabled" >&2
CDS_OPTS=()
if [[ -f "$HERE/app-cds.jsa" ]]; then
  CDS_OPTS=(-Xshare:auto -XX:SharedArchiveFile="$HERE/app-cds.jsa")
fi

exec "$JAVA_BIN" "${NATIVE_OPTS[@]}" "${JAVA_OPTS[@]}" \
  "${PROFILER_OPTS[@]}" \
  "${CDS_OPTS[@]}" \
  -Dpaper.precomputedRemapClasspathDir="$HERE/remap-classpath" \
  -Dpaper.precomputedPluginRemapsDir="$HERE/plugin-remaps" \
  -Dpaper.precomputedReversedMappingsDir="$HERE/reversed-mappings" \
  -cp "$(cat "$HERE/classpath.txt")" \
  org.bukkit.craftbukkit.Main "$@"
RUN
sed -i \
  -e "s/__PAPER_NATIVE_CLIMATE_RTREE_DEFAULT__/$PAPER_NATIVE_CLIMATE_RTREE_DEFAULT/g" \
  -e "s/__PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT__/$PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT/g" \
  -e "s/__PAPER_NATIVE_AREA_MAP_HOOK_PRESENT__/$PAPER_NATIVE_AREA_MAP_HOOK_PRESENT/g" \
  -e "s/__PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT__/$PAPER_NATIVE_IMPROVED_NOISE_HOOK_PRESENT/g" \
  -e "s/__PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT__/$PAPER_NATIVE_NORMAL_NOISE_HOOK_PRESENT/g" \
  -e "s/__PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT__/$PAPER_NATIVE_PERLIN_NOISE_HOOK_PRESENT/g" \
  -e "s/__PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT__/$PAPER_NATIVE_CHUNK_PACKET_ENCODE_HOOK_PRESENT/g" \
  "$OUT/run.sh"
chmod +x "$OUT/run.sh"
printf '%s\n' "$JAR_HASH" > "$OUT/runtime.jar.sha256"

echo "$OUT"
