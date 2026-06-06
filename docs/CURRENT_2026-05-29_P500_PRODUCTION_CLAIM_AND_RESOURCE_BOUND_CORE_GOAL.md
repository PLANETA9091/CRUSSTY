# Current 2026-05-29: P500 production claim and resource-bound core goal

Date: 2026-05-29 CEST

This is the active execution goal for `/root/rust`.

The host is allowed to be dirty for diagnostics. A dirty host is useful because
it exposes RAM, CPU, IO, scheduler, and queue pressure that a clean lab can
hide. But a dirty host is not allowed to produce the final production claim.
The final claim must still pass the strict validator on a verified artifact.

Plugins and datapacks are stress inputs only. Do not update, patch, simplify,
or tune plugins/datapacks to make the gate green. The core runtime must carry
the optimization work.

## Allowed Claim

Allowed only after all required gates are green:

> production-ready for measured 500 bots / 32 view / 32 simulation / creative
> block on a verified current artifact, with cold+warm soak, repeat quorum,
> plugin matrix, restart/recovery, forced-ticket persistence, bounded resource
> growth, no watchdog/sync-load failures, and a validated self-contained
> evidence bundle.

## Current Truth

- [x] Historical P500 green evidence exists only for older artifact hashes.
- [x] Current artifact evidence must be regenerated from fresh runs.
- [x] Dirty-host P500 diagnostics are allowed and should continue.
- [x] Dirty-host diagnostics are not production claims.
- [x] The current P500 run target is still 500 bots, 32 view, 32 simulation,
  creative block actions.
- [x] Bot harness memory pressure is measured separately from server memory.
- [x] Plugin/datapack optimization is out of scope.
- [x] Core-only changes must preserve Bukkit/Paper semantics and recovery.
- [ ] Current artifact has a fresh green strict P500 production bundle.

## Active Heavy Run

- [x] Start full P500 profiled diagnostic on the dirty host.
- [x] Use 500 bots.
- [x] Use 32 view distance.
- [x] Use 32 simulation distance.
- [x] Use creative block workload.
- [x] Keep backpressure enabled.
- [x] Capture resource CSV.
- [x] Capture thread samples.
- [x] Capture server memory snapshots.
- [x] Enable JFR.
- [x] Enable GC/safepoint log.
- [x] Enable Native Memory Tracking.
- [ ] Produce summary report.
- [ ] Produce gate report.
- [ ] Produce hotspot rank.
- [ ] Produce memory peak summary.
- [ ] Produce JFR hot-methods report.
- [ ] Produce JFR allocation-by-site report.
- [ ] Produce GC pause report.
- [ ] Pick the next core hotspot from this run, not from guesses.

Current run label:

```text
p500-profiled-20260529-094738
```

Expected evidence paths:

```text
reports/load-p500-profiled-20260529-094738-summary.txt
reports/load-p500-profiled-20260529-094738-gate.txt
reports/load-p500-profiled-20260529-094738-resources.csv
reports/load-p500-profiled-20260529-094738-hotspot-rank.txt
reports/load-p500-profiled-20260529-094738-memory/peak-latest.txt
reports/p500-profiled-20260529-094738.jfr
reports/gc-p500-profiled-20260529-094738.log
```

## Immediate Core Optimization Rules

- [ ] Optimize the hottest measured core path first.
- [ ] Prefer server CPU/RAM/queue reductions over harness-only wins.
- [ ] Keep all native hooks opt-in and fallback-safe.
- [ ] Keep Java fallback behavior correct if native loading fails.
- [ ] Do not revive rejected `NoiseChunk` capacity changes without new
  size-distribution evidence and a strict load win.
- [ ] Do not claim native noise integration unless runtime jar contains the
  Java hook and the run proves the hook is active.
- [ ] Do not rebuild patches casually while feature-patch deletion/source-patch
  migration state is unresolved.

## Candidate Hotspot Buckets

These buckets are only candidates until the active P500 evidence ranks them:

- [ ] Player chunk sending and network backpressure:
  `PlayerChunkSender`, `Connection`, `PacketProcessor`.
- [ ] Ticket and chunk lifecycle:
  `TicketStorage`, `DistanceManager`, `ChunkMap`, forced-ticket recompute.
- [ ] Worldgen density graph churn:
  `NoiseChunk`, `DensityFunctions.mapAll`, wrapper maps, flat caches.
- [ ] Noise/math leaf paths:
  `ImprovedNoise`, `PerlinNoise`, `NormalNoise`, only if currently hooked.
- [ ] Area update churn:
  `SingleUserAreaMap` and `PaperNativeAreaMap`.
- [ ] Climate search:
  `ClimateRTree`, only if JFR still shows it after native hook.
- [ ] GC/allocation pressure:
  `FlatCache`, `SinglePointContext`, transient arrays, packet buffers.

## Diagnostic Pass Criteria

Dirty-host diagnostic is useful if it gives enough evidence to optimize:

- [ ] `observed_online_max` reaches 500 or the failure explains why not.
- [ ] Resource CSV contains server RSS and bot aggregate RSS/PSS.
- [ ] Thread samples cover the load window.
- [ ] Memory snapshot captures peak RSS/heap/metaspace/NMT.
- [ ] JFR exists and contains hot methods.
- [ ] GC log exists and contains pause data.
- [ ] Gate report records TPS/MSPT/watchdog/sync-load outcome.
- [ ] The next optimization target is named with file paths.

## Strict Production Claim Gate

Production claim requires:

- [ ] Go/no-go preflight passes without strict foreign-process blockers.
- [ ] Artifact hashes are fresh.
- [ ] Source freshness check passes.
- [ ] Cold P500 soak passes.
- [ ] Warm P500 soak passes.
- [ ] Repeat quorum passes.
- [ ] Plugin matrix passes.
- [ ] Restart/recovery passes.
- [ ] Forced-ticket persistence passes.
- [ ] Runtime logs are clean.
- [ ] Evidence bundle validates.
- [ ] Claim assertion passes.
- [ ] Stable current claim files are published.

Required claim metrics:

- [ ] 500 online in the load window.
- [ ] 32 view distance.
- [ ] 32 simulation distance.
- [ ] Creative block place/dig packets exceed required floor.
- [ ] TPS average is within production threshold.
- [ ] TPS minimum is within production threshold.
- [ ] MSPT average is within production threshold.
- [ ] MSPT max is within production threshold.
- [ ] Watchdog thread dumps equal 0.
- [ ] Sync-load stack hits equal 0.
- [ ] Stability failures equal 0.
- [ ] No unbounded heap/RSS/queue growth.

## Definition Of Done

- [ ] Current artifact has a fresh validated P500 production bundle.
- [ ] Published claim text matches exactly what was measured.
- [ ] Dirty-host diagnostics are preserved as optimization evidence, not as
  claim evidence.
- [ ] At least one measured core bottleneck is improved and re-tested.
- [ ] The next higher resource-bound tier is defined only after P500 is green.

