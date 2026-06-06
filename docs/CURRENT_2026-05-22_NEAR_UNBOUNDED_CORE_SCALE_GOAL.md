# Current Near-Unbounded Core Scale Goal

Дата: 2026-05-22 CEST

Это активная большая цель после current-artifact refresh. Цель звучит как
"сервер выдерживает почти всё", но формально это не literal unlimited:
железо, сеть, диск, JVM, протокол Minecraft и клиентские соединения всегда
конечны. Практическая цель: убрать искусственные потолки ядра и довести каждый
оставшийся лимит до измеряемого hardware/network/disk ceiling.

Плагины и датапаки здесь не оптимизируются и не переписываются. Они только
используются как нагрузочная совместимость. Оптимизация должна быть в ядре,
Paper runtime, Java hot paths, Rust/JNI native modules, chunk/entity/network
backpressure и evidence tooling.

## Current Truth

- [x] Ложный claim запрещён: текущий `500 bots / 32 view / 32 simulation /
  creative block` claim на current artifact ещё не восстановлен.
- [x] Последний strict current run
  `load-production-500-block-500bots-current-20260522-030956` был прерван
  host contention: `steal_percent=47.63`, `load_per_cpu=0.785`.
- [x] Последний noisy diagnostic
  `load-noisy-diagnostic-50block-current-20260522-034756` дошёл до `50/50`
  ботов и реального block workload, но тоже загрязнён host steal:
  `host_cpu_steal_percent_avg=41.49`, `max=49.80`.
- [x] Нельзя засчитывать noisy host как production proof.
- [x] Текущий `reports/production-500-readiness-bundle-current` невалиден:
  stale bundle, current artifact hash drift, missing runtime/native hash proof
  and missing referenced logs block publication.
- [x] Нельзя менять, обновлять или "чинить" плагины/датапаки ради результата.
- [x] Нельзя заявлять полный Rust Paper runtime.
- [x] Нельзя заявлять unlimited players/mobs/ticks/chunks без tier evidence.

## Active Core Work

- [x] Native Rust/JNI library already contains Climate RTree exported symbols.
- [x] Current artifact reports `native_area_map=true`,
  `native_improved_noise=true`, `native_perlin_noise_no_y_scale=true`.
- [x] Add guarded runtime hook for `Climate.RTree` in Paper source.
- [x] Verify Java/native Climate RTree checksum parity at runtime build.
- [x] Rebuild optimized Paper artifact after the hook.
- [x] Rebuild optimized runtime and confirm
  `native_climate_rtree_hook_present=true`.
- [x] Confirm launcher enables `native_climate_rtree=true` only when the class
  is present and native library is bundled.
- [x] Keep Java fallback on native load/build/search failure.
- [x] Add or regenerate feature patch so the hook survives `applyPatches`.

## Current Climate RTree Hook Evidence

- [x] `git apply --check` passes for
  `0144-Enable-native-Climate-RTree-runtime-hook.patch`.
- [x] `MC_EULA_AGREE=true ./scripts/build_optimized.sh` passes after the hook.
- [x] Runtime jar contains
  `net/minecraft/world/level/biome/PaperNativeClimateRTree.class`.
- [x] `artifacts/optimized-runtime/run.sh` has
  `PAPER_NATIVE_CLIMATE_RTREE_DEFAULT="auto"` and
  `PAPER_NATIVE_CLIMATE_RTREE_HOOK_PRESENT="true"`.
- [x] Wrapper dry-run emits `native_climate_rtree=true` and
  `native_climate_rtree_hook_present=true`.
- [x] `nm` and `javap` confirm the five production JNI methods:
  build, free, checksum, current packed search, bounded packed search.
- [x] `reports/native-climate-rtree-jni-bench.txt` has
  `native_rtree_equivalence=PASS` and
  `native_rtree_packed_equivalence=PASS`.
- [x] `reports/native-climate-rtree-build-bench.txt` has
  `equivalence=PASS` and matching Java/native tree checksums.
- [x] `reports/native-climate-rtree-lifecycle-bench.txt` has
  `java_native_lifecycle_equivalence=PASS`.
- [x] `reports/native-climate-rtree-runtime-smoke-20260522.txt` starts the
  current artifact with `Paper: Using native Climate RTree from
  paper_native_jni.`, exits cleanly, and has no
  `Native Climate RTree unavailable` or `checksum mismatch`.
