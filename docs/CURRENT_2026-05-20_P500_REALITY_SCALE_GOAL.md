# Current Reality-Scale Goal

Date: 2026-05-20 CEST

This is the active goal for the current `/root/rust` loop.
Literal unlimited is not a claim. The target is measured scale with fresh
gates and evidence.

## Target Claim

> production-ready for measured `500 bots / 32 view / 32 simulation /
> creative block` profile on a verified artifact, with cold+warm soak,
> repeat quorum, plugin matrix, restart/recovery, forced-ticket persistence,
> and a self-contained evidence bundle.

Not allowed to claim:

- [ ] unlimited players
- [ ] unlimited mobs
- [ ] full Rust Paper runtime
- [ ] real-player parity without live client evidence
- [ ] any claim without fresh summary + gate + artifact hashes

## Current Verified Evidence

- [x] Current artifact hashes verified:
  - `artifacts/optimized-paper-1.21.10-mojmap.jar`
    `d2b676b0d533d31d58c3b9b366aaf944197e5e26d1b7db57405b41c1463a6ae9`
  - `artifacts/optimized-runtime/run.sh`
    `b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a`
- [x] Fresh red baseline exists:
  `reports/load-p500-current-waitteleport-diagnostic-20260520-181858-summary.txt`
  with `online_max=494`, `bot_active_max=494`, `bot_kicked_max=1`,
  `bot_errors_max=6`, `tps1_avg=9.24`, `avg_tick_ms_avg=132.70`.
- [x] Fresh red block diagnostic exists:
  `reports/load-p500-current-optflush-containerupdate4-20260520-191049-summary.txt`
  with `online_max=13`, `bot_errors_max=258`, `tps1_avg=12.39`,
  `avg_tick_ms_avg=94.40`, `gate_pass=false`.
- [x] The failing run produced a real gate report:
  `reports/load-p500-current-optflush-containerupdate4-20260520-191049-gate.txt`.
- [x] Hot samples show the current bottleneck is not one thing:
  main thread in `AbstractContainerMenu.synchronizeCarriedToRemote`,
  workers in `LevelChunkSection.fillBiomesFromNoise`, and host steal noise was
  visible in `top` during the run.
- [x] `PAPER_TICK_RATE_CONTAINER_UPDATE=4` was measured and rejected for the
  current artifact.
- [x] Host-steal telemetry is present in new summaries:
  `reports/load-hostcpu-smoke-20260520-193155-summary.txt` recorded
  `host_cpu_windows=6`, `host_cpu_steal_percent_max=45.15`, and
  `host_cpu_steal_percent_avg=14.49`. This smoke is telemetry evidence only,
  not a production claim.

## Current Open Work

- [ ] Fresh P500 block gate passes on the current artifact.
- [ ] Fresh cold + warm soak passes on the accepted tier.
- [ ] Repeat quorum passes on the accepted tier.
- [ ] Plugin matrix passes on the same artifact.
- [ ] Restart / recovery passes under load.
- [ ] Forced-ticket persistence passes under load.
- [ ] Evidence bundle validates and matches current hashes.
- [x] Host-steal metrics are tracked in the load harness so bad VPS noise is
  visible in evidence.
- [ ] Login / worldgen / container hot paths are separated into measurable
  sub-gates.
- [ ] The next tier is chosen from fresh evidence, not from wishful
  extrapolation.

## Scale Ladder

- [ ] P750 mixed/block diagnostic.
- [ ] P1000 mixed/block diagnostic.
- [ ] P2500 mixed/block diagnostic if hardware allows.
- [ ] M10k mixed mobs diagnostic.
- [ ] C25k loaded chunks diagnostic.
- [ ] P5000 only if the earlier measured tiers stay green.

## Working Rule

A tier only becomes a claim when:

- [ ] the summary is fresh,
- [ ] the gate is green,
- [ ] the artifact hashes match,
- [ ] the evidence bundle validates,
- [ ] the run is repeatable on the same artifact.
