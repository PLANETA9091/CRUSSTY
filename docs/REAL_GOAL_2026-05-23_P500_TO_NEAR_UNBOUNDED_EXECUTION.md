# REAL GOAL 2026-05-23: P500 current claim, then measured near-unbounded core scale

This is the execution goal for `/root/rust` after the latest current-artifact
P500 attempts.

Literal unlimited is not a valid engineering claim. The real goal is harder
and more useful: remove core bottlenecks until accepted player, mob, chunk,
worldgen, plugin, network, and recovery tiers are limited by measured hardware
or policy ceilings, not by an unexamined Paper/runtime hot path.

Plugins and datapacks are stress inputs. Do not update, remove, patch, or
optimize them to make a gate pass. The optimization surface is the core:
Paper hot paths, Rust/JNI native modules, chunk/entity/network scheduling,
backpressure, IO, and validation tooling.

## Current Execution Rules

- [x] Acceleration mode is mandatory after 2026-05-23T14:08:10Z:
  run independent checks in parallel, keep the main agent on the critical
  path, and use subagents only for sidecar diagnosis or disjoint patches.
- [x] The live goal is not "unlimited". The goal is to push measured accepted
  tiers upward until the limit is a documented hardware/policy ceiling or a
  specific core bottleneck with evidence.
- [x] Keep the active target unchanged: restore the current-artifact P500
  production-ready claim before making any wider scale statement.
- [x] Main agent owns the critical path locally: source/bytecode proof,
  artifact rebuild/update, host preflight, gate launch, bundle export, and
  final validators.
- [x] Subagents are sidecar workers only. Use them for independent read-only
  diagnosis or disjoint patches while the main path keeps moving; never wait
  on them when the next local action is already known.
- [x] Every continuation must either produce a stronger artifact, a stronger
  report, a stronger gate result, or a narrower verified blocker.
- [x] Do not stop on microbench, source inspection, or stale reports if the
  next gate/action is runnable.
- [x] If the host is contaminated, do not waste a production attempt. Produce a
  fresh blocker report, keep cheap verifications green, and move to
  optimization-frontier work that does not depend on a clean host.
- [x] Do not optimize plugins or datapacks. If they are heavy, they remain
  part of the measured compatibility surface.
- [x] Do not launch the full P500 gate on a contaminated host. First prove the
  synthetic canary and strict foreign-process preflight are green, or prove
  exactly which external process/resource blocks the run.
- [x] Do not publish or say the P500 claim until
  `validate_production_readiness_bundle.py` and
  `assert_production_ready_claim.py` pass against
  `reports/production-500-readiness-bundle-current`.

## Claim Ladder

No claim moves to the next rung until the previous rung has a fresh evidence
bundle on the same artifact.

- [ ] Rung 0: current artifact P500 claim restored.
- [ ] Rung 1: P500 mixed-gameplay full stress corpus passes.
- [ ] Rung 2: P750/P1000 mixed-gameplay tiers pass if the host allows.
- [ ] Rung 3: mob tiers M10k/M25k/M50k pass with bounded AI/pathfinding.
- [ ] Rung 4: chunk/worldgen tiers C10k/C25k/C50k pass with datapack hooks.
- [ ] Rung 5: combined player+mob+chunk+plugin tiers pass.
- [ ] Rung 6: long soak, restart/recovery, disk pressure, and reconnect storm
  gates pass for the leading accepted tier.
- [ ] Rung 7: every accepted tier has a timestamped self-contained evidence
  bundle and a stable current publication file.

## Current Artifact Floor

- [x] Optimized Paper artifact:
  `131dd4c02b15c8f0d3fedef57043535ede7df8ddeb1ed24f14986588ec5510c4`.
- [x] Optimized runtime launcher:
  `108e51a63a97739964438c2dcba169e3d66889d454b0f7e049beee4614568f6c`.
- [x] Native runtime library:
  `270639cc1ecdb642b6944d84675679a349702fdaa44b6723cd5a78e387e632fd`.
- [x] AppCDS archive:
  `ffc24e84c1df646b55482a458c4013d7dd850fc7ec249057d9db8b8d8fe9a049`.
- [x] Runtime jar SHA256 file:
  `5d943e7e164c615bf01f4eec38b03f3d2a2369e91b2bfef71e1a14325a44d41a`.
- [x] Remap classpath jar:
  `3f0a698373188db309a2e987d0e96871a44d45e8fc6720ec02447a6461bbed48`
  under remap classpath id
  `E7E9833114B557088D8FBBF856CA9178259F4032675A6A3E3465D14E4AF4AD23`.
- [x] Native NormalNoise direct JNI output is shipped in the current native
  runtime library.
- [x] Current artifact manifests verified after the latest rebuild:
  `sha256sum -c reports/artifact-hashes.txt`,
  `sha256sum -c reports/paper-native-jni.sha256`,
  `sha256sum -c artifacts/optimized-runtime/native/libpaper_native_jni.so.sha256`,
  and `JAVA_BIN=/bin/true artifacts/optimized-runtime/run.sh --nogui`.
- [x] Previous P500 retry evidence is historical after the current native
  rebuild; it is not claim evidence for this exact artifact.
- [x] Latest host checks are not claim evidence because they abort on host
  contention or foreign `server.jar` process state before the load window.
- [x] Current bundle is protected from stale publication: `*-current` bundle
  validation requires live freshness even with `--allow-stale-freshness`.
- [ ] Current artifact has a fresh green P500 cold+warm gate.
- [ ] Current artifact has fresh repeat quorum.
- [x] Current artifact has fresh plugin matrix evidence.
- [x] Current artifact has fresh restart/recovery evidence.
- [x] Current artifact has fresh forced-ticket persistence evidence.
- [ ] Current artifact has a regenerated and validated
  `reports/production-500-readiness-bundle-current`.

## Live Blocker 2026-05-23T15:58Z

- [x] Fresh synthetic canary is red:
  `reports/host-synthetic-canary-live-20260523-fresh.txt` reports
  `host_synthetic_canary_ok=false`,
  `host_synthetic_canary_steal_percent_max=24.53`, limit `10.00`.
- [x] A foreign live server blocks a production claim run: PID `2871654`,
  cwd `/var/lib/pufferpanel/servers/6a11c76a`, command includes
  `java21 ... -jar server.jar nogui`.
- [x] The current `reports/production-500-readiness-bundle-current` is stale:
  it points at older artifact hashes, lacks required raw logs, and fails both
  current-bundle validation and claim assertion.
- [x] `reports/production-500-readiness-gate.txt` now separates the current
  state clearly: plugin matrix, restart/recovery, forced-ticket persistence,
  artifact hash checks are green, while cold/warm soak and same-artifact repeat
  quorum are not current claim evidence.
- [ ] Full P500 gate is blocked until the same host passes synthetic canary,
  strict foreign-process gate, and stable host-ready preflight.

## Acceleration Work Queue

- [ ] Critical path: wait for or create a clean host window, then run the full
  production readiness retry with cold+warm, repeat quorum, plugin matrix,
  restart/recovery, forced-ticket persistence, and current bundle rebuild.
- [ ] Validation path: after the retry, run
  `validate_production_readiness_bundle.py --require-current-freshness` and
  `assert_production_ready_claim.py` against
  `reports/production-500-readiness-bundle-current`.
- [ ] Failure path: if the gate fails on server behavior instead of host
  contamination, immediately analyze the newest summary/gate/log/resources and
  patch only the proven core hot path.
- [ ] Sidecar path: keep subagents on gate-contract audit, host audit, and
  optimization-frontier discovery while the main path runs checks or patches.

## Patch Queue Status

- [x] `0156` was neutralized back to the conservative sent-chunk path.
- [x] `0159` reuses a single `ChunkPos` in `sendUnloadChunkRaw`.
- [x] `0160` removes the unused `manhattanDistance` local in the player
  chunk-loader rebuild loop.
- [x] `0161` is a narrow AP2 MIN/MAX partial fallback candidate limited to
  `Noise` / `ShiftedNoise`; compile and focused equivalence benches passed.
- [x] `0162` fixes player chunk-loader send-budget draining on a blocked head:
  budget is previewed, charged only for an actual dequeue, and committed once
  in `finally`.
- [x] The `ChunkEntitySlices` split-loop experiment was rejected by benchmark
  and reverted; do not treat it as an accepted optimization.
- [x] Patch application and `paper-server:compileJava` both passed after these
  changes.
- [ ] `0161` is not a production-speed claim until the optimized jar is rebuilt
  and a same-artifact server gate moves.
- [ ] These edits do not change the fact that the current bundle is stale/red.

## Immediate Blocking Gate

This gate must be green before any higher-scale claim:

- [ ] Host preflight stable before launch.
- [ ] In-run host watcher does not mark `environment_invalid=true`.
- [ ] `500/500` bots reach ready, active, settled, and block-armed states.
- [ ] Cold and warm load windows both reach full online.
- [ ] `load_window_metrics_samples >= 300`.
- [ ] `loaded_chunks_max >= 4000`.
- [ ] `tps1_avg >= 19.50`.
- [ ] `tps1_min >= 18.00`.
- [ ] `avg_tick_ms_avg <= 50.00`.
- [ ] `avg_tick_ms_max <= 100.00`.
- [ ] `bot_block_place_packets_max >= 120000`.
- [ ] `bot_block_dig_packets_max >= 120000`.
- [ ] `bot_block_action_errors_max=0`.
- [ ] `watchdog_thread_dumps=0`.
- [ ] `sync_load_stack_hits=0`.
- [ ] `stability_failures=0`.
- [ ] No kicked bots or protocol errors in the claim window.

## Core Optimization Frontier

Patch only paths backed by profiler, trace, benchmark, or gate evidence.

- [ ] Density wrapper/mapAll pipeline in `NoiseChunk`, `DensityFunctions`,
  `DensityFunction`, `NoiseRouter`, and `RandomState`.
- [ ] Verify or remove the `Ap2 MIN/MAX` batch candidate contradiction before
  treating it as production evidence.
- [ ] Chunk generation orchestration and queue backpressure.
- [ ] Chunk send packet budgets and slow-client backpressure.
- [ ] Entity tracker scaling and packet fanout limits.
- [ ] Mob AI/pathfinding budget scheduler.
- [ ] Collision and nearby-entity lookup scaling.
- [ ] Region IO writeback and save throttling.
- [ ] Restart/recovery under forced tickets and high chunk load.
- [ ] Rust/JNI hook expansion only where the Java hot path is proven hot.

## Evidence Contract

- [x] No production claim from stale bundle.
- [x] No production claim from microbench alone.
- [x] No broad claim from synthetic bots without exact profile wording.
- [x] No plugin/datapack optimization to pass the core gate.
- [ ] Every accepted tier has `bundle.json`, `MANIFEST.txt`, `CLAIM.md`,
  copied evidence, raw logs, resource CSVs, artifact hashes, and native proof.
- [ ] `validate_production_readiness_bundle.py` passes for the accepted bundle.
- [ ] `assert_production_ready_claim.py` passes for the accepted bundle.
- [ ] Publication text includes exact non-claims.

## Exact Non-Claims

Even after P500 is restored, do not claim:

- [ ] full Rust Paper runtime
- [ ] literal infinite players
- [ ] literal infinite mobs
- [ ] literal infinite chunks or ticks
- [ ] unlimited plugin compatibility
- [ ] unlimited datapack compatibility
- [ ] real-player gameplay parity without real-client evidence
- [ ] multi-hour soak unless that exact soak was measured

## Next Commands

Run only cheap checks while the host is noisy:

```bash
python3 -m py_compile scripts/validate_production_readiness_bundle.py scripts/assert_production_ready_claim.py
bash scripts/validate_production_readiness_bundle_smoke.sh
python3 scripts/validate_production_readiness_bundle.py reports/production-500-readiness-bundle-current --allow-stale-freshness --reports-dir reports
```

Run the production gate only after a clean host preflight:

```bash
MC_EULA_AGREE=true \
PRODUCTION_READINESS_REFRESH_SOAK=true \
PRODUCTION_READINESS_REFRESH_REPEAT=true \
PRODUCTION_READINESS_REFRESH_COMPAT=true \
./scripts/run_production_readiness_gate_retry.sh
```

## Done

The goal is done only when the current artifact has a fresh green P500 claim
again, then the next accepted measured scale tier has its own fresh bundle,
and every wider statement is tied to evidence instead of hope.
