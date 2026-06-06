# Current Real Scale Goal

Date: 2026-05-18 CEST

Status: historical/superseded. The active 2026-05-23 goal is now
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md` and
`docs/REAL_GOAL_2026-05-23_P500_TO_NEAR_UNBOUNDED_EXECUTION.md`.
Do not treat this file as the claim source now.

This file was the active goal file for that push. The target was not a
literal "unlimited" server. The target is to keep raising measured tiers until
the next limit is hardware, network, disk IO, or an explicit policy limit,
while preserving Paper/Bukkit, plugin, datapack, and worldgen semantics.

## Allowed Claim Shape

Only claim a tier after its summary, gate report, logs, resource data, artifact
hashes, and evidence bundle are present and green on the artifact recorded in
`reports/artifacts.json` for that run. The 2026-05-23 native rebuild snapshot
used jar `aaece1b92672639da5ceee370b28029fe8a44b8a43eb9de72df6877865d07524`
with native runtime library
`270639cc1ecdb642b6944d84675679a349702fdaa44b6723cd5a78e387e632fd`; it is
now a historical snapshot, not current-artifact evidence. The P500 claim on
that snapshot was still red/stale:

> production-ready for the measured tier on the verified artifact, with the
> exact bot count, view/simulation distance, workload, plugin/datapack corpus,
> mob pressure, restart/recovery scope, and non-claims stated.

Do not claim:

- literal infinite players, mobs, chunks, ticks, plugins, or datapacks
- full Rust Paper runtime
- real-player gameplay parity without live client evidence
- plugin compatibility beyond the tested matrix
- a higher tier because a lower tier passed
- a benchmark result from a busy/noisy host as a clean production claim

## Verified Floor

- [x] Measured `500 bots / 32 view / 32 simulation / creative block`
  production-ready claim exists on the historical `d4b27...` artifact.
- [x] Historical artifacts such as `31a15...` and `d4b27...` are stale for
  the later 2026-05-23 artifact snapshot.
- [x] Historical 2026-05-23 snapshot was jar `aaece1...` plus native runtime
  `270639...`, and the old P500 claim was stale there.
- [ ] Measured `500 bots / 32 view / 32 simulation / creative block`
  production-ready claim exists on a fresh recorded artifact with cold+warm soak,
  repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence,
  and a self-contained evidence bundle.
- [x] Heavy stress corpus exists with `26` total plugin jars and `10`
  datapacks.
- [x] Mixed-gameplay workload exists with movement, block place/break, held
  item switches, arm animation, player input, use-item, commands, plugin
  counters, datapacks, and mobstorm pressure.
- [x] Fresh `50` bot stress-corpus mixed-gameplay gate passed on the
  then-recorded artifact.
- [x] Fresh `100` bot stress-corpus mixed-gameplay gate passed on the
  then-recorded artifact with `26` plugin jars, `10` datapacks, `150` mobs, and zero
  kicks/errors/watchdog/sync-load/moved-too-quickly failures.

## Historical Active Ladder

- [ ] P250 stress-corpus mixed-gameplay fresh-world gate passes.
- [ ] P500 stress-corpus mixed-gameplay fresh-world diagnostic exists.
- [ ] P500 stress-corpus mixed-gameplay fresh-world gate passes.
- [ ] P500 stress-corpus mixed-gameplay cold+warm repeat quorum exists.
- [ ] P500 stress-corpus mixed-gameplay 2h cold+warm soak exists.
- [ ] P750 stress-corpus mixed-gameplay diagnostic exists.
- [ ] P1000 stress-corpus mixed-gameplay diagnostic exists.
- [ ] Leading accepted mixed-gameplay tier has restart/recovery evidence.
- [ ] Leading accepted mixed-gameplay tier has forced-ticket persistence
  evidence.
- [ ] Leading accepted mixed-gameplay tier has a self-contained evidence
  bundle and published claim file.

## Entity And Worldgen Ladder

- [ ] M1k mixed mobs with an accepted player tier.
- [ ] M5k mixed mobs with an accepted player tier.
- [ ] M10k mixed mobs with an accepted player tier.
- [ ] M25k mixed mobs diagnostic.
- [ ] Pathfinding budget evidence.
- [ ] Collision lookup budget evidence.
- [ ] Entity tracker budget evidence.
- [ ] Chunk generation queue backpressure evidence.
- [ ] Chunk send queue backpressure evidence.
- [ ] Datapack worldgen compatibility gate at the accepted tier.

## Pass Requirements For New Player Tiers

- [ ] requested bot tier reaches `online_max`, `bot_ready_max`, and
  `bot_active_max`
- [ ] `bot_kicked_max = 0`
- [ ] `bot_errors_max = 0`
- [ ] `load_window_tps1_avg >= 18.00`
- [ ] `load_window_tps1_min >= 15.00`
- [ ] `load_window_avg_tick_ms_avg <= 75.00`
- [ ] `load_window_avg_tick_ms_max <= 150.00`
- [ ] `watchdog_thread_dumps = 0`
- [ ] `sync_load_stack_hits = 0`
- [ ] `stability_failures = 0`
- [ ] no unbounded heap, packet queue, chunk queue, or disk growth

## Working Rule

If a tier fails, the next task is to use the failure evidence to pick the real
bottleneck. Do not skip from a P100 pass to a P500 claim. Do not mark a box
until the relevant recorded artifact has a fresh green gate.
