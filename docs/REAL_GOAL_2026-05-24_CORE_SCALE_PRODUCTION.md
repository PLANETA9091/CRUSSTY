# REAL GOAL 2026-05-24: Core-only scale to measured production tiers

This is the active goal for the current `/root/rust` work loop.

The target is not a literal infinite server. Finite CPU, RAM, disk, network,
Linux scheduler, JVM limits, Minecraft protocol limits, and plugin behavior
exist. The real goal is stronger: keep removing core bottlenecks until every
player, mob, chunk, worldgen, tick, network, plugin, and recovery ceiling is
explained by current evidence instead of by an unmeasured Paper/runtime hot
path.

Plugins and datapacks are stress inputs. Do not patch, update, remove, or
optimize them to make a gate pass. If they are heavy, the core must absorb the
load or the evidence must name the exact core limit.

## Execution Rules

- [x] Main agent owns the critical path: build, hash, source freshness, gates,
  reports, bundle export, and final validators.
- [x] Subagents are used for sidecar diagnosis or disjoint work while the main
  path keeps moving.
- [x] Foreign/live servers are not a coding blocker. They only block an honest
  production claim if strict gates prove host contamination.
- [x] No production claim from stale artifact, stale bundle, microbench-only
  result, or noisy-host result.
- [x] No plugin/datapack optimization to pass a core gate.
- [x] Hotspot-first rule: before any optimization patch, rank the latest
  measured evidence and patch only the hottest core path supported by profiler,
  thread-sample, benchmark, gate, or resource evidence.
- [x] Evidence loop: diagnostic -> hotspot rank -> one bounded candidate ->
  focused bench -> rebuild/hash/freshness -> diagnostic or strict gate ->
  accept, revert, or record as diagnostic-only.
- [x] Every accepted claim must name exact profile, artifact hash, evidence
  bundle, and exact non-claims.

## Immediate Current Turn

- [x] Add a separate non-claim contended P500 diagnostic path so busy-host work
  continues without weakening `production-*` claim gates.
- [ ] Pass `scripts/run_p500_contended_diagnostic_smoke.sh`.
- [ ] Run `MC_EULA_AGREE=true scripts/run_p500_contended_diagnostic.sh` as
  diagnostic-only evidence if the smoke passes.
- [ ] Rebuild the optimized Paper artifact from the current source tree.
- [ ] Rebuild or verify the native runtime shipped with the artifact.
- [ ] Regenerate artifact reports and hash manifests.
- [ ] Pass `scripts/check_artifact_source_freshness.sh`.
- [ ] Pass source compile after the latest optimization patches.
- [ ] Pass focused microbenchmarks for the latest hot-path changes.
- [ ] Add at least one core-only hot-path improvement if a bounded target is
  verified during this loop.
- [ ] Run cheap evidence smokes after the rebuild.
- [ ] Produce a current go/no-go report that says whether a P500 production
  claim can honestly be attempted on this host now.

## P0 - Current Artifact Integrity

- [ ] `artifacts/optimized-paper-1.21.10-mojmap.jar` exists and is newer than
  all Paper source and patch inputs.
- [ ] `artifacts/optimized-runtime/run.sh` points at that jar.
- [ ] `artifacts/optimized-runtime/native/libpaper_native_jni.so` exists or the
  runtime is explicitly marked Java-only for the run.
- [ ] `reports/artifacts.json` records the current artifact, launcher, runtime,
  native library, AppCDS, and classpath hashes.
- [ ] `sha256sum -c reports/artifact-hashes.txt` passes.
- [ ] `JAVA_BIN=/bin/true artifacts/optimized-runtime/run.sh --nogui` proves
  the launcher binds to the expected runtime.

## P1 - Core Observability Required For Scale

- [ ] Server-wide chunk send pressure is visible in summaries.
- [ ] Server-wide connection pending actions and outbound pressure are visible.
- [ ] Packet processor queue depth is visible during load.
- [ ] Chunk generation queue depth and in-flight work are visible.
- [ ] Entity tracker fanout pressure is visible.
- [ ] Mob AI/pathfinding budget pressure is visible.
- [ ] Region IO queue/writeback pressure is visible.
- [ ] Host CPU steal, IO wait, RAM pressure, and process contamination are
  recorded in every production gate.
- [ ] Evidence bundle includes raw logs, summary, gate report, resource CSV,
  artifact hashes, and validator output.

