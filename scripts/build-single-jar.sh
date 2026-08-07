#!/usr/bin/env bash
# Build a self-contained kernel jar: drop it in place of server.jar and run.
#   java -Xmx2G -jar crussty-1.21.10.jar --nogui
# The jar bundles: paperclip kernel (unmodified), the native runtime
# (libcrussty_runtime.so), and every module .so — extracted to ./crussty/
# on first boot by dev.dist.launcher.Boot, which then loads the runtime
# via JNI_OnLoad (no -agentpath needed) and starts the kernel.
set -euo pipefail
cd "$(dirname "$0")/.."

V=${1:-1.21.10}
KERNEL=versions/purpur-$V.jar
OUT=dist/crussty-$V.jar

[ -f "$KERNEL" ] || { echo "kernel not found: $KERNEL" >&2; exit 1; }

mkdir -p dist launcher/out
javac -cp "$KERNEL" -d launcher/out launcher/src/main/java/dev/dist/launcher/Boot.java

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT

unzip -q "$KERNEL" -d "$stage"
mkdir -p "$stage/dev/dist/launcher" "$stage/modules"
cp launcher/out/dev/dist/launcher/Boot.class "$stage/dev/dist/launcher/"
cp libcrussty_runtime.so "$stage/"
for m in modules/*/; do
    id=$(basename "$m")
    [ "$id" = "target" ] && continue
    [ -f "$m/cplugin.json" ] || continue
    mkdir -p "$stage/modules/$id"
    cp "$m/cplugin.json" "$stage/modules/$id/"
    for lib in "$m"lib*.so; do
        [ -f "$lib" ] && cp "$lib" "$stage/modules/$id/"
    done
done

manifest=$(mktemp)
cat > "$manifest" <<EOF
Manifest-Version: 1.0
Main-Class: dev.dist.launcher.Boot
Enable-Native-Access: ALL-UNNAMED

EOF

jar cmf "$manifest" "$OUT" -C "$stage" .
echo "built $OUT ($(du -h "$OUT" | cut -f1))"
