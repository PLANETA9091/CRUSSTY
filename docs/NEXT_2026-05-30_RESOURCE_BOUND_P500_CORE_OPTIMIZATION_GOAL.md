# NEXT 2026-05-30: Resource-bound P500 core optimization goal

This is the next active goal for `/root/rust`.

The task is not to claim infinity. The task is to push the current core
runtime harder under real host pressure, while keeping the claim wording
honest and leaving plugins/datapacks untouched.

## Starting Truth

- [x] The current artifact floor is the 068... jar in `reports/artifacts.json`.
- [x] The current runtime launcher hash is `1fe8a1f2d0b8b6f1de7a21657e1220e199fc0b8c0b44888522d4c780e65c4d91`.
- [x] The current native library hash is `30f9ab3eceb61246ae66e90e690c9127c39315f95f4570eb1b2a7857d61c18d2`.
- [x] `sha256sum -c reports/artifact-hashes.txt` passes for the current artifact set.
- [x] `reports/production-500-go-nogo-current.txt` is red on `diagnostic_host_synthetic_canary_failed`.
- [x] `reports/production-500-soak-gate.txt` is stale against the current artifact floor and cannot be reused as proof.
- [x] `reports/production-500-readiness-gate.txt` is stale against older artifact hashes and cannot be reused as proof.
- [x] Plugins and datapacks are fixed stress inputs, not tuning targets.
- [x] The current claim remains bounded: measured 500 bots / 32 view / 32 simulation / creative block on a verified current artifact, with cold+warm soak, repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence, and a validated self-contained evidence bundle.
- [ ] A fresh strict P500 bundle exists for the 068... artifact.

## Core-Only Execution Ladder

- [ ] Rank the hottest remaining core bottleneck from fresh evidence.
- [ ] Patch one bounded core hot path at a time.
- [ ] Rebuild, rehash, and refresh source-freshness after every accepted patch.
- [ ] Re-run a focused bench for the patched path.
- [ ] Re-run the current diagnostic and strict claim gates only on the verified current artifact.
- [ ] Keep plugin matrix, datapacks, and stress corpus fixed while measuring core changes.
- [ ] Use subagents only for disjoint evidence gathering or disjoint patch slices.

## Likely Core Surfaces

- [ ] Density-function graph churn and `mapAll` wrapper overhead.
- [ ] `ShiftedNoise.fillArray` materialization.
- [ ] `NoiseChunk` flat-cache / interpolator pressure if fresh evidence ranks it again.
- [ ] Chunk generation orchestration and queue budgeting.
- [ ] Entity tracker fanout, nearby-player lookup, and chunk send backpressure if profiler evidence moves there.
- [ ] Region IO and recovery only if fresh evidence moves there.

## Non-Claims

- [ ] not a full Paper runtime rewrite to Rust
- [ ] not literal unlimited players
- [ ] not literal unlimited mobs
- [ ] not literal unlimited chunks or ticks
- [ ] not unlimited plugin compatibility
- [ ] not unlimited datapack compatibility
- [ ] not real-player gameplay parity from bots alone
- [ ] not multi-hour soak unless that exact soak was measured

## Definition Of Done

- [ ] The current artifact has a fresh strict P500 production bundle.
- [ ] The next measured core hotspot is improved and re-tested.
- [ ] The claim text stays exact, bounded, and evidence-backed.
- [ ] No plugin or datapack changes were needed to make the core claim stronger.
- [ ] The next wider tier is defined only after the current measured tier is green.
