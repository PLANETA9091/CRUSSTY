# Реальная цель production-ready scale

Дата: 2026-05-21 CEST

Эта цель фиксирует текущий честный путь к claim уровня "production-ready" для
ядра `/root/rust`. Мы не оптимизируем плагины и датапаки под тест. Оптимизация
должна быть в ядре, runtime, harness и доказательном процессе.

Важно: literal unlimited не является claim. Практическая цель - убрать
искусственные потолки ядра и поднимать измеренные tier-ы до тех пор, пока
первым лимитом не станет железо, сеть, диск или явно описанная политика.

## Разрешенные будущие claim-ы

Разрешенный claim после прохождения gate:

> production-ready для измеренного tier-а на проверенном artifact, с точным
> количеством bots/players, view distance, simulation distance, workload,
> plugin/datapack corpus, worldgen scope, mob/chunk pressure, cold+warm soak,
> repeat quorum, restart/recovery, forced-ticket persistence, artifact hashes
> и self-contained evidence bundle.

Запрещено заявлять без отдельного fresh evidence:

- [x] literal unlimited players, mobs, chunks, ticks, plugins или datapacks.
- [x] полный Rust Paper runtime.
- [x] arbitrary plugin compatibility.
- [x] arbitrary datapack/worldgen compatibility.
- [x] real-player gameplay parity.
- [x] claim от stale artifact или stale bundle.

## Текущий статус

- [x] Пользователь разрешил принять Minecraft EULA для локального стенда.
- [x] `eula.txt` в этом workspace выставлен в `eula=true`.
- [x] Исторический narrow claim `500 bots / 32 view / 32 simulation /
  creative block` существовал для старого artifact.
- [x] Старый
  `reports/production-500-readiness-bundle-current` теперь считается stale для
  текущего artifact.
- [x] Последний current-artifact P500 run дошел до `500` online.
- [x] Последний current-artifact P500 run не может быть claim, потому что
  failure analysis зафиксировал host contention:
  `steal_max=52.58 > 10.00` и `iowait_max=65.25 > 10.00`.
- [x] Evidence validators требуют `optimized_runtime_run_sh_sha256` и native
  library hash, чтобы bundle не мог пройти без hash proof runtime launcher и
  bundled `.so`.
- [ ] Current artifact имеет fresh green P500 cold soak.
- [ ] Current artifact имеет fresh green P500 warm soak.
- [ ] Current artifact имеет fresh green repeat quorum.
- [ ] Current artifact имеет fresh green plugin matrix.
- [ ] Current artifact имеет fresh green restart/recovery.
- [ ] Current artifact имеет fresh green forced-ticket persistence.
- [ ] Current artifact имеет validated self-contained evidence bundle.
- [ ] Можно честно опубликовать P500 production-ready claim.
- [x] Immediate next measured step зафиксирован отдельно:
  `docs/NEXT_2026-05-21_P250_WARM_SOURCE_RECOVERY_GOAL.md`.

## Phase 0 - чистота gate и host

- [x] Host CPU steal/iowait попадает в failure analysis.
- [x] Перед тяжелым прогоном preflight блокирует запуск при чужих Java/server
  процессах, которые могут испортить измерение. Smoke:
  `scripts/run_load_test_strict_foreign_process_gate_smoke.sh`.
- [x] Перед тяжелым прогоном preflight блокирует запуск при уже высоком
  steal/iowait/load.
- [x] Во время тяжелого прогона sustained host-contention watcher завершает
  contaminated run и пишет `early_abort_reason=host_contention`. Smoke:
  `scripts/run_load_test_host_contention_watcher_smoke.sh`.
- [x] Gate явно пишет, что run invalid из-за host contention, а не из-за
  server/runtime regression, когда resources/preflight превышают лимиты.
- [ ] Evidence bundle включает preflight, resources CSV, summary, logs, gate,
  artifact hashes, runtime hash и native library hash.

## Phase 1 - восстановить current-artifact P500

- [ ] Включить low-risk join-path patch `0142-Avoid-join-player-info-
  snapshot-allocation` в активный build tree, если он еще не включен.
- [ ] Пересобрать optimized Paper artifact после `0142`.
- [ ] Зафиксировать release policy для `area_map`: сейчас optimized runtime
  включает `paper.nativeAreaMap` автоматически при наличии bundled native
  library; если это не должно идти в release, явно отключить через
  `PAPER_NATIVE_AREA_MAP=false` или изменить launcher policy.
- [ ] Обновить artifact hash report после rebuild, including native library
  hash.
- [ ] Запустить fresh P500 readiness gate только на тихом host.
- [ ] Если gate red - классифицировать limiting subsystem: join, packet,
  entity tracking, container broadcast, chunk task, lighting, IO, memory,
  scheduler, plugin hook, native hook или host.

## Phase 2 - production-ready для P500 32/32

- [ ] `500 bots / 32 view / 32 simulation / creative block` достигает полного
  online без bot errors и kicks.
- [ ] `tps1_avg >= 19.50`.
- [ ] `tps1_min >= 18.00`.
- [ ] `avg_tick_ms_avg <= 50.00`.
- [ ] `avg_tick_ms_max <= 100.00`, либо более строгий/явный spike budget в
  claim text.
- [ ] `watchdog_thread_dumps=0`.
- [ ] `sync_load_stack_hits=0`.
- [ ] `stability_failures=0`.
- [ ] Cold+warm soak green на том же artifact.
- [ ] Repeat quorum green на том же artifact.
- [ ] Plugin matrix green на том же artifact.
- [ ] Restart/recovery green на том же artifact.
- [ ] Forced-ticket persistence green на том же artifact.
- [ ] Bundle validation green на том же artifact.

## Phase 3 - тяжелый mixed gameplay corpus

- [ ] P500 mixed profile: movement, block place/break, held item switch,
  arm animation, use-item, commands, plugin counters, datapacks и mob pressure.
- [ ] Plugin corpus не изменяется ради результата, только фиксируется и
  валидируется.
- [ ] Datapack corpus не изменяется ради результата, только фиксируется и
  валидируется.
- [ ] Worldgen output не меняется поведенчески и не ломает datapack/plugin
  generation hooks.
- [ ] Mob AI/entity tracking имеет отдельные counters и failure budget.
- [ ] Packet/container broadcast pressure имеет отдельные counters.
- [ ] Chunk generation/loading pressure имеет отдельные counters.
- [ ] Mixed P500 проходит cold+warm/repeat/recovery/bundle gates.

## Phase 4 - near-unbounded measured ladder

Каждый tier ниже является diagnostic, пока не пройдет полный production gate.

- [ ] P750 mixed diagnostic.
- [ ] P1000 mixed diagnostic.
- [ ] P1500 mixed diagnostic.
- [ ] P2000 mixed diagnostic.
- [ ] P3000 hardware-ceiling diagnostic.
- [ ] P5000 hardware-ceiling diagnostic.
- [ ] M10k mobs diagnostic with accepted player tier.
- [ ] M25k mobs diagnostic with accepted player tier.
- [ ] C10k loaded chunks diagnostic with forced-ticket evidence.
- [ ] C25k loaded chunks diagnostic with forced-ticket evidence.
- [ ] 24h soak на самом высоком accepted tier.

## Definition of done

- [ ] Нет stale evidence.
- [ ] Нет claim без fresh gate.
- [ ] Нет green claim при host contention.
- [ ] Нет plugin/datapack "оптимизации" вместо core optimization.
- [ ] Все изменения имеют compile/test/smoke evidence.
- [ ] Production claim публикуется только через свежий validated bundle.
