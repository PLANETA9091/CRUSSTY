# P500 Block Packet And Container Goal

Дата: 2026-05-20 CEST

Эта цель заменяет расплывчатое "дожать до конца" на текущий измеряемый
блокер. Текущий artifact не получает `production-ready` claim, пока свежий
gate красный.

## Target Claim

Разрешённая формулировка после зелёных gate:

> production-ready для измеренного `500 bots / 32 view / 32 simulation /
> creative block` профиля на проверенном artifact, с cold+warm soak, repeat
> quorum, plugin matrix, restart/recovery, forced-ticket persistence и
> self-contained evidence bundle.

Запрещено заявлять:

- [ ] literal unlimited players/mobs/ticks
- [ ] полный Rust Paper runtime
- [ ] real-player gameplay parity
- [ ] arbitrary plugin compatibility
- [ ] green claim по старому artifact вместо текущего

## Current Artifact

- [x] `artifacts/optimized-paper-1.21.10-mojmap.jar`
  `d2b676b0d533d31d58c3b9b366aaf944197e5e26d1b7db57405b41c1463a6ae9`
- [x] `artifacts/optimized-runtime/run.sh`
  `b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a`
- [x] Старые `d4b27...` bundle/claim файлы считаются stale для текущего
  artifact.

## Fresh Red Baseline

Прогон: `load-p500-current-waitteleport-diagnostic-20260520-181858`.

- [x] Harness noise по движению убран: `moved_too_quickly_warnings=0`.
- [x] Сервер принял все логины: `server_join_events=500`.
- [x] Bot swarm дошёл до `created=500 connected=500 ready=500`.
- [x] Gate красный: `claim_eligible=false`, `gate_pass=false`.
- [x] Online plateau не достиг 500: `online_max=494`.
- [x] Bot side красный: `bot_active_max=494`, `bot_kicked_max=1`,
  `bot_errors_max=6`.
- [x] TPS/MSPT красные: `tps1_avg=9.24`, `tps1_min=2.18`,
  `avg_tick_ms_avg=132.70`, `avg_tick_ms_max=825.18`.
- [x] Блоковая нагрузка реально шла:
  `compat_probe_block_places_max=29142`,
  `compat_probe_block_breaks_max=28648`,
  `compat_probe_interact_events_max=57790`.
- [x] Stability noise не является текущей причиной:
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`,
  `stability_failures=0`.
- [x] Host caveat зафиксирован: высокий внешний load/steal был в `top`, но
  run всё равно считается красным.

## Current Hot Paths

- [x] Previous hot sample: `ServerPlayer.tick ->
  AbstractContainerMenu.broadcastChanges -> ItemStack.matches`.
- [x] New hot sample during active block phase:
  `ChunkMap.newTrackerTick -> ServerEntity.sendChanges ->
  Connection.sendPacket`, with ProtocolLib in outbound packet processing.
- [x] Вывод: следующая работа должна разделить packet/entity-tracking cost и
  container broadcast cost, а не возвращаться к старой проблеме движения.

## Next Gate Ladder

- [x] `BOT_BLOCK_MOVEMENT_MODE=wait-for-teleport` теперь является default для
  `block` и `mixed-gameplay` block-action profiles.
- [x] Добавлен harness knob `PAPER_TICK_RATE_CONTAINER_UPDATE` для
  `tick-rates.container-update`.
- [ ] Запустить P500 block diagnostic с одной переменной:
  `PAPER_OPTIMIZE_NON_FLUSH_PACKET_SENDING=true`.
- [ ] Сравнить hot stack: packet send / ProtocolLib / entity tracker.
- [ ] Запустить P500 block diagnostic с одной переменной:
  `PAPER_TICK_RATE_CONTAINER_UPDATE=4`.
- [ ] Сравнить hot stack: container broadcast / `ItemStack.matches`.
- [ ] Если обе оси дают улучшение, запустить combo diagnostic.
- [ ] Если dense arena остаётся bottleneck, отдельно проверить spatial split
  profile с тем же `500 bots / 32 view / 32 simulation / creative block`, но
  без искусственного all-players-in-one-entity-tracking-cluster эффекта.
- [ ] Если red остаётся после config-only осей, перейти к NMS/Paper patch для
  packet/entity-tracking или container broadcast fast path.

## Acceptance Gate

- [ ] `online_max >= 500` and load window starts.
- [ ] `bot_active_max >= 500`.
- [ ] `bot_kicked_max=0`.
- [ ] `bot_errors_max=0`.
- [ ] `bot_block_primed_max >= 500`.
- [ ] `bot_block_creative_slot_packets_max >= 500`.
- [ ] `tps1_avg >= 19.50`.
- [ ] `tps1_min >= 18.00`.
- [ ] `avg_tick_ms_avg <= 50.00`.
- [ ] `avg_tick_ms_max <= 100.00`.
- [ ] `watchdog_thread_dumps=0`.
- [ ] `sync_load_stack_hits=0`.
- [ ] `stability_failures=0`.
- [ ] Fresh cold+warm soak passes.
- [ ] Repeat quorum passes.
- [ ] Plugin matrix passes on the same artifact.
- [ ] Restart/recovery passes.
- [ ] Forced-ticket persistence passes.
- [ ] Evidence bundle validates and includes artifact hashes.
