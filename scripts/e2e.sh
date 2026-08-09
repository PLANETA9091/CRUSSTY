#!/usr/bin/env bash
# E2E smoke test: boot a real Purpur server with the launcher (run.sh) and the
# cplug modules, and verify the platform markers appear.
#
#  1. download the kernel:   purpur-<PURPUR_VERSION> (cached in versions/)
#  2. build:                 native runtime, launcher, bundled modules
#  3. boot the server:       ./run.sh (launcher->java -jar purpur) with modules/
#  4. markers:               "[crussty-runtime] pipeline ready: ..."
#                            "hello from native c-plugin"
#                            (optional) "[crussty-module] native surface live"
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
TIMEOUT_SEC="${TIMEOUT_SEC:-420}"
POLL_SEC="${POLL_SEC:-2}"
MODULES_DEFAULT="hello dist crussty c-moduleslist"
MODULES="${MODULES:-$MODULES_DEFAULT}"

# Pick a port that is actually free right now (python3 is a hard depend two
# stages down): other dev servers on the box (e.g. a long-lived cells-e2e)
# must never turn a run into a bind collision.
free_port() {
    python3 - "$1" <<'EOF'
import socket, sys
base = int(sys.argv[1])
for p in range(base, base + 200):
    with socket.socket() as s:
        try:
            s.bind(("127.0.0.1", p))
            print(p)
            sys.exit(0)
        except OSError:
            continue
sys.exit(1)
EOF
}
if [ -n "${SERVER_PORT:-}" ]; then
    SERVER_PORT="$SERVER_PORT"
else
    SERVER_PORT="$(free_port 25566)" || fail "no free port found in range"
fi

SERIAL_DIR="logs/e2e.$$"
# The launcher tees the server's stdout+stderr into logs/server.log (and
# echoes it to its own stdout). Markers are grepped on BOTH the session
# capture (launcher.out) and the launcher's server.log (which the e2e once
# falsely expected at $SERIAL_DIR/server.log — it never exists).
SERVER_LOG="$SERIAL_DIR/server.log"
LAUNCHER_LOG="$SERIAL_DIR/launcher.out"
SERVER_TEE="logs/server.log"
KERNEL="versions/purpur-$PURPUR_VERSION.jar"

log()  { printf '%s\n' "[e2e] $*"; }
fail() { printf '%s\n' "[e2e] FAIL: $*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || fail "missing tool: $1"; }

if command -v setsid >/dev/null 2>&1; then SETSID="setsid"; else SETSID=""; fi

mkdir -p "$SERIAL_DIR" versions

stage() { printf '\n%s\n' "=== STAGE $1 ==="; }

# ---------------------------------------------------------------- 1. toolchain
stage "1. toolchain"
require java; require javac; require jar; require curl; require cargo; require git
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

# Modules live in their own repos (c-<id> convention); clone fresh and
# install into modules/<id>/ (this dir is also where the launcher discovers
# them at boot).
module_repo() {
    case "$1" in
        c-*) echo "PLANETA9091/$1" ;;
        *) echo "PLANETA9091/c-$1" ;;
    esac
}
for id in $MODULES; do
    repo="$(module_repo "$id")"
    src="$SERIAL_DIR/mod-src/$id"
    log "clone + build + install module: $id ($repo)"
    git clone --depth 1 "https://github.com/$repo.git" "$src" >/dev/null 2>&1 \
        || fail "cannot clone $repo"
    [ -f "$src/Cargo.toml" ] || fail "module $id has no Cargo.toml in $repo"
    [ -f "$src/cplugin.json" ] || fail "module $id has no cplugin.json"
    cargo build --manifest-path "$src/Cargo.toml" || fail "cargo build $id"
    # The [lib] name may differ from the module id (cargo forbids hyphens in
    # library target names, while manifest ids may carry them), so resolve the
    # produced artifact instead of assuming `lib$id.so`.
    LIB_NAME="$(sed -n '/^\[lib\]/,/^\[/p' "$src/Cargo.toml" | sed -n 's/^name *= *"\(.*\)"/\1/p' | head -1)"
    LIB_NAME="${LIB_NAME:-$id}"
    mkdir -p "$REPO/modules/$id"
    cp "$src/cplugin.json" "$REPO/modules/$id/"
    cp "$src/target/debug/lib$LIB_NAME.so" "$REPO/modules/$id/lib$id.so"
    # bundled native deps (e.g. c-crussty/native) are part of the module
    [ -d "$src/native" ] && cp -r "$src/native" "$REPO/modules/$id/"
done

# --------------------------------------------------------------- 4. boot
stage "4. boot server via run.sh"
printf 'eula=true\n' > eula.txt
rm -f "$SERVER_LOG" "$LAUNCHER_LOG" "$SERVER_TEE"
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
    for target in "$SERVER_TEE" "$LAUNCHER_LOG"; do
        [ -f "$target" ] || continue
        if grep -q "pipeline ready" "$target"; then found_pipeline=1; fi
        if grep -q "hello from native c-plugin" "$target"; then found_hello=1; fi
        if grep -q "native surface live" "$target"; then found_native=1; fi
    done
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

# ---------------------------------------------------------------- 5. reload
stage "5. hot-reload: rebuild a module (new inode), SIGUSR1, assert purge + alive"
log "pipeline ready=$found_pipeline  hello native=$found_hello  crussty live=$found_native"

RELOAD_ID="${E2E_RELOAD_ID:-hello}"
m="$SERIAL_DIR/mod-src/$RELOAD_ID"
[ -f "$m/Cargo.toml" ] || fail "reload module not found: $RELOAD_ID (mod-src clone)"

# Honest reload: the .so the server has dlopened must change on disk (new
# inode) BEFORE the trigger, exactly like a developer rebuilding a module.
# Force a real recompile: cargo's up-to-date check can no-op even when its
# output got replaced by our previous install (artifacts end up hardlinked
# to the installed copy), so the artifact is removed first.
INSTALLED="$REPO/modules/$RELOAD_ID"
OLD_INODE="$(stat -c '%i' "$INSTALLED/lib$RELOAD_ID.so")"
touch "$m/src/lib.rs"
rm -f "$m/target/debug/lib$RELOAD_ID.so"
cargo build --manifest-path "$m/Cargo.toml" || fail "rebuild $RELOAD_ID for reload"
# Replace via mv (new directory entry): guaranteed new inode — in-place cp
# would keep the inode and the server's dlopen cache could return the same
# mapping.
mv -f "$m/target/debug/lib$RELOAD_ID.so" "$INSTALLED/lib$RELOAD_ID.so"
NEW_INODE="$(stat -c '%i' "$INSTALLED/lib$RELOAD_ID.so")"
if [ "$OLD_INODE" = "$NEW_INODE" ]; then
    fail "reload artifact not actually replaced (same inode $OLD_INODE)"
fi
log "artifact replaced: inode $OLD_INODE -> $NEW_INODE"

# Match the server by port, not by jar name alone: a foreign server running
# the same jar (e.g. an older rollout) must never receive our trigger.
JAVA_PID="$(pgrep -f "purpur-$PURPUR_VERSION.jar.*--port $SERVER_PORT" | head -1 || true)"
if [ -z "$JAVA_PID" ]; then
    fail "no server pid on port $SERVER_PORT for SIGUSR1 reload"
fi
kill -USR1 "$JAVA_PID" 2>/dev/null || fail "kill -USR1 $JAVA_PID failed"
log "sent SIGUSR1 to $JAVA_PID; waiting for reload + stale-hook purge"

found_reload=0; found_purge=0; found_hooks_after=0
RDEADLINE=$(( $(date +%s) + 40 ))
while [ "$(date +%s)" -lt "$RDEADLINE" ]; do
    for target in "$SERVER_TEE" "$LAUNCHER_LOG"; do
        [ -f "$target" ] || continue
        grep -q "reloaded (dlopen'd fresh" "$target" && found_reload=1
        grep -q "hook purge" "$target" && found_purge=1
        grep -q "pipeline ready: " "$target" && found_hooks_after=1
    done
    if [ "$found_reload" -eq 1 ] && [ "$found_purge" -eq 1 ]; then break; fi
    sleep 2
done
kill -0 "$JAVA_PID" 2>/dev/null || fail "server died during hot-reload (reload=$found_reload purge=$found_purge)"
[ "$found_reload" -eq 1 ] || fail "hot-reload never completed (reload=$found_reload purge=$found_purge)"
[ "$found_purge" -eq 1 ] || fail "stale hooks not purged on reload (would SIGSEGV on next class load)"
log "hot-reload ok: reloaded=$found_reload hook-purge=$found_purge"

# -------------------------------------------- 5b. /modules native command
stage "5b. /modules command replies like /plugins (no plugin involved)"
# The console accepts input once the server is up; keep typing until the
# reply shows up (bounded), so this stage has no ordering burden on boot.
found_registered=0; found_reply=0; found_listed=0
CDEADLINE=$(( $(date +%s) + 120 ))
while [ "$(date +%s)" -lt "$CDEADLINE" ]; do
    echo "modules" >&9
    for target in "$SERVER_TEE" "$LAUNCHER_LOG"; do
        [ -f "$target" ] || continue
        grep -q "\[c-moduleslist\] /modules registered" "$target" && found_registered=1
        grep -q "Modules (" "$target" && found_reply=1
        grep -q "Modules ([0-9]\+): .*c-moduleslist" "$target" && found_listed=1
    done
    if [ "$found_registered" -eq 1 ] && [ "$found_reply" -eq 1 ] && [ "$found_listed" -eq 1 ]; then break; fi
    sleep 3
done
kill -0 "$JAVA_PID" 2>/dev/null || fail "server died before /modules check (reg=$found_registered reply=$found_reply)"
[ "$found_registered" -eq 1 ] || fail "/modules command never registered"
[ "$found_reply" -eq 1 ] || fail "/modules never replied (tail: $(tail -c 2000 "$LAUNCHER_LOG" 2>/dev/null))"
[ "$found_listed" -eq 1 ] || fail "/modules reply does not list c-moduleslist (tail: $(tail -c 2000 "$LAUNCHER_LOG" 2>/dev/null))"
N="$(grep -o 'Modules ([0-9]\+):' "$SERVER_TEE" "$LAUNCHER_LOG" 2>/dev/null | head -1)"
log "modules command ok: ${N:-Modules (?)}, reply listed c-moduleslist"

# ---------------------------------------------------------------- 6. stop
stage "6. graceful stop"

echo "stop" >&9   # vanilla console command
log "sent 'stop'; waiting for shutdown"
for _ in $(seq 1 30); do
    kill -0 "$LAUNCHER_PID" 2>/dev/null || break
    sleep 1
done

# -------------------------------------------- 7. single-jar distribution path
stage "7. single-jar boot (java -jar, no -agentpath) + reload"
# README's recommended distribution: dist/crussty-<ver>.jar extracts the
# runtime + modules through Boot.java and loads them via JNI_OnLoad. The
# pgrep matcher differs from stage 5 (jar is named crussty-*.jar, not
# purpur-*.jar), so this stage is the only automated coverage of that path.
SJ_PORT="$(free_port 25600)" || fail "no free port found in range (single-jar)"
bash scripts/build-single-jar.sh "$PURPUR_VERSION" >/dev/null 2>&1 \
    || fail "build-single-jar.sh failed"
SJ_DIR="$SERIAL_DIR/sjrun"
mkdir -p "$SJ_DIR"
SJ_JAR="$REPO/dist/crussty-$PURPUR_VERSION.jar"
log "booting $SJ_JAR on port $SJ_PORT (workdir $SJ_DIR)"

printf 'eula=true\n' > "$SJ_DIR/eula.txt"
( cd "$SJ_DIR" && $SETSID java -Xmx2G -jar "$SJ_JAR" --nogui --port "$SJ_PORT" \
    > sj.log 2>&1 < /dev/null ) &
SJ_LAUNCHER_PID=$!
trap 'kill -TERM -- "$SJ_LAUNCHER_PID" 2>/dev/null || true; cleanup' EXIT

SJ_LOG="$SJ_DIR/sj.log"
found_sj_pipeline=0; found_sj_done=0
SJ_DEADLINE=$(( $(date +%s) + TIMEOUT_SEC ))
while :; do
    [ -f "$SJ_LOG" ] || { sleep 2; continue; }
    grep -q "pipeline ready" "$SJ_LOG" && found_sj_pipeline=1
    grep -q "Done (" "$SJ_LOG" && found_sj_done=1
    [ "$found_sj_pipeline" -eq 1 ] && [ "$found_sj_done" -eq 1 ] && break
    if ! kill -0 "$SJ_LAUNCHER_PID" 2>/dev/null; then
        fail "single-jar exited early; tail: $(tail -c 1500 "$SJ_LOG" 2>/dev/null)"
    fi
    if [ "$(date +%s)" -ge "$SJ_DEADLINE" ]; then
        fail "single-jar timeout (pipeline=$found_sj_pipeline done=$found_sj_done); tail: $(tail -c 2500 "$SJ_LOG" 2>/dev/null)"
    fi
    sleep "$POLL_SEC"
done
log "single-jar boot ok: pipeline=$found_sj_pipeline done=$found_sj_done"

SJ_PID="$(pgrep -f "crussty-$PURPUR_VERSION.jar.*--port $SJ_PORT" | head -1 || true)"
[ -n "$SJ_PID" ] || fail "no single-jar server pid on port $SJ_PORT"
kill -USR1 "$SJ_PID" 2>/dev/null || fail "kill -USR1 $SJ_PID failed"
log "sent SIGUSR1 to $SJ_PID; waiting for reload + hook purge"

found_sj_reload=0; found_sj_purge=0
SJ_RDEADLINE=$(( $(date +%s) + 40 ))
while [ "$(date +%s)" -lt "$SJ_RDEADLINE" ]; do
    grep -q "reloaded (dlopen'd fresh" "$SJ_LOG" 2>/dev/null && found_sj_reload=1
    grep -q "hook purge" "$SJ_LOG" 2>/dev/null && found_sj_purge=1
    if [ "$found_sj_reload" -eq 1 ] && [ "$found_sj_purge" -eq 1 ]; then break; fi
    sleep 2
done
kill -0 "$SJ_PID" 2>/dev/null || fail "single-jar server died during hot-reload (reload=$found_sj_reload purge=$found_sj_purge)"
[ "$found_sj_reload" -eq 1 ] || fail "single-jar hot-reload never completed (reload=$found_sj_reload purge=$found_sj_purge)"
[ "$found_sj_purge" -eq 1 ] || fail "single-jar stale hooks not purged (reload=$found_sj_reload purge=$found_sj_purge)"
log "single-jar hot-reload ok: reloaded=$found_sj_reload hook-purge=$found_sj_purge"
kill -TERM -- "$SJ_PID" 2>/dev/null || true

echo
echo "================ E2E PASS ================"
echo "  kernel:    purpur-$PURPUR_VERSION"
echo "  modules:   $MODULES"
echo "  pipeline:  ready"
echo "  hello:     from native c-plugin"
echo "  crussty:   $([ "$found_native" -eq 1 ] && echo "native surface live" || echo "NOT verified (check modules/crussty/native/)")"
echo "  single-jar: boot + hot-reload ok"
echo "  boot time: $((SECONDS - START))s"
exit 0