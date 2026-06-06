#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/chunk-packet-data/out"
REPORT="$ROOT/reports/clientbound-level-chunk-packet-data-ctor-bench.txt"

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -d "$OUT" "$ROOT/bench/chunk-packet-data/ClientboundLevelChunkPacketDataCtorBench.java"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "command=java -Xms512m -Xmx512m -Diterations=\"${CHUNK_PACKET_CTOR_ITERATIONS:-250000}\" -Dwarmup=\"${CHUNK_PACKET_CTOR_WARMUP:-10000}\" -Drounds=\"${CHUNK_PACKET_CTOR_ROUNDS:-5}\" -DblockEntityLimit=\"${CHUNK_PACKET_BLOCK_ENTITY_LIMIT:-750}\" -cp \"$OUT\" ClientboundLevelChunkPacketDataCtorBench"
  java -Xms512m -Xmx512m \
    -Diterations="${CHUNK_PACKET_CTOR_ITERATIONS:-250000}" \
    -Dwarmup="${CHUNK_PACKET_CTOR_WARMUP:-10000}" \
    -Drounds="${CHUNK_PACKET_CTOR_ROUNDS:-5}" \
    -DblockEntityLimit="${CHUNK_PACKET_BLOCK_ENTITY_LIMIT:-750}" \
    -cp "$OUT" ClientboundLevelChunkPacketDataCtorBench
} | tee "$REPORT"
