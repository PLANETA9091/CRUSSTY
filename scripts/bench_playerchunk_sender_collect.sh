#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/bench/playerchunk-sender/PlayerChunkSenderCollectChunksBench.java"
OUT="$ROOT/bench/playerchunk-sender/.classes"
REPORT="$ROOT/reports/playerchunk-sender-collect-bench.txt"
RUNTIME_CP_FILE="$ROOT/artifacts/optimized-runtime/classpath.txt"
PLAYER_CHUNK_SENDER_SRC="$ROOT/upstream/Paper/paper-server/src/minecraft/java/net/minecraft/server/network/PlayerChunkSender.java"
SOURCE_CHECK_OUT="$ROOT/.verify-classes/playerchunk-sender"

if [[ ! -s "$RUNTIME_CP_FILE" ]]; then
  echo "optimized runtime classpath not found; run scripts/build_optimized.sh first" >&2
  exit 1
fi

RUNTIME_CP="$(<"$RUNTIME_CP_FILE")"

rm -rf "$SOURCE_CHECK_OUT"
mkdir -p "$OUT" "$SOURCE_CHECK_OUT" "$(dirname "$REPORT")"
javac -proc:none -cp "$RUNTIME_CP" -d "$SOURCE_CHECK_OUT" "$PLAYER_CHUNK_SENDER_SRC"
javac -proc:none -cp "$RUNTIME_CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "player_chunk_sender_compile=PASS"
  echo "player_chunk_sender_compile_source=$PLAYER_CHUNK_SENDER_SRC"
  echo "player_chunk_sender_compile_classpath_file=$RUNTIME_CP_FILE"
  echo "command=java \${JAVA_PROPS:-} -cp \"$OUT:\$(cat $RUNTIME_CP_FILE)\" PlayerChunkSenderCollectChunksBench"
  # shellcheck disable=SC2086
  java ${JAVA_PROPS:-} -cp "$OUT:$RUNTIME_CP" PlayerChunkSenderCollectChunksBench
} | tee "$REPORT"
