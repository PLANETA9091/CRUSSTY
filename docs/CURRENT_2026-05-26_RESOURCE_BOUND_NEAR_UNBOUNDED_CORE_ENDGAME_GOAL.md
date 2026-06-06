# Current 2026-05-26: resource-bound near-unbounded core endgame

Date: 2026-05-26 CEST

This is the current large goal for `/root/rust`.

Literal infinity is not a claim. CPU, RAM, disk, network, JVM, Minecraft
protocol, client behavior, and host scheduling are finite. The real endgame is
harder and more useful: keep pushing the core upward through measured tiers
until every remaining ceiling is named, bounded, and evidenced instead of
being an unexamined Paper/runtime hot path.

Plugins and datapacks remain stress inputs. They are not the optimization
target and must not be patched, simplified, or updated just to force a gate
green. Core runtime, scheduling, backpressure, IO, recovery, and evidence
tooling carry the scale work.

## Allowed Endgame Claim

Only after the checklist below is green:

> resource-bound near-unbounded production scaling for measured Minecraft
> player, mob, chunk, plugin, network, IO, and recovery tiers on a verified
> artifact, with bounded queues, bounded memory growth, graceful overload
> behavior, repeatable gates, restart/recovery proof, and self-contained
> evidence bundles.

## Current Truth

- [x] Literal unlimited players is not a valid claim.
- [x] Literal unlimited mobs is not a valid claim.
- [x] Literal unlimited chunks, ticks, worlds, IO, or network is not a valid
  claim.
- [x] The ceiling must be measured on the current artifact, not borrowed from
  a historical bundle.
- [x] The next optimization target must come from the latest hotspot-ranked
  evidence, not from intuition or agent preference.
- [x] Plugins and datapacks are compatibility and stress inputs, not the
  optimization target.
- [x] Core work means hot paths, scheduling, chunk/entity/network pressure,
  recovery, and evidence tooling.
- [ ] The current artifact has the leading tier freshly certified with same-
  artifact evidence.
- [ ] The current publication bundle validates without stale hashes or stale
  logs.

## Endgame Ladder

### Player Scale

- [ ] P500 current-artifact production floor is green on a quiet host.
- [ ] P500 cold and warm windows are both green on the same artifact.
- [ ] P500 repeat quorum is green on the same artifact.
- [ ] P500 restart/recovery is green on the same artifact.
- [ ] P500 forced-ticket persistence is green on the same artifact.
- [ ] P750 mixed gameplay diagnostic is green.
- [ ] P1000 mixed gameplay diagnostic is green.
- [ ] P1500 mixed gameplay diagnostic is green.
- [ ] P2500 mixed gameplay diagnostic is green if hardware allows.
- [ ] Each promoted player tier has exact hashes, exact logs, and exact non-
  claims.

### Mob Scale

- [ ] M10k mixed mobs diagnostic is green.
- [ ] M25k mixed mobs diagnostic is green.
- [ ] M50k mixed mobs diagnostic is green.
- [ ] M100k mixed mobs diagnostic is green if hardware allows.
- [ ] Pathfinding budget is bounded and observable.
- [ ] Goal selection budget is bounded and observable.
- [ ] Collision and target-acquisition cost are bounded and observable.
- [ ] Despawn and persistence cleanup stay correct under load.

### Chunk, Worldgen, And IO Scale

- [ ] C10k loaded or forced chunks diagnostic is green.
- [ ] C25k loaded or forced chunks diagnostic is green.
- [ ] C50k diagnostic is green if hardware allows.
- [ ] Fresh worldgen stays compatible with datapack and plugin hooks.
- [ ] Chunk send backlog is bounded and observable.
- [ ] Chunk generation backlog is bounded and observable.
- [ ] Region IO writeback pressure is bounded and observable.
- [ ] Restart after chunk pressure preserves world and ticket state.

### Network And Backpressure

- [ ] Slow-client pressure triggers bounded backpressure before runaway debt.
- [ ] Login burst pressure stays bounded.
- [ ] Disconnect storm pressure stays bounded.
- [ ] Packet burst pressure stays bounded.
- [ ] Queue depth, queue age, and drop/defer policy are visible in reports.
- [ ] No accepted gate depends on silent queue growth.

### Recovery, Soak, And Stability

- [ ] Cold+warm soak exists for every accepted tier.
- [ ] Long soak exists for the leading tier and is honestly reported.
- [ ] Restart/recovery under load is green for the leading tier.
- [ ] Forced-ticket persistence is green for the leading tier.
- [ ] No accepted gate has watchdog dumps.
- [ ] No accepted gate has sync-load stack hits.
- [ ] No accepted gate has unbounded RSS, heap, or queue growth.
- [ ] No accepted gate relies on a noisy host or stale bundle.

### Evidence And Publication

- [ ] Every accepted tier has raw logs, summary, gate report, resource CSV,
  and validator output.
- [ ] Every accepted tier has exact artifact hashes.
- [ ] Every accepted tier has a self-contained evidence bundle.
- [ ] Every accepted tier has a stable publication file.
- [ ] Every accepted tier states exact non-claims.
- [ ] Failed tiers are published as failure evidence, not quietly omitted.

## Strict Non-Claims

- [x] No literal infinity claim.
- [x] No full Rust Paper runtime claim.
- [x] No real-player parity claim from synthetic bots alone.
- [x] No unlimited players, mobs, chunks, worlds, ticks, IO, or network.
- [x] No plugin/datapack optimization claim.
- [x] No multi-hour soak claim unless that exact soak was measured.
- [x] No production claim from stale hashes or stale evidence.

## Definition Of Done

- [ ] The current artifact has a fresh green same-artifact production floor.
- [ ] The next higher accepted player tier has its own green evidence set.
- [ ] The mob, chunk, network, and recovery ladders all have at least one
  honest green tier.
- [ ] Remaining ceilings are named as hardware, protocol, JVM, IO, network,
  policy, or explicit budget limits.
- [ ] The server degrades predictably under pressure instead of corrupting
  state or building unbounded debt.
- [ ] The final wording says exactly what was measured and what was not
  measured.
