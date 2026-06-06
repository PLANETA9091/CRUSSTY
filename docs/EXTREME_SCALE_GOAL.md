# Extreme Scale Goal

Date: 2026-05-18 CEST

Status: historical/superseded roadmap. The superseding 2026-05-23 claim source is
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md`.

This file is the hard target for the scary version of the project. The goal is
not a fake "infinite server" claim. The goal is to remove artificial ceilings,
make every remaining ceiling measurable, and push players, mobs, chunks,
plugins, datapacks, packets, and worldgen until the first limit is hardware,
network, disk, or explicit policy.

## Final Claim Shape

Allowed only after the checklist below is green:

> production-ready for measured extreme Minecraft scale on the verified
> artifact, with no known artificial core ceiling before hardware limits,
> tiered player/mob/chunk/plugin/worldgen evidence, cold and warm profiles,
> restart/recovery, long soak, backpressure behavior, and self-contained
> published evidence bundles.

This is still not a claim for literally infinite players, infinite mobs,
infinite chunks, unlimited plugins, unlimited datapacks, all maps, all
hardware, or a full Rust rewrite of the Paper runtime.

## Current Hard Baseline

- [x] Historical narrow production claim exists for measured `500 bots / 32
  view / 32 simulation / creative block` on an older artifact.
- [ ] Superseded 2026-05-23 artifact snapshot `7b5ab...` did not have a
  fresh 500-bot claim, repeat quorum, restart/recovery, forced-ticket
  persistence, artifact hash proof, or self-contained regenerated bundle.
- [x] Heavy stress corpus exists: 34 plugin jars total when matrix and stress
  jars are combined.
- [x] Heavy datapack corpus exists: 10 stress datapacks.
- [x] Stress corpus boot/join gate passes with 34 plugin jars, `StressProbe`
  held join/quit, empty hard-error report, and datapacks enabled.
- [x] Stress mixed load gate exists:
  `scripts/run_stress_mixed_load_gate.sh`.
- [x] Stress mixed evaluator exists through the `stress-mixed` profile in
  `scripts/evaluate_load_gate.py`.
- [x] Stress mixed 50-bot baseline has been run and is documented as a
  failure, not a claim.
- [x] The 50-bot stress mixed failure reached all 50 bots with 150 spawned
  zombies and 26 plugin jars plus 10 datapacks.
- [x] The 50-bot stress mixed failure had zero watchdog dumps, zero sync-load
  stack hits, zero nearby-player stack hits, and zero stability failures.
- [x] A later cold-fresh DFC-bridge diagnostic isolated a datapack startup
  hotspot to `/forceload` command-path sync chunk loading, and the command now
  stays on the ticket update path.
- [x] Stress mixed 100-bot diagnostic has a current report.
- [x] Stress mixed 100-bot diagnostic failed honestly with full corpus,
  fresh worldgen pressure, watchdog dumps, and sync-load hits.
- [x] Stress mixed 100-bot warm-source diagnostic exists and reached all 100
  bots, but still failed TPS/MSPT/watchdog/sync-load requirements.
- [x] P100 warm plateau axis diagnostics exist for slow movement, true idle,
  and no-mob true idle.
- [x] The load harness can now run true idle clients through
  `BOT_SEND_STATIONARY_POSITIONS=false`, so parked diagnostics no longer fake
  idling by sending position packets.
- [x] Stress mixed 50-bot gate passed on the then-fresh artifact snapshot.
- [x] Stress mixed 250-bot diagnostic has a current report.
- [x] Stress mixed 250-bot slow-move axis diagnostic has a current report.
- [ ] Stress mixed 500-bot diagnostic has a current report.
- [ ] Stress mixed 500-bot gate passes.
- [ ] Stress mixed 1000-bot diagnostic has a current report.
- [ ] Stress mixed 1000-bot gate passes.

Current measured stress mixed 50-bot pass:

- `load_window_tps1_avg=18.33`, required `18.00`.
- `load_window_tps1_min=15.88`, required `15.00`.
- `load_window_avg_tick_ms_avg=26.84`, allowed `75.00`.
- `load_window_avg_tick_ms_max=78.32`, allowed `150.00`.
- `process_rss_mib_max=5503.3`.
- `watchdog_thread_dumps=0`.
- `sync_load_stack_hits=0`.
- `stability_failures=0`.

Historical measured stress mixed 50-bot failure before the `/forceload`
command-path fix:

- `load_window_tps1_avg=7.96`, required `18.00`.
- `load_window_tps1_min=4.13`, required `15.00`.
- `load_window_avg_tick_ms_avg=161.33`, allowed `75.00`.
- `load_window_avg_tick_ms_max=475.54`, allowed `150.00`.
- `process_rss_mib_max=24947.1`.

The native-improved diagnostic was better but still failed:

- `load_window_tps1_avg=10.42`, required `18.00`.
- `load_window_tps1_min=4.69`, required `15.00`.
- `load_window_avg_tick_ms_avg=75.87`, allowed `75.00`.
- `load_window_avg_tick_ms_max=197.81`, allowed `150.00`.

Current measured stress mixed 100-bot failure:

- `bot_connected_max=76`, `bot_ready_max=76`, `bot_active_max=76`.
- `load_window_reached_full_online=false`.
- `load_window_online_max=67`.
- `load_window_tps1_avg=9.38`, required `18.00`.
- `load_window_tps1_min=4.74`, required `15.00`.
- `load_window_avg_tick_ms_avg=71.67`, allowed `75.00`.
- `load_window_avg_tick_ms_max=141.90`, allowed `150.00`.
- `watchdog_thread_dumps=8`.
- `sync_load_stack_hits=6`.
- `moved_too_quickly_warnings=55`.

Current measured stress mixed 100-bot native-improved failure:

- Runtime flags included `paper.nativeImprovedNoise=true`.
- `bot_connected_max=78`, `bot_ready_max=78`, `bot_active_max=78`.
- `load_window_reached_full_online=false`.
- `load_window_online_max=66`.
- `load_window_tps1_avg=7.27`, required `18.00`.
- `load_window_tps1_min=2.45`, required `15.00`.
- `load_window_avg_tick_ms_avg=72.19`, allowed `75.00`.
- `load_window_avg_tick_ms_max=127.28`, allowed `150.00`.
- `watchdog_thread_dumps=7`.
- `sync_load_stack_hits=4`.
- `nearby_players_stack_hits=1`.
- Result: native improved noise is not an accepted fix for the extreme P100
  profile.

Current measured stress mixed 100-bot warm-source failure:

- Source world: `runs/load-extreme-stress-mixed-100-20260517-121716`.
- `bot_connected_max=100`, `bot_ready_max=100`, `bot_active_max=100`.
- `load_window_reached_full_online=true`.
- `load_window_online_max=100`.
- `load_window_tps1_avg=14.10`, required `18.00`.
- `load_window_tps1_min=6.23`, required `15.00`.
- `load_window_avg_tick_ms_avg=157.99`, allowed `75.00`.
- `load_window_avg_tick_ms_max=629.12`, allowed `150.00`.
- `watchdog_thread_dumps=2`.
- `sync_load_stack_hits=2`.
- Decision: warm world removes the P100 join collapse, but not the plateau
  cost.

Current P100 warm plateau axis diagnostics:

- `slow-move`: `speed=12`, `move_interval=500ms`, `150 zombies`,
  `online_max=100`, `tps1_avg=12.75`, `tps1_min=3.12`,
  `avg_tick_ms_avg=235.18`, `avg_tick_ms_max=1441.92`,
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`.
- `true-idle parked`: `speed=0`,
  `BOT_SEND_STATIONARY_POSITIONS=false`, `150 zombies`,
  `bot_position_packets_max=0`, `moved_too_quickly_warnings=0`,
  `online_max=100`, `tps1_avg=13.02`, `tps1_min=9.64`,
  `avg_tick_ms_avg=60.82`, `avg_tick_ms_max=185.88`,
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`.
- `no-mobs true-idle parked`: `speed=0`,
  `BOT_SEND_STATIONARY_POSITIONS=false`, `0 requested zombies`,
  `bot_position_packets_max=0`, `moved_too_quickly_warnings=0`,
  `online_max=97`, `bot_ready_max=100`, `bot_errors_max=3`,
  `tps1_avg=14.10`, `tps1_min=9.64`, `avg_tick_ms_avg=56.71`,
  `avg_tick_ms_max=134.71`.
- Decision: movement packet spam is not the main P100 blocker. Mobs worsen
  MSPT tail, but even no-mob true-idle still has login/chunk/plugin pressure
  and does not establish a production claim.

Current P250 fresh stress mixed diagnostics:

- Base worker10/send60/gen20, `300` zombies:
  `online_max=229`, `bot_connected_max=250`, `bot_ready_max=250`,
  `bot_active_max=229`, `bot_errors_max=21`,
  `load_window_tps1_avg=12.32`, `load_window_tps1_min=7.58`,
  `load_window_avg_tick_ms_avg=79.43`,
  `load_window_avg_tick_ms_max=144.78`,
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`,
  `moved_too_quickly_warnings=10566`.
