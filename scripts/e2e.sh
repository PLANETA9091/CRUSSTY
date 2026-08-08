#!/usr/bin/env bash
# E2E smoke test: boot a real Purpur server with the launcher (run.sh) and the
# cplug modules, and verify the platform markers appear.
#
#  1. download the kernel:   purpur-<PURPUR_VERSION> (cached in versions/)
#  2. build:                 native runtime, launcher, bundled modules
#  3. boot the server:       ./run.sh (launcher->java -jar purpur) with modules/
#  4. markers:               "[crussty-runtime] pipeline ready: ..."
#                            "hello from native c-plugin"
#                            (optional) "[crussty-plugin] native surface live"
#  5. stop the server cleanly; exit 0 on success, 1 on any failure.
#
# Requirements: java 21+ (javac/jar), cargo, curl, python3, setsid (util-linux).
# Env overrides: PURPUR_VERSION (1.21.10), SERVER_PORT (25566),
#   TIMEOUT_SEC (boot wait, 420), MODULES (ids to build+install:
#   "hello dist crussty").
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

PURPUR_VERSION="${PURPUR_VERSION:-1.21.10}"
SERVER_PORT="${SERVER_PORT:-25566}"
TIMEOUT_SEC="${TIMEOUT_SEC:-420}"
POLL_SEC="${POLL_SEC:-2}"
MODULES_DEFAULT="hello dist crussty"
MODULES="${MODULES:-$MODULES_DEFAULT}"

SERIAL_DIR="logs/e2e.$$"
SERVER_LOG="$SERIAL_DIR/server.log"
LAUNCHER_LOG="$SERIAL_DIR/launcher.out"
KERNEL="versions/purpur-$PURPUR_VERSION.jar"

