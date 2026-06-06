# REAL GOAL 2026-05-23: Core runtime wrapper optimization

This is the hard-but-realistic core-runtime goal for `/root/rust`.

The target is not a slogan about unlimited scale. The target is
resource-aware scaling: remove measured wrapper and orchestration overhead
from the runtime path until the next accepted player, mob, chunk, worldgen,
network, or recovery ceiling is explained by evidence: CPU, memory, network,
disk, host contention, or explicit policy. At every ceiling, adaptive
backpressure must preserve no JVM/native crash, no watchdog death, no region or
playerdata corruption, no forced-ticket/recovery state loss, no unbounded queue
growth, and no silent data loss.

Plugins and datapacks stay as compatibility and stress inputs. They are not the
optimization strategy for this goal.

## Starting Truth

- [x] The current optimized artifact is the `131dd4c02...` jar recorded in
  `reports/artifacts.json`.
- [x] The current native runtime library is `270639...`.
- [x] The historical P500 production claim is stale for this current artifact.
- [x] The latest current-artifact host checks did not produce a claim; they
  aborted as host contention or strict foreign `server.jar` process state
  before a valid load window.
- [x] Fresh acceleration-mode checks on 2026-05-23 confirmed the same blocker:
  synthetic canary red on host steal and production preflight red on a foreign
  `server.jar`.
- [x] Native NormalNoise direct JNI output is already wired below this
  frontier.
- [x] Fresh direct `ShiftedNoise` diagnostics did not show a useful standalone
  win.
- [x] Fresh AP2 MIN/MAX diagnostics showed shape-specific wins, not a universal
  server production claim.
- [x] Fresh AP2 MIN/MAX rerun at `2026-05-23T14:20:05Z` still shows Java
  shape wins, but the native path is not a replacement: Java new paths hit
  `2.717x..8.156x` on simple returns and `4.242x..4.640x` on batch overlap,
  while `max_partial_mixed_special` regresses to `0.747x` and several native
  new paths are slower than Java new.
- [x] The next optimization frontier is above the leaf noise path: density
  function wrappers, `mapAll` churn, `ShiftedNoise.fillArray` materialization,
  chunk-generation orchestration, entity/chunk/network backpressure, and
  validation evidence.
- [ ] A fresh quiet-host P500 claim is green again on the current artifact.
- [ ] A same-artifact runtime-wrapper candidate has moved a server gate, not
  just a microbenchmark.

## Exact Non-Claims

- [x] No plugin optimization claim.
- [x] No datapack optimization claim.
- [x] No full Paper runtime rewrite to Rust.
- [x] No literal unlimited player claim.
- [x] No literal unlimited mob claim.
- [x] No literal unlimited chunk, tick, plugin, or datapack claim.
- [x] No real-player gameplay parity claim without live-client evidence.
- [x] No multi-hour soak claim unless that exact soak was measured.
- [x] No microbenchmark-only production claim.
- [x] No noisy-host production claim.

## Work Ladder

- [ ] Restore the current-artifact P500 floor on a quiet host before treating a
  higher tier as accepted.
- [ ] Capture the limiting profile for the first red quiet-host gate:
  `NoiseChunk`, `DensityFunctions`, chunk generation, entity tracking, packet
  fanout, region IO, or network compression.
- [ ] Pick one wrapper or orchestration path from evidence, not from guesswork.
- [ ] Build the smallest candidate that removes real allocation, traversal,
  wrapper transformation, queueing, or fanout cost.
- [ ] Prove the candidate with a focused benchmark or trace that measures the
  exact hot path.
- [ ] Run the same-artifact server gate that previously exposed the limiter.
- [ ] Accept the candidate only if the server gate improves without weakening
  correctness, compatibility, artifact binding, or evidence checks.
- [ ] If the candidate does not move the server gate, revert the candidate from
  the goal track and record the negative result before moving to another path.
- [ ] Regenerate only fresh evidence bundles for accepted tiers.
- [ ] Keep the next ceiling tied to measured hardware, network, disk, host, or
  policy limits.

## Core Runtime Checklist

- [ ] Density-function graph and wrapper pipeline in `NoiseChunk`,
  `DensityFunctions`, `DensityFunction`, `NoiseRouter`, and `RandomState`.
- [ ] `mapAll` wrapper churn and repeated holder/AP2/marker transformation.
- [ ] `ShiftedNoise.fillArray` materialization only as part of the wrapper
  pipeline, not as a standalone leaf-noise claim.
- [ ] Chunk-generation orchestration and queue ownership.
- [ ] Chunk send budgeting and slow-client backpressure.
- [ ] Entity tracker membership refresh, purge, and packet fanout.
- [ ] Nearby-player, collision, and spawn-radius lookup.
- [ ] Region IO save/writeback pressure.
- [ ] Packet encoding and compression hot paths.
- [ ] Restart/recovery and forced-ticket persistence under the leading accepted
  tier.
- [ ] Validation tooling that prevents stale bundles and overbroad claim text.

## Current Frontier Verdict

- [x] Do not spend the next pass promoting AP2 MIN/MAX as a native/server
  claim. It remains a shape-specific Java candidate until a quiet-host gate
  proves end-to-end value.
- [x] The higher-leverage next target is still wrapper/orchestration overhead:
  map only needed density roots, reduce repeated visitor traversal, and prove
  any candidate with the density visitor/mapAll bench before a server gate.

## Next Patch Queue

1. Density wrapper and `mapAll` churn in `NoiseChunk`, `DensityFunctions`,
   `DensityFunction.Visitor`, `NoiseRouter`, and `RandomState`.
2. Entity tracker membership refresh and packet fanout.
3. Nearby-player and finite-radius lookup paths.
4. Chunk send budgeting, queue ownership, and slow-client backpressure.
5. Region IO and restart/recovery under forced tickets.

## Pass Requirements

- [ ] The relevant gate reaches full online for the claimed tier.
- [ ] The action gate opens before performance metrics are counted.
- [ ] `tps1_avg >= 19.50`.
- [ ] `tps1_min >= 18.00`.
- [ ] `avg_tick_ms_avg <= 50.00`.
- [ ] `avg_tick_ms_max <= 100.00`.
- [ ] `watchdog_thread_dumps = 0`.
- [ ] `sync_load_stack_hits = 0`.
- [ ] `stability_failures = 0`.
- [ ] Host watcher does not mark `environment_invalid=true`.
- [ ] No unbounded heap growth.
- [ ] No unbounded packet queue growth.
- [ ] No unbounded chunk queue growth.
- [ ] No kicked-bot or protocol-error pattern inside the claim window.
- [ ] Artifact hashes and native runtime proof match the accepted bundle.
- [ ] Bundle validation and claim assertion pass before publication.

## Definition Of Done

- [ ] The current artifact has a fresh green P500 claim again.
- [ ] At least one core runtime wrapper or orchestration bottleneck has a
  profiler-backed fix and same-artifact server-gate proof.
- [ ] The next accepted tier has fresh soak, repeat quorum, restart/recovery,
  forced-ticket, artifact-hash, and native-runtime evidence.
- [ ] The accepted claim text includes the exact tier, profile, artifact,
  evidence path, and exact non-claims.
- [ ] Plugins and datapacks were not edited to manufacture the pass.
- [ ] The next known ceiling is written down as measured CPU, memory, network,
  disk, host contention, or explicit policy.
