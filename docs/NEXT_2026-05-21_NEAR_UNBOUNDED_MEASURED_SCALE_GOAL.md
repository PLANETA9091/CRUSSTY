# Near-Unbounded Measured Scale Goal

Date: 2026-05-21 CEST

This is the next extremely large scale target, but the claim remains bounded by
fresh evidence. "Near-unbounded" here means a ladder of measured tiers that can
keep expanding while every accepted tier is tied to an artifact, workload,
runtime profile, resource envelope, and self-contained evidence bundle.

## Current Status

- [x] Historical `500 bots / 32 view / 32 simulation / creative block` claim
  exists for an older verified artifact.
- [x] A current-artifact P500 block profile has a fresh red baseline with all
  joins accepted, block workload active, and packet/container hot paths
  identified.
- [x] Existing docs already reject literal unlimited, full Rust Paper runtime,
  arbitrary plugin compatibility, arbitrary datapack/worldgen compatibility,
  and real-player parity claims without separate gates.
- [ ] Current artifact does not yet have a fresh green P500 cold+warm soak,
  repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence,
  and validated evidence bundle.
- [ ] No larger player/mob/chunk tier may inherit the old P500 claim without
  fresh same-artifact evidence.

## Target Claim Shape

The only allowed public claim after a tier is accepted:

> production-ready for the measured tier on the verified artifact, with exact
> bot count, view distance, simulation distance, workload, plugin/datapack
> corpus, mob pressure, world source, restart/recovery scope, soak duration,
> resource envelope, artifact hashes, failure budget, and non-claims stated.

Every tier must be stated as a measured envelope, not as a universal runtime
property.

## Measured Tier Ladder

- [ ] `P500-BLOCK-RECOVERY`: restore green `500 bots / 32 view / 32 simulation
  / creative block` on the current artifact.
- [ ] `P500-MIXED`: pass `500 bots / 32 view / 32 simulation` with movement,
  block place/break, held item switches, arm animation, player input, use-item,
  commands, plugin counters, datapacks, and mob pressure.
- [ ] `P750-MIXED-DIAGNOSTIC`: run only after `P500-MIXED` is green; this is
  diagnostic until all acceptance gates pass.
- [ ] `P1000-MIXED-DIAGNOSTIC`: run only after P750 identifies no hard
  protocol, packet, tick, memory, or recovery ceiling.
- [ ] `P1500-MIXED-DIAGNOSTIC`: run only if P1000 passes with headroom and
  host resource telemetry shows the bottleneck is not external load/steal.
- [ ] `P2000-MIXED-DIAGNOSTIC`: optional stretch tier; no claim until soak,
  repeat, recovery, and bundle gates pass on the exact same artifact.
- [ ] `M10k-MIXED`: accept `10,000` mixed mobs only with an accepted player
  tier, explicit mob AI/profile settings, and resource telemetry.
- [ ] `M25k-MIXED-DIAGNOSTIC`: run only if M10k has repeatable green evidence
  and no watchdog/sync-load/queue blow-up.
- [ ] `C10k-LOADED-CHUNKS`: accept `10,000` loaded chunks only with an accepted
  player tier, exact world source, ticket source, and forced-ticket evidence.
- [ ] `C25k-LOADED-CHUNKS-DIAGNOSTIC`: run only after C10k is green and memory,
  IO, and chunk task queues stay inside the declared envelope.
- [ ] `24H-SOAK`: complete a full 24h soak at the highest accepted player tier
  before any long-duration scale claim.
- [ ] `RECOVERY-UNDER-LOAD`: prove restart/recovery, reconnect, forced-ticket
  persistence, and claim revalidation at the highest accepted tier.

## Acceptance Gates Per Tier

- [ ] Fresh summary marks `claim_eligible=true` and `gate_pass=true`.
- [ ] The tier reaches the declared bot/player, mob, and chunk counts during
  the measured load window.
- [ ] `bot_kicked_max=0` unless the tier explicitly declares a nonzero failure
  budget and the claim text includes it.
- [ ] `bot_errors_max=0` unless the tier explicitly declares a nonzero failure
  budget and the claim text includes it.
- [ ] `tps1_avg >= 19.50`.
- [ ] `tps1_min >= 18.00`.
- [ ] `avg_tick_ms_avg <= 50.00`.
- [ ] `avg_tick_ms_max <= 100.00`, or the tier declares a stricter measured
  spike budget with exact evidence.
- [ ] `watchdog_thread_dumps=0`.
- [ ] `sync_load_stack_hits=0`.
- [ ] `stability_failures=0`.
- [ ] Packet queue, entity tracking, container broadcast, chunk task, IO, and
  memory telemetry are captured for the accepted run.
- [ ] Cold soak passes on the exact artifact.
- [ ] Warm soak passes on the exact artifact.
- [ ] Repeat quorum passes on the exact artifact.
- [ ] Restart/recovery passes on the exact artifact and tier.
- [ ] Forced-ticket persistence passes when chunk scale is part of the claim.
- [ ] Plugin matrix passes for the declared plugin corpus.
- [ ] Datapack/worldgen matrix passes for the declared datapack/world corpus.
- [ ] Evidence bundle validates and contains summaries, gate reports, logs,
  resource data, artifact hashes, matrix reports, repeat evidence, recovery
  evidence, and non-claims.

## Scale Discipline

- [ ] Do not raise player tier while the current P500 block/mixed tier is red.
- [ ] Do not raise mob tier without a green accepted player tier.
- [ ] Do not raise chunk tier without a green accepted player tier and
  forced-ticket evidence.
- [ ] Do not merge diagnostic and accepted results in the same claim.
- [ ] Do not use stale artifact bundles to justify current-artifact claims.
- [ ] Do not publish a higher tier if external host load/steal makes the result
  non-reproducible.
- [ ] On every red run, record the limiting subsystem before changing knobs:
  protocol, packet/entity tracking, container broadcast, chunk load, mob AI,
  scheduler, IO, memory, host CPU, or harness.

## Explicit Non-Claims

- [x] No literal unlimited players, mobs, chunks, ticks, worlds, plugins,
  datapacks, or runtime behavior.
- [x] No full Rust Paper runtime claim.
- [x] No arbitrary Bukkit/Paper plugin compatibility without a measured matrix
  gate.
- [x] No arbitrary datapack, worldgen, or gameplay compatibility without a
  measured matrix gate.
- [x] No real-player parity without live client evidence.
- [x] No production claim from diagnostic tiers.
- [x] No old-artifact claim transfer to a new artifact.
- [x] No long-duration claim without fresh soak evidence for that duration.
- [x] No "near-unbounded" marketing claim without listing the exact accepted
  tier and its measured envelope.
