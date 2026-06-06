#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

mkdir -p "$WORKDIR/logs/load-smoke-live-bots" "$WORKDIR/reports"

cat >"$WORKDIR/reports/load-smoke-live-summary.txt" <<'EOF'
bots=3 online_max=3 server_join_events=3
compat_probe_send_pressure_samples=12
compat_probe_send_pressure_players_max=3
compat_probe_send_pressure_connections_max=3
compat_probe_send_pressure_chunk_senders_max=2
compat_probe_send_pending_actions_max=7
compat_probe_send_pending_outbound_bytes_max=4096
compat_probe_send_pending_outbound_bytes_read_count_max=3
compat_probe_send_pending_outbound_bytes_unavailable_count_max=0
compat_probe_send_bytes_before_writable_max=2048
compat_probe_send_bytes_before_writable_read_count_max=3
compat_probe_send_bytes_before_writable_unavailable_count_max=0
compat_probe_send_bytes_before_unwritable_min=512
compat_probe_send_bytes_before_unwritable_read_count_max=3
compat_probe_send_bytes_before_unwritable_unavailable_count_max=0
compat_probe_send_non_writable_connections_max=1
compat_probe_chunk_send_pending_chunks_max=9
compat_probe_chunk_send_pending_chunks_read_count_max=2
compat_probe_chunk_send_pending_chunks_unavailable_count_max=0
compat_probe_chunk_send_unacknowledged_batches_max=4
compat_probe_chunk_send_batch_quota_max=12.50
compat_probe_chunk_send_desired_chunks_per_tick_max=8.75
compat_probe_chunk_send_max_unacknowledged_batches_max=16
compat_probe_chunk_send_channel_not_writable_pending_chunks_peak=5
compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_read_count_max=2
compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_unavailable_count_max=0
compat_probe_chunk_send_near_unwritable_pending_chunks_peak=4
compat_probe_chunk_send_near_unwritable_pending_chunks_peak_read_count_max=2
compat_probe_chunk_send_near_unwritable_pending_chunks_peak_unavailable_count_max=0
compat_probe_chunk_send_near_unwritable_skips_max=6
compat_probe_chunk_send_near_unwritable_skips_read_count_max=2
compat_probe_chunk_send_near_unwritable_skips_unavailable_count_max=0
bot_log_tail:
early_abort_reason=host_contention_bad_samples=3_load_per_cpu=0.785_max_load_per_cpu=0.750
EOF

cat >"$WORKDIR/reports/load-smoke-live-gate.txt" <<'EOF'
gate_pass=false
failure=example
claim_eligible=false
run_class=environment-invalid
environment_invalid=true
environment_invalid_kind=host_contention
environment_invalid_reason=host_contention_bad_samples=3_load_per_cpu=0.785_max_load_per_cpu=0.750
EOF

cat >"$WORKDIR/reports/load-smoke-live-status.json" <<'EOF'
{"version":{"name":"Paper 1.21.10","protocol":773},"description":"Paper Rust Load Test","players":{"max":3,"online":3}}
EOF

