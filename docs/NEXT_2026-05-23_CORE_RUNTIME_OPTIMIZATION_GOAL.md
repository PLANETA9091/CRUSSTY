# NEXT 2026-05-23: Core Runtime Optimization Goal

This is the next real scale track for `/root/rust`.

Literal unlimited players, mobs, chunks, ticks, plugins, or datapacks is not
a claim. The target is resource-aware scaling: keep raising measured tiers on
the current artifact until the next limit is clearly hardware, network, disk,
host, or explicit policy. At each ceiling, adaptive backpressure must preserve
no JVM/native crash, no watchdog death, no region or playerdata corruption, no
forced-ticket/recovery state loss, no unbounded queue growth, and no silent
data loss. Plugins and datapacks stay stress inputs; do not tune them to fake a
pass.

Run the work in parallel where possible, but do not wait on sidecars for
critical-path validation. Keep the current artifact honest and current.

## Starting Truth

- [x] Older live artifacts included the `aaece1...` and `7b5ab691...`
  optimized jars; they are no longer the current hash floor.
- [x] The current live artifact is now the rebuilt `131dd4c02...` optimized jar
  with the same native runtime library `270639...`.
- [x] The historical P500 claim is stale on this artifact.
- [x] The latest retry failed because the host was noisy or contended, not
  because the artifact hashes were wrong.
- [x] A fresh current-artifact validation run still fails because the bundle is
  stale and missing required evidence, not because the new hashes are wrong.
- [x] The current optimization surface is the core runtime: Paper hot paths,
  Java hot paths, Rust/JNI native modules, chunk/entity/network backpressure,
  and validation tooling.
- [ ] Fresh quiet-host P500 recovery on the current artifact is green again.
- [ ] Fresh bundle validation and claim assertion pass for the current
  artifact.

## Harness Hardening

- [x] Production claim profiles fail fast on synthetic host contention before
  server startup.
- [x] Fresh preflight on the current host still fails on a foreign
  `server.jar` process, so a production claim attempt is blocked.
- [x] Sharded bot launches use shard-aware ramp indexing so distributed runs
  keep one global launch schedule.
- [x] Dedicated synthetic-canary smoke coverage exists for the probe and the
  production harness wiring.
- [x] Host-ready prelaunch aborts now write the same summary/gate evidence
  shape as synthetic-canary aborts.
- [x] The full readiness gate now runs host-ready stable-window and
  sharding-default smoke coverage.
- [ ] Keep the harness smoke green while the runtime and load surface keep
  changing.

## Measured Ladder

- [ ] P500 current-artifact recovery.
- [ ] P750 mixed-gameplay gate on the same artifact.
- [ ] P1000 mixed-gameplay gate on the same artifact.
- [ ] M1k mixed mobs with player load.
- [ ] M5k mixed mobs with player load.
- [ ] M10k mixed mobs diagnostic with bounded AI and pathfinding.
- [ ] M25k mixed mobs diagnostic if hardware allows.
- [ ] C10k chunk/worldgen pressure diagnostic.
- [ ] C25k chunk/worldgen pressure diagnostic if hardware allows.
- [ ] Combined player + mob + chunk + plugin gate on the leading tier.
- [ ] Restart/recovery and forced-ticket persistence on every accepted tier.
- [ ] Long soak on the leading accepted tier.
- [ ] A self-contained evidence bundle for every accepted tier.

## Core Workstreams

- [x] The next work order is now explicit: wrapper/orchestration first,
  then entity tracker fanout, nearby-player lookup, chunk send/backpressure,
  and region IO/recovery.
- [ ] Entity tracker membership refresh and purge
- [ ] Tracked-entity packet fanout
- [ ] Nearby-player, collision, and spawn-radius lookup
- [ ] Chunk send backpressure and queue budgeting
- [ ] Chunk generation orchestration and worldgen hot paths
- [ ] Region IO and save/write backpressure
- [ ] Density wrapper pipeline only where profiler and same-artifact gates
  prove value
- [ ] Packet encode and compression hot paths
- [ ] Restart/recovery and forced-ticket persistence under heavy load
- [ ] Validation tooling and evidence publication

## Fresh Diagnostics

- [x] `bench_shift_noise_direct.sh` showed no real direct-compute benefit for
  the current shape: `shift_direct_speedup=0.989x`, `shift_a_direct_speedup=
  0.990x`, `shift_b_direct_speedup=1.009x`.
- [x] Fresh `bench_density_ap2_minmax_fill.sh` still shows strong Java wins on
  exact MIN/MAX shapes: simple returns `2.717x..8.156x`, batch overlap
  `4.242x..4.640x`, equivalence `PASS`.
- [x] Fresh `bench_density_ap2_minmax_fill.sh` also shows a regression shape:
  `max_partial_mixed_special_speedup=0.747x`, so this is not universal.
- [x] Fresh `bench_native_density_ap2_minmax_fill.sh` confirms the native path
  is mixed and often slower than Java new for the new fast path; do not make it
  the next production replacement.
- [x] `bench_density_visitor_hooks.sh` remains the stronger wrapper signal:
  `8.323x` hooked speedup, `10.537x` shared remembering speedup, and zero
  temporary holder/marker allocations.
- [ ] Do not promote any of these diagnostics to a runtime claim until a
  same-artifact server gate moves the failure out of host contention.

## Pass Requirements

- [ ] `tps1_avg >= 19.50`
- [ ] `tps1_min >= 18.00`
- [ ] `avg_tick_ms_avg <= 50.00`
- [ ] `avg_tick_ms_max <= 100.00`
- [ ] `watchdog_thread_dumps = 0`
- [ ] `sync_load_stack_hits = 0`
- [ ] `stability_failures = 0`
- [ ] no unbounded heap growth
- [ ] no unbounded packet or chunk queue growth
- [ ] no plugin/datapack edits to make a gate pass

## Non-Claims

- [ ] not a full Paper runtime rewrite to Rust
- [ ] not literal unlimited players
- [ ] not literal unlimited mobs
- [ ] not literal unlimited chunks or ticks
- [ ] not unlimited plugin compatibility
- [ ] not unlimited datapack compatibility
- [ ] not real-player parity without live client evidence
- [ ] not a multi-hour soak claim unless that exact soak was measured

## Definition Of Done

- [ ] the current artifact has a fresh green P500 claim again
- [ ] the next accepted tier has fresh soak, repeat quorum, restart/recovery,
  and forced-ticket evidence
- [ ] every accepted tier has exact claim text and exact non-claims
- [ ] the next ceiling is measured hardware, network, disk, or policy, not an
  unexamined hot path
