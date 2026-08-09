#!/usr/bin/env bash
# Build a self-contained kernel jar: drop it in place of server.jar and run.
#   java -Xmx2G -jar crussty-1.21.10.jar --nogui
# The jar bundles: paperclip kernel (unmodified), the native runtime
# (libcrussty_runtime.so), and every published module bundle from the catalog
# (cplugin.json + lib*.so + native deps) — extracted to ./crussty/ on first
# boot by dev.dist.launcher.Boot, which then loads the runtime via JNI_OnLoad
# (no -agentpath needed) and starts the kernel.
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

# Modules are published as release bundles (see PLANETA9091/crussty-catalog);
# fetch every linux-x64 entry and unpack it into the jar.
CATALOG=https://raw.githubusercontent.com/PLANETA9091/crussty-catalog/main/catalog.json
curl -fsSL --retry 3 --retry-delay 2 "$CATALOG" -o "$stage/catalog.json"
python3 - "$stage/catalog.json" "$stage/modules" <<'PY'
import io, json, os, subprocess, sys, tarfile
entries = json.load(open(sys.argv[1]))
mods = sys.argv[2]
os.makedirs(mods, exist_ok=True)
for e in entries:
    if e.get("platform") != "linux-x64":
        continue
    print("fetching", e["id"], e["url"])
    data = subprocess.check_output(["curl", "-fsSL", e["url"]])
    tarfile.open(fileobj=io.BytesIO(data), mode="r:gz").extractall(mods)
PY

manifest=$(mktemp)
cat > "$manifest" <<EOF
Manifest-Version: 1.0
Main-Class: dev.dist.launcher.Boot
Enable-Native-Access: ALL-UNNAMED

EOF

jar cmf "$manifest" "$OUT" -C "$stage" .
echo "built $OUT ($(du -h "$OUT" | cut -f1))"
