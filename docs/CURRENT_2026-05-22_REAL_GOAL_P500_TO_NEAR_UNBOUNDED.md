# Real Goal: P500 Recovery To Near-Unbounded Scale

Дата: 2026-05-22 CEST

Status: historical/superseded. The active 2026-05-23 goal is
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md`.
This file keeps the 2026-05-22 state only; it is not current claim evidence.

Это была реальная цель для `/root/rust` на 2026-05-22.

Фраза "unlimited players/mobs/ticks/chunks" не является честным claim:
железо, сеть, диск, JVM, протокол Minecraft и клиентские соединения конечны.
Реальная цель - **near-unbounded measured scale**: убрать искусственные
потолки ядра и поднимать измеренные tier-ы до момента, где первый лимит уже
явно CPU/RAM/network/disk/policy, а не известный hot path ядра.

Первый обязательный шаг на 2026-05-22: восстановить production-ready claim
для P500 creative-block профиля на тогдашнем artifact snapshot. Пока это не сделано, P750,
P1000, mobs, worldgen и stress corpus могут быть только diagnostic, не claim.

Плагины и датапаки не обновляются, не чинятся и не оптимизируются ради
результата. Они используются как нагрузочная совместимость. Оптимизация должна
быть в ядре, Paper runtime, Java hot paths, Rust/JNI native modules,
chunk/entity/network backpressure и evidence tooling.

## Allowed Claim For Phase 1

Разрешённая формулировка только после fresh green bundle:

> production-ready для измеренного 500 bots / 32 view / 32 simulation /
> creative block профиля на 2026-05-22 optimized artifact snapshot
> `b0bfd02e5f4d90826495613f5eb49b35c4b3a8b2e54de71e25b81f4e3f64d91b`, с
> cold+warm soak, repeat quorum, plugin matrix, restart/recovery,
> forced-ticket persistence, runtime/native hash proof и валидируемым
> self-contained evidence bundle.
> Эта формулировка историческая; она не означает current-artifact evidence now.

Запрещённые claim-ы без отдельного evidence:

- [x] literal unlimited players, mobs, chunks, ticks, plugins или datapacks.
- [x] полный Rust Paper runtime.
- [x] real-player gameplay parity.
- [x] arbitrary plugin/datapack compatibility.
- [x] claim от stale artifact или stale bundle.
- [x] claim от noisy host, host contention или timer-open block workload.
- [x] claim от microbench/native parity без strict server gate.

## Historical Truth

- [x] Пользователь разрешил принять Minecraft EULA для локального стенда.
- [x] Исторический P500 creative-block production claim существовал для
  старого artifact snapshot `d4b27d49c9aba3502b46cf75637f1fe2a4707143a1f01afbbf7315bed52b2efa`.
- [x] Этот исторический claim теперь stale и сохранён только как архивная
  ссылка для актуального state.
- [x] На момент обновления `reports/artifacts.json` указывает тогдашний
  optimized artifact snapshot
  `b0bfd02e5f4d90826495613f5eb49b35c4b3a8b2e54de71e25b81f4e3f64d91b`.
- [x] `reports/production-500-readiness-bundle-current` is still stale for the
  then-recorded artifact snapshot; validator/assert/freshness smokes pass and
  keep stale publication blocked.
- [x] Последний fresh-by-default strict P500 retry
  `reports/production-500-readiness-gate-retry-20260523-023251.txt` exhausted
  after 4 attempts; он не считается production evidence, потому что все
  попытки были environment-invalid на host contention до 500/500 online.
- [x] Последний strict P500 soak attempt
  `load-production-500-cold-soak-current-artifact-20260523-024408` не
  считается: stable 60s host-ready preflight был green, но in-run watcher
  aborted on environment-invalid host contention.
- [x] Early abort reason:
  `host_contention_bad_samples=3_load_per_cpu=1.046_max_load_per_cpu=0.750_steal_percent=49.12_max_steal_percent=10.00_iowait_percent=0.32_max_iowait_percent=10.00`.
- [x] Итоговая summary: `bot_created_max=20`, `bot_connected_max=10`,
  `load_window_reached_full_online=false`, `host_cpu_steal_percent_max=49.59`,
  `host_system_load1_per_cpu_max=1.047`.
- [x] Noisy diagnostics с 50/100 bots полезны только для поиска hot path, но
  не дают production claim.
- [x] Validator уже режет stale bundle, missing runtime hash, missing
  native hash proof и artifact-drift.
- [x] `run_load_test.sh` пишет `optimized_runtime_native_library_sha256` в
  новые summaries.
- [ ] P500 claim восстановлен на artifact `b0bfd...` или более новом явно
  зафиксированном artifact snapshot.

## Phase 0 - Evidence First

- [x] Artifact hashes записываются через `reports/artifacts.json` и
  `reports/artifact-hashes.txt`.
- [x] Bundle validator требует current freshness для
  `production-500-readiness-bundle-current`.
- [x] Bundle validator требует `optimized_runtime_run_sh_sha256`.
- [x] Bundle validator требует `optimized_runtime_native_library_sha256`.
- [x] Bundle validator требует referenced logs из plugin/restart/forced-ticket
  summaries.
- [x] Обновить top state docs, чтобы они не называли `d4b27...`
  активным claim.
- [x] `sha256sum -c reports/artifact-hashes.txt` проходит для тогдашнего
  artifact/runtime/native/stress corpus.
- [x] Current bundle validation/assert/freshness smokes pass and keep stale
  publication blocked.
- [x] `scripts/run_production_readiness_gate_retry.sh` добавлен как fresh-by-default
  production retry wrapper: `PRODUCTION_READINESS_REFRESH_SOAK=true`,
  `PRODUCTION_READINESS_REFRESH_REPEAT=true`,
  `PRODUCTION_READINESS_REFRESH_COMPAT=true`, `PRODUCTION_RELEASE_REPEAT_COUNT=3`,
  retry только на `host_contention` / `environment-invalid`.
- [x] `scripts/run_production_readiness_gate_retry_smoke.sh` проходит.
- [ ] После следующего fresh-artifact P500 gate пересобрать
  `reports/production-500-readiness-bundle-current`.
- [ ] `python3 scripts/validate_production_readiness_bundle.py
  reports/production-500-readiness-bundle-current --require-current-freshness`
  проходит без failures.
- [ ] `python3 scripts/assert_production_ready_claim.py
  reports/production-500-readiness-bundle-current` проходит без failures.

## Phase 1 - Restore P500 Creative Block Claim

- [ ] Перед тяжёлым запуском host preflight green:
  `steal_percent <= 10`, `iowait_percent <= 10`, `load_per_cpu <= 0.750`.
- [x] Production launch требует устойчивое host-ready окно, а не один
  случайно чистый sample перед стартом.
- [ ] Strict foreign-process gate green.
- [ ] Production block workload запускается только через `all-ready` action
  gate, не через timer-open.
- [ ] Required bots ready/active/settled/block-armed count equals bot count.
- [ ] P500 cold/fresh reaches 500/500 online and full block workload.
- [ ] P500 cold/fresh has `tps1_avg >= 19.50`.
- [ ] P500 cold/fresh has `tps1_min >= 18.00`.
- [ ] P500 cold/fresh has `avg_tick_ms_avg <= 50.00`.
- [ ] P500 cold/fresh has `avg_tick_ms_max <= 100.00`.
- [ ] P500 warm-source reaches 500/500 online and full block workload.
- [ ] P500 warm-source has `tps1_avg >= 19.50`.
- [ ] P500 warm-source has `tps1_min >= 18.00`.
- [ ] P500 warm-source has `avg_tick_ms_avg <= 50.00`.
- [ ] P500 warm-source has `avg_tick_ms_max <= 100.00`.
- [ ] Accepted runs have `watchdog_thread_dumps=0`.
- [ ] Accepted runs have `sync_load_stack_hits=0`.
- [ ] Accepted runs have `stability_failures=0`.
- [ ] Accepted runs have no bot kicks/errors in the claim window.
- [ ] Cold+warm soak green on the same artifact.
- [ ] Repeat quorum green on the same artifact.
- [ ] Plugin matrix green on the same artifact.
- [ ] Restart/recovery green on the same artifact.
- [ ] Forced-ticket persistence green on the same artifact.
- [ ] Self-contained evidence bundle validates.
- [ ] Published claim files are refreshed only after validation passes.

## Phase 2 - Core Optimization Loop

- [x] Native Climate RTree hook exists with Java fallback and runtime smoke.
- [x] Native NormalNoise is enabled and wired into `NormalNoise.getValue`,
  `fillPositions`, `fillVertical`, and `fillCell`.
- [x] Native Perlin/ImprovedNoise paths are enabled in optimized runtime.
- [ ] For every red same-artifact P500 run, classify the limiter:
  host, join, packet, entity tracking, block events, chunk generation,
  chunk send, lighting, IO, memory, scheduler, plugin hook, native hook.
- [ ] If `NormalNoise.getValue` remains hot, reduce JNI batch overhead before
  accepting more broad changes.
- [ ] If chunk/entity/network paths dominate, pivot to that subsystem with a
  targeted patch and strict gate.
- [ ] No Rust/JNI replacement is accepted on parity alone.
- [ ] No microbench win is accepted without a same-artifact server gate.
- [ ] Every accepted patch survives rebuild, runtime smoke, hash refresh and
  strict load gate.

## Phase 3 - Stress Corpus Mixed Gameplay

- [x] Stress plugin/datapack corpus exists and is inspectable.
- [x] Mixed gameplay harness exists.
- [ ] P100 mixed gameplay with stress corpus passes fresh-world gate.
- [ ] P250 mixed gameplay with stress corpus passes fresh-world gate.
- [ ] P500 mixed gameplay with stress corpus reaches full online.
- [ ] P500 mixed gameplay passes TPS/MSPT/no-watchdog/no-sync-load gate.
- [ ] Mixed workload includes movement, block actions, item held, animation,
  interact, commands, plugin counters and mob pressure.
- [ ] Datapack/worldgen output is not changed to pass the test.
- [ ] Plugin-visible Bukkit behavior remains intact.
- [ ] Stress-corpus bundle validates before any stress-corpus claim.

## Phase 4 - Near-Unbounded Ladder

Каждый tier ниже diagnostic, пока не прошёл full gate, soak, repeat,
recovery и bundle.

- [ ] P750 mixed gameplay diagnostic.
- [ ] P1000 mixed gameplay diagnostic.
- [ ] P1500 mixed gameplay diagnostic.
- [ ] P2000 mixed gameplay diagnostic.
- [ ] P3000/P5000 only if hardware ceiling allows.
- [ ] M1k mixed mobs gate with accepted player tier.
- [ ] M5k mixed mobs gate with accepted player tier.
- [ ] M10k mixed mobs diagnostic.
- [ ] M25k mixed mobs diagnostic.
- [ ] C10k loaded chunks diagnostic with forced-ticket evidence.
- [ ] C25k loaded chunks diagnostic with forced-ticket evidence.
- [ ] Stress datapack worldgen gate with unchanged datapacks.
- [ ] Chunk generation queue backpressure evidence.
- [ ] Chunk send queue backpressure evidence.
- [ ] Entity tracker budget evidence.
- [ ] Pathfinding/goal-selector budget evidence.
- [ ] Slow-client and packet-burst backpressure gates.
- [ ] 24h soak for the highest accepted tier.

## Definition Of Done

- [ ] The latest recorded artifact has a validated P500 creative-block claim.
- [ ] The latest accepted mixed gameplay tier has fresh bundle evidence.
- [ ] The latest accepted mob tier has Bukkit-visible behavior evidence.
- [ ] The latest accepted chunk/worldgen tier has forced-ticket and IO evidence.
- [ ] Every stale bundle is either replaced or clearly marked historical.
- [ ] Every accepted tier has exact non-claims.
- [ ] Every remaining ceiling is tied to measured CPU/RAM/network/disk/policy,
  not an unexamined core bottleneck.

## Immediate Execution

- [x] Create this real goal file.
- [x] Refresh `docs/STATE.md` top section to point at this goal.
- [x] Run cheap recorded-artifact hash validation.
- [x] Run bundle validation and keep the exact failure list.
- [ ] If the failure list is only stale evidence, rerun the production
  readiness gate on a quiet host through
  `MC_EULA_AGREE=true ./scripts/run_production_readiness_gate_retry.sh`.
- [ ] If the failure list includes harness bugs, fix harness before more huge
  runs.
- [ ] Keep subagents on independent tracks: evidence, hot path, and docs.