- Slow-move control, same corpus and `300` zombies:
  `online_max=218`, `bot_connected_max=235`, `bot_ready_max=250`,
  `bot_active_max=218`, `bot_errors_max=32`,
  `load_window_tps1_avg=13.53`, `load_window_tps1_min=9.35`,
  `load_window_avg_tick_ms_avg=112.30`,
  `load_window_avg_tick_ms_max=1237.58`,
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`,
  `moved_too_quickly_warnings=8876`.
- Decision: the current P250 ceiling is not watchdog/sync-load anymore. It is
  full-online loss, bot socket errors, TPS/MSPT miss, and movement-warning
  storm under heavy join/chunk/plugin/worldgen pressure. Slower movement did
  not fix the tier.

Observed hot frames from the 100-bot diagnostic:

- `ImprovedNoise.noise(...)`
- `DensityFunctions$HolderHolder.mapAll(...)`
- `DensityFunctions$Ap2.mapAll(...)`
- `DensityFunctions$MarkerOrMarked.mapAll(...)`
- `CubicSpline$Multipoint.mapAll(...)`
- `ServerChunkCache.syncLoad(...)`

## Extreme Ladder

Every tier must produce summary, gate report, logs, resource CSV, and optional
thread-sample summary. A failed tier is useful evidence but never a claim.

- [x] P50 stress mixed fresh world, 16/16, 150 mobs, full corpus:
  diagnostic run exists and passes.
- [x] P100 stress mixed fresh world, 16/16, 150 mobs, full corpus:
  diagnostic run exists and fails.
- [x] P100 stress mixed warm-source, 16/16, 150 mobs, full corpus:
  diagnostic run exists and fails after reaching all bots.
- [x] P100 true-idle warm-source axis diagnostic exists.
- [x] P100 no-mob true-idle warm-source axis diagnostic exists.
- [x] P250 stress mixed fresh world, 16/16, 300 mobs, full corpus:
  diagnostic run exists and fails.
- [x] P250 slow-move stress mixed fresh world, 16/16, 300 mobs, full corpus:
  diagnostic run exists and fails.
- [ ] P500 stress mixed fresh world, 16/16, 500 mobs, full corpus.
- [ ] P750 stress mixed fresh world, 16/16, 750 mobs, full corpus.
- [ ] P1000 stress mixed fresh world, 16/16, 1000 mobs, full corpus.
- [ ] P1500 stress mixed fresh world, 12/12 fallback, full corpus.
- [ ] P2000 stress mixed fresh world, 12/12 fallback, full corpus.
- [ ] P3000 diagnostic only, hardware ceiling discovery.
- [ ] P5000 diagnostic only, hardware ceiling discovery.

Pass requirements for production tiers:

- [ ] `online_max >= tier`.
- [ ] `bot_ready_max >= tier`.
- [ ] `bot_active_max >= tier`.
- [ ] `bot_kicked_max = 0`.
- [ ] `bot_errors_max = 0`.
- [ ] `tps1_avg >= 18.00` for stress mixed tiers.
- [ ] `tps1_min >= 15.00` for stress mixed tiers.
- [ ] `avg_tick_ms_avg <= 75.00`.
- [ ] `avg_tick_ms_max <= 150.00`.
- [ ] `watchdog_thread_dumps = 0`.
- [ ] `sync_load_stack_hits = 0`.
- [ ] `stability_failures = 0`.
- [ ] no unbounded heap/RSS growth.
- [ ] no unbounded packet queue growth.
- [ ] no unbounded chunk-generation queue growth.
- [ ] no shutdown or restart hang.

## Mob And Entity Ladder

- [ ] M1k mixed mobs with P100 clients.
- [ ] M5k mixed mobs with P100 clients.
- [ ] M10k mixed mobs with P250 clients.
- [ ] M25k mixed mobs with P500 clients.
- [ ] M50k mixed mobs diagnostic.
- [ ] M100k mixed mobs diagnostic if hardware allows.
- [ ] Pathfinding budget evidence.
- [ ] Goal-selector budget evidence.
- [ ] Collision lookup budget evidence.
- [ ] Entity tracker packet budget evidence.
- [ ] Despawn/removal cleanup evidence.
- [ ] Mob persistence save/load evidence.

Required entity optimizations:

- [ ] Profile mob AI under accepted stress profile.
- [ ] Remove hot O(n) entity scans where plugin semantics allow it.
- [ ] Add or prove spatial partitioning on collision and target paths.
- [ ] Add bounded AI/pathfinding work queues.
- [ ] Keep vanilla-critical behavior visible to plugins.
- [ ] Add fairness checks so overloaded AI degrades predictably.

## Worldgen And Datapack Ladder

The server must optimize world generation without changing datapack/plugin
semantics.

- [x] Stress datapack corpus downloaded and hashed.
- [x] Stress datapack inspection records worldgen and structure counts.
- [x] Stress corpus boot proves datapacks can be enabled.
- [ ] Fresh worldgen P100 stress mixed gate passes.
- [ ] Fresh worldgen P250 stress mixed gate passes.
- [ ] Fresh worldgen P500 stress mixed gate passes.
- [ ] Pregenerated warm-world P500 stress mixed gate passes.
- [ ] Cold vs warm worldgen delta report exists.
- [ ] Structure-heavy datapack profile exists.
- [ ] Noise-heavy datapack profile exists.
- [ ] Nether/end datapack profile exists.
- [ ] Worldgen JFR/thread-sample evidence exists for each profile.
- [ ] No accepted optimization changes generated chunks without explicit
  parity evidence.
- [ ] Datapack and plugin generation hooks remain compatible.

Required worldgen optimizations:

- [ ] Profile biome generation, noise, structure starts, carvers, surface
  rules, aquifers, and feature placement under the stress datapacks.
- [ ] Cache or batch safe immutable worldgen lookups.
- [ ] Bound chunk generation queues so player count cannot cause unbounded
  backlog.
- [ ] Move safe work off the main thread without changing callback semantics.
- [ ] Preserve generated-world correctness through hash/parity sampling.

## Plugin And Packet Ladder

- [x] Current compatibility matrix exists.
- [x] Stress corpus adds protocol/chat/map/render/region/entity plugins.
- [ ] P100 stress mixed with full corpus passes.
- [ ] P250 stress mixed with full corpus passes.
- [ ] P500 stress mixed with full corpus passes.
- [ ] Packet queue saturation diagnostic exists.
- [ ] Slow-client diagnostic exists.
- [ ] Login burst P500 diagnostic exists.
- [ ] Disconnect storm P500 diagnostic exists.
- [ ] Scoreboard/tab/chat high-update diagnostic exists.
- [ ] Map-render plugin load diagnostic exists.
- [ ] Plugin scheduler starvation diagnostic exists.

Required packet/plugin optimizations:

- [ ] Profile packet encode and compression under stress mixed.
- [ ] Add bounded non-critical packet queues.
- [ ] Keep critical packets bypassing bulk queues.
- [ ] Coalesce or drop only work that is safe under protocol and plugin
  semantics.
- [ ] Prove plugin event ordering parity for optimized paths.

## Evidence System

- [x] Existing 500-bot claim has a bundle model.
- [x] Extreme ladder runner exists.
- [x] Extreme ladder current report exists.
- [x] Extreme warm plateau axis runner exists:
  `scripts/run_extreme_plateau_axis_matrix.sh`.
- [x] Load summaries include bot movement/chunk-rate fields for new runs.
- [ ] Extreme tier bundle exporter exists.
- [ ] Extreme tier bundle validator exists.
- [ ] Extreme claim assertion script exists.
- [ ] Negative tests reject broadened "unlimited" claims.
- [ ] Negative tests reject failed gates.
- [ ] Negative tests reject tampered summaries.

## Immediate Work Queue

- [x] Add an extreme stress ladder runner.
- [x] Add thread-sample summarizer for stress profiles.
- [x] Add a P100 warm plateau axis runner.
- [x] Add true-idle bot mode to avoid stationary position spam.
- [x] Run P100 stress mixed diagnostic.
- [x] Run P100 warm-source diagnostic.
- [x] Run P100 warm true-idle diagnostic.
- [x] Run P100 warm no-mob true-idle diagnostic.
- [ ] Run P250 stress mixed diagnostic.
- [ ] Run P500 stress mixed diagnostic.
- [x] Identify the top hot path from P100 evidence.
- [x] Isolate movement as non-primary for P100 warm plateau.
- [x] Isolate mobs as a meaningful MSPT-tail contributor but not the only
  blocker.
- [ ] Identify the top hot path from P250/P500 evidence.
- [ ] Patch one real hot path.
- [ ] Rebuild optimized artifact.
- [ ] Rerun P100/P250/P500 diagnostics.
- [ ] Only then decide the next production gate target.

## Claim Rules

- [x] No broad claim from a failed 50-bot stress mixed gate.
- [x] No broad claim from boot/join evidence.
- [x] No broad claim from synthetic creative block evidence.
- [x] No literal "unlimited" claim.
- [ ] Every future claim includes exact hardware and JVM settings.
- [ ] Every future claim includes exact plugin/datapack list and hashes.
- [ ] Every future claim includes non-claims.
- [ ] Every future claim has a stable current publication file.
