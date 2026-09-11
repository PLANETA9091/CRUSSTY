#!/usr/bin/env bash
# Build a self-contained kernel jar: drop it in place of server.jar and run.
#   java -Xmx2G -jar crussty-1.21.10.jar --nogui
# The jar bundles: paperclip kernel (unmodified), the native runtime
# (libcrussty_runtime.so), and every published module bundle from the catalog
# (module.json + lib*.so + native deps) — extracted to ./crussty/ on first
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
#
# The catalog INDEX (catalog.json) lives in the private PLANETA9091/
# crussty-catalog repo — an anonymous fetch 404s (TASK-168: this silently
# red-ed e2e stage 7 on every commit since it landed). When CATALOG_TOKEN is
# set (a PAT with read access to that repo; wired in e2e.yml) it authenticates
# the index fetch; module release assets themselves are public. An
# unreachable index exits 42 = "catalog unavailable", which e2e.sh reports as
# an honest stage-7 SKIP instead of a red run.
CATALOG=https://raw.githubusercontent.com/PLANETA9091/crussty-catalog/main/catalog.json
if [ -n "${CATALOG_TOKEN:-}" ]; then
    AUTH=(-H "Authorization: Bearer $CATALOG_TOKEN")
else
    AUTH=()
fi
curl -fsSL --retry 3 --retry-delay 2 "${AUTH[@]}" "$CATALOG" -o "$stage/catalog.json" || exit 42
python3 - "$stage/catalog.json" "$stage/modules" <<'PY'
import io, json, os, subprocess, sys, tarfile
entries = json.load(open(sys.argv[1]))
mods = sys.argv[2]
os.makedirs(mods, exist_ok=True)
tok = os.environ.get("CATALOG_TOKEN", "")
auth = ["-H", f"Authorization: Bearer {tok}"] if tok else []
for e in entries:
    if e.get("platform") != "linux-x64":
        continue
    print("fetching", e["id"], e["url"])
    data = subprocess.check_output(["curl", "-fsSL", *auth, e["url"]])
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
