# Benchmarks

Historical artifact snapshot recorded on 2026-05-23 (superseded; not
current-artifact evidence):

```text
optimized_artifact_sha256=ece63dbd93423ac5797e439b54680c4d0a08b3f34f95d3de505cd375940b9ecc
```

The old P500 publication files and `production-500-readiness-bundle-current`
are historical/stale report names, not current-artifact evidence, until
regenerated from fresh green same-artifact P500 evidence.

## Current 2026-05-21 CEST Split Perlin Runtime Smoke

Fresh runtime smoke on the rebuilt artifact:

```text
optimized_artifact_sha256=f418a8e0cc2b5bbd9cbaf28721b3de2e368b3d13a56cd72854d6d6bd47d87169
optimized_runtime_run_sh_sha256=734918063b0a8c11e4684814d5cc5670b8b4bad7de5f5279895332f11cbd9920
```

Evidence:

- `reports/load-1bots-vd32-sd32-summary.txt`
- `reports/load-perlin-generic-smoke-20260521-summary.txt`
- `reports/native-perlin-getvalue-bench.txt`

The no-y-scale smoke loaded `native_perlin_noise_no_y_scale=true` with
`native_perlin_noise_generic=false`, zero watchdog dumps, zero sync-load
hits, and zero stability failures. The generic-only smoke loaded
`native_perlin_noise_generic=true` and also reported the no-y-scale symbol as
available, which is expected in the current split implementation because the
generic request also verifies that symbol path. The fresh Perlin bench keeps
the parity check green and shows the split direct no-y-scale path as a real
native hot-path win, but this is still just module-level proof, not a server
scale claim.

## 2026-05-19 CEST Production-Ready 500 Refresh (historical evidence)

Later artifact snapshot recorded in this historical section (superseded; not
active evidence now):

```text
optimized_artifact_sha256=ece63dbd93423ac5797e439b54680c4d0a08b3f34f95d3de505cd375940b9ecc
```

The old 2026-05-19 refresh below is retained as historical evidence:

```text
optimized_artifact_sha256=d4b27d49c9aba3502b46cf75637f1fe2a4707143a1f01afbbf7315bed52b2efa
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

That historical 500-bot refresh was claimable for the measured profile only.
The accepted claim was:

```text
production-ready for measured 500 bots / 32 view / 32 simulation / creative block
```

This was backed by a fresh `2400s` cold+warm soak, a fresh three-pass repeat
quorum, plugin matrix, restart/recovery, forced-ticket persistence, bundle
validation, claim assertion, and published claim files. The repeat-quorum
harness now keeps old-artifact repeat dirs strictly historical instead of
mixing them into an active baseline.

Final readiness report:

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
artifact_hash_count=12
```

Evidence:

- `reports/production-500-readiness-gate.txt`
- `reports/production-500-readiness-bundle-20260519-040502`
- `reports/production-500-readiness-bundle-current` (historical/stale until
  regenerated from fresh same-artifact evidence)
- `reports/production-500-claim-verdict-20260519-040502.txt`
- `reports/production-500-claim-current.txt` (historical/stale until
  regenerated from fresh same-artifact evidence)
- `reports/production-500-claim-current.md` (historical/stale until
  regenerated from fresh same-artifact evidence)
- `reports/production-500-claim-current.json` (historical/stale until
  regenerated from fresh same-artifact evidence)

Fresh `2400s` soak metrics:

| surface | samples | TPS avg/min | MSPT max | block place/dig | online/chunks | failures |
| --- | ---: | --- | ---: | --- | --- | ---: |
| cold/fresh | 356 | `19.77 / 18.86` | `66.82` | `399000 / 398500` | `500 / 5476` | `0` |
| warm-source | 358 | `19.92 / 19.38` | `56.90` | `407000 / 407000` | `500 / 5476` | `0` |

Fresh repeat quorum from `reports/release-repeat-20260519-022412`
(historical `d4b27...` repeat dir):

| run | cold TPS avg/min/MSPT max | warm TPS avg/min/MSPT max | artifact |
| --- | --- | --- | --- |
| 1 | `19.75 / 19.18 / 71.48` | `19.76 / 19.30 / 59.26` | `d4b27d49...` |
| 2 | `19.86 / 19.67 / 54.93` | `19.92 / 19.65 / 50.04` | `d4b27d49...` |
| 3 | `19.92 / 19.70 / 53.35` | `19.81 / 19.55 / 56.28` | `d4b27d49...` |

This still does not claim a full Rust Paper runtime, literal unlimited scale,
unbounded plugin compatibility, unmeasured real-player gameplay, or multi-hour
soak behavior.

## Current 2026-05-18 CEST Extreme P100/P250 Stress Mixed

The stress-corpus artifact for this track remains:

```text
optimized_artifact_sha256=68c170ae8313396beb38603ca69ef526a732b370ff0eeba34212b9d926a667ac
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

Fresh extreme stress-corpus runs with `26` plugin jars and `10` datapacks:

| run | bots | mobs | speed / interval | online max | TPS avg/min | MSPT avg/max | watchdog / sync-load | gate |
| --- | ---: | ---: | --- | ---: | --- | --- | --- | --- |
| `p100-current-uncapped` | 100 | 150 | `48 / 100ms` | 100 | `12.95 / 2.22` | `110.95 / 545.11` | `2 / 0` | fail |
| `p100-worker10-send60-gen20` | 100 | 150 | `48 / 100ms` | 100 | `12.48 / 3.12` | `104.91 / 481.90` | `1 / 0` | fail |
| `p250-worker10-send60-gen20-mob300` | 250 | 300 | `48 / 100ms` | 229 | `12.32 / 7.58` | `79.43 / 144.78` | `0 / 0` | fail |
| `p250-worker10-send60-gen20-mob300-slowmove` | 250 | 300 | `12 / 500ms` | 218 | `13.53 / 9.35` | `112.30 / 1237.58` | `0 / 0` | fail |

Evidence files:

- `reports/load-extreme-stress-mixed-worker10-send60-gen20-mob300-harnessfix-250-20260518-055503-summary.txt`
- `reports/extreme-stress-mixed-worker10-send60-gen20-mob300-harnessfix-250-20260518-055503-gate.txt`
- `reports/load-extreme-stress-mixed-worker10-send60-gen20-mob300-slowmove-250-20260518-060053-summary.txt`
- `reports/extreme-stress-mixed-worker10-send60-gen20-mob300-slowmove-250-20260518-060053-gate.txt`
- `reports/compare-p100-worker10-vs-p250-worker10-mob300-20260518.txt`
- `reports/compare-p250-harnessfix-vs-slowmove-20260518.txt`

The P250 rung is measured and still red. The useful improvement is that the
worker10/send60/gen20 profile keeps `watchdog_thread_dumps=0` and
`sync_load_stack_hits=0` at P250, but it misses full active online, TPS, and
movement-warning gates. Slow movement reduced warning count only partially and
worsened active online plus MSPT tail, so it is not the next accepted fix.

## Current 2026-05-18 CEST Mixed-Gameplay 50-Bot Pass

The current mixed-gameplay gate is now green on the fresh artifact:

```text
optimized_artifact_sha256=68c170ae8313396beb38603ca69ef526a732b370ff0eeba34212b9d926a667ac
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

Fresh 50-bot stress-corpus mixed-gameplay comparison:

| run | native ImprovedNoise | native PerlinNoise | TPS avg/min | MSPT avg/max | RSS max | gate |
| --- | --- | --- | --- | --- | ---: | --- |
| `default-improved-20260517-182723` | true | false | `15.22 / 5.14` | `48.35 / 92.19` | `23743.7` | fail |
| `dfc-forceload-async-20260518` | true | false | `18.40 / 15.88` | `26.79 / 78.32` | `5503.3` | pass |

`reports/compare-default-improved-vs-dfc-forceload-async-20260518.txt`
captures the same comparison in machine-readable form. The fresh artifact
reaches `50/50` bots with `26` plugins, `10` datapacks, `150` mobs, zero
kicks/errors/watchdog/sync-load/stability failures, and `native_improved_noise_loaded=true`.

## Previous 2026-05-17 CEST Mixed-Gameplay Native ImprovedNoise A/B

The current mixed-gameplay work is still below the production/scale gate, but
the runtime now has one measured default improvement:

```text
optimized_artifact_sha256=b58b307f17ee68e868105473a393d3696ac6c5356fd9afa27d3e9a4188681bc0
optimized_runtime_run_sh_sha256=b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a
```

`scripts/prepare_fast_runtime.sh` now defaults to:

```text
PAPER_NATIVE_IMPROVED_NOISE=true
PAPER_NATIVE_PERLIN_NOISE=false
```

Paired 50-bot stress-corpus mixed-gameplay comparison:

| run | native ImprovedNoise | native PerlinNoise | TPS avg/min | MSPT avg/max | RSS max | gate |
| --- | --- | --- | --- | --- | ---: | --- |
| `current-nonative-20260517-182054` | false | false | `14.73 / 5.17` | `48.86 / 108.19` | `24299.0` | fail |
| `default-improved-20260517-182723` | true | false | `15.22 / 5.14` | `48.35 / 92.19` | `23743.7` | fail |

The default-improved run reached 50/50 bots with 26 plugins, 10 datapacks,
150 mobs, zero kicks/errors/watchdog/sync-load/stability failures, and
`native_improved_noise_loaded=true`. It still failed the mixed-gameplay gate
on TPS avg/min, so it is an accepted incremental runtime change, not a claim.

Rejected A/B evidence:

- `holdercache-20260517-180208` regressed against `noisechunk8192`.
  Report: `reports/compare-noisechunk8192-vs-holdercache-20260517.txt`.
- `native-noise-20260517-181058` with both ImprovedNoise and PerlinNoise
  enabled regressed against `noisechunk8192`.
  Report: `reports/compare-noisechunk8192-vs-native-both-20260517.txt`.

## Previous 2026-05-17 CEST Production-Ready 500 Certification Gate

The historical top-level benchmark/certification result for this run was:

```bash
MC_EULA_AGREE=true ./scripts/run_production_readiness_gate.sh
```

Result:

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
bundle_validation_pass=true
claim_assertion_pass=true
claim_publication_pass=true
evidence_file_count=8
optimized_artifact_sha256=4064700022a879d83b16323cfbd0a769caf4551fdd8ed21dc7332afdd39d6b47
soak_report_sha256=d0700e75d6588f36e79ad5bbe8ce64ecc16c8677a7f41be40c74d23255449c3e
repeat_report_sha256=dabe75757ddcb4153fb8b91c29e45a667fddd76ce491a69bb7bdeb58786e44cc
plugin_matrix_summary_sha256=0273efad76e154ad13421ce75e158991830bc85e05bd761c3f9931765eacf301
restart_recovery_summary_sha256=476bd7bfd258f2dc653648a2a3034f2910f0845b264305189096679ebbc05d29
forced_ticket_summary_sha256=b9e2ac162245d07e77f8f4be45897bfcb5a007606d8e2aed48399511a7e89882
```

Benchmark/load layer preserved inside the readiness report:

| surface | TPS avg/min | MSPT max | block place/dig | online/chunks |
| --- | --- | ---: | --- | --- |
| cold/fresh soak | `19.84 / 19.19` | `60.48` | `264000 / 264000` | `500 / 5476` |
| warm-source soak | `19.95 / 19.28` | `56.32` | `267500 / 267000` | `500 / 5476` |

Compatibility/recovery layer from fresh runs:

| gate | result |
| --- | --- |
| plugin matrix | `Done (21.929s)`, `Initialized 11 plugins`, `COMPAT_PROBE command=ok events=4` |
| restart/recovery | `Done (15.527s)`, `Saved the game`, `COMPAT_PROBE command=ok events=2` |
| forced-ticket persistence | `PASS`, first/restart `Done (11.386s) / Done (8.551s)` |

Verdict: this supported the measured production-ready claim for the 500-bot
32/32 creative block workload on the then-verified artifact. It is a
historical snapshot, not current-artifact evidence now. The tested
plugin-matrix/restart/forced-ticket support still did not claim a full Rust
rewrite of Paper, unlimited plugin compatibility, unmeasured real-player
gameplay, or multi-hour soak behavior.

The export bundle for this run is
`reports/production-500-readiness-bundle-20260517-091520`, with `CLAIM.md`,
`MANIFEST.txt`, `bundle.json`, and copied evidence files. The bundle validates
with:

```bash
python3 scripts/validate_production_readiness_bundle.py \
  reports/production-500-readiness-bundle-20260517-091520
python3 scripts/assert_production_ready_claim.py \
  reports/production-500-readiness-bundle-20260517-091520
scripts/production_ready_claim.sh
python3 scripts/publish_production_ready_claim.py
```

The claim verdict is
`reports/production-500-claim-verdict-20260517-091520.txt` and has
`claim_assertion_pass=true`. The published file names for that historical run
were `reports/production-500-claim-current.txt`,
`reports/production-500-claim-current.md`, and
`reports/production-500-claim-current.json`; the `current` suffix is a report
name here, not proof that the claim is current-artifact evidence now.

## Previous 2026-05-17 CEST Production 500 Soak Gate

The historical top benchmark evidence for the measured `500 bots / production
ready` statement was a cold/fresh plus warm-source soak gate. It used the same
accepted release profile as the three-pass quorum, but extended each load
surface to a dynamically enforced soak duration floor. With the default
`BOT_BLOCK_RAMP_SECONDS=600`, `PRODUCTION_SOAK_MIN_LOAD_WINDOW_SAMPLES=300`,
`PRODUCTION_SOAK_METRICS_SAMPLE_INTERVAL_SECONDS=5`, and
`PRODUCTION_SOAK_DURATION_BUFFER_SECONDS=300`, the default floor is
`2400` seconds per cold/warm surface. Shorter explicit
`PRODUCTION_SOAK_DURATION_SECONDS` values are rejected before the run starts.
The gate also requires at least 120000 block place and dig packets on both
surfaces:

```bash
MC_EULA_AGREE=true ./scripts/run_production_soak_gate.sh

python3 scripts/evaluate_production_soak.py \
  --cold-summary reports/load-production-500-cold-soak-current-artifact-20260517-052252-summary.txt \
  --warm-summary reports/load-production-500-warm-soak-current-artifact-20260517-052252-summary.txt \
  --artifact-hashes reports/artifact-hashes.txt \
  --artifacts-json reports/artifacts.json \
  --require-current-artifacts \
  --report reports/production-500-soak-gate.txt
```

Soak report:

```text
production_ready_soak_claim_eligible=true
soak_gate_pass=true
failure_count=0
base_cold_gate_pass=true
base_warm_gate_pass=true
artifact_hashes_pass=true
optimized_artifact_sha256=4064700022a879d83b16323cfbd0a769caf4551fdd8ed21dc7332afdd39d6b47
```

Load-window benchmark matrix:

| surface | TPS avg/min | MSPT avg/max | block place/dig | online/chunks | failures |
| --- | --- | --- | --- | --- | --- |
| cold/fresh | `19.84 / 19.19` | `41.98 / 60.48` | `264000 / 264000` | `500 / 5476` | `0` |
| warm-source | `19.95 / 19.28` | `38.68 / 56.32` | `267500 / 267000` | `500 / 5476` | `0` |

Verdict: this supported the measured 500-bot production-ready claim for the
32/32 creative block workload on the then-verified artifact, not
current-artifact evidence now, with
`PAPER_CHUNK_WORKER_THREADS=10`, `PAPER_PLAYER_MAX_SEND_RATE=60`, and
`PAPER_PLAYER_MAX_GEN_RATE=20`. It does not claim a full Rust rewrite of
Paper, unlimited plugin compatibility, or gameplay outside the measured
profile.

## Previous 2026-05-17 CEST Production 500 Release Gate

The release verifier in this historical run combined the then-recorded
artifact worker10 cold/fresh and warm-source 500-bot gates with artifact hash
verification. The accepted profile used `PAPER_PLAYER_MAX_SEND_RATE=60` and
`PAPER_PLAYER_MAX_GEN_RATE=20`. A separate repeat verifier now requires at
least three preserved release passes before the measured production claim is
treated as quorum-backed:

```bash
MC_EULA_AGREE=true ./scripts/run_production_release_gate.sh

python3 scripts/evaluate_production_release.py \
  --cold-summary reports/load-production-500-cold-repeat-20260517-033126-run1-20260517-033126-summary.txt \
  --warm-summary reports/load-production-500-warm-repeat-20260517-033126-run1-20260517-033126-summary.txt \
  --artifact-hashes reports/artifact-hashes.txt \
  --artifacts-json reports/artifacts.json \
  --require-current-artifacts \
  --report reports/production-500-release-gate.txt

python3 scripts/evaluate_production_release_repeat.py \
  --repeat-dir auto \
  --min-passes 3 \
  --report reports/production-500-repeat-quorum.txt
```

Latest single release gate after the repeat run:

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
cold_process_rss_mib_max=11993.3
cold_bot_block_place_packets_max=63000
cold_bot_block_dig_packets_max=63000
warm_gate_pass=true
warm_load_window_online_max=500
warm_load_window_loaded_chunks_max=5476
warm_load_window_tps1_avg=19.90
warm_load_window_tps1_min=19.33
warm_load_window_avg_tick_ms_avg=36.61
warm_load_window_avg_tick_ms_max=56.58
warm_process_rss_mib_max=5116.1
warm_bot_block_place_packets_max=63500
warm_bot_block_dig_packets_max=63500
cold_watchdog_thread_dumps=0
cold_sync_load_stack_hits=0
cold_stability_failures=0
warm_watchdog_thread_dumps=0
warm_sync_load_stack_hits=0
warm_stability_failures=0
```

Verdict: this supported the measured 500-bot production claim for the 32/32
creative block workload on the then-verified artifact, not
current-artifact evidence now, with
`PAPER_CHUNK_WORKER_THREADS=10`, `PAPER_PLAYER_MAX_SEND_RATE=60`, and
`PAPER_PLAYER_MAX_GEN_RATE=20`. The default-generation send60 full release run
failed cold/fresh on `load_window_tps1_min=17.92`, so gen-rate 20 is part of
the accepted profile, not an optional detail. This still does not claim a full
Rust Paper runtime, unlimited plugin compatibility, or gameplay outside the
measured profile.

Repeat evidence is preserved under `reports/release-repeat-20260517-033126/run-1`,
`reports/release-repeat-20260517-041001/run-1`, and
`reports/release-repeat-20260517-041001/run-2`. The quorum report
`reports/production-500-repeat-quorum.txt` passed with `required_min_passes=3`,
`repeat_run_count=3`, `repeat_passes=3`, `repeat_failures=0`, and
`repeat_quorum_pass=true`.

Repeat matrix:

| run | cold TPS avg/min/max MSPT | warm TPS avg/min/max MSPT |
| --- | --- | --- |
| 1 | `19.84 / 18.62 / 61.17` | `19.88 / 19.32 / 53.27` |
| 2 | `19.91 / 18.72 / 54.87` | `19.90 / 19.12 / 59.48` |
| 3 | `19.84 / 19.06 / 55.86` | `19.90 / 19.33 / 56.58` |

## Previous 2026-05-16 Fresh 500 Cold Gate Pass With Load-Window Metrics

The load harness now records both overall metrics and `load_window_*` metrics.
The production evaluator prefers `load_window_*` when present. That window
starts at server metrics collection and stops before post-run teardown, at the
first metrics sample where online count drops after reaching the target. This
keeps cold ramp, chunk generation, and the 500-online block phase in the gate
while keeping shutdown/disconnect artifacts visible as separate diagnostics.

Command:

```bash
MC_EULA_AGREE=true \
  PAPER_CHUNK_WORKER_THREADS=8 \
  LOAD_TEST_LABEL=production-500-cold-worker8-defaultheap-windowed-20260516-223952 \
  ./scripts/run_production_claim_gate.sh
```

Gate report:

```text
report=reports/load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-summary.txt
gate=reports/load-production-500-cold-worker8-defaultheap-windowed-20260516-223952-gate.txt
claim_eligible=true
gate_pass=true
failure_count=0
world_mode=fresh
claim_surface=cold-fresh
load_window_policy=until_first_online_drop_after_reaching_bots
load_window_reached_full_online=true
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

Rejected nearby candidates:

```text
production-500-cold-worker8-xms16g-20260516-220341:
claim_eligible=false
bot_exit=75
tps1_min=17.02
avg_tick_ms_max=66.77
reason=8 workers plus 16G/pre-touch heap regressed the 500 block phase

production-500-cold-worker8-defaultheap-20260516-221636:
claim_eligible=false under the old teardown-inclusive summary
tps1_min=17.15
avg_tick_ms_max=226.65
reason=post-run disconnect/shutdown tail polluted the production-load metric
```

Verdict: the cold/fresh `production-500` benchmark profile is now backed by a
passing gate for 500 bots at 32/32 with the creative block workload, default
heap, and 8 Paper chunk workers. This does not claim a Rust rewrite of Paper or
unbounded real-player/plugin behavior outside this measured profile.

## Previous 2026-05-16 Native Perlin Runtime Hook Cold Probe

Command shape:

```bash
MC_EULA_AGREE=true PAPER_NATIVE_PERLIN_NOISE=true \
  LOAD_TEST_REQUIRE_NATIVE_MODULES=perlin_noise \
  LOAD_TEST_LABEL=production-500-native-perlin-post0099-20260516-214550 \
  ./scripts/run_production_claim_gate.sh
```

Runtime evidence:

```text
native_perlin_noise=true
paper.nativePerlinNoise=true
Paper: Using native PerlinNoise from paper_native_jni.
```

The run was stopped early because the strict `production-500` gate could no
longer pass after the early sample:

```text
online=59
loadedChunks=3248
tps1=17.65
avgTickMs=84.30
```

Verdict: rejected for production. Native Perlin loaded, but it did not remove
the cold/fresh join/chunk/worldgen spike, so it stays diagnostic and default
off.

## Previous 2026-05-16 Fresh 500 Cold Gate After `0097`

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=production-500-block-500bots-post0097-20260516-200739 ./scripts/run_production_claim_gate.sh
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-production-500-block-500bots-post0097-20260516-200739-summary.txt
```

Gate outcome:

```text
report=reports/load-production-500-block-500bots-post0097-20260516-200739-summary.txt
gate=reports/load-production-500-block-500bots-post0097-20260516-200739-gate.txt
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
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_place_packets_max=60000
bot_block_dig_packets_max=59500
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Failure location from the run log:

```text
min_tps online=75 loadedChunks=5106 tps1=12.80 avgTickMs=112.72
max_tick online=65 loadedChunks=4186 tps1=14.87 avgTickMs=117.43
first_500 online=500 loadedChunks=5476 tps1=19.41 avgTickMs=51.11
final_sample online=500 loadedChunks=5476 tps1=19.92 avgTickMs=46.22
```

This does not support the broad cold/fresh-world `500 bots / production ready`
claim yet. It does show that the remaining blocker is now the early join/load
spike; the final 500-online block-action plateau was close to the strict
thresholds.

## Previous 2026-05-16 Warm 500 Gate Pass After Tracked-Entity Noop Fast Path

Fresh commands from this continuation:

```bash
./gradlew applyPatches :paper-server:compileJava
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt
LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 \
  LOAD_TEST_LABEL=production-500-warm-block-500bots-post0097-20260516-194812 \
  MC_EULA_AGREE=true ./scripts/run_production_warm_claim_gate.sh
```

Build outcome:

```text
applyPatches + compileJava: PASS
build_optimized.sh: PASS
native tests: 291 passed, 0 failed
artifact hashes: PASS
```

Warm 500 gate outcome:

```text
report=reports/load-production-500-warm-block-500bots-post0097-20260516-194812-summary.txt
gate=reports/load-production-500-warm-block-500bots-post0097-20260516-194812-gate.txt
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
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_place_packets_max=60500
bot_block_dig_packets_max=60000
bot_block_action_errors_max=0
```

This is a gate-backed warm-world `500 bots / production ready` claim for the saved/pregenerated world profile. It does not replace the separate cold/fresh-world `production-500` gate.

## Previous 2026-05-16 Entity-Tracker and Ticket Fast Paths

Fresh commands from this continuation:

```bash
bash scripts/bench_ticket_compare.sh
bash scripts/bench_nearest_affects_spawning.sh
BUILD_NATIVE=false scripts/build_optimized.sh
MC_EULA_AGREE=true LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 LOAD_TEST_LABEL=production-500-warm-block-500bots-post0088-20260516-150108 ./scripts/run_production_warm_claim_gate.sh
```

Microbench outcome:

```text
ticket_compare ref_fast_speedup=1.070x
nearest_affects_spawning specialized_speedup=1.067x
ticket_expire equivalence=PASS
```

Build outcome:

```text
0086-0088 rebuild after applyPatches: PASS
build_optimized after patch stack refresh: PASS
```

Warm 500 gate outcome:

```text
report=reports/load-production-500-warm-block-500bots-post0088-20260516-150108-summary.txt
gate=reports/load-production-500-warm-block-500bots-post0088-20260516-150108-gate.txt
claim_eligible=false
failure_count=2
online_max=500
loaded_chunks_max=5476
tps1_avg=18.42
tps1_min=14.66
avg_tick_ms_avg=44.41
avg_tick_ms_max=88.62
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
bot_block_armed_max=500
bot_block_primed_max=500
bot_block_place_packets_max=61000
bot_block_dig_packets_max=60500
```

This is not a `production-ready` claim. It is the closest warm run so far, with TPS still below the strict gate.

## Previous 2026-05-15 Warm 500 JFR After PlayerList Broadcast Tweak

Fresh commands from this continuation:

```bash
MC_EULA_AGREE=true LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 LOAD_TEST_LABEL=warm500-playerlist-jfr2-20260515 LOAD_TEST_SCENARIO=block LOAD_TEST_GAMEMODE=creative BOT_COUNT=500 DURATION_SECONDS=360 BOT_BLOCK_RAMP_SECONDS=180 BOT_BLOCK_ACTION_INTERVAL_MS=1000 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-warm500-playerlist-jfr2-20260515.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
jfr view hot-methods reports/load-warm500-playerlist-jfr2-20260515.jfr | head -n 40
jfr view allocation-by-site reports/load-warm500-playerlist-jfr2-20260515.jfr | head -n 40
```

Observed runtime shape:

```text
online_max=500
loaded_chunks_max=5476
tps1_avg=13.71
avg_tick_ms_avg=98.19
watchdog_thread_dumps=3
nearby_players_stack_hits=5
```

Top CPU methods:

```text
ChunkMap$TrackedEntity.updatePlayer(ServerPlayer) 9.30%
ReferenceOpenHashSet.contains(Object) 6.50%
Entity.getBukkitEntity() 4.40%
HashMap.getNode(Object) 3.23%
TargetingConditions.test(...) 2.58%
CraftPlayer.canSee(Entity) 0.89%
```

Allocation heavy sites:

```text
DirectMethodHandle.allocateInstance(Object) 12.50%
Arrays.copyOf(Object[], int) 7.49%
Arrays.copyOfRangeByte(byte[], int, int) 3.66%
AbstractStringBuilder.<init>(int) 3.15%
InetSocketAddress$InetSocketAddressHolder.toString() 2.80%
```

This JFR says the next target is entity tracking, not PlayerList broadcast.

## Previous 2026-05-15 PlayerList Broadcast Distance-First Candidate

Fresh commands from this continuation:

```bash
./gradlew :paper-server:compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
./scripts/bench_playerlist_broadcast_cansee.sh
MC_EULA_AGREE=true LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 ./scripts/run_production_warm_claim_gate.sh
python3 scripts/evaluate_load_gate.py --profile production-500-warm reports/load-production-500-warm-block-500bots-20260515-221802-summary.txt
```

Microbench outcome:

```text
empty_candidate_speedup=1.213x
populated_candidate_speedup=1.718x
equivalence=PASS
```

Gate result:

```text
gate_profile=production-500-warm
claim_eligible=false
gate_pass=false
online_max=500
loaded_chunks_max=5476
tps1_avg=15.53
tps1_min=5.44
avg_tick_ms_avg=76.90
avg_tick_ms_max=239.69
watchdog_thread_dumps=3
nearby_players_stack_hits=2
```

This is a real improvement, but it still does not clear the 500-bot claim gate.

## Previous 2026-05-15 500-Bot Claim Gate Surfaces

The benchmark harness now records whether a load run used a cold fresh world or
a copied saved world. `production-500` remains the cold/fresh-world gate;
`production-500-warm` is the separate saved/pregenerated-world gate. Both
profiles require the same 500-bot creative block scenario, 32/32 view and
simulation distance, full block-action coverage, `tps1_avg >= 19.5`,
`tps1_min >= 18.0`, `avg_tick_ms_avg <= 50.0`,
`avg_tick_ms_max <= 100.0`, RSS below `28672 MiB`, and zero watchdog,
sync-load, thread, chunk-system, feature-placement, POI, bot-error, or external
thread-print failures.

Warm-world command shape:

```bash
LOAD_TEST_WORLD_SOURCE=runs/load-production-500-block-500bots-20260515-201310 \
  MC_EULA_AGREE=true ./scripts/run_production_warm_claim_gate.sh
```

This command shape is not a pass by itself. A claim still requires the matching
gate report to say `claim_eligible=true`.

Full warm 500 evidence from this continuation:

```text
report=reports/load-production-500-warm-block-500bots-20260515-210412-summary.txt
gate=reports/load-production-500-warm-block-500bots-20260515-210412-gate.txt
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

The earlier profiler-enabled warm run also failed. Disabling the Spark
background profiler made the harness cleaner but did not move the 500-bot
action plateau close to the claim thresholds.

## Previous 2026-05-15 SurfaceRules Sequence Chain Candidate Rejected

Fresh commands from this continuation:

```bash
./scripts/bench_surfacerules_sequence_array.sh
./gradlew applyPatches --no-daemon
./gradlew :paper-server:compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_SCENARIO=block LOAD_TEST_GAMEMODE=creative BOT_COUNT=500 DURATION_SECONDS=600 BOT_BLOCK_RAMP_SECONDS=300 BOT_BLOCK_ACTION_INTERVAL_MS=1000 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_LABEL=block-500-surfacerules-chain-20260515 ./scripts/run_load_test.sh
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-block-500-surfacerules-chain-20260515-summary.txt
```

Microbench outcome:

```text
rules=14
list_enhanced_best_ms=529.303
array_indexed_best_ms=283.931
linked_best_ms=459.239
linked_speedup=1.153x
equivalence=PASS
```

The linked-chain shape beats the current modeled list path but is still much
slower than the already-rejected array-indexed synthetic shape. The full
runtime candidate compiled and built, but the real claim gate rejected it:

```text
online_max=500
loaded_chunks_max=5476
tps1_avg=8.98
tps1_min=3.97
avg_tick_ms_avg=148.42
avg_tick_ms_max=626.01
process_rss_mib_max=13949.3
bot_block_place_packets_max=60000
bot_block_dig_packets_max=60000
watchdog_thread_dumps=5
nearby_players_stack_hits=8
sync_load_stack_hits=0
```

Formal evaluator:

```text
gate_profile=production-500
claim_eligible=false
gate_pass=false
failure_count=6
failure=tps1_avg=8.98 < required 19.50
failure=tps1_min=3.97 < required 18.00
failure=avg_tick_ms_avg=148.42 > allowed 50.00
failure=avg_tick_ms_max=626.01 > allowed 100.00
failure=watchdog_thread_dumps=5 > allowed 0
failure=nearby_players_stack_hits=8 > allowed 0
```

Decision: reject and revert `0064-Optimize-SurfaceRules-sequence-chain.patch`.
Post-revert artifact rebuild, AppCDS/hash refresh, plugin matrix, restart
recovery, and forced-ticket persistence all passed. No "500 bots / production
ready" claim is allowed.

## Previous 2026-05-15 Native ImprovedNoise Handle Decision

Fresh commands from this continuation:

```bash
cargo test --manifest-path native/Cargo.toml -p paper-native-jni --lib
./scripts/bench_native_improved_noise.sh
```

Outcome:

```text
cargo test --manifest-path native/Cargo.toml -p paper-native-jni --lib: PASS
paper-native-jni compile/test path: PASS

./scripts/bench_native_improved_noise.sh: PASS
native_speedup_vs_java=1.160x
equivalence=PASS
handle_native_speedup_vs_java=0.917x
handle_equivalence=PASS
```

Decision: keep the native ImprovedNoise handle path diagnostic-only. The batch
JNI shape is faster, but the per-call handle path is slower than the Java
baseline on this host, so `paper.nativeImprovedNoise` must not be enabled by
default and does not support any production or 500-bot claim.

## Previous 2026-05-15 PerlinNoise GetValue Fast Path Candidate

Fresh commands from this continuation:

```bash
jfr view hot-methods reports/load-block-100-autoramp-throttle0-jfr-20260514.jfr | head -n 25
scripts/bench_perlin_getvalue.sh
scripts/bench_noise_interpolator_slice.sh
git -C upstream/Paper apply --check --directory=paper-server/src/minecraft/java paper-server/patches/features/0062-Optimize-PerlinNoise-wrap-floor.patch
./gradlew :paper-server:compileJava --no-daemon
javap -classpath upstream/Paper/paper-server/build/classes/java/main -c net.minecraft.world.level.levelgen.synth.PerlinNoise | rg -n "Math\\.floor|lfloor"
MC_EULA_AGREE=true ./scripts/build_optimized.sh
python3 scripts/update_artifact_reports.py && sha256sum -c reports/artifact-hashes.txt && python3 -m json.tool reports/artifacts.json >/dev/null
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=100 DURATION_SECONDS=240 BOT_BLOCK_ACTION_INTERVAL_MS=1000 LOAD_TEST_LABEL=block-100-perlin-mathwrap-guarded-20260515 ./scripts/run_load_test.sh
python3 scripts/evaluate_load_gate.py --profile production-500 reports/load-block-100-perlin-mathwrap-guarded-20260515-summary.txt
```

Outcome:

```text
JFR hot methods:
ImprovedNoise.sampleAndLerp(...)
ImprovedNoise.noise(...)
PerlinNoise.getValue(...)
Aquifer$NoiseBasedAquifer.computeSubstance(...)
NoiseChunk$NoiseInterpolator.compute(...)

scripts/bench_perlin_getvalue.sh: PASS
direct_math_wrap_guarded_getvalue_best_ms=633.477
direct_math_wrap_no_y_scale_guarded_getvalue_best_ms=674.450
math_wrap_guarded_speedup=1.169x
math_wrap_speedup=1.134x
math_wrap_no_y_scale_vs_guarded_speedup=0.939x
equivalence=PASS

scripts/bench_noise_interpolator_slice.sh: PASS
flat_speedup=0.934x
equivalence=PASS

git -C upstream/Paper apply --check --directory=paper-server/src/minecraft/java paper-server/patches/features/0062-Optimize-PerlinNoise-wrap-floor.patch: PASS
./gradlew :paper-server:compileJava --no-daemon: PASS
Applied 913 patches
BUILD SUCCESSFUL
javap bytecode proof: Math.floor
javap bytecode proof: exact-class getValue guard with local arrays

MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
native tests: 291 passed
artifact hashes: all listed OK

100-bot block rerun:
online_max=100
loaded_chunks_max=5184
tps1_avg=11.46
tps1_min=1.31
avg_tick_ms_avg=95.89
avg_tick_ms_max=448.32
process_rss_mib_max=10787.6
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0

production-500 evaluator on that 100-bot summary:
claim_eligible=false
gate_pass=false
failure_count=16
```

The runtime candidate is now a real source-level and artifact-level win on the
Perlin bench shape. The follow-up no-y-scale shape is rejected at microbench
stage because it is slower than the guarded Math.floor runtime shape
(`0.939x`). The follow-up 100-bot block plateau improved the prior 100-bot
result (`9.88 TPS / 223.62 ms / 11515.4 MiB`) to `11.46 TPS / 95.89 ms /
10787.6 MiB`. This is useful progress, but it still does not justify any
"500 bots / production ready" claim: the formal `production-500` evaluator
still fails until a fresh 500-bot run meets the TPS/MSPT/action/stability gate.

## Current 2026-05-14 21:49 CEST 50-Bot Block Plateau After Arena-Window Fix

The block load path now keeps re-seating late joiners through the full bot
window instead of stopping after the short ramp window. On the creative 32/32
block scenario, the patched harness reached all 50 bots armed and primed.

```text
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=50 DURATION_SECONDS=150 BOT_RAMP_SECONDS=10 LOAD_TEST_LABEL=block-50-arenafix-20260514 ./scripts/run_load_test.sh: PASS
load_test_scenario=block
load_test_gamemode=creative
block_arena=center_x=0 center_z=0 target_y=160 spacing=4 columns=8 item=stone
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

This is real 50-bot block-pressure evidence for the harness. It is still not
a production gate win or a 500-player claim because TPS/MSPT are too low.

## Current 2026-05-14 22:47 CEST 100-Bot Block Plateau With Join Throttle Disabled

The harness now writes a per-run `bukkit.yml` with `connection-throttle: 0`
for localhost synthetic load. On the creative 32/32 block scenario, the 100
bot run reached a full plateau.

```text
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

This is useful plateau evidence, not a production-ready claim. The next work
is reducing the block plateau's tick cost and chunk pressure before any 500
target language is allowed.

