#!/usr/bin/env bash
# c-menus E2E: build the platform runtime + launcher from this repo, install
# the c-menus module, boot a real Paper 26.3 kernel (Java 25) and verify the
# module markers appear.
#
#   1. download paper-26.3 (fill API, cached in versions/)
#   2. build: libcrussty_runtime.so, launcher.jar, libcmenus.so (release)
#   3. server dir: versions/, launcher/, modules/c-menus/
#   4. boot via run.sh, grep for:
#        "pipeline ready"             (runtime hook pipeline)
#        "[c-menus] ready:"           (module boot: bridge + menus loaded)
#        "[c-menus] tick ... alive"   (per-tick pump actually firing)
#   5. stop cleanly; exit 0 only when every marker appeared.
#
# Requires: cargo, javac+jar (JDK 25 recommended — Paper 26.3 needs 25),
# curl, python3, setsid. Env: PAPER_VERSION (26.3), SERVER_PORT (auto),
# TIMEOUT_SEC (420), JAVA_HOME.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
MOD="$REPO/modules/c-menus"
cd "$REPO"

PAPER_VERSION="${PAPER_VERSION:-26.3}"
TIMEOUT_SEC="${TIMEOUT_SEC:-420}"
POLL_SEC="${POLL_SEC:-2}"

log() { printf '[cmenus-e2e] %s\n' "$*"; }
fail() { printf '[cmenus-e2e] FAIL: %s\n' "$*" >&2; exit 1; }
stage() { log "--- $* ---"; }

command -v cargo >/dev/null || fail "cargo not on PATH"
command -v javac >/dev/null || fail "javac not on PATH (install JDK 25)"
command -v curl >/dev/null || fail "curl not on PATH"

# free port
SERVER_PORT="${SERVER_PORT:-$(python3 - <<'EOF'
import socket
for p in range(25570, 25770):
    try:
        s = socket.socket(); s.bind(("127.0.0.1", p)); s.close(); print(p); break
    except OSError:
        continue
EOF
)}"

SERIAL_DIR="$REPO/logs/cmenus-e2e.$$"
mkdir -p "$SERIAL_DIR"
SERVER_LOG="$REPO/logs/server.log"
LAUNCHER_LOG="$SERIAL_DIR/launcher.out"
KERNEL="versions/paper-$PAPER_VERSION.jar"

cleanup() {
    [[ -n "${LAUNCHER_PID:-}" ]] && kill -TERM -- "-$LAUNCHER_PID" 2>/dev/null || true
    sleep 2
    kill -KILL -- "-$LAUNCHER_PID" 2>/dev/null || true
}
trap cleanup EXIT

# ---------------------------------------------------------------- 1. kernel
stage "1. download paper $PAPER_VERSION"
if [ ! -f "$KERNEL" ]; then
    mkdir -p versions
    URL="$(curl -fsSL "https://fill.papermc.io/v3/projects/paper/versions/$PAPER_VERSION/builds/latest" \
        | python3 -c 'import json,sys; print(json.load(sys.stdin)["downloads"]["server:default"]["url"])')" \
        || fail "cannot resolve paper download url"
    log "downloading $URL"
    curl -fL --retry 3 --retry-delay 2 -o "$KERNEL.tmp" "$URL" || fail "download failed"
    mv "$KERNEL.tmp" "$KERNEL"
fi
log "kernel: $KERNEL ($(du -h "$KERNEL" | cut -f1))"

# ---------------------------------------------------------------- 2. build
stage "2. build runtime + launcher + c-menus"
cargo build --manifest-path runtime/Cargo.toml --release || fail "cargo build runtime"
cp runtime/target/release/libcrussty_runtime.so "$REPO/libcrussty_runtime.so"

mkdir -p launcher/out
javac -d launcher/out launcher/src/main/java/dev/dist/launcher/Main.java || fail "javac launcher"
jar cfe launcher/launcher.jar dev.dist.launcher.Main -C launcher/out . || fail "jar launcher"

cargo build --release --manifest-path "$MOD/Cargo.toml" || fail "cargo build c-menus"

# ---------------------------------------------------------------- 3. install
stage "3. install module into server dir"
SERVER_DIR="$SERIAL_DIR/server"
mkdir -p "$SERVER_DIR/modules/c-menus" "$SERVER_DIR/logs" "$SERVER_DIR/versions"
cp "$KERNEL" "$SERVER_DIR/versions/paper-$PAPER_VERSION.jar"
cp launcher/launcher.jar "$SERVER_DIR/"
cp "$REPO/libcrussty_runtime.so" "$SERVER_DIR/"
cp "$MOD/module.json" "$SERVER_DIR/modules/c-menus/"
cp "$MOD/target/release/libcmenus.so" "$SERVER_DIR/modules/c-menus/"
cp -r "$MOD/menus" "$SERVER_DIR/modules/c-menus/"
cp -r "$MOD/pack" "$SERVER_DIR/modules/c-menus/"
cp "$MOD/config.json" "$SERVER_DIR/modules/c-menus/"
printf 'eula=true\n' > "$SERVER_DIR/eula.txt"

# ---------------------------------------------------------------- 4. boot
stage "4. boot paper $PAPER_VERSION (port $SERVER_PORT)"
cd "$SERVER_DIR"
mkfifo "$SERIAL_DIR/stdin" 2>/dev/null || true
exec 9<>"$SERIAL_DIR/stdin"
setsid java -Xmx2G -jar launcher.jar --nogui --port "$SERVER_PORT" \
    < "$SERIAL_DIR/stdin" > "$LAUNCHER_LOG" 2>&1 &
LAUNCHER_PID=$!
cd "$REPO"

START="$SECONDS"
DEADLINE=$(( $(date +%s) + TIMEOUT_SEC ))
found_pipeline=0; found_ready=0; found_tick=0
while :; do
    for target in "$LAUNCHER_LOG" "$SERVER_LOG" "$SERVER_DIR/logs/server.log"; do
        [ -f "$target" ] || continue
        grep -q "pipeline ready" "$target" && found_pipeline=1
        grep -q "\[c-menus\] ready:" "$target" && found_ready=1
        grep -qE "\[c-menus\] tick [0-9]+ alive" "$target" && found_tick=1
    done
    if ! kill -0 "$LAUNCHER_PID" 2>/dev/null; then
        RC=0; wait "$LAUNCHER_PID" 2>/dev/null || RC=$?
        fail "launcher exited early (rc=$RC); tail: $(tail -c 2000 "$LAUNCHER_LOG" 2>/dev/null || true)"
    fi
    if [ "$found_pipeline" = 1 ] && [ "$found_ready" = 1 ] && [ "$found_tick" = 1 ]; then
        break
    fi
    [ "$(date +%s)" -ge "$DEADLINE" ] && \
        fail "timeout after ${TIMEOUT_SEC}s (pipeline=$found_pipeline ready=$found_ready tick=$found_tick); tail: $(tail -c 4000 "$LAUNCHER_LOG" 2>/dev/null || true)"
    sleep "$POLL_SEC"
done

log "OK: pipeline ready, c-menus booted, tick pump alive ($(( $(date +%s) - START ))s)"
log "markers: pipeline=$found_pipeline ready=$found_ready tick=$found_tick"
exit 0
