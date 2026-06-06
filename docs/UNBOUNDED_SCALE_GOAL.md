# Unbounded Scale Goal

Date: 2026-05-18 CEST

Status: historical/superseded roadmap. The superseding 2026-05-23 claim source is
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md`.

Literal unlimited is impossible. CPU, RAM, network, disk IO, and protocol
limits always exist. The real target is narrower and harder:

make the Minecraft core stop being the first bottleneck, then push every
remaining limit into measured tier gates, published evidence, and explicit
hardware or policy ceilings.

## Final Claim Target

Allowed only after every required gate below is green:

> near-unbounded production scaling for measured Minecraft player, mob,
> chunk, plugin, datapack, packet, and world-event workloads on a verified
> artifact, with repeatable tier gates, long soak, restart/recovery,
> backpressure behavior, artifact hash proof, and self-contained evidence
> bundles.

Not allowed to claim:

- full Rust Paper runtime
- infinite players
- infinite mobs
- unlimited plugins
- unlimited datapacks
- real-player gameplay parity without live client evidence
- long soak without fresh soak evidence

## Current Verified Baseline

- [x] Historical narrow 500 bots / 32 view / 32 simulation / creative block
  claim exists for an older artifact.
- [ ] Superseded 2026-05-23 artifact snapshot `ece63...` still had no fresh
  green P500 claim, repeat quorum, restart/recovery, forced-ticket
  persistence, or self-contained regenerated bundle.
- [x] Stress corpus exists with heavy plugins and heavy datapacks.
- [x] Mixed-gameplay harness exists.
- [x] True-idle bot mode exists.
- [x] P100 warm-source diagnostics exist.
- [x] P100 no-mob true-idle diagnostics exist.
- [x] Fresh 50-bot stress-corpus mixed-gameplay gate passes on the current
  artifact.
- [x] Current runtime still has open hot spots in density, chunk streaming,
  worldgen, and higher-tier entity pressure paths.
- [x] `/forceload` command path no longer sync-loads fresh chunks before
  returning.
- [x] Regression coverage exists for `forceload add` staying off the sync
  chunk-load path.
- [x] P250 full-corpus fresh diagnostics now exist and fail with complete
  summary/gate evidence, not a partial run.

## Current Working Goal

The next real goal is not "infinite". It is:

- [ ] Raise measured mixed-gameplay player tiers beyond the current 50-bot
  pass and P100 diagnostics without reintroducing watchdogs, sync-load spikes,
  or runaway memory growth.
- [ ] Raise mixed mob tiers while keeping activation, pathfinding, collision,
  and tracker costs bounded.
- [ ] Keep worldgen and datapack semantics intact while cutting the hot cost
  of generation, feature placement, and chunk streaming.
- [ ] Prove restart, recovery, and forced-ticket persistence under heavy load
  at the new tiers.
- [ ] Publish a fresh evidence bundle for every accepted tier.

## Phase 1: Harness And Coverage

Goal: test pressure that looks like a real server, not a single synthetic loop.

- [x] Mixed movement exists.
- [x] Block place / break / interact exists.
- [x] Command traffic exists.
- [x] Item switch / animation / use-item traffic exists.
- [x] Mob pressure exists.
- [ ] Chunk churn across multiple regions.
- [ ] Combat, damage, death, and respawn pressure.
- [ ] Container open / close pressure.
- [ ] Teleport pressure across regions.
- [ ] Chat / scoreboard / team pressure.
- [ ] Load bursts with real plugin and datapack mixes.
- [ ] Slow-client and disconnect-storm pressure.

## Phase 2: Player Tiers

Goal: scale by measured tiers, not by wishful extrapolation.

- [x] P500 creative block passed.
- [ ] P500 mixed gameplay cold + warm gate.
- [ ] P500 mixed gameplay 24h soak.
- [ ] P750 mixed gameplay gate.
- [ ] P1000 mixed gameplay gate.
- [ ] P1500 mixed gameplay gate.
- [ ] P2000 mixed gameplay gate.
- [ ] P3000 mixed gameplay gate if hardware allows.
- [ ] P5000 mixed gameplay gate if hardware allows.

Pass requirements:

- [ ] tps1_avg >= 19.50
- [ ] tps1_min >= 18.00
- [ ] avg_tick_ms_avg <= 50.00
- [ ] avg_tick_ms_max <= 100.00
- [ ] watchdog_thread_dumps = 0
- [ ] sync_load_stack_hits = 0
- [ ] stability_failures = 0
- [ ] no unbounded heap growth
- [ ] no unbounded queue growth

## Phase 3: Mob Tiers

Goal: mobs scale by budgets, not by random TPS collapse.

- [ ] M10k passive mobs stable gate.
- [ ] M10k hostile mobs stable gate.
- [ ] M25k mixed mobs stable gate.
- [ ] M50k mixed mobs stable gate.
- [ ] M100k mixed mobs diagnostic if hardware allows.
- [ ] Pathfinding budget gate.
- [ ] Goal selector budget gate.
- [ ] Collision lookup budget gate.
- [ ] Target acquisition budget gate.
- [ ] Tracker budget gate.
- [ ] Despawn cleanup gate.
- [ ] Save/load persistence gate.
- [ ] AI backpressure gate with graceful degradation.

Required work:

- [ ] Profile entity ticking under accepted stress tiers.
- [ ] Remove hot O(n) scans where semantics allow it.
- [ ] Add or prove spatial partitioning on hot entity paths.
- [ ] Bound AI and pathfinding work per tick.
- [ ] Preserve plugin-visible Bukkit semantics.

## Phase 4: Chunk, Region, And World IO

Goal: chunk and region pressure must not collapse the tick loop.

- [ ] C5k loaded chunks gate.
- [ ] C10k loaded chunks gate.
- [ ] C25k loaded chunks gate.
- [ ] C50k loaded chunks gate if hardware allows.
- [ ] Chunk send queue backpressure gate.
- [ ] Chunk generation queue backpressure gate.
- [ ] Region read/write latency gate.
- [ ] Save-all under load gate.
- [ ] Restart after high-chunk load gate.
- [ ] Forced chunk persistence under high load gate.
- [ ] Pregenerated world gate.
- [ ] Cold fresh-world generation gate.

Required work:

- [ ] Profile sync chunk loads until accepted sync-load hits are gone.
- [ ] Move safe IO off the main thread.
- [ ] Bound send and generation queues.
- [ ] Batch region writes safely.
- [ ] Avoid repeated remap/compression work where the artifact proves identical input.
- [ ] Preserve crash consistency and world save correctness.

## Phase 5: Plugins, Datapacks, And Network

Goal: a heavy plugin and datapack matrix should be a measured constraint, not a hidden crash source.

- [x] Heavy stress plugin corpus exists.
- [x] Heavy stress datapack corpus exists.
- [x] P50 stress mixed with full corpus passes.
- [x] P250 stress mixed diagnostic with full corpus exists and fails.
- [ ] P100 stress mixed with full corpus passes.
- [ ] P250 stress mixed with full corpus passes.
- [ ] P500 stress mixed with full corpus passes.
- [ ] Packet budget per player.
- [ ] Chunk packet budget.
- [ ] Entity tracker packet budget.
- [ ] Login burst gate.
- [ ] Disconnect storm gate.
- [ ] Slow-client backpressure gate.
- [ ] No OOM under packet burst.
- [ ] No main-thread death under packet burst.

Required work:

- [ ] Profile packet encode hot paths.
- [ ] Profile compression hot paths.
- [ ] Add strict queue limits.
- [ ] Prefer dropping or deprioritizing non-critical work over killing TPS.
- [ ] Prove protocol correctness with real client compatibility probes.

## Phase 6: Soak And Recovery

Goal: no accepted tier is real until it survives time and restart.

- [ ] 2h cold + warm soak for each accepted player tier.
- [ ] 24h soak for the leading player tier.
- [ ] Restart during load.
- [ ] Restart after load.
- [ ] Recovery after disk pressure.
- [ ] Recovery after plugin churn.
- [ ] Recovery after worldgen churn.
- [ ] Recovery after forced-ticket heavy load.

## Phase 7: Publication

Goal: every accepted tier leaves an evidence trail that another machine can verify.

- [ ] Timestamped evidence bundle for every accepted tier.
- [ ] Stable current publication file for every accepted tier.
- [ ] Independent validation script for every accepted tier.
- [ ] Published claim text that includes exact non-claims.
- [ ] Hash manifest for every accepted bundle.

## Current Next Work

- [x] Re-run the mixed stress gate after the `/forceload` command-path fix and
  re-baseline sync-load hits.
- [ ] Push the same artifact to the next P100 mixed stress gate.
- [ ] Keep the `NoiseChunk` sequential fast path moving through compile and load gates.
- [ ] Re-run the mixed stress gate after the density / chunk-stream changes.
- [ ] Use the next measured failure to choose the next hotspot, not the next slogan.