## Current 2026-05-14 21:24 CEST Block-Aware Load Smoke

The new block-aware load path now runs a creative arena setup, bot-side
creative slot priming, and alternating place/dig packets without server
stability failures on a short smoke.

```text
MC_EULA_AGREE=true LOAD_TEST_ALLOW_BUSY_HOST=true LOAD_TEST_SCENARIO=block BOT_COUNT=12 DURATION_SECONDS=35 BOT_RAMP_SECONDS=5 LOAD_TEST_LABEL=block-smoke-12-rerun ./scripts/run_load_test.sh: PASS
load_test_scenario=block
load_test_gamemode=creative
block_arena=center_x=0 center_z=0 target_y=160 spacing=4 columns=4 item=stone
online_max=12
loaded_chunks_max=192
tps1_avg=14.25
avg_tick_ms_avg=131.61
bot_block_armed_max=12
bot_block_primed_max=12
bot_block_creative_slot_packets_max=12
bot_block_place_packets_max=624
bot_block_dig_packets_max=616
bot_block_action_errors_max=0
compat_probe_block_places_max=617
compat_probe_block_breaks_max=609
compat_probe_arena_commands_max=5
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
thread_check_failures=0
chunk_system_errors=0
feature_placement_errors=0
off_main_poi_hits=0
stability_failures=0
```

This is block-harness validation only. It does not support a 500-player
claim.

## Current 2026-05-14 20:09 CEST Warm-World Benchmark and Native Area-Map Bench

The warm-world benchmark now separates startup/plugin load on a saved
`runs/plugin-matrix` world from first-run world creation and config
generation.

```text
MC_EULA_AGREE=true ./scripts/warm_world_benchmark.sh: PASS
stock-paper-1.21.10: status_ms=73810 done_ms=73872 rss_kb=1828804
optimized-paper-1.21.10: status_ms=46223 done_ms=46268 rss_kb=1666672
optimized-runtime-1.21.10: status_ms=36121 done_ms=36185 rss_kb=1251232
optimized_paper_vs_stock_done_speedup=1.597x
optimized_runtime_vs_stock_done_speedup=2.042x
optimized_runtime_vs_optimized_paper_done_speedup=1.279x
```

The guarded/default-off area-map path still has focused Java/native parity
evidence, but it is not an accepted runtime load result.

```text
reports/native-area-map-bench.txt: PASS
update_java_best_ms=522.344
update_native_best_ms=428.848
update_native_speedup_vs_java=1.218x
add_java_best_ms=643.796
add_native_best_ms=529.573
add_native_speedup_vs_java=1.216x
remove_java_best_ms=633.914
remove_native_best_ms=542.739
remove_native_speedup_vs_java=1.168x
equivalence=PASS

reports/load-50bots-area-map-native-gate-20260514-summary.txt: REJECT
tps1_avg=17.24
avg_tick_ms_avg=75.12
loaded_chunks_max=2766
watchdog_thread_dumps=6
```

Decision: keep the warm-world numbers as startup evidence and the area-map
numbers as parity/hot-path evidence only. No end-to-end TPS/MSPT or 500-player
claim is made from either.

## Current 2026-05-14 11:30 CEST Full Native Mega-All Pack

The full `all` pack now passes across all configured native diagnostic
domains in one bounded run.

```text
PACK_WRITE_MANIFEST=1 PACK_LABEL=mega-all-complete-v4 PACK_FAIL_FAST=1 PACK_GROUPS=all scripts/bench_native_pack.sh: PASS
pack_groups=all
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
pack_heavy_defaults=1
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
```

## Current 2026-05-14 02:33 CEST Native Mega-Pack Runner and Report Summary

The new multi-domain pack runner now covers aquifer, climate, entity,
waypoint, plugin, storage, and ticket diagnostics in one bounded pass. The
pack report summarizer turns the run into a machine-readable totals line.

```text
PACK_LABEL=mega-bounded PACK_FAIL_FAST=1 PACK_GROUPS='aquifer climate entity waypoint plugin storage ticket' scripts/bench_native_pack.sh: PASS
pack_groups=aquifer climate entity waypoint plugin storage ticket
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
```

## Current 2026-05-13 23:03 CEST Native Coverage Audit and Wide Worldgen Pack Smoke

The new structural audit and worldgen pack runner are now part of the native
diagnostic workflow. The audit reports full structural coverage for the Rust
core module list, native bench/script surface, and Java wrapper to Rust JNI
export surface. The wide pack smoke ran 16 worldgen diagnostics through one
native build and kept the whole group parity-clean.

```text
python3 scripts/native_coverage_audit.py --strict-docs: PASS
modules_total=89
required_bench_dirs_covered=92
required_scripts_covered=97
native_wrappers_checked=90
native_exports_checked=243
warnings=0
errors=0

PACK_FAIL_FAST=1 PACK_SCRIPTS='scripts/bench_native_improved_noise.sh scripts/bench_native_improved_noise_inline.sh scripts/bench_native_improved_noise_derivative.sh scripts/bench_native_perlin_noise.sh scripts/bench_native_perlin_getvalue.sh scripts/bench_native_blended_noise.sh scripts/bench_native_noise_generator_settings.sh scripts/bench_native_density_ap2_fill.sh scripts/bench_native_density_ap2_minmax_fill.sh scripts/bench_native_density_visitor_hooks.sh scripts/bench_native_surface_rules_sequence_array.sh scripts/bench_native_surface_rules_test_rule_state.sh scripts/bench_native_placed_feature_traversal.sh scripts/bench_native_ore_feature_loop.sh scripts/bench_native_carver_iteration.sh scripts/bench_native_cave_carver_skip.sh' scripts/bench_native_worldgen_pack.sh: PASS
pack_mode=core
script_count=16
pack_fail_fast=1
pack_status=PASS failures=0
PACK_RESULT script=scripts/bench_native_improved_noise.sh status=PASS
PACK_RESULT script=scripts/bench_native_improved_noise_inline.sh status=PASS
PACK_RESULT script=scripts/bench_native_improved_noise_derivative.sh status=PASS
PACK_RESULT script=scripts/bench_native_perlin_noise.sh status=PASS
PACK_RESULT script=scripts/bench_native_perlin_getvalue.sh status=PASS
PACK_RESULT script=scripts/bench_native_blended_noise.sh status=PASS
PACK_RESULT script=scripts/bench_native_noise_generator_settings.sh status=PASS
PACK_RESULT script=scripts/bench_native_density_ap2_fill.sh status=PASS
PACK_RESULT script=scripts/bench_native_density_ap2_minmax_fill.sh status=PASS
PACK_RESULT script=scripts/bench_native_density_visitor_hooks.sh status=PASS
PACK_RESULT script=scripts/bench_native_surface_rules_sequence_array.sh status=PASS
PACK_RESULT script=scripts/bench_native_surface_rules_test_rule_state.sh status=PASS
PACK_RESULT script=scripts/bench_native_placed_feature_traversal.sh status=PASS
PACK_RESULT script=scripts/bench_native_ore_feature_loop.sh status=PASS
PACK_RESULT script=scripts/bench_native_carver_iteration.sh status=PASS
PACK_RESULT script=scripts/bench_native_cave_carver_skip.sh status=PASS
```

Дата: 2026-05-13

## Current 2026-05-13 22:03 UTC Rust Compression/IO Shape Native Batch

The new `lz4_stream_roundtrip`, `nbt_gzip_buffer_shape`, and
`compression_threshold_shape` diagnostic modules now have Java/native parity
benches. LZ4 round-trip validates restored payload parity across three block
sizes but is slower than Java through JNI on this host. The NBT/GZIP and
threshold benches are shape models, not production encoder hooks; both pass
equivalence and show faster native counting.

```text
JAVA_PROPS='-Diterations=16 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_lz4_stream_roundtrip.sh: PASS
payload_bytes=196608
block_32768_native_speedup_vs_java=0.426x
block_65536_native_speedup_vs_java=0.404x
block_131072_native_speedup_vs_java=0.419x
equivalence=PASS

JAVA_PROPS='-Drepeats=1024 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_nbt_gzip_buffer_shape.sh: PASS
writes=256
repeats=1024
current_native_speedup_vs_java=1.735x
gzip64k_native_speedup_vs_java=1.830x
prebuffer64k_native_speedup_vs_java=1.708x
both64k_native_speedup_vs_java=1.699x
equivalence=PASS

JAVA_PROPS='-Diterations=1024 -Dwarmup=1 -Drounds=3' ./scripts/bench_native_compression_threshold_shape.sh: PASS
packets=256
default_native_speedup_vs_java=6.236x
tight_native_speedup_vs_java=5.301x
equivalence=PASS
```

## Current 2026-05-13 22:38 CEST Rust ObfHelper Maps Native Batch

The new `obfhelper_maps` native diagnostic module now covers the mapping
bootstrap shapes for old stream/default maps, direct maps, and presized
StringPool-backed maps. The new bench passes equivalence on the real
`reobf.tiny` mapping jar and stays diagnostic-only.

```text
./scripts/bench_native_obfhelper_maps.sh: PASS
classes=7554
methods=47786
fields=31113
old_stream_default_java_best_ms=205.577
old_stream_default_native_best_ms=520.922
old_stream_default_native_speedup_vs_java=0.395x
direct_maps_java_best_ms=213.393
direct_maps_native_best_ms=535.938
direct_maps_native_speedup_vs_java=0.398x
presized_string_pool_java_best_ms=215.256
presized_string_pool_native_best_ms=502.081
presized_string_pool_native_speedup_vs_java=0.429x
equivalence=PASS
```

## Current 2026-05-13 21:33 CEST Rust VarInt/VarLong Read-Batch and Plugin Startup Rollup Native Batch

The native `varint` JNI bench now covers VarInt and VarLong size, write-batch,
and read-batch parity. The new `plugin_startup_rollup` module combines
plugin-name join and plugin startup log aggregation into one normal/debug
startup diagnostic. Both benches pass equivalence. Native remains slower than
Java on the VarInt/VarLong JNI shapes on this host, while the optimized plugin
startup rollup is still the useful same-runtime signal (`3.065x` normal,
`3.137x` debug in Java; `1.937x` normal, `1.948x` debug in native). No Paper
runtime hook has been installed.

```text
./scripts/bench_native_varint.sh: PASS
values=1000000 long_values=1000000 warmup=5 rounds=8
int_encoded_bytes=1427205
long_encoded_bytes=1927561
varint_write_native_speedup_vs_java=0.340x
varint_read_native_speedup_vs_java=0.554x
varint_size_native_speedup_vs_java=0.301x
varlong_write_native_speedup_vs_java=0.333x
varlong_read_native_speedup_vs_java=0.638x
varlong_size_native_speedup_vs_java=0.384x
equivalence=PASS

./scripts/bench_native_plugin_startup_rollup.sh: PASS
plugins=512
paper_names=68
bukkit_names=472
iterations=5000
old_normal_native_speedup_vs_java=0.834x
new_normal_native_speedup_vs_java=0.527x
old_debug_native_speedup_vs_java=0.875x
new_debug_native_speedup_vs_java=0.543x
java_new_normal_speedup_vs_old=3.065x
native_new_normal_speedup_vs_old=1.937x
java_new_debug_speedup_vs_old=3.137x
native_new_debug_speedup_vs_old=1.948x
equivalence=PASS
```

## Current 2026-05-13 18:49 CEST Rust Improved Noise Inline, Derivative, Hash Path, and NBT Capacity Native Batch

The new `improved_noise_inline`, `improved_noise_derivative`,
`hash_path_summary`, and `nbt_compound_map_capacity` diagnostic modules now
have Java/native parity benches. All four benches pass equivalence. Native is
slower on all five `improved_noise_inline` shapes, faster on all four
`improved_noise_derivative` shapes, roughly parity/slightly slower on the
hash-path read-all case and slower on the streaming case, and faster on all
tested NBT compound-map capacities.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_improved_noise_inline.sh: PASS
iterations=1000000
old_p_method_native_speedup_vs_java=0.656x
inline_byte_access_native_speedup_vs_java=0.670x
flat_gradient_native_speedup_vs_java=0.718x
arithmetic_native_speedup_vs_java=0.732x
switch_gradient_native_speedup_vs_java=0.681x
equivalence=PASS

SKIP_NATIVE_BUILD=1 ./scripts/bench_native_improved_noise_derivative.sh: PASS
samples=8192 iterations=1000000
old_derivative_native_speedup_vs_java=1.396x
inline_derivative_native_speedup_vs_java=1.351x
int_table_derivative_native_speedup_vs_java=1.409x
flat_gradient_derivative_native_speedup_vs_java=1.423x
equivalence=PASS

SKIP_NATIVE_BUILD=1 ./scripts/bench_native_hash_path.sh: PASS
inputs=13 bytes=38017023
read_all_native_speedup_vs_java=0.987x
streaming_native_speedup_vs_java=0.703x
equivalence=PASS

SKIP_NATIVE_BUILD=1 ./scripts/bench_native_nbt_compound_map_capacity.sh: PASS
regions_used=1 chunks_used=256
java_cap8_best_ms=57.693
native_cap2_speedup_vs_java_cap8=2.413x
native_cap4_speedup_vs_java_cap8=2.648x
native_cap8_speedup_vs_java_cap8=2.709x
native_cap16_speedup_vs_java_cap8=2.722x
equivalence=PASS
```

## Current 2026-05-13 17:14 CEST Rust Paletted, Density, and Entity Chunk Native Batch

The new `paletted_reencode_scratch`, `paletted_reencode_remap_cache`,
`density_spline_context`, `density_visitor_hook`, and
`entity_chunk_transient` diagnostic modules now have Java/native parity
benches. All five benches pass equivalence. Native is slower on the
`paletted_reencode_scratch` scratch/direct shapes, faster on the
`paletted_reencode_remap_cache` cached shape, faster on both
`density_spline_context` shapes, much faster on both `density_visitor_hook`
shapes, and faster on both `entity_chunk_transient` shapes.

```text
bash scripts/bench_native_paletted_reencode_scratch.sh: PASS
old_newarray_native_speedup_vs_java=2.284x
scratch_threadlocal_native_speedup_vs_java=0.571x
direct_packed_native_speedup_vs_java=0.505x
equivalence=PASS

bash scripts/bench_native_paletted_reencode_remap_cache.sh: PASS
current_previous_only_native_speedup_vs_java=0.737x
cached_palette_ids_native_speedup_vs_java=1.294x
equivalence=PASS

bash scripts/bench_native_density_spline_context.sh: PASS
old_wrapper_native_speedup_vs_java=1.343x
new_direct_native_speedup_vs_java=1.328x
equivalence=PASS

bash scripts/bench_native_density_visitor_hooks.sh: PASS
old_unwrapping_native_speedup_vs_java=1782.305x
hooked_unwrapping_native_speedup_vs_java=66.322x
equivalence=PASS

bash scripts/bench_native_entity_chunk_transient.sh: PASS
old_mixed_native_speedup_vs_java=14.323x
new_mixed_native_speedup_vs_java=14.130x
equivalence=PASS
```

## Current 2026-05-13 16:58 CEST Rust Waypoint Chunk Update and Remapper Hash Threshold Native Batch

The new `waypoint_chunk_update` and `remapper_hash_threshold` diagnostic
modules now have Java/native parity benches. Both benches pass equivalence.
The waypoint chunk-key shape is a useful same-runtime Java signal, but the
native JNI path is slower for this tiny hot loop. The remapper hash-threshold
bench verifies Java/native summary parity on real plugin/library jars; native
only wins the one-jar tiny case in this short run and is slower at larger
subsets, though native parallel beats native put at size 12.

```text
WAYPOINT_CHUNK_ITERATIONS=4000000 WAYPOINT_CHUNK_WARMUP=2 WAYPOINT_CHUNK_ROUNDS=4 ./scripts/bench_native_waypoint_chunk_update.sh: PASS
iterations=4000000
distance_native_speedup_vs_java=0.266x
long_key_native_speedup_vs_java=0.197x
long_key_speedup=2.587x
distance_java_value=1990296
distance_native_value=1990296
long_key_java_value=1990296
long_key_native_value=1990296
equivalence=PASS

HASH_BENCH_ITERATIONS=3 HASH_BENCH_ROUNDS=2 HASH_BENCH_WARMUP=1 ./scripts/bench_native_remapper_hash_threshold.sh: PASS
inputs=13
sizes=1,2,4,8,12
size=1 compute_if_absent_native_speedup_vs_java=10.993x
size=1 put_native_speedup_vs_java=12.792x
size=1 hybrid_native_speedup_vs_java=10.784x
size=1 parallel_native_speedup_vs_java=2.153x
size=12 compute_if_absent_native_speedup_vs_java=0.646x
size=12 put_native_speedup_vs_java=0.683x
size=12 hybrid_native_speedup_vs_java=0.602x
size=12 parallel_native_speedup_vs_java=0.650x
size=12 native_parallel_speedup_vs_put=2.579x
equivalence=PASS
```

## Current 2026-05-13 16:08 CEST Rust Waypoint Snapshot, Table View, and Manager Skip Native Batch

The new `waypoint_snapshot`, `waypoint_table_view`, and
`waypoint_manager_skip` diagnostic modules now have Java/native parity
benches. All three benches pass equivalence. The same-runtime Java signal is
mixed for manager-skip, but the Rust/JNI summaries match Java exactly and the
native diagnostic models are faster than Java on this host.

```text
./scripts/bench_native_waypoint_snapshot.sh: PASS
iterations=50000
toArray_native_speedup_vs_java=18362.610x
sizedArray_native_speedup_vs_java=28326.422x
manual_native_speedup_vs_java=12246.901x
sizedArray_speedup=0.649x
manual_speedup=1.495x
equivalence=PASS

./scripts/bench_native_waypoint_table_view.sh: PASS
iterations=200000
transpose_row_native_speedup_vs_java=14612.526x
column_native_speedup_vs_java=17070.012x
column_speedup=0.994x
equivalence=PASS

./scripts/bench_native_waypoint_manager_skip.sh: PASS
iterations=1000000
current_player_full_native_speedup_vs_java=3872.955x
skip_player_full_native_speedup_vs_java=2162.004x
current_player_partial_native_speedup_vs_java=3930.484x
skip_player_partial_native_speedup_vs_java=2412.447x
current_waypoint_full_native_speedup_vs_java=2649.895x
skip_waypoint_full_native_speedup_vs_java=2225.330x
current_waypoint_partial_native_speedup_vs_java=4522.427x
skip_waypoint_partial_native_speedup_vs_java=4337.273x
skip_player_full_java_speedup=1.795x
skip_player_partial_java_speedup=1.017x
skip_waypoint_full_java_speedup=1.573x
skip_waypoint_partial_java_speedup=0.957x
equivalence=PASS
```

## Current 2026-05-13 15:15 CEST Rust Improved Noise Floor, Surface Rules, Placed Feature Traversal, Ore Loop, and Ticketset Native Batch

The newest diagnostic native batch covers three already-added parity models
(`improved_noise_floor`, `surface_rules_sequence_array`,
`surface_rules_test_rule_state`) plus three new ones
(`placed_feature_traversal`, `ore_feature_loop`, `ticketset_search`). All
benchmarks pass equivalence. Improved-noise-floor native is slower than Java;
surface-rules native beats Java on every measured shape; placed-feature
traversal native is much faster than both Java shapes; ore-loop native beats
Java on both old and optimized shapes; and ticketset-search native beats Java
across binary, unchecked-binary, and linear thresholds.

```text
./scripts/bench_native_improved_noise_floor.sh: PASS
iterations=1000000
current_mth_floor_native_speedup_vs_java=0.588x
math_floor_native_speedup_vs_java=0.701x
java_math_floor_speedup=0.900x
native_math_floor_speedup=1.073x
equivalence=PASS

./scripts/bench_native_surface_rules_sequence_array.sh: PASS
iterations=20000000
rules=14
list_enhanced_native_speedup_vs_java=2.456x
list_indexed_native_speedup_vs_java=6.337x
array_foreach_native_speedup_vs_java=1.938x
array_indexed_native_speedup_vs_java=3.567x
equivalence=PASS

./scripts/bench_native_surface_rules_test_rule_state.sh: PASS
iterations=20000000
period7_old_native_speedup_vs_java=1.445x
period7_new_native_speedup_vs_java=1.321x
period7_java_speedup_vs_old=1.174x
period7_native_speedup_vs_old=1.073x
period2_old_native_speedup_vs_java=1.593x
period2_new_native_speedup_vs_java=1.410x
period2_java_speedup_vs_old=1.119x
period2_native_speedup_vs_old=0.990x
equivalence=PASS

./scripts/bench_native_placed_feature_traversal.sh: PASS
traversals=200000
java_recursive_speedup_vs_stream=0.729x
native_speedup_vs_java_stream=21.813x
native_speedup_vs_java_recursive=29.905x
equivalence=PASS

./scripts/bench_native_ore_feature_loop.sh: PASS
blobs=65536
java_speedup_vs_old=1.011x
native_speedup_vs_old=0.946x
native_old_speedup_vs_java=1.593x
native_optimized_speedup_vs_java=1.491x
equivalence=PASS

./scripts/bench_native_ticketset_search.sh: PASS
iterations=6000000
binary_native_speedup_vs_java=3.220x
unchecked_binary_native_speedup_vs_java=3.209x
linear4_native_speedup_vs_java=3.608x
linear8_native_speedup_vs_java=3.174x
linear12_native_speedup_vs_java=3.498x
equivalence=PASS
```

## Current 2026-05-13 13:45 CEST Rust Protochunk Heightmap and Range Choice Native Batch

The new `protochunk_heightmap` and `range_choice` diagnostic modules now
have Java/native parity benches. Both benches pass equivalence.
Protochunk-heightmap native is faster than Java on the old and new loop
shapes, while range-choice native only wins on some old shapes and loses on
all optimized shapes.

```text
./scripts/bench_native_protochunk_heightmap.sh: PASS
iterations=8000000
old_enumset_foreach_native_speedup_vs_java=7.615x
new_cached_contains_native_speedup_vs_java=1.344x
java_speedup_vs_old=1.208x
native_speedup_vs_old=0.213x
equivalence=PASS

./scripts/bench_native_range_choice.sh: PASS
samples=1000000
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
```

## Current 2026-05-13 13:10 CEST Rust Climate Parameter Distance and Noise Generator Settings Native Batch

The new `climate_parameter_distance` and `noise_generator_settings`
diagnostic modules now have Java/native parity benches. Both benches pass
equivalence. Climate parameter-distance native is faster on old, branch, and
subtract-first shapes; noise-generator-settings native is faster on all five
shapes.

```text
./scripts/bench_native_climate_parameter_distance.sh: PASS
nodes=1024 queries=8192 parameters=7
old_distance_native_speedup_vs_java=3.124x
branch_distance_native_speedup_vs_java=5.274x
subtract_first_distance_native_speedup_vs_java=3.072x
equivalence=PASS

