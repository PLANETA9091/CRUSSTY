# State

## Current 2026-05-31 CEST: current-artifact recertification blocked by host contention, next core-only bounded stage

The active goal is now tracked in:

```text
docs/NEXT_2026-05-31_RESOURCE_BOUND_NEAR_UNBOUNDED_CORE_STAGE_GOAL.md
```

The current P500 production-claim execution track remains:

```text
docs/CURRENT_2026-05-29_P500_PRODUCTION_CLAIM_AND_RESOURCE_BOUND_CORE_GOAL.md
```

The broader scale / endgame track remains:

```text
docs/CURRENT_2026-05-26_RESOURCE_BOUND_NEAR_UNBOUNDED_CORE_ENDGAME_GOAL.md
```

The live current artifact recorded by `reports/artifacts.json` and validated by
`reports/artifact-hashes.txt` is:

```text
optimized_artifact_sha256=3348f7ae833d0de4edd53b463c09920b01462d90a4df5adc8f263c8092dd9d90
optimized_runtime_run_sh_sha256=1fe8a1f2d0b8b6f1de7a21657e1220e199fc0b8c0b44888522d4c780e65c4d91
optimized_runtime_native_library_sha256=30f9ab3eceb61246ae66e90e690c9127c39315f95f4570eb1b2a7857d61c18d2
```

Current truth:

- Literal unlimited scale is not a claim. The current engineering target is
  resource-aware scaling: accepted measured tiers on a verified artifact, with
  finite hardware, network, disk, host, or policy ceilings named explicitly.
  At each ceiling, adaptive backpressure must preserve no JVM/native crash, no
  watchdog death, no region or playerdata corruption, no forced-ticket/recovery
  state loss, no unbounded queue growth, and no silent data loss.
- Plugins and datapacks remain stress inputs only. Do not patch, update, tune,
  or simplify them to make the core gate pass.
- `reports/production-500-go-nogo-current.txt` is the current live go/no-go
  report and is red on `diagnostic_host_synthetic_canary_failed`.
- `reports/production-500-soak-gate.txt` and
  `reports/production-500-readiness-gate.txt` are stale relative to the current
  artifact floor and must not be reused as proof for the current
  `3348f7ae833d0de4edd53b463c09920b01462d90a4df5adc8f263c8092dd9d90` artifact.
- The current optimization surface remains the core runtime: density graphs,
  `mapAll` churn, `ShiftedNoise.fillArray`, chunk orchestration, and only then
  the next ranked hot path from fresh evidence.
- The historical P500 creative-block production claim remains stale for the
  current artifact.

Immediate execution order:

- [x] Register the new next goal file.
- [x] Keep the P500 claim path separate from the broader core optimization
  track.
- [x] Keep the production claim gates strict and evidence-backed.
- [ ] Refresh or regenerate the current P500 diagnostic and strict claim
  evidence on the `3348f7ae833d0de4edd53b463c09920b01462d90a4df5adc8f263c8092dd9d90` artifact.
- [ ] Use the latest profiler, summary, and bench evidence to pick the next
  core-only bottleneck.
- [ ] Rebuild and rehash after any accepted core patch.
- [ ] Do not touch plugin or datapack tuning for the claim path.
- [ ] Do not widen the claim wording beyond the exact measured tier and exact
  non-claims.

## Current 2026-05-20 CEST: P250 stress-mixed radius-preload gate is red after reaching full-online admission

The live current artifact is:

```text
optimized_artifact_sha256=0618661f515f668602b9a86e2978b789ec4001d5921b776865a0a2263f6456c2
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

Fresh evidence:

- Summary:
  `reports/load-current-0127-radiuspreload-joins20-p250-ramp180-20260520-053648-summary.txt`
- Gate:
  `reports/load-current-0127-radiuspreload-joins20-p250-ramp180-20260520-053648-gate.txt`
- Log:
  `logs/load-current-0127-radiuspreload-joins20-p250-ramp180-20260520-053648.log`
- Bot log dir:
  `logs/load-current-0127-radiuspreload-joins20-p250-ramp180-20260520-053648-bots/`

Fresh result:

```text
gate_pass=false
claim_eligible=false
failure_count=9
observed_online_max=250
observed_bot_connected_max=250
observed_bot_ready_max=250
observed_bot_active_max=250
observed_bot_errors_max=14
observed_load_window_reached_full_online=true
observed_bot_action_gate_opened=true
observed_tps1_avg=14.31
observed_tps1_min=0.61
observed_avg_tick_ms_avg=79.31
observed_avg_tick_ms_max=915.73
observed_process_rss_mib_max=26448.1
observed_watchdog_thread_dumps=12
observed_external_thread_prints=10
observed_stability_failures=0
```

Interpretation: the join/ready admission path is no longer the blocker. An
explicit `misc.max-joins-per-tick=20` control let the tier reach full online
and open the action gate, but the first full-online window is still unstable
on the fresh world with the full stress corpus. The next measured move is a
warm-source P250 diagnostic on the same artifact. The forward checklist is
in `docs/NEXT_2026-05-20_P250_POSTFULLONLINE_GOAL.md`.

## Previous 2026-05-19 CEST: measured 500 production-ready refresh was green for artifact d4b27...

The artifact for this previous P500 claim was:

```text
optimized_artifact_sha256=d4b27d49c9aba3502b46cf75637f1fe2a4707143a1f01afbbf7315bed52b2efa
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

The measured 500-bot production-ready claim was green for this artifact, but
it is not a claim for the newer live current artifact recorded above. The
claim was deliberately narrow:

```text
production-ready for measured 500 bots / 32 view / 32 simulation / creative block
```

Fresh evidence:

- Soak: `reports/production-500-soak-gate.txt`
- Repeat quorum: `reports/production-500-repeat-quorum.txt`
- Repeat dir: `reports/release-repeat-20260519-022412`
- Readiness gate: `reports/production-500-readiness-gate.txt`
- Bundle: `reports/production-500-readiness-bundle-20260519-040502`
- Stable bundle at that time, now historical/stale for the live artifact:
  `reports/production-500-readiness-bundle-current`
- Published claim: `reports/production-500-claim-current.{txt,md,json}`

Final readiness result:

```text
production_ready_500_claim=true
readiness_gate_pass=true
failure_count=0
soak_gate_pass=true
repeat_quorum_pass=true
plugin_matrix_pass=true
restart_recovery_pass=true
forced_ticket_persistence_pass=true
artifact_hashes_pass=true
current_artifact_consistency_pass=true
```

The fresh `2400s` soak produced cold `19.77 / 18.86 TPS` with `66.82 ms`
max MSPT and warm `19.92 / 19.38 TPS` with `56.90 ms` max MSPT. Both
surfaces reached `500` players and `5476` loaded chunks with zero
watchdog/sync-load/stability failures. The repeat quorum produced three
same-artifact passes with zero failures:

| run | cold TPS avg/min/MSPT max | warm TPS avg/min/MSPT max |
| --- | --- | --- |
| 1 | `19.75 / 19.18 / 71.48` | `19.76 / 19.30 / 59.26` |
| 2 | `19.86 / 19.67 / 54.93` | `19.92 / 19.65 / 50.04` |
| 3 | `19.92 / 19.70 / 53.35` | `19.81 / 19.55 / 56.28` |

During this refresh, the readiness harness was fixed so a fresh repeat refresh
evaluates the exact `production_release_repeat_out_dir` it just created. That
prevents old-artifact historical repeat dirs from being mixed into a
same-artifact claim.

Non-claims remain unchanged: this is not literal unlimited scale, not a full
Rust Paper runtime, not arbitrary plugin compatibility, not real-player
gameplay parity, and not a multi-hour soak claim.

## Current 2026-05-18 CEST: P250 extreme stress now has complete evidence and is still red

The evidence harness was hardened during this pass:

- `scripts/run_load_test.sh` now treats late console FIFO writes as
  best-effort, so a server that is already stopping cannot abort summary
  generation with `SIGPIPE`.
- `scripts/run_stress_mixed_load_gate.sh` now keeps going after a non-zero
  load-test exit long enough to write/read the gate report when a summary is
  present.

Fresh P250 stress-mixed runs on the current artifact:

- Artifact: `68c170ae8313396beb38603ca69ef526a732b370ff0eeba34212b9d926a667ac`
- Runtime launcher: `b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a`
- Full-corpus P250, `300` zombies, worker10/send60/gen20:
  `reports/load-extreme-stress-mixed-worker10-send60-gen20-mob300-harnessfix-250-20260518-055503-summary.txt`
- Gate:
  `reports/extreme-stress-mixed-worker10-send60-gen20-mob300-harnessfix-250-20260518-055503-gate.txt`
- Result: `gate_pass=false`, `online_max=229`, `bot_connected_max=250`,
  `bot_ready_max=250`, `bot_active_max=229`, `bot_errors_max=21`,
  `load_window_tps1_avg=12.32`, `load_window_tps1_min=7.58`,
  `load_window_avg_tick_ms_avg=79.43`,
  `load_window_avg_tick_ms_max=144.78`, `watchdog_thread_dumps=0`,
  `sync_load_stack_hits=0`, `moved_too_quickly_warnings=10566`.
- Slow-move control, same corpus and 300 zombies:
  `reports/load-extreme-stress-mixed-worker10-send60-gen20-mob300-slowmove-250-20260518-060053-summary.txt`
- Result: `gate_pass=false`, `online_max=218`, `bot_connected_max=235`,
  `bot_ready_max=250`, `bot_active_max=218`, `bot_errors_max=32`,
  `load_window_tps1_avg=13.53`, `load_window_tps1_min=9.35`,
  `load_window_avg_tick_ms_avg=112.30`,
  `load_window_avg_tick_ms_max=1237.58`, `watchdog_thread_dumps=0`,
  `sync_load_stack_hits=0`, `moved_too_quickly_warnings=8876`.

Conclusion: P250 is not production-ready. Worker throttling removed the
watchdog/sync-load failure class at this tier, but the server still misses full
active online, TPS, average MSPT, and movement-warning gates. Slow movement
reduced warning count but worsened active online, errors, RSS, and MSPT tail,
so movement speed is not the primary fix.

## Current 2026-05-18 CEST: stress mixed-gameplay 50-bot gate is green on the current artifact

The current optimized artifact is:

```text
optimized_artifact_sha256=68c170ae8313396beb38603ca69ef526a732b370ff0eeba34212b9d926a667ac
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

Fresh mixed-gameplay verification:

- Summary: `reports/load-stress-mixed-gameplay-50-dfc-forceload-async-20260518-summary.txt`
- Gate: `reports/load-stress-mixed-gameplay-50-dfc-forceload-async-20260518-gate.txt`
- `online_max=50`
- `load_window_reached_full_online=true`
- `load_window_tps1_avg=18.33`
- `load_window_tps1_min=15.88`
- `load_window_avg_tick_ms_avg=26.84`
- `load_window_avg_tick_ms_max=78.32`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `stability_failures=0`

This is a real mixed-gameplay pass, not a 500-bot or unlimited claim. The next
work is to take the same current artifact to the next tier and keep the
failure mode honest if it turns red again.

## Current 2026-05-17 CEST: default ImprovedNoise native hook is measured, but mixed-gameplay gate is still red

The current optimized artifact is:

```text
optimized_artifact_sha256=b58b307f17ee68e868105473a393d3696ac6c5356fd9afa27d3e9a4188681bc0
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

`scripts/prepare_fast_runtime.sh` now enables only the guarded
`ImprovedNoise` native runtime hook by default:

```text
PAPER_NATIVE_IMPROVED_NOISE=true
PAPER_NATIVE_PERLIN_NOISE=false
```

This is intentionally not a broad native-noise enable. The combined
`ImprovedNoise + PerlinNoise` run was rejected because it regressed the
same 50-bot mixed-gameplay stress profile. The temporary holder-cache
visitor patch was also rejected and removed.

Fresh no-env default verification:

- Summary: `reports/load-stress-mixed-gameplay-50-default-improved-20260517-182723-summary.txt`
- `native_improved_noise_loaded=true`
- `native_perlin_noise_loaded=false`
- `online_max=50`, 26 plugin jars, 10 datapacks, 150 requested/spawned mobs
- zero kicks, bot errors, moved-too-quickly warnings, watchdog dumps,
  sync-load hits, and stability failures
- gate still failed on TPS:
  `load_window_tps1_avg=15.22 < 18.00` and
  `load_window_tps1_min=5.14 < 15.00`

Paired same-artifact comparison against native-noise disabled is recorded in
`reports/compare-current-nonative-vs-default-improved-20260517.txt`:

| metric | non-native | default ImprovedNoise | verdict |
| --- | ---: | ---: | --- |
| `load_window_tps1_avg` | `14.73` | `15.22` | better |
| `load_window_tps1_min` | `5.17` | `5.14` | slightly worse |
| `load_window_avg_tick_ms_avg` | `48.86` | `48.35` | better |
| `load_window_avg_tick_ms_max` | `108.19` | `92.19` | better |
| `process_rss_mib_max` | `24299.0` | `23743.7` | better |

Rejected candidates from this pass:

- `0102-Cache-density-function-holder-map-values.patch`: deleted after
  `holdercache` worsened TPS min, avg/max tick, and RSS versus 8192.
- `PAPER_NATIVE_IMPROVED_NOISE=true PAPER_NATIVE_PERLIN_NOISE=true`: rejected
  after `native-both` worsened TPS avg/min, avg/max tick, and RSS versus the
  non-native 8192 baseline.

Decision: keep default `ImprovedNoise` native hook as a measured guarded
runtime improvement, keep `PerlinNoise` native off, and do not claim
production-ready mixed gameplay. The next bottleneck remains raising the
50-bot stress-corpus mixed-gameplay profile to the 18/15 TPS gate and then
profiling the remaining worldgen/chunk-streaming work.

## Current 2026-05-17 CEST: mixed-gameplay stress gate is real and red on TPS

The next scale target now has a real `mixed-gameplay` harness instead of the
old movement-only `mixed` behavior. New pieces:

```bash
MC_EULA_AGREE=true ./scripts/run_stress_mixed_gameplay_gate.sh
```

The bot swarm now emits separate mixed counters for held-item switches,
animation, player input, use-item, commands, block place/dig, and mixed action
errors. `CompatProbe` now exposes server-side command, item-held, animation,
interact, and entity-damage counters. Entity attack packets are deliberately
opt-in through `BOT_MIXED_USE_ENTITY_ATTACKS=true`; the default is off because
a smoke run proved unsafe entity-id attacks can kick clients with
`invalid_entity_attacked`.

Fresh verification:

- 4-bot mixed-gameplay smoke passed with `online_max=4`,
  `bot_kicked_max=0`, `moved_too_quickly_warnings=0`,
  `bot_mixed_action_errors_max=0`, block place/dig counters active, and
  server-side command/item/animation/interact counters active.
- 50-bot full stress-corpus mixed-gameplay diagnostic reached all bots with
  26 plugins, 10 datapacks, 150 spawned zombies, zero kicks, zero mixed action
  errors, zero moved-too-quickly warnings, zero watchdog dumps, and zero
  sync-load hits.
- The 50-bot gate failed honestly on TPS only:
  `load_window_tps1_avg=13.97 < 18.00` and
  `load_window_tps1_min=5.42 < 15.00`.
- Mixed workload coverage in that 50-bot run included
  `bot_mixed_held_item_packets_max=1300`,
  `bot_mixed_arm_animation_packets_max=3650`,
  `bot_mixed_player_input_packets_max=1300`,
  `bot_mixed_use_item_packets_max=1300`,
  `bot_mixed_command_packets_max=250`,
  `bot_mixed_block_place_packets_max=1300`,
  `bot_mixed_block_dig_packets_max=1300`,
  and `compat_probe_interact_events_max=6141`.

Decision: the next real bottleneck is not harness coverage anymore; it is
raising `stress-mixed-gameplay` from `13.97/5.42 TPS` to the gate requirement
without reintroducing watchdogs, sync-load, kicks, or packet-action errors.

## Current 2026-05-17 CEST: P100 warm plateau is isolated past movement

The extreme stress mixed profile now has warm-source axis diagnostics. The
important result is that movement packet spam is not the primary blocker.

New harness support:

```bash
MC_EULA_AGREE=true ./scripts/run_extreme_plateau_axis_matrix.sh
```

`scripts/run_load_test.sh` now exposes `BOT_MOVE_INTERVAL_MS` and
`BOT_SEND_STATIONARY_POSITIONS`. `scripts/mc_bot_swarm.cjs` respects
`--send-stationary-positions=false`, so parked bots can be true idle clients
with zero synthetic `position_look` packets. New summaries and gate reports
include `bot_speed_blocks_per_second`, `bot_move_interval_ms`,
`bot_send_stationary_positions`, `bot_position_packets_max`,
`bot_positions_per_sec_max`, and chunk receive rate fields.

Fresh diagnostic outcomes:

- Warm-source P100 from the cold P100 world reached all bots but still failed:
  `online_max=100`, `tps1_avg=14.10`, `tps1_min=6.23`,
  `avg_tick_ms_avg=157.99`, `avg_tick_ms_max=629.12`,
  `watchdog_thread_dumps=2`, `sync_load_stack_hits=2`.
- Slow-move P100 (`speed=12`, `move_interval=500ms`, 150 zombies) reached all
  bots and removed watchdog/sync-load, but worsened tick tail:
  `tps1_avg=12.75`, `tps1_min=3.12`, `avg_tick_ms_avg=235.18`,
  `avg_tick_ms_max=1441.92`.
- True-idle P100 (`speed=0`, `BOT_SEND_STATIONARY_POSITIONS=false`,
  150 zombies) reached all bots with `bot_position_packets_max=0`,
  `moved_too_quickly_warnings=0`, `watchdog_thread_dumps=0`,
  `sync_load_stack_hits=0`, but still failed on TPS/max MSPT:
  `tps1_avg=13.02`, `tps1_min=9.64`, `avg_tick_ms_avg=60.82`,
  `avg_tick_ms_max=185.88`.
- No-mob true-idle P100 improved MSPT tail (`avg_tick_ms_avg=56.71`,
  `avg_tick_ms_max=134.71`) but only reached `online_max=97` /
  `bot_active_max=97` with `bot_errors_max=3`, so it is not a pass and not a
  claim.

Decision: do not spend the next pass on another movement-only tweak or the
rejected native ImprovedNoise toggle. The next real target is the persistent
density/worldgen/chunk-streaming/plugin/entity idle overhead visible even when
position packets are zero.

## Current 2026-05-17 CEST: extreme scale diagnostics are now above 50 bots

The project now has a new extreme-scale ladder and an actual P100 stress
mixed diagnostic on the current artifact. The runner is:

```bash
MC_EULA_AGREE=true ./scripts/run_extreme_scale_ladder.sh
```

The first measured tier, `P100` stress mixed with full corpus and fresh
worldgen, failed honestly and produced a real hot-frame summary:

- `bot_connected_max=76`
- `bot_ready_max=76`
- `bot_active_max=76`
- `load_window_reached_full_online=false`
- `load_window_online_max=67`
- `load_window_tps1_avg=9.38`
- `load_window_tps1_min=4.74`
- `load_window_avg_tick_ms_avg=71.67`
- `load_window_avg_tick_ms_max=141.90`
- `watchdog_thread_dumps=8`
- `sync_load_stack_hits=6`
- `moved_too_quickly_warnings=55`

The corresponding thread-sample summary points at:
`ImprovedNoise.noise(...)`, `DensityFunctions$HolderHolder.mapAll(...)`,
`DensityFunctions$Ap2.mapAll(...)`, `DensityFunctions$MarkerOrMarked.mapAll(...)`,
`CubicSpline$Multipoint.mapAll(...)`, and `ServerChunkCache.syncLoad(...)`.

A follow-up P100 run with `paper.nativeImprovedNoise=true` also failed:
`bot_connected_max=78`, `bot_ready_max=78`, `bot_active_max=78`,
`load_window_online_max=66`, `load_window_tps1_avg=7.27`,
`load_window_tps1_min=2.45`, `watchdog_thread_dumps=7`,
`sync_load_stack_hits=4`, and `nearby_players_stack_hits=1`. This rules out
the native improved-noise toggle as a sufficient fix for the extreme profile.

That is a better optimization target than the earlier 50-bot evidence, but it
is still a failure, not a claim.

## Previous 2026-05-17 CEST: historical top-level production-ready 500 certification gate

The measured `500 bots / production ready` claim now has a top-level
certification runner and verifier:

```bash
MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh
```

Fresh outcome:

- `production_ready_500_claim=true`
- `readiness_gate_pass=true`
- `failure_count=0`
- `soak_gate_pass=true`
- `repeat_quorum_pass=true`
- `plugin_matrix_pass=true`
- `restart_recovery_pass=true`
- `forced_ticket_persistence_pass=true`
- `artifact_hashes_pass=true`
- `artifact_hash_count=12`
- `repeat_passes=3`
- `readiness_run_report=reports/production-500-readiness-run-20260517-091520.txt`
- `claim_bundle=reports/production-500-readiness-bundle-20260517-091520`
- `claim_bundle_current=reports/production-500-readiness-bundle-current`
- `claim_bundle_index=reports/production-500-readiness-bundle-20260517-091520/bundle.json`
- `bundle_validation_pass=true`
- `claim_assertion_pass=true`
- `claim_publication_pass=true`
- `claim_verdict=reports/production-500-claim-verdict-20260517-091520.txt`
- `claim_publication=reports/production-500-claim-current.{txt,md,json}`

This top-level gate consumes the 30-minute cold+warm soak, the three-pass
repeat quorum, fresh plugin matrix evidence, fresh restart/recovery evidence,
fresh forced-ticket persistence evidence, and the current artifact hash
manifest. The latest runner refreshed the compatibility/recovery layer and
then wrote `reports/production-500-readiness-gate.txt` with sha256 hashes for
each evidence file. It now exports a self-contained claim bundle with
`CLAIM.md`, `MANIFEST.txt`, `bundle.json`, copied evidence files, an
independent validator, and a claim assertion report that prints the exact
allowed claim text only after validation:

```bash
python3 scripts/validate_production_readiness_bundle.py \
  reports/production-500-readiness-bundle-20260517-091520
python3 scripts/assert_production_ready_claim.py \
  reports/production-500-readiness-bundle-20260517-091520
scripts/production_ready_claim.sh
python3 scripts/publish_production_ready_claim.py
```

The artifact for that historical claim was:

```text
optimized_artifact_sha256=4064700022a879d83b16323cfbd0a769caf4551fdd8ed21dc7332afdd39d6b47
soak_report_sha256=d0700e75d6588f36e79ad5bbe8ce64ecc16c8677a7f41be40c74d23255449c3e
repeat_report_sha256=dabe75757ddcb4153fb8b91c29e45a667fddd76ce491a69bb7bdeb58786e44cc
plugin_matrix_summary_sha256=0273efad76e154ad13421ce75e158991830bc85e05bd761c3f9931765eacf301
restart_recovery_summary_sha256=476bd7bfd258f2dc653648a2a3034f2910f0845b264305189096679ebbc05d29
forced_ticket_summary_sha256=b9e2ac162245d07e77f8f4be45897bfcb5a007606d8e2aed48399511a7e89882
```

Fresh supporting compatibility checks from this pass:

- plugin matrix: `Done (21.929s)`, `Initialized 11 plugins`,
  `COMPAT_PROBE command=ok events=4`, join/quit observed, async/sync
  scheduler ticks observed, and `LibraryProbe` loaded through the Paper
  plugin-library path
- restart/recovery: `Done (15.527s)`, `COMPAT_PROBE command=ok events=2`,
  and `Saved the game`
- forced-ticket persistence: first/restart `Done (11.386s)` /
  `Done (8.551s)`, and chunk `[0, 0]` remained force-loaded after restart

The exact allowed claim emitted by the claim assertion layer is:

```text
production-ready для измеренного 500 bots / 32 view / 32 simulation / creative block профиля на проверенном artifact, с cold+warm soak, repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence и валидируемым self-contained evidence bundle.
Это не claim про полный Rust Paper runtime, unlimited plugins, real-player gameplay или multi-hour soak.
```

Decision at that point: the honest top-level claim was "production-ready for
the measured 500-bot, 32/32, creative-block profile on the verified artifact,
with tested plugin-matrix/restart/forced-ticket support." The limits remained
explicit: this was not a full Paper runtime rewrite to Rust, not unbounded
plugin compatibility, not unmeasured real-player gameplay, and not a multi-hour
soak claim. The current 2026-05-19 refresh above supersedes this artifact.

## Previous 2026-05-17 CEST: 30-minute cold+warm soak gate for measured production 500

The measured `500 bots / production ready` statement is now backed by a
soak gate on top of the earlier three-pass release quorum. The accepted claim
is still deliberately narrow: current optimized artifact, 500 bots, 32 view
distance, 32 simulation distance, creative block workload, cold/fresh plus
warm-source surfaces, `PAPER_CHUNK_WORKER_THREADS=10`,
`PAPER_PLAYER_MAX_SEND_RATE=60`, and `PAPER_PLAYER_MAX_GEN_RATE=20`.

The soak runner is:

```bash
MC_EULA_AGREE=true ./scripts/run_production_soak_gate.sh
```

It runs a cold/fresh load gate and a warm-source load gate with a dynamically
enforced duration floor. With the default 600-second block ramp, 300 required
load-window samples, 5-second metrics interval, and 300-second buffer, that
floor is `2400` seconds per surface; shorter explicit
`PRODUCTION_SOAK_DURATION_SECONDS` values fail before the run starts. The gate
then writes:

```text
reports/production-500-soak-gate.txt
```

Fresh soak outcome:

- `production_ready_soak_claim_eligible=true`
- `soak_gate_pass=true`
- `failure_count=0`
- `base_cold_gate_pass=true`
- `base_warm_gate_pass=true`
- `artifact_hashes_pass=true`
- `required_load_window_metrics_samples_min=300`
- `required_block_place_packets_min=120000`
- `required_block_dig_packets_min=120000`
- optimized artifact
  `sha256=4064700022a879d83b16323cfbd0a769caf4551fdd8ed21dc7332afdd39d6b47`
- runtime launcher
  `sha256=28c1c4832fce638503443a6dcb8443a69f8890fb7015fa5152b3440f641a4cfd`

Soak matrix:

| surface | samples | TPS avg/min | MSPT avg/max | packets place/dig | RSS max | stability |
| --- | ---: | --- | --- | --- | ---: | --- |
| cold/fresh | `357` | `19.84 / 19.19` | `41.98 / 60.48` | `264000 / 264000` | `12388.7 MiB` | `0 watchdog, 0 sync-load, 0 stability` |
| warm-source | `359` | `19.95 / 19.28` | `38.68 / 56.32` | `267500 / 267000` | `5164.6 MiB` | `0 watchdog, 0 sync-load, 0 stability` |

Both soak surfaces reached `online_max=500` and
`loaded_chunks_max=5476`. Teardown tails are still reported separately for
diagnostics (`cold_avg_tick_ms_max=239.73`,
`warm_avg_tick_ms_max=327.47` overall), but the production gate is based on
the load window that includes startup, cold join/chunk generation, reaching
500 online, and the full 500-online block phase before shutdown/disconnect.

Decision: the honest claim is now "soak-backed production-ready for the
measured 500-bot, 32/32, creative-block profile on the verified current
artifact." This is stronger than the prior repeat quorum, but it is not a
claim that the full Paper runtime is rewritten to Rust, not a claim of
unbounded plugin compatibility, and not proof for unmeasured real-player
gameplay.

## Previous 2026-05-17 CEST: 3-pass quorum for measured production 500 release

The measured `500 bots / production ready` claim is now backed by a repeat
quorum, not a single lucky run. The accepted profile is still deliberately
narrow: current optimized artifact, 500 bots, 32 view distance, 32 simulation
distance, creative block workload, cold/fresh plus warm-source release gates,
`PAPER_CHUNK_WORKER_THREADS=10`, `PAPER_PLAYER_MAX_SEND_RATE=60`, and
`PAPER_PLAYER_MAX_GEN_RATE=20`.

The canonical single release rerun command is:

```bash
MC_EULA_AGREE=true ./scripts/run_production_release_gate.sh
```

That wrapper records artifact hashes, runs both load gates, then evaluates
`reports/production-500-release-gate.txt`.

The repeat harness is:

```bash
MC_EULA_AGREE=true \
  PRODUCTION_RELEASE_REPEAT_COUNT=2 \
  ./scripts/run_production_release_repeat_gate.sh
```

The quorum verifier is:

```bash
python3 scripts/evaluate_production_release_repeat.py \
  --repeat-dir auto \
  --min-passes 3 \
  --report reports/production-500-repeat-quorum.txt
```

Fresh quorum outcome from `reports/production-500-repeat-quorum.txt`:

- `required_min_passes=3`
- `repeat_dir_count=2`
- `repeat_run_count=3`
- `repeat_passes=3`
- `repeat_failures=0`
- `repeat_quorum_pass=true`
- all three release reports have `production_ready_claim_eligible=true`,
  `release_gate_pass=true`, and `failure_count=0`
- every run used the same optimized artifact
  (`sha256=4064700022a879d83b16323cfbd0a769caf4551fdd8ed21dc7332afdd39d6b47`)
  and runtime launcher
  (`sha256=28c1c4832fce638503443a6dcb8443a69f8890fb7015fa5152b3440f641a4cfd`)

Repeat runs preserved as release evidence:

| run | report dir | cold load-window TPS avg/min/max MSPT | warm load-window TPS avg/min/max MSPT |
| --- | --- | --- | --- |
| 1 | `reports/release-repeat-20260517-033126/run-1` | `19.84 / 18.62 / 61.17` | `19.88 / 19.32 / 53.27` |
| 2 | `reports/release-repeat-20260517-041001/run-1` | `19.91 / 18.72 / 54.87` | `19.90 / 19.12 / 59.48` |
| 3 | `reports/release-repeat-20260517-041001/run-2` | `19.84 / 19.06 / 55.86` | `19.90 / 19.33 / 56.58` |

All three repeat runs reached `online_max=500` and
`loaded_chunks_max=5476` for both cold and warm surfaces, with zero
watchdog thread dumps, zero sync-load stack hits, zero stability failures, and
the creative block packet workload completed.

Decision: the measured release claim is now quorum-backed for the current
optimized artifact: 500 bots, 32/32 view and simulation distance, creative
block workload, cold/fresh plus warm-source evidence, worker10 chunk workers,
send rate 60, generation rate 20, and verified artifact hashes.

Rejected comparison: the earlier full current-artifact release runner with
worker10/send60 but default generation rate failed cold/fresh only on
`load_window_tps1_min=17.92 < 18.00`. Capping player generation at 20 fixed the
early cold chunk-generation spike while preserving the 500-online block phase.

Scope limits remain strict: this does not claim a full Paper runtime rewrite
to Rust, arbitrary plugin compatibility, or unmeasured real-player gameplay.

## Previous 2026-05-16: cold/fresh worker8 gate introduced load-window metrics

This continuation split load-test metrics into the active load window and the
post-run teardown tail. The new `load_window_*` fields cover startup, cold
join/chunk generation, reaching the full target, and the full 500-online block
phase until the first metrics sample where online count drops after reaching
the target. Overall metrics remain in the summary for diagnostics.

The passing cold/fresh run used the default heap and pinned chunk workers to 8:

```bash
MC_EULA_AGREE=true \
  PAPER_CHUNK_WORKER_THREADS=8 \
  LOAD_TEST_LABEL=production-500-cold-worker8-defaultheap-windowed-20260516-223952 \
  ./scripts/run_production_claim_gate.sh
```

Gate outcome:

- `report=reports/load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-summary.txt`
- `gate=reports/load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-gate.txt`
- `claim_eligible=true`
- `gate_pass=true`
- `failure_count=0`
- `world_mode=fresh`
- `claim_surface=cold-fresh`
- `online_max=500`
- `loaded_chunks_max=5476`
- `bot_block_armed_max=500`
- `bot_block_primed_max=500`
- `bot_block_place_packets_max=59000`
- `bot_block_dig_packets_max=59000`
- `bot_block_action_errors_max=0`
- `load_window_tps1_avg=19.55`
- `load_window_tps1_min=18.07`
- `load_window_avg_tick_ms_avg=42.61`
- `load_window_avg_tick_ms_max=65.09`
- `process_rss_mib_max=12044.1`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `stability_failures=0`

Decision: the broad cold/fresh `production-500` claim is now gate-backed for
this benchmark profile: 500 bots, 32/32 view/simulation distance, creative
block workload, default heap, and 8 Paper chunk workers.

Scope limits still matter: this is not a claim that the entire Paper runtime
has been rewritten to Rust, not a claim of unlimited plugin compatibility, and
not proof for arbitrary real-player gameplay beyond the measured 500-bot block
profile.

Rejected/diagnostic runs from the same pass:

- `production-500-cold-worker8-xms16g-20260516-220341`: failed with
  `claim_eligible=false`, `bot_exit=75`, `tps1_min=17.02`; the 16G/pre-touch
  heap profile hurt the 500-online block phase.
- `production-500-cold-worker8-defaultheap-20260516-221636`: full old-window
  run reached the shape but failed on teardown-inclusive metrics
  (`tps1_min=17.15`, `avg_tick_ms_max=226.65`). This led to the load-window
  split so production load and post-run teardown are reported separately.

## Previous 2026-05-16: native Perlin runtime hook is diagnostic only

The native `PerlinNoise` runtime path was enabled after the `0099` hook and
loaded correctly:

- `label=production-500-native-perlin-post0099-20260516-214550`
- `runtime=native_perlin_noise=true`
- log evidence: `Paper: Using native PerlinNoise from paper_native_jni.`
- current launcher policy is narrower now: Perlin stays explicit opt-in and
  the split no-y-scale/generic flags are not enabled by default.

It still did not support a cold/fresh-world `500 bots / production ready`
claim. The run was stopped early once the strict gate was mathematically unable
to pass:

- `online=59`
- `loadedChunks=3248`
- `tps1=17.65`, below the required `>=18.00`
- `avgTickMs=84.30`

Decision: keep native Perlin disabled by default. The hook is useful for
diagnostics/parity work, but the next production blocker remains the early
cold join/chunk/worldgen spike.

## Previous 2026-05-16: cold/fresh 500 gate reached full shape but still fails early join/load thresholds

After the `0097` warm pass, the broader cold/fresh-world claim gate was run without `LOAD_TEST_WORLD_SOURCE`:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=production-500-block-500bots-post0097-20260516-200739 ./scripts/run_production_claim_gate.sh
```

It reached the requested load shape but failed the strict production gate:

- `report=reports/load-production-500-block-500bots-post0097-20260516-200739-summary.txt`
- `gate=reports/load-production-500-block-500bots-post0097-20260516-200739-gate.txt`
- `claim_eligible=false`
- `failure_count=3`
- `online_max=500`
- `loaded_chunks_max=5476`
- `bot_block_place_packets_max=60000`
- `bot_block_dig_packets_max=59500`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `stability_failures=0`
- `tps1_avg=19.38` vs required `>=19.50`
- `tps1_min=12.80` vs required `>=18.00`
- `avg_tick_ms_max=117.43` vs allowed `<=100.00`

The bad window was early player/chunk loading, not the final 500-online block-action plateau:

- `min_tps at online=75 loadedChunks=5106 tps1=12.80 avgTickMs=112.72`
- `max_tick at online=65 loadedChunks=4186 tps1=14.87 avgTickMs=117.43`
- `first_500 at online=500 loadedChunks=5476 tps1=19.41 avgTickMs=51.11`
- final sample held `online=500 loadedChunks=5476 tps1=19.92 avgTickMs=46.22`

The honest status at that checkpoint was:

- warm saved-world `production-500-warm`: PASS
- cold/fresh-world `production-500`: FAIL, very close, blocked by early join/load spike

## Previous 2026-05-16: `0097` passes the warm 500 gate; warm-only evidence is gate-backed

This continuation added two runtime patches and reran the full optimized build and warm 500 gate on the same saved world:

- `0096` specializes `ServerLevel.getNearestPlayer(...)` for `predicate == null`
- `0097` skips the tracked-entity nearby-player scan when the tracked chunk and update count did not change

Verified outcomes:

- `./gradlew applyPatches :paper-server:compileJava`: PASS
- `MC_EULA_AGREE=true ./scripts/build_optimized.sh`: PASS
- `python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt`: PASS

Warm 500 gate outcome on `LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310`:

- `report=reports/load-production-500-warm-block-500bots-post0097-20260516-194812-summary.txt`
- `gate=reports/load-production-500-warm-block-500bots-post0097-20260516-194812-gate.txt`
- `claim_eligible=true`
- `online_max=500`
- `loaded_chunks_max=5476`
- `tps1_avg=19.86`
- `tps1_min=19.03`
- `avg_tick_ms_avg=36.95`
- `avg_tick_ms_max=67.13`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `stability_failures=0`
- `process_rss_mib_max=5149.8`

The honest status is now:

- warm 500 production gate: PASS
- cold/fresh-world gate still separate by policy

## Previous 2026-05-16: `0086-0088` are built; warm 500 is close but still fails TPS

This continuation added three real runtime patches and rebuilt the optimized runtime after regenerating the Paper patch stack.

- `0086` reuses the entity chunk-sent key inside `ChunkMap$TrackedEntity.updatePlayerFast(...)`
- `0087` adds a reference fast path in `Ticket.compareTo(...)`
- `0088` specializes nearest-player scans for `EntitySelector.PLAYER_AFFECTS_SPAWNING`

Verified outcomes so far:

- `ref_fast_speedup=1.070x` on `bench/ticket-compare`
- `specialized_speedup=1.067x` on `bench/nearest-affects-spawning`
- `BUILD_NATIVE=false scripts/build_optimized.sh`: PASS after patch stack refresh

The `production-500-warm` gate was launched with
`LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310`
and reached the full 500-bot block shape:

- `report=reports/load-production-500-warm-block-500bots-post0088-20260516-150108-summary.txt`
- `gate=reports/load-production-500-warm-block-500bots-post0088-20260516-150108-gate.txt`
- `online_max=500`
- `loaded_chunks_max=5476`
- `bot_block_armed_max=500`
- `bot_block_primed_max=500`
- `bot_block_place_packets_max=61000`
- `bot_block_dig_packets_max=60500`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `stability_failures=0`
- `avg_tick_ms_avg=44.41`
- `avg_tick_ms_max=88.62`

It still failed `production-500-warm`:

- `tps1_avg=18.42` vs required `>=19.50`
- `tps1_min=14.66` vs required `>=18.00`

The claim remains blocked. The next needed evidence is a post-`0088` JFR on the same warm 500/block shape.

## Previous 2026-05-15: fresh 500-bot JFR points at entity-tracker update path, not PlayerList anymore

The current warm diagnostic run used the same warm source as the claim gate
but captured a fresh JFR after the `PlayerList` broadcast tweak. The runtime
is still not production-ready, and the remaining hot path is now entity
tracking.

- `report=reports/load-warm500-playerlist-jfr2-20260515-summary.txt`
- `jfr=reports/load-warm500-playerlist-jfr2-20260515.jfr`
- `online_max=500`
- `loaded_chunks_max=5476`
- `tps1_avg=13.71`
- `avg_tick_ms_avg=98.19`
- `watchdog_thread_dumps=3`
- `nearby_players_stack_hits=5`

Top JFR methods:

- `net.minecraft.server.level.ChunkMap$TrackedEntity.updatePlayer(ServerPlayer)` `9.30%`
- `it.unimi.dsi.fastutil.objects.ReferenceOpenHashSet.contains(Object)` `6.50%`
- `net.minecraft.world.entity.Entity.getBukkitEntity()` `4.40%`
- `java.util.HashMap.getNode(Object)` `3.23%`
- `net.minecraft.world.entity.ai.targeting.TargetingConditions.test(...)` `2.58%`
- `org.bukkit.craftbukkit.entity.CraftPlayer.canSee(Entity)` `0.89%`

This is the next real hotspot to attack. The earlier PlayerList broadcast
candidate is still valid, but it is no longer the dominant cost in this JFR.

## Previous 2026-05-15: `PlayerList` broadcast candidate is built and bench-clean, but warm 500 still fails

The visible-player broadcast path now does the distance check before
`canSee(...)`, which removes one hot visibility call for out-of-radius players
without changing the send condition.

- `./gradlew :paper-server:compileJava --no-daemon`: PASS
- `MC_EULA_AGREE=true ./scripts/build_optimized.sh`: PASS
- `python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null`: PASS
- `./scripts/bench_playerlist_broadcast_cansee.sh`: PASS
- `empty_candidate_speedup=1.213x`
- `populated_candidate_speedup=1.718x`
- `equivalence=PASS`

Fresh warm 500 evidence on the same candidate:

- `report=reports/load-production-500-warm-block-500bots-20260515-221802-summary.txt`
- `gate=reports/load-production-500-warm-block-500bots-20260515-221802-gate.txt`
- `online_max=500`
- `loaded_chunks_max=5476`
- `tps1_avg=15.53`
- `tps1_min=5.44`
- `avg_tick_ms_avg=76.90`
- `avg_tick_ms_max=239.69`
- `watchdog_thread_dumps=3`
- `nearby_players_stack_hits=2`

This is better than the previous warm 500 run on TPS/MSPT/watchdog count, but
it still fails the warm production gate, so there is no honest
`500 bots / production ready` claim yet.

## Previous 2026-05-15: production claim split into cold and warm-world gates

The claim path is now split instead of pretending that the cold fresh-world
500-bot failure is close to production-ready.

- `scripts/run_load_test.sh` accepts `LOAD_TEST_WORLD_SOURCE`.
- The source may be a full saved server run containing `world/level.dat`, or a
  single world directory containing `level.dat`.
- Warm-source runs record `world_mode=warm-source`,
  `claim_surface=warm-world`, `world_warm_source_present=true`,
  `world_warm_source=...`, `world_warm_source_kind=...`, and
  `world_warm_copy_method=...` in the summary.
- By default the harness strips copied `world/playerdata`,
  `world/advancements`, and `world/stats` so the saved world warms chunks
  without reusing old bot identities or positions. Set
  `LOAD_TEST_WORLD_SOURCE_KEEP_PLAYERDATA=true` only when that is intentional.
- `scripts/evaluate_load_gate.py --profile production-500` stays the
  cold/fresh-world gate and rejects summaries that explicitly record
  `world_warm_source_present=true`.
- `scripts/evaluate_load_gate.py --profile production-500-warm` keeps the same
  500-bot block, 32/32, TPS/MSPT, RSS, and zero-failure thresholds, but
  requires a warm source via `world_warm_source_present=true` and
  `world_mode=warm-source`.
- `scripts/run_production_warm_claim_gate.sh` is the wrapper for that narrower
  warm-world claim gate and refuses to run without `LOAD_TEST_WORLD_SOURCE`.
- The harness now writes `plugins/spark/config.json` with
  `backgroundProfiler=false` by default and records
  `spark_background_profiler=false` in summaries. Production gates reject a
  summary that explicitly records `spark_background_profiler=true`.

The first full warm-world gate used
`LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310`
and reached the full requested shape:

- `report=reports/load-production-500-warm-block-500bots-20260515-210412-summary.txt`
- `gate=reports/load-production-500-warm-block-500bots-20260515-210412-gate.txt`
- `bots=500`
- `world_mode=warm-source`
- `online_max=500`
- `loaded_chunks_max=5476`
- `bot_block_armed_max=500`
- `bot_block_primed_max=500`
- `bot_block_place_packets_max=57000`
- `bot_block_dig_packets_max=57000`
- `bot_kicked_max=0`
- `bot_errors_max=0`

It still failed `production-500-warm`:

- `tps1_avg=15.15` vs required `>=19.50`
- `tps1_min=4.55` vs required `>=18.00`
- `avg_tick_ms_avg=81.72` vs allowed `<=50.00`
- `avg_tick_ms_max=271.61` vs allowed `<=100.00`
- `watchdog_thread_dumps=5` vs allowed `0`

The action phase held 500 bots but sat around `5.2-5.6 TPS` while block
places/breaks climbed to `57000/56625`. The watchdog dumps appeared during the
mass disconnect/stop path, with stacks in `AdventureCodecs`/`PlayerList.remove`,
`ChunkHolderManager`/`RegionizedPlayerChunkLoader`, `PlayerAdvancements.save`,
and LuckPerms shutdown. `nearby_players_stack_hits=0` in this clean-profiler
rerun. The follow-up smoke verified the new
Spark-profiler-off config: `reports/load-warm-source-smoke-sparkoff-5-20260515-summary.txt`
recorded `spark_background_profiler=false` and passed the warm evaluator with
`--min-bots 5`.

This creates an honest gate surface and records a real failed full warm run. It
is not a 500-bot production-ready claim.

## Previous 2026-05-15 11:12 CEST: 500-bot block gate reached full online, SurfaceRules chain candidate rejected and reverted

This continuation tested a new `SurfaceRules.SequenceRule` linked-chain runtime
shape. The standalone model improved over the current list/enhanced-for path
but did not beat the already-rejected array/indexed model:

- `reports/surfacerules-sequence-array-bench.txt`
- `rules=14`
- `list_enhanced_best_ms=529.303`
- `array_indexed_best_ms=283.931`
- `linked_best_ms=459.239`
- `linked_speedup=1.153x`
- `equivalence=PASS`

The temporary feature patch `0064-Optimize-SurfaceRules-sequence-chain.patch`
compiled, built, refreshed AppCDS/artifact hashes, and passed plugin matrix,
restart/recovery, and forced-ticket persistence before load testing. The real
500-bot creative block scenario then reached the requested online/action shape:

- `report=reports/load-block-500-surfacerules-chain-20260515-summary.txt`
- `bots=500`
- `load_test_scenario=block`
- `load_test_gamemode=creative`
- `view_distance=32`
- `simulation_distance=32`
- `block_action_interval_ms=1000`
- `online_max=500`
- `loaded_chunks_max=5476`
- `bot_block_place_packets_max=60000`
- `bot_block_dig_packets_max=60000`
- `compat_probe_block_places_max=59500`
- `compat_probe_block_breaks_max=59000`
- `bot_kicked_max=0`
- `bot_errors_max=0`

It failed the production claim gate decisively:

- `tps1_avg=8.98` vs required `>=19.50`
- `tps1_min=3.97` vs required `>=18.00`
- `avg_tick_ms_avg=148.42` vs allowed `<=50.00`
- `avg_tick_ms_max=626.01` vs allowed `<=100.00`
- `watchdog_thread_dumps=5` vs allowed `0`
- `nearby_players_stack_hits=8` vs allowed `0`

`python3 scripts/evaluate_load_gate.py --profile production-500
reports/load-block-500-surfacerules-chain-20260515-summary.txt` returned
`claim_eligible=false`, `gate_pass=false`, `failure_count=6`.

Decision: reject and revert the SurfaceRules chain runtime patch. The optimized
runtime artifact was rebuilt without `0064`; AppCDS and artifact hashes were
refreshed; plugin matrix passed (`Done (25.838s)`), restart/recovery passed
(`Done (15.952s)`), and forced-ticket persistence passed (`Done (11.871s)` /
`Done (9.284s)`). The next production work should not repeat
`SurfaceRules.SequenceRule` storage/iteration shapes; the current target must
move back to noise/worldgen hot paths and watchdog removal.

## Current 2026-05-14 22:47 CEST: 100-bot block plateau now reaches full online, but TPS/MSPT are still far from production-ready

This continuation changed `scripts/run_load_test.sh` to write a per-run
`bukkit.yml` with `connection-throttle: 0` for localhost synthetic load and
then reran the creative 32/32 block scenario at 100 bots. The run reached a
full connected/ready plateau:

- `load_test_scenario=block`
- `load_test_gamemode=creative`
- `bukkit_connection_throttle=0`
- `online_max=100`
- `loaded_chunks_max=5184`
- `tps1_avg=9.88`
- `avg_tick_ms_avg=223.62`
- `bot_block_armed_max=100`
- `bot_block_primed_max=100`
- `bot_block_creative_slot_packets_max=100`
- `bot_block_place_packets_max=5046`
- `bot_block_dig_packets_max=5031`
- `compat_probe_arena_prepared_max=100`
- `compat_probe_arena_skipped_total=1176`
- `server_join_events=100`
- `server_quit_events=100`
- `process_rss_mib_max=11515.4`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `nearby_players_stack_hits=0`
- `thread_check_failures=0`
- `chunk_system_errors=0`
- `feature_placement_errors=0`
- `off_main_poi_hits=0`
- `stability_failures=0`

This is the first full 100-bot block plateau in this harness, but it is
still not production-ready. The next target is lowering the block plateau's
tick cost and chunk pressure, not making a 500-player claim.

## Current 2026-05-14 21:49 CEST: 50-bot block arena now arms all bots; still not production-ready

This continuation extended the block arena re-seat window in
`scripts/run_load_test.sh` so late joiners keep getting teleported through
the full bot window. The patched 50-bot creative 32/32 block run reached a
full armed/primed plateau:

- `load_test_scenario=block`
- `load_test_gamemode=creative`
- `online_max=50`
- `loaded_chunks_max=3055`
- `tps1_avg=12.50`
- `avg_tick_ms_avg=183.80`
- `bot_block_armed_max=50`
- `bot_block_primed_max=50`
- `bot_block_creative_slot_packets_max=50`
- `bot_block_place_packets_max=20750`
- `bot_block_dig_packets_max=20716`
- `compat_probe_block_places_max=16565`
- `compat_probe_block_breaks_max=17816`
- `compat_probe_arena_commands_max=28`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `nearby_players_stack_hits=0`
- `thread_check_failures=0`
- `chunk_system_errors=0`
- `feature_placement_errors=0`
- `off_main_poi_hits=0`

This is the first full 50-bot block plateau on this harness, but it is still
not a production-ready claim. TPS/MSPT are far below target, and the runtime
is still Java/Paper.

## Current 2026-05-14 21:24 CEST: block-aware creative arena smoke is wired; 500-player claim still not made

This continuation added a real block path to `scripts/run_load_test.sh`,
`scripts/mc_bot_swarm.cjs`, and `CompatProbe`, then verified it on a 12-bot
creative smoke:

- `load_test_scenario=block`
- `load_test_gamemode=creative`
- `online_max=12`
- `loaded_chunks_max=192`
- `bot_block_place_packets_max=624`
- `bot_block_dig_packets_max=616`
- `compat_probe_block_places_max=617`
- `compat_probe_block_breaks_max=609`
- `bot_block_action_errors_max=0`
- `watchdog_thread_dumps=0`
- `sync_load_stack_hits=0`
- `nearby_players_stack_hits=0`
- `thread_check_failures=0`
- `chunk_system_errors=0`
- `feature_placement_errors=0`
- `off_main_poi_hits=0`

This is harness validation only. It is not a 500-player claim and it does
not make the runtime Rust-native.

## Current 2026-05-14 20:09 CEST: warm-world harness is in place; area-map is auto-enabled by optimized runtime when bundled but still not strict-gate accepted

This continuation added `scripts/warm_world_benchmark.sh` and fixed a
fast-path status-ping race so the warm-world runs now write nonempty status
JSON files even when `Done` arrives quickly. The warm-world benchmark on the
saved `runs/plugin-matrix` world produced:

- `stock-paper-1.21.10`: `status_ms=73810`, `done_ms=73872`,
  `rss_kb=1828804`.
- `optimized-paper-1.21.10`: `status_ms=46223`, `done_ms=46268`,
  `rss_kb=1666672`.
- `optimized-runtime-1.21.10`: `status_ms=36121`, `done_ms=36185`,
  `rss_kb=1251232`.
- Warm-start done-time ratios: optimized Paper `1.597x` faster than stock;
  optimized runtime `2.042x` faster than stock and `1.279x` faster than
  optimized Paper.

This is startup/plugin-load evidence only. It is not a 500-player claim and
it does not change the strict-gate story.

The area-map path is source-wired through `PaperNativeAreaMap` behind
`paper.nativeAreaMap`. In the optimized runtime launcher it is enabled by
default when `libpaper_native_jni.so` is bundled, and it can be disabled with
`PAPER_NATIVE_AREA_MAP=false` or `PAPER_NATIVE_AREA_MAP=0`. This is still not
accepted load-gate evidence. Current report files show:

- `reports/native-area-map-bench.txt`: `equivalence=PASS`,
  `update_native_speedup_vs_java=1.218x`,
  `add_native_speedup_vs_java=1.216x`,
  `remove_native_speedup_vs_java=1.168x`.
- `reports/load-50bots-area-map-native-gate-20260514-summary.txt`:
  `tps1_avg=17.24`, `avg_tick_ms_avg=75.12`, `loaded_chunks_max=2766`,
  `watchdog_thread_dumps=6`, so the runtime path was rejected.
- `reports/load-50bots-nearby-player-map-presize-gate-20260514-preflight.txt`:
  `host_preflight_ok=false`, `load_per_cpu=1.946`, `idle_percent_1s=0.75`,
  so the current host still blocks a comparable strict rerun.

Decision for now: keep `area_map` diagnostic-first and keep the warm-world
benchmark as startup evidence only, not a TPS or 500-player claim.

## Current 2026-05-14 11:30 CEST: full native mega-all pack passes 96/96

This continuation ran the full native diagnostic pack across every configured
domain while keeping the Paper runtime unchanged:

- Ran `PACK_WRITE_MANIFEST=1 PACK_LABEL=mega-all-complete-v4
  PACK_FAIL_FAST=1 PACK_GROUPS=all scripts/bench_native_pack.sh`.
- The pack included worldgen, worldgen-extra, aquifer, climate, entity,
  waypoint, plugin, storage, core, and ticket diagnostics in one bounded
  pass.
- The run passed all `96` scripts: `summary_scripts=96`,
  `summary_pass=96`, `summary_fail=0`, `pack_status=PASS failures=0`, and
  `summary_status=PASS`.
- `scripts/bench_native_pack.sh` now enforces the `PACK_GROUPS=all`
  contract by comparing the selected list against every real
  `bench_native_*.sh` script, excluding only the two meta-runners. The report
  records `all_real_scripts_expected=96` and `all_real_scripts_covered=96`.
- `scripts/native_coverage_audit.py --strict-docs` now checks the same
  `PACK_GROUPS=all` contract and reports `pack_all_real_expected=96`,
  `pack_all_scripts_listed=96`, `pack_all_scripts_unique=96`, with zero
  missing, extra, or duplicate entries.
- `scripts/native_pack_report.py` now validates declared script counts,
  `all_real` counts, `PACK_START`/`PACK_RESULT` set equality, required
  `pack_status`, manifest group count, and duplicate `PACK_RESULT` scripts
  before calling a report PASS.
- `scripts/bench_native_pack.sh` now exposes `PACK_LIST_GROUPS=1` and
  `PACK_MANIFEST=1`; the current leaf manifest has `10` groups and `96`
  memberships, and the v4 report records `summary_manifest_entries=96`,
  `summary_manifest_groups=10`, `summary_manifest_groups_match_leaf_count=TRUE`,
  `leaf_group_count=10`, and `leaf_group_memberships=96`.
- The v4 report parser also records `summary_pack_status_present=TRUE`,
  `summary_started_scripts=96`, and `summary_start_result_sets_match=TRUE`.
- Total measured pack duration was `2582751 ms`.
- The slowest script was
  `scripts/bench_native_spigot_load_order_dependency.sh` at `260472 ms`.
- The report recorded `96` `equivalence=PASS` lines, `0`
  equivalence failures, `504` speedup lines, `308` lines at or above `1x`,
  and `196` below `1x`.
- Final checks after the run: `PACK_LIST=1 PACK_GROUPS=all` lists `96`
  scripts with no real `bench_native_*.sh` omissions after excluding the two
  meta-runners; `python3 scripts/native_coverage_audit.py --strict-docs`,
  `bash -n` across every `bench_native_*.sh`, `python3 -m py_compile`,
  `sha256sum -c
  reports/paper-native-jni.sha256`, and `git diff --check` all pass.
- Added `scripts/verify_native_pack_complete.sh` as the one-command contract
  check for the complete native pack baseline and current report.

Decision for now: this is the strongest current batch verification baseline
for the diagnostic Rust/native layer. It is not a Paper runtime performance
claim and does not change the rejected `NoiseChunk` runtime decision.

## Current 2026-05-14 02:33 CEST: native mega-pack runner and report summarizer are in place

This continuation added a larger orchestration layer around the modular
Rust/native diagnostic surface while keeping the Paper runtime unchanged:

- Added `scripts/bench_native_pack.sh`, a multi-domain pack runner for
  aquifer, climate, entity, waypoint, plugin, storage, and ticket native
  diagnostics. It runs the structural audit first, builds the native library
  once, then executes the selected pack with bounded defaults for the heaviest
  cases.
- Added `scripts/native_pack_report.py`, a report summarizer that counts pack
  passes/failures, duration totals, equivalence lines, and speedup lines.
- Extended `scripts/native_coverage_audit.py` to verify `System.loadLibrary`
  in wrapper files and to count actual Java wrapper sources and JNI exports.
- Added bounded pack defaults for the two heaviest scripts:
  `waypoint_hotpath` now uses `0/1` warmup/rounds in pack mode, and
  `remapper_hash_threshold` uses `3/1/2` iterations/warmup/rounds in pack
  mode. Standalone scripts keep their original defaults.
- The bounded mega-pack passed all `56` scripts:
  `script_count=56`, `summary_scripts=56`, `summary_pass=56`,
  `summary_fail=0`, `summary_total_duration_ms=2048339`, and
  `summary_status=PASS`.
- The slowest script in that pack was
  `scripts/bench_native_spigot_load_order_dependency.sh` at `272706 ms`.
  `waypoint_hotpath` stayed heavy but bounded at `217651 ms` instead of the
  earlier much longer default run.
- `python3 scripts/native_coverage_audit.py --strict-docs` still passes with
  `89` core modules, `92` covered bench dirs, `97` covered scripts,
  `90` load-library wrappers checked, `243` JNI exports checked, and `0`
  errors.

Decision for now: keep the new pack tooling diagnostic-only. It expands the
measured modular Rust surface and the batch verification path, but it does
not install any new Paper runtime hook.

## Current 2026-05-13 23:03 CEST: native coverage audit and wide worldgen pack runner are in place

This continuation added a bigger coordination layer around the existing
modular Rust/native diagnostic surface while keeping the Paper runtime
unchanged:

- Added `scripts/native_coverage_audit.py`, a structural audit for the Rust
  core module list, JNI references, native Java bench directories, native
  `PaperNative*.java` wrapper methods, and executable bench scripts. The
  current strict pass reports `89` core modules, `92` required bench dirs
  covered, `97` required scripts covered, `90` wrapper files checked, `243`
  JNI exports checked, `0` errors, and `0` doc-term warnings.
- Added `scripts/bench_native_worldgen_pack.sh`, a pack-runner for grouped
  native worldgen diagnostics. It runs the coverage audit first, builds the
  native library once, and then executes a configurable script pack.
- `scripts/bench_native_noisechunk_wrap_capacity.sh` now respects
  `SKIP_NATIVE_BUILD=1`, so the pack-runner can reuse a single native build
  when the runtime library is already fresh.
- The wide pack smoke completed with 16 grouped scripts in one invocation:
  `improved_noise`, `improved_noise_inline`, `improved_noise_derivative`,
  `perlin_noise`, `perlin_getvalue`, `blended_noise`,
  `noise_generator_settings`, `density_ap2_fill`, `density_ap2_minmax_fill`,
  `density_visitor_hooks`, `surface_rules_sequence_array`,
  `surface_rules_test_rule_state`, `placed_feature_traversal`,
  `ore_feature_loop`, `carver_iteration`, and `cave_carver_skip`. All 16
  scripts passed, and the pack report ended with `pack_status=PASS
  failures=0`.
- `cargo test --manifest-path native/Cargo.toml --workspace` still passes on
  the current tree, and `sha256sum -c reports/paper-native-jni.sha256`
  still matches the rebuilt release JNI library hash
  `4a86ff616bccffcd2fbe73e26e8a458c0e84f3e7ba88b3203fd452e0cb8dde0c`.

Decision for now: keep the new audit and pack tooling diagnostic-only. It
strengthens batch verification across the modular Rust surface, but it does
not change the Paper runtime or claim a strict-gate win.

## Current 2026-05-13 22:03 UTC: Rust compression/IO shape native batch is parity-clean

This continuation added three more modular Rust/native diagnostic checkpoints
while keeping the Paper runtime unchanged:

- Added `paper-native-core::lz4_stream_roundtrip`, a Rust LZ4 block-stream
  round-trip verifier that compresses and decompresses payloads through the
  existing Java-compatible LZ4 stream backend, then returns a parity-safe
  restored-payload and modeled-capacity summary.
- Added `paper-native-core::nbt_gzip_buffer_shape`, a Rust model for the
  current, gzip-64k, prebuffer-64k, and both-64k NBT/GZIP buffering shapes.
  This is a buffer-shape model, not a byte-identical GZIP encoder.
- Added `paper-native-core::compression_threshold_shape`, a Rust model for
  network compression threshold/framing decisions across disabled, default,
  and tighter threshold mixes. This is a framing model, not a zlib hook.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_lz4_stream_roundtrip.sh`,
  `scripts/bench_native_nbt_gzip_buffer_shape.sh`, and
  `scripts/bench_native_compression_threshold_shape.sh`.
- `cargo test --manifest-path native/Cargo.toml --workspace` passed with
  `287` `paper-native-core` tests.
- `JAVA_PROPS='-Diterations=16 -Dwarmup=1 -Drounds=3'
  ./scripts/bench_native_lz4_stream_roundtrip.sh` passed equivalence across
  `32768`, `65536`, and `131072` block sizes. Native is slower than Java on
  this host (`0.426x`, `0.404x`, `0.419x`), so this is validation evidence,
  not a native speed win.
- `JAVA_PROPS='-Drepeats=1024 -Dwarmup=1 -Drounds=3'
  ./scripts/bench_native_nbt_gzip_buffer_shape.sh` passed equivalence on all
  four buffer shapes. Native shape-model speedups were `1.735x`, `1.830x`,
  `1.708x`, and `1.699x`.
- `JAVA_PROPS='-Diterations=1024 -Dwarmup=1 -Drounds=3'
  ./scripts/bench_native_compression_threshold_shape.sh` passed equivalence
  for default and tight threshold sets. Native shape-model speedups were
  `6.236x` and `5.301x`.
- `bash -n` passed for all three new scripts, `sha256sum -c
  reports/paper-native-jni.sha256` passed, and `git diff --check` passed. The
  release JNI library hash is
  `c30c240a031747828c75d112d3ba0902186b568a3bba7a7d2671d5b0ae60b4e2`.

Decision for now: keep all three modules diagnostic-only. They expand the
Rust rewrite surface and prove Java/native parity for focused compression/IO
models, but they do not install Paper runtime hooks and do not make strict
gate or TPS claims.

## Current 2026-05-13 22:38 CEST: Rust ObfHelper maps native diagnostic batch is parity-clean

This continuation added a new modular Rust/native diagnostic checkpoint for
the `ObfHelper` mapping bootstrap while keeping the Paper runtime unchanged:

- Added `paper-native-core::obfhelper_maps`, a pure Rust model for the
  mapping-bootstrap paths that build class/method/field maps plus the backing
  `StringPool` for old stream/default, direct-map, and presized-string-pool
  shapes.
- Added JNI exports and a compact Java/native parity bench pair under
  `bench/native-obfhelper-maps/`.
- Added `scripts/bench_native_obfhelper_maps.sh` to compile the new bench
  against the real `reobf.tiny` mapping jar and the native release library.
- `cargo check --manifest-path native/Cargo.toml --workspace` passed.
- `./scripts/bench_native_obfhelper_maps.sh` passed equivalence on the real
  Paper `1.21.10` mapping jar with `7554` classes, `47786` methods, and
  `31113` fields. Native is slower than Java on this host because the new
  bench moves a large string fixture through JNI, but the class/entry
  checksums match exactly and the module is diagnostic-only.
- `bash -n scripts/bench_native_obfhelper_maps.sh`, `sha256sum -c
  reports/paper-native-jni.sha256`, and the bench-run verification all
  passed. The release JNI library hash is
  `745f0ba70d9543cd6216ff681967aaa884642cbdaae33ae985dd9f59d777c555`.

Decision for now: keep this module diagnostic-only. It is useful mapping
bootstrap evidence, not a Paper runtime hook or strict-gate claim.

## Current 2026-05-13 21:33 CEST: Rust VarInt/VarLong and plugin-startup rollup native batch is parity-clean

This continuation added two more modular Rust/native diagnostic modules while
keeping the Paper runtime unchanged:

- Extended `paper-native-core::varint` with batch VarInt/VarLong write and
  read helpers, then wired them through JNI so the native bench now covers
  VarInt/VarLong size, write-batch, and read-batch parity.
- Added `paper-native-core::plugin_startup_rollup`, a pure Rust diagnostic
  rollup that combines plugin-name join plus plugin startup log aggregation
  for both normal and debug delimiters.
- Added JNI exports, Java/native parity benches, and an executable script:
  `scripts/bench_native_plugin_startup_rollup.sh`.
- `cargo check --manifest-path native/Cargo.toml --workspace` passed.
- `cargo test --manifest-path native/Cargo.toml --workspace` passed with
  `263` `paper-native-core` tests.
- `./scripts/bench_native_varint.sh` passed equivalence on `1000000` VarInt
  values and `1000000` VarLong values. Native is slower than Java on every
  measured JNI shape, but the encoded bytes, decoded values, and byte-size
  calculations all match.
- `./scripts/bench_native_plugin_startup_rollup.sh` passed equivalence on
  `5000` iterations for normal and debug delimiters. Native is slower than
  Java in absolute JNI timing, but the optimized same-runtime Java rollup is
  still the useful signal (`3.065x` normal, `3.137x` debug in Java;
  `1.937x` normal, `1.948x` debug in native).
- `bash -n` for the new script, `sha256sum -c
  reports/paper-native-jni.sha256`, and `git diff --check` passed. The release
  JNI library hash is
  `6b4ef0e20a2c9a17059a365138b1556b19f4f0665d4461e8a323f04a639e1d70`.

Decision for now: keep both modules diagnostic-only. VarInt/VarLong parity is
clean but not a JNI win on this host, and the plugin-startup rollup is still
bench evidence rather than a runtime hook.

## Current 2026-05-13 16:58 CEST: Rust waypoint chunk update and remapper hash-threshold native batch is parity-clean

This continuation added two more modular Rust/native diagnostic modules while
keeping the Paper runtime unchanged:

- Added `paper-native-core::waypoint_chunk_update`, a pure Rust parity model
  for distance-based vs chunk-long-key waypoint chunk-change checks.
- Moved the waypoint chunk-update fixture storage to heap-backed `Vec`s after
  `./scripts/build_native.sh` exposed a stack overflow in the first large
  array fixture shape.
- Added `paper-native-core::remapper_hash_threshold`, a pure Rust parity
  model for plugin-remapper hash-cache build shapes: computeIfAbsent, put,
  hybrid, and parallel. The parallel native shape uses scoped Rust threads.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_waypoint_chunk_update.sh` and
  `scripts/bench_native_remapper_hash_threshold.sh`.
- `cargo check --manifest-path native/Cargo.toml --workspace` passed.
- `./scripts/build_native.sh` passed with `245` `paper-native-core` tests and
  release native library hash
  `6c09aeedf3a9fb96166a93d8068bf0ff4b1bc0df854519c0b1bbbe6e1c3d8fc9`.
- `WAYPOINT_CHUNK_ITERATIONS=4000000 WAYPOINT_CHUNK_WARMUP=2
  WAYPOINT_CHUNK_ROUNDS=4 ./scripts/bench_native_waypoint_chunk_update.sh`
  passed equivalence. Java long-key checking is faster than Java distance
  checking (`2.587x`), but the JNI native model is slower than Java in this
  tiny hot loop: distance `0.266x`, long-key `0.197x`.
- `HASH_BENCH_ITERATIONS=3 HASH_BENCH_ROUNDS=2 HASH_BENCH_WARMUP=1
  ./scripts/bench_native_remapper_hash_threshold.sh` passed equivalence over
  `13` real plugin/library jars and subset sizes `1`, `2`, `4`, `8`, and
  `12`. At size `12`, Java/native count, total-entry, checksum, and
  last-digest fields match for all four modes. Native is slower than Java at
  that size (`0.646x`, `0.683x`, `0.602x`, `0.650x`), while native parallel
  is faster than native put (`2.579x`).
- `bash -n` for the two new scripts, `sha256sum -c
  reports/paper-native-jni.sha256`, and `git diff --check` passed.

Decision for now: keep both modules diagnostic-only. The waypoint long-key
shape is a same-runtime Java signal, but the native JNI path is not a win.
The remapper hash-threshold model is parity-clean on real jars, but it is not
a Paper runtime hook or strict-gate claim.

## Current 2026-05-13 16:08 CEST: Rust waypoint snapshot, table view, and manager-skip native batch is parity-clean

This continuation expanded the modular Rust/native layer with three more
diagnostic waypoint modules while keeping the Paper runtime unchanged:

- Added `paper-native-core::waypoint_snapshot`, a pure Rust parity model for
  `HashBasedTable` + `Tables.transpose(table)` snapshotting over the
  toArray, sized-array, and manual copy shapes.
- Added `paper-native-core::waypoint_table_view`, a pure Rust parity model
  for the transposed-row vs column scan shapes over the waypoint connection
  table.
- Added `paper-native-core::waypoint_manager_skip`, a pure Rust parity model
  for the current/skip player and current/skip waypoint full/partial shapes.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_waypoint_snapshot.sh`,
  `scripts/bench_native_waypoint_table_view.sh`, and
  `scripts/bench_native_waypoint_manager_skip.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with `242` `paper-native-core` tests and
  release native library hash
  `d3ddef2d4224c4f35fd40a55640555f020190fc88251fcdb8bc9a130a95dc2aa`.
- `./scripts/bench_native_waypoint_snapshot.sh` passed equivalence on
  `50_000` iterations. Native matches all three Java shapes and is faster on
  this host: toArray `18362.610x`, sized-array `28326.422x`, and manual
  `12246.901x`.
- `./scripts/bench_native_waypoint_table_view.sh` passed equivalence on
  `200_000` iterations. Native matches the transposed-row and column scans
  and is faster on this host: `14612.526x` and `17070.012x`.
- `./scripts/bench_native_waypoint_manager_skip.sh` passed equivalence on
  `1_000_000` iterations. Native matches all eight current/skip shapes and is
  faster on this host: `3872.955x`, `2162.004x`, `3930.484x`, `2412.447x`,
  `2649.895x`, `2225.330x`, `4522.427x`, and `4337.273x`.
- `bash -n` for the three new scripts, `sha256sum -c
  reports/paper-native-jni.sha256`, and `git diff --check` passed.

Decision for now: keep these modules diagnostic-only. They are Java/native
parity evidence for focused waypoint algorithms, not Paper runtime hooks and
not strict-gate claims.

## Current 2026-05-13 15:15 CEST: Rust worldgen and ticketset diagnostic batch is parity-clean

This continuation expanded the modular Rust/native layer with six more
diagnostic modules while keeping the Paper runtime unchanged:

- Added/verified `paper-native-core::improved_noise_floor`, covering the
  current `Mth.floor(...)` path and a `Math.floor(...)` shape inside
  `ImprovedNoise`.
- Added/verified `paper-native-core::surface_rules_sequence_array`, covering
  list-enhanced, list-indexed, array-foreach, and array-indexed sequence-rule
  traversal shapes.
- Added/verified `paper-native-core::surface_rules_test_rule_state`, covering
  old and new state-rule object shapes over period-7 and period-2 hit tests.
- Added `paper-native-core::placed_feature_traversal`, a pure Rust recursive
  placement traversal model with Java-compatible `Random` ordering and FNV
  result hashing.
- Added `paper-native-core::ore_feature_loop`, a pure Rust model for the
  old and optimized ore-blob inner loops over Java-provided blob arrays.
- Added `paper-native-core::ticketset_search`, a pure Rust model for the
  `TicketSetSearchBench` binary, unchecked-binary, and linear-threshold
  search shapes.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_improved_noise_floor.sh`,
  `scripts/bench_native_surface_rules_sequence_array.sh`,
  `scripts/bench_native_surface_rules_test_rule_state.sh`,
  `scripts/bench_native_placed_feature_traversal.sh`,
  `scripts/bench_native_ore_feature_loop.sh`, and
  `scripts/bench_native_ticketset_search.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with `235` `paper-native-core` tests and
  release native library hash
  `6538828a942f7d1183a4cfed03d6a7dd12c85cb2af1d8aaa1e3100003a78dc1f`.
- `./scripts/bench_native_improved_noise_floor.sh` passed equivalence on
  `1000000` iterations. Native is slower than Java on both measured shapes:
  current Mth floor `0.588x`, Math floor `0.701x`.
- `./scripts/bench_native_surface_rules_sequence_array.sh` passed equivalence
  on `20000000` iterations and `14` rules. Native beats Java on all four
  measured shapes: `2.456x`, `6.337x`, `1.938x`, and `3.567x`.
- `./scripts/bench_native_surface_rules_test_rule_state.sh` passed
  equivalence on `20000000` iterations. Native beats Java for period-7 old
  and new (`1.445x`, `1.321x`) and period-2 old/new (`1.593x`, `1.410x`);
  the period-2 native new-vs-old result is neutral/slightly slower (`0.990x`).
- `./scripts/bench_native_placed_feature_traversal.sh` passed equivalence on
  `200000` traversals. Native recursive traversal was `21.813x` faster than
  Java stream and `29.905x` faster than Java recursive on this host.
- `./scripts/bench_native_ore_feature_loop.sh` passed equivalence for
  `65536` blobs. Native old and optimized shapes beat Java (`1.593x` and
  `1.491x`), while the optimized native shape is slightly slower than native
  old (`0.946x`).
- `./scripts/bench_native_ticketset_search.sh` passed equivalence on
  `6000000` operations. Native beats Java on binary, unchecked-binary,
  linear4, linear8, and linear12 shapes (`3.220x`, `3.209x`, `3.608x`,
  `3.174x`, `3.498x`).
- `bash -n` for the three new scripts, `sha256sum -c
  reports/paper-native-jni.sha256`, and `git diff --check` passed.

Decision for now: keep these modules diagnostic-only. They are Java/native
parity evidence for focused algorithms, not Paper runtime hooks and not
strict server-gate claims.

## Current 2026-05-13 13:45 CEST: Rust protochunk-heightmap and range-choice diagnostics are parity-clean

This continuation expanded the modular Rust/native pass with two more
diagnostic modules while keeping the Paper runtime unchanged:

- Added `paper-native-core::protochunk_heightmap`, a pure Rust model for
  `ProtoChunk` heightmap set/update loops: old EnumSet foreach vs cached
  heightmap-type values plus `contains(...)` checks.
- Added `paper-native-core::range_choice`, a pure Rust model for
  `RangeChoice` old `fillArray(...)` and optimized constant-in,
  constant-out, both-constant, and both-dynamic shapes.
- Corrected the `both_constant` range-choice model so the optimized path has
  zero `forIndex(...)` calls, matching the Java `RangeChoiceConstantOut`
  in-range-constant branch.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_protochunk_heightmap.sh` and
  `scripts/bench_native_range_choice.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with `223` `paper-native-core` tests and
  release native library hash
  `59605ce068888933fd5d30313fd767cfcdfb9ea1729752182cae55082d3dc3a7`.
- `./scripts/bench_native_protochunk_heightmap.sh` passed equivalence on
  `8000000` iterations. Old EnumSet foreach native is faster than Java
  (`7.615x`), new cached-contains native is faster than Java (`1.344x`),
  and Java cached-contains is faster than Java old (`1.208x`). Native
  cached-contains is slower than native old (`0.213x`), so this stays
  diagnostic-only.
- `./scripts/bench_native_range_choice.sh` passed equivalence on `1000000`
  samples across four scenarios. Old native vs Java speedups were
  `1.056x`, `0.966x`, `1.009x`, and `1.061x`; optimized native vs Java was
  slower on all four scenarios (`0.777x`, `0.883x`, `0.808x`, `0.963x`).
  Java optimized-vs-old still shows the useful same-runtime signal:
  `1.107x`, `1.059x`, `1.239x`, and `1.034x`.
- Range-choice Java/native summaries match exactly, including
  `forIndex(...)` counts: old is `1000000` for every scenario, optimized is
  `400495`, `599505`, `0`, and `1000000`.
- `bash -n scripts/bench_native_range_choice.sh scripts/bench_native_protochunk_heightmap.sh`,
  `sha256sum -c reports/paper-native-jni.sha256`, and `git diff --check`
  passed.

Decision for now: keep both modules diagnostic-only. They are Java/native
parity evidence for protochunk heightmap loops and range-choice fill-array
specialization, not Paper runtime hooks and not strict-gate claims.

## Current 2026-05-13 13:10 CEST: Rust climate-parameter-distance and noise-generator-settings diagnostics are parity-clean

This continuation expanded the modular Rust/native pass with two more
diagnostic modules while keeping the Paper runtime unchanged:

- Added `paper-native-core::climate_parameter_distance`, a pure Rust model
  for old, branch, and subtract-first climate parameter-distance scoring
  across 7-parameter node/query batches.
- Added `paper-native-core::noise_generator_settings`, a pure Rust model for
  holder-value, memoized-supplier, lazy-primitive, manual-lazy-object, and
  cached-int access shapes.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_climate_parameter_distance.sh` and
  `scripts/bench_native_noise_generator_settings.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 218 `paper-native-core` tests and
  release native library hash
  `2603ee607be92705ef1435de6ee2e5d499df6bcfdcbfb12e6ca8c7ef1815d383`.
- `./scripts/bench_native_climate_parameter_distance.sh` passed equivalence
  on `1024` nodes and `8192` queries. Native is faster than Java on all
  three shapes: old `3.124x`, branch `5.274x`, and subtract-first `3.072x`.
  The Java branch shape is slower than old (`0.784x`), while the Java
  subtract-first shape is slightly faster (`1.050x`).
- `./scripts/bench_native_noise_generator_settings.sh` passed equivalence on
  `1024` generators and `20000000` iterations. Native is faster than Java on
  all five shapes: holder `3.113x`, memoized `6.056x`, lazy primitive
  `2.543x`, manual lazy object `3.514x`, and cached `1.306x`. The cached
  Java path remains the strongest same-runtime improvement (`2.382x` over
  old), so this stays diagnostic-only.

Decision for now: keep both modules diagnostic-only. They are Java/native
parity evidence for climate and noise-generator settings, not Paper runtime
hooks and not strict-gate claims.

## Current 2026-05-13 13:08 CEST: Rust chunk-expire-count, craftplayer-cansee, and levelchunk-heightmap diagnostics are parity-clean

This continuation expanded the modular Rust/native pass with three more
diagnostic modules while keeping the Paper runtime unchanged:

- Added `paper-native-core::chunk_expire_count`, a pure Rust model for
  dynamic and cached expire-count paths.
- Added `paper-native-core::craftplayer_cansee`, a pure Rust model for
  current, guarded, candidate, and chunk-map candidate can-see paths.
- Added `paper-native-core::levelchunk_heightmap`, a pure Rust model for old
  four-update and new combined-update heightmap update shapes.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_chunk_expire_count.sh`,
  `scripts/bench_native_craftplayer_cansee.sh`, and
  `scripts/bench_native_levelchunk_heightmap.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 218 `paper-native-core` tests and
  release native library hash
  `2603ee607be92705ef1435de6ee2e5d499df6bcfdcbfb12e6ca8c7ef1815d383`.
- `./scripts/bench_native_chunk_expire_count.sh` passed equivalence. Native
  is slower on every measured shape: dynamic hot `0.491x` / `0.421x` /
  `0.538x` / `0.356x` / `0.307x` and cold `0.628x` / `0.282x` / `0.389x` /
  `0.367x` / `0.386x` across the reported variants.
- `./scripts/bench_native_craftplayer_cansee.sh` passed equivalence. Native
  is faster on all measured shapes: `61.448x`, `40.344x`, `33.700x`,
  `10.747x`, `13.782x`, `16.757x`, `107.348x`, and `38.142x`.
- `./scripts/bench_native_levelchunk_heightmap.sh` passed equivalence. Native
  beats Java on the old four-update shape (`1.484x`) but loses on the new
  combined-update shape (`0.920x`), and the combined Java-vs-old speedup is
  `0.646x`.

Decision for now: keep all three modules diagnostic-only. They are Java/native
parity evidence for chunk-expire counting, can-see gating, and heightmap
updates, not Paper runtime hooks and not strict-gate claims.

## Current 2026-05-13 09:15 CEST: Rust nearby-player-map capacity diagnostics are parity-clean

This continuation expanded the modular Rust/native pass with one more
capacity-focused diagnostic while keeping the Paper runtime unchanged:

- Added `paper-native-core::nearby_player_map_capacity`, a pure Rust model
  for fastutil `Reference2ReferenceOpenHashMap` default vs presized
  allocation/rehash behavior in the nearby-player-map capacity bench.
- Added JNI exports, a Java/native parity bench, and
  `scripts/bench_native_nearby_player_map.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed and produced release native hash
  `0d2606fe5dc02d1b76c83816faf8b8ddd0adb6b3b0f11549c47f034c4a953d16`.
- `./scripts/bench_native_nearby_player_map.sh` passed equivalence for both
  `players=50` / `iterations=80000` and `players=500` / `iterations=12000`.
  Native is faster than Java on both scenarios. For 50 players the default
  path measured `69.919x` and the presized path `39.047x`; for 500 players
  the default path measured `87.489x` and the presized path `41.880x`.
  The Java same-runtime speedups still show the presized win clearly:
  `2.138x` for 50 players and `2.543x` for 500 players. The module stays
  diagnostic-only.

Decision for now: keep this module diagnostic-only. It is Java/native parity
evidence for nearby-player-map capacity behavior, not a Paper runtime hook or
strict server-gate claim.

## Current 2026-05-13 09:40 CEST: Rust marker-cache and waypoint-distance guard diagnostics are parity-clean

This continuation expanded the modular Rust/native pass with two more
diagnostic modules while keeping the Paper runtime unchanged:

- Completed `paper-native-core::marker_cache` with JNI exports, a standalone
  Java/native parity bench, and `scripts/bench_native_marker_cache.sh`.
- Completed `paper-native-core::waypoint_distance_guard` with JNI exports, a
  standalone Java/native parity bench, and
  `scripts/bench_native_waypoint_distance_guard.sh`.
- Fixed the waypoint checksum model so old and guarded summaries can be
  compared directly; the digest no longer includes the mode discriminator.
- Hardened the two active native bench scripts so they rebuild the release
  native library when Rust/JNI sources are newer than the existing `.so`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 205 `paper-native-core` tests and
  release native library hash
  `b96ac2a4e7067c2453fc9ecb23f4ee12582c6f903ab394474e17225caa3efcdc`.
- `./scripts/bench_native_marker_cache.sh` passed equivalence on
  `roots=256`, `depth=24`, `iterations=300`. Native beats Java on the old
  non-cached shape (`1.311x`) but loses on the cached-marker shape (`0.364x`);
  the same-runtime cached shape is also slower in this native summary bench
  (`0.956x` Java, `0.265x` native).
- `./scripts/bench_native_waypoint_distance_guard.sh` passed equivalence on
  `65536` entries and `8000000` iterations. Native is slower on old range
  (`0.827x`), guarded range (`0.873x`), and old really-far (`0.905x`), and
  only slightly faster on guarded really-far (`1.018x`). Same-runtime guarded
  range and really-far are both slower (`0.907x`).

Decision for now: keep both modules diagnostic-only. They are Java/native
parity evidence, not Paper runtime hooks and not strict server-gate evidence.

## Current 2026-05-13 09:01 CEST: Rust remapper and plugin-directory diagnostics are parity-clean

This continuation expanded the modular Rust/native pass with three related
startup/remapper diagnostics while keeping the Paper runtime unchanged:

- Added `paper-native-core::remapper_index_cleanup`, a pure Rust model for
  eager remapper-index cleanup work vs the lazy count-check path.
- Added `paper-native-core::remapper_skip_hashes`, a pure Rust model for
  stream-style skip-hash parsing vs direct loop parsing.
- Added `paper-native-core::plugin_directory_scan`, a pure Rust/filesystem
  model for plugin-directory `walk(depth=1)`, `list`, and
  `DirectoryStream`-style scans.
- Added JNI exports, Java/native parity benches, and executable scripts:
  `scripts/bench_native_remapper_index_cleanup.sh`,
  `scripts/bench_native_remapper_skip_hashes.sh`, and
  `scripts/bench_native_plugin_directory_scan.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 202 `paper-native-core` tests and
  release native library hash
  `e7d8db4f384fb9596bdbd97b40cceffbb26db1b4ac59dc3ea715f75fe7be7c3b`.
- `./scripts/bench_native_remapper_index_cleanup.sh` passed equivalence on
  `12` inputs / `4` remapped / `250000` iterations. Native is slower than
  Java in absolute terms (`0.232x` old eager cleanup, `0.198x` new lazy
  cleanup), but both runtimes still show the lazy path reducing modeled work:
  Java `1.756x`, native `1.493x`.
- `./scripts/bench_native_remapper_skip_hashes.sh` passed equivalence on
  `300000` iterations. Native is faster than Java for this string parser
  model (`2.314x` old stream, `2.840x` new loop), while the same-runtime Java
  loop path was neutral/slightly slower (`0.979x`).
- `./scripts/bench_native_plugin_directory_scan.sh` passed equivalence against
  `/root/rust/plugins/matrix` with `12` plugins per scan. Native was faster
  than Java on the three absolute scan shapes (`2.267x` walk, `1.296x` list,
  `1.190x` directory stream), while the Java same-runtime result still
  favors `Files.list` over `Files.walk(depth=1)` (`1.624x`) and
  `DirectoryStream` over `Files.list` only slightly (`1.050x`).

Decision for now: keep all three modules diagnostic-only. They are
Java/native parity evidence for startup/remapper work shapes, not Paper
runtime hooks and not strict server-gate evidence.

## Current 2026-05-13 07:53 CEST: Rust Spigot load-order and topographic-sort diagnostics are parity-clean

This continuation widened the modular Rust/native pass and kept the Paper
runtime unchanged:

- Added `paper-native-core::spigot_load_order_dependency`, a pure Rust parity
  model for Spigot load-order dependency work: default vs pre-sized
  `loadAfter` construction and old temporary-HashSet back-reference checking
  vs direct hard/soft dependency-list checks.
- Added `paper-native-core::topographic_graph_sort_capacity`, a pure Rust
  parity model for `TopographicGraphSorter` default-capacity containers vs
  pre-sized sorted/root/non-root containers.
- Added JNI exports, Java/native parity benches, and executable bench scripts:
  `scripts/bench_native_spigot_load_order_dependency.sh` and
  `scripts/bench_native_topographic_graph_sort_capacity.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 196 `paper-native-core` tests and
  release native library hash
  `f6fc23b0617b06cb66174ef671415b1d6b1186eaa6be854d107f943b10c2338b`.
- `./scripts/bench_native_spigot_load_order_dependency.sh` passed
  equivalence. Native is a poor candidate for the `loadAfter` list-copy
  forms (`0.112x` old, `0.116x` new vs Java), but it is faster than Java on
  the direct removed-count shape (`139.952 ms` native vs `327.579 ms` Java,
  `2.341x`). The useful same-runtime Java signal remains new direct removed
  count over old temporary `HashSet` (`8.236x`), while the native model shows
  `79.281x` old-to-new inside native.
- `./scripts/bench_native_topographic_graph_sort_capacity.sh` passed
  equivalence. Native is slower than Java on both absolute shapes
  (`0.700x` old, `0.514x` new vs Java), but both runtimes still show
  pre-sizing improvement: Java `1.685x`, native `1.236x`.

Decision for now: keep both modules diagnostic-only. They prove Rust parity
for the two load-order allocation models, but no Paper runtime hook or strict
server-gate proof exists for either module.

## Current 2026-05-13 07:19 CEST: Rust plugin-loading allocation and alias-removal diagnostics are parity-clean

This continuation added two modular Rust/native checkpoints and kept the
Paper runtime unchanged:

- Added `paper-native-core::plugin_loading_allocation`, a pure Rust parity
  model for plugin-loading startup allocation shapes: default vs pre-sized
  setup maps/lists, eager vs lazy missing hard-dependency set allocation, and
  eager vs lazy validate-no-miss list allocation.
- Added `paper-native-core::legacy_provided_alias_removal`, a pure Rust
  parity model for legacy `pluginsProvided.values().removeIf(...)` cleanup
  vs the reverse provided-alias index cleanup.
- Added JNI exports, Java/native parity benches, and executable bench scripts:
  `scripts/bench_native_plugin_loading_allocation.sh` and
  `scripts/bench_native_legacy_provided_alias_removal.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 188 `paper-native-core` tests and
  release native library hash
  `29757d73a12cde363fdae44df09c6a90edbfa2212046ca615dfb5ebbad134407`.
- `./scripts/bench_native_plugin_loading_allocation.sh` passed equivalence.
  Native was slower than Java on every absolute Java/native shape on this
  host, but the same native model still shows the intended old/new allocation
  reductions: setup `2.780x`, missing-set scan `1.173x`, validate-no-miss
  `0.980x`. The same Java bench measured setup `1.628x`, missing `1.166x`,
  and validate `0.982x`.
- `./scripts/bench_native_legacy_provided_alias_removal.sh` passed
  equivalence. Native beat the old Java `values().removeIf(...)` path
  (`392.070 ms` Java vs `184.060 ms` native, `2.130x`), but the already
  optimized Java reverse-index path remains faster than native
  (`32.777 ms` Java vs `77.657 ms` native, `0.422x`). The useful
  same-runtime Java signal is reverse-index over old removeIf (`11.962x`).

Decision for now: keep both modules diagnostic-only. They prove Rust parity
for plugin-loading allocation and legacy alias-removal models, but no Paper
runtime hook or strict server-gate proof exists for either module.

## Current 2026-05-12 20:49 CEST: Rust plugin classloader-group diagnostic is parity-clean

This continuation added the next modular Rust/native checkpoint and kept the
Paper runtime unchanged:

- Added `paper-native-core::plugin_classloader_group`, a pure Rust parity
  model for plugin classloader-group lookup across miss, hit-other, and
  hit-requester paths.
- Covered the old lookup shape and the requester-skip shape that avoids
  retrying the requester inside the loader list after the initial requester
  attempt.
- Added JNI exports, a Java/native parity bench, and
  `scripts/bench_native_plugin_classloader_group.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 180 `paper-native-core` tests and
  release native library hash
  `11dc835730fb924302d621ce89f664c11228addb6ed2ddb1df5a27cd2a4ec001`.
- `./scripts/bench_native_plugin_classloader_group.sh` passed equivalence.
  Native was faster on five of six measured shapes on this host: miss old
  `1.805 ms` Java vs `0.485 ms` native (`3.723x`), miss skip `1.532 ms`
  vs `1.100 ms` (`1.393x`), hit-other old `0.525 ms` vs `0.217 ms`
  (`2.418x`), hit-other skip `0.359 ms` vs `0.391 ms` (`0.918x`),
  hit-requester old `0.181 ms` vs `0.098 ms` (`1.839x`), and
  hit-requester skip `0.218 ms` vs `0.166 ms` (`1.314x`).
- The Java requester-skip same-runtime signal is small/noisy on this synthetic
  bench: miss skip was `1.178x` vs old, hit-other skip was `1.465x` vs old.

Decision for now: keep this module diagnostic-only. It proves Rust parity and
shows native batch wins on this synthetic lookup model, but no Paper runtime
hook or strict server-gate proof exists for classloader lookup yet.

## Current 2026-05-12 20:19 CEST: Rust plugin metadata dependency diagnostic is parity-clean

This continuation added the next modular Rust/native checkpoint and kept the
Paper runtime unchanged:

- Added `paper-native-core::plugin_meta_dependency`, a pure Rust parity model
  for Paper plugin metadata dependency-list extraction across required
  join-classpath, soft join-classpath, load-before, and load-after lists.
- Covered the old stream shape, the direct loop shape, and cached repeated
  access to the four immutable lists.
- Added JNI exports, a Java/native parity bench, and
  `scripts/bench_native_plugin_meta_dependency.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 176 `paper-native-core` tests and
  release native library hash
  `a60ac6075a69d78f701835867a7bef2c6fbfaad57a46ff0ad55662d37e15ad20`.
- `./scripts/bench_native_plugin_meta_dependency.sh` passed equivalence.
  Native beat the old Java stream path (`2531.886 ms` Java vs `978.124 ms`
  native, `2.589x`) but lost to the already optimized Java loop
  (`807.677 ms` Java vs `961.744 ms` native, `0.840x`) and cached repeated
  access (`142.618 ms` Java vs `705.693 ms` native, `0.202x`). The useful
  same-runtime Java signals remain the loop over stream (`3.135x`) and cached
  over loop (`5.663x`).

Decision for now: keep this module diagnostic-only. It proves parity for the
Rust metadata dependency-list model, but the existing Java loop/cache path is
still the better runtime shape and there is no new strict-gate evidence for a
Paper native hook.

## Current 2026-05-12 19:42 CEST: Rust plugin startup string diagnostics are parity-clean

This continuation added two more modular Rust/native checkpoints and kept the
Paper runtime unchanged:

- Added `paper-native-core::plugin_name_join`, a pure Rust parity model for
  plugin name `String.join(...)` vs manual `StringBuilder` joining over the
  normal `", "` and debug `"\n - "` delimiters.
- Added `paper-native-core::plugin_name_log`, a pure Rust parity model for
  plugin log-name aggregation over the old `TreeSet` shape and the newer
  `ArrayList` sort/deduplicate shape.
- Added JNI exports, Java/native parity benches, and executable bench scripts
  for both modules.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 172 `paper-native-core` tests and
  release native library hash
  `3f2329fab6581aeba4ecb05ce7a034c9189dda751f31b56133167763f5e37af4`.
- `./scripts/bench_native_plugin_name_join.sh` passed equivalence. Native was
  slower than Java on this host: string normal `43.139 ms` Java vs
  `81.247 ms` native (`0.531x`), manual normal `75.108 ms` Java vs
  `153.670 ms` native (`0.489x`), string debug `47.399 ms` Java vs
  `133.320 ms` native (`0.356x`), manual debug `81.115 ms` Java vs
  `112.218 ms` native (`0.723x`).
- `./scripts/bench_native_plugin_name_log.sh` passed equivalence. Native was
  also slower than Java on this host: old TreeSet `376.078 ms` Java vs
  `415.921 ms` native (`0.904x`), new ArrayList sort `74.726 ms` Java vs
  `217.677 ms` native (`0.343x`). The same-runtime Java rewrite remains the
  important signal here: ArrayList sort is `5.033x` faster than TreeSet.

Decision for now: keep both modules diagnostic-only. They prove parity for
the Rust string/list models, but JNI string conversion and allocation make
them poor runtime-hook candidates.

## Current 2026-05-12 18:22 CEST: Rust shift-noise-direct native diagnostic is parity-clean

This continuation added another modular Rust/native checkpoint and kept the
Paper runtime unchanged:

- Added `paper-native-core::shift_noise_direct`, a pure Rust parity model
  for the `ShiftNoiseDirectBench` helper/directed-shift shapes over the
  current, direct, current-A, direct-A, current-B, and direct-B paths.
- Added JNI exports, a Java/native parity bench, and
  `scripts/bench_native_shift_noise_direct.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 166 `paper-native-core` tests and
  release native library hash
  `eb343a5c3f9d712586fdb7c13319374102c8c2587d2eb1cee5619f3d1f929211`.
- `./scripts/bench_native_shift_noise_direct.sh` passed equivalence and
  measured current default `8.624 ms` Java vs `8.006 ms` native (`1.077x`),
  direct default `8.898 ms` Java vs `8.505 ms` native (`1.046x`), current A
  `8.660 ms` Java vs `7.627 ms` native (`1.135x`), direct A `8.833 ms`
  Java vs `7.677 ms` native (`1.151x`), current B `8.537 ms` Java vs
  `8.056 ms` native (`1.060x`), and direct B `11.097 ms` Java vs
  `8.069 ms` native (`1.375x`).

Decision for now: keep this diagnostic-only. It is useful parity evidence for
the shift helper/direct math, but it does not justify a Paper runtime hook.

## Current 2026-05-12 18:55 CEST: Rust entity-bounding-box native diagnostic is parity-clean

This continuation completed another modular Rust/native checkpoint and kept
the Paper runtime unchanged:

- Added `paper-native-core::entity_bounding_box`, a pure Rust parity model
  for the `EntityDimensions.makeBoundingBox(...)` then `setBoundingBox(...)`
  path and the direct dimensions-based `setBoundingBox(...)` shape.
- Added JNI exports, a Java/native parity bench, and
  `scripts/bench_native_entity_bounding_box.sh`.
- Fixed the native summary contract after the first parity run caught an
  `old`-path `last_bits` mismatch; the Rust model now leaves `last_bits` on
  the final boxed `maxZ`, matching the Java bench.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 162 `paper-native-core` tests and
  release native library hash
  `0f4f659fb93d84188839d5093ede4df6299d365b97551754940c3b08fb9453b6`.
- `./scripts/bench_native_entity_bounding_box.sh` passed equivalence and
  measured old make-then-set `1894.853 ms` Java vs `395.719 ms` native
  (`4.788x`), direct dimensions-set `813.001 ms` Java vs `406.124 ms`
  native (`2.002x`), Java direct vs old `2.331x`, and native direct vs old
  `0.974x`.
- The Java bench allocated `1536000000` bytes on the old make-then-set path
  and `768000000` bytes on the direct dimensions path; the native summaries
  reported `0` allocated bytes for both shapes.

Decision for now: keep this diagnostic-only. The earlier
`Entity.setPosRaw(...)` bounding-box shortcut was rejected and rolled back by
the runtime gate, so this native parity evidence does not promote the shortcut
or install a Paper hook.

## Current 2026-05-12 18:10 CEST: Rust entity-lookup-status native diagnostic is parity-clean

This continuation added another modular Rust/native checkpoint and kept the
Paper runtime unchanged:

- Added `paper-native-core::entity_lookup_status`, a pure Rust parity model
  for `EntityLookup.getEntityStatus(...)` over the old
  `Visibility.fromFullChunkStatus(...)` path, the direct status mapping path,
  and the old/direct accessibility checks.
- Added JNI exports, a Java/native parity bench, and
  `scripts/bench_native_entity_lookup_status.sh`.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 159 `paper-native-core` tests and
  release native library hash
  `e833388f354c08171d196a0ce31e348c8b416fab83088bd17423096aa50432be`.
- `./scripts/bench_native_entity_lookup_status.sh` passed equivalence and
  measured old status `579.948 ms` Java vs `251.147 ms` native (`2.309x`),
  direct status `588.884 ms` Java vs `251.209 ms` native (`2.344x`),
  old accessible `721.905 ms` Java vs `258.580 ms` native (`2.792x`), and
  direct accessible `685.592 ms` Java vs `258.715 ms` native (`2.650x`).
- `sha256sum -c reports/paper-native-jni.sha256` and `git diff --check`
  passed after the new module and docs edits.

Decision for now: keep this diagnostic-only. Earlier EntityLookup runtime
experiments were rejected by the real load gate, so this native parity evidence
does not promote a Paper runtime hook.

## Current 2026-05-12 17:26 CEST: Rust chunk-dependencies and ownable-rule native diagnostics are parity-clean

This continuation added two more modular Rust/native checkpoints and kept the
native layer diagnostic-only:

- Added `paper-native-core::chunk_dependencies`, a pure Rust parity model for
  the `ChunkDependencies` radius/index lookup shape, with the old immutable
  list backing and the array snapshot backed by the same summary contract.
- Added `paper-native-core::ownable_rule`, a pure Rust parity model for the
  `OwnableRewriteRule.matchesOwner(...)` stream path vs direct loop path.
- Added JNI exports, Java/native parity benches, and executable bench scripts
  for both modules.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 155 `paper-native-core` tests and
  release native library hash
  `4343fb74925b96106c7c4025a4efc537e4b77988aa8f8f4b7353d79d264609a3`.
- `./scripts/bench_native_chunk_dependencies_array.sh` passed equivalence and
  measured old `791.860 ms` Java vs `477.905 ms` native (`1.657x`), array
  `794.043 ms` Java vs `482.147 ms` native (`1.647x`).
- `./scripts/bench_native_ownable_rule.sh` passed equivalence and measured old
  stream `1711.676 ms` Java vs `314.278 ms` native (`5.446x`), new loop
  `626.597 ms` Java vs `254.995 ms` native (`2.457x`).

Decision for now: keep both modules diagnostic-only. The native parity layer
is expanded, but no runtime hook promotion is made from these benches alone.

## Current 2026-05-12 16:50 CEST: Rust density-ap2 native diagnostics are parity-clean

This continuation added density AP2 native wrappers and kept the native layer
diagnostic-only:

- Added `paper-native-core::density_ap2_fill`, a pure Rust parity model for
  the old/new flat and old/new nested `DensityFunctions.Ap2.fillArray(...)`
  shapes.
- Added `paper-native-core::density_ap2_minmax_fill`, a pure Rust parity model
  for the old/new `MIN` and `MAX` fast-path shapes, with Java `Math.min` /
  `Math.max` signed-zero and `NaN` semantics covered by Rust tests.
- Added JNI exports, Java/native parity benches, and executable bench scripts
  for both density AP2 modules.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 147 `paper-native-core` tests and
  release native library hash
  `0c6dc16633265dec999545746bc54b2c47a68cc746fcbb075ffba8ec802f2bd7`.
- `./scripts/bench_native_density_ap2_fill.sh` passed equivalence and
  measured old flat `985.014 ms` Java vs `637.126 ms` native (`1.546x`),
  scratch flat `280.489 ms` Java vs `512.679 ms` native (`0.547x`), old
  nested `1884.341 ms` Java vs `1102.905 ms` native (`1.709x`), and scratch
  nested `1202.939 ms` Java vs `1025.159 ms` native (`1.173x`).
- `./scripts/bench_native_density_ap2_minmax_fill.sh` passed equivalence and
  measured the six scenarios across old/new paths; native was faster on
  `min_returns_second`, `max_returns_first`, `max_returns_second`,
  `min_overlap`, and `max_overlap`, while `min_returns_first` stayed slower.

Decision for now: keep both modules diagnostic-only. The native parity layer is
expanded, but no runtime hook promotion is made from these benches alone.

## Current 2026-05-12 16:12 CEST: Rust interpolator-array and flat-cache-context diagnostics are parity-clean

This continuation added `paper-native-core::noisechunk_interpolator_array`
and `paper-native-core::noisechunk_flatcache_context` and kept the native
layer diagnostic-only:

- Added `paper-native-core::noisechunk_interpolator_array`, a pure Rust parity
  model for the list / indexed-list / array interpolator loop shapes.
- Added `paper-native-core::noisechunk_flatcache_context`, a pure Rust parity
  model for the old/new false-context and old/new true-context shapes around
  the `NoiseChunk.FlatCache` allocation path.
- Added Rust tests, JNI exports, Java/native parity benches, and executable
  bench scripts for both modules.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 137 `paper-native-core` tests and
  release native library hash
  `977fe4a649cbf5a4eb14bb7889b48e3a3a48a9767a26211df1b3301567730a70`.
- `./scripts/bench_native_noisechunk_interpolator_array.sh` passed
  equivalence and measured list `1174.474 ms` Java vs `695.013 ms` native
  (`1.690x`), indexed list `1069.111 ms` Java vs `686.470 ms` native
  (`1.557x`), and array `1145.747 ms` Java vs `731.872 ms` native (`1.566x`).
- `./scripts/bench_native_noisechunk_flatcache_context.sh` passed equivalence
  and measured old false `108.479 ms` Java vs `137.700 ms` native (`0.788x`),
  new false `89.412 ms` Java vs `104.712 ms` native (`0.854x`), old true
  `1.038 ms` Java vs `1.074 ms` native (`0.967x`), and new true `1.006 ms`
  Java vs `1.083 ms` native (`0.929x`).

Decision for now: keep both modules diagnostic-only. `noisechunk_interpolator_array`
is parity-clean and fast enough to keep as evidence, while
`noisechunk_flatcache_context` does not restore the previously rejected
`NoiseChunk.FlatCache` runtime candidate.

## Current 2026-05-12 15:37 CEST: Rust slice and blend-cache diagnostics are parity-clean

This continuation added `paper-native-core::noise_interpolator_slice` and
`paper-native-core::noisechunk_blendcache` and kept the native layer
diagnostic-only:

- Added `paper-native-core::noise_interpolator_slice`, a pure Rust parity
  model for old jagged `double[][]` slices vs flat `double[]` slices.
- Added `paper-native-core::noisechunk_blendcache`, a pure Rust parity model
  for the empty-blender `FlatCache` allocation shape vs no-allocation shape.
- Added Rust tests, JNI exports, Java/native parity benches, and executable
  bench scripts for both modules.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 129 `paper-native-core` tests and
  release native library hash
  `d0ef661f50f80b33ed1dad953a735213ca201930c701b89211e80a3ebfa70c5f`.
- `./scripts/bench_native_noisechunk_blendcache.sh` passed equivalence and
  measured old empty blender `417.205 ms` Java vs `739.598 ms` native
  (`0.564x`) and new empty blender `10.404 ms` Java vs `5.234 ms` native
  (`1.988x`).
- `./scripts/bench_native_noise_interpolator_slice.sh` passed equivalence and
  measured old jagged `279.685 ms` Java vs `415.066 ms` native (`0.674x`)
  and flat `304.545 ms` Java vs `261.091 ms` native (`1.166x`).

Decision for now: keep both modules diagnostic-only. The Paper runtime is
unchanged, and `noisechunk_blendcache` does not restore the previously
rejected empty-blendcache runtime patch.

## Current 2026-05-12 15:21 CEST: Rust noise-interpolator fractions are parity-clean

This continuation added `paper-native-core::noise_interpolator_fractions`
and kept the native layer diagnostic-only:

- Added `paper-native-core::noise_interpolator_fractions`, a pure Rust parity
  model for the fixed `CaveCarver`-style fraction lookup workload over the
  division and precomputed-array shapes.
- Added Rust tests for regular, alternate-shape, empty, and repeated-run
  paths, `PaperNativeNoiseInterpolatorFractions.divisionSummary(...)` /
  `arraySummary(...)` JNI exports, and a Java/native parity bench.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 122 `paper-native-core` tests and
  release native library hash
  `54a3fe4da7460f8ebb194e0c424b9bcb216f5639d8ee6910a764de9c7db4704a`.
- `./scripts/bench_native_noise_interpolator_fractions.sh` passed
  equivalence and measured division `17.238 ms` Java vs `12.280 ms` native
  (`1.404x`) and array fraction `11.919 ms` Java vs `11.437 ms` native
  (`1.042x`).

Decision for now: keep `noise_interpolator_fractions` diagnostic-only. The
native model is parity-clean and faster on this host, but it still needs a
guarded Paper hook and strict-gate proof before it can become a runtime
change.

## Current 2026-05-12 15:00 CEST: Rust carver diagnostics are parity-clean

This continuation added `paper-native-core::carver_iteration` and
`paper-native-core::cave_carver_skip` and kept the native layer
diagnostic-only:

- Added `paper-native-core::carver_iteration`, a pure Rust parity model for
  the `CaveCarver` iteration shape over foreach vs indexed loops.
- Added `paper-native-core::cave_carver_skip`, a pure Rust parity model for
  the cave-floor skip checker shapes over old lambda, reused checker, and
  direct helper loops.
- Added Rust tests for regular, stable, empty, and repeated-run paths for
  both modules, `PaperNativeCarverIteration.*` /
  `PaperNativeCaveCarverSkip.*` JNI exports, and Java/native parity benches.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 118 `paper-native-core` tests and
  release native library hash
  `61d13f8c42fd79e2a038d1d6c83092e593db624cecae6a9533e5a64f91206cd7`.
- `./scripts/bench_native_carver_iteration.sh` passed equivalence and
  measured foreach `133.704 ms` Java vs `64.958 ms` native (`2.058x`) and
  indexed `89.380 ms` Java vs `76.765 ms` native (`1.164x`).
- `./scripts/bench_native_cave_carver_skip.sh` passed equivalence and
  measured old `61.044 ms` Java vs `83.470 ms` native (`0.731x`), reused
  checker `58.211 ms` Java vs `89.915 ms` native (`0.647x`), and direct
  helper `58.899 ms` Java vs `80.163 ms` native (`0.735x`).

Decision for now: keep both modules diagnostic-only. `carver_iteration` is
promising but still needs a guarded Paper hook and strict-gate proof, and
`cave_carver_skip` loses on absolute JNI overhead on this host.

## Current 2026-05-12 14:34 CEST: Rust ServerEntity delta diagnostics are parity-clean

This continuation added `paper-native-core::serverentity_delta_identity` and
kept the native layer diagnostic-only:

- Added `paper-native-core::serverentity_delta_identity`, a pure Rust parity
  model for the `ServerEntity.sendChanges()` delta-motion distance path and
  the existing identity-guard shape.
- Added Rust tests for regular, stable, empty, and tiny stop-delta paths,
  `PaperNativeServerEntityDeltaIdentity.oldDistanceSummary(...)` /
  `identityGuardSummary(...)` JNI exports, and
  `bench/native-serverentity-delta-identity` Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  serverentity_delta_identity -- --nocapture` passed with 4 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 110 `paper-native-core` tests and
  release native library hash
  `b90af35e0fc10c996d93dcbc70f5f8bb709ed01127217d49ff2e5b271718fabe`.
- `./scripts/bench_native_serverentity_delta_identity.sh` passed equivalence
  and measured old `193.459 ms` Java vs `151.916 ms` native (`1.273x`) and
  identity guard `110.046 ms` Java vs `116.559 ms` native (`0.944x`).

Decision for now: keep `serverentity_delta_identity` diagnostic-only. The
native model is useful parity evidence, but it does not replace the existing
Java identity-guard runtime path because the already-guarded Java shape is
faster than the JNI summary on this host.

## Current 2026-05-12 14:13 CEST: Rust StaticCache2D get diagnostics are parity-clean but slower

This continuation added `paper-native-core::static_cache_get` and kept the
native layer diagnostic-only:

- Added `paper-native-core::static_cache_get`, a pure Rust parity model for
  `StaticCache2D.get(...)` over the old `contains(...)` + `getIndex(...)`
  shape versus the single-offset lookup shape.
- Added Rust tests for regular, stable, empty, and out-of-range panic paths,
  `PaperNativeStaticCacheGet.oldBatchSummary(...)` /
  `newBatchSummary(...)` JNI exports, and `bench/native-static-cache-get`
  Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  static_cache_get -- --nocapture` passed with 4 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 106 `paper-native-core` tests and
  release native library hash
  `308b95ddd630b577f13804e636e04370fb9b82fc00f420b200bfc2ccf5fcaab2`.
- `./scripts/bench_native_static_cache_get.sh` passed equivalence and
  measured old `733.176 ms` Java vs `944.437 ms` native (`0.776x`) and new
  `693.851 ms` Java vs `864.624 ms` native (`0.802x`).

Decision for now: keep `static_cache_get` diagnostic-only. This does not
restore the previously rejected single-offset `StaticCache2D.get(...)`
runtime shape; the native path is slower on this host.

## Current 2026-05-12 13:56 CEST: Rust CubicSpline create diagnostics are parity-clean and faster

This continuation added `paper-native-core::cubic_spline_create` and kept the
native layer diagnostic-only:

- Added `paper-native-core::cubic_spline_create`, a pure Rust parity model for
  the `CubicSpline` create/min-max scan workload over iterator and index loop
  shapes.
- Added Rust tests for regular, stable, empty, and signed-zero float paths,
  `PaperNativeCubicSplineCreate.oldIteratorSummary(...)` /
  `indexSummary(...)` JNI exports, and `bench/native-cubic-spline-create`
  Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  cubic_spline_create -- --nocapture` passed with 4 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 102 `paper-native-core` tests and
  release native library hash
  `2b98c1b7e0f86dd3c00e289f04b2e0cb02fc2fdb5a1318027a175f51b955226d`.
- `./scripts/bench_native_cubic_spline_create.sh` passed equivalence and
  measured iterator `120.308 ms` Java vs `86.421 ms` native (`1.392x`) and
  index `114.063 ms` Java vs `80.360 ms` native (`1.419x`).

Decision for now: keep `cubic_spline_create` diagnostic-only. This does not
restore the previously rejected `CubicSpline.Multipoint.mapAll` runtime
allocation-cleanup patch; a guarded Paper hook still needs separate
strict-gate proof.

## Current 2026-05-12 13:38 CEST: Rust Jigsaw canAttach diagnostics are parity-clean and much faster

This continuation added `paper-native-core::jigsaw_canattach` and kept the
native layer diagnostic-only:

- Added `paper-native-core::jigsaw_canattach`, a pure Rust parity model for
  `JigsawBlock.canAttach(...)` over old orientation lookup, optimized direct
  orientation access, and target-first decision shapes.
- Added Rust tests for regular, stable, empty, and orientation-ordinal lookup
  batches, `PaperNativeJigsawCanAttach.oldBatchSummary(...)` /
  `optimizedBatchSummary(...)` / `targetFirstBatchSummary(...)` JNI exports,
  and `bench/native-jigsaw-canattach` Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  jigsaw_canattach -- --nocapture` passed with 4 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 98 `paper-native-core` tests and
  release native library hash
  `d9f6e2f53043aba95b690bb8119b257d99188537975c904106b44ff56cc1134b`.
- `./scripts/bench_native_jigsaw_canattach.sh` passed equivalence and
  measured old `1144.244 ms` Java vs `36.889 ms` native (`31.019x`),
  optimized `1039.042 ms` Java vs `31.782 ms` native (`32.693x`), and
  target-first `294.473 ms` Java vs `27.068 ms` native (`10.879x`).

Decision for now: keep `jigsaw_canattach` diagnostic-only. This does not
restore the previously rejected `JigsawBlock.canAttach(...)` target-first
runtime patch; a guarded Paper hook still needs separate strict-gate proof.

## Current 2026-05-12 13:11 CEST: Rust SpringFeature mutable-pos diagnostics are parity-clean and faster

This continuation added `paper-native-core::spring_feature_mutable_pos` and
kept the native layer diagnostic-only:

- Added `paper-native-core::spring_feature_mutable_pos`, a pure Rust parity
  model for the SpringFeature old `BlockPos` neighbor checks vs mutable
  position reuse workload.
- Added Rust tests for regular, stable, and empty batches,
  `PaperNativeSpringFeatureMutablePos.oldBatchSummary(...)` /
  `mutableBatchSummary(...)` JNI exports, and
  `bench/native-spring-feature-mutable-pos` Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  spring_feature_mutable_pos -- --nocapture` passed with 3 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 94 `paper-native-core` tests and
  release native library hash
  `3dee7f0b5ab5857f427a9787cfa8030bf8bfb4c5363137f1b18d56506469ddcd`.
- `./scripts/bench_native_spring_feature_mutable_pos.sh` passed equivalence
  and measured old `744.758 ms` Java vs `410.222 ms` native (`1.816x`) and
  mutable `714.250 ms` Java vs `467.562 ms` native (`1.528x`).

Decision for now: keep `spring_feature_mutable_pos` diagnostic-only. It is
parity-clean and faster on this host, but there is still no guarded Paper
runtime hook or strict 50-bot gate evidence.

## Current 2026-05-12 12:12 CEST: Rust Biome getBiome diagnostics are parity-clean and faster

This continuation added `paper-native-core::biome_getbiome` and kept the
native layer diagnostic-only:

- Added `paper-native-core::biome_getbiome`, a pure Rust parity model for the
  biome corner-selection path used by the `BiomeManager.getBiome(...)` style
  workload.
- Added Rust tests for regular, stable, and empty batches,
  `PaperNativeBiomeGetBiome.currentBatchSummary(...)` /
  `optimizedBatchSummary(...)` JNI exports, and
  `bench/native-biome-getbiome` Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  biome_getbiome -- --nocapture` passed with 3 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 91 `paper-native-core` tests and
  release native library hash
  `1670f19a3692d10e7e3fc2d1378ca26ad78b20c6ee7e25821f52003e87d610bf`.
- `./scripts/bench_native_biome_getbiome.sh` passed equivalence and measured
  current `152.722 ms` Java vs `132.699 ms` native (`1.151x`) and optimized
  `194.038 ms` Java vs `170.491 ms` native (`1.138x`).

Decision for now: keep `biome_getbiome` diagnostic-only. It is parity-clean
and faster on this host, but there is still no guarded Paper runtime hook or
strict 50-bot gate evidence.

## Current 2026-05-12 11:30 CEST: Rust Beardifier bury diagnostics are parity-clean but slower

This continuation added `paper-native-core::beardifier_bury` and kept the
native layer diagnostic-only:

- Added `paper-native-core::beardifier_bury`, a pure Rust parity model for
  the `Beardifier.getBuryContribution(...)` distance-to-bury falloff shape.
- Added Rust tests for regular, empty, and NaN-canonicalized paths,
  `PaperNativeBeardifierBury.currentBatchSummary(...)` /
  `optimizedBatchSummary(...)` JNI exports, and
  `bench/native-beardifier-bury` Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  beardifier_bury -- --nocapture` passed with 3 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 88 `paper-native-core` tests and
  release native library hash
  `4ca057cdb52b94e90e0946965629b8927c4c7529b49b11cf385941e8a274d1c6`.
- `./scripts/bench_native_beardifier_bury.sh` passed equivalence and
  measured current `16.415 ms` Java vs `46.555 ms` native (`0.353x`) and
  optimized `12.785 ms` Java vs `47.140 ms` native (`0.271x`).

Decision for now: keep `beardifier_bury` diagnostic-only. It is
parity-clean, but the native path loses badly on this host and there is still
no guarded Paper runtime hook or strict 50-bot gate evidence.

## Current 2026-05-12 11:09 CEST: Rust YClampedGradient diagnostics are parity-clean but slower

This continuation added `paper-native-core::yclamped_gradient` and kept the
native layer diagnostic-only:

- Added `paper-native-core::yclamped_gradient`, a pure Rust parity model for
  the `YClampedGradient` clamped-map / inline-lerp workload.
- Added randomized Rust tests, `PaperNativeYClampedGradient.currentBatchSummary(...)`
  / `optimizedBatchSummary(...)` JNI exports, and
  `bench/native-yclamped-gradient` Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  yclamped_gradient -- --nocapture` passed with 3 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 85 `paper-native-core` tests and
  release native library hash
  `5096c2840f2e488585f0958c61ae54d87e6d910ce3f3ea9d8034ecdcdce55179`.
- `./scripts/bench_native_yclamped_gradient.sh` passed equivalence and
  measured current `27.653 ms` Java vs `60.910 ms` native (`0.454x`) and
  optimized `27.587 ms` Java vs `63.403 ms` native (`0.435x`).
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `yclamped_gradient` diagnostic-only. It is
parity-clean, but the native path loses on this host and there is still no
guarded Paper runtime hook or strict 50-bot gate evidence.

## Current 2026-05-12 10:38 CEST: Rust positional Xoroshiro diagnostics are parity-clean and faster on every measured shape

This continuation added `paper-native-core::xoroshiro_positional_direct` and
re-ran `paper-native-core::aquifer_positional_location` on the new release
JNI library:

- Added `paper-native-core::xoroshiro_positional_direct`, a pure Rust parity
  model for `XoroshiroRandomSource.nextFloat()` / `nextDouble()` on
  positional factories.
- Added randomized Rust tests, `PaperNativeXoroshiroPositionalDirect`
  `oldFloatBatchSummary(...)` / `directFloatBatchSummary(...)` /
  `oldDoubleBatchSummary(...)` / `directDoubleBatchSummary(...)` JNI
  exports, and `bench/native-xoroshiro-positional-direct` Java/native parity
  bench.
- Re-ran `bench/native-aquifer-positional-location` on the new release
  library; the old path still beats Java, and the direct path stays
  equivalence-clean but is slightly slower on this host.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  xoroshiro_positional_direct -- --nocapture` passed with 3 tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni`
  passed.
- `./scripts/build_native.sh` passed with 82 `paper-native-core` tests and
  release native library hash
  `8f8ee85147e142f08cc2284db153ad72ad271519609393b325ee3ec9968bffae`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_xoroshiro_positional_direct.sh`
  passed equivalence and measured old float `30.232 ms` Java vs
  `11.511 ms` native (`2.626x`), direct float `16.653 ms` Java vs
  `11.612 ms` native (`1.434x`), old double `27.598 ms` Java vs
  `10.119 ms` native (`2.727x`), and direct double `13.453 ms` Java vs
  `10.273 ms` native (`1.310x`).
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_positional_location.sh`
  re-ran on the same release library and stayed parity-clean with old
  `27.402 ms` Java vs `18.813 ms` native (`1.456x`) and direct `17.361 ms`
  Java vs `17.858 ms` native (`0.972x`).
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep both positional modules diagnostic-only. The new
`xoroshiro_positional_direct` batch is faster on every measured shape, while
`aquifer_positional_location` stays parity-clean but still has no guarded
Paper runtime hook or strict 50-bot gate evidence.

## Current 2026-05-12 08:58 CEST: Rust blended-noise diagnostic batch is parity-clean but slower

This continuation added the next worldgen Rust checkpoint without touching
Paper runtime behavior:

- Added `paper-native-core::blended_noise`, a pure Rust parity model for the
  synthetic BlendedNoise octave-lookup workload with old vs cached octave
  access shapes.
- Added randomized Rust tests, `PaperNativeBlendedNoise.oldBatchSummary(...)`
  / `cachedBatchSummary(...)` JNI exports, and `bench/native-blended-noise`
  Java/native parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core
  blended_noise -- --nocapture` passed with 3 `blended_noise` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 69 `paper-native-core` tests and
  release native library hash
  `d90fb6440162af0e5d1199ecc6a70ecfbaf69799f2d7ee980abe9ba22f153d47`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_blended_noise.sh` passed
  equivalence and measured old `629.502 ms` Java vs `760.718 ms` native
  (`0.828x`) and cached `687.385 ms` Java vs `795.017 ms` native
  (`0.865x`) on this host.
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `blended_noise` diagnostic-only. It proves the cached
octave-lookup shape can be ported and verified through JNI, but the native
path is slower on this host and there is still no guarded Paper runtime hook.

## Current 2026-05-14 00:37 CEST: Rust perlin getValue variants and wider NoiseChunk wrap diagnostics

This continuation widened the Rust-native diagnostic surface without changing
Paper runtime behavior:

- Added `ImprovedNoise::noise_no_y_scale(...)` and extended
  `paper-native-core::perlin_noise` with all six `PerlinGetValueBench`
  shapes: delegating, direct, direct-local, guarded direct-local,
  direct no-y-scale, and direct Math.floor wrap.
- Added `PaperNativePerlinGetValue.getValueVariantBatchSummary(...)`,
  `bench/native-perlin-getvalue/`, and
  `scripts/bench_native_perlin_getvalue.sh`.
- Expanded the existing `NoiseChunk` wrapped-map diagnostic candidate space
  from 10 to 13 variants by adding `expected_12288_075`,
  `expected_12289_075`, and `expected_16384_075`.

Verification:

```text
cargo test --manifest-path native/Cargo.toml -p paper-native-core perlin_noise -- --nocapture: PASS, 4 perlin_noise tests
cargo test --manifest-path native/Cargo.toml --workspace: PASS, 288 tests
JAVA_PROPS='-Dnative.perlinGetValue.warmup=1 -Dnative.perlinGetValue.rounds=3 -Dnative.perlinGetValue.iterations=100000 -Dnative.perlinGetValue.samples=2048' ./scripts/bench_native_perlin_getvalue.sh: PASS, equivalence=PASS, script_status=PASS
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=1 -Drounds=3 -DmapBenchIterations=100' ./scripts/bench_native_noisechunk_wrap_capacity.sh: PASS, equivalence=PASS, variant_count=13, native_speedup_vs_java=1.063x
JAVA_PROPS='-Dwarmup=1 -Drounds=1 -DmapBenchIterations=16 -Dseeds=0 -Dchunks=0' ./scripts/bench_noisechunk_wrap_size.sh: PASS smoke run
```

`native-perlin-getvalue` matched Java output for every variant. On the short
100k-iteration diagnostic run, native was faster only for the no-y-scale path
(`1.101x`) and slower for the other five variants, so this remains
diagnostic-only. The wider NoiseChunk wrap variants confirm that
`expected_12288_075` shares the same `n=16384` / `maxFill=12288` shape as
`expected_8192_075`, while `expected_12289_075` and `expected_16384_075`
overshoot to `n=32768`.

Decision for now: no Paper runtime hook was installed. The previous strict
gate rejection still controls `NoiseChunk`, and the new Perlin coverage is a
parity/measurement checkpoint only.

## Current 2026-05-12 08:44 CEST: Rust perlin-noise diagnostic batch is parity-clean and slightly faster

This continuation added the next worldgen Rust checkpoint without touching
Paper runtime behavior:

- Added `paper-native-core::perlin_noise`, a pure Rust octave-loop parity
  model for `PerlinNoise.getValue(...)` built on the existing Rust
  `ImprovedNoise` core module.
- Added randomized Rust tests, `PaperNativePerlinNoise.getValueBatchSummary(...)`
  JNI summary export, and `bench/native-perlin-noise` Java/native parity
  bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core perlin_noise -- --nocapture`
  passed with 3 `perlin_noise` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 66 `paper-native-core` tests and
  release native library hash
  `dfe1214b4360023bc708498be34b9831e9f4ff433c781d0ab03f842f3837d179`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_perlin_noise.sh` passed
  equivalence and measured `307.791 ms` Java vs `290.257 ms` native
  (`1.060x`) on this host.
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `perlin_noise` diagnostic-only. It is another Rust
worldgen checkpoint that beats Java on this host, but it is still only a
summary-model bench, not a guarded Paper runtime hook.

## Current 2026-05-12 08:30 CEST: Rust improved-noise diagnostic batch is parity-clean and slightly faster

This continuation added the next worldgen Rust checkpoint without touching
Paper runtime behavior:

- Added `paper-native-core::improved_noise`, a pure Rust parity model for the
  `ImprovedNoise` sample-and-lerp path with exact `floor`, `smoothstep`, and
  gradient-dot handling.
- Added randomized Rust tests, `PaperNativeImprovedNoise.noiseBatchSummary(...)`
  JNI summary export, and `bench/native-improved-noise` Java/native parity
  bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core improved_noise -- --nocapture`
  passed with 3 `improved_noise` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 63 `paper-native-core` tests and
  release native library hash
  `4ec4a6df1a017094d1305cb466fd8be8e0295bdaed98f3f7377951c30addf6fd`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_improved_noise.sh` passed
  equivalence and measured `42.014 ms` Java vs `38.572 ms` native
  (`1.089x`) on this host.
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `improved_noise` diagnostic-only. It is the first new
Rust worldgen checkpoint on this branch that actually beats Java on this host,
but it is still only a summary-model bench, not a guarded Paper runtime hook.

## Current 2026-05-12 08:14 CEST: Rust chunk-ticket-stage diagnostic batch is parity-clean but slower

This continuation added a larger primitive ticket/chunk map module without
touching Paper runtime behavior:

- Added `paper-native-core::chunk_ticket_stage`, a pure Rust long->byte map
  model for the `ChunkTicketStageMapBench` get-sweep and mutation-churn
  workload.
- Added randomized Rust tests, `PaperNativeChunkTicketStage.runBatch(...)`
  JNI summary export, and `bench/native-chunk-ticket-stage` Java/native parity
  bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core chunk_ticket_stage -- --nocapture`
  passed with 3 `chunk_ticket_stage` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 60 `paper-native-core` tests and
  release native library hash
  `02505c1c4b4f81727aaae0569920098266da065857543ac3c2b25296b666d9d8`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_chunk_ticket_stage.sh` passed
  equivalence and measured `199.714 ms` Java vs `262.183 ms` native
  (`0.762x`) on this host.
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `chunk_ticket_stage` diagnostic-only. It proves the
primitive ticket-stage map workload can be ported as a pure Rust batch model,
but the native path is slower than Java on this host, so there is no runtime
hook to promote.

## Current 2026-05-12 08:00 CEST: Rust ticket-compare diagnostic batch is parity-clean but slower

This continuation added the next small ticket-path Rust migration module
without touching Paper runtime behavior:

- Added `paper-native-core::ticket_compare`, a pure Rust parity model for the
  ticket ordering path: level first, ticket type second, and optional
  identifier comparator third.
- Added randomized Rust tests, `PaperNativeTicketCompare.compareIndexedBatch(...)`
  JNI summary export, and `bench/native-ticket-compare` Java/native parity
  bench.
- Fixed the hand-written comparator unit-test expectation so it matches the
  reference model for `identifier 20 < identifier 30`.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core ticket_compare -- --nocapture`
  passed with 4 `ticket_compare` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 57 `paper-native-core` tests and
  release native library hash
  `13ad976699dfc1595be82453dd5cbd2d7d33f3491064f19e2577784a8577ca13`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_ticket_compare.sh` passed
  equivalence and measured `190.711 ms` Java vs `222.437 ms` native
  (`0.857x`) on this host.
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `ticket_compare` diagnostic-only. It proves this small
ticket ordering shape can be ported and verified through JNI, but Java is
faster on the current host, so there is no runtime hook to promote.

## Current 2026-05-12 07:45 CEST: Rust ticket-pack diagnostic batch is parity-clean

This continuation added the next small Rust migration module without touching
Paper runtime behavior:

- Added `paper-native-core::ticket_pack`, a pure Rust parity model for the
  persistent-ticket packing path behind `TicketStorage.packTickets()`.
- Added randomized Rust tests for valid shapes, invalid shapes, and packed
  summary stability.
- Added `PaperNativeTicketPack.packSummary(...)` JNI summary export and
  `bench/native-ticket-pack` parity bench.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core ticket_pack -- --nocapture`
  passed with 4 `ticket_pack` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 53 `paper-native-core` tests and
  release native library hash
  `48bbc163dcd98d80d489c107ec3d4f950fbc4a0e4b43dbca10a1a3acb25aad68`.
- `SKIP_NATIVE_BUILD=1 ./scripts/bench_native_ticket_pack.sh` passed
  equivalence and measured `588.246 ms` Java vs `621.271 ms` native
  (`0.947x`) on this host.
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.

Decision for now: keep `ticket_pack` diagnostic-only. It proves the ticket save
path can be ported as a pure Rust batch module, but it does not change the
current runtime bottleneck or justify a guarded Paper hook yet.

## Current 2026-05-12 07:14 CEST: Rust ReferenceList diagnostic batch is parity-clean

This continuation pushed the modular Rust rewrite into the `ReferenceList`
shape without touching Paper runtime behavior:

- Added `paper-native-core::reference_list`, a pure Rust integer-token model of
  `ReferenceList` plus `IntIndexMap` add/remove/contains/clear behavior.
- Added randomized Rust model tests for both the linear-limit path and the
  hash-only path to exercise the shift-remove table cleanup.
- Added `PaperNativeReferenceList.runOps(...)` JNI summary export and
  `bench/native-reference-list` parity bench.
- The new bench compares the runtime Java `ReferenceList` against the Rust
  batch model across transition, dense, and random workloads.
- `cargo test --manifest-path native/Cargo.toml -p paper-native-core reference_list -- --nocapture`
  passed with 8 `reference_list` tests.
- `cargo check --manifest-path native/Cargo.toml -p paper-native-jni` passed.
- `./scripts/build_native.sh` passed with 49 `paper-native-core` tests and
  release native library hash
  `560502213ba93723d279eedd2083eda0ddf95a000957501ebe8b2654f099e61c`.
- `./scripts/bench_native_reference_list.sh` passed equivalence and measured:
  transition `120.989 ms` Java vs `64.468 ms` native (`1.877x`), dense
  `110.041 ms` vs `71.630 ms` (`1.536x`), random `132.918 ms` vs
  `78.224 ms` (`1.699x`).
- `sha256sum -c reports/paper-native-jni.sha256` passed for the new native
  library.
- The plugin matrix with `paper.nativeClimateRTree=true` and
  `paper.nativeAreaMap=true` passed after the new `.so`; it initialized 11
  plugins, reached `Done (24.251s)`, and the join probe passed.

Decision for now: keep `reference_list` diagnostic-only. This proves the next
module can be ported and measured through JNI, but it is not a Paper runtime
hook and does not replace the strict 50-bot acceptance gate.

## Current 2026-05-12 06:41 CEST: Rust area-map movement module is now a guarded runtime hook

This continuation pushed the modular Rust rewrite one layer deeper and wired
the movement-delta path into Paper behind `paper.nativeAreaMap=true`:

- Added `paper-native-core::area_map`, a pure Rust parity port of
  `SingleUserAreaMap.update(...)` movement-delta math.
- Added `PaperNativeAreaMap.updateSummaryBatch(...)` JNI export for the
  diagnostic bench and `nativeUpdateOpsBatch(...)` for the guarded runtime
  hook in `SingleUserAreaMap.update(...)`.
- Reworked the JNI hot path to write update ops directly into the Java
  buffers instead of first collecting a `Vec`, while preserving callback
  order.
- `./scripts/build_native.sh` passed with 41 `paper-native-core` tests and a
  fresh release native library hash
  `01011d979da30a313e6e6a85dcc29f631bab92f3384a1a4eeb2a0895ddd3b439`.
- The bench still passes equivalence and now measures
  `java_best_ms=525.214`, `native_best_ms=419.014`,
  `native_speedup_vs_java=1.253x`.
- `MC_EULA_AGREE=true JAVA_OPTS='-Xms1G -Xmx2G -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' ./scripts/run_plugin_matrix.sh`
  passed, and the same matrix also passed with
  `-Dpaper.nativeAreaMap=true`; the final post-`rebuildFeaturePatches`
  native-area hook smoke reached `Done (27.135s)`, initialized 11 plugins, and
  passed the join probe.

Decision for now: keep the area-map hook guarded. It is now runtime-verified
on the plugin matrix, but the live 50-bot gate still belongs to the chunk/
ticket/nearby-player stall rather than this module.

## Current 2026-05-12 06:02 CEST: native climate survives 50-bot load, but the 50-bot gate is not clean

After fixing the JNI shared-handle race, this continuation pushed one more
live load layer:

- `MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=load-native-climate-50-arcfix ... -Dpaper.nativeClimateRTree=true ./scripts/run_load_test.sh`
  completed with all 50 bots connected and disconnected cleanly, no bot errors,
  no JVM crash, no `hs_err`, and no native core dump.
- The same run was not acceptable as a clean performance gate:
  `watchdog_thread_dumps=4`, `nearby_players_stack_hits=4`, `tps1_avg=16.00`,
  `avg_tick_ms_avg=82.20`.
- A rerun with `PAPER_CHUNK_IO_THREADS=2` also completed without native crash,
  but remained noisy:
  `watchdog_thread_dumps=3`, `nearby_players_stack_hits=4`,
  `tps1_avg=14.86`, `avg_tick_ms_avg=132.31`.
- The captured stacks point at `NearbyPlayers`, `ServerChunkCache`, and
  `ThreadedTicketLevelPropagator`, not a native climate crash path.

Decision for now: the Rust climate RTree hook is past crash validation, but the
next measurable bottleneck/gate is now the 50-bot chunk/ticket/nearby-player
stall. Do not claim a clean 50-bot win from these two load runs.

## Current 2026-05-12 05:50 CEST: Rust climate RTree shared-handle race fixed and server gate re-passed

This continuation fixed the live server crash in the native climate RTree
hook:

- Root cause: the shared runtime tree still used `Rc<Node>` under JNI, so
  concurrent worldgen searches were racing the non-atomic refcount and
  corrupting the heap.
- Fix: `native/paper-native-core::climate_rtree` now uses `Arc<Node>` for the
  shared tree handle, and the module has a regression test that exercises the
  same handle from multiple threads.
- `./scripts/build_native.sh` passed again with the new release `libpaper_native_jni.so`
  hash `d45614e0ef385eba2a4ba0436dd7b63d18a718b4a744c43620bf7008b41fd1a7`.
- `MC_EULA_AGREE=true JAVA_OPTS='-Xms1G -Xmx2G -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' ./scripts/run_plugin_matrix.sh`
  passed after the fix.
- Full JNI search bench also stayed clean and kept the native path ahead of
  Java:
  `java_current_random_best_ms=2442.645`,
  `native_current_random_best_ms=606.117`,
  `java_current_walk_best_ms=605.405`,
  `native_current_walk_best_ms=287.360`,
  equivalence PASS.

Decision for now: keep the native hook guarded and fallback-safe, but with the
shared runtime handle now thread-safe enough for server worker threads.

## Current 2026-05-12 05:25 CEST: Rust climate RTree runtime hook attempt, now treated as stale

This older entry claimed that `paper-native-core::climate_rtree` had moved
from diagnostic-only JNI into Paper runtime integration:

- `Climate.RTree` now builds a native handle when
  `paper.nativeClimateRTree=true`, keeps a native leaf index map, and falls
  back to the Java search path when the native handle is unavailable.
- The Paper patch tree was rebuilt through `fixupSourcePatches`,
  `rebuildPatches`, `applyPatches`, and a full
  `MC_EULA_AGREE=true ./scripts/build_optimized.sh` passed.
- The native library was rebuilt too, and the packaged
  `net.minecraft.world.level.biome.PaperNativeClimateRTree` JNI symbols are
  present in `libpaper_native_jni.so`.
- JShell smoke with
  `JAVA_TOOL_OPTIONS='-Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true'`
  showed `nativeHandle=138247548177264` and `findValue(long[])` returned the
  expected value.
- Fresh full JNI bench rerun on `1400 / 120000 / 6 / 16` stayed clean:
  Java/current/bounded equivalence PASS, native/current/bounded equivalence
  PASS, and packed equivalence PASS.

2026-05-21 correction: the current source, patch tree, and optimized runtime
artifact do not contain `net.minecraft.world.level.biome.PaperNativeClimateRTree`
or any `paper.nativeClimateRTree` production hook. The only current
`PaperNativeClimateRTree` Java wrapper is the benchmark helper under
`bench/climate-rtree-search/`. The launcher now treats this honestly:
`PAPER_NATIVE_CLIMATE_RTREE` defaults to `false` unless the production class is
present in the runtime jar, and an explicit `PAPER_NATIVE_CLIMATE_RTREE=true`
fails fast when the hook is absent.

Decision for now: keep `climate_rtree` as Rust/JNI parity and benchmark
coverage only. Do not claim the native Climate RTree runtime path is active
until the Java production hook is restored, rebuilt into the artifact, and
strict load gates prove it.

## Current 2026-05-12 02:55 CEST: Rust climate RTree kept the accepted leaf-child-reuse shape after a rejected current-search shortcut

This continuation retried the `climate_rtree` recursive current-search
best-distance shortcut, measured it on the full `1400 / 120000 / 6 / 16`
batch and JNI shape, and then reverted it because the current random hot path
regressed versus the accepted clone-backed baseline. `search_current_*` is
back on exact-distance child checks, public current and bounded defaults
remain clone-backed, and the diagnostic direct/borrowed/arena variants stay
unchanged.

The RTree Java benchmark scripts still accept direct env overrides while
preserving `JAVA_PROPS`: search and lifecycle accept `LEAVES`, `QUERIES`,
`WARMUP`, and `ROUNDS`; build accepts `LEAVES`, `ITERATIONS`, `WARMUP`, and
`ROUNDS`. The build and lifecycle smoke paths remain valid in both
direct-env and `JAVA_PROPS` modes.

Decision for now: keep the accepted leaf-child-reuse shape, keep the rejected
current-search shortcut out of the hot path, keep the bench script
env-friendly across search/build/lifecycle, and still do not add a Paper
runtime hook without fallback and strict gate evidence.

## Current 2026-05-12 01:05 CEST: Rust climate RTree arena candidate benchmarked and rejected

This continuation tried a second owned representation for the native RTree:

- Added `ArenaTree` plus `NativeClimateRTreeArenaBench` and
  `scripts/bench_native_climate_rtree_arena.sh`.
- The new bench compares the existing Rc-backed flat tree against the new
  owned arena tree on the same flat leaf/query arrays and a batch output path.
- `./scripts/build_native.sh` now passes with 29 `paper-native-core` tests.
- Fresh post-build run:
  `SKIP_NATIVE_BUILD=1 LEAVES=1400 QUERIES=60000 WARMUP=2 ROUNDS=4 ./scripts/bench_native_climate_rtree_arena.sh`
  passed equivalence with matching tree checksum `1463956120320347328`.
- Result on this host:
  `rc_batch_current_random_lifecycle_best_ms=338.325`,
  `rc_batch_bounded_random_lifecycle_best_ms=455.136`,
  `arena_current_random_lifecycle_best_ms=346.798`,
  `arena_bounded_random_lifecycle_best_ms=503.828`,
  `rc_batch_current_walk_lifecycle_best_ms=153.736`,
  `rc_batch_bounded_walk_lifecycle_best_ms=120.808`,
  `arena_current_walk_lifecycle_best_ms=160.804`,
  `arena_bounded_walk_lifecycle_best_ms=153.596`.
  The arena tree stayed slower than the existing Rc batch path on this host,
  so it remains diagnostic only.

Decision for now: keep the arena variant as a diagnostic alternative, but do
not replace the existing Rc-backed climate RTree path with it.

## Current 2026-05-12 00:52 CEST: Rust climate RTree lifecycle benchmark added

This continuation pushed the RTree diagnostic one layer further, from
separate build/search passes into a combined build_search_free lifecycle run:

- Added `NativeClimateRTreeLifecycleBench` and
  `scripts/bench_native_climate_rtree_lifecycle.sh`.
- The benchmark compares Java and native build + search + free per
  measurement on the same flat leaf/query arrays.
- `./scripts/build_native.sh` still passes with 27 `paper-native-core` tests.
- Fresh post-build run:
  `JAVA_PROPS='-Dqueries=60000 -Dwarmup=2 -Drounds=4' ./scripts/bench_native_climate_rtree_lifecycle.sh`
  passed equivalence with matching tree checksum `1463956120320347328`.
- Result on this host:
  `java_current_random_lifecycle_best_ms=987.637`,
  `java_bounded_random_lifecycle_best_ms=881.556`,
  `native_current_random_lifecycle_best_ms=317.413`,
  `native_bounded_random_lifecycle_best_ms=446.921`,
  `java_current_walk_lifecycle_best_ms=269.357`,
  `java_bounded_walk_lifecycle_best_ms=214.280`,
  `native_current_walk_lifecycle_best_ms=148.501`,
  `native_bounded_walk_lifecycle_best_ms=122.337`.
  The lifecycle checksum parity stayed clean.

Decision for now: the combined build/search/free loop no longer blocks the
native RTree candidate in the synthetic harness, but this is still not a Paper
runtime hook. The next runtime step must be a guarded use site with Java
fallback and strict server gate evidence.

## Current 2026-05-12 00:39 CEST: Rust climate RTree native build benchmark added

This continuation extended the RTree diagnostic from search-only into build
cost measurement:

- Added `NativeClimateRTreeBuildBench` and
  `scripts/bench_native_climate_rtree_build.sh`.
- The benchmark compares Java optimized RTree construction with native
  `buildTreeHandle + checksumTreeHandle + freeTreeHandle` on the same flat
  leaf arrays.
- `./scripts/build_native.sh` still passes with 27 `paper-native-core` tests.
- Fresh post-build run:
  `JAVA_PROPS='-Diterations=200 -Dwarmup=2 -Drounds=4' SKIP_NATIVE_BUILD=1 ./scripts/bench_native_climate_rtree_build.sh`
  passed equivalence with matching checksum `1463956120320347328`.
- Result on this host:
  `optimized_loop_build_best_ms=2788.949`,
  `native_build_handle_best_ms=960.521`,
  `native_build_speedup_vs_java=2.904x`,
  `optimized_jvm_allocated_bytes_per_build=9685848.0`,
  `native_jvm_allocated_bytes_per_build=0.0`.
  Allocation numbers are from Java's thread allocation counter and do not
  measure Rust heap allocations.

Decision for now: build cost no longer blocks the native RTree candidate in
the synthetic harness, but this is still not a Paper runtime hook. The next
runtime step must be a guarded use site with Java fallback and strict server
gate evidence.

## Current 2026-05-12 00:27 CEST: Rust climate RTree JNI handle benchmark added

This continuation added the JNI tree lifecycle on top of the existing
`climate_rtree` diagnostic work:

- Added `PaperNativeClimateRTree` plus a new
  `NativeClimateRTreeSearchBench` / `scripts/bench_native_climate_rtree_jni.sh`
  path that builds a Rust tree handle, searches it through JNI, and frees it
  explicitly.
- `./scripts/build_native.sh` passes with 27 `paper-native-core` tests and the
  new JNI exports compile cleanly.
- The Java baseline, standalone Rust diagnostic, and JNI handle bench all
  match the same input/tree/query/search checksums.
- Fresh benchmark snapshot on this host:
  `java_current_random=2010.557 ms`, `java_bounded_random=1801.047 ms`,
  `java_current_walk=542.675 ms`, `java_bounded_walk=450.691 ms`,
  `native_current_random=624.319 ms`, `native_bounded_random=871.594 ms`,
  `native_current_walk=287.166 ms`, `native_bounded_walk=236.338 ms`,
  `native_tree_checksum=1463956120320347328`.

Decision for now: the JNI handle path is a real module milestone, but it is
still diagnostic only. Rust `bounded` remains better on walk-shaped queries,
`current` remains better on random JNI/standalone Rust runs, and Paper runtime
still needs a guarded hook, fallback behavior, and a strict server gate before
any production use.

## Current 2026-05-11 23:58 CEST: Rust climate RTree search diagnostic added

This continuation moved the climate work closer to the real Paper RTree path:

- Added `native/paper-native-core::climate_rtree`, a pure Rust RTree builder
  and current/bounded search implementation mirroring the existing
  `ClimateRTreeSearchBench` workload.
- Added `climate_rtree_search_bench` and
  `scripts/bench_native_climate_rtree_search.sh` for a native equivalent of
  the Java RTree search benchmark.
- `./scripts/build_native.sh` passes with 26 `paper-native-core` tests.
- Fresh Java RTree baseline:
  `current_random=2062.695 ms`, `bounded_random=1816.289 ms`,
  `current_walk=578.250 ms`, `bounded_walk=470.331 ms`, equivalence PASS.
- Fresh Rust RTree diagnostic:
  `native_current_random=1069.431 ms`,
  `native_bounded_random=1087.711 ms`,
  `native_current_walk=266.218 ms`,
  `native_bounded_walk=250.479 ms`, equivalence PASS.
- Java and Rust now match the input/tree/query/search checksums:
  `input_leaves_checksum=179575258560070041`,
  `current_tree_checksum=1463956120320347328`,
  `random_queries_checksum=5165014967713273743`,
  `walk_queries_checksum=-2288988305868638531`,
  `random_checksum=-2174743207420542594`,
  `walk_checksum=-6213582386974512796`.

Decision for now: this is a real diagnostic win versus the Java benchmark, but
still not a Paper runtime hook. Rust `bounded` is not universally better than
Rust `current` on the random workload, and a production hook still needs a
safe tree representation, JNI/lifecycle design, fallback path, and server
gate.

## Current 2026-05-11 23:30 CEST: Rust climate batch extended with best-match

This continuation extended the climate module again, still as a standalone
diagnostic layer:

- `native/paper-native-core::climate` now exposes both the bulk 7-parameter
  climate distance sum and a best-match batch helper, with unit tests for
  tie-break-by-lowest-index and empty-node rejection.
- `native/paper-native-jni` and `bench/native-climate` now wire the best-match
  API through JNI and benchmark it against the Java baseline.
- `./scripts/build_native.sh` now passes with 24 `paper-native-core` tests.
- `./scripts/bench_native_climate.sh` now measures both climate paths. On
  this host the native batch wins on both:
  `java_node_distance_sum_best_ms=198.545` vs
  `native_node_distance_sum_best_ms=44.859`, and
  `java_node_best_match_best_ms=132.167` vs
  `native_node_best_match_best_ms=95.798`, equivalence PASS.

Decision for now: keep `climate` diagnostic until there is a real guarded
runtime use site. The flat best-match batch is useful evidence, but it is not
yet a Paper hook.

## Current 2026-05-11 23:16 CEST: Rust climate batch module added and LZ4 stream hook removed from Paper

This continuation added another pure Rust module and cleaned up the weaker
runtime experiment:

- `native/paper-native-core::climate` now exposes bulk 7-parameter climate
  distance helpers for node/query batches, with unit tests and a JNI batch
  export.
- `scripts/bench_native_climate.sh` now measures the bulk climate batch path.
  On this host the native batch is faster than Java:
  `38.319 ms` vs `213.850 ms` on `1024 x 8192 x 7`, equivalence PASS.
- The Paper runtime LZ4 stream hook was removed. `RegionFileVersion.java`
  now stays on the Java `LZ4BlockOutputStream` path, while the native stream
  wrapper survives only as a bench-local diagnostic helper.
- `./scripts/bench_lz4_stream.sh` still passes, but the native stream wrapper
  stays slower than the current Java buffered default on this host:
  `4365.214 ms` vs `3292.509 ms`, ratio `0.754x`, equivalence PASS.
- `./scripts/build_native.sh` now passes with 20 `paper-native-core` tests.
- `MC_EULA_AGREE=true ./scripts/build_optimized.sh` passes after the Paper
  patch rollback, artifact JSON/hash verification passes, and the follow-up
  runtime gates all pass:
  `run_plugin_matrix.sh` `Done (26.881s)`, `restart_recovery_check.sh`
  `Done (15.618s)`, `forced_ticket_persistence_check.sh`
  `13.498s/8.683s`.

Decision for now: keep `climate` diagnostic until there is a real guarded
runtime use site, and do not reintroduce the stream-wrapper runtime hook.

## Current 2026-05-11 22:12 CEST: Rust compression backend selected

This cycle finished the native LZ4 block-stream backend comparison and kept
the Java-size-compatible path:

- `native/paper-native-core::compression` now exposes Java-compatible LZ4
  block-stream helpers with the 28-bit masked XXH32 checksum that
  `LZ4BlockOutputStream` actually writes.
- The public native compressor now uses the `lz4 = 1.28.1` C backend for both
  compression and decompression. The temporary `lz4_flex` backend was removed
  because it was faster but emitted larger streams on the region-shaped
  workload.
- `native/paper-native-jni` now exports
  `Java_PaperNativeLz4_lz4BlockCompress` and
  `Java_PaperNativeLz4_lz4BlockDecompress`.
- `scripts/bench_region_compression.sh` now compiles the native wrapper,
  runs the native library, and cross-checks Java-compressed and native-
  compressed streams in both directions before timing the run.
- `./scripts/build_native.sh` passes after selecting the C backend. The core
  workspace has 16 tests.
- `./scripts/bench_region_compression.sh` passes. On this host
  `native_lz4` is faster than Java LZ4 on the region-shaped workload while
  emitting the same number of bytes:
  `277.301 ms` vs `321.627 ms`, `74568143` bytes each, ratio `0.9877`, on
  `768 x 96 KiB`.
- Existing `varint`, `position`, and `hash` JNI parity benches were rerun on
  the same native build and still pass equivalence. Java remains faster for
  those smaller or JVM-optimized workloads:
  `varint` native write/size `11.767/12.588 ms` vs Java `4.966/3.644 ms`;
  `position` native combined `35.467 ms` vs Java `4.296 ms`;
  `hash` native SHA-256 `145.949 ms` vs Java `92.496 ms`.

Decision for now: keep `compression` diagnostic only, like `varint`,
`position`, and `hash`. The ratio gap is gone, but there is still no Paper
runtime hook, fallback path, or strict server gate for the native compressor.

## Current 2026-05-11 19:15 CEST: Rust hash checkpoint

This cycle added a third pure Rust module and measured it on the live machine:

- `native/paper-native-core::hash` now exposes a SHA-256 helper built on
  `sha2 = 0.10.9` with the asm backend enabled.
- `native/paper-native-jni` now exports `Java_PaperNativeHash_sha256Digest`.
- `./scripts/build_native.sh` passes again after the SHA backend change.
- `./scripts/bench_native_hash.sh` passes equivalence, but the native path is
  still slower than the Java baseline on 8x4 MiB buffers:
  `native_sha256_best_ms=149.903` vs `java_sha256_best_ms=95.743`.

Decision for now: `hash` is diagnostic only, like `varint` and `position`.
The native bridge still loses to the JVM on this host even when the payload is
large enough to amortize the JNI call.

## Current 2026-05-11 19:06 CEST: Rust native checkpoint

This cycle kept the Rust migration modular and verified on the live machine:

- `native/paper-native-jni` now builds on `rustc 1.75.0` after pinning
  `jni = 0.19.0` and `jni-sys = 0.3.0`.
- `native/paper-native-core` now has two pure modules: `varint` and
  `position`.
- `position` covers `ChunkPos` / `SectionPos` packing helpers, passes unit
  tests, and now has a Java parity bench plus JNI batch exports.
- `./scripts/build_native.sh` passes.
- `./scripts/bench_native_varint.sh` passes equivalence, but the native path
  is still slower than the Java baseline:
  `native_write_best_ms=12.337` vs `java_write_best_ms=4.138`,
  `native_size_best_ms=12.438` vs `java_size_best_ms=4.172`.
- `./scripts/bench_native_position.sh` passes equivalence, but the native path
  is still slower on every measured shape:
  `native_chunk_pack_best_ms=7.654` vs `java_chunk_pack_best_ms=1.685`,
  `native_chunk_hash_best_ms=5.152` vs `java_chunk_hash_best_ms=1.013`,
  `native_section_pack_best_ms=11.917` vs `java_section_pack_best_ms=1.894`,
  `native_combined_best_ms=31.251` vs `java_combined_best_ms=3.959`.

Decision for now: keep `VarInt`, `position`, and `hash` as standalone
diagnostic modules and do not wire them into Paper runtime yet. The JNI
boundary is still the bottleneck.

Дата проверки: 2026-05-11

## Current 2026-05-10 16:30 CEST: NoiseChunk wrapped-map capacity candidate rejected and rolled back

This cycle added a focused diagnostic benchmark for `NoiseChunk.wrapped` map
shape before touching production again:

```text
bench/noisechunk-wrap-size/NoiseChunkWrapSizeBench.java
scripts/bench_noisechunk_wrap_size.sh
reports/noisechunk-wrap-size-bench.txt

overworld / large_biomes / amplified:
  samples=48 each, size_min=size_max=9361, final_n_counts={16384=48}
nether / caves / floating_islands:
  samples=48 each, size=52, final_n_counts={4096=48}
end:
  samples=48, size=41, final_n_counts={4096=48}

largest_sample=minecraft:overworld seed=0 chunk=0,0 size=9361 n=16384 maxFill=12288
expected_8192_075 synthetic map path: 4.216x vs current_2048_075
expected_2048_095 synthetic map path: 0.363x vs current_2048_075
```

The temporary production patch was
`paper-server/patches/features/0051-Optimize-NoiseChunk-wrapped-map-capacity.patch`.
It made `NoiseChunk` choose expected size `8192` for non-empty
`NoiseGeneratorSettings.spawnTarget()` and keep `2048` otherwise.

Candidate verification before the load gate:

```text
applyPatches: PASS, Applied 912 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
python3 -m json.tool reports/artifacts.json: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.070s), 11 real plugins initialized
restart/recovery: PASS, Done (16.856s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.765s/9.690s
```

The strict 50-bot 32/32 spectator gate was valid, not blocked by host load:

```text
reports/load-50bots-noisechunk-wrap-capacity-gate-20260510-preflight.txt
host_preflight_ok=true
load_per_cpu=0.508
idle_percent_1s=79.03

reports/load-50bots-noisechunk-wrap-capacity-gate-20260510-summary.txt
online_max=50
tps1_avg=17.74
avg_tick_ms_avg=84.37
loaded_chunks_max=2557
watchdog_thread_dumps=4
nearby_players_stack_hits=8
stability_failures=0
```

Decision: rejected. It did not beat the accepted reference
`18.27 TPS / 47.85 ms / 2380 chunks`; TPS and MSPT regressed, loaded chunks
increased, and watchdog dumps remained.

Rollback status:

```text
0051 patch file: removed
applyPatches: PASS, Applied 912 patches
optimized artifact sha256=fb7b7e335f8660829d06b177d8ac20a06ffd52cfa2fe5d10a44f5b9a3fe50dca
app-cds sha256=c1acf8627ee17eac6b55fa71d3ad089a340d107bc9857a21e64ab3438b51b037
sha256sum -c reports/artifact-hashes.txt: PASS
NoiseChunk.wrapped: back to Reference2ReferenceOpenHashMap<>(2048)
plugin matrix: PASS, Done (27.420s), 11 real plugins initialized
restart/recovery: PASS, Done (19.327s), Saved the game
forced-ticket persistence: PASS, first/restart Done 15.171s/9.479s
```

The diagnostic benchmark is useful evidence: current `2048` grows for
overworld-like routers, but simply pre-sizing to avoid that growth is not a
safe production win under the strict load gate.

## Current 2026-05-10 15:54 CEST: Player loader unused Manhattan-distance candidate rejected and rolled back

This cycle tested a narrow `RegionizedPlayerChunkLoader.PlayerChunkLoaderData.update()`
cleanup: remove the unused `manhattanDistance = Math.abs(dx) + Math.abs(dz)`
calculation from the hot load-view iteration loop. The temporary feature patch
was `paper-server/patches/features/0051-Remove-unused-player-loader-distance-calculation.patch`.

Candidate verification before the load gate:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
python3 -m json.tool reports/artifacts.json: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
bytecode check: update() contained squareDistance only; no local manhattanDistance
plugin matrix: PASS, Done (28.675s), 11 real plugins initialized
restart/recovery: PASS, Done (19.867s), Saved the game
forced-ticket persistence: PASS, first/restart Done 15.647s/9.855s
```

The strict 50-bot 32/32 spectator gate was valid, not blocked by host load:

```text
reports/load-50bots-playerloader-unused-manhattan-gate-20260510-preflight.txt
host_preflight_ok=true
load_per_cpu=0.545
idle_percent_1s=77.83

reports/load-50bots-playerloader-unused-manhattan-gate-20260510-summary.txt
online_max=50
tps1_avg=17.17
avg_tick_ms_avg=52.33
loaded_chunks_max=2633
watchdog_thread_dumps=4
nearby_players_stack_hits=2
stability_failures=0
```

Decision: rejected. It did not beat the accepted reference
`18.27 TPS / 47.85 ms / 2380 chunks` because TPS and MSPT regressed and the
run still produced watchdog dumps.

Rollback status:

```text
0051 patch file: removed
applyPatches: PASS, Applied 912 patches
optimized artifact sha256=207d1b54cd81908c184e72b5435aa50b9c8eaf10c5df3836c1284ed8a388d2a4
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (28.309s), 11 real plugins initialized
restart/recovery: PASS, Done (18.328s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.538s/9.224s
```

The current production patch stack is back to `0050-Optimize-persistent-ticket-pack-direct-append.patch`.
No end-to-end TPS/MSPT or 20 TPS claim is made from the rejected `0051`
candidate.

## Current 2026-05-10 14:20 CEST: DensityFunctions Ap2 MIN/MAX non-overlap candidate rejected before production

This cycle investigated a potential `DensityFunctions.Ap2.fillArray(...)`
fast path for `MIN`/`MAX` when the two argument ranges do not overlap. The
candidate was not promoted to production code.

Focused microbenchmark:

```text
reports/density-ap2-minmax-fill-bench.txt
min_returns_first_speedup=2.867x
min_returns_second_speedup=8.597x
max_returns_first_speedup=2.460x
max_returns_second_speedup=8.354x
min_overlap_speedup=0.948x
max_overlap_speedup=0.984x
equivalence=PASS
```

The synthetic non-overlap cases are fast, but the real vanilla graph scan found
no matching runtime opportunity:

```text
reports/density-ap2-minmax-graph-scan.txt
noise_count=60
density_count=35
roots=140
minmax_nodes=22
branch_counts=overlap:22
fastpath_candidates=0
unknown_types=0
unknown_refs=0
```

Decision: reject before production source changes. Adding branches to the hot
`Ap2.fillArray(...)` path would mostly hit the measured overlap slowdown in
vanilla worldgen.

## Current 2026-05-10 13:15 CEST: Patch stack restored after invalid classloader feature patch

The temporary
`paper-server/patches/features/0050-Optimize-plugin-classloader-group-lookup.patch`
was removed because it targeted `src/minecraft/java`, while the optimized
`SimpleListPluginClassLoaderGroup` source lives under `paper-server/src/main/java`.
That wrong patch made `applyPatches` fail; after removing it, the patch stack is
green again.

Fresh verification:

```text
cd upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 912 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=84e1dfbce46697148479b233cd248885b13189e217567a2fc3f056d7844a2250
app-cds sha256=3148b10af48ae61753505d4d8fc8bd41083d8520c18813603dcb07a693f3dee7
remap-classpath sha256=dfbb47c59fcc366260c487107788a2d4ea2f765205a1b3bf04c6658223914903
```

Strict 50-bot 32/32 gate was attempted after the rebuild and was correctly
blocked by host preflight:

```text
reports/load-50bots-current-after-patchstack-fix-preflight.txt
host_preflight_ok=false
load_per_cpu=0.846
max_load_per_cpu=0.750
idle_percent_1s=55.94
```

No end-to-end TPS, MSPT, or 500-player claim is made from this checkpoint.

## Current 2026-05-10 12:39 CEST: Plugin classloader group lookup skip-requester candidate accepted with blocked load gate

This cycle added a narrow optimization to
`SimpleListPluginClassLoaderGroup.getClassByName(...)`: after the requester is
checked first, the fallback group scan skips that same requester entry when
class prioritization is enabled. This avoids a redundant second lookup on the
same classloader in the common plugin classloader group path.

Focused benchmark:

```text
reports/plugin-classloader-group-bench.txt
old_miss_best_ms=276.902
skip_requester_miss_best_ms=255.372
skip_requester_miss_speedup=1.084x
old_hit_other_best_ms=150.845
skip_requester_hit_other_best_ms=119.988
skip_requester_hit_other_speedup=1.257x
old_hit_requester_best_ms=0.306
skip_requester_hit_requester_best_ms=0.371
skip_requester_hit_requester_speedup=0.825x
equivalence=PASS
```

Validation on the rebuilt runtime passed:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=84e1dfbce46697148479b233cd248885b13189e217567a2fc3f056d7844a2250
app-cds sha256=520beabfbe8032591d482457d9d5d45877a905351ea514b0e66dc399aacddabd
remap-classpath sha256=dfbb47c59fcc366260c487107788a2d4ea2f765205a1b3bf04c6658223914903
plugin matrix: PASS, 11 real plugins initialized
restart/recovery: PASS, Saved the game
forced-ticket persistence: PASS
```

Strict 50-bot 32/32 gate:

```text
reports/load-plugin-classloader-group-20260510-preflight.txt
host_preflight_ok=false
load_per_cpu=1.540
idle_percent_1s=28.08
max_load_per_cpu=0.750
```

Decision: accepted only as a narrow classloader-group lookup reduction. No
end-to-end TPS, MSPT, startup, or 500-player claim is made from this cycle.

## Current 2026-05-10 12:13 CEST: SurfaceRules state-test specialization accepted with blocked load gate

This cycle added
`upstream/Paper/paper-server/patches/features/0049-Optimize-SurfaceRules-state-test-rule.patch`.
The change specializes `SurfaceRules.TestRuleSource.apply(...)` when the
follow-up rule source is exactly `BlockRuleSource`: the generated runtime rule
now checks the same condition and returns the same `BlockState` directly,
instead of calling through `StateRule.tryApply(...)` for every surface-rule
sample. Codec shape, rule source data, condition ordering, and returned block
state are unchanged.

Focused benchmark:

```text
reports/surfacerules-testrule-state-bench.txt
old_state_rule_mostly_true_best_ms=51.347
new_state_rule_mostly_true_best_ms=50.095
mostly_true_speedup=1.025x
old_state_rule_mostly_false_best_ms=49.377
new_state_rule_mostly_false_best_ms=47.994
mostly_false_speedup=1.029x
equivalence=PASS
```

Validation on the rebuilt runtime passed:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=938d573e0bb9dc3816ac2b6464e264191d75ac576a9e02fce41f6801833c0d87
app-cds sha256=180cdde24eea942c37c5ed80aa93c39ccb415617790d1f97697ee43d317126f1
plugin matrix: PASS, Done (34.742s), 11 real plugins initialized
restart/recovery: PASS, Done (19.091s), Saved the game
forced-ticket persistence: PASS, first/restart Done 16.112s/11.165s
```

The strict 50-bot 32/32 load gate is still not claimable because host
preflight was blocked by unrelated background Java/Velocity load:

```text
reports/load-surfacerules-state-test-20260510-preflight.txt
host_preflight_ok=false
load_per_cpu=0.985
idle_percent_1s=55.32
```

Decision: keep the patch as a narrow worldgen surface-rule dispatch reduction
only. Do not claim end-to-end TPS/MSPT or 500-player improvement from it.

## Current 2026-05-10 11:48 CEST: chunk expire-count lookup now uses explicit get/putIfAbsent fast path

This cycle changed only
`ChunkHolderManager.addExpireCount(...)` in the Paper chunk-ticket path.
The previous `computeIfAbsent(...)` call on
`sectionToChunkToExpireCount` was replaced with an explicit `get(...)`
followed by atomic `putIfAbsent(...)` and a direct `addTo(...)` on the
returned `Long2IntOpenHashMap`. The observable ticket semantics and removal
path are unchanged.

Focused benchmark on the new lookup shape:

```text
reports/chunk-expire-count-bench.txt
dynamic_compute_hot_best_ms=333.257
dynamic_manual_hot_best_ms=277.137
dynamic_manual_hot_speedup=1.203x
dynamic_compute_cold_best_ms=0.566
dynamic_manual_cold_best_ms=0.478
dynamic_manual_cold_speedup=1.182x
equivalence=PASS
```

Validation on the rebuilt runtime passed:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
plugin matrix: PASS, Done (53.157s), 11 real plugins initialized
restart/recovery: PASS, Done (43.350s), Saved the game
forced-ticket persistence: PASS, first/restart Done 28.668s/22.220s
```

The strict 50-bot 32/32 load gate was not claimable because host preflight
was blocked by unrelated background load:

```text
reports/load-chunk-expire-lookup-20260510-preflight.txt
host_preflight_ok=false
load_per_cpu=1.809
idle_percent_1s=13.93
```

Decision: keep the `addExpireCount(...)` explicit lookup/insert path as a
narrow hot-path optimization only. Do not claim end-to-end TPS or 500-player
improvement from it. The next work should come from fresh profiling in the
movement/ticket or chunk-generation cluster, not from this exact map lookup.

## Current 2026-05-10 11:39 CEST: CompressionEncoder deflater-input candidate accepted as a narrow fallback-path optimization

This cycle added a new feature patch,
`upstream/Paper/paper-server/patches/features/0047-Optimize-CompressionEncoder-deflater-input.patch`,
which changes only the Java `Deflater` fallback in
`net.minecraft.network.CompressionEncoder`. The fallback now feeds
`Deflater.setInput(ByteBuffer)` from `ByteBuf.nioBuffer(...)` and skips the
source bytes in place instead of copying into a fresh `byte[]`. The Velocity
native compression path remains unchanged.

Focused benchmark on the committed patch stack:

```text
reports/compression-deflater-input-bench.txt
old_heap_best_ms=137.266
bytebuffer_heap_best_ms=131.327
bytebuffer_heap_speedup=1.045x
old_direct_best_ms=129.531
bytebuffer_direct_best_ms=124.865
bytebuffer_direct_speedup=1.037x
equivalence=PASS
```

Validation on the rebuilt runtime passed:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
plugin matrix: PASS, Done (51.284s), 11 real plugins initialized
restart/recovery: PASS, Done (24.522s), Saved the game
forced-ticket persistence: PASS, first/restart Done 20.692s/17.519s
```

The strict 50-bot 32/32 load gate is still not claimable here because the
host preflight was blocked by background Java load:

```text
reports/load-50bots-compression-deflater-bytebuffer-gate-20260510-v2-preflight.txt
host_preflight_ok=false
load_per_cpu=1.160
idle_percent_1s=41.32
```

Decision: keep the patch as a narrow compression-fallback optimization only.
Do not claim end-to-end boot, TPS, or 500-player improvement from it. The
next work should still come from fresh movement/ticket or chunk-generation
profiling, not from this compression path.

## Current 2026-05-10 10:42 CEST: NoiseInterpolator fraction-array candidate rejected; rollback runtime verified

This cycle tested a narrow `NoiseChunk.NoiseInterpolator.compute(...)`
candidate: precompute `cellWidth` / `cellHeight` interpolation fractions on
`NoiseChunk` and use array lookups inside the `fillingCell` `Mth.lerp3(...)`
path instead of three divisions per sample. The candidate preserved the same
fraction values and interpolation order.

Focused evidence was strong:

```text
reports/noise-interpolator-fractions-bench.txt
division_best_ms=29.308
array_fraction_best_ms=5.943
array_fraction_speedup=4.932x
equivalence=PASS
```

The temporary production build passed functional gates before load testing:
build/hash/json verification passed, the real 11-plugin matrix passed at
`Done (33.112s)`, restart/recovery passed at `Done (17.049s)`, and
forced-ticket persistence passed with first/restart `Done (14.103s)` /
`Done (9.155s)`.

The strict 50-bot 32/32 spectator gate rejected the candidate:

```text
reports/load-50bots-noiseinterp-fractions-gate-20260510-preflight.txt
host_preflight_ok=true
load_per_cpu=0.575
idle_percent_1s=58.53

reports/load-50bots-noiseinterp-fractions-gate-20260510-summary.txt
online_max=50
tps1_avg=16.75
avg_tick_ms_avg=63.54
loaded_chunks_max=2891
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=3
sync_load_stack_hits=0
nearby_players_stack_hits=7
stability_failures=0
```

Decision: rejected and rolled back. Generated `NoiseChunk.java` no longer
contains the fraction arrays or the candidate `buildFractions(...)` helper.
Post-rollback runtime verification:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applyPatches Applied 912 patches
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=421edbef592cb75b3e74fa2b1010f82fcc384512ba4773ada1dc78b6b52e28e0
app-cds sha256=737b5aae8ac745a693c2446845cff8efb7610032814ee54c17906ef8321e1899
remap-classpath sha256=799ab1c010b9f1a39806624db931911442fe34d9dde7462f803e82d19033d60f
plugin matrix: PASS, Done (29.035s), 11 real plugins initialized
restart/recovery: PASS, Done (18.063s), Saved the game
forced-ticket persistence: PASS, first/restart Done 16.174s/12.176s
```

The bench and report are kept as rejected evidence only. Do not retry this
`NoiseInterpolator` fraction-array shape without new profile evidence and a
server gate that beats the accepted `18.27 TPS / 47.85 ms / 2380 chunks`
reference. At this historical checkpoint, the 20 TPS / 500-bot / 32+32 target
was still not met; the current release state is superseded by the 2026-05-17
worker10/send60/gen20 production gate at the top of this file.

## Current 2026-05-10 10:07 CEST: NearbyPlayers limit64 and player-loader cached-manager candidates rejected; rollback runtime verified

This continuation tested two movement/ticket-pressure candidates and kept
neither in production.

`RegionizedPlayerChunkLoader.PlayerChunkLoaderData` briefly cached
`ChunkTaskScheduler` and `ChunkHolderManager` fields to avoid repeated
`((ChunkSystemServerLevel)this.world).moonrise$getChunkTaskScheduler()` chains
in ticket and chunk-load paths. Functional gates passed, but the strict
50-bot 32/32 spectator gate rejected it:

```text
reports/load-50bots-playerloader-cache-manager-20260510-summary.txt
online_max=50
tps1_avg=17.45
avg_tick_ms_avg=65.35
loaded_chunks_max=2412
watchdog_thread_dumps=4
sync_load_stack_hits=0
nearby_players_stack_hits=8
stability_failures=0
```

`NearbyPlayers.TrackedChunk.SPARSE_PLAYER_LIST_LINEAR_LIMIT` was then tested at
`64` on the same movement hot path. This also failed the accepted reference:

```text
reports/load-50bots-nearby-list-limit64-20260510-summary.txt
online_max=50
tps1_avg=16.90
avg_tick_ms_avg=88.49
loaded_chunks_max=2365
watchdog_thread_dumps=6
sync_load_stack_hits=0
nearby_players_stack_hits=4
stability_failures=0
```

Both candidates are rolled back. The production patch stack is again on the
baseline `NearbyPlayers.TrackedChunk` limit `2` and without the cached
player-loader manager fields. Final rollback verification:

```text
applyPatches: PASS, Applied 912 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=421edbef592cb75b3e74fa2b1010f82fcc384512ba4773ada1dc78b6b52e28e0
app-cds sha256=46b205c64a6131fda6dea3a1530d51a02e01a3e1a02541f0431bf52ef7daebbf
remap-classpath sha256=799ab1c010b9f1a39806624db931911442fe34d9dde7462f803e82d19033d60f
plugin matrix: PASS, Done (29.443s), 11 real plugins initialized
restart/recovery: PASS, Done (21.228s), Saved the game
forced-ticket persistence: PASS, first/restart Done 21.372s/11.272s
```

Two harness fixes were kept because they correct artifact validity rather than
gameplay behavior: `prepare_fast_runtime.sh` now invalidates remap/plugin/
reversed-mapping caches whenever `runtime.jar.sha256` changes, even if
`app-cds.jsa` is absent; `generate_app_cds.sh` now resolves the output
directory to an absolute path before passing `-XX:ArchiveClassesAtExit=...`.

Verdict: current runtime is buildable, runnable, plugin-matrix verified, and
restart-safe. At this historical checkpoint, the 20 TPS / 500-bot / 32+32
target was still not met and the latest accepted 50-bot reference remained
`18.27 TPS / 47.85 ms / 2380 chunks`; the current release state is superseded
by the 2026-05-17 worker10/send60/gen20 production gate at the top of this
file.
Do not retry the cached-manager, `NearbyPlayers` limit `64`, map-capacity,
or limit `3` shapes without new profile evidence.

## Current 2026-05-10 08:39 CEST: ProtoChunk heightmap candidate fully rolled back; current 50-bot gate not accepted

The temporary `ProtoChunk.setBlockState(...)` heightmap iterator-removal
candidate is no longer in the production patch stack. The feature patch
`upstream/Paper/paper-server/patches/features/0048-Optimize-ProtoChunk-heightmap-iterator-removal.patch`
was deleted, `applyPatches` now reports `Applied 912 patches`, and generated
`ProtoChunk.java` no longer contains `HEIGHTMAP_TYPES` or the cached
`Heightmap.Types[]` loops. The required Paper `getBlockState(int, int, int)`
override remains.

Fresh rollback verification:

```text
cd upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 912 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=8813443f40c8b7cf230287c9e0f03daa924b60cc45a16ef83ffd23ae3b4b9911
app-cds sha256=8a052f5a8e11579857e09a81b1d163006d1ae092427ebb8d65d8d7dab23d070f
plugin matrix: PASS, Done (26.859s), 11 real plugins initialized
restart/recovery: PASS, Done (16.028s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.244s/9.550s
```

Fresh strict 50-bot 32/32 spectator evidence on this rollback runtime:

```text
reports/load-50bots-protochunk-postrollback-20260510-preflight.txt
host_preflight_ok=true
load_per_cpu=0.546
idle_percent_1s=72.83

reports/load-50bots-protochunk-postrollback-20260510-summary.txt
online_max=50
tps1_avg=18.08
avg_tick_ms_avg=96.12
loaded_chunks_max=2609
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=3
sync_load_stack_hits=0
nearby_players_stack_hits=0
stability_failures=0
```

Verdict: rollback is verified, but the current 50-bot 32/32 run is not an
accepted load baseline. It is below the accepted `18.27 TPS / 47.85 ms / 2380`
reference on TPS/MSPT and has watchdog dumps. Thread dumps point at movement
ticket churn (`ChunkHolderManager.addExpireCount`,
`RegionizedPlayerChunkLoader.flushDelayedTicketOps`, `ReferenceList.remove`)
and worker-side chunk-generation noise (`PerlinNoise`, `NoiseChunk`,
`RangeChoice`). Next work should target one of those paths with a focused
bench first; do not retry the same `ProtoChunk` heightmap iterator shape.

## Current 2026-05-10 08:05 CEST: marker-cache load gate failed promotion; climate distance branch rejected

The existing `NoiseChunk` marker-wrapper cache remains only a narrow allocation
win. A clean strict 50-bot 32/32 spectator gate finally passed preflight, but
did not qualify as an accepted load result:

```text
reports/load-50bots-marker-cache-clean-gate-20260510-preflight.txt
host_preflight_ok=true
load_per_cpu=0.611
idle_percent_1s=61.15

reports/load-50bots-marker-cache-clean-gate-20260510-summary.txt
online_max=50
tps1_avg=18.72
avg_tick_ms_avg=42.07
loaded_chunks_max=1806
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=3
sync_load_stack_hits=0
nearby_players_stack_hits=4
stability_failures=0
```

The TPS/MSPT numbers improved, but the gate fails the stability/coverage
standard because of 3 watchdog thread dumps and lower chunk coverage than the
accepted `18.27/47.85/2380` reference. Jstacks point at movement-pressure
paths: `NearbyPlayers`, `RegionizedPlayerChunkLoader.flushDelayedTicketOps`,
and `WaypointTransmitter.EntityChunkConnection.update`.

`Climate.Parameter.distance(...)` branch rewrites were also rejected at
microbench stage:

```text
reports/climate-parameter-distance-bench.txt
old_distance_best_ms=194.276
branch_distance_best_ms=202.207
branch_distance_speedup=0.961x
subtract_first_distance_best_ms=195.008
subtract_first_speedup=0.996x
equivalence=PASS
```

No production source was changed for the climate-distance branch rewrite.
Next comparable gate candidate is the already-built `ProtoChunk` heightmap
iterator-removal patch, but it must wait for a host preflight window.

## Current 2026-05-10 07:57 CEST: OreFeature loop cleanup rejected and removed

`OreFeature.doPlace(...)` loop cleanup was tested as a small worldgen
candidate: reuse `d5 * d5`, reuse `d5 * d5 + d6 * d6`, and precompute
`width * height` for the bitset index. The previous focused benchmark was
positive (`60.507 ms -> 58.403 ms`, `1.036x`, equivalence PASS), but the real
strict 50-bot 32/32 spectator gate rejected it:

```text
reports/load-50bots-orefeature-loop-gate-20260510-preflight.txt
host_preflight_ok=true
load_per_cpu=0.282
idle_percent_1s=75.25

reports/load-50bots-orefeature-loop-gate-20260510-summary.txt
online_max=50
tps1_avg=18.27
avg_tick_ms_avg=65.21
loaded_chunks_max=2911
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=2
sync_load_stack_hits=0
nearby_players_stack_hits=4
stability_failures=0
```

The source patch
`paper-server/patches/sources/net/minecraft/world/level/levelgen/feature/OreFeature.java.patch`
was deleted. A direct grep confirms the candidate variables
`widthHeight`, `d5Squared`, and `d5d6Squared` no longer exist in generated
`OreFeature.java`.

Rollback verification on the rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, Applied 912 patches
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
optimized artifact sha256=b3d568056545f7d8ca3d7a0cb3b2df205d8b5f2d8483759c292121e344c80427
app-cds sha256=41a6626fc776e505fc14b6860dd56a5a3322c691d8f7c09a26c6b418b8705201
plugin matrix: PASS, Done (26.953s), 11 real plugins initialized
restart/recovery: PASS, Done (17.037s), Saved the game
forced-ticket persistence: PASS, first/restart Done 12.862s/8.382s
```

Verdict: rejected and removed from production. Do not retry this same
`OreFeature` scalar-hoist shape without new profile evidence explaining the
watchdog/MSPT regression. The 20 TPS / 500-bot / 32+32 target is still not
met.

## Current 2026-05-10 06:39 CEST: Waypoint chunk-key candidate rejected, rollback baseline refreshed

## Current 2026-05-10 07:40 CEST: RangeChoice constant-out candidate rejected and removed

`DensityFunctions.RangeChoice.fillArray(...)` was tested with a
constant-`whenOutOfRange` specialization. The focused benchmark was positive
for mixed/constant branch shapes:

```text
reports/range-choice-bench.txt
in_constant_out_dynamic: 9.947 ms -> 9.124 ms, 1.090x
in_dynamic_out_constant: 9.977 ms -> 9.507 ms, 1.049x
both_constant: 10.004 ms -> 7.321 ms, 1.366x
both_dynamic: 10.501 ms -> 10.742 ms, 0.978x
equivalence=PASS
```

The clean strict 50-bot 32/32 spectator gate rejected it:

```text
reports/load-50bots-rangechoice-constant-out-gate-20260510-summary.txt
host_preflight_ok=true
worker_line=[07:25:05 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 1 I/O threads
online_max=50
tps1_avg=17.63
avg_tick_ms_avg=192.39
loaded_chunks_max=2768
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=5
sync_load_stack_hits=0
nearby_players_stack_hits=4
stability_failures=0
```

The candidate patch
`paper-server/patches/features/0041-Optimize-RangeChoice-constant-out-fillArray.patch`
was removed. `applyPatches` confirms no `RangeChoiceConstantOut` or
`rangeChoiceLike` code remains in generated source or patches.

Rollback verification on the rebuilt runtime:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=f903772a4889c1b6a9b346b118b8a9ec250d4e6a2a44d45017422cd6db5ce9e7
app-cds sha256=830b21687a877c0b9fee51e0694c262afc00314ab0236e0adb93ae2f24c4dac9
plugin matrix: PASS, Done (26.927s), 11 real plugins initialized
restart/recovery: PASS, Done (16.461s), Saved the game
forced-ticket persistence: PASS, first/restart Done 12.714s/8.332s
```

Verdict: rejected and removed from production. Do not retry this same
`RangeChoiceConstantOut` shape without new profile evidence explaining the
watchdog/nearby-player regression. The project target is still not met: no
stable 20 TPS 50-bot result, no 500-bot result, and no vanilla parity claim.

`WaypointTransmitter.EntityChunkConnection.update()` was temporarily tested
with `chunkPos.longKey != this.lastChunkKey` instead of the original
`chunkPos.getChessboardDistance(this.lastPosition) > 0` condition. The
standalone loop looked positive:

```text
reports/waypoint-chunk-update-bench.txt
distance_best_ms=80.686
long_key_best_ms=34.099
long_key_speedup=2.366x
equivalence=PASS
```

The strict 50-bot 32/32 spectator gate rejected it:

```text
reports/load-waypoint-chunkkey-update-20260510-summary.txt
worker_line=[06:15:52 INFO]: [MoonriseCommon] Paper is using 6 worker threads, 1 I/O threads
online_max=50
tps1_avg=17.99
avg_tick_ms_avg=63.66
loaded_chunks_max=2516
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

The production source and source patch were rolled back to the accepted
chunk-distance update condition while keeping the already accepted
`lastChunkKey` cache for `isBroken()` chunk-visibility checks.

Rollback verification:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=33a22c0e172bd2dc839cddccb46d2b843042be035e04277f03e2f92512047fbc
app-cds sha256=702e108e49bfae54c86b3fdca275f6297c58104dd9815cbb6ef7bdc29b28a5a2
plugin matrix: PASS, Done (27.799s), 11 real plugins initialized
restart/recovery: PASS, Done (16.968s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.274s/8.602s
```

Fresh post-rollback strict baseline on the same 6-worker pinned-cpuset shape:

```text
reports/load-50bots-post-rollback-baseline-20260510-summary.txt
worker_line=[06:32:07 INFO]: [MoonriseCommon] Paper is using 6 worker threads, 1 I/O threads
online_max=50
tps1_avg=18.29
avg_tick_ms_avg=50.90
loaded_chunks_max=2441
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Verdict: the waypoint chunk-key update-condition candidate is rejected and
not in production. The rollback runtime is buildable, compatibility-verified,
and stable at 50 bots / 32/32 in this run, but the project target is still not
met: no stable 20 TPS 50-bot result, no 500-bot result, and no vanilla parity
claim.

## Current 2026-05-10 05:47 CEST: NearbyPlayers limit=3 and chunkTicketStage pre-sizing rejected and rolled back

This cycle tested two narrow movement/ticket hot-path candidates and kept
neither of them in production.

`NearbyPlayers.TrackedChunk.SPARSE_PLAYER_LIST_LINEAR_LIMIT` was raised from
`2` to `3` and validated with two strict 50-bot 32/32 spectator runs. The
first run stayed stable but did not beat the accepted reference, and the
rerun regressed further:

```text
reports/load-50bots-referencelist-linear3-gate-20260510-summary.txt
online_max=50
tps1_avg=18.06
avg_tick_ms_avg=46.77
loaded_chunks_max=2396
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0

reports/load-50bots-referencelist-linear3-rerun-20260510-summary.txt
online_max=50
tps1_avg=17.83
avg_tick_ms_avg=62.80
loaded_chunks_max=2427
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

In the same cycle, `PlayerChunkLoaderData.chunkTicketStage` pre-sizing to
`new Long2ByteOpenHashMap(4096, 0.6F)` was rejected before production. The
focused bench regressed on both read and mutation churn:

```text
reports/chunk-ticket-stage-map-bench.txt
default_get_best_ms=176.812
lowload_get_best_ms=195.766
lowload_get_speedup=0.903x
default_mutation_best_ms=15.825
lowload_mutation_best_ms=16.091
lowload_mutation_speedup=0.983x
equivalence=PASS
```

Rollback verification on the restored `limit=2` runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=4b53093957a73270684ed764da39bb17ae69cd207ac057a19c2a5e728e82e450
app-cds sha256=c7fc590b53cb34477c779b007169dad2e8d29c31ee179e85bc6ce36505f3b8f0
plugin matrix: PASS, Done (27.251s), 11 real plugins initialized
restart/recovery: PASS, Done (15.819s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.055s/8.863s
```

Verdict: both candidates are rejected. The current runtime is back on
`NearbyPlayers.TrackedChunk` limit `2`, and no stable 20 TPS / 500-player /
32-view-distance claim is made.

## Current 2026-05-10 03:40 CEST: ReferenceList sparse transition remove kept as a narrow movement hot-path reduction

`ReferenceList.remove(...)` now has a bounded transition fast path for tiny
linear-search lists. When a list with `linearSearchLimit <= 4` drops from
`limit + 1` entries back into linear mode, it scans the small backing array,
removes by swap-with-end, clears the hash index, and avoids the
`Reference2IntOpenHashMap.removeInt/shiftKeys` path. The only current
production user with this mode is `NearbyPlayers.TrackedChunk` player lists
with limit `2`. No event order, scheduler behavior, permissions/services,
entity mutation semantics, or plugin classloading/remap semantics are changed.

Focused evidence:

```text
reports/reference-list-transition-remove-bench.txt
baseline_transition_best_ms=449.432
candidate_transition_best_ms=324.509
candidate_transition_speedup=1.385x
baseline_miss_best_ms=48.443
candidate_miss_best_ms=13.181
candidate_miss_speedup=3.675x
baseline_dense_best_ms=306.302
candidate_dense_best_ms=307.813
candidate_dense_speedup=0.995x
equivalence=PASS
```

Production verification:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applyPatches/build/runtime generation complete
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=e01176c27f59f0bd92d3a0ea7d884e692f7f07fe9e1945ad42ac5bdf26fa1a7e
app-cds sha256=8f2df11632803bb2325c3865c35085317984fb261623abd1b5e00351b8f56778
plugin matrix: PASS, Done (26.747s), 11 real plugins initialized, precomputed remaps used
restart/recovery: PASS, Done (16.102s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.346s/8.585s
```

Strict 50-bot 32/32 spectator gate:

```text
reports/load-50bots-reference-list-transition-remove-gate-20260510-summary.txt
online_max=50
tps1_avg=18.07
avg_tick_ms_avg=51.73
loaded_chunks_max=2782
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=3
sync_load_stack_hits=0
nearby_players_stack_hits=0
stability_failures=0
```

Verdict: keep only as a narrow sparse-list movement hot-path reduction. It
removed `NearbyPlayers/ReferenceList` from the watchdog stacks in this run,
but the run still has watchdog dumps and does not beat the accepted reference
around `18.27 TPS / 47.85 ms / 2380 chunks`. No stable 20 TPS, 500-player,
sub-second cold boot, all-plugin support, or vanilla-parity claim is made.

## Current 2026-05-10 02:26 CEST: NearbyPlayers map-capacity candidate rejected and rolled back

`NearbyPlayers` player/player-state map pre-sizing was tested as a temporary
candidate to avoid first-join `Reference2ReferenceOpenHashMap` rehashes under
large player counts. The standalone benchmark was positive, but the real
strict 50-bot 32/32 spectator gate did not beat the accepted load reference,
so the production patch was removed.

Focused evidence from `reports/nearby-player-map-bench.txt`:

```text
50 players: 199.097 ms -> 88.669 ms, 2.245x, rehashes 4.000 -> 0.000
500 players: 339.920 ms -> 139.127 ms, 2.443x, rehashes 10.000 -> 0.000
equivalence=PASS
```

Rejected real-load evidence from
`reports/load-50bots-nearby-map-capacity-gate-20260510-summary.txt`:

```text
host_preflight_ok=true
online_max=50
tps1_avg=17.95
avg_tick_ms_avg=52.03
loaded_chunks_max=2059
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

That is worse than the accepted reference around
`18.27 TPS / 47.85 ms / 2380 chunks`, so no performance win is claimed.

Current rollback runtime verification:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=a7e95bd2da35771fce15c9f322b6ab3aeca902967bd124e3cc99aaca7487d941
plugin matrix: PASS, Done (26.143s), 11 real plugins initialized, precomputed remaps used
restart/recovery: PASS, Done (16.191s), Saved the game
forced-ticket persistence: PASS, first/restart Done 12.884s/9.473s
```

Verdict: candidate rejected and rolled back. Current artifact is buildable and
compatibility-verified, but the 20 TPS / 500-player / 32-view-distance target
is still unmet.

## Current 2026-05-10 01:09 CEST: ProtoChunk heightmap iterator cleanup kept without load-baseline win

`ProtoChunk.setBlockState(...)` now uses a cached
`Heightmap.Types[]` traversal and `EnumSet.contains(...)` instead of
allocating `EnumSet` iterators in the two heightmap update scans. The persisted
heightmap set is unchanged, update order follows `Heightmap.Types.values()`,
and no plugin-visible world, scheduler, event, or entity semantics are changed.

Focused evidence:

```text
reports/protochunk-heightmap-bench.txt
old_enumset_foreach_best_ms=133.632
new_cached_values_contains_best_ms=100.017
new_speedup=1.336x
old_iterator_allocations_per_setblock=2
new_iterator_allocations_per_setblock=0
equivalence=PASS
```

Production verification:

```text
patch=upstream/Paper/paper-server/patches/features/0048-Optimize-ProtoChunk-heightmap-iterator-removal.patch
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, includes applyPatches
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=5bd64433892e0656586c1163b681e6bb5e184dd55ac55865b3a6abd6b77d5dca
remap-classpath hash=65448F2905E5954D33593C9908926782954CD79B5A3D4D95B29748AB5B363882
plugin matrix: PASS, Done (27.655s), 11 real plugins initialized
restart/recovery: PASS, Done (15.839s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.433s/8.960s
```

Strict 50-bot 32/32 spectator runs were stable but did not beat the accepted
load reference around `18.27 TPS / 47.85 ms / 2380 chunks`:

```text
reports/load-50bots-protochunk-heightmap-restored-gate-20260510-summary.txt
online_max=50
tps1_avg=18.51
avg_tick_ms_avg=54.42
loaded_chunks_max=2217
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0

reports/load-50bots-protochunk-heightmap-rerun-20260510-summary.txt
online_max=50
tps1_avg=17.84
avg_tick_ms_avg=46.13
loaded_chunks_max=2215
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Verdict: keep only as a narrow heightmap/setBlock allocation and work
reduction with compatibility gates. Do not claim a 50-bot TPS improvement,
stable 20 TPS, 500 players, sub-second boot, all-plugin support, or vanilla
parity from this patch.

## Current 2026-05-09 23:56 CEST: Climate RTree search pruning kept with strict-load blocker

`Climate.RTree.SubTree.search(...)` now computes the default child
distance with the current best distance as a bound. The exact `distance(...)`
method is still present for callers that need the full value, and the custom
`DistanceMetric` search path is unchanged. If partial squared-distance
accumulation reaches the current best, that child cannot win, so the method
returns the bound and preserves the same branch decision.

Focused evidence:

```text
reports/climate-rtree-search-bench.txt
current_random_best_ms=2068.513
bounded_random_best_ms=1856.135
bounded_random_speedup=1.114x
current_walk_best_ms=544.258
bounded_walk_best_ms=450.860
bounded_walk_speedup=1.207x
equivalence=PASS
```

Production verification:

```text
patch=paper-server/patches/features/0047-Prune-climate-RTree-search-by-current-best-distance.patch
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 913 patches, Saved modified patches (44/47)
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, includes applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=2943cacf3d945cbb7e49e739d295d2789444b8b25c65762b72f22bdcacb09aec
remap-classpath hash=65448F2905E5954D33593C9908926782954CD79B5A3D4D95B29748AB5B363882
plugin matrix: PASS, Done (30.298s)
restart/recovery: PASS, Done (22.644s), Saved the game
forced-ticket persistence: PASS, first/restart Done 18.153s/18.919s
```

The strict 50-bot 32/32 spectator gate is currently blocked before Minecraft
start by host load:

```text
reports/load-50bots-rtree-search-prune-gate-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=1.657
idle_percent_1s=33.05
max_load_per_cpu=0.750
min_idle_percent=40.00
```

A forced noisy diagnostic reached all 50 bots and found no kicks/errors,
watchdog dumps, sync-load stacks, or stability failures, but it is not
comparable performance evidence:

```text
reports/load-50bots-rtree-search-prune-noisy-20260509-summary.txt
online_max=50
tps1_avg=17.23
avg_tick_ms_avg=58.78
loaded_chunks_max=1750
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Verdict: keep as a small default-biome-search work reduction with compatibility
gates. Do not claim a 50-bot TPS improvement: strict load is blocked on the
busy host and the 20 TPS / 500-player target is still unmet.

## Current 2026-05-09 23:36 CEST: Carver iteration cleanup kept with strict-load blocker

`BiomeGenerationSettings.getCarvers()` now returns the underlying `HolderSet`
instead of hiding it behind `Iterable`, and
`NoiseBasedChunkGenerator.applyCarvers(...)` uses `size()/get(i)` rather than
allocating an iterator in the 17x17 neighbor-chunk carving loop. The loop
order, `i3` value, and `setLargeFeatureSeed(seed + i3, ...)` semantics are
unchanged.

Focused evidence:

```text
reports/carver-iteration-bench.txt
foreach_best_ms=124.919
indexed_best_ms=85.075
indexed_speedup=1.468x
foreach_allocated_bytes_per_iteration=32.000
indexed_allocated_bytes_per_iteration=0.000
saved_allocated_bytes_per_iteration=32.000
equivalence=PASS
```

Production verification:

```text
patch=paper-server/patches/features/0046-Optimize-carver-iteration-in-chunk-generation.patch
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 913 patches, Saved modified patches (43/46)
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=c68f94185320568b687ed876a31b6da28ac5a47c19b3587de4deb9bfabf164fd
remap-classpath hash=65448F2905E5954D33593C9908926782954CD79B5A3D4D95B29748AB5B363882
plugin matrix: PASS, Done (31.900s)
restart/recovery: PASS, Done (25.501s), Saved the game
forced-ticket persistence: PASS, first/restart Done 15.097s/10.702s
```

The strict 50-bot 32/32 spectator gate is currently blocked before Minecraft
start by host load:

```text
reports/load-50bots-carver-iteration-gate-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=0.962
idle_percent_1s=27.15
max_load_per_cpu=0.750
min_idle_percent=40.00
```

A forced noisy diagnostic reached all 50 bots and found no kicks/errors,
watchdog dumps, sync-load stacks, or stability failures, but it is not
comparable performance evidence:

```text
reports/load-50bots-carver-iteration-noisy-20260509-summary.txt
online_max=50
tps1_avg=17.24
avg_tick_ms_avg=95.80
loaded_chunks_max=1824
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Verdict: keep as a small chunk-generation allocation/work reduction with
compatibility gates. Do not claim a 50-bot TPS improvement: strict load is
blocked on the busy host and the 20 TPS / 500-player target is still unmet.

## Current 2026-05-09 23:12 CEST: Climate RTree build allocation cleanup kept with load limits

`Climate.RTree.build(...)` now avoids the recursive stream/collector path when
rebuilding chosen subtrees, pre-sizes the `bucketize(...)` output buckets, and
seeds `buildParameterSpace(...)` from the first child instead of filling a
temporary list with `null` and spanning the first child back into it. This is
kept only as a narrow startup/tree-build allocation reduction in
`paper-server/patches/features/0045-Optimize-Climate-RTree-build-allocation.patch`.

Focused evidence:

```text
reports/climate-rtree-build-bench.txt
current_stream_build_best_ms=543.404
optimized_loop_build_best_ms=530.904
optimized_speedup=1.024x
current_allocated_bytes_per_build=10021748.6
optimized_allocated_bytes_per_build=9685848.0
saved_allocated_bytes_per_build=335900.6
equivalence=PASS
```

Production verification:

```text
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 913 patches, Saved modified patches (42/45)
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=b5b21e47b38b9f7aab4955f4465568198df9c9bb2fa082da2f86acc655c96265
remap-classpath hash=FF1D0EFF6D4A4E714D9DA6F20AD5C76998531BE03019033E0647C5C3BC1E5BEB
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (30.428s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (20.358s), Saved the game
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.099s/9.797s
```

Strict 50-bot 32/32 spectator gate ran on a clean host preflight and stayed
stable, but it did not meet the project goal or beat the accepted load
reference:

```text
reports/load-50bots-climate-rtree-build-gate-20260509-preflight.txt
host_preflight_ok=true
load_per_cpu=0.670
idle_percent_1s=77.59

reports/load-50bots-climate-rtree-build-gate-20260509-summary.txt
online_max=50
tps1_avg=18.04
avg_tick_ms_avg=56.39
loaded_chunks_max=2429
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Verdict: keep as a small tree-build allocation/work reduction because build,
plugin, restart, persistence, hash, and focused equivalence gates passed. Do
not claim a 50-bot TPS improvement: accepted load reference remains about
`18.27 TPS / 47.85 ms / 2380 chunks`, and the 20 TPS / 500-player target is
still unmet.

## Current 2026-05-09 22:38 CEST: YClamped/waypoint micro-candidates rejected, density hooks remain current

No new production optimization was kept in this continuation. The current
runnable artifact remains on the previously accepted density visitor hook
state. Two small follow-up candidates were measured and rejected before being
kept:

```text
reports/waypoint-snapshot-bench.txt
sizedArray_speedup=0.782x
equivalence=PASS
verdict=REJECTED, slower than current toArray snapshot path

reports/yclamped-gradient-bench.txt
current_clamped_map_best_ms=25.894
optimized_inline_best_ms=26.244
optimized_speedup=0.987x
equivalence=PASS
verdict=REJECTED_AND_ROLLED_BACK
```

The temporary `YClampedGradient.compute(...)` inline hunk was removed from the
source patch, `applyPatches` re-applied cleanly, and bytecode is back to
calling `Mth.clampedMap(...)`.

Current artifact verification:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=6bd43f336ea79bf81aa594fa5ac1223315641912ad5e24f7f38e8a75b801667f
app-cds sha256=82adb3ed68f5a86d1dd7e956bb234c5aa0a83933c153006e3e006f27e984ac1a
remap-classpath sha256=c85057d8a6663b15d1e49c00220da852a4596369a2168df61aa145b6b571c292
plugin matrix: PASS, Done (35.098s)
restart/recovery: PASS, Done (25.464s), Saved the game
forced-ticket persistence: PASS, first/restart Done 15.876s/11.482s
```

Strict 50-bot 32/32 spectator gate is still blocked before Minecraft startup
by host load:

```text
reports/load-50bots-post-yclamped-reject-gate-20260509-preflight.txt
host_preflight_ok=false
load1=15.76
load_per_cpu=1.313
idle_percent_1s=58.97
max_load_per_cpu=0.750
```

No 20 TPS stable, end-to-end TPS/MSPT, sub-second boot, or 500-player claim is
made from this cycle.

## Current 2026-05-09 22:06 CEST: Density visitor hooks wired into production path

The existing `DensityFunction.Visitor.applyHolder(...)` and
`applyMarker(...)` hooks are now actually used by
`DensityFunctions.HolderHolder.mapAll(...)` and
`DensityFunctions.MarkerOrMarked.mapAll(...)`. Generic visitor behavior is
unchanged because the default hook implementations still create the same
wrapper objects and call `apply(...)`. The optimized `NoiseChunk` and
`RandomState` visitors already override these hooks, so this removes temporary
`HolderHolder` / `Marker` wrappers in those worldgen visitor paths instead of
allocating objects that are immediately unwrapped again.

Focused evidence:

```text
reports/density-visitor-hooks-bench.txt
old_best_ms=504.111
hooked_best_ms=21.346
hooked_speedup=23.617x
old_temp_holder_allocations=3072000
old_temp_marker_allocations=3072000
hooked_temp_holder_allocations=0
hooked_temp_marker_allocations=0
equivalence=PASS
```

Production verification:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=10f53b1c4dc35cb7b8c0e3c62ce8d440cfa4c9afd0f42fbb44706088a487892b
app-cds sha256=0019f9d05b603893714bc075e4e30d67df9612cd6872e4d5aab6d1954444cfdc
remap-classpath sha256=8dfc8d050fcda82b271014c38c89b8395fa7faa8670813494bc234ad7de11a7a
javap HolderHolder.mapAll: invokes DensityFunction$Visitor.applyHolder
javap MarkerOrMarked.mapAll: invokes DensityFunction$Visitor.applyMarker
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (38.265s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (29.923s), Saved the game
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 21.751s/15.859s
```

Strict 50-bot 32/32 spectator gate is blocked before Minecraft startup by host
load, so this cycle makes no end-to-end TPS/MSPT, 20 TPS, or 500-player claim:

```text
reports/load-50bots-density-visitor-hooks-gate-20260509-preflight.txt
host_preflight_ok=false
load1=20.15
load_per_cpu=1.679
idle_percent_1s=19.21
max_load_per_cpu=0.750
```

Verdict: keep as a semantics-preserving production allocation/work reduction
with build/plugin/restart/persistence evidence. Load-performance proof remains
blocked on a clean host window.

## Current 2026-05-09 21:36 CEST: Jigsaw target-first and waypoint distance guards rejected

`JigsawBlock.canAttach(...)` target-first evaluation was built and
compatibility-gated, but the clean strict 50-bot 32/32 spectator gate regressed
against the accepted reference. The production patch was rolled back to the
previous orientation-first `FrontAndTop` shape.

Rejected Jigsaw evidence:

```text
reports/jigsaw-canattach-bench.txt
old_can_attach_best_ms=1119.278
optimized_can_attach_best_ms=860.110
target_first_can_attach_best_ms=90.603
optimized_speedup=1.301x
target_first_speedup=12.354x
equivalence=PASS

reports/load-50bots-jigsaw-targetfirst-gate-20260509-rerun1-preflight.txt
host_preflight_ok=true
load_per_cpu=0.433

reports/load-50bots-jigsaw-targetfirst-gate-20260509-rerun1-summary.txt
online_max=50
loaded_chunks_max=1540
tps1_avg=17.28
avg_tick_ms_avg=276.57
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
stability_failures=0
```

Rollback runtime verification:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=9ec8accb68af16ab4b4aef208a937668bea39aaefd50fdb6f0d7d2b808a826ea
app-cds sha256=13ba91c686442eefb93b0e8c82837e5547f61259a0db98358ea9107d21c91b6c
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (31.794s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (22.176s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 20.284s/13.382s
```

A follow-up `WaypointTransmitter` range/`isReallyFar(...)` distance-guard
candidate was also rejected before production because the focused benchmark was
slower even though equivalence passed:

```text
reports/waypoint-distance-guard-bench.txt
old_range_best_ms=157.593
guarded_range_best_ms=177.407
guarded_range_speedup=0.888x
old_really_far_best_ms=95.283
guarded_really_far_best_ms=108.328
guarded_really_far_speedup=0.880x
equivalence=PASS
```

The latest post-rollback strict 50-bot 32/32 control is blocked before
Minecraft startup by host load:

```text
reports/load-50bots-jigsaw-targetfirst-postrollback-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=1.243
max_load_per_cpu=0.750
```

Verdict: no new production performance win from this continuation. Current
runtime remains on the previous orientation-first Jigsaw shape; no 20 TPS
stable claim and no 500-player claim are made.

## Current 2026-05-09 20:51 CEST: DensityFunctions Ap2 ADD scratch rejected and rolled back

The candidate briefly reused a per-thread scratch array for `ADD` temporary
values. It kept an `inUse` fallback so nested or reentrant `ADD` calls still
received a fresh array instead of clobbering the outer calculation. This was
intended to reduce worldgen allocation pressure without changing
density-function results.

The focused benchmark was positive, but the clean real 50-bot 32/32 spectator
gate did not beat the accepted reference, so the production patch was rolled
back. The patch layer no longer contains `AddScratch`, `ADD_SCRATCH`, or the
scratch acquire/release path.

Focused evidence:

```text
reports/density-ap2-fill-bench.txt
old_flat_best_ms=1061.005
scratch_flat_best_ms=300.083
flat_speedup=3.536x
old_nested_best_ms=2066.358
scratch_nested_best_ms=1314.058
nested_speedup=1.573x
old_flat_allocated_bytes=3278432784
scratch_flat_allocated_bytes=32784
flat_saved_allocated_bytes=3278400000
old_nested_allocated_bytes=6556832784
scratch_nested_allocated_bytes=3278432784
nested_saved_allocated_bytes=3278400000
equivalence=PASS
reentrant_equivalence=PASS
```

Rejected real gate:

```text
reports/load-50bots-density-ap2-fill-gate-20260509-rerun1-preflight.txt
host_preflight_ok=true
load_per_cpu=0.471

reports/load-50bots-density-ap2-fill-gate-20260509-rerun1-summary.txt
online_max=50
loaded_chunks_max=1933
tps1_avg=17.75
avg_tick_ms_avg=78.14
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
stability_failures=0
```

Verdict: rejected. The candidate was stable, but worse than the accepted
reference line around `18.27 TPS / 47.85 ms / 2380 chunks`.

Rollback artifact verification:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=24a5b2132f4d6e77f81cf94c9089ae0767e271243f55d5dacdf35fb04df44395
app-cds sha256=a2a323ff68d96b5d280607ea65ae136fc4575b520d89a30fa2875e35c9a74af1
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (39.577s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (21.865s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 18.679s/11.969s
```

The post-rollback strict 50-bot control was blocked before Minecraft startup
by host preflight (`load_per_cpu=1.185` > `0.750`). No end-to-end TPS/MSPT win,
no 20 TPS stable claim, and no 500-player claim are made from this cycle.

## Current 2026-05-09 20:12 CEST: Entity bounding-box shortcut rejected and rolled back

`Entity.setPosRaw(...)` briefly used a direct dimensions-based bounding-box
path for normal entities, with explicit custom-box opt-outs for `Interaction`,
`Shulker`, and `AbstractWindCharge`. The focused benchmark was positive, but
the real 50-bot 32/32 spectator gate did not beat the accepted baseline, so
the production patch was rolled back. The patch layer no longer contains
`usesCustomBoundingBoxForSetPos()` or `setBoundingBoxFromDimensions(...)`.

Focused evidence:

```text
reports/entity-bounding-box-bench.txt
old_make_then_set_best_ms=748.115
direct_dimensions_set_best_ms=525.432
direct_dimensions_speedup=1.424x
old_allocated_bytes=1536000000
direct_allocated_bytes=768000000
equivalence=PASS
```

Rejected real gate:

```text
reports/load-50bots-entity-bbox-direct-gate-20260509-preflight.txt
host_preflight_ok=true
load_per_cpu=0.625

reports/load-50bots-entity-bbox-direct-gate-20260509-summary.txt
online_max=50
loaded_chunks_max=1721
tps1_avg=17.58
avg_tick_ms_avg=67.63
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Verdict: rejected. The candidate was stable, but it failed the accepted
reference line around `18.27 TPS / 47.85 ms / 2380 chunks`.

Rollback artifact verification:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 913 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
optimized artifact sha256=6411420229730d84c5b6b1f91a602043b17bafe07252ec65fd0649c5770538ae
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (30.454s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (19.537s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 19.076s/11.778s
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 18:45 CEST: ReferenceList smallMode candidate rejected and rolled back

`ReferenceList` briefly gained an explicit `smallMode` state to avoid repeated
hash-map churn around the linear-search threshold, but the candidate was
rejected after live verification. The focused microbench was positive for
single/pair churn, yet the noisy 50-bot diagnostic had
`watchdog_thread_dumps=6` and `loaded_chunks_max=824`, and the strict 50-bot
gate was blocked by busy-host preflight (`load_per_cpu=0.840`).

The runtime is now back on the baseline `ReferenceList` threshold-2 shape
without the rejected `smallMode` field. Rebuilt artifact gates on the restored
runtime pass:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (32.115s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (19.754s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 15.532s/9.772s
```

The latest noisy 50-bot run on the restored baseline is still diagnostic-only:

```text
reports/load-50bots-referencelist-smallmode-state-noisy-20260509-summary.txt
online_max=50
loaded_chunks_max=824
tps1_avg=18.50
avg_tick_ms_avg=35.14
watchdog_thread_dumps=6
nearby_players_stack_hits=13
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 18:10 CEST: POI main-thread scheduling fix landed; waypoint skip remains provisional

`ServerLevel.updatePOIOnBlockStateChange(...)` now routes POI mutations
through `runPoiUpdateOnServerThread(...)`, which runs inline only on the real
server thread and otherwise uses `scheduleOnMain(...)`. That closes the
off-main POI crash exposed by the noisy 50-bot run.

The `ServerWaypointManager` complete-row skip candidate is still in the tree,
but the strict 50-bot 32/32 gate remains blocked by busy-host preflight
(`load_per_cpu=1.824`), so no accepted load win is claimed from it.

Fresh gates on the rebuilt artifact pass:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (57.490s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (44.201s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 32.221s/20.055s
```

The noisy 50-bot run is now stable with zero thread-check, off-main POI,
chunk-system, feature-placement, or stability failures:

```text
reports/load-50bots-poi-mainthread-noisy-20260509-summary.txt
online_max=50
loaded_chunks_max=1796
tps1_avg=17.84
avg_tick_ms_avg=224.77
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 15:55 CEST: waypoint snapshot candidate rejected, baseline runtime restored

`ServerWaypointManager.snapshotEntries(...)` is back to the compatible baseline
implementation:

```text
return map.entrySet().toArray(Entry[]::new);
```

The manual-copy snapshot candidate was not kept. Its focused benchmark was
positive, but the real 50-bot 32/32 spectator gate failed the accepted
reference and produced watchdog thread dumps:

```text
reports/waypoint-snapshot-bench.txt
toArray_best_ms=795.043
manual_best_ms=489.372
manual_speedup=1.625x
equivalence=PASS

reports/load-50bots-waypoint-snapshot-manual-gate-20260509-1544-summary.txt
online_max=50
tps1_avg=17.74
avg_tick_ms_avg=37.32
loaded_chunks_max=2077
watchdog_thread_dumps=3
nearby_players_stack_hits=8
sync_load_stack_hits=0
bot_errors_max=0
```

Ticket-side candidate benches were also rejected before production:

```text
reports/ticketset-search-bench.txt
binary_best_ms=856.032
unchecked_binary_speedup=0.966x
linear4_speedup=0.945x
linear8_speedup=0.959x
linear12_speedup=0.973x
equivalence=PASS

reports/ticket-compare-bench.txt
old_best_ms=168.504
cached_best_ms=169.166
cached_speedup=0.996x
equivalence=PASS
```

Restored-baseline gates after rollback:

```text
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (29.440s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.388s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 13.805s/9.338s
```

Current artifact mapping hash is
`0E9FFA0C1447BEEB54B35CA4990AF02DA3594AEF613E7694D9B2461D09A31B62`.
The optimized jar hash is
`af11fea9e49ca69a1e8c0af8b46e30ca0952c41894086350bc6847304877c54f`;
AppCDS hash is
`4b55a793621407f28a98d9bf1ccfa1725972928394046fdc879708e2512d0796`.

No 20 TPS stable claim, no 500-player claim, and no all-plugins claim are
made from this state. The next measured target is the `NearbyPlayers` /
`ReferenceList.add(...)` movement hot path shown by the latest jstacks.

## Current 2026-05-09 14:27 CEST: ChunkHolderManager transient entity-chunk lazy-init candidate built and measured

`ChunkHolderManager.getOrCreateEntityChunk(...)` now lazily allocates
`AtomicBoolean` and `Thread.currentThread()` only on the non-transient entity
load path. The focused mixed-path benchmark improved from old
`65.410 ms` to new `61.437 ms` (`1.065x`) and reduced allocated bytes from
`140000000` to `20000000`, with equivalence PASS:

```text
reports/entity-chunk-transient-bench.txt
old_mixed_best_ms=65.410
new_mixed_best_ms=61.437
mixed_speedup=1.065x
old_allocated_bytes=140000000
new_allocated_bytes=20000000
saved_allocated_bytes=120000000
equivalence=PASS
```

Fresh gates on the rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (30.140s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (19.105s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.037s/9.349s
```

The strict 50-bot 32/32 spectator gate is blocked by host preflight
(`load_per_cpu=1.003` > `0.750`), so there is no accepted load claim yet:

```text
reports/load-50bots-entitychunk-lazy-transient-gate-20260509-preflight.txt
host_preflight_ok=false
cpu_count=12
load1=12.04
load5=10.85
load15=8.50
load_per_cpu=1.003
idle_percent_1s=58.28
max_load_per_cpu=0.750
```

The explicitly noisy diagnostic did complete and stayed stable, but it is not
comparable to the accepted baseline:

```text
reports/load-50bots-entitychunk-lazy-transient-noisy-20260509-summary.txt
online_max=50
tps1_avg=18.21
avg_tick_ms_avg=63.30
loaded_chunks_max=2295
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

## Current 2026-05-09 13:30 CEST: CaveWorldCarver floor-skip candidate rejected and rolled back

The direct `CaveWorldCarver` floor-skip helper candidate replaced the per-cave
capturing lambda with a specialized `carveCaveEllipsoid(...)` path and passed
the synthetic bench:

```text
reports/cave-carver-skip-bench.txt
old_lambda_best_ms=59.294
reused_checker_best_ms=58.955
direct_helper_best_ms=50.624
direct_helper_speedup=1.171x
equivalence=PASS
```

The strict 50-bot 32/32 spectator gate did not improve the accepted baseline:

```text
reports/load-50bots-cavecarver-floor-skip-gate-20260509-summary.txt
online_max=50
tps1_avg=17.79
avg_tick_ms_avg=108.48
loaded_chunks_max=1867
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The patch was removed from production and the rollback runtime was rebuilt and
verified:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (27.768s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.133s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.395s/9.335s
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 12:45 CEST: Marker applyMarker hook candidate rejected and rolled back

The `DensityFunctions.MarkerOrMarked.mapAll(...)` hook candidate was tested
because `MarkerCacheBench` showed the existing marker-allocation path could be
made much cheaper:

```text
reports/marker-cache-bench.txt
old_best_ms=175.121
cached_best_ms=35.148
cached_speedup=4.982x
old_marker_allocations=1920000
cached_marker_allocations=84000
equivalence=PASS
```

The temporary production patch was
`upstream/Paper/paper-server/patches/features/0044-Use-applyMarker-hook-for-density-function-markers.patch`.
It passed build/hash/plugin/restart/forced-ticket gates on the temporary
artifact, but the strict 50-bot 32/32 spectator load comparison did not beat
the accepted reference line:

```text
reports/load-50bots-marker-hook-gate-20260509-summary.txt
online_max=50
tps1_avg=17.84
avg_tick_ms_avg=67.37
loaded_chunks_max=2081
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

That is below the accepted reference around `18.27 TPS / 47.85 ms / 2380
chunks`, so the patch was removed. Fresh rollback rebuild verification:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applySourcePatches Applied 913 patches, createMojmapBundlerJar PASS
sha256sum -c reports/artifact-hashes.txt: PASS
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 12:29 CEST: BlendedNoise octave-cache candidate rejected and rolled back

The `BlendedNoise` octave-cache candidate was tested because the focused
microbench showed fewer repeated `PerlinNoise.getOctaveNoise(...)` lookups:

```text
reports/blended-noise-octaves-bench.txt
old_getoctave_best_ms=675.507
cached_octaves_best_ms=573.567
cached_octaves_speedup=1.178x
equivalence=PASS
```

The temporary production patch was
`upstream/Paper/paper-server/patches/features/0044-Cache-BlendedNoise-octave-lookups.patch`.
It passed build/hash/plugin/restart/forced-ticket gates, but failed the real
50-bot 32/32 spectator load comparison:

```text
reports/load-50bots-blended-octave-cache-gate-20260509-summary.txt
online_max=50
tps1_avg=17.93
avg_tick_ms_avg=56.72
loaded_chunks_max=2079
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

This is worse than the accepted reference line around
`18.27 TPS / 47.85 ms / 2380 chunks`, so the production patch was removed.
The microbench and reports remain as rejected evidence only.

Fresh rollback verification:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applySourcePatches Applied 913 patches, createMojmapBundlerJar PASS
sha256sum -c reports/artifact-hashes.txt: PASS
generated BlendedNoise.java: no cached octave fields remain
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (28.079s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.050s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 12.727s/8.805s
```

Post-rollback strict 50-bot 32/32 spectator gate is stable but still not the
target:

```text
reports/load-50bots-blended-octave-cache-rollback-20260509-summary.txt
online_max=50
tps1_avg=17.85
avg_tick_ms_avg=56.02
loaded_chunks_max=2176
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 11:51 CEST: EntityLookup experiments rejected and rolled back

Two `EntityLookup` movement-path candidates were tested and rejected:

- direct `FullChunkStatus -> Visibility` mapping in `getEntityStatus(...)`;
- moving status reads into section-change handling to avoid same-section status reads.

The focused direct-status microbench was positive, but production load gates did
not improve the accepted 50-bot baseline:

```text
reports/entity-lookup-status-bench.txt
direct_status_speedup=1.039x
direct_accessible_speedup=1.054x

reports/load-50bots-entitylookup-direct-gate-20260509-summary.txt
online_max=50
tps1_avg=17.53
avg_tick_ms_avg=46.96
loaded_chunks_max=2083
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The section-change status-skip candidate could not get a strict comparable gate
at first because the host was busy, and its explicit noisy diagnostic was worse
than the accepted baseline:

```text
reports/load-50bots-entitymove-status-skip-gate-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=0.928

reports/load-50bots-entitymove-status-skip-noisy-20260509-summary.txt
online_max=50
tps1_avg=17.22
avg_tick_ms_avg=45.42
loaded_chunks_max=1827
watchdog_thread_dumps=1
```

Both candidates were rolled back. Current production source is back to
`Visibility.fromFullChunkStatus(...)` in
`EntityLookup.getEntityStatus(...)` and the original `EntityCallback.onMove()`
status-change flow.

Fresh verification on the restored artifact:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applySourcePatches Applied 913 patches, createMojmapBundlerJar PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (26.884s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (16.695s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 13.839s/8.889s
```

Restored-baseline strict 50-bot 32/32 spectator gate completed without bot
failures, watchdog dumps, or sync-load stack hits, but it is not a new 20 TPS
claim and does not beat the older accepted `18.27/47.85/2380` load baseline:

```text
reports/load-50bots-baseline-restored-20260509-summary.txt
online_max=50
tps1_avg=17.66
avg_tick_ms_avg=47.78
loaded_chunks_max=1964
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

## Current 2026-05-09 08:21 CEST: ReferenceList limit-64 sweep rejected and rolled back

`ReferenceList` now has an optional small-list linear path, enabled only for
`NearbyPlayers.TrackedChunk` player lists with threshold `2` after the
rejected limit-64 sweep was rolled back. The intended hot path is sparse fast
spectator movement where many chunk watch lists contain one or two players;
broader `ReferenceList` users still use the old default hash-index path.

Focused runtime benchmark on the rejected limit-64 artifact:

```text
reports/reference-list-threshold64-bench.txt
single_runtime_speedup_vs_old=2.133x
pair_runtime_speedup_vs_old=1.811x
dense_runtime_speedup_vs_old=1.119x
```

Verdict from the microbench and noisy load profile: the isolated benchmark
looked good, but the noisy 50-bot movement run regressed to
`tps1_avg=17.67`, `avg_tick_ms_avg=65.91`, `loaded_chunks_max=2001`, one
watchdog dump, and `ReferenceList.add(...)` on the server thread, so the
experiment was rolled back.

Fresh verification:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, applySourcePatches Applied 912 patches, createMojmapBundlerJar PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (27.920s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (19.373s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.297s/9.507s
```

Strict 50-bot 32/32 spectator gate did not start because host preflight failed:

```text
reports/load-50bots-referencelist64-strict-20260509-0610-preflight.txt
host_preflight_ok=false
load_per_cpu=0.809
max_load_per_cpu=0.750
```

Explicit noisy diagnostic only:

```text
reports/load-50bots-referencelist64-noisy-20260509-0612-summary.txt
online_max=50
tps1_avg=17.67
avg_tick_ms_avg=65.91
loaded_chunks_max=2001
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=4
```

This noisy run is not comparable and is not a 20 TPS or 500-player claim. The
captured server-thread jstack was dominated by
`NearbyPlayers.tickPlayer -> TrackedChunk.removePlayer ->
ReferenceList.remove` on the earlier run and then
`ReferenceList.add(...)` on the limit-64 run, so the next movement-side
profile should focus on the `NearbyPlayers` list shape before promoting the
sparse-list candidate.

## Current 2026-05-09 04:00 CEST: PlacedFeature traversal built, compat gates pass, strict 50-bot run not accepted

`PlacedFeature.placeWithContext(...)` now uses a depth-first recursive traversal
over placement modifiers instead of a `Stream.flatMap` chain. The recursion
preserves modifier order, closes each child stream, and keeps debug feature
counting at the same placement point.

Focused microbench:

```text
reports/placed-feature-traversal-bench.txt
equivalence=PASS
stream_total_ns=393666514
recursive_total_ns=276173886
speedup=1.425x
```

Fresh verification on the rebuilt artifact:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
runtime remap hash: 7D8197836863DC2647D53F142E738251AF2ADDD919D5FA6054EFCCF17946F33A
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (27.869s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (18.230s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.263s/9.582s
```

Strict 50-bot 32/32 spectator gate was runnable but not accepted:

```text
reports/load-50bots-placedfeature-traversal-gate-20260509-0558-summary.txt
host_preflight_ok=true
load_per_cpu=0.693
online_max=50
tps1_avg=17.71
avg_tick_ms_avg=42.70
loaded_chunks_max=1928
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

No 20 TPS stable claim is made from this run, and no 500-player claim is made.

Current hot path from the fresh jstack is still worldgen-heavy:
`NoiseChunk$NoiseInterpolator.compute`, `NoiseBasedChunkGenerator.getBaseHeight`,
`ChunkGenerator.tryGenerateStructure`, and the movement snap path through
`ServerGamePacketListenerImpl.handleMovePlayer -> Entity.absSnapToNoChunkLoad ->
WaypointTransmitter`.

## Current 2026-05-09 03:30 CEST: current build verified, strict load gate blocked by host

Current source patch stack applies cleanly again: `applySourcePatches` applies
`912` patches, and `MC_EULA_AGREE=true ./scripts/build_optimized.sh` passes.
The current movement candidate keeps only the verified spectator no-sync-load
guards in `ServerGamePacketListenerImpl` movement reset/final snap paths; the
broader `tickPlayer()` attempt was not kept.

Fresh runtime gates on the rebuilt artifact:

```text
plugin matrix: PASS, Done (30.599s)
restart/recovery: PASS, Done (18.990s)
forced-ticket persistence: PASS, first/restart Done 14.791s/10.665s
```

Strict 50-bot 32/32 spectator gate is blocked before launch:

```text
reports/load-50bots-spectator-nosyncload-reset-gate-20260509-0355-preflight.txt
host_preflight_ok=false
load_per_cpu=0.885
max_load_per_cpu=0.750
```

No 20 TPS stable, no 50-bot accepted baseline, and no 500-player claim is made
from this state.

## Current 2026-05-09 Continuation: build path restored, runtime gates pass, 50-bot gate still not accepted

The immediate build blocker was a broken worktree source-patch layer:
`applySourcePatches` was only seeing one source patch, so Moonrise feature
patching merged against the wrong base. The missing worktree source patches
were mechanically restored from the current git objects/index without resetting
the existing dirty optimization stack. The source patch tree now has `916`
files, with no deleted or zero-byte source patches.

Fresh verification on the rebuilt artifact:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
applySourcePatches: Applied 912 patches
applyFeaturePatches: PASS
createMojmapBundlerJar: PASS
runtime remap hash: FBE33F5C9C15DFE407681ED1912619F0809570B13565512F7ABAD53BA7E2EB5C
sha256sum -c reports/artifact-hashes.txt: PASS after refreshing rebuilt artifact hashes
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (27.348s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.499s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 15.120s/9.131s
```

Fresh pinned 50-bot 32/32 spectator gate:

```text
reports/load-50bots-buildrestore-gate-20260509-0157-preflight.txt
host_preflight_ok=true
load_per_cpu=0.742
idle_percent_1s=63.26

reports/load-50bots-buildrestore-gate-20260509-0157-summary.txt
worker_line=[01:57:03 INFO]: [MoonriseCommon] Paper is using 6 worker threads, 1 I/O threads
online_max=50
tps1_min=18.73
tps1_avg=19.52
avg_tick_ms_max=40.74
avg_tick_ms_avg=26.57
loaded_chunks_max=1406
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=0
watchdog_thread_dumps=8
sync_load_stack_hits=7
nearby_players_stack_hits=4
```

Verdict: not an accepted 50-bot baseline and not a 500-player claim. The run
has strong TPS/MSPT and no bot failures, but the watchdog/thread-dump and
`ServerChunkCache.syncLoad` hits fail the stability bar. The repeated stack is
`ServerGamePacketListenerImpl.handleMovePlayer -> Entity.absSnapTo ->
Level.getChunk -> ServerChunkCache.syncLoad` while chunk workers are busy in
fresh chunk generation (`NoiseChunk`, `SurfaceSystem`, `ChunkStatusTasks`).

Config diagnostic without CPU pinning was also rejected:

```text
reports/load-50bots-buildrestore-nocpuset-20260509-0200-summary.txt
worker_line=[02:05:10 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
online_max=50
tps1_min=1.93
tps1_avg=16.79
avg_tick_ms_avg=353.82
loaded_chunks_max=4764
watchdog_thread_dumps=5
sync_load_stack_hits=5
```

Giving the run all 12 host CPUs increased chunk coverage but collapsed latency
and still did not remove sync-load stalls. It is recorded as rejected
configuration evidence, not as an optimization.

## Current 2026-05-08 Continuation: NoiseChunk marker wrapper cache built, clean load gate blocked by host

Built and kept pending a clean gate: `NoiseChunk` wrapping now reuses the
existing reference wrapper cache for repeated `DensityFunctions.MarkerOrMarked`
nodes. The visitor still receives the already mapped child function and creates
the same `NoiseInterpolator` / `FlatCache` / `Cache2D` / `CacheOnce` /
`CacheAllInCell` wrapper type; it only avoids allocating duplicate wrappers
when the same marker node is shared in the density-function DAG.

Focused benchmark:

```text
reports/marker-cache-bench.txt
old_best_ms=173.517
cached_best_ms=33.489
cached_speedup=5.181x
old_marker_allocations=1920000
cached_marker_allocations=84000
equivalence=PASS
```

Functional verification:

```text
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 912 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS, compileJava and createMojmapBundlerJar executed
javap NoiseChunk$1: PASS, applyMarker calls wrapMarker(MarkerOrMarked, DensityFunction)
sha256sum -c reports/artifact-hashes.txt: PASS after refreshing rebuilt artifact hashes
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (31.651s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (18.882s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 15.372s/10.768s
```

Strict 50-bot gate status:

```text
reports/load-50bots-marker-cache-gate-20260508-preflight.txt
host_preflight_ok=false
load_per_cpu=0.807
idle_percent_1s=43.05
max_load_per_cpu=0.750
```

Noisy diagnostic-only run:

```text
reports/load-50bots-marker-cache-noisy-20260508-summary.txt
online_max=50
tps1_avg=17.38
avg_tick_ms_avg=429.99
loaded_chunks_max=2745
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Verdict: this is a measured microbench/allocation win and compatibility-passing
runtime build, not an accepted end-to-end 50-bot or 500-bot performance claim.
The clean gate is blocked by host load, and the noisy diagnostic run is
explicitly non-comparable to the accepted baseline (`18.27/47.85/2380`).

Also checked and rejected before production: direct byte-array lookup inside
`ImprovedNoise.sampleWithDerivative(...)`. The focused derivative benchmark
passed equivalence but did not improve (`56.989 ms` old vs `57.170 ms`
inline, `0.997x`), so no production source was changed for that candidate.

## Current 2026-05-08 Continuation: OreFeature exact loop cleanup built, clean load gate blocked by host

Built and kept pending a clean gate: `OreFeature.doPlace(...)` now reuses the
exact same `d5 * d5` and `d5 * d5 + d6 * d6` intermediate values inside the
ore blob loop and hoists `width * height` out of the innermost index
calculation. The change does not replace division with reciprocal
multiplication because that can alter floating-point boundary behavior and
worldgen parity near the `< 1.0` checks.

Focused benchmark:

```text
reports/ore-feature-loop-bench.txt
old_loop_best_ms=60.507
optimized_loop_best_ms=58.403
optimized_speedup=1.036x
equivalence=PASS
```

Functional verification:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 912 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (29.608s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.992s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.978s/10.573s
```

Strict 50-bot gate status:

```text
reports/load-50bots-orefeature-loop-gate-rerun1-preflight.txt
host_preflight_ok=false
load_per_cpu=1.970
idle_percent_1s=56.47
max_load_per_cpu=0.750
```

Noisy diagnostic-only run:

```text
reports/load-50bots-orefeature-loop-noisy-summary.txt
online_max=50
tps1_avg=17.40
avg_tick_ms_avg=87.38
loaded_chunks_max=2210
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Verdict: not an accepted end-to-end TPS/load win yet. The clean gate is blocked
by host load, and the noisy run is explicitly non-comparable to the accepted
baseline (`18.27/47.85/2380`).

## Current 2026-05-08 Continuation: Beardifier bury branch rejected and reverted

Rejected and reverted: `Beardifier.getBuryContribution(...)` direct branch
avoided `Mth.clampedMap(...)` for lengths above `6.0`, but it did not beat the
accepted 50-bot / 32 view-distance / 32 simulation-distance gate.

Focused benchmark evidence:

```text
reports/beardifier-bury-bench.txt
current_clamped_map_best_ms=8.304
optimized_branch_best_ms=7.063
optimized_speedup=1.176x
equivalence=PASS
```

Candidate strict gate:

```text
reports/load-load-50bots-beardifier-bury-gate-20260508-2312-preflight.txt
host_preflight_ok=true
load_per_cpu=0.725
idle_percent_1s=60.43

reports/load-load-50bots-beardifier-bury-gate-20260508-2312-summary.txt
online_max=50
tps1_avg=17.97
avg_tick_ms_avg=65.67
loaded_chunks_max=2539
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0

accepted baseline:
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Post-revert verification on the rebuilt runtime:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (27.842s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.406s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.870s/10.043s
```

Post-revert strict gate ran but did not become a new accepted baseline:

```text
reports/load-load-50bots-post-beardifier-revert-gate-20260508-2323-summary.txt
online_max=50
tps1_avg=16.57
avg_tick_ms_avg=112.19
loaded_chunks_max=3212
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Current status: the production path is back to the pre-Beardifier runtime. The
project still has no 20 TPS / 50-bot or 500-bot claim.

## Current 2026-05-08 Continuation: ProtoChunk heightmap iterator removal built, strict load gate blocked by host

Built and kept: `ProtoChunk.setBlockState(...)` now iterates `Heightmap.Types`
through a cached `Heightmap.Types[]` and `EnumSet.contains(...)`, which removes
the `RegularEnumSet.iterator()` allocation site from the hot heightmap update
path without changing serialization or plugin-visible behavior. The durable
change lives in
`paper-server/patches/sources/net/minecraft/world/level/chunk/ProtoChunk.java.patch`.

Fresh benchmark evidence:

```text
reports/protochunk-heightmap-bench.txt
old_enumset_foreach_best_ms=138.483
new_cached_values_contains_best_ms=105.978
new_speedup=1.307x
old_iterator_allocations_per_setblock=2
new_iterator_allocations_per_setblock=0
equivalence=PASS
```

Functional verification on the rebuilt runtime:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 911 source patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (29.098s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (17.664s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.544s/10.444s
```

The strict 50-bot 32/32 spectator gate was blocked before Minecraft started
because the host was too busy:

```text
reports/load-load-50bots-protochunk-heightmap-spectator-gate-20260508-2045-preflight.txt
host_preflight_ok=false
load_per_cpu=0.792
idle_percent_1s=58.15
max_load_per_cpu=0.750
```

This is a microbench win plus compatibility-passing runtime build, not a
50-bot or 500-bot load claim.

## Current 2026-05-08 Continuation: RangeChoice constant-out fillArray fast-path built, strict load gate blocked by host

Built and kept: `DensityFunctions.RangeChoice.fillArray(...)` now skips
`contextProvider.forIndex(i)` for constant `whenOutOfRange`, and skips all
child context calls when both branches are constant. The production change is
in `paper-server/patches/features/0041-Optimize-RangeChoice-constant-out-fillArray.patch`.

Fresh benchmark evidence:

```text
reports/range-choice-bench.txt
scenario=in_dynamic_out_constant
old_fillarray_best_ms=10.564
optimized_fillarray_best_ms=9.792
optimized_fillarray_speedup=1.079x
old_for_index_calls=1000000
optimized_for_index_calls=599505

scenario=both_constant
old_fillarray_best_ms=10.218
optimized_fillarray_best_ms=7.418
optimized_fillarray_speedup=1.377x
old_for_index_calls=1000000
optimized_for_index_calls=0

scenario=both_dynamic
old_fillarray_best_ms=12.465
optimized_fillarray_best_ms=12.649
optimized_fillarray_speedup=0.985x
old_for_index_calls=1000000
optimized_for_index_calls=1000000
```

Functional verification on the rebuilt runtime:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (30.224s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (21.237s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.619s/10.043s
```

The strict 50-bot 32/32 gate was blocked before Minecraft started because the
host was busy:

```text
reports/load-load-50bots-rangechoice-constant-out-gate-preflight.txt
host_preflight_ok=false
load_per_cpu=0.799
idle_percent_1s=47.13
max_load_per_cpu=0.750
```

This is a measured microbench win and a compatibility-passing runtime build,
not an end-to-end 50-bot load claim.

## Current 2026-05-08 Continuation: PalettedContainer remap-cache rejected, scratch-only runtime rebuilt

Rejected and reverted: `PalettedContainer.reencodeContents(...)` no longer
keeps the old-palette-id to new-palette-id remap table. The production path is
back to the earlier scratch-only unpack buffer reuse, which is the current
runtime.

```text
reports/paletted-reencode-remap-cache-bench.txt
current_previous_only_best_ms=967.335
cached_palette_ids_best_ms=937.103
cached_speedup=1.032x
equivalence=PASS

reports/load-50bots-paletted-remap-cache-gate-rerun1-preflight.txt
host_preflight_ok=true
load_per_cpu=0.415
idle_percent_1s=75.94

reports/load-50bots-paletted-remap-cache-gate-rerun1-summary.txt
online_max=50
tps1_avg=16.48
avg_tick_ms_avg=76.59
loaded_chunks_max=2813
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0

accepted baseline:
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Functional verification after revert:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (31.022s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (20.065s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 15.145s/10.071s
```

The strict post-revert 50-bot rerun is blocked before Minecraft starts because
the host is again above the benchmark preflight threshold:

```text
reports/load-50bots-post-paletted-remap-revert-gate-rerun1-preflight.txt
host_preflight_ok=false
load_per_cpu=0.807
idle_percent_1s=57.20
max_load_per_cpu=0.750
```

This is not an end-to-end TPS/load/cold-start claim.

Fresh diagnostic-only JFR on the current scratch-only runtime:

```text
reports/load-10bots-post-paletted-remap-revert-jfr-summary.txt
online_max=10
tps1_avg=18.89
avg_tick_ms_avg=38.18
loaded_chunks_max=1828
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Top hot methods remain noise/worldgen dominated:

```text
ImprovedNoise.sampleAndLerp(...) 23.83%
ImprovedNoise.noise(...) 9.26%
PerlinNoise.getValue(...) 7.01%
Climate$RTree$SubTree.search(...) 3.16%
Aquifer$NoiseBasedAquifer.computeSubstance(...) 2.46%
```

The follow-up `BiomeManager.getBiome(...)` lower-bound early-exit candidate was
rejected at microbench stage:

```text
reports/biome-getbiome-bench.txt
old_getbiome_best_ms=136.628
optimized_getbiome_best_ms=193.205
optimized_speedup=0.707x
equivalence=PASS
```

No production code was changed for that biome candidate.

## Current 2026-05-08 Continuation: NoiseChunk interpolator indexed traversal rejected and reverted

Rejected and reverted: the `NoiseChunk` interpolator indexed-traversal
candidate had a small focused win, but the real strict 50-bot 32/32 gate that
passed preflight failed the accepted baseline.

```text
reports/noisechunk-interpolator-array-bench.txt
list_loop_best_ms=1108.416
array_loop_best_ms=1052.171
array_speedup=1.053x
equivalence=PASS

reports/load-50bots-current-goal-interpolator-array-gate-rerun2-summary.txt
online_max=50
tps1_avg=17.87
avg_tick_ms_avg=142.23
loaded_chunks_max=2336
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The source patch is back to foreach/forEach traversal for
`NoiseChunk.interpolators`. Post-revert build/hash/plugin/restart/forced-ticket
gates passed, but the strict post-revert 50-bot gate was blocked by host
preflight (`load_per_cpu=0.923`). A noisy 10-bot JFR on the post-revert runtime
passed as diagnostic-only evidence at `19.39 TPS / 38.78 ms / 1955 chunks`.

## Current 2026-05-08 Continuation: DensityFunctions.Spline context-direct candidate rejected and reverted

Rejected and reverted: `DensityFunctions.Spline.compute(...)` briefly
switched from wrapping `FunctionContext` in `Spline.Point` to passing the
context directly through the cubic spline, which removed the hot
`Spline.Point` allocation site in a focused benchmark. The standalone
benchmark was real and positive, but the strict 50-bot 32/32 gate regressed
the accepted baseline, so the production patch was removed again.

Standalone benchmark:

```text
reports/density-spline-context-bench.txt
old_wrapper_best_ms=33.380
new_direct_best_ms=21.615
direct_speedup=1.544x
old_wrapper_allocated_bytes_per_call=16.0
new_direct_allocated_bytes_per_call=0.0
saved_allocated_bytes_per_call=16.0
equivalence=PASS
```

Strict 50-bot gate on the temporary artifact:

```text
reports/load-50bots-density-spline-context-gate-summary.txt
online_max=50
tps1_avg=18.51
avg_tick_ms_avg=66.01
loaded_chunks_max=2101
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Post-revert verification on the rebuilt runtime:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
plugin matrix: PASS, Done (36.988s)
restart/recovery: PASS, Done (19.526s)
forced-ticket persistence: PASS, first/restart Done 21.483s/18.688s
sha256sum -c reports/artifact-hashes.txt: PASS after refreshing artifact hashes
```

This is a narrow worldgen allocation cleanup that failed the strict load
baseline. It is not an accepted load/TPS improvement.

## Current 2026-05-08 Continuation: Plugin startup name-log aggregation optimized

Accepted with limits: `PluginInitializerManager.load(...)` no longer builds
sorted unique Paper/Bukkit plugin name lists with two `TreeSet<String>`
instances while iterating provider storage. It now appends raw display names to
`ArrayList<String>`, sorts once, and deduplicates in place before logging. The
observable startup log order remains sorted and unique; plugin discovery,
provider registration, remapping, lifecycle, classloading, events, scheduler,
permissions/services, and command behavior are unchanged.

Standalone benchmark:

```text
reports/plugin-name-log-bench.txt
plugins=512
warmup=3 rounds=6 iterations=5000
old_treeset_best_ms=343.898
new_arraylistsort_best_ms=45.491
arraylistsort_speedup=7.560x
```

Production verification on the rebuilt runtime:

```text
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 911 source patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
plugin matrix: PASS, Done (32.863s)
restart/recovery: PASS, Done (23.341s)
forced-ticket persistence: PASS, first/restart Done 21.545s/13.276s
sha256sum -c reports/artifact-hashes.txt: PASS after refreshing artifact hashes
```

Strict 50-bot 32/32 load gate after this cycle is blocked by host preflight
before Minecraft starts:

```text
reports/load-50bots-plugin-name-log-current-gate-preflight.txt
host_preflight_ok=false
load_per_cpu=0.812
idle_percent_1s=62.15
max_load_per_cpu=0.750
```

This is a narrow plugin-startup logging allocation/work reduction. It is not an
end-to-end cold-start claim and it is not load/TPS evidence for the 50/500 bot
target.

## Current 2026-05-08 Continuation: Legacy plugin provided-alias removal optimized

Accepted with limits: `LegacyPluginLoadingStrategy` no longer removes aliases
for each loaded/failed legacy plugin by scanning `pluginsProvided.values()` with
`removeIf(...)`. It now keeps a reverse alias index from provider name to
currently owned provided aliases, updates that index on alias replacement/name
shadowing, and removes only the aliases owned by the provider being loaded or
discarded. This preserves the legacy semantics of `pluginsProvided` while
removing an O(providers * provided-aliases) startup scan.

Standalone benchmark:

```text
reports/legacy-provided-alias-removal-bench.txt
providers=512
aliases_per_provider=4
iterations=200 warmup=3 rounds=6
old_values_removeif_best_ms=503.237
new_reverse_alias_remove_best_ms=32.279
alias_removal_speedup=15.590x
```

Production verification on the rebuilt runtime:

```text
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 911 source patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
plugin matrix: PASS, Done (32.124s)
restart/recovery: PASS, Done (28.224s)
forced-ticket persistence: PASS, first/restart Done 18.894s/11.749s
sha256sum -c reports/artifact-hashes.txt: PASS after refreshing artifact hashes
```

This is a narrow plugin-loading startup-work reduction. It is not promoted as
an end-to-end cold-start win and it is not load/TPS evidence for the 50/500 bot
target.

Rejected before production: increasing `NbtIo` GZIP/pre-GZIP buffers for
`writeCompressed(...)`. The benchmark used a real `level.dat` and confirmed the
compressed bytes were identical for all variants, but every larger-buffer
variant was slower:

```text
reports/nbt-gzip-write-bench.txt
current_best_ms=1328.262
gzip64k_best_ms=1564.097
prebuffer64k_best_ms=1580.647
both64k_best_ms=1686.143
gzip64k_speedup=0.849x
prebuffer64k_speedup=0.840x
both64k_speedup=0.788x
```

No `NbtIo` production code was changed for that candidate.

## Current 2026-05-08 Continuation: Xoroshiro direct positional helpers rejected and reverted

The `Xoroshiro` positional direct-helper candidate is not in the current
production path. It had strong standalone evidence, but both split production
paths failed the strict 50-bot 32/32 baseline requirement.

First rejected split:

```text
reports/load-50bots-xoroshiro-aquifer-location-gate-rerun1-summary.txt
online_max=50
tps1_avg=17.65
avg_tick_ms_avg=76.12
loaded_chunks_max=2016
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Second rejected split:

```text
reports/load-50bots-xoroshiro-direct-no-aquiferlocation-gate-rerun1-summary.txt
online_max=50
tps1_avg=15.45
avg_tick_ms_avg=92.58
loaded_chunks_max=1264
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Accepted baseline to beat:

```text
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

The current source tree no longer contains `nextFloatAt(...)`,
`nextDoubleAt(...)`, `aquiferLocationAt(...)`, or `firstLongAt(...)` in the
Paper source patches or generated Minecraft sources.

Post-revert verification on the rebuilt runtime:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.689s)
restart/recovery: PASS, Done (19.938s)
forced-ticket persistence: PASS, first/restart Done 17.550s/11.041s
```

The strict post-revert 50-bot 32/32 rebaseline is blocked by host preflight:

```text
reports/load-50bots-post-xoroshiro-direct-revert-gate-rerun1-preflight.txt
host_preflight_ok=false
load_per_cpu=0.920
idle_percent_1s=41.04
max_load_per_cpu=0.750
```

Current boot benchmark is still nowhere near the `<1s` target:

```text
vanilla-1.21.10 done_ms=17466 rss_kb=1029748
stock-paper-1.21.10 done_ms=36903 rss_kb=1594944
optimized-paper-1.21.10 done_ms=25473 rss_kb=1313432
optimized-runtime-1.21.10 done_ms=21629 rss_kb=934156
```

## Current 2026-05-08 Continuation: SurfaceRules SequenceRule Array Candidate Rejected And Reverted

`SurfaceRules.SequenceRule` is back to the upstream-style runtime
`List<SurfaceRule>` plus `ImmutableList.builder()` construction. The temporary
runtime `SurfaceRule[]` / indexed-loop candidate is not in the production path.
It looked good in a narrow microbench, but lost the strict 50-bot 32/32 server
gate and was reverted.

Standalone benchmark:

```text
reports/surfacerules-sequence-array-bench.txt
list_enhanced_best_ms=587.609
list_indexed_best_ms=565.372
array_best_ms=314.925
array_indexed_best_ms=309.618
array_indexed_speedup=1.898x vs list-enhanced baseline
array_indexed_vs_array=1.017x
equivalence=PASS
```

Rejected before production: a modeled `Aquifer` cache-index stride rewrite was
slower than the current inlined `getIndex(...)` loop:

```text
reports/aquifer-index-stride-bench.txt
old_getindex_loop_best_ms=277.865
new_stride_loop_best_ms=313.746
stride_speedup=0.886x
equivalence=PASS
```

Current artifact verification after the `SurfaceRules` loop edit:

```text
reports/load-50bots-surfacerules-array-index-gate-rerun2-summary.txt
preflight: PASS, host_preflight_ok=true, load_per_cpu=0.547, idle_percent_1s=74.43
online_max=50
loaded_chunks_max=1785
tps1_avg=15.95
avg_tick_ms_avg=117.42
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

That is worse than the accepted baseline `18.27 TPS / 47.85 ms / 2380 chunks`,
so the candidate was rejected despite the standalone improvement.

Post-revert verification on the rebuilt runtime:

```text
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 910 source patches
./gradlew applyPatches --no-daemon: PASS, Applied 910 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
optimized jar sha256=5613a8078e28d28c295979acdbcea3383c777ce402df50e1f240c689efbcaeb4
app-cds sha256=298476ac14be581fdb09513712ae93ea748d43f4e84a7495ba4f3b1b75f90370
plugin matrix: PASS, Done (34.263s)
restart/recovery: PASS, Done (21.224s)
forced-ticket persistence: PASS, first/restart Done 17.307s/11.562s
```

Noisy stability smoke on the rejected candidate artifact remains diagnostic
only:

```text
reports/load-10bots-surfacerules-array-index-noisy-summary.txt
online_max=10
tps1_avg=18.61
avg_tick_ms_avg=243.85
loaded_chunks_max=2492
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Decision: rejected and reverted. Do not repeat this shape unless a future JFR
profile explains the server-side regression and a fresh strict gate beats the
accepted `18.27/47.85/2380` baseline.

## Current 2026-05-08 Continuation: Aquifer Surface Offset Candidate Rejected

Rejected and reverted: `Aquifer.NoiseBasedAquifer.computeFluid(...)` briefly
avoided the per-sample `int[][]` row lookup plus
`SectionPos.sectionToBlockCoord(...)` in the fixed surface-sampling loop. The
candidate kept the same 13 sample positions and passed the standalone
equivalence benchmark, but it lost the real strict 50-bot 32/32 gate.

Narrow benchmark:

```text
reports/aquifer-surface-sampling-bench.txt
old_chunk_offsets_best_ms=275.983
new_block_offsets_best_ms=244.223
block_offsets_speedup=1.130x
equivalence=PASS
```

Verification:

```text
temporary candidate applyPatches: PASS, Applied 910 source patches plus 0041 feature patch
compileJava: PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (35.474s)
restart/recovery: PASS, Done (35.817s)
forced-ticket persistence: PASS, first/restart Done 27.624s/14.883s
noisy 10-bot 32/32 smoke: PASS, 18.04 TPS / 49.35 ms / 1441 chunks
```

The first strict attempt was blocked by host preflight, but rerun1 passed
preflight and rejected the candidate:

```text
reports/load-50bots-aquifer-surface-offsets-gate-rerun1-summary.txt
online_max=50
tps1_avg=17.14
avg_tick_ms_avg=82.71
loaded_chunks_max=2030
watchdog_thread_dumps=0
sync_load_stack_hits=0

accepted baseline to beat:
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Decision: rejected. `0041-Optimize-Aquifer-surface-sampling-offsets.patch` was
deleted, `applyPatches` returned to `Applied 910 patches`, and the optimized
runtime was rebuilt.

Post-revert verification:

```text
compileJava: PASS
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
optimized jar sha256=97720a304176d0f6fa8d222a3b1374de4390aa5debc96924ecd844e12906e3ff
mappings_hash=9383762D002E33F5BFB2E2D9BB59DBCE11135EE10227DB71E8270AB56F0AF16A
plugin matrix: PASS, Done (32.234s)
restart/recovery: PASS, Done (20.809s)
forced-ticket persistence: PASS, first/restart Done 15.835s/11.609s
strict post-revert 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.041 > 0.750
noisy post-revert 10-bot 32/32 smoke: PASS, 19.17 TPS / 36.29 ms / 1572 chunks,
zero kicks/errors/watchdog/sync-load
```

No 20 TPS / 500-bot / vanilla-parity claim is made.

## Current 2026-05-08 Continuation: PerlinNoise Guarded Direct-Local Candidate Rejected

Fresh perlin bench on the restored runtime kept subclass semantics intact by
guarding the direct-local fast path behind an exact-class check, but it still
lost to the current production path:

```text
reports/perlin-getvalue-bench.txt
delegating_getvalue_best_ms=757.621
direct_getvalue_best_ms=755.162
direct_local_getvalue_best_ms=790.294
direct_local_guarded_getvalue_best_ms=772.579
direct_no_y_scale_getvalue_best_ms=831.940
direct_math_wrap_getvalue_best_ms=727.694
direct_speedup=1.003x
direct_local_speedup=0.959x
direct_local_guarded_speedup=0.981x
direct_no_y_scale_speedup=0.911x
math_wrap_speedup=1.041x
equivalence=PASS
```

Decision: rejected. The public 3-arg `getValue(...)` path remains delegated to
the deprecated six-arg method so subclass semantics stay unchanged.

## Current 2026-05-08 Continuation: ImprovedNoise Arithmetic sampleAndLerp Rejected

The local C2ME/DivineMC arithmetic shape for `ImprovedNoise.sampleAndLerp(...)`
was tested only in the standalone benchmark. It stayed bit-exact, but it was
slower than the current flat-gradient implementation:

```text
reports/improved-noise-inline-bench.txt
old_p_method_best_ms=43.172
inline_byte_access_best_ms=48.075
flat_gradient_best_ms=42.764
arithmetic_best_ms=46.272
switch_gradient_best_ms=49.815
flat_gradient_speedup=1.010x
arithmetic_speedup=0.933x
arithmetic_vs_flat_speedup=0.924x
switch_vs_flat_speedup=0.858x
equivalence=PASS
```

Decision: rejected before production changes.

## Current 2026-05-08 Continuation: ImprovedNoise Switch Gradient Rejected

Fresh noisy 10-bot JFR on the current post-revert artifact passed without
kicks, bot errors, watchdog dumps, or sync-load hits:

```text
reports/load-10bots-current-jfr.jfr
online_max=9
tps1_avg=18.80
avg_tick_ms_avg=39.14
loaded_chunks_max=1824
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Top CPU frames from `jfr view hot-methods`:

```text
ImprovedNoise.sampleAndLerp 23.18%
ImprovedNoise.noise 10.13%
PerlinNoise.getValue 7.20%
Climate.RTree.SubTree.search 3.59%
Aquifer.NoiseBasedAquifer.computeSubstance 2.12%
NoiseChunk.NoiseInterpolator.compute 1.60%
```

Based on that, a switch-based replacement for the current flat gradient lookup
inside `ImprovedNoise.sampleAndLerp` was measured only in the standalone bench.
It was bit-exact but slower than the current flat-gradient table:

```text
reports/improved-noise-switchgrad-bench.txt
flat_gradient_best_ms=39.535
switch_gradient_best_ms=47.174
switch_vs_flat_speedup=0.838x
equivalence=PASS
```

Decision: rejected at microbench stage; production source remains unchanged.

## Current 2026-05-08 Continuation: FlatCache Context Candidate Rejected

Rejected and reverted: `NoiseChunk.FlatCache` briefly moved
`MutableSinglePointContext` allocation inside `if (computeValues)`. The
microbench was positive, but the real strict 50-bot 32/32 gate regressed badly
and produced a watchdog thread dump:

```text
reports/noisechunk-flatcache-context-bench.txt
old_false_context_best_ms=100.405
new_false_context_best_ms=87.944
false_context_speedup=1.142x
saved_false_allocated_bytes_per_iteration=24.0
equivalence=PASS

reports/load-50bots-flatcache-context-gate-summary.txt
online_max=50
tps1_avg=15.36
avg_tick_ms_avg=254.43
loaded_chunks_max=1621
watchdog_thread_dumps=1
sync_load_stack_hits=0
```

Accepted comparable baseline remains:

```text
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Production path is reverted: `MutableSinglePointContext` is again created
before `if (computeValues)`, while the earlier accepted reusable-context
behavior remains. Post-revert verification:

```text
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
optimized-paper sha256=611631d31122b7126fc22523a8107d68ddbfd0ecad76d78e02abb893ee9fe32b
plugin matrix: PASS, Done (38.170s)
restart/recovery: PASS, Done (22.820s)
forced-ticket persistence: PASS, first/restart Done 19.230s/18.186s
```

The clean post-revert strict 50-bot gate is blocked by current host load:

```text
host_preflight_ok=false
load1=16.34
load_per_cpu=1.362
idle_percent_1s=18.29
max_load_per_cpu=0.750
```

Noisy 10-bot 32/32 smoke on the reverted artifact passed without kicks,
bot errors, watchdog dumps, or sync-load hits, but it is not comparable
performance evidence:

```text
online_max=10
tps1_avg=17.86
avg_tick_ms_avg=39.33
loaded_chunks_max=939
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

## Current 2026-05-08 Continuation

`ImprovedNoise.sampleAndLerp` now uses a flat gradient table plus direct
permutation-byte access in the hot sample path. The production patch is
persisted as `paper-server/patches/features/0040-Optimize-ImprovedNoise-sampleAndLerp.patch`.

Fresh benchmark and verification:

```text
reports/improved-noise-inline-bench.txt
old_p_method_best_ms=48.060
inline_byte_access_best_ms=47.097
flat_gradient_best_ms=42.729
inline_speedup=1.020x
flat_gradient_speedup=1.125x
equivalence=PASS
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (32.501s)
restart/recovery: PASS, Done (21.912s)
forced-ticket persistence: PASS, first/restart Done 16.900s/10.704s
noisy 10-bot 32/32 smoke: PASS, 18.07 TPS / 40.83 ms / 1469 chunks
```

The strict 50-bot 32/32 gate is still blocked by host preflight:

```text
host_preflight_ok=false
load1=12.50
load_per_cpu=1.041
idle_percent_1s=57.78
max_load_per_cpu=0.750
```

This is build/compat/smoke evidence, not a stable 50-bot TPS baseline.

## Previous 2026-05-08 Continuation

Accepted with limits: `NoiseBasedChunkGenerator` now lazily caches
`getGenDepth()`, `getSeaLevel()`, and `getMinY()` with a manual double-checked
primitive cache, so the hot getter path no longer repeats
`settings.value().noiseSettings()` on every call. The earlier
`Supplier`-heavy shape was rejected by microbench; the current primitive-cache
shape is the safe production candidate.

Fresh benchmark and verification:

```text
reports/noise-generator-settings-cache-bench.txt
lazy_primitive_speedup=1.288x
equivalence=PASS
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (39.776s)
restart/recovery: PASS, Done (47.881s)
forced-ticket persistence: PASS, first/restart Done 34.630s/15.000s
noisy 10-bot 32/32 smoke: PASS, 18.10 TPS / 42.12 ms / 679 chunks
```

The strict 50-bot 32/32 gate is still blocked by host preflight:

```text
host_preflight_ok=false
load1=21.48
load_per_cpu=1.790
idle_percent_1s=7.98
max_load_per_cpu=0.750
```

This is build/compat/smoke evidence, not a stable 50-bot TPS baseline.

## Current Candidate Outcome: PerlinNoise.wrap Math.floor Rejected

На 2026-05-08 07:03 CEST candidate `PerlinNoise.wrap(double)` через
`Math.floor(...)` был полностью собран и проверен, но strict 50-bot 32/32 gate
не побил accepted baseline. Итоговый strict run:

```text
tps1_avg=18.16
avg_tick_ms_avg=47.33
loaded_chunks_max=1720
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Accepted baseline remained:

```text
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Result: rejected and reverted back to `Mth.lfloor(...)`. Full доказательства:
`./scripts/bench_perlin_getvalue.sh` `math_wrap_speedup=1.092x`,
`applyPatches` PASS (`Applied 910 patches`), full build PASS, bytecode check
PASS on `java/lang/Math.floor`, `sha256sum -c reports/artifact-hashes.txt`
PASS, plugin matrix PASS `Done (33.758s)`, restart/recovery PASS
`Done (19.015s)`, forced-ticket persistence PASS (`15.816s` first boot /
`12.097s` restart).

Post-revert runtime is back on `Mth.lfloor(...)`; `sha256sum -c
reports/artifact-hashes.txt` PASS and latest post-revert plugin matrix PASS
`Done (29.612s)`.

The prior strict load attempt was only blocked by host preflight; that blocker
was cleared on retry and the candidate was then rejected by the actual load
gate. Exact command, preflight, and rejection evidence are in `BLOCKED.md`.

## Итог

`/root/rust` сейчас является рабочим engineering checkout на базе upstream Paper `ver/1.21.10`, а не переписанным с нуля сервером. Paper/Bukkit compatibility остается контрактом: plugin runtime, scheduler, event order, services, permissions, command semantics и classloading остаются в Java/Paper слое.

EULA принята пользователем в чате. Скрипты всё равно требуют явный `MC_EULA_AGREE=true`; без него они не выставляют `eula=true`.

## Current 2026-05-08 Continuation

Accepted with limits: `ServerEntity.sendChanges()` теперь не вызывает `Vec3.distanceToSqr(lastSentMovement)`, когда `entity.getDeltaMovement()` вернул тот же immutable `Vec3` object, что и уже сохранённый `lastSentMovement`. В этом случае старый код всё равно получал `d == 0` и не отправлял motion packet, поэтому observable packet semantics сохраняются. Узкий bench `reports/serverentity-delta-identity-bench.txt` на реальном runtime `Vec3` показал old distance path `80.075 ms`, identity guard `28.626 ms`, `2.797x`, equivalence PASS при 75% same-identity workload. Full build PASS на финальном patch stack (`applySourcePatches Applied 910 patches`, `compileJava`, `createMojmapBundlerJar`, AppCDS), `sha256sum -c reports/artifact-hashes.txt` PASS, plugin matrix PASS `Done (26.447s)`, restart/recovery PASS `Done (15.842s)`, forced-ticket persistence PASS (`12.865s` first boot / `8.526s` restart). Strict 50-bot 32/32 gate прошёл preflight (`load1=4.79`, `load_per_cpu=0.399`, `idle_percent_1s=82.94`) и завершился без kicks/errors/watchdog/sync-load: `tps1_avg=18.85`, `avg_tick_ms_avg=64.88`, `loaded_chunks_max=1829`. Это не новый accepted load baseline и не доказательство 500 players/20 TPS/32 chunks.

Accepted with limits: `OwnableRewriteRule.matchesOwner(...)` в server jar теперь не строит `owners().stream().map(DescriptorUtils::toOwner).anyMatch(...)`, а проходит `Set<ClassDesc>` прямым циклом и сравнивает owner с `L...;` descriptor через `regionMatches`. Это startup/class-rewrite allocation cleanup для plugin remap/rewrite path, не gameplay/TPS оптимизация. Standalone bench `reports/ownable-rule-bench.txt`: old stream `2052.795 ms`, new loop `326.972 ms`, `6.278x`, equivalence PASS. Runtime jar подтверждён через `jar tf`/`javap`: `io/papermc/asm/rules/OwnableRewriteRule.class` лежит в optimized paper jar, bytecode содержит iterator loop, без stream/map/anyMatch. Final reverted-runtime gates после этого цикла: full `MC_EULA_AGREE=true ./scripts/build_optimized.sh` PASS, `sha256sum -c reports/artifact-hashes.txt` PASS, plugin matrix `Done (29.313s)`, restart/recovery `Done (17.743s)`, forced-ticket persistence `Done (14.023s)` / `Done (9.280s)`.

Accepted with limits: `ObfHelper.loadMappingsIfPresent()` теперь строит class/method/field maps с known-size capacities, pre-size'ит `StringPool` map по реальному числу class/method/field mapping inputs, и напрямую возвращает готовые `mappingsByObfName` / `mappingsByMojangName` maps без промежуточного top-level `Set<ClassMapping>` и stream collectors. Duplicate-key behavior сохранён через fail-fast `putUnique(...)`. Это узкая startup/reflection/remap allocation reduction, не gameplay/TPS оптимизация. Свежий `reports/obfhelper-maps-bench.txt` на реальном `META-INF/mappings/reobf.tiny`: old stream/default maps `256.414 ms`, production-shaped direct top maps + pre-sized `StringPool` bench label `presized_string_pool_best_ms=209.872` (`1.222x` vs old), equivalence PASS. Full build PASS, `sha256sum -c reports/artifact-hashes.txt` PASS, plugin matrix `Done (28.748s)`, restart/recovery `Done (17.694s)`, forced-ticket persistence `Done (13.384s)` / `Done (10.195s)`. Boot benchmark не дал end-to-end startup win: optimized runtime `17776 ms`, stock Paper `32226 ms`. Strict 50-bot run прошёл без kicks/errors/watchdog/sync-load (`18.34/297.56/2635`), но не стал baseline и не приблизил цель stable 20 TPS.

Rejected: bounded early-exit distance для default `Climate.RTree.SubTree.search(...)`. Узкий standalone bench был положительным (`reports/climate-rtree-bound-bench.txt`: old `1410.567 ms`, bounded `1170.869 ms`, `1.205x`, equivalence PASS), но production 50-bot 32/32 gate с patch провалил accepted baseline: `tps1_avg=17.65`, `avg_tick_ms_avg=58.37`, `loaded_chunks_max=2620`, без watchdog/sync-load. Patch удалён из source patch, generated source проверен без `distance(long[] values, long limit)`, artifact пересобран. Контрольный post-revert 50-bot run завершился стабильно без kicks/errors/watchdog/sync-load, но тоже не стал baseline: `tps1_avg=17.06`, `avg_tick_ms_avg=68.42`, `loaded_chunks_max=2758`.

Текущий artifact SHA после отката rejected LZ4 stream wrapper candidate: `artifacts/optimized-paper-1.21.10-mojmap.jar` = `b4d9b690776553b0e3014608dc7276e522b06d32ce04b22e5c0a33e7ec647d30`. Цель `500 bots, 32/32, stable 20 TPS` всё ещё не достигнута; последний accepted load baseline остаётся `18.27 TPS / 47.85 ms / 2380 chunks`, а свежий post-revert strict 50-bot run был stability-only evidence (`18.17 TPS / 48.19 ms / 1923 chunks`, no kicks/errors/watchdog/sync-load).

Rejected: removing the outer `BufferedOutputStream` around `LZ4BlockOutputStream` for region-file LZ4 writes. The standalone stream bench was positive (`reports/lz4-stream-bench.txt`: default buffered `3432.518 ms`, no outer buffer `3028.499 ms`, `1.133x`, equivalence PASS), but the real 50-bot 32/32 gate regressed versus the accepted baseline: `tps1_avg=18.53`, `avg_tick_ms_avg=80.71`, `loaded_chunks_max=2085`, no watchdog/sync-load. The production patch was removed; generated source is back to `new BufferedOutputStream(new LZ4BlockOutputStream(stream))`. Post-revert build/hash/plugin/restart/forced-ticket gates passed: plugin matrix `Done (29.009s)`, restart/recovery `Done (17.360s)`, forced-ticket first/restart `13.751s` / `9.511s`.

## Latest 2026-05-08 Cycle

Проверен candidate `NoiseChunk` empty-blender blend cache allocation: в normal `Blender.empty()` path он убирал две `FlatCache`/`double[]` allocations на `NoiseChunk`, оставляя старое заполнение для non-singleton empty blenders. Узкий benchmark прошёл equivalence и показал `430.571 ms` old vs `10.449 ms` new (`41.207x`). Но real 50-bot 32/32 gate с production patch не стал performance win: `tps1_avg=17.96`, `avg_tick_ms_avg=158.83`, `loaded_chunks_max=2424`, no watchdog/sync-load, хуже accepted baseline `18.27/47.85/2380`. Patch откатан, source patch rebuilt на `910 patches`, optimized artifact пересобран (`sha256=0b64abf35e9b1390190d57e077fed434a20e23a933bd5214bd7ed57b4e986bda`). После отката gates снова зелёные: plugin matrix `Done (27.986s)`, restart/recovery `Done (25.739s)`, forced-ticket persistence `18.289s` first boot / `8.967s` restart, `sha256sum -c reports/artifact-hashes.txt` PASS. Строгий 50-bot rerun после отката прошёл preflight (`load1=8.78`, `load_per_cpu=0.731`, `idle_percent_1s=55.64`) и завершился без kicks/errors/watchdog/sync-load, но тоже не стал baseline: `tps1_avg=17.79`, `avg_tick_ms_avg=86.26`, `loaded_chunks_max=2981`.

Свежий JFR на откатанном artifact (`50bots-after-noisechunk-revert-jfr`) прошёл preflight (`load1=5.69`, `load_per_cpu=0.474`, `idle_percent_1s=78.19`) и завершился без kicks/errors/watchdog/sync-load: `tps1_avg=18.04`, `avg_tick_ms_avg=70.58`, `loaded_chunks_max=2148`. Hot methods: `ImprovedNoise.p(int)` `48.83%`, `Climate$RTree$SubTree.search(...)` `2.59%`, `ImprovedNoise.noise(...)` `2.17%`, `NoiseChunk.updateForZ(...)` `2.15%`, `NoiseChunk$NoiseInterpolator.compute(...)` `1.82%`. Allocation: `NoiseChunk$FlatCache.<init>` `11.30%`, `Iterators.forArrayWithPosition` `10.38%`, `LZ4BlockOutputStream.<init>` `3.27%`, `NoiseChunk.wrapMarker(...)` `3.16%`. GC pauses: `51` pauses, `5.65s` total, P95 `444ms`. A config-only retry of fixed `-Xms10G -Xmx10G` G1 did not start because strict host preflight blocked it (`load1=11.00`, `load_per_cpu=0.917` > `0.750`), so no JVM flag change is promoted.

Проверен новый candidate для `ImprovedNoise.sampleAndLerp`: прямой доступ к локальному `byte[]` вместо вызовов `p(index)`. Standalone microbench прошёл equivalence и показал локальный выигрыш (`47.592 ms` old vs `42.544 ms` inline, `1.119x`), но временный production patch не прошёл server gate как улучшение: 50-bot 32/32 run дал `tps1_avg=17.78`, `avg_tick_ms_avg=62.90`, `loaded_chunks_max=2693`, без watchdog/sync-load, что хуже accepted baseline `18.27/47.85/2380`. Production patch удалён; bench/report сохранены как rejected evidence.

Финальная сборка после отката кандидата снова на `applySourcePatches Applied 910 patches`, `compileJava` PASS, `MC_EULA_AGREE=true ./scripts/build_optimized.sh` PASS, `sha256sum -c reports/artifact-hashes.txt` PASS. Финальный plugin matrix PASS на `Done (29.272s)`, restart/recovery PASS на `Done (17.902s)`, forced-ticket persistence PASS (`13.935s` first boot, `10.021s` restart). Clean final 50-bot rerun заблокирован host preflight до запуска Minecraft: `load1=10.03`, `load_per_cpu=0.835` при лимите `0.750`; подробность в `BLOCKED.md` и `reports/load-50bots-final-after-inline-reject-preflight.txt`.

Текущий optimized runtime собирается, запускается, проходит real plugin matrix и выдержал accepted 50-bot spectator run без падения процесса. Последний accepted load baseline после `NoiseChunk.NoiseInterpolator` delta-cache остается `tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, `moved_too_quickly_warnings=1`, без watchdog/sync-load hits. Свежий 2026-05-07 цикл добавил precomputed reversed-mappings cache для plugin remapper и подтвердил функциональный cache hit, но A/B дал только `34.734s` vs `34.950s` plugin startup, поэтому это не заявляется как real performance win. 50-bot 32/32 gate после cache work прошел без crash/watchdog/sync-load hits, но не стал новым baseline: `tps1_avg=17.53`, `avg_tick_ms_avg=63.15`, `loaded_chunks_max=1059`. Свежий JFR профиль текущего reverted artifact дал `17.23/50.40/1233` без watchdog/sync-load и снова показал `ImprovedNoise.p` как главный CPU hot method. Следующие кандидаты были построены и измерены, но отвергнуты: `Climate.RTree.Node` cached `parameter0..parameter6` (`17.39/47.48/1236`, `watchdog_thread_dumps=1`), `CubicSpline.Multipoint.mapAll` stream/iterator removal (`17.45/126.93/968`, `watchdog_thread_dumps=1`), `BlendedNoise.compute` power-of-two divide-to-multiply rewrite (`17.50/90.04/2376`, `watchdog_thread_dumps=1`), `FindTopSurface` thread-local scratch context (`17.67/59.76/2449`, no watchdog/sync-load), `NoiseChunk.preliminarySurfaceLevel` quart-mask rewrite (`15.83/108.32/2280`, no watchdog/sync-load), `PerlinNoise` active-octaves arrays (`16.76/138.50/1126`, `watchdog_thread_dumps=1`), `NoiseChunk.wrap` fastutil load factor `0.95F` (`16.85/74.43/1020`, `watchdog_thread_dumps=1`), lazy `NoiseChunk` blend caches (`16.02/65.09/562`, no watchdog/sync-load but only `online_max=34`), `Climate.Sampler` combined `SampleState` ThreadLocal (`16.91/96.16/1993`, no watchdog/sync-load), config-only `PAPER_CHUNK_IO_THREADS=2` under the pinned 6-CPU gate (`16.96/74.18/861`, `watchdog_thread_dumps=1`), config-only unlimited chunk load/send/gen rates (`17.16/42.69/1565`, no watchdog/sync-load, but lower TPS/chunk coverage), `ImprovedNoise.gradDot` inline (`17.37/103.93/2312`, no watchdog/sync-load), `Mth.lerp2/lerp3` inline arithmetic (`18.02/43.93/1625`, no watchdog/sync-load, but lower TPS and much lower chunk coverage than accepted baseline), `SurfaceRules.SequenceRule` indexed iteration (`18.79/38.68/1216`, `watchdog_thread_dumps=1`), `PalettedContainer.reencodeContents` `ZeroBitStorage` fast path (`16.32/112.44/1430`, `watchdog_thread_dumps=1`, `sync_load_stack_hits=1`), and spectator no-sync-load movement path without `PlayerMoveEvent` listeners (`17.16/50.81/1266`, no watchdog/sync-load, but below accepted TPS/chunk coverage). Artifact после plugin-remapper SHA cache reuse пересобран на `909 patches` и прошел plugin matrix (`Done (33.147s)`). Узкий hash microbenchmark для 11 real plugin jars / `36.25 MiB` улучшился с old two-pass `182.522 ms best` до one-pass parallel `25.707 ms best`, но это не заявляется как end-to-end startup win. Текущий gate 2026-05-07 дополнительно исправил `scripts/precompute_plugin_remaps.sh`: build harness теперь корректно принимает reversed mappings, уже загруженные из precomputed cache, полный `MC_EULA_AGREE=true ./scripts/build_optimized.sh` снова PASS, а pinned plugin matrix PASS на `Done (52.172s)`. Clean 50-bot load gate в этот момент не запускался: на хосте параллельно работала тяжелая live-нагрузка (`java --add-modules` около `410% CPU`, load average `18.67` на 12 CPU).

Позже в том же цикле добавлен exact-SHA precomputed skip cache для plugin jars, которым ремап не нужен. Precompute stage пишет `precomputed_plugin_skips=7`, production path принимает только `mappingsHash + sha256(plugin.jar)`, full build PASS, pinned plugin matrix PASS на `Done (32.401s)`. Контроль без `skipped-hashes.txt` был `Done (29.630s)`, поэтому end-to-end startup speedup не заявляется; это пока startup-work reduction с функциональной совместимостью, а не доказанный общий boot win.

Текущий следующий цикл убрал ещё один узкий startup overhead в plugin remapper: batch plugin-directory/extra-plugin cache miss теперь передаёт уже вычисленный SHA-256 в `index.input(...)` и `index.skip(...)`, вместо повторного полного чтения jar. Затем `Hashing.sha256(InputStream)` переведён с `IOUtils.toByteArray(...)` на streaming SHA-256 через Guava `Hasher`, чтобы не делать полную `byte[]` копию mappings stream. Полный `MC_EULA_AGREE=true ./scripts/build_optimized.sh` прошёл: `applySourcePatches` применил `909 patches`, `createMojmapBundlerJar` успешен, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=7`, AppCDS пересоздан. Pinned plugin matrix прошёл на `Done (32.998s)`, protocol `773`, `CompatProbe` join/events/scheduler/command pass, fresh index `hashes=4`, `skippedHashes=7`. Restart/recovery на том же matrix world прошёл на `Done (18.470s)`, `COMPAT_PROBE command=ok events=2 ownServices=0`, `Saved the game`, clean disable, region files present. Это считается startup-work/memory reduction в production path, но не заявляется как end-to-end startup speedup: текущий clean 50-bot gate снова заблокирован preflight до запуска Minecraft (`load1=16.80`, `load_per_cpu=1.400`, `idle_percent_1s=1.42`, live Java около `508% CPU`).

Последний classpath/remap цикл расширил эту оптимизацию на Paper plugin library path. `PluginRemapper.remapLibraries(...)` теперь считает SHA-256 jar-библиотек пачкой и передаёт hash в `getIfPresent/input/skip`, а `UnknownOriginRemappedPluginIndex` сохраняет cleanup-семантику `used` для hash-aware методов. Precompute cache получил отдельный namespace `plugin-remaps/<mappingsHash>/libraries/skipped-hashes.txt`, поэтому library skip decisions не смешиваются с plugin skip decisions. Добавлен реальный `LibraryProbe` Paper plugin с отдельной `library-probe-dep.jar`; matrix подтвердил загрузку через `PluginClasspathBuilder`/`JarLibrary`: `LIBRARY_PROBE dependency=loaded-from-plugin-library`. Полный build PASS, precompute сообщил `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_remaps=0`, `precomputed_library_skips=1`; итоговый pinned plugin matrix PASS на `Done (45.204s)` на сильно загруженном host, restart/recovery PASS на `Done (22.182s)`, forced-ticket persistence PASS. Это принято как startup/classpath work reduction and coverage improvement, но не как end-to-end startup speedup: clean load gate снова заблокирован preflight (`load1=30.42`, `load_per_cpu=2.535`, `idle_percent_1s=7.91`).

Первый waypoint цикл добавил два совместимых CPU hot-path сокращения для locator-bar/player-waypoint нагрузки: `EntityAzimuthConnection` больше не создаёт временные `Vec3` для `subtract(...).rotateClockwise90()`, а считает тот же `atan2(dx, -dz)` напрямую; `isReallyFar(...)` получил точный axis early-out перед старым `sqrt`-расчётом, когда любая ось уже дальше `332` блоков. Полный build PASS: `applySourcePatches` применил `909 patches`, `createMojmapBundlerJar` успешен, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Тогда pinned plugin matrix PASS на `Done (33.100s)`, `PlayerJoinEvent sequence=3`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library`; restart/recovery PASS на `Done (19.406s)`, `Saved the game`; forced-ticket persistence PASS (`17.827s` first boot, `13.269s` restart). Один noisy join-smoke с 1 spectator bot прошёл (`online_max=1`, `bot_connected_max=1`, `server_join_events=1`, без watchdog/sync-load), но performance verdict по waypoint-правке остался открыт: strict load preflight отказал до запуска Minecraft из-за busy host (`load1=14.13`, `load_per_cpu=1.177`, лимит `0.750`, live Java `-Xmx28G` и rcon-loop).

Continuation 2026-05-07 добавил второй безопасный guard в тот же waypoint hot path: `isAtOrBeyondRange(...)` и `isReallyFar(...)` теперь возвращают `false` без `sqrt`, когда все оси находятся внутри половины range, а все граничные/спорные случаи остаются на старом float `sqrt` сравнении. Сначала правка accidentally была внесена в generated source и была сброшена `applyPatches`; затем она перенесена в source-patch файл и подтверждена через `rg halfRange` в generated source после полной сборки. Full build PASS: `applySourcePatches` применил `909 patches`, `createMojmapBundlerJar` успешен, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Latest pinned plugin matrix PASS на `Done (32.097s)`, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library`; restart/recovery PASS на `Done (22.366s)`, `Saved the game`; forced-ticket persistence PASS (`18.208s` first boot, `20.204s` restart). Noisy 2-bot survival smoke прошёл (`online_max=2`, `bot_connected_max=2`, `bot_ready_max=2`, `bot_active_max=2`, kicks/errors `0`, watchdog/sync-load `0`). Clean 50-bot verdict всё ещё blocked: strict preflight exit `75`, `load1=19.86`, `load_per_cpu=1.655`, `idle_percent_1s=32.80`, limit `0.750`, live Java около `406% CPU`.

Следующий remapper/classpath цикл убрал лишний старт mappings/reversed-mappings для cache-miss jar, которые после manifest inspection оказываются skip-only Paper plugins или plugin libraries без namespace. Раньше callers запускали `reversedMappingsFuture()` до чтения manifest; теперь mappings стартуют только в actual remap path после skip checks. Это сохраняет behavior для реально remap-нутых jars и убирает startup work для first-run skip-only plugin/library path. Full build PASS: `applySourcePatches` применил `909 patches`, `compileJava`, `createMojmapBundlerJar`, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Latest pinned plugin matrix PASS на `Done (32.836s)`, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library`; restart/recovery PASS на `Done (26.519s)`, `Saved the game`; forced-ticket persistence PASS (`21.304s` first boot, `17.690s` restart). Targeted skip-only run без precomputed remap/skips properties и с `Paper.PluginRemapperDebug=true` подтвердил: `LibraryProbe` plugin и `library-probe-dep.jar` были skipped как no-namespace, `LIBRARY_PROBE dependency=loaded-from-plugin-library`, и `mapping_load=not_started_for_skip_only`. Это accepted startup-work reduction, но не end-to-end startup speedup claim: current strict load preflight всё ещё blocked (`load1=12.00`, `load_per_cpu=1.000`, `idle_percent_1s=12.42`, live Java около `467% CPU`).

Последний continuation цикл убрал дублирующий `PaperReflection` startup map: `strippedMethods` больше не копируются в отдельный `Map<className, strippedMethods>`, а берутся напрямую из уже загруженного `ObfHelper.ClassMapping`. Следом recursive method reflection lookup стал строить `name + descriptor` key один раз на внешний вызов, а не заново на каждом superclass/interface уровне; методы без параметров теперь возвращают JVM descriptor `()` без `StringBuilder`. Это не меняет reflection remap semantics и уменьшает работу/память в plugin reflection bridge. После этого `./gradlew rebuildPatches --no-daemon` PASS (`Rebuilt 909 patches`, `Saved modified patches (34/37) for java`), затем full build PASS: `applySourcePatches` применил `909 patches`, `compileJava`, `createMojmapBundlerJar`, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Pinned plugin matrix PASS на `Done (40.233s)` на busy host, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library`; restart/recovery PASS на `Done (21.118s)`, `Saved the game`; forced-ticket persistence PASS (`17.024s` first boot, `11.387s` restart). Clean 50-bot/500-bot load verdict снова blocked before Minecraft start: strict preflight exit `75`, `load1=14.53`, `load_per_cpu=1.211`, `idle_percent_1s=71.39`, live Java `-Xmx28G` около `135% CPU`.

Следующий remapper/classpath micro-cycle добавил fixed-capacity `ArrayList` там, где размер batch уже известен: `remapLibraries`, `rewriteExtraPlugins`, `rewritePluginDirectory`, `waitForAll`, `RemappedPluginIndex.getAllIfPresent`. Это не меняет порядок, cache keys, skip/remap semantics или plugin lifecycle; цель - убрать мелкие resize/copy на startup/classpath path. Full build PASS: `applySourcePatches` применил `909 patches`, `compileJava`, `createMojmapBundlerJar`, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Pinned plugin matrix PASS на `Done (44.904s)` на busy host; `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library`; restart/recovery PASS на `Done (31.857s)`, `Saved the game`; forced-ticket persistence PASS (`16.734s` first boot, `10.518s` restart). Matrix log contained a non-fatal `PaperVersionFetcher` GitHub `403` during version checking. Clean load gate still blocked before Minecraft start: `load1=16.56`, `load_per_cpu=1.380`, `idle_percent_1s=69.43`, live Java `-Xmx28G` около `363% CPU`.

Следующий remapper/hash micro-cycle добавил expected-capacity helper для `HashMap`/`HashSet` в `RemappedPluginIndex.hashInputs(...)` и `getAllIfPresent(...)`, чтобы known-size exact-SHA hash/cache sets не ресайзились на заполнении из-за Java load factor. Cache keys, cleanup, skip/remap decisions and plugin order are unchanged. Full build PASS: `applySourcePatches` применил `909 patches`, `compileJava`, `createMojmapBundlerJar`, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Pinned plugin matrix PASS на `Done (39.019s)` на busy host; `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library`; restart/recovery PASS на `Done (38.650s)`, `Saved the game`; forced-ticket persistence PASS (`29.097s` first boot, `21.599s` restart). Clean load gate still blocked before Minecraft start: `load1=39.62`, `load_per_cpu=3.302`, `idle_percent_1s=19.75`, live Java `-Xmx28G` около `396% CPU` plus mineflayer probe.

Continuation 2026-05-07 проверил DivineMC/Velocity-style branch-expanded `VarInt.write`/`VarLong.write`. Алгоритмическая equivalence проверка на 200k random int/long values прошла, временный artifact собирался и проходил plugin matrix (`Done (31.449s)`), restart/recovery (`Done (19.962s)`) и forced-ticket persistence. Но узкий Netty `ByteBuf` microbench на этом CPU показал регресс: `varint_old_best_ms=5.326`, `varint_new_best_ms=5.992` (`0.889x`), `varlong_old_best_ms=6.844`, `varlong_new_best_ms=8.250` (`0.830x`). Поэтому `VarLong.java.patch` удалён, `VarInt.java.patch` возвращён к Paper two-case write path, production artifact пересобран на `909 patches`. Финальная проверка после отката PASS: build successful, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, plugin matrix `Done (56.876s)` на busy host, restart/recovery `Done (32.770s)`, forced-ticket persistence PASS (`22.788s` first boot, `20.056s` restart). Clean load gate снова blocked до запуска Minecraft: `load1=36.34`, `load_per_cpu=3.028`, `idle_percent_1s=8.52`, live Java около `303% CPU` plus mineflayer probe.

Следующий hash micro-cycle добавил `bench/hash/HashPathBench.java` и проверил Guava vs direct `MessageDigest` на 13 real plugin/library jars (`38,017,023` bytes). Результат разделённый: `Path` hashing через Guava быстрее (`105.405 ms` vs direct `120.725 ms`, поэтому `Hashing.sha256(Path)` оставлен без изменений), а `InputStream` hashing через direct `MessageDigest` быстрее (`guava_stream_best_ms=127.334`, `direct_stream_best_ms=119.580`, `1.065x`). Production change ограничен только `Hashing.sha256(InputStream)`: 64 KiB buffer + `MessageDigest` + uppercase `HexFormat`, exact uppercase SHA-256 string preserved. Full build PASS, plugin matrix PASS `Done (46.867s)`, restart/recovery PASS `Done (29.418s)`, forced-ticket persistence PASS (`16.933s` first boot, `11.230s` restart). Clean load gate still blocked before Minecraft start: `load1=14.23`, `load_per_cpu=1.186`, `idle_percent_1s=73.87`, live Java около `324% CPU`.

Следующий remapper index cleanup cycle сделал `RemappedPluginIndex.getAllIfPresent(...)` ленивее на стабильном cached startup path. Когда число cached/skip entries совпадает с числом текущих input jars, метод больше не строит `HashSet` всех входных SHA и не проходит cleanup перед lookup; cleanup всё ещё запускается при изменении размера набора plugins, cache miss или precomputed install, поэтому stale entries удаляются до remap/miss fallback или сразу после precomputed install. Редкий duplicate-content all-cached batch может отложить удаление лишнего stale file до следующего size-change/miss cleanup, но cache keys, plugin order, skip/remap decisions and classloading are unchanged. Узкий microbench `scripts/bench_remapper_index_cleanup.sh` на размере текущей matrix (`12` inputs, `4` remapped, `8` skipped, `5,000,000` iterations) показал old eager cleanup `2060.532 ms` vs new lazy count-check path `626.871 ms`, `3.287x`. После `./gradlew rebuildPatches --no-daemon` PASS (`Rebuilt 909 patches`, `Saved modified patches (34/37) for java`) финальный full build PASS: `applySourcePatches` применил `909 patches`, `compileJava`, `createMojmapBundlerJar`, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, AppCDS пересоздан. Pinned plugin matrix PASS на `Done (44.640s)`, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`; restart/recovery PASS на `Done (27.677s)`, `Saved the game`; forced-ticket persistence PASS (`15.625s` first boot, `11.293s` restart). Clean 50/500-bot load verdict всё ещё blocked before Minecraft start: strict preflight exit `75`, `load1=17.18`, `load_per_cpu=1.432`, `idle_percent_1s=17.19`, live Java около `388% CPU`.

Следующий remapper index write cycle добавил dirty flag для `RemappedPluginIndex`: unchanged `.paper-remapped/*/index.json` больше не переписывается на shutdown/restart, но write остаётся обязательным для нового index, mappings hash mismatch, precomputed remap/skip install, remap/skip recording и cleanup removals. `UnknownOriginRemappedPluginIndex` помечает dirty при clean removal, поэтому cleanup semantics сохранены. После `./gradlew rebuildPatches --no-daemon` PASS и full build PASS plugin matrix прошёл на `Done (36.844s)`, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`; restart/recovery PASS на `Done (26.059s)`, forced-ticket persistence PASS (`16.056s` first boot, `12.553s` restart). Targeted dirty-write check запустил второй restart на том же `runs/plugin-matrix` и сравнил mtime четырёх index files; `reports/remapper-index-dirty-write-check.txt` зафиксировал `remapper_index_mtime_unchanged=PASS` и restart `Done (18.663s)`. Clean 50/500-bot load verdict всё ещё blocked before Minecraft start: strict preflight exit `75`, `load1=13.62`, `load_per_cpu=1.135`, `idle_percent_1s=42.74`.

Следующий ReobfServer/precomputed remap cycle убрал лишнюю загрузку reobf mappings в first-run plugin-remap path, когда precomputed remapped-server jar уже доступен. `ReobfServer.load()` теперь сначала пробует установить `paper.precomputedRemapClasspathDir/<mappingsHash>.jar`; только если precomputed jar отсутствует, он запускает обычную загрузку mappings и server remap. Cleanup каталога `remap-classpath` сохранён перед фактической установкой precomputed jar и перед обычным remap, поэтому stale classpath jar semantics не меняются. После `./gradlew rebuildPatches --no-daemon` PASS и full build PASS targeted run без precomputed plugin-remaps, но с precomputed server/remapped reversed mappings подтвердил: `install_precomputed_server_count=1`, `loading_precomputed_reversed_count=1`, `loading_reobf_mappings_count=0`, `compatprobe_plugin_remap_count=1`, `reobf_precomputed_mapping_check=PASS`. Pinned plugin matrix PASS на `Done (32.332s)`, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`; restart/recovery PASS на `Done (19.249s)`, `Saved the game`; forced-ticket persistence PASS (`17.516s` first boot, `12.120s` restart). Это accepted startup-work reduction для first-run remap path, но не end-to-end startup speedup claim: clean 50/500-bot load verdict всё ещё blocked before Minecraft start (`load1=14.50`, `load_per_cpu=1.209`, `idle_percent_1s=8.52`).

Следующий precomputed install cycle заменил byte-for-byte copy готовых remap artifacts на atomic hard-link-or-copy helper. `AtomicFiles.atomicLinkOrCopy(...)` сначала создаёт hard link во временный файл рядом с destination и затем делает atomic move; если hard links недоступны или source на другом filesystem, fallback остаётся прежним copy path. Это сохраняет plugin-visible destination paths (`.paper-remapped/<original-plugin-name>.jar` и `.paper-remapped/remap-classpath/<mappingsHash>.jar`) и не меняет classloading/order/cache semantics, но на одном filesystem убирает копирование precomputed remapped server jar и precomputed remapped plugin jars. После `./gradlew rebuildPatches --no-daemon` PASS и full build PASS pinned plugin matrix PASS на `Done (47.750s)` на сильно загруженном host, `PlayerJoinEvent sequence=3`, `PlayerQuitEvent sequence=4`, `COMPAT_PROBE command=ok events=4`; `reports/precomputed-hardlink-check.txt` подтвердил `precomputed_plugin_hardlink_check=PASS` для 4 plugin jars; `reports/server-remap-hardlink-check.txt` подтвердил `server_remap_hardlink_check=PASS`, `server_remap_samefile=true`, same inode `11087432`, `loading_reobf_mappings_count=0`, `compatprobe_plugin_remap_count=1`; restart/recovery PASS на `Done (41.970s)`, `Saved the game`; forced-ticket persistence PASS (`29.763s` first boot, `23.963s` restart). Это accepted startup/disk I/O reduction, но не end-to-end speedup claim: clean 50/500-bot load verdict снова blocked before Minecraft start (`load1=41.43`, `load_per_cpu=3.452`, `idle_percent_1s=11.09`, live Java plus mineflayer probe).

Последний plugin-directory scan cycle заменил плоский обход `plugins/` с `Files.walk(..., depth=1)` на `Files.list(...)` с явным закрытием stream, убрал no-op provider path при пустом `--add-plugin`, а также убрал маленькие stream/Formatter allocation paths при обработке add-plugin/logging. Совместимость сохранена: remap keys, plugin paths, classloader URLs, event order, scheduler/services semantics не менялись. Узкий microbench на реальной `plugins/matrix` показал `walk_depth1_best_ms=220.139`, `list_best_ms=123.419`, `list_speedup=1.784x`. После `./gradlew rebuildPatches --no-daemon` PASS и full build PASS pinned plugin matrix PASS на `Done (39.186s)`, restart/recovery PASS на `Done (27.825s)`, forced-ticket persistence PASS (`16.175s` first boot, `11.521s` restart), `sha256sum -c reports/artifact-hashes.txt` PASS. Clean 50-bot 32/32 gate снова остановлен до запуска Minecraft строгим preflight (`load1=19.50`, `load_per_cpu=1.625`, лимит `0.750`), поэтому это accepted plugin-discovery work reduction, но не end-to-end cold-start/TPS claim.

Следующий Paper plugin metadata cycle заменил `stream/filter/map/toList` в `PaperPluginMeta` dependency accessors на прямые циклы с lazy `ArrayList` allocation и `List.copyOf(...)`, сохранив immutable return list и порядок обхода dependency map. Также `ServerPluginProviderStorage.processProvided(...)` больше не использует `String.format` для обычного onLoad log. После этого dependency lists начали кэшироваться внутри `PaperPluginMeta`: повторные вызовы во время load order/classloader/dump paths больше не пересобирают одинаковые immutable lists. Узкий synthetic microbench того же algorithm shape показал old stream `1960.882 ms`, direct loop `566.406 ms`, cached path `5.926 ms`, `cached_vs_loop_speedup=95.586x`. После `./gradlew rebuildPatches --no-daemon` PASS и full build PASS pinned plugin matrix PASS на `Done (32.283s)`, restart/recovery PASS на `Done (18.863s)`, forced-ticket persistence PASS (`15.387s` first boot, `9.792s` restart). Clean 50-bot 32/32 gate снова остановлен preflight (`load1=11.50`, `load_per_cpu=0.958`, limit `0.750`), поэтому это accepted plugin metadata startup/dependency-resolution work reduction, но не end-to-end cold-start/TPS claim.

Следующий Bukkit/Spigot load-order cleanup убрал временный `HashSet` из `SpigotLoadOrderConfiguration`: проверка, ссылается ли dependency provider обратно на текущий plugin, теперь делает прямой `contains(...)` по hard/soft dependency lists. Это сохраняет тот же union-membership результат, не меняет load order rules и уменьшает allocation work на startup. Узкий microbench показал old `HashSet` path `2714.681 ms`, new direct `contains` path `423.704 ms`, `contains_speedup=6.407x`. После `./gradlew rebuildPatches --no-daemon` PASS и full build PASS pinned plugin matrix PASS на `Done (30.637s)`, restart/recovery PASS на `Done (17.714s)`, forced-ticket persistence PASS (`14.350s` first boot, `9.685s` restart). Clean 50-bot 32/32 gate снова остановлен preflight (`load1=10.14`, `load_per_cpu=0.845`, `idle_percent_1s=37.74`, limit `0.750`), поэтому это accepted load-order allocation reduction, но не end-to-end cold-start/TPS claim.

Следующий plugin-loading allocation cleanup заранее задаёт capacity для startup maps/lists в `ModernPluginLoadingStrategy` и `LegacyPluginLoadingStrategy`, а Spigot/Paper dependency validation paths лениво создают missing-dependency collections только при реальном miss. Затем `LegacyPluginLoadingStrategy` получил такой же lazy allocation для temporary missing-hard-dependency `HashSet` внутри hard-dependency scan. Load-order rules, dependency order, plugin lifecycle, classloading and event semantics не менялись. Узкий synthetic startup-shape bench `scripts/bench_plugin_loading_allocations.sh` показал setup path old default-capacity `371.559 ms` vs new pre-sized `233.823 ms` (`1.589x`), legacy missing-set scan `319.678 ms` vs `321.456 ms` (`0.994x`, no speed claim), validate-no-miss `248.706 ms` vs `232.648 ms` (`1.069x`). Поэтому заявляется narrow setup/allocation reduction, а legacy missing-set subpath только как neutral allocation-pressure cleanup. После `./gradlew rebuildPatches --no-daemon` PASS (`Rebuilt 909 patches`), full build PASS, pinned plugin matrix PASS на `Done (29.708s)`, restart/recovery PASS на `Done (19.566s)`, forced-ticket persistence PASS (`16.276s` first boot, `10.644s` restart), `sha256sum -c reports/artifact-hashes.txt` PASS. Clean 50-bot 32/32 gate снова blocked before Minecraft start: `load1=15.76`, `load_per_cpu=1.313`, live Java server load present.

Последний Spigot load-after construction pre-size cycle расширил предыдущую load-order работу: `SpigotLoadOrderConfiguration` теперь создаёт `loadAfter` с capacity `depend.size + softDepend.size`, вместо default-capacity `ArrayList`. Это не меняет dependency order, lifecycle, classloading, scheduler, services или event semantics. Расширенный `scripts/bench_spigot_load_order.sh` показал `old_load_after_build_best_ms=146.978`, `new_load_after_presized_build_best_ms=121.139`, `load_after_build_speedup=1.213x`; уже принятый back-reference path на этом же прогоне показал `old_hashset_best_ms=2631.046`, `new_contains_best_ms=409.024`, `contains_speedup=6.433x`. После `./gradlew rebuildPatches --no-daemon` PASS (`Rebuilt 909 patches`, `Saved modified patches (34/37)`), full build PASS, pinned plugin matrix PASS на `Done (28.874s)`, restart/recovery PASS на `Done (19.041s)`, forced-ticket persistence PASS (`14.910s` first boot, `10.688s` restart). Strict 50-bot 32/32 gate снова blocked before Minecraft start: `load1=9.95`, `load_per_cpu=0.829`, limit `0.750`, live Java server load present. Это accepted narrow load-order allocation reduction, не end-to-end cold-start/TPS claim.

Предыдущий `TopographicGraphSorter` capacity cycle pre-size'ит `sorted`, `roots` и `nonRoots` по известному `graph.nodes().size()` в plugin load-order topological sort. Порядок graph traversal, cycle detection, dependency rules, classloading, scheduler, services и event semantics не менялись. Узкий synthetic DAG benchmark `scripts/bench_topographic_sort.sh` показал `old_default_capacity_best_ms=633.295`, `new_presized_best_ms=428.129`, `presized_speedup=1.479x`. После `./gradlew rebuildPatches --no-daemon` PASS, full build PASS, pinned plugin matrix PASS на `Done (28.578s)`, restart/recovery PASS на `Done (17.361s)`, forced-ticket persistence PASS (`13.774s` first boot, `9.489s` restart). Strict 50-bot 32/32 gate прошёл запуск и завершился без kicks/errors/watchdog/sync-load, но не стал новым baseline: `tps1_avg=16.93`, `avg_tick_ms_avg=145.06`, `loaded_chunks_max=2005`, preflight был `load1=8.56`, `load_per_cpu=0.713`, `idle_percent_1s=65.59`. Это accepted startup/load-order allocation reduction по microbench и compatibility gates, не end-to-end TPS claim.

Текущий 2026-05-08 cycle добавил `0038-Reuse-PalettedContainer-reencode-scratch-buffer`: `PalettedContainer.reencodeContents(...)` больше не выделяет новый `int[4096]` на каждый unpack/remap, а использует per-thread scratch buffer. `SimpleBitStorage(int[])` только синхронно упаковывает входной массив в собственный `long[]`, поэтому ссылка на scratch не сохраняется. Узкий benchmark `scripts/bench_paletted_reencode_scratch.sh` подтвердил equivalence и показал old `728.576 ms` vs scratch `244.271 ms`, `2.983x`, с оценкой allocation drop с `1,966,080,000` bytes/round до `16,384` bytes/thread. Попытка объединить `SimpleBitStorage` unpack+remap в один direct packed pass была отвергнута тем же benchmark: `858.637 ms`, `0.849x` vs old and `0.284x` vs scratch; production path оставлен на scratch-only. `./gradlew rebuildPatches --no-daemon` PASS (`Saved modified patches (35/38)`), full build PASS, `sha256sum -c reports/artifact-hashes.txt` PASS, plugin matrix PASS `Done (28.810s)`, restart/recovery PASS `Done (18.243s)`, forced-ticket persistence PASS (`14.358s`/`11.217s`). Strict 50-bot 32/32 gate с scratch-only artifact прошёл без kicks/errors/watchdog/sync-load, но не стал baseline: `tps1_avg=16.82`, `avg_tick_ms_avg=154.53`, `loaded_chunks_max=2127`, preflight `load1=5.52`, `load_per_cpu=0.460`, `idle_percent_1s=62.26`.

Следующий 2026-05-08 cycle добавил visitor hooks для `DensityFunction.mapAll(...)`: default-поведение для generic visitors осталось тем же (`HolderHolder`/`Marker` создаются и передаются в `apply(...)`), а внутренние unwrapping visitors в `NoiseChunk` и `RandomState` обходят только временные `HolderHolder`/`Marker` wrappers, которые раньше сразу разворачивались обратно в `mappedFunction`/`mappedWrapped`. Узкий synthetic benchmark `scripts/bench_density_visitor_hooks.sh` показал old `481.076 ms` vs hooked `20.770 ms`, `23.162x`, и убрал `3,072,000` temporary holder + `3,072,000` temporary marker allocations в benchmark shape; equivalence guard PASS. Patch stack теперь `910` source patches; `applyPatches`, `compileJava`, full `MC_EULA_AGREE=true ./scripts/build_optimized.sh`, `sha256sum -c reports/artifact-hashes.txt`, plugin matrix, restart/recovery, forced-ticket persistence и noisy 10-bot 32/32 smoke прошли. Strict 50-bot 32/32 gate не стартовал Minecraft из-за host preflight (`load1=13.07`, `load_per_cpu=1.089` > `0.750`), поэтому load/TPS win не заявляется. Noisy 10-bot smoke с `LOAD_TEST_ALLOW_BUSY_HOST=true` дал `online_max=10`, `tps1_avg=19.55`, `avg_tick_ms_avg=31.39`, `loaded_chunks_max=1842`, kicks/errors/watchdog/sync-load `0`; это crash/regression signal, не baseline.

Текущий plugin-directory scan cycle заменил оставшийся flat `Files.list(...).stream()` в `DirectoryProviderSource` и `PluginRemapper.list(...)` на `DirectoryStream` с тем же фильтром и без изменения порядка/сортировки, remap keys, classloading, plugin lifecycle, scheduler, services или event semantics. Обновлённый `scripts/bench_plugin_scan.sh` на реальной `plugins/matrix` показал `Files.walk(depth=1)` `249.466 ms`, `Files.list` `153.480 ms`, `DirectoryStream` `132.363 ms`; это `1.160x` быстрее текущего `Files.list` path и `1.884x` быстрее old walk path в этом прогоне. Verification после production patch: `./gradlew rebuildPatches --no-daemon` PASS, `applyPatches` PASS (`910` source patches), `compileJava` PASS, full `MC_EULA_AGREE=true ./scripts/build_optimized.sh` PASS, `sha256sum -c reports/artifact-hashes.txt` PASS. Latest pinned plugin matrix PASS на `Done (30.020s)`, restart/recovery PASS на `Done (18.986s)`, forced-ticket persistence PASS (`15.655s` first boot, `10.159s` restart). Boot benchmark на `BENCHMARK_CPUSET=6-11`: vanilla `14855 ms`, stock Paper `32747 ms`, optimized jar `24342 ms`, optimized runtime `16488 ms`. Strict 50-bot 32/32 gate прошёл preflight (`load1=6.17`, `load_per_cpu=0.514`, `idle_percent_1s=77.21`) и завершился без kicks/errors/watchdog/sync-load, но не стал baseline: `tps1_avg=16.70`, `avg_tick_ms_avg=262.37`, `loaded_chunks_max=2771`, worse than accepted `18.27/47.85/2380`. Поэтому это accepted startup/plugin-discovery work reduction only, не end-to-end TPS/500-player claim.

Свежий JFR на текущем artifact (`50bots-dirstream-current-jfr`) тоже прошёл 50-bot 32/32 без kicks/errors/watchdog/sync-load. Он дал `tps1_avg=18.79`, `avg_tick_ms_avg=40.66`, но не стал accepted baseline из-за меньшего chunk coverage: `loaded_chunks_max=1835` против accepted `2380`. CPU hot methods стали ещё более noise-heavy: `ImprovedNoise.p(int)` `49.18%`, `Climate$RTree$SubTree.search(...)` `3.07%`, `ImprovedNoise.noise(...)` `2.18%`, `Aquifer$NoiseBasedAquifer.computeSubstance(...)` `1.80%`. Allocation hot sites после последних cleanup: `NoiseChunk$FlatCache.<init>` `11.49%`, `Iterators.forArrayWithPosition` `10.56%`, `LZ4BlockOutputStream.<init>` `3.68%`, `NoiseChunk.wrapMarker(...)` `3.26%`; GC pauses: `56` pauses, total pause `6.00s`, P95 `318ms`. `Iterators.forArrayWithPosition` указывает на `SurfaceRules.SequenceRule.tryApply`, но exact indexed-iteration candidate уже был измерен и отвергнут. Следующий проверенный кандидат `NoiseInterpolator` flat `double[]` slice был остановлен на microbench: old jagged `284.036 ms`, flat `286.847 ms`, `0.990x`, equivalence PASS; production не изменён.

## Архитектура

- `upstream/Paper`: Paper source branch `ver/1.21.10`, commit `8043efd4d0e5bdc9dd0cbc33e9e8bb49e6d8c012`.
- `artifacts/vanilla-1.21.10.jar`: vanilla oracle.
- `artifacts/paper-1.21.10-130.jar`: stock Paper oracle.
- `artifacts/optimized-paper-1.21.10-mojmap.jar`: rebuilt optimized bundler jar.
- `artifacts/optimized-runtime/run.sh`: direct classpath runtime with AppCDS, precomputed remap cache hooks, and optional precomputed reversed mappings.
- `plugins/matrix/`: real plugin jars plus local `CompatProbe` and `LibraryProbe`.
- `plugins/matrix-libraries/`: real jar dependencies used by plugin-library compatibility probes.
- `scripts/`: reproducible build, runtime, matrix, join, restart and benchmark harnesses.

## Implemented Optimizations

- Default new region-file compression changed from `ZLIB` to Paper's existing `LZ4` codec.
- Moonrise worker defaults use more CPU safely for chunk/IO helper pools while preserving config/system-property overrides.
- Plugin remapper mapping load is lazy; no-plugin boot no longer loads/reverses mappings.
- Plugin remapper now delays mappings/reversed-mappings startup until after manifest skip checks, so first-run skip-only Paper plugins and plugin libraries no longer start mapping load unnecessarily.
- Server reobf remap is lazy, can install a precomputed remapped server jar, and now checks that precomputed server jar before starting the expensive reobf mappings load.
- Precomputed remap artifact installs now use atomic hard-link-or-copy, preserving destination plugin/server jar paths while avoiding full file copies when the runtime cache and server run directory are on the same filesystem.
- Plugin directory discovery now uses flat `DirectoryStream` iteration instead of `Files.walk(..., depth=1)` / `Files.list(...).stream()` where the path is a non-recursive plugin directory scan. It skips the no-op `--add-plugin` provider path when no add-plugin files are present, and the small add-plugin flag/log-name startup paths avoid stream/Formatter allocation. The latest matrix-sized microbench measured old walk `249.466 ms`, `Files.list` `153.480 ms`, and `DirectoryStream` `132.363 ms` (`1.160x` faster than the current `Files.list` path in that run).
- Paper plugin metadata dependency-list accessors now use direct loops and cached immutable list results instead of rebuilding stream-derived lists on repeated access. A synthetic metadata dependency benchmark measured old stream `1960.882 ms`, direct loop `566.406 ms`, and cached path `5.926 ms` (`95.586x` faster than the loop path on repeated calls).
- Spigot load-order dependency back-reference checks no longer allocate a temporary `HashSet` per checked provider; direct hard/soft dependency-list membership checks preserve the same result. A synthetic load-order benchmark improved old `HashSet` path `2714.681 ms` to direct `contains` path `423.704 ms` (`6.407x`).
- Spigot load-order `loadAfter` list construction now pre-sizes the `ArrayList` from known hard+soft dependency counts. The latest synthetic benchmark improved `146.978 ms` to `121.139 ms` (`1.213x`) for that path, while the back-reference path measured `6.433x` in the same run.
- Plugin load-order topological sort now pre-sizes its result list, root deque, and non-root fastutil map from known graph node count. A synthetic DAG benchmark improved `633.295 ms` to `428.129 ms` (`1.479x`).
- `PalettedContainer.reencodeContents(...)` now reuses a per-thread scratch `int[]` for the temporary unpack/remap array. A dedicated equivalence microbench improved old `new int[]` path `728.576 ms` to `244.271 ms` (`2.983x`) and reduces repeated save/serialization scratch allocation pressure. The following strict 50-bot run was stable but did not beat the accepted load baseline (`16.82/154.53/2127` vs `18.27/47.85/2380`), so no end-to-end TPS claim is made.
- `DensityFunction.Visitor` now has default `applyHolder(...)` and `applyMarker(...)` hooks. Generic visitor semantics are unchanged by default, while `NoiseChunk` and `RandomState` override those hooks to avoid temporary `HolderHolder`/`Marker` wrappers they previously unwrapped immediately. A synthetic visitor benchmark measured `481.076 ms` old vs `20.770 ms` hooked (`23.162x`) and zero temporary holder/marker allocations in the hooked path; build/plugin/restart/forced-ticket/noisy 10-bot gates passed, but no 50-bot/TPS claim is made because strict preflight blocked.
- Plugin loading strategies now pre-size known-size startup maps/lists, and missing-dependency collections in Spigot/Paper provider validation/legacy scan are allocated only on actual misses. A synthetic startup-shape benchmark improved default-capacity setup from `371.559 ms` to `233.823 ms` (`1.589x`) and validate-no-miss from `248.706 ms` to `232.648 ms` (`1.069x`); the legacy missing-set scan is recorded as neutral allocation reduction only (`0.994x`).
- Direct runtime extracts Paper bundler classpath once and launches `org.bukkit.craftbukkit.Main` directly.
- AppCDS archive is regenerated on each optimized runtime build.
- Plugin remap cache can install precomputed remapped plugin jars keyed by `mappingsHash + sha256(plugin.jar)`.
- Plugin remapper can load a precomputed reversed mappings `.tiny` file from `artifacts/optimized-runtime/reversed-mappings`; measured A/B delta was too small/noisy to claim as a performance win.
- Plugin remapper reuses already computed SHA-256 hashes across the batch-cache miss fallback and hashes larger plugin batches in parallel while preserving exact `sha256(plugin.jar)` cache keys.
- Plugin remapper now also reuses that batch SHA inside the remap/skip miss path, avoiding a second full jar read when recording cache destinations or exact-SHA skip decisions for plugin directory and extra-plugin batches.
- Plugin remapper applies the same hash-aware cache path to Paper plugin libraries, with a separate precomputed `libraries/skipped-hashes.txt` namespace to avoid mixing library and plugin remap semantics.
- Plugin remapper pre-sizes known-size batch result/task lists in plugin-directory, extra-plugin, and library remap paths to avoid small startup allocation resizes without changing result order or cache semantics.
- Plugin remapper also pre-sizes known-size hash maps/sets in exact-SHA batch hash/cache checks using an expected-capacity helper that accounts for Java `HashMap`/`HashSet` load factor.
- Plugin remapper now skips eager stale-cache cleanup allocation/work on stable all-cached plugin-directory batches, while still cleaning on plugin-set size changes, cache misses, or precomputed installs.
- Plugin remapper index writes now use a dirty flag so stable cached restarts do not rewrite unchanged `.paper-remapped/*/index.json` files.
- `Hashing.sha256(InputStream)` now hashes streams incrementally instead of materializing the whole stream into a temporary byte array.
- `Hashing.sha256(InputStream)` now uses a direct `MessageDigest`/64 KiB buffer path after a real-jar microbench showed it faster than the previous Guava stream hasher; `Hashing.sha256(Path)` intentionally stays on Guava because the same bench showed direct path hashing slower.
- `ObfHelper` mapping bootstrap now pre-sizes known class/method/field maps, builds top-level obf/mojang lookup maps manually instead of stream collectors, and pre-sizes its `StringPool` backing map. The real mappings microbench improved old stream/default maps from `257.222 ms` to `196.038 ms` (`1.312x`) and improved the previous pre-sized double-pool path by `1.122x`.
- `PaperReflection` no longer builds a duplicate `Map<className, strippedMethods>` during construction; method reflection mappings are read directly from the existing `ObfHelper.ClassMapping` entries, recursive method lookup reuses one stripped method key across superclass/interface traversal, and empty-parameter descriptors return without a `StringBuilder`.
- `scripts/precompute_plugin_remaps.sh` now treats either a fresh generated reversed mappings file or an already-installed precomputed reversed mappings file as valid precompute output, and exports precomputed plugin-library skip hashes separately under `libraries/`.
- Plugin remapper can install precomputed "skip remap" decisions for exact plugin hashes via `skipped-hashes.txt`, avoiding repeated manifest/namespace inspection for jars already proven not to need remapping. This is compatibility-safe by exact SHA and mapping hash, but the noisy A/B did not prove end-to-end startup speedup.
- `WaypointTransmitter.EntityAzimuthConnection` now computes waypoint azimuth directly from entity/player coordinates instead of allocating temporary `Vec3` objects for `subtract(...).rotateClockwise90()` on connect/update.
- `WaypointTransmitter.EntityChunkConnection` now keeps the last chunk long key alongside the `ChunkPos`, avoiding repeated key recomputation for chunk-visibility checks.
- Rejected: `WaypointTransmitter` distance/inner-range guards were re-measured in `reports/waypoint-distance-guard-bench.txt` and were slower (`0.888x` range, `0.880x` really-far), so they are not in the current production patch.
- `ServerEntity.sendChanges()` now skips the motion `distanceToSqr` calculation when the current immutable `Vec3` delta movement is the exact same object as `lastSentMovement`; a focused runtime-`Vec3` bench improved the modeled identity-heavy path from `80.075 ms` to `28.626 ms` (`2.797x`). The strict 50-bot gate was stable but did not beat the accepted load baseline, so no end-to-end TPS claim is made.
- `scripts/run_load_test.sh` now has a default-on host preflight for benchmark integrity. It records load/idle/process evidence and exits `75` before starting Minecraft when the host is too busy, unless `LOAD_TEST_ALLOW_BUSY_HOST=true` is explicitly set for a known noisy run.
- `scripts/run_plugin_matrix.sh` includes a real protocol client join check using `minecraft-protocol@1.66.0`.
- Density-function hot paths reuse mutable/scratch contexts where measured safe.
- `NoiseChunk`/`RandomState` density wrapper caches use fastutil `Reference2ReferenceOpenHashMap<>(2048)` to keep identity semantics while reducing Java `IdentityHashMap` overhead.
- `PerlinNoise` copies amplitudes into a primitive `double[]` and avoids stream-based constructor validation and hot-loop `DoubleList` reads.
- `Climate.RTree` and `Climate.Sampler` reuse thread-local arrays/contexts and add a no-metric search path; repeated 50-bot runs improved TPS/loaded-chunk throughput, but average tick time regressed versus the postrevert baseline.
- `Climate.SpawnFinder` now reuses sampled `long[]` values and a `fitness(long[])` overload to avoid a per-check `TargetPoint` allocation during spawn search; the accepted effect is faster cold boot / plugin startup, not a new 50-bot load baseline.
- `NoiseChunk.NoiseInterpolator` now caches Y/X deltas and uses direct arithmetic instead of repeated `Mth.lerp` calls in the update path; the completed patch passed `applySourcePatches`, full optimized build, plugin matrix, and a 50-bot 32/32 load rerun.
- `ServerGamePacketListenerImpl` now rate-limits `moved too quickly` warnings with a shared tick gate, cutting warning spam on the latest 50-bot 32/32 run from `911` to `1` without changing movement validation or teleport behavior.
- `network.optimize-non-flush-packet-sending` / Netty `lazyExecute` was wired behind a default-off Paper config flag and measured against the real plugin matrix; the on-state regressed the 50-bot 32/32 load to `tps1_avg=16.31`, `avg_tick_ms_avg=80.82`, so the toggle remains experimental and disabled by default.
- Config-only generational ZGC was measured and rejected for the current 50-bot 32/32 gate (`15.71/203.15/1604`, `watchdog_thread_dumps=2`), so JVM defaults were not changed.

Rejected in this cycle:

- `PrepareSpawnTask` playerdata cache during spawn preparation regressed the 50-bot 32/32 run to `tps1_avg=16.98`, `avg_tick_ms_avg=96.49`, `loaded_chunks_max=3487`, so the patch was deleted.
- `LinearPalette.idFor` reference-map cache was fixed into the correct source-patch layer and measured, but it regressed the 50-bot 32/32 run to `tps1_avg=14.53`, `avg_tick_ms_avg=82.04`, `loaded_chunks_max=2760`, so the patch was deleted.
- `Reference2ReferenceOpenHashMap<>(4096)` reduced no-crash risk in one run but regressed plugin startup and 50-bot load (`tps1_avg=16.09`, `avg_tick_ms_avg=70.68`), so the source was restored to `2048`.
- `ImprovedNoise` `byte[]` -> `int[]` permutation table was built and tested, but the 50-bot run regressed from `tps1_avg=17.93` / `avg_tick_ms_avg=45.51` to `tps1_avg=15.35` / `avg_tick_ms_avg=53.84`, and the load-run startup regressed from `Done (40.292s)` to `Done (49.454s)`. The experiment was reverted.
- `ImprovedNoise` direct-masking reduction with duplicated tail was built and tested, but the 50-bot run regressed to `tps1_avg=16.40` / `avg_tick_ms_avg=75.99` with `watchdog_thread_dumps=1` and `sync_load_stack_hits=1`. The experiment was reverted.
- `NoiseChunk.forIndex` integer fast-div rewrite was built and tested, but the control rerun after revert measured `tps1_avg=16.73` / `avg_tick_ms_avg=56.58` / `loaded_chunks_max=805` with `watchdog_thread_dumps=1`. The experiment was reverted to keep the accepted `floorMod`/`floorDiv` path.
- `network.optimize-non-flush-packet-sending` / Netty `lazyExecute` regressed the 50-bot load under the real plugin matrix to `tps1_avg=16.31` / `avg_tick_ms_avg=80.82`, so it stays default-off and is not accepted for the production path.
- Branch-expanded `VarInt.write`/`VarLong.write` was rejected after a direct Netty `ByteBuf` microbench regressed the write path (`VarInt 0.889x`, `VarLong 0.830x`); the temporary production patch was removed and the final artifact was rebuilt from the previous Paper VarInt path.
- `NoiseChunk.NoiseInterpolator` filling-cell fraction lookup tables compiled and passed plugin matrix, but regressed the 50-bot 32/32 load to `tps1_avg=17.47`, `avg_tick_ms_avg=82.04`, `loaded_chunks_max=4692`, so the experiment was reverted.
- `Aquifer` air-constant / cached-fluid read micro-optimization compiled and passed plugin matrix, but regressed the 50-bot 32/32 load to `tps1_avg=17.94`, `avg_tick_ms_avg=81.51`, `loaded_chunks_max=4644`, so the experiment was reverted.
- Final reverted-artifact 50-bot rerun after the Aquifer rollback came back at `tps1_avg=18.52`, `avg_tick_ms_avg=61.43`, `loaded_chunks_max=3567`, with no watchdog/sync-load hits, but the average tick time was still worse than the accepted `47.85`, so it was not promoted to a new load baseline.
- `PerlinNoise.wrap` in-range fast path was built, sanity-checked, and measured under pinned affinity. It did not beat the accepted load baseline and produced a worse `avg_tick_ms_avg=88.06` with `watchdog_thread_dumps=1`, so it was reverted.
- `BlockStateData` `Object2IntOpenHashMap` pre-sizing was built and measured for cold bootstrap, but the pinned boot benchmark regressed optimized runtime to `17784 ms` and plugin matrix to `Done (34.600s)`, so it was reverted.
- `Climate.RTree.Node` cached `parameter0..parameter6` fields compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed TPS to `17.39` and produced `watchdog_thread_dumps=1` during `save-all`, so the field-cache part was reverted.
- `CubicSpline.Multipoint.mapAll` stream/iterator removal compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed badly to `tps1_avg=17.45`, `avg_tick_ms_avg=126.93`, `loaded_chunks_max=968`, with `watchdog_thread_dumps=1`, so the patch was deleted.
- `BlendedNoise.compute` replacing division by powers of two with an explicit multiplier compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed to `tps1_avg=17.50`, `avg_tick_ms_avg=90.04`, `loaded_chunks_max=2376`, with `watchdog_thread_dumps=1` during `save-all`, so the source patch was deleted.
- `DensityFunctions.FindTopSurface` thread-local `MutableSinglePointContext` compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed against the accepted baseline to `tps1_avg=17.67`, `avg_tick_ms_avg=59.76`, `loaded_chunks_max=2449`; no watchdog/sync-load hit, but not enough to accept, so it was reverted and the artifact was rebuilt.
- `NoiseChunk.preliminarySurfaceLevel` quart-alignment bit-mask rewrite compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed to `tps1_avg=15.83`, `avg_tick_ms_avg=108.32`, `loaded_chunks_max=2280`; no watchdog/sync-load hit, but it was much worse than the accepted baseline, so it was reverted and the artifact was rebuilt.
- `PerlinNoise` active-octaves arrays compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed to `tps1_avg=16.76`, `avg_tick_ms_avg=138.50`, `loaded_chunks_max=1126`, with `watchdog_thread_dumps=1`; it was reverted and the artifact was rebuilt.
- `NoiseChunk.wrap` fastutil load factor `0.95F` compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed to `tps1_avg=16.85`, `avg_tick_ms_avg=74.43`, `loaded_chunks_max=1020`, with `watchdog_thread_dumps=1`; it was reverted and the artifact was rebuilt.
- Lazy allocation of `NoiseChunk` blend alpha/offset flat caches compiled and passed plugin matrix, but the 50-bot 32/32 gate regressed to `tps1_avg=16.02`, `avg_tick_ms_avg=65.09`, `loaded_chunks_max=562`, and only `online_max=34`; no watchdog/sync-load hit, but it was reverted and the artifact was rebuilt.
- `Climate.Sampler` combined `SampleState` ThreadLocal compiled and passed plugin matrix on the temporary build (`Done (32.711s)`), but the 50-bot 32/32 gate regressed to `tps1_avg=16.91`, `avg_tick_ms_avg=96.16`, `loaded_chunks_max=1993`; no watchdog/sync-load hit, but it was reverted and the artifact was rebuilt. Postrevert plugin matrix passed at `Done (30.549s)`.
- Config-only `PAPER_CHUNK_IO_THREADS=2` under `BENCHMARK_CPUSET=6-11` regressed the 50-bot 32/32 gate to `tps1_avg=16.96`, `avg_tick_ms_avg=74.18`, `loaded_chunks_max=861`, with `watchdog_thread_dumps=1`, so the default auto I/O-thread formula was not changed.
- `ImprovedNoise.gradDot` inline of `SimplexNoise.dot(...)` compiled and passed plugin matrix on the temporary build (`Done (30.132s)`), but the 50-bot 32/32 gate regressed to `tps1_avg=17.37`, `avg_tick_ms_avg=103.93`, `loaded_chunks_max=2312`; no watchdog/sync-load hit, but it was reverted and the artifact was rebuilt. Postrevert plugin matrix passed at `Done (33.371s)`.
- `NoiseInterpolator` flat-slice storage was benchmarked but not applied. A standalone equivalence microbench changed two `double[][]` slices into flat `double[]` slices with index `z * (cellCountY + 1) + y`; arrays per chunk dropped in the model (`1152` -> `192`), but runtime did not improve (`284.036 ms` old vs `286.847 ms` flat, `0.990x`), so production remains on the existing jagged representation.
- `Mth.lerp2/lerp3` inline arithmetic compiled and passed plugin matrix on the temporary build (`Done (29.892s)`), but the 50-bot 32/32 gate did not beat the accepted baseline: `tps1_avg=18.02`, `avg_tick_ms_avg=43.93`, `loaded_chunks_max=1625`; no watchdog/sync-load hit, but lower TPS and much lower chunk coverage made it a rejected result. The source patch was deleted, the artifact was rebuilt back to `909 patches`, and postrevert plugin matrix passed at `Done (30.460s)`.
- `SurfaceRules.SequenceRule` indexed loop over the rule list compiled and passed plugin matrix on the temporary build (`Done (31.894s)`), but the 50-bot 32/32 gate hit `watchdog_thread_dumps=1` and had much lower chunk coverage: `tps1_avg=18.79`, `avg_tick_ms_avg=38.68`, `loaded_chunks_max=1216`. The source hunk was deleted, the artifact was rebuilt back to `909 patches`, and postrevert plugin matrix passed at `Done (44.811s)`.
- `PalettedContainer.reencodeContents` `ZeroBitStorage` fast path compiled and passed plugin matrix on the temporary build (`Done (47.375s)`), but the 50-bot 32/32 gate regressed badly: `tps1_avg=16.32`, `avg_tick_ms_avg=112.44`, `loaded_chunks_max=1430`, with `watchdog_thread_dumps=1` and `sync_load_stack_hits=1`. The feature patch was deleted, the artifact was rebuilt back to `909 patches`, and postrevert plugin matrix passed at `Done (36.608s)`.
- Spectator movement no-sync-load fast path without `PlayerMoveEvent` listeners compiled and passed plugin matrix on the temporary build (`Done (42.489s)`) and removed the observed sync-load/watchdog in the 50-bot 32/32 run, but still failed the accepted baseline: `tps1_avg=17.16`, `avg_tick_ms_avg=50.81`, `loaded_chunks_max=1266`, no watchdog/sync-load. The feature patch was deleted, the artifact was rebuilt back to `909 patches`, and postrevert plugin matrix passed at `Done (31.605s)`.
- Config-only unlimited chunk load/send/gen rates (`PAPER_PLAYER_MAX_LOAD_RATE=-1`, `PAPER_PLAYER_MAX_SEND_RATE=-1`, `PAPER_PLAYER_MAX_GEN_RATE=-1`) lowered average tick time to `42.69 ms` and avoided watchdog/sync-load, but still failed the accepted load gate with `tps1_avg=17.16` and `loaded_chunks_max=1565`, so no default/config change was made.

## Environment

```text
Linux vm231moce.yourlocaldomain.com 6.8.0-110-generic x86_64
OpenJDK 21.0.10+7-Ubuntu-124.04
CPU cores: 12
RAM: 62 GiB
```

## Artifact Hashes

```text
97720a304176d0f6fa8d222a3b1374de4390aa5debc96924ecd844e12906e3ff  artifacts/optimized-paper-1.21.10-mojmap.jar
158703f75a26f842ea656b3dc6d75bf3d1ec176b97a2c36384d0b80b3871af53  artifacts/paper-1.21.10-130.jar
5bb64dc47379903e8f288bd6a4b276e889075c5c0f4c0b714e958d835c1874e7  artifacts/vanilla-1.21.10.jar
c0d00b0cbb2f7bb57f3ebf3ea07b6050932d17f328cc09b944d699e8f0315531  artifacts/optimized-runtime/app-cds.jsa
9aca08dec8295e8c41f627a5c6b84a5d6a9e9bda923d0642cefc3a5a8b77b2c5  artifacts/optimized-runtime/run.sh
e0729a2c14364b24e0381d0c6cc4fc0307816faa7fe86d5f3aaa90eade4a25d3  artifacts/optimized-runtime/runtime.jar.sha256
fe31ac71e97410f9b78e26da03a80a2c9577cb7aec06a72a809aa32691a98a66  artifacts/optimized-runtime/reversed-mappings/9383762D002E33F5BFB2E2D9BB59DBCE11135EE10227DB71E8270AB56F0AF16A.tiny
c5ccf591f1676c87dfc4ad7eefcd7b4e3de1a769ea359abc0823926d1cd1c583  artifacts/optimized-runtime/plugin-remaps/9383762D002E33F5BFB2E2D9BB59DBCE11135EE10227DB71E8270AB56F0AF16A/skipped-hashes.txt
21090b930f00d2c23d05bbc1014eba1283c27253033ea73d2caa47ea34632570  artifacts/optimized-runtime/plugin-remaps/9383762D002E33F5BFB2E2D9BB59DBCE11135EE10227DB71E8270AB56F0AF16A/libraries/skipped-hashes.txt
c6af31a9c24d9a3b71e94c0fe0fdcf6c18c7bf8aef5c095512ac65b5eceba933  plugins/matrix/LibraryProbe-0.1.0.jar
67f8733cbdcec008ec7038cae5e9199db53e00639fac8a0a2a4e86822566a8a8  plugins/matrix-libraries/library-probe-dep.jar
```

## Gate Status

| Gate | Status | Evidence |
| --- | --- | --- |
| build from scratch | PASS | `MC_EULA_AGREE=true ./scripts/build_optimized.sh`, `BUILD SUCCESSFUL`; latest run after rejected Aquifer candidate was removed applied 910 source patches and rebuilt bundler/runtime/AppCDS |
| runnable artifact | PASS | `artifacts/optimized-paper-1.21.10-mojmap.jar`, `artifacts/optimized-runtime/run.sh` |
| EULA-gate behavior | PASS | `./scripts/eula_gate_smoke.sh` |
| cold boot benchmark | PASS WITH LIMITS | latest pinned `BENCHMARK_CPUSET=6-11` optimized runtime `done_ms=16613`, stock Paper `32528`, optimized jar `24385`, vanilla `16958`; still not `<1s` and not an end-to-end startup win |
| server boots | PASS | latest post-revert plugin matrix reached `Done (32.234s)` |
| status ping | PASS | `reports/plugin-matrix-status.json`, protocol `773` |
| player join works | PASS | `PlayerJoinEvent sequence=3 detail=CodexJoinProbe` |
| world save/load works | PASS | `save-all flush`, restart/recovery check |
| commands work | PASS | `plugins`, `version`, `compatprobe`, `save-all flush` |
| permissions/services work | PASS WITH LIMITATIONS | LuckPerms/Vault/Essentials hooks loaded; deeper semantics not exhausted |
| plugin loading works | PASS WITH LIMITATIONS | tested matrix only; includes real plugin-library loading via `LibraryProbe`; no "all plugins" claim |
| scheduler works | PASS | `COMPAT_PROBE scheduler=sync/async ticked=true` |
| event dispatch order stable | PASS FOR PROBE | PluginEnableEvent -> ServerLoadEvent -> PlayerJoinEvent -> PlayerQuitEvent |
| restart/recovery stable | PASS | `MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh`, latest post-revert gate `Done (20.809s)`, `Saved the game`, clean disable |
| forced ticket persistence | PASS | `scripts/forced_ticket_persistence_check.sh` saved a forced chunk and after restart `forceload query 0 0` reported the chunk remained marked for force loading; latest post-revert first/restart boot `15.835s`/`11.609s` |
| short 50-bot load | PASS WITH LIMITS | latest strict candidate run reached all 50 bots but regressed accepted baseline: `17.14/82.71/2030` vs `18.27/47.85/2380`; candidate reverted |
| load host preflight | BLOCKED CURRENT STRICT 50-BOT | post-revert `50bots-aquifer-surface-offsets-postrevert-gate` refused before Minecraft start (`load1=12.50`, `load_per_cpu=1.041` > `0.750`, `idle_percent_1s=74.98`) |
| noisy 10-bot 32/32 smoke | PASS WITH LIMITS | forced with `LOAD_TEST_ALLOW_BUSY_HOST=true`; 10 connected/ready/active, `tps1_avg=19.17`, `avg_tick_ms_avg=36.29`, `loaded_chunks_max=1572`, kicks/errors/watchdog/sync-load `0`; non-comparable diagnostic only |
| noisy 50-bot diagnostic | PASS WITH LIMITS | forced with `LOAD_TEST_ALLOW_BUSY_HOST=true`; 50 connected/ready/active, `tps1_avg=17.84`, `avg_tick_ms_avg=38.70`, `loaded_chunks_max=139`, `watchdog_thread_dumps=1` during `save-all`; not promoted to baseline |
| measured 500-bot production gate | PASS WITH LIMITS | `reports/production-500-repeat-quorum.txt`: 3/3 preserved cold/fresh + warm-source release runs passed for 500 bots, 32/32, creative block workload, `repeat_quorum_pass=true`; this is not a multi-hour soak or real-player/general-gameplay claim |
| vanilla parity differential | NOT COMPLETE | broad mechanics differential matrix is not complete |

## Current Limits

- `<1s` real cold boot with plugins and fresh world is not achieved. Latest pinned optimized runtime boot benchmark is `17145 ms`; best previous optimized runtime benchmark is `13595 ms`.
- The measured 500-bot, 32/32, creative block profile is achieved by the
  current 3-pass repeat quorum. Stable literal `20.00 TPS`, arbitrary
  real-player gameplay, multi-hour soak, vanilla parity, and all-plugin
  compatibility are not claimed.
- The fresh spawn-search load gate improved startup but not the load baseline: `tps1_avg=17.78`, `avg_tick_ms_avg=70.20`, `loaded_chunks_max=5255`, `moved_too_quickly_warnings=2`, no watchdog/sync-load hits.
- The latest accepted 50-bot run did not hit watchdog or `ServerChunkCache.syncLoad`, but later pinned reruns were noisy and did not improve the accepted baseline.
- The latest rejected `Climate.Node` field-cache run hit a watchdog thread dump during `save-all`/chunk serialization, so it was reverted even though the average tick time was near the accepted baseline.
- The latest rejected `CubicSpline.mapAll` allocation cleanup also hit a watchdog thread dump during `save-all`/chunk light serialization and worsened average tick time, so it was reverted.
- The latest rejected `BlendedNoise.compute` arithmetic rewrite also hit a watchdog thread dump during `save-all`/chunk section copy and worsened average tick time, so it was reverted.
- The latest rejected `FindTopSurface` scratch-context rewrite avoided watchdog/sync-load hits but still worsened the accepted 50-bot TPS/MSPT baseline, so it was reverted.
- The latest rejected `preliminarySurfaceLevel` quart-mask rewrite also avoided watchdog/sync-load hits but regressed badly on TPS/MSPT, so it was reverted.
- The latest rejected `PerlinNoise` active-octaves arrays rewrite hit a watchdog thread dump and badly worsened average tick time, so it was reverted.
- The latest rejected `NoiseChunk.wrap` load-factor rewrite hit a watchdog thread dump and worsened TPS/MSPT, so it was reverted.
- The latest rejected lazy `NoiseChunk` blend-cache rewrite lowered neither TPS nor MSPT and had only `online_max=34`, so it was reverted.
- The latest rejected `Climate.Sampler` combined `SampleState` ThreadLocal avoided watchdog/sync-load hits but worsened TPS/MSPT, so it was reverted.
- The latest config-only `PAPER_CHUNK_IO_THREADS=2` gate hit a watchdog thread dump and worsened the accepted baseline, so no default change was made.
- The latest config-only unlimited load/send/gen rates run improved average tick time but still failed accepted TPS and loaded-chunk coverage, so no default change was made.
- The latest rejected `ImprovedNoise.gradDot` inline avoided watchdog/sync-load hits but badly worsened average tick time, so it was reverted.
- The latest rejected `Mth.lerp2/lerp3` inline avoided watchdog/sync-load hits and reduced average tick time in a lower-coverage run, but it failed the accepted baseline on TPS and loaded-chunk coverage, so it was reverted.
- The latest rejected `SurfaceRules.SequenceRule` indexed loop improved TPS/MSPT in a low-coverage run but hit a watchdog thread dump and loaded far fewer chunks than the accepted baseline, so it was reverted.
- The latest rejected `PalettedContainer.reencodeContents` zero-storage branch targeted a real save-serialization JFR site, but it hit watchdog/sync-load and badly worsened TPS/MSPT, so it was reverted.
- The latest rejected spectator no-sync-load path removed the specific movement sync-load stack but still worsened TPS and chunk coverage, so it was reverted.
- The latest plugin-remapper SHA cache reuse has only microbenchmark evidence so far; it is not an accepted `<1s` or end-to-end boot-speed claim until a clean A/B startup benchmark is run without the concurrent live-server CPU load.
- The latest precomputed plugin skip cache, batch-miss hash reuse, and streaming InputStream hash have functional evidence (`4` remapped hashes and `7` skipped hashes in a fresh matrix index), but no end-to-end speedup claim: skip-enabled runs include `30.330s` / `31.116s` / `32.401s` / `29.899s` / `32.998s`, while one noisy control without `skipped-hashes.txt` was `29.630s`.
- The waypoint azimuth/chunk-key changes have build/plugin/restart/forced-ticket evidence, but no accepted load-performance claim yet. The later distance/inner-range guard shape is rejected by `reports/waypoint-distance-guard-bench.txt` and is not in the current production patch.
- The latest remapper deferred-mappings change has build/plugin/restart/forced-ticket and a targeted skip-only debug run proving mapping load is not started for a no-namespace Paper plugin plus no-namespace library. It is not a clean end-to-end startup speedup claim because the host remains CPU-busy.
- The latest `PaperReflection` duplicate-map/key-reuse/empty-descriptor changes have build/plugin/restart/forced-ticket evidence. They are not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host.
- The latest remapper batch-list capacity hints have build/plugin/restart/forced-ticket evidence. They are not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host.
- The latest remapper hash collection capacity hints have build/plugin/restart/forced-ticket evidence. They are not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host.
- The latest remapper lazy index cleanup has microbench, `rebuildPatches`, build/plugin/restart/forced-ticket evidence. It is not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host (`load_per_cpu=1.432` > `0.750`).
- The latest remapper dirty index write change has `rebuildPatches`, build/plugin/restart/forced-ticket and targeted unchanged-mtime evidence. It is not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host (`load_per_cpu=1.135` > `0.750`).
- The latest ReobfServer precomputed-server-before-mappings change has `rebuildPatches`, build/plugin/restart/forced-ticket and a targeted no-precomputed-plugin-remaps run proving `loading_reobf_mappings_count=0` while CompatProbe was actually remapped. It is not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host (`load_per_cpu=1.209` > `0.750`).
- The latest atomic hard-link-or-copy install change has `rebuildPatches`, build/plugin/restart/forced-ticket and targeted same-inode evidence for 4 precomputed plugin jars plus the precomputed server remap jar. It is not a clean end-to-end startup speedup claim because strict preflight still blocks load/startup performance gates on the busy host (`load_per_cpu=3.452` > `0.750`).
- The latest plugin-directory scan/no-op add-plugin/DirectoryStream change has a narrow real-matrix microbench (`132.363 ms` vs current `Files.list` `153.480 ms`, `1.160x`), `rebuildPatches`, build/plugin/restart/forced-ticket and artifact-hash evidence. It is not an accepted TPS/load claim because the strict 50-bot gate failed the accepted baseline (`16.70/262.37/2771` vs `18.27/47.85/2380`).
- The latest Paper plugin metadata dependency-list cache has a synthetic microbench (`5.926 ms` cached vs `566.406 ms` direct loop for repeated calls), `rebuildPatches`, build/plugin/restart/forced-ticket and artifact-hash evidence. It is not a clean end-to-end startup speedup claim because strict 50-bot preflight still blocks load gates on the busy host (`load_per_cpu=0.958` > `0.750`).
- The latest Spigot load-order dependency back-reference cleanup has a synthetic microbench (`6.407x` faster than allocating a temporary `HashSet` per provider check), `rebuildPatches`, build/plugin/restart/forced-ticket and artifact-hash evidence. It is not a clean end-to-end startup speedup claim because strict 50-bot preflight still blocks load gates on the busy host (`load_per_cpu=0.845` > `0.750`, `idle_percent_1s=37.74` < `40.00`).
- The latest Spigot load-after pre-size cleanup has a synthetic microbench (`1.213x` faster load-after build, `6.433x` faster back-reference path in the same bench), `rebuildPatches`, build/plugin/restart/forced-ticket/hash evidence. It is not a clean end-to-end startup speedup claim because strict 50-bot preflight blocked before Minecraft start (`load_per_cpu=0.829` > `0.750`).
- The latest `TopographicGraphSorter` capacity pre-size cleanup has a synthetic microbench (`1.479x` faster synthetic DAG sort), `rebuildPatches`, build/plugin/restart/forced-ticket/hash evidence. It is not a TPS/load claim; the following strict 50-bot gate ran but failed the accepted baseline (`16.93/145.06/2005` vs `18.27/47.85/2380`).
- The latest `PalettedContainer.reencodeContents` scratch reuse has an equivalence microbench (`2.983x` faster synthetic reencode and far less temporary `int[]` allocation), `rebuildPatches`, build/plugin/restart/forced-ticket/hash evidence, and one strict 50-bot run without crashes/watchdog/sync-load. It is not an accepted TPS/load claim because that run failed the accepted baseline (`16.82/154.53/2127` vs `18.27/47.85/2380`).
- The latest `DensityFunction.Visitor` holder/marker hook change has a synthetic microbench (`23.162x` in the visitor-unwrapping shape and zero temporary wrapper allocations), `applyPatches`, `compileJava`, build/plugin/restart/forced-ticket/hash evidence, and one noisy 10-bot 32/32 smoke without crashes/watchdog/sync-load. It is not an accepted 50-bot/TPS claim because strict 50-bot preflight blocked before Minecraft start (`load_per_cpu=1.089` > `0.750`).
- The latest DirectoryStream plugin-directory scan change has a current boot benchmark (`optimized-runtime done_ms=16488`, stock Paper `32747`), but no `<1s` or clean cross-run startup claim is made because startup timings remain host-sensitive and the end-to-end target is still far away.
- The latest JFR gate confirms the next real bottleneck is still chunk generation/noise plus GC pressure, not plugin load order: `ImprovedNoise.p` alone reached `49.18%` of Java method samples and G1 recorded `6.00s` total pause time. ZGC was measured and rejected earlier; no GC default changed.
- The latest plugin-loading allocation cleanup has a synthetic setup microbench (`1.589x` faster pre-sized setup), validate microbench (`1.069x`), neutral legacy missing-set timing (`0.994x`, no speed claim), and build/plugin/restart/forced-ticket/hash evidence. It is not a clean end-to-end startup speedup claim because strict 50-bot preflight blocked before Minecraft start (`load_per_cpu=1.313` > `0.750`).
- The persistent-ticket save packing change has build, plugin matrix, restart/recovery, and forced chunk restart persistence evidence. It is not yet promoted as a load-performance win until the next bot gate shows the `save-all` watchdog path is gone or reduced.
- The noisy rerun after persistent-ticket save packing reached all 50 bots with no kicks/errors and no watchdog/sync-load hits (`tps1_avg=17.30`, `avg_tick_ms_avg=82.93`, `loaded_chunks_max=819`), so it is accepted only as save-watchdog risk reduction, not as a new load baseline.
- `NoiseChunk.FlatCache` per-NoiseChunk reusable context was tested and reverted. It passed build/plugin/restart/forced-ticket gates, but the 50-bot noisy gate hit `watchdog_thread_dumps=3` with stack frames in `NoiseChunk$FlatCache.<init>` and low `loaded_chunks_max=385`.
- Current 2026-05-08 50-bot 32/32 load gate ran and is not accepted as a new baseline: `tps1_avg=16.93`, `avg_tick_ms_avg=145.06`, `loaded_chunks_max=2005`, no watchdog/sync-load hits. The host still had live Java/Velocity processes, but preflight thresholds passed (`load_per_cpu=0.713`).
- The load harness now enforces that limitation by default; `LOAD_TEST_ALLOW_BUSY_HOST=true` is required to run a non-comparable noisy gate.
- The forced noisy 2026-05-07 run is recorded as diagnostic only. It reached all 50 bots but had low chunk coverage and one watchdog dump during `save-all`; it points at `TicketStorage.packTickets()` copying regular tickets before filtering persistent tickets.
- Vanilla parity is not claimed.
- "All plugins supported" is not claimed.

## 2026-05-08 SurfaceRules SequenceRule Array Candidate Rejected

`SurfaceRules.SequenceRule` was briefly changed from runtime
`List<SurfaceRule>` storage to runtime `SurfaceRule[]` storage, then advanced
to an indexed array loop. The codec-facing `SequenceRuleSource.sequence`
remained a `List<RuleSource>`, and the intended rule order / first-non-null
behavior did not change. The production path has now been reverted to
`List<SurfaceRule>` plus `ImmutableList.builder()`.

Verification completed on candidate artifacts before rejection:

```text
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS, compileJava executed and createMojmapBundlerJar succeeded
sha256sum -c reports/artifact-hashes.txt: PASS
optimized-paper sha256: 54ad99e465e925f85023132529918fb5cb59d95c4b5e889c1915fb5f673636f3
plugin matrix: PASS, Done (33.652s)
restart/recovery: PASS, Done (20.881s)
forced-ticket persistence: PASS, first/restart Done (16.478s)/(13.288s)
```

The earlier strict comparable 50-bot 32/32 performance gate did not run because
host preflight blocked before Minecraft start:

```text
host_preflight_ok=false
load1=16.85
load_per_cpu=1.404
idle_percent_1s=49.27
max_load_per_cpu=0.750
report: reports/load-50bots-surfacerules-sequence-array-gate-preflight.txt
```

A later strict gate on the array-indexed candidate passed preflight and failed
the accepted baseline:

```text
report: reports/load-50bots-surfacerules-array-index-gate-rerun2-summary.txt
online_max=50
tps1_avg=15.95
avg_tick_ms_avg=117.42
loaded_chunks_max=1785
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The accepted baseline remains `18.27/47.85/2380`, so this is a regression.

A forced noisy diagnostic run was recorded only for crash/watchdog signal:

```text
online_max=50
tps1_avg=16.75
avg_tick_ms_avg=64.76
loaded_chunks_max=1571
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
report: reports/load-50bots-surfacerules-sequence-array-noisy-summary.txt
```

Post-revert verification:

```text
rebuildPatches: PASS, Rebuilt 910 source patches
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (34.263s)
restart/recovery: PASS, Done (21.224s)
forced-ticket persistence: PASS, first/restart Done (17.307s)/(11.562s)
```

This candidate is rejected and no load-performance win is claimed.

## 2026-05-10 NoiseChunk Interpolator Array Candidate Rejected

`NoiseChunk` interpolator hot loops were tested as a lazy
`NoiseInterpolator[]` snapshot instead of iterating the existing
`List<NoiseInterpolator>`. The patch-stack failure from the first attempt was
fixed by editing the generated source, running `paper-server:fixupSourcePatches`,
then `paper-server:rebuildSourcePatches`, and verifying that `applyPatches`
still applied all `912` patches.

The candidate itself is rejected. The corrected microbenchmark now compares
the old enhanced-for list loop, an indexed-list loop, and the array snapshot
loop on equivalent data:

```text
report: reports/noisechunk-interpolator-array-bench.txt
enhanced_for_loop_best_ms=1137.729
indexed_list_loop_best_ms=1158.104
array_loop_best_ms=1164.487
array_vs_enhanced_for_speedup=0.977x
array_vs_indexed_list_speedup=0.995x
equivalence=PASS
```

Because the array snapshot lost to the current enhanced-for loop, the runtime
change was reverted. The source patch was rebuilt back to the accepted
enhanced-for path, and `rg` confirms that neither `interpolatorsArray` nor
`interpolatorArray()` remains in generated `NoiseChunk.java` or its source
patch.

Verification after rejection:

```text
paper-server:rebuildSourcePatches: PASS, Rebuilt 912 patches
applyPatches: PASS, Applied 912 patches
build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
python3 -m json.tool reports/artifacts.json: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
```

The strict 50-bot 32/32 gate was intentionally not run after rejection because
the host preflight blocked it:

```text
report: reports/load-50bots-noisechunk-interpolator-array-postreject-preflight.txt
host_preflight_ok=false
load1=11.28
load_per_cpu=0.940
idle_percent_1s=38.09
max_load_per_cpu=0.750
```

No 50-bot/500-bot performance improvement is claimed from this candidate.

## 2026-05-10 Persistent Ticket Pack Direct Append

Added feature patch
`0050-Optimize-persistent-ticket-pack-direct-append.patch`.
`TicketStorage.packTickets()` still serializes only persistent tickets and keeps
the same `TicketType.persist()` guard, but the regular persistent-ticket path
now appends directly into the packed output list instead of routing through a
`BiConsumer` lambda and then checking/appending in the caller.

Targeted benchmark:

```text
report: reports/ticket-pack-bench.txt
callback_pack_best_ms=3631.163
direct_pack_best_ms=3239.248
direct_speedup=1.121x
equivalence=PASS
```

Verification:

```text
rebuildPatches: PASS, Rebuilt 912 source patches, Saved modified patches (47/50)
applyPatches: PASS, Applied 912 patches
build_optimized.sh: PASS, compileJava/createMojmapBundlerJar/AppCDS completed
python3 scripts/update_artifact_reports.py: PASS
python3 -m json.tool reports/artifacts.json: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
forced-ticket persistence: PASS, first/restart Done (18.168s)/(14.334s)
plugin matrix: PASS, Done (41.256s)
restart/recovery: PASS, Done (19.539s)
```

The strict 50-bot 32/32 gate did not run because preflight blocked the busy
host before Minecraft start:

```text
report: reports/load-50bots-ticket-pack-direct-gate-20260510-preflight.txt
host_preflight_ok=false
load1=13.66
load_per_cpu=1.139
idle_percent_1s=65.85
max_load_per_cpu=0.750
```

Verdict: accepted as a narrow save-path work reduction with forced-ticket
persistence coverage. It is not a 50-bot/500-bot load-performance claim.

## 2026-05-10 Rejected ChunkDependencies Radius Lookup

The JFR rebaseline pointed at `RegularImmutableList.get(int)` under
`WorldGenRegion.getChunk(...)` / biome and surface generation chunk requests.
The tested candidate kept the original immutable list for `asList()` and
`toString()`, but also snapshotted it into a `ChunkStatus[]` for `get(int)`;
`size()` and `getRadius()` were cached scalar fields. Dependency construction
semantics, exception text source, and public list exposure were unchanged.

Targeted benchmark:

```text
report: reports/chunk-dependencies-array-bench.txt
old_immutable_list_get_best_ms=419.919
array_get_best_ms=341.251
array_get_speedup=1.231x
equivalence=PASS
```

Patch-stack verification:

```text
rebuildPatches: PASS, Rebuilt 912 source patches, Saved modified patches (48/51)
applyPatches: PASS, Applied 912 source patches and 51 feature patches
candidate post-apply source check: PASS, dependencyByRadiusArray existed before rejection
```

Runtime gates with the candidate:

```text
build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS after AppCDS warmup completed
python3 -m json.tool reports/artifacts.json: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (27.278s)
restart/recovery: PASS, Done (16.842s)
forced-ticket persistence: PASS, first/restart Done (13.206s)/(9.446s)
50-bot 32/32 gate: ran with clean preflight, online_max=50, no kicks/errors/sync-load, but tps1_avg=17.89, avg_tick_ms_avg=57.67, loaded_chunks_max=2792, watchdog_thread_dumps=3
```

The strict gate failed the accepted `18.27/47.85/2380` baseline and produced
watchdog dumps in movement/chunk-ticket paths (`NearbyPlayers`,
`ChunkHolderManager`, `TicketSet`), not in `ChunkDependencies`.

Verdict: rejected and removed from the production patch stack. The microbench
and report are kept as evidence, but no runtime source change remains and no
50-bot/500-bot load-performance claim is made.

Post-rejection verification: `applyPatches` PASS with the patch stack ending at
`0050-Optimize-persistent-ticket-pack-direct-append.patch`; final
`MC_EULA_AGREE=true ./scripts/build_optimized.sh` PASS; AppCDS was generated;
`python3 scripts/update_artifact_reports.py`, `python3 -m json.tool
reports/artifacts.json`, and `sha256sum -c reports/artifact-hashes.txt` PASS.
`rg` confirms no `dependencyByRadiusArray` remains in generated source or
feature patches.

## 2026-05-10 Rejected ImprovedNoise Derivative Flat Gradient

The derivative path in `ImprovedNoise.sampleWithDerivative(...)` was tested as
a narrow follow-up to the accepted `sampleAndLerp` flat-gradient cleanup. The
candidate kept the existing `p(...)` permutation path but replaced
`SimplexNoise.GRADIENT[index]` int-array references and `SimplexNoise.dot(...)`
calls with a local flat `int[]` gradient table and direct dot products.

Targeted benchmark:

```text
report: reports/improved-noise-derivative-bench.txt
old_derivative_best_ms=53.103
inline_derivative_best_ms=54.344
int_table_derivative_best_ms=56.539
flat_gradient_derivative_best_ms=50.027
flat_gradient_derivative_speedup=1.061x
equivalence=PASS
```

Runtime gates with the candidate:

```text
patch stack: PASS, rebuildPatches/applyPatches produced 0051-Optimize-ImprovedNoise-derivative-gradients.patch
build_optimized.sh: PASS
artifact JSON/hash checks: PASS
plugin matrix: PASS, Done (26.221s)
restart/recovery: PASS, Done (17.577s)
forced-ticket persistence: PASS, first/restart Done (13.630s)/(9.101s)
50-bot 32/32 gate: preflight PASS, online_max=50, no kicks/errors/sync-load, but tps1_avg=15.36, avg_tick_ms_avg=94.24, loaded_chunks_max=3850, watchdog_thread_dumps=2, nearby_players_stack_hits=8
```

The standalone derivative loop improved, but the real gate regressed far below
the accepted `18.27/47.85/2380` baseline and produced watchdog evidence in
movement/chunk proximity paths, not in the changed derivative function.

Verdict: rejected and removed from the production patch stack. The feature
patch `0051-Optimize-ImprovedNoise-derivative-gradients.patch` was deleted and
`applyPatches` now ends at `0050-Optimize-persistent-ticket-pack-direct-append.patch`.
Post-rejection `build_optimized.sh` completed, AppCDS was regenerated, artifact
JSON and hash checks passed, and `rg` confirms no `FLAT_SIMPLEX_GRAD_INT`
remains in generated source or feature patches. Current rebuilt artifact hashes:
optimized jar `a8e0d476f77a86fb6f94db670d351cd5bcd66239bcc2452074b705e847fcbaf6`;
AppCDS `78a7cd0b24e896ae63577d9d54fff7a763081b4ece06686185c6b1735a9744d0`.

## 2026-05-10 Rejected Before Production: CompoundTag Map Initial Capacity

The fresh JFR showed `Object2ObjectOpenHashMap.<init>(int, float)` allocation
under `CompoundTag.loadCompound(...)` while reading chunk NBT. A standalone
parser benchmark was added to replay real `.mca` chunk payloads and compare the
current Paper `Object2ObjectOpenHashMap<>(8, 0.8F)` with smaller/larger initial
capacities.

Result:

```text
report: reports/nbt-compound-map-capacity-bench.txt
chunks_used=512
compound_count=228744
compound_entries_max=40
equivalence=PASS
cap2_best_ms=2080.649
cap4_best_ms=1922.989
cap8_best_ms=1907.510
cap16_best_ms=1957.953
cap4_vs_current_speedup=0.992x
cap16_vs_current_speedup=0.974x
```

Verdict: rejected before production. The current initial capacity `8` is still
the fastest tested shape on real chunk NBT samples, so `CompoundTag` source was
not changed.

## 2026-05-13 Native Rust batch: NoiseChunk wrap capacity + Deflater input shape

Two new diagnostic Rust modules were added without wiring them into the Paper runtime:

- `native/paper-native-core/src/noisechunk_wrap_capacity.rs`
- `native/paper-native-core/src/deflater_input_shape.rs`
- `native/paper-native-jni/src/lib.rs`
- `bench/native-noisechunk-wrap-capacity/`
- `bench/native-deflater-input-shape/`
- `scripts/bench_native_noisechunk_wrap_capacity.sh`
- `scripts/bench_native_deflater_input_shape.sh`

Verification:

```text
cargo test --manifest-path native/Cargo.toml --workspace: PASS, 275 tests
JAVA_PROPS='-DmapBenchIterations=100 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_noisechunk_wrap_capacity.sh: PASS, equivalence=PASS, script_status=PASS, native_speedup_vs_java=1.161x
./scripts/bench_native_deflater_input_shape.sh: PASS, equivalence=PASS, script_status=PASS, copied_native_speedup_vs_java=1.354x, slice_native_speedup_vs_java=1.067x
```

Verdict: diagnostic-only. These modules are for parity and measurement, not a Paper hot-path claim.
