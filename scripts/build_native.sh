#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORTS="$ROOT/reports"

cargo test --manifest-path "$ROOT/native/Cargo.toml" --workspace
cargo build --manifest-path "$ROOT/native/Cargo.toml" --release --workspace

mkdir -p "$REPORTS"

MAIN_LIB="$ROOT/native/target/release/libpaper_native_jni.so"
CHUNK_ENCODE_LIB="$ROOT/native/target/release/libpaper_native_chunk_encode_jni.so"

if [[ ! -s "$MAIN_LIB" ]]; then
  echo "native library not found: $MAIN_LIB" >&2
  exit 1
fi
if [[ ! -s "$CHUNK_ENCODE_LIB" ]]; then
  echo "native library not found: $CHUNK_ENCODE_LIB" >&2
  exit 1
fi

sha256sum "$MAIN_LIB" > "$REPORTS/paper-native-jni.sha256"
sha256sum "$CHUNK_ENCODE_LIB" > "$REPORTS/paper-native-chunk-encode-jni.sha256"
cat "$REPORTS/paper-native-jni.sha256"
cat "$REPORTS/paper-native-chunk-encode-jni.sha256"
