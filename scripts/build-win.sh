#!/usr/bin/env bash
# Cross-compile the v2 artifacts for Windows (x86_64-pc-windows-msvc):
#   agent libcrussty_runtime.so -> crussty_runtime.dll
#   modules lib<id>.so     -> <id>.dll
# Run from v2/ (or any dir; paths are relative to this script).
# Requires: rustup target add x86_64-pc-windows-msvc
#           (MSVC toolchain with `link.exe` on PATH — e.g. x64 Native Tools prompt).
set -euo pipefail
cd "$(dirname "$0")"

TARGET=x86_64-pc-windows-msvc
CARGO="${CARGO:-cargo}"
TOOLCHAIN="${RUSTUP_TOOLCHAIN:-+stable}"

rustup target list --installed | grep -q "$TARGET" || {
    echo "missing target $TARGET — run: rustup target add $TARGET" >&2
    exit 1
}

$CARGO "$TOOLCHAIN" build --release --target "$TARGET" --manifest-path agent/Cargo.toml
$CARGO "$TOOLCHAIN" build --release --target "$TARGET" --manifest-path modules/crussty/Cargo.toml
$CARGO "$TOOLCHAIN" build --release --target "$TARGET" --manifest-path modules/dist/Cargo.toml
$CARGO "$TOOLCHAIN" build --release --target "$TARGET" --manifest-path modules/hello/Cargo.toml

cp "agent/target/$TARGET/release/crussty_runtime.dll" ./crussty_runtime.dll
cp "modules/crussty/target/$TARGET/release/crussty.dll" modules/crussty/crussty.dll
cp "modules/dist/target/$TARGET/release/dist.dll" modules/dist/dist.dll
cp "modules/hello/target/$TARGET/release/hello.dll" modules/hello/hello.dll

# The Crussty CE native libs are also Windows .dll builds — drop them next to
# the module like on Linux (see modules/crussty/native/).
mkdir -p modules/crussty/native
for name in paper_native_jni paper_native_chunk_encode_jni; do
    src="crussty-native-windows/$name.dll"
    [ -f "$src" ] && cp "$src" "modules/crussty/native/$name.dll"
done

echo "Windows artifacts copied. Boot with: run.bat"
