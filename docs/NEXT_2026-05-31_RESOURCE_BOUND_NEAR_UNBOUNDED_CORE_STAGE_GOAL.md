# NEXT 2026-05-31: Resource-bound core stage

This is the next core-only bounded goal for `/root/rust`.

The target is bounded and evidence-first: keep pushing the current artifact
upward on the core runtime surfaces that actually limit scale, while keeping
every statement honest. Literal unlimited scale and production-ready wording
are not allowed here. Plugins and datapacks stay fixed as stress inputs.

## Starting Truth

- [x] The current artifact floor is the optimized jar
  `3348f7ae833d0de4edd53b463c09920b01462d90a4df5adc8f263c8092dd9d90` in
  `reports/artifacts.json`.
- [x] The current runtime launcher hash is
  `1fe8a1f2d0b8b6f1de7a21657e1220e199fc0b8c0b44888522d4c780e65c4d91`.
- [x] The current native library hash is
  `30f9ab3eceb61246ae66e90e690c9127c39315f95f4570eb1b2a7857d61c18d2`.
- [x] `sha256sum -c reports/artifact-hashes.txt` passes for the current
  artifact set.
- [x] `reports/production-500-go-nogo-current.txt` is red on host contention
  and is diagnostic only.
- [x] Plugins and datapacks are fixed stress inputs, not tuning targets.
- [x] Any older bundle, hash floor, or claim text is historical only unless it
  matches the current artifact.
- [ ] Fresh current-artifact evidence exists for the next measured core tier.

## Path Forward

- [ ] Rank the hottest remaining core bottleneck from fresh evidence, not from
  guesswork.
- [ ] Patch one bounded core hot path at a time.
- [ ] Rebuild, rehash, and refresh source freshness after every accepted patch.
- [ ] Re-run a focused bench for the patched path on the same artifact.
- [ ] Re-run the cheapest useful same-artifact server diagnostic after the
  patch.
- [ ] Keep plugins and datapacks untouched while measuring core changes.
- [ ] Use subagents only for disjoint evidence gathering or disjoint patch
  slices.

## Core Surfaces To Rank

- [ ] Network ingress, outbound packet debt, compression, and slow-client
  backpressure.
- [ ] Chunk ticketing, chunk send backlog, generation orchestration, and queue
  budgeting.
- [ ] Entity tracker fanout, nearby-player lookup, collision, and
  target-acquisition cost.
- [ ] Worldgen density graphs, wrapper churn, feature traversal, and noise
  materialization.
- [ ] Region IO, writeback pressure, playerdata, forced-ticket recovery, and
  restart behavior.

## Strict Non-Claims

- [x] No literal unlimited players, mobs, chunks, ticks, worlds, IO, or
  network.
- [x] No production-ready claim.
- [x] No full Paper runtime rewrite to Rust.
- [x] No unlimited plugin compatibility claim.
- [x] No unlimited datapack compatibility claim.
- [x] No real-player parity claim from bots alone.
- [x] No multi-hour soak claim unless that exact soak was measured.
- [x] No claim from stale hashes, stale logs, or stale bundles.

## Definition Of Done

- [ ] The current artifact has fresh evidence that ranks the next core
  bottleneck.
- [ ] At least one core hot path in network, chunk, entity, worldgen, or IO is
  improved and re-tested on the same artifact.
- [ ] Artifact hashes and source freshness are refreshed after the accepted
  patch.
- [ ] The next same-artifact diagnostic shows bounded behavior or names the
  next ceiling precisely.
- [ ] The final wording stays bounded, evidence-first, and explicit about
  non-claims.
- [ ] No plugin or datapack edits were needed to advance the core ladder.
