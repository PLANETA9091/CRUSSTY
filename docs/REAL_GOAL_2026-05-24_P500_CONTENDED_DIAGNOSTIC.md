# REAL GOAL 2026-05-24: P500 contended diagnostic path

This is a tactical goal under the broader current production-scale goal:

```text
docs/REAL_GOAL_2026-05-24_CORE_SCALE_PRODUCTION.md
```

The problem is not that another server exists. The strict production claim gate
must reject contaminated evidence, but engineering work must still continue on
the real busy host. This goal creates and runs a separate non-claim P500
diagnostic path that keeps moving while the production claim remains honest.

## Boundaries

- [x] Do not weaken `production-*` claim profiles.
- [x] Do not publish or refresh `production-500-readiness-bundle-current` from
  contaminated evidence.
- [x] Do not call contaminated evidence `production-ready`.
- [x] Allow foreign processes and busy host only in explicitly labeled
  non-claim diagnostics.
- [x] Keep the measured surface close to the production target:
  `500 bots / 32 view / 32 simulation / creative block`.

## Current Execution

- [x] Add a dedicated wrapper:
  `scripts/run_p500_contended_diagnostic.sh`.
- [x] The wrapper uses a non-production gate profile and P500-like settings.
- [x] The wrapper writes a stamped diagnostic report in `reports/`.
- [x] The wrapper states `production_claim_eligible=false`.
- [x] The wrapper refuses `production-*` gate profiles.
- [ ] Run the wrapper smoke:
  `scripts/run_p500_contended_diagnostic_smoke.sh`.
- [ ] Re-run strict production contamination smokes to prove claim gates still
  reject unsafe overrides.
- [x] Launch the real contended P500 diagnostic:
  `MC_EULA_AGREE=true scripts/run_p500_contended_diagnostic.sh`.
- [x] Compare the resulting summary/gate against the latest failed P500 cold
  evidence and identify the next core-only bottleneck.

## Latest Result: 2026-05-24 18:13:14 Europe/Berlin

- [x] Report:
  `reports/p500-contended-diagnostic-20260524-181314.txt`.
- [x] Summary:
  `reports/load-p500-contended-diagnostic-current-artifact-20260524-181314-summary.txt`.
- [x] Gate:
  `reports/load-p500-contended-diagnostic-current-artifact-20260524-181314-gate.txt`.
- [x] The previous `waiting-for-server-ready` issue is no longer the blocker:
  `startup_done_seconds=62.852`, and the bot phase ran.
- [x] The load harness reached the intended synthetic surface:
  `bot_created_max=500`, `bot_connected_max=500`,
  `bot_ready_max=500`, `bot_active_max=500`, `bot_errors_max=0`,
  `bot_kicked_max=0`.
- [x] The creative block workload executed:
  `bot_block_place_packets_max=67984`,
  `bot_block_dig_packets_max=67700`,
  `compat_probe_block_places_max=67249`,
  `compat_probe_block_breaks_max=67000`.
- [x] The diagnostic is useful non-claim evidence, not a production pass:
  `p500_contended_diagnostic_exit_code=1`,
  `claim_eligible=false`, `gate_pass=false`, `failure_count=11`.
- [x] Primary measured failures:
  `load_window_tps1_avg=6.29 < 19.00`,
  `load_window_tps1_min=1.31 < 17.50`,
  `load_window_avg_tick_ms_avg=607.06 > 55.00`,
  `load_window_avg_tick_ms_max=3047.91 > 125.00`,
  `load_window_loaded_chunks_max=686 < 4000`.
- [x] Host pressure was measured, but it is not a coding stop:
  `host_system_load1_per_cpu_max=2.80`,
  `host_cpu_steal_percent_max=51.93`,
  `host_cpu_iowait_percent_max=25.35`.
  This evidence can guide diagnostics; it still cannot create a production
  claim.
- [x] Additional core-facing symptoms to reduce next:
  `moved_too_quickly_warnings=420`, `watchdog_thread_dumps=14`,
  `external_thread_prints=11`.

## Promotion Rules

- [ ] A contended diagnostic can guide optimization.
- [ ] A contended diagnostic cannot restore the P500 production claim.
- [ ] A production claim still requires clean go/no-go, cold+warm soak, repeat
  quorum, plugin matrix, restart/recovery, forced-ticket persistence, bundle
  validation, and claim assertion on the same current artifact.
