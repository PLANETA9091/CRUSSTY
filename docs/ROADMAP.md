# Roadmap

Цель остается прежней: Paper/Bukkit compatibility first, optimization second. Rust/native work is allowed only for measured hot paths that do not alter plugin-visible semantics.

## Current Direction

- 2026-05-20 CEST: the current P250 radius-preload mixed-gameplay diagnostic
  reached full online (`250/250` ready/active), opened the action gate, and
  completed radius preload with the stress corpus enabled (`26` plugin jars,
  `10` datapacks, `8553` preloaded chunks, zero preload failures). The same
  run is still red on the first full-online window:
  `observed_bot_errors_max=14`, `observed_tps1_avg=14.31`,
  `observed_tps1_min=0.61`, `observed_avg_tick_ms_avg=79.31`,
  `observed_avg_tick_ms_max=915.73`, `watchdog_thread_dumps=12`,
  `external_thread_prints=10`. The explicit `misc.max-joins-per-tick=20`
  knob solved admission, so the next measured move is a warm-source P250
  diagnostic on the same artifact. The refreshed forward checklist is
  `docs/NEXT_2026-05-21_P250_WARM_SOURCE_RECOVERY_GOAL.md`.

- 2026-05-20 CEST: the current `P100` fresh-world radius-preload
  mixed-gameplay gate passed on the live artifact
  (`online_max=100`, `load_window_tps1_avg=19.15`,
  `load_window_tps1_min=17.13`, `load_window_avg_tick_ms_avg=49.57`,
  `load_window_avg_tick_ms_max=87.62`, `watchdog_thread_dumps=0`,
  `sync_load_stack_hits=0`, `stability_failures=0`). The next measured step
  is not another P100 redo; it is the next tier and the next bottleneck,
  likely P250 fresh-world mixed gameplay or the accepted P100 warm/repeat
  follow-up. The active goal file is
  `docs/CURRENT_2026-05-20_MAXIMUM_SCALE_GOAL.md`.