## P2 - Restore Honest P500 Production Claim

- [ ] Cold P500 `500 bots / 32 view / 32 simulation / creative block` reaches
  500 ready, active, settled, and block-armed bots.
- [ ] Warm P500 reaches the same states on the same artifact.
- [ ] Both windows reach full online and stay inside TPS/MSPT thresholds.
- [ ] `load_window_metrics_samples >= 300`.
- [ ] No watchdog dumps, sync-load hits, kicked bots, protocol errors, or block
  workload errors in the claim window.
- [ ] Repeat quorum passes on the same artifact.
- [ ] Plugin matrix passes on the same artifact without plugin edits.
- [ ] Restart/recovery passes on the same artifact.
- [ ] Forced-ticket persistence passes on the same artifact.
- [ ] `reports/production-500-readiness-bundle-current` validates with current
  freshness required.
- [ ] `scripts/assert_production_ready_claim.py` passes against that bundle.

## P3 - Player Scale Ladder

- [ ] P750 diagnostic passes on a fresh artifact.
- [ ] P1000 diagnostic passes on a fresh artifact.
- [ ] P1500 diagnostic passes only after P1000 is green.
- [ ] P2500 diagnostic passes only if hardware/resource evidence supports it.
- [ ] Each promoted player tier gets cold+warm, repeat, recovery, plugin
  matrix, forced-ticket, and bundle evidence before any claim.

## P4 - Mob Scale Ladder

- [ ] M10k mob diagnostic passes with Bukkit-visible behavior intact.
- [ ] M25k mob diagnostic passes with bounded AI/pathfinding pressure.
- [ ] M50k mob diagnostic is attempted only after M25k evidence identifies no
  unresolved core blocker.
- [ ] Mob despawn, activation range, pathfinding, sensor, and goal-selector
  behavior remain plugin-compatible.
- [ ] No mob scale claim is made without exact entity mix, world, difficulty,
  tick budget, and evidence bundle.

## P5 - Chunk And Worldgen Scale Ladder

- [ ] C10k loaded/forced chunks diagnostic passes.
- [ ] C25k loaded/forced chunks diagnostic passes.
- [ ] C50k diagnostic is attempted only after C25k is stable.
- [ ] Worldgen optimization preserves vanilla/datapack/plugin hook behavior.
- [ ] Chunk generation queue, ticket pressure, IO writeback, lighting, and send
  pressure are all visible in evidence.
- [ ] No chunk/worldgen claim is made without forced-ticket persistence and
  restart/recovery evidence.

## P6 - Combined Stress

- [ ] P500 plus M10k plus C10k combined diagnostic passes.
- [ ] P1000 plus M25k plus C25k combined diagnostic is attempted only after the
  individual tiers are green.
- [ ] Reconnect storm gate passes for the leading accepted tier.
- [ ] Restart under load passes for the leading accepted tier.
- [ ] Disk pressure and low-free-space behavior are measured for the leading
  accepted tier.
- [ ] Long soak is run only after shorter cold/warm/repeat gates are green.

## P7 - Rust And Native Runtime Work

- [ ] Rust/JNI hooks are accepted only for proven hot paths.
- [ ] Every native replacement has Java parity tests or equivalence benches.
- [ ] Native load gate is strict: missing or wrong library means no native
  production claim.
- [ ] Native fallback behavior is explicit and measured.
- [ ] No claim says "full Rust Paper runtime" until the Paper runtime is
  actually replaced and plugin compatibility is proven.

## Non-Claims Until Separate Evidence Exists

- [ ] Literal unlimited players.
- [ ] Literal unlimited mobs.
- [ ] Literal unlimited chunks, ticks, or plugins.
- [ ] Full Rust Paper runtime.
- [ ] Real-player gameplay parity from synthetic bot evidence alone.
- [ ] Multi-hour or 24h soak unless that exact soak was measured.

## Done Definition

- [ ] Current artifact P500 production claim is restored with a fresh validated
  bundle.
- [ ] The next accepted player, mob, chunk, and combined tiers each have their
  own fresh bundles.
- [ ] Every wider statement is tied to exact artifact, exact profile, exact
  gate, exact hash, and exact non-claims.
- [ ] Remaining limits are named as measured hardware, policy, protocol,
  plugin, or core limits, not hidden unknowns.