- [x] `MC_EULA_AGREE=true ./scripts/native_climate_rtree_fallback_smoke.sh`
  starts the current artifact with a deliberately incomplete
  `libpaper_native_jni.so`, logs `Paper: Native Climate RTree unavailable,
  using Java fallback`, exits cleanly, and writes
  `reports/native-climate-rtree-fallback-smoke.txt`.
- [x] This evidence is not a `500 bots`, unlimited-player, full-Rust-runtime,
  or multi-hour soak claim.

## Player Scale Ladder

- [ ] P500 creative block current artifact strict gate passes on quiet host.
- [ ] P500 creative block cold+warm soak passes on current artifact.
- [ ] P500 creative block repeat quorum passes on current artifact.
- [ ] P500 creative block restart/recovery passes on current artifact.
- [ ] P500 creative block forced-ticket persistence passes on current artifact.
- [ ] P500 creative block self-contained evidence bundle validates.
- [ ] P500 mixed gameplay with stress corpus reaches full online.
- [ ] P500 mixed gameplay passes TPS/MSPT/no-watchdog/no-sync-load gate.
- [ ] P750 mixed gameplay diagnostic exists.
- [ ] P1000 mixed gameplay diagnostic exists.
- [ ] P1500/P2000 diagnostic only until hardware ceiling is measured.

## Mob And Entity Scale Ladder

- [ ] M1k mixed mobs gate with accepted player tier.
- [ ] M5k mixed mobs gate with accepted player tier.
- [ ] M10k mixed mobs diagnostic.
- [ ] Pathfinding queue budget evidence.
- [ ] Goal selector budget evidence.
- [ ] Collision lookup budget evidence.
- [ ] Entity tracker budget evidence.
- [ ] Despawn/removal cleanup budget evidence.
- [ ] Plugin-visible Bukkit semantics preserved for entity events.

## Worldgen And Chunk Scale Ladder

- [ ] Stress datapack worldgen gate with unchanged datapacks.
- [ ] Chunk generation queue backpressure evidence.
- [ ] Chunk send queue backpressure evidence.
- [ ] Forced-ticket persistence under high chunk pressure.
- [ ] No sync chunk loads in accepted gates.
- [ ] No unbounded region-file IO backlog in accepted gates.
- [ ] Native worldgen hooks accepted only after parity and strict gate data.

## Evidence Requirements

- [x] Artifact hashes recorded after every accepted rebuild.
- [x] `sha256sum -c reports/artifact-hashes.txt` passes after rebuild.
- [x] Production action-gate config regression is blocked before a server run:
  `scripts/run_load_test_production_action_gate_smoke.sh` rejects timer mode,
  settle windows below 15s, missing block-armed readiness, and min-count below
  bot count for `production-*` block profiles. The smoke is wired into
  `scripts/run_production_readiness_gate.sh`.
- [ ] Summary, gate report, logs, resources CSV, and runner metadata exist for
  every claimed tier.
- [ ] Production block-action workload starts through `all-ready` action gate
  only after every required bot is ready, active, settled, block-armed, and
  stable for the required settle window; timer-open summaries are invalid.
- [ ] Host contention thresholds are clean for production claims:
  `steal_percent <= 10`, `iowait_percent <= 10`,
  `load_per_cpu <= 0.750` unless explicitly marked diagnostic only.
- [ ] Accepted gate has `watchdog_thread_dumps=0`.
- [ ] Accepted gate has `sync_load_stack_hits=0`.
- [ ] Accepted gate has `stability_failures=0`.
- [ ] Accepted gate has bounded RSS/heap/queue growth.
- [ ] Evidence bundle validator passes.
- [ ] Claim publisher includes exact non-claims.

## Definition Of Done

- [ ] Highest accepted player tier has cold+warm soak, repeat quorum,
  restart/recovery, forced-ticket persistence, plugin matrix, and bundle.
- [ ] Highest accepted mixed tier survives plugins/datapacks without changing
  them.
- [ ] Highest accepted mob tier survives with Bukkit-visible behavior intact.
- [ ] Every remaining ceiling is tied to measured CPU/RAM/network/disk limits,
  not a known avoidable core hot path.
- [ ] Final wording says exactly what was measured and what was not measured.

## Working Rule

- [x] Low resources do not end the work.
- [x] Low resources mean: optimize core hot paths, add backpressure, and rerun
  when host noise is clean.
- [x] No box becomes `[x]` because of intent, microbench only, or noisy-host
  hope. It becomes `[x]` only after current-artifact evidence exists.