- 2026-05-18 CEST: the extreme-scale track now has complete P250 evidence
  instead of a missing/partial run. The load evidence harness was hardened:
  late FIFO console writes are best-effort in `scripts/run_load_test.sh`, and
  `scripts/run_stress_mixed_load_gate.sh` now still evaluates the gate after a
  non-zero load-test exit when a summary exists. The fresh P250
  worker10/send60/gen20 run with full stress corpus and 300 zombies reached
  `bot_connected_max=250`, `bot_ready_max=250`, but only
  `online_max=229` / `bot_active_max=229`, with
  `load_window_tps1_avg=12.32`, `load_window_tps1_min=7.58`,
  `load_window_avg_tick_ms_avg=79.43`, `load_window_avg_tick_ms_max=144.78`,
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`, and
  `moved_too_quickly_warnings=10566`. A slow-move P250 control still failed
  (`online_max=218`, `bot_errors_max=32`, `avg_tick_ms_max=1237.58`), so the
  next target is not another movement-speed tweak. Focus next on join/chunk/
  plugin/worldgen pressure visible in P250 thread samples, especially native
  ImprovedNoise/density compute and login plugin overhead, while preserving
  the zero watchdog/sync-load state.

- 2026-05-18 CEST: the fresh DFC + `/forceload` command-path fix turned the
  50-bot stress-corpus mixed-gameplay gate green on the current artifact:
  `load_window_tps1_avg=18.33`, `load_window_tps1_min=15.88`,
  `avg_tick_ms_avg=26.84`, `avg_tick_ms_max=78.32`,
  `watchdog_thread_dumps=0`, `sync_load_stack_hits=0`,
  `stability_failures=0`. This is a real milestone, not a 500-bot or
  unlimited claim. The next work is to push the same artifact into P100 and
  beyond, and keep the failure mode honest if it turns red again.

- 2026-05-17 CEST: a measured runtime change was accepted for the current
  mixed-gameplay track, but it is not a production claim. The current artifact
  `b58b307f17ee68e868105473a393d3696ac6c5356fd9afa27d3e9a4188681bc0`
  now gets a generated runtime where `PAPER_NATIVE_IMPROVED_NOISE` defaults
  to `true` and `PAPER_NATIVE_PERLIN_NOISE` remains `false`. The no-env
  default run loaded `native_improved_noise=true` and improved same-artifact
  stress-corpus 50-bot mixed-gameplay tail cost versus native disabled:
  TPS avg `14.73 -> 15.22`, avg tick `48.86 -> 48.35`, max tick
  `108.19 -> 92.19`, RSS `24299.0 -> 23743.7`; TPS min was effectively flat
  but slightly lower `5.17 -> 5.14`. The gate is still red because
  `15.22 < 18.00` avg TPS and `5.14 < 15.00` min TPS. `PerlinNoise` native
  stays opt-in because the combined native-noise run regressed, and the
  temporary holder-cache visitor patch was deleted after a gate regression.
  Next: profile the default-ImprovedNoise runtime and attack the next
  worldgen/chunk-streaming hotspot; do not broaden this to 500 mixed gameplay
  or production-ready.

- 2026-05-17 CEST: `mixed-gameplay` is now a real harness stage, tracked in
  `docs/MIXED_GAMEPLAY_SCALE_GOAL.md`. The new runner
  `scripts/run_stress_mixed_gameplay_gate.sh` drives movement, block
  place/dig, item switches, animation, player input, use-item, client command
  packets, mobstorm, stress plugins, and stress datapacks. `CompatProbe` now
  records server-side command, held-item, animation, interact, and entity
  counters. A 4-bot smoke passed with zero kicks, zero moved-too-quickly, and
  zero mixed action errors. A 50-bot full stress-corpus diagnostic reached all
  bots and all required workload counters with zero watchdog/sync-load/kick/
  stability failures, but failed `stress-mixed-gameplay` on TPS only:
  `load_window_tps1_avg=13.97 < 18.00` and
  `load_window_tps1_min=5.42 < 15.00`. Entity attack packets are kept opt-in
  because the unsafe variant can kick clients with `invalid_entity_attacked`.
  The next target is a real TPS improvement for this profile, not more harness
  plumbing.

- 2026-05-17 CEST: the extreme-scale track now has P100 warm plateau axis
  diagnostics instead of only fresh-world failure evidence. A warm-source P100
  from `runs/load-extreme-stress-mixed-100-20260517-121716` reached
  `100/100` bots, but still failed (`14.10 TPS avg`, `6.23 TPS min`,
  `157.99 ms avg tick`, `629.12 ms max tick`, `2 watchdogs`,
  `2 sync-load hits`). `scripts/run_extreme_plateau_axis_matrix.sh` now runs
  controlled warm-source variants, and the load harness has
  `BOT_MOVE_INTERVAL_MS` plus `BOT_SEND_STATIONARY_POSITIONS=false` so parked
  clients can truly stop sending movement packets. New P100 diagnostics:
  slow movement reached all bots but failed hard (`12.75 TPS avg`,
  `235.18 ms avg tick`, no watchdog/sync-load); true-idle with 150 zombies
  sent zero position packets and had zero moved-too-quickly/watchdog/sync-load
  failures, but still failed `13.02 TPS avg`, `9.64 TPS min`, and
  `185.88 ms max tick`; no-mob true-idle improved tick tail
  (`56.71 ms avg`, `134.71 ms max`) but only reached `97` active bots and
  still missed TPS. Decision: movement packet spam is not the main P100
  blocker; mobs contribute to tail MSPT, but the next accepted optimization
  target must still come from density/worldgen/chunk-streaming and plugin/
  entity idle overhead under the full stress corpus.

- 2026-05-17 CEST: the project now has an explicit extreme-scale target in
  `docs/EXTREME_SCALE_GOAL.md`. This target does not pretend that literal
  unlimited scale is possible; it defines a measured ladder for full stress
  corpus profiles at P100/P250/P500/P1000+, mob/entity tiers, worldgen and
  datapack tiers, packet/plugin tiers, and evidence bundles. The baseline is
  intentionally hard: current stress mixed 50-bot evidence with 26 plugins,
  10 datapacks, 150 zombies, and fresh worldgen fails
  (`7.96 TPS avg`, `4.13 TPS min`, `161.33 ms avg tick`,
  `475.54 ms max tick`) while preserving zero watchdog/sync-load/stability
  failures. A fresh 100-bot diagnostic has now been run as well:
  `load_window_tps1_avg=9.38`, `load_window_tps1_min=4.74`,
  `load_window_avg_tick_ms_avg=71.67`, `load_window_avg_tick_ms_max=141.90`,
  `bot_connected_max=76`, `bot_ready_max=76`, `bot_active_max=76`,
  `load_window_online_max=67`, `watchdog_thread_dumps=8`,
  `sync_load_stack_hits=6`, with hot frames dominated by
  `ImprovedNoise.noise(...)`, `DensityFunctions$HolderHolder.mapAll(...)`,
  `DensityFunctions$Ap2.mapAll(...)`, `DensityFunctions$MarkerOrMarked.mapAll(...)`,
  `CubicSpline$Multipoint.mapAll(...)`, and `ServerChunkCache.syncLoad(...)`.
  A follow-up P100 with `paper.nativeImprovedNoise=true` also failed
  (`bot_connected_max=78`, `load_window_online_max=66`,
  `load_window_tps1_avg=7.27`, `load_window_tps1_min=2.45`,
  `watchdog_thread_dumps=7`, `sync_load_stack_hits=4`), so that native path
  is not accepted as the fix for this extreme profile.
  New scripts support this track:
  `scripts/run_extreme_scale_ladder.sh` runs higher stress tiers and keeps
  per-tier reports, and `scripts/summarize_thread_samples.py` aggregates
  jstack samples into hot-frame reports. The next real optimization target
  must come from those extreme diagnostics, not from broadening the existing
  500 creative-block claim.

- 2026-05-17 CEST: the near-unbounded track now has a real stress corpus
  layer instead of only a goal statement. `scripts/fetch_stress_corpus.py`
  downloads 22 additional Modrinth plugin jars and 10 heavy datapacks
  (Terralith, Incendium, Nullscape, Structory, Tectonic, Dungeons and
  Taverns, Geophilic, Continents, Explorify, Amplified Nether), writing
  `reports/stress-corpus-artifacts.csv` and
  `reports/stress-corpus-manifest.json` with sha256/source evidence.
  `scripts/inspect_stress_corpus.py` passes with `plugin_count=22`,
  `datapack_count=10`, and `failure_count=0`. The new
  `scripts/run_stress_corpus_gate.sh` then boots the current matrix plus the
  stress corpus: `matrix_plugin_count=12`, `stress_plugin_count=22`,
  `plugin_count=34`, `datapack_count=10`, `Done (153.340s)`,
  `StressProbe` join/quit, and `13 data pack(s) enabled`.
  `run_load_test.sh` can now opt in
  to the corpus with `LOAD_TEST_STRESS_CORPUS=true`. This is boot/join/
  datapack evidence only; the next work is a stress-corpus mixed gameplay
  load gate with mob/worldgen pressure, not a broader production claim.

- 2026-05-26 CEST: the current giant target is now tracked in
  `docs/CURRENT_2026-05-26_RESOURCE_BOUND_NEAR_UNBOUNDED_CORE_ENDGAME_GOAL.md`.
  It defines the scary but measurable
  "near-unbounded" direction: players, mobs, chunks, plugin workload, network,
  IO, memory durability, mixed gameplay, long soak, and tiered evidence
  bundles. The rule remains strict: do not claim literal unlimited scale; only
  climb the claim ladder after repeatable gates, zero watchdog/sync-load
  failures, bounded memory/queue growth, and published evidence.

- 2026-05-17 CEST: the measured `500 bots / production ready` claim now has a
  top-level certification runner and gate.
  `scripts/run_production_readiness_gate.sh` refreshes the fast
  compatibility/recovery layer and then calls
  `scripts/evaluate_production_readiness.py`, which
  consumes the 30-minute cold+warm soak gate, the three-pass repeat quorum,
  fresh plugin matrix, fresh restart/recovery, fresh forced-ticket
  persistence, and artifact hash verification, then writes
  `reports/production-500-readiness-gate.txt`. The fresh report has
  `production_ready_500_claim=true`, `readiness_gate_pass=true`,
  `failure_count=0`, `soak_gate_pass=true`, `repeat_quorum_pass=true`,
  `plugin_matrix_pass=true`, `restart_recovery_pass=true`,
  `forced_ticket_persistence_pass=true`, and `artifact_hashes_pass=true`,
  plus sha256 hashes for the evidence files. The latest runner report is
  `reports/production-500-readiness-run-20260517-091520.txt`, and the claim
  bundle is `reports/production-500-readiness-bundle-20260517-091520`.
  The bundle now includes `bundle.json` and is checked by
  `scripts/validate_production_readiness_bundle.py`, which reports
  `bundle_validation_pass=true` and `evidence_file_count=8`. The final
  `scripts/assert_production_ready_claim.py` layer writes
  `reports/production-500-claim-verdict-20260517-091520.txt` with
  `claim_assertion_pass=true` and the exact allowed claim text. The runner
  also refreshes `reports/production-500-readiness-bundle-current`, so
  `scripts/production_ready_claim.sh` can print the exact current allowed
  claim text without a timestamped bundle argument. The publication layer now
  emits `reports/production-500-claim-current.{txt,md,json}` with
  `claim_publication_pass=true`.
  Fresh support gates were rerun on the current runtime: plugin matrix
  `Done (21.929s)` with 11 plugins and `COMPAT_PROBE command=ok events=4`,
  restart/recovery `Done (15.527s)` with `Saved the game`, and forced-ticket
  first/restart `Done (11.386s)` / `Done (8.551s)`. Next work should increase
  the claim envelope with multi-hour soak, broader plugin/gameplay matrices,
  and larger measured profiles, not by loosening this gate.

- 2026-05-17 CEST: the measured `500 bots / production ready` claim now has a
  30-minute cold/fresh plus 30-minute warm-source soak gate on top of the
  earlier repeat quorum. `scripts/run_production_soak_gate.sh` runs the
  current-artifact worker10/send60/gen20 profile, and
  `scripts/evaluate_production_soak.py` writes
  `reports/production-500-soak-gate.txt`. The fresh report has
  `production_ready_soak_claim_eligible=true`, `soak_gate_pass=true`,
  `failure_count=0`, `base_cold_gate_pass=true`, `base_warm_gate_pass=true`,
  and `artifact_hashes_pass=true`. Cold/fresh load-window metrics are
  `19.84/19.19/60.48` with `264000/264000` block packets; warm-source metrics
  are `19.95/19.28/56.32` with `267500/267000` block packets
  (`load_window_tps1_avg / load_window_tps1_min /
  load_window_avg_tick_ms_max`, then block place/dig packets). Both surfaces
  reached `online_max=500` and `loaded_chunks_max=5476` with zero
  watchdog/sync-load/stability failures. The next direction is margin,
  multi-hour soak, broader plugin/gameplay validation, and then larger
  measured profiles. Do not broaden this to a full Rust runtime, unlimited
  plugins, or unmeasured real-player gameplay claim.

- 2026-05-17 CEST: the measured `500 bots / production ready` claim gained a
  three-pass repeat quorum. `scripts/evaluate_production_release.py`
  recomputes the cold/fresh `production-500` and warm-source
  `production-500-warm` gates from their summaries and verifies
  `reports/artifact-hashes.txt`; `scripts/evaluate_production_release_repeat.py`
  then evaluates preserved release runs under `reports/release-repeat-*`.
  The quorum report is `reports/production-500-repeat-quorum.txt` with
  `required_min_passes=3`, `repeat_run_count=3`, `repeat_passes=3`,
  `repeat_failures=0`, and `repeat_quorum_pass=true`. The accepted
  current-artifact profile uses `PAPER_CHUNK_WORKER_THREADS=10`,
  `PAPER_PLAYER_MAX_SEND_RATE=60`, and `PAPER_PLAYER_MAX_GEN_RATE=20`.
  The three preserved passes are:
  `19.84/18.62/61.17` cold and `19.88/19.32/53.27` warm;
  `19.91/18.72/54.87` cold and `19.90/19.12/59.48` warm;
  `19.84/19.06/55.86` cold and `19.90/19.33/56.58` warm
  (`load_window_tps1_avg / load_window_tps1_min / load_window_avg_tick_ms_max`).
  Every run reached `online_max=500` and `loaded_chunks_max=5476` on both
  surfaces with zero watchdog/sync-load/stability failures. The
  default-generation worker10/send60 release attempt failed cold/fresh on
  `load_window_tps1_min=17.92`, so gen-rate 20 is part of the release profile.
  The next direction is margin, long soak, broader plugin/gameplay validation,
  and then larger measured profiles. Do not broaden this to a full Rust
  runtime, unlimited plugins, or unmeasured real-player gameplay claim.

- 2026-05-16 CEST: the earlier worker8 cold/fresh `production-500` pass
  introduced load-window reporting and remains useful historical evidence, but
  it is superseded for release claims by the current-artifact worker10
  cold+warm release gate.

- 2026-05-16 CEST: this continuation pushed three runtime patches through the full build pipeline: `0086` reuses the entity chunk-sent key inside `ChunkMap$TrackedEntity.updatePlayerFast(...)`, `0087` adds a type-reference fast path in `Ticket.compareTo(...)`, and `0088` specializes nearest-player scans for `EntitySelector.PLAYER_AFFECTS_SPAWNING`. The source tree rebuilt cleanly after each patch and `build_optimized.sh` passed again after the patch stack was regenerated. Synthetic benches show `ref_fast_speedup=1.070x` for ticket comparison and `specialized_speedup=1.067x` for the spawning-player scan. The fresh warm 500 gate reached the full requested shape with zero watchdog/sync-load/stability failures and passed MSPT/RSS, but still failed `production-500-warm` on TPS only: `tps1_avg=18.42 < 19.50` and `tps1_min=14.66 < 18.00`. The next target is a current JFR from the post-`0088` 500/block plateau.

- 2026-05-15 CEST: the latest warm 500 JFR moved the dominant CPU target to
  `ChunkMap$TrackedEntity.updatePlayer(...)`, with `ReferenceOpenHashSet.contains`,
  `Entity.getBukkitEntity()`, `HashMap.getNode`, and `TargetingConditions.test`
  directly behind it. That means the next source-level target should be the
  entity tracker path, not another broad PlayerList tweak. The current 500-bot
  run still fails (`13.71 TPS / 98.19 ms / 5476 loaded chunks / 3 watchdogs /
  5 nearby-player stack hits`), so the next step is a real entity-tracker
  patch with a fresh build+bench+gate loop.

- 2026-05-15 CEST: the newest candidate is a distance-first visible-player
  broadcast check in `PlayerList.broadcast(...)`. It compiled, the optimized
  runtime rebuilt cleanly, and the focused microbench improved both empty and
  populated cases (`1.213x` and `1.718x` speedups). The fresh warm 500 gate
  still failed (`15.53 TPS / 76.90 ms / 5476 loaded chunks / 3 watchdogs /
  2 nearby-player stack hits`), so the next target is not a claim but the
  remaining disconnect/cleanup hot path that still trips watchdogs.

- 2026-05-15 CEST: the 500-bot claim path is now split into two honest
  surfaces. `production-500` remains the cold/fresh-world gate and rejects
  summaries that explicitly use `LOAD_TEST_WORLD_SOURCE`. The new
  `production-500-warm` profile and `scripts/run_production_warm_claim_gate.sh`
  keep the same 500-bot, creative block, 32/32, TPS/MSPT, RSS, and zero
  watchdog/sync-load/stability requirements, but require
  `world_mode=warm-source`. A warm-world pass would only justify a saved or
  pregenerated world claim; it would not erase the current cold-generation
  failure.

- 2026-05-15 CEST: the first full warm-world 500-bot block gate reached
  `online_max=500`, `bot_block_armed_max=500`,
  `bot_block_primed_max=500`, and `57000/56625` block place/dig packets, but
  still failed: `tps1_avg=15.15`, `tps1_min=4.55`,
  `avg_tick_ms_avg=81.72`, `avg_tick_ms_max=271.61`,
  `watchdog_thread_dumps=5`, `nearby_players_stack_hits=0`. The next target is
  no longer cold chunk generation for this surface; it is the 500-player block
  action plateau plus mass-disconnect/quit cleanup. The harness now disables
  Spark background profiling by default and records that in summaries, so the
  next comparable run should use `spark_background_profiler=false`.

- 2026-05-15 CEST: the native `ImprovedNoise` handle experiment is now
  diagnostic-only. Batch JNI benches still win on the raw summary shape, but
  the per-call runtime handle path is slower than the Java baseline on this
  host (`handle_native_speedup_vs_java=0.917x`), so the runtime wrapper
  keeps `paper.nativeImprovedNoise=false` by default. This is a useful bench
  checkpoint, not a production gate win.

- 2026-05-15 CEST: `scripts/evaluate_load_gate.py` and
  `scripts/run_production_claim_gate.sh` define the hard gate for any
  "500 bots / production ready" claim. A claim now requires a fresh
  `production-500` block run at 32/32 view and simulation distance with 500
  online/ready/active bots, full block armed/primed/action coverage, zero
  kicks/errors/watchdog/sync-load/thread failures, `tps1_avg >= 19.5`,
  `tps1_min >= 18.0`, `avg_tick_ms_avg <= 50.0`, and
  `avg_tick_ms_max <= 100.0`. Current 50/100-bot block evidence still fails
  this gate, so the next performance target remains reducing tick cost and
  chunk pressure before scaling the run.

- 2026-05-15 CEST: `jfr view hot-methods` on the current 100-bot block JFR
  points at `ImprovedNoise.sampleAndLerp(...)`, `ImprovedNoise.noise(...)`,
  `PerlinNoise.getValue(...)`, `Aquifer$NoiseBasedAquifer.computeSubstance(...)`,
  and `NoiseChunk$NoiseInterpolator.compute(...)`. The fresh source-level
  changes are `PerlinNoise.wrap(double) -> Math.floor(...)` and the exact-class
  `PerlinNoise.getValue(double,double,double)` fast path with local
  `noiseLevels`/`amplitudeValues`; the exact runtime bench shape now reports
  `math_wrap_guarded_speedup=1.169x`, while the flat `NoiseInterpolator` slice
  rewrite is still a loss (`0.934x`) and the no-y-scale follow-up is rejected
  (`0.939x`). The rebuilt runtime improved the
  creative 32/32 100-bot block plateau from `9.88 TPS / 223.62 ms /
  11515.4 MiB RSS` to `11.46 TPS / 95.89 ms / 10787.6 MiB RSS`, with zero
  watchdog/sync-load/stability failures, but the `production-500` evaluator
  still fails with 16 failures. The next target is a fresh JFR on the improved
  plateau and another source-level hot-path candidate, not a production claim.

- 2026-05-14 22:47 CEST: `scripts/run_load_test.sh` now writes a per-run
  `bukkit.yml` with `connection-throttle: 0` for localhost synthetic load.
  The creative 32/32 block run now reaches `online_max=100`,
  `bot_block_armed_max=100`, `bot_block_primed_max=100`, and
  `stability_failures=0`, but it still lands at `tps1_avg=9.88`,
  `avg_tick_ms_avg=223.62`, and `process_rss_mib_max=11515.4`. The next
  target is lowering the block plateau's tick cost and chunk pressure, not
  making a 500-player claim.

- 2026-05-14 21:49 CEST: the block arena window now stays open for the full
  bot duration, and the first real 50-bot creative 32/32 plateau reached
  `bot_block_armed_max=50` and `bot_block_primed_max=50`. This is useful
  harness evidence, but the run still landed at `tps1_avg=12.50` and
  `avg_tick_ms_avg=183.80`, so the next target is lowering the block
  plateau's tick cost, not making a 500-player claim.

- 2026-05-14 20:09 CEST: the warm-world benchmark harness now exists and
  records startup/plugin-load evidence on a saved world. The current run
  shows optimized Paper `1.597x` faster than stock on warm-start `done_ms`,
  and optimized runtime `2.042x` faster than stock, but this is still only
  startup evidence, not a 500-player or strict-gate claim.

- 2026-05-14 17:43 CEST: `paper.nativeAreaMap` is now source-wired through
  `PaperNativeAreaMap` and `SingleUserAreaMap` as a default-off guarded
  runtime path. The focused bench is parity-clean
  (`update_native_speedup_vs_java=1.218x`,
  `add_native_speedup_vs_java=1.216x`,
  `remove_native_speedup_vs_java=1.168x`), but the 2026-05-14 50-bot gate
  still rejected the runtime path and current host preflight blocks a clean
  rerun. Keep it diagnostic-first until a fresh strict gate beats the
  accepted baseline.

- 2026-05-13 22:03 UTC: `paper-native-core::lz4_stream_roundtrip`,
  `paper-native-core::nbt_gzip_buffer_shape`, and
  `paper-native-core::compression_threshold_shape` are the newest modular
  Rust rewrite checkpoints. They have Rust tests, JNI exports, Java/native
  parity benches, executable scripts, and fresh reports. LZ4 round-trip
  passes equivalence across `32768`, `65536`, and `131072` block sizes but is
  slower than Java on this host (`0.426x`, `0.404x`, `0.419x`). The NBT/GZIP
  and threshold modules are shape counters, not encoder hooks; they pass
  equivalence and show native model-counting wins (`1.735x`, `1.830x`,
  `1.708x`, `1.699x`; `6.236x`, `5.301x`). Keep all three diagnostic-only;
  this is not a Paper runtime hook or strict-gate claim.

- 2026-05-13 22:38 CEST: `paper-native-core::obfhelper_maps` is the newest
  modular Rust rewrite checkpoint for the mapping-bootstrap hot path. It has
  a pure Rust model, JNI exports, a Java/native parity bench, and an
  executable script. The bench passes equivalence on the real `reobf.tiny`
  mapping jar (`7554` classes, `47786` methods, `31113` fields); native is
  slower than Java on this host because the fixture is string-heavy over JNI,
  so keep it diagnostic-only. This is not a Paper runtime hook or strict-gate
  claim.

- 2026-05-13 21:33 CEST: `paper-native-core::varint` now covers
  VarInt/VarLong size, write-batch, and read-batch parity through JNI, and
  `paper-native-core::plugin_startup_rollup` is the newest modular Rust
  rewrite checkpoint for combined plugin-name join plus startup log-name
  aggregation. Both benches pass equivalence. Native remains slower than Java
  on the VarInt/VarLong JNI shapes, while the optimized plugin-startup rollup
  is the useful same-runtime signal (`3.065x` normal and `3.137x` debug in
  Java; `1.937x` normal and `1.948x` debug in native). Keep both
  diagnostic-only; this is not a Paper runtime hook or strict-gate claim.

- 2026-05-13 16:58 CEST: `paper-native-core::waypoint_chunk_update` and
  `paper-native-core::remapper_hash_threshold` are the newest modular Rust
  rewrite checkpoints. Both have pure Rust parity models, JNI exports,
  Java/native parity benches, and executable bench scripts. The waypoint
  chunk-update bench passes equivalence; the same-runtime Java long-key shape
  is faster than Java distance (`2.587x`), but native JNI is slower than Java
  on both shapes (`0.266x`, `0.197x`). The remapper hash-threshold bench
  passes equivalence on `13` real plugin/library jars across subset sizes
  `1`, `2`, `4`, `8`, and `12`; native is slower than Java at size `12`
  (`0.646x`, `0.683x`, `0.602x`, `0.650x`), though native parallel beats
  native put (`2.579x`). Keep both diagnostic-only; this is not a Paper
  runtime hook or strict-gate claim.

- 2026-05-13 16:08 CEST: `paper-native-core::waypoint_snapshot`,
  `paper-native-core::waypoint_table_view`, and
  `paper-native-core::waypoint_manager_skip` are the newest modular Rust
  rewrite checkpoints. They have pure Rust parity models, Rust tests, JNI
  exports, Java/native parity benches, and executable bench scripts. The
  snapshot bench passes equivalence and native is faster on all three shapes
  (`18362.610x`, `28326.422x`, `12246.901x`); the table-view bench passes
  equivalence and native is faster on both shapes (`14612.526x`,
  `17070.012x`); the manager-skip bench passes equivalence and native is
  faster on all eight shapes (`3872.955x`, `2162.004x`, `3930.484x`,
  `2412.447x`, `2649.895x`, `2225.330x`, `4522.427x`, `4337.273x`). Keep
  all three diagnostic-only; this is not a Paper runtime hook or strict-gate
  claim.

- 2026-05-13 15:15 CEST: `paper-native-core::improved_noise_floor`,
  `paper-native-core::surface_rules_sequence_array`,
  `paper-native-core::surface_rules_test_rule_state`,
  `paper-native-core::placed_feature_traversal`,
  `paper-native-core::ore_feature_loop`, and
  `paper-native-core::ticketset_search` are the newest modular Rust rewrite
  checkpoints. They have pure Rust parity models, Rust tests, JNI exports,
  Java/native parity benches, and executable bench scripts. Improved-noise
  floor passes equivalence but native is slower than Java (`0.588x`,
  `0.701x`). Surface-rules sequence-array passes equivalence and native wins
  all four shapes (`2.456x`, `6.337x`, `1.938x`, `3.567x`). Surface-rules
  test-state passes equivalence and native is faster in the Java/native
  comparisons (`1.445x`, `1.321x`, `1.593x`, `1.410x`). Placed-feature
  traversal passes equivalence and native is faster than Java stream and
  recursive (`21.813x`, `29.905x`). Ore-feature loop passes equivalence and
  native wins old/optimized shapes (`1.593x`, `1.491x`). Ticketset-search
  passes equivalence and native wins binary/unchecked/linear shapes
  (`3.220x`, `3.209x`, `3.608x`, `3.174x`, `3.498x`). Keep all six
  diagnostic-only; this is not a Paper runtime hook or strict-gate claim.

- 2026-05-13 13:45 CEST: `paper-native-core::protochunk_heightmap`
  and `paper-native-core::range_choice` are the newest modular Rust rewrite
  checkpoints. Both have pure Rust parity models, Rust tests, JNI exports,
  Java/native parity benches, and executable bench scripts. The protochunk
  heightmap bench passes equivalence; native beats Java on both old and new
  loop shapes (`7.615x`, `1.344x`), while the Java cached-contains shape is
  `1.208x` faster than Java old and native cached-contains is slower than
  native old (`0.213x`). The range-choice bench passes equivalence on four
  scenarios and verifies Java/native `forIndex(...)` counts exactly; the
  useful signal remains same-runtime Java optimized-vs-old
  (`1.107x`, `1.059x`, `1.239x`, `1.034x`) because optimized native is slower
  than optimized Java on all measured scenarios. Keep both diagnostic-only;
  this is not a Paper runtime hook or strict-gate claim.

- 2026-05-13 13:10 CEST: `paper-native-core::climate_parameter_distance`
  and `paper-native-core::noise_generator_settings` are the newest modular
  Rust rewrite checkpoints. Both have pure Rust parity models, Rust tests,
  JNI exports, Java/native parity benches, and executable bench scripts. The
  climate parameter-distance bench passes equivalence and native is faster
  than Java on old/branch/subtract-first shapes (`3.124x`, `5.274x`,
  `3.072x`). The noise-generator-settings bench passes equivalence and
  native is faster than Java on all five shapes (`3.113x`, `6.056x`,
  `2.543x`, `3.514x`, `1.306x`). Keep both diagnostic-only; this is not a
  Paper runtime hook or strict-gate claim.

- 2026-05-13 13:08 CEST: `paper-native-core::chunk_expire_count`,
  `paper-native-core::craftplayer_cansee`, and
  `paper-native-core::levelchunk_heightmap` are the newest modular Rust
  rewrite checkpoints. All three have pure Rust parity models, Rust tests,
  JNI exports, Java/native parity benches, and executable bench scripts.
  Chunk-expire-count passes equivalence but native is slower on every
  measured shape. CraftPlayer can-see passes equivalence and native is much
  faster on all measured shapes. LevelChunk heightmap passes equivalence;
  native wins on the old four-update shape but loses on the new
  combined-update shape. Keep all three diagnostic-only; this is not a Paper
  runtime hook or strict-gate claim.

- 2026-05-13 09:15 CEST: `paper-native-core::nearby_player_map_capacity`
  is the newest modular Rust rewrite checkpoint. It now has a pure Rust
  model, Rust tests, JNI exports, a Java/native parity bench, and an
  executable bench script. The bench passes equivalence for both the 50- and
  500-player scenarios. Native is faster than Java on both scenarios
  (`69.919x` / `39.047x` at 50 players, `87.489x` / `41.880x` at 500
  players), while the Java same-runtime presized win remains
  `2.138x` / `2.543x`. Keep it diagnostic-only; this is not a Paper runtime
  hook or strict-gate claim.

- 2026-05-13 09:40 CEST: `paper-native-core::marker_cache` and
  `paper-native-core::waypoint_distance_guard` are the newest modular Rust
  rewrite checkpoints. Both have pure Rust models, Rust tests, JNI exports,
  Java/native parity benches, and executable bench scripts. Marker-cache
  passes equivalence; native wins only on the old non-cached shape (`1.311x`)
  and loses on the cached shape (`0.364x`), with the same-runtime cached
  summary also slower (`0.956x` Java). Waypoint-distance guard passes
  equivalence; native is slower on old range (`0.827x`), guarded range
  (`0.873x`), and old really-far (`0.905x`), and only slightly faster on
  guarded really-far (`1.018x`). Keep both diagnostic-only; this is not a
  Paper runtime hook or strict-gate claim.

- 2026-05-13 09:01 CEST: `paper-native-core::remapper_index_cleanup`,
  `paper-native-core::remapper_skip_hashes`, and
  `paper-native-core::plugin_directory_scan` are the newest modular Rust
  rewrite checkpoints. All three have pure Rust models, Rust tests, JNI
  exports, Java/native parity benches, and executable bench scripts. The
  remapper-index cleanup bench passes equivalence; native is slower than Java
  (`0.232x` old, `0.198x` new), but both runtimes still show the lazy cleanup
  reduction (`1.756x` Java, `1.493x` native). The skip-hashes bench passes
  equivalence and native is faster than Java (`2.314x` old stream, `2.840x`
  new loop), though Java old/new is neutral (`0.979x`). The plugin-directory
  scan bench passes equivalence and native is faster than Java on walk/list/
  directory-stream shapes (`2.267x`, `1.296x`, `1.190x`). Keep them
  diagnostic-only; this is not a Paper runtime hook or strict-gate claim.

- 2026-05-13 07:53 CEST: `paper-native-core::spigot_load_order_dependency`
  and `paper-native-core::topographic_graph_sort_capacity` are the newest
  modular Rust rewrite checkpoints. Both have pure Rust parity models, Rust
  tests, JNI batch exports, Java/native parity benches, and executable bench
  scripts. The Spigot load-order bench passes equivalence; native loses badly
  on the loadAfter copy shapes (`0.112x`/`0.116x`) but wins on the direct
  removed-count shape (`2.341x` vs Java new), while the Java same-runtime
  removed-count rewrite is `8.236x`. The topographic bench passes
  equivalence; native is slower than Java (`0.700x` old, `0.514x` new), but
  pre-sizing still improves Java `1.685x` and native `1.236x`. Keep both
  diagnostic-only; this is not a Paper runtime hook or strict-gate claim.

- 2026-05-13 07:19 CEST: `paper-native-core::plugin_loading_allocation`
  and `paper-native-core::legacy_provided_alias_removal` are the newest
  modular Rust rewrite checkpoints. Both have pure Rust parity models, Rust
  tests, JNI batch exports, Java/native parity benches, and executable bench
  scripts. The allocation bench passes equivalence; native is slower than
  Java in absolute terms, but native old/new setup improves `2.780x` and
  missing-set scan improves `1.173x` while validate is neutral/slightly worse
  (`0.980x`). The alias-removal bench passes equivalence; native beats old
  Java removeIf (`2.130x`) but loses to the already optimized Java
  reverse-index path (`0.422x`), with the Java same-runtime reverse-index
  signal at `11.962x`. Keep both diagnostic-only; this is not a Paper runtime
  hook or strict-gate claim.

- 2026-05-12 20:49 CEST: `paper-native-core::plugin_classloader_group` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust
  plugin-classloader lookup model for miss, hit-other, and hit-requester
  paths, Rust tests, JNI batch exports, and a Java/native parity bench. The
  bench passes equivalence and native is faster on five of six measured
  shapes on this host (`3.723x` miss old, `1.393x` miss skip, `2.418x`
  hit-other old, `0.918x` hit-other skip, `1.839x` hit-requester old,
  `1.314x` hit-requester skip). Keep it diagnostic-only; it is synthetic
  lookup evidence, not a Paper runtime hook.

- 2026-05-12 20:19 CEST: `paper-native-core::plugin_meta_dependency` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust
  dependency-list parity model for Paper plugin metadata required/
  soft/load-before/load-after extraction, Rust tests, JNI batch exports, and
  a Java/native parity bench. The bench passes equivalence; native beats the
  old stream path (`2.589x`) but loses to the already optimized Java loop
  (`0.840x`) and cached repeated access (`0.202x`) on this host. Keep it
  diagnostic-only; the same-runtime Java loop/cache rewrite remains the useful
  signal, not a native hook.

- 2026-05-12 19:42 CEST: `paper-native-core::plugin_name_join` and
  `paper-native-core::plugin_name_log` are the newest modular Rust rewrite
  checkpoints. Both now have pure Rust parity models, Rust tests, JNI batch
  exports, Java/native parity benches, and executable bench scripts. The
  benches pass equivalence, but native is slower than Java on the measured
  string-heavy JNI shapes: join normal `0.531x` / `0.489x`, join debug
  `0.356x` / `0.723x`, log TreeSet `0.904x`, and log ArrayList sort
  `0.343x`. Keep them diagnostic-only. The useful signal is still the
  same-runtime Java list rewrite in the log bench (`5.033x` ArrayList sort
  over TreeSet), not a native hook.

- 2026-05-12 18:22 CEST: `paper-native-core::shift_noise_direct` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust parity model
  for the helper/direct `ShiftNoiseDirectBench` current, direct, current-A,
  direct-A, current-B, and direct-B shapes, Rust tests, JNI batch exports,
  and a Java/native parity bench. The bench passes equivalence and native is
  faster than Java on all six measured shapes on this host (`8.624 ms`
  current default Java vs `8.006 ms` native, `8.898 ms` direct default Java
  vs `8.505 ms` native, `8.660 ms` current A Java vs `7.627 ms` native,
  `8.833 ms` direct A Java vs `7.677 ms` native, `8.537 ms` current B Java
  vs `8.056 ms` native, `11.097 ms` direct B Java vs `8.069 ms` native).
  Keep it diagnostic-only; it is parity evidence for the helper math, not a
  Paper runtime hook.

- 2026-05-12 18:55 CEST: `paper-native-core::entity_bounding_box` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust parity model
  for old `EntityDimensions.makeBoundingBox(...)` plus `setBoundingBox(...)`
  and the direct dimensions-based `setBoundingBox(...)` shape, Rust tests,
  JNI batch exports, and a Java/native parity bench. The bench passes
  equivalence and native is faster than Java on both measured shapes on this
  host (`1894.853 ms` old make-then-set Java vs `395.719 ms` native,
  `813.001 ms` direct dimensions Java vs `406.124 ms` native). The Java
  direct path is still `2.331x` faster than Java old and halves sample
  allocation (`1536000000` to `768000000` bytes), while native direct is
  `0.974x` vs native old. Keep it diagnostic-only: the previous
  `Entity.setPosRaw(...)` bounding-box shortcut was rejected and rolled back
  by runtime gate evidence, so this native module is not enough for a Paper
  hook.

- 2026-05-12 18:10 CEST: `paper-native-core::entity_lookup_status` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust
  `EntityLookup.getEntityStatus(...)` parity model, Rust tests, JNI batch
  exports, and a Java/native parity bench over old/direct status and
  old/direct accessibility shapes. The bench passes equivalence and native is
  faster on all four measured shapes on this host (`579.948 ms` old status
  Java vs `251.147 ms` native, `588.884 ms` direct status Java vs
  `251.209 ms` native, `721.905 ms` old accessible Java vs `258.580 ms`
  native, `685.592 ms` direct accessible Java vs `258.715 ms` native).
  Keep it diagnostic-only: previous EntityLookup runtime candidates were
  rejected by real load testing, so this native evidence is not enough for a
  Paper hook.

- 2026-05-12 17:26 CEST: `paper-native-core::chunk_dependencies` and
  `paper-native-core::ownable_rule` are the newest modular Rust rewrite
  checkpoints. `chunk_dependencies` now has a pure Rust dependency-radius
  parity model, Rust tests, JNI batch exports, and a Java/native parity bench.
  The bench passes equivalence and native is faster on both measured shapes
  on this host (`791.860 ms` old Java vs `477.905 ms` native,
  `794.043 ms` array Java vs `482.147 ms` native). `ownable_rule` now has a
  pure Rust descriptor-owner matching model, Rust tests, JNI batch exports,
  and a Java/native parity bench. The bench passes equivalence and native is
  faster on both measured shapes (`1711.676 ms` old stream Java vs
  `314.278 ms` native, `626.597 ms` new loop Java vs `254.995 ms` native).
  Keep both diagnostic-only until there is a guarded runtime hook with
  fallback, plugin matrix coverage, and strict-gate proof.

- 2026-05-12 16:12 CEST: `paper-native-core::noisechunk_interpolator_array`
  and `paper-native-core::noisechunk_flatcache_context` are the newest
  modular Rust rewrite checkpoints. The interpolator-array module now has a
  pure Rust list/indexed/array parity model, Rust tests, JNI batch exports,
  and a Java/native parity bench. The bench passes equivalence and native is
  faster on all three measured shapes on this host (`1174.474 ms` Java list
  vs `695.013 ms` native list, `1069.111 ms` Java indexed list vs
  `686.470 ms` native indexed list, `1145.747 ms` Java array vs
  `731.872 ms` native array). The flat-cache-context module now has a pure
  Rust old/new false and old/new true parity model, Rust tests, JNI batch
  exports, and a Java/native parity bench. The bench passes equivalence, but
  native is slower on all measured shapes (`108.479 ms` Java old false vs
  `137.700 ms` native old false, `89.412 ms` Java new false vs `104.712 ms`
  native new false, `1.038 ms` Java old true vs `1.074 ms` native old true,
  `1.006 ms` Java new true vs `1.083 ms` native new true). Keep both
  diagnostic-only until there is a guarded runtime hook with fallback and
  strict-gate proof.

- 2026-05-12 15:37 CEST: `paper-native-core::noisechunk_blendcache` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust
  empty-blender blend-cache parity model, Rust tests, JNI batch exports, and
  a Java/native parity bench. The bench passes equivalence; native is slower
  on the old allocation-heavy path (`417.205 ms` Java vs `739.598 ms`
  native) and faster on the no-allocation shape (`10.404 ms` Java vs
  `5.234 ms` native). Keep it diagnostic-only. This is not a restoration of
  the previously rejected empty-blendcache Paper runtime patch.

- 2026-05-12 15:37 CEST: `paper-native-core::noise_interpolator_slice` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust jagged vs
  flat slice parity model, Rust tests, JNI batch exports, and a Java/native
  parity bench. The bench passes equivalence; native loses on old jagged
  (`279.685 ms` Java vs `415.066 ms` native) but wins on flat (`304.545 ms`
  Java vs `261.091 ms` native). Keep it diagnostic-only until there is a
  guarded runtime hook with fallback and strict-gate proof.

- 2026-05-12 15:21 CEST: `paper-native-core::noise_interpolator_fractions`
  is the newest modular Rust rewrite checkpoint. It now has a pure Rust
  fraction-lookup parity model, Rust tests, JNI batch exports, and a
  Java/native parity bench. The bench passes equivalence and native is
  faster on both measured shapes on this host (`17.238 ms` Java division vs
  `12.280 ms` native division, `11.919 ms` Java array fraction vs
  `11.437 ms` native array fraction), but keep it diagnostic-only until
  there is a guarded runtime hook with fallback and strict-gate proof.

- 2026-05-12 15:00 CEST: `paper-native-core::carver_iteration` is the newest
  modular Rust rewrite checkpoint. It now has a pure Rust
  `CaveCarver` iteration parity model, Rust tests, JNI batch exports, and a
  Java/native parity bench. The bench passes equivalence and native beats
  Java on both measured shapes on this host (`133.704 ms` Java foreach vs
  `64.958 ms` native foreach, `89.380 ms` Java indexed vs `76.765 ms`
  native indexed), but keep it diagnostic-only. The native indexed shape is
  still slower than native foreach, so the shape choice still needs care and
  any runtime hook still needs a guarded fallback and strict-gate proof.

- 2026-05-12 14:59 CEST: `paper-native-core::cave_carver_skip` is the newest
  modular Rust rewrite checkpoint. It now has a pure Rust cave-floor skip
  parity model, Rust tests, JNI batch exports, and a Java/native parity
  bench. The bench passes equivalence, but JNI overhead makes every native
  shape slower on this host (`61.044 ms` Java old vs `83.470 ms` native old,
  `58.211 ms` Java reused vs `89.915 ms` native reused, `58.899 ms` Java
  direct vs `80.163 ms` native direct), so keep it diagnostic-only until
  there is a guarded runtime hook with fallback and strict-gate proof.

- 2026-05-12 14:34 CEST: `paper-native-core::serverentity_delta_identity`
  is the newest modular Rust rewrite checkpoint. It now has a pure Rust
  `ServerEntity.sendChanges()` delta-motion parity model, Rust tests, JNI
  batch exports, and a Java/native parity bench. The bench passes
  equivalence; native beats the old Java distance path (`193.459 ms` Java vs
  `151.916 ms` native), but native is slower than the already optimized Java
  identity guard (`110.046 ms` Java vs `116.559 ms` native). Keep it
  diagnostic only and do not replace the existing Java runtime guard with a
  JNI hook.

- 2026-05-12 14:13 CEST: `paper-native-core::static_cache_get` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust
  `StaticCache2D.get(...)` parity model, Rust tests, JNI batch exports, and a
  Java/native parity bench. The bench passes equivalence, but native is
  slower on this host (`733.176 ms` Java old vs `944.437 ms` native old,
  `693.851 ms` Java new vs `864.624 ms` native new), so keep it diagnostic
  only. This is not a restoration of the rejected single-offset runtime
  shape; any runtime hook still needs a guarded fallback and strict-gate
  proof.

- 2026-05-12 13:56 CEST: `paper-native-core::cubic_spline_create` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust
  `CubicSpline` create/min-max scan parity model, Rust tests, JNI batch
  exports, and a Java/native parity bench. The bench passes equivalence and
  native is faster on this host (`120.308 ms` Java iterator vs `86.421 ms`
  native iterator, `114.063 ms` Java index vs `80.360 ms` native index), but
  keep it diagnostic-only. This is not a restoration of the rejected
  `CubicSpline.Multipoint.mapAll` runtime cleanup; any runtime hook still
  needs a guarded fallback and strict-gate proof.

- 2026-05-12 13:38 CEST: `paper-native-core::jigsaw_canattach` is the newest
  modular Rust rewrite checkpoint. It now has a pure Rust
  `JigsawBlock.canAttach(...)` parity model, Rust tests, JNI batch exports,
  and a Java/native parity bench. The bench passes equivalence and native is
  much faster on this host (`1144.244 ms` Java old vs `36.889 ms` native old,
  `1039.042 ms` Java optimized vs `31.782 ms` native optimized,
  `294.473 ms` Java target-first vs `27.068 ms` native target-first), but keep
  it diagnostic-only. This is not a restoration of the rejected target-first
  Paper runtime patch; any runtime hook still needs a guarded fallback and
  strict-gate proof.

- 2026-05-12 13:11 CEST: `paper-native-core::spring_feature_mutable_pos` is
  the newest modular Rust rewrite checkpoint. It now has a pure Rust
  SpringFeature neighbor-check parity model, Rust tests, JNI batch exports,
  and a Java/native parity bench. The bench passes equivalence and native is
  faster on this host (`744.758 ms` Java old vs `410.222 ms` native old,
  `714.250 ms` Java mutable vs `467.562 ms` native mutable), but keep it
  diagnostic-only until there is a guarded Paper hook with fallback and
  strict-gate proof.

- 2026-05-12 12:12 CEST: `paper-native-core::biome_getbiome` is the newest
  modular Rust rewrite checkpoint. It now has a pure Rust biome
  corner-selection parity model, Rust tests, JNI batch exports, and a
  Java/native parity bench. The bench passes equivalence and the native path
  is faster on this host (`152.722 ms` Java current vs `132.699 ms` native
  current, `194.038 ms` Java optimized vs `170.491 ms` native optimized),
  but keep it diagnostic-only until there is a guarded Paper hook with
  fallback and strict-gate proof.

- 2026-05-12 11:30 CEST: `paper-native-core::beardifier_bury` is the newest
  modular Rust rewrite checkpoint. It now has a pure Rust distance-falloff
  parity model for `Beardifier.getBuryContribution(...)`, Rust tests,
  JNI batch exports, and a Java/native parity bench. The bench passes
  equivalence, but the native path is slower on this host (`16.415 ms` Java
  current vs `46.555 ms` native current, `12.785 ms` Java optimized vs
  `47.140 ms` native optimized), so keep it diagnostic-only until there is a
  guarded Paper hook with fallback and strict-gate proof.

- 2026-05-12 11:09 CEST: `paper-native-core::yclamped_gradient` is the
  newest modular Rust rewrite checkpoint. It now has a pure Rust clamped-map
  parity model, randomized Rust tests, JNI batch exports, and a Java/native
  parity bench. The bench passes equivalence, but the native path is slower
  on this host (`27.653 ms` Java current vs `60.910 ms` native current,
  `27.587 ms` Java optimized vs `63.403 ms` native optimized), so keep it
  diagnostic-only until there is a guarded Paper hook with fallback and
  strict-gate proof.

- 2026-05-12 10:38 CEST: `paper-native-core::xoroshiro_positional_direct` is
  the newest modular Rust rewrite checkpoint, and
  `paper-native-core::aquifer_positional_location` was re-run on the same
  release library. Both have pure Rust parity models, randomized Rust tests,
  JNI batch exports, and Java/native parity benches. The new
  `xoroshiro_positional_direct` bench passes equivalence and is faster on
  this host on every measured shape (`30.232 ms` Java old float vs
  `11.511 ms` native old float, `16.653 ms` Java direct float vs
  `11.612 ms` native direct float, `27.598 ms` Java old double vs
  `10.119 ms` native old double, `13.453 ms` Java direct double vs
  `10.273 ms` native direct double). The rerun
  `aquifer_positional_location` bench still passes equivalence; the old path
  remains faster (`27.402 ms` Java old vs `18.813 ms` native old), but the
  direct path is now slightly slower on this host (`17.361 ms` Java direct
  vs `17.858 ms` native direct). Keep both diagnostic-only until there is a
  guarded Paper hook with fallback and strict-gate proof.

- 2026-05-12 09:39 CEST: `paper-native-core::aquifer_index_stride` is the newest
  modular Rust rewrite checkpoint, and `paper-native-core::aquifer_surface_sampling`
  was reverified on the same release library. Both have pure Rust parity
  models, randomized Rust tests, JNI batch exports, and Java/native parity
  benches. The `aquifer_index_stride` bench passes equivalence and is faster
  on this host (`288.438 ms` Java old vs `263.596 ms` native old,
  `319.463 ms` Java new vs `263.117 ms` native new), and the re-run
  `aquifer_surface_sampling` bench stays faster too (`295.584 ms` Java old
  vs `275.199 ms` native old, `272.365 ms` Java new vs `230.479 ms`
  native new). Keep both diagnostic-only until there is a guarded Paper hook
  with fallback and strict-gate proof.

- 2026-05-12 08:58 CEST: `paper-native-core::blended_noise` is the next
  modular Rust rewrite checkpoint. It now has a pure Rust synthetic
  BlendedNoise octave-lookup parity model, randomized Rust tests, JNI summary
  exports for old/cached paths, and a Java/native parity bench. The bench
  passes equivalence, but the native summary is slower on this host
  (`629.502 ms` Java old vs `760.718 ms` native old, `687.385 ms` Java
  cached vs `795.017 ms` native cached), so keep it diagnostic-only until
  there is a real guarded Paper hook with fallback and strict-gate proof.

- 2026-05-12 08:44 CEST: `paper-native-core::perlin_noise` is the next
  modular Rust rewrite checkpoint. It now has a pure Rust octave-loop parity
  model, randomized Rust tests, JNI summary export, and a Java/native parity
  bench over `PerlinNoise.getValue(...)`. The latest batch shows the
  no-y-scale variant ahead on this host after the loop-shape rewrite, but keep
  it diagnostic-only / explicit-opt-in until there is a real guarded Paper
  hook with fallback and strict-gate proof.

- 2026-05-12 08:30 CEST: `paper-native-core::improved_noise` is the next
  modular Rust rewrite checkpoint. It now has a pure Rust sample-and-lerp
  parity model, randomized Rust tests, JNI summary export, and a
  Java/native parity bench over the `ImprovedNoise` hot path. The bench
  passes equivalence and the native summary is slightly ahead on this host
  (`42.014 ms` Java vs `38.572 ms` native, `1.089x`), but keep it
  diagnostic-only until there is a real guarded Paper hook with fallback and
  strict-gate proof.

- 2026-05-12 08:14 CEST: `paper-native-core::chunk_ticket_stage` is the next
  modular Rust rewrite checkpoint. It now has a primitive long-byte map
  model, randomized Rust tests, JNI summary export, and a Java/native parity
  bench over the chunk-ticket-stage get-sweep and mutation-churn workload.
  The bench passes equivalence, but native loses on this host
  (`199.714 ms` Java vs `262.183 ms` native, `0.762x`), so it stays
  diagnostic-only and must not become a runtime hook.

- 2026-05-12 08:00 CEST: `paper-native-core::ticket_compare` is now a
  completed diagnostic checkpoint for the ticket ordering path. It has a pure
  Rust ordering model, randomized Rust tests, JNI summary export, and a
  Java/native parity bench. The bench passes equivalence, but native loses on
  this host (`190.711 ms` Java vs `222.437 ms` native, `0.857x`), so it stays
  diagnostic-only and must not become a runtime hook.

- 2026-05-12 07:45 CEST: `paper-native-core::ticket_pack` is the next
  modular Rust rewrite checkpoint. It now has a pure Rust persistent-ticket
  packing model, randomized Rust tests, JNI summary export, and a Java/native
  parity bench over the forced-ticket save path. The current bench passes
  equivalence but the native summary is slightly slower than the Java
  summary (`0.947x` on this host), so it stays diagnostic-only.

- 2026-05-12 07:14 CEST: `paper-native-core::reference_list` is now the next
  modular Rust rewrite checkpoint. It has a pure Rust integer-token model,
  randomized Rust tests, JNI summary export, and a Java/native parity bench
  across transition/dense/random workloads (`1.877x`, `1.536x`, `1.699x`
  native batch speedups, equivalence PASS). Keep it diagnostic-only until a
  concrete runtime hook has fallback behavior and strict-gate proof.

- 2026-05-12 02:42 CEST: the RTree bench harness is now more uniform. The
  build, lifecycle, and JNI search scripts all accept direct env overrides in
  addition to `JAVA_PROPS`, and direct-env plus `JAVA_PROPS` smoke tests
  passed for build and lifecycle. This is a harness and verification cleanup,
  not a search-path change.

- 2026-05-12 02:37 CEST: two follow-up `climate_rtree` candidates were
  rejected and kept out of the hot path. Generic batch helper dispatch did
  not beat the current clone-backed shape on the full 16-round run, and
  boxed child slices regressed the batch/JNI search shapes enough to roll back
  to the Vec-backed representation. Keep the accepted leaf-child-reuse search
  shape as the current native RTree baseline. The JNI bench script now also
  accepts direct `LEAVES`, `QUERIES`, `WARMUP`, and `ROUNDS` overrides in
  addition to `JAVA_PROPS`.

- 2026-05-12 02:16 CEST: the `climate_rtree` recursive current-search
  best-distance shortcut was benchmarked and rejected. The full
  `1400 / 120000 / 6 / 16` batch and JNI runs regressed the hot current
  random path versus the accepted clone-backed baseline
  (`625.279 ms` batch current random, `641.975 ms` JNI current random), even
  though some walk/bounded numbers moved slightly. The change was reverted
  back to exact-distance child checks inside `search_current_*`. Keep both
  public defaults clone-backed and keep direct-current / borrowed-current /
  borrowed-bounded / arena as diagnostics only, but still do not add a Paper
  runtime hook without fallback and strict gate evidence.

- 2026-05-12 01:34 CEST: `climate_rtree` batch defaults were split by the
  measured winner for one iteration. Current search stayed on the clone-backed
  batch path, bounded search briefly moved to the borrowed batch path, and a
  direct current specialization was tried and rolled back after repeated
  release runs stayed behind the clone-backed helper. The later leaf-child
  reuse pass moved bounded back to clone-backed.

- 2026-05-12 01:05 CEST: a second owned `climate_rtree` representation was
  benchmarked via `ArenaTree`, but it did not beat the existing Rc-backed
  batch lifecycle path on this host (`346.798 ms` vs `338.325 ms` random,
  `503.828 ms` vs `455.136 ms` bounded, walk also slower). Keep the arena
  variant only as a diagnostic alternative; do not replace the current path
  with it unless a different representation wins the same parity-backed
  lifecycle harness.

- 2026-05-12 00:52 CEST: `climate_rtree` now has a combined JNI lifecycle
  bench in addition to the separate build and search diagnostics. The
  measured build_search_free loop on `60000` queries still keeps the native
  handle path ahead of Java on both random and walk shapes
  (`317.413 ms` vs `987.637 ms` random current, `122.337 ms` vs
  `214.280 ms` walk bounded). This is the broadest synthetic signal so far,
  but it stays diagnostic until there is a guarded Paper use site, Java
  fallback, and strict server gate evidence.

- 2026-05-12 00:39 CEST: `climate_rtree` now has both JNI search and JNI
  build lifecycle coverage. The new build diagnostic measured native
  `buildTreeHandle + checksum + free` at `960.521 ms` versus optimized Java
  build at `2788.949 ms` over `200` repeated builds, with tree checksum parity.
  This removes a synthetic build-cost concern, but it still stays diagnostic
  until there is a guarded Paper use site, Java fallback, and strict server
  gate evidence.

- 2026-05-12 00:27 CEST: `paper-native-core::climate_rtree` now has a JNI
  handle lifecycle bench in addition to the standalone pure Rust diagnostic.
  The Rust handle path still beats the Java baseline on both measured
  workloads, and the JNI batch path keeps the win after crossing Java ↔ Rust
  (`624.319 ms` vs Java `1845.883 ms` random, `287.166 ms` vs `471.294 ms`
  walk). Keep this as the next serious runtime candidate, but do not hook it
  into Paper until the tree lifetime, fallback behavior, and strict server
  gate are designed and measured.

- 2026-05-11 23:58 CEST: the Rust climate work now includes a standalone
  `paper-native-core::climate_rtree` diagnostic and a native RTree search
  benchmark. Against the same synthetic `1400 leaves x 120000 queries`
  workload, Java bounded search measured `1816.289 ms` random and
  `470.331 ms` walk, while Rust measured `1087.711 ms` random and
  `250.479 ms` walk with equivalence PASS. Native current search was even
  better on random (`1069.431 ms`) but still a little slower than bounded on
  walk (`266.218 ms` vs `250.479 ms`). The input/tree/query/search checksums
  match too. Keep this as the next serious runtime candidate, but do not hook
  it into Paper until the tree lifetime, JNI shape, fallback behavior, and
  strict server gate are designed and measured.

- 2026-05-11 23:30 CEST: `paper-native-core::climate` now has both the batch
  distance sum and a batch best-match path wired through JNI and the
  `bench/native-climate` harness. On this host both paths still beat Java
  (`44.859 ms` vs `198.545 ms` for the sum path, `95.798 ms` vs `132.167 ms`
  for best-match), so the module stays diagnostic for now. Keep it modular,
  keep the tie-break behavior explicit, and do not turn either call into a
  runtime hook until there is a guarded Paper use site and a strict server
  gate.

- 2026-05-11 23:16 CEST: `paper-native-core::climate` is now a new bulk Rust
  module for 7-parameter climate distance sums. The batch JNI path wins on
  this host (`38.319 ms` vs Java `213.850 ms` on `1024 x 8192 x 7`), so it
  is worth keeping as a diagnostic module and candidate for a guarded runtime
  use site. The Paper LZ4 stream wrapper was removed from runtime after the
  stream bench stayed slower than the current Java buffered default
  (`4365.214 ms` vs `3292.509 ms`); keep only the diagnostic helper and do not
  reintroduce the Paper hook without new evidence.

- 2026-05-11 22:12 CEST: `paper-native-core::compression` now uses the
  selected C LZ4 backend through `lz4 = 1.28.1`. The temporary `lz4_flex`
  compressor was removed after the side-by-side bench: it was faster, but it
  produced larger streams. The retained `native_lz4` path cross-checks Java
  and native streams in both directions and now matches Java's compressed
  bytes on the current region-shaped workload (`74568143`, ratio `0.9877`)
  while running faster (`277.301 ms` vs Java LZ4 `321.627 ms`). Keep it
  diagnostic until a guarded Paper runtime hook, fallback path, and strict
  server gate exist.

- 2026-05-11 19:15 CEST: the Rust migration now has a third pure module,
  `paper-native-core::hash`, with SHA-256 over large byte slices. Even after
  enabling `sha2`'s asm backend, the native path on this host is still slower
  than Java (`149.903 ms` vs `95.743 ms` on 8x4 MiB buffers), so the module
  stays diagnostic only for now. The next Rust target should still prefer
  pure functions with more compute per boundary crossing, or a different
  bridge strategy, not a runtime hook.

- 2026-05-11 19:06 CEST: the Rust migration is now split into small pure
  modules instead of a full rewrite. `paper-native-core::varint` and
  `paper-native-core::position` are both in place. The direct JNI path is
  still slower than Java on this machine even after adding a combined batch
  call for `position` (`31.251 ms` vs `3.959 ms` combined,
  `7.654 ms` vs `1.685 ms` chunk pack, `5.152 ms` vs `1.013 ms` chunk hash,
  `11.917 ms` vs `1.894 ms` section pack, `12.337 ms` vs `4.138 ms` write,
  `12.438 ms` vs `4.172 ms` size), so both modules stay diagnostic only. The
  next Rust target should be a similarly small pure module with more work per
  boundary crossing or a different bridge strategy, not a direct runtime hook
  yet.

- 2026-05-10 16:30 CEST: `NoiseChunk.wrapped` map capacity was measured and
  then tested as a temporary production candidate. The new diagnostic bench
  showed overworld-like routers consistently produce `9361` wrapped entries
  and grow the fastutil table to `n=16384`, while nether/caves/floating-islands
  stay at `52` and end at `41`. Synthetic map insertion/get work liked a
  no-rehash `8192` expected size (`4.216x` vs current-shape `2048`), but the
  strict 50-bot 32/32 gate rejected the real patch:
  `17.74 TPS / 84.37 ms / 2557 chunks`, `watchdog_thread_dumps=4`,
  `nearby_players_stack_hits=8`, with valid preflight (`load_per_cpu=0.508`,
  `idle_percent_1s=79.03`). The temporary `0051` patch was removed, runtime
  rebuilt on 912 patches, artifact hashes passed, and rollback
  plugin/restart/forced-ticket gates passed (`Done 27.420s`, `19.327s`,
  `15.171s/9.479s`). Keep the diagnostic bench, but do not promote simple
  `NoiseChunk.wrapped` pre-sizing without a different real-load profile.

- 2026-05-10 15:54 CEST: `RegionizedPlayerChunkLoader.PlayerChunkLoaderData.update()`
  unused `manhattanDistance` removal was tested and rejected. The temporary
  `0051` patch built cleanly, bytecode confirmed the extra `abs(dx)+abs(dz)`
  was gone, and plugin/restart/forced-ticket gates passed. A clean strict
  50-bot 32/32 spectator gate then failed the accepted reference:
  `17.17 TPS / 52.33 ms / 2633 chunks` with `watchdog_thread_dumps=4` and
  `nearby_players_stack_hits=2`, while preflight was valid
  (`load_per_cpu=0.545`, `idle_percent_1s=77.83`). The patch was removed,
  runtime rebuilt on 912 source patches, artifact hash verification passed,
  and rollback plugin/restart/forced-ticket gates passed (`Done 28.309s`,
  `18.328s`, `13.538s/9.224s`). Do not retry this single unused-local removal
  as a load-performance candidate without a new profile reason.

- 2026-05-10 12:39 CEST: `SimpleListPluginClassLoaderGroup.getClassByName(...)`
  now skips the requester on the fallback group scan when class prioritization
  is enabled. Focused bench: miss `1.084x`, other-loader hit `1.257x`,
  requester-hit `0.825x`, equivalence PASS. Full build, artifact hashes,
  plugin matrix, restart/recovery, and forced-ticket persistence passed, but
  strict 50-bot 32/32 was blocked by host preflight
  (`load_per_cpu=1.540`, `idle_percent_1s=28.08`). Keep this as classloader
  lookup reduction only; no end-to-end startup/TPS claim.

- 2026-05-10 12:34 CEST: two follow-up focused candidates were rejected
  before production changes. `StaticCache2D.get(...)` single-offset lookup was
  bit-equivalent but slower (`0.957x`). `ReferenceList.add(...)`
  transition-add clear-removal was also equivalent but slower on the transition
  case (`0.799x`) and neutral/slower on pair/dense cases (`1.005x` /
  `0.981x`). Do not retry these exact shapes without new profile evidence.

- 2026-05-10 12:13 CEST: `SurfaceRules.TestRuleSource.apply(...)` now
  specializes the exact `BlockRuleSource` follow-up case into a
  `TestStateRule`, so surface-rule samples check the same condition and return
  the same `BlockState` directly instead of routing through
  `StateRule.tryApply(...)`. Focused benchmark:
  `mostly_true_speedup=1.025x`, `mostly_false_speedup=1.029x`,
  equivalence PASS. Full build, artifact hashes, plugin matrix
  `Done 34.742s`, restart/recovery `Done 19.091s`, and forced-ticket
  persistence `16.112s/11.165s` pass. Strict 50-bot 32/32 is blocked by host
  preflight (`load_per_cpu=0.985`), so this is a narrow worldgen rule-dispatch
  reduction only, not an end-to-end TPS/MSPT or 500-player claim.

- 2026-05-10 11:48 CEST: `ChunkHolderManager.addExpireCount(...)` now uses
  an explicit `get(...)` / atomic `putIfAbsent(...)` fast path for
  `sectionToChunkToExpireCount` instead of `computeIfAbsent(...)`, preserving
  the same section/chunk keys and expire-count semantics. Focused benchmark:
  `dynamic_compute_hot_best_ms=333.257`,
  `dynamic_manual_hot_best_ms=277.137`, `1.203x`, equivalence PASS; cold
  create also moved `0.566 ms -> 0.478 ms`, `1.182x`. `applyPatches`, full
  build, artifact hashes, plugin matrix `Done 53.157s`, restart/recovery
  `Done 43.350s`, and forced-ticket persistence `28.668s/22.220s` pass.
  The strict 50-bot 32/32 gate is blocked by host preflight
  (`load_per_cpu=1.809`, `idle_percent_1s=13.93`) due to unrelated live Java
  server load, so this remains a narrow ticket-map lookup optimization, not
  an end-to-end TPS or 500-player claim. Follow-up microbench filters rejected
  `Ticket.compareTo` cached fields (`0.964x`), `TicketSet` unchecked/linear
  search (`0.789x..0.910x`), `SpringFeature` mutable neighbor position reuse
  (`0.971x`), and shift-noise direct helper removal (`0.968x..1.005x`), so do
  not promote those shapes without new evidence.

- 2026-05-10 11:39 CEST: `CompressionEncoder`'s Java `Deflater` fallback
  now feeds `ByteBuf.nioBuffer(...)` into `Deflater.setInput(ByteBuffer)`
  instead of copying into a temporary `byte[]`. The focused benchmark moved
  from `137.266 ms` to `131.327 ms` on heap input and from `129.531 ms` to
  `124.865 ms` on direct input, equivalence PASS. `applyPatches`, full build,
  artifact hashes, plugin matrix `Done 51.284s`, restart/recovery
  `Done 24.522s`, and forced-ticket persistence `20.692s/17.519s` pass.
  The strict 50-bot 32/32 gate is blocked by host preflight
  (`load_per_cpu=1.160`), so this remains a narrow fallback-path copy
  reduction, not an end-to-end TPS or 500-player claim.

- 2026-05-10 10:42 CEST: `NoiseChunk.NoiseInterpolator.compute(...)`
  fraction-array lookup was tested and rejected. The focused benchmark was
  bit-exact and strong (`29.308 ms -> 5.943 ms`, `4.932x`), but the strict
  50-bot 32/32 spectator gate failed the accepted reference:
  `16.75 TPS / 63.54 ms / 2891 chunks` with `watchdog_thread_dumps=3` and
  `nearby_players_stack_hits=7`. The production path is rolled back to the
  division-based `Mth.lerp3(...)` fractions. Rollback gates pass: build/hash/
  json, plugin matrix `Done 29.035s`, restart/recovery `Done 18.063s`, and
  forced-ticket persistence `16.174s/12.176s`. Do not retry this
  `NoiseInterpolator` fraction-array shape without new profile evidence.

- 2026-05-10 10:07 CEST: two movement/ticket-pressure candidates were
  rejected and rolled back. `PlayerChunkLoaderData` cached `ChunkTaskScheduler`
  / `ChunkHolderManager` fields passed functional gates but failed the strict
  50-bot 32/32 spectator gate at `17.45 TPS / 65.35 ms / 2412 chunks` with
  `watchdog_thread_dumps=4` and `nearby_players_stack_hits=8`.
  `NearbyPlayers.TrackedChunk.SPARSE_PLAYER_LIST_LINEAR_LIMIT=64` then
  regressed further at `16.90 TPS / 88.49 ms / 2365 chunks` with
  `watchdog_thread_dumps=6`. Production is back on the `limit=2` baseline and
  without cached player-loader manager fields. Final rollback gates pass:
  build/hash/json, plugin matrix `Done 29.443s`, restart/recovery
  `Done 21.228s`, and forced-ticket persistence `21.372s/11.272s`. Keep the
  harness fixes for runtime-cache invalidation and absolute CDS output paths.
  Next target should come from fresh JFR/load evidence, likely chunk-generation
  noise or a different movement/ticket shape; do not repeat `NearbyPlayers`
  limit 3/64 or map pre-size.

- 2026-05-10 08:39 CEST: the temporary `ProtoChunk` heightmap iterator
  candidate was fully rolled back. The feature patch was deleted, generated
  `ProtoChunk.java` no longer contains `HEIGHTMAP_TYPES`, and the rebuilt
  runtime passes artifact hashes, the 11-plugin matrix (`Done 26.859s`),
  restart/recovery (`Done 16.028s`), and forced-ticket persistence
  (`13.244s/9.550s`). The fresh strict 50-bot 32/32 spectator run on this
  rollback runtime was stable but not accepted: `18.08 TPS / 96.12 ms /
  2609 chunks` with `watchdog_thread_dumps=3`. Current hot spots remain the
  movement ticket path (`ChunkHolderManager.addExpireCount`,
  `RegionizedPlayerChunkLoader.flushDelayedTicketOps`,
  `ReferenceList.remove`) and worker-side chunk generation (`PerlinNoise`,
  `NoiseChunk`, `RangeChoice`). Do not retry the rejected ProtoChunk shape.

- 2026-05-10 08:05 CEST: the already-present `NoiseChunk` marker-wrapper
  cache finally got a clean strict 50-bot 32/32 rerun. It is still not an
  accepted load win: the run reached 50 bots and improved raw TPS/MSPT
  (`18.72 TPS / 42.07 ms`) but had low chunk coverage (`1806` chunks) and
  `watchdog_thread_dumps=3`. Jstacks again point at movement pressure in
  `NearbyPlayers`, `RegionizedPlayerChunkLoader.flushDelayedTicketOps`, and
  `WaypointTransmitter.EntityChunkConnection.update`. Keep this framed only as
  the earlier marker-wrapper allocation reduction; do not claim a 50-bot or
  20 TPS load result from it. `Climate.Parameter.distance(...)` explicit-branch
  variants were rejected at microbench stage (`0.961x` and `0.996x`), so no
  production source was changed there. Next comparable gate should be the
  already-built `ProtoChunk` heightmap iterator-removal candidate when host
  preflight clears.

- 2026-05-10 07:57 CEST: `OreFeature.doPlace(...)` scalar-hoist loop cleanup
  was rejected and removed. The focused loop benchmark was positive
  (`60.507 ms -> 58.403 ms`, `1.036x`, equivalence PASS), but the clean
  strict 50-bot 32/32 spectator gate passed preflight and failed the accepted
  runtime standard: `18.27 TPS / 65.21 ms / 2911 chunks` with
  `watchdog_thread_dumps=2` and `nearby_players_stack_hits=4`. The source
  patch `OreFeature.java.patch` was deleted, generated source no longer has
  `widthHeight`, `d5Squared`, or `d5d6Squared`, and the rollback runtime
  passes artifact hashes, plugin matrix `Done 26.953s`, restart/recovery
  `Done 17.037s`, and forced-ticket persistence `12.862s/8.382s`. Next work
  should use fresh JFR/load evidence instead of repeating this scalar-hoist
  ore-placement shape.

- 2026-05-10 07:40 CEST: `DensityFunctions.RangeChoice.fillArray(...)`
  constant-out specialization was rejected and removed. The focused benchmark
  improved mixed/constant branch shapes (`1.049x` to `1.366x`, equivalence
  PASS), but the clean strict 50-bot 32/32 spectator gate passed preflight and
  regressed badly: `17.63 TPS / 192.39 ms / 2768 chunks` with
  `watchdog_thread_dumps=5` and `nearby_players_stack_hits=4`. Patch
  `0041-Optimize-RangeChoice-constant-out-fillArray.patch` was deleted,
  `applyPatches` rebuilt without `RangeChoiceConstantOut`, and the rollback
  runtime passes artifact hashes, plugin matrix `Done 26.927s`,
  restart/recovery `Done 16.461s`, and forced-ticket persistence
  `12.714s/8.332s`. Next target should not come from this RangeChoice shape;
  continue with fresh JFR/load evidence, likely worldgen/noise or the already
  pending `OreFeature` gate.

- 2026-05-10 06:39 CEST: the
  `WaypointTransmitter.EntityChunkConnection.update()` long-key update
  condition was rejected. The standalone bench was strong
  (`80.686 ms -> 34.099 ms`, `2.366x`, equivalence PASS), but the strict
  50-bot 32/32 spectator gate regressed to `17.99 TPS / 63.66 ms / 2516
  chunks`. The runtime is rolled back to the chunk-distance update condition,
  while the accepted `lastChunkKey` visibility cache remains. Current
  rollback gates pass: `applyPatches` Applied 913 patches, full build,
  artifact hashes, plugin matrix `Done 27.799s`, restart/recovery
  `Done 16.968s`, and forced-ticket persistence `13.274s/8.602s`. A fresh
  post-rollback strict baseline on the 6-worker pinned-cpuset shape is stable
  at `18.29 TPS / 50.90 ms / 2441 chunks`, with no kicks/errors/watchdog/
  sync-load. Next work should move from already rejected waypoint/ticket
  shapes back to fresh JFR-backed worldgen/noise or allocation pressure.

- 2026-05-10 05:47 CEST: `NearbyPlayers.TrackedChunk.SPARSE_PLAYER_LIST_LINEAR_LIMIT`
  was tested at `3` and rejected. The first strict 50-bot 32/32 spectator run
  stayed stable at `18.06 TPS / 46.77 ms / 2396 chunks`, but the rerun
  regressed to `17.83 TPS / 62.80 ms / 2427 chunks`; both runs stayed below
  the accepted `18.27/47.85/2380` reference. In the same cycle,
  `PlayerChunkLoaderData.chunkTicketStage` pre-sizing to
  `new Long2ByteOpenHashMap(4096, 0.6F)` lost the focused bench
  (`0.903x` get, `0.983x` mutation) and was also rejected. Runtime is back on
  `limit=2`, and the rollback build/hash/plugin/restart/forced-ticket gates
  pass. Next target should still come from the movement/ticket-pressure
  cluster, but only with fresh profile evidence.

- 2026-05-10 03:40 CEST: `ReferenceList.remove(...)` now has a bounded
  transition fast path for tiny sparse lists, targeting
  `NearbyPlayers.TrackedChunk` lists with `linearSearchLimit=2`.
  Focused bench passed: transition remove `449.432 ms -> 324.509 ms`
  (`1.385x`), miss path `48.443 ms -> 13.181 ms` (`3.675x`), dense path
  neutral at `0.995x`, equivalence PASS. Final verification passes
  `applyPatches` (`913 patches`), full optimized build, artifact hashes,
  JSON, plugin matrix (`Done 26.747s`), restart/recovery (`Done 16.102s`),
  and forced-ticket persistence (`13.346s/8.585s`). The strict 50-bot 32/32
  spectator gate is still not accepted: `18.07 TPS / 51.73 ms / 2782 chunks`
  with `watchdog_thread_dumps=3`. Next target should come from the latest
  movement stacks around waypoint packet fanout and ticket pressure, not from
  already rejected map-capacity or TicketSet search variants.

- 2026-05-10 02:26 CEST: the `NearbyPlayers` player-map capacity pre-size
  candidate is rejected and rolled back. The standalone map benchmark was
  strong (`50 players 2.245x`, `500 players 2.443x`, rehashes removed,
  equivalence PASS), but the strict 50-bot 32/32 spectator gate came back at
  `17.95 TPS / 52.03 ms / 2059 chunks`, worse than the accepted
  `18.27/47.85/2380` reference. The rollback runtime rebuilds cleanly:
  `applyPatches` Applied 913 patches, hash manifest PASS, plugin matrix
  `Done (26.143s)`, restart/recovery `Done (16.191s)`, and forced-ticket
  persistence `12.884s/9.473s`. Do not retry this same map pre-size shape
  without new profile evidence; next target should come from fresh load/JFR
  evidence around chunk generation, movement, and plugin packet pressure.

- 2026-05-10 01:09 CEST: `ProtoChunk.setBlockState(...)` now avoids
  `EnumSet` iterator allocations in the heightmap priming/update scans by
  walking a cached `Heightmap.Types[]` and checking `EnumSet.contains(...)`.
  Focused bench: `133.632 ms -> 100.017 ms` (`1.336x`), iterator allocations
  `2 -> 0`, equivalence PASS. `applyPatches`, full optimized build, artifact
  hashes, plugin matrix (`Done 27.655s`), restart/recovery (`Done 15.839s`),
  and forced-ticket persistence (`13.433s/8.960s`) pass. Two strict 50-bot
  32/32 spectator runs completed without kicks/errors/watchdog/sync-load, but
  did not beat the accepted reference: `18.51 TPS / 54.42 ms / 2217 chunks`
  and `17.84 TPS / 46.13 ms / 2215 chunks` vs about `18.27/47.85/2380`.
  Next target still needs fresh load/JFR evidence around chunk generation and
  player movement pressure; do not claim end-to-end TPS from this patch.

- 2026-05-09 23:56 CEST: `Climate.RTree.SubTree.search(...)` now uses a
  bounded distance check against the current best leaf in the default search
  path. Focused bench: random queries `2068.513 ms -> 1856.135 ms` (`1.114x`)
  and random-walk queries `544.258 ms -> 450.860 ms` (`1.207x`), equivalence
  PASS. Build, hashes, plugin matrix (`Done 30.298s`), restart/recovery
  (`Done 22.644s`), and forced-ticket persistence (`18.153s/18.919s`) pass.
  Strict 50-bot 32/32 is blocked by host preflight (`load_per_cpu=1.657`,
  idle `33.05%`), and the forced noisy diagnostic is stability-only
  (`17.23 TPS / 58.78 ms / 1750 chunks`, no kicks/errors/watchdog/sync-load).
  Next work still needs a clean load/JFR window; no end-to-end TPS claim is
  made.

- 2026-05-09 23:36 CEST: `NoiseBasedChunkGenerator.applyCarvers(...)` now
  uses indexed `HolderSet` access instead of the `Iterable` iterator path from
  `BiomeGenerationSettings.getCarvers()`. Focused bench:
  `124.919 ms -> 85.075 ms` (`1.468x`) and `32.000` allocated bytes saved per
  synthetic iteration, with equivalence PASS. Build, hashes, plugin matrix
  (`Done 31.900s`), restart/recovery (`Done 25.501s`), and forced-ticket
  persistence (`15.097s/10.702s`) pass. The strict 50-bot 32/32 gate is
  blocked by host load (`load_per_cpu=0.962`, idle `27.15%`), and the forced
  noisy run is stability-only (`17.24 TPS / 95.80 ms / 1824 chunks`, no
  kicks/errors/watchdog/sync-load). Next target still needs a clean load/JFR
  window around chunk generation/noise; do not claim end-to-end TPS from this
  patch.

- 2026-05-09 23:12 CEST: `Climate.RTree.build(...)` now avoids a recursive
  stream/collector and two small default-allocation shapes during RTree
  construction. Focused bench: `543.404 ms -> 530.904 ms` (`1.024x`) and
  about `335900.6` bytes/build saved with equivalence PASS. Build, hashes,
  plugin matrix (`Done 30.428s`), restart/recovery (`Done 20.358s`), and
  forced-ticket persistence (`14.099s/9.797s`) pass. A clean strict 50-bot
  32/32 spectator gate stayed stable but did not beat the accepted load
  reference (`18.04 TPS / 56.39 ms / 2429 chunks` vs about
  `18.27/47.85/2380`), so this is startup/tree-build work reduction only.
  Next target should come from fresh JFR/load evidence around noise/chunk
  generation, because the 20 TPS / 500-bot goal is still not met.

- 2026-05-09 22:38 CEST: the brief `YClampedGradient.compute(...)`
  inline branch and `WaypointSnapshotBench` sized-array variant were both
  rejected on fresh reruns, so the current runtime stays on the previously
  accepted density visitor hook state. The current strict 50-bot 32/32 gate
  is still blocked by host load (`load_per_cpu=1.313`), so the next safe
  target remains a fresh load/JFR-backed hotspot rather than these weak
  scalar/snapshot shapes.

- 2026-05-09 22:06 CEST: `DensityFunctions.HolderHolder.mapAll(...)` and
  `DensityFunctions.MarkerOrMarked.mapAll(...)` now call the existing
  `DensityFunction.Visitor.applyHolder/applyMarker` hooks. This keeps generic
  visitor semantics the same while allowing the already-hooked `NoiseChunk`
  and `RandomState` visitors to skip temporary wrappers. Focused benchmark:
  `504.111 ms -> 21.346 ms`, `23.617x`, zero temporary holder/marker wrapper
  allocations, equivalence PASS. Build, artifact hashes, bytecode hook check,
  plugin matrix (`Done 38.265s`), restart/recovery (`Done 29.923s`), and
  forced-ticket persistence (`21.751s/15.859s`) pass. Strict 50-bot 32/32
  gate is blocked by host load (`load_per_cpu=1.679`), so this is not a
  TPS/MSPT or 500-player claim. Next target should still come from a fresh
  clean load/JFR window around chunk generation/noise and movement pressure.

- 2026-05-09 21:36 CEST: `JigsawBlock.canAttach(...)` target-first evaluation
  was rejected and rolled back. The focused benchmark was strong
  (`target_first_speedup=12.354x`, equivalence PASS), but the clean strict
  50-bot 32/32 spectator gate regressed (`17.28 TPS / 276.57 ms / 1540
  chunks`) against the accepted reference around `18.27/47.85/2380`.
  Post-rollback build/hash/plugin/restart/forced-ticket gates pass. A
  follow-up `WaypointTransmitter` distance-guard candidate was also rejected at
  focused-benchmark stage (`guarded_range_speedup=0.888x`,
  `guarded_really_far_speedup=0.880x`, equivalence PASS). Current direction:
  continue from a fresh load/JFR-backed hot path, not from these rejected
  branch-order/distance-guard shapes.

- 2026-05-09 20:51 CEST: the
  `DensityFunctions.Ap2.fillArray(ADD)` scratch-buffer candidate was rejected
  and rolled back. The focused benchmark was positive
  (`flat_speedup=3.536x`, `nested_speedup=1.573x`, equivalence and reentrant
  equivalence PASS), but the clean strict 50-bot 32/32 spectator gate failed
  the accepted reference (`17.75 TPS / 78.14 ms / 1933 chunks` vs about
  `18.27 TPS / 47.85 ms / 2380 chunks`). Post-rollback build/hash/plugin/
  restart/forced-ticket gates pass. Do not retry this same ThreadLocal ADD
  scratch shape without a new reason; continue from the next JFR/load-backed
  hotspot.

- 2026-05-09 20:12 CEST: the `Entity.setPosRaw(...)` direct
  dimensions-based bounding-box shortcut was rejected and rolled back. It won
  the focused allocation benchmark (`1.424x`, equivalence PASS), but the clean
  strict 50-bot 32/32 spectator gate failed the accepted reference
  (`17.58 TPS / 67.63 ms / 1721 chunks` vs `18.27 TPS / 47.85 ms / 2380
  chunks`). The next target should come from the latest JFR/load evidence
  around chunk generation/noise, `NearbyPlayers`, and GC pressure, not from
  another entity-bbox shortcut.

- 2026-05-09 18:45 CEST: the explicit `ReferenceList` small-mode experiment
  was rejected and rolled back. It improved the focused singleton/pair churn
  bench, but the noisy 50-bot 32/32 spectator diagnostic still showed
  `watchdog_thread_dumps=6` and only `824` loaded chunks, and strict load was
  blocked by busy-host preflight (`load_per_cpu=0.840`). The next measured
  target remains the `NearbyPlayers` / `ReferenceList.add(...)` movement hot
  path, but only with a candidate that survives a clean real gate.

## Current Validation

- 2026-05-09 18:10 CEST: `ServerLevel.updatePOIOnBlockStateChange(...)`
  now routes POI mutations through `runPoiUpdateOnServerThread(...)`, which
  runs inline only on the real server thread and otherwise uses
  `scheduleOnMain(...)`. This fixed the off-main POI crash seen in the noisy
  50-bot run. The `ServerWaypointManager` complete-row skip candidate is still
  provisional: build/hash/plugin/restart/forced-ticket pass, the strict
  50-bot 32/32 gate is blocked by busy-host preflight
  (`load_per_cpu=1.824`), and the stable noisy run is diagnostic-only
  (`17.84 TPS / 224.77 ms / 1796 chunks`, zero thread-check/off-main/stability
  failures). Next performance target remains the `NearbyPlayers` /
  `ReferenceList.add(...)` movement hot path.

- 2026-05-09 15:55 CEST: ticket-side search/compare candidates were rejected
  at focused-benchmark stage (`unchecked_binary_speedup=0.966x`,
  `linear4_speedup=0.945x`, `linear8_speedup=0.959x`,
  `linear12_speedup=0.973x`; cached ticket compare `0.996x`). The
  `ServerWaypointManager.snapshotEntries(...)` manual-copy candidate improved
  the standalone snapshot bench (`795.043 ms -> 489.372 ms`, `1.625x`) but
  failed the real 50-bot 32/32 spectator gate
  (`17.74 TPS / 37.32 ms / 2077 chunks`) with `watchdog_thread_dumps=3` and
  `nearby_players_stack_hits=8`. It was rolled back; production is restored to
  `map.entrySet().toArray(Entry[]::new)`. Current restored-baseline gates pass:
  hash manifest, plugin matrix `Done (29.440s)`, restart/recovery
  `Done (17.388s)`, and forced-ticket persistence `13.805s/9.338s`. Next
  target is the `NearbyPlayers` / `ReferenceList.add(...)` movement hot path,
  not ticket compare or waypoint snapshot.
- 2026-05-09 14:27 CEST: `ChunkHolderManager.getOrCreateEntityChunk(...)`
  now lazily allocates `AtomicBoolean` and `Thread.currentThread()` only on the
  non-transient entity load path. The focused mixed-path benchmark improved
  from `65.410 ms` to `61.437 ms` (`1.065x`) and cut allocated bytes from
  `140000000` to `20000000`, with equivalence PASS. Full build,
  `sha256sum -c reports/artifact-hashes.txt`, plugin matrix `Done (30.140s)`,
  restart/recovery `Done (19.105s)`, and forced-ticket persistence
  `14.037s/9.349s` passed. The strict 50-bot 32/32 spectator gate is blocked
  by host preflight (`load_per_cpu=1.003` > `0.750`), so this remains
  allocation reduction plus compatibility evidence only. The explicitly noisy
  diagnostic stayed stable at `18.21 TPS / 63.30 ms / 2295 chunks`, with no
  watchdog or sync-load hits, but it is not comparable to the accepted
  baseline.
- 2026-05-09 13:30 CEST: `CaveWorldCarver` floor-skip helper was measured
  and rejected. The direct helper bench improved (`1.171x` and zero checker
  allocations in the synthetic shape), but the strict 50-bot 32/32 spectator
  gate regressed to `17.79 TPS / 108.48 ms / 1867 chunks`. The patch was
  removed, the rollback runtime rebuilt, and the plugin matrix still passed at
  `Done (27.768s)`.
- 2026-05-09 12:45 CEST: `DensityFunctions.MarkerOrMarked.mapAll(...)`
  `applyMarker(...)` hook candidate was rejected and rolled back. The focused
  benchmark improved marker allocations (`175.121 ms -> 35.148 ms`,
  `4.982x`, `1,920,000 -> 84,000` allocations), but the strict 50-bot 32/32
  spectator gate failed the accepted reference (`17.84 TPS / 67.37 ms /
  2081 chunks`, no watchdog/sync-load). Production patch
  `0044-Use-applyMarker-hook-for-density-function-markers.patch` was removed.
  Fresh rollback build and hash checks pass; no new load or 500-player claim
  is made.
- 2026-05-09 12:29 CEST: `BlendedNoise` octave-cache candidate was rejected
  and rolled back. The focused benchmark improved repeated octave lookup
  (`675.507 ms -> 573.567 ms`, `1.178x`, equivalence PASS), but the strict
  50-bot 32/32 spectator gate failed the accepted reference
  (`17.93 TPS / 56.72 ms / 2079 chunks`, no watchdog/sync-load). Production
  patch `0044-Cache-BlendedNoise-octave-lookups.patch` was removed. Fresh
  rollback gates pass: build, hash manifest, plugin matrix `Done (28.079s)`,
  restart/recovery `Done (17.050s)`, forced-ticket persistence
  `12.727s/8.805s`, and a stable but not-target 50-bot rollback control
  (`17.85 TPS / 56.02 ms / 2176 chunks`).
- 2026-05-09 11:51 CEST: two `EntityLookup` movement-path experiments were
  rejected and rolled back: direct `FullChunkStatus -> Visibility` mapping and
  section-change-only status reads. The direct microbench was positive
  (`1.039x` status, `1.054x` accessible), but the strict 50-bot gate did not
  beat the accepted baseline (`17.53 TPS / 46.96 ms / 2083 chunks`). The
  status-skip candidate was blocked by busy-host preflight and its explicit
  noisy run regressed (`17.22 TPS / 45.42 ms / 1827 chunks`, one watchdog).
  Restored baseline gates pass: build, hash manifest, plugin matrix
  `Done (26.884s)`, restart/recovery `Done (16.695s)`, and forced-ticket
  persistence `13.839s/8.889s`. Current restored-baseline 50-bot run is stable
  but not target (`17.66 TPS / 47.78 ms / 1964 chunks`, no watchdog/sync-load).
- 2026-05-09 07:33 CEST: `ReferenceList` got an optional small-list linear mode,
  enabled only for `NearbyPlayers.TrackedChunk` player lists with threshold 2.
  Runtime microbench says singleton/pair churn improves
  (`2.513x` / `2.090x`), but dense 32-player churn regresses (`0.717x`), so
  this is not yet an accepted load win. Full build, artifact hashes, plugin
  matrix, restart/recovery, and forced-ticket persistence pass, but the strict
  50-bot gate is blocked by host preflight (`load_per_cpu=0.893`) and the
  noisy diagnostic only reaches `17.76 TPS / 48.46 ms / 2326 chunks` with one
  watchdog dump. The fresh jstack now points at waypoint packet send/update
  pressure through `WaypointTransmitter -> ProtocolLib NettyEventLoopProxy`,
  so that is the next movement-side profile.
- 2026-05-09 03:30 CEST: current artifact rebuilds cleanly, plugin matrix
  passes at `Done (30.599s)`, restart/recovery passes at `Done (18.990s)`,
  and forced-ticket persistence passes at `14.791s/10.665s`. The strict
  50-bot 32/32 spectator gate is blocked by host preflight
  (`load_per_cpu=0.885` > `0.750`), so the next actionable step is a clean
  comparable load gate on a quieter host before accepting or rejecting the
  movement no-sync-load candidate.
- 2026-05-09 build path is restored. The broken state was a worktree
  source-patch layer where `applySourcePatches` saw only one patch; after
  restoring missing source patches from the current git objects/index,
  `MC_EULA_AGREE=true ./scripts/build_optimized.sh` passed with
  `applySourcePatches: Applied 912 patches`, `applyFeaturePatches` PASS, and
  `createMojmapBundlerJar` PASS. Artifact hashes were refreshed and
  `sha256sum -c reports/artifact-hashes.txt` passes on the new
  `FBE33F5C9C15DFE407681ED1912619F0809570B13565512F7ABAD53BA7E2EB5C`
  runtime cache.
- Fresh runtime gates pass on the rebuilt artifact: plugin matrix
  `Done (27.348s)`, restart/recovery `Done (17.499s)`, and forced-ticket
  persistence `15.120s/9.131s`.
- Fresh pinned 50-bot 32/32 spectator run is not accepted despite strong
  TPS/MSPT (`19.52 TPS`, `26.57 ms avg`, no bot kicks/errors) because it hit
  `watchdog_thread_dumps=8` and `sync_load_stack_hits=7`. Current bottleneck:
  movement-triggered `ServerGamePacketListenerImpl.handleMovePlayer ->
  Entity.absSnapTo -> Level.getChunk -> ServerChunkCache.syncLoad` while
  fresh chunk generation is still catching up.
- No-cpuset/all-core diagnostic is rejected: 12 worker / 2 I/O threads loaded
  more chunks (`4764`) but regressed to `16.79 TPS` and `353.82 ms` average
  tick time with remaining sync-load/thread-dump hits.
- Latest `NoiseChunk` marker wrapper cache is a measured allocation win
  (`173.517 ms -> 33.489 ms`, `5.181x`, `1920000 -> 84000` marker
  allocations, equivalence PASS). Full build, artifact hashes, plugin matrix
  (`Done (31.651s)`), restart/recovery (`Done (18.882s)`), and forced-ticket
  persistence (`15.372s/10.768s`) passed. The strict 50-bot gate is still
  blocked by host preflight at `load_per_cpu=0.807`, and the noisy 50-bot run
  is diagnostic-only (`17.38/429.99/2745`), so this is not an accepted
  end-to-end load claim.
- A derivative inline `ImprovedNoise.sampleWithDerivative(...)` microbench was
  checked and rejected before production because it did not improve on this
  CPU/JIT shape (`56.989 ms -> 57.170 ms`, `0.997x`, equivalence PASS).
- Latest candidate `Beardifier.getBuryContribution(...)` direct branch was
  rejected and reverted. Its microbench improved `8.304 ms -> 7.063 ms`
  (`1.176x`, equivalence PASS), but the real 50-bot 32/32 spectator gate was
  worse than the accepted baseline: `17.97/65.67/2539` vs `18.27/47.85/2380`.
- Post-revert build/hash/plugin/restart/forced-ticket gates passed:
  `applyPatches` applied 911 patches, `build_optimized.sh` passed,
  `sha256sum -c reports/artifact-hashes.txt` passed, plugin matrix passed at
  `Done (27.842s)`, restart/recovery at `Done (17.406s)`, and forced-ticket
  persistence at `14.870s/10.043s`.
- Post-revert 50-bot 32/32 also completed without kicks/errors/watchdog or
  sync-load hits, but was not a new baseline (`16.57/112.19/3212`).
- `ProtoChunk.setBlockState(...)` now uses a cached `Heightmap.Types[]`
  traversal plus `EnumSet.contains(...)` to remove the iterator allocation
  from the hot heightmap update path.
- Microbench `reports/protochunk-heightmap-bench.txt` shows
  `138.483 ms -> 105.978 ms` (`1.307x`) and iterator allocations per set block
  dropped from `2` to `0`, with equivalence PASS.
- `applyPatches`, `rebuildPatches`, `build_optimized.sh`, `sha256sum -c
  reports/artifact-hashes.txt`, plugin matrix, restart/recovery, and
  forced-ticket persistence all passed on the rebuilt runtime.
- The strict 50-bot 32/32 spectator gate is blocked by host preflight at
  `load_per_cpu=0.792` on this machine, so there is no fresh end-to-end load
  claim yet.

## Completed

- Rejected and reverted `PalettedContainer.reencodeContents(...)`
  old-palette-id remap cache on top of the existing per-thread unpack scratch
  array. The synthetic remap-cache bench measured current previous-only remap
  `967.335 ms` vs cached palette-id remap `937.103 ms` (`1.032x`, equivalence
  PASS), but the real strict 50-bot 32/32 gate that passed preflight failed
  the accepted baseline (`16.48/76.59/2813` vs `18.27/47.85/2380`). The
  production path is back to scratch-only `PalettedContainer.reencodeContents`;
  post-revert build, artifact hashes, plugin matrix (`Done (31.022s)`),
  restart/recovery (`Done (20.065s)`), and forced-ticket persistence
  (`15.145s`/`10.071s`) passed. The strict post-revert gate is blocked by host
  preflight at `load_per_cpu=0.807`.
- Rejected and reverted `NoiseChunk` interpolator indexed-traversal candidate
  after its focused benchmark (`1.053x`, equivalence PASS) failed the strict
  50-bot 32/32 accepted baseline (`17.87/142.23/2336` vs
  `18.27/47.85/2380`). The production path is back to foreach/forEach
  interpolator traversal.
- Rejected and reverted `DensityFunctions.Spline` context-direct candidate
  after a real standalone benchmark (`1.544x`, `16.0` bytes saved per call)
  failed the strict 50-bot 32/32 gate (`18.51/66.01/2101` vs accepted
  `18.27/47.85/2380`). Post-revert build, plugin matrix, restart/recovery,
  forced-ticket persistence, and artifact hash verification passed. This is
  worldgen allocation cleanup only, not a load/TPS win.
- Upstream Paper `ver/1.21.10` source prepared.
- Vanilla and stock Paper oracle jars downloaded.
- Optimized Paper artifact rebuilt from source.
- Accepted with limits `OwnableRewriteRule.matchesOwner(...)` stream-free owner loop in the server jar. Microbench improved old stream path `2052.795 ms` to direct loop `326.972 ms` (`6.278x`) with equivalence PASS; full build, hash verification, plugin matrix (`Done (29.313s)`), restart/recovery (`Done (17.743s)`), forced-ticket persistence (`14.023s`/`9.280s`) and a 50-bot stability run passed. This is class-rewrite allocation reduction only, not a cold-start/TPS claim.
- Accepted with limits `ObfHelper` mapping bootstrap pre-size/manual map construction plus pre-sized `StringPool` backing map. Real mappings microbench improved old stream/default maps `257.222 ms` to current pre-sized maps+StringPool `196.038 ms` (`1.312x`, `1.122x` over the previous pre-sized-map path), equivalence PASS; full build, artifact hashes, plugin matrix (`Done (30.297s)`), restart/recovery (`Done (17.136s)`), forced-ticket persistence (`13.261s`/`9.103s`), and a strict 50-bot stability run passed without kicks/errors/watchdog/sync-load. Boot/load did not become new baselines (`17145 ms` optimized runtime boot; 50-bot `18.86/66.90/1825`), so this is mapping-bootstrap allocation reduction only.
- Accepted with limits `PluginInitializerManager.load(...)` name-log aggregation pre-size/sort/dedup rewrite. Real synthetic startup-shape benchmark improved `343.898 ms` tree-set log aggregation to `45.491 ms` array-list sort/dedup (`7.560x`), and full build, plugin matrix (`Done (32.863s)`), restart/recovery (`Done (23.341s)`), forced-ticket persistence (`21.545s`/`13.276s`), and artifact hash verification passed. This is plugin-startup logging allocation/work reduction only, not an end-to-end cold-start claim.
- Rejected manual `String.join(...)` replacement for plugin name logging: both normal and debug join shapes were slower (`0.643x` / `0.665x`), so keep the `String.join(...)` path.
- Rejected hybrid sequential `RemappedPluginIndex.hashInputs(...)` `computeIfAbsent`/`put` split after a short confirmation run: the results were noisy across small batch sizes and did not justify changing the current `computeIfAbsent` sequential path or the `PARALLEL_HASH_THRESHOLD=4` split.
- Rejected `Climate.RTree` bounded distance early-exit despite a positive microbench (`1.205x`) because the 50-bot 32/32 production gate failed the accepted baseline (`17.65/58.37/2620` vs `18.27/47.85/2380`). The patch was removed, runtime rebuilt, hash/plugin/restart/forced gates passed, and post-revert 50-bot completed without crash/watchdog/sync-load but was not a new baseline (`17.06/68.42/2758`).
- Rejected `NoiseChunk` empty-blender blend-cache allocation skip after measured 50-bot regression. Standalone allocation benchmark improved the modeled empty-blender path (`430.571 ms` old vs `10.449 ms` new, `41.207x`, equivalence PASS), but the production 50-bot 32/32 gate failed the accepted baseline (`17.96/158.83/2424` vs `18.27/47.85/2380`). The patch was removed, artifact rebuilt on `910 patches`, plugin matrix/restart/forced-ticket/hash passed, and the postrevert 50-bot rerun was stable but still below baseline (`17.79/86.26/2981`).
- Rejected `SurfaceRules.SequenceRule` runtime-array / indexed-loop candidate after measured 50-bot regression. Standalone microbench improved the modeled rule sequence path (`array_indexed_speedup=1.898x`, equivalence PASS), but the strict 50-bot 32/32 gate passed preflight and failed the accepted baseline (`15.95/117.42/1785` vs `18.27/47.85/2380`, with one moved-too-quickly warning). The patch was removed, runtime rebuilt on `910 patches`, and hash/plugin/restart/forced-ticket gates passed.
- Latest `ImprovedNoise.sampleAndLerp` local `byte[]` inline candidate was measured and rejected correctly: standalone microbench passed (`1.119x`), temporary build/plugin/restart/persistence gates passed, but the 50-bot 32/32 server gate failed the accepted baseline (`17.78/62.90/2693` vs `18.27/47.85/2380`), so the production patch was removed and the artifact rebuilt on 910 source patches.
- Rejected `ImprovedNoise.sampleAndLerp` switch-gradient lookup at microbench stage. It was bit-exact, but slower than the current flat-gradient table (`switch_vs_flat_speedup=0.838x`), so production source was not touched.
- Rejected `PerlinNoise.getValue` exact-class guarded direct-local path at microbench stage. It preserved subclass semantics, but regressed the current delegating path (`direct_local_guarded_speedup=0.981x`), so production source was not touched.
- Added a wider native diagnostic batch for the Java `PerlinGetValueBench` shapes: all six variants now have Rust/JNI parity coverage, but only the no-y-scale path was faster on the short run, so this remains a measurement branch and not a runtime change.
- Expanded the `NoiseChunk` wrapped-map matrix to 13 candidate shapes, including `expected_12288_075`, `expected_12289_075`, and `expected_16384_075`; the runtime hook is still blocked by the earlier strict-gate rejection, so keep treating this as diagnostic-only.
- Rejected C2ME/DivineMC arithmetic `ImprovedNoise.sampleAndLerp` shape at microbench stage. It was bit-exact, but slower than the current flat-gradient table (`arithmetic_vs_flat_speedup=0.924x`), so production source was not touched.
- Direct classpath runtime prepared.
- EULA-gated scripts implemented and verified.
- Real plugin matrix created.
- Real protocol join check added.
- AppCDS training path added and regenerated on each runtime build.
- Precomputed server and plugin remap cache paths added.
- Precomputed reversed mappings cache path added for plugin remapper startup; measured A/B was only `34.734s` vs `34.950s`, so it is infrastructure, not a claimed speedup.
- Plugin-remapper batch SHA-256 cache reuse added: exact plugin hash keys are computed once per batch and reused through the cache-miss fallback; a narrow 11-jar hash microbenchmark improved from `182.522 ms best` two-pass to `25.707 ms best` one-pass parallel, but clean end-to-end startup A/B is still pending.
- `scripts/precompute_plugin_remaps.sh` cache-hit handling fixed: if the server loads reversed mappings from the precomputed runtime cache and therefore does not generate a fresh `.paper-remapped/mappings/reversed/*.tiny`, the precompute harness now accepts the existing cache file. Full build and pinned plugin matrix passed after the fix.
- Precomputed plugin skip cache added: `skipped-hashes.txt` stores exact SHA-256 hashes for jars already proven by the real remapper to need no remap. Full build and plugin matrix passed, but noisy A/B/latest runs (`32.401s` enabled vs `29.630s` control) do not justify an end-to-end startup speed claim.
- Plugin-remapper batch miss path now reuses the already computed batch SHA when creating cache destinations and recording skip decisions, avoiding a second full jar read for remap/skip misses. Full build passed and pinned plugin matrix passed at `Done (32.998s)`, but this is not promoted as an end-to-end startup win because the host was busy.
- `Hashing.sha256(InputStream)` now hashes streams incrementally instead of copying the whole stream to a temporary `byte[]`; full build, plugin matrix, and restart/recovery passed. This is memory-copy reduction, not a measured startup speed claim.
- Plugin-library remap path now uses the same hash-aware cache flow as plugin batches, and precompute exports library skip hashes under a separate `libraries/` namespace. A real `LibraryProbe` Paper plugin loads `library-probe-dep.jar` through `PluginClasspathBuilder`/`JarLibrary`; full build, plugin matrix, restart/recovery, and forced-ticket persistence passed, but this is not promoted as an end-to-end startup win because the host was busy.
- Plugin remapper mappings/reversed-mappings startup is now delayed until after manifest skip checks. A targeted run with only a no-namespace Paper plugin plus no-namespace plugin library confirmed `mapping_load=not_started_for_skip_only`; full build, plugin matrix (`Done (32.836s)`), restart/recovery, and forced-ticket persistence passed. This is accepted as startup-work reduction, not a clean end-to-end boot-speed claim.
- Plugin remapper index cleanup is now lazy on stable all-cached batches: it avoids building the temporary input-hash set and cleanup iteration when the cached/skip entry count already matches current plugin inputs. A narrow matrix-sized microbench improved old eager cleanup `2060.532 ms` to `626.871 ms` (`3.287x`), and `rebuildPatches`, full build, plugin matrix (`Done (44.640s)`), restart/recovery, and forced-ticket persistence passed. Busy-host preflight blocked clean load/startup A/B, so this is startup-work reduction only.
- Plugin remapper index writes now use a dirty flag: stable cached restarts skip rewriting unchanged `.paper-remapped/*/index.json`. Targeted mtime check passed on all four index files after a second restart of the same `runs/plugin-matrix`; full build, plugin matrix (`Done (36.844s)`), restart/recovery, and forced-ticket persistence passed. This is shutdown/restart disk-I/O reduction, not an end-to-end boot-speed claim.
- ReobfServer now checks and installs the precomputed remapped-server jar before starting the expensive reobf mappings load. A targeted run without precomputed plugin-remaps still remapped CompatProbe and recorded `loading_reobf_mappings_count=0`; full build, plugin matrix (`Done (32.332s)`), restart/recovery, and forced-ticket persistence passed. This is first-run remap startup-work reduction, not an end-to-end boot-speed claim.
- Precomputed remap installs now use atomic hard-link-or-copy for remapped server and plugin jars. Targeted checks confirmed 4 plugin remap artifacts and the server remap artifact were installed under the same `.paper-remapped` destination paths as hard links, with fallback to copy for unsupported filesystems. Full build, plugin matrix (`Done (47.750s)` on a heavily busy host), restart/recovery, and forced-ticket persistence passed. This is disk-I/O reduction, not an end-to-end boot-speed claim.
- Plugin directory discovery now uses `Files.list(...)` with try-with-resources for the flat plugin folder instead of `Files.walk(..., depth=1)`, skips the no-op `--add-plugin` provider path when no add-plugin files are present, and small add-plugin/log-name startup paths avoid stream/Formatter allocation. A matrix-sized scan microbench improved `220.139 ms` to `123.419 ms` (`1.784x`), and full build, plugin matrix (`Done (39.186s)`), restart/recovery, forced-ticket persistence, and artifact hash verification passed. This is narrow plugin-discovery work reduction, not an end-to-end boot-speed claim.
- Paper plugin metadata dependency-list accessors now use direct loops and cached immutable list results instead of rebuilding stream-derived lists on repeated access, while preserving dependency iteration order. A synthetic metadata dependency benchmark measured old stream `1960.882 ms`, direct loop `566.406 ms`, cached path `5.926 ms` (`95.586x` faster than the loop path for repeated calls), and full build, plugin matrix (`Done (32.283s)`), restart/recovery, forced-ticket persistence, and artifact hash verification passed. This is narrow dependency-resolution work reduction, not an end-to-end boot-speed claim.
- Spigot load-order dependency back-reference checks no longer allocate a temporary `HashSet` for each checked provider, and `loadAfter` list construction now pre-sizes from known hard+soft dependency counts. The direct membership check preserves the same result, and the latest synthetic startup-shape benchmark measured `146.978 ms` old load-after build vs `121.139 ms` pre-sized (`1.213x`) plus `2631.046 ms` old back-reference path vs `409.024 ms` direct contains (`6.433x`). Full build, plugin matrix (`Done (28.874s)`), restart/recovery (`Done (19.041s)`), forced-ticket persistence (`14.910s`/`10.688s`), and artifact hash verification passed. This is load-order allocation reduction, not an end-to-end boot-speed claim.
- `TopographicGraphSorter.sortGraph` now pre-sizes its result list, root deque, and fastutil non-root map from known graph node count. A synthetic DAG benchmark measured old default capacity `633.295 ms` vs pre-sized `428.129 ms` (`1.479x`), and full build, plugin matrix (`Done (28.578s)`), restart/recovery (`Done (17.361s)`), forced-ticket persistence (`13.774s`/`9.489s`), artifact hash verification, and a strict 50-bot run passed without crashes. The 50-bot run did not beat the accepted load baseline (`16.93/145.06/2005`), so this remains load-order allocation reduction only.
- `PalettedContainer.reencodeContents(...)` now reuses a per-thread scratch `int[]` for the temporary unpack/remap array. This is not the rejected `ZeroBitStorage` special-case; it preserves the same palette remap loop and only removes repeated temporary array allocation. `scripts/bench_paletted_reencode_scratch.sh` confirmed equivalence and measured old `728.576 ms` vs scratch `244.271 ms` (`2.983x`). A direct packed `SimpleBitStorage` unpack+remap variant was tested and rejected (`858.637 ms`, `0.849x` vs old). Full build, plugin matrix (`Done (28.810s)`), restart/recovery (`Done (18.243s)`), forced-ticket persistence (`14.358s`/`11.217s`), and artifact hash verification passed. Strict 50-bot 32/32 passed without crashes/watchdog/sync-load but failed the accepted baseline (`16.82/154.53/2127` vs `18.27/47.85/2380`), so this remains allocation reduction only.
- `DensityFunction.Visitor` now exposes default holder/marker hooks so generic visitors keep old behavior while `NoiseChunk` and `RandomState` can avoid temporary `HolderHolder`/`Marker` wrappers they immediately unwrap. `scripts/bench_density_visitor_hooks.sh` measured old `481.076 ms` vs hooked `20.770 ms` (`23.162x`) and zero temporary holder/marker allocations in that visitor-unwrapping shape. `applyPatches`, `compileJava`, full build, plugin matrix (`Done (29.184s)`), restart/recovery (`Done (25.338s)`), forced-ticket persistence (`17.691s`/`10.039s`), artifact hash verification, and a noisy 10-bot 32/32 smoke passed. Strict 50-bot was blocked by host preflight (`load_per_cpu=1.089`), so this remains allocation reduction only.
- Plugin loading strategies now pre-size known-size startup maps/lists, and Spigot/Paper/legacy missing-dependency collections are allocated only on actual misses. A synthetic startup-shape benchmark measured old default-capacity setup `371.559 ms` vs new pre-sized setup `233.823 ms` (`1.589x`), validate-no-miss `248.706 ms` vs `232.648 ms` (`1.069x`), and neutral legacy missing-set scan `0.994x`. Full build, plugin matrix (`Done (29.708s)`), restart/recovery (`Done (19.566s)`), forced-ticket persistence (`16.276s`/`10.644s`), and artifact hash verification passed. This is plugin-loading allocation reduction, not an end-to-end boot-speed claim.
- `LegacyPluginLoadingStrategy` now keeps a reverse provided-alias index so legacy plugin load/fail cleanup removes only aliases owned by the provider instead of scanning `pluginsProvided.values().removeIf(...)` for every provider. The focused synthetic benchmark measured old cleanup `503.237 ms` vs reverse-index cleanup `32.279 ms` (`15.590x`), and `rebuildPatches`, full build, plugin matrix (`Done (32.124s)`), restart/recovery (`Done (28.224s)`), forced-ticket persistence (`18.894s`/`11.749s`), and artifact hash verification passed. This is a legacy plugin-loading startup-work reduction, not an end-to-end cold-start or load/TPS claim.
- `TicketStorage.packTickets()` now uses a persistent-ticket counter to avoid copying every regular ticket during save serialization before filtering. Full build, plugin matrix, restart/recovery, and forced chunk restart persistence passed; load-performance impact still needs the next bot gate.
- `WaypointTransmitter` locator-bar hot path now avoids temporary `Vec3` allocation for azimuth updates and caches the last chunk long key for chunk-visibility checks. The later distance/inner-range guard shape is rejected after a fresh focused regression (`0.888x` range, `0.880x` really-far), so it is not part of the current production patch.
- Rejected `Aquifer.NoiseBasedAquifer.computeFluid(...)` surface-sampling block-offset arrays. The standalone bench improved `275.983 ms` to `244.223 ms` (`1.130x`) with equivalence PASS, and build/plugin/restart/forced-ticket gates passed on the temporary artifact, but the strict 50-bot 32/32 rerun failed the accepted baseline (`17.14/82.71/2030` vs `18.27/47.85/2380`). Patch `0041` was removed, `applyPatches` returned to `910 patches`, artifact hash verification passed, plugin matrix passed (`Done (32.234s)`), restart/recovery passed (`Done (20.809s)`), forced-ticket persistence passed (`15.835s`/`11.609s`), and the post-revert noisy 10-bot smoke passed (`19.17/36.29/1572`) without kicks/errors/watchdog/sync-load. Strict post-revert 50-bot rerun is blocked by host preflight (`load_per_cpu=1.041` > `0.750`).
- Load-test host preflight added to protect benchmark integrity: default runs now exit `75` before starting Minecraft if host idle/load thresholds fail. Noisy runs require explicit `LOAD_TEST_ALLOW_BUSY_HOST=true`.
- Region compression default changed to LZ4 and validated in saved region files.
- Noise/density allocation reductions added for scratch arrays and mutable contexts.
- `NoiseChunk`/`RandomState` density wrapper caches moved from Java `IdentityHashMap` to fastutil reference maps at the best measured `2048` expected size.
- `NoiseChunk.forIndex` integer fast-div rewrite was measured and rejected after a noisy control rerun (`tps1_avg=16.73`, `avg_tick_ms_avg=56.58`, `loaded_chunks_max=805`, `watchdog_thread_dumps=1`); the accepted `floorMod`/`floorDiv` path stayed in place.
- `LinearPalette.idFor` reference-map cache was measured and rejected after end-to-end load regression.
- Rejected `ImprovedNoise int[]` permutation-table experiment after measured 50-bot regression.
- Rejected `ImprovedNoise` masking-reduction experiment after measured 50-bot regression.
- Postrevert 50-bot rebaseline after reverting the rejected `0036` experiment landed at `tps1_avg=17.49`, `avg_tick_ms_avg=58.12`, `loaded_chunks_max=2472`.
- `Climate.RTree` / `Climate.Sampler` no-metric fast path was added and measured twice: `tps1_avg=18.32/18.38`, `loaded_chunks_max=2991/2986`, no watchdog/sync-load hits, but average tick time regressed to `91.97/99.65 ms`.
- `NoiseChunk.NoiseInterpolator` delta interpolation was completed in the durable source patch and measured in a 50-bot 32/32 rerun at `tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, with no watchdog/sync-load hits. It is accepted as current-source evidence, but still below the 20 TPS target.
- `ServerGamePacketListenerImpl` moved-too-quickly warning spam was rate-limited in two stages, ending at a shared tick gate that reduced the latest 50-bot warning count from `911` to `1` without changing movement validation.
- `network.optimize-non-flush-packet-sending` / Netty `lazyExecute` was wired behind a default-off Paper config flag and measured against the real plugin matrix, but the on-state regressed the 50-bot load to `tps1_avg=16.31`, `avg_tick_ms_avg=80.82`, so it remains experimental only.
- Rejected `PerlinNoise.wrap` in-range fast path after pinned 50-bot rerun produced `tps1_avg=18.66`, `avg_tick_ms_avg=88.06`, and `watchdog_thread_dumps=1`; it was not promoted over the accepted `18.27/47.85` baseline.
- Rejected `BlockStateData` map pre-sizing after pinned boot/plugin startup regression (`optimized-runtime done_ms=17784`, plugin matrix `Done (34.600s)`).
- Rejected `Climate.RTree.Node` cached `parameter0..parameter6` fields after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=17.39`, `avg_tick_ms_avg=47.48`, `loaded_chunks_max=1236`, and `watchdog_thread_dumps=1`, so the artifact was rebuilt after revert.
- Rejected `CubicSpline.Multipoint.mapAll` stream/iterator cleanup after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=17.45`, `avg_tick_ms_avg=126.93`, `loaded_chunks_max=968`, and `watchdog_thread_dumps=1`, so the patch was deleted and the artifact was rebuilt after revert.
- Rejected `BlendedNoise.compute` power-of-two scale rewrite after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=17.50`, `avg_tick_ms_avg=90.04`, `loaded_chunks_max=2376`, and `watchdog_thread_dumps=1`, so the patch was deleted and the artifact was rebuilt after revert.
- Rejected `DensityFunctions.FindTopSurface` thread-local scratch context after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=17.67`, `avg_tick_ms_avg=59.76`, `loaded_chunks_max=2449`, no watchdog/sync-load hits, but still worse than the accepted `18.27/47.85` baseline.
- Rejected `NoiseChunk.preliminarySurfaceLevel` quart-alignment bit-mask rewrite after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=15.83`, `avg_tick_ms_avg=108.32`, `loaded_chunks_max=2280`, no watchdog/sync-load hits, but it badly regressed the accepted `18.27/47.85` baseline.
- Rejected `PerlinNoise` active-octaves arrays after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=16.76`, `avg_tick_ms_avg=138.50`, `loaded_chunks_max=1126`, and `watchdog_thread_dumps=1`, so it was reverted and the artifact was rebuilt after revert.
- Rejected `NoiseChunk.wrap` fastutil load factor `0.95F` after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=16.85`, `avg_tick_ms_avg=74.43`, `loaded_chunks_max=1020`, and `watchdog_thread_dumps=1`, so it was reverted and the artifact was rebuilt after revert.
- Rejected lazy `NoiseChunk` blend alpha/offset flat caches after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=16.02`, `avg_tick_ms_avg=65.09`, `loaded_chunks_max=562`, and only `online_max=34`, so it was reverted and the artifact was rebuilt after revert.
- Rejected `Climate.Sampler` combined `SampleState` ThreadLocal after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=16.91`, `avg_tick_ms_avg=96.16`, `loaded_chunks_max=1993`, with no watchdog/sync-load hits, so it was reverted and the artifact was rebuilt after revert.
- Rejected config-only `PAPER_CHUNK_IO_THREADS=2` under the pinned 6-CPU load gate; the run produced `tps1_avg=16.96`, `avg_tick_ms_avg=74.18`, `loaded_chunks_max=861`, and `watchdog_thread_dumps=1`, so no default I/O-thread change was made.
- Rejected `ImprovedNoise.gradDot` inline of `SimplexNoise.dot(...)` after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=17.37`, `avg_tick_ms_avg=103.93`, `loaded_chunks_max=2312`, with no watchdog/sync-load hits, so it was reverted and the artifact was rebuilt after revert.
- Rejected `NoiseInterpolator` flat `double[]` slice storage at microbench stage: equivalence passed and modeled array count dropped, but time regressed slightly (`284.036 ms` old vs `286.847 ms` flat), so production was not touched.
- Rejected config-only generational ZGC for the current 50-bot 32/32 gate: it passed host preflight but regressed to `tps1_avg=15.71`, `avg_tick_ms_avg=203.15`, `loaded_chunks_max=1604`, and `watchdog_thread_dumps=2`. No JVM default changed.
- Rejected `Mth.lerp2/lerp3` inline arithmetic after fixing its source-patch hunk metadata, full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=18.02`, `avg_tick_ms_avg=43.93`, `loaded_chunks_max=1625`, with no watchdog/sync-load hits, but it did not beat the accepted `18.27/47.85/2380` baseline because TPS and chunk coverage were lower.
- Rejected `SurfaceRules.SequenceRule` indexed iteration after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=18.79`, `avg_tick_ms_avg=38.68`, `loaded_chunks_max=1216`, with `watchdog_thread_dumps=1`, so it was reverted despite lower average tick time.
- Rejected `PalettedContainer.reencodeContents` `ZeroBitStorage` fast path after full build, plugin matrix, and 50-bot 32/32 gate; the load run produced `tps1_avg=16.32`, `avg_tick_ms_avg=112.44`, `loaded_chunks_max=1430`, with `watchdog_thread_dumps=1` and `sync_load_stack_hits=1`, so it was reverted despite targeting a real save-serialization JFR site.
- Rejected spectator movement no-sync-load path without `PlayerMoveEvent` listeners after full build, plugin matrix, and 50-bot 32/32 gate; it removed `sync_load_stack_hits` and watchdog in that run, but produced only `tps1_avg=17.16`, `avg_tick_ms_avg=50.81`, `loaded_chunks_max=1266`, so it failed the accepted baseline and was reverted.
- Rejected config-only unlimited chunk load/send/gen rates after 50-bot 32/32 gate; it improved `avg_tick_ms_avg` to `42.69` and avoided watchdog/sync-load, but `tps1_avg=17.16` and `loaded_chunks_max=1565` still failed the accepted baseline, so no default changed.
- Rejected `NoiseChunk.FlatCache` reusable context after build, plugin matrix, restart/recovery, forced-ticket persistence, and noisy 50-bot gate; the load run produced `tps1_avg=18.08`, `avg_tick_ms_avg=36.69`, `loaded_chunks_max=385`, and `watchdog_thread_dumps=3` with stack frames in `NoiseChunk$FlatCache.<init>`, so patch `0038` was deleted and the artifact rebuilt.
- Rejected branch-expanded `VarInt.write`/`VarLong.write` after direct Netty `ByteBuf` microbench showed regressions (`VarInt 0.889x`, `VarLong 0.830x`). The temporary source patches were reverted, the artifact was rebuilt on `909 patches`, and plugin/restart/forced-ticket gates passed after the revert.
- Rejected the Aquifer `aquiferLocationAt(...)` direct positional-location split after the clean strict 50-bot 32/32 gate regressed the accepted baseline (`17.65/76.12/2016` vs `18.27/47.85/2380`). The Aquifer call site is back to `positionalRandomFactory.at(...).nextInt(...)`. The later Xoroshiro direct `nextFloatAt(...)` / `nextDoubleAt(...)` follow-up was also rejected after the strict 50-bot gate regressed (`15.45/92.58/1264`), so those helpers were removed from the production path and the runtime was rebuilt.
- `Hashing.sha256(InputStream)` now uses direct `MessageDigest` with a 64 KiB buffer after a real plugin-jar microbench showed a small stream-hash win (`1.004x` in the latest rerun). `Hashing.sha256(Path)` intentionally remains on Guava because direct path hashing regressed to `0.867x`.

## Active Bottlenecks

Measured optimized direct runtime still spends major time in:

- JVM/Minecraft bootstrap and classloading;
- datapack/registry/recipe initialization;
- fresh world spawn generation;
- real plugin initialization/config generation/update checks;
- plugin remapper still has startup overhead around exact jar hashing and precomputed remap installation; the new batch hash reuse reduces the measured hash subpath but is not yet an end-to-end startup win;
- plugin remapper skip-only first-run mapping load is reduced, but precomputed remap installation still copies remapped jars and real plugin initialization remains dominant in the matrix;
- plugin remapper batch list capacity hints are accepted as tiny allocation/copy reduction, but they need clean A/B before any end-to-end startup claim;
- plugin remapper hash collection capacity hints are accepted as tiny allocation/resize reduction, but they need clean A/B before any end-to-end startup claim;
- plugin remapper lazy index cleanup reduces stable cached-start work, but it still needs clean A/B before any end-to-end startup claim;
- plugin remapper dirty index writes reduce stable cached restart/shutdown disk work, but they still need clean A/B before any end-to-end startup/restart claim;
- ReobfServer no longer loads reobf mappings when a precomputed remapped-server jar is available, but this only helps first-run remap paths that lack precomputed plugin remaps and still needs clean A/B before an end-to-end startup claim;
- atomic hard-link install removes same-filesystem bytes copied for precomputed remapped plugin/server jars, but clean A/B is still needed before claiming end-to-end startup impact;
- plugin remapper `InputStream` SHA-256 is now a direct `MessageDigest` path, but file path hashing remains on Guava and a clean startup A/B is still needed before claiming boot-speed impact;
- plugin directory scanning is now flatter and measured faster in isolation, but real plugin startup remains dominated by plugin initialization, config generation, update checks, and classloading;
- Paper plugin metadata dependency-list extraction/cached repeated access is now faster in isolation, but broader plugin startup is still dominated by real plugin initialization and classpath work;
- Spigot load-order back-reference checking now avoids temporary set allocation, but broad startup remains dominated by real plugin initialization and classpath work;
- Spigot load-after construction now avoids a small default-capacity ArrayList growth path, but broad startup remains dominated by real plugin initialization and classpath work;
- Plugin load-order topological sort now avoids default-capacity collection growth, but broad startup remains dominated by real plugin initialization and classpath work;
- plugin loading strategy map/list pre-sizing reduces startup allocation work in isolation, but broad startup remains dominated by classloading and real plugin initialization;
- `PaperReflection` no longer builds a duplicate stripped-method map, reuses one method key during recursive lookup, and skips `StringBuilder` for empty descriptors, but broader plugin reflection/classloading startup costs still need clean A/B evidence before any end-to-end speedup claim;
- Paper plugin library remap is now covered by `LibraryProbe`, but broader third-party plugins with MavenLibraryResolver-heavy classpaths still need real matrix coverage;
- plugin listeners during login, especially LuckPerms/Essentials/ProtocolLib;
- locator-bar/player-waypoint work can become O(players * transmitters) under high player counts; the current azimuth allocation and distance early-out changes are functional-gated but still need a clean load gate before any performance claim;
- fresh chunk noise generation: `ImprovedNoise.p`, `ImprovedNoise.noise`, `PerlinNoise.getValue`, `NoiseChunk.NoiseInterpolator.updateForZ/X/Y`;
- current JFR also shows GC pressure under the G1 load flags (`51` pauses, `5.65s` total pause time, P95 `444ms`), but the first ZGC alternative regressed and the latest fixed-10G G1 retry was blocked by preflight (`load_per_cpu=0.917`);
- biome lookup: `Climate$RTree$SubTree.search(long[], Leaf)` remains visible after the no-metric fast path, but the 2026-05-07 Node field-cache attempt regressed end-to-end load and was reverted;
- `BiomeManager.getBiome(...)` lower-bound early-exit was tested after the latest JFR and rejected at microbench stage (`0.707x`), so the current path stays unchanged;
- chunk generation wrapper map traversal/rehash inside `NoiseChunk.wrap`;
- allocation cleanup inside `CubicSpline.Multipoint.mapAll` is not currently a safe win; the stream-removal candidate regressed the end-to-end gate despite reducing a JFR allocation site;
- allocation cleanup inside `DensityFunctions.FindTopSurface` is not currently a safe win; the thread-local scratch candidate regressed the 50-bot gate despite removing a small allocation source;
- reusing mutable context inside `NoiseChunk.FlatCache` is not currently a safe win; the candidate passed functional gates but hit watchdog in the 50-bot gate;
- simple arithmetic rewrites inside `BlendedNoise.compute` are not currently a safe win; the divide-to-multiply candidate regressed the end-to-end gate and hit watchdog;
- simple alignment rewrites inside `NoiseChunk.preliminarySurfaceLevel` are not currently a safe win; the quart-mask candidate regressed the end-to-end gate badly despite being semantically equivalent;
- compact active-octave arrays inside `PerlinNoise.getValue` are not currently a safe win; the candidate passed plugin matrix but regressed the end-to-end gate and hit watchdog;
- increasing `NoiseChunk.wrap` fastutil load factor is not currently a safe win; the `0.95F` candidate passed plugin matrix but regressed the end-to-end gate and hit watchdog;
- lazily allocating or skipping `NoiseChunk` blend alpha/offset flat caches is not currently a safe win; both the broad lazy-cache candidate and the later empty-blender-only allocation skip reduced obvious allocation sources but regressed the end-to-end gate;
- combining `Climate.Sampler` thread-local state is not currently a safe win; it passed plugin matrix but regressed the end-to-end load gate;
- forcing 2 chunk I/O threads under the pinned 6-CPU load gate is not currently a safe win; it hit watchdog and regressed throughput;
- removing per-player load/send/gen rate limits is not currently enough; it reduced average tick time in one run but did not improve TPS or chunk coverage enough;
- inlining `ImprovedNoise.gradDot` into direct `GRADIENT` array arithmetic is not currently a safe win; it passed plugin matrix but regressed the end-to-end load gate;
- inlining `Mth.lerp2/lerp3` arithmetic is not currently a proven safe win; it reduced average tick time in one lower-coverage run but failed TPS and loaded-chunk acceptance against the current baseline;
- indexed iteration in `SurfaceRules.SequenceRule` is not currently a safe win; it removed a visible iterator allocation candidate but hit watchdog and lowered chunk coverage in the end-to-end gate;
- special-casing `PalettedContainer.reencodeContents` for `ZeroBitStorage` is not currently a safe win; it removed an obvious unpack/fill path but regressed the save/load gate and hit watchdog plus sync-load;
- skipping CraftBukkit's forced chunk load in spectator movement when no `PlayerMoveEvent` listeners are registered is not currently a safe performance win; it fixed one sync-load symptom but lowered TPS and loaded-chunk coverage;
- main-thread sync chunk load when fast spectator movement reaches chunks not ready for entity movement commit.
- branch-expanded VarInt/VarLong writes are not currently a safe network optimization on this CPU; the direct write microbench regressed and the candidate was reverted.
- Current completed-delta 50-bot run improved average tick time versus the shared-warning-gate rerun, but fresh noise generation remains the dominant CPU target from the latest JFR evidence; biome lookup is improved enough to stop blind edits there until a narrower profile proves another safe change.

These cannot be deleted or blindly parallelized without breaking compatibility.

## External Kernel Notes

- Pulse-MC/Pulse was cloned at `bench/upstream-research/Pulse` (`675d816`). Its main portable idea is bounded packet batching with critical/instant packet bypass, but its broader virtual block/entity and packet-interception model is not safe as a default Paper compatibility change.
- BX-Team/DivineMC was cloned at `bench/upstream-research/DivineMC` (`7619684`). It contains aggressive regionized chunk ticking, async tracker, async mob-spawn and async join experiments; these are useful references, but mutable world/entity/plugin-visible ordering makes them high-risk until isolated behind defaults-off experiments and differential/plugin tests.

## Next Safe Optimization Candidates

- DirectoryStream plugin-directory scan is now accepted as a narrow startup-work reduction (`1.160x` over `Files.list` in the matrix microbench) with build/plugin/restart/forced-ticket/hash evidence. Do not spend more cycles on plugin directory enumeration unless a fresh startup profile shows it again; the current strict 50-bot gate still points at worldgen/load pressure.
- Rerun the 50-bot noisy diagnostic after persistent-ticket save packing and check whether the previous `save-all` watchdog stack disappears.
- Rerun a clean 50-bot 32/32 gate for the waypoint azimuth/distance candidate when preflight passes; compare against `18.27/47.85/2380` before promoting or reverting.
- Run a fresh JFR on the completed delta-cache build before choosing the next worldgen hot path.
- Add a targeted measurement for final `NoiseChunk.wrap` map size before changing fastutil capacity again; `4096` was measured and rejected.
- Investigate a new noise-generation optimization that avoids the already-rejected `ImprovedNoise int[]` and duplicated-tail experiments.
- Avoid repeating the rejected `BlendedNoise.compute` divide-to-multiply rewrite unless a future JFR isolates a different cause and the gate is rerun.
- Avoid repeating the rejected `FindTopSurface` scratch-context rewrite unless a future JFR shows that path is dominant and a fresh gate beats `18.27/47.85`.
- Avoid repeating the rejected `NoiseChunk.preliminarySurfaceLevel` quart-mask rewrite unless a future JFR shows a different bottleneck shape and a fresh gate beats `18.27/47.85`.
- Avoid repeating the rejected `PerlinNoise` active-octaves arrays rewrite unless a future JFR shows a different bottleneck shape and a fresh gate beats `18.27/47.85`.
- Avoid repeating `NoiseChunk.wrap` fastutil capacity/load-factor tuning (`4096`, `0.95F`) unless a future targeted measurement shows a different size distribution and a fresh gate beats `18.27/47.85`.
- Avoid repeating lazy `NoiseChunk` blend-cache allocation unless a future profile shows it alongside a clean online count and a fresh gate beats `18.27/47.85`.
- Avoid repeating `Climate.Sampler` combined ThreadLocal state unless a future profile proves the `ThreadLocal.get()` cost is dominant and a fresh gate beats `18.27/47.85`.
- Avoid changing chunk I/O thread defaults based only on the pinned 6-CPU run; `PAPER_CHUNK_IO_THREADS=2` regressed and hit watchdog in the current harness.
- Avoid changing chunk load/send/gen rate defaults based only on the unlimited-rates run; it failed TPS and loaded-chunk acceptance.
- Avoid repeating `ImprovedNoise.gradDot` / `SimplexNoise.dot` inlining unless a future JFR proves a different compilation shape and a fresh gate beats `18.27/47.85`.
- Avoid repeating `NoiseInterpolator` flat-slice storage unless a future profile shows allocation/GC dominates more than slice access time and a fresh microbench beats the current jagged representation.
- Avoid repeating `Mth.lerp2/lerp3` inlining unless a future JFR plus gate shows the same or higher chunk coverage and beats `18.27/47.85`.
- Avoid repeating `SurfaceRules.SequenceRule` foreach-to-index unless a future profile explains the watchdog/coverage regression and a fresh gate beats `18.27/47.85/2380`.
- Avoid repeating the `PalettedContainer.reencodeContents` `ZeroBitStorage` branch unless a future profile isolates save serialization without simultaneous chunk-generation pressure and a fresh gate beats `18.27/47.85/2380`.
- Avoid repeating `NoiseChunk.FlatCache` context reuse unless a future profile explains the watchdog stack and a fresh gate beats `18.27/47.85/2380`.
- Avoid repeating the LZ4 no-outer-buffer region stream wrapper unless a future profile changes the hypothesis; the stream microbench improved `1.133x`, but the real 50-bot gate regressed to `18.53/80.71/2085`.
- Avoid repeating spectator movement no-sync-load bypass unless paired with a chunk readiness/prefetch strategy that improves chunk coverage and preserves `PlayerMoveEvent` behavior.
- Avoid repeating branch-expanded VarInt/VarLong writes unless a future JIT/profile proves a different implementation shape and a direct Netty `ByteBuf` microbench beats the current Paper path.
- Avoid switching high-load defaults to generational ZGC based on theory; the measured 50-bot run regressed and hit watchdog dumps.
- Avoid repeating `PerlinNoise.getValue` exact-class guarded direct-local dispatch unless a future benchmark beats the current delegating path while preserving subclass semantics.
- Avoid repeating C2ME/DivineMC arithmetic `ImprovedNoise.sampleAndLerp` unless a future JIT profile changes; the current bit-exact arithmetic shape is slower than the flat-gradient implementation.
- Investigate async preparation for movement into not-yet-ready chunks, with deterministic commit and no plugin event reorder.
- The moved-too-quickly warning/log overhead is mostly addressed; keep it bounded, but the next real gains should come from chunk generation, worldgen wrappers, and load/scheduler behavior rather than more logging work.
- Warm-world benchmark exists; use it to separate JVM/plugin startup from
  fresh chunk generation on the saved-world path, and only promote any result
  after a clean idle-host A/B run.
- Add differential worlds for redstone/liquids/entities/inventories/datapacks.
- Profile plugin startup under cached config vs first-run config generation.
- Measure plugin startup with generated plugin configs already present; the latest matrix is dominated by real plugin initialization/config/update work, not remap jar copy time.
- Rerun clean startup A/B for plugin-remapper hash reuse when `/var/lib/pufferpanel/servers/6805cd25` is not consuming all CPUs; do not promote it beyond a hash-path optimization until then.
- Rerun clean startup A/B for the accumulated plugin load-order/allocation changes on an idle host; current evidence is microbench plus compatibility gates, not an end-to-end boot-speed claim.
- Rerun clean startup A/B for precomputed plugin/library skip cache, batch-miss hash reuse, and deferred mappings load on an idle host; keep them framed as exact-SHA/plugin-remapper work reduction until a clean run shows real startup improvement.
- Rerun clean startup A/B for ReobfServer precomputed-server-before-mappings on an idle host with no precomputed plugin remaps; the targeted proof only shows work avoided (`loading_reobf_mappings_count=0`), not end-to-end startup speed.
- Rerun clean startup A/B for atomic hard-link install on an idle host; current evidence proves same-inode installs, not total startup speedup.
- Rerun the current post-plugin-name-log 50-bot 32/32 gate only when host preflight clears; the latest attempt refused before Minecraft start at `load_per_cpu=0.812` > `0.750`, with live Java servers and Velocity consuming CPU.
- Rerun the planned 50-bot 32/32 concurrent chunk load/gen gate only when the host is clean; the latest 2026-05-07 preflight showed `load1=10.14`, `load_per_cpu=0.845`, `idle_percent_1s=37.74`, with live Java and Velocity processes still outside the strict `0.750` load-per-CPU / `40.00%` idle thresholds.
- Current waypoint/remapper candidate preflight is also blocked by host load; do not run a comparable 50/500-bot verdict until this clears.
- Keep the preflight thresholds strict enough for comparable runs; adjust only with documented evidence if a dedicated benchmark host needs different limits.
- Evaluate a Pulse-style packet batching candidate only behind measurement: critical/instant packet bypass, bounded flush limits, and block-update coalescing are portable ideas; virtual block/entity behavior and broad packet interception are not default-safe for Paper compatibility.
- Consider native/Rust NBT or region serialization only after profiling proves it beats Java without changing format/ordering.
- Avoid `ImprovedNoise byte[] -> int[]` unless a future profile proves a different implementation; the direct experiment regressed load.
- Avoid `ImprovedNoise` duplicated-tail masking rewrite unless a future profile proves it; the direct experiment regressed load.
- Avoid repeating `ImprovedNoise.sampleAndLerp` local `byte[]` inline access unless a future profile shows a different JIT shape and a fresh server gate beats the accepted baseline; the standalone loop improved, but the real 50-bot gate did not.
- Avoid repeating `ImprovedNoise.sampleWithDerivative` flat-gradient/direct-dot unless a future profile proves a different compilation shape; the standalone derivative bench improved `1.061x`, but the strict 50-bot gate regressed to `15.36/94.24/3850` with 2 watchdog dumps.
- Avoid repeating the `SurfaceRules.SequenceRule` runtime-array / indexed-loop candidate unless a future profile explains the regression; the latest strict gate failed the accepted baseline (`15.95/117.42/1785` vs `18.27/47.85/2380`) and the patch was reverted.
- Avoid repeating the `RangeChoiceConstantOut` specialization unless a future
  profile explains the clean-gate regression; the strict 50-bot run failed at
  `17.63/192.39/2768` with 5 watchdog dumps and the patch was removed.
- Avoid repeating the `OreFeature.doPlace(...)` scalar-hoist cleanup unless a
  future profile explains the clean-gate watchdog/MSPT regression; the strict
  50-bot run failed the accepted runtime standard at `18.27/65.21/2911` with
  2 watchdog dumps and the source patch was removed.
- Avoid repeating `Climate.Parameter.distance(...)` explicit-branch rewrites
  unless a future profile/JIT shape changes; the latest standalone benchmark
  regressed (`branch_distance_speedup=0.961x`) or was neutral
  (`subtract_first_speedup=0.996x`).
- Avoid increasing `NbtIo.writeCompressed(...)` GZIP/pre-GZIP buffers based on the latest real `level.dat` microbench: all tested 64 KiB buffer variants were byte-identical but slower (`0.849x`, `0.840x`, `0.788x`) than the current default-buffer chain.
- Avoid changing `CompoundTag.loadCompound(...)` fastutil map initial capacity
  away from `8` based only on allocation pressure; real `.mca` parsing kept
  `cap8` fastest (`1907.510 ms`) versus `cap4` (`1922.989 ms`) and `cap16`
  (`1957.953 ms`).
- Keep the new native Rust `NoiseChunk` wrap-capacity and `Deflater`
  input-shape modules diagnostic-only until a strict gate or a stronger runtime
  profile proves they belong in Paper; the current evidence is parity and
  microbench speedup only.

## Do Not Do

- Do not claim `<1s` until measured.
- Do not claim vanilla parity until differential tests exist.
- Do not claim all plugins supported.
- Do not parallelize mutable world state, scheduler/event order, service manager, permissions or entity mutation paths.