./scripts/bench_native_noise_generator_settings.sh: PASS
iterations=20000000
generators=1024
holder_value_settings_native_speedup_vs_java=3.113x
memoized_supplier_settings_native_speedup_vs_java=6.056x
lazy_primitive_settings_native_speedup_vs_java=2.543x
manual_lazy_object_settings_native_speedup_vs_java=3.514x
cached_int_settings_native_speedup_vs_java=1.306x
equivalence=PASS
```

## Current 2026-05-13 13:08 CEST Rust Chunk Expire Count, CraftPlayer CanSee, and LevelChunk Heightmap Native Batch

The new `chunk_expire_count`, `craftplayer_cansee`, and `levelchunk_heightmap`
diagnostic modules now have Java/native parity benches. All three benches
pass equivalence. Chunk-expire-count native is slower on every measured
shape; CraftPlayer can-see native is faster on every measured shape; and
LevelChunk heightmap native wins on the old four-update shape but loses on
the new combined-update shape.

```text
./scripts/bench_native_chunk_expire_count.sh: PASS
dynamic_compute_hot_native_speedup_vs_java=0.491x
dynamic_manual_hot_native_speedup_vs_java=0.421x
cached_compute_hot_native_speedup_vs_java=0.538x
cached_hybrid_hot_native_speedup_vs_java=0.356x
cached_manual_hot_native_speedup_vs_java=0.307x
dynamic_compute_cold_native_speedup_vs_java=0.628x
dynamic_manual_cold_native_speedup_vs_java=0.282x
cached_compute_cold_native_speedup_vs_java=0.389x
cached_hybrid_cold_native_speedup_vs_java=0.367x
cached_manual_cold_native_speedup_vs_java=0.386x
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
```

## Current 2026-05-13 09:15 CEST Rust Nearby Player Map Native Batch

The new `nearby_player_map_capacity` diagnostic module now has a Java/native
parity bench for fastutil nearby-player-map capacity/rehash behavior across
50-player and 500-player scenarios. The bench passes equivalence for both
scenarios.

```text
./scripts/bench_native_nearby_player_map.sh: PASS
players=50 iterations=80000 warmup=4 rounds=8
default_java_best_ms=230.090
default_native_best_ms=3.291
default_native_speedup_vs_java=69.919x
presized_java_best_ms=107.641
presized_native_best_ms=2.757
presized_native_speedup_vs_java=39.047x
java_presized_speedup_vs_default=2.138x
native_presized_speedup_vs_default=1.194x
equivalence=PASS
players=500 iterations=12000 warmup=4 rounds=8
default_java_best_ms=372.105
default_native_best_ms=4.253
default_native_speedup_vs_java=87.489x
presized_java_best_ms=146.313
presized_native_best_ms=3.494
presized_native_speedup_vs_java=41.880x
java_presized_speedup_vs_default=2.543x
native_presized_speedup_vs_default=1.217x
equivalence=PASS
```

## Current 2026-05-13 09:40 CEST Rust Marker Cache Native Batch

The new `marker_cache` diagnostic module now has a Java/native parity bench
for old marker visitor behavior vs cached-marker visitor behavior. The bench
passes equivalence. Native wins on the old shape, but the cached shape is
slower in both the Java and native summary bench.

```text
./scripts/bench_native_marker_cache.sh: PASS
roots=256
depth=24
iterations=300
warmup=4 rounds=8
old_java_best_ms=15.907
old_native_best_ms=12.132
old_native_speedup_vs_java=1.311x
cached_java_best_ms=16.645
cached_native_best_ms=45.718
cached_native_speedup_vs_java=0.364x
java_cached_speedup_vs_old=0.956x
native_cached_speedup_vs_old=0.265x
equivalence=PASS
```

## Current 2026-05-13 09:40 CEST Rust Waypoint Distance Guard Native Batch

The new `waypoint_distance_guard` diagnostic module now has a Java/native
parity bench for old and guarded range checks plus old and guarded
really-far checks. The bench passes equivalence and verifies matching output
between old/guarded and Java/native summaries.

```text
./scripts/bench_native_waypoint_distance_guard.sh: PASS
entries=65536
iterations=8000000
warmup=3 rounds=5
old_range_java_best_ms=186.898
old_range_native_best_ms=226.127
old_range_native_speedup_vs_java=0.827x
guarded_range_java_best_ms=206.150
guarded_range_native_best_ms=236.115
guarded_range_native_speedup_vs_java=0.873x
guarded_range_speedup=0.907x
old_really_far_java_best_ms=118.896
old_really_far_native_best_ms=131.378
old_really_far_native_speedup_vs_java=0.905x
guarded_really_far_java_best_ms=131.110
guarded_really_far_native_best_ms=128.746
guarded_really_far_native_speedup_vs_java=1.018x
guarded_really_far_speedup=0.907x
equivalence=PASS
```

## Current 2026-05-13 09:01 CEST Rust Remapper Index Cleanup Native Batch

The new `remapper_index_cleanup` diagnostic module now has a Java/native
parity bench for eager cleanup work vs the lazy count-check path. The bench
passes equivalence. Native is slower than Java in absolute timing, but the
old-to-new reduction is visible inside both runtimes.

```text
./scripts/bench_native_remapper_index_cleanup.sh: PASS
inputs=12
remapped=4
iterations=250000
warmup=2 rounds=5
old_eager_cleanup_java_best_ms=163.046
old_eager_cleanup_native_best_ms=701.908
old_eager_cleanup_native_speedup_vs_java=0.232x
new_lazy_cleanup_java_best_ms=92.868
new_lazy_cleanup_native_best_ms=470.202
new_lazy_cleanup_native_speedup_vs_java=0.198x
java_cleanup_speedup_vs_old=1.756x
native_cleanup_speedup_vs_old=1.493x
equivalence=PASS
```

## Current 2026-05-13 09:01 CEST Rust Remapper Skip Hashes Native Batch

The new `remapper_skip_hashes` diagnostic module now has a Java/native parity
bench for stream-style skip-hash parsing vs direct loop parsing. The bench
passes equivalence. Native is faster than Java on this small parser model;
the Java old/new result is neutral/slightly slower for the loop shape.

```text
./scripts/bench_native_remapper_skip_hashes.sh: PASS
iterations=300000
hash_lines_per_iteration=8
warmup=4 rounds=8
old_stream_java_best_ms=1475.177
old_stream_native_best_ms=637.540
old_stream_native_speedup_vs_java=2.314x
new_loop_java_best_ms=1506.895
new_loop_native_best_ms=530.600
new_loop_native_speedup_vs_java=2.840x
java_loop_speedup_vs_old=0.979x
native_loop_speedup_vs_old=1.202x
equivalence=PASS
```

## Current 2026-05-13 09:01 CEST Rust Plugin Directory Scan Native Batch

The new `plugin_directory_scan` diagnostic module now has a Java/native
parity bench for the plugin-directory walk/list/directory-stream scan shapes.
The bench passes equivalence against the live `/root/rust/plugins/matrix`
directory with `12` plugin files per scan.

```text
./scripts/bench_native_plugin_directory_scan.sh: PASS
directory=/root/rust/plugins/matrix
plugins_per_scan=12
warmup=4 rounds=8 iterations=5000
walk_depth1_java_best_ms=216.364
walk_depth1_native_best_ms=95.438
walk_depth1_native_speedup_vs_java=2.267x
list_java_best_ms=133.211
list_native_best_ms=102.777
list_native_speedup_vs_java=1.296x
directory_stream_java_best_ms=126.839
directory_stream_native_best_ms=106.598
directory_stream_native_speedup_vs_java=1.190x
java_list_speedup_vs_walk=1.624x
native_list_speedup_vs_walk=0.929x
java_directory_stream_speedup_vs_list=1.050x
native_directory_stream_speedup_vs_list=0.964x
equivalence=PASS
```

## Current 2026-05-13 07:53 CEST Rust Spigot Load-Order Dependency Native Batch

The new `spigot_load_order_dependency` diagnostic module now has a
Java/native parity bench for loadAfter construction and load-order removal
checks. The bench passes equivalence. Native loses badly on the loadAfter
copy shapes, but wins on the direct removed-count shape.

```text
./scripts/bench_native_spigot_load_order_dependency.sh: PASS
load_after=8
dependencies_per_provider=6
iterations=2000000
warmup=6 rounds=10
old_load_after_build_java_best_ms=139.846
old_load_after_build_native_best_ms=1251.358
old_load_after_build_native_speedup_vs_java=0.112x
old_load_after_build_java_value=-1141897171473225081
old_load_after_build_native_value=-1141897171473225081
new_load_after_build_java_best_ms=122.122
new_load_after_build_native_best_ms=1054.730
new_load_after_build_native_speedup_vs_java=0.116x
new_load_after_build_java_value=-1141897171473225081
new_load_after_build_native_value=-1141897171473225081
old_removed_count_java_best_ms=2698.012
old_removed_count_native_best_ms=11095.549
old_removed_count_native_speedup_vs_java=0.243x
old_removed_count_java_value=7569867351958714709
old_removed_count_native_value=7569867351958714709
new_removed_count_java_best_ms=327.579
new_removed_count_native_best_ms=139.952
new_removed_count_native_speedup_vs_java=2.341x
new_removed_count_java_value=7569867351958714709
new_removed_count_native_value=7569867351958714709
java_load_after_build_speedup_vs_old=1.145x
native_load_after_build_speedup_vs_old=1.186x
java_removed_count_speedup_vs_old=8.236x
native_removed_count_speedup_vs_old=79.281x
equivalence=PASS
```

## Current 2026-05-13 07:53 CEST Rust Topographic Graph Sort Capacity Native Batch

The new `topographic_graph_sort_capacity` diagnostic module now has a
Java/native parity bench for default-capacity vs pre-sized graph sort
containers. The bench passes equivalence. Native is slower than Java on both
measured shapes, but both runtimes still show pre-sizing improvement.

```text
./scripts/bench_native_topographic_graph_sort_capacity.sh: PASS
nodes=256
edges_per_node=4
warmup=4 rounds=8 iterations=20000
old_default_capacity_java_best_ms=728.066
old_default_capacity_native_best_ms=1039.753
old_default_capacity_native_speedup_vs_java=0.700x
old_default_capacity_java_value=398200962819901925
old_default_capacity_native_value=398200962819901925
new_presized_java_best_ms=431.974
new_presized_native_best_ms=840.945
new_presized_native_speedup_vs_java=0.514x
new_presized_java_value=398200962819901925
new_presized_native_value=398200962819901925
java_presized_speedup_vs_old=1.685x
native_presized_speedup_vs_old=1.236x
equivalence=PASS
```

## Current 2026-05-13 07:19 CEST Rust Plugin Loading Allocation Native Batch

The new `plugin_loading_allocation` diagnostic module now has a Java/native
parity bench for startup allocation shapes. The bench passes equivalence.
Native is slower than Java in absolute JNI timing on this host, so this stays
diagnostic-only.

```text
./scripts/bench_native_plugin_loading_allocation.sh: PASS
providers=256
hard_dependencies=4
iterations=20000
warmup=4 rounds=6
old_default_capacity_setup_java_best_ms=385.282
old_default_capacity_setup_native_best_ms=1271.136
old_default_capacity_setup_native_speedup_vs_java=0.303x
old_default_capacity_setup_java_value=-1057111528435909091
old_default_capacity_setup_native_value=-1057111528435909091
new_presized_setup_java_best_ms=236.709
new_presized_setup_native_best_ms=457.304
new_presized_setup_native_speedup_vs_java=0.518x
new_presized_setup_java_value=-1057111528435909091
new_presized_setup_native_value=-1057111528435909091
old_eager_missing_set_java_best_ms=292.169
old_eager_missing_set_native_best_ms=759.771
old_eager_missing_set_native_speedup_vs_java=0.385x
old_eager_missing_set_java_value=1950547922433864181
old_eager_missing_set_native_value=1950547922433864181
new_lazy_missing_set_java_best_ms=250.620
new_lazy_missing_set_native_best_ms=647.858
new_lazy_missing_set_native_speedup_vs_java=0.387x
new_lazy_missing_set_java_value=1950547922433864181
new_lazy_missing_set_native_value=1950547922433864181
old_eager_validate_java_best_ms=257.557
old_eager_validate_native_best_ms=473.605
old_eager_validate_native_speedup_vs_java=0.544x
old_eager_validate_java_value=-7670358977812336292
old_eager_validate_native_value=-7670358977812336292
new_lazy_validate_java_best_ms=262.321
new_lazy_validate_native_best_ms=483.201
new_lazy_validate_native_speedup_vs_java=0.543x
new_lazy_validate_java_value=-7670358977812336292
new_lazy_validate_native_value=-7670358977812336292
java_setup_speedup_vs_old=1.628x
native_setup_speedup_vs_old=2.780x
java_missing_speedup_vs_old=1.166x
native_missing_speedup_vs_old=1.173x
java_validate_speedup_vs_old=0.982x
native_validate_speedup_vs_old=0.980x
equivalence=PASS
```

## Current 2026-05-13 07:19 CEST Rust Legacy Provided-Alias Removal Native Batch

The new `legacy_provided_alias_removal` diagnostic module now has a
Java/native parity bench for the old values-removeIf cleanup and the reverse
provided-alias index cleanup. The bench passes equivalence. Native beats the
old Java removeIf path, but the optimized Java reverse-index path is still
faster than native on this host.

```text
./scripts/bench_native_legacy_provided_alias_removal.sh: PASS
providers=512
aliases_per_provider=4
iterations=200
warmup=3 rounds=6
old_values_removeif_java_best_ms=392.070
old_values_removeif_native_best_ms=184.060
old_values_removeif_native_speedup_vs_java=2.130x
old_values_removeif_java_value=-5744275592234625982
old_values_removeif_native_value=-5744275592234625982
new_reverse_alias_remove_java_best_ms=32.777
new_reverse_alias_remove_native_best_ms=77.657
new_reverse_alias_remove_native_speedup_vs_java=0.422x
new_reverse_alias_remove_java_value=-5744275592234625982
new_reverse_alias_remove_native_value=-5744275592234625982
java_alias_removal_speedup_vs_old=11.962x
native_alias_removal_speedup_vs_old=2.370x
equivalence=PASS
```

## Current 2026-05-12 20:49 CEST Rust Plugin ClassLoader Group Native Batch

The new `plugin_classloader_group` diagnostic module now has a Java/native
parity bench for miss, hit-other, and hit-requester paths with the
requester-skip shape. The bench passes equivalence. Native is faster on five
of six measured shapes on this host.

```text
./scripts/bench_native_plugin_classloader_group.sh: PASS
loaders=16
iterations=20000
warmup=3 rounds=6
miss_old_java_best_ms=1.805
miss_old_native_best_ms=0.485
miss_old_native_speedup_vs_java=3.723x
miss_old_java_value=-6009745692455826133
miss_old_native_value=-6009745692455826133
miss_skip_java_best_ms=1.532
miss_skip_native_best_ms=1.100
miss_skip_native_speedup_vs_java=1.393x
miss_skip_java_value=1801930673918154232
miss_skip_native_value=1801930673918154232
hit_other_old_java_best_ms=0.525
hit_other_old_native_best_ms=0.217
hit_other_old_native_speedup_vs_java=2.418x
hit_other_old_java_value=-8651361668984448826
hit_other_old_native_value=-8651361668984448826
hit_other_skip_java_best_ms=0.359
hit_other_skip_native_best_ms=0.391
hit_other_skip_native_speedup_vs_java=0.918x
hit_other_skip_java_value=-1884222056628612456
hit_other_skip_native_value=-1884222056628612456
hit_requester_old_java_best_ms=0.181
hit_requester_old_native_best_ms=0.098
hit_requester_old_native_speedup_vs_java=1.839x
hit_requester_old_java_value=-2727919425866261555
hit_requester_old_native_value=-2727919425866261555
hit_requester_skip_java_best_ms=0.218
hit_requester_skip_native_best_ms=0.166
hit_requester_skip_native_speedup_vs_java=1.314x
hit_requester_skip_java_value=-2727919425866261555
hit_requester_skip_native_value=-2727919425866261555
java_skip_miss_speedup_vs_old=1.178x
native_skip_miss_speedup_vs_old=0.441x
java_skip_hit_other_speedup_vs_old=1.465x
native_skip_hit_other_speedup_vs_old=0.556x
equivalence=PASS
```

## Current 2026-05-12 20:19 CEST Rust Plugin Metadata Dependency Native Batch

The new `plugin_meta_dependency` diagnostic module now has a Java/native
parity bench for the old stream shape, the direct loop shape, and the cached
repeated-access shape. The bench passes equivalence. Native beats the old
stream path on this host, but the Java loop and cached paths remain the
useful same-runtime wins.

```text
./scripts/bench_native_plugin_meta_dependency.sh: PASS
dependencies=12
iterations=2000000
warmup=6 rounds=10
old_stream_java_best_ms=2531.886
old_stream_native_best_ms=978.124
old_stream_native_speedup_vs_java=2.589x
old_stream_java_value=-1971533389749933458
old_stream_native_value=-1971533389749933458
new_loop_java_best_ms=807.677
new_loop_native_best_ms=961.744
new_loop_native_speedup_vs_java=0.840x
new_loop_java_value=-1971533389749933458
new_loop_native_value=-1971533389749933458
cached_java_best_ms=142.618
cached_native_best_ms=705.693
cached_native_speedup_vs_java=0.202x
cached_java_value=-1971533389749933458
cached_native_value=-1971533389749933458
java_new_loop_speedup_vs_old_stream=3.135x
native_new_loop_speedup_vs_old_stream=1.017x
java_cached_speedup_vs_new_loop=5.663x
native_cached_speedup_vs_new_loop=1.363x
equivalence=PASS
```

## Current 2026-05-12 19:42 CEST Rust Plugin Startup String Native Batch

The new `plugin_name_join` and `plugin_name_log` diagnostic modules now have
Java/native parity benches. Both pass equivalence. Native is slower than Java
on these string-heavy shapes on this host, so the result is diagnostic only.

```text
./scripts/bench_native_plugin_name_join.sh: PASS
plugins=512
iterations=5000
warmup=3 rounds=6
string_join_normal_java_best_ms=43.139
string_join_normal_native_best_ms=81.247
string_join_normal_native_speedup_vs_java=0.531x
string_join_normal_java_value=-8528451353182201141
string_join_normal_native_value=-8528451353182201141
manual_join_normal_java_best_ms=75.108
manual_join_normal_native_best_ms=153.670
manual_join_normal_native_speedup_vs_java=0.489x
manual_join_normal_java_value=-8528451353182201141
manual_join_normal_native_value=-8528451353182201141
string_join_debug_java_best_ms=47.399
string_join_debug_native_best_ms=133.320
string_join_debug_native_speedup_vs_java=0.356x
string_join_debug_java_value=6465241064178742549
string_join_debug_native_value=6465241064178742549
manual_join_debug_java_best_ms=81.115
manual_join_debug_native_best_ms=112.218
manual_join_debug_native_speedup_vs_java=0.723x
manual_join_debug_java_value=6465241064178742549
manual_join_debug_native_value=6465241064178742549
java_manual_normal_speedup_vs_string=0.574x
native_manual_normal_speedup_vs_string=0.529x
java_manual_debug_speedup_vs_string=0.584x
native_manual_debug_speedup_vs_string=1.188x
equivalence=PASS
```

```text
./scripts/bench_native_plugin_name_log.sh: PASS
plugins=512
paper_names=68
bukkit_names=472
iterations=5000
warmup=3 rounds=6
old_treeset_java_best_ms=376.078
old_treeset_native_best_ms=415.921
old_treeset_native_speedup_vs_java=0.904x
old_treeset_java_value=-4064966571156380128
old_treeset_native_value=-4064966571156380128
new_arraylistsort_java_best_ms=74.726
new_arraylistsort_native_best_ms=217.677
new_arraylistsort_native_speedup_vs_java=0.343x
new_arraylistsort_java_value=-4064966571156380128
new_arraylistsort_native_value=-4064966571156380128
java_arraylistsort_speedup_vs_treeset=5.033x
native_arraylistsort_speedup_vs_treeset=1.911x
equivalence=PASS
```

## Current 2026-05-12 18:22 CEST Rust Shift-Noise-Direct Native Batch

The new `shift_noise_direct` diagnostic module now has a Java/native parity
bench over the current, direct, current-A, direct-A, current-B, and direct-B
helper/direct shapes. The bench passes equivalence. Native is faster than
Java on all six measured shapes on this host. This remains diagnostic only.

```text
./scripts/bench_native_shift_noise_direct.sh: PASS
samples=1000000
verify_samples=16000
warmup=4 rounds=8
current_default_java_best_ms=8.624
current_default_native_best_ms=8.006
current_default_native_speedup_vs_java=1.077x
current_default_java_value=-732249154560925870
current_default_native_value=-732249154560925870
direct_default_java_best_ms=8.898
direct_default_native_best_ms=8.505
direct_default_native_speedup_vs_java=1.046x
direct_default_java_value=-5400637462255951743
direct_default_native_value=-5400637462255951743
current_a_java_best_ms=8.660
current_a_native_best_ms=7.627
current_a_native_speedup_vs_java=1.135x
current_a_java_value=5930795501788521756
current_a_native_value=5930795501788521756
direct_a_java_best_ms=8.833
direct_a_native_best_ms=7.677
direct_a_native_speedup_vs_java=1.151x
direct_a_java_value=5487995865039730244
direct_a_native_value=5487995865039730244
current_b_java_best_ms=8.537
current_b_native_best_ms=8.056
current_b_native_speedup_vs_java=1.060x
current_b_java_value=-4681550829423282671
current_b_native_value=-4681550829423282671
direct_b_java_best_ms=11.097
direct_b_native_best_ms=8.069
direct_b_native_speedup_vs_java=1.375x
direct_b_java_value=-8638949885602327916
direct_b_native_value=-8638949885602327916
java_direct_default_speedup_vs_current=0.969x
native_direct_default_speedup_vs_current=0.941x
java_direct_a_speedup_vs_current=0.980x
native_direct_a_speedup_vs_current=0.993x
java_direct_b_speedup_vs_current=0.769x
native_direct_b_speedup_vs_current=0.998x
equivalence=PASS
```

## Current 2026-05-12 18:55 CEST Rust Entity Bounding-Box Native Batch

The new `entity_bounding_box` diagnostic module now has a Java/native parity
bench over the old `EntityDimensions.makeBoundingBox(...)` then
`setBoundingBox(...)` shape and the direct dimensions-based
`setBoundingBox(...)` shape. The bench passes equivalence. Native is faster
than Java on both measured shapes on this host, while Java direct remains the
stronger same-runtime allocation reduction. This remains diagnostic only and
does not restore the previously rejected `Entity.setPosRaw(...)` runtime
shortcut.

```text
./scripts/bench_native_entity_bounding_box.sh: PASS
entries=16384
iterations=12000000
verify_iterations=16000
warmup=5 rounds=8
old_make_then_set_java_best_ms=1894.853
old_make_then_set_native_best_ms=395.719
old_make_then_set_native_speedup_vs_java=4.788x
old_make_then_set_java_allocated_bytes=1536000000
old_make_then_set_native_allocated_bytes=0
old_make_then_set_java_value=-7033733087056263035
old_make_then_set_native_value=-7033733087056263035
direct_dimensions_set_java_best_ms=813.001
direct_dimensions_set_native_best_ms=406.124
direct_dimensions_set_native_speedup_vs_java=2.002x
direct_dimensions_set_java_allocated_bytes=768000000
direct_dimensions_set_native_allocated_bytes=0
direct_dimensions_set_java_value=-5473495681565068597
direct_dimensions_set_native_value=-5473495681565068597
java_direct_speedup_vs_old=2.331x
native_direct_speedup_vs_old=0.974x
equivalence=PASS
```

## Current 2026-05-12 18:10 CEST Rust EntityLookup Status Native Batch

The new `entity_lookup_status` diagnostic module now has a Java/native parity
bench over `EntityLookup.getEntityStatus(...)` old status mapping, direct
status mapping, old accessibility, and direct accessibility. The bench passes
equivalence. Native is faster on all four measured shapes on this host. This
remains diagnostic only and does not restore the previously rejected
EntityLookup runtime candidates.

```text
./scripts/bench_native_entity_lookup_status.sh: PASS
entries=1048576
iterations=64000000
verify_iterations=16000
warmup=5 rounds=9
old_status_java_best_ms=579.948
old_status_native_best_ms=251.147
old_status_native_speedup_vs_java=2.309x
old_status_java_value=-2889940703601034638
old_status_native_value=-2889940703601034638
direct_status_java_best_ms=588.884
direct_status_native_best_ms=251.209
direct_status_native_speedup_vs_java=2.344x
direct_status_java_value=-2889940703601034638
direct_status_native_value=-2889940703601034638
old_accessible_java_best_ms=721.905
old_accessible_native_best_ms=258.580
old_accessible_native_speedup_vs_java=2.792x
old_accessible_java_value=-373183248314202015
old_accessible_native_value=-373183248314202015
direct_accessible_java_best_ms=685.592
direct_accessible_native_best_ms=258.715
direct_accessible_native_speedup_vs_java=2.650x
direct_accessible_java_value=-373183248314202015
direct_accessible_native_value=-373183248314202015
java_direct_status_speedup_vs_old=0.985x
native_direct_status_speedup_vs_old=1.000x
java_direct_accessible_speedup_vs_old=1.053x
native_direct_accessible_speedup_vs_old=0.999x
equivalence=PASS
```

## Current 2026-05-12 17:26 CEST Rust Chunk Dependencies Array Native Batch

The new `chunk_dependencies` diagnostic module now has a Java/native parity
bench over the old immutable-list dependency-radius lookup shape and the new
fixed-array shape. The bench passes equivalence. Native is faster than Java
on both measured shapes on this host. This remains diagnostic only.

```text
./scripts/bench_native_chunk_dependencies_array.sh: PASS
dependencies_size=9
iterations=128000000
verify_iterations=16000
warmup=5 rounds=9
old_java_best_ms=791.860
old_native_best_ms=477.905
old_native_speedup_vs_java=1.657x
old_java_value=-6631395432765096255
old_native_value=-6631395432765096255
array_java_best_ms=794.043
array_native_best_ms=482.147
array_native_speedup_vs_java=1.647x
array_java_value=-6631395432765096255
array_native_value=-6631395432765096255
java_array_speedup_vs_old=0.997x
native_array_speedup_vs_old=0.991x
equivalence=PASS
```

## Current 2026-05-12 17:26 CEST Rust Ownable Rule Native Batch

The new `ownable_rule` diagnostic module now has a Java/native parity bench
over the old stream/descriptor conversion shape and the new direct loop
shape. The bench passes equivalence. Native is faster than Java on both
measured shapes on this host; the Java loop rewrite is also much faster than
the Java stream baseline. This remains diagnostic only.

```text
./scripts/bench_native_ownable_rule.sh: PASS
iterations=12000000
verify_iterations=16000
owners=6
queries=6
warmup=5 rounds=9
old_stream_java_best_ms=1711.676
old_stream_native_best_ms=314.278
old_stream_native_speedup_vs_java=5.446x
old_stream_java_value=-7061642016323729141
old_stream_native_value=-7061642016323729141
new_loop_java_best_ms=626.597
new_loop_native_best_ms=254.995
new_loop_native_speedup_vs_java=2.457x
new_loop_java_value=-7061642016323729141
new_loop_native_value=-7061642016323729141
java_new_speedup_vs_old=2.732x
native_new_speedup_vs_old=1.232x
equivalence=PASS
```

## Current 2026-05-12 16:12 CEST Rust NoiseChunk Interpolator Array Native Batch

The new `noisechunk_interpolator_array` diagnostic module now has a
Java/native parity bench over the list, indexed-list, and array interpolator
loop shapes. The bench passes equivalence. Native is faster on all three
measured shapes on this host. This remains diagnostic only.

```text
./scripts/bench_native_noisechunk_interpolator_array.sh: PASS
interpolators=96
cell_count_xz=4
cell_count_y=48
iterations=1000000
warmup=4 rounds=8
list_java_best_ms=1174.474
list_native_best_ms=695.013
list_native_speedup_vs_java=1.690x
list_java_value=8798052272526521617
list_native_value=8798052272526521617
indexed_list_java_best_ms=1069.111
indexed_list_native_best_ms=686.470
indexed_list_native_speedup_vs_java=1.557x
indexed_list_java_value=8798052272526521617
indexed_list_native_value=8798052272526521617
array_java_best_ms=1145.747
array_native_best_ms=731.872
array_native_speedup_vs_java=1.566x
array_java_value=8798052272526521617
array_native_value=8798052272526521617
java_array_speedup_vs_list=1.025x
java_array_speedup_vs_indexed=0.933x
selected_operations_per_iteration=384
equivalence=PASS
```

## Current 2026-05-12 16:12 CEST Rust NoiseChunk FlatCache Context Native Batch

The new `noisechunk_flatcache_context` diagnostic module now has a
Java/native parity bench over the old/new false-context and old/new true-
context shapes around the `NoiseChunk.FlatCache` allocation path. The bench
passes equivalence. Native is slower on every measured shape on this host.
This remains diagnostic only and does not restore the previously rejected
runtime candidate.

```text
./scripts/bench_native_noisechunk_flatcache_context.sh: PASS
iterations=2000000
true_path_iterations=20000
size_xz=5
warmup=4 rounds=8
old_false_context_java_best_ms=108.479
old_false_context_native_best_ms=137.700
old_false_context_native_speedup_vs_java=0.788x
old_false_context_java_value=-7115423500006558480
old_false_context_native_value=-7115423500006558480
new_false_context_java_best_ms=89.412
new_false_context_native_best_ms=104.712
new_false_context_native_speedup_vs_java=0.854x
new_false_context_java_value=-7115423500006558480
new_false_context_native_value=-7115423500006558480
old_true_context_java_best_ms=1.038
old_true_context_native_best_ms=1.074
old_true_context_native_speedup_vs_java=0.967x
old_true_context_java_value=4743211603170849225
old_true_context_native_value=4743211603170849225
new_true_context_java_best_ms=1.006
new_true_context_native_best_ms=1.083
new_true_context_native_speedup_vs_java=0.929x
new_true_context_java_value=4743211603170849225
new_true_context_native_value=4743211603170849225
java_false_speedup_vs_old=1.213x
java_true_speedup_vs_old=1.032x
false_path_context_allocations_old=1
false_path_context_allocations_new=0
true_path_context_allocations_old=1
true_path_context_allocations_new=1
equivalence=PASS
```

## Current 2026-05-12 15:37 CEST Rust NoiseChunk BlendCache Native Batch

The new `noisechunk_blendcache` diagnostic module now has a Java/native
parity bench over the old empty-blender `FlatCache` allocation path and the
no-allocation empty-blender shape. The bench passes equivalence. Native is
slower on the old allocation-heavy shape but faster on the no-allocation
shape. This remains diagnostic only and does not restore the previously
rejected empty-blendcache runtime patch.

```text
./scripts/bench_native_noisechunk_blendcache.sh: PASS
iterations=5000000
size_xz=5
warmup=4 rounds=8
old_empty_blender_java_best_ms=417.205
old_empty_blender_native_best_ms=739.598
old_empty_blender_native_speedup_vs_java=0.564x
old_empty_blender_java_value=-5294264891731334115
old_empty_blender_native_value=-5294264891731334115
new_empty_blender_java_best_ms=10.404
new_empty_blender_native_best_ms=5.234
new_empty_blender_native_speedup_vs_java=1.988x
new_empty_blender_java_value=-5294264891731334115
new_empty_blender_native_value=-5294264891731334115
java_speedup_vs_old=40.100x
old_arrays_per_noisechunk=2
new_arrays_per_empty_blender_noisechunk=0
equivalence=PASS
```

## Current 2026-05-12 15:37 CEST Rust NoiseInterpolator Slice Native Batch

The new `noise_interpolator_slice` diagnostic module now has a Java/native
parity bench over old jagged `double[][]` slices and flat `double[]` slices.
The bench passes equivalence. Native loses on the old jagged shape but wins
on the flat shape on this host. This remains diagnostic only; the Paper
runtime is unchanged.

```text
./scripts/bench_native_noise_interpolator_slice.sh: PASS
interpolators=96
cell_count_xz=4
cell_count_y=48
iterations=2000
warmup=4 rounds=6
old_jagged_java_best_ms=279.685
old_jagged_native_best_ms=415.066
old_jagged_native_speedup_vs_java=0.674x
old_jagged_java_value=-3284915378152908107
old_jagged_native_value=-3284915378152908107
flat_java_best_ms=304.545
flat_native_best_ms=261.091
flat_native_speedup_vs_java=1.166x
flat_java_value=-3284915378152908107
flat_native_value=-3284915378152908107
java_flat_speedup_vs_old=0.918x
old_arrays_per_chunk=1152
flat_arrays_per_chunk=192
equivalence=PASS
```

## Current 2026-05-12 15:21 CEST Rust NoiseInterpolatorFractions Native Batch

The new `noise_interpolator_fractions` diagnostic module now has a
Java/native parity bench over the fixed `CaveCarver`-style 4x8 fraction
lookup workload. The bench passes equivalence and native is faster on both
measured shapes on this host, but it stays diagnostic only until there is a
guarded runtime hook and strict-gate proof.

```text
./scripts/bench_native_noise_interpolator_fractions.sh: PASS
iterations=2000000
cell_width=4
cell_height=8
warmup=4 rounds=8
division_java_best_ms=17.238
division_native_best_ms=12.280
division_native_speedup_vs_java=1.404x
division_java_value=-4251310216153051296
division_native_value=-4251310216153051296
array_fraction_java_best_ms=11.919
array_fraction_native_best_ms=11.437
array_fraction_native_speedup_vs_java=1.042x
array_fraction_java_value=-3958060204772962262
array_fraction_native_value=-3958060204772962262
java_array_fraction_speedup_vs_division=1.446x
equivalence=PASS
```

## Current 2026-05-12 15:00 CEST Rust Carver Iteration Diagnostic Batch

The new `carver_iteration` diagnostic module now has a Java/native parity
bench over the `CaveCarver` foreach vs indexed loop shapes. The bench passes
equivalence and native beats Java on both measured shapes on this host, but
the native indexed shape is still slower than native foreach, so keep it
diagnostic only until there is a guarded runtime hook and strict-gate proof.

```text
./scripts/bench_native_carver_iteration.sh: PASS
sets=9
values=36
iterations=8000000
warmup=5 rounds=8
foreach_java_best_ms=133.704
foreach_native_best_ms=64.958
foreach_native_speedup_vs_java=2.058x
foreach_java_value=-6684959247420485600
foreach_native_value=-6684959247420485600
indexed_java_best_ms=89.380
indexed_native_best_ms=76.765
indexed_native_speedup_vs_java=1.164x
indexed_java_value=-6684959247420485600
indexed_native_value=-6684959247420485600
java_indexed_speedup_vs_foreach=1.496x
native_indexed_speedup_vs_foreach=0.846x
equivalence=PASS
```

## Current 2026-05-12 14:59 CEST Rust CaveCarverSkip Diagnostic Batch

The new `cave_carver_skip` diagnostic module now has a Java/native parity
bench over the old lambda, reused checker, and direct helper loop shapes.
The bench passes equivalence, but JNI overhead makes every native shape
slower than Java on this host, so keep it diagnostic only until there is a
guarded runtime hook and strict-gate proof.

```text
./scripts/bench_native_cave_carver_skip.sh: PASS
carves=80000
caves_per_carve=6
samples_per_cave=48
old_java_best_ms=61.044
old_native_best_ms=83.470
old_native_speedup_vs_java=0.731x
old_java_value=7136908098897119975
old_native_value=7136908098897119975
reused_checker_java_best_ms=58.211
reused_checker_native_best_ms=89.915
reused_checker_native_speedup_vs_java=0.647x
reused_checker_java_value=7136908098897119975
reused_checker_native_value=7136908098897119975
direct_helper_java_best_ms=58.899
direct_helper_native_best_ms=80.163
direct_helper_native_speedup_vs_java=0.735x
direct_helper_java_value=7136908098897119975
direct_helper_native_value=7136908098897119975
reused_checker_speedup=1.049x
direct_helper_speedup=1.036x
old_checker_allocations_per_run=480000
reused_checker_allocations_per_run=80000
direct_checker_allocations_per_run=0
equivalence=PASS
```

## Current 2026-05-12 14:34 CEST Rust ServerEntity Delta Diagnostic Batch

The new `serverentity_delta_identity` diagnostic module now has a Java/native
parity bench over the `ServerEntity.sendChanges()` delta-motion old distance
path and identity-guard path. The bench passes equivalence. Native is faster
than the old Java distance path, but the native identity-guard summary is
slower than the already optimized Java guard, so the module stays diagnostic
only.

```text
./scripts/bench_native_serverentity_delta_identity.sh: PASS
entries=16384
iterations=16000000
same_identity_percent=75
old_java_best_ms=193.459
old_native_best_ms=151.916
old_native_speedup_vs_java=1.273x
old_java_value=6717808649417850587
old_native_value=6717808649417850587
identity_guard_java_best_ms=110.046
identity_guard_native_best_ms=116.559
identity_guard_native_speedup_vs_java=0.944x
identity_guard_java_value=6717808649417850587
identity_guard_native_value=6717808649417850587
java_identity_guard_speedup_vs_old=1.758x
equivalence=PASS
```

## Current 2026-05-12 14:13 CEST Rust StaticCache2D Diagnostic Batch

The new `static_cache_get` diagnostic module now has a Java/native parity
bench over the `StaticCache2D.get(...)` single-offset lookup workload. The
bench passes equivalence, but both native summaries are slower on this host,
so the module stays diagnostic only. This does not restore the previously
rejected single-offset runtime shape.

```text
./scripts/bench_native_static_cache_get.sh: PASS
radius=12
size=25
iterations=96000000
old_java_best_ms=733.176
old_native_best_ms=944.437
old_native_speedup_vs_java=0.776x
old_java_value=-1111864224777249497
old_native_value=-1111864224777249497
new_java_best_ms=693.851
new_native_best_ms=864.624
new_native_speedup_vs_java=0.802x
new_java_value=-1111864224777249497
new_native_value=-1111864224777249497
java_new_speedup_vs_old=1.057x
equivalence=PASS
```

## Current 2026-05-12 13:56 CEST Rust CubicSpline Create Diagnostic Batch

The new `cubic_spline_create` diagnostic module now has a Java/native parity
bench over the `CubicSpline` create/min-max scan workload. The bench passes
equivalence and both native summaries are faster on this host, but the module
stays diagnostic only until there is a guarded runtime hook and strict-gate
proof. This does not restore the previously rejected
`CubicSpline.Multipoint.mapAll` runtime cleanup.

```text
./scripts/bench_native_cubic_spline_create.sh: PASS
size=16
iterations=2000000
warmup=5 rounds=8
iterator_java_best_ms=120.308
iterator_native_best_ms=86.421
iterator_native_speedup_vs_java=1.392x
iterator_java_value=-6659558178047778137
iterator_native_value=-6659558178047778137
index_java_best_ms=114.063
index_native_best_ms=80.360
index_native_speedup_vs_java=1.419x
index_java_value=-6659558178047778137
index_native_value=-6659558178047778137
java_index_speedup_vs_iterator=1.055x
equivalence=PASS
```

## Current 2026-05-12 13:38 CEST Rust Jigsaw canAttach Diagnostic Batch

The new `jigsaw_canattach` diagnostic module now has a Java/native parity
bench over `JigsawBlock.canAttach(...)` old, optimized, and target-first
decision shapes. The bench passes equivalence and all native summaries are
much faster on this host, but the module stays diagnostic only until there is
a guarded runtime hook and strict-gate proof. This does not restore the
previously rejected target-first runtime patch.

```text
./scripts/bench_native_jigsaw_canattach.sh: PASS
entries=32768
iterations=4000000
old_can_attach_java_best_ms=1144.244
old_can_attach_native_best_ms=36.889
old_can_attach_native_speedup_vs_java=31.019x
optimized_can_attach_java_best_ms=1039.042
optimized_can_attach_native_best_ms=31.782
optimized_can_attach_native_speedup_vs_java=32.693x
target_first_can_attach_java_best_ms=294.473
target_first_can_attach_native_best_ms=27.068
target_first_can_attach_native_speedup_vs_java=10.879x
old_can_attach_java_value=569501294128725638
old_can_attach_native_value=569501294128725638
optimized_can_attach_java_value=569501294128725638
optimized_can_attach_native_value=569501294128725638
target_first_can_attach_java_value=569501294128725638
target_first_can_attach_native_value=569501294128725638
optimized_speedup=1.101x
target_first_speedup=3.886x
equivalence=PASS
```

## Current 2026-05-12 13:11 CEST Rust SpringFeature Mutable-Pos Diagnostic Batch

The new `spring_feature_mutable_pos` diagnostic module now has a Java/native
parity bench over the SpringFeature old `BlockPos` neighbor checks vs mutable
position reuse workload. The bench passes equivalence and both native
summaries are faster on this host, but the module stays diagnostic only until
there is a guarded runtime hook and strict-gate proof.

```text
./scripts/bench_native_spring_feature_mutable_pos.sh: PASS
positions=1048576
iterations=8000000
warmup=4 rounds=8
java_old_best_ms=744.758
native_old_best_ms=410.222
native_old_speedup_vs_java=1.816x
java_mutable_best_ms=714.250
native_mutable_best_ms=467.562
native_mutable_speedup_vs_java=1.528x
java_old_value=-4570851622231235949
native_old_value=-4570851622231235949
java_mutable_value=-4570851622231235949
native_mutable_value=-4570851622231235949
old_neighbor_positions_per_call=11
mutable_neighbor_positions_per_call=1
equivalence=PASS
```

## Current 2026-05-12 12:12 CEST Rust Biome getBiome Diagnostic Batch

The new `biome_getbiome` diagnostic module now has a Java/native parity bench
over the biome corner-selection workload. The bench passes equivalence and
both native summaries are faster on this host, but the module stays
diagnostic only until there is a guarded runtime hook and strict-gate proof.

```text
./scripts/bench_native_biome_getbiome.sh: PASS
samples=1000000
verify_samples=2000000
warmup=4 rounds=8
java_current_best_ms=152.722
native_current_best_ms=132.699
native_current_speedup_vs_java=1.151x
java_optimized_best_ms=194.038
native_optimized_best_ms=170.491
native_optimized_speedup_vs_java=1.138x
java_current_value=5810035875225132455
native_current_value=5810035875225132455
java_optimized_value=5810035875225132455
native_optimized_value=5810035875225132455
equivalence=PASS
```

## Current 2026-05-12 11:30 CEST Rust Beardifier Bury Diagnostic Batch

The new `beardifier_bury` diagnostic module now has a Java/native parity
bench over the `Beardifier.getBuryContribution(...)` distance-falloff
workload. The bench passes equivalence, but both native summaries are slower
on this host, so the module stays diagnostic only.

```text
./scripts/bench_native_beardifier_bury.sh: PASS
samples=2000000
warmup=4 rounds=8
java_current_best_ms=16.415
native_current_best_ms=46.555
native_current_speedup_vs_java=0.353x
java_optimized_best_ms=12.785
native_optimized_best_ms=47.140
native_optimized_speedup_vs_java=0.271x
java_current_value=-5952716895127920433
native_current_value=-5952716895127920433
java_optimized_value=-5952716895127920433
native_optimized_value=-5952716895127920433
equivalence=PASS
```

## Current 2026-05-12 11:09 CEST Rust YClampedGradient Diagnostic Batch

The new `yclamped_gradient` diagnostic module now has a Java/native parity
bench over the `YClampedGradient` clamped-map vs inline-lerp workload. The
bench passes equivalence, but both native summaries are slower on this host,
so the module stays diagnostic only.

```text
./scripts/bench_native_yclamped_gradient.sh: PASS
samples=2000000
warmup=4 rounds=8
java_current_best_ms=27.653
native_current_best_ms=60.910
native_current_speedup_vs_java=0.454x
java_optimized_best_ms=27.587
native_optimized_best_ms=63.403
native_optimized_speedup_vs_java=0.435x
java_current_value=-4584374797780407612
native_current_value=-4584374797780407612
java_optimized_value=-4584374797780407612
native_optimized_value=-4584374797780407612
equivalence=PASS
```

## Current 2026-05-12 10:38 CEST Rust Positional Xoroshiro Diagnostic Batches

The new `xoroshiro_positional_direct` diagnostic module now has a Java/native
parity bench over the positional `XoroshiroRandomSource.nextFloat()` and
`nextDouble()` paths, and the existing `aquifer_positional_location`
diagnostic bench was re-run on the same release library. Both benches pass
equivalence.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_xoroshiro_positional_direct.sh: PASS
coord_count=1048576
warmup_rounds=4
measure_rounds=8
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
java_old_float_value=5975327274269636668
native_old_float_value=5975327274269636668
java_direct_float_value=5975327274269636668
native_direct_float_value=5975327274269636668
java_old_double_value=-526535410355703026
native_old_double_value=-526535410355703026
java_direct_double_value=-526535410355703026
native_direct_double_value=-526535410355703026
equivalence=PASS
```

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_positional_location.sh: PASS
coord_count=1048576
warmup_rounds=4
measure_rounds=8
java_old_best_ms=27.402
native_old_best_ms=18.813
native_old_speedup_vs_java=1.456x
java_direct_best_ms=17.361
native_direct_best_ms=17.858
native_direct_speedup_vs_java=0.972x
allocated_bytes_supported=true
java_old_allocated_bytes_per_call=88.0
native_old_allocated_bytes_per_call=0.0
java_direct_allocated_bytes_per_call=0.0
native_direct_allocated_bytes_per_call=0.0
java_old_value=3681464667370276650
native_old_value=3681464667370276650
java_direct_value=3681464667370276650
native_direct_value=3681464667370276650
equivalence=PASS
```

## Current 2026-05-12 09:39 CEST Rust Aquifer Diagnostic Batches

The new `aquifer_index_stride` diagnostic module now has a Java/native parity
bench over the fixed-grid cache-index stride workload, and the existing
`aquifer_surface_sampling` diagnostic bench was re-run on the same release
library. Both paths pass equivalence and both native summaries are faster on
this host, so they stay diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_index_stride.sh: PASS
iterations=5000000
warmup=4 rounds=7
java_old_best_ms=288.438
native_old_best_ms=263.596
native_old_speedup_vs_java=1.094x
java_new_best_ms=319.463
native_new_best_ms=263.117
native_new_speedup_vs_java=1.214x
java_old_value=-2235408210929596250
native_old_value=-2235408210929596250
java_new_value=-2235408210929596250
native_new_value=-2235408210929596250
equivalence=PASS
```

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_aquifer_surface_sampling.sh: PASS
iterations=10000000
warmup=4 rounds=8
java_old_best_ms=295.584
native_old_best_ms=275.199
native_old_speedup_vs_java=1.074x
java_new_best_ms=272.365
native_new_best_ms=230.479
native_new_speedup_vs_java=1.182x
java_old_value=-7079792990984317396
native_old_value=-7079792990984317396
java_new_value=-7079792990984317396
native_new_value=-7079792990984317396
equivalence=PASS
```

## Current 2026-05-12 08:58 CEST Rust Blended Noise Diagnostic Batch

The new `blended_noise` diagnostic module now has a Java/native parity bench
over the synthetic BlendedNoise octave-lookup workload. Both paths pass
equivalence, but the native summary is slower on this host, so it stays
diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_blended_noise.sh: PASS
iterations=2000000
warmup=5 rounds=9
java_old_best_ms=629.502
native_old_best_ms=760.718
native_old_speedup_vs_java=0.828x
java_cached_best_ms=687.385
native_cached_best_ms=795.017
native_cached_speedup_vs_java=0.865x
java_old_value=-7534101404011476115
native_old_value=-7534101404011476115
java_cached_value=-7534101404011476115
native_cached_value=-7534101404011476115
equivalence=PASS
```

## Current 2026-05-12 08:44 CEST Rust Perlin Noise Diagnostic Batch

The new `perlin_noise` diagnostic module now has a Java/native parity bench
over the `PerlinNoise.getValue(...)` octave loop. The native summary is
slightly faster on this host, so it stays diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_perlin_noise.sh: PASS
octaves=16 samples=16384 iterations=500000 warmup=3 rounds=7
java_best_ms=307.791
native_best_ms=290.257
native_speedup_vs_java=1.060x
java_value=-3700938052026644427
native_value=-3700938052026644427
equivalence=PASS
```

## Current 2026-05-21 17:39 UTC Native Perlin getValue Variant Batch

The `native-perlin-getvalue` bench mirrors the older pure-Java
`PerlinGetValueBench` candidate space in Rust/JNI. It compares all six Java
variants against `PaperNativePerlinGetValue.getValueVariantBatchSummary(...)`
with the same permutations, amplitudes, origins, and input coordinates. The
no-y-scale Rust loop now uses the same indexed local loop shape as the faster
direct-local path.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_perlin_getvalue.sh
octaves=16 samples=8192 iterations=250000 warmup=2 rounds=5 variants=6
variant=delegating java_best_ms=155.758 native_best_ms=149.088 native_speedup_vs_java=1.045x
variant=direct java_best_ms=142.554 native_best_ms=172.378 native_speedup_vs_java=0.827x
variant=direct_local java_best_ms=133.526 native_best_ms=139.971 native_speedup_vs_java=0.954x
variant=direct_local_guarded java_best_ms=147.528 native_best_ms=133.698 native_speedup_vs_java=1.103x
variant=direct_no_y_scale java_best_ms=248.178 native_best_ms=207.178 native_speedup_vs_java=1.198x
variant=direct_math_wrap java_best_ms=228.972 native_best_ms=186.093 native_speedup_vs_java=1.230x
equivalence=PASS
script_status=PASS
```

Decision: keep native Perlin diagnostic/opt-in for now. The split no-y-scale
runtime hook exists, but `scripts/prepare_fast_runtime.sh` leaves generic and
no-y-scale Perlin disabled by default until a same-artifact load gate proves it
helps the server profile rather than just the microbench.

## Current 2026-05-12 08:30 CEST Rust Improved Noise Diagnostic Batch

The new `improved_noise` diagnostic module now has a Java/native parity bench
over the `ImprovedNoise` sample-and-lerp path. The native summary is slightly
faster on this host, so it stays diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_improved_noise.sh: PASS
samples=8192 iterations=1000000 warmup=4 rounds=8
java_best_ms=42.014
native_best_ms=38.572
native_speedup_vs_java=1.089x
java_value=5834936886027946920
native_value=5834936886027946920
equivalence=PASS
```

## Current 2026-05-12 08:14 CEST Rust Chunk Ticket Stage Diagnostic Batch

The new `chunk_ticket_stage` diagnostic module now has a Java/native parity
bench over the chunk-ticket-stage get-sweep and mutation-churn workload. The
native summary is slower on this host, so it stays diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_chunk_ticket_stage.sh: PASS
radius=34 query_keys=4761 staged_keys=3174 get_iterations=4000 mutation_iterations=200000 warmup=4 rounds=8
java_best_ms=199.714
native_best_ms=262.183
native_speedup_vs_java=0.762x
java_value=6992662693376471258
native_value=6992662693376471258
equivalence=PASS
```

## Current 2026-05-12 08:00 CEST Rust Ticket Compare Diagnostic Batch