log()  { printf '%s\n' "[e2e] $*"; }
fail() { printf '%s\n' "[e2e] FAIL: $*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || fail "missing tool: $1"; }

if command -v setsid >/dev/null 2>&1; then SETSID="setsid"; else SETSID=""; fi

mkdir -p "$SERIAL_DIR" versions

stage() { printf '\n%s\n' "=== STAGE $1 ==="; }

# ---------------------------------------------------------------- 1. toolchain
stage "1. toolchain"
require java; require javac; require jar; require curl; require cargo
JMAJOR="$(java -version 2>&1 | head -1 | sed 's/^[^0-9]*\([0-9][0-9]*\).*/\1/')"
if [ "${JMAJOR:-0}" -lt 21 ] && [ "${CRUSSTY_ALLOW_OLD_JAVA:-0}" != "1" ]; then
    fail "Java 21+ required (found $JMAJOR); set CRUSSTY_ALLOW_OLD_JAVA=1 to bypass"
fi

# ------------------------------------------------------------ 2. kernel
stage "2 download kernel (purpur $PURPUR_VERSION)"
[ -f "$KERNEL" ] && log "kernel already present: $KERNEL" || {
    API="https://api.purpurmc.org/v2/purpur/$PURPUR_VERSION/latest/"
    log "querying $API"
    META="$(curl -fsSL --retry 3 --retry-delay 2 "$API")" || fail "cannot fetch $API"
    BUILD="$(printf '%s' "$META" | sed -n 's/.*"build":"\([0-9][0-9]*\)".*/\1/p' | head -1)"
    [ -n "$BUILD" ] || fail "cannot parse build id from: ${META:0:200}"
    URL="https://api.purpurmc.org/v2/purpur/$PURPUR_VERSION/$BUILD/download"
    log "downloading $URL"
    curl -fL --retry 3 --retry-delay 2 -o "$KERNEL.tmp" "$URL" || fail "cannot download $URL"
    mv "$KERNEL.tmp" "$KERNEL"
}
log "kernel: $KERNEL ($(du -h "$KERNEL" | cut -f1 2>/dev/null || echo ?))"

# -------------------------------------------------------------- 3. build
stage "3. build runtime + launcher + modules"
[ -f runtime/Cargo.toml ] || fail "not a checked-out repo (missing runtime/Cargo.toml)"

cargo build --manifest-path runtime/Cargo.toml || fail "cargo build runtime"
cp runtime/target/debug/libcrussty_runtime.so "$REPO/libcrussty_runtime.so"

mkdir -p launcher/out
javac -d launcher/out launcher/src/main/java/dev/dist/launcher/Main.java \
    || fail "javac launcher"
jar cfe "launcher/launcher.jar" dev.dist.launcher.Main -C launcher/out . \
    || fail "jar launcher"

for id in $MODULES; do
    m="$REPO/modules/$id"
    [ -f "$m/Cargo.toml" ] || fail "module not found: $id"
    [ -f "$m/cplugin.json" ] || fail "module $id has no cplugin.json"
    log "build + install module: $id"
    cargo build --manifest-path "$m/Cargo.toml" || fail "cargo build $id"
    cp "$m/target/debug/lib$id.so" "$m/lib$id.so"
done

# --------------------------------------------------------------- 4. boot
stage "4. boot server via run.sh"
printf 'eula=true\n' > eula.txt
rm -f "$SERVER_LOG" "$LAUNCHER_LOG"
mkfifo "$SERIAL_DIR/stdin"
exec 9<>"$SERIAL_DIR/stdin"   # keep a writer open so the fifo never blocks

$SETSID ./run.sh --port "$SERVER_PORT" \
    < "$SERIAL_DIR/stdin" > "$LAUNCHER_LOG" 2>&1 &
LAUNCHER_PID=$!

cleanup() {
    exec 9>&- 2>/dev/null || true
    kill -TERM -- "-$LAUNCHER_PID" 2>/dev/null || true
    sleep 2
    kill -KILL -- "-$LAUNCHER_PID" 2>/dev/null || true
}
trap cleanup EXIT

START="$SECONDS"
DEADLINE=$(( $(date +%s) + TIMEOUT_SEC ))
log "launcher pid $LAUNCHER_PID; waiting up to ${TIMEOUT_SEC}s for markers"

found_pipeline=0; found_hello=0; found_native=0
while :; do
    if [ -f "$SERVER_LOG" ]; then
        if grep -q "pipeline ready" "$SERVER_LOG"; then found_pipeline=1; fi
        if grep -q "hello from native c-plugin" "$SERVER_LOG"; then found_hello=1; fi
        if grep -q "native surface live" "$SERVER_LOG"; then found_native=1; fi
    fi
    if ! kill -0 "$LAUNCHER_PID" 2>/dev/null; then
        RC=0; wait "$LAUNCHER_PID" 2>/dev/null || RC=$?
        fail "launcher exited early (exit $RC); tail: $(tail -c 1500 "$LAUNCHER_LOG" 2>/dev/null)"
    fi
    [ "$found_pipeline" -eq 1 ] && [ "$found_hello" -eq 1 ] && break
    if [ "$(date +%s)" -ge "$DEADLINE" ]; then
        fail "timeout after ${TIMEOUT_SEC}s (pipeline=$found_pipeline hello=$found_hello native=$found_native); server log tail: $(tail -c 4000 "$SERVER_LOG" 2>/dev/null)"
    fi
    sleep "$POLL_SEC"
done

# ---------------------------------------------------------------- 5. stop
stage "5. markers seen; graceful stop"
log "pipeline ready=$found_pipeline  hello native=$found_hello  crussty live=$found_native"

echo "stop" >&9   # vanilla console command
log "sent 'stop'; waiting for shutdown"
for _ in $(seq 1 30); do
    kill -0 "$LAUNCHER_PID" 2>/dev/null || break
    sleep 1
done

echo
echo "================ E2E PASS ================"
echo "  kernel:    purpur-$PURPUR_VERSION"
echo "  modules:   $MODULES"
echo "  pipeline:  ready"
echo "  hello:     from native c-plugin"
echo "  crussty:   $([ "$found_native" -eq 1 ] && echo "native surface live" || echo "NOT verified (check modules/crussty/native/)")"
echo "  boot time: $((SECONDS - START))s"
exit 0