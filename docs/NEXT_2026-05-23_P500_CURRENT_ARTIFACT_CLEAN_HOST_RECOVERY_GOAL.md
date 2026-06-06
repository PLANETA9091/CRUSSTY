# NEXT 2026-05-23: P500 Current Artifact Clean-Host Recovery Goal

This is the next blocking goal before any higher-tier or near-unbounded claim.

The target is narrow on purpose: restore the measured production-ready claim
for `500 bots / 32 view / 32 simulation / creative block` on the current
artifact only. Higher tiers, mixed gameplay, mobs, chunk ladders, and broad
near-unbounded wording are blocked until this file is green.

## Current Artifact

- [x] Optimized Paper artifact:
  `131dd4c02b15c8f0d3fedef57043535ede7df8ddeb1ed24f14986588ec5510c4`
- [x] Optimized runtime launcher:
  `108e51a63a97739964438c2dcba169e3d66889d454b0f7e049beee4614568f6c`
- [x] Native runtime library:
  `270639cc1ecdb642b6944d84675679a349702fdaa44b6723cd5a78e387e632fd`
- [x] AppCDS archive:
  `ffc24e84c1df646b55482a458c4013d7dd850fc7ec249057d9db8b8d8fe9a049`
- [x] Runtime jar SHA256 file:
  `5d943e7e164c615bf01f4eec38b03f3d2a2369e91b2bfef71e1a14325a44d41a`
- [x] Remap classpath jar:
  `3f0a698373188db309a2e987d0e96871a44d45e8fc6720ec02447a6461bbed48`
  under remap classpath id
  `E7E9833114B557088D8FBBF856CA9178259F4032675A6A3E3465D14E4AF4AD23`
- [x] `sha256sum -c reports/artifact-hashes.txt` passes for the live tree.

## Current Red Facts

- [x] `reports/production-500-readiness-gate-retry-20260523-095930.txt`
  exhausted `3` retries / `4` total attempts; all failed as
  environment-invalid host contention before a claim window. This is now
  historical evidence because the native runtime hash changed afterward.
- [x] `reports/host-synthetic-canary-live-20260523-fresh.txt` is the latest
  cheap host canary evidence and is red: `host_synthetic_canary_ok=false`,
  `steal_percent_max=24.53`.
- [x] `reports/load-host-preflight-live-20260523-143031-preflight.txt` is also
  red: `strict_foreign_process_gate_pass=false` because a `server.jar` process
  is running.
- [x] `reports/production-500-soak-gate.txt` was generated at
  `2026-05-23T08:16:16.765388+00:00` and is red:
  `soak_gate_pass=false`, `failure_count=6`.
- [x] Latest cold soak summary is current-artifact-bound, but
  environment-invalid from host contention:
  `host_cpu_steal_percent_max=50.71`,
  `host_system_load1_per_cpu_max=1.545`,
  `host_cpu_iowait_percent_max=0.83`, and `environment_invalid=true`.
- [x] Latest cold soak reached only `8` ready bots / `7` active bots and no
  full load window: `load_window_reached_full_online=false`.
- [x] Latest cold soak had `0` block place packets and `0` block dig packets.
- [x] Warm soak was skipped because cold failed.
- [x] `reports/production-500-readiness-bundle-current` is historical/stale:
  it contains old `d4b27...` claim evidence, only `8` evidence files, missing
  raw logs, and missing current runtime/native proof.
- [x] `reports/production-500-claim-verdict.txt` has
  `claim_assertion_pass=false` and `bundle_validation_pass=false`.
- [x] Fresh same-artifact plugin matrix passed after the latest rebuild:
  `reports/plugin-matrix-summary.txt`.
- [x] Fresh same-artifact restart/recovery passed after the latest rebuild:
  `reports/restart-recovery-summary.txt`.
- [x] Fresh same-artifact forced-ticket persistence passed after the latest
  rebuild: `reports/forced-ticket-persistence-summary.txt`.

## Harness Hardening Done

- [x] Production claim profiles require `LOAD_TEST_HOST_SYNTHETIC_CANARY=true`.
- [x] `run_load_test.sh` requires a stable host-ready window, then executes
  the synthetic canary before staging, server startup, and launcher
  execution.
- [x] Host-ready prelaunch aborts now write the same summary/gate evidence
  shape as synthetic-canary aborts.
- [x] Synthetic canary failures write `bot_exit=75` and
  `early_abort_reason=host_contention_prelaunch_canary...`.
- [x] `evaluate_load_gate.py` classifies that reason as
  `environment_invalid_kind=host_contention`.
- [x] Sharded bot ramps now pass `--ramp-shard-index` and
  `--ramp-shard-count` into `mc_bot_swarm`.
- [x] Dedicated smoke coverage exists in
  `scripts/run_load_test_host_synthetic_canary_smoke.sh`.
- [x] The full readiness gate also runs host-ready stable-window and sharding
  default smoke coverage before exporting or publishing any claim bundle.
- [x] Readiness and release-current-binding smokes use current fixture data
  with strict all-ready action-gate fields, so they no longer fail because the
  live current-artifact reports are intentionally red.
- [x] Dry run `reports/production-500-readiness-run-20260523-120956.txt`
  proved the smoke strip passes before the gate rejects the real red soak and
  old-artifact repeat evidence.
- [x] The shared production-ready claim fixture now validates through bundle,
  claim, claim-only, and publication smoke paths without duplicate summary
  keys.

## Allowed Claim After Completion

Only this exact scope is allowed after all checks pass:

> Production-ready for the measured 500-bot, 32 view-distance,
> 32 simulation-distance, creative block workload profile on the verified
> current optimized artifact.

Non-claims that must remain explicit:

- [ ] not a full Paper runtime rewrite to Rust
- [ ] not unlimited plugin compatibility
- [ ] not proof for unmeasured real-player gameplay
- [ ] not a multi-hour soak claim
- [ ] not literal unlimited players, mobs, chunks, ticks, plugins, or datapacks

## Required Evidence Bundle

The final bundle must contain:

- [ ] `bundle.json`
- [ ] `MANIFEST.txt`
- [ ] `CLAIM.md`
- [ ] `production-500-readiness-gate.txt`
- [ ] `production-500-soak-gate.txt`
- [ ] `production-500-repeat-quorum.txt`
- [ ] `plugin-matrix-summary.txt`
- [ ] `plugin-matrix.log`
- [ ] `restart-recovery-summary.txt`
- [ ] `restart-recovery.log`
- [ ] `forced-ticket-persistence-summary.txt`
- [ ] `forced-ticket-persistence-first.log`
- [ ] `forced-ticket-persistence-restart.log`
- [ ] `artifact-hashes.txt`
- [ ] `artifacts.json`
- [ ] native proof: `libpaper_native_jni.so.sha256` or
  `paper-native-jni.sha256`
- [ ] referenced cold/warm summaries, preflight files, resources CSVs, logs,
  status JSONs, and hash proof files

## Required Pass State

- [ ] `production_ready_500_claim=true`
- [ ] `readiness_gate_pass=true`
- [ ] `failure_count=0`
- [ ] `soak_gate_pass=true`
- [ ] `repeat_quorum_pass=true`
- [ ] `plugin_matrix_pass=true`
- [ ] `restart_recovery_pass=true`
- [ ] `forced_ticket_persistence_pass=true`
- [ ] `artifact_hashes_pass=true`
- [ ] `current_artifact_consistency_pass=true`
- [ ] `repeat_passes >= 3`

## Cold And Warm Gate Requirements

Both cold and warm surfaces must pass on the same artifact:

- [ ] `bots=500`
- [ ] `view_distance=32`
- [ ] `simulation_distance=32`
- [ ] `load_test_scenario=block`
- [ ] `load_test_gamemode=creative`
- [ ] full online reached
- [ ] `load_window_metrics_samples >= 300`
- [ ] `loaded_chunks_max >= 4000`
- [ ] `tps1_avg >= 19.50`
- [ ] `tps1_min >= 18.00`
- [ ] `avg_tick_ms_avg <= 50.00`
- [ ] `avg_tick_ms_max <= 100.00`
- [ ] `bot_block_place_packets_max >= 120000`
- [ ] `bot_block_dig_packets_max >= 120000`
- [ ] `bot_block_action_errors_max=0`
- [ ] `watchdog_thread_dumps=0`
- [ ] `sync_load_stack_hits=0`
- [ ] `stability_failures=0`
- [ ] no bot kicks/errors in the claim window

## Clean-Host Gate

The production run is invalid if the host is noisy:

- [ ] `environment_invalid=false`
- [ ] `host_cpu_steal_percent_max <= 10.00`
- [ ] `host_cpu_iowait_percent_max <= 10.00`
- [ ] `host_system_load1_per_cpu_max <= 0.750`
- [ ] host-ready preflight is stable before launch
- [ ] in-run host watcher does not abort the run

## Action-Gate Requirements

- [ ] Production block workload starts through `all-ready`, not timer mode.
- [ ] Gate opens only after `500` ready bots.
- [ ] Gate opens only after `500` active bots.
- [ ] Gate opens only after `500` settled bots.
- [ ] Gate opens only after `500` block-armed bots.
- [ ] Settle window is at least `15000 ms`.
- [ ] Required ready min-count equals the bot count.

## Validation Commands

After fresh evidence exists:

```bash
python3 scripts/validate_production_readiness_bundle.py reports/production-500-readiness-bundle-current --require-current-freshness
python3 scripts/assert_production_ready_claim.py reports/production-500-readiness-bundle-current --report reports/production-500-claim-verdict.txt
```

For fresh evidence, only when the host is clean enough:

```bash
MC_EULA_AGREE=true \
PRODUCTION_READINESS_REFRESH_SOAK=true \
PRODUCTION_READINESS_REFRESH_REPEAT=true \
PRODUCTION_READINESS_REFRESH_COMPAT=true \
./scripts/run_production_readiness_gate.sh
```

## If The Clean-Host Run Still Fails

- [ ] Classify the limiter from the failed run: host, join, packet, entity
  tracking, block events, chunk generation, chunk send, lighting, IO, memory,
  scheduler, plugin hook, or native hook.
- [ ] Patch only the hottest core path supported by evidence.
- [ ] Do not change, update, or optimize plugins/datapacks to make the gate
  pass.
- [ ] Do not accept Rust/JNI parity or microbench wins without a same-artifact
  server gate.
- [ ] Do not publish a claim until the bundle validator and claim assertion
  both pass.