The new `ticket_compare` diagnostic module now has a Java/native parity bench
over the ticket ordering shape. The native summary is slower on this host, so
it stays diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_ticket_compare.sh: PASS
size=32768 iterations=16000000 warmup=5 rounds=9
java_best_ms=190.711
native_best_ms=222.437
native_speedup_vs_java=0.857x
java_value=4526833328770928678
native_value=4526833328770928678
equivalence=PASS
```

## Current 2026-05-12 07:45 CEST Rust Ticket Pack Diagnostic Batch

The new `ticket_pack` diagnostic module now has a Java/native parity bench
over the persistent-ticket save path. The native summary is slightly slower on
this host, so it stays diagnostic only.

```text
SKIP_NATIVE_BUILD=1 ./scripts/bench_native_ticket_pack.sh: PASS
chunks=4096 tickets_per_chunk=8 iterations=20000 warmup=3 rounds=6
java_best_ms=588.246
native_best_ms=621.271
native_speedup_vs_java=0.947x
java_value=8289504869446747640
native_value=8289504869446747640
equivalence=PASS
```

## Current 2026-05-12 07:14 CEST Rust ReferenceList Diagnostic Batch

The new `reference_list` diagnostic module now has a Java/native parity bench
that exercises transition, dense, and random workloads through a single JNI
summary call. The native path is ahead on this host, but the module stays
diagnostic only.

```text
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
```

## Current 2026-05-12 02:16 CEST Rust Climate RTree Mixed Batch Default Diagnostic

The public batch defaults are back on clone-backed helpers for both current
and bounded search. The direct-current, borrowed-current, borrowed-bounded,
and arena paths remain diagnostic comparison points. The recursive search
helpers now carry the known best distance into child recursion and skip a
duplicate leaf exact-distance pass.

```text
cargo test --manifest-path native/Cargo.toml: PASS
paper-native-core tests: 30 passed

SKIP_NATIVE_BUILD=1 LEAVES=1400 QUERIES=120000 WARMUP=6 ROUNDS=16 ./scripts/bench_native_climate_rtree_batch_borrow.sh: PASS
tree_checksum=1463956120320347328
random_queries_checksum=5165014967713273743
walk_queries_checksum=-2288988305868638531
cloned_current_random_best_ms=568.915
direct_current_random_best_ms=579.340
direct_current_random_speedup_vs_cloned=0.982x
borrowed_current_random_best_ms=628.854
borrowed_current_random_speedup_vs_cloned=0.905x
cloned_bounded_random_best_ms=810.055
borrowed_bounded_random_best_ms=857.735
borrowed_bounded_random_speedup_vs_cloned=0.944x
cloned_current_walk_best_ms=268.908
direct_current_walk_best_ms=264.668
direct_current_walk_speedup_vs_cloned=1.016x
borrowed_current_walk_best_ms=299.807
borrowed_current_walk_speedup_vs_cloned=0.897x
cloned_bounded_walk_best_ms=230.583
borrowed_bounded_walk_best_ms=245.468
borrowed_bounded_walk_speedup_vs_cloned=0.939x
borrowed_batch_equivalence=PASS
random_checksum=-2174743207420542594
walk_checksum=-6213582386974512796

SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=6 -Drounds=16 -Dqueries=120000' ./scripts/bench_native_climate_rtree_jni.sh: PASS
native_tree_checksum=1463956120320347328
java_current_random_best_ms=1851.480
java_bounded_random_best_ms=1660.604
native_current_random_best_ms=588.766
native_bounded_random_best_ms=839.889
java_current_walk_best_ms=480.781
java_bounded_walk_best_ms=386.739
native_current_walk_best_ms=270.944
native_bounded_walk_best_ms=230.809
java_rtree_equivalence=PASS
native_rtree_equivalence=PASS
```

## Current 2026-05-12 01:05 CEST Rust Climate RTree Arena Diagnostic

Arena RTree diagnostic:

```text
SKIP_NATIVE_BUILD=1 LEAVES=1400 QUERIES=60000 WARMUP=2 ROUNDS=4 ./scripts/bench_native_climate_rtree_arena.sh: PASS
arena_node_count=1682
input_leaves_checksum=179575258560070041
rc_tree_checksum=1463956120320347328
arena_tree_checksum=1463956120320347328
random_queries_checksum=-6493225202231793453
walk_queries_checksum=-7894764894806409779
rc_batch_current_random_lifecycle_best_ms=338.325
rc_batch_bounded_random_lifecycle_best_ms=455.136
rc_batch_bounded_random_lifecycle_speedup=0.743x
arena_current_random_lifecycle_best_ms=346.798
arena_bounded_random_lifecycle_best_ms=503.828
arena_current_random_lifecycle_speedup_vs_rc=0.976x
arena_bounded_random_lifecycle_speedup_vs_rc=0.903x
rc_batch_current_walk_lifecycle_best_ms=153.736
rc_batch_bounded_walk_lifecycle_best_ms=120.808
rc_batch_bounded_walk_lifecycle_speedup=1.273x
arena_current_walk_lifecycle_best_ms=160.804
arena_bounded_walk_lifecycle_best_ms=153.596
arena_current_walk_lifecycle_speedup_vs_rc=0.956x
arena_bounded_walk_lifecycle_speedup_vs_rc=0.787x
rc_arena_lifecycle_equivalence=PASS
```

The owned arena tree is parity-correct, but it is slower than the current
Rc-backed batch lifecycle on this host, so it stays diagnostic only.

## Current 2026-05-12 00:52 CEST Rust Climate RTree Native Lifecycle Diagnostic

Native RTree lifecycle diagnostic:

```text
JAVA_PROPS='-Dqueries=60000 -Dwarmup=2 -Drounds=4' ./scripts/bench_native_climate_rtree_lifecycle.sh: PASS
lifecycle_scope=build_search_free
leaves=1400
queries=60000
warmup=2 rounds=4
input_leaves_checksum=179575258560070041
java_tree_checksum=1463956120320347328
native_tree_checksum=1463956120320347328
random_queries_checksum=-6493225202231793453
walk_queries_checksum=-7894764894806409779
java_current_random_lifecycle_best_ms=987.637
java_bounded_random_lifecycle_best_ms=881.556
java_bounded_random_lifecycle_speedup=1.120x
native_current_random_lifecycle_best_ms=317.413
native_bounded_random_lifecycle_best_ms=446.921
native_current_random_lifecycle_speedup_vs_java=3.112x
native_bounded_random_lifecycle_speedup_vs_java=1.973x
java_current_walk_lifecycle_best_ms=269.357
java_bounded_walk_lifecycle_best_ms=214.280
java_bounded_walk_lifecycle_speedup=1.257x
native_current_walk_lifecycle_best_ms=148.501
native_bounded_walk_lifecycle_best_ms=122.337
native_current_walk_lifecycle_speedup_vs_java=1.814x
native_bounded_walk_lifecycle_speedup_vs_java=1.752x
java_native_lifecycle_equivalence=PASS
random_checksum=-4612682530353248446
walk_checksum=-5174070138039077165
```

This is the broadest synthetic RTree pass so far: build, search, and free all
sit in the measured loop. The native handle path still stays ahead of Java on
both query shapes, but it remains diagnostic until there is a guarded Paper
hook, Java fallback, and strict server gate.

## Current 2026-05-12 00:39 CEST Rust Climate RTree Native Build Diagnostic

Native RTree build handle diagnostic:

```text
JAVA_PROPS='-Diterations=200 -Dwarmup=2 -Drounds=4' SKIP_NATIVE_BUILD=1 ./scripts/bench_native_climate_rtree_build.sh: PASS
leaves=1400
iterations=200
warmup=2 rounds=4
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

This measures native build plus checksum and explicit free, not just raw
construction. The allocation counter is JVM-thread allocation only, so it does
not count Rust heap allocations. It strengthens the RTree candidate, but does
not replace the need for a guarded Paper hook and strict load gate.

## Current 2026-05-12 00:27 CEST Rust Climate RTree JNI Handle Diagnostic

Java RTree search baseline:

```text
./scripts/bench_climate_rtree_search.sh: PASS
leaves=1400 queries=120000
current_random_best_ms=2010.557
bounded_random_best_ms=1801.047
bounded_random_speedup=1.116x
current_walk_best_ms=542.675
bounded_walk_best_ms=450.691
bounded_walk_speedup=1.204x
input_leaves_checksum=179575258560070041
current_tree_checksum=1463956120320347328
random_queries_checksum=5165014967713273743
walk_queries_checksum=-2288988305868638531
random_checksum=-2174743207420542594
walk_checksum=-6213582386974512796
equivalence=PASS
```

Standalone Rust RTree diagnostic:

```text
./scripts/bench_native_climate_rtree_search.sh: PASS
leaves=1400 queries=120000
native_current_random_best_ms=624.181
native_bounded_random_best_ms=852.528
native_bounded_random_speedup=0.732x
native_current_walk_best_ms=283.886
native_bounded_walk_best_ms=222.712
native_bounded_walk_speedup=1.275x
input_leaves_checksum=179575258560070041
current_tree_checksum=1463956120320347328
random_queries_checksum=5165014967713273743
walk_queries_checksum=-2288988305868638531
random_checksum=-2174743207420542594
walk_checksum=-6213582386974512796
equivalence=PASS
```

JNI handle diagnostic:

```text
./scripts/bench_native_climate_rtree_jni.sh: PASS
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

The JNI tree lifecycle keeps the native win intact after the Java ↔ Rust
boundary, but it is still diagnostic only until there is a guarded Paper hook
and a strict server gate.

## Current 2026-05-11 23:58 CEST Rust Climate RTree Search Diagnostic

Java RTree search baseline:

```text
./scripts/bench_climate_rtree_search.sh: PASS
leaves=1400 queries=120000
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
equivalence=PASS
```

Rust RTree search diagnostic:

```text
./scripts/bench_native_climate_rtree_search.sh: PASS
leaves=1400 queries=120000
native_current_random_best_ms=1069.431
native_bounded_random_best_ms=1087.711
native_bounded_random_speedup=0.983x
native_current_walk_best_ms=266.218
native_bounded_walk_best_ms=250.479
native_bounded_walk_speedup=1.063x
input_leaves_checksum=179575258560070041
current_tree_checksum=1463956120320347328
random_queries_checksum=5165014967713273743
walk_queries_checksum=-2288988305868638531
random_checksum=-2174743207420542594
walk_checksum=-6213582386974512796
equivalence=PASS
```

Rust wins the standalone diagnostic against Java's RTree search benchmark,
but this is not a runtime result yet. Native `bounded` loses slightly to
native `current` on random queries and wins on walk queries, so the production
shape needs more design than simply calling the bounded variant everywhere.

## Current 2026-05-11 23:30 CEST Rust Climate Batch Extended With Best-Match

Native climate batch bench:

```text
./scripts/bench_native_climate.sh: PASS
nodes=1024 queries=8192 parameters=7
java_node_distance_sum_best_ms=198.545
native_node_distance_sum_best_ms=44.859
native_node_distance_sum_speedup_vs_java=4.426x
java_node_best_match_best_ms=132.167
native_node_best_match_best_ms=95.798
native_node_best_match_speedup_vs_java=1.380x
equivalence=PASS
```

The climate module now shows native wins on both measured batch shapes, but
it is still diagnostic only until a guarded Paper runtime use site is chosen.

## Current 2026-05-11 23:16 CEST Rust Climate Batch And LZ4 Stream Verdict

Native climate batch bench:

```text
./scripts/bench_native_climate.sh: PASS
nodes=1024 queries=8192 parameters=7
java_node_distance_sum_best_ms=213.850
native_node_distance_sum_best_ms=38.319
native_node_distance_sum_speedup_vs_java=5.581x
equivalence=PASS
```

This is the first non-compression Rust module in this tree with a measured
native win on the current host. It is still diagnostic until a guarded Paper
runtime use site is selected and server-gated.

LZ4 stream wrapper bench after moving the wrapper out of Paper runtime:

```text
./scripts/bench_lz4_stream.sh: PASS
iterations=6000 data_size=196608 block_size=65536
buffered_default_best_ms=3292.509
native_lz4_best_ms=4365.214
native_lz4_speedup=0.754x
buffered_default_compressed_size=90585
native_lz4_compressed_size=90585
equivalence=PASS
```

Verdict: keep byte-array native LZ4 diagnostics, but do not wire the stream
wrapper into Paper runtime.

## Current 2026-05-11 22:12 CEST Rust Native Compression Backend Selected

Native workspace verification:

```text
./scripts/build_native.sh: PASS
paper-native-core tests: 16 passed
```

Region compression parity/perf bench:

```text
./scripts/bench_region_compression.sh: PASS
java/native LZ4 block-stream interop: PASS
chunks=768 chunk_bytes=98304
zlib_best_ms=3424.376 ratio=0.7659
gzip_best_ms=3457.330 ratio=0.7660
java_lz4_best_ms=321.627 ratio=0.9877
native_lz4_best_ms=277.301 ratio=0.9877
```

The native LZ4 stream path is faster on this benchmark and now matches Java's
compressed size on the same payloads, but it is still diagnostic only because
there is no guarded runtime hook yet.

Same native build, existing JNI parity benches rerun:

```text
varint: java_write_best_ms=4.966, native_write_best_ms=11.767,
        java_size_best_ms=3.644, native_size_best_ms=12.588, equivalence=PASS
position: java_chunk_pack_best_ms=1.692, native_chunk_pack_best_ms=8.290,
          java_chunk_hash_best_ms=1.356, native_chunk_hash_best_ms=5.295,
          java_section_pack_best_ms=1.932, native_section_pack_best_ms=12.595,
          java_combined_best_ms=4.296, native_combined_best_ms=35.467,
          equivalence=PASS
hash: java_sha256_best_ms=92.496, native_sha256_best_ms=145.949,
      equivalence=PASS
```

## Current 2026-05-11 19:15 CEST Rust Native Hash Checkpoint

Native workspace verification:

```text
./scripts/build_native.sh: PASS
paper-native-core tests: 9 passed
```

Hash parity/perf bench:

```text
./scripts/bench_native_hash.sh: PASS
equivalence=PASS
blocks=8 block_size=4194304
java_sha256_best_ms=95.743
native_sha256_best_ms=149.903
```

The asm backend narrowed the gap a little, but Java still wins on this host.

## Current 2026-05-11 19:06 CEST Rust Native Checkpoint

Native workspace verification:

```text
./scripts/build_native.sh: PASS
paper-native-core tests: 7 passed
```

VarInt parity/perf bench:

```text
./scripts/bench_native_varint.sh: PASS
equivalence=PASS
java_write_best_ms=4.138
native_write_best_ms=12.337
java_size_best_ms=4.172
native_size_best_ms=12.438
```

Position parity/perf bench:

```text
./scripts/bench_native_position.sh: PASS
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

The direct JNI path is still slower than the Java baseline, so these modules
remain diagnostic only.

## Current 2026-05-10 16:30 CEST NoiseChunk Wrapped Map Capacity Candidate Rejected

Diagnostic benchmark added:

```text
bench/noisechunk-wrap-size/NoiseChunkWrapSizeBench.java
scripts/bench_noisechunk_wrap_size.sh
reports/noisechunk-wrap-size-bench.txt

overworld / large_biomes / amplified:
  size_min=9361
  size_max=9361
  final_n_counts={16384=48}
nether / caves / floating_islands:
  size=52
  final_n_counts={4096=48}
end:
  size=41
  final_n_counts={4096=48}

map_variant=current_2048_075 best_ms=267.758 speedup_vs_current=1.000x
map_variant=expected_8192_075 best_ms=63.503 speedup_vs_current=4.216x
map_variant=expected_2048_095 best_ms=738.303 speedup_vs_current=0.363x
```

Candidate `0051-Optimize-NoiseChunk-wrapped-map-capacity.patch` used expected
size `8192` for non-empty `NoiseGeneratorSettings.spawnTarget()` and `2048`
otherwise. Build and functional gates passed:

```text
applyPatches/build/hash/json: PASS
plugin matrix: PASS, Done (29.070s)
restart/recovery: PASS, Done (16.856s)
forced-ticket persistence: PASS, first/restart Done 13.765s/9.690s
```

Strict 50-bot 32/32 spectator gate:

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

Verdict: rejected and rolled back. The current accepted reference remains
`18.27 TPS / 47.85 ms / 2380 chunks`; this candidate was worse on TPS/MSPT and
still produced watchdog dumps.

Post-rollback validation:

```text
0051 patch file: removed
applyPatches: PASS, Applied 912 patches
optimized artifact sha256=fb7b7e335f8660829d06b177d8ac20a06ffd52cfa2fe5d10a44f5b9a3fe50dca
app-cds sha256=c1acf8627ee17eac6b55fa71d3ad089a340d107bc9857a21e64ab3438b51b037
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (27.420s)
restart/recovery: PASS, Done (19.327s)
forced-ticket persistence: PASS, first/restart Done 15.171s/9.479s
```

## Current 2026-05-10 15:54 CEST Player Loader Unused Manhattan Distance Candidate Rejected

Candidate: remove the unused `manhattanDistance = Math.abs(dx) + Math.abs(dz)`
calculation in `RegionizedPlayerChunkLoader.PlayerChunkLoaderData.update()`.

Build and functional gates passed on the temporary candidate:

```text
build/hash/json: PASS
runtime bytecode: no local manhattanDistance in update() loop
plugin matrix: PASS, Done (28.675s)
restart/recovery: PASS, Done (19.867s)
forced-ticket persistence: PASS, first/restart Done 15.647s/9.855s
```

Strict 50-bot 32/32 spectator gate:

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

Verdict: rejected and rolled back. The current accepted reference remains
`18.27 TPS / 47.85 ms / 2380 chunks`; this run had worse TPS/MSPT and watchdog
dumps despite higher loaded-chunk coverage.

Post-rollback validation:

```text
0051 patch file: removed
applyPatches: PASS, Applied 912 patches
optimized artifact sha256=207d1b54cd81908c184e72b5435aa50b9c8eaf10c5df3836c1284ed8a388d2a4
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (28.309s)
restart/recovery: PASS, Done (18.328s)
forced-ticket persistence: PASS, first/restart Done 13.538s/9.224s
```

## Current 2026-05-10 14:20 CEST DensityFunctions Ap2 MIN/MAX Candidate Rejected Before Production

Candidate: add `MIN`/`MAX` whole-array fast paths in
`DensityFunctions.Ap2.fillArray(...)` when `argument1` and `argument2` have
non-overlapping min/max ranges.

Focused benchmark:

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

Native parity evidence:

```text
reports/native-density-ap2-minmax-fill-bench.txt
min_returns_first_native_old_speedup_vs_java=0.464x
min_returns_first_native_new_speedup_vs_java=0.516x
min_returns_second_native_old_speedup_vs_java=1.735x
min_returns_second_native_new_speedup_vs_java=0.548x
max_returns_first_native_old_speedup_vs_java=1.701x
max_returns_first_native_new_speedup_vs_java=5.065x
max_returns_second_native_old_speedup_vs_java=2.191x
max_returns_second_native_new_speedup_vs_java=3.351x
min_overlap_native_old_speedup_vs_java=2.407x
min_overlap_native_new_speedup_vs_java=2.259x
max_overlap_native_old_speedup_vs_java=1.628x
max_overlap_native_new_speedup_vs_java=2.841x
equivalence=PASS
```

Vanilla density graph scan:

```text
reports/density-ap2-minmax-graph-scan.txt
minmax_nodes=22
branch_counts=overlap:22
fastpath_candidates=0
unknown_types=0
unknown_refs=0
```

Decision: reject before production. The branch only wins for cases that the
vanilla density graph does not expose, while overlap cases are slightly slower.

## Current 2026-05-10 12:39 CEST Plugin ClassLoader Group Lookup Candidate (accepted with limits)

This cycle measured a narrow lookup reduction in
`SimpleListPluginClassLoaderGroup.getClassByName(...)`: the requester is
already checked first, so the common group scan now skips that same requester
entry on the second pass when class prioritization is enabled.

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

Runtime validation:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=84e1dfbce46697148479b233cd248885b13189e217567a2fc3f056d7844a2250
app-cds sha256=520beabfbe8032591d482457d9d5d45877a905351ea514b0e66dc399aacddabd
remap-classpath sha256=dfbb47c59fcc366260c487107788a2d4ea2f765205a1b3bf04c6658223914903
plugin matrix: PASS, 11 real plugins initialized
restart/recovery: PASS
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

Verdict: accepted only as a classloader lookup reduction. No end-to-end
startup/TPS claim is made.

## Current 2026-05-10 12:34 CEST Focused Candidates Rejected Before Production

Two follow-up candidates were measured and rejected before touching production
source.

`StaticCache2D.get(...)` single-offset lookup:

```text
reports/static-cache-get-bench.txt
old_contains_getindex_best_ms=408.902
new_single_offset_best_ms=427.130
single_offset_speedup=0.957x
equivalence=PASS
```

Decision: keep the existing `contains(...)` plus `getIndex(...)` shape. The
single-offset variant is bit-equivalent in the harness but slower on this JVM.

`ReferenceList.add(...)` transition-add path:

```text
reports/reference-list-transition-add-bench.txt
baseline_transition_best_ms=292.081
candidate_transition_best_ms=365.561
candidate_transition_speedup=0.799x
baseline_pair_best_ms=102.774
candidate_pair_best_ms=102.214
candidate_pair_speedup=1.005x
baseline_dense_best_ms=335.878
candidate_dense_best_ms=342.383
candidate_dense_speedup=0.981x
equivalence=PASS
```

Decision: do not productionize the transition-add clear-removal shape. It loses
badly on the transition case and does not provide a meaningful compensating win
on pair/dense cases.

## Current 2026-05-10 12:13 CEST SurfaceRules State Test Rule Accepted With Blocked Load Gate

Candidate: specialize `SurfaceRules.TestRuleSource.apply(...)` when
`thenRun` is exactly a `BlockRuleSource`. The runtime rule now tests the same
condition and returns the same `BlockState` directly instead of delegating
through `StateRule.tryApply(...)` on every surface-rule sample.

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

Validation on the rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=938d573e0bb9dc3816ac2b6464e264191d75ac576a9e02fce41f6801833c0d87
app-cds sha256=180cdde24eea942c37c5ed80aa93c39ccb415617790d1f97697ee43d317126f1
remap-classpath sha256=eb13423e8cb740b81c4e576e80cfee09df382b10339b8b111db5dc3ded228bff
plugin matrix: PASS, Done (34.742s), 11 real plugins initialized
restart/recovery: PASS, Done (19.091s), Saved the game
forced-ticket persistence: PASS, first/restart Done 16.112s/11.165s
```

Strict 50-bot 32/32 gate:

```text
reports/load-surfacerules-state-test-20260510-preflight.txt
host_preflight_ok=false
load_per_cpu=0.985
idle_percent_1s=55.32
```

Decision: accepted only as a narrow surface-rule dispatch reduction. No
end-to-end TPS, MSPT, startup, or 500-player claim is made.

## Current 2026-05-10 11:48 CEST Chunk Expire-Count Lookup Accepted With Blocked Load Gate

Candidate: change `ChunkHolderManager.addExpireCount(...)` from
`sectionToChunkToExpireCount.computeIfAbsent(...).addTo(...)` to an explicit
`get(...)`, atomic `putIfAbsent(...)` on misses, and direct `addTo(...)`.
The same chunk key, section key, `Long2IntOpenHashMap`, and expire-count
semantics are used.

Focused benchmark:

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

Validation on the rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
optimized artifact sha256=1520db1a6f32fc703df95c36b5c7e846afb15c173801cee7fa87fbc6e3333a64
app-cds sha256=810927baa7e04f8afddeb14d936b9043062c2c081d3750369bbda46eaebd0639
remap-classpath sha256=4f931556480b7a7aded1e8f07c21d1636b6e303266e5e5d8bafc8d338e971898
plugin matrix: PASS, Done (53.157s), 11 real plugins initialized
restart/recovery: PASS, Done (43.350s), Saved the game
forced-ticket persistence: PASS, first/restart Done 28.668s/22.220s
```

Strict 50-bot 32/32 gate:

```text
reports/load-chunk-expire-lookup-20260510-preflight.txt
host_preflight_ok=false
load_per_cpu=1.809
idle_percent_1s=13.93
```

Decision: accepted only as a narrow ticket-map lookup reduction. No
end-to-end TPS, MSPT, startup, or 500-player claim is made.

Follow-up microbench filters rejected in the same continuation, with no
production edits:

```text
reports/ticket-compare-bench.txt
old_best_ms=178.560
cached_best_ms=185.287
cached_speedup=0.964x
equivalence=PASS

reports/ticketset-search-bench.txt
binary_best_ms=938.540
unchecked_binary_speedup=0.815x
linear4_speedup=0.789x
linear8_speedup=0.851x
linear12_speedup=0.910x
equivalence=PASS

reports/spring-feature-mutable-pos-bench.txt
old_blockpos_best_ms=759.425
mutable_blockpos_best_ms=781.900
mutable_speedup=0.971x
equivalence=PASS

reports/shift-noise-direct-bench.txt
shift_direct_speedup=0.972x
shift_a_direct_speedup=0.968x
shift_b_direct_speedup=1.005x
equivalence=PASS
```

Decision: do not change `Ticket` compare fields, `TicketSet` search shape,
`SpringFeature` neighbor position creation, or shift-noise helper call sites
from these benchmark shapes.

## Current 2026-05-10 11:39 CEST CompressionEncoder Deflater Input Accepted

Candidate: change the Java `Deflater` fallback in
`net.minecraft.network.CompressionEncoder` to use `ByteBuf.nioBuffer(...)`
and `Deflater.setInput(ByteBuffer)` instead of copying into a temporary
`byte[]`. The Velocity native compression path was left unchanged.

Focused benchmark:

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

Validation on the rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
python3 scripts/update_artifact_reports.py: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
python3 -m json.tool reports/artifacts.json: PASS
plugin matrix: PASS, Done (51.284s), 11 real plugins initialized
restart/recovery: PASS, Done (24.522s), Saved the game
forced-ticket persistence: PASS, first/restart Done 20.692s/17.519s
```

Strict 50-bot 32/32 gate:

```text
reports/load-50bots-compression-deflater-bytebuffer-gate-20260510-v2-preflight.txt
host_preflight_ok=false
load_per_cpu=1.160
idle_percent_1s=41.32
```

Decision: accepted only as a narrow fallback-path copy reduction. No
end-to-end TPS, boot, or 500-player claim is made.

## Current 2026-05-10 10:42 CEST NoiseInterpolator Fractions Rejected

Candidate: precompute `NoiseChunk` cell interpolation fractions and replace the
three `NoiseInterpolator.compute(...)` divisions in the `fillingCell`
`Mth.lerp3(...)` path with array lookups. This was a local arithmetic change
only; density functions, interpolation order, chunk generation inputs, plugin
runtime, and scheduler/event semantics were not changed.

Focused benchmark:

```text
reports/noise-interpolator-fractions-bench.txt
iterations=2000000
division_best_ms=29.308
array_fraction_best_ms=5.943
array_fraction_speedup=4.932x
equivalence=PASS
```

Strict 50-bot 32/32 spectator evidence:

```text
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

Decision: rejected and rolled back. The candidate failed the accepted
`18.27 TPS / 47.85 ms / 2380 chunks` reference and introduced watchdog dumps.
Post-rollback gates pass: full build/hash/json, plugin matrix `Done 29.035s`,
restart/recovery `Done 18.063s`, and forced-ticket persistence
`16.174s/12.176s`. The microbench remains useful rejected evidence, not a
production optimization claim.

## Current 2026-05-10 10:07 CEST NearbyPlayers limit64 and Player-Loader Cache Rejected

Candidate A: cache `ChunkTaskScheduler` and `ChunkHolderManager` inside
`RegionizedPlayerChunkLoader.PlayerChunkLoaderData` to avoid repeated
server-level casts and scheduler lookups on movement/ticket paths.

Candidate B: raise `NearbyPlayers.TrackedChunk.SPARSE_PLAYER_LIST_LINEAR_LIMIT`
from `2` to `64` after watchdog jstacks landed in
`NearbyPlayers.tickPlayer -> ReferenceList.add/remove`.

Strict 50-bot 32/32 spectator evidence:

```text
reports/load-50bots-playerloader-cache-manager-20260510-summary.txt
online_max=50
tps1_avg=17.45
avg_tick_ms_avg=65.35
loaded_chunks_max=2412
watchdog_thread_dumps=4
nearby_players_stack_hits=8
stability_failures=0

reports/load-50bots-nearby-list-limit64-20260510-summary.txt
online_max=50
tps1_avg=16.90
avg_tick_ms_avg=88.49
loaded_chunks_max=2365
watchdog_thread_dumps=6
nearby_players_stack_hits=4
stability_failures=0
```

Decision: both candidates are rejected and rolled back. Neither beats the
accepted `18.27 TPS / 47.85 ms / 2380 chunks` reference, and both introduced
watchdog dumps. The final rollback runtime passes build/hash/json verification,
plugin matrix `Done (29.443s)`, restart/recovery `Done (21.228s)`, and
forced-ticket persistence `21.372s/11.272s`.

Harness fixes kept: `prepare_fast_runtime.sh` now clears stale remap/plugin/
reversed caches whenever the runtime jar hash changes, and
`generate_app_cds.sh` now writes the CDS archive through an absolute output
path. These are measurement/runtime packaging fixes, not gameplay
optimizations.

## Current 2026-05-10 08:39 CEST ProtoChunk Heightmap Candidate Fully Rolled Back

The temporary `ProtoChunk.setBlockState(...)` heightmap iterator-removal
candidate was removed from the feature patch stack and the runtime was
reverified after rollback.

Fresh verification:

```text
applyPatches: PASS, Applied 912 patches
build_optimized.sh: PASS
artifact hashes: PASS
plugin matrix: PASS, Done (26.859s)
restart/recovery: PASS, Done (16.028s)
forced-ticket persistence: PASS, first/restart Done 13.244s/9.550s
```

Fresh strict 50-bot 32/32 spectator gate:

```text
reports/load-50bots-protochunk-postrollback-20260510-summary.txt
online_max=50
tps1_avg=18.08
avg_tick_ms_avg=96.12
loaded_chunks_max=2609
watchdog_thread_dumps=3
sync_load_stack_hits=0
nearby_players_stack_hits=0
stability_failures=0
```

This is stable but not accepted. It does not beat the accepted
`18.27 TPS / 47.85 ms / 2380 chunks` reference.

## Current 2026-05-10 07:57 CEST OreFeature Loop Cleanup Rejected

Candidate: hoist repeated scalar work inside `OreFeature.doPlace(...)` by
reusing `d5 * d5`, reusing `d5 * d5 + d6 * d6`, and precomputing
`width * height` for the bitset index. This did not change ore placement
targets, but the real server gate did not accept it.

Focused evidence from the prior candidate bench:

```text
old_loop_best_ms=60.507
optimized_loop_best_ms=58.403
speedup=1.036x
equivalence=PASS
```

Strict 50-bot 32/32 spectator candidate gate:

```text
reports/load-50bots-orefeature-loop-gate-20260510-summary.txt
online_max=50
tps1_avg=18.27
avg_tick_ms_avg=65.21
loaded_chunks_max=2911
watchdog_thread_dumps=2
sync_load_stack_hits=0
nearby_players_stack_hits=4
stability_failures=0
```

Decision: rejected and rolled back. Post-rollback build and gates pass:
`build_optimized.sh` PASS with `Applied 912 patches`, artifact hashes PASS,
plugin matrix `Done (26.953s)`, restart/recovery `Done (17.037s)`, and
forced-ticket persistence `12.862s/8.382s`. No end-to-end TPS or 500-player
claim is made.

## Current 2026-05-10 06:39 CEST Waypoint Chunk-Key Update Rejected

Candidate: replace `WaypointTransmitter.EntityChunkConnection.update()`
chunk-change detection with a cached long-key comparison. This does not change
the packet type or update call site, but the real gate did not accept it.

Focused evidence:

```text
reports/waypoint-chunk-update-bench.txt
distance_best_ms=80.686
long_key_best_ms=34.099
long_key_speedup=2.366x
equivalence=PASS
```

Strict 50-bot 32/32 spectator candidate gate:

```text
reports/load-waypoint-chunkkey-update-20260510-summary.txt
online_max=50
tps1_avg=17.99
avg_tick_ms_avg=63.66
loaded_chunks_max=2516
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Decision: rejected and rolled back to the original chunk-distance update
condition. Post-rollback build and gates pass: `applyPatches` Applied 913
patches, `build_optimized.sh` PASS, artifact hashes PASS, plugin matrix
`Done (27.799s)`, restart/recovery `Done (16.968s)`, forced-ticket
persistence `13.274s/8.602s`.

Fresh rollback baseline:

```text
reports/load-50bots-post-rollback-baseline-20260510-summary.txt
online_max=50
tps1_avg=18.29
avg_tick_ms_avg=50.90
loaded_chunks_max=2441
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

The rollback baseline is stable evidence for the current runtime, not a 20
TPS / 500-player or vanilla-parity claim. The current JFR still points mostly
at chunk-generation noise (`ImprovedNoise.sampleAndLerp`,
`ImprovedNoise.noise`, `PerlinNoise.getValue`), but previously tested
`ImprovedNoise` table/inline/arithmetic variants remain rejected unless a new
shape survives both microbench and server gate.

## Current 2026-05-10 05:47 CEST NearbyPlayers Limit 3 and chunkTicketStage Capacity Rejected

Candidate A: raise `NearbyPlayers.TrackedChunk` sparse
`ReferenceList` threshold from `2` to `3`. Candidate B: pre-size
`PlayerChunkLoaderData.chunkTicketStage` with `new Long2ByteOpenHashMap(4096,
0.6F)`.

Focused `chunkTicketStage` evidence:

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

The pre-sized map shape is rejected at benchmark stage because it is slower
than the default map in this rerun.

Strict 50-bot 32/32 spectator evidence for `NearbyPlayers` limit `3`:

```text
reports/load-50bots-referencelist-linear3-gate-20260510-summary.txt
online_max=50
tps1_avg=18.06
avg_tick_ms_avg=46.77
loaded_chunks_max=2396
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0

reports/load-50bots-referencelist-linear3-rerun-20260510-summary.txt
online_max=50
tps1_avg=17.83
avg_tick_ms_avg=62.80
loaded_chunks_max=2427
watchdog_thread_dumps=0
sync_load_stack_hits=0
stability_failures=0
```

Decision: rejected and rolled back to limit `2`. Both runs were stable, but
the candidate did not beat the accepted `18.27 TPS / 47.85 ms / 2380 chunks`
reference on the combined TPS/MSPT/chunk-throughput signal. Rollback build
and gates pass: `build_optimized.sh` PASS, `sha256sum -c
reports/artifact-hashes.txt` PASS, `artifacts.json` PASS, plugin matrix
`Done (27.251s)`, restart/recovery `Done (15.819s)`, and forced-ticket
persistence `13.055s/8.863s`.

## Current 2026-05-10 03:40 CEST ReferenceList Transition Remove

Candidate: avoid the hash-map remove/shift path when a sparse
`ReferenceList` transitions from hash-backed mode back into tiny linear mode.

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

Runtime verification after the patch:

```text
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
optimized artifact sha256=e01176c27f59f0bd92d3a0ea7d884e692f7f07fe9e1945ad42ac5bdf26fa1a7e
app-cds sha256=8f2df11632803bb2325c3865c35085317984fb261623abd1b5e00351b8f56778
plugin matrix: PASS, Done (26.747s)
restart/recovery: PASS, Done (16.102s)
forced-ticket persistence: PASS, first/restart Done 13.346s/8.585s
strict 50-bot gate: stable bots, but not accepted (`18.07 TPS`, `51.73 ms`, `2782 chunks`, `watchdog_thread_dumps=3`)
```

This is a local movement hot-path reduction only. It is not an accepted
end-to-end TPS improvement, because the gate still has watchdog dumps and
does not beat the accepted `18.27 TPS / 47.85 ms / 2380 chunks` reference.

## Current 2026-05-10 02:26 CEST NearbyPlayers Map Capacity Candidate Rejected

Candidate: pre-size the two `NearbyPlayers` `Reference2ReferenceOpenHashMap`
instances on first player add. This preserved map semantics and only targeted
first-join rehash work, but it did not survive the real server gate.

Focused evidence:

```text
reports/nearby-player-map-bench.txt
scenario_players=50 default_best_ms=199.097 presized_best_ms=88.669 presized_speedup=2.245x
scenario_players=500 default_best_ms=339.920 presized_best_ms=139.127 presized_speedup=2.443x
default_rehashes_per_iteration=4.000/10.000
presized_rehashes_per_iteration=0.000
equivalence=PASS
```

Strict load gate:

