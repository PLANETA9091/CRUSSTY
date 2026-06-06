# Testing

Дата: 2026-05-17 CEST

## Historical 2026-05-17 CEST Production-Ready 500 Certification Gate

The historical top-level gate for the measured `500 bots / production ready`
claim was:

```bash
MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh
scripts/evaluate_production_readiness_smoke.sh
scripts/export_production_readiness_bundle_smoke.sh
scripts/validate_production_readiness_bundle_smoke.sh
scripts/assert_production_ready_claim_smoke.sh
scripts/production_ready_claim_smoke.sh
scripts/publish_production_ready_claim_smoke.sh
```

Fresh outcome:

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
artifact_hash_count=12
repeat_passes=3
evaluate_production_readiness_smoke=PASS
export_production_readiness_bundle_smoke=PASS
validate_production_readiness_bundle_smoke=PASS
assert_production_ready_claim_smoke=PASS
production_ready_claim_smoke=PASS
publish_production_ready_claim_smoke=PASS
bundle_validation_pass=true
claim_assertion_pass=true
evidence_file_count=8
claim_publication_pass=true
```

This certification gate requires all lower layers to pass: the cold+warm soak
gate, the repeat quorum, fresh plugin matrix, fresh restart/recovery, fresh
forced-ticket persistence, and the artifact hash manifest.
It also exports `reports/production-500-readiness-bundle-20260517-091520`
with `CLAIM.md`, `MANIFEST.txt`, `bundle.json`, and copied evidence files.
The exported bundle is independently checked by
`scripts/validate_production_readiness_bundle.py`, then
`scripts/assert_production_ready_claim.py` writes
`reports/production-500-claim-verdict-20260517-091520.txt` with the exact
allowed claim text. `scripts/publish_production_ready_claim.py` then wrote
`reports/production-500-claim-current.{txt,md,json}`. In this historical
section, `current` is a report filename suffix, not current-artifact evidence.
The stable short command for printing that text was:

```bash
scripts/production_ready_claim.sh
```

Fresh supporting commands run for this layer:

```bash
MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh
MC_EULA_AGREE=true ./scripts/run_plugin_matrix.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh
python3 -m py_compile scripts/evaluate_production_readiness.py \
  scripts/export_production_readiness_bundle.py \
  scripts/validate_production_readiness_bundle.py \
  scripts/assert_production_ready_claim.py \
  scripts/evaluate_production_soak.py \
  scripts/evaluate_production_release.py \
  scripts/evaluate_production_release_repeat.py
python3 scripts/validate_production_readiness_bundle.py \
  reports/production-500-readiness-bundle-20260517-091520
python3 scripts/assert_production_ready_claim.py \
  reports/production-500-readiness-bundle-20260517-091520
scripts/production_ready_claim.sh
python3 scripts/publish_production_ready_claim.py
```

Latest compatibility/recovery evidence:

```text
plugin matrix: Done (21.929s), Initialized 11 plugins,
  COMPAT_PROBE command=ok events=4
restart/recovery: Done (15.527s), Saved the game,
  COMPAT_PROBE command=ok events=2
forced-ticket persistence: PASS, first/restart Done (11.386s)/(8.551s)
```

## Stress Corpus Gate For The Next Scale Envelope

The stress corpus is the next compatibility surface for the near-unbounded
track. It is deliberately heavier than the historical production 500 matrix,
but it is still only boot/join/datapack evidence until a load gate consumes it.

```bash
./scripts/fetch_stress_corpus.py
./scripts/inspect_stress_corpus.py
MC_EULA_AGREE=true ./scripts/run_stress_corpus_gate.sh
```

Fresh outcome:

```text
stress_corpus_inspection_pass=true
plugin_count=22
datapack_count=10
failure_count=0

stress_corpus_gate=PASS
matrix_plugin_count=12
stress_plugin_count=22
plugin_count=34
datapack_count=10
java_opts=-Xms4G -Xmx16G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100
Done (153.340s)
StressProbe joined the game
StressProbe left the game
There are 13 data pack(s) enabled
```

The corpus artifacts are published in:

```text
reports/stress-corpus-artifacts.csv
reports/stress-corpus-manifest.json
reports/stress-corpus-inspection.txt
reports/stress-corpus-inspection.json
reports/stress-corpus-summary.txt
```

To include the same corpus in a real load run, use the explicit opt-in flag:

```bash
MC_EULA_AGREE=true LOAD_TEST_STRESS_CORPUS=true ./scripts/run_load_test.sh
```

This does not create a production load claim by itself. A future stress-corpus
claim still needs mixed gameplay bots, mob pressure, worldgen/chunk pressure,
zero watchdog/sync-load failures, bounded memory/queues, repeat quorum, and a
self-contained evidence bundle.

## Previous 2026-05-17 CEST Soak Gate for Production 500

The historical measured `500 bots / production ready` evidence came from a
30-minute cold/fresh soak plus a 30-minute warm-source soak, both checked
against the then-current artifact hashes. This is not current-artifact
evidence now. The accepted profile was
`PAPER_CHUNK_WORKER_THREADS=10`, `PAPER_PLAYER_MAX_SEND_RATE=60`, and
`PAPER_PLAYER_MAX_GEN_RATE=20`:

```bash
MC_EULA_AGREE=true ./scripts/run_production_soak_gate.sh

python3 scripts/evaluate_production_soak.py \
  --cold-summary reports/load-production-500-cold-soak-current-artifact-20260517-052252-summary.txt \
  --warm-summary reports/load-production-500-warm-soak-current-artifact-20260517-052252-summary.txt \
  --artifact-hashes reports/artifact-hashes.txt \
  --artifacts-json reports/artifacts.json \
  --require-current-artifacts \
  --report reports/production-500-soak-gate.txt

sha256sum -c reports/artifact-hashes.txt
scripts/evaluate_production_soak_smoke.sh
```

Fresh soak outcome:

```text
production_ready_soak_claim_eligible=true
soak_gate_pass=true
failure_count=0
base_cold_gate_pass=true
base_warm_gate_pass=true
artifact_hashes_pass=true
required_load_window_metrics_samples_min=300
required_block_place_packets_min=120000
required_block_dig_packets_min=120000
cold_load_window_online_max=500
cold_load_window_loaded_chunks_max=5476
cold_load_window_tps1_avg=19.84
cold_load_window_tps1_min=19.19
cold_load_window_avg_tick_ms_avg=41.98
cold_load_window_avg_tick_ms_max=60.48
cold_bot_block_place_packets_max=264000
cold_bot_block_dig_packets_max=264000
warm_load_window_online_max=500
warm_load_window_loaded_chunks_max=5476
warm_load_window_tps1_avg=19.95
warm_load_window_tps1_min=19.28
warm_load_window_avg_tick_ms_avg=38.68
warm_load_window_avg_tick_ms_max=56.32
warm_bot_block_place_packets_max=267500
warm_bot_block_dig_packets_max=267000
cold_watchdog_thread_dumps=0
cold_sync_load_stack_hits=0
cold_stability_failures=0
warm_watchdog_thread_dumps=0
warm_sync_load_stack_hits=0
warm_stability_failures=0
evaluate_production_soak_smoke=PASS
```

The load window still excludes teardown/disconnect tail noise, which is
reported separately in the summaries for diagnostics.

## Previous 2026-05-17 CEST Production 500 Release Gate

The historical measured `500 bots / production ready` release statement was
evaluated through a single release verifier plus a repeat-quorum verifier. The
release verifier recomputed both load gates from their summaries and checked
the artifact hash manifest. The accepted historical profile used 10 chunk
workers,
`PAPER_PLAYER_MAX_SEND_RATE=60`, and `PAPER_PLAYER_MAX_GEN_RATE=20`:

```bash
MC_EULA_AGREE=true ./scripts/run_production_release_gate.sh

python3 scripts/evaluate_production_release.py \
  --cold-summary reports/load-production-500-cold-repeat-20260517-033126-run1-20260517-033126-summary.txt \
  --warm-summary reports/load-production-500-warm-repeat-20260517-033126-run1-20260517-033126-summary.txt \
  --artifact-hashes reports/artifact-hashes.txt \
  --artifacts-json reports/artifacts.json \
  --require-current-artifacts \
  --report reports/production-500-release-gate.txt
scripts/evaluate_production_release_smoke.sh

python3 scripts/evaluate_production_release_repeat.py \
  --repeat-dir auto \
  --min-passes 3 \
  --report reports/production-500-repeat-quorum.txt
```

Latest single release outcome after the repeat run:

```text
production_ready_claim_eligible=true
release_gate_pass=true
failure_count=0
artifact_hashes_pass=true
artifact_hash_count=12
cold_gate_pass=true
cold_load_window_online_max=500
cold_load_window_loaded_chunks_max=5476
cold_load_window_tps1_avg=19.84
cold_load_window_tps1_min=19.06
cold_load_window_avg_tick_ms_avg=38.47
cold_load_window_avg_tick_ms_max=55.86
cold_bot_block_place_packets_max=63000
cold_bot_block_dig_packets_max=63000
warm_gate_pass=true
warm_load_window_online_max=500
warm_load_window_loaded_chunks_max=5476
warm_load_window_tps1_avg=19.90
warm_load_window_tps1_min=19.33
warm_load_window_avg_tick_ms_avg=36.61
warm_load_window_avg_tick_ms_max=56.58
warm_bot_block_place_packets_max=63500
warm_bot_block_dig_packets_max=63500
cold_watchdog_thread_dumps=0
cold_sync_load_stack_hits=0
cold_stability_failures=0
warm_watchdog_thread_dumps=0
warm_sync_load_stack_hits=0
warm_stability_failures=0
```

Earlier single-run then-current artifact evidence, before the preserved repeat
run:

```bash
MC_EULA_AGREE=true \
  PAPER_CHUNK_WORKER_THREADS=10 \
  PAPER_PLAYER_MAX_SEND_RATE=60 \
  PAPER_PLAYER_MAX_GEN_RATE=20 \
  LOAD_TEST_LABEL=production-500-cold-current-artifact-20260517-025235 \
  ./scripts/run_production_claim_gate.sh

MC_EULA_AGREE=true \
  PAPER_CHUNK_WORKER_THREADS=10 \
  PAPER_PLAYER_MAX_SEND_RATE=60 \
  PAPER_PLAYER_MAX_GEN_RATE=20 \
  LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 \
  LOAD_TEST_LABEL=production-500-warm-current-artifact-20260517-025235 \
  ./scripts/run_production_warm_claim_gate.sh
```

Regression note: the same release path with worker10/send60 but default player
generation rate failed cold/fresh on `load_window_tps1_min=17.92 < 18.00`.
The accepted profile therefore includes `PAPER_PLAYER_MAX_GEN_RATE=20`.

Repeatability harness:

```bash
MC_EULA_AGREE=true \
  PRODUCTION_RELEASE_REPEAT_COUNT=2 \
  ./scripts/run_production_release_repeat_gate.sh
```

The repeat wrapper runs the same cold/fresh plus warm-source release gate with
unique labels for each iteration, copies each run's cold/warm summaries and
release report into `reports/release-repeat-*/run-*`, and exits non-zero on the
first failed release iteration.

Fresh repeat/quorum evidence:

```text
command=MC_EULA_AGREE=true PRODUCTION_RELEASE_REPEAT_COUNT=2 ./scripts/run_production_release_repeat_gate.sh
latest_repeat_out_dir=reports/release-repeat-20260517-041001
latest_repeat_run_1=PASS
latest_repeat_run_2=PASS
previous_repeat_out_dir=reports/release-repeat-20260517-033126
previous_repeat_run_1=PASS
quorum_report=reports/production-500-repeat-quorum.txt
required_min_passes=3
repeat_run_count=3
repeat_passes=3
repeat_failures=0
repeat_quorum_pass=true
run_1_cold_load_window_tps1_avg/min/max_mspt=19.84/18.62/61.17
run_1_warm_load_window_tps1_avg/min/max_mspt=19.88/19.32/53.27
run_2_cold_load_window_tps1_avg/min/max_mspt=19.91/18.72/54.87
run_2_warm_load_window_tps1_avg/min/max_mspt=19.90/19.12/59.48
run_3_cold_load_window_tps1_avg/min/max_mspt=19.84/19.06/55.86
run_3_warm_load_window_tps1_avg/min/max_mspt=19.90/19.33/56.58
```

## Previous 2026-05-16 Fresh 500 Cold Gate After `0097`

Earlier passing fresh command:

```bash
MC_EULA_AGREE=true \
  PAPER_CHUNK_WORKER_THREADS=8 \
  LOAD_TEST_LABEL=production-500-cold-worker8-defaultheap-windowed-20260516-223952 \
  ./scripts/run_production_claim_gate.sh
```

Outcome:

```text
claim_eligible=true
gate_pass=true
failure_count=0
world_mode=fresh
claim_surface=cold-fresh
load_window_policy=until_first_online_drop_after_reaching_bots
load_window_reached_full_online=true
load_window_metrics_samples=176
load_window_online_max=500
load_window_loaded_chunks_max=5476
load_window_tps1_avg=19.55
load_window_tps1_min=18.07
load_window_avg_tick_ms_avg=42.61
load_window_avg_tick_ms_max=65.09
process_rss_mib_max=12044.1
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_place_packets_max=59000
bot_block_dig_packets_max=59000
bot_block_action_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

This was historical gate-backed cold/fresh `production-500` evidence for the
measured 500-bot, 32/32, creative block workload profile, not
current-artifact evidence now. The harness still records overall metrics, but
the evaluator uses `load_window_*` when present and requires a complete
load-window metric set.

Regression check for that evaluator behavior:

```bash
./scripts/evaluate_load_gate_window_smoke.sh
```

Older pre-window commands from this continuation:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=production-500-block-500bots-post0097-20260516-200739 ./scripts/run_production_claim_gate.sh
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-production-500-block-500bots-post0097-20260516-200739-summary.txt
```

Outcome:

```text
claim_eligible=false
gate_pass=false
failure_count=3
online_max=500
loaded_chunks_max=5476
tps1_avg=19.38
tps1_min=12.80
avg_tick_ms_avg=42.52
avg_tick_ms_max=117.43
process_rss_mib_max=14989.3
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_place_packets_max=60000
bot_block_dig_packets_max=59500
```

The failure is narrow and happens in the early join/load window, not in the
steady 500-online plateau:

```text
min_tps at online=75 loadedChunks=5106 tps1=12.80 avgTickMs=112.72
max_tick at online=65 loadedChunks=4186 tps1=14.87 avgTickMs=117.43
first_500 at online=500 loadedChunks=5476 tps1=19.41 avgTickMs=51.11
```

This older run explains the previous blocker. It is superseded by the later
worker10 historical cold/fresh release gate above.

## Previous 2026-05-16 Warm 500 Production Gate After `0097`

Fresh commands from this continuation:

```bash
./gradlew applyPatches :paper-server:compileJava
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt
LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 \
  LOAD_TEST_LABEL=production-500-warm-block-500bots-post0097-20260516-194812 \
  MC_EULA_AGREE=true ./scripts/run_production_warm_claim_gate.sh
```

Outcome:

```text
applyPatches + compileJava: PASS
build_optimized.sh: PASS
native tests: 291 passed, 0 failed
artifact hashes: PASS

gate_profile=production-500-warm
claim_eligible=true
gate_pass=true
failure_count=0
online_max=500
loaded_chunks_max=5476
tps1_avg=19.86
tps1_min=19.03
avg_tick_ms_avg=36.95
avg_tick_ms_max=67.13
process_rss_mib_max=5149.8
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_place_packets_max=60500
bot_block_dig_packets_max=60000
bot_block_action_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

This is sufficient evidence for the narrow saved/pregenerated-world
`production-500-warm` claim: 500 bots, 32/32 view and simulation distance,
creative block actions, strict TPS/MSPT thresholds, RSS cap, and zero recorded
stability failures.

It is not evidence for the broader cold/fresh-world `production-500` claim.
The cold gate still rejects warm-source summaries and must pass separately; the
later fresh-world evidence was the worker10 historical cold gate above.

## Previous 2026-05-15 Warm 500 JFR After PlayerList Broadcast Tweak

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 LOAD_TEST_LABEL=warm500-playerlist-jfr2-20260515 LOAD_TEST_SCENARIO=block LOAD_TEST_GAMEMODE=creative BOT_COUNT=500 DURATION_SECONDS=360 BOT_BLOCK_RAMP_SECONDS=180 BOT_BLOCK_ACTION_INTERVAL_MS=1000 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-warm500-playerlist-jfr2-20260515.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
jfr view hot-methods reports/load-warm500-playerlist-jfr2-20260515.jfr | head -n 40
jfr view allocation-by-site reports/load-warm500-playerlist-jfr2-20260515.jfr | head -n 40
```

Outcome:

```text
warm JFR run: PASS
online_max=500
loaded_chunks_max=5476
tps1_avg=13.71
avg_tick_ms_avg=98.19
watchdog_thread_dumps=3
nearby_players_stack_hits=5
```

This JFR shifts the next target to `ChunkMap$TrackedEntity.updatePlayer(...)`
and away from PlayerList broadcast.

## Previous 2026-05-15 PlayerList Broadcast Candidate

Fresh commands from this continuation:

```bash
./gradlew :paper-server:compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
./scripts/bench_playerlist_broadcast_cansee.sh
MC_EULA_AGREE=true LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 ./scripts/run_production_warm_claim_gate.sh
```

Outcome:

```text
compileJava: PASS
build_optimized.sh: PASS
artifact_hashes: PASS
bench_playerlist_broadcast_cansee.sh: PASS
empty_candidate_speedup=1.213x
populated_candidate_speedup=1.718x
equivalence=PASS
warm production gate: FAIL
online_max=500
loaded_chunks_max=5476
tps1_avg=15.53
tps1_min=5.44
avg_tick_ms_avg=76.90
avg_tick_ms_max=239.69
watchdog_thread_dumps=3
nearby_players_stack_hits=2
```

This is still not a production-ready claim.

## Previous 2026-05-15 Production Claim Gate

The project now has a hard claim gate for the phrase "500 bots / production
ready":

```bash
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-...-summary.txt
MC_EULA_AGREE=true ./scripts/run_production_claim_gate.sh
```

The `production-500` profile is the cold/fresh-world claim gate. It requires
a real block scenario at 32/32 view and simulation distance, 500
online/ready/active bots, 500 armed and primed block bots, block place and dig
activity, zero kicks/errors/watchdog/sync-load/thread failures,
`tps1_avg >= 19.5`, `tps1_min >= 18.0`, `avg_tick_ms_avg <= 50.0`,
`avg_tick_ms_max <= 100.0`, and RSS below `28672 MiB`. If a summary records
`world_warm_source_present=true`, this profile rejects it. If a summary records
`load_window_policy`, the evaluator requires a complete `load_window_*` metric
set and evaluates the load window rather than teardown noise. The current
fresh-world pass is
`reports/load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-summary.txt`.

There is now a separate warm-world gate for a narrower claim:

```bash
LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 \
  MC_EULA_AGREE=true ./scripts/run_production_warm_claim_gate.sh
python3 scripts/evaluate_load_gate.py --profile production-500-warm reports/load-...-summary.txt
```

`production-500-warm` keeps the same 500-bot, 32/32, block-action, TPS/MSPT,
RSS, and zero-stability-failure requirements, but additionally requires
`world_warm_source_present=true` and `world_mode=warm-source`. This is evidence
only for a saved/pregenerated world. It does not replace the cold
`production-500` gate.

Fresh warm-world evidence as of this run:

```text
bots=500
world_mode=warm-source
spark_background_profiler=false
online_max=500
loaded_chunks_max=5476
tps1_avg=15.15
tps1_min=4.55
avg_tick_ms_avg=81.72
avg_tick_ms_max=271.61
watchdog_thread_dumps=5
nearby_players_stack_hits=0
```

That 2026-05-15 warm-world run was still a fail for `production-500-warm`.
Newer warm and cold gate evidence above supersedes that historical status.

## Previous 2026-05-15 500-Bot Block Gate With SurfaceRules Chain Candidate

Fresh commands:

```bash
./scripts/bench_surfacerules_sequence_array.sh
./gradlew applyPatches --no-daemon
./gradlew :paper-server:compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true LOAD_TEST_SCENARIO=block LOAD_TEST_GAMEMODE=creative BOT_COUNT=500 DURATION_SECONDS=600 BOT_BLOCK_RAMP_SECONDS=300 BOT_BLOCK_ACTION_INTERVAL_MS=1000 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_LABEL=block-500-surfacerules-chain-20260515 ./scripts/run_load_test.sh
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-block-500-surfacerules-chain-20260515-summary.txt
```

Outcome:

```text
bench linked_speedup=1.153x
bench equivalence=PASS
build_optimized.sh=PASS
artifact_hashes=PASS
online_max=500
loaded_chunks_max=5476
bot_block_place_packets_max=60000
bot_block_dig_packets_max=60000
bot_kicked_max=0
bot_errors_max=0
tps1_avg=8.98
tps1_min=3.97
avg_tick_ms_avg=148.42
avg_tick_ms_max=626.01
watchdog_thread_dumps=5
nearby_players_stack_hits=8
```

Gate result: `production-500` FAIL with `failure_count=6`. The candidate was
reverted. Post-revert verification passed: rebuilt optimized runtime,
refreshed AppCDS/artifact hashes, plugin matrix, restart/recovery, and
forced-ticket persistence.

## Previous 2026-05-15 100-Bot Block Plateau After Perlin MathWrap Guarded

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=100 DURATION_SECONDS=240 BOT_BLOCK_ACTION_INTERVAL_MS=1000 LOAD_TEST_LABEL=block-100-perlin-mathwrap-guarded-20260515 ./scripts/run_load_test.sh
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-block-100-perlin-mathwrap-guarded-20260515-summary.txt
```

Outcome:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
native tests: 291 passed
artifact hashes: all listed OK

load_test_scenario=block
load_test_gamemode=creative
bukkit_connection_throttle=0
online_max=100
loaded_chunks_max=5184
tps1_avg=11.46
tps1_min=1.31
avg_tick_ms_avg=95.89
avg_tick_ms_max=448.32
process_rss_mib_max=10787.6
bot_block_armed_max=100
bot_block_primed_max=100
bot_block_place_packets_max=5197
bot_block_dig_packets_max=5097
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0

Load claim gate failed for profile production-500: 16 failure(s).
claim_eligible=false
gate_pass=false
observed_bots=100
failure=bots=100 < required 500
failure=tps1_avg=11.46 < required 19.50
failure=avg_tick_ms_avg=95.89 > allowed 50.00
failure=avg_tick_ms_max=448.32 > allowed 100.00
```

Verdict: this is a real improvement over the prior 100-bot block plateau, but
it still misses the hard 500-bot production gate by a wide margin.

## Previous 2026-05-15 Runtime Artifact Refresh After ImprovedNoise Handle Rejection

Fresh commands from this continuation:

```bash
./gradlew :paper-server:applyPatches --no-daemon
./gradlew :paper-server:compileJava --no-daemon
bash -n scripts/prepare_fast_runtime.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true ./scripts/generate_app_cds.sh artifacts/optimized-runtime/run.sh artifacts/optimized-runtime
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
rg -n "native_improved_noise" logs/app-cds.log
```

Outcome:

```text
applyPatches: PASS
compileJava: PASS
bash -n scripts/prepare_fast_runtime.sh: PASS
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
native tests: 291 passed
generate_app_cds.sh: PASS
artifact hashes: all listed OK
logs/app-cds.log: native_improved_noise=false
```

Verdict: current artifacts are reproducible after rejecting the per-call
native ImprovedNoise runtime hook. This is build/artifact evidence only; it
does not replace the `production-500` load gate.

## Current 2026-05-14 22:47 CEST 100-Bot Block Plateau With Join Throttle Disabled

Fresh commands from this continuation:

```bash
bash -n scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=100 DURATION_SECONDS=240 BOT_BLOCK_ACTION_INTERVAL_MS=1000 LOAD_TEST_LABEL=block-100-autoramp-throttle0-20260514 ./scripts/run_load_test.sh
```

Outcome:

```text
bash -n scripts/run_load_test.sh: PASS
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=100 DURATION_SECONDS=240 BOT_BLOCK_ACTION_INTERVAL_MS=1000 LOAD_TEST_LABEL=block-100-autoramp-throttle0-20260514 ./scripts/run_load_test.sh: PASS

bukkit_connection_throttle=0
online_max=100
loaded_chunks_max=5184
tps1_avg=9.88
avg_tick_ms_avg=223.62
bot_block_armed_max=100
bot_block_primed_max=100
bot_block_creative_slot_packets_max=100
bot_block_place_packets_max=5046
bot_block_dig_packets_max=5031
bot_block_action_errors_max=0
bot_block_actions_per_sec_max=42.1
compat_probe_arena_prepared_max=100
compat_probe_arena_skipped_total=1176
server_join_events=100
server_quit_events=100
process_rss_mib_max=11515.4
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
```

This is the first full 100-bot creative block plateau with the localhost join
throttle explicitly disabled in the run directory. It still is not a
production-ready or 500-player claim because TPS/MSPT and RSS are too high.

## Current 2026-05-14 21:49 CEST 50-Bot Block Plateau After Arena-Window Fix

Fresh commands from this continuation:

```bash
bash -n scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=50 DURATION_SECONDS=150 BOT_RAMP_SECONDS=10 LOAD_TEST_LABEL=block-50-arenafix-20260514 ./scripts/run_load_test.sh
```

Outcome:

```text
bash -n scripts/run_load_test.sh: PASS
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=50 DURATION_SECONDS=150 BOT_RAMP_SECONDS=10 LOAD_TEST_LABEL=block-50-arenafix-20260514 ./scripts/run_load_test.sh: PASS

online_max=50
loaded_chunks_max=3055
tps1_avg=12.50
avg_tick_ms_avg=183.80
bot_block_armed_max=50
bot_block_primed_max=50
bot_block_creative_slot_packets_max=50
bot_block_place_packets_max=20750
bot_block_dig_packets_max=20716
bot_block_action_errors_max=0
bot_block_actions_per_sec_max=276.4
compat_probe_block_places_max=16565
compat_probe_block_breaks_max=17816
compat_probe_arena_commands_max=28
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
```

Verdict: the block harness now reaches a real full 50-bot armed/primed
plateau, but the server is not production-ready for the requested target:
TPS/MSPT are still far below the target and this was a busy-host run.

## Current 2026-05-14 20:09 CEST Warm-World Benchmark and Current Area-Map Reports

Fresh commands from this continuation:

```bash
bash -n scripts/warm_world_benchmark.sh
MC_EULA_AGREE=true ./scripts/warm_world_benchmark.sh
```

Outcome:

```text
bash -n scripts/warm_world_benchmark.sh: PASS
MC_EULA_AGREE=true ./scripts/warm_world_benchmark.sh: PASS

name,port,status_ms,done_ms,rss_kb,stop_ms,log,source
stock-paper-1.21.10,56443,73810,73872,1828804,79029,/root/rust/logs/warm-world-stock-paper-1.21.10.log,/root/rust/runs/plugin-matrix
optimized-paper-1.21.10,34315,46223,46268,1666672,51366,/root/rust/logs/warm-world-optimized-paper-1.21.10.log,/root/rust/runs/plugin-matrix
optimized-runtime-1.21.10,36675,36121,36185,1251232,41280,/root/rust/logs/warm-world-optimized-runtime-1.21.10.log,/root/rust/runs/plugin-matrix
```

Derived warm-start done-time ratios:

```text
optimized_paper_vs_stock_done_speedup=1.597x
optimized_runtime_vs_stock_done_speedup=2.042x
optimized_runtime_vs_optimized_paper_done_speedup=1.279x
```

Current area-map report files read in this continuation:

```text
reports/native-area-map-bench.txt:
equivalence=PASS
update_native_speedup_vs_java=1.218x
add_native_speedup_vs_java=1.216x
remove_native_speedup_vs_java=1.168x

reports/load-50bots-area-map-native-gate-20260514-summary.txt:
tps1_avg=17.24
avg_tick_ms_avg=75.12
loaded_chunks_max=2766
watchdog_thread_dumps=6

reports/load-50bots-nearby-player-map-presize-gate-20260514-preflight.txt:
host_preflight_ok=false
load_per_cpu=1.946
idle_percent_1s=0.75
```

Verdict: warm-world startup evidence is valid for the tested saved world, and
area-map Java/native parity is still valid for the focused bench. Neither is
an accepted strict load-gate win or a 500-player claim.

## Current 2026-05-14 11:30 CEST Full Native Mega-All Pack

Fresh commands from this continuation:

```bash
PACK_LIST=1 PACK_GROUPS=all scripts/bench_native_pack.sh
PACK_WRITE_MANIFEST=1 PACK_LABEL=mega-all-complete-v4 PACK_FAIL_FAST=1 PACK_GROUPS=all scripts/bench_native_pack.sh
scripts/verify_native_pack_complete.sh
python3 scripts/native_pack_report.py reports/native-pack-mega-all-complete-v4.txt
python3 scripts/native_coverage_audit.py --strict-docs
find scripts -maxdepth 1 -name 'bench_native_*.sh' -type f -print0 | sort -z | xargs -0 bash -n
python3 -m py_compile scripts/native_coverage_audit.py scripts/native_pack_report.py
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
PACK_LIST=1 PACK_GROUPS=all scripts/bench_native_pack.sh: 96 scripts listed

PACK_WRITE_MANIFEST=1 PACK_LABEL=mega-all-complete-v4 PACK_FAIL_FAST=1 PACK_GROUPS=all scripts/bench_native_pack.sh: PASS
script_count=96
leaf_group_count=10
leaf_group_memberships=96
all_real_scripts_expected=96
all_real_scripts_covered=96
summary_manifest_entries=96
summary_manifest_groups=10
summary_manifest_duplicate_scripts=0
summary_result_scripts_missing_manifest=0
summary_manifest_scripts_missing_result=0
summary_manifest_entries_match_scripts=TRUE
summary_manifest_groups_match_leaf_count=TRUE
pack_status=PASS failures=0
summary_scripts=96
summary_pack_status_present=TRUE
summary_started_scripts=96
summary_duplicate_starts=0
summary_results_missing_start=0
summary_started_scripts_missing_result=0
summary_start_result_sets_match=TRUE
summary_declared_script_count=96
summary_script_count_matches_declared=TRUE
summary_all_real_expected=96
summary_all_real_covered=96
summary_all_real_coverage_matches=TRUE
summary_leaf_group_count=10
summary_leaf_group_memberships=96
summary_leaf_group_memberships_match_scripts=TRUE
summary_duplicate_scripts=0
summary_pass=96
summary_fail=0
summary_total_duration_ms=2582751
summary_slowest_script=scripts/bench_native_spigot_load_order_dependency.sh
summary_slowest_duration_ms=260472
summary_equivalence_pass_lines=96
summary_equivalence_fail_lines=0
summary_speedup_lines=504
summary_speedup_ge_1x=308
summary_speedup_lt_1x=196
summary_status=PASS

scripts/verify_native_pack_complete.sh: PASS
python3 scripts/native_coverage_audit.py --strict-docs: PASS
modules_total=89
required_bench_dirs_covered=92
required_scripts_covered=97
native_wrappers_checked=90
native_load_wrappers_checked=90
native_exports_checked=243
pack_all_real_expected=96
pack_all_scripts_listed=96
pack_all_scripts_unique=96
pack_all_missing=0
pack_all_extra=0
pack_all_duplicates=0
warnings=0
errors=0

bash -n all bench_native_*.sh scripts: PASS
python3 -m py_compile scripts/native_coverage_audit.py scripts/native_pack_report.py: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

## Current 2026-05-14 02:33 CEST Native Mega-Pack Runner and Report Summary

Fresh commands from this continuation:

```bash
bash -n scripts/bench_native_pack.sh scripts/bench_native_waypoint_hotpath.sh
python3 -m py_compile scripts/native_coverage_audit.py scripts/native_pack_report.py
PACK_LIST=1 PACK_GROUPS='aquifer climate entity waypoint plugin storage ticket' scripts/bench_native_pack.sh
python3 scripts/native_coverage_audit.py --strict-docs
PACK_LABEL=mega-bounded PACK_FAIL_FAST=1 PACK_GROUPS='aquifer climate entity waypoint plugin storage ticket' scripts/bench_native_pack.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
bash -n scripts/bench_native_pack.sh scripts/bench_native_waypoint_hotpath.sh: PASS
python3 -m py_compile scripts/native_coverage_audit.py scripts/native_pack_report.py: PASS
PACK_LIST=1 PACK_GROUPS='aquifer climate entity waypoint plugin storage ticket' scripts/bench_native_pack.sh: 56 scripts listed
python3 scripts/native_coverage_audit.py --strict-docs: PASS
modules_total=89
required_bench_dirs_covered=92
required_scripts_covered=97
native_wrappers_checked=90
native_load_wrappers_checked=90
native_exports_checked=243
warnings=0
errors=0

PACK_LABEL=mega-bounded PACK_FAIL_FAST=1 PACK_GROUPS='aquifer climate entity waypoint plugin storage ticket' scripts/bench_native_pack.sh: PASS
script_count=56
pack_heavy_defaults=1
summary_scripts=56
summary_pass=56
summary_fail=0
summary_total_duration_ms=2048339
summary_slowest_script=scripts/bench_native_spigot_load_order_dependency.sh
summary_slowest_duration_ms=272706
summary_equivalence_pass_lines=57
summary_equivalence_fail_lines=0
summary_speedup_lines=318
summary_speedup_ge_1x=190
summary_speedup_lt_1x=128
summary_status=PASS

sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

## Current 2026-05-13 23:03 CEST Native Coverage Audit and Wide Worldgen Pack Smoke

Fresh commands from this continuation:

```bash
python3 scripts/native_coverage_audit.py --strict-docs
bash -n scripts/bench_native_worldgen_pack.sh scripts/bench_native_noisechunk_wrap_capacity.sh
python3 -m py_compile scripts/native_coverage_audit.py
PACK_FAIL_FAST=1 PACK_SCRIPTS='scripts/bench_native_improved_noise.sh scripts/bench_native_improved_noise_inline.sh scripts/bench_native_improved_noise_derivative.sh scripts/bench_native_perlin_noise.sh scripts/bench_native_perlin_getvalue.sh scripts/bench_native_blended_noise.sh scripts/bench_native_noise_generator_settings.sh scripts/bench_native_density_ap2_fill.sh scripts/bench_native_density_ap2_minmax_fill.sh scripts/bench_native_density_visitor_hooks.sh scripts/bench_native_surface_rules_sequence_array.sh scripts/bench_native_surface_rules_test_rule_state.sh scripts/bench_native_placed_feature_traversal.sh scripts/bench_native_ore_feature_loop.sh scripts/bench_native_carver_iteration.sh scripts/bench_native_cave_carver_skip.sh' scripts/bench_native_worldgen_pack.sh
tail -n 120 reports/native-worldgen-pack.txt
```

Outcome:

```text
python3 scripts/native_coverage_audit.py --strict-docs: PASS
modules_total=89
required_bench_dirs_covered=92
required_scripts_covered=97
native_wrappers_checked=90
native_exports_checked=243
warnings=0
errors=0

bash -n scripts/bench_native_worldgen_pack.sh scripts/bench_native_noisechunk_wrap_capacity.sh: PASS
python3 -m py_compile scripts/native_coverage_audit.py: PASS

wide PACK_SCRIPTS scripts/bench_native_worldgen_pack.sh: PASS
script_count=16
PACK_RESULT script=scripts/bench_native_improved_noise.sh status=PASS duration_ms=2119
PACK_RESULT script=scripts/bench_native_improved_noise_inline.sh status=PASS duration_ms=6704
PACK_RESULT script=scripts/bench_native_improved_noise_derivative.sh status=PASS duration_ms=5288
PACK_RESULT script=scripts/bench_native_perlin_noise.sh status=PASS duration_ms=7650
PACK_RESULT script=scripts/bench_native_perlin_getvalue.sh status=PASS duration_ms=14074
PACK_RESULT script=scripts/bench_native_blended_noise.sh status=PASS duration_ms=43740
PACK_RESULT script=scripts/bench_native_noise_generator_settings.sh status=PASS duration_ms=4610
PACK_RESULT script=scripts/bench_native_density_ap2_fill.sh status=PASS duration_ms=98877
PACK_RESULT script=scripts/bench_native_density_ap2_minmax_fill.sh status=PASS duration_ms=5204
PACK_RESULT script=scripts/bench_native_density_visitor_hooks.sh status=PASS duration_ms=7926
PACK_RESULT script=scripts/bench_native_surface_rules_sequence_array.sh status=PASS duration_ms=35129
PACK_RESULT script=scripts/bench_native_surface_rules_test_rule_state.sh status=PASS duration_ms=10657
PACK_RESULT script=scripts/bench_native_placed_feature_traversal.sh status=PASS duration_ms=6398
PACK_RESULT script=scripts/bench_native_ore_feature_loop.sh status=PASS duration_ms=3915
PACK_RESULT script=scripts/bench_native_carver_iteration.sh status=PASS duration_ms=8937
PACK_RESULT script=scripts/bench_native_cave_carver_skip.sh status=PASS duration_ms=8099
pack_status=PASS failures=0
```

## Current 2026-05-13 22:03 UTC Rust Compression/IO Shape Native Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml --workspace
bash -n scripts/bench_native_lz4_stream_roundtrip.sh
bash -n scripts/bench_native_nbt_gzip_buffer_shape.sh
bash -n scripts/bench_native_compression_threshold_shape.sh
JAVA_PROPS='-Diterations=16 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_lz4_stream_roundtrip.sh
JAVA_PROPS='-Drepeats=1024 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_nbt_gzip_buffer_shape.sh
JAVA_PROPS='-Diterations=1024 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_compression_threshold_shape.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
cargo test --manifest-path native/Cargo.toml --workspace: PASS, 287 paper-native-core tests passed
bash -n scripts/bench_native_lz4_stream_roundtrip.sh: PASS
bash -n scripts/bench_native_nbt_gzip_buffer_shape.sh: PASS
bash -n scripts/bench_native_compression_threshold_shape.sh: PASS
./scripts/bench_native_lz4_stream_roundtrip.sh: PASS
block_32768_native_speedup_vs_java=0.426x
block_65536_native_speedup_vs_java=0.404x
block_131072_native_speedup_vs_java=0.419x
equivalence=PASS
./scripts/bench_native_nbt_gzip_buffer_shape.sh: PASS
current_native_speedup_vs_java=1.735x
gzip64k_native_speedup_vs_java=1.830x
prebuffer64k_native_speedup_vs_java=1.708x
both64k_native_speedup_vs_java=1.699x
equivalence=PASS
./scripts/bench_native_compression_threshold_shape.sh: PASS
default_native_speedup_vs_java=6.236x
tight_native_speedup_vs_java=5.301x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new compression/IO modules are parity-clean diagnostic evidence
only. LZ4 round-trip is slower than Java in this JNI shape, and the NBT/GZIP
and threshold modules are model counters rather than production runtime hooks.

## Current 2026-05-13 22:38 CEST Rust ObfHelper Maps Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml --workspace
./scripts/bench_native_obfhelper_maps.sh
bash -n scripts/bench_native_obfhelper_maps.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
cargo check --manifest-path native/Cargo.toml --workspace: PASS
./scripts/bench_native_obfhelper_maps.sh: PASS
classes=7554
methods=47786
fields=31113
old_stream_default_native_speedup_vs_java=0.395x
direct_maps_native_speedup_vs_java=0.398x
presized_string_pool_native_speedup_vs_java=0.429x
equivalence=PASS
bash -n scripts/bench_native_obfhelper_maps.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: the new obfhelper mapping-bootstrap module is parity-clean
diagnostic evidence only. It does not install a Paper runtime hook.

## Current 2026-05-13 16:58 CEST Rust Waypoint Chunk Update and Remapper Hash Threshold Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml --workspace
./scripts/build_native.sh
WAYPOINT_CHUNK_ITERATIONS=4000000 WAYPOINT_CHUNK_WARMUP=2 WAYPOINT_CHUNK_ROUNDS=4 ./scripts/bench_native_waypoint_chunk_update.sh
HASH_BENCH_ITERATIONS=3 HASH_BENCH_ROUNDS=2 HASH_BENCH_WARMUP=1 ./scripts/bench_native_remapper_hash_threshold.sh
bash -n scripts/bench_native_waypoint_chunk_update.sh
bash -n scripts/bench_native_remapper_hash_threshold.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
cargo check --manifest-path native/Cargo.toml --workspace: PASS
./scripts/build_native.sh: PASS, 245 paper-native-core tests passed
release lib hash: 6c09aeedf3a9fb96166a93d8068bf0ff4b1bc0df854519c0b1bbbe6e1c3d8fc9
./scripts/bench_native_waypoint_chunk_update.sh: PASS
distance_native_speedup_vs_java=0.266x
long_key_native_speedup_vs_java=0.197x
long_key_speedup=2.587x
equivalence=PASS
./scripts/bench_native_remapper_hash_threshold.sh: PASS
inputs=13
sizes=1,2,4,8,12
size=12 compute_if_absent_native_speedup_vs_java=0.646x
size=12 put_native_speedup_vs_java=0.683x
size=12 hybrid_native_speedup_vs_java=0.602x
size=12 parallel_native_speedup_vs_java=0.650x
size=12 native_parallel_speedup_vs_put=2.579x
equivalence=PASS
bash -n scripts/bench_native_waypoint_chunk_update.sh: PASS
bash -n scripts/bench_native_remapper_hash_threshold.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new waypoint chunk-update and remapper hash-threshold modules
are parity-clean diagnostic evidence only. They do not install Paper runtime
hooks and they do not make strict-gate claims.

## Current 2026-05-13 16:08 CEST Rust Waypoint Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_waypoint_snapshot.sh
./scripts/bench_native_waypoint_table_view.sh
./scripts/bench_native_waypoint_manager_skip.sh
bash -n scripts/bench_native_waypoint_snapshot.sh scripts/bench_native_waypoint_table_view.sh scripts/bench_native_waypoint_manager_skip.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 242 paper-native-core tests passed
release lib hash: d3ddef2d4224c4f35fd40a55640555f020190fc88251fcdb8bc9a130a95dc2aa
./scripts/bench_native_waypoint_snapshot.sh: PASS
toArray_native_speedup_vs_java=18362.610x
sizedArray_native_speedup_vs_java=28326.422x
manual_native_speedup_vs_java=12246.901x
equivalence=PASS
./scripts/bench_native_waypoint_table_view.sh: PASS
transpose_row_native_speedup_vs_java=14612.526x
column_native_speedup_vs_java=17070.012x
equivalence=PASS
./scripts/bench_native_waypoint_manager_skip.sh: PASS
current_player_full_native_speedup_vs_java=3872.955x
skip_player_full_native_speedup_vs_java=2162.004x
current_player_partial_native_speedup_vs_java=3930.484x
skip_player_partial_native_speedup_vs_java=2412.447x
current_waypoint_full_native_speedup_vs_java=2649.895x
skip_waypoint_full_native_speedup_vs_java=2225.330x
current_waypoint_partial_native_speedup_vs_java=4522.427x
skip_waypoint_partial_native_speedup_vs_java=4337.273x
equivalence=PASS
bash -n scripts/bench_native_waypoint_snapshot.sh scripts/bench_native_waypoint_table_view.sh scripts/bench_native_waypoint_manager_skip.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new waypoint snapshot/table-view/manager-skip modules are
parity-clean diagnostic evidence only. They do not install Paper runtime
hooks and they do not make strict-gate claims.

## Current 2026-05-13 15:15 CEST Rust Worldgen and Ticketset Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_improved_noise_floor.sh
./scripts/bench_native_surface_rules_sequence_array.sh
./scripts/bench_native_surface_rules_test_rule_state.sh
./scripts/bench_native_placed_feature_traversal.sh
./scripts/bench_native_ore_feature_loop.sh
./scripts/bench_native_ticketset_search.sh
bash -n scripts/bench_native_placed_feature_traversal.sh scripts/bench_native_ore_feature_loop.sh scripts/bench_native_ticketset_search.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 235 paper-native-core tests passed
release lib hash: 6538828a942f7d1183a4cfed03d6a7dd12c85cb2af1d8aaa1e3100003a78dc1f
./scripts/bench_native_improved_noise_floor.sh: PASS
current_mth_floor_native_speedup_vs_java=0.588x
math_floor_native_speedup_vs_java=0.701x
equivalence=PASS
./scripts/bench_native_surface_rules_sequence_array.sh: PASS
list_enhanced_native_speedup_vs_java=2.456x
list_indexed_native_speedup_vs_java=6.337x
array_foreach_native_speedup_vs_java=1.938x
array_indexed_native_speedup_vs_java=3.567x
equivalence=PASS
./scripts/bench_native_surface_rules_test_rule_state.sh: PASS
period7_old_native_speedup_vs_java=1.445x
period7_new_native_speedup_vs_java=1.321x
period2_old_native_speedup_vs_java=1.593x
period2_new_native_speedup_vs_java=1.410x
equivalence=PASS
./scripts/bench_native_placed_feature_traversal.sh: PASS
native_speedup_vs_java_stream=21.813x
native_speedup_vs_java_recursive=29.905x
equivalence=PASS
./scripts/bench_native_ore_feature_loop.sh: PASS
native_old_speedup_vs_java=1.593x
native_optimized_speedup_vs_java=1.491x
equivalence=PASS
./scripts/bench_native_ticketset_search.sh: PASS
binary_native_speedup_vs_java=3.220x
unchecked_binary_native_speedup_vs_java=3.209x
linear4_native_speedup_vs_java=3.608x
linear8_native_speedup_vs_java=3.174x
linear12_native_speedup_vs_java=3.498x
equivalence=PASS
bash -n scripts/bench_native_placed_feature_traversal.sh scripts/bench_native_ore_feature_loop.sh scripts/bench_native_ticketset_search.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new worldgen/surface-rules/ticketset modules are parity-clean
diagnostic evidence only. They do not install Paper runtime hooks and they do
not make strict-gate claims.

## Current 2026-05-13 13:45 CEST Rust Protochunk Heightmap and Range Choice Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_protochunk_heightmap.sh
./scripts/bench_native_range_choice.sh
bash -n scripts/bench_native_protochunk_heightmap.sh scripts/bench_native_range_choice.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 223 paper-native-core tests passed
release lib hash: 59605ce068888933fd5d30313fd767cfcdfb9ea1729752182cae55082d3dc3a7
./scripts/bench_native_protochunk_heightmap.sh: PASS
old_enumset_foreach_native_speedup_vs_java=7.615x
new_cached_contains_native_speedup_vs_java=1.344x
java_speedup_vs_old=1.208x
native_speedup_vs_old=0.213x
equivalence=PASS
./scripts/bench_native_range_choice.sh: PASS
scenario=in_constant_out_dynamic
old_fillarray_native_speedup_vs_java=1.056x
optimized_fillarray_native_speedup_vs_java=0.777x
java_optimized_speedup_vs_old=1.107x
native_optimized_speedup_vs_old=0.815x
scenario=in_dynamic_out_constant
old_fillarray_native_speedup_vs_java=0.966x
optimized_fillarray_native_speedup_vs_java=0.883x
java_optimized_speedup_vs_old=1.059x
native_optimized_speedup_vs_old=0.968x
scenario=both_constant
old_fillarray_native_speedup_vs_java=1.009x
optimized_fillarray_native_speedup_vs_java=0.808x
java_optimized_speedup_vs_old=1.239x
native_optimized_speedup_vs_old=0.992x
scenario=both_dynamic
old_fillarray_native_speedup_vs_java=1.061x
optimized_fillarray_native_speedup_vs_java=0.963x
java_optimized_speedup_vs_old=1.034x
native_optimized_speedup_vs_old=0.939x
equivalence=PASS
bash -n scripts/bench_native_protochunk_heightmap.sh scripts/bench_native_range_choice.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new protochunk-heightmap and range-choice modules are
parity-clean diagnostic evidence only. They do not install Paper runtime
hooks and they do not make strict-gate claims.

## Current 2026-05-13 13:10 CEST Rust Climate Parameter Distance and Noise Generator Settings Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_climate_parameter_distance.sh
./scripts/bench_native_noise_generator_settings.sh
./scripts/bench_native_chunk_expire_count.sh
./scripts/bench_native_craftplayer_cansee.sh
./scripts/bench_native_levelchunk_heightmap.sh
bash -n scripts/bench_native_climate_parameter_distance.sh scripts/bench_native_noise_generator_settings.sh scripts/bench_native_chunk_expire_count.sh scripts/bench_native_craftplayer_cansee.sh scripts/bench_native_levelchunk_heightmap.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 218 paper-native-core tests passed
release lib hash: 2603ee607be92705ef1435de6ee2e5d499df6bcfdcbfb12e6ca8c7ef1815d383
./scripts/bench_native_climate_parameter_distance.sh: PASS
old_distance_native_speedup_vs_java=3.124x
branch_distance_native_speedup_vs_java=5.274x
subtract_first_distance_native_speedup_vs_java=3.072x
equivalence=PASS
./scripts/bench_native_noise_generator_settings.sh: PASS
holder_value_settings_native_speedup_vs_java=3.113x
memoized_supplier_settings_native_speedup_vs_java=6.056x
lazy_primitive_settings_native_speedup_vs_java=2.543x
manual_lazy_object_settings_native_speedup_vs_java=3.514x
cached_int_settings_native_speedup_vs_java=1.306x
equivalence=PASS
./scripts/bench_native_chunk_expire_count.sh: PASS
dynamic_compute_hot_native_speedup_vs_java=0.491x
dynamic_manual_hot_native_speedup_vs_java=0.421x
cached_compute_hot_native_speedup_vs_java=0.538x
cached_hybrid_hot_native_speedup_vs_java=0.356x
cached_manual_hot_native_speedup_vs_java=0.307x
equivalence=PASS
./scripts/bench_native_craftplayer_cansee.sh: PASS
current_empty_native_speedup_vs_java=61.448x
guarded_empty_native_speedup_vs_java=40.344x
candidate_empty_native_speedup_vs_java=33.700x
no_equals_hash_empty_native_speedup_vs_java=10.747x
current_populated_native_speedup_vs_java=13.782x
guarded_populated_native_speedup_vs_java=16.757x
candidate_populated_native_speedup_vs_java=107.348x
no_equals_hash_populated_native_speedup_vs_java=38.142x
equivalence=PASS
./scripts/bench_native_levelchunk_heightmap.sh: PASS
old_four_update_native_speedup_vs_java=1.484x
new_combined_update_native_speedup_vs_java=0.920x
combined_speedup=0.646x
equivalence=PASS
bash -n scripts/bench_native_climate_parameter_distance.sh scripts/bench_native_noise_generator_settings.sh scripts/bench_native_chunk_expire_count.sh scripts/bench_native_craftplayer_cansee.sh scripts/bench_native_levelchunk_heightmap.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new climate-parameter-distance, noise-generator-settings,
chunk-expire-count, craftplayer-cansee, and levelchunk-heightmap modules are
parity-clean diagnostic evidence only. They do not install Paper runtime
hooks and they do not make strict-gate claims.

## Current 2026-05-13 09:15 CEST Rust Nearby Player Map Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_nearby_player_map.sh
bash -n scripts/bench_native_nearby_player_map.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS
release lib hash: 0d2606fe5dc02d1b76c83816faf8b8ddd0adb6b3b0f11549c47f034c4a953d16
./scripts/bench_native_nearby_player_map.sh: PASS
default_native_speedup_vs_java=69.919x (50 players)
presized_native_speedup_vs_java=39.047x (50 players)
default_native_speedup_vs_java=87.489x (500 players)
presized_native_speedup_vs_java=41.880x (500 players)
equivalence=PASS
bash -n scripts/bench_native_nearby_player_map.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the nearby-player-map capacity module is parity-clean diagnostic
evidence only. It does not install a Paper runtime hook and it does not make
a strict-gate/server throughput claim.

## Current 2026-05-13 09:40 CEST Rust Marker Cache and Waypoint Distance Guard Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_marker_cache.sh
./scripts/bench_native_waypoint_distance_guard.sh
bash -n scripts/bench_native_marker_cache.sh
bash -n scripts/bench_native_waypoint_distance_guard.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 205 paper-native-core tests passed
release lib hash: b96ac2a4e7067c2453fc9ecb23f4ee12582c6f903ab394474e17225caa3efcdc
./scripts/bench_native_marker_cache.sh: PASS
old_native_speedup_vs_java=1.311x
cached_native_speedup_vs_java=0.364x
java_cached_speedup_vs_old=0.956x
native_cached_speedup_vs_old=0.265x
equivalence=PASS
./scripts/bench_native_waypoint_distance_guard.sh: PASS
old_range_native_speedup_vs_java=0.827x
guarded_range_native_speedup_vs_java=0.873x
old_really_far_native_speedup_vs_java=0.905x
guarded_really_far_native_speedup_vs_java=1.018x
guarded_range_speedup=0.907x
guarded_really_far_speedup=0.907x
equivalence=PASS
bash -n scripts/bench_native_marker_cache.sh: PASS
bash -n scripts/bench_native_waypoint_distance_guard.sh: PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the marker-cache and waypoint-distance guard modules are
parity-clean diagnostic evidence only. The marker cached path does not win in
the native summary bench, and the waypoint guarded Java shapes remain slower,
so neither module is a runtime hook or strict-gate claim.

## Current 2026-05-13 09:01 CEST Rust Remapper and Plugin Directory Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_remapper_index_cleanup.sh
./scripts/bench_native_remapper_skip_hashes.sh
./scripts/bench_native_plugin_directory_scan.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 202 paper-native-core tests passed
release lib hash: e7d8db4f384fb9596bdbd97b40cceffbb26db1b4ac59dc3ea715f75fe7be7c3b
./scripts/bench_native_remapper_index_cleanup.sh: PASS
old_eager_cleanup_native_speedup_vs_java=0.232x
new_lazy_cleanup_native_speedup_vs_java=0.198x
java_cleanup_speedup_vs_old=1.756x
native_cleanup_speedup_vs_old=1.493x
equivalence=PASS
./scripts/bench_native_remapper_skip_hashes.sh: PASS
old_stream_native_speedup_vs_java=2.314x
new_loop_native_speedup_vs_java=2.840x
java_loop_speedup_vs_old=0.979x
native_loop_speedup_vs_old=1.202x
equivalence=PASS
./scripts/bench_native_plugin_directory_scan.sh: PASS
walk_depth1_native_speedup_vs_java=2.267x
list_native_speedup_vs_java=1.296x
directory_stream_native_speedup_vs_java=1.190x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the remapper-index cleanup, remapper skip-hashes, and
plugin-directory scan modules are parity-clean diagnostic evidence only.
They do not install Paper runtime hooks and do not make a strict-gate/server
throughput claim.

## Current 2026-05-13 07:53 CEST Rust Spigot Load-Order and Topographic Sort Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_spigot_load_order_dependency.sh
./scripts/bench_native_topographic_graph_sort_capacity.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 196 paper-native-core tests passed
release lib hash: f6fc23b0617b06cb66174ef671415b1d6b1186eaa6be854d107f943b10c2338b
./scripts/bench_native_spigot_load_order_dependency.sh: PASS
old_load_after_build_native_speedup_vs_java=0.112x
new_load_after_build_native_speedup_vs_java=0.116x
old_removed_count_native_speedup_vs_java=0.243x
new_removed_count_native_speedup_vs_java=2.341x
equivalence=PASS
./scripts/bench_native_topographic_graph_sort_capacity.sh: PASS
old_default_capacity_native_speedup_vs_java=0.700x
new_presized_native_speedup_vs_java=0.514x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new Spigot load-order dependency and topographic sort capacity
modules are parity-clean diagnostic evidence only. The Spigot direct
removed-count native path wins, but the list-copy path does not. The
topographic native paths are slower than Java, so the useful signal remains
container pre-sizing rather than a runtime native hook.

## Current 2026-05-13 07:19 CEST Rust Plugin Loading Allocation and Legacy Alias Removal Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_plugin_loading_allocation.sh
./scripts/bench_native_legacy_provided_alias_removal.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 188 paper-native-core tests passed
release lib hash: 29757d73a12cde363fdae44df09c6a90edbfa2212046ca615dfb5ebbad134407
./scripts/bench_native_plugin_loading_allocation.sh: PASS
old_default_capacity_setup_native_speedup_vs_java=0.303x
new_presized_setup_native_speedup_vs_java=0.518x
old_eager_missing_set_native_speedup_vs_java=0.385x
new_lazy_missing_set_native_speedup_vs_java=0.387x
old_eager_validate_native_speedup_vs_java=0.544x
new_lazy_validate_native_speedup_vs_java=0.543x
equivalence=PASS
./scripts/bench_native_legacy_provided_alias_removal.sh: PASS
old_values_removeif_native_speedup_vs_java=2.130x
new_reverse_alias_remove_native_speedup_vs_java=0.422x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: the new plugin-loading allocation and legacy alias-removal modules
are parity-clean diagnostic evidence only. The allocation model is useful for
the Rust parity layer, but the native JNI path is slower than Java in the
absolute bench. The alias-removal model beats old Java removeIf but loses to
the already optimized Java reverse-index path.

## Current 2026-05-12 20:49 CEST Rust Plugin Classloader-Group Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_plugin_classloader_group.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 180 paper-native-core tests passed
release lib hash: 11dc835730fb924302d621ce89f664c11228addb6ed2ddb1df5a27cd2a4ec001
./scripts/bench_native_plugin_classloader_group.sh: PASS
miss_old_native_speedup_vs_java=3.723x
miss_skip_native_speedup_vs_java=1.393x
hit_other_old_native_speedup_vs_java=2.418x
hit_other_skip_native_speedup_vs_java=0.918x
hit_requester_old_native_speedup_vs_java=1.839x
hit_requester_skip_native_speedup_vs_java=1.314x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `plugin_classloader_group` is parity-clean diagnostic evidence for
plugin classloader-group lookup. Native wins on five of six measured
synthetic shapes, but this still stays diagnostic-only because there is no
Paper runtime hook or strict-gate proof yet.

## Current 2026-05-12 20:19 CEST Rust Plugin Metadata Dependency Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_plugin_meta_dependency.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 176 paper-native-core tests passed
release lib hash: a60ac6075a69d78f701835867a7bef2c6fbfaad57a46ff0ad55662d37e15ad20
./scripts/bench_native_plugin_meta_dependency.sh: PASS
old_stream_native_speedup_vs_java=2.589x
new_loop_native_speedup_vs_java=0.840x
cached_native_speedup_vs_java=0.202x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `plugin_meta_dependency` is parity-clean diagnostic evidence for the
Paper plugin metadata dependency-list shapes. The old stream path wins in
native, but the already optimized Java loop and cached repeated-access paths
remain the better same-runtime result on this host.

## Current 2026-05-12 19:42 CEST Rust Plugin Startup String Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_plugin_name_join.sh
./scripts/bench_native_plugin_name_log.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 172 paper-native-core tests passed
release lib hash: 3f2329fab6581aeba4ecb05ce7a034c9189dda751f31b56133167763f5e37af4
./scripts/bench_native_plugin_name_join.sh: PASS
string_join_normal_native_speedup_vs_java=0.531x
manual_join_normal_native_speedup_vs_java=0.489x
string_join_debug_native_speedup_vs_java=0.356x
manual_join_debug_native_speedup_vs_java=0.723x
equivalence=PASS
./scripts/bench_native_plugin_name_log.sh: PASS
old_treeset_native_speedup_vs_java=0.904x
new_arraylistsort_native_speedup_vs_java=0.343x
java_arraylistsort_speedup_vs_treeset=5.033x
native_arraylistsort_speedup_vs_treeset=1.911x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `plugin_name_join` and `plugin_name_log` are parity-clean diagnostic
modules. They are not runtime promotion candidates on this evidence because
native is slower than Java across the measured string-heavy JNI shapes.

## Current 2026-05-12 18:22 CEST Rust Shift-Noise-Direct Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_shift_noise_direct.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 166 paper-native-core tests passed
release lib hash: eb343a5c3f9d712586fdb7c13319374102c8c2587d2eb1cee5619f3d1f929211
./scripts/bench_native_shift_noise_direct.sh: PASS
current_default_java_best_ms=8.624
current_default_native_best_ms=8.006
current_default_native_speedup_vs_java=1.077x
direct_default_java_best_ms=8.898
direct_default_native_best_ms=8.505
direct_default_native_speedup_vs_java=1.046x
current_a_java_best_ms=8.660
current_a_native_best_ms=7.627
current_a_native_speedup_vs_java=1.135x
direct_a_java_best_ms=8.833
direct_a_native_best_ms=7.677
direct_a_native_speedup_vs_java=1.151x
current_b_java_best_ms=8.537
current_b_native_best_ms=8.056
current_b_native_speedup_vs_java=1.060x
direct_b_java_best_ms=11.097
direct_b_native_best_ms=8.069
direct_b_native_speedup_vs_java=1.375x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `shift_noise_direct` is parity-clean diagnostic evidence with native
wins on all six measured shapes on this host. It is not a runtime promotion.

## Current 2026-05-12 18:55 CEST Rust Entity Bounding-Box Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_entity_bounding_box.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 162 paper-native-core tests passed
release lib hash: 0f4f659fb93d84188839d5093ede4df6299d365b97551754940c3b08fb9453b6
./scripts/bench_native_entity_bounding_box.sh: PASS
old_make_then_set_java_best_ms=1894.853
old_make_then_set_native_best_ms=395.719
old_make_then_set_native_speedup_vs_java=4.788x
old_make_then_set_java_allocated_bytes=1536000000
old_make_then_set_native_allocated_bytes=0
direct_dimensions_set_java_best_ms=813.001
direct_dimensions_set_native_best_ms=406.124
direct_dimensions_set_native_speedup_vs_java=2.002x
direct_dimensions_set_java_allocated_bytes=768000000
direct_dimensions_set_native_allocated_bytes=0
java_direct_speedup_vs_old=2.331x
native_direct_speedup_vs_old=0.974x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `entity_bounding_box` is parity-clean diagnostic evidence with
native wins against the Java sample loops on this host. It is not a runtime
promotion because the previous `Entity.setPosRaw(...)` bounding-box shortcut
was rejected by the runtime gate and rolled back.

## Current 2026-05-12 18:10 CEST Rust EntityLookup Status Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_entity_lookup_status.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 159 paper-native-core tests passed
release lib hash: e833388f354c08171d196a0ce31e348c8b416fab83088bd17423096aa50432be
./scripts/bench_native_entity_lookup_status.sh: PASS
old_status_java_best_ms=579.948
old_status_native_best_ms=251.147
old_status_native_speedup_vs_java=2.309x
direct_status_java_best_ms=588.884
direct_status_native_best_ms=251.209
direct_status_native_speedup_vs_java=2.344x
old_accessible_java_best_ms=721.905
old_accessible_native_best_ms=258.580
old_accessible_native_speedup_vs_java=2.792x
direct_accessible_java_best_ms=685.592
direct_accessible_native_best_ms=258.715
direct_accessible_native_speedup_vs_java=2.650x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `entity_lookup_status` is parity-clean diagnostic evidence with
native wins on this host. It is not a runtime promotion because the previous
EntityLookup movement-path candidates were rejected by real load testing.

## Current 2026-05-12 17:26 CEST Rust Chunk Dependencies and Ownable Rule Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_chunk_dependencies_array.sh
./scripts/bench_native_ownable_rule.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 155 paper-native-core tests passed
release lib hash: 4343fb74925b96106c7c4025a4efc537e4b77988aa8f8f4b7353d79d264609a3
./scripts/bench_native_chunk_dependencies_array.sh: PASS
old_java_best_ms=791.860
old_native_best_ms=477.905
old_native_speedup_vs_java=1.657x
array_java_best_ms=794.043
array_native_best_ms=482.147
array_native_speedup_vs_java=1.647x
equivalence=PASS
./scripts/bench_native_ownable_rule.sh: PASS
old_stream_java_best_ms=1711.676
old_stream_native_best_ms=314.278
old_stream_native_speedup_vs_java=5.446x
new_loop_java_best_ms=626.597
new_loop_native_best_ms=254.995
new_loop_native_speedup_vs_java=2.457x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `chunk_dependencies` and `ownable_rule` are parity-clean diagnostic
modules with measured native wins on this host. They remain outside the Paper
runtime path until a guarded hook, fallback, plugin matrix, and strict load
gate prove that the wins survive real server execution.

## Current 2026-05-12 16:50 CEST Rust Density AP2 Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_density_ap2_fill.sh
./scripts/bench_native_density_ap2_minmax_fill.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 147 paper-native-core tests passed
release lib hash: 0c6dc16633265dec999545746bc54b2c47a68cc746fcbb075ffba8ec802f2bd7
./scripts/bench_native_density_ap2_fill.sh: PASS
old_flat_native_speedup_vs_java=1.374x
scratch_flat_native_speedup_vs_java=0.576x
old_nested_native_speedup_vs_java=1.600x
scratch_nested_native_speedup_vs_java=1.002x
equivalence=PASS
reentrant_equivalence=PASS
./scripts/bench_native_density_ap2_minmax_fill.sh: PASS
min_returns_first_native_old_speedup_vs_java=0.464x
min_returns_second_native_old_speedup_vs_java=1.735x
max_returns_first_native_old_speedup_vs_java=1.701x
max_returns_second_native_old_speedup_vs_java=2.191x
min_overlap_native_old_speedup_vs_java=2.407x
max_overlap_native_old_speedup_vs_java=1.628x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

## Current 2026-05-12 16:12 CEST Rust NoiseChunk Interpolator Array and Flat-Cache Context Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
chmod +x scripts/bench_native_noisechunk_flatcache_context.sh scripts/bench_native_noisechunk_interpolator_array.sh
./scripts/bench_native_noisechunk_interpolator_array.sh
./scripts/bench_native_noisechunk_flatcache_context.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 137 paper-native-core tests passed
release lib hash: 977fe4a649cbf5a4eb14bb7889b48e3a3a48a9767a26211df1b3301567730a70
./scripts/bench_native_noisechunk_interpolator_array.sh: PASS
list_java_best_ms=1174.474
list_native_best_ms=695.013
list_native_speedup_vs_java=1.690x
indexed_list_java_best_ms=1069.111
indexed_list_native_best_ms=686.470
indexed_list_native_speedup_vs_java=1.557x
array_java_best_ms=1145.747
array_native_best_ms=731.872
array_native_speedup_vs_java=1.566x
equivalence=PASS
./scripts/bench_native_noisechunk_flatcache_context.sh: PASS
old_false_context_java_best_ms=108.479
old_false_context_native_best_ms=137.700
old_false_context_native_speedup_vs_java=0.788x
new_false_context_java_best_ms=89.412
new_false_context_native_best_ms=104.712
new_false_context_native_speedup_vs_java=0.854x
old_true_context_java_best_ms=1.038
old_true_context_native_best_ms=1.074
old_true_context_native_speedup_vs_java=0.967x
new_true_context_java_best_ms=1.006
new_true_context_native_best_ms=1.083
new_true_context_native_speedup_vs_java=0.929x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `noisechunk_interpolator_array` is parity-clean and native wins on
all three measured shapes on this host. `noisechunk_flatcache_context` is
parity-clean too, but native remains slower on every measured shape, and it
does not revive the previously rejected `NoiseChunk.FlatCache` runtime
candidate.

## Current 2026-05-12 15:37 CEST Rust NoiseChunk Slice and BlendCache Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
chmod +x scripts/bench_native_noise_interpolator_slice.sh scripts/bench_native_noisechunk_blendcache.sh
./scripts/bench_native_noisechunk_blendcache.sh
./scripts/bench_native_noise_interpolator_slice.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 129 paper-native-core tests passed
release lib hash: d0ef661f50f80b33ed1dad953a735213ca201930c701b89211e80a3ebfa70c5f
./scripts/bench_native_noisechunk_blendcache.sh: PASS
old_empty_blender_java_best_ms=417.205
old_empty_blender_native_best_ms=739.598
old_empty_blender_native_speedup_vs_java=0.564x
new_empty_blender_java_best_ms=10.404
new_empty_blender_native_best_ms=5.234
new_empty_blender_native_speedup_vs_java=1.988x
equivalence=PASS
./scripts/bench_native_noise_interpolator_slice.sh: PASS
old_jagged_java_best_ms=279.685
old_jagged_native_best_ms=415.066
old_jagged_native_speedup_vs_java=0.674x
flat_java_best_ms=304.545
flat_native_best_ms=261.091
flat_native_speedup_vs_java=1.166x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `noisechunk_blendcache` and `noise_interpolator_slice` are
parity-clean diagnostic modules. They do not restore the previously rejected
Paper runtime candidates; either runtime path would still need guarded
fallbacks and strict-gate proof.

## Current 2026-05-12 15:21 CEST Rust NoiseInterpolatorFractions Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
chmod +x scripts/bench_native_noise_interpolator_fractions.sh
./scripts/bench_native_noise_interpolator_fractions.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 122 paper-native-core tests passed
release lib hash: 54a3fe4da7460f8ebb194e0c424b9bcb216f5639d8ee6910a764de9c7db4704a
./scripts/bench_native_noise_interpolator_fractions.sh: PASS
division_java_best_ms=17.238
division_native_best_ms=12.280
division_native_speedup_vs_java=1.404x
array_fraction_java_best_ms=11.919
array_fraction_native_best_ms=11.437
array_fraction_native_speedup_vs_java=1.042x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `noise_interpolator_fractions` is parity-clean and faster on both
shapes on this host, but it still stays diagnostic-only until there is a
guarded runtime hook and strict-gate proof.

## Current 2026-05-12 15:00 CEST Rust Carver Iteration and CaveCarverSkip Native Batch

Fresh commands from this continuation:

```bash
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
chmod +x scripts/bench_native_cave_carver_skip.sh
./scripts/bench_native_carver_iteration.sh
./scripts/bench_native_cave_carver_skip.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 118 paper-native-core tests passed
release lib hash: 61d13f8c42fd79e2a038d1d6c83092e593db624cecae6a9533e5a64f91206cd7
./scripts/bench_native_carver_iteration.sh: PASS
foreach_java_best_ms=133.704
foreach_native_best_ms=64.958
foreach_native_speedup_vs_java=2.058x
indexed_java_best_ms=89.380
indexed_native_best_ms=76.765
indexed_native_speedup_vs_java=1.164x
equivalence=PASS
./scripts/bench_native_cave_carver_skip.sh: PASS
old_java_best_ms=61.044
old_native_best_ms=83.470
old_native_speedup_vs_java=0.731x
reused_checker_java_best_ms=58.211
reused_checker_native_best_ms=89.915
reused_checker_native_speedup_vs_java=0.647x
direct_helper_java_best_ms=58.899
direct_helper_native_best_ms=80.163
direct_helper_native_speedup_vs_java=0.735x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `carver_iteration` is parity-clean and faster on this host, but the
native indexed shape is still slower than native foreach. `cave_carver_skip`
is parity-clean but JNI overhead makes every native shape slower than Java
here, so both remain diagnostic-only.

## Current 2026-05-12 14:34 CEST Rust ServerEntity Delta Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core serverentity_delta_identity -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
chmod +x scripts/bench_native_serverentity_delta_identity.sh
./scripts/build_native.sh
./scripts/bench_native_serverentity_delta_identity.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core serverentity_delta_identity tests: PASS, 4 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 110 paper-native-core tests passed
release lib hash: b90af35e0fc10c996d93dcbc70f5f8bb709ed01127217d49ff2e5b271718fabe
./scripts/bench_native_serverentity_delta_identity.sh: PASS
old_java_best_ms=193.459
old_native_best_ms=151.916
old_native_speedup_vs_java=1.273x
identity_guard_java_best_ms=110.046
identity_guard_native_best_ms=116.559
identity_guard_native_speedup_vs_java=0.944x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `serverentity_delta_identity` is parity-clean as a diagnostic
module. It does not replace the existing Java identity guard with JNI because
the already guarded Java path is faster than the native guard summary on this
host.

## Current 2026-05-12 14:13 CEST Rust StaticCache2D Diagnostic Batch

Fresh commands from this continuation:

```bash
chmod +x scripts/bench_native_static_cache_get.sh
cargo test --manifest-path native/Cargo.toml -p paper-native-core static_cache_get -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_static_cache_get.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core static_cache_get tests: PASS, 4 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 106 paper-native-core tests passed
release lib hash: 308b95ddd630b577f13804e636e04370fb9b82fc00f420b200bfc2ccf5fcaab2
./scripts/bench_native_static_cache_get.sh: PASS
old_java_best_ms=733.176
old_native_best_ms=944.437
old_native_speedup_vs_java=0.776x
new_java_best_ms=693.851
new_native_best_ms=864.624
new_native_speedup_vs_java=0.802x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `static_cache_get` is parity-clean but slower on this host, so it
stays diagnostic-only. It does not restore the previously rejected
single-offset `StaticCache2D.get(...)` runtime shape.

## Current 2026-05-12 13:56 CEST Rust CubicSpline Create Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core cubic_spline_create -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_cubic_spline_create.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core cubic_spline_create tests: PASS, 4 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 102 paper-native-core tests passed
release lib hash: 2b98c1b7e0f86dd3c00e289f04b2e0cb02fc2fdb5a1318027a175f51b955226d
./scripts/bench_native_cubic_spline_create.sh: PASS
iterator_java_best_ms=120.308
iterator_native_best_ms=86.421
iterator_native_speedup_vs_java=1.392x
index_java_best_ms=114.063
index_native_best_ms=80.360
index_native_speedup_vs_java=1.419x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `cubic_spline_create` is parity-clean and faster on this host, but
still diagnostic-only. It does not restore the previously rejected
`CubicSpline.Multipoint.mapAll` runtime cleanup.

## Current 2026-05-12 13:38 CEST Rust Jigsaw canAttach Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core jigsaw_canattach -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_jigsaw_canattach.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core jigsaw_canattach tests: PASS, 4 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 98 paper-native-core tests passed
release lib hash: d9f6e2f53043aba95b690bb8119b257d99188537975c904106b44ff56cc1134b
./scripts/bench_native_jigsaw_canattach.sh: PASS
old_can_attach_java_best_ms=1144.244
old_can_attach_native_best_ms=36.889
old_can_attach_native_speedup_vs_java=31.019x
optimized_can_attach_java_best_ms=1039.042
optimized_can_attach_native_best_ms=31.782
optimized_can_attach_native_speedup_vs_java=32.693x
target_first_can_attach_java_best_ms=294.473
target_first_can_attach_native_best_ms=27.068
target_first_can_attach_native_speedup_vs_java=10.879x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `jigsaw_canattach` is parity-clean and much faster on this host, but
still diagnostic-only. It does not restore the previously rejected
target-first Paper runtime patch.

## Current 2026-05-12 13:11 CEST Rust SpringFeature Mutable-Pos Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core spring_feature_mutable_pos -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_spring_feature_mutable_pos.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core spring_feature_mutable_pos tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 94 paper-native-core tests passed
release lib hash: 3dee7f0b5ab5857f427a9787cfa8030bf8bfb4c5363137f1b18d56506469ddcd
./scripts/bench_native_spring_feature_mutable_pos.sh: PASS
java_old_best_ms=744.758
native_old_best_ms=410.222
native_old_speedup_vs_java=1.816x
java_mutable_best_ms=714.250
native_mutable_best_ms=467.562
native_mutable_speedup_vs_java=1.528x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `spring_feature_mutable_pos` is parity-clean and faster on this host,
but still diagnostic-only until a guarded Paper runtime hook and strict gate
exist.

## Current 2026-05-12 12:12 CEST Rust Biome getBiome Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core biome_getbiome -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_biome_getbiome.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core biome_getbiome tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 91 paper-native-core tests passed
release lib hash: 1670f19a3692d10e7e3fc2d1378ca26ad78b20c6ee7e25821f52003e87d610bf
./scripts/bench_native_biome_getbiome.sh: PASS
java_current_best_ms=152.722
native_current_best_ms=132.699
native_current_speedup_vs_java=1.151x
java_optimized_best_ms=194.038
native_optimized_best_ms=170.491
native_optimized_speedup_vs_java=1.138x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `biome_getbiome` is parity-clean and faster on this host, but still
diagnostic-only until a guarded Paper runtime hook and strict gate exist.

## Current 2026-05-12 11:30 CEST Rust Beardifier Bury Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core beardifier_bury -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_beardifier_bury.sh
sha256sum -c reports/paper-native-jni.sha256
git diff --check
```

Outcome:

```text
paper-native-core beardifier_bury tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 88 paper-native-core tests passed
release lib hash: 4ca057cdb52b94e90e0946965629b8927c4c7529b49b11cf385941e8a274d1c6
./scripts/bench_native_beardifier_bury.sh: PASS
java_current_best_ms=16.415
native_current_best_ms=46.555
native_current_speedup_vs_java=0.353x
java_optimized_best_ms=12.785
native_optimized_best_ms=47.140
native_optimized_speedup_vs_java=0.271x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
git diff --check: PASS
```

Verdict: `beardifier_bury` is parity-clean, but native loses clearly on this
host. Keep it diagnostic-only.

## Current 2026-05-12 11:09 CEST Rust YClampedGradient Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core yclamped_gradient -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_yclamped_gradient.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core yclamped_gradient tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 85 paper-native-core tests passed
release lib hash: 5096c2840f2e488585f0958c61ae54d87e6d910ce3f3ea9d8034ecdcdce55179
./scripts/bench_native_yclamped_gradient.sh: PASS
java_current_best_ms=27.653
native_current_best_ms=60.910
native_current_speedup_vs_java=0.454x
java_optimized_best_ms=27.587
native_optimized_best_ms=63.403
native_optimized_speedup_vs_java=0.435x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `yclamped_gradient` is parity-clean, but native loses clearly on this
host. Keep it diagnostic-only.

## Current 2026-05-12 10:38 CEST Rust Positional Xoroshiro Diagnostic Batches

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core xoroshiro_positional_direct -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_xoroshiro_positional_direct.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_positional_location.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core xoroshiro_positional_direct tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 82 paper-native-core tests passed
release lib hash: 8f8ee85147e142f08cc2284db153ad72ad271519609393b325ee3ec9968bffae
./scripts/bench_native_xoroshiro_positional_direct.sh: PASS
java_old_float_best_ms=30.232
native_old_float_best_ms=11.511
native_old_float_speedup_vs_java=2.626x
java_direct_float_best_ms=16.653
native_direct_float_best_ms=11.612
native_direct_float_speedup_vs_java=1.434x
java_old_double_best_ms=27.598
native_old_double_best_ms=10.119
native_old_double_speedup_vs_java=2.727x
java_direct_double_best_ms=13.453
native_direct_double_best_ms=10.273
native_direct_double_speedup_vs_java=1.310x
equivalence=PASS
./scripts/bench_native_aquifer_positional_location.sh: PASS
java_old_best_ms=27.402
native_old_best_ms=18.813
native_old_speedup_vs_java=1.456x
java_direct_best_ms=17.361
native_direct_best_ms=17.858
native_direct_speedup_vs_java=0.972x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: the positional Xoroshiro diagnostic batch is parity-clean and faster
on every measured shape, and the aquifer positional-location rerun stays
parity-clean with the old path faster and the direct path slightly slower on
this host.

## Current 2026-05-12 09:39 CEST Rust Aquifer Diagnostic Batches

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core aquifer_index_stride -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_index_stride.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_surface_sampling.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core aquifer_index_stride tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 75 paper-native-core tests passed
release lib hash: 967fb6081256b5832417384b4d82106ba6270db19be67caf7b707757d9b3666a
./scripts/bench_native_aquifer_index_stride.sh: PASS
java_old_best_ms=288.438
native_old_best_ms=263.596
native_old_speedup_vs_java=1.094x
java_new_best_ms=319.463
native_new_best_ms=263.117
native_new_speedup_vs_java=1.214x
equivalence=PASS
./scripts/bench_native_aquifer_surface_sampling.sh: PASS
java_old_best_ms=295.584
native_old_best_ms=275.199
native_old_speedup_vs_java=1.074x
java_new_best_ms=272.365
native_new_best_ms=230.479
native_new_speedup_vs_java=1.182x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: both Aquifer diagnostic batch paths are parity-clean on the current
release library and both are faster than Java on this host.

## Current 2026-05-12 08:58 CEST Rust Blended Noise Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core blended_noise -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_blended_noise.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core blended_noise tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 69 paper-native-core tests passed
release lib hash: d90fb6440162af0e5d1199ecc6a70ecfbaf69799f2d7ee980abe9ba22f153d47
./scripts/bench_native_blended_noise.sh: PASS
java_old_best_ms=629.502
native_old_best_ms=760.718
native_old_speedup_vs_java=0.828x
java_cached_best_ms=687.385
native_cached_best_ms=795.017
native_cached_speedup_vs_java=0.865x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `blended_noise` is parity-clean as a diagnostic batch path, but both
measured native shapes are slower than Java on this host. It is not wired
into Paper runtime yet.

## Current 2026-05-12 08:44 CEST Rust Perlin Noise Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core perlin_noise -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_perlin_noise.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core perlin_noise tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 66 paper-native-core tests passed
release lib hash: dfe1214b4360023bc708498be34b9831e9f4ff433c781d0ab03f842f3837d179
./scripts/bench_native_perlin_noise.sh: PASS
java_best_ms=307.791
native_best_ms=290.257
native_speedup_vs_java=1.060x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `perlin_noise` is parity-clean as a diagnostic batch path and is the
next Rust worldgen checkpoint after `improved_noise`. It is not wired into
Paper runtime yet.

## Current 2026-05-12 08:30 CEST Rust Improved Noise Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core improved_noise -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_improved_noise.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core improved_noise tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 63 paper-native-core tests passed
release lib hash: 4ec4a6df1a017094d1305cb466fd8be8e0295bdaed98f3f7377951c30addf6fd
./scripts/bench_native_improved_noise.sh: PASS
java_best_ms=42.014
native_best_ms=38.572
native_speedup_vs_java=1.089x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `improved_noise` is parity-clean as a diagnostic batch path and is
the first new Rust worldgen checkpoint in this sequence that beats Java on
this host. It is not wired into Paper runtime yet.

## Current 2026-05-12 08:14 CEST Rust Chunk Ticket Stage Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core chunk_ticket_stage -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_chunk_ticket_stage.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core chunk_ticket_stage tests: PASS, 3 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 60 paper-native-core tests passed
release lib hash: 02505c1c4b4f81727aaae0569920098266da065857543ac3c2b25296b666d9d8
./scripts/bench_native_chunk_ticket_stage.sh: PASS
java_best_ms=199.714
native_best_ms=262.183
native_speedup_vs_java=0.762x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `chunk_ticket_stage` is parity-clean as a diagnostic batch path. It
is not wired into Paper runtime because the native summary path is slower than
the Java summary on this host.

## Current 2026-05-12 08:00 CEST Rust Ticket Compare Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core ticket_compare -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_ticket_compare.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core ticket_compare tests: PASS, 4 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 57 paper-native-core tests passed
release lib hash: 13ad976699dfc1595be82453dd5cbd2d7d33f3491064f19e2577784a8577ca13
./scripts/bench_native_ticket_compare.sh: PASS
java_best_ms=190.711
native_best_ms=222.437
native_speedup_vs_java=0.857x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `ticket_compare` is parity-clean as a diagnostic batch path. It is
not wired into Paper runtime because the native summary path is slower than
the Java summary on this host.

## Current 2026-05-12 07:45 CEST Rust Ticket Pack Diagnostic Batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core ticket_pack -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_ticket_pack.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core ticket_pack tests: PASS, 4 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 53 paper-native-core tests passed
release lib hash: 48bbc163dcd98d80d489c107ec3d4f950fbc4a0e4b43dbca10a1a3acb25aad68
./scripts/bench_native_ticket_pack.sh: PASS
java_best_ms=588.246
native_best_ms=621.271
native_speedup_vs_java=0.947x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: `ticket_pack` is parity-clean as a diagnostic batch path. It is not
wired into Paper runtime, and the native summary path is slightly slower than
the Java summary on this host.

## Current 2026-05-12 07:14 CEST Rust ReferenceList diagnostic batch

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core reference_list -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_reference_list.sh
sha256sum -c reports/paper-native-jni.sha256
MC_EULA_AGREE=true JAVA_OPTS='-Xms1G -Xmx2G -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true -Dpaper.nativeAreaMap=true' ./scripts/run_plugin_matrix.sh
```

Outcome:

```text
paper-native-core reference_list tests: PASS, 8 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 49 paper-native-core tests passed
release lib hash: 560502213ba93723d279eedd2083eda0ddf95a000957501ebe8b2654f099e61c
./scripts/bench_native_reference_list.sh: PASS
transition_java_best_ms=120.989
transition_native_best_ms=64.468
transition_native_batch_speedup_vs_java=1.877x
dense_java_best_ms=110.041
dense_native_best_ms=71.630
dense_native_batch_speedup_vs_java=1.536x
random_java_best_ms=132.918
random_native_best_ms=78.224
random_native_batch_speedup_vs_java=1.699x
equivalence=PASS
sha256sum -c reports/paper-native-jni.sha256: PASS
run_plugin_matrix.sh with paper.nativeClimateRTree=true and paper.nativeAreaMap=true: PASS, Done (24.251s), initialized 11 plugins, join probe PASS
```

Verdict: the new `reference_list` Rust module is parity-clean as a diagnostic
batch path. It is not wired into Paper runtime yet, and the existing native
runtime hooks still pass the plugin matrix with the new native library.

## Current 2026-05-12 06:41 CEST Rust area-map guarded runtime hook

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-core --lib area_map -- --nocapture
cargo check --manifest-path native/Cargo.toml -p paper-native-jni
./scripts/build_native.sh
./scripts/bench_native_area_map.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true JAVA_OPTS='-Xms1G -Xmx2G -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' ./scripts/run_plugin_matrix.sh
MC_EULA_AGREE=true JAVA_OPTS='-Xms1G -Xmx2G -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true -Dpaper.nativeAreaMap=true' ./scripts/run_plugin_matrix.sh
sha256sum -c reports/paper-native-jni.sha256
```

Outcome:

```text
paper-native-core area_map tests: PASS, 7 passed
paper-native-jni cargo check: PASS
./scripts/build_native.sh: PASS, 41 paper-native-core tests passed
release lib hash: 01011d979da30a313e6e6a85dcc29f631bab92f3384a1a4eeb2a0895ddd3b439
./scripts/bench_native_area_map.sh: PASS
java_best_ms=525.214
native_best_ms=419.014
native_speedup_vs_java=1.253x
equivalence=PASS
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
run_plugin_matrix.sh with paper.nativeClimateRTree=true: PASS, Done (23.071s), join probe PASS
run_plugin_matrix.sh with paper.nativeClimateRTree=true and paper.nativeAreaMap=true: PASS, Done (27.135s), join probe PASS after rebuildFeaturePatches
sha256sum -c reports/paper-native-jni.sha256: PASS
```

Verdict: the new `area_map` Rust module is parity-clean and measurable, but
the later strict 50-bot gate regressed the accepted baseline and the runtime
hook was rolled back. The module stays diagnostic-only.

## Current 2026-05-12 06:02 CEST Rust Climate RTree 50-bot load diagnostics

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=load-native-climate-50-arcfix LOAD_TEST_GAMEMODE=survival BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=load-native-climate-50-arcfix-io2 LOAD_TEST_GAMEMODE=survival BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 PAPER_CHUNK_IO_THREADS=2 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' ./scripts/run_load_test.sh
```

Outcome:

```text
load-native-climate-50-arcfix: PASS to shutdown, but not a clean gate
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[05:55:19 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 1 I/O threads
watchdog_thread_dumps=4
nearby_players_stack_hits=4
tps1_avg=16.00
avg_tick_ms_avg=82.20

load-native-climate-50-arcfix-io2: PASS to shutdown, but not a clean gate
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=2 prevent_unloaded_move=false
worker_line=[05:59:06 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
watchdog_thread_dumps=3
nearby_players_stack_hits=4
tps1_avg=14.86
avg_tick_ms_avg=132.31
```

Verdict: native climate no longer crashes under 50-bot load, but the current
50-bot gate is still noisy in chunk/ticket/nearby-player paths and should not
be called accepted.

## Current 2026-05-12 05:50 CEST Rust Climate RTree threaded runtime fix

Fresh commands from this continuation:

```bash
cargo test -p paper-native-core --lib climate_rtree -- --nocapture
cargo check -p paper-native-jni
./scripts/build_native.sh
MC_EULA_AGREE=true JAVA_OPTS='-Xms1G -Xmx2G -Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' ./scripts/run_plugin_matrix.sh
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=6 -Drounds=16 -Dqueries=120000 -Dleaves=1400' ./scripts/bench_native_climate_rtree_jni.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
```

Outcome:

```text
cargo test -p paper-native-core --lib climate_rtree: PASS, 8 passed
new shared_handle_search_is_thread_safe regression test: PASS
cargo check -p paper-native-jni: PASS
./scripts/build_native.sh: PASS, paper-native-core tests 34 passed
release lib hash: d45614e0ef385eba2a4ba0436dd7b63d18a718b4a744c43620bf7008b41fd1a7
run_plugin_matrix.sh with paper.nativeClimateRTree=true: PASS, Done (23.699s), join probe PASS
sha256sum -c reports/artifact-hashes.txt: PASS

bench_native_climate_rtree_jni.sh: PASS
java_current_random_best_ms=2442.645
java_bounded_random_best_ms=1913.137
native_current_random_best_ms=606.117
native_bounded_random_best_ms=904.320
java_current_walk_best_ms=605.405
java_bounded_walk_best_ms=450.467
native_current_walk_best_ms=287.360
native_bounded_walk_best_ms=283.348
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS
native_rtree_packed_equivalence=PASS
```

Verdict: the native RTree runtime crash was a shared-handle `Rc` refcount
race under concurrent worldgen. The runtime tree now uses `Arc`, and the
native-enabled server/plugin gate re-passes.

## Current 2026-05-12 05:25 CEST Rust Climate RTree runtime hook verification

Fresh commands from this continuation:

```bash
./gradlew fixupSourcePatches --no-daemon
./gradlew rebuildPatches --no-daemon
./gradlew paper-server:compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
./scripts/build_native.sh
JAVA_TOOL_OPTIONS='-Djava.library.path=/root/rust/native/target/release -Dpaper.nativeClimateRTree=true' jshell --class-path "$(cat artifacts/optimized-runtime/classpath.txt)"
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=6 -Drounds=16 -Dqueries=120000 -Dleaves=1400' ./scripts/bench_native_climate_rtree_jni.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
```

Outcome:

```text
fixupSourcePatches: PASS
rebuildPatches: PASS, rebuilt 912 source patches
paper-server:compileJava: PASS
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
./scripts/build_native.sh: PASS, paper-native-core tests 33 passed
release lib hash: 6fe3c36a7ab9f90873fa7df6fb95153d56a9c5b8143403c1887c8e83634fac54
JShell runtime smoke: nativeHandle=138247548177264, value=A
optimized-paper sha256: e5563244d6420895ef38f5bd931ebc37368f53c8dad5fce26989ca5a7f2d9970
sha256sum -c reports/artifact-hashes.txt: PASS

bench_native_climate_rtree_jni.sh: PASS
java_current_random_best_ms=1833.086
java_bounded_random_best_ms=1641.053
native_current_random_best_ms=589.236
native_bounded_random_best_ms=825.523
native_current_packed_random_best_ms=599.095
native_bounded_packed_random_best_ms=832.820
java_current_walk_best_ms=482.242
java_bounded_walk_best_ms=411.255
native_current_walk_best_ms=276.919
native_bounded_walk_best_ms=236.410
native_current_packed_walk_best_ms=285.102
native_bounded_packed_walk_best_ms=243.205
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS
native_rtree_packed_equivalence=PASS
```

Verdict: the Paper runtime hook is now patch-backed, compiles from patches,
loads the packaged JNI bridge when explicitly enabled, and keeps Java fallback
when native is unavailable.

## Current 2026-05-12 02:55 CEST Rust Climate RTree current-search retry, reject, and revert

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 LEAVES=1400 QUERIES=120000 WARMUP=6 ROUNDS=16 ./scripts/bench_native_climate_rtree_batch_borrow.sh
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=6 -Drounds=16 -Dqueries=120000 -Dleaves=1400' ./scripts/bench_native_climate_rtree_jni.sh
SKIP_NATIVE_BUILD=1 LEAVES=32 QUERIES=64 WARMUP=1 ROUNDS=1 ./scripts/bench_native_climate_rtree_jni.sh
```

Outcome:

```text
cargo test --manifest-path native/Cargo.toml: PASS
paper-native-core tests: 30 passed

./scripts/build_native.sh after revert: PASS
release lib hash: 44d27f3f27cc28daa99cfa09ac14cb1efbfec688d78b54c4b863174e46567f82

bench_native_climate_rtree_batch_borrow.sh on the candidate: PASS, but rejected
cloned_current_random_best_ms=625.279
direct_current_random_best_ms=636.501
borrowed_current_random_best_ms=690.459
cloned_bounded_random_best_ms=833.604
borrowed_bounded_random_best_ms=852.593
cloned_current_walk_best_ms=255.609
cloned_bounded_walk_best_ms=224.109
borrowed_batch_equivalence=PASS

bench_native_climate_rtree_jni.sh on the candidate: PASS, but rejected
native_current_random_best_ms=641.975
native_bounded_random_best_ms=826.041
native_current_walk_best_ms=276.690
native_bounded_walk_best_ms=240.646
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS

bench_native_climate_rtree_jni.sh smoke after revert and rebuild: PASS
leaves=32
queries=64
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS
```

Verdict: the current-search subtree best-distance shortcut is rejected and
reverted. It mixed in small walk/bounded changes, but regressed the current
random path versus the accepted clone-backed baseline, so `search_current_*`
is back on exact-distance child checks.

## Current 2026-05-12 02:42 CEST Rust Climate RTree Java bench env override smoke-tests

Fresh commands from this continuation:

```bash
SKIP_NATIVE_BUILD=1 LEAVES=32 QUERIES=64 WARMUP=1 ROUNDS=1 ./scripts/bench_native_climate_rtree_jni.sh
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dleaves=33 -Dqueries=64 -Dwarmup=1 -Drounds=1' ./scripts/bench_native_climate_rtree_jni.sh
SKIP_NATIVE_BUILD=1 LEAVES=32 ITERATIONS=5 WARMUP=1 ROUNDS=1 ./scripts/bench_native_climate_rtree_build.sh
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dleaves=33 -Diterations=5 -Dwarmup=1 -Drounds=1' ./scripts/bench_native_climate_rtree_build.sh
SKIP_NATIVE_BUILD=1 LEAVES=32 QUERIES=64 WARMUP=1 ROUNDS=1 ./scripts/bench_native_climate_rtree_lifecycle.sh
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dleaves=33 -Dqueries=64 -Dwarmup=1 -Drounds=1' ./scripts/bench_native_climate_rtree_lifecycle.sh
```

Outcome:

```text
bench_native_climate_rtree_jni.sh with env overrides: PASS
command=java -Dleaves=32 -Dqueries=64 -Dwarmup=1 -Drounds=1 ...
leaves=32
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS

bench_native_climate_rtree_jni.sh with JAVA_PROPS: PASS
command=java -Dleaves=33 -Dqueries=64 -Dwarmup=1 -Drounds=1 ...
leaves=33
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS

bench_native_climate_rtree_build.sh with env overrides: PASS
command=java -Dleaves=32 -Diterations=5 -Dwarmup=1 -Drounds=1 ...
leaves=32
optimized_tree_checksum=-261480495027163337
native_tree_checksum=-261480495027163337
equivalence=PASS

bench_native_climate_rtree_build.sh with JAVA_PROPS: PASS
command=java -Dleaves=33 -Diterations=5 -Dwarmup=1 -Drounds=1 ...
leaves=33
optimized_tree_checksum=-3993828528708162323
native_tree_checksum=-3993828528708162323
equivalence=PASS

bench_native_climate_rtree_lifecycle.sh with env overrides: PASS
command=java -Dleaves=32 -Dqueries=64 -Dwarmup=1 -Drounds=1 ...
leaves=32
java_native_lifecycle_equivalence=PASS

bench_native_climate_rtree_lifecycle.sh with JAVA_PROPS: PASS
command=java -Dleaves=33 -Dqueries=64 -Dwarmup=1 -Drounds=1 ...
leaves=33
java_native_lifecycle_equivalence=PASS
```

The RTree Java bench scripts now accept direct env overrides while preserving
the existing `JAVA_PROPS` path. Search and lifecycle take `LEAVES`,
`QUERIES`, `WARMUP`, and `ROUNDS`; build takes `LEAVES`, `ITERATIONS`,
`WARMUP`, and `ROUNDS`.

## Current 2026-05-12 02:16 CEST Rust Climate RTree Mixed Batch Default Diagnostic

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml
SKIP_NATIVE_BUILD=1 LEAVES=1400 QUERIES=120000 WARMUP=6 ROUNDS=16 ./scripts/bench_native_climate_rtree_batch_borrow.sh
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=6 -Drounds=16 -Dqueries=120000' ./scripts/bench_native_climate_rtree_jni.sh
```

Outcome:

```text
cargo test --manifest-path native/Cargo.toml: PASS
paper-native-core tests: 30 passed

bench_native_climate_rtree_batch_borrow.sh: PASS
direct_current_random_speedup_vs_cloned=0.982x
direct_current_walk_speedup_vs_cloned=1.016x
borrowed_current_random_speedup_vs_cloned=0.905x
borrowed_current_walk_speedup_vs_cloned=0.897x
borrowed_bounded_random_speedup_vs_cloned=0.944x
borrowed_bounded_walk_speedup_vs_cloned=0.939x
borrowed_batch_equivalence=PASS

bench_native_climate_rtree_jni.sh: PASS
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS
java_current_random_best_ms=1851.480
java_bounded_random_best_ms=1660.604
native_current_random_best_ms=588.766
native_bounded_random_best_ms=839.889
java_current_walk_best_ms=480.781
java_bounded_walk_best_ms=386.739
native_current_walk_best_ms=270.944
native_bounded_walk_best_ms=230.809
```

Verdict: public current and bounded batch search both use clone-backed paths,
direct-current / borrowed-current / borrowed-bounded stay diagnostic only, the
recursive helpers now carry the known best distance down the search tree and
skip the duplicate leaf exact-distance pass, and the direct-current
specialization still does not win repeatably enough to become default.

## Current 2026-05-12 01:05 CEST Rust Climate RTree Arena Diagnostic

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml
./scripts/build_native.sh
SKIP_NATIVE_BUILD=1 LEAVES=1400 QUERIES=60000 WARMUP=2 ROUNDS=4 ./scripts/bench_native_climate_rtree_arena.sh
```

Outcome:

```text
cargo test --manifest-path native/Cargo.toml: PASS
paper-native-core tests: 29 passed

build_native.sh: PASS
paper-native-core tests: 29 passed
paper-native-jni compile: PASS

bench_native_climate_rtree_arena.sh: PASS
arena_node_count=1682
rc_tree_checksum=1463956120320347328
arena_tree_checksum=1463956120320347328
rc_arena_lifecycle_equivalence=PASS
```

The new arena representation is covered by unit tests and the lifecycle bench,
but it did not beat the existing Rc path on this host.

## Current 2026-05-12 00:52 CEST Rust Climate RTree Native Lifecycle Diagnostic

Fresh commands from this continuation:

```bash
./scripts/build_native.sh
JAVA_PROPS='-Dqueries=60000 -Dwarmup=2 -Drounds=4' ./scripts/bench_native_climate_rtree_lifecycle.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 27 passed
paper-native-jni compile: PASS

bench_native_climate_rtree_lifecycle.sh: PASS
lifecycle_scope=build_search_free
java_tree_checksum=1463956120320347328
native_tree_checksum=1463956120320347328
java_current_random_lifecycle_best_ms=987.637
java_bounded_random_lifecycle_best_ms=881.556
native_current_random_lifecycle_best_ms=317.413
native_bounded_random_lifecycle_best_ms=446.921
java_current_walk_lifecycle_best_ms=269.357
java_bounded_walk_lifecycle_best_ms=214.280
native_current_walk_lifecycle_best_ms=148.501
native_bounded_walk_lifecycle_best_ms=122.337
java_native_lifecycle_equivalence=PASS
```

## Current 2026-05-12 00:39 CEST Rust Climate RTree Native Build Diagnostic

Fresh commands from this continuation:

```bash
./scripts/build_native.sh
JAVA_PROPS='-Diterations=200 -Dwarmup=2 -Drounds=4' SKIP_NATIVE_BUILD=1 ./scripts/bench_native_climate_rtree_build.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 27 passed
paper-native-jni compile: PASS

bench_native_climate_rtree_build.sh: PASS
optimized_tree_checksum=1463956120320347328
native_tree_checksum=1463956120320347328
optimized_loop_build_best_ms=2788.949
native_build_handle_best_ms=960.521
native_build_speedup_vs_java=2.904x
allocation_counter=jvm_thread_allocated_bytes
optimized_jvm_allocated_bytes_per_build=9685848.0
native_jvm_allocated_bytes_per_build=0.0
equivalence=PASS
```

## Current 2026-05-12 00:27 CEST Rust Climate RTree JNI Handle Diagnostic

Fresh commands from this continuation:

```bash
./scripts/build_native.sh
./scripts/bench_climate_rtree_search.sh
./scripts/bench_native_climate_rtree_search.sh
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_climate_rtree_jni.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 27 passed
paper-native-jni compile: PASS

bench_climate_rtree_search.sh: PASS
java_rtree_equivalence=PASS
current_random_best_ms=2010.557
bounded_random_best_ms=1801.047
current_walk_best_ms=542.675
bounded_walk_best_ms=450.691

bench_native_climate_rtree_search.sh: PASS
native_rtree_equivalence=PASS
native_current_random_best_ms=624.181
native_bounded_random_best_ms=852.528
native_current_walk_best_ms=283.886
native_bounded_walk_best_ms=222.712

bench_native_climate_rtree_jni.sh: PASS
native_tree_checksum=1463956120320347328
java_current_random_best_ms=1845.883
java_bounded_random_best_ms=1671.103
native_current_random_best_ms=624.319
native_bounded_random_best_ms=871.594
java_current_walk_best_ms=471.294
java_bounded_walk_best_ms=391.978
native_current_walk_best_ms=287.166
native_bounded_walk_best_ms=236.338
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS
```

## Current 2026-05-11 23:58 CEST Rust Climate RTree Search Diagnostic

Fresh commands from this continuation:

```bash
./scripts/bench_climate_rtree_search.sh
./scripts/build_native.sh
./scripts/bench_native_climate_rtree_search.sh
```

Outcome:

```text
bench_climate_rtree_search.sh: PASS
java_rtree_equivalence=PASS
current_random_best_ms=2062.695
bounded_random_best_ms=1816.289
bounded_random_speedup=1.136x
current_walk_best_ms=578.250
bounded_walk_best_ms=470.331
bounded_walk_speedup=1.229x
input_leaves_checksum=179575258560070041
current_tree_checksum=1463956120320347328
random_queries_checksum=5165014967713273743
walk_queries_checksum=-2288988305868638531
random_checksum=-2174743207420542594
walk_checksum=-6213582386974512796

build_native.sh: PASS
paper-native-core tests: 26 passed
paper-native-jni compile: PASS

bench_native_climate_rtree_search.sh: PASS
native_rtree_equivalence=PASS
native_current_random_best_ms=1069.431
native_bounded_random_best_ms=1087.711
native_bounded_random_speedup=0.983x
native_current_walk_best_ms=266.218
native_bounded_walk_best_ms=250.479
native_bounded_walk_speedup=1.063x
```

## Current 2026-05-11 23:30 CEST Rust Climate Batch Extended With Best-Match

Fresh commands from this continuation:

```bash
./scripts/build_native.sh
./scripts/bench_native_climate.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 24 passed
paper-native-jni compile: PASS
bench_native_climate.sh: PASS
equivalence=PASS
java_node_distance_sum_best_ms=198.545
native_node_distance_sum_best_ms=44.859
native_node_distance_sum_speedup_vs_java=4.426x
java_node_best_match_best_ms=132.167
native_node_best_match_best_ms=95.798
native_node_best_match_speedup_vs_java=1.380x
```

`cargo fmt --all --manifest-path native/Cargo.toml` is still unavailable on
this host because `cargo fmt` / `rustfmt` are not installed.

## Current 2026-05-11 23:16 CEST Rust Climate Batch Module And LZ4 Runtime Rollback

Fresh commands from this continuation:

```bash
./scripts/build_native.sh
./scripts/bench_native_climate.sh
./scripts/bench_lz4_stream.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh artifacts/optimized-runtime/run.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 20 passed
paper-native-jni compile: PASS
bench_native_climate.sh: PASS
equivalence=PASS
java_node_distance_sum_best_ms=213.850
native_node_distance_sum_best_ms=38.319
native_node_distance_sum_speedup_vs_java=5.581x
bench_lz4_stream.sh: PASS
equivalence=PASS
buffered_default_best_ms=3292.509
native_lz4_best_ms=4365.214
native_lz4_speedup=0.754x
build_optimized.sh: PASS
applyPatches: Applied 912 patches
plugin matrix: PASS, Done (26.881s)
restart/recovery: PASS, Done (15.618s)
forced-ticket persistence: PASS, first/restart Done 13.498s/8.683s
```

`cargo fmt --all --manifest-path native/Cargo.toml` was not available on this
host because `cargo fmt` / `rustfmt` are not installed.

## Current 2026-05-11 22:12 CEST Rust Native Compression Backend Selected

Fresh commands from this continuation:

```bash
./scripts/build_native.sh
./scripts/bench_region_compression.sh
./scripts/bench_native_varint.sh
./scripts/bench_native_position.sh
./scripts/bench_native_hash.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 16 passed
paper-native-jni compile: PASS
bench_region_compression.sh: PASS
java/native LZ4 block-stream interop: PASS
java_lz4_best_ms=321.627
native_lz4_best_ms=277.301
native_lz4_ratio=0.9877
bench_native_varint.sh: PASS
equivalence=PASS
java_write_best_ms=4.966
native_write_best_ms=11.767
java_size_best_ms=3.644
native_size_best_ms=12.588
bench_native_position.sh: PASS
equivalence=PASS
java_combined_best_ms=4.296
native_combined_best_ms=35.467
bench_native_hash.sh: PASS
equivalence=PASS
java_sha256_best_ms=92.496
native_sha256_best_ms=145.949
```

The native compression backend now matches Java's compressed size and wins on
this host, but the other JNI parity paths still trail Java and remain
diagnostic only.

## Current 2026-05-11 19:06 CEST Rust Native Checkpoint

Fresh commands from this continuation:

```bash
cargo update -p jni --precise 0.19.0
cargo update -p jni-sys:0.3.1 --precise 0.3.0
./scripts/build_native.sh
./scripts/bench_native_varint.sh
./scripts/bench_native_position.sh
```

Outcome:

```text
build_native.sh: PASS
paper-native-core tests: 7 passed
paper-native-jni compile: PASS
bench_native_varint.sh: PASS
equivalence=PASS
java_write_best_ms=4.138
native_write_best_ms=12.337
java_size_best_ms=4.172
native_size_best_ms=12.438
bench_native_position.sh: PASS
equivalence=PASS
java_chunk_pack_best_ms=1.685
native_chunk_pack_best_ms=7.654
java_chunk_hash_best_ms=1.013
native_chunk_hash_best_ms=5.152
java_section_pack_best_ms=1.894
native_section_pack_best_ms=11.917
java_combined_best_ms=3.959
native_combined_best_ms=31.251
```

The Rust migration step is valid, but the current JNI bridge is still too
expensive to justify hooking either module into Paper runtime.

## Current 2026-05-10 16:30 CEST NoiseChunk Wrapped Capacity Rejection And Rollback Verification

Fresh commands from this continuation:

```bash
JAVA_PROPS='-DmapBenchIterations=200 -Dwarmup=2 -Drounds=4' ./scripts/bench_noisechunk_wrap_size.sh
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json >/dev/null
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-noisechunk-wrap-capacity-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json >/dev/null
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh artifacts/optimized-runtime/run.sh
```

Outcome:

```text
diagnostic bench: PASS
overworld-like NoiseChunk.wrapped final size: 9361 entries, n=16384
small-settings final size: nether/caves/floating_islands=52, end=41
synthetic expected_8192_075 map path: 4.216x vs current_2048_075
temporary candidate: 0051 Optimize NoiseChunk wrapped map capacity
candidate build/hash/json: PASS
candidate plugin matrix: PASS, Done (29.070s)
candidate restart/recovery: PASS, Done (16.856s)
candidate forced-ticket persistence: PASS, first/restart Done 13.765s/9.690s
strict 50-bot preflight: PASS, load_per_cpu=0.508, idle_percent_1s=79.03
strict 50-bot result: REJECTED, 17.74 TPS / 84.37 ms / 2557 chunks, watchdog_thread_dumps=4, nearby_players_stack_hits=8, stability_failures=0
rollback: 0051 patch removed, applyPatches Applied 912 patches
rollback optimized artifact sha256=fb7b7e335f8660829d06b177d8ac20a06ffd52cfa2fe5d10a44f5b9a3fe50dca
rollback app-cds sha256=c1acf8627ee17eac6b55fa71d3ad089a340d107bc9857a21e64ab3438b51b037
rollback hash/json: PASS
rollback NoiseChunk.wrapped: Reference2ReferenceOpenHashMap<>(2048)
rollback plugin matrix: PASS, Done (27.420s)
rollback restart/recovery: PASS, Done (19.327s)
rollback forced-ticket persistence: PASS, first/restart Done 15.171s/9.479s
```

The current runtime is back on the `0050` patch-stack state. The diagnostic
bench remains for future capacity analysis, but no production capacity change
is accepted from this cycle.

## Current 2026-05-10 15:54 CEST Player Loader Candidate Rejection And Rollback Verification

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json >/dev/null
sha256sum -c reports/artifact-hashes.txt
javap -classpath artifacts/optimized-runtime/bundler/versions/1.21.10/paper-1.21.10.jar -c -p -l 'ca.spottedleaf.moonrise.patches.chunk_system.player.RegionizedPlayerChunkLoader$PlayerChunkLoaderData'
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-playerloader-unused-manhattan-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json >/dev/null
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh artifacts/optimized-runtime/run.sh
```

Outcome:

```text
temporary candidate: 0051 Remove unused player loader distance calculation
candidate build/hash/json: PASS
candidate bytecode: update() loop had squareDistance only; no local manhattanDistance
candidate plugin matrix: PASS, Done (28.675s)
candidate restart/recovery: PASS, Done (19.867s)
candidate forced-ticket persistence: PASS, first/restart Done 15.647s/9.855s
strict 50-bot preflight: PASS, load_per_cpu=0.545, idle_percent_1s=77.83
strict 50-bot result: REJECTED, 17.17 TPS / 52.33 ms / 2633 chunks, watchdog_thread_dumps=4, nearby_players_stack_hits=2, stability_failures=0
rollback: 0051 patch removed, applyPatches Applied 912 patches
rollback optimized artifact sha256=207d1b54cd81908c184e72b5435aa50b9c8eaf10c5df3836c1284ed8a388d2a4
rollback hash/json: PASS
rollback plugin matrix: PASS, Done (28.309s)
rollback restart/recovery: PASS, Done (18.328s)
rollback forced-ticket persistence: PASS, first/restart Done 13.538s/9.224s
```

The current runtime is back on the `0050` patch-stack state. No 20 TPS or
end-to-end load improvement claim is made.

## Current 2026-05-10 14:20 CEST Focused Rejection Verification

Fresh commands from this continuation:

```bash
bash scripts/bench_density_ap2_minmax_fill.sh
python3 - <<'PY'
# structured JSON graph scan over vanilla density_function and noise_settings
# resources; see reports/density-ap2-minmax-graph-scan.txt for the captured
# result summary.
PY
```

Outcome:

```text
DensityFunctions.Ap2 MIN/MAX non-overlap candidate: REJECTED before production
microbench non-overlap: 2.460x to 8.597x faster, equivalence=PASS
microbench overlap: min 0.948x, max 0.984x
vanilla graph scan: minmax_nodes=22, branch_counts=overlap:22, fastpath_candidates=0
production source diff for DensityFunctions.java: empty
```

No rebuild or load test was run for this rejected candidate because no
production source patch was made.

## Current 2026-05-10 13:15 CEST Patch stack restore and load preflight

Fresh commands from this continuation:

```bash
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-current-after-patchstack-fix LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
applyPatches: PASS, Applied 912 patches
build_optimized.sh: PASS
artifacts hash/json: PASS
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.846 > max_load_per_cpu=0.750, idle_percent_1s=55.94
```

No end-to-end TPS/MSPT measurement was taken because the host was too busy
for a trustworthy comparable run.

## Current 2026-05-10 12:39 CEST Verification

Fresh commands from this continuation:

```bash
bash scripts/bench_plugin_classloader_group.sh
cd upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=plugin-classloader-group-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
plugin-classloader-group bench: miss 1.084x, other-loader hit 1.257x, requester-hit 0.825x, equivalence=PASS
build_optimized.sh: PASS
artifacts hash/json: PASS
plugin matrix: PASS, 11 real plugins initialized
restart/recovery: PASS
forced-ticket persistence: PASS
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.540 > max_load_per_cpu=0.750, idle_percent_1s=28.08 < min_idle_percent=40.00
```

The candidate is accepted only as a classloader lookup reduction. It is not
an end-to-end startup, TPS, MSPT, or 500-player result.

## Current 2026-05-10 12:34 CEST Focused Rejection Verification

Fresh commands:

```bash
./scripts/bench_static_cache_get.sh
./scripts/bench_reference_list_transition_add.sh
```

Outcome:

```text
StaticCache2D get candidate: REJECTED before production, old 408.902 ms, candidate 427.130 ms, speedup 0.957x, equivalence=PASS
ReferenceList add transition candidate: REJECTED before production, transition speedup 0.799x, pair speedup 1.005x, dense speedup 0.981x, equivalence=PASS
```

No source patch was promoted from these measurements.

## Current 2026-05-10 12:13 CEST Verification

Fresh commands from this continuation:

```bash
bash scripts/bench_surfacerules_testrule_state.sh
cd upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=surfacerules-state-test-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
SurfaceRules state-test candidate: ACCEPTED WITH LIMITS
microbench: mostly_true 51.347 ms -> 50.095 ms, 1.025x; mostly_false 49.377 ms -> 47.994 ms, 1.029x; equivalence=PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (34.742s), 11 real plugins initialized
restart/recovery: PASS, Done (19.091s), Saved the game
forced-ticket persistence: PASS, first/restart Done 16.112s/11.165s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.985 > max_load_per_cpu=0.750
```

The candidate remains a surface-rule dispatch optimization only. It is not an
end-to-end startup, TPS, MSPT, or 500-player result.

## Current 2026-05-10 11:48 CEST Verification

Fresh commands from this continuation:

```bash
bash scripts/bench_chunk_expire_count.sh
cd upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=chunk-expire-lookup-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
ChunkHolderManager addExpireCount lookup candidate: ACCEPTED WITH LIMITS
microbench: dynamic_compute_hot_best_ms=333.257, dynamic_manual_hot_best_ms=277.137, speedup=1.203x; dynamic_compute_cold_best_ms=0.566, dynamic_manual_cold_best_ms=0.478, speedup=1.182x; equivalence=PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (53.157s), 11 real plugins initialized
restart/recovery: PASS, Done (43.350s), Saved the game
forced-ticket persistence: PASS, first/restart Done 28.668s/22.220s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.809 > max_load_per_cpu=0.750, idle_percent_1s=13.93 < min_idle_percent=40.00
```

The candidate remains a ticket expire-count map lookup optimization only. It
is not an end-to-end startup, TPS, MSPT, or 500-player result.

## Current 2026-05-10 11:39 CEST Verification

Fresh commands from this continuation:

```bash
bash scripts/bench_compression_deflater_input.sh
cd upstream/Paper && ./gradlew applyPatches --no-daemon
cd upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-compression-deflater-bytebuffer-gate-20260510-v2 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
CompressionEncoder deflater-input candidate: ACCEPTED WITH LIMITS
microbench: heap 137.266 ms -> 131.327 ms, 1.045x; direct 129.531 ms -> 124.865 ms, 1.037x; equivalence=PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (51.284s), 11 real plugins initialized
restart/recovery: PASS, Done (24.522s), Saved the game
forced-ticket persistence: PASS, first/restart Done 20.692s/17.519s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.160 > max_load_per_cpu=0.750
```

The candidate remains a Java fallback compression copy reduction only. It is
not an end-to-end startup, TPS, MSPT, or 500-player result.

## Current 2026-05-10 10:42 CEST Verification

Fresh commands from this continuation:

```bash
bash scripts/bench_noise_interpolator_fractions.sh
cd upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-noiseinterp-fractions-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
NoiseInterpolator fraction-array candidate: REJECTED
microbench: division_best_ms=29.308, array_fraction_best_ms=5.943, speedup=4.932x, equivalence=PASS
temporary build/hash/json: PASS
temporary plugin matrix: PASS, Done (33.112s), 11 real plugins initialized
temporary restart/recovery: PASS, Done (17.049s), Saved the game
temporary forced-ticket persistence: PASS, first/restart Done 14.103s/9.155s
strict 50-bot gate: online_max=50, tps1_avg=16.75, avg_tick_ms_avg=63.54, loaded_chunks_max=2891, watchdog_thread_dumps=3, nearby_players_stack_hits=7
rollback build_optimized.sh: PASS, applyPatches Applied 912 patches
rollback sha256sum -c reports/artifact-hashes.txt: PASS
rollback artifacts.json: PASS
rollback plugin matrix: PASS, Done (29.035s), 11 real plugins initialized
rollback restart/recovery: PASS, Done (18.063s), Saved the game
rollback forced-ticket persistence: PASS, first/restart Done 16.174s/12.176s
```

The benchmark script and report remain as rejected evidence. The production
runtime is back on the baseline `NoiseInterpolator.compute(...)` division path.

## Current 2026-05-10 10:07 CEST Verification

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-playerloader-cache-manager-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-nearby-list-limit64-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
player-loader cached-manager candidate: REJECTED, online_max=50, tps1_avg=17.45, avg_tick_ms_avg=65.35, loaded_chunks_max=2412, watchdog_thread_dumps=4, nearby_players_stack_hits=8
NearbyPlayers limit64 candidate: REJECTED, online_max=50, tps1_avg=16.90, avg_tick_ms_avg=88.49, loaded_chunks_max=2365, watchdog_thread_dumps=6, nearby_players_stack_hits=4
rollback build_optimized.sh: PASS, applyPatches Applied 912 patches
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
rollback plugin matrix: PASS, Done (29.443s), 11 real plugins initialized
rollback restart/recovery: PASS, Done (21.228s), Saved the game
rollback forced-ticket persistence: PASS, first/restart Done 21.372s/11.272s
optimized artifact sha256=421edbef592cb75b3e74fa2b1010f82fcc384512ba4773ada1dc78b6b52e28e0
app-cds sha256=46b205c64a6131fda6dea3a1530d51a02e01a3e1a02541f0431bf52ef7daebbf
```

This continuation keeps only two harness fixes:
`prepare_fast_runtime.sh` invalidates stale runtime caches on jar-hash change,
and `generate_app_cds.sh` writes the CDS archive through an absolute path.
Neither rejected movement candidate remains in production.

## Current 2026-05-10 08:39 CEST Verification

Fresh commands from this continuation:

```bash
cd upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-protochunk-postrollback-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
ProtoChunk heightmap candidate: FULLY ROLLED BACK
applyPatches: PASS, Applied 912 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (26.859s), 11 real plugins initialized
restart/recovery: PASS, Done (16.028s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.244s/9.550s
strict 50-bot gate: stable but not accepted, online_max=50, tps1_avg=18.08, avg_tick_ms_avg=96.12, loaded_chunks_max=2609, watchdog_thread_dumps=3, sync_load_stack_hits=0
```

The current runtime is compatible and restart-safe, but the fresh 50-bot run
did not beat the accepted baseline. The next target should come from fresh
movement-ticket or chunk-generation profiling, not from this rejected
ProtoChunk shape.

## Current 2026-05-10 08:05 CEST Verification

Fresh commands from this continuation:

```bash
./scripts/bench_climate_parameter_distance.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-marker-cache-clean-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
Climate.Parameter distance branch rewrite: REJECTED AT MICROBENCH
branch_distance_speedup=0.961x
subtract_first_speedup=0.996x
equivalence=PASS

NoiseChunk marker-wrapper cache clean gate: NOT PROMOTED AS LOAD WIN
host_preflight_ok=true
online_max=50
tps1_avg=18.72
avg_tick_ms_avg=42.07
loaded_chunks_max=1806
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=3
sync_load_stack_hits=0
nearby_players_stack_hits=4
```

The marker-cache run is useful diagnostic evidence but fails the accepted
stability/coverage standard. The next strict gate candidate is `ProtoChunk`
heightmap iterator removal when host preflight clears.

## Current 2026-05-10 07:57 CEST Verification

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-orefeature-loop-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Outcome:

```text
OreFeature scalar-hoist candidate: REJECTED
candidate gate: online_max=50, tps1_avg=18.27, avg_tick_ms_avg=65.21, loaded_chunks_max=2911, watchdog_thread_dumps=2, sync_load_stack_hits=0
production patch removed: paper-server/patches/sources/net/minecraft/world/level/levelgen/feature/OreFeature.java.patch
generated source grep: widthHeight/d5Squared/d5d6Squared absent
build_optimized.sh: PASS, applySourcePatches Applied 912 patches
artifact reports refreshed: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (26.953s), 11 real plugins initialized
restart/recovery: PASS, Done (17.037s), Saved the game
forced-ticket persistence: PASS, first/restart Done 12.862s/8.382s
```

This continuation rejects another weak worldgen micro-optimization and restores
the production runtime. It does not add a new accepted production optimization
and does not satisfy the 20 TPS / 500-bot target.

## Current 2026-05-10 06:39 CEST Verification

Fresh commands from this continuation:

```bash
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-post-rollback-baseline-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
jfr view hot-methods reports/load-50bots-current-jfr-rebaseline-20260510.jfr
```

Outcome:

```text
waypoint chunk-key candidate: REJECTED, reports/load-waypoint-chunkkey-update-20260510-summary.txt = 17.99 TPS / 63.66 ms / 2516 chunks
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
artifact reports refreshed: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (27.799s), 11 real plugins initialized
restart/recovery: PASS, Done (16.968s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.274s/8.602s
fresh rollback 50-bot baseline: PASS with limits, online_max=50, tps1_avg=18.29, avg_tick_ms_avg=50.90, loaded_chunks_max=2441, no kicks/errors/watchdog/sync-load
JFR top methods: ImprovedNoise.sampleAndLerp 20.38%, ImprovedNoise.noise 13.92%, PerlinNoise.getValue 11.13%
```

This continuation restores the production runtime after a rejected candidate
and records a clean current baseline. It does not add a new accepted
production optimization and does not satisfy the 20 TPS / 500-bot target.

## Current 2026-05-10 05:47 CEST Verification

Fresh commands from this continuation:

```bash
./scripts/bench_chunk_ticket_stage_map.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-referencelist-linear3-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-referencelist-linear3-rerun-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
chunkTicketStage capacity bench: REJECTED, get 0.903x, mutation 0.983x, equivalence PASS
NearbyPlayers limit=3 strict run 1: REJECTED, online_max=50, tps1_avg=18.06, avg_tick_ms_avg=46.77, loaded_chunks_max=2396, no kicks/errors/watchdog/sync-load
NearbyPlayers limit=3 strict run 2: REJECTED, online_max=50, tps1_avg=17.83, avg_tick_ms_avg=62.80, loaded_chunks_max=2427, no kicks/errors/watchdog/sync-load
rollback build_optimized.sh: PASS, applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
rollback plugin matrix: PASS, Done (27.251s), CompatProbe command/events ok
rollback restart/recovery: PASS, Done (15.819s), Saved the game
rollback forced-ticket persistence: PASS, first/restart Done 13.055s/8.863s
```

This continuation does not add a production optimization. The temporary
`NearbyPlayers` limit `3` and `chunkTicketStage` capacity candidates failed
the acceptance comparison and were removed.

## Current 2026-05-10 03:40 CEST Verification

Fresh commands from this continuation:

```bash
./scripts/bench_reference_list_transition_remove.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-reference-list-transition-remove-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
ReferenceList transition-remove bench: PASS, transition 1.385x, miss 3.675x, dense 0.995x, equivalence PASS
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (26.747s), CompatProbe command/events ok
restart/recovery: PASS, Done (16.102s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.346s/8.585s
strict 50-bot gate: not accepted, online_max=50, tps1_avg=18.07, avg_tick_ms_avg=51.73, loaded_chunks_max=2782, watchdog_thread_dumps=3
candidate stability: no kicks, no bot errors, no sync-load stacks, no NearbyPlayers stack hits, no stability failures
```

This continuation keeps the patch only as a narrow movement hot-path
reduction. It is not a 20 TPS, 500-player, sub-second boot, all-plugin, or
vanilla-parity claim.

## Current 2026-05-10 02:26 CEST Verification

Fresh commands from this continuation:

```bash
./scripts/bench_nearby_player_maps.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-nearby-map-capacity-gate-20260510 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Outcome:

```text
NearbyPlayers map-capacity microbench: PASS, 50 players 2.245x, 500 players 2.443x, equivalence PASS
strict 50-bot candidate gate: REJECTED, online_max=50, tps1_avg=17.95, avg_tick_ms_avg=52.03, loaded_chunks_max=2059
candidate stability: no kicks, no bot errors, no watchdog dumps, no sync-load stacks, no stability failures
rollback build_optimized.sh: PASS, applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
rollback plugin matrix: PASS, Done (26.143s), CompatProbe command/events ok
rollback restart/recovery: PASS, Done (16.191s), Saved the game
rollback forced-ticket persistence: PASS, first/restart Done 12.884s/9.473s
```

This continuation does not add a production optimization. The temporary
`NearbyPlayers` map pre-size candidate failed the accepted 50-bot load
reference and was removed.

## Current 2026-05-10 01:09 CEST Verification

Fresh commands from this continuation:

```bash
bash ./scripts/bench_protochunk_heightmap.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-protochunk-heightmap-restored-gate-20260510 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-protochunk-heightmap-rerun-20260510 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
ProtoChunk heightmap bench: PASS, 133.632 ms -> 100.017 ms, 1.336x, iterator allocations 2 -> 0, equivalence PASS
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (27.655s), CompatProbe command/events ok
restart/recovery: PASS, Done (15.839s), Saved the game
forced-ticket persistence: PASS, first/restart Done 13.433s/8.960s
strict 50-bot run 1: stable, online_max=50, tps1_avg=18.51, avg_tick_ms_avg=54.42, loaded_chunks_max=2217, no kicks/errors/watchdog/sync-load
strict 50-bot run 2: stable, online_max=50, tps1_avg=17.84, avg_tick_ms_avg=46.13, loaded_chunks_max=2215, no kicks/errors/watchdog/sync-load
```

`./gradlew rebuildPatches --no-daemon` was attempted before the manual
normalization of feature patch `0048`, but Paperweight failed inside
`rebuildSourcePatches` on `git -c commit.gpgsign=false -c core.safecrlf=false
switch -`. That tooling issue is recorded in `BLOCKED.md`; it is not used as
a passing verification claim in this cycle.

This is accepted only as a narrow heightmap/setBlock allocation and work
reduction. No stable 20 TPS, 500-player, sub-second boot, or vanilla-parity
claim is made, and the strict load reference is still not beaten.

## Current 2026-05-09 23:56 CEST Verification

Fresh commands from this continuation:

```bash
./scripts/bench_climate_rtree_search.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-rtree-search-prune-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-rtree-search-prune-noisy-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
Climate RTree search bench: PASS, random 1.114x, walk 1.207x, equivalence PASS
rebuildPatches: PASS, Rebuilt 913 patches, Saved modified patches (44/47)
build_optimized.sh: PASS, includes applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (30.298s), CompatProbe command/events ok
restart/recovery: PASS, Done (22.644s), Saved the game
forced-ticket persistence: PASS, first/restart Done 18.153s/18.919s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.657 > max_load_per_cpu=0.750 and idle_percent_1s=33.05 < min_idle_percent=40.00
forced noisy 50-bot diagnostic: 50 connected/ready/active, no kicks/errors/watchdog/sync-load, but non-comparable at tps1_avg=17.23 and avg_tick_ms_avg=58.78
```

This is accepted only as a small biome-search work reduction. No stable 20
TPS, 500-player, sub-second boot, or vanilla-parity claim is made.

## Current 2026-05-09 23:36 CEST Verification

Fresh commands from this continuation:

```bash
./scripts/bench_carver_iteration.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-carver-iteration-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-carver-iteration-noisy-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
carver iteration bench: PASS, indexed_speedup=1.468x, saved_allocated_bytes_per_iteration=32.000, equivalence PASS
rebuildPatches: PASS, Rebuilt 913 patches, Saved modified patches (43/46)
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (31.900s), CompatProbe command/events ok
restart/recovery: PASS, Done (25.501s), Saved the game
forced-ticket persistence: PASS, first/restart Done 15.097s/10.702s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.962 > max_load_per_cpu=0.750 and idle_percent_1s=27.15 < min_idle_percent=40.00
forced noisy 50-bot diagnostic: 50 connected/ready/active, no kicks/errors/watchdog/sync-load, but non-comparable at tps1_avg=17.24 and avg_tick_ms_avg=95.80
```

This is accepted only as a small chunk-generation allocation/work reduction.
No stable 20 TPS, 500-player, sub-second boot, or vanilla-parity claim is
made.

## Current 2026-05-09 23:12 CEST Verification

Fresh commands from this continuation:

```bash
./gradlew rebuildPatches --no-daemon
./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-climate-rtree-build-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
rebuildPatches: PASS, Rebuilt 913 patches, Saved modified patches (42/45) for java
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (30.428s), CompatProbe command/events ok
restart/recovery: PASS, Done (20.358s), Saved the game
forced-ticket persistence: PASS, first/restart Done 14.099s/9.797s
strict 50-bot gate: PASS preflight, then 50 connected/ready/active, no kicks/errors/watchdog/sync-load, but only tps1_avg=18.04, avg_tick_ms_avg=56.39, loaded_chunks_max=2429
```

The `Climate.RTree.build(...)` cleanup is kept as a safe allocation/work
reduction, but it is not a 20 TPS or 500-player claim.

## Current 2026-05-09 22:38 CEST Verification

Fresh commands from this continuation:

```bash
bash ./scripts/bench_waypoint_snapshot.sh
bash ./scripts/bench_yclamped_gradient.sh
./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
javap -classpath artifacts/optimized-runtime/bundler/versions/1.21.10/paper-1.21.10.jar -c -p 'net.minecraft.world.level.levelgen.DensityFunctions$YClampedGradient'
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-post-yclamped-reject-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
waypoint sized-array snapshot: REJECTED, 0.782x, equivalence PASS
YClampedGradient inline: REJECTED_AND_ROLLED_BACK, 0.987x, equivalence PASS
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
javap YClampedGradient.compute: calls Mth.clampedMap again after rollback
plugin matrix: PASS, Done (35.098s), CompatProbe command/events ok
restart/recovery: PASS, Done (25.464s), Saved the game
forced-ticket persistence: PASS, first/restart Done 15.876s/11.482s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.313 > max_load_per_cpu=0.750
```

No new production optimization was kept from this continuation. No 20 TPS,
500-player, or sub-second boot claim is made.

## Current 2026-05-09 22:06 CEST Verification

Fresh commands for the `DensityFunction.Visitor` holder/marker hook production
path:

```bash
./scripts/bench_density_visitor_hooks.sh
./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
javap -classpath "$(cat artifacts/optimized-runtime/classpath.txt)" -c 'net.minecraft.world.level.levelgen.DensityFunctions$HolderHolder'
javap -classpath "$(cat artifacts/optimized-runtime/classpath.txt)" -c 'net.minecraft.world.level.levelgen.DensityFunctions$MarkerOrMarked'
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-density-visitor-hooks-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
density visitor hook bench: PASS, 23.617x, zero temporary holder/marker allocations, equivalence PASS
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
javap HolderHolder.mapAll: calls DensityFunction$Visitor.applyHolder
javap MarkerOrMarked.mapAll: calls DensityFunction$Visitor.applyMarker
plugin matrix: PASS, Done (38.265s), CompatProbe command/events ok
restart/recovery: PASS, Done (29.923s), Saved the game
forced-ticket persistence: PASS, first/restart Done 21.751s/15.859s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.679 > max_load_per_cpu=0.750
```

This is accepted only as allocation/work reduction with compatibility gates.
No 20 TPS stable, end-to-end TPS/MSPT, or 500-player claim is made.

## Current 2026-05-09 21:08 CEST Verification

Fresh commands for the `JigsawBlock.canAttach(...)` target-first candidate:

```bash
./scripts/bench_jigsaw_canattach.sh
./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-jigsaw-targetfirst-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
jigsaw canAttach bench: PASS, target_first 12.354x vs old, equivalence PASS
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (32.953s), CompatProbe command/events ok
restart/recovery: PASS, Done (18.681s), Saved the game
forced-ticket persistence: PASS, first/restart Done 14.874s/12.361s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.187 > max_load_per_cpu=0.750
```

This is not end-to-end load evidence and not a 20 TPS/500-player claim.

## Current 2026-05-09 20:51 CEST Verification

Fresh commands for the rejected `DensityFunctions.Ap2.fillArray(ADD)`
scratch-buffer candidate and rollback:

```bash
./scripts/bench_density_ap2_fill.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-density-ap2-fill-gate-20260509-rerun1 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-density-ap2-fill-postrollback-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
density-ap2-fill bench: PASS, flat 3.536x, nested 1.573x, equivalence PASS, reentrant equivalence PASS
strict 50-bot gate: REJECTED, preflight ok load_per_cpu=0.471, 50 connected/ready/active, no kicks/errors/watchdog/sync-load/stability failures, but only tps1_avg=17.75, avg_tick_ms_avg=78.14, loaded_chunks_max=1933
rollback applyPatches: PASS, Applied 913 patches
rollback build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (39.577s), CompatProbe command/events ok
restart/recovery: PASS, Done (21.865s), Saved the game
forced-ticket persistence: PASS, first/restart Done 18.679s/11.969s
post-rollback strict 50-bot control: BLOCKED by host preflight, load_per_cpu=1.185 > max_load_per_cpu=0.750
```

No end-to-end load win, 20 TPS stable claim, or 500-player claim is made from
this cycle.

## Current 2026-05-09 20:12 CEST Verification

Fresh commands from the rejected `Entity.setPosRaw(...)` bounding-box shortcut
and rollback:

```bash
./scripts/bench_entity_bounding_box.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-entity-bbox-direct-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
entity-bounding-box bench: PASS, 1.424x direct-shape speedup, equivalence PASS
strict 50-bot gate: REJECTED, preflight ok load_per_cpu=0.625, 50 connected/ready/active, no kicks/errors/watchdog/sync-load/stability failures, but only tps1_avg=17.58, avg_tick_ms_avg=67.63, loaded_chunks_max=1721
rollback applyPatches: PASS, Applied 913 patches
rollback build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (30.454s), CompatProbe command/events ok
restart/recovery: PASS, Done (19.537s)
forced-ticket persistence: PASS, first/restart Done 19.076s/11.778s
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 18:45 CEST Verification

Fresh commands from the `ReferenceList` small-mode experiment and rollback:

```bash
./scripts/bench_reference_list_remove.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-referencelist-smallmode-state-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=50bots-referencelist-smallmode-state-noisy-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
reference-list bench: PASS, single runtime 2.056x, pair runtime 2.293x, dense runtime 0.729x versus old
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (32.115s)
restart/recovery: PASS, Done (19.754s)
forced-ticket persistence: PASS, first/restart Done 15.532s/9.772s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.840
noisy 50-bot diagnostic: PASS WITH LIMITS, 50 connected/ready/active, no kicks/errors/sync-load, tps1_avg=18.50, avg_tick_ms_avg=35.14, loaded_chunks_max=824
watchdog_thread_dumps=6, nearby_players_stack_hits=13, thread_check_failures=0, chunk_system_errors=0, feature_placement_errors=0, off_main_poi_hits=0, stability_failures=0
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 18:10 CEST Verification

Fresh commands from the POI main-thread fix and waypoint skip cycle:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-poi-mainthread-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=50bots-poi-mainthread-noisy-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (57.490s)
restart/recovery: PASS, Done (44.201s)
forced-ticket persistence: PASS, first/restart Done 32.221s/20.055s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.824
noisy 50-bot diagnostic: PASS WITH LIMITS, 50 connected/ready/active, no kicks/errors/watchdog/sync-load, tps1_avg=17.84, avg_tick_ms_avg=224.77, loaded_chunks_max=1796
thread_check_failures=0, chunk_system_errors=0, feature_placement_errors=0, off_main_poi_hits=0, stability_failures=0
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 15:55 CEST Verification

Fresh commands from the rejected ticket/waypoint cycle and restored baseline:

```bash
./scripts/bench_ticketset_search.sh
./scripts/bench_ticket_compare.sh
./scripts/bench_waypoint_snapshot.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-waypoint-snapshot-manual-gate-20260509-1544 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Outcome:

```text
ticketset-search bench: rejected, unchecked_binary 0.966x, linear4 0.945x, linear8 0.959x, linear12 0.973x, equivalence PASS
ticket-compare bench: rejected, old 168.504 ms, cached 169.166 ms, 0.996x, equivalence PASS
waypoint-snapshot bench: manual copy 1.625x, equivalence PASS
50bots-waypoint-snapshot-manual-gate-20260509-1544: REJECTED
online_max=50
tps1_avg=17.74
avg_tick_ms_avg=37.32
loaded_chunks_max=2077
watchdog_thread_dumps=3
nearby_players_stack_hits=8
sync_load_stack_hits=0
bot_errors_max=0

restored baseline source: ServerWaypointManager.snapshotEntries returns map.entrySet().toArray(Entry[]::new)
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.440s)
restart/recovery: PASS, Done (17.388s)
forced-ticket persistence: PASS, first/restart Done 13.805s/9.338s
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 14:27 CEST Verification

Fresh commands from the `ChunkHolderManager` transient entity-chunk lazy-init
candidate and its verification cycle:

```bash
./scripts/bench_entity_chunk_transient.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=50bots-entitychunk-lazy-transient-noisy-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-entitychunk-lazy-transient-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
entity-chunk transient bench: old 65.410 ms, new 61.437 ms, 1.065x, allocated bytes 140000000 -> 20000000, equivalence PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (30.140s)
restart/recovery: PASS, Done (19.105s)
forced-ticket persistence: PASS, first/restart Done 14.037s/9.349s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.003
noisy 50-bot diagnostic: PASS WITH LIMITS, 50 connected/ready/active, no kicks/errors/watchdog/sync-load, tps1_avg=18.21, avg_tick_ms_avg=63.30, loaded_chunks_max=2295
```

No accepted 20 TPS or 500-player claim is made from this cycle.

## Current 2026-05-09 13:30 CEST Verification

Fresh commands from the `CaveWorldCarver` floor-skip experiment and rollback:

```bash
./scripts/bench_cave_carver_skip.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-cavecarver-floor-skip-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
cave-carver skip bench: direct helper 1.171x, equivalence PASS
candidate build/hash/plugin/restart/forced-ticket gates: PASS
50bots-cavecarver-floor-skip-gate-20260509: REJECTED
online_max=50
tps1_avg=17.79
avg_tick_ms_avg=108.48
loaded_chunks_max=1867
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0

rollback build_optimized.sh: PASS
rollback sha256sum -c reports/artifact-hashes.txt: PASS
rollback plugin matrix: PASS, Done (27.768s)
rollback restart/recovery: PASS, Done (17.133s)
rollback forced-ticket persistence: PASS, first/restart Done 14.395s/9.335s
patch removed from production: PASS
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 12:45 CEST Verification

Fresh commands from the marker hook experiment and rollback:

```bash
./scripts/bench_marker_cache.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-marker-hook-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
marker-cache bench: 4.982x, equivalence PASS
candidate build/hash/plugin/restart/forced-ticket gates: PASS
50bots-marker-hook-gate-20260509: REJECTED
tps1_avg=17.84
avg_tick_ms_avg=67.37
loaded_chunks_max=2081
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0

rollback build_optimized.sh: PASS
rollback sha256sum -c reports/artifact-hashes.txt: PASS
patch removed from production: PASS
```

No 20 TPS stable claim and no 500-player claim are made from this state.

## Current 2026-05-09 12:29 CEST Verification

Fresh commands from the `BlendedNoise` octave-cache experiment and rollback:

```bash
./scripts/bench_blended_noise_octaves.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-blended-octave-cache-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-blended-octave-cache-rollback-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
blended-noise octave bench: 1.178x, equivalence PASS
candidate build/hash/plugin/restart/forced-ticket gates: PASS
50bots-blended-octave-cache-gate-20260509: REJECTED
tps1_avg=17.93
avg_tick_ms_avg=56.72
loaded_chunks_max=2079
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0

rollback build_optimized.sh: PASS
rollback sha256sum -c reports/artifact-hashes.txt: PASS
rollback plugin matrix: PASS, Done (28.079s)
rollback restart/recovery: PASS, Done (17.050s)
rollback forced-ticket persistence: PASS, first/restart Done 12.727s/8.805s
rollback generated BlendedNoise.java: cached octave fields absent

50bots-blended-octave-cache-rollback-20260509: STABLE BUT NOT TARGET
tps1_avg=17.85
avg_tick_ms_avg=56.02
loaded_chunks_max=2176
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

No 20 TPS stable claim and no 500-player claim are made from this state.

## Current 2026-05-09 11:51 CEST Verification

Fresh commands from the latest EntityLookup experiment cycle and rollback:

```bash
./scripts/bench_entity_lookup_status.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-entitylookup-direct-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-entitymove-status-skip-gate-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=50bots-entitymove-status-skip-noisy-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-baseline-restored-20260509 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
entity-lookup status bench: direct_status 1.039x, direct_accessible 1.054x, equivalence PASS
build_optimized.sh after rollback: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix after rollback: PASS, Done (26.884s)
restart/recovery after rollback: PASS, Done (16.695s)
forced-ticket persistence after rollback: PASS, first/restart Done 13.839s/8.889s

50bots-entitylookup-direct-gate-20260509: REJECTED
tps1_avg=17.53
avg_tick_ms_avg=46.96
loaded_chunks_max=2083

50bots-entitymove-status-skip-gate-20260509: BLOCKED by host preflight
host_preflight_ok=false
load_per_cpu=0.928

50bots-entitymove-status-skip-noisy-20260509: DIAGNOSTIC ONLY, REJECTED
tps1_avg=17.22
avg_tick_ms_avg=45.42
loaded_chunks_max=1827
watchdog_thread_dumps=1

50bots-baseline-restored-20260509: STABLE BUT NOT TARGET
tps1_avg=17.66
avg_tick_ms_avg=47.78
loaded_chunks_max=1964
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 07:33 CEST Verification

Fresh commands on the current `ReferenceList` sparse `NearbyPlayers` artifact:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
./scripts/bench_reference_list_remove.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-referencelist-linear-20260509-0529 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=50bots-referencelist-linear-noisy-20260509-0529 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
build_optimized.sh: PASS
reference-list bench: sparse singleton 2.513x, sparse pair 2.090x, dense 0.717x
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (28.720s)
restart/recovery: PASS, Done (18.333s)
forced-ticket persistence: PASS, first/restart Done 14.325s/10.410s
50bots-referencelist-linear-20260509-0529: BLOCKED by host preflight
host_preflight_ok=false
load_per_cpu=0.893
50bots-referencelist-linear-noisy-20260509-0529: DIAGNOSTIC ONLY, NOT ACCEPTED
online_max=50
tps1_avg=17.76
avg_tick_ms_avg=48.46
loaded_chunks_max=2326
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=4
```

No 20 TPS stable claim and no 500-player claim are made from this state.

## Current 2026-05-09 04:00 CEST Verification

Fresh commands on the current artifact:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-placedfeature-traversal-gate-20260509-0558 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (27.869s)
restart/recovery: PASS, Done (18.230s)
forced-ticket persistence: PASS, first/restart Done 14.263s/9.582s
50bots-placedfeature-traversal-gate-20260509-0558: NOT ACCEPTED
host_preflight_ok=true
online_max=50
tps1_avg=17.71
avg_tick_ms_avg=42.70
loaded_chunks_max=1928
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The focused `PlacedFeature` traversal microbench is still positive:

```text
reports/placed-feature-traversal-bench.txt
equivalence=PASS
stream_total_ns=393666514
recursive_total_ns=276173886
speedup=1.425x
```

## Current 2026-05-09 03:30 CEST Verification

Fresh commands on the current artifact:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-spectator-nosyncload-reset-gate-20260509-0355 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
build_optimized.sh: PASS
plugin matrix: PASS, Done (30.599s)
restart/recovery: PASS, Done (18.990s)
forced-ticket persistence: PASS, first/restart Done 14.791s/10.665s
50bots-spectator-nosyncload-reset-gate-20260509-0355: BLOCKED by host preflight
host_preflight_ok=false
load_per_cpu=0.885
```

## Fresh Commands Run

Latest 2026-05-09 build-restore and runtime gates:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-buildrestore-gate-20260509-0157 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-buildrestore-nocpuset-20260509-0200 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Outcome:

```text
build_optimized.sh: PASS, applySourcePatches Applied 912 patches, createMojmapBundlerJar PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (27.348s)
restart/recovery: PASS, Done (17.499s)
forced-ticket persistence: PASS, first/restart Done 15.120s/9.131s

50bots-buildrestore-gate-20260509-0157: NOT ACCEPTED
online_max=50
tps1_avg=19.52
avg_tick_ms_avg=26.57
loaded_chunks_max=1406
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=8
sync_load_stack_hits=7

50bots-buildrestore-nocpuset-20260509-0200: REJECTED CONFIG DIAGNOSTIC
worker threads=12, io threads=2
tps1_avg=16.79
avg_tick_ms_avg=353.82
loaded_chunks_max=4764
watchdog_thread_dumps=5
sync_load_stack_hits=5
```

The current blocker to accepting the good TPS/MSPT pinned run is stability:
movement-triggered `ServerChunkCache.syncLoad` thread dumps remain under fast
spectator movement. The project still has no 500-player or stable 20 TPS
claim.

Latest `NoiseChunk` marker wrapper cache:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
javap -classpath artifacts/optimized-runtime/bundler/versions/1.21.10/paper-1.21.10.jar -c -p 'net.minecraft.world.level.levelgen.NoiseChunk$1'
sha256sum -c reports/artifact-hashes.txt
bash ./scripts/bench_improved_noise_derivative.sh
bash ./scripts/bench_marker_cache.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-marker-cache-gate-20260508 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=50bots-marker-cache-noisy-20260508 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
reports/marker-cache-bench.txt
old_best_ms=173.517
cached_best_ms=33.489
cached_speedup=5.181x
old_marker_allocations=1920000
cached_marker_allocations=84000
equivalence=PASS

reports/improved-noise-derivative-bench.txt
old_derivative_best_ms=56.989
inline_derivative_best_ms=57.170
inline_derivative_speedup=0.997x
equivalence=PASS

rebuildPatches: PASS, Rebuilt 912 patches
build_optimized.sh: PASS
javap NoiseChunk$1: PASS, applyMarker calls wrapMarker(MarkerOrMarked, DensityFunction)
sha256sum -c reports/artifact-hashes.txt: PASS after refreshing rebuilt artifact hashes
plugin matrix: PASS, Done (31.651s)
restart/recovery: PASS, Done (18.882s)
forced-ticket persistence: PASS, first/restart Done 15.372s/10.768s

strict 50-bot gate: BLOCKED by host preflight
host_preflight_ok=false
load_per_cpu=0.807
max_load_per_cpu=0.750

noisy diagnostic run:
online_max=50
tps1_avg=17.38
avg_tick_ms_avg=429.99
loaded_chunks_max=2745
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Verdict: marker-cache is a safe allocation win with green build/compat gates,
but not an accepted 50-bot or 500-bot load claim. The derivative inline
candidate was rejected at microbench stage.

Latest `OreFeature.doPlace(...)` exact loop cleanup:

```bash
bash ./scripts/bench_ore_feature_loop.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-orefeature-loop-gate-rerun1 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
reports/ore-feature-loop-bench.txt
old_loop_best_ms=60.507
optimized_loop_best_ms=58.403
optimized_speedup=1.036x
equivalence=PASS

applyPatches: PASS, Applied 912 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.608s)
restart/recovery: PASS, Done (17.992s)
forced-ticket persistence: PASS, first/restart Done 14.978s/10.573s

strict 50-bot gate: BLOCKED by host preflight
host_preflight_ok=false
load_per_cpu=1.970
max_load_per_cpu=0.750

noisy diagnostic run:
tps1_avg=17.40
avg_tick_ms_avg=87.38
loaded_chunks_max=2210
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Verdict: built and compatibility-passing, but not an accepted end-to-end
load/TPS win until a clean strict gate runs below the preflight threshold.

Latest Beardifier candidate reject/revert cycle:

```bash
bash ./scripts/bench_beardifier_bury.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=load-50bots-beardifier-bury-gate-20260508-2312 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=load-50bots-post-beardifier-revert-gate-20260508-2323 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
reports/beardifier-bury-bench.txt
current_clamped_map_best_ms=8.304
optimized_branch_best_ms=7.063
optimized_speedup=1.176x
equivalence=PASS

candidate 50-bot gate:
host_preflight_ok=true
tps1_avg=17.97
avg_tick_ms_avg=65.67
loaded_chunks_max=2539
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=0
sync_load_stack_hits=0

post-revert:
applyPatches: PASS, Applied 911 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (27.842s)
restart/recovery: PASS, Done (17.406s)
forced-ticket persistence: PASS, first/restart Done 14.870s/10.043s
post-revert 50-bot gate: completed but not accepted, tps1_avg=16.57, avg_tick_ms_avg=112.19, loaded_chunks_max=3212
```

Verdict: `Beardifier.getBuryContribution(...)` branch was rejected and reverted.
It is not in the production path.

Current `ProtoChunk` heightmap iterator-removal candidate:

```bash
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
bash ./scripts/bench_protochunk_heightmap.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=load-50bots-protochunk-heightmap-spectator-gate-20260508-2045 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
reports/protochunk-heightmap-bench.txt
old_enumset_foreach_best_ms=138.483
new_cached_values_contains_best_ms=105.978
new_speedup=1.307x
old_iterator_allocations_per_setblock=2
new_iterator_allocations_per_setblock=0
equivalence=PASS

applyPatches: PASS, Applied 911 patches
rebuildPatches: PASS, Rebuilt 911 source patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.098s)
restart/recovery: PASS, Done (17.664s)
forced-ticket persistence: PASS, first/restart Done 14.544s/10.444s
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.792 > 0.750
```

Rejected `RangeChoice` constant-out fillArray candidate:

```bash
bash ./scripts/bench_range_choice.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-rangechoice-constant-out-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
rm upstream/Paper/paper-server/patches/features/0041-Optimize-RangeChoice-constant-out-fillArray.patch
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust && MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Outcome:

```text
reports/range-choice-bench.txt
scenario=in_constant_out_dynamic
old_fillarray_best_ms=9.947
optimized_fillarray_best_ms=9.124
optimized_fillarray_speedup=1.090x

scenario=in_dynamic_out_constant
old_fillarray_best_ms=9.977
optimized_fillarray_best_ms=9.507
optimized_fillarray_speedup=1.049x

scenario=both_constant
old_fillarray_best_ms=10.004
optimized_fillarray_best_ms=7.321
optimized_fillarray_speedup=1.366x

scenario=both_dynamic
old_fillarray_best_ms=10.501
optimized_fillarray_best_ms=10.742
optimized_fillarray_speedup=0.978x

candidate applyPatches/build/hash/plugin/restart/forced-ticket: PASS before load gate
strict 50-bot gate: REJECTED, preflight ok load_per_cpu=0.484, online_max=50, tps1_avg=17.63, avg_tick_ms_avg=192.39, loaded_chunks_max=2768, watchdog_thread_dumps=5, nearby_players_stack_hits=4
rollback applyPatches: PASS, Applied 913 patches, no RangeChoiceConstantOut/rangeChoiceLike remains
build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix after rollback: PASS, Done (26.927s)
restart/recovery after rollback: PASS, Done (16.461s), Saved the game
forced-ticket persistence after rollback: PASS, first/restart Done 12.714s/8.332s
```

Verdict: rejected and removed from production. The standalone loop reduction
did not transfer to the server gate and produced watchdog/nearby-player stack
pressure. Do not retry this exact `RangeChoiceConstantOut` shape without new
profile evidence.

Latest `BiomeManager.getBiome(...)` early-exit experiment:

```bash
./scripts/bench_biome_getbiome.sh
```

Outcome:

```text
reports/biome-getbiome-bench.txt
samples=1000000
verify_samples=2000000
equivalence=PASS
old_getbiome_best_ms=136.628
optimized_getbiome_best_ms=193.205
optimized_speedup=0.707x
```

This candidate was rejected at microbench stage. The safe lower-bound partial
exit was slower than the current `BiomeManager.getBiome(...)` path, so no
production patch was kept.

Latest `PalettedContainer.reencodeContents(...)` remap-cache reject/revert cycle:

```bash
./scripts/bench_paletted_reencode_remap_cache.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-paletted-remap-cache-gate-rerun1 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-post-paletted-remap-revert-gate-rerun1 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
PalettedContainer reencode remap-cache: REJECTED AND REVERTED
current_previous_only_best_ms=967.335
cached_palette_ids_best_ms=937.103
cached_speedup=1.032x
equivalence=PASS

candidate strict 50-bot gate:
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

post-revert scratch-only runtime:
applyPatches: PASS, Applied 911 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (31.022s)
restart/recovery: PASS, Done (20.065s)
forced-ticket persistence: PASS, first/restart Done 15.145s/10.071s

post-revert strict 50-bot gate: BLOCKED by host preflight
reports/load-50bots-post-paletted-remap-revert-gate-rerun1-preflight.txt
load_per_cpu=0.807
idle_percent_1s=57.20
max_load_per_cpu=0.750
```

The current production path is back to the earlier scratch-only
`PalettedContainer.reencodeContents(...)` optimization. This is not a
50-bot/500-bot or TPS success claim.

Latest DensityFunctions.Spline candidate cycle:

```bash
./scripts/bench_density_spline_context.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-density-spline-context-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true ./scripts/run_plugin_matrix.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh artifacts/optimized-runtime/run.sh
sha256sum -c reports/artifact-hashes.txt
```

Outcome:

```text
DensityFunctions.Spline context-direct candidate: REJECTED AND REVERTED
old_wrapper_best_ms=33.380
new_direct_best_ms=21.615
direct_speedup=1.544x

strict 50-bot gate:
tps1_avg=18.51
avg_tick_ms_avg=66.01
loaded_chunks_max=2101
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0

post-revert rebuild/gates:
applyPatches: PASS, Applied 911 patches
build_optimized.sh: PASS
plugin matrix: PASS, Done (36.988s)
restart/recovery: PASS, Done (19.526s)
forced-ticket persistence: PASS, first/restart Done 21.483s/18.688s
sha256sum -c reports/artifact-hashes.txt: PASS
```

Latest plugin startup name-log aggregation cycle:

```bash
./scripts/bench_plugin_name_log.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-plugin-name-log-current-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
PluginInitializerManager name-log aggregation: ACCEPTED WITH LIMITS
old_treeset_best_ms=343.898
new_arraylistsort_best_ms=45.491
arraylistsort_speedup=7.560x

rebuildPatches: PASS, Rebuilt 911 source patches
build_optimized.sh: PASS
plugin matrix: PASS, Done (32.863s)
restart/recovery: PASS, Done (23.341s)
forced-ticket persistence: PASS, first/restart Done 21.545s/13.276s
sha256sum -c reports/artifact-hashes.txt: PASS
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.812 > 0.750
```

This is compatibility and focused startup-work evidence only. It is not a clean
end-to-end cold-start speedup claim and not a 50/500-bot load claim.

Latest `Xoroshiro` positional direct-helper cycle was rejected and reverted:

```bash
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-xoroshiro-direct-no-aquiferlocation-gate-rerun1 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-post-xoroshiro-direct-revert-gate-rerun1 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
applyPatches: PASS, Applied 911 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.689s)
restart/recovery: PASS, Done (19.938s)
forced-ticket persistence: PASS, first/restart Done 17.550s/11.041s

Aquifer aquiferLocationAt strict gate: REJECTED
reports/load-50bots-xoroshiro-aquifer-location-gate-rerun1-summary.txt
online_max=50
tps1_avg=17.65
avg_tick_ms_avg=76.12
loaded_chunks_max=2016

direct helper strict gate: REJECTED
reports/load-50bots-xoroshiro-direct-no-aquiferlocation-gate-rerun1-summary.txt
online_max=50
tps1_avg=15.45
avg_tick_ms_avg=92.58
loaded_chunks_max=1264

post-revert strict 50-bot gate: BLOCKED by host preflight
host_preflight_ok=false
reports/load-50bots-post-xoroshiro-direct-revert-gate-rerun1-preflight.txt
load_per_cpu=0.920
idle_percent_1s=41.04
```

## Previous Cycles

Latest `SurfaceRules.SequenceRule` array-indexed candidate and post-revert verification:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-surfacerules-array-index-gate-rerun2 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Outcome:

```text
rebuildPatches: PASS, Rebuilt 910 source patches
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS after updating stale app-cds hash
plugin matrix: PASS, Done (34.263s)
restart/recovery: PASS, Done (21.224s)
forced-ticket persistence: PASS, first/restart Done 17.307s/11.562s
strict 50-bot gate: PASS preflight, then failed accepted baseline
online_max=50
tps1_avg=15.95
avg_tick_ms_avg=117.42
loaded_chunks_max=1785
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Latest `Aquifer` surface-offset candidate and post-revert verification:

```bash
./scripts/bench_aquifer_surface_sampling.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-aquifer-surface-offsets-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-aquifer-surface-offsets-postrevert-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-aquifer-surface-offsets-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-aquifer-surface-offsets-postrevert-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Result: standalone bench PASS (`1.130x`, equivalence PASS), build/hash/plugin
matrix/restart/forced-ticket gates PASS. The first strict 50-bot attempt was
blocked by host preflight (`load_per_cpu=0.967` over `0.750`), the rerun
passed preflight but regressed the accepted baseline at
`17.14 / 82.71 / 2030`, and the candidate was reverted. Post-revert
build/hash/plugin/restart/forced gates passed again, the strict post-revert
50-bot attempt was blocked by host preflight (`load_per_cpu=1.041`), and the
noisy post-revert 10-bot smoke passed at `19.17 / 36.29 / 1572` with zero
kicks/errors/watchdog/sync-load. The noisy 10-bot run is not a comparable load
baseline.

Latest post-revert profiling and microbench rejects:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-post-aquifer-revert-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-10bots-post-aquifer-revert-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
jfr view hot-methods reports/load-10bots-post-aquifer-revert-jfr.jfr
jfr view allocation-by-site reports/load-10bots-post-aquifer-revert-jfr.jfr
./scripts/bench_perlin_getvalue.sh
./scripts/bench_improved_noise_inline.sh
```

Result: JFR smoke passed at `18.89 / 36.75 / 1183` with zero
kicks/errors/watchdog/sync-load. Hot methods were still noise-heavy:
`ImprovedNoise.sampleAndLerp` `20.98%`, `ImprovedNoise.noise` `13.73%`,
`PerlinNoise.getValue` `11.88%`. The exact-class guarded Perlin direct-local
candidate was rejected at microbench (`direct_local_guarded_speedup=0.981x`).
The C2ME/DivineMC arithmetic `ImprovedNoise.sampleAndLerp` shape was also
rejected at microbench (`arithmetic_vs_flat_speedup=0.924x`, equivalence PASS).

Latest `ImprovedNoise` switch-gradient microbench:

```bash
./scripts/bench_improved_noise_inline.sh
```

Result:

```text
flat_gradient_best_ms=39.535
switch_gradient_best_ms=47.174
switch_vs_flat_speedup=0.838x
equivalence=PASS
```

Latest rejected `NoiseChunk.FlatCache` context-allocation cycle and post-revert
verification:

```bash
./scripts/bench_noisechunk_flatcache_context.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
# fixupSourcePatches was attempted after direct patch editing and returned
# "nothing to commit, working tree clean"; the source patch was already synced.
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-flatcache-context-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-flatcache-context-postrevert-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-flatcache-context-postrevert-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Rejected candidate result: `15.36 TPS / 254.43 ms / 1621 chunks`,
`watchdog_thread_dumps=1`. Post-revert strict gate was blocked by host
preflight (`load1=16.34`, `load_per_cpu=1.362`). Post-revert noisy 10-bot
smoke passed without kicks/errors/watchdog/sync-load.

Latest `ImprovedNoise.sampleAndLerp` flat-gradient cycle:

```bash
./scripts/bench_improved_noise_inline.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew rebuildFeaturePatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-improvednoise-flatgrad-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-improvednoise-flatgrad-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Latest `NoiseBasedChunkGenerator` primitive-cache cycle:

```bash
./scripts/bench_noise_generator_settings.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-noise-dim-cache-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-noise-dim-cache-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Latest rejected `PerlinNoise.wrap` Math.floor candidate:

```bash
./scripts/bench_perlin_getvalue.sh
cd /root/rust/upstream/Paper && ./gradlew fixupSourcePatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
javap -classpath artifacts/optimized-runtime/bundler/versions/1.21.10/paper-1.21.10.jar -c net.minecraft.world.level.levelgen.synth.PerlinNoise
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-perlin-wrap-mathfloor-gate2 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

The strict load retry passed host preflight, then completed at
`18.16 TPS / 47.33 ms / 1720 chunks` and was rejected versus the accepted
`18.27 / 47.85 / 2380` baseline. Exact evidence is in `BLOCKED.md` and
`reports/load-50bots-perlin-wrap-mathfloor-gate2b-summary.txt`.

Latest ServerEntity delta identity cycle:

```bash
./scripts/bench_serverentity_delta_identity.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-serverentity-delta-identity-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/boot_benchmark.sh
```

Latest LZ4 stream-wrapper reject/revert cycle:

```bash
./scripts/bench_lz4_stream.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-lz4-no-outer-buffer-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Latest Ownable/Climate continuation cycle:

```bash
./scripts/bench_ownable_rule.sh
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-ownable-rule-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
# Climate bounded-distance candidate was then tested and rejected.
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true bash -x scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-climate-rtree-bound-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
# Revert Climate candidate and rebuild current artifact.
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-climate-rtree-bound-revert LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Latest 2026-05-08 NoiseChunk empty-blender blend-cache candidate/reject cycle:

```bash
./scripts/bench_noisechunk_blendcache.sh
cd /root/rust/upstream/Paper && ./gradlew fixupSourcePatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-noisechunk-empty-blendcache-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-noisechunk-blendcache-revert LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-noisechunk-revert-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-after-noisechunk-revert-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-g1-xms10g-retry-after-revert LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms10G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Latest 2026-05-08 inline-noise candidate/reject cycle:

```bash
./scripts/bench_improved_noise_inline.sh
cd /root/rust/upstream/Paper && ./gradlew fixupSourcePatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-improvednoise-inline-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-final-after-inline-reject LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

```bash
cd /root/rust/upstream/Paper && ./gradlew fixupSourcePatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
scripts/bench_density_visitor_hooks.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-density-hooks-strict LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=10bots-density-hooks-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=60 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/boot_benchmark.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/boot_benchmark.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-concurrent-unlimited LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 PAPER_PLAYER_MAX_CONCURRENT_LOADS=-1 PAPER_PLAYER_MAX_CONCURRENT_GENS=-1 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-reversed-cache-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-spawn-fitness-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260505-noiseinterp-delta-complete LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=load-50bots-20260505-move-log-shared-rate-limit LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260505-noiseinterp-delta LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260505-noiseinterp-delta.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-linearpalette-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-linearpalette-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-restored-postrevert-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-restored-postrevert-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-postrevert-rebaseline LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-postrevert-rebaseline.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-climate-fastpath LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-climate-fastpath.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-climate-fastpath-rerun LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-climate-fastpath-rerun.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Latest waypoint/distance candidate commands:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
rg -n "halfRange|REALLY_FAR_DISTANCE / 2\\.0F" /root/rust/upstream/Paper/paper-server/src/minecraft/java/net/minecraft/world/waypoints/WaypointTransmitter.java /root/rust/upstream/Paper/paper-server/patches/sources/net/minecraft/world/waypoints/WaypointTransmitter.java.patch
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_LABEL=smoke-waypoint-inner-range-2bots-noisy2 LOAD_TEST_GAMEMODE=survival BOT_COUNT=2 DURATION_SECONDS=25 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 BOT_RAMP_SECONDS=2 BOT_START_MOVING_AFTER_MS=1000 ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-waypoint-inner-range-current BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Latest plugin-remapper deferred mappings commands:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
# targeted manual run recorded in reports/remapper-skip-only-summary.txt
```

Latest remapper-index lazy cleanup commands:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
scripts/bench_remapper_index_cleanup.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-remap-index-lazy-cleanup-count-only BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Latest remapper-index dirty-write commands:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
find runs/plugin-matrix/plugins/.paper-remapped -name index.json -type f -exec stat -c '%y %n' {} \; | sort > reports/remapper-index-mtime-before.txt
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
find runs/plugin-matrix/plugins/.paper-remapped -name index.json -type f -exec stat -c '%y %n' {} \; | sort > reports/remapper-index-mtime-after.txt
cmp -s reports/remapper-index-mtime-before.txt reports/remapper-index-mtime-after.txt
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-remapper-index-dirty-write BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Latest ReobfServer precomputed-server-before-mappings commands:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
# targeted manual run recorded in reports/reobf-precomputed-mapping-check.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-reobf-precomputed-mapping BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Latest atomic hard-link precomputed-remap install commands:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
# targeted inode checks recorded in reports/precomputed-hardlink-check.txt and reports/server-remap-hardlink-check.txt
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-hardlink-remap-install BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Latest plugin-directory scan/no-op add-plugin optimization commands:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
scripts/bench_plugin_scan.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-addplugin-skip-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Latest Paper plugin metadata dependency-cache commands:

```bash
scripts/bench_plugin_meta_dependencies.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-plugin-meta-cache-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
sha256sum -c reports/artifact-hashes.txt
```

Latest Spigot load-order allocation cleanup commands:

```bash
scripts/bench_spigot_load_order.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-spigot-loadorder-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
sha256sum -c reports/artifact-hashes.txt
```

Latest Spigot load-after pre-size commands:

```bash
scripts/bench_spigot_load_order.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-spigot-loadafter-presize-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
sha256sum -c reports/artifact-hashes.txt
```

Latest TopographicGraphSorter capacity pre-size commands:

```bash
scripts/bench_topographic_sort.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-toposort-capacity-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
sha256sum -c reports/artifact-hashes.txt
```

Latest JFR / GC configuration commands:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-toposort-current-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-toposort-current-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
jfr view hot-methods reports/load-50bots-toposort-current-jfr.jfr
jfr view allocation-by-site reports/load-50bots-toposort-current-jfr.jfr
jfr view gc-pauses reports/load-50bots-toposort-current-jfr.jfr
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-zgc-generational-gate-retry LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseZGC -XX:+ZGenerational -XX:+DisableExplicitGC -XX:+AlwaysPreTouch' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-g1-xms10g-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms10G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Also run in this cycle and rejected:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260505-noisechunk-forindex-fastdiv LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260505-after-forindex-revert-confirm LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-cellfrac LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-refmap4096-jfr ... ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-playerdata-cache-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-playerdata-cache-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-intperm-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-intperm-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-shortperm-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-shortperm-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-linearpalette-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-linearpalette-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-blendednoise-scale-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Additional rejected command from the latest cycle:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-findtopsurface-threadlocal LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-prelim-quart-mask LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-current-jfr3 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260507-current-jfr3.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-perlin-active-octaves LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-wrap-loadfactor095 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-lazy-blend-cache LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-climate-samplestate-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-io2-config-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 PAPER_CHUNK_IO_THREADS=2 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-improvednoise-graddot-inline LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-mth-lerp-inline LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-surfacerules-sequence-index LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Additional accepted classpath-library/remap commands from the latest cycle:

```bash
./scripts/build_library_probe_plugin.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-library-cache BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| ObfHelper mapping map/StringPool microbench | PASS WITH LIMITS | `reports/obfhelper-maps-bench.txt`: old stream/default maps `256.414 ms`, production-shaped direct top maps + pre-sized StringPool `209.872 ms`, `1.222x` vs old; equivalence PASS |
| Unpromoted ObfHelper variants | NOT PROMOTED | same bench measured default-pool direct top maps `224.191 ms` and single-pool/set variant `227.452 ms`; they are slower/noisier than the kept direct-map + pre-sized StringPool path |
| Current post-revert build/hash | PASS | latest `MC_EULA_AGREE=true ./scripts/build_optimized.sh` PASS after Aquifer 0041 revert; `sha256sum -c reports/artifact-hashes.txt` PASS; optimized jar hash `97720a304176d0f6fa8d222a3b1374de4390aa5debc96924ecd844e12906e3ff` |
| Current post-revert plugin/restart/forced gates | PASS | plugin matrix `Done (32.234s)`, restart/recovery `Done (20.809s)`, forced-ticket persistence first/restart `15.835s`/`11.609s` |
| Current post-revert load gates | PASS WITH LIMITS | no post-Aquifer-revert strict 50-bot verdict because host preflight blocked (`load_per_cpu=1.041` > `0.750`); noisy post-revert 10-bot smoke passed at `19.17/36.29/1572` with no kicks/errors/watchdog/sync-load but is not a comparable baseline |
| ImprovedNoise inline microbench | REJECTED FOR PRODUCTION | standalone equivalence passed and local loop improved `47.592 ms -> 42.544 ms` (`1.119x`), but server gate failed accepted baseline: `tps1_avg=17.78`, `avg_tick_ms_avg=62.90`, `loaded_chunks_max=2693`; production patch removed |
| Final post-reject build | PASS | `applyPatches` applied 910 patches, `compileJava`, `build_optimized.sh`, and `sha256sum -c reports/artifact-hashes.txt` all pass |
| Final post-reject plugin matrix | PASS | `Done (29.272s)`, real jars loaded, `PlayerJoinEvent sequence=3`, `COMPAT_PROBE command=ok events=4`, `LIBRARY_PROBE dependency=loaded-from-plugin-library` |
| Final post-reject restart/recovery | PASS | `Done (17.902s)`, `COMPAT_PROBE command=ok`, `Saved the game`, clean disable |
| Final post-reject forced ticket persistence | PASS | first/restart `Done (13.935s)` / `Done (10.021s)`, forced chunk `[0,0]` remained marked after restart |
| Final strict 50-bot rerun | BLOCKED | preflight refused before Minecraft start: `load1=10.03`, `load_per_cpu=0.835` > `0.750`; see `BLOCKED.md` |
| Patch rebuild | PASS | latest `cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon`: `Rebuilt 910 patches`, `Saved modified patches (35/38) for java` |
| Build | PASS | latest `MC_EULA_AGREE=true ./scripts/build_optimized.sh`: `applySourcePatches` applied 911 patches, `compileJava`, `createMojmapBundlerJar` successful, `precomputed_plugin_remaps=4`, `precomputed_plugin_skips=8`, `precomputed_library_skips=1`, runtime/AppCDS/remaps regenerated after plugin startup name-log aggregation and legacy alias cleanup |
| DensityFunction visitor hook microbench | PASS WITH LIMITS | `scripts/bench_density_visitor_hooks.sh`: old wrapper path `481.076 ms`, hooked path `20.770 ms`, `23.162x`, temporary holder/marker allocations dropped from `3,072,000 + 3,072,000` to `0 + 0`, equivalence PASS; accepted as allocation/work reduction, not end-to-end TPS claim |
| PalettedContainer reencode scratch/remap-cache microbench | PASS WITH LIMITS / REJECTED | scratch reuse: old `new int[]` path `728.576 ms`, ThreadLocal scratch `244.271 ms`, `2.983x`, equivalence PASS and remains in production; remap-cache: current previous-only remap `967.335 ms`, cached palette-id remap `937.103 ms`, `1.032x`, equivalence PASS, but strict 50-bot gate failed baseline (`16.48/76.59/2813` vs `18.27/47.85/2380`), so remap-cache was reverted |
| Remapper index cleanup microbench | PASS | `scripts/bench_remapper_index_cleanup.sh`: old eager cleanup `2060.532 ms`, new lazy count-check path `626.871 ms`, `3.287x` |
| Remapper index dirty write | PASS | targeted second restart on unchanged `runs/plugin-matrix` kept all four `.paper-remapped/*/index.json` mtimes unchanged; `reports/remapper-index-dirty-write-check.txt` records `remapper_index_mtime_unchanged=PASS` |
| ReobfServer precomputed server before mappings | PASS WITH LIMITS | targeted no-precomputed-plugin-remaps run recorded `install_precomputed_server_count=1`, `loading_precomputed_reversed_count=1`, `loading_reobf_mappings_count=0`, `compatprobe_plugin_remap_count=1`; accepted as startup-work reduction, not end-to-end speed claim |
| Atomic hard-link precomputed remap install | PASS WITH LIMITS | `reports/precomputed-hardlink-check.txt` confirms 4 precomputed plugin jars installed as hard links under `.paper-remapped`; `reports/server-remap-hardlink-check.txt` confirms precomputed server remap destination is same inode as artifact; accepted as disk-I/O reduction, not end-to-end speed claim |
| Plugin directory scan microbench | PASS WITH LIMITS | `scripts/bench_plugin_scan.sh`: `Files.walk(depth=1)` `249.466 ms`, `Files.list` `153.480 ms`, `DirectoryStream` `132.363 ms`, `1.160x` over `Files.list`; accepted as narrow plugin-discovery work reduction, not end-to-end boot claim |
| Paper plugin metadata dependency microbench | PASS WITH LIMITS | `scripts/bench_plugin_meta_dependencies.sh`: old stream path `1960.882 ms`, direct-loop path `566.406 ms`, cached path `5.926 ms`, `95.586x` faster than loop for repeated calls; accepted as metadata/dependency-resolution work reduction, not end-to-end boot claim |
| Spigot load-order dependency microbench | PASS WITH LIMITS | `scripts/bench_spigot_load_order.sh`: old load-after build `146.978 ms`, pre-sized `121.139 ms`, `1.213x`; old temporary-HashSet path `2631.046 ms`, direct `contains` path `409.024 ms`, `6.433x`; accepted as load-order allocation reduction, not end-to-end boot claim |
| TopographicGraphSorter capacity microbench | PASS WITH LIMITS | `scripts/bench_topographic_sort.sh`: old default-capacity synthetic DAG sort `633.295 ms`, pre-sized `428.129 ms`, `1.479x`; accepted as load-order allocation reduction, not end-to-end boot claim |
| Plugin loading allocation microbench | PASS WITH LIMITS | `scripts/bench_plugin_loading_allocations.sh`: old default-capacity setup `371.559 ms`, new pre-sized setup `233.823 ms`, `1.589x`; missing-set scan `0.994x` so no speed claim there; validate-no-miss `1.069x` |
| Native plugin loading allocation diagnostic | PASS WITH LIMITS | `scripts/bench_native_plugin_loading_allocation.sh`: equivalence PASS; native absolute timing is slower than Java across measured shapes, while native setup old/new is `2.780x`, missing-set old/new `1.173x`, validate `0.980x`; diagnostic only, no runtime hook |
| Native legacy provided-alias removal diagnostic | PASS WITH LIMITS | `scripts/bench_native_legacy_provided_alias_removal.sh`: equivalence PASS; native beats old Java removeIf `2.130x` but loses to optimized Java reverse-index `0.422x`; Java reverse-index vs old removeIf is `11.962x`; diagnostic only, no runtime hook |
| NoiseInterpolator flat slice microbench | REJECTED | `scripts/bench_noise_interpolator_slice.sh`: flat `double[]` slices were equivalent and reduced modeled arrays per chunk (`1152` -> `192`), but timing regressed slightly: old `284.036 ms`, flat `286.847 ms`, `0.990x`; no production change |
| NoiseChunk interpolator indexed traversal | REJECTED | `scripts/bench_noisechunk_interpolator_array.sh`: list-backed traversal `1108.416 ms`, indexed traversal `1052.171 ms`, `1.053x`, equivalence PASS; strict 50-bot gate passed preflight but failed accepted baseline: `17.87/142.23/2336` vs `18.27/47.85/2380`; production path reverted to foreach/forEach traversal |
| Plugin matrix | PASS | latest pinned run `Done (31.022s)` after rejecting/reverting `PalettedContainer` remap-cache; `CompatProbe` command/events/scheduler/join passed; `LibraryProbe` loaded its external jar dependency; precomputed plugin/library remap caches active |
| Player join | PASS | `PlayerJoinEvent sequence=3 detail=CodexJoinProbe` |
| Status ping | PASS | protocol `773`, Paper `1.21.10` |
| Commands | PASS | `plugins`, `version`, `compatprobe`, `save-all flush` |
| Scheduler | PASS | sync and async `CompatProbe` tasks ticked |
| Plugin library classpath | PASS | `LibraryProbe` logged `LIBRARY_PROBE dependency=loaded-from-plugin-library`; dependency came from `plugins/matrix-libraries/library-probe-dep.jar` via Paper `JarLibrary` |
| Save/load/restart | PASS | latest restart/recovery gate `Done (20.065s)`, `COMPAT_PROBE command=ok events=2 ownServices=0`, `Saved the game`, clean disable, region files present |
| Forced ticket persistence | PASS | `scripts/forced_ticket_persistence_check.sh` added a forced chunk, saved, restarted the same world, and `forceload query 0 0` reported `Chunk at [0, 0] ... is marked for force loading`; latest first/restart boot `15.145s`/`10.071s` |
| Boot benchmark | PASS WITH LIMITS | latest pinned optimized runtime `16488 ms` vs stock Paper `32747 ms` and optimized jar `24342 ms`; still not `<1s`, no startup-speed claim beyond this run |
| Spawn-search patch | PASS WITH LIMITS | boot benchmark `13595 ms`, current plugin matrix `Done (24.571s)`, fresh 50-bot gate `17.78/70.20/5255`, no crash; not a new load baseline |
| 50-bot load | PASS WITH LIMITS | latest completed-delta rerun: 50 connected/ready/active, `tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, `moved_too_quickly_warnings=1`, no process crash |
| Latest accepted 50-bot gate | PASS WITH LIMITS | completed-delta rerun remains baseline: `tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, no watchdog/sync-load hits |
| Latest strict 50-bot gate | PASS WITH LIMITS | after `TopographicGraphSorter` capacity pre-size: 50 connected/ready/active, no kicks/errors/watchdog/sync-load, but failed accepted baseline: `tps1_avg=16.93`, `avg_tick_ms_avg=145.06`, `loaded_chunks_max=2005` |
| Current strict 50-bot gate | BLOCKED | current post-remap-cache-revert strict 50-bot 32/32 refused before Minecraft start: `load1=9.69`, `load_per_cpu=0.807` > `0.750`, `idle_percent_1s=57.20`; no comparable load/TPS claim is made |
| Rejected LZ4 stream wrapper | REJECTED | standalone LZ4 stream microbench improved `3432.518 ms` -> `3028.499 ms` (`1.133x`), but the real 50-bot 32/32 gate regressed to `18.53/80.71/2085`; production patch was reverted and post-revert build/hash/plugin/restart/forced gates passed |
| Current noisy 10-bot 32/32 smoke | PASS WITH LIMITS | forced non-comparable post-`NoiseChunk` revert JFR run: `online_max=10`, `tps1_avg=19.39`, `avg_tick_ms_avg=38.78`, `loaded_chunks_max=1955`, kicks/errors/watchdog/sync-load `0`; diagnostic only |
| Current plugin-directory scan | PASS WITH LIMITS | `DirectoryStream` microbench on real matrix: `Files.list` `153.480 ms`, `DirectoryStream` `132.363 ms`, `1.160x`; build/plugin/restart/forced-ticket/hash passed; no end-to-end TPS claim |
| Current JFR 50-bot gate | PASS WITH LIMITS | `50bots-after-noisechunk-revert-jfr`: 50 connected/ready/active, no kicks/errors/watchdog/sync-load, `tps1_avg=18.04`, `avg_tick_ms_avg=70.58`, `loaded_chunks_max=2148`; hot methods are dominated by `ImprovedNoise.p` (`48.83%`) |
| Rejected generational ZGC config | REJECTED | `50bots-zgc-generational-gate-retry`: `tps1_avg=15.71`, `avg_tick_ms_avg=203.15`, `loaded_chunks_max=1604`, `watchdog_thread_dumps=2`; no JVM default changed |
| Fixed 10G G1 config check | BLOCKED | `50bots-g1-xms10g-retry-after-revert` stopped before Minecraft start: `load1=11.00`, `load_per_cpu=0.917` > `0.750` |
| Latest rejected 2026-05-07 50-bot gates | REVERTED | `Climate.Node`: `17.39/47.48/1236`; `CubicSpline.mapAll`: `17.45/126.93/968`; `BlendedNoise`: `17.50/90.04/2376`; `FindTopSurface`: `17.67/59.76/2449`; `preliminarySurfaceLevel`: `15.83/108.32/2280`; `PerlinNoise.activeOctaves`: `16.76/138.50/1126`; `NoiseChunk.wrapLoadFactor095`: `16.85/74.43/1020`; `NoiseChunk.lazyBlendCaches`: `16.02/65.09/562`; `Climate.SampleState`: `16.91/96.16/1993`; `chunkIOThreads2`: `16.96/74.18/861`; `ImprovedNoise.gradDotInline`: `17.37/103.93/2312`; `Mth.lerpInline`: `18.02/43.93/1625`; `SurfaceRules.sequenceIndex`: `18.79/38.68/1216`, watchdog |
| Watchdog/sync load | PASS WITH LIMITS | accepted baseline has no watchdog and no `ServerChunkCache.syncLoad`; some rejected experiments hit watchdog during `save-all`, and the non-watchdog regressions were also reverted |
| Measured 500-bot production gate | PASS WITH LIMITS | `reports/production-500-repeat-quorum.txt`: 3/3 preserved cold/fresh + warm-source release runs passed for 500 bots, 32/32, creative block workload, `repeat_quorum_pass=true`; needs separate long soak and real-player/general-workload validation |
| Full differential mechanics | NOT RUN | needs scripted oracle harness |
| Native `PerlinGetValue` variant diagnostic | PASS DIAGNOSTIC | `scripts/bench_native_perlin_getvalue.sh`: six Java variants matched Rust/JNI summaries with `equivalence=PASS`; no Paper runtime hook |
| Expanded native `NoiseChunk` wrap capacity matrix | PASS DIAGNOSTIC | 13 shape variants, `equivalence=PASS`, `native_speedup_vs_java=1.063x`; still blocked from runtime by the earlier strict-gate rejection |
| Current `PerlinNoise` primitive amplitude-cache patch | PASS WITH LIMITS | `tps1_avg=17.49`, `avg_tick_ms_avg=58.12`, `loaded_chunks_max=2472`, no watchdog/sync-load hits |
| Current `Climate.RTree` / `Climate.Sampler` fast-path patch | PASS WITH LIMITS | repeated runs: `18.32/91.97/2991` and `18.38/99.65/2986`, no watchdog/sync-load hits, but still below 20 TPS |
| Current `NoiseChunk.NoiseInterpolator` delta interpolation | PASS WITH LIMITS | completed rerun: `tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, no watchdog/sync-load hits |
| Rejected `NoiseChunk.forIndex` fast-div experiment | REVERTED | experiment run `tps1_avg=16.93`, `avg_tick_ms_avg=173.54`; control after revert `tps1_avg=16.73`, `avg_tick_ms_avg=56.58`, `watchdog_thread_dumps=1`; bytecode now uses `Math.floorMod`/`Math.floorDiv` again |
| Rejected `PrepareSpawnTask` playerdata cache experiment | REVERTED | `tps1_avg=16.98`, `avg_tick_ms_avg=96.49`, `loaded_chunks_max=3487` |
| Rejected `LinearPalette` reference-map cache experiment | REVERTED | `tps1_avg=14.53`, `avg_tick_ms_avg=82.04`, `loaded_chunks_max=2760` |
| Rejected `ImprovedNoise int[]` experiment | REVERTED | `tps1_avg=15.35`, `avg_tick_ms_avg=53.84`, load-run `Done (49.454s)` |
| Rejected `ImprovedNoise masking-reduction` experiment | REVERTED | `tps1_avg=16.40`, `avg_tick_ms_avg=75.99`, watchdog + sync-load hit |
| Rejected `NoiseChunk.NoiseInterpolator` cell-fraction lookup experiment | REVERTED | `tps1_avg=17.47`, `avg_tick_ms_avg=82.04`, `loaded_chunks_max=4692`; no watchdog/sync-load hit, but worse than accepted baseline |
| Rejected `Aquifer` air-cache experiment | REVERTED | `tps1_avg=17.94`, `avg_tick_ms_avg=81.51`, `loaded_chunks_max=4644`; no watchdog/sync-load hit, but worse than accepted baseline |
| Rejected `PerlinNoise.wrap` in-range fast path | REVERTED | pinned rerun `tps1_avg=18.66`, `avg_tick_ms_avg=88.06`, `watchdog_thread_dumps=1`; not accepted over `18.27/47.85` |
| Rejected `BlockStateData` map pre-sizing | REVERTED | pinned boot/plugin regression: optimized runtime `17784 ms`, plugin matrix `Done (34.600s)` |
| Rejected `Climate.Node` parameter-field cache | REVERTED | plugin matrix passed, but 50-bot load regressed TPS and hit watchdog: `17.39/47.48/1236`, `watchdog_thread_dumps=1` |
| Rejected `CubicSpline.mapAll` stream cleanup | REVERTED | plugin matrix passed, but 50-bot load regressed badly: `17.45/126.93/968`, `watchdog_thread_dumps=1` |
| Rejected `BlendedNoise.compute` scale rewrite | REVERTED | plugin matrix passed, but 50-bot load regressed: `17.50/90.04/2376`, `watchdog_thread_dumps=1` during `save-all` |
| Rejected `FindTopSurface` thread-local scratch context | REVERTED | plugin matrix passed (`Done (29.155s)`), but 50-bot load regressed versus accepted baseline: `17.67/59.76/2449`; no watchdog/sync-load hit |
| Rejected `preliminarySurfaceLevel` quart-mask rewrite | REVERTED | plugin matrix passed (`Done (35.023s)`), but 50-bot load regressed badly versus accepted baseline: `15.83/108.32/2280`; no watchdog/sync-load hit |
| Rejected `PerlinNoise` active-octaves arrays | REVERTED | plugin matrix passed (`Done (40.923s)`), but 50-bot load regressed badly versus accepted baseline: `16.76/138.50/1126`, `watchdog_thread_dumps=1`; postrevert plugin matrix `Done (42.581s)` |
| Rejected `NoiseChunk.wrap` load factor `0.95F` | REVERTED | plugin matrix passed (`Done (40.881s)`), but 50-bot load regressed versus accepted baseline: `16.85/74.43/1020`, `watchdog_thread_dumps=1`; postrevert plugin matrix `Done (32.291s)` |
| Rejected lazy `NoiseChunk` blend caches | REVERTED | plugin matrix passed (`Done (48.547s)`), but 50-bot load regressed versus accepted baseline: `16.02/65.09/562`, `online_max=34`; no watchdog/sync-load hit; postrevert plugin matrix `Done (41.903s)` |
| Rejected `NoiseChunk` empty-blender blend-cache allocation skip | REVERTED | standalone allocation benchmark improved `430.571 ms` to `10.449 ms` (`41.207x`), but 50-bot load regressed versus accepted baseline: `17.96/158.83/2424`; postrevert build/plugin/restart/forced-ticket/hash passed and postrevert 50-bot was stable but still below baseline: `17.79/86.26/2981` |
| Rejected `Climate.Sampler` combined `SampleState` ThreadLocal | REVERTED | plugin matrix passed (`Done (32.711s)`), but 50-bot load regressed versus accepted baseline: `16.91/96.16/1993`; no watchdog/sync-load hit; postrevert plugin matrix `Done (30.549s)` |
| Rejected `PAPER_CHUNK_IO_THREADS=2` config gate | REJECTED | config-only run regressed versus accepted baseline: `16.96/74.18/861`, `watchdog_thread_dumps=1`; no default change made |
| Rejected unlimited chunk load/send/gen rates | REJECTED | config-only run improved average tick time but failed accepted TPS/chunks: `17.16/42.69/1565`; no watchdog/sync-load hit; no default change made |
| Rejected `ImprovedNoise.gradDot` inline | REVERTED | plugin matrix passed (`Done (30.132s)`), but 50-bot load regressed versus accepted baseline: `17.37/103.93/2312`; no watchdog/sync-load hit; postrevert plugin matrix `Done (33.371s)` |
| Rejected `Mth.lerp2/lerp3` inline | REVERTED | plugin matrix passed (`Done (29.892s)`), but 50-bot load did not beat accepted baseline: `18.02/43.93/1625`; no watchdog/sync-load hit; postrevert plugin matrix `Done (30.460s)` |
| Rejected `SurfaceRules.SequenceRule` indexed iteration | REVERTED | plugin matrix passed (`Done (31.894s)`), but 50-bot load hit watchdog and lowered coverage: `18.79/38.68/1216`; postrevert plugin matrix `Done (44.811s)` |
| Rejected `PalettedContainer.reencodeContents` zero-storage branch | REVERTED | plugin matrix passed (`Done (47.375s)`), but 50-bot load regressed badly: `16.32/112.44/1430`, `watchdog_thread_dumps=1`, `sync_load_stack_hits=1`; postrevert plugin matrix `Done (36.608s)` |
| Accepted `PalettedContainer.reencodeContents` scratch reuse | PASS WITH LIMITS | separate from the rejected zero-storage branch: preserves the same remap loop, only reuses the temporary `int[]`; build/plugin/restart/forced-ticket/hash passed, strict 50-bot run had no kicks/errors/watchdog/sync-load but failed accepted baseline: `16.82/154.53/2127` |
| Rejected `SimpleBitStorage` direct packed reencode | REJECTED | equivalence passed, but microbench regressed to `858.637 ms`, slower than old `728.576 ms` and current scratch `244.271 ms`; production code was reverted to scratch-only and `applyPatches`/hash verification passed |
| Rejected spectator no-sync-load movement path | REVERTED | plugin matrix passed (`Done (42.489s)`) and load had no watchdog/sync-load, but failed accepted baseline: `17.16/50.81/1266`; postrevert plugin matrix `Done (31.605s)` |
| Rejected `VarInt.write`/`VarLong.write` branch expansion | REVERTED | algorithm equivalence and temporary functional gates passed, but direct Netty `ByteBuf` microbench regressed: VarInt `5.326 ms -> 5.992 ms` (`0.889x`), VarLong `6.844 ms -> 8.250 ms` (`0.830x`); final artifact rebuilt after revert |
| Direct `MessageDigest` stream SHA-256 | PASS WITH LIMITS | real-jar microbench kept `Path` hashing on Guava (`direct_path_speedup=0.867x`) and showed only a small direct `InputStream` edge (`direct_stream_speedup=1.004x`); build/plugin/restart/forced-ticket passed; no end-to-end startup claim on busy host |
| Precomputed reversed mappings cache | PASS WITH LIMITS | functional cache hit and A/B `34.734s` enabled vs `34.950s` disabled; delta too small to claim speedup |
| Plugin remapper SHA cache reuse | PASS WITH LIMITS | full build passed, plugin matrix `Done (33.147s)`, hash microbenchmark improved `182.522 ms` old two-pass best to `25.707 ms` one-pass parallel best; no end-to-end startup claim until clean A/B |
| Precompute harness cache-hit repair | PASS | fixed missing fresh-run reversed mappings expectation when `run.sh` loads an existing precomputed reversed mappings file; second full build and plugin matrix passed |
| Precomputed plugin skip cache | PASS WITH LIMITS | precompute produced `precomputed_plugin_skips=7`; skip-enabled matrix passed at `Done (32.401s)` with fresh index `hashes=4`, `skippedHashes=7`; control without skip file was `Done (29.630s)`, so no end-to-end speed claim |
| Plugin remapper batch-miss hash reuse | PASS WITH LIMITS | build passed and pinned plugin matrix passed at `Done (32.998s)`; this removes duplicate jar SHA reads in the remap/skip miss path, but no clean end-to-end startup speed claim is made on the busy host |
| Streaming InputStream SHA-256 | PASS WITH LIMITS | build passed, plugin matrix `Done (32.998s)`, restart/recovery `Done (18.470s)`; this removes a full `byte[]` copy while hashing streams, but no end-to-end speed claim is made |
| Plugin library remap cache | PASS WITH LIMITS | build passed, precompute generated `precomputed_library_skips=1`, plugin matrix `Done (45.204s)`, restart/recovery `Done (22.182s)`; this removes repeated library remap inspection for exact SHA cache hits, but no end-to-end speed claim is made |
| Plugin remapper deferred mappings load | PASS WITH LIMITS | build passed, plugin matrix `Done (32.836s)`, restart/recovery `Done (26.519s)`, forced-ticket persistence PASS; targeted skip-only debug run logged no-namespace plugin/library skip and `mapping_load=not_started_for_skip_only`; no end-to-end startup claim on busy host |
| Persistent ticket save packing | PASS WITH LIMITS | final build passed, plugin matrix `Done (31.880s)`, restart/recovery `Done (18.614s)`, forced-ticket restart persistence passed; noisy 50-bot rerun had no watchdog/sync-load but did not beat baseline |
| Waypoint azimuth/distance hot path | PASS WITH LIMITS | build passed, source after `applyPatches` contains direct azimuth arithmetic, axis early-out, and inner half-range early-false guards; plugin matrix `Done (32.097s)`, restart/recovery `Done (22.366s)`, forced-ticket persistence PASS, noisy 2-bot smoke had `online_max=2`, kicks/errors `0`; strict load preflight blocked, so no performance claim yet |
| Rejected `NoiseChunk.FlatCache` reusable context | REVERTED | build/plugin/restart/forced-ticket gates passed, but noisy 50-bot gate hit `watchdog_thread_dumps=3` and `loaded_chunks_max=385`; patch `0038` was deleted and artifact rebuilt |
| Load host preflight | PASS | latest strict 50-bot run passed preflight with `load1=6.17`, `load_per_cpu=0.514`, `idle_percent_1s=77.21` |
| Noisy diagnostic 50-bot gate | PASS WITH LIMITS | intentionally forced with `LOAD_TEST_ALLOW_BUSY_HOST=true`; 50 connected/ready/active, `tps1_avg=17.84`, `avg_tick_ms_avg=38.70`, `loaded_chunks_max=139`, `watchdog_thread_dumps=1` during `save-all`; not comparable to clean baseline |
| Final reverted load rerun | PASS WITH LIMITS | `tps1_avg=18.52`, `avg_tick_ms_avg=61.43`, `loaded_chunks_max=3567`, no watchdog/sync-load hits; TPS/chunks improved, avg tick worsened |
| Postrevert rebaseline after reverting rejected `0036` | PASS WITH LIMITS | `tps1_avg=17.49`, `avg_tick_ms_avg=58.12`, `loaded_chunks_max=2472`, `watchdog_thread_dumps=0`, `sync_load_stack_hits=0` |

## SurfaceRules SequenceRule Array Indexed Loop Candidate

Commands:

```bash
./scripts/bench_surfacerules_sequence_array.sh
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-surfacerules-array-index-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=10bots-surfacerules-array-index-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=10 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Results:

```text
bench: array_best_ms=314.925, array_indexed_best_ms=309.618, array_indexed_speedup=1.898x, equivalence=PASS
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (30.085s)
restart/recovery: PASS, Done (20.836s)
forced-ticket persistence: PASS, first/restart Done (16.195s)/(12.205s)
strict 50-bot gate: BLOCKED by host preflight, load1=12.04, load_per_cpu=1.004, idle_percent_1s=41.34
noisy 10-bot smoke: PASS WITH LIMITS, tps1_avg=18.61, avg_tick_ms_avg=243.85, loaded_chunks_max=2492, watchdog_thread_dumps=0, sync_load_stack_hits=0
```

Current verdict: pending, not accepted as load-performance evidence.

## SurfaceRules SequenceRule Array Candidate

Commands:

```bash
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-surfacerules-sequence-array-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-surfacerules-sequence-array-noisy LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Results:

```text
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (33.652s)
restart/recovery: PASS, Done (20.881s)
forced-ticket persistence: PASS, first/restart Done (16.478s)/(13.288s)
strict 50-bot gate: BLOCKED by host preflight, load1=16.85, load_per_cpu=1.404, idle_percent_1s=49.27
noisy 50-bot gate: PASS WITH LIMITS, tps1_avg=16.75, avg_tick_ms_avg=64.76, loaded_chunks_max=1571, watchdog_thread_dumps=0, sync_load_stack_hits=0
```

Current verdict: pending, not accepted as a load-performance win.

## Minimum Gate Notes

- The join client is real protocol traffic via `minecraft-protocol@1.66.0`, not a Bukkit mock.
- The plugin matrix uses real jars.
- Online-mode authentication is not tested because the harness intentionally runs localhost offline-mode.
- EssentialsX, ProtocolLib and WorldEdit emit upstream compatibility warnings for this Minecraft version; those are recorded as limitations, not hidden.
- The current best load run is not vanilla parity evidence; it is only performance/load evidence.
- The plugin-remapper SHA cache reuse evidence is only a remapper hash-path microbenchmark plus plugin-matrix compatibility, not a full cold-boot performance claim.
- The deferred mappings-load evidence includes a targeted skip-only debug run, but the full matrix startup timing is still noisy and not a clean cold-boot speed claim.
- The precomputed plugin skip cache is exact-SHA compatibility infrastructure; current A/B was noisy and did not prove startup speedup.
- Persistent ticket save packing is covered by a dedicated forced-chunk restart gate because it changes the `chunks.dat` serialization path. Portal persistence still needs a targeted portal-ticket scenario before making a broader claim than forced/portal code-path preservation.
- The historical 500-bot release target was met only for the measured profile:
  cold/fresh plus warm-source, 32/32, creative block workload,
  then-current artifact, worker10/send60/gen20. It is not current-artifact
  evidence now. Long soak, real-player behavior, broad plugin/gameplay
  coverage, and literal stable `20.00 TPS` remain separate unclaimed targets.
- The waypoint/remapper/ReobfServer/hardlink/plugin-scan/plugin-metadata/load-order candidates have compatibility and microbench evidence; their older 50-bot verdicts remain historical and are superseded for the measured 500-bot release claim by `reports/production-500-release-gate.txt`.
- `run_load_test.sh` now blocks by default on a busy host. Use `LOAD_TEST_ALLOW_BUSY_HOST=true` only when intentionally producing a non-comparable noisy run.
- `reports/load-50bots-20260507-noisy-current-final-summary.txt` and `.jfr` are retained only as noisy diagnostics. The run identified `TicketStorage.packTickets()` during `save-all` as a save-path hotspot, but it is not baseline evidence.

## NoiseChunk Interpolator Array Candidate

Commands:

```bash
./scripts/bench_noisechunk_interpolator_array.sh
cd /root/rust/upstream/Paper && ./gradlew paper-server:fixupSourcePatches paper-server:rebuildSourcePatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-noisechunk-interpolator-array-postreject LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=1 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 ./scripts/run_load_test.sh
```

Results:

```text
bench: enhanced_for_loop_best_ms=1137.729, indexed_list_loop_best_ms=1158.104, array_loop_best_ms=1164.487
bench: array_vs_enhanced_for_speedup=0.977x, array_vs_indexed_list_speedup=0.995x, equivalence=PASS
applyPatches after patch-stack repair: PASS, Applied 912 patches
runtime code after rejection: no interpolatorsArray/interpolatorArray() in generated NoiseChunk.java or source patch
build_optimized.sh: PASS
artifacts.json validation: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=0.940, idle_percent_1s=38.09
```

Current verdict: rejected and reverted. The corrected microbenchmark showed the
array snapshot path slower than the existing enhanced-for loop, so no load
performance claim is made.

## Persistent Ticket Pack Direct Append

Commands:

```bash
./scripts/bench_ticket_pack.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-ticket-pack-direct-gate-20260510 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Results:

```text
bench: callback_pack_best_ms=3631.163, direct_pack_best_ms=3239.248, direct_speedup=1.121x, equivalence=PASS
patch stack: PASS, applyPatches Applied 912 patches
build_optimized.sh: PASS
artifacts.json validation: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
forced-ticket persistence: PASS, first/restart Done (18.168s)/(14.334s)
plugin matrix: PASS, Done (41.256s), CompatProbe command/events/scheduler/join passed
restart/recovery: PASS, Done (19.539s)
strict 50-bot gate: BLOCKED by host preflight, load_per_cpu=1.139 > 0.750
```

Current verdict: accepted only as a narrow persistent-ticket save-path work
reduction. It is not a load/TPS baseline or 500-bot claim until a clean bot gate
proves that separately.

## Rejected ChunkDependencies Radius Lookup

Commands:

```bash
./scripts/bench_chunk_dependencies_array.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json >/dev/null
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-chunkdependencies-radius-lookup-gate-20260510 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Results:

```text
bench: old_immutable_list_get_best_ms=419.919, array_get_best_ms=341.251, array_get_speedup=1.231x, equivalence=PASS
patch stack: PASS, rebuildPatches Saved modified patches (48/51)
applyPatches: PASS, Applied 912 source patches and 51 feature patches
candidate post-apply source check: PASS, dependencyByRadiusArray existed before rejection
build_optimized.sh: PASS
artifact JSON/hash checks: PASS
plugin matrix: PASS, Done (27.278s)
restart/recovery: PASS, Done (16.842s)
forced-ticket persistence: PASS, first/restart Done (13.206s)/(9.446s)
50-bot gate: online_max=50, tps1_avg=17.89, avg_tick_ms_avg=57.67, loaded_chunks_max=2792, watchdog_thread_dumps=3, sync_load_stack_hits=0, bot_kicked_max=0, bot_errors_max=0
```

Current verdict: rejected and removed from production source. The microbench
won, but the real 50-bot 32/32 gate failed the accepted load baseline and hit
watchdog dumps in movement/ticket paths.

Post-rejection verification: `applyPatches` PASS with no `0051/0052` feature
patches, final `build_optimized.sh` PASS, AppCDS generated, artifact JSON
valid, `sha256sum -c reports/artifact-hashes.txt` PASS, and no
`dependencyByRadiusArray` remains in generated source or feature patches.

## Rejected ImprovedNoise Derivative Flat Gradient

Commands:

```bash
bash scripts/bench_improved_noise_derivative.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py
python3 -m json.tool reports/artifacts.json >/dev/null
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true scripts/run_plugin_matrix.sh
MC_EULA_AGREE=true scripts/restart_recovery_check.sh
MC_EULA_AGREE=true scripts/forced_ticket_persistence_check.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-improvednoise-derivative-flat-gradient-gate-20260510 BOT_COUNT=50 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 DURATION_SECONDS=120 scripts/run_load_test.sh
```

Results:

```text
bench: old_derivative_best_ms=53.103, flat_gradient_derivative_best_ms=50.027, flat_gradient_derivative_speedup=1.061x, equivalence=PASS
patch stack: PASS, candidate 0051-Optimize-ImprovedNoise-derivative-gradients.patch applied
build_optimized.sh: PASS
artifact JSON/hash checks: PASS
plugin matrix: PASS, Done (26.221s)
restart/recovery: PASS, Done (17.577s)
forced-ticket persistence: PASS, first/restart Done (13.630s)/(9.101s)
50-bot gate: preflight PASS, online_max=50, tps1_avg=15.36, avg_tick_ms_avg=94.24, loaded_chunks_max=3850, watchdog_thread_dumps=2, nearby_players_stack_hits=8, sync_load_stack_hits=0, bot_kicked_max=0, bot_errors_max=0
```

Current verdict: rejected and removed from production source. The production
patch stack is back to `0050-Optimize-persistent-ticket-pack-direct-append.patch`;
post-rejection `build_optimized.sh`, artifact JSON validation, and
`sha256sum -c reports/artifact-hashes.txt` passed. Current optimized jar hash:
`a8e0d476f77a86fb6f94db670d351cd5bcd66239bcc2452074b705e847fcbaf6`.

## CompoundTag Map Initial Capacity Bench

Command:

```bash
NBT_CHUNK_SAMPLE_LIMIT=512 NBT_REGION_SAMPLE_LIMIT=24 bash scripts/bench_nbt_compound_map_capacity.sh
```

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
```

Current verdict: rejected before production. `cap8` is the current Paper source
shape and remained fastest, so no source patch was created.

## Native Rust Batch Verification

Commands:

```bash
cargo test --manifest-path native/Cargo.toml --workspace
JAVA_PROPS='-DmapBenchIterations=100 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_noisechunk_wrap_capacity.sh
./scripts/bench_native_deflater_input_shape.sh
```

Results:

```text
cargo test --manifest-path native/Cargo.toml --workspace: PASS, 275 tests
bench_native_noisechunk_wrap_capacity.sh: PASS, equivalence=PASS, script_status=PASS
bench_native_deflater_input_shape.sh: PASS, equivalence=PASS, script_status=PASS
```
