#!/usr/bin/env bash
set -euo pipefail

if [[ "${MC_EULA_AGREE:-}" != "true" ]]; then
  echo "Refusing to run without explicit MC_EULA_AGREE=true." >&2
  exit 78
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/launcher_utils.sh"
source "$ROOT/scripts/load_test_defaults.sh"
source "$ROOT/scripts/stress_plugin_overrides.sh"
SERVER_READY_REGEX="$(load_test_server_ready_regex)"
PRECOMPUTE_STRESS_CORPUS="${PRECOMPUTE_STRESS_CORPUS:-false}"
JAVA_OPTS_PRECOMPUTE="${JAVA_OPTS_PRECOMPUTE:--Xms2G -Xmx8G -XX:+UseG1GC -XX:+ParallelRefProcEnabled}"
if [[ $# -gt 0 ]]; then
  LAUNCHER="$1"
elif [[ -x "$ROOT/artifacts/optimized-runtime/run.sh" ]]; then
  LAUNCHER="$ROOT/artifacts/optimized-runtime/run.sh"
else
  LAUNCHER="$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"
fi
LAUNCHER="$(resolve_launcher_path "$LAUNCHER")"
OUT="${2:-$ROOT/artifacts/optimized-runtime/plugin-remaps}"
REVERSED_OUT="${3:-$ROOT/artifacts/optimized-runtime/reversed-mappings}"
RUN="$ROOT/runs/precompute-plugin-remaps"
LOG="$ROOT/logs/precompute-plugin-remaps.log"
FIFO="$RUN/console.in"
PORT="$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"
free_tcp_port() {
  python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}
SQUAREMAP_PORT="${SQUAREMAP_PORT:-$(free_tcp_port)}"
BLUEMAP_PORT="${BLUEMAP_PORT:-$(free_tcp_port)}"
VOICECHAT_PORT="${VOICECHAT_PORT:-$(free_tcp_port)}"

if [[ "$PRECOMPUTE_STRESS_CORPUS" != "true" && "$PRECOMPUTE_STRESS_CORPUS" != "false" ]]; then
  echo "PRECOMPUTE_STRESS_CORPUS must be true or false." >&2
  exit 1
fi

mkdir -p "$OUT" "$REVERSED_OUT" "$ROOT/logs" "$ROOT/reports"
rm -rf "$RUN"
mkdir -p "$RUN/plugins"
cp "$ROOT/plugins/matrix"/*.jar "$RUN/plugins/"
if compgen -G "$ROOT/plugins/matrix-libraries/*" > /dev/null; then
  cp -R "$ROOT/plugins/matrix-libraries"/. "$RUN/"
fi
if [[ "$PRECOMPUTE_STRESS_CORPUS" == "true" ]]; then
  if ! compgen -G "$ROOT/plugins/stress/*.jar" > /dev/null; then
    echo "PRECOMPUTE_STRESS_CORPUS=true but plugins/stress has no jars. Run scripts/fetch_stress_corpus.py first." >&2
    exit 1
  fi
  cp "$ROOT/plugins/stress"/*.jar "$RUN/plugins/"
  mkdir -p "$RUN/plugins/spark" "$RUN/plugins/squaremap" "$RUN/plugins/BlueMap" "$RUN/plugins/voicechat"
  cat > "$RUN/plugins/spark/config.json" <<SPARKCFG
{
  "_header": "precompute keeps spark background profiler disabled",
  "backgroundProfiler": false
}
SPARKCFG
  cat > "$RUN/plugins/squaremap/config.yml" <<SQUAREMAP
config-version: 2
settings:
    language-file: lang-en.yml
    debug-mode: false
    update-checker: false
    web-address: http://127.0.0.1:$SQUAREMAP_PORT
    internal-webserver:
        enabled: true
        bind: 127.0.0.1
        port: $SQUAREMAP_PORT
world-settings:
    default:
        map:
            enabled: true
SQUAREMAP
  cat > "$RUN/plugins/BlueMap/core.conf" <<BLUEMAPCORE
accept-download: false
data: "bluemap"
render-thread-count: 1
scan-for-mod-resources: true
metrics: false
BLUEMAPCORE
  cat > "$RUN/plugins/BlueMap/webserver.conf" <<BLUEMAPWEB
enabled: true
webroot: "bluemap/web"
port: $BLUEMAP_PORT
BLUEMAPWEB
  cat > "$RUN/plugins/voicechat/voicechat-server.properties" <<VOICECHAT
port=$VOICECHAT_PORT
bind_address=127.0.0.1
allow_pings=false
VOICECHAT
  mkdir -p "$RUN/plugins/SkinsRestorer"
  cat > "$RUN/plugins/SkinsRestorer/config.yml" <<SKINSRESTORER
storage:
    defaultSkins:
        enabled: false
    disallowAutoUpdateSkin: true
    skinExpiresAfter: 10080
    uuidExpiresAfter: 10080
api:
    mineskinAPIKey: ''
    fetchRecommendedSkins: false
    mojangBatchWindowSeconds: 1
    elyByEnabled: false
advanced:
    disableOnJoinSkins: true
    enablePaperJoinListener: false
    noConnections: true
SKINSRESTORER
  touch "$RUN/plugins/SkinsRestorer/noautoupdate.txt"
fi
seed_luckperms_vault_unsafe_lookups "$RUN" "$ROOT"

cat > "$RUN/eula.txt" <<'EULA'
eula=true
EULA
cat > "$RUN/server.properties" <<PROPS
online-mode=false
server-ip=127.0.0.1
server-port=$PORT
level-seed=paper-rust-plugin-remap-precompute
view-distance=2
simulation-distance=2
enable-rcon=false
motd=Paper plugin remap precompute
PROPS

mkfifo "$FIFO"
if [[ "$LAUNCHER" == *.sh ]]; then
  CMD=("$LAUNCHER" --nogui)
else
  CMD=(java $JAVA_OPTS_PRECOMPUTE -jar "$LAUNCHER" --nogui)
fi

(
  cd "$RUN"
  JAVA_OPTS="$JAVA_OPTS_PRECOMPUTE" "${CMD[@]}" < "$FIFO" > "$LOG" 2>&1 &
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

exec 3>"$FIFO"

deadline=$((SECONDS + 180))
while (( SECONDS < deadline )); do
  if rg -q "$SERVER_READY_REGEX" "$LOG"; then
    break
  fi
  if rg -q "Failed to bind to port|Encountered an unexpected exception|UnsupportedClassVersionError|Could not load plugin|Failed to start" "$LOG"; then
    tail -n 160 "$LOG" >&2
    exit 1
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    break
  fi
  sleep 1
done

if ! rg -q "$SERVER_READY_REGEX" "$LOG"; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

printf 'stop\n' >&3
exec 3>&-

deadline=$((SECONDS + 90))
while kill -0 "$pid" 2>/dev/null && (( SECONDS < deadline )); do
  sleep 1
done
if kill -0 "$pid" 2>/dev/null; then
  tail -n 180 "$LOG" >&2
  exit 1
fi

python3 - "$RUN/plugins/.paper-remapped" "$OUT" "$REVERSED_OUT" "$ROOT/reports/precomputed-plugin-remaps.csv" <<'PY'
import csv
import json
import pathlib
import shutil
import sys

remapped = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2])
reversed_out = pathlib.Path(sys.argv[3])
report = pathlib.Path(sys.argv[4])
index_path = remapped / "index.json"

state = json.loads(index_path.read_text(encoding="utf-8"))
mappings_hash = state["mappingsHash"]
target_dir = out / mappings_hash
target_dir.mkdir(parents=True, exist_ok=True)
reversed_out.mkdir(parents=True, exist_ok=True)

reversed_destination = reversed_out / f"{mappings_hash}.tiny"
reversed_source = remapped / "mappings" / "reversed" / f"{mappings_hash}.tiny"
if not reversed_source.is_file():
    cached_reversed = reversed_destination
    if cached_reversed.is_file():
        reversed_source = cached_reversed
    else:
        raise SystemExit(f"Missing reversed mappings precompute output: {reversed_source}")

if reversed_source != reversed_destination:
    reversed_tmp = reversed_destination.with_suffix(reversed_destination.suffix + ".tmp")
    shutil.copyfile(reversed_source, reversed_tmp)
    reversed_tmp.replace(reversed_destination)

rows = [("kind", "mappings_hash", "input_hash", "file", "bytes", "destination")]

def export_index(kind, source_dir, index, destination_dir):
    if not index.is_file():
        return 0, 0

    indexed = json.loads(index.read_text(encoding="utf-8"))
    if indexed["mappingsHash"] != mappings_hash:
        raise SystemExit(f"Mismatched mappings hash in {index}: {indexed['mappingsHash']} != {mappings_hash}")

    destination_dir.mkdir(parents=True, exist_ok=True)
    skipped = set(indexed.get("skippedHashes", []))
    skipped_hashes = destination_dir / "skipped-hashes.txt"
    skipped_tmp = skipped_hashes.with_suffix(skipped_hashes.suffix + ".tmp")
    skipped_tmp.write_text("".join(f"{plugin_hash}\n" for plugin_hash in sorted(skipped)), encoding="utf-8")
    skipped_tmp.replace(skipped_hashes)

    count = 0
    for plugin_hash, file_name in sorted(indexed.get("hashes", {}).items()):
        if plugin_hash in skipped:
            continue
        source = source_dir / file_name
        if not source.is_file():
            raise SystemExit(f"Missing remapped {kind} for {plugin_hash}: {source}")
        destination = destination_dir / f"{plugin_hash}.jar"
        tmp = destination.with_suffix(destination.suffix + ".tmp")
        shutil.copyfile(source, tmp)
        tmp.replace(destination)
        count += 1
        rows.append((kind, mappings_hash, plugin_hash, file_name, str(destination.stat().st_size), str(destination)))
    return count, len(skipped)

count, skipped_count = export_index("plugin", remapped, index_path, target_dir)
library_count, library_skipped_count = export_index(
    "library",
    remapped / "libraries",
    remapped / "libraries" / "index.json",
    target_dir / "libraries",
)

report.parent.mkdir(parents=True, exist_ok=True)
with report.open("w", encoding="utf-8", newline="") as handle:
    csv.writer(handle).writerows(rows)

print(f"precomputed_plugin_remaps={count}")
print(f"precomputed_plugin_skips={skipped_count}")
print(f"precomputed_library_remaps={library_count}")
print(f"precomputed_library_skips={library_skipped_count}")
print(f"mappings_hash={mappings_hash}")
print(f"output_dir={target_dir}")
print(f"reversed_mappings={reversed_destination}")
PY

{
  echo "precompute_stress_corpus=$PRECOMPUTE_STRESS_CORPUS"
  echo "java_opts=$JAVA_OPTS_PRECOMPUTE"
  echo "server_port=$PORT"
  echo "squaremap_port=$SQUAREMAP_PORT"
  echo "bluemap_port=$BLUEMAP_PORT"
  echo "voicechat_port=$VOICECHAT_PORT"
  rg -n \
    -e "$SERVER_READY_REGEX" \
    -e "PluginRemapper|Installing precomputed remapped plugin|Could not load plugin|ERROR|Exception" \
    "$LOG" || true
} > "$ROOT/reports/precomputed-plugin-remaps-summary.txt"