```text
reports/load-50bots-nearby-map-capacity-gate-20260510-summary.txt
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

Decision: rejected and rolled back. The run was stable, but did not beat the
accepted `18.27 TPS / 47.85 ms / 2380 chunks` load reference. Rollback build
and gates pass: optimized artifact
`a7e95bd2da35771fce15c9f322b6ab3aeca902967bd124e3cc99aaca7487d941`,
`sha256sum -c reports/artifact-hashes.txt` PASS, plugin matrix
`Done (26.143s)`, restart/recovery `Done (16.191s)`, and forced-ticket
`12.884s/9.473s`.

## Current 2026-05-10 01:09 CEST ProtoChunk Heightmap Iterator Cleanup

`ProtoChunk.setBlockState(...)` now scans a cached `Heightmap.Types[]` and
checks membership with `EnumSet.contains(...)`, removing two iterator
allocations from the heightmap priming/update path.

Focused evidence:

```text
reports/protochunk-heightmap-bench.txt
old_enumset_foreach_best_ms=133.632
new_cached_values_contains_best_ms=100.017
new_speedup=1.336x
old_iterator_allocations_per_setblock=2
new_iterator_allocations_per_setblock=0
iterations=8000000
warmup=3 rounds=6
equivalence=PASS
```

Runtime verification after the patch:

```text
applyPatches: PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
optimized artifact sha256=5bd64433892e0656586c1163b681e6bb5e184dd55ac55865b3a6abd6b77d5dca
plugin matrix: PASS, Done (27.655s)
restart/recovery: PASS, Done (15.839s)
forced-ticket persistence: PASS, first/restart Done 13.433s/8.960s
strict 50-bot run 1: stable (`18.51 TPS`, `54.42 ms`, `2217 chunks`)
strict 50-bot run 2: stable (`17.84 TPS`, `46.13 ms`, `2215 chunks`)
```

The two strict 50-bot 32/32 runs had zero kicks/errors/watchdog/sync-load
hits, but neither beat the accepted load reference on the combined TPS,
MSPT, and loaded-chunk signal. This patch is a local heightmap allocation/work
reduction only, not a 20 TPS / 500-player or end-to-end load claim.

## Current 2026-05-09 23:56 CEST Climate RTree Search Pruning

`Climate.RTree.SubTree.search(...)` now uses bounded distance accumulation for
the default search path. It keeps the exact distance method and the custom
`DistanceMetric` path unchanged; the bounded method only short-circuits when a
child cannot beat the already-known best distance.

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

Runtime verification after the patch:

```text
rebuildPatches: PASS, Rebuilt 913 patches, Saved modified patches (44/47)
build_optimized.sh: PASS, includes applyPatches Applied 913 patches
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
optimized artifact sha256=2943cacf3d945cbb7e49e739d295d2789444b8b25c65762b72f22bdcacb09aec
plugin matrix: PASS, Done (30.298s)
restart/recovery: PASS, Done (22.644s)
forced-ticket persistence: PASS, first/restart Done 18.153s/18.919s
strict 50-bot gate: BLOCKED by host preflight (`load_per_cpu=1.657`, idle `33.05%`)
forced noisy 50-bot diagnostic: stable, but non-comparable (`tps1_avg=17.23`, `avg_tick_ms_avg=58.78`, `loaded_chunks_max=1750`)
```

The patch is kept as a local biome-search work reduction, not as a 20 TPS /
500-player or end-to-end TPS claim.

## Current 2026-05-09 23:36 CEST Carver Iteration Allocation Cleanup

`NoiseBasedChunkGenerator.applyCarvers(...)` no longer iterates carvers through
the `Iterable`/iterator path. `BiomeGenerationSettings.getCarvers()` exposes
the same `HolderSet`, and the call site now uses an indexed loop. This keeps
carver order and the `seed + i3` sequence intact.

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

Runtime verification after the patch:

```text
rebuildPatches: PASS, Rebuilt 913 patches, Saved modified patches (43/46)
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
optimized artifact sha256=c68f94185320568b687ed876a31b6da28ac5a47c19b3587de4deb9bfabf164fd
plugin matrix: PASS, Done (31.900s)
restart/recovery: PASS, Done (25.501s)
forced-ticket persistence: PASS, first/restart Done 15.097s/10.702s
strict 50-bot gate: BLOCKED by host preflight (`load_per_cpu=0.962`, idle `27.15%`)
forced noisy 50-bot diagnostic: stable, but non-comparable (`tps1_avg=17.24`, `avg_tick_ms_avg=95.80`, `loaded_chunks_max=1824`)
```

The patch is kept as a local worldgen allocation reduction, not as a 20 TPS /
500-player or end-to-end TPS claim.

## Current 2026-05-09 23:12 CEST Climate RTree build allocation, kept with load limits

The `Climate.RTree.build(...)` cleanup is a narrow startup/tree-build
allocation reduction only. The focused algorithm-shape bench improved the
recursive subtree build path from `543.404 ms` to `530.904 ms`
(`1.024x`) and saved about `335900.6` bytes/build while preserving
equivalence:

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

Runtime verification after the patch stayed green on build, hash, plugin,
restart, and forced-ticket gates, but the strict 50-bot 32/32 spectator run
did not beat the accepted load reference:

```text
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (30.428s)
restart/recovery: PASS, Done (20.358s)
forced-ticket persistence: PASS, first/restart Done 14.099s/9.797s
strict 50-bot gate: PASS preflight, then tps1_avg=18.04, avg_tick_ms_avg=56.39, loaded_chunks_max=2429
```

The patch is kept as a startup-work reduction, not as a 20 TPS / 500-player
claim.

## Current 2026-05-09 22:38 CEST Rejected Micro-Candidates, Current Runtime Reverified

This continuation did not keep a new production performance patch. The
`YClampedGradient.compute(...)` direct inline candidate was first promoted for
testing, then rejected after the fresh focused rerun showed it was slower on
this host/JVM:

```text
reports/yclamped-gradient-bench.txt
current_clamped_map_best_ms=25.894
optimized_inline_best_ms=26.244
optimized_speedup=0.987x
equivalence=PASS
```

The hunk was rolled back, and `javap` confirms
`DensityFunctions$YClampedGradient.compute(...)` calls `Mth.clampedMap(...)`
again. The waypoint snapshot `toArray(new Entry[map.size()])` variant was also
rejected at focused stage:

```text
reports/waypoint-snapshot-bench.txt
toArray_best_ms=815.130
sizedArray_best_ms=1042.345
sizedArray_speedup=0.782x
manual_best_ms=462.096
manual_speedup=1.764x
equivalence=PASS
```

The manual-copy waypoint snapshot remains rejected from the earlier real load
gate despite its standalone win.

Current runtime verification after rollback:

```text
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
optimized artifact sha256=6bd43f336ea79bf81aa594fa5ac1223315641912ad5e24f7f38e8a75b801667f
plugin matrix: PASS, Done (35.098s)
restart/recovery: PASS, Done (25.464s)
forced-ticket persistence: PASS, first/restart Done 15.876s/11.482s
strict 50-bot gate: BLOCKED by host preflight (`load_per_cpu=1.313`)
```

Verdict: no new benchmark-backed production win in this continuation. The
current accepted production optimization remains the density visitor hook work
below.

## Current 2026-05-09 22:06 CEST Density Visitor Hook Production Path

`DensityFunctions.HolderHolder.mapAll(...)` and
`DensityFunctions.MarkerOrMarked.mapAll(...)` now use the existing visitor
hooks instead of always allocating temporary wrapper objects that
`NoiseChunk` and `RandomState` immediately unwrap. The focused microbench
shows the same functional result with much lower wrapper churn:

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

Production verification is still compatibility-only, not an end-to-end load
claim:

```text
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
javap HolderHolder.mapAll: applyHolder hook is in bytecode
javap MarkerOrMarked.mapAll: applyMarker hook is in bytecode
plugin matrix: PASS, Done (38.265s)
restart/recovery: PASS, Done (29.923s)
forced-ticket persistence: PASS, first/restart Done 21.751s/15.859s
strict 50-bot gate: BLOCKED by host preflight (`load_per_cpu=1.679`)
```

## Current 2026-05-09 21:36 CEST Rejected Jigsaw Target-First And Waypoint Distance Guards

`JigsawBlock.canAttach(...)` target-first evaluation kept boolean semantics in
the focused harness, but the clean strict 50-bot 32/32 spectator gate failed
the accepted load reference. It was rolled back from production. A subsequent
`WaypointTransmitter` distance-guard candidate was also rejected at focused
benchmark stage.

Rejected Jigsaw focused evidence:

```text
reports/jigsaw-canattach-bench.txt
old_can_attach_best_ms=1119.278
optimized_can_attach_best_ms=860.110
target_first_can_attach_best_ms=90.603
optimized_speedup=1.301x
target_first_speedup=12.354x
equivalence=PASS
```

Rejected Jigsaw strict load gate:

```text
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
stability_failures=0
```

Post-rollback artifact and compatibility gates:

```text
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
optimized artifact sha256=9ec8accb68af16ab4b4aef208a937668bea39aaefd50fdb6f0d7d2b808a826ea
app-cds sha256=13ba91c686442eefb93b0e8c82837e5547f61259a0db98358ea9107d21c91b6c
plugin matrix: PASS, Done (31.794s), 11 plugins initialized
restart/recovery: PASS, Done (22.176s)
forced-ticket persistence: PASS, first/restart Done 20.284s/13.382s
```

Rejected waypoint distance guard focused evidence:

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

Post-rollback strict control:

```text
reports/load-50bots-jigsaw-targetfirst-postrollback-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=1.243
max_load_per_cpu=0.750
```

Verdict: rejected and rolled back. The current runtime has no new end-to-end
TPS/MSPT win from these two candidates.

## Current 2026-05-09 20:51 CEST DensityFunctions Ap2 ADD Scratch Candidate (rejected)

This candidate targeted `DensityFunctions.Ap2.fillArray(...)` for
`ADD`. It replaces one per-call temporary `double[]` allocation with a
per-thread scratch buffer, plus an `inUse` fallback for nested/reentrant calls.
The focused benchmark was strongly positive and bit-equivalent, but the
clean strict 50-bot 32/32 spectator gate failed the accepted reference, so the
patch was rolled back from production.

Focused benchmark evidence:

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

Native parity evidence:

```text
reports/native-density-ap2-fill-bench.txt
old_flat_java_best_ms=1029.241
old_flat_native_best_ms=748.883
old_flat_native_speedup_vs_java=1.374x
scratch_flat_java_best_ms=361.825
scratch_flat_native_best_ms=627.639
scratch_flat_native_speedup_vs_java=0.576x
old_nested_java_best_ms=1952.954
old_nested_native_best_ms=1220.833
old_nested_native_speedup_vs_java=1.600x
scratch_nested_java_best_ms=1285.807
scratch_nested_native_best_ms=1283.728
scratch_nested_native_speedup_vs_java=1.002x
equivalence=PASS
reentrant_equivalence=PASS
```

Rejected strict 50-bot 32/32 spectator gate:

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
stability_failures=0
```

Post-rollback artifact and compatibility gates:

```text
applyPatches: PASS, Applied 913 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
artifacts.json: PASS
plugin matrix: PASS, Done (39.577s), 11 plugins initialized
restart/recovery: PASS, Done (21.865s)
forced-ticket persistence: PASS, first/restart Done 18.679s/11.969s
```

Verdict: rejected and rolled back. The run was stable, but worse than the
accepted reference around `18.27 TPS / 47.85 ms / 2380 chunks`; no end-to-end
load win is claimed.

## Current 2026-05-09 20:12 CEST Entity Bounding-Box Shortcut Candidate (rejected)

The `Entity.setPosRaw(...)` direct dimensions-based bounding-box shortcut
reduced the standalone allocation/model cost, but failed the comparable real
load gate and was rolled back from the production patch layer.

Focused benchmark evidence:

```text
reports/entity-bounding-box-bench.txt
old_make_then_set_best_ms=748.115
direct_dimensions_set_best_ms=525.432
direct_dimensions_speedup=1.424x
old_allocated_bytes=1536000000
direct_allocated_bytes=768000000
saved_allocated_bytes=768000000
equivalence=PASS
```

Strict 50-bot 32/32 spectator gate:

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

Verdict: rejected and rolled back. The run was stable but worse than the
accepted `18.27 TPS / 47.85 ms / 2380 chunks` reference, so no end-to-end load
or 500-player claim is made.

## Current 2026-05-09 18:45 CEST ReferenceList SmallMode Candidate (rejected)

The latest `ReferenceList` experiment added an explicit small-mode state on the
runtime list to avoid repeated hash-map churn around the linear threshold. The
focused benchmark was positive, but the live 50-bot evidence was not good
enough for production, so the change was rolled back.

Focused benchmark evidence:

```text
reports/reference-list-smallmode-state-rejected-bench.txt
single_candidate_speedup_vs_old=2.379x
pair_candidate_speedup_vs_old=2.327x
dense_candidate_speedup_vs_old=0.827x
single_runtime_speedup_vs_old=2.056x
pair_runtime_speedup_vs_old=2.293x
dense_runtime_speedup_vs_old=0.729x
verdict=REJECTED_AND_ROLLED_BACK
```

Noisy diagnostic evidence:

```text
reports/load-50bots-referencelist-smallmode-state-noisy-20260509-summary.txt
online_max=50
loaded_chunks_max=824
tps1_avg=18.50
avg_tick_ms_avg=35.14
watchdog_thread_dumps=6
nearby_players_stack_hits=13
```

The strict 50-bot gate was blocked by busy-host preflight
(`load_per_cpu=0.840`), so no accepted load win is claimed.

## Current 2026-05-09 18:10 CEST POI Main-Thread Fix And Waypoint Skip Cycle

The latest cycle combined a provisional `ServerWaypointManager`
complete-row skip with a root-cause fix for POI updates in
`ServerLevel.updatePOIOnBlockStateChange(...)`. The skip candidate still lacks
an accepted strict 50-bot load win because the host is too busy for a clean
comparison. The POI fix is stability evidence, not a performance claim.