cat >"$WORKDIR/logs/load-smoke-live.log" <<'EOF'
[00:00:01 INFO]: [CompatProbe] COMPAT_PROBE metrics online=1 loadedChunks=10 tps1=20.00 tps5=20.00 tps15=20.00 avgTickMs=1.00 usedMemMiB=100 blockPlaces=0 blockBreaks=0 arenaCommands=1 arenaPrepared=1 arenaSkipped=0 sendPressurePlayers=3 sendPressureConnections=3 sendPressureChunkSenders=2 connectionPendingOutboundBytesMax=4096 connectionPendingOutboundBytesReadCount=3 connectionPendingOutboundBytesUnavailableCount=0 connectionBytesBeforeWritableMax=2048 connectionBytesBeforeWritableReadCount=3 connectionBytesBeforeWritableUnavailableCount=0 connectionBytesBeforeUnwritableMin=512 connectionBytesBeforeUnwritableReadCount=3 connectionBytesBeforeUnwritableUnavailableCount=0 chunkSenderPendingChunksMax=9 chunkSenderPendingChunksReadCount=2 chunkSenderPendingChunksUnavailableCount=0 chunkSenderChannelNotWritablePendingChunksPeak=5 chunkSenderChannelNotWritablePendingChunksPeakReadCount=2 chunkSenderChannelNotWritablePendingChunksPeakUnavailableCount=0 chunkSenderChannelNotWritableSkipsMax=7 chunkSenderChannelNotWritableSkipsReadCount=2 chunkSenderChannelNotWritableSkipsUnavailableCount=0 chunkSenderNearUnwritablePendingChunksPeak=4 chunkSenderNearUnwritablePendingChunksPeakReadCount=2 chunkSenderNearUnwritablePendingChunksPeakUnavailableCount=0 chunkSenderNearUnwritableSkipsMax=6 chunkSenderNearUnwritableSkipsReadCount=2 chunkSenderNearUnwritableSkipsUnavailableCount=0 errors=0 kicked=0
[00:00:03 INFO]: [93mLoadBot000 joined the game[0m
[00:00:04 INFO]: [93mLoadBot001 joined the game[0m
[00:00:05 INFO]: [93mLoadBot002 joined the game[0m
EOF

cat >"$WORKDIR/logs/load-smoke-live-bots/shard-0.log" <<'EOF'
[00:00:02 INFO]: [Swarm] swarm_strict_failure kind=move username=LoadBot000 detail=example-detail
EOF

cat >"$WORKDIR/logs/load-smoke-live-bots.log" <<'EOF'
2026-05-21T00:00:01.000Z bot_player_join username=LoadBot000 elapsedMs=10
2026-05-21T00:00:02.000Z bot_player_join username=LoadBot001 elapsedMs=20
2026-05-21T00:00:03.000Z bot_player_join username=LoadBot002 elapsedMs=30
EOF

cat >"$WORKDIR/reports/load-smoke-live-resources.csv" <<'EOF'
ts_ms,pid_cpu,pid_rss_kb,system_load1,system_mem_available_kb
1,10,1000,0.10,9000
2,20,2000,0.20,8000
3,30,3000,0.30,7000
EOF

OUT_LABEL="$WORKDIR/out-label.txt"
OUT_PATHS="$WORKDIR/out-paths.txt"

python3 "$ROOT/scripts/summarize_live_load_run.py" \
  --label smoke-live \
  --reports-dir "$WORKDIR/reports" \
  --logs-dir "$WORKDIR/logs" \
  >"$OUT_LABEL"

python3 "$ROOT/scripts/summarize_live_load_run.py" \
  "$WORKDIR/logs/load-smoke-live.log" \
  "$WORKDIR/logs/load-smoke-live-bots.log" \
  "$WORKDIR/reports/load-smoke-live-summary.txt" \
  "$WORKDIR/reports/load-smoke-live-gate.txt" \
  "$WORKDIR/reports/load-smoke-live-status.json" \
  "$WORKDIR/reports/load-smoke-live-resources.csv" \
  >"$OUT_PATHS"

grep -q '^metrics_errors_max=0$' "$OUT_LABEL"
grep -q '^strict_failure_lines=1$' "$OUT_LABEL"
grep -q '^strict_failure_reasons=example-detail$' "$OUT_LABEL"
grep -q '^join_max=3$' "$OUT_LABEL"
grep -q '^compat_probe_send_pressure_samples=12$' "$OUT_LABEL"
grep -q '^compat_probe_send_pressure_players_max=3$' "$OUT_LABEL"
grep -q '^compat_probe_send_pressure_connections_max=3$' "$OUT_LABEL"
grep -q '^compat_probe_send_pressure_chunk_senders_max=2$' "$OUT_LABEL"
grep -q '^compat_probe_send_pending_actions_max=7$' "$OUT_LABEL"
grep -q '^compat_probe_send_pending_outbound_bytes_max=4096$' "$OUT_LABEL"
grep -q '^compat_probe_send_pending_outbound_bytes_read_count_max=3$' "$OUT_LABEL"
grep -q '^compat_probe_send_pending_outbound_bytes_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_send_bytes_before_writable_max=2048$' "$OUT_LABEL"
grep -q '^compat_probe_send_bytes_before_writable_read_count_max=3$' "$OUT_LABEL"
grep -q '^compat_probe_send_bytes_before_writable_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_send_bytes_before_unwritable_min=512$' "$OUT_LABEL"
grep -q '^compat_probe_send_bytes_before_unwritable_read_count_max=3$' "$OUT_LABEL"
grep -q '^compat_probe_send_bytes_before_unwritable_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_send_non_writable_connections_max=1$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_pending_chunks_max=9$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_pending_chunks_read_count_max=2$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_pending_chunks_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_unacknowledged_batches_max=4$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_channel_not_writable_pending_chunks_peak=5$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_channel_not_writable_skips_max=7$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_read_count_max=2$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_channel_not_writable_pending_chunks_peak_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_near_unwritable_pending_chunks_peak=4$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_near_unwritable_pending_chunks_peak_read_count_max=2$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_near_unwritable_pending_chunks_peak_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_near_unwritable_skips_max=6$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_near_unwritable_skips_read_count_max=2$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_near_unwritable_skips_unavailable_count_max=0$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_batch_quota_max=12.50$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_desired_chunks_per_tick_max=8.75$' "$OUT_LABEL"
grep -q '^compat_probe_chunk_send_max_unacknowledged_batches_max=16$' "$OUT_LABEL"
grep -q '^resources_tail_rows=3$' "$OUT_LABEL"
grep -q '^early_abort_reason=host_contention_bad_samples=3_load_per_cpu=0.785_max_load_per_cpu=0.750$' "$OUT_LABEL"
grep -q '^run_class=environment-invalid$' "$OUT_LABEL"

grep -q '^metrics_errors_max=0$' "$OUT_PATHS"
grep -q '^strict_failure_lines=1$' "$OUT_PATHS"
grep -q '^online_max=1$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_channel_not_writable_pending_chunks_peak=5$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_channel_not_writable_skips_max=7$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_channel_not_writable_skips_read_count_max=2$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_channel_not_writable_skips_unavailable_count_max=0$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_near_unwritable_pending_chunks_peak=4$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_near_unwritable_pending_chunks_peak_read_count_max=2$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_near_unwritable_pending_chunks_peak_unavailable_count_max=0$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_near_unwritable_skips_max=6$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_near_unwritable_skips_read_count_max=2$' "$OUT_PATHS"
grep -q '^compat_probe_chunk_send_near_unwritable_skips_unavailable_count_max=0$' "$OUT_PATHS"
grep -q '^environment_invalid=true$' "$OUT_PATHS"

if grep -q 'failure=.*errors=0' "$OUT_LABEL"; then
  echo "errors=0 was incorrectly treated as a failure" >&2
  exit 1
fi

echo "smoke_ok"
