#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/native-chunk-packet-encode/.classes"
REPORT="$ROOT/reports/native-chunk-packet-encode-bench.txt"
LIB_DIR="$ROOT/native/target/release"
LIB="$LIB_DIR/libpaper_native_chunk_encode_jni.so"
JAVA_ARGS=(
  "-Dnative.chunkEncode.warmup=${CHUNK_ENCODE_WARMUP:-200}"
  "-Dnative.chunkEncode.iterations=${CHUNK_ENCODE_ITERATIONS:-2000}"
)

if [[ "${SKIP_NATIVE_BUILD:-0}" != "1" ]] && { [[ ! -f "${LIB}" ]] || find "${ROOT}/native" -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -newer "${LIB}" -print -quit | grep -q .; }; then
  "$ROOT/scripts/build_native.sh" >/dev/null
fi

mkdir -p "$OUT" "$ROOT/reports"

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_ARGS+=(${JAVA_PROPS})
fi

javac -d "$OUT" \
  "$ROOT/bench/native-chunk-packet-encode/PaperNativeChunkPacketEncode.java" \
  "$ROOT/bench/native-chunk-packet-encode/NativeChunkPacketEncodeHarness.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "native_lib=$LIB"
  echo "command=java ${JAVA_ARGS[*]} -Djava.library.path=\"$LIB_DIR\" -cp \"$OUT\" net.minecraft.network.protocol.game.NativeChunkPacketEncodeHarness"
  java "${JAVA_ARGS[@]}" -Djava.library.path="$LIB_DIR" -cp "$OUT" net.minecraft.network.protocol.game.NativeChunkPacketEncodeHarness
  echo "equivalence=PASS"
} | tee "$REPORT"
