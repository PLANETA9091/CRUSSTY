# Near-Unbounded Scale Goal

Дата: 2026-05-19 CEST

Status: historical/superseded. The active 2026-05-23 goal is tracked in
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md` and
`docs/REAL_GOAL_2026-05-23_P500_TO_NEAR_UNBOUNDED_EXECUTION.md`.
This file remains a roadmap archive, not current claim evidence.

Эта цель звучит страшно специально: довести ядро до состояния, где лимит
игроков, мобов, чанков и событий упирается сначала в железо и сеть, а не в
очевидные узкие места ядра. Буквально unlimited не существует: CPU, RAM,
network, disk IO и Minecraft protocol всегда конечны. Поэтому рабочая цель:
**near-unbounded measured scaling**. То есть каждый лимит должен быть либо
убран архитектурно, либо превращен в измеряемый tier-gate с честным отчетом.

## Final Claim Target

Итоговая формулировка, которую можно будет заявлять только после всех gate:

> near-unbounded production scaling for measured Minecraft player, mob,
> chunk, plugin, and world-event workloads, with repeatable tier gates,
> long-duration soak, restart/recovery, artifact hash proof, and published
> self-contained evidence bundles.

Запрещено заявлять это до полного закрытия checklist ниже.

## Historical Refresh Status

- [x] Исторический узкий 500-bot claim есть для старого verified artifact
  `d4b27d49c9aba3502b46cf75637f1fe2a4707143a1f01afbbf7315bed52b2efa`.
- [x] Тогдашний 2026-05-23 artifact snapshot для новых gate, now
  superseded and not current-artifact evidence:
  `7b5ab6911a51aaa3bad9322c4a067e485fef81d57efbe570d4c58b873c62f75c`.
- [x] Старый green production-ready refresh на `d4b27...` теперь только
  historical evidence.
- [x] Причина последнего red refresh зафиксирована: старый `1800s` soak
  слишком короткий для `300` load-window samples после `600s` ramp.
- [x] Soak wrapper теперь требует динамический floor, по умолчанию `2400s`.
- [ ] Fresh cold+warm `2400s` soak на тогдашнем artifact snapshot
  `7b5ab...` прошёл.
- [ ] Новый regenerated evidence bundle опубликован после fresh green gate.

## Historical Proven Baseline

- [x] Есть historical 2026-05-23 artifact snapshot for that проверка:
  `7b5ab6911a51aaa3bad9322c4a067e485fef81d57efbe570d4c58b873c62f75c`.
- [x] Есть historical узкий production-ready claim для `500 bots / 32 view /
  32 simulation / creative block` на старом `d4b27...` artifact.
- [ ] Есть узкий production-ready claim для `500 bots / 32 view /
  32 simulation / creative block` на тогдашнем artifact snapshot `7b5ab...`.
- [ ] Есть cold+warm soak gate для 500-bot creative block профиля на
  тогдашнем artifact snapshot `7b5ab...`.
- [ ] Есть repeat quorum на тогдашнем artifact snapshot `7b5ab...`.
- [ ] Есть plugin matrix gate на тогдашнем runtime: 11 plugins, lifecycle,
  scheduler, command, join/quit.
- [ ] Есть restart/recovery gate на тогдашнем artifact snapshot `7b5ab...`.
- [ ] Есть forced-ticket persistence gate на тогдашнем artifact snapshot
  `7b5ab...`.
- [ ] Есть self-contained evidence bundle на тогдашнем artifact snapshot
  `7b5ab...`.
- [ ] Есть stable bundle file after regeneration:
  `reports/production-500-readiness-bundle-current`.
- [ ] Есть published claim files after regeneration:
  `reports/production-500-claim-current.{txt,md,json}`.
- [x] Historical `d4b27...` repeat quorum retained only as archive.
- [x] Claim guarded by scripts:
  `scripts/validate_production_readiness_bundle.py`,
  `scripts/assert_production_ready_claim.py`,
  `scripts/publish_production_ready_claim.py`.

## Stress Corpus Baseline

This is the new "make it scary" compatibility surface. It is not a player/mob
scale claim yet; it is the corpus that future scale gates must survive.

- [x] Added a reproducible stress-corpus downloader:
  `scripts/fetch_stress_corpus.py`.
- [x] Downloaded 22 additional Modrinth plugin jars into `plugins/stress`.
- [x] Downloaded 10 heavy worldgen/structure datapacks into
  `datapacks/stress`: Terralith, Incendium, Nullscape, Structory, Tectonic,
  Dungeons and Taverns, Geophilic, Continents, Explorify, Amplified Nether.
- [x] Wrote artifact manifest with sha256 and source URLs:
  `reports/stress-corpus-artifacts.csv` and
  `reports/stress-corpus-manifest.json`.
- [x] Added descriptor/datapack inspection:
  `scripts/inspect_stress_corpus.py`.
- [x] Stress corpus inspection passed:
  `plugin_count=22`, `datapack_count=10`, `failure_count=0`.
- [x] Added a real server boot/join gate:
  `scripts/run_stress_corpus_gate.sh`.
- [x] Stress corpus gate passed with the current matrix plus stress corpus:
  `matrix_plugin_count=12`, `stress_plugin_count=22`,
  `plugin_count=34`, `datapack_count=10`, `Done (153.340s)`,
  strict held `StressProbe` join/quit, empty hard-error report, and
  `13 data pack(s) enabled`.
- [x] `run_load_test.sh` has opt-in stress corpus loading through
  `LOAD_TEST_STRESS_CORPUS=true`.
- [x] Stress corpus mixed gameplay load gate exists:
  `scripts/run_stress_mixed_load_gate.sh`.
- [x] Stress corpus mixed 50-bot baseline was run and failed honestly:
  `load_window_tps1_avg=7.96`, `load_window_tps1_min=4.13`,
  `load_window_avg_tick_ms_avg=161.33`,
  `load_window_avg_tick_ms_max=475.54`.
- [x] Real `mixed-gameplay` harness stage exists:
  `docs/MIXED_GAMEPLAY_SCALE_GOAL.md` and
  `scripts/run_stress_mixed_gameplay_gate.sh`.
- [x] Stress corpus mixed-gameplay 50-bot diagnostic was run with 26 plugins,
  10 datapacks, 150 mobs, block actions, commands, item switches, animation,
  and interact workload. It reached 50/50 bots with zero kicks, zero
  `moved_too_quickly`, zero watchdogs, zero sync-load hits, and zero mixed
  action errors, but failed the gate on TPS only:
  `load_window_tps1_avg=13.97`, `load_window_tps1_min=5.42`.
- [x] Stress corpus P100 warm-source axis diagnostics now exist through
  `scripts/run_extreme_plateau_axis_matrix.sh`.
- [x] True-idle bot mode exists:
  `BOT_SEND_STATIONARY_POSITIONS=false`.
- [x] P100 true-idle with 150 zombies reached all bots with zero position
  packets and zero watchdog/sync-load, but still failed TPS/max MSPT:
  `13.02 TPS avg`, `9.64 TPS min`, `185.88 ms max tick`.
- [x] P100 no-mob true-idle improved MSPT tail but still failed as a claim:
  `97 online max`, `14.10 TPS avg`, `9.64 TPS min`.
- [ ] Stress corpus P500 mixed gameplay gate passes.
- [ ] Stress corpus mob+worldgen mixed gate passes.
- [ ] Stress corpus long soak passes.
- [ ] Stress corpus evidence bundle is exported and validated.

## Hard Rule

- [x] No claim without gate.
- [x] No broad claim from microbench alone.
- [x] No "all plugins" claim from 11-plugin matrix.
- [x] No stress-corpus load claim from boot/join evidence alone.
- [x] No "real players" claim from synthetic creative block bots alone.
- [x] No "full Rust Paper runtime" claim.
- [ ] Every new scale claim must have a timestamped evidence bundle.
- [ ] Every new scale claim must have a stable current publication file.
- [ ] Every new scale claim must include exact non-claims.

## Definition Of Done

Near-unbounded is considered real only when all of these are true:

- [ ] The server passes all player tier gates from 500 to the selected
  hardware ceiling.
- [ ] The server passes all mob tier gates from 10k to the selected hardware
  ceiling.
- [ ] The server passes combined player+mob+chunk+plugin mixed gates.
- [ ] The server passes at least one 24h soak.
- [ ] The server passes restart/recovery during and after heavy load.
- [ ] The server has zero watchdog dumps in accepted gates.
- [ ] The server has zero sync-load stack hits in accepted gates.
- [ ] The server has bounded memory growth over long soak.
- [ ] The server has bounded queue growth under backpressure.
- [ ] The server has published evidence bundles for every accepted tier.
- [ ] The server can reject overloaded work gracefully instead of dying.

## Phase 1: Harness Expansion

Goal: stop testing only one behavior. Build a mixed gameplay harness that
looks more like real Minecraft pressure.

- [x] Add `mixed-gameplay` scenario to the load harness.
- [x] Bot movement: walking, sprinting/jump input, and bounded target orbit.
- [ ] Chunk churn: spread players over multiple regions, not one flat arena.
- [x] Block workload: place, break, interact, inventory slot changes.
- [ ] Combat workload: hit entities, receive damage, death/respawn cycle.
- [x] Mob workload: spawn and maintain mixed mobs during the gate.
- [ ] Container workload: open/close chest-like inventories.
- [x] Command workload: client command packets through `CompatProbe`.
- [ ] Teleport workload: controlled cross-chunk and cross-region teleports.
- [ ] Chat/scoreboard/team workload.
- [x] Plugin event workload: join, quit, block, command, scheduler, item held,
  animation, and interact counters.
- [x] Metrics fields for bot movement/chunk receive pressure.
- [x] Metrics fields for mixed movement/block/command/item/interact workload.
- [x] Gate fails if required mixed workload counters silently stop running.

## Phase 2: Player Scale Tiers

Goal: make player count scale by measured tiers, not by guessing.

- [x] Tier P500 creative block profile passed with published claim.
- [ ] Tier P500 mixed gameplay: 2h cold+warm gate.
- [ ] Tier P500 mixed gameplay: 24h soak.
- [ ] Tier P750 mixed gameplay: cold+warm gate.
- [ ] Tier P1000 mixed gameplay: cold+warm gate.
- [ ] Tier P1500 mixed gameplay: cold+warm gate.
- [ ] Tier P2000 mixed gameplay: cold+warm gate.
- [ ] Tier P3000 mixed gameplay if hardware allows.
- [ ] Tier P5000 mixed gameplay if hardware allows.

Per-tier requirements:

- [ ] `tps1_avg >= 19.50`.
- [ ] `tps1_min >= 18.00`.
- [ ] `avg_tick_ms_avg <= 50.00`.
- [ ] `avg_tick_ms_max <= 100.00`.
- [ ] `watchdog_thread_dumps=0`.
- [ ] `sync_load_stack_hits=0`.
- [ ] `stability_failures=0`.
- [ ] no unbounded packet queue growth.
- [ ] no unbounded chunk send backlog.
- [ ] no unbounded heap/RSS growth.

## Phase 3: Mob Scale Tiers

Goal: make mobs scale by budgets, spatial indexes, AI throttling, activation,
and measured limits.

- [ ] Tier M10k passive mobs stable gate.
- [ ] Tier M10k hostile mobs stable gate.
- [ ] Tier M25k mixed mobs stable gate.
- [ ] Tier M50k mixed mobs stable gate.
- [ ] Tier M100k mixed mobs if hardware allows.
- [ ] Mob pathfinding budget gate.
- [ ] Goal selector budget gate.
- [ ] Collision lookup budget gate.
- [ ] Target acquisition budget gate.
- [ ] Entity tracker budget gate.
- [ ] Entity activation fairness gate.
- [ ] Despawn/removal cleanup gate.
- [ ] Mob persistence save/load gate.
- [ ] Mob AI backpressure gate: overloaded AI degrades gracefully.

Required optimizations:

- [ ] Profile entity ticking under high mob count.
- [ ] Profile pathfinding queue contention.
- [ ] Replace hot O(n) entity scans where safe.
- [ ] Add or prove better spatial partitioning for hot entity paths.
- [ ] Add strict budgets for mob AI work per tick.
- [ ] Preserve plugin-visible Bukkit semantics.
- [ ] Prove no broken vanilla-critical behavior from AI throttling.

## Phase 4: Chunk, Region, And World IO Scale

Goal: chunk count and region IO must not collapse the tick loop.

- [ ] Tier C5k loaded chunks mixed gate.
- [ ] Tier C10k loaded chunks mixed gate.
- [ ] Tier C25k loaded chunks mixed gate.
- [ ] Tier C50k loaded chunks if hardware allows.
- [ ] Chunk send queue backpressure gate.
- [ ] Chunk generation queue backpressure gate.
- [ ] Region read/write latency gate.
- [ ] Save-all under load gate.
- [ ] Restart after high-chunk load gate.
- [ ] Forced chunk persistence under high load gate.
- [ ] Pregenerated world gate.
- [ ] Cold/fresh world generation gate.

Required optimizations:

- [ ] Profile sync chunk loads until zero accepted sync-load stack hits.
- [ ] Move safe IO work off the main thread.
- [ ] Bound chunk send and generation queues.
- [ ] Batch region writes safely.
- [ ] Avoid repeated remap/compression work where artifact hashes prove same
  input.
- [ ] Preserve crash consistency and world save correctness.

## Phase 5: Network And Packet Scale

Goal: thousands of clients should be limited by network bandwidth and
configured policy, not accidental packet storms.

- [ ] Packet budget per player.
- [ ] Chunk packet send-rate budget.
- [ ] Entity tracker packet budget.
- [ ] Inventory/window packet budget.
- [ ] Compression CPU budget.
- [ ] Login burst gate: 500 joins in controlled window.
- [ ] Login burst gate: 1000 joins in controlled window.
- [ ] Disconnect storm gate.
- [ ] Packet queue saturation gate.
- [ ] Slow-client backpressure gate.
- [ ] No OOM under slow clients.
- [ ] No main-thread death under packet burst.

Required optimizations:

- [ ] Profile packet encode hot paths.
- [ ] Profile compression hot paths.
- [ ] Profile chunk serialization hot paths.
- [ ] Add strict max queue policies.
- [ ] Prefer dropping/deprioritizing non-critical work over killing TPS.
- [ ] Prove protocol correctness with real client compatibility probes.

## Phase 6: Plugin Compatibility At Scale

Goal: optimization cannot break Bukkit/Paper semantics.

- [x] Current 11-plugin matrix passes.
- [ ] Expand matrix to 25 common plugins.
- [ ] Expand matrix to 50 common plugins.
- [ ] Economy/permissions/chat stack gate.
- [ ] WorldEdit/region-edit gate.
- [ ] Protection plugin gate.
- [ ] Anti-cheat style packet listener gate.
- [ ] Placeholder/scoreboard/chat formatting gate.
- [ ] Scheduler-heavy plugin gate.
- [ ] Plugin reload/restart recovery gate.
- [ ] Plugin event ordering parity gate.
- [ ] Plugin classloader/remap cache stress gate.
- [ ] Library loading compatibility gate.

Per-plugin requirements:

- [ ] No startup hard failures.
- [ ] No classloader/remap regressions.
- [ ] No event loss in probes.
- [ ] No scheduler starvation.
- [ ] No new unsupported API behavior without explicit non-claim.

## Phase 7: Memory And Leak Durability

Goal: long servers die from leaks and queues before raw TPS fails. Kill that.

- [ ] 6h mixed 500-player soak.
- [ ] 12h mixed 500-player soak.
- [ ] 24h mixed 500-player soak.
- [ ] 24h mixed player+mob soak.
- [ ] Heap trend report.
- [ ] RSS trend report.
- [ ] GC pause trend report.
- [ ] Queue-size trend report.
- [ ] Entity count trend report.
- [ ] Chunk count trend report.
- [ ] Player churn leak gate.
- [ ] Plugin churn leak gate.
- [ ] Restart after 24h soak gate.

Pass requirements:

- [ ] No monotonic unbounded heap growth.
- [ ] No monotonic unbounded RSS growth.
- [ ] No growing async queue that never drains.
- [ ] No growing packet queue that never drains.
- [ ] No growing entity tracker state after disconnect.
- [ ] No growing chunk references after unload.

## Phase 8: Real Optimization Workstreams

These are the actual engineering targets, not documentation targets.

- [ ] Entity tracker hot path optimization.
- [ ] Nearby player lookup optimization.
- [ ] Mob AI/pathfinding scheduler and budget optimization.
- [ ] Entity collision/spatial index optimization.
- [ ] Chunk send/serialization optimization.
- [ ] Chunk generation queue and worker balance optimization.
- [ ] Region IO/save batching optimization.
- [ ] Packet encoding/compression optimization.
- [ ] Plugin remap/classloader startup optimization.
- [ ] Scoreboard/team/chat broadcast optimization.
- [ ] Inventory/container event hot path optimization.
- [ ] Teleport/chunk-ticket lifecycle optimization.
- [ ] World border and forced ticket edge-case optimization.
- [ ] Shutdown/disconnect storm cleanup optimization.

Each optimization must have:

- [ ] source patch isolated cleanly.
- [ ] microbench if the shape is small enough.
- [ ] real gate before/after comparison.
- [ ] rollback if strict gate regresses.
- [ ] plugin-visible semantic check.
- [ ] artifact hash update.
- [ ] docs update.

## Phase 9: Evidence System For Near-Unbounded Claims

Goal: every scaling tier gets the same evidence discipline as current 500-bot
claim.

- [ ] `evaluate_near_unbounded_tier.py`.
- [ ] `run_near_unbounded_tier_gate.sh`.
- [ ] `export_near_unbounded_bundle.py`.
- [ ] `validate_near_unbounded_bundle.py`.
- [ ] `assert_near_unbounded_claim.py`.
- [ ] `publish_near_unbounded_claim.py`.
- [ ] Negative smoke tests for broadened claims.
- [ ] Negative smoke tests for tampered evidence.
- [ ] Negative smoke tests for weakened metrics.
- [ ] Stable publication files:
  `reports/near-unbounded-claim-current.{txt,md,json}`.

## Phase 10: Claim Ladder

The project may only climb this ladder in order.

- [x] Claim 1: production-ready for measured 500-bot 32/32 creative block
  profile.
- [ ] Claim 2: production-ready for measured 500 mixed gameplay clients.
- [ ] Claim 3: production-ready for measured 500 mixed gameplay clients for
  24h.
- [ ] Claim 4: production-ready for measured 1000 mixed gameplay clients.
- [ ] Claim 5: production-ready for measured 1000 mixed clients plus 10k mobs.
- [ ] Claim 6: production-ready for measured 2000 mixed clients plus 25k mobs.
- [ ] Claim 7: near-unbounded on this hardware class, with published hardware
  ceiling.
- [ ] Claim 8: near-unbounded across multiple hardware classes.

## Non-Claims Until Proven

- [ ] Not literally infinite players.
- [ ] Not literally infinite mobs.
- [ ] Not all plugins.
- [ ] Not all datapacks.
- [ ] Not all maps.
- [ ] Not all gameplay modes.
- [ ] Not all hardware.
- [ ] Not full Paper runtime rewritten to Rust.
- [ ] Not safe to remove gates.

## Immediate Next Work Packet

This is the first real engineering packet after this goal file:

- [x] Build a heavy stress corpus instead of testing only the old matrix.
- [x] Prove the stress corpus can boot, expose datapacks, and accept one join.
- [x] Add a stress-corpus mixed load gate wrapper.
- [x] Add `mixed-gameplay` scenario to bot harness.
- [x] Add movement + chunk receive metrics.
- [ ] Add combat + mob interaction metrics.
- [x] Add inventory + command metrics.
- [ ] Add evaluator for mixed 500-client gate.
- [x] Add evaluator for stress-corpus mixed gate.
- [x] Run baseline stress-corpus mixed gate and record the 50-bot failure.
- [x] Run baseline stress-corpus mixed-gameplay gate and record the 50-bot
  TPS-only failure.
- [ ] Run baseline mixed 500 gate.
- [ ] Identify top hot path from JFR/profile.
- [ ] Patch one real hot path.
- [ ] Rebuild optimized artifact.
- [ ] Rerun mixed 500 gate.
- [ ] Publish evidence bundle only if gate passes.

## Finish Line

The scary version of the goal is accepted only when this file has no unchecked
critical gate items in Phases 1-9 and the current published claim is no longer
the narrow 500-bot creative block profile, but a broader near-unbounded claim
with its own evidence bundle.
