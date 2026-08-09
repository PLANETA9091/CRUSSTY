#!/usr/bin/env bash
# Cross-compile the v2 artifacts for Windows (x86_64-pc-windows-msvc):
#   runtime libcrussty_runtime.so -> crussty_runtime.dll
#   modules lib<id>.so     -> <id>.dll
# Modules are built from their own repositories (convention: c-<id>),
# vendored cplug-abi/cplug-sdk so they build standalone; clones land in
# $MODULE_CLONE_DIR (default: ./modules-src — gitignored).
# Requires: rustup target add x86_64-pc-windows-msvc
#           (MSVC toolchain with `link.exe` on PATH — e.g. x64 Native Tools prompt).
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET=x86_64-pc-windows-msvc
CARGO="${CARGO:-cargo}"
TOOLCHAIN="${RUSTUP_TOOLCHAIN:-+stable}"
CLONE_DIR="${MODULE_CLONE_DIR:-modules-src}"
mkdir -p dist "$CLONE_DIR"

rustup target list --installed | grep -q "$TARGET" || {
    echo "missing target $TARGET — run: rustup target add $TARGET" >&2
    exit 1
}

clone_module() { # repo dir
    local repo="$1" dir="$2"
    [ -d "$CLONE_DIR/$dir/.git" ] || git clone --depth 1 \
        "https://github.com/PLANETA9091/$repo.git" "$CLONE_DIR/$dir"
}

$CARGO "$TOOLCHAIN" build --release --target "$TARGET" --manifest-path runtime/Cargo.toml

MODULES=(
    "c-crussty:crussty"
    "c-dist:dist"
    "c-hello:hello"
)
for spec in "${MODULES[@]}"; do
    repo="${spec%%:*}"
    id="${spec##*:}"
    clone_module "$repo" "$id"
    $CARGO "$TOOLCHAIN" build --release --target "$TARGET" \
        --manifest-path "$CLONE_DIR/$id/Cargo.toml"
    mkdir -p "modules/$id"
    cp "$CLONE_DIR/$id/target/$TARGET/release/$id.dll" "modules/$id/$id.dll"
done

# The Crussty CE native libs are also Windows .dll builds — drop them next to
# the module like on Linux (see c-crussty/native/).
mkdir -p modules/crussty/native
for name in paper_native_jni paper_native_chunk_encode_jni; do
    src="crussty-native-windows/$name.dll"
    [ -f "$src" ] && cp "$src" "modules/crussty/native/$name.dll"
done

echo "Windows artifacts copied. Boot with: run.bat"