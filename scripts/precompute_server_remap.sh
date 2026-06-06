#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JAR="${1:-$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar}"
OUT="${2:-$ROOT/artifacts/optimized-runtime/remap-classpath}"
RUN="$ROOT/runs/precompute-server-remap"
LOG="$ROOT/logs/precompute-server-remap.log"
PLUGIN="$ROOT/plugins/matrix/CompatProbe-0.1.0.jar"

if [[ ! -f "$JAR" ]]; then
  echo "Missing server jar: $JAR" >&2
  exit 1
fi
if [[ ! -f "$PLUGIN" ]]; then
  echo "Missing compat probe plugin: $PLUGIN" >&2
  exit 1
fi

mkdir -p "$OUT" "$ROOT/logs"
JAR="$(readlink -f "$JAR")"
OUT="$(cd "$OUT" && pwd)"
rm -rf "$RUN"
mkdir -p "$RUN/plugins"
cp "$PLUGIN" "$RUN/plugins/"

cat > "$RUN/eula.txt" <<'EULA'
eula=true
EULA
cat > "$RUN/server.properties" <<'PROPS'
online-mode=false
server-ip=127.0.0.1
server-port=0
level-name=precompute-world
motd=Paper remap precompute
PROPS

(
  cd "$RUN"
  java -Xms512M -Xmx1G -jar "$JAR" --nogui > "$LOG" 2>&1 &
  echo $! > server.pid
)
pid="$(cat "$RUN/server.pid")"

cleanup() {
  if kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

deadline=$((SECONDS + 120))
cache_file=""
while (( SECONDS < deadline )); do
  cache_file="$(find "$RUN/plugins/.paper-remapped/remap-classpath" -maxdepth 1 -type f -name '*.jar' -size +1024c -print -quit 2>/dev/null || true)"
  if [[ -n "$cache_file" ]] && rg -q "Done remapping server|Done installing precomputed remapped server" "$LOG"; then
    break
  fi
  if rg -q "Failed to start|Encountered an unexpected exception" "$LOG"; then
    tail -n 160 "$LOG" >&2
    exit 1
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    break
  fi
  sleep 1
done

if [[ -z "$cache_file" || ! -f "$cache_file" ]]; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

cp "$cache_file" "$OUT/$(basename "$cache_file")"
echo "$OUT/$(basename "$cache_file")"