Rebuilt artifact gates:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (57.490s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (44.201s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 32.221s/20.055s
```

Strict load gate:

```text
reports/load-50bots-poi-mainthread-gate-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=1.824
```

Noisy diagnostic:

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

## Current 2026-05-09 15:55 CEST Waypoint Snapshot And Ticket Candidates (rejected)

Rejected before or after real-gate validation:

- `TicketSet` unchecked/linear search variants: rejected at focused bench.
- cached ticket compare metadata: rejected at focused bench.
- `ServerWaypointManager.snapshotEntries(...)` manual copy: rejected after the
  real 50-bot 32/32 spectator gate.

Focused ticket search benchmark:

```text
reports/ticketset-search-bench.txt
binary_best_ms=856.032
unchecked_binary_speedup=0.966x
linear4_speedup=0.945x
linear8_speedup=0.959x
linear12_speedup=0.973x
equivalence=PASS
```

Focused ticket compare benchmark:

```text
reports/ticket-compare-bench.txt
old_best_ms=168.504
cached_best_ms=169.166
cached_speedup=0.996x
equivalence=PASS
```

Focused waypoint snapshot benchmark:

```text
reports/waypoint-snapshot-bench.txt
toArray_best_ms=795.043
manual_best_ms=489.372
manual_speedup=1.625x
equivalence=PASS
```

Real load gate for the manual snapshot candidate:

```text
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

Verdict: rejected. The standalone waypoint snapshot shape was faster, but the
real candidate gate failed the accepted reference line around
`18.27 TPS / 47.85 ms / 2380 chunks` and introduced watchdog dumps. Production
is restored to `map.entrySet().toArray(Entry[]::new)`.

Rollback/current runtime verification:

```text
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.440s)
restart/recovery: PASS, Done (17.388s)
forced-ticket persistence: PASS, first/restart Done 13.805s/9.338s
```

No 20 TPS stable claim and no 500-player claim are made from this cycle.

## Current 2026-05-09 14:27 CEST ChunkHolderManager Transient Entity-Chunk Lazy-Init Candidate

Candidate:

```text
Move `AtomicBoolean` and `Thread.currentThread()` setup in
`ChunkHolderManager.getOrCreateEntityChunk(...)` behind the
`!transientChunk` branch so the transient entity-chunk path stops allocating
those objects unnecessarily.
```

Focused benchmark:

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

Compatibility and runtime gates:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (30.140s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, Done (19.105s)
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh: PASS, first/restart Done 14.037s/9.349s
```

Strict load gate:

```text
reports/load-50bots-entitychunk-lazy-transient-gate-20260509-preflight.txt
host_preflight_ok=false
load_per_cpu=1.003
```

Noisy diagnostic:

```text
reports/load-50bots-entitychunk-lazy-transient-noisy-20260509-summary.txt
online_max=50
tps1_avg=18.21
avg_tick_ms_avg=63.30
loaded_chunks_max=2295
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

Verdict: allocation reduction is real, but the strict comparable load proof is
blocked on this host, so there is no accepted 20 TPS or 500-player claim.

## Current 2026-05-09 13:30 CEST CaveWorldCarver Floor-Skip Candidate (rejected)

Candidate:

```text
Replace `CaveWorldCarver`'s per-cave capturing `CarveSkipChecker` lambda with a
direct floor-level helper path that keeps the same ellipsoid bounds and skip
predicate.
```

Focused benchmark:

```text
reports/cave-carver-skip-bench.txt
old_lambda_best_ms=59.294
reused_checker_best_ms=58.955
direct_helper_best_ms=50.624
direct_helper_speedup=1.171x
old_checker_allocations_per_run=480000
direct_checker_allocations_per_run=0
equivalence=PASS
```

Strict 50-bot 32/32 spectator gate on the candidate:

```text
reports/load-50bots-cavecarver-floor-skip-gate-20260509-summary.txt
online_max=50
tps1_avg=17.79
avg_tick_ms_avg=108.48
loaded_chunks_max=1867
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

Rollback verification:

```text
build_optimized.sh after rollback: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix after rollback: PASS, Done (27.768s)
restart/recovery after rollback: PASS, Done (17.133s)
forced-ticket persistence after rollback: PASS, first/restart Done 14.395s/9.335s
patch removed from production: PASS
```

Verdict: rejected. The direct helper shape was faster in the synthetic
skip-checker benchmark, but the real server load gate regressed versus the
accepted reference line around `18.27 TPS / 47.85 ms / 2380 chunks`, so
`0044-Optimize-CaveWorldCarver-floor-skip-path.patch` was removed.

## Current 2026-05-09 12:45 CEST Marker ApplyMarker Hook Candidate (rejected)

Candidate:

```text
Change `DensityFunctions.MarkerOrMarked.mapAll(...)` to call the visitor's
`applyMarker(...)` hook instead of constructing a fresh `Marker` node
unconditionally.
```

Focused benchmark:

```text
reports/marker-cache-bench.txt
old_best_ms=175.121
cached_best_ms=35.148
cached_speedup=4.982x
old_marker_allocations=1920000
cached_marker_allocations=84000
equivalence=PASS
```

Strict 50-bot 32/32 spectator gate on the candidate:

```text
reports/load-50bots-marker-hook-gate-20260509-summary.txt
online_max=50
tps1_avg=17.84
avg_tick_ms_avg=67.37
loaded_chunks_max=2081
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

Rollback verification:

```text
build_optimized.sh after rollback: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
patch removed from production: PASS
```

Verdict: rejected. The marker-allocation microbench is positive, but the real
server gate regressed versus the accepted reference line, so the production
patch was removed.

## Current 2026-05-09 12:29 CEST BlendedNoise Octave Cache Candidate (rejected)

Candidate:

```text
Cache `BlendedNoise` octave arrays at construction and index arrays in
`compute(...)` instead of repeatedly calling `PerlinNoise.getOctaveNoise(...)`.
```

Focused benchmark:

```text
reports/blended-noise-octaves-bench.txt
old_getoctave_best_ms=675.507
cached_octaves_best_ms=573.567
cached_octaves_speedup=1.178x
equivalence=PASS
```

Strict 50-bot 32/32 spectator gate on the candidate:

```text
reports/load-50bots-blended-octave-cache-gate-20260509-summary.txt
online_max=50
tps1_avg=17.93
avg_tick_ms_avg=56.72
loaded_chunks_max=2079
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

Rollback verification:

```text
build_optimized.sh after rollback: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix after rollback: PASS, Done (28.079s)
restart/recovery after rollback: PASS, Done (17.050s)
forced-ticket persistence after rollback: PASS, first/restart Done 12.727s/8.805s
generated BlendedNoise.java no longer has cached octave fields
```

Post-rollback strict 50-bot baseline control:

```text
reports/load-50bots-blended-octave-cache-rollback-20260509-summary.txt
online_max=50
tps1_avg=17.85
avg_tick_ms_avg=56.02
loaded_chunks_max=2176
watchdog_thread_dumps=0
sync_load_stack_hits=0
bot_errors_max=0
```

Verdict: rejected. The standalone benchmark was positive, but the real server
gate failed the accepted reference line around `18.27 TPS / 47.85 ms / 2380
chunks`, so `0044-Cache-BlendedNoise-octave-lookups.patch` was removed from
production.

## Current 2026-05-09 11:51 CEST EntityLookup Movement Candidates (rejected)

Two movement-path candidates were tested against the current 50-bot 32/32
spectator gate and rolled back:

- direct `FullChunkStatus -> Visibility` mapping in `EntityLookup.getEntityStatus(...)`;
- section-change-only status reads in `EntityCallback.onMove()`.

Focused microbench on the direct-status shape:

```text
reports/entity-lookup-status-bench.txt
old_status_best_ms=232.665
direct_status_best_ms=224.039
direct_status_speedup=1.039x
old_accessible_best_ms=353.857
direct_accessible_best_ms=335.878
direct_accessible_speedup=1.054x
equivalence=PASS
```

Strict/direct load run:

```text
reports/load-50bots-entitylookup-direct-gate-20260509-summary.txt
online_max=50
tps1_avg=17.53
avg_tick_ms_avg=46.96
loaded_chunks_max=2083
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Section-change skip attempt:

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

Restored baseline run:

```text
reports/load-50bots-baseline-restored-20260509-summary.txt
online_max=50
tps1_avg=17.66
avg_tick_ms_avg=47.78
loaded_chunks_max=1964
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

Verdict: no accepted gain. The direct-status shape helped the standalone
microbench, but both the strict load run and the noisy skip run failed to beat
the accepted 50-bot baseline, so the repo is back on the original
`Visibility.fromFullChunkStatus(...)` / old `onMove()` path.

## Current 2026-05-09 06:02 CEST ReferenceList Sparse NearbyPlayers Candidate (limit 64, rejected)

Candidate:

```text
`ReferenceList` keeps its default hash-index behavior unless constructed with a
small-list threshold. `NearbyPlayers.TrackedChunk` player lists were tested at
threshold 64 so singleton/pair chunk watch lists could use linear array search
and avoid `Reference2IntOpenHashMap.removeInt(...)` / `shiftKeys`.
```

Focused runtime benchmark after rebuild:

```text
reports/reference-list-threshold64-bench.txt
single_runtime_speedup_vs_old=2.133x
pair_runtime_speedup_vs_old=1.811x
dense_runtime_speedup_vs_old=1.119x
```

Noisy diagnostic on the same shape:

```text
reports/load-50bots-referencelist64-noisy-20260509-0612-summary.txt
online_max=50
tps1_avg=17.67
avg_tick_ms_avg=65.91
loaded_chunks_max=2001
watchdog_thread_dumps=1
nearby_players_stack_hits=4
```

Strict 50-bot command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-referencelist64-strict-20260509-0610 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Result: blocked by host preflight, no accepted strict load result.

```text
reports/load-50bots-referencelist64-strict-20260509-0610-preflight.txt
host_preflight_ok=false
load_per_cpu=0.809
max_load_per_cpu=0.750
```

Verdict: benchmark-only gain. The 64-limit shape looked better in the isolated
microbench, but the noisy movement profile regressed and the server-thread
jstack landed in `ReferenceList.add(...)` under `NearbyPlayers.tickPlayer`, so
the experiment was rolled back to the threshold-2 baseline.

## Current 2026-05-09 04:00 CEST Load Gate Status

Rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
applySourcePatches: Applied 912 patches
createMojmapBundlerJar: PASS
runtime remap hash: 7D8197836863DC2647D53F142E738251AF2ADDD919D5FA6054EFCCF17946F33A
```

Runtime gates:

```text
plugin matrix: PASS, Done (27.869s)
restart/recovery: PASS, Done (18.230s)
forced-ticket persistence: PASS, first/restart Done 14.263s/9.582s
```

Focused benchmark for `PlacedFeature.placeWithContext(...)`:

```text
reports/placed-feature-traversal-bench.txt
equivalence=PASS
stream_total_ns=393666514
recursive_total_ns=276173886
speedup=1.425x
```

Strict 50-bot command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-placedfeature-traversal-gate-20260509-0558 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Result:

```text
reports/load-50bots-placedfeature-traversal-gate-20260509-0558-preflight.txt
host_preflight_ok=true
load_per_cpu=0.693

reports/load-50bots-placedfeature-traversal-gate-20260509-0558-summary.txt
online_max=50
tps1_avg=17.71
avg_tick_ms_avg=42.70
loaded_chunks_max=1928
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

No 20 TPS stable claim is made from this run, and no 500-player claim is made.

## Current 2026-05-09 03:30 CEST Load Gate Status

Rebuilt runtime:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
applySourcePatches: Applied 912 patches
createMojmapBundlerJar: PASS
runtime remap hash: FBE33F5C9C15DFE407681ED1912619F0809570B13565512F7ABAD53BA7E2EB5C
```

Runtime gates:

```text
plugin matrix: PASS, Done (30.599s)
restart/recovery: PASS, Done (18.990s)
forced-ticket persistence: PASS, first/restart Done 14.791s/10.665s
```

Strict 50-bot command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-spectator-nosyncload-reset-gate-20260509-0355 BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 LOAD_TEST_GAMEMODE=spectator BOT_SPEED=48 BOT_RAMP_SECONDS=60 ./scripts/run_load_test.sh
```

Result: blocked by benchmark preflight, not an accepted run.

```text
reports/load-50bots-spectator-nosyncload-reset-gate-20260509-0355-preflight.txt
host_preflight_ok=false
load_per_cpu=0.885
idle_percent_1s=66.36
max_load_per_cpu=0.750
```

No end-to-end performance claim is made from this blocked gate.

## Current Runtime Revalidation After Build Restore

Build path restoration:

```text
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
applySourcePatches: Applied 912 patches
applyFeaturePatches: PASS
createMojmapBundlerJar: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
runtime remap hash: FBE33F5C9C15DFE407681ED1912619F0809570B13565512F7ABAD53BA7E2EB5C
```

Compatibility/runtime gates:

```text
plugin matrix: PASS, Done (27.348s)
restart/recovery: PASS, Done (17.499s)
forced-ticket persistence: PASS, first/restart Done 15.120s/9.131s
```

Pinned 50-bot 32/32 spectator load:

```text
reports/load-50bots-buildrestore-gate-20260509-0157-summary.txt
cpu pinning: BENCHMARK_CPUSET=6-11
worker_line=[01:57:03 INFO]: [MoonriseCommon] Paper is using 6 worker threads, 1 I/O threads
online_max=50
tps1_min=18.73
tps1_avg=19.52
avg_tick_ms_max=40.74
avg_tick_ms_avg=26.57
loaded_chunks_max=1406
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=8
sync_load_stack_hits=7
```

This is useful evidence but not an accepted baseline: TPS/MSPT are better than
the older accepted `18.27/47.85/2380` run, but stability failed because Paper
printed server-thread dumps and the server thread repeatedly blocked in
`ServerChunkCache.syncLoad` from player movement.

No-cpuset diagnostic:

```text
reports/load-50bots-buildrestore-nocpuset-20260509-0200-summary.txt
worker_line=[02:05:10 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
tps1_avg=16.79
avg_tick_ms_avg=353.82
loaded_chunks_max=4764
watchdog_thread_dumps=5
sync_load_stack_hits=5
```

Verdict: rejected config signal. More worker/I/O threads loaded more chunks but
caused much worse latency and still did not remove sync-load stalls. No
50-bot, 500-bot, or 20 TPS claim is made from this run.

## Current Candidate: NoiseChunk Marker Wrapper Cache

Candidate:

```text
`NoiseChunk` wrapping visitor now caches wrappers for repeated
`DensityFunctions.MarkerOrMarked` nodes through the existing reference
`wrapped` map. The mapped child function and wrapper type are unchanged; only
duplicate wrapper allocation for shared marker nodes is avoided.
```

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

Verification:

```text
rebuildPatches: PASS, Rebuilt 912 patches
build_optimized.sh: PASS, bytecode confirmed by javap
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (31.651s)
restart/recovery: PASS, Done (18.882s)
forced-ticket persistence: PASS, first/restart Done 15.372s/10.768s
```

Strict 50-bot gate:

```text
reports/load-50bots-marker-cache-gate-20260508-preflight.txt
host_preflight_ok=false
load_per_cpu=0.807
max_load_per_cpu=0.750
```

Clean strict 50-bot rerun:

```text
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
```

Noisy diagnostic-only 50-bot run:

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

Verdict: kept only as a narrow allocation/microbench win pending a clean
strict load gate. The clean rerun improved raw TPS/MSPT but failed promotion
because watchdog dumps and low chunk coverage violate the accepted load
standard. No 50-bot, 500-bot, or 20 TPS claim is made.

## Latest Microbench Rejected Candidate: Climate Parameter distance branches

Candidate:

```text
Replace the `Math.max(...)`-based `Climate.Parameter.distance(...)` shape with
explicit branches or subtract-first branches.
```

Benchmark:

```text
reports/climate-parameter-distance-bench.txt
old_distance_best_ms=194.276
branch_distance_best_ms=202.207
branch_distance_speedup=0.961x
subtract_first_distance_best_ms=195.008
subtract_first_speedup=0.996x
equivalence=PASS
```

Verdict: rejected before production. The explicit branch shape is slower on
this CPU/JIT, and the subtract-first shape is effectively neutral.

## Latest Microbench Rejected Candidate: ImprovedNoise Derivative Inline Permutation Lookup

Candidate:

```text
Inline direct `byte[]` permutation access inside
`ImprovedNoise.sampleWithDerivative(...)` instead of 14 calls to private
`p(int)`.
```

Benchmark:

```text
reports/improved-noise-derivative-bench.txt
old_derivative_best_ms=56.989
inline_derivative_best_ms=57.170
inline_derivative_speedup=0.997x
equivalence=PASS
```

Verdict: rejected before production. It is bit-exact but does not improve on
this CPU/JIT shape, so no production source patch was added.

## Current Candidate: OreFeature Exact Loop Cleanup

Candidate:

```text
`OreFeature.doPlace(...)` now caches the repeated `d5 * d5` and
`d5 * d5 + d6 * d6` intermediates and hoists `width * height` out of the
innermost index calculation. Reciprocal-multiply replacement was rejected
because it can change floating-point boundary behavior and ore placement.
```

Benchmark:

```text
reports/ore-feature-loop-bench.txt
old_loop_best_ms=60.507
optimized_loop_best_ms=58.403
optimized_speedup=1.036x
equivalence=PASS
```

Verdict: built and kept pending a clean strict load gate. The 50-bot
preflight is currently blocked by host load, and the noisy diagnostic run is
not an accepted baseline comparison.

## Latest Rejected Candidate: Beardifier bury contribution branch

Candidate:

```text
`Beardifier.getBuryContribution(...)` replaced the general
`Mth.clampedMap(len, 0.0, 6.0, 1.0, 0.0)` path with an equivalent direct
branch for `len > 6.0` plus `1.0 - len / 6.0`.
```

Focused benchmark:

```text
reports/beardifier-bury-bench.txt
current_clamped_map_best_ms=8.304
optimized_branch_best_ms=7.063
optimized_speedup=1.176x
equivalence=PASS
```

Strict 50-bot gate:

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

Post-revert strict gate:

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

Verdict: rejected and reverted. The microbench was positive, but the real gate
did not beat the accepted baseline. No production Beardifier patch is kept.

## Current Measured Candidate: ProtoChunk heightmap iterator removal

Candidate:

```text
`ProtoChunk.setBlockState(...)` now walks cached `Heightmap.Types[]` values
and uses `EnumSet.contains(...)` instead of `EnumSet` iteration, removing the
iterator allocation from the hot heightmap update path.
```

Benchmark:

```text
reports/protochunk-heightmap-bench.txt
old_enumset_foreach_best_ms=138.483
new_cached_values_contains_best_ms=105.978
new_speedup=1.307x
old_iterator_allocations_per_setblock=2
new_iterator_allocations_per_setblock=0
equivalence=PASS
```

Verdict: built and kept, with the durable patch in
`paper-server/patches/sources/net/minecraft/world/level/chunk/ProtoChunk.java.patch`.
The strict 50-bot 32/32 spectator gate is still blocked by host preflight
(`load_per_cpu=0.792` > `0.750`), so this is not an end-to-end 50-bot or
500-bot load claim.

No unsupported performance claim is made. Every number below comes from commands run in this checkout.

## Rejected Candidate: RangeChoice constant-out fillArray

Candidate:

```text
`DensityFunctions.RangeChoice.fillArray(...)` temporarily special-cased
constant `whenOutOfRange` values, and skipped all child context calls when
both branches were constant.
```

Benchmark:

```text
reports/range-choice-bench.txt
samples=1000000
equivalence=PASS

scenario=in_constant_out_dynamic
old_fillarray_best_ms=9.947
optimized_fillarray_best_ms=9.124
optimized_fillarray_speedup=1.090x
old_for_index_calls=1000000
optimized_for_index_calls=400495

scenario=in_dynamic_out_constant
old_fillarray_best_ms=9.977
optimized_fillarray_best_ms=9.507
optimized_fillarray_speedup=1.049x
old_for_index_calls=1000000
optimized_for_index_calls=599505

scenario=both_constant
old_fillarray_best_ms=10.004
optimized_fillarray_best_ms=7.321
optimized_fillarray_speedup=1.366x
old_for_index_calls=1000000
optimized_for_index_calls=0

scenario=both_dynamic
old_fillarray_best_ms=10.501
optimized_fillarray_best_ms=10.742
optimized_fillarray_speedup=0.978x
old_for_index_calls=1000000
optimized_for_index_calls=1000000
```

Strict server gate:

```text
reports/load-50bots-rangechoice-constant-out-gate-20260510-summary.txt
host_preflight_ok=true
online_max=50
tps1_avg=17.63
avg_tick_ms_avg=192.39
loaded_chunks_max=2768
watchdog_thread_dumps=5
sync_load_stack_hits=0
nearby_players_stack_hits=4
stability_failures=0
```

Verdict: rejected and removed from production. Patch
`0041-Optimize-RangeChoice-constant-out-fillArray.patch` was deleted,
`applyPatches` rebuilt without `RangeChoiceConstantOut`, and the rollback
runtime passes artifact hashes, plugin matrix `Done (26.927s)`,
restart/recovery `Done (16.461s)`, and forced-ticket persistence
`12.714s/8.332s`.

## Latest Rejected Microbench: BiomeManager GetBiome Early-Exit

Candidate:

```text
Try to short-circuit `BiomeManager.getBiome(...)` corner evaluation with
safe lower-bound distance checks inspired by DivineMC/Carpet-Fixes.
```

Benchmark:

```text
reports/biome-getbiome-bench.txt
samples=1000000
verify_samples=2000000
equivalence=PASS
old_getbiome_best_ms=136.628
optimized_getbiome_best_ms=193.205
optimized_speedup=0.707x
```

Verdict: rejected at microbench stage. The safe lower-bound partial-exit shape
was slower than the current `BiomeManager.getBiome(...)` path, so no production
patch was kept.

## Latest Rejected Candidate: PalettedContainer Reencode Remap Cache

Candidate:

```text
`PalettedContainer.reencodeContents(...)` keeps a per-thread remap table from
old palette id to new palette id, on top of the existing per-thread unpack
scratch array. This avoids repeated palette lookup/id insertion work for old
palette ids that recur non-consecutively.
```

Standalone benchmark:

```text
reports/paletted-reencode-remap-cache-bench.txt
current_previous_only_best_ms=967.335
cached_palette_ids_best_ms=937.103
cached_speedup=1.032x
equivalence=PASS
```

Strict load gate:

```text
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

Post-revert production verification on the scratch-only runtime:

```text
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix PASS, Done (31.022s)
restart/recovery PASS, Done (20.065s)
forced-ticket persistence PASS, first/restart Done 15.145s/10.071s
```

Post-revert strict load gate status:

```text
reports/load-50bots-post-paletted-remap-revert-gate-rerun1-preflight.txt
host_preflight_ok=false
load_per_cpu=0.807
idle_percent_1s=57.20
max_load_per_cpu=0.750
```

Verdict: rejected and reverted. The current runtime keeps only the earlier
scratch-buffer reuse in `PalettedContainer.reencodeContents(...)`; the remap
table is not in the production path.

## Latest Rejected Candidate: NoiseChunk Interpolator Indexed Traversal

Candidate:

```text
Replace several `NoiseChunk.interpolators` foreach/forEach traversals with
indexed `List.get(...)` loops.
```

Standalone benchmark:

```text
reports/noisechunk-interpolator-array-bench.txt
list_loop_best_ms=1108.416
array_loop_best_ms=1052.171
array_speedup=1.053x
equivalence=PASS
```

Strict load gate:

```text
reports/load-50bots-current-goal-interpolator-array-gate-rerun2-summary.txt
online_max=50
tps1_avg=17.87
avg_tick_ms_avg=142.23
loaded_chunks_max=2336
watchdog_thread_dumps=0
sync_load_stack_hits=0

accepted baseline:
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Verdict: rejected and reverted. The production path is back to foreach/forEach
interpolator traversal.

## Latest Microbench Rejected Candidate: DensityFunctions.Spline Context Wrapper

Candidate:

```text
`DensityFunctions.Spline.compute(...)` temporarily stopped wrapping
`FunctionContext` in `Spline.Point` and passed the context directly through the
inner cubic spline, which removed the hot `Spline.Point` allocation site.
```

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

Production verification:

```text
./gradlew applyPatches --no-daemon: PASS, Applied 911 patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
plugin matrix PASS, Done (36.988s)
restart/recovery PASS, Done (19.526s)
forced-ticket persistence PASS, first/restart Done 21.483s/18.688s
sha256sum -c reports/artifact-hashes.txt PASS
```

Strict load gate status:

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

Verdict: rejected and reverted. The benchmark was real, but the end-to-end
load gate did not beat the accepted baseline `18.27/47.85/2380`.

## Latest Accepted-Limited Candidate: Plugin Startup Name-Log Aggregation

Candidate:

```text
`PluginInitializerManager.load(...)` now appends Paper/Bukkit plugin display
names into `ArrayList<String>`, sorts once, and deduplicates in place before
logging instead of building sorted unique name sets with `TreeSet<String>`
during iteration.
```

Standalone benchmark:

```text
reports/plugin-name-log-bench.txt
plugins=512
warmup=3 rounds=6 iterations=5000
old_treeset_best_ms=343.898
new_arraylistsort_best_ms=45.491
arraylistsort_speedup=7.560x
```

Production verification:

```text
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 911 source patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
plugin matrix PASS, Done (32.863s)
restart/recovery PASS, Done (23.341s)
forced-ticket persistence PASS, first/restart Done 21.545s/13.276s
sha256sum -c reports/artifact-hashes.txt PASS
```

Strict load gate status:

```text
reports/load-50bots-plugin-name-log-current-gate-preflight.txt
host_preflight_ok=false
load_per_cpu=0.812
idle_percent_1s=62.15
max_load_per_cpu=0.750
```

Verdict: accepted with limits as a narrow plugin-startup logging allocation/work reduction. There is no clean end-to-end cold-start speedup claim and no load/TPS claim.

## Latest Microbench Rejected Candidate: Plugin Name Manual Join

Candidate:

```text
Replace `String.join(...)` for plugin name logging with a manual `StringBuilder`
join over the already-sorted deduplicated list.
```

Standalone benchmark:

```text
reports/plugin-name-join-bench.txt
plugins=512
warmup=3 rounds=6 iterations=5000
string_join_normal_best_ms=34.712
manual_join_normal_best_ms=53.976
manual_join_normal_speedup=0.643x
string_join_debug_best_ms=36.942
manual_join_debug_best_ms=55.514
manual_join_debug_speedup=0.665x
```

Verdict: rejected before production. Manual join was slower in both normal and debug log shapes.

## Latest Microbench Rejected Candidate: Remapper Hash Hybrid/Put

Candidate:

```text
Change the sequential `RemappedPluginIndex.hashInputs(...)` branch from
`computeIfAbsent(...)` to a hybrid that keeps `computeIfAbsent(...)` only for a
single path and uses direct `put(...)` for larger sequential batches.
```

Short confirmation benchmark:

```text
reports/remapper-hash-threshold-bench.txt
size=1  compute_if_absent_best_ms=1.390  put_best_ms=1.240  hybrid_best_ms=1.349
size=2  compute_if_absent_best_ms=40.523  put_best_ms=45.803  hybrid_best_ms=42.313
size=4  compute_if_absent_best_ms=318.976 put_best_ms=321.943 hybrid_best_ms=305.037
size=8  compute_if_absent_best_ms=1333.339 put_best_ms=1401.164 hybrid_best_ms=1349.541
size=12 compute_if_absent_best_ms=4383.586 put_best_ms=4332.995 hybrid_best_ms=4466.193
```

Verdict: rejected before production. The hybrid was noisy and not consistently
better than the existing `computeIfAbsent(...)` path across small batches, so
`PARALLEL_HASH_THRESHOLD=4` and the sequential hashInputs code stay unchanged.

## Previous Accepted-Limited Candidate: Legacy Provided-Alias Reverse Index

Candidate:

```text
`LegacyPluginLoadingStrategy` tracks which provided aliases are currently owned
by each provider. Provider load/fail cleanup now removes the provider's aliases
directly instead of scanning `pluginsProvided.values().removeIf(...)` for every
provider.
```

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

Production verification:

```text
./gradlew rebuildPatches --no-daemon: PASS, Rebuilt 911 source patches
MC_EULA_AGREE=true ./scripts/build_optimized.sh: PASS
plugin matrix PASS, Done (32.124s)
restart/recovery PASS, Done (28.224s)
forced-ticket persistence PASS, first/restart Done 18.894s/11.749s
sha256sum -c reports/artifact-hashes.txt PASS
```

Verdict: accepted with limits as a legacy plugin-loading startup-work reduction.
There is no clean end-to-end cold-start speedup claim and no load/TPS claim.

## Latest Microbench Rejected Candidate: NbtIo GZIP Buffer Size

Candidate:

```text
Increase GZIP and/or pre-GZIP buffering in `NbtIo.writeCompressed(...)` while
keeping the gzip output byte-identical.
```

Standalone benchmark on a real `level.dat`:

```text
reports/nbt-gzip-write-bench.txt
gzip64k_byte_equal=true
prebuffer64k_byte_equal=true
both64k_byte_equal=true
current_best_ms=1328.262
gzip64k_best_ms=1564.097
prebuffer64k_best_ms=1580.647
both64k_best_ms=1686.143
gzip64k_speedup=0.849x
prebuffer64k_speedup=0.840x
both64k_speedup=0.788x
```

Verdict: rejected before production source changes. Bigger buffers preserved
bytes but were slower in the focused path.

## Latest Candidate: Xoroshiro Positional Direct Helpers (Rejected And Reverted)

Candidate:

```text
Add direct first-draw helpers on `PositionalRandomFactory` and override them
in `XoroshiroPositionalRandomFactory` so worldgen hot paths can skip the
allocate-a-positional-random-object step when the caller only needs the first
float/double draw.
```

Standalone benchmark:

```text
reports/xoroshiro-positional-direct-bench.txt
old_float_best_ms=25.927
direct_float_best_ms=3.845
direct_float_speedup=6.744x
old_double_best_ms=25.141
direct_double_best_ms=4.463
direct_double_speedup=5.633x
old_float_allocated_bytes_per_call=88.0
direct_float_allocated_bytes_per_call=0.0
old_double_allocated_bytes_per_call=88.0
direct_double_allocated_bytes_per_call=0.0
equivalence=PASS
```

Split-out subcandidate that was rejected first:

```text
Aquifer.NoiseBasedAquifer briefly used positionalRandomFactory.aquiferLocationAt(...)
to skip a positional RandomSource allocation for three bounded ints. The
strict 50-bot 32/32 gate regressed to 17.65 TPS / 76.12 ms / 2016 chunks, so
the Aquifer call site was restored to the old RandomSource path.
```

Second split-out subcandidate, now also rejected:

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

Post-revert production verification:

```text
applyPatches PASS, Applied 911 patches
build_optimized.sh PASS
sha256sum -c reports/artifact-hashes.txt PASS
plugin matrix PASS, Done (29.689s)
restart/recovery PASS, Done (19.938s)
forced-ticket persistence PASS, first/restart Done 17.550s/11.041s
```

The strict comparable 50-bot 32/32 rebaseline is blocked by host preflight:

```text
reports/load-50bots-post-xoroshiro-direct-revert-gate-rerun1-preflight.txt
host_preflight_ok=false
load_per_cpu=0.920
idle_percent_1s=41.04
max_load_per_cpu=0.750
```

Verdict: rejected and reverted. The helper microbench is a real win, but the
production gates did not support keeping it.

## Latest Candidate: SurfaceRules SequenceRule Array Indexed Loop (Rejected And Reverted)

Candidate:

```text
Keep `SurfaceRules.SequenceRule` runtime storage as `SurfaceRule[]`, but use an
indexed array loop in `tryApply(...)` instead of enhanced-for. Rule order,
codec/list source, and first-non-null short-circuit behavior are unchanged.
```

Standalone benchmark:

```text
reports/surfacerules-sequence-array-bench.txt
list_enhanced_best_ms=587.609
list_indexed_best_ms=565.372
array_best_ms=314.925
array_indexed_best_ms=309.618
array_speedup=1.866x
array_indexed_speedup=1.898x
equivalence=PASS
```

Production verification:

```text
rebuildPatches PASS, Rebuilt 910 source patches
applyPatches PASS, Applied 910 patches
build_optimized.sh PASS
sha256sum -c reports/artifact-hashes.txt PASS
optimized jar sha256=5613a8078e28d28c295979acdbcea3383c777ce402df50e1f240c689efbcaeb4
app-cds sha256=298476ac14be581fdb09513712ae93ea748d43f4e84a7495ba4f3b1b75f90370
plugin matrix PASS, Done (34.263s)
restart/recovery PASS, Done (21.224s)
forced-ticket persistence PASS, first/restart Done 17.307s/11.562s
```

Strict comparable 50-bot 32/32 gate:

```text
reports/load-50bots-surfacerules-array-index-gate-rerun2-summary.txt
host_preflight_ok=true
load1=6.56
load_per_cpu=0.547
idle_percent_1s=74.43
online_max=50
tps1_avg=15.95
avg_tick_ms_avg=117.42
loaded_chunks_max=1785
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Verdict: rejected and reverted. The strict gate passed preflight but did not
beat the accepted `18.27/47.85/2380` baseline.

## Latest Microbench Rejected Candidate: Aquifer Cache Index Stride Loop

Candidate:

```text
Replace the fixed 12-cell Aquifer cache loop's repeated `getIndex(...)`
calculation with a precomputed base index plus Y/Z strides.
```

Standalone benchmark:

```text
reports/aquifer-index-stride-bench.txt
old_getindex_loop_best_ms=277.865
new_stride_loop_best_ms=313.746
stride_speedup=0.886x
equivalence=PASS
```

Decision: rejected before production changes. Current `getIndex(...)` stays.

## Latest Candidate: Aquifer Surface Sampling Offsets (Rejected)

Candidate:

```text
Replace the fixed `int[][]` chunk-offset rows in
Aquifer.NoiseBasedAquifer.computeFluid(...) with two precomputed block-offset
`int[]` arrays. The sample order and sample coordinates are unchanged.
```

Standalone benchmark:

```text
reports/aquifer-surface-sampling-bench.txt
old_chunk_offsets_best_ms=275.983
new_block_offsets_best_ms=244.223
block_offsets_speedup=1.130x
equivalence=PASS
```

Functional verification before rejection:

```text
applyPatches PASS
compileJava PASS
build_optimized.sh PASS
sha256sum -c reports/artifact-hashes.txt PASS
plugin matrix PASS, Done (35.474s)
restart/recovery PASS, Done (35.817s)
forced-ticket persistence PASS, first/restart Done 27.624s/14.883s
noisy 10-bot 32/32 smoke PASS, 18.04 TPS / 49.35 ms / 1441 chunks
```

Strict 50-bot verdicts:

```text
first attempt blocked before Minecraft start by host preflight
load1=11.60
load_per_cpu=0.967
max_load_per_cpu=0.750

rerun1:
online_max=50
tps1_avg=17.14
avg_tick_ms_avg=82.71
loaded_chunks_max=2030
watchdog_thread_dumps=0
sync_load_stack_hits=0

accepted comparable baseline:
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Decision: rejected and reverted. The standalone microbench was positive, but the strict rerun regressed the accepted load baseline.

Post-revert runtime verification:

```text
applyPatches PASS, Applied 910 patches
compileJava PASS
build_optimized.sh PASS
sha256sum -c reports/artifact-hashes.txt PASS
optimized jar sha256=97720a304176d0f6fa8d222a3b1374de4390aa5debc96924ecd844e12906e3ff
mappings_hash=9383762D002E33F5BFB2E2D9BB59DBCE11135EE10227DB71E8270AB56F0AF16A
plugin matrix PASS, Done (32.234s)
restart/recovery PASS, Done (20.809s)
forced-ticket persistence PASS, first/restart Done 15.835s/11.609s
strict post-revert 50-bot gate BLOCKED by host preflight,
load1=12.50, load_per_cpu=1.041, max_load_per_cpu=0.750
noisy post-revert 10-bot smoke PASS, 19.17 TPS / 36.29 ms / 1572 chunks,
kicks/errors/watchdog/sync-load=0
```

The post-revert 10-bot smoke is explicitly non-comparable noisy-host evidence,
not a load-performance baseline.

## Latest Microbench Rejected Candidate: PerlinNoise Guarded Direct-Local getValue

Candidate:

```text
Keep `PerlinNoise.getValue(double,double,double)` subclass semantics by falling
back to the virtual six-arg method for subclasses, while using a direct-local
loop for exact `PerlinNoise` instances.
```

Standalone benchmark:

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

Decision: rejected before production. The exact-subclass-safe guarded shape is
slower than the current delegating path, and the faster `Math.floor` wrap shape
has already failed real load gates.

## Latest Microbench Rejected Candidate: ImprovedNoise Arithmetic sampleAndLerp

Candidate:

```text
Inline the C2ME/DivineMC arithmetic form of `ImprovedNoise.sampleAndLerp(...)`
with direct lerp math and flat-gradient lookup.
```

Standalone benchmark:

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

Decision: rejected before production. The arithmetic form is bit-exact, but
slower than the current flat-gradient implementation.

## Latest Microbench Rejected Candidate: ImprovedNoise Switch Gradient

Candidate:

```text
Replace the current FLAT_SIMPLEX_GRAD array lookup in ImprovedNoise.flatGradDot
with a switch expression that keeps the same arithmetic order for each gradient.
```

Standalone benchmark:

```text
reports/improved-noise-switchgrad-bench.txt
old_p_method_best_ms=43.022
inline_byte_access_best_ms=44.640
flat_gradient_best_ms=39.535
switch_gradient_best_ms=47.174
inline_speedup=0.964x
flat_gradient_speedup=1.088x
switch_gradient_speedup=0.912x
switch_vs_flat_speedup=0.838x
equivalence=PASS
```

Decision: rejected before production changes. The switch variant was bit-exact
in the benchmark but slower than the current flat-gradient table.

## Latest Rejected Candidate: NoiseChunk.FlatCache Context Allocation

Candidate:

```text
Move MutableSinglePointContext allocation inside NoiseChunk.FlatCache's
if (computeValues) branch, saving one object for false-path blendAlpha/blendOffset
FlatCache construction.
```

Standalone benchmark was positive:

```text
reports/noisechunk-flatcache-context-bench.txt
old_false_context_best_ms=100.405
new_false_context_best_ms=87.944
false_context_speedup=1.142x
old_true_context_best_ms=0.982
new_true_context_best_ms=0.901
true_context_speedup=1.089x
old_false_allocated_bytes_per_iteration=240.0
new_false_allocated_bytes_per_iteration=216.0
saved_false_allocated_bytes_per_iteration=24.0
equivalence=PASS
```

Real strict 50-bot 32/32 gate rejected it:

```text
reports/load-50bots-flatcache-context-gate-summary.txt
online_max=50
tps1_avg=15.36
avg_tick_ms_avg=254.43
loaded_chunks_max=1621
bot_kicked_max=0
bot_errors_max=0
watchdog_thread_dumps=1
sync_load_stack_hits=0
```

Decision: rejected and reverted, because the candidate regressed far below the
accepted comparable baseline (`18.27 TPS / 47.85 ms / 2380 chunks`) and hit a
watchdog thread dump. The microbench remains as evidence only.

Post-revert verification:

```text
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (38.170s)
restart/recovery: PASS, Done (22.820s)
forced-ticket persistence: PASS, first/restart Done 19.230s/18.186s
strict 50-bot post-revert gate: blocked by host preflight,
load1=16.34, load_per_cpu=1.362, max_load_per_cpu=0.750
noisy 10-bot post-revert smoke: 17.86 TPS / 39.33 ms / 939 chunks,
no kicks/errors/watchdog/sync-load
```

## Latest ImprovedNoise Candidate

`ImprovedNoise.sampleAndLerp` now uses a flat gradient table and direct
permutation-byte access inside the hot sample path. The patch is persisted as
`paper-server/patches/features/0040-Optimize-ImprovedNoise-sampleAndLerp.patch`.

Standalone benchmark:

```text
old_p_method_best_ms=48.060
inline_byte_access_best_ms=47.097
flat_gradient_best_ms=42.729
inline_speedup=1.020x
flat_gradient_speedup=1.125x
equivalence=PASS
```

Verification after build:

```text
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

Decision: accepted only as a narrow hot-path cleanup so far. No strict 50-bot
TPS/load claim is made.

## Latest Noise Generator Settings Candidate

`NoiseBasedChunkGenerator` now uses a manual double-checked primitive cache
for `getGenDepth()`, `getSeaLevel()`, and `getMinY()`. The earlier
`Supplier`-heavy variant was rejected by benchmark; the primitive cache is the
production shape.

Standalone benchmark:

```text
holder_value_settings_best_ms=108.936
memoized_supplier_settings_best_ms=208.800
lazy_primitive_settings_best_ms=84.588
manual_lazy_object_settings_best_ms=143.050
cached_int_settings_best_ms=45.127
memoized_supplier_speedup=0.522x
lazy_primitive_speedup=1.288x
manual_lazy_object_speedup=0.762x
cached_settings_speedup=2.414x
equivalence=PASS
```

Verification after build:

```text
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (39.776s)
restart/recovery: PASS, Done (47.881s)
forced-ticket persistence: PASS, first/restart Done 34.630s/15.000s
noisy 10-bot 32/32 smoke: PASS, 18.10 TPS / 42.12 ms / 679 chunks
```

The strict 50-bot 32/32 gate is currently blocked by host preflight:

```text
host_preflight_ok=false
load1=21.48
load_per_cpu=1.790
idle_percent_1s=7.98
max_load_per_cpu=0.750
```

Decision: accepted only as a narrow, safe primitive-cache cleanup. No strict
50-bot TPS/load claim is made.

## Current Rejected Candidate

`PerlinNoise.wrap(double)` briefly switched to `Math.floor(...)` in a candidate
artifact. The standalone microbench was positive and equivalent:

```text
delegating_getvalue_best_ms=748.316
direct_getvalue_best_ms=746.537
direct_local_getvalue_best_ms=745.237
direct_math_wrap_getvalue_best_ms=685.531
math_wrap_speedup=1.092x
equivalence=PASS
```

Verification after build:

```text
applyPatches: PASS, Applied 910 patches
build_optimized.sh: PASS
bytecode: PerlinNoise.wrap invokes java/lang/Math.floor
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (33.758s)
restart/recovery: PASS, Done (19.015s)
forced-ticket persistence: PASS, first/restart Done 15.816s/12.097s
```

The strict 50-bot 32/32 gate is currently blocked by host preflight, not by
the server artifact itself:

```text
host_preflight_ok=false
load1=11.21
load_per_cpu=0.934
max_load_per_cpu=0.750
```

The strict 50-bot 32/32 load gate then ran and the candidate lost on load
coverage and TPS, so it was reverted:

```text
tps1_avg=18.16
avg_tick_ms_avg=47.33
loaded_chunks_max=1720
```

Accepted baseline remained:

```text
tps1_avg=18.27
avg_tick_ms_avg=47.85
loaded_chunks_max=2380
```

Decision: rejected, despite the better tick time, because the candidate did not
beat the accepted baseline on the required strict shape.

## Hardware And Flags

```text
OS: Linux 6.8.0-110-generic x86_64
Java: OpenJDK 21.0.10+7-Ubuntu-124.04
CPU cores: 12
RAM: 62 GiB
Default Java flags in scripts: -Xms1G -Xmx2G
Load flags: -Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100
Optimized runtime default flags: -Xms256m -Xmx2G -Xshare:auto -XX:SharedArchiveFile=artifacts/optimized-runtime/app-cds.jsa
Pinned runs use BENCHMARK_CPUSET=6-11 via taskset.
```

## Latest ServerEntity Identity Guard

Candidate:

```text
Skip ServerEntity.sendChanges() motion distanceToSqr() when the current immutable Vec3 delta movement is the same object as lastSentMovement. The old path still produced d == 0 and sent nothing, so observable packet semantics are unchanged.
```

Standalone microbench:

```text
iterations=16000000
entries=16384
same_identity_percent=75
old_distance_best_ms=80.075
identity_guard_best_ms=28.626
identity_guard_speedup=2.797x
equivalence=PASS
```

Current build/runtime verification:

```text
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (26.447s)
restart/recovery: PASS, Done (15.842s)
forced-ticket persistence: PASS, first/restart Done 12.865s/8.526s
boot benchmark: vanilla 16958 ms, stock Paper 32528 ms, optimized jar 24385 ms, optimized runtime 16613 ms
50-bot 32/32: PASS WITH LIMITS, 18.85 TPS / 64.88 ms / 1829 chunks
```

Decision: accepted with limits as a narrow entity-tracker CPU reduction. No end-to-end `<1s` boot, `500 players`, or stable `20 TPS` claim is made from this cycle.

## Latest Ownable Rewrite Rule Candidate

Candidate:

```text
Replace OwnableRewriteRule.matchesOwner stream/map/anyMatch with a direct
ClassDesc loop and descriptor/owner comparison. This is a plugin class-rewrite
startup path optimization only.
```

Raw microbench from `reports/ownable-rule-bench.txt`:

```text
iterations=12000000
owners=6
queries=6
old_stream_best_ms=2052.795
new_loop_best_ms=326.972
loop_speedup=6.278x
equivalence=PASS
```

Current artifact verification after the later rejected Climate candidate was removed:

```text
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.313s)
restart/recovery: PASS, Done (17.743s)
forced-ticket persistence: PASS, first/restart Done 14.023s/9.280s
50-bot post-revert stability: PASS without kicks/errors/watchdog/sync-load
50-bot post-revert performance: not a baseline, 17.06 TPS / 68.42 ms / 2758 chunks
```

Decision: accepted with limits as class-rewrite allocation reduction. No end-to-end cold-start or TPS speedup is claimed from this change.

## Latest Rejected Climate RTree Candidate

Candidate:

```text
Add a bounded distance(long[] values, long limit) overload for the default
Climate.RTree search path, so distance calculation can stop once the partial
non-negative squared distance is already worse than the current best leaf.
Custom DistanceMetric search was not changed.
```

Raw microbench from `reports/climate-rtree-bound-bench.txt`:

```text
leaves=1024
queries=40000
old_search_best_ms=1410.567
bounded_search_best_ms=1170.869
bounded_speedup=1.205x
equivalence=PASS
```

Production gate with the patch:

```text
LOAD_TEST_LABEL=50bots-climate-rtree-bound-gate
online_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=17.65
avg_tick_ms_avg=58.37
loaded_chunks_max=2620
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Decision: rejected and reverted. It failed the accepted 50-bot baseline (`18.27 TPS / 47.85 ms / 2380 chunks`) despite a positive microbench.

## Latest ImprovedNoise Inline Candidate

Candidate:

```text
Inline the byte-array permutation lookup inside ImprovedNoise.sampleAndLerp
while keeping the same byte[] table, the same index mask, and the same gradient
math. No plugin, scheduler, command, event, service, permission, or classloader
semantics were touched.
```

Standalone microbenchmark command:

```bash
./scripts/bench_improved_noise_inline.sh
```

Raw result from `reports/improved-noise-inline-bench.txt`:

```text
iterations=1000000
warmup=4 rounds=8
old_p_method_best_ms=47.592
inline_byte_access_best_ms=42.544
inline_speedup=1.119x
equivalence=PASS
```

Temporary production build gates passed, but the strict 50-bot server gate did
not beat the accepted baseline:

```text
LOAD_TEST_LABEL=50bots-improvednoise-inline-gate
online_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=17.78
avg_tick_ms_avg=62.90
loaded_chunks_max=2693
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Decision: rejected for production. The source patch was removed and the final
artifact was rebuilt. Final post-reject gates:

```text
applyPatches: PASS, Applied 910 patches
compileJava: PASS
build_optimized.sh: PASS
sha256sum -c reports/artifact-hashes.txt: PASS
plugin matrix: PASS, Done (29.272s)
restart/recovery: PASS, Done (17.902s)
forced-ticket persistence: PASS, first/restart Done 13.935s/10.021s
```

Final comparable 50-bot rerun is blocked by host preflight, not by server
startup:

```text
LOAD_TEST_LABEL=50bots-final-after-inline-reject
host_preflight_ok=false
load1=10.03
load_per_cpu=0.835
max_load_per_cpu=0.750
idle_percent_1s=69.91
```

## Boot Benchmark

Command:

```bash
MC_EULA_AGREE=true ./scripts/boot_benchmark.sh
```

Raw result:

```csv
name,port,status_ms,done_ms,rss_kb,stop_ms,log
vanilla-1.21.10,50439,,16589,1066860,20646,/root/rust/logs/boot-vanilla-1.21.10.log
stock-paper-1.21.10,55169,,29626,1535788,33682,/root/rust/logs/boot-stock-paper-1.21.10.log
optimized-paper-1.21.10,33229,19480,19543,1419084,23600,/root/rust/logs/boot-optimized-paper-1.21.10.log
optimized-runtime-1.21.10,57167,13551,13595,1015012,17658,/root/rust/logs/boot-optimized-runtime-1.21.10.log
```

Optimized runtime external `done_ms` was `13.595s`, about `54.1%` lower than stock Paper external `29.626s` in this run. This is not `<1s`.

Latest pinned command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/boot_benchmark.sh
```

Latest pinned raw result:

```csv
name,port,status_ms,done_ms,rss_kb,stop_ms,log
vanilla-1.21.10,44089,14824,14855,1053344,18911,/root/rust/logs/boot-vanilla-1.21.10.log
stock-paper-1.21.10,57059,32713,32747,1497428,36802,/root/rust/logs/boot-stock-paper-1.21.10.log
optimized-paper-1.21.10,54959,24298,24342,1432352,28400,/root/rust/logs/boot-optimized-paper-1.21.10.log
optimized-runtime-1.21.10,51109,16444,16488,1055080,20560,/root/rust/logs/boot-optimized-runtime-1.21.10.log
```

Latest pinned optimized runtime was `16.488s`, about `49.7%` lower than same-run stock Paper `32.747s`, but worse than the earlier best `13.595s`. This is not a `<1s` startup claim; it is a same-run regression check on the current host.

## Plugin Matrix Startup

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Latest current-source result:

```text
Done (30.020s)
join_client=login username=CodexJoinProbe
COMPAT_PROBE event=PlayerJoinEvent sequence=3 detail=CodexJoinProbe
COMPAT_PROBE command=ok events=4 ownServices=0
LIBRARY_PROBE dependency=loaded-from-plugin-library
fresh .paper-remapped/index.json: hashes=4, skippedHashes=8
fresh .paper-remapped/libraries/index.json: hashes=0, skippedHashes=1
```

This run installed precomputed remapped plugin jars from mapping hash `478C3D7AE203F013AD5E055D2CF0165EC45ADD64943A054168678C09D284B223`. It also includes the DirectoryStream plugin-directory scan, `DensityFunction.Visitor` holder/marker hooks, `PalettedContainer` reencode scratch reuse, `TopographicGraphSorter` capacity pre-size cleanup, Spigot load-after pre-size cleanup, plugin-loading allocation cleanup, Spigot load-order allocation cleanup, Paper plugin metadata dependency-list cache, batch-miss hash reuse path, batch list/hash capacity hints, lazy remap-index cleanup, dirty index writes, a separate precomputed skip namespace for Paper plugin libraries, deferred mappings load for skip-only jars, ReobfServer precomputed-server-before-mappings, the `PaperReflection` stripped-method map removal plus recursive lookup key reuse and empty-descriptor shortcut, the waypoint inner-range hot-path change, and the direct `MessageDigest` stream hash path. Plugin startup is still noisy and not close to `<1s`; this is compatibility evidence, not a clean startup-speed claim.

## Direct MessageDigest Stream SHA-256

Change:

```text
Hashing.sha256(InputStream) now uses MessageDigest with a 64 KiB buffer and
HexFormat uppercase output. Hashing.sha256(Path) remains on Guava because the
same benchmark showed Guava's file path hasher faster for real plugin jars.
```

Microbenchmark command:

```bash
bash scripts/bench_hash_path.sh
```

Raw result from `reports/hash-path-bench.txt`:

```text
inputs=13
bytes=38017023
warmup=4 rounds=8 buffer=65536
guava_path_best_ms=106.561
direct_path_best_ms=122.850
direct_path_speedup=0.867x
guava_stream_best_ms=120.240
direct_stream_best_ms=119.801
direct_stream_speedup=1.004x
```

Verification:

```text
build: PASS, applySourcePatches Applied 909 patches, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (46.867s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (29.418s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 16.933s/11.230s
strict load preflight: BLOCKED, load1=14.23, load_per_cpu=1.186, idle_percent_1s=73.87
```

## Plugin Directory Scan

Change:

```text
Plugin directory discovery now uses Files.list(...) with try-with-resources for
the flat plugins directory instead of Files.walk(..., depth=1). Add-plugin flag
conversion skips its no-op provider path when no --add-plugin files are present,
and plugin-list log-name formatting avoids small stream/Formatter allocation
paths. Plugin order guarantees, remap keys, classloader URLs, event order,
scheduler behavior, and services are unchanged.
```

Microbenchmark command:

```bash
scripts/bench_plugin_scan.sh
```

Raw result from `reports/plugin-scan-bench.txt`:

```text
directory=/root/rust/plugins/matrix
plugins_per_scan=12
warmup=4 rounds=8 iterations=5000
walk_depth1_best_ms=220.139
list_best_ms=123.419
list_speedup=1.784x
```

Functional gates:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (39.186s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (27.825s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 16.175s/11.521s
50-bot 32/32 gate: BLOCKED before starting Minecraft, load1=19.50, load_per_cpu=1.625, idle_percent_1s=71.20
artifact hash: optimized-paper-1.21.10-mojmap.jar 8b84e90ee1d1c29947a449f8d2419698597434acf15039dab9c553f8c1e6bc55
```

This is accepted as a narrow startup/plugin-discovery work reduction, not as an
end-to-end cold-start speedup claim. The clean 50-bot/500-bot verdict remains
blocked by shared host load.

## Paper Plugin Metadata Dependency Lists And Cache

Change:

```text
PaperPluginMeta dependency-list accessors now use direct loops with lazy
ArrayList allocation and List.copyOf(...) instead of stream/filter/map/toList.
The returned lists remain immutable and preserve dependency iteration order.
The computed immutable lists are cached inside each PaperPluginMeta instance, so
load order, classloader, and diagnostics paths do not rebuild the same lists.
Server plugin onLoad logging also avoids a String.format call.
```

Microbenchmark command:

```bash
scripts/bench_plugin_meta_dependencies.sh
```

Raw result from `reports/plugin-meta-dependencies-bench.txt`:

```text
dependencies=12
warmup=6 rounds=10 iterations=2000000
old_stream_best_ms=1960.882
new_loop_best_ms=566.406
cached_best_ms=5.926
loop_speedup=3.462x
cached_vs_loop_speedup=95.586x
```

Functional gates:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (32.283s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (18.863s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 15.387s/9.792s
50-bot 32/32 gate: BLOCKED before starting Minecraft, load1=11.50, load_per_cpu=0.958, idle_percent_1s=74.85
artifact hash: optimized-paper-1.21.10-mojmap.jar 3d04f4ecdaeae58a009e4fdb7762aaa18674b3014125d53a2b347a7d010f5ba3
```

This is accepted as Paper plugin metadata startup/dependency-resolution work
reduction. It is not an end-to-end cold-start or TPS claim.

## Spigot Load Order Back-Reference And Load-After Pre-Size

Change:

```text
SpigotLoadOrderConfiguration no longer allocates a temporary HashSet for each
dependency-provider back-reference check. It checks the provider name, hard
dependencies, and soft dependencies directly. The result is the same union
membership test used before, with no change to load order rules.

The `loadAfter` list is now also created with capacity `depend.size +
softDepend.size`, avoiding default-capacity ArrayList growth while preserving
the existing dependency order and load-order semantics.
```

Microbenchmark command:

```bash
scripts/bench_spigot_load_order.sh
```

Raw result from `reports/spigot-load-order-bench.txt`:

```text
load_after=8
dependencies_per_provider=6
warmup=6 rounds=10 iterations=2000000
old_load_after_build_best_ms=146.978
new_load_after_presized_build_best_ms=121.139
load_after_build_speedup=1.213x
old_hashset_best_ms=2631.046
new_contains_best_ms=409.024
contains_speedup=6.433x
```

Functional gates:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (28.874s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (19.041s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 14.910s/10.688s
50-bot 32/32 gate: BLOCKED before starting Minecraft, load1=9.95, load_per_cpu=0.829, idle_percent_1s=49.49
artifact hash: optimized-paper-1.21.10-mojmap.jar 203bac9244f5b0410a712cc6dadf7e88203403b9aee8812e2337abf4eac48640
```

This is accepted as Bukkit/Spigot load-order allocation reduction. It is not an
end-to-end cold-start or TPS claim.

## TopographicGraphSorter Capacity Pre-Size

Change:

```text
Plugin load-order topological sort now creates its sorted list, roots deque,
and Object2IntOpenHashMap with the known graph node count. Graph traversal,
cycle detection, dependency rules, classloading, scheduler, services, and event
semantics are unchanged.
```

Microbenchmark command:

```bash
scripts/bench_topographic_sort.sh
```

Raw result from `reports/topographic-sort-bench.txt`:

```text
nodes=256
edges_per_node=4
warmup=4 rounds=8 iterations=20000
old_default_capacity_best_ms=633.295
new_presized_best_ms=428.129
presized_speedup=1.479x
```

Functional gates:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (28.578s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (17.361s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 13.774s/9.489s
50-bot 32/32 gate: PASS WITH LIMITS, 50 connected/ready/active, no kicks/errors/watchdog/sync-load, but not a new baseline: tps1_avg=16.93, avg_tick_ms_avg=145.06, loaded_chunks_max=2005
artifact hash: optimized-paper-1.21.10-mojmap.jar 1ea271b1902436469d15e5d7ad30875d0a6b20fa00d02c4a1d48488c5a6f6f42
```

This is accepted as plugin load-order allocation reduction. It is not an
end-to-end cold-start, TPS, MSPT, or 500-player claim.

## Current 50-Bot JFR And GC Checks

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-toposort-current-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-toposort-current-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Raw result:

```text
preflight: PASS, load1=7.59, load_per_cpu=0.632, idle_percent_1s=83.04
online_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=17.65
avg_tick_ms_avg=68.31
loaded_chunks_max=1975
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

## DensityFunction Visitor Holder/Marker Hooks

Change:

```text
DensityFunction.Visitor gained default applyHolder(...) and applyMarker(...)
hooks. Generic visitors keep the old behavior because the default methods
construct the same HolderHolder/Marker wrappers and call apply(...). NoiseChunk
and RandomState override only the unwrapping cases where those wrappers were
immediately converted back to mappedFunction/mappedWrapped.
```

Microbenchmark:

```bash
scripts/bench_density_visitor_hooks.sh
```

Raw result from `reports/density-visitor-hooks-bench.txt`:

```text
roots=256
depth=40
iterations=600
old_best_ms=481.076
hooked_best_ms=20.770
hooked_speedup=23.162x
old_temp_holder_allocations=3072000
old_temp_marker_allocations=3072000
hooked_temp_holder_allocations=0
hooked_temp_marker_allocations=0
equivalence=PASS
```

Functional gates:

```text
applyPatches: PASS, Applied 910 patches
compileJava: PASS
build: PASS, applySourcePatches Applied 910 patches, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
artifact hashes: PASS
plugin matrix: PASS, Done (29.184s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (25.338s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 17.691s/10.039s
strict 50-bot 32/32: BLOCKED before Minecraft start, load1=13.07, load_per_cpu=1.089, idle_percent_1s=66.35
noisy 10-bot 32/32: PASS WITH LIMITS, online_max=10, tps1_avg=19.55, avg_tick_ms_avg=31.39, loaded_chunks_max=1842, kicks/errors/watchdog/sync-load=0
artifact hash: optimized-paper-1.21.10-mojmap.jar 8fee18251bd67a1564541dd810b900ee6525f77e469157e0bebddbfce3695b00
```

This is accepted as a narrow worldgen/density-map allocation reduction. It is
not an end-to-end TPS/MSPT or 500-player claim because the strict 50-bot gate did
not start on the busy host.

JFR hot methods:

```text
ImprovedNoise.p(int)                                           27.32%
ImprovedNoise.noise(double, double, double, double, double)    10.69%
PerlinNoise.getValue(...)                                       9.55%
Climate$RTree$SubTree.search(long[], Leaf)                      3.04%
Reference2ReferenceOpenHashMap.rehash(int)                      2.38%
NoiseChunk.updateForZ(int, double)                              2.03%
Aquifer$NoiseBasedAquifer.computeSubstance(...)                 1.95%
```

Allocation sites:

```text
PalettedContainer.reencodeContents(...)                        10.22%
NoiseChunk$FlatCache.<init>(...)                                9.59%
Iterators.forArrayWithPosition(Object[], int)                   7.74%
DensityFunctions$HolderHolder.mapAll(...)                       4.95%
Reference2ReferenceOpenHashMap.rehash(int)                      2.98%
LZ4BlockOutputStream.<init>(...)                                2.82%
NoiseChunk.wrapNew(DensityFunction)                             2.71%
```

GC signal:

```text
GC Count: 61
Total Pause Time: 9.32 s
Median Pause Time: 133 ms
Average Pause Time: 153 ms
P95 Pause Time: 598 ms
Maximum Pause Time: 713 ms
```

Config-only ZGC test:

```text
command label: 50bots-zgc-generational-gate-retry
preflight: PASS, load1=8.39, load_per_cpu=0.699, idle_percent_1s=64.30
flags: -Xms4G -Xmx10G -XX:+UseZGC -XX:+ZGenerational -XX:+DisableExplicitGC -XX:+AlwaysPreTouch
result: REJECTED
tps1_avg=15.71
avg_tick_ms_avg=203.15
loaded_chunks_max=1604
watchdog_thread_dumps=2
sync_load_stack_hits=0
```

Config-only fixed 10G G1 heap test:

```text
command label: 50bots-g1-xms10g-gate
preflight: BLOCKED before starting Minecraft, load1=10.15, load_per_cpu=0.846, idle_percent_1s=76.06
```

Decision: no JVM default changed. ZGC regressed the load gate, and the fixed
10G G1 test did not get a clean start.

## Plugin Loading Allocation Cleanup

Change:

```text
ModernPluginLoadingStrategy and LegacyPluginLoadingStrategy pre-size startup
maps/lists when the provider count is already known. Spigot and Paper provider
dependency validation paths allocate missing-dependency collections only after
the first actual miss. Load-order rules, plugin lifecycle, classloading,
scheduler, services, and event semantics are unchanged.
```

Microbenchmark command:

```bash
scripts/bench_plugin_loading_allocations.sh
```

Raw result from `reports/plugin-loading-allocation-bench.txt`:

```text
providers=256
hard_dependencies=4
warmup=4 rounds=6 iterations=20000
old_default_capacity_setup_best_ms=371.559
new_presized_setup_best_ms=233.823
setup_speedup=1.589x
old_eager_missing_set_best_ms=319.678
new_lazy_missing_set_best_ms=321.456
missing_set_speedup=0.994x
old_eager_validate_best_ms=248.706
new_lazy_validate_best_ms=232.648
validate_speedup=1.069x
```

Functional gates:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (29.708s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (19.566s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 16.276s/10.644s
50-bot 32/32 gate: BLOCKED before starting Minecraft, load1=15.76, load_per_cpu=1.313, idle_percent_1s=54.72
artifact hash at that gate: optimized-paper-1.21.10-mojmap.jar 9f7ef294c0bff716b5504553f04ced29fd471088b57d5250ba52a256aef4b9a2
latest artifact hash after the following Spigot load-after pre-size cycle: optimized-paper-1.21.10-mojmap.jar 203bac9244f5b0410a712cc6dadf7e88203403b9aee8812e2337abf4eac48640
```

This is accepted as plugin-loading setup/allocation reduction. It is not an
end-to-end cold-start or TPS claim. The legacy missing-set subpath is not
claimed as a speedup because the latest narrow timing was neutral (`0.994x`).

## Rejected VarInt/VarLong Branch-Expanded Write

Candidate:

```text
Replace the current Paper VarInt one/two-byte fast path and vanilla VarLong loop
with branch-expanded 1..5 byte VarInt and 1..10 byte VarLong writes using Netty
ByteBuf writeShort/writeMedium/writeInt.
```

Verification:

```bash
bash scripts/bench_varint_write.sh
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw microbenchmark result from `reports/varint-write-bench.txt`:

```text
cpu_model=Intel(R) Xeon(R) Gold 6252 CPU @ 2.10GHz
java=openjdk version "21.0.10" 2026-01-20
values=1000000 warmup=5 rounds=8
varint_old_best_ms=5.326
varint_new_best_ms=5.992
varint_speedup=0.889x
varlong_old_best_ms=6.844
varlong_new_best_ms=8.250
varlong_speedup=0.830x
```

Decision:

```text
REVERTED. The temporary artifact passed functional gates, but the direct changed
hot path was slower on this CPU. VarLong.java.patch was deleted and VarInt.java.patch
was restored to the prior Paper two-case fast path. The final production artifact
was rebuilt and revalidated after the revert.
```

Postrevert verification:

```text
build: PASS, applySourcePatches Applied 909 patches, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (56.876s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (32.770s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 22.788s/20.056s
strict load preflight: BLOCKED, load1=36.34, load_per_cpu=3.028, idle_percent_1s=8.52
```

## Plugin Remapper Hash Collection Capacity

Change:

```text
RemappedPluginIndex now sizes exact-SHA HashMap/HashSet instances using an
expected-capacity helper that accounts for Java load factor, avoiding resize
while building known-size plugin/library hash batches.
```

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-remapper-hash-capacity BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Raw result:

```text
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (39.019s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (38.650s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 29.097s/21.599s
strict load preflight: BLOCKED, load1=39.62, load_per_cpu=3.302, idle_percent_1s=19.75
```

## Plugin Remapper Batch List Capacity

Change:

```text
PluginRemapper and RemappedPluginIndex now construct known-size task/result
ArrayLists with explicit capacity in plugin-directory, extra-plugin, and Paper
plugin-library remap/cache paths.
```

This preserves result order, exact SHA cache keys, skip/remap decisions, plugin
lifecycle, scheduler semantics, and classloading. The accepted effect is small
startup allocation/copy reduction, not an end-to-end speed claim.

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-remapper-capacity BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Raw result:

```text
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (44.904s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (31.857s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 16.734s/10.518s
strict load preflight: BLOCKED, load1=16.56, load_per_cpu=1.380, idle_percent_1s=69.43
```

## PaperReflection Stripped-Method Map Removal

Change:

```text
PaperReflection no longer builds a duplicate Map<className, strippedMethods>
from ObfHelper mappings during construction. Method reflection remap lookups now
read strippedMethods directly from the existing ObfHelper.ClassMapping,
recursive superclass/interface method lookup reuses one stripped method key, and
empty-parameter descriptors return `()` without allocating a StringBuilder.
```

This preserves plugin reflection behavior: class, method, and field mapping still
use the same mappings and inheritance lookup. The accepted effect is startup
work/memory reduction in the plugin reflection bridge, not a measured end-to-end
startup-speed claim.

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-rebuildpatches-paperreflection BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Raw result:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (40.233s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (21.118s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 17.024s/11.387s
strict load preflight: BLOCKED, load1=14.53, load_per_cpu=1.211, idle_percent_1s=71.39
```

## Plugin Remapper Deferred Mappings Load

Change:

```text
PluginRemapper no longer starts reversedMappingsFuture() in the callers before
manifest inspection. It now starts mappings/reversed-mappings only after a jar
fails the no-remap skip checks and is actually going to ART remap.
```

This targets first-run plugin/library startup for Paper plugins and plugin
libraries that have no namespace and therefore do not need remapping. It does
not change plugin classloading, lifecycle, scheduler, services, or event order.

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw result:

```text
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (32.836s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (26.519s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 21.304s/17.690s
```

Targeted skip-only debug run:

```text
log=/root/rust/logs/remapper-skip-only.log
LibraryProbe plugin: Paper plugin with no namespace specified
library-probe-dep.jar: no mappings namespace, not remapping
LIBRARY_PROBE dependency=loaded-from-plugin-library
mapping_load=not_started_for_skip_only
```

This is accepted as startup-work reduction for skip-only remap misses. It is not
an end-to-end startup-speed claim because the host was busy during the matrix
run.

## Plugin Library Remap Cache

Change:

```text
PluginRemapper.remapLibraries(...) now hashes jar libraries as a batch and passes the already computed SHA-256 through getIfPresent/input/skip.
Precomputed library skip decisions are stored under plugin-remaps/<mappingsHash>/libraries/skipped-hashes.txt so library and plugin remap semantics stay separate.
```

Verification:

```bash
./scripts/build_library_probe_plugin.sh
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw result:

```text
build: BUILD SUCCESSFUL
precomputed_plugin_remaps=4
precomputed_plugin_skips=8
precomputed_library_remaps=0
precomputed_library_skips=1
plugin matrix: Done (45.204s)
LibraryProbe: LIBRARY_PROBE dependency=loaded-from-plugin-library
fresh libraries index: hashes=0, skippedHashes=1
restart/recovery: Done (22.182s), Saved the game
forced_ticket_persistence=PASS
```

This is accepted as classpath/remap work reduction and coverage improvement. It is not a claimed end-to-end startup speedup because the host was busy and the matrix is dominated by real plugin initialization/config/update work.

## Plugin Remapper Batch-Miss Hash Reuse

Change:

```text
For normal plugin-directory and extra-plugin batches, PluginRemapper now passes the already computed SHA-256 into the cache-miss remap/skip path.
This avoids a second full jar read when `index.input(...)` and `index.skip(...)` mark a plugin after the batch hash cache has already read it.
```

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw result:

```text
applySourcePatches: Applied 909 patches
createMojmapBundlerJar: BUILD SUCCESSFUL
precomputed_plugin_remaps=4
precomputed_plugin_skips=7
app-cds.jsa regenerated
plugin matrix: Done (32.998s)
status protocol=773
fresh .paper-remapped/index.json: hashes=4, skippedHashes=7
```

This is accepted as startup-work reduction in the plugin remapper code path. A clean end-to-end startup speedup is not claimed because the host was busy during the matrix run.

## Streaming InputStream SHA-256

Change:

```text
Hashing.sha256(InputStream) now feeds the stream into a Guava Hasher incrementally instead of first materializing the whole stream through IOUtils.toByteArray(...).
```

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw result:

```text
build: BUILD SUCCESSFUL
plugin matrix: Done (32.998s), protocol=773, CompatProbe join/events/scheduler/command passed
restart/recovery: Done (18.470s), COMPAT_PROBE command=ok events=2 ownServices=0, Saved the game
fresh .paper-remapped/index.json: hashes=4, skippedHashes=7
```

This is memory-copy reduction, not a measured startup speedup claim.

## Precompute Harness Cache-Hit Repair

Command:

```bash
./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw result:

```text
rebuildPatches: BUILD SUCCESSFUL, Rebuilt 909 patches, Saved modified patches (33/36)
first build attempt: Gradle jar build passed, then failed with "Missing reversed mappings precompute output"
fixed scripts/precompute_plugin_remaps.sh fallback for already-installed reversed mappings cache
second build attempt: BUILD SUCCESSFUL, precomputed_plugin_remaps=4, app-cds.jsa regenerated
plugin matrix: Done (52.172s), command/events/scheduler/join passed
```

This is a build/reproducibility fix, not a server-speed claim. A clean load benchmark was not run immediately after it because the host was already under heavy live load: `uptime` reported load average `18.67` on 12 CPUs and `mpstat` averaged only `24.17%` idle.

## Precomputed Plugin Skip Cache

Change:

```text
RemappedPluginIndex can now consume artifacts/optimized-runtime/plugin-remaps/<mappingsHash>/skipped-hashes.txt.
The file contains exact SHA-256 hashes for plugin jars that the real remapper already proved do not need remapping.
```

Verification:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Raw result:

```text
precomputed_plugin_remaps=4
precomputed_plugin_skips=8
skip-enabled plugin matrix: Done (30.330s), then Done (31.116s), then Done (32.401s)
control without skipped-hashes.txt: Done (29.630s)
fresh skip-enabled .paper-remapped/index.json: hashes=4, skippedHashes=8
```

The skip cache is accepted only as exact-SHA startup work reduction and compatibility-safe cache infrastructure. The noisy A/B did not prove an end-to-end startup speedup, so no performance claim is made from these numbers.

## Load Host Preflight

`scripts/run_load_test.sh` now refuses to start a benchmark on a host that is already too busy, unless `LOAD_TEST_ALLOW_BUSY_HOST=true` is explicitly set.

Smoke command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-current-host BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Raw result:

```text
exit=75
host_preflight_ok=false
cpu_count=12
load1=30.42
load_per_cpu=2.535
idle_percent_1s=7.91
min_idle_percent=40.00
max_load_per_cpu=0.750
```

No Minecraft server was started in this check. The latest report is `reports/load-preflight-holder-unwrapping-preflight.txt`.

## Reversed Mappings Cache A/B

Command with cache enabled:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Result:

```text
Done (34.734s)
PlayerJoinEvent sequence=3 detail=CodexJoinProbe
COMPAT_PROBE command=ok events=4 ownServices=0
```

Control command temporarily moved `artifacts/optimized-runtime/reversed-mappings/*.tiny` away, ran the same plugin matrix command, then restored the file via shell trap.

Control result:

```text
Done (34.950s)
PlayerJoinEvent sequence=3 detail=CodexJoinProbe
COMPAT_PROBE command=ok events=4 ownServices=0
```

The cache is functional and avoids creating `plugins/.paper-remapped/mappings/reversed/*.tiny` during the enabled run, but the measured delta was only `216 ms` / `0.6%`; this is not claimed as a real startup speedup.

## Latest 50-Bot Reversed-Cache Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-reversed-cache-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
online_max=50
loaded_chunks_max=1059
tps1_min=13.34
tps1_avg=17.53
avg_tick_ms_max=296.20
avg_tick_ms_avg=63.15
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

This is a functional pass, not a new performance baseline.

## Rejected FindTopSurface ThreadLocal Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-findtopsurface-threadlocal LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
online_max=50
loaded_chunks_max=2449
tps1_min=3.45
tps1_avg=17.67
avg_tick_ms_max=304.97
avg_tick_ms_avg=59.76
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate compiled and passed plugin matrix on the temporary build (`Done (29.155s)`), but it did not beat the accepted `18.27/47.85/2380` load baseline. It was reverted, `applyPatches` returned to 909 patches, and the artifact was rebuilt.

## Rejected Preliminary Surface Quart-Mask Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-prelim-quart-mask LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
online_max=50
loaded_chunks_max=2280
tps1_min=2.93
tps1_avg=15.83
avg_tick_ms_max=839.70
avg_tick_ms_avg=108.32
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate compiled and passed plugin matrix on the temporary build (`Done (35.023s)`), but it regressed far below the accepted `18.27/47.85/2380` load baseline. It was reverted, `applyPatches` returned to 909 patches, the artifact was rebuilt, and the postrevert plugin matrix passed at `Done (40.611s)`.

## Current JFR Profile Before Perlin Active-Octaves Attempt

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-current-jfr3 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260507-current-jfr3.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Raw summary:

```text
loaded_chunks_max=1233
tps1_avg=17.23
avg_tick_ms_avg=50.40
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Top JFR Java samples:

```text
ImprovedNoise.p(int)                                             42.33%
Climate$RTree$SubTree.search(long[], Leaf)                         3.24%
Reference2ReferenceOpenHashMap.rehash(int)                         2.18%
Aquifer$NoiseBasedAquifer.computeSubstance(...)                    2.04%
ImprovedNoise.noise(...)                                           1.90%
NoiseChunk$NoiseInterpolator.compute(...)                          1.66%
PerlinNoise.getValue(double, double, double)                       1.51%
```

This profile was used only to choose the next candidate. It is not a new accepted load baseline because `17.23/50.40/1233` is still worse than the accepted `18.27/47.85/2380`.

## Rejected Perlin Active-Octaves Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-perlin-active-octaves LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
loaded_chunks_max=1126
tps1_min=9.21
tps1_avg=16.76
avg_tick_ms_max=1164.37
avg_tick_ms_avg=138.50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate compiled and passed plugin matrix on the temporary build (`Done (40.923s)`), but it regressed badly versus the accepted `18.27/47.85/2380` load baseline and hit a watchdog thread dump. It was reverted, `applyPatches` returned to 909 patches, the artifact was rebuilt, and the postrevert plugin matrix passed at `Done (42.581s)`.

## Rejected NoiseChunk Wrap Load-Factor Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-wrap-loadfactor095 LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
loaded_chunks_max=1020
tps1_min=13.82
tps1_avg=16.85
avg_tick_ms_max=394.81
avg_tick_ms_avg=74.43
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate compiled and passed plugin matrix on the temporary build (`Done (40.881s)`), but it regressed versus the accepted `18.27/47.85/2380` load baseline and hit a watchdog thread dump. It was reverted, `applyPatches` returned to 909 patches, the artifact was rebuilt, and the postrevert plugin matrix passed at `Done (32.291s)`.

## Rejected Lazy Blend Cache Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-lazy-blend-cache LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
online_max=34
loaded_chunks_max=562
tps1_min=12.40
tps1_avg=16.02
avg_tick_ms_max=174.44
avg_tick_ms_avg=65.09
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate compiled and passed plugin matrix on the temporary build (`Done (48.547s)`), but it regressed versus the accepted `18.27/47.85/2380` load baseline and only reached `online_max=34`. It was reverted, `applyPatches` returned to 909 patches, the artifact was rebuilt, and the postrevert plugin matrix passed at `Done (41.903s)`.

## Rejected Climate Sampler SampleState Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-climate-samplestate-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
online_max=50
loaded_chunks_max=1993
tps1_min=2.86
tps1_avg=16.91
avg_tick_ms_max=632.92
avg_tick_ms_avg=96.16
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate combined `Climate.Sampler`'s sampled-value array and mutable sample context into one `ThreadLocal` state object. It compiled and passed plugin matrix on the temporary build (`Done (32.711s)`), but the 50-bot 32/32 gate regressed versus the accepted `18.27/47.85/2380` load baseline. It was reverted, `applyPatches` returned to 909 patches, the artifact was rebuilt, and the postrevert plugin matrix passed at `Done (30.549s)`.

## Rejected Chunk I/O Threads 2 Config Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-io2-config-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 PAPER_CHUNK_IO_THREADS=2 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=2 prevent_unloaded_move=false
worker_line=[11:06:02 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
online_max=50
loaded_chunks_max=861
tps1_min=11.43
tps1_avg=16.96
avg_tick_ms_max=290.35
avg_tick_ms_avg=74.18
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

This was a config-only tuning probe, not a source patch. It regressed the accepted `18.27/47.85/2380` load baseline and hit a watchdog thread dump, so no default I/O-thread change was made.

## Rejected ImprovedNoise GradDot Inline Gate

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-20260507-improvednoise-graddot-inline LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
online_max=50
loaded_chunks_max=2312
tps1_min=3.11
tps1_avg=17.37
avg_tick_ms_max=879.66
avg_tick_ms_avg=103.93
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
```

The candidate inlined `SimplexNoise.dot(...)` inside `ImprovedNoise.gradDot` while keeping the same gradient index and arithmetic. It compiled and passed plugin matrix on the temporary build (`Done (30.132s)`), but the 50-bot 32/32 gate regressed versus the accepted `18.27/47.85/2380` load baseline. It was reverted, `applyPatches` returned to 909 patches, the artifact was rebuilt, and the postrevert plugin matrix passed at `Done (33.371s)`.

## Spawn-Fitness Load Gate

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-spawn-fitness-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[13:06:37 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=15
online_max=50
loaded_chunks_max=5255
tps1_min=3.57
tps1_avg=17.78
avg_tick_ms_max=263.57
avg_tick_ms_avg=70.20
used_mem_mib_max=9697
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=849.00
process_rss_mib_max=11754.1
moved_too_quickly_warnings=2
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This gate kept all 50 bots connected/ready/active and had no watchdog or sync-load stack hits, but it is not a new load baseline: `tps1_avg=17.78` and `avg_tick_ms_avg=70.20` are worse than the accepted completed-delta rerun (`18.27` / `47.85`) under a much higher `loaded_chunks_max=5255`.

## Rejected Cell-Fraction Lookup Load Attempt

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-cellfrac LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[14:48:28 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=15
online_max=50
loaded_chunks_max=4692
tps1_min=3.39
tps1_avg=17.47
avg_tick_ms_max=412.23
avg_tick_ms_avg=82.04
used_mem_mib_max=9272
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=838.00
process_rss_mib_max=11404.2
moved_too_quickly_warnings=2
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This experiment compiled and passed plugin matrix, but it regressed the accepted load baseline (`18.27` / `47.85`) to `17.47` / `82.04`, so it was reverted.

## Rejected Aquifer Air Cache Attempt

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-aquifer-air LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[15:10:58 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=15
online_max=50
loaded_chunks_max=4644
tps1_min=3.33
tps1_avg=17.94
avg_tick_ms_max=444.08
avg_tick_ms_avg=81.51
used_mem_mib_max=8728
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=789.00
process_rss_mib_max=11670.3
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This experiment compiled and passed plugin matrix at `Done (24.876s)` on the temporary build, but it regressed the accepted 50-bot load baseline to `17.94` / `81.51`, so it was reverted.

## Final Reverted Load Rerun

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-final-reverted LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[15:22:52 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=14
online_max=50
loaded_chunks_max=3567
tps1_min=13.10
tps1_avg=18.52
avg_tick_ms_max=273.36
avg_tick_ms_avg=61.43
used_mem_mib_max=9255
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=798.00
process_rss_mib_max=11475.6
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This is the final current-source load rerun on the reverted artifact. TPS and loaded chunks improved versus the accepted baseline, but average tick time worsened, so it was not promoted to a new baseline.

## Non-Flush Packet Send Experiment

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260506-lazyexecute-on LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 PAPER_OPTIMIZE_NON_FLUSH_PACKET_SENDING=true JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[10:04:17 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=11
online_max=50
loaded_chunks_max=2468
tps1_min=11.46
tps1_avg=16.31
avg_tick_ms_max=285.57
avg_tick_ms_avg=80.82
used_mem_mib_max=8063
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=712.00
process_rss_mib_max=11861.1
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This experiment kept the flag default-off in production, but the on-state regressed the real plugin load versus the accepted baseline (`tps1_avg=18.27`, `avg_tick_ms_avg=47.85`), so it is rejected for now.

## Completed NoiseInterpolator Delta Rerun

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260505-noiseinterp-delta-complete LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[14:46:50 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=13
online_max=50
loaded_chunks_max=2380
tps1_min=16.37
tps1_avg=18.27
avg_tick_ms_max=74.07
avg_tick_ms_avg=47.85
used_mem_mib_max=7581
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=788.00
process_rss_mib_max=11627.7
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This remains the latest accepted short-load measurement. It is a latency win versus the shared-warning-gate rerun (`avg_tick_ms_avg=70.81` -> `47.85`) and a TPS improvement (`17.95` -> `18.27`) under a lower loaded-chunk count (`3233` -> `2380`). It still does not hold 20 TPS.

## Shared Warning Gate Load Rerun

Command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=load-50bots-20260505-move-log-shared-rate-limit LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[12:42:20 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=15
online_max=50
loaded_chunks_max=3233
tps1_min=3.64
tps1_avg=17.95
avg_tick_ms_max=466.85
avg_tick_ms_avg=70.81
used_mem_mib_max=7774
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=796.00
process_rss_mib_max=11811.1
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This run remains useful as the previous same-flag comparison point. The later completed-delta rerun is the current latest short-load measurement.

## 50-Bot Load Benchmark

Contaminated rerun command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260505-noiseinterp-delta LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260505-noiseinterp-delta.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[09:54:33 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=14
online_max=50
loaded_chunks_max=2726
tps1_min=12.87
tps1_avg=18.07
avg_tick_ms_max=677.51
avg_tick_ms_avg=92.83
used_mem_mib_max=6839
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=756.00
process_rss_mib_max=11979.5
moved_too_quickly_warnings=972
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

This rerun is not a clean regression gate because the host was already under heavy unrelated CPU load when it started.

Postrevert comparison baseline command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-postrevert-rebaseline LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-postrevert-rebaseline.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Raw summary:

```text
bots=50 view_distance=32 simulation_distance=32 bot_exit=0
load_test_gamemode=spectator
paper_chunk_config=max_loads=auto max_gens=auto load_rate=default gen_rate=default send_rate=default workers=12 io=auto prevent_unloaded_move=false
worker_line=[18:39:09 INFO]: [MoonriseCommon] Paper is using 12 worker threads, 2 I/O threads
metrics_samples=12
online_max=50
loaded_chunks_max=2472
tps1_min=16.02
tps1_avg=17.49
avg_tick_ms_max=112.01
avg_tick_ms_avg=58.12
used_mem_mib_max=6797
bot_created_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
process_cpu_max=757.00
process_rss_mib_max=13677.1
moved_too_quickly_warnings=1081
watchdog_thread_dumps=0
sync_load_stack_hits=0
nearby_players_stack_hits=0
external_thread_prints=0
```

Current `NoiseChunk.NoiseInterpolator` completed delta-cache patch:

```text
change: NoiseInterpolator caches Y/X/Z deltas and uses direct arithmetic in updateForY/updateForX/updateForZ
current latest rerun: tps1_avg=18.27, avg_tick_ms_avg=47.85, loaded_chunks_max=2380, moved_too_quickly_warnings=1, watchdog_thread_dumps=0, sync_load_stack_hits=0
```

Current `PerlinNoise` primitive amplitude-cache patch:

```text
change: PerlinNoise copies amplitudes into primitive double[] and removes stream/nonNull constructor validation
current postrevert rebaseline: tps1_avg=17.49, avg_tick_ms_avg=58.12, loaded_chunks_max=2472, watchdog_thread_dumps=0, sync_load_stack_hits=0
```

Current `Climate.RTree` / `Climate.Sampler` fast-path command:

```bash
MC_EULA_AGREE=true LOAD_TEST_LABEL=50bots-20260504-climate-fastpath-rerun LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx24G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=100 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260504-climate-fastpath-rerun.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Repeated result:

```text
first climate fast-path run: tps1_avg=18.32, tps1_min=13.54, avg_tick_ms_avg=91.97, loaded_chunks_max=2991, moved_too_quickly_warnings=1009, watchdog_thread_dumps=0, sync_load_stack_hits=0
rerun: tps1_avg=18.38, tps1_min=13.76, avg_tick_ms_avg=99.65, loaded_chunks_max=2986, moved_too_quickly_warnings=994, watchdog_thread_dumps=0, sync_load_stack_hits=0
```

The climate fast-path is throughput-positive versus the postrevert rebaseline (`17.49` TPS avg and `2472` loaded chunks), but it is not a clean latency win because average tick time rose. The run still failed the target because it did not hold 20 TPS. The earlier restored run at `18.78 / 60.83 / 2917` remains historical evidence, not the current baseline.

Rejected `Climate.RTree.Node` cached parameter fields experiment:

```text
command label: 50bots-20260507-climate-node-fields-gate
change: cached parameterSpace[0..6] into final parameter0..parameter6 fields inside Climate.RTree.Node
patch gate: applyPatches applied 909 patches after hunk-count fix
build gate: MC_EULA_AGREE=true ./scripts/build_optimized.sh, BUILD SUCCESSFUL
plugin matrix with experiment: Done (34.451s)
load-run after experiment: tps1_avg=17.39
avg_tick_ms_avg=47.48
loaded_chunks_max=1236
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
raw summary: reports/load-50bots-20260507-climate-node-fields-gate-summary.txt
watchdog stack: save-all -> SerializableChunkData.copyOf -> PalettedContainer.copy
postrevert plugin matrix: Done (29.901s), CompatProbe command/events/scheduler passed
```

The candidate did not beat the accepted `NoiseChunk.NoiseInterpolator` baseline (`tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, no watchdog). It was reverted to the prior unrolled `parameterSpace[0..6]` distance path, and the runtime artifact was rebuilt after the revert.

Rejected `CubicSpline.Multipoint.mapAll` stream/iterator cleanup experiment:

```text
command label: 50bots-20260507-cubicspline-mapall-gate
change: replaced mapAll stream().map(...).toList() and one for-each pass with indexed loops / ImmutableList builder
patch gate: applyPatches applied 910 patches while candidate was present
build gate: MC_EULA_AGREE=true ./scripts/build_optimized.sh, BUILD SUCCESSFUL
plugin matrix with experiment: Done (29.676s)
load-run after experiment: tps1_avg=17.45
avg_tick_ms_avg=126.93
loaded_chunks_max=968
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
raw summary: reports/load-50bots-20260507-cubicspline-mapall-gate-summary.txt
watchdog stack: save-all -> SerializableChunkData.copyOf -> SWMRNibbleArray.getSaveState
postrevert plugin matrix: Done (30.645s), CompatProbe command/events/scheduler passed
```

Even though the source change was semantically small, the end-to-end gate was much worse than the accepted baseline and hit watchdog. The patch was deleted, `applyPatches` returned to 909 source patches, and the runtime artifact was rebuilt after the revert.

Rejected `BlendedNoise.compute` power-of-two scale experiment:

```text
command label: 50bots-20260507-blendednoise-scale-gate
change: replaced `/ d11` where d11 is a power of two with an explicit doubling multiplier
patch gate: applyPatches applied 910 patches while candidate was present
build gate: MC_EULA_AGREE=true ./scripts/build_optimized.sh, BUILD SUCCESSFUL
plugin matrix with experiment: Done (30.199s)
load-run after experiment: tps1_avg=17.50
avg_tick_ms_avg=90.04
loaded_chunks_max=2376
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
raw summary: reports/load-50bots-20260507-blendednoise-scale-gate-summary.txt
watchdog stack: save-all -> SerializableChunkData.copyOf -> LevelChunkSection.copy
postrevert plugin matrix: Done (29.989s), CompatProbe command/events/scheduler passed
```

The candidate preserved the worldgen-only scope but did not beat the accepted `18.27/47.85` baseline and hit watchdog during the save path. The source patch was deleted, `applyPatches` returned to 909 source patches, and the runtime artifact was rebuilt after the revert.

Rejected `NoiseChunk.forIndex` fast-div experiment:

```text
command label: 50bots-20260505-noisechunk-forindex-fastdiv
change: replaced floorMod/floorDiv in NoiseChunk.forIndex with direct / and %
plugin matrix with experiment: Done (30.669s)
load-run after experiment: tps1_avg=16.93
avg_tick_ms_avg=173.54
loaded_chunks_max=2182
watchdog_thread_dumps=0
sync_load_stack_hits=0
control rerun after revert: tps1_avg=16.73, avg_tick_ms_avg=56.58, loaded_chunks_max=805, watchdog_thread_dumps=1
```

The bytecode after revert uses `Math.floorMod`/`Math.floorDiv` again. The candidate was not accepted.

Rejected `LinearPalette` reference-map cache experiment:

```text
command label: 50bots-20260504-linearpalette-jfr
change: LinearPalette.idFor cached exact identities in Reference2IntOpenHashMap
plugin matrix with experiment: Done (50.056s)
load-run after experiment: tps1_avg=14.53
avg_tick_ms_avg=82.04
loaded_chunks_max=2760
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The map lookup removed the linear scan but regressed the end-to-end 50-bot run, so it was deleted.

Rejected `PrepareSpawnTask` playerdata cache experiment:

```text
command label: 50bots-20260504-playerdata-cache-jfr
change: cached loaded playerdata during spawn preparation to avoid a second load
load-run result: tps1_avg=16.98
avg_tick_ms_avg=96.49
loaded_chunks_max=3487
tps1_min=2.65
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

This regressed the restored baseline, so the patch was removed.

Rejected `Reference2ReferenceOpenHashMap` 4096 map-size experiment:

```text
4096: tps1_avg=16.09, avg_tick_ms_avg=70.68, loaded_chunks_max=1866, watchdog_thread_dumps=0
2048 restored baseline: tps1_avg=17.49, avg_tick_ms_avg=40.93, loaded_chunks_max=1205, watchdog_thread_dumps=0
```

Rejected `ImprovedNoise` permutation-table experiment:

```text
command label: 50bots-intperm-jfr
change: ImprovedNoise byte[] permutation table changed to int[]
plugin matrix after experiment: Done (40.576s)
load-run startup after experiment: Done (49.454s)
tps1_avg=15.35
avg_tick_ms_avg=53.84
loaded_chunks_max=533
moved_too_quickly_warnings=953
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

This was worse than the `50bots-refmap-jfr` baseline for load throughput and tick time, so the source was reverted to the vanilla-style `byte[]` permutation table.

Rejected `ImprovedNoise` masking-reduction experiment:

```text
command label: 50bots-shortperm-jfr
change: ImprovedNoise short[] table with duplicated tail and direct nested indexing
plugin matrix after experiment: Done (41.509s)
load-run startup after experiment: not captured separately from the load summary
tps1_avg=16.40
avg_tick_ms_avg=75.99
loaded_chunks_max=1348
moved_too_quickly_warnings=846
watchdog_thread_dumps=1
sync_load_stack_hits=1
```

This was worse than baseline on the only metrics that matter, so it was reverted.

Rejected `Mth.lerp2/lerp3` inline arithmetic experiment:

```text
command label: 50bots-20260507-mth-lerp-inline
change: inline `Mth.lerp2` and `Mth.lerp3` arithmetic instead of nested `Mth.lerp` calls
temporary build: applySourcePatches applied 910 patches, optimized runtime built successfully
plugin matrix with experiment: Done (29.892s)
load-run result: tps1_avg=18.02
avg_tick_ms_avg=43.93
loaded_chunks_max=1625
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
postrevert plugin matrix: Done (30.460s)
```

Although average tick time was lower in this run, TPS and loaded-chunk coverage were both below the accepted `18.27/47.85/2380` baseline. The source patch was deleted and the runtime was rebuilt back to `909 patches`.

Rejected `SurfaceRules.SequenceRule` indexed iteration experiment:

```text
command label: 50bots-20260507-surfacerules-sequence-index
change: replace foreach over `SequenceRule.rules` with indexed `List.get(i)` iteration
temporary build: optimized runtime built successfully
plugin matrix with experiment: Done (31.894s)
load-run result: tps1_avg=18.79
avg_tick_ms_avg=38.68
loaded_chunks_max=1216
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
postrevert plugin matrix: Done (44.811s)
```

The change targeted JFR `Iterators$ArrayItr` allocation in surface generation and preserved rule order, but it failed the gate because chunk coverage dropped far below the accepted baseline and a watchdog thread dump was observed. The source hunk was deleted and the runtime was rebuilt.

Rejected `PalettedContainer.reencodeContents` zero-storage branch experiment:

```text
command label: 50bots-20260507-paletted-zero-reencode
change: special-case `ZeroBitStorage` in `PalettedContainer.reencodeContents` to avoid unpacking/filling and remapping every zero palette entry during chunk serialization
temporary build: optimized runtime built successfully
plugin matrix with experiment: Done (47.375s)
load-run result: tps1_avg=16.32
avg_tick_ms_avg=112.44
avg_tick_ms_max=934.52
loaded_chunks_max=1430
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=1
postrevert plugin matrix: Done (36.608s)
```

The change targeted the JFR `Arrays.fill -> ZeroBitStorage.unpack -> PalettedContainer.reencodeContents -> SerializableChunkData.write` stack seen during chunk save serialization. It preserved the intended palette-id result, but the end-to-end gate regressed badly and hit watchdog/sync-load, so the feature patch was deleted and the runtime was rebuilt back to `909 patches`.

Rejected spectator movement no-sync-load experiment:

```text
command label: 50bots-20260507-spectator-nosyncload
change: for spectator players with no registered `PlayerMoveEvent` listeners, use an internal position snap that does not force `Level.getChunk(...)` from `Entity.absSnapTo`
temporary build: optimized runtime built successfully
plugin matrix with experiment: Done (42.489s)
load-run result: tps1_avg=17.16
avg_tick_ms_avg=50.81
avg_tick_ms_max=119.03
loaded_chunks_max=1266
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
postrevert plugin matrix: Done (31.605s)
```

The change removed the exact `ServerGamePacketListenerImpl.handleMovePlayer -> Entity.absSnapTo -> Level.getChunk -> ServerChunkCache.syncLoad` stack seen in the previous rejected run, but the overall load gate still regressed against the accepted `18.27/47.85/2380` baseline. Because it did not improve TPS/chunk coverage, the feature patch was deleted and the runtime was rebuilt back to `909 patches`.

Rejected unlimited chunk load/send/gen rates experiment:

```text
command label: 50bots-20260507-rates-unlimited
change: config-only `PAPER_PLAYER_MAX_LOAD_RATE=-1`, `PAPER_PLAYER_MAX_SEND_RATE=-1`, `PAPER_PLAYER_MAX_GEN_RATE=-1`
load-run result: tps1_avg=17.16
avg_tick_ms_avg=42.69
avg_tick_ms_max=95.22
loaded_chunks_max=1565
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Removing the per-player rates reduced average tick time in this run, but did not improve TPS or loaded-chunk coverage enough to beat the accepted `18.27/47.85/2380` baseline. No config default was changed.

## JFR Hot Set

Current noiseinterp-delta rerun JFR top methods:

```text
ImprovedNoise.p                                           27.25%
PerlinNoise.getValue                                       8.45%
ImprovedNoise.noise                                        7.43%
NoiseChunk$NoiseInterpolator.updateForZ                    3.55%
Climate$RTree$SubTree.search(long[], Leaf)                 2.60%
Reference2ReferenceOpenHashMap.rehash                      2.25%
Aquifer$NoiseBasedAquifer.computeSubstance                 1.79%
NoiseChunk$NoiseInterpolator.compute                       1.75%
NoiseChunk$NoiseInterpolator.updateForX                    1.38%
NoiseChunk$FlatCache.compute                               1.30%
NoiseChunk.lambda$new$0                                    1.27%
LinearPalette.idFor                                        1.15%
NoiseChunk$NoiseInterpolator.updateForY                    0.80%
PalettedContainer.reencodeContents                         0.54%
```

The old Java `IdentityHashMap.resize` hot path inside `NoiseChunk.wrap` is gone, but fastutil rehash remains. Increasing expected size to `4096` made load and plugin startup worse, and the LinearPalette map-cache experiment also regressed end-to-end load, so those paths are currently left at their measured baselines. The new `NoiseInterpolator` delta arithmetic is a semantic no-op over `Mth.lerp`, but the contaminated rerun did not yet prove a clean end-to-end win.

The `Climate` fast-path changed the visible biome-search method from the generic distance-metric path to `search(long[], Leaf)`, but biome lookup is still visible and fresh chunk noise generation remains dominant. The project keeps the end-to-end benchmark as the decision gate, not isolated method percentage.

## Region Compression Microbenchmark

Command:

```bash
./scripts/bench_region_compression.sh
```

Raw result:

```csv
codec,chunks,chunk_bytes,best_ms,throughput_mib_s,total_compressed_bytes,ratio
zlib,768,98304,5412.321,13.30,57823555,0.7659
gzip,768,98304,5330.875,13.51,57832771,0.7660
lz4,768,98304,472.615,152.34,74568143,0.9877
```

LZ4 was `~11.5x` faster than ZLIB in this CPU microbenchmark, with larger output in this synthetic workload.

## Plugin Remapper SHA Cache Reuse Microbenchmark

Change:

```text
RemappedPluginIndex now computes plugin jar SHA-256 hashes once for a rewrite batch,
reuses those hashes when the all-cached fast path misses, and hashes batches of
4+ jars in parallel. The cache key remains exact sha256(plugin.jar); no mtime/size
trust shortcut is used.
```

Build and functional gate:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Result:

```text
build: BUILD SUCCESSFUL, applySourcePatches Applied 909 patches
plugin matrix: Done (33.147s)
CompatProbe: scheduler sync/async ticked, PlayerJoinEvent sequence=3, command=ok events=4
precomputed plugin remaps installed for spark, Vault, CompatProbe, PlaceholderAPI
```

Narrow hash-path microbenchmark over the current real plugin matrix jars:

```text
files=11
total_size=36.25 MiB
old_seq_two_passes best_ms=182.522 median_ms=189.974
reuse_seq_one_pass best_ms=89.230 median_ms=92.430
reuse_parallel_one_pass best_ms=25.707 median_ms=30.038
```

This is accepted only as a remapper hash-path improvement plus plugin-matrix
compatibility evidence. It is not claimed as an end-to-end server startup win:
the host was concurrently running a heavy live `/var/lib/pufferpanel/servers/6805cd25`
Java workload during this cycle, so clean startup A/B is still required.

## Noisy Diagnostic Load Run: 2026-05-07

This run was intentionally allowed on a saturated host after the default
preflight correctly rejected the machine as too busy. It is diagnostic only and
must not replace the accepted clean 50-bot baseline.

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_ALLOW_BUSY_HOST=true \
LOAD_TEST_LABEL=50bots-20260507-noisy-current-final \
LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 \
VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 \
PAPER_PLAYER_MAX_CONCURRENT_LOADS=-1 PAPER_PLAYER_MAX_CONCURRENT_GENS=-1 \
JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-20260507-noisy-current-final.jfr,settings=profile,dumponexit=true' \
./scripts/run_load_test.sh
```

Raw summary:

```text
NOISY / non-comparable
bots=50
online_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
server_join_events=50
server_quit_events=50
loaded_chunks_max=139
tps1_min=16.93
tps1_avg=17.84
avg_tick_ms_max=65.67
avg_tick_ms_avg=38.70
process_cpu_max=358.00
process_rss_mib_max=9380.2
moved_too_quickly_warnings=1
watchdog_thread_dumps=1
sync_load_stack_hits=0
```

Host noise during the run was severe: another Java process was consuming about
`481%` CPU and system load climbed to roughly `45`. The run still proved the
current artifact can bring up all 50 spectator bots and move them without kicks,
but the low `loaded_chunks_max=139` means it did not exercise the target chunk
coverage cleanly.

JFR from `reports/load-50bots-20260507-noisy-current-final.jfr` kept the same
generation-heavy shape:

```text
Mth.lerp3(...)                                  26.59%
ImprovedNoise.noise(...)                         7.42%
PerlinNoise.getValue(...)                        6.45%
PerlinNoise.wrap(double)                         6.00%
Climate$RTree$SubTree.search(long[], Leaf)       3.52%
```

The watchdog dump happened during `save-all`, not during the gameplay movement
loop. The main thread was inside
`TicketStorage.redirectRegularTickets -> forEachTicket -> packTickets ->
DimensionDataStorage.encodeUnchecked -> scheduleSave -> ServerLevel.save ->
MinecraftServer.saveAllChunks -> SaveAllCommand.saveAll`. That makes ticket
packing during save the next narrow optimization candidate, while preserving
forced/portal ticket persistence.

## Persistent Ticket Save Packing

Change:

```text
Add Moonrise ticket counter type COUNTER_TYPE_PERSISTENT.
Register it for every TicketType where TicketType.persist() is true.
Use that counter in TicketStorage.packTickets() so save serialization only
copies tickets from chunks that may contain persistent tickets.
Keep the final TicketType.persist() filter in packTickets().
```

This targets the noisy-run watchdog stack in `save-all` where
`TicketStorage.redirectRegularTickets()` copied all regular player/chunk tickets
before filtering down to forced/portal persistent tickets.

Functional gates:

```bash
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
sha256sum -c reports/artifact-hashes.txt
```

Results:

```text
build: PASS, applySourcePatches Applied 909 patches, createMojmapBundlerJar BUILD SUCCESSFUL
plugin matrix: PASS, Done (31.880s), PlayerJoinEvent sequence=3, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (18.614s), Saved the game, clean disable
forced ticket persistence: PASS, restart query reported chunk [0, 0] is marked for force loading
artifact hashes: PASS
```

Noisy load rerun after the change:

```text
label=50bots-20260507-persistent-ticket-save-noisy
NOISY / non-comparable
online_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=17.30
avg_tick_ms_avg=82.93
avg_tick_ms_max=814.44
loaded_chunks_max=819
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

This is accepted as a production-path save packing optimization with forced
ticket persistence evidence and no repeated `save-all` watchdog in the noisy
rerun. It is not an end-to-end TPS/MSPT win and is not promoted over the clean
accepted baseline.

## Waypoint Locator-Bar Hot Path

Current accepted production shape:

```text
WaypointTransmitter.EntityAzimuthConnection computes the same
atan2(receiverX - sourceX, sourceZ - receiverZ) directly instead of allocating
temporary Vec3 objects through subtract(...).rotateClockwise90().

WaypointTransmitter.EntityChunkConnection keeps the last chunk long key next to
the last ChunkPos, so isBroken() can call isChunkVisible(long, player) without
recomputing the key.
```

Rejected continuation:

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

Verdict: the distance/inner-range guard shape is not in the current production
patch. The accepted waypoint work remains direct azimuth and cached chunk key
only, with no end-to-end TPS/MSPT claim.

## Rejected FlatCache Context Reuse

Candidate:

```text
Reuse a per-NoiseChunk MutableSinglePointContext while constructing FlatCache
instead of allocating one MutableSinglePointContext per FlatCache wrapper.
```

Functional gates passed on the temporary artifact:

```text
build: PASS
plugin matrix: PASS, Done (52.049s)
restart/recovery: PASS, Done (30.762s)
forced ticket persistence: PASS
```

Noisy load gate:

```text
label=50bots-20260507-flatcache-context-noisy
NOISY / non-comparable
online_max=50
bot_active_max=50
tps1_avg=18.08
avg_tick_ms_avg=36.69
loaded_chunks_max=385
watchdog_thread_dumps=3
sync_load_stack_hits=0
```

The candidate was reverted because the watchdog stack included
`NoiseChunk$FlatCache.<init>` and `NoiseChunk.wrapNew(...)`, and chunk coverage
was far below the accepted baseline. Patch `0038-Reuse-NoiseChunk-FlatCache-context.patch`
was deleted and the final artifact was rebuilt without it.

## Plugin Remapper Index Lazy Cleanup

Change:

```text
RemappedPluginIndex.getAllIfPresent(...) now avoids eager stale-cache cleanup on
the stable all-cached startup path. When current cached/remap-skip entry count
matches the current plugin input count, it resolves cached/precomputed entries
directly and skips building a temporary input-hash HashSet plus cleanup
iteration.

Cleanup still runs when the plugin set size changes, any cache miss occurs, or a
precomputed remap/skip install changes the index entry count. Cache keys remain
exact sha256(plugin.jar), result order is still the input plugin order, and
plugin classloading/remap/skip decisions are unchanged. Duplicate-content
all-cached batches can defer deletion of extra stale cache files until a later
size-change/miss cleanup, which is not plugin-visible behavior.
```

Functional gates:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
scripts/bench_remapper_index_cleanup.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-remap-index-lazy-cleanup-count-only BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Results:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
microbench: PASS, old_eager_cleanup_best_ms=2060.532, new_lazy_cleanup_best_ms=626.871, lazy_cleanup_speedup=3.287x
plugin matrix: PASS, Done (44.640s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (27.677s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 15.625s/11.293s
strict load preflight: BLOCKED, exit=75 before starting Minecraft, load1=17.18, load_per_cpu=1.432, idle_percent_1s=17.19
artifact hash: optimized-paper-1.21.10-mojmap.jar 2a6651866719560bc446ae11c6b869c86b65c875ac7df64e9c2bdd13ca625353
```

This is accepted only as startup/classpath work reduction with a narrow
microbench and real plugin matrix evidence. No end-to-end boot, TPS, MSPT, or
500-player claim is made from this run because the host preflight blocked
comparable load benchmarking. Duplicate-content all-cached batches may defer
stale file cleanup until a later size-change/miss cleanup, but that is not a
plugin-visible remap/classloading behavior change.

## Plugin Remapper Dirty Index Writes

Change:

```text
RemappedPluginIndex now tracks whether its index changed. Stable cached
startups/restarts that only read existing .paper-remapped/*/index.json files no
longer rewrite the same JSON during PluginRemapper shutdown.

Dirty is marked for a new index, mappings-hash mismatch, precomputed remap/skip
install, remap/skip recording, and cleanup removals. Unknown-origin cleanup also
marks dirty when it removes an unused cached jar.
```

Functional gates:

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

Results:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (36.844s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (26.059s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 16.056s/12.553s
dirty-write check: PASS, remapper_index_mtime_unchanged=PASS across index.json, extra-plugins/index.json, libraries/index.json, unknown-origin/index.json; targeted restart Done (18.663s)
strict load preflight: BLOCKED, exit=75 before starting Minecraft, load1=13.62, load_per_cpu=1.135, idle_percent_1s=42.74
artifact hash: optimized-paper-1.21.10-mojmap.jar f3a5f0c746b400c880e1827e24207de81d8c6ccf946c82720d224b7ccba4ac30
```

This is accepted as stable restart/shutdown disk-I/O reduction with direct mtime
evidence. No end-to-end startup, TPS, MSPT, or 500-player claim is made from
this run because the strict load preflight still blocked comparable load
benchmarking.

## ReobfServer Precomputed Server Before Mappings

Change:

```text
ReobfServer no longer starts the expensive reobf mappings load before checking
whether paper.precomputedRemapClasspathDir already contains the remapped server
classpath jar for the current mappings hash. If the precomputed server jar is
present, it is installed first and mappings are not loaded for ReobfServer at
all. If the precomputed jar is absent, the normal mappings load and server
remap path still runs. The remap-classpath cleanup still runs immediately before
installing the precomputed jar or writing the freshly remapped server jar.
```

Functional gates:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
# targeted manual run recorded in reports/reobf-precomputed-mapping-check.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-reobf-precomputed-mapping BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Results:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
targeted precomputed-server check: PASS, install_precomputed_server_count=1, loading_precomputed_reversed_count=1, loading_reobf_mappings_count=0, compatprobe_plugin_remap_count=1, Done (14.313s)
plugin matrix: PASS, Done (32.332s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (19.249s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 17.516s/12.120s
strict load preflight: BLOCKED, exit=75 before starting Minecraft, load1=14.50, load_per_cpu=1.209, idle_percent_1s=8.52
artifact hash: optimized-paper-1.21.10-mojmap.jar fb9439f8ab3136e7c2055ebdc02fd3dde24be254d66f29708db4e73917455bea
```

This is accepted as first-run remap startup-work reduction for servers that
have a precomputed server remap but do not have precomputed plugin remaps for a
specific plugin jar. It is not an end-to-end cold-start speedup claim because
the comparable load/startup preflight is still blocked by unrelated host load.

## Atomic Hard-Link Install For Precomputed Remaps

Change:

```text
Precomputed remapped server jars and precomputed remapped plugin jars now
install through AtomicFiles.atomicLinkOrCopy(...). The helper creates a hard
link at a temporary file next to the destination, then atomically moves that
temp path into place. If hard links are unavailable, unsupported, or cross a
filesystem boundary, it falls back to the previous full copy behavior.

This preserves the previous destination paths and file names under
.paper-remapped, so plugin source path, classloader URL, and remap-index
semantics stay the same. The accepted effect is disk I/O reduction on the common
same-filesystem optimized-runtime path.
```

Functional gates:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
python3 ... > reports/precomputed-hardlink-check.txt
# targeted manual server-remap run recorded in reports/server-remap-hardlink-check.txt
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-after-hardlink-remap-install BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Results:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (34/37) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
plugin matrix: PASS, Done (47.750s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
plugin hardlink check: PASS, precomputed_plugin_hardlinks=4, all source/destination pairs samefile=true
server remap hardlink check: PASS, server_remap_samefile=true, source_inode=11087432, destination_inode=11087432, loading_reobf_mappings_count=0, compatprobe_plugin_remap_count=1
restart/recovery: PASS, Done (41.970s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 29.763s/23.963s
strict load preflight: BLOCKED, exit=75 before starting Minecraft, load1=41.43, load_per_cpu=3.452, idle_percent_1s=11.09
artifact hash: optimized-paper-1.21.10-mojmap.jar 06698b07a531aeac0f4cda36565dba6ba1711639210022241c6296582700a55c
```

This is accepted as startup/remap install disk-I/O reduction with direct inode
evidence. No end-to-end cold-start, TPS, MSPT, or 500-player claim is made from
this run because the host was heavily loaded.

## PalettedContainer Reencode Scratch Buffer

Change:

```text
PalettedContainer.reencodeContents(...) now reuses a per-thread int[] scratch
buffer for the temporary unpack/remap array. The remap loop, palette lookup
order, returned prefix values, and SimpleBitStorage packing semantics are
unchanged. SimpleBitStorage(int[]) consumes the array synchronously into its own
long[] storage and does not retain the int[] reference.
```

Microbenchmark:

```bash
./scripts/bench_paletted_reencode_scratch.sh
```

Raw result:

```text
old_newarray_best_ms=728.576
scratch_unpack_then_remap_best_ms=244.271
direct_packed_threadlocal_best_ms=858.637
threadlocal_speedup=2.983x
direct_vs_old_speedup=0.849x
direct_vs_scratch_speedup=0.284x
old_allocated_bytes_per_round=1966080000
scratch_allocated_bytes_per_thread=16384
equivalence=PASS
```

Functional gates:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true LOAD_TEST_LABEL=preflight-paletted-scratch-current BOT_COUNT=1 DURATION_SECONDS=1 VIEW_DISTANCE=2 SIMULATION_DISTANCE=2 ./scripts/run_load_test.sh
```

Results:

```text
rebuildPatches: PASS, Rebuilt 909 patches, Saved modified patches (35/38) for java
build: PASS, applySourcePatches Applied 909 patches, compileJava, createMojmapBundlerJar BUILD SUCCESSFUL
precompute: precomputed_plugin_remaps=4, precomputed_plugin_skips=8, precomputed_library_skips=1
artifact hashes: PASS
plugin matrix: PASS, Done (28.810s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (18.243s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 14.358s/11.217s
strict load preflight: BLOCKED, exit=75 before starting Minecraft, load1=11.88, load_per_cpu=0.990, idle_percent_1s=29.03
artifact hash: optimized-paper-1.21.10-mojmap.jar 9c792510f8f65a42d591dae373dfc17e1fb2192f48caa10fb9e7de57dab290b5
```

The direct packed pass was rejected and removed from production because it
regressed the microbench. Scratch reuse is accepted as save/serialization
allocation-pressure reduction. It is not an end-to-end TPS/MSPT or 500-player
claim: the following strict 50-bot gate was stable but failed the accepted
`18.27/47.85/2380` baseline.

Strict 50-bot 32/32 gate after scratch reuse:

```text
preflight: PASS, load1=5.52, load_per_cpu=0.460, idle_percent_1s=62.26
online_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=16.82
avg_tick_ms_avg=154.53
loaded_chunks_max=2127
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

## Plugin Directory DirectoryStream Scan

Change:

```text
DirectoryProviderSource and PluginRemapper.list now use DirectoryStream for flat
directory iteration instead of Files.list(...).stream(). The same validity filter
and unsorted directory traversal semantics are preserved.
```

Microbenchmark:

```bash
./scripts/bench_plugin_scan.sh
```

Raw result:

```text
directory=/root/rust/plugins/matrix
plugins_per_scan=12
warmup=4 rounds=8 iterations=5000
walk_depth1_best_ms=249.466
list_best_ms=153.480
directory_stream_best_ms=132.363
list_speedup=1.625x
directory_stream_vs_list_speedup=1.160x
```

Functional gates:

```bash
cd /root/rust/upstream/Paper && ./gradlew rebuildPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew applyPatches --no-daemon
cd /root/rust/upstream/Paper && ./gradlew compileJava --no-daemon
MC_EULA_AGREE=true ./scripts/build_optimized.sh
sha256sum -c reports/artifact-hashes.txt
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/restart_recovery_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true ./scripts/forced_ticket_persistence_check.sh /root/rust/artifacts/optimized-runtime/run.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/boot_benchmark.sh
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-plugin-dirstream-strict LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Results:

```text
rebuildPatches: PASS, Rebuilt 910 patches, Saved modified patches (35/38) for java
applyPatches: PASS, Applied 910 patches
compileJava: PASS
build: PASS, createMojmapBundlerJar BUILD SUCCESSFUL, AppCDS regenerated
artifact hashes: PASS
plugin matrix: PASS, Done (30.020s), PlayerJoinEvent sequence=3, PlayerQuitEvent sequence=4, COMPAT_PROBE command=ok events=4
restart/recovery: PASS, Done (18.986s), Saved the game, clean disable
forced ticket persistence: PASS, first/restart boot 15.655s/10.159s
boot benchmark: vanilla 14855 ms, stock Paper 32747 ms, optimized jar 24342 ms, optimized runtime 16488 ms
strict 50-bot preflight: PASS, load1=6.17, load_per_cpu=0.514, idle_percent_1s=77.21
strict 50-bot result: online_max=50, bot_kicked_max=0, bot_errors_max=0, tps1_avg=16.70, avg_tick_ms_avg=262.37, loaded_chunks_max=2771, watchdog_thread_dumps=0, sync_load_stack_hits=0
artifact hash: optimized-paper-1.21.10-mojmap.jar dfd2fc254c47dd98ac7b10f9772209e15341864302211363e444c2d4e3d3dd4d
```

This is accepted as plugin-discovery startup work reduction. It is not accepted
as a TPS/MSPT/load win: the strict 50-bot run was stable but worse than the
accepted `18.27/47.85/2380` baseline. It is also not a `<1s` startup claim.

## Current JFR After DirectoryStream

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-dirstream-current-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-dirstream-current-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Load result:

```text
preflight: PASS, load1=5.32, load_per_cpu=0.443, idle_percent_1s=85.48
online_max=50
bot_connected_max=50
bot_ready_max=50
bot_active_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=18.79
avg_tick_ms_avg=40.66
loaded_chunks_max=1835
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

This is a useful profile run but not a new accepted baseline because loaded
chunk coverage is lower than `2380`.

JFR hot methods:

```text
ImprovedNoise.p(int)                                           49.18%
Climate$RTree$SubTree.search(long[], Climate$RTree$Leaf)        3.07%
ImprovedNoise.noise(double, double, double, double, double)     2.18%
Aquifer$NoiseBasedAquifer.computeSubstance(...)                 1.80%
NoiseChunk$NoiseInterpolator.compute(...)                       1.54%
```

JFR allocation sites:

```text
NoiseChunk$FlatCache.<init>(...)                              11.49%
Iterators.forArrayWithPosition(Object[], int)                 10.56%
LZ4BlockOutputStream.<init>(...)                               3.68%
NoiseChunk.wrapMarker(...)                                     3.26%
XoroshiroRandomSource.<init>(long, long)                       2.92%
```

GC:

```text
pauses=56
total_pause_time=6.00s
p95_pause=318ms
max_pause=390ms
```

## Rejected NoiseInterpolator Flat Slice Microbench

Command:

```bash
./scripts/bench_noise_interpolator_slice.sh
```

Raw result:

```text
interpolators=96
cell_count_xz=4
cell_count_y=48
iterations=2000
warmup=4 rounds=6
old_jagged_best_ms=284.036
flat_best_ms=286.847
flat_speedup=0.990x
old_arrays_per_chunk=1152
flat_arrays_per_chunk=192
equivalence=PASS
```

The flat representation reduced modeled array count but did not improve runtime,
so `NoiseChunk.NoiseInterpolator` production code was not changed.

## Rejected NoiseChunk Empty-BlendCache Allocation Skip

Microbench command:

```bash
./scripts/bench_noisechunk_blendcache.sh
```

Microbench result:

```text
iterations=5000000
size_xz=5
old_empty_blender_best_ms=430.571
new_empty_blender_best_ms=10.449
speedup=41.207x
old_arrays_per_noisechunk=2
new_arrays_per_empty_blender_noisechunk=0
equivalence=PASS
```

Temporary production gate:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-noisechunk-empty-blendcache-gate LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Temporary production result:

```text
preflight: PASS, load1=8.67, load_per_cpu=0.723, idle_percent_1s=44.54
online_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=17.96
avg_tick_ms_avg=158.83
loaded_chunks_max=2424
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The candidate was rejected because it failed the accepted comparable baseline
`18.27/47.85/2380`. The production patch was removed and the runtime rebuilt.

Postrevert verification:

```text
applyPatches: PASS, Applied 910 patches
compileJava: PASS
build_optimized: PASS
artifact hashes: PASS
plugin matrix: PASS, Done (27.986s)
restart/recovery: PASS, Done (25.739s)
forced-ticket persistence: PASS, first/restart 18.289s/8.967s
optimized-paper sha256: 0b64abf35e9b1390190d57e077fed434a20e23a933bd5214bd7ed57b4e986bda
```

Postrevert strict 50-bot rerun:

```text
preflight: PASS, load1=8.78, load_per_cpu=0.731, idle_percent_1s=55.64
online_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=17.79
avg_tick_ms_avg=86.26
loaded_chunks_max=2981
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The postrevert run is stable evidence, not a new baseline: TPS and average tick
time still fail the accepted baseline, even though chunk coverage was higher.

## Current JFR After NoiseChunk Revert

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-after-noisechunk-revert-jfr LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms4G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:StartFlightRecording=filename=/root/rust/reports/load-50bots-after-noisechunk-revert-jfr.jfr,settings=profile,dumponexit=true' ./scripts/run_load_test.sh
```

Load result:

```text
preflight: PASS, load1=5.69, load_per_cpu=0.474, idle_percent_1s=78.19
online_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_avg=18.04
avg_tick_ms_avg=70.58
loaded_chunks_max=2148
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

JFR hot methods:

```text
ImprovedNoise.p(int)                                           48.83%
Climate$RTree$SubTree.search(long[], Climate$RTree$Leaf)        2.59%
ImprovedNoise.noise(double, double, double, double, double)     2.17%
NoiseChunk.updateForZ(int, double)                              2.15%
NoiseChunk$NoiseInterpolator.compute(...)                       1.82%
Aquifer$NoiseBasedAquifer.computeSubstance(...)                 1.75%
```

JFR allocation sites:

```text
NoiseChunk$FlatCache.<init>(...)                              11.30%
Iterators.forArrayWithPosition(Object[], int)                 10.38%
LZ4BlockOutputStream.<init>(...)                               3.27%
NoiseChunk.wrapMarker(...)                                     3.16%
XoroshiroRandomSource.<init>(long, long)                       2.93%
```

GC:

```text
pauses=51
total_pause_time=5.65s
p95_pause=444ms
max_pause=646ms
```

The old rejected areas remain visible, but they are not safe to repeat without
a new design: `ImprovedNoise.p` direct/int/local forms regressed, `FlatCache`
allocation skipping/context reuse regressed, and `SurfaceRules.SequenceRule`
indexed iteration hit watchdog.

## Blocked Fixed-10G G1 Retry

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 LOAD_TEST_LABEL=50bots-g1-xms10g-retry-after-revert LOAD_TEST_GAMEMODE=spectator BOT_COUNT=50 DURATION_SECONDS=120 VIEW_DISTANCE=32 SIMULATION_DISTANCE=32 PAPER_CHUNK_WORKER_THREADS=12 JAVA_OPTS_LOAD='-Xms10G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15' ./scripts/run_load_test.sh
```

Preflight result:

```text
host_preflight_ok=false
load1=11.00
load_per_cpu=0.917
idle_percent_1s=85.35
max_load_per_cpu=0.750
```

No JVM flag change is made from this blocked run.

## ObfHelper Mapping Bootstrap Pre-Size

Production scope:

- `io.papermc.paper.util.ObfHelper` pre-sizes known class/method/field maps while loading `META-INF/mappings/reobf.tiny`.
- The same path now pre-sizes the `StringPool` backing map from the real class/method/field mapping counts.
- Top-level `mappingsByObfName` and `mappingsByMojangName` are built with direct loops instead of stream collectors.
- Plugin runtime, classloading, mapping keys, reflection remap results, scheduler, services, events, and gameplay state are not changed.

Microbench command:

```bash
bash scripts/bench_obfhelper_maps.sh
```

Raw result from `reports/obfhelper-maps-bench.txt`:

```text
classes=7555
old_stream_default_maps_best_ms=256.414
presized_double_pool_best_ms=216.409
presized_double_pool_string_pool_best_ms=216.119
direct_top_maps_best_ms=224.191
presized_string_pool_best_ms=209.872
new_presized_single_pool_best_ms=227.452
presized_double_pool_speedup=1.185x
presized_double_pool_string_pool_speedup=1.186x
presized_double_pool_string_pool_vs_double_pool_speedup=1.001x
direct_top_maps_speedup=1.144x
direct_top_maps_vs_double_pool_speedup=0.965x
presized_string_pool_speedup=1.222x
presized_string_pool_vs_double_pool_speedup=1.031x
presized_speedup=1.127x
single_pool_vs_double_pool_speedup=0.951x
equivalence=PASS
```

Verdict:

- accepted with limits: current direct top-level map path plus pre-sized `StringPool` is kept as mapping-bootstrap allocation/work reduction;
- not promoted as separate wins: default-pool direct maps and the single-pool/set variant are slower/noisier in this run;
- no end-to-end cold-start, TPS, or 500-player claim is made from this microbench.

Final verification after the direct-map + pre-sized `StringPool` build:

```text
build_optimized: PASS, applySourcePatches Applied 910 patches
artifact hashes: PASS
plugin matrix: PASS, Done (28.748s)
restart/recovery: PASS, Done (17.694s)
forced-ticket persistence: PASS, first/restart Done (13.384s)/(10.195s)
optimized-paper sha256: 974595c6deaabfecf6457f2465eda505bfe39b725731bb8511192f35d1609b7e
app-cds sha256: e84f0c8c2741b5b703bc5b39eecaaaa396ace21946af61bfb32516d47920e07b
```

Boot benchmark after this cycle:

```text
vanilla-1.21.10 done_ms=15964
stock-paper-1.21.10 done_ms=32226
optimized-paper-1.21.10 done_ms=23102
optimized-runtime-1.21.10 done_ms=17776
```

This did not improve the previous optimized-runtime boot report, so no startup-speed claim is made.

Strict 50-bot 32/32 gate after the direct-map path:

```text
preflight: PASS, load1=6.09, load_per_cpu=0.508, idle_percent_1s=77.52
online_max=50
bot_kicked_max=0
bot_errors_max=0
tps1_min=8.02
tps1_avg=18.34
avg_tick_ms_avg=297.56
loaded_chunks_max=2635
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

The run is stability evidence only. It failed the accepted comparable baseline
`18.27/47.85/2380` on average tick time and did not meet stable 20 TPS.

## Rejected LZ4 Stream Wrapper

The region-file LZ4 write path was tested with the outer `BufferedOutputStream`
removed around `LZ4BlockOutputStream`. A standalone equivalence microbench on
the real runtime classpath showed the no-outer-buffer variant faster
(`3432.518 ms` buffered default vs `3028.499 ms` no outer buffer, `1.133x`),
but the real 50-bot 32/32 gate regressed versus the accepted baseline:
`tps1_avg=18.53`, `avg_tick_ms_avg=80.71`, `loaded_chunks_max=2085`, with
`watchdog_thread_dumps=0` and `sync_load_stack_hits=0`.

The production patch was removed and the generated source is back to
`new BufferedOutputStream(new LZ4BlockOutputStream(stream))`. Post-revert
verification passed: `MC_EULA_AGREE=true ./scripts/build_optimized.sh`,
`sha256sum -c reports/artifact-hashes.txt`, plugin matrix `Done (29.009s)`,
restart/recovery `Done (17.360s)`, and forced-ticket persistence
`13.751s` / `9.511s`.

A fresh strict post-revert control run on the current artifact completed with
all 50 spectator bots and no kicks/errors/watchdog/sync-load, but still did not
beat the accepted baseline: `tps1_avg=18.17`, `avg_tick_ms_avg=48.19`,
`loaded_chunks_max=1923` (`reports/load-50bots-post-lz4-revert-current-summary.txt`).

## SurfaceRules SequenceRule Array Candidate Rejected And Reverted

Standalone microbench evidence from
`reports/surfacerules-sequence-array-bench.txt`:

```text
iterations=20000000
rules=14
list_enhanced_best_ms=587.609
list_indexed_best_ms=565.372
array_best_ms=314.925
array_indexed_best_ms=309.618
list_indexed_speedup=1.039x
array_speedup=1.866x
array_indexed_speedup=1.898x
equivalence=PASS
```

Production candidate:

```text
change: store computed runtime SurfaceRule entries in SurfaceRule[] instead of ImmutableList
source: net.minecraft.world.level.levelgen.SurfaceRules.SequenceRule
codec/list source: unchanged
rebuildPatches: PASS, Rebuilt 910 source patches
applyPatches: PASS, Applied 910 patches
build_optimized: PASS
artifact hashes: PASS
plugin matrix: PASS, Done (34.263s)
restart/recovery: PASS, Done (21.224s)
forced-ticket persistence: PASS, first/restart Done (17.307s)/(11.562s)
```

The earlier strict comparable 50-bot 32/32 gate was blocked by host load
before Minecraft start:

```text
command label: 50bots-surfacerules-sequence-array-gate
host_preflight_ok=false
load1=16.85
load_per_cpu=1.404
idle_percent_1s=49.27
max_load_per_cpu=0.750
```

Later strict gate after the source-level array-indexed change:

```text
command label: 50bots-surfacerules-array-index-gate-rerun2
host_preflight_ok=true
load1=6.56
load_per_cpu=0.547
idle_percent_1s=74.43
online_max=50
tps1_avg=15.95
avg_tick_ms_avg=117.42
loaded_chunks_max=1785
moved_too_quickly_warnings=1
watchdog_thread_dumps=0
sync_load_stack_hits=0
```

Verdict: rejected and reverted. The candidate did not beat the accepted
`18.27/47.85/2380` strict 50-bot baseline.

## Rejected ChunkDependencies Radius Lookup

This candidate targets the `RegularImmutableList.get(int)` stack observed in
the current worldgen JFR:
`ChunkDependencies.get(int) -> WorldGenRegion.getChunk(...)`, with callers from
biome and surface generation. The runtime change keeps the original
`ImmutableList` as the visible backing list but adds a `ChunkStatus[]` snapshot
for the hot indexed lookup path.

Raw result from `reports/chunk-dependencies-array-bench.txt`:

```text
timestamp_utc=2026-05-10T12:28:29Z
java=openjdk version "21.0.10" 2026-01-20
dependencies_size=9
iterations=128000000
old_immutable_list_get_best_ms=419.919
array_get_best_ms=341.251
array_get_speedup=1.231x
equivalence=PASS
```

Interpretation: the standalone lookup path improved, but the production
candidate was rejected after the strict 50-bot 32/32 gate failed the accepted
baseline and hit watchdog dumps:

```text
report: reports/load-50bots-chunkdependencies-radius-lookup-gate-20260510-summary.txt
preflight: host_preflight_ok=true, load_per_cpu=0.475, idle_percent_1s=82.06
online_max=50
tps1_avg=17.89
avg_tick_ms_avg=57.67
loaded_chunks_max=2792
watchdog_thread_dumps=3
sync_load_stack_hits=0
bot_kicked_max=0
bot_errors_max=0
```

No end-to-end TPS/load improvement is claimed, and the production patch was
removed.

## Rejected ImprovedNoise Derivative Flat Gradient

This candidate targeted `ImprovedNoise.sampleWithDerivative(...)`, replacing
the `SimplexNoise.GRADIENT[index]` int-array references and
`SimplexNoise.dot(...)` calls with a local flat `int[]` gradient table and
direct dot products. The existing `p(...)` permutation path was deliberately
left unchanged.

Raw result from `reports/improved-noise-derivative-bench.txt`:

```text
old_derivative_best_ms=53.103
inline_derivative_best_ms=54.344
int_table_derivative_best_ms=56.539
flat_gradient_derivative_best_ms=50.027
flat_gradient_derivative_speedup=1.061x
equivalence=PASS
```

Production gates passed before the load gate:

```text
rebuildPatches/applyPatches: PASS, candidate patch 0051 present
build_optimized.sh: PASS
artifact JSON/hash checks: PASS
plugin matrix: PASS, Done (26.221s)
restart/recovery: PASS, Done (17.577s)
forced-ticket persistence: PASS, first/restart Done (13.630s)/(9.101s)
```

Strict comparable gate:

```text
report: reports/load-50bots-improvednoise-derivative-flat-gradient-gate-20260510-summary.txt
preflight: host_preflight_ok=true, load_per_cpu=0.576, idle_percent_1s=79.35
online_max=50
tps1_avg=15.36
avg_tick_ms_avg=94.24
loaded_chunks_max=3850
watchdog_thread_dumps=2
nearby_players_stack_hits=8
sync_load_stack_hits=0
bot_kicked_max=0
bot_errors_max=0
```

Decision: rejected and reverted. The microbench was positive, but the real
50-bot 32/32 gate failed the accepted `18.27/47.85/2380` baseline and produced
watchdog dumps. The production runtime is rebuilt without the candidate.

## Rejected Before Production: CompoundTag Map Initial Capacity

The JFR allocation view pointed at
`Object2ObjectOpenHashMap.<init>(int, float)` inside
`CompoundTag.loadCompound(...)` during real chunk NBT reads. A standalone bench
was added to parse real `.mca` chunk NBT payloads with different initial
capacities for the same fastutil map shape.

Raw result from `reports/nbt-compound-map-capacity-bench.txt`:

```text
chunks_used=512
compound_count=228744
compound_entries_total=844594
compound_entries_max=40
equivalence=PASS
cap2_best_ms=2080.649
cap4_best_ms=1922.989
cap8_best_ms=1907.510
cap16_best_ms=1957.953
cap2_vs_current_speedup=0.917x
cap4_vs_current_speedup=0.992x
cap16_vs_current_speedup=0.974x
```

Decision: rejected before production. Smaller capacities reduce modeled
allocation, but they are slower on this real chunk sample; larger capacity is
also slower. Keep `Object2ObjectOpenHashMap<>(8, 0.8F)`.

## Native NoiseChunk Wrap Capacity

Command:

```bash
SKIP_NATIVE_BUILD=1 JAVA_PROPS='-Dwarmup=1 -Drounds=3 -DmapBenchIterations=100' ./scripts/bench_native_noisechunk_wrap_capacity.sh
```

Result:

```text
report: reports/native-noisechunk-wrap-capacity-bench.txt
sample_count=336
variant_count=13
map_bench_iterations=100
map_shape=expected_12288_075 n=16384 maxFill=12288
map_shape=expected_16384_075 n=32768 maxFill=24576
java_summary_best_ms=10.650
native_summary_best_ms=10.023
native_speedup_vs_java=1.063x
equivalence=PASS
script_status=PASS
```

The candidate matrix now includes `expected_12288_075`,
`expected_12289_075`, and `expected_16384_075`. This is still diagnostic-only:
the earlier strict 50-bot gate rejected the runtime NoiseChunk capacity patch,
so these expanded shapes are only for future measurement.

## Native Deflater Input Shape

Command:

```bash
./scripts/bench_native_deflater_input_shape.sh
```

Result:

```text
report: reports/native-deflater-input-shape-bench.txt
payloads=256
bytes=16384
iterations=8
copied_native_speedup_vs_java=1.354x
slice_native_speedup_vs_java=1.067x
equivalence=PASS
script_status=PASS
```
