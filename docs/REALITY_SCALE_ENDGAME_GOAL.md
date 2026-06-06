# Reality Scale Endgame Goal

Date: 2026-05-18 CEST

Status: historical/superseded roadmap. The superseding 2026-05-23 claim source is
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md`.

Literal unlimited is not a claim. The real endgame target is to remove every
artificial ceiling one by one until the next hard limit is hardware, network,
disk, or explicit policy, while keeping Bukkit, plugin, datapack, and worldgen
semantics intact and publishing proof for every accepted tier.

## Allowed Final Claim

Allowed only after every required gate below is green:

> production-ready for measured near-unbounded Minecraft scale on a verified
> artifact, with repeatable player/mob/chunk/worldgen/plugin/network tier
> gates, restart/recovery, long soak, backpressure behavior, artifact hash
> proof, and self-contained evidence bundles.

Not allowed to claim:

- full Rust Paper runtime
- literal infinite players
- literal infinite mobs
- unlimited plugins
- unlimited datapacks
- real-player parity without live client evidence
- any tier without fresh gate data

## Current Verified Floor

- [x] Historical narrow 500 bots / 32 view / 32 simulation / creative block
  claim exists for an older artifact.
- [ ] Superseded 2026-05-23 artifact snapshot `ece63...` still had no fresh
  green P500 claim; this endgame ladder was not current-artifact evidence.
- [x] Stress corpus exists with heavy plugins and heavy datapacks.
- [x] Mixed-gameplay harness exists.
- [x] Fresh 50-bot stress-corpus mixed-gameplay gate passes on the current
  artifact.
- [x] The 50-bot pass keeps 50/50 bots online with 26 plugins, 10 datapacks,
  and 150 spawned mobs.
- [x] The 50-bot pass keeps `watchdog_thread_dumps=0`,
  `sync_load_stack_hits=0`, and `stability_failures=0`.
- [x] `/forceload` command path no longer sync-loads fresh chunks before
  returning.
- [x] Regression coverage exists for `forceload add` staying off the sync
  chunk-load path.
- [x] P100 warm-source diagnostics exist and still show the next ceiling is
  not just movement spam.
- [x] P250 full-corpus fresh diagnostics exist with `300` mobs and current
  artifact hashes.
- [x] P250 slow-move control exists and shows slower movement does not fix the
  tier.

## Giant Ladder

### Player Tiers

- [ ] P100 mixed-gameplay fresh world with full corpus passes.
- [ ] P250 mixed-gameplay fresh world with full corpus passes.
- [ ] P500 mixed-gameplay fresh world with full corpus passes.
- [ ] P1000 mixed-gameplay fresh world with full corpus passes.
- [ ] P2500 mixed-gameplay fresh world with full corpus passes.
- [ ] P5000 mixed-gameplay fresh world with full corpus passes if hardware
  allows.
- [ ] P10000 diagnostic if hardware allows.

Pass requirements:

- [ ] `tps1_avg >= 18.00`
- [ ] `tps1_min >= 15.00`
- [ ] `avg_tick_ms_avg <= 75.00`
- [ ] `avg_tick_ms_max <= 150.00`
- [ ] `watchdog_thread_dumps = 0`
- [ ] `sync_load_stack_hits = 0`
- [ ] no unbounded memory or queue growth

### Mob And Entity Tiers

- [ ] M10k mixed mobs stable.
- [ ] M50k mixed mobs stable.
- [ ] M100k mixed mobs diagnostic.
- [ ] M250k mixed mobs diagnostic if hardware allows.
- [ ] Pathfinding budget evidence.
- [ ] Goal-selector budget evidence.
- [ ] Collision lookup budget evidence.
- [ ] Entity tracker packet budget evidence.
- [ ] Despawn/removal cleanup evidence.
- [ ] Mob persistence save/load evidence.

### Chunk, World, And IO Tiers

- [ ] C25k loaded chunks.
- [ ] C50k loaded chunks.
- [ ] C100k loaded chunks diagnostic.
- [ ] Chunk send queue backpressure gate.
- [ ] Chunk generation queue backpressure gate.
- [ ] Forced-ticket persistence under high load.
- [ ] Restart after high-chunk load.
- [ ] Fresh worldgen and warm-world profiles both pass at accepted tiers.
- [ ] Datapack and plugin generation hooks remain compatible.

### Plugins And Network Tiers

- [ ] Full stress corpus passes at P100/P250/P500.
- [ ] Packet budgets per player, chunk, and entity tracker.
- [ ] Login burst gate.
- [ ] Disconnect storm gate.
- [ ] Slow-client backpressure gate.
- [ ] No OOM under packet burst.
- [ ] No main-thread death under packet burst.

### Soak, Recovery, And Evidence

- [ ] 2h cold + warm soak for each accepted tier.
- [ ] 24h soak for the leading tier.
- [ ] restart/recovery under load.
- [ ] disk-pressure recovery.
- [ ] plugin-churn recovery.
- [ ] worldgen-churn recovery.
- [ ] timestamped evidence bundle for every accepted tier.
- [ ] hash manifest for every accepted bundle.
- [ ] comparison report for every accepted tier.
- [ ] stable current publication file for every accepted tier.

## Working Rule

This file is the replacement for the vague "unlimited" slogan. A tier only
counts after its fresh summary, gate report, logs, and bundle are all present
and the numbers stay green on a repeatable artifact.
