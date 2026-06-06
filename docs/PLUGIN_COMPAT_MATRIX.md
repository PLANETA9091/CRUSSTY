# Plugin Compatibility Matrix

Claim scope: this server passes the tested Paper/Bukkit plugin matrix below. It does not claim support for all plugins.

Stress-corpus note: on 2026-05-17 CEST a separate boot/join/datapack gate was
added for the next compatibility envelope. `scripts/run_stress_corpus_gate.sh`
boots the current matrix plus 22 additional stress plugins and 10 heavy
datapacks. The fresh gate passed with `matrix_plugin_count=12`,
`stress_plugin_count=22`, `plugin_count=34`, `datapack_count=10`,
`Done (153.340s)`, `StressProbe` join/quit, and `13 data pack(s) enabled`.
This is not a scale/load claim and does not replace the current production
500 plugin matrix; it is the corpus future mixed gameplay gates must survive.

Current runtime note: after the 2026-05-17 production-readiness certification
runner, the current optimized runtime passes the fresh real plugin matrix at
`Done (21.929s)`. The matrix initialized 11 plugins, installed precomputed
remaps for spark, Vault, CompatProbe, and PlaceholderAPI, loaded
`LibraryProbe` through the Paper plugin-library path, and `CompatProbe`
observed plugin lifecycle, async/sync scheduler ticks, command handling, and
join/quit coverage (`COMPAT_PROBE command=ok events=4`). Fresh
restart/recovery passes at `Done (15.527s)` with `Saved the game`, and
forced-ticket persistence passes with first/restart `Done (11.386s)` /
`Done (8.551s)`. This is tested-matrix compatibility evidence only, not an
all-plugin, multi-hour, real-player, or full-Rust-runtime claim.

Current runtime note: after adding
`0049-Optimize-SurfaceRules-state-test-rule.patch`, the rebuilt runtime
passes artifact hash verification and the real plugin matrix at
`Done (34.742s)`. The matrix used 11 real plugins, installed precomputed
remaps for spark, Vault, CompatProbe, and PlaceholderAPI, loaded
`LibraryProbe` through the Paper plugin-library path, and `CompatProbe`
observed plugin lifecycle, `ServerLoadEvent`, join/quit events, sync/async
scheduler ticks, and command handling (`COMPAT_PROBE command=ok events=4
ownServices=0`). Restart/recovery passes at `Done (19.091s)`, and
forced-ticket persistence passes with first/restart `Done (16.112s)` /
`Done (11.165s)`. EssentialsX still logs `You are running an unsupported
server version!`, and ProtocolLib/WorldEdit also warn that this Minecraft
version is not tested by those plugins. This is tested-matrix compatibility
evidence only, not an all-plugin, sub-second boot, 20 TPS, 500-player, or
vanilla-parity claim.

Current runtime note: after adding
`0048-Optimize-chunk-expire-count-lookup.patch`, the rebuilt runtime passes
artifact hash verification and the real plugin matrix at `Done (53.157s)`.
The matrix used 11 real plugins, installed precomputed remaps for spark,
Vault, CompatProbe, and PlaceholderAPI, loaded `LibraryProbe` through the
Paper plugin-library path, and `CompatProbe` observed plugin lifecycle,
`ServerLoadEvent`, join/quit events, sync/async scheduler ticks, and command
handling (`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery
passes at `Done (43.350s)`, and forced-ticket persistence passes with
first/restart `Done (28.668s)` / `Done (22.220s)`. EssentialsX still logs
`You are running an unsupported server version!`, and ProtocolLib/WorldEdit
also warn that this Minecraft version is not tested by those plugins. This is
tested-matrix compatibility evidence only, not an all-plugin, sub-second
boot, 20 TPS, 500-player, or vanilla-parity claim.

Current runtime note: after adding
`0047-Optimize-CompressionEncoder-deflater-input.patch`, the rebuilt runtime
passes artifact hash verification and the real plugin matrix at
`Done (51.284s)`. The matrix used 11 real plugins, installed precomputed
remaps for spark, Vault, CompatProbe, and PlaceholderAPI, loaded
`LibraryProbe` through the Paper plugin-library path, and `CompatProbe`
observed plugin lifecycle, `ServerLoadEvent`, join/quit events, sync/async
scheduler ticks, and command handling (`COMPAT_PROBE command=ok events=4
ownServices=0`). Restart/recovery passes at `Done (24.522s)`, and
forced-ticket persistence passes with first/restart `Done (20.692s)` /
`Done (17.519s)`. This is tested-matrix compatibility evidence only, not an
all-plugin, sub-second boot, 20 TPS, 500-player, or vanilla-parity claim.

Current runtime note: after rejecting and rolling back the temporary
`NoiseChunk.NoiseInterpolator.compute(...)` fraction-array candidate, the
rebuilt runtime passes artifact hash verification and the real plugin matrix
at `Done (29.035s)`. The matrix used 11 real plugins, installed precomputed
remaps for spark, Vault, CompatProbe, and PlaceholderAPI, loaded
`LibraryProbe` through the Paper plugin-library path, and `CompatProbe`
observed plugin lifecycle, `ServerLoadEvent`, join/quit events, sync/async
scheduler ticks, and command handling (`COMPAT_PROBE command=ok events=4
ownServices=0`). Restart/recovery passes at `Done (18.063s)`, and
forced-ticket persistence passes with first/restart `Done (16.174s)` /
`Done (12.176s)`. This is tested-matrix compatibility evidence only, not an
all-plugin, sub-second boot, 20 TPS, 500-player, or vanilla-parity claim.

Current runtime note: after rejecting and rolling back the temporary
player-loader cached-manager and `NearbyPlayers` limit64 candidates, the
rebuilt runtime passes artifact hash verification and the real plugin matrix
at `Done (29.443s)`. The matrix used 11 real plugins, installed precomputed
remaps for spark, Vault, CompatProbe, and PlaceholderAPI, loaded
`LibraryProbe` through the Paper plugin-library path, and `CompatProbe`
observed plugin lifecycle, `ServerLoadEvent`, join/quit events, sync/async
scheduler ticks, and command handling (`COMPAT_PROBE command=ok events=4
ownServices=0`). Restart/recovery passes at `Done (21.228s)`, and
forced-ticket persistence passes with first/restart `Done (21.372s)` /
`Done (11.272s)`. This is tested-matrix compatibility evidence only, not an
all-plugin, sub-second boot, 20 TPS, 500-player, or vanilla-parity claim.

Current runtime note: after fully rolling back the temporary
`ProtoChunk.setBlockState(...)` heightmap iterator-removal candidate, the
rebuilt runtime passes artifact hash verification and the real plugin matrix
at `Done (26.859s)`. The matrix used 11 real plugins, installed precomputed
remaps for spark, Vault, CompatProbe, and PlaceholderAPI, loaded
`LibraryProbe` through the Paper plugin-library path, and `CompatProbe`
observed plugin lifecycle, `ServerLoadEvent`, join/quit events, sync/async
scheduler ticks, and command handling (`COMPAT_PROBE command=ok events=4
ownServices=0`). Restart/recovery passes at `Done (16.028s)`, and
forced-ticket persistence passes with first/restart `Done (13.244s)` /
`Done (9.550s)`. This is tested-matrix compatibility evidence only, not an
all-plugin, 20 TPS, 500-player, or vanilla-parity claim.

Current runtime note: after rejecting and removing the temporary
`OreFeature.doPlace(...)` scalar-hoist candidate, the rebuilt runtime passes
artifact hash verification and the real plugin matrix at `Done (26.953s)`.
The matrix used 11 real plugins, installed precomputed remaps for spark,
Vault, CompatProbe, and PlaceholderAPI, loaded `LibraryProbe` through the
Paper plugin-library path, and `CompatProbe` observed plugin lifecycle,
`ServerLoadEvent`, join/quit events, sync/async scheduler ticks, and command
handling (`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery
passes at `Done (17.037s)`, and forced-ticket persistence passes with
first/restart `Done (12.862s)` / `Done (8.382s)`. This is tested-matrix
compatibility evidence only, not an all-plugin, 20 TPS, 500-player, or
vanilla-parity claim.

Current runtime note: after rejecting and removing the temporary
`RangeChoice` constant-out fillArray candidate, the rebuilt runtime passes
artifact hash verification and the real plugin matrix at `Done (26.927s)`.
The matrix used 11 real plugins, installed precomputed remaps for spark,
Vault, CompatProbe, and PlaceholderAPI, loaded `LibraryProbe` through the
Paper plugin-library path, and `CompatProbe` observed plugin lifecycle,
`ServerLoadEvent`, join/quit events, sync/async scheduler ticks, and command
handling (`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery
passes at `Done (16.461s)`, and forced-ticket persistence passes with
first/restart `Done (12.714s)` / `Done (8.332s)`. This is tested-matrix
compatibility evidence only, not an all-plugin, 20 TPS, 500-player, or
vanilla-parity claim.

Current runtime note: after rejecting and rolling back the temporary waypoint
chunk-key update-condition candidate, the rebuilt runtime passes artifact hash
verification and the real plugin matrix at `Done (27.799s)`. The matrix used
11 real plugins, installed precomputed remaps for spark, Vault, CompatProbe,
and PlaceholderAPI, loaded `LibraryProbe` through the Paper plugin-library
path, and `CompatProbe` observed plugin lifecycle, `ServerLoadEvent`, join/
quit events, sync/async scheduler ticks, and command handling
(`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery passes
at `Done (16.968s)`, and forced-ticket persistence passes with first/restart
`Done (13.274s)` / `Done (8.602s)`. This is tested-matrix compatibility
evidence only, not an all-plugin, 20 TPS, or 500-player claim.

Current runtime note: after rejecting the `NearbyPlayers` limit `3`
candidate and restoring `limit=2`, the rebuilt runtime passes artifact hash
verification and the real plugin matrix at `Done (27.251s)`. The matrix used
11 real plugins, installed precomputed remaps for spark, Vault, CompatProbe,
and PlaceholderAPI, loaded `LibraryProbe` through the Paper plugin-library
path, and `CompatProbe` observed plugin lifecycle, `ServerLoadEvent`,
join/quit events, sync/async scheduler ticks, and command handling
(`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery passes
at `Done (15.819s)`, and forced-ticket persistence passes with first/restart
`Done (13.055s)` / `Done (8.863s)`. This is tested-matrix compatibility
evidence only, not an all-plugin or 500-player claim.

Current runtime note: after keeping the narrow `ReferenceList.remove(...)`
transition fast path, the rebuilt runtime passes artifact hash verification
and the real plugin matrix at `Done (26.747s)`. The matrix used 11 real
plugins, installed precomputed remaps for spark, Vault, CompatProbe, and
PlaceholderAPI, loaded `LibraryProbe` through the Paper plugin-library path,
and `CompatProbe` observed plugin lifecycle, `ServerLoadEvent`, join/quit
events, sync/async scheduler ticks, and command handling
(`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery passes
at `Done (16.102s)`, and forced-ticket persistence passes with first/restart
`Done (13.346s)` / `Done (8.585s)`. This is tested-matrix compatibility
evidence only, not an all-plugin, 20 TPS, or 500-player claim.

Current runtime note: after rejecting and rolling back the temporary
`NearbyPlayers` map-capacity candidate, the rebuilt runtime passes artifact
hash verification and the real plugin matrix at `Done (26.143s)`. The matrix
used 11 real plugins, installed precomputed remaps for spark, Vault,
CompatProbe, and PlaceholderAPI, loaded `LibraryProbe` through the Paper
plugin-library path, and `CompatProbe` observed plugin lifecycle,
`ServerLoadEvent`, join/quit events, sync/async scheduler ticks, and command
handling (`COMPAT_PROBE command=ok events=4 ownServices=0`). Restart/recovery
also passes at `Done (16.191s)`, and forced-ticket persistence passes with
first/restart `Done (12.884s)` / `Done (9.473s)`. This is tested-matrix
compatibility evidence only, not an all-plugin or 500-player claim.

Current runtime note: after adding
`0047-Prune-climate-RTree-search-by-current-best-distance.patch`, the rebuilt
runtime passes artifact hash verification, the real plugin matrix at `Done
(30.298s)`, restart/recovery at `Done (22.644s)`, and forced-ticket
persistence at `18.153s` / `18.919s`. `CompatProbe` observed lifecycle,
sync/async scheduler ticks, command handling, join, and quit events, and
`LibraryProbe` still loaded its dependency through the Paper plugin library
path. The strict 50-bot 32/32 spectator gate is blocked by host preflight, so
this remains tested-matrix compatibility evidence and a narrow default
Climate RTree search work reduction only.

Current runtime note: after adding
`0046-Optimize-carver-iteration-in-chunk-generation.patch`, the rebuilt
runtime passes artifact hash verification, the real plugin matrix at `Done
(31.900s)`, restart/recovery at `Done (25.501s)`, and forced-ticket
persistence at `15.097s` / `10.702s`. `CompatProbe` observed lifecycle,
sync/async scheduler ticks, command handling, join, and quit events, and
`LibraryProbe` still loaded its dependency through the Paper plugin library
path. The strict 50-bot 32/32 spectator gate is blocked by host preflight, so
this remains tested-matrix compatibility evidence and a narrow carver
iteration allocation reduction only.

Current runtime note: after adding
`0045-Optimize-Climate-RTree-build-allocation.patch`, the rebuilt runtime
passes artifact hash verification, the real plugin matrix at `Done
(30.428s)`, restart/recovery at `Done (20.358s)`, and forced-ticket
persistence at `14.099s` / `9.797s`. `CompatProbe` observed lifecycle,
sync/async scheduler ticks, command handling, join, and quit events, and
`LibraryProbe` still loaded its dependency through the Paper plugin library
path. The strict 50-bot 32/32 spectator gate stayed stable but did not beat
the accepted load baseline, so this remains tested-matrix compatibility
evidence and a narrow startup/tree-build allocation reduction only.

Current runtime note: after wiring the existing `DensityFunction.Visitor`
holder/marker hooks into `DensityFunctions.HolderHolder.mapAll(...)` and
`DensityFunctions.MarkerOrMarked.mapAll(...)`, the rebuilt runtime passes
artifact hash verification, the real plugin matrix at `Done (38.265s)`,
restart/recovery at `Done (29.923s)`, and forced-ticket persistence at
`21.751s` / `15.859s`. `CompatProbe` still observed lifecycle, scheduler,
command, join, and quit events, and `LibraryProbe` still loaded its
dependency through the Paper plugin library path. The strict 50-bot 32/32
spectator gate is blocked by host preflight, so this is tested-matrix
compatibility evidence only, not a load-performance claim.

Current runtime note: after the `JigsawBlock.canAttach(...)` target-first
candidate, the rebuilt runtime passes artifact hash verification, the real
plugin matrix at `Done (32.953s)`, restart/recovery at `Done (18.681s)`, and
forced-ticket persistence at `14.874s` / `12.361s`. `CompatProbe` still
observed lifecycle, scheduler, command, join, and quit events, and
`LibraryProbe` still loaded its dependency through the Paper plugin library
path. The strict 50-bot 32/32 spectator gate is blocked by host preflight, so
this is tested-matrix compatibility evidence only, not a load-performance
claim.

Current runtime note: after rejecting and rolling back the
`DensityFunctions.Ap2.fillArray(ADD)` scratch-buffer candidate, the rebuilt
rollback runtime passes artifact hash verification, the real plugin matrix at
`Done (39.577s)`, restart/recovery at `Done (21.865s)`, and forced-ticket
persistence at `18.679s` / `11.969s`. `CompatProbe` still observed lifecycle,
scheduler, command, join, and quit events, and `LibraryProbe` still loaded
its dependency through the Paper plugin library path. The rejected candidate
is not in production because the clean 50-bot 32/32 spectator gate failed the
accepted performance reference.

Current runtime note: after rejecting and rolling back the temporary
`Entity.setPosRaw(...)` bounding-box shortcut, the rebuilt rollback runtime
passes artifact hash verification, the real plugin matrix at `Done
(30.454s)`, restart/recovery at `Done (19.537s)`, and forced-ticket
persistence at `19.076s` / `11.778s`. `CompatProbe` still observed lifecycle,
scheduler, command, join, and quit events, and `LibraryProbe` still loaded
its dependency through the Paper plugin library path. The rejected candidate
is not in production because the clean 50-bot 32/32 spectator gate failed the
accepted performance reference.

Current runtime note: after the POI main-thread scheduling fix and the
waypoint complete-row skip candidate, the rebuilt runtime still passes the
real plugin matrix at `Done (57.490s)`, restart/recovery at `Done
(44.201s)`, and forced-ticket persistence at `32.221s` / `20.055s`.
`CompatProbe` still observed lifecycle, scheduler, command, join, and quit
events, and `LibraryProbe` still loaded its dependency through the Paper
plugin library path. The strict 50-bot gate is still blocked by host
preflight, but the noisy 50-bot run is now stable with zero thread-check or
off-main POI failures.

Current runtime note: after rejecting and rolling back the
`ServerWaypointManager.snapshotEntries(...)` manual-copy candidate, the
production path is again `map.entrySet().toArray(Entry[]::new)`. The restored
baseline runtime passes artifact hash verification, the real plugin matrix at
`Done (29.440s)`, restart/recovery at `Done (17.388s)`, and forced-ticket
persistence at `13.805s` / `9.338s`. `CompatProbe` still observed lifecycle,
scheduler, command, join, and quit events, and `LibraryProbe` still loaded its
dependency through the Paper plugin library path. The rejected candidate is
not in production because the 50-bot 32/32 spectator gate had
`watchdog_thread_dumps=3` and did not beat the accepted reference.

Current runtime note: after the `ChunkHolderManager.getOrCreateEntityChunk(...)`
lazy-init candidate, the rebuilt runtime still passes the real plugin matrix at
`Done (30.140s)`, restart/recovery at `Done (19.105s)`, and forced-ticket
persistence at `14.037s` / `9.349s`. `CompatProbe` still observed lifecycle,
scheduler, command, join, and quit events, and `LibraryProbe` still loaded its
dependency through the Paper plugin library path. The strict 50-bot gate is
blocked by host preflight (`load_per_cpu=1.003`), so the noisy diagnostic is
the only load signal for this cycle and it is not a baseline.

Current runtime note: after rejecting and removing the temporary
`CaveWorldCarver` direct floor-skip helper patch, the rebuilt rollback runtime
still passes the real plugin matrix at `Done (27.768s)`. `CompatProbe`
observed lifecycle, scheduler, command, join, and quit events, and
`LibraryProbe` still loaded its dependency through the Paper plugin library
path. The rejected candidate is not in production because its strict 50-bot
32/32 spectator gate regressed to `17.79 TPS / 108.48 ms / 1867 chunks`
versus the accepted reference line around `18.27 TPS / 47.85 ms / 2380
chunks`.

Current runtime note: after rejecting and removing the temporary
`DensityFunctions.MarkerOrMarked.mapAll(...)` applyMarker-hook patch, the
rebuilt runtime still passes the real plugin matrix at `Done (28.435s)`,
restart/recovery at `Done (18.191s)`, and forced-ticket persistence at
`13.391s` / `9.567s`. `CompatProbe` still observed lifecycle, scheduler,
command, join, and quit events, and `LibraryProbe` still loaded its
dependency through the Paper plugin library path. The rejected candidate is
not in production because its strict 50-bot 32/32 spectator gate regressed to
`17.84 TPS / 67.37 ms / 2081 chunks` versus the accepted reference line
around `18.27 TPS / 47.85 ms / 2380 chunks`.

Current runtime note: after rejecting and removing the temporary
`BlendedNoise` octave-cache production patch, the rebuilt rollback runtime
passes the real plugin matrix at `Done (28.079s)`, restart/recovery at
`Done (17.050s)`, and forced-ticket persistence at `12.727s` / `8.805s`.
`CompatProbe` still observed lifecycle, scheduler, command, join, and quit
events, and `LibraryProbe` still loaded its dependency through the Paper
plugin library path. The rejected candidate is not in production because its
strict 50-bot 32/32 spectator gate regressed to `17.93 TPS / 56.72 ms /
2079 chunks` versus the accepted reference line around `18.27 TPS / 47.85 ms /
2380 chunks`.

Current runtime note: after rolling the rejected limit-64 `NearbyPlayers`
experiment back to the threshold-2 baseline, the rebuilt runtime still passes
the real plugin matrix at `Done (27.920s)`, restart/recovery at `Done
(19.373s)`, and forced-ticket persistence at `14.297s` / `9.507s`.
`CompatProbe` still observed lifecycle, scheduler, command, join, and quit
events, and `LibraryProbe` still loaded its dependency through the Paper
plugin library path. This is compatibility evidence only; the noisy limit-64
movement diagnostic regressed and was reverted.

Current runtime note: after the `PlacedFeature.placeWithContext(...)` traversal
rewrite, the rebuilt runtime still passes the real plugin matrix at
`Done (27.869s)`, restart/recovery at `Done (18.230s)`, and forced-ticket
persistence at `14.263s` / `9.582s`. `CompatProbe` still observed plugin
lifecycle, scheduler, command, join, and quit events in stable order, and
`LibraryProbe` still loaded its dependency through the Paper plugin library
path. This is compatibility evidence only; the strict 50-bot run completed but
was not accepted (`tps1_avg=17.71`, `avg_tick_ms_avg=42.70`,
`watchdog_thread_dumps=1`).

Current runtime note: after the spectator movement no-sync-load reset/final
snap candidate, the rebuilt runtime passes the real plugin matrix at
`Done (30.599s)`, restart/recovery at `Done (18.990s)`, and forced-ticket
persistence at `14.791s` / `10.665s`. The strict 50-bot load gate is blocked
by host preflight (`load_per_cpu=0.885` > `0.750`), so this remains
compatibility evidence only, not a 50-bot or 500-player performance claim.

Current runtime note: after the `NoiseChunk` marker wrapper cache, the rebuilt
runtime still passes the real plugin matrix at `Done (31.651s)`,
restart/recovery at `Done (18.882s)`, and forced-ticket persistence at
`15.372s` / `10.768s`. `CompatProbe` still observed plugin lifecycle,
scheduler, command, join, and quit events in order, and `LibraryProbe` still
loaded its dependency through the Paper plugin library path. This is
compatibility evidence for the tested matrix only; the strict 50-bot gate is
currently blocked by host preflight and the noisy diagnostic run is not a load
baseline.

Current runtime note: after the `OreFeature.doPlace(...)` exact loop cleanup,
the rebuilt runtime still passes the real plugin matrix at `Done (29.608s)`,
restart/recovery at `Done (17.992s)`, and forced-ticket persistence at
`14.978s` / `10.573s`. The ore-loop candidate is not in the production path
yet as an accepted load win because the strict 50-bot gate is currently
blocked by host preflight and the noisy diagnostic run is not comparable to
the accepted baseline.

Current runtime note: after the rejected `Beardifier.getBuryContribution(...)`
direct-branch candidate was reverted, the rebuilt runtime passes the real
plugin matrix again at `Done (27.842s)`, restart/recovery at `Done (17.406s)`,
and forced-ticket persistence at `14.870s` / `10.043s`. The Beardifier
candidate is not in the production path because its strict 50-bot gate did not
beat the accepted baseline (`17.97/65.67/2539` vs `18.27/47.85/2380`).

Current runtime note: after the rejected `NoiseChunk` interpolator
indexed-traversal candidate and the rejected `PalettedContainer.reencodeContents`
old-palette-id remap-cache candidate were reverted, the current runtime keeps
the earlier scratch-only `PalettedContainer.reencodeContents(...)` optimization
plus the accepted-limited plugin startup/load-order work such as
`PluginInitializerManager.load(...)` name-log aggregation and
`LegacyPluginLoadingStrategy` provided-alias reverse-index cleanup. The new
`DensityFunctions.RangeChoice.fillArray(...)` constant-out fast-path is built
and compatibility-passing, but the strict 50-bot gate is still blocked by host
preflight, so there is no fresh end-to-end load claim yet. The later
`DensityFunctions.Spline` direct-context candidate was also rejected on the
strict 50-bot gate and reverted. The real plugin matrix still passes on the
optimized runtime. This is compatibility evidence only; there is no new
load-performance claim beyond the tested matrix below.

Current post-build evidence after the `ProtoChunk.setBlockState(...)`
heightmap iterator removal:

- plugin matrix PASS at `Done (29.098s)`;
- `CompatProbe` observed `PluginEnableEvent`, `ServerLoadEvent`,
  `PlayerJoinEvent`, and `PlayerQuitEvent` in stable order;
- sync and async scheduler tasks ticked;
- `COMPAT_PROBE command=ok events=4 ownServices=0`;
- restart/recovery PASS at `Done (17.664s)`, including `Saved the game`;
- forced-ticket persistence PASS with first/restart `Done (14.544s)` /
  `Done (10.444s)`;
- strict 50-bot spectator gate blocked by host preflight
  (`load_per_cpu=0.792` > `0.750`).

Command:

```bash
MC_EULA_AGREE=true BENCHMARK_CPUSET=6-11 ./scripts/run_plugin_matrix.sh /root/rust/artifacts/optimized-runtime/run.sh
```

Latest current-source evidence after rejecting and reverting the
`PalettedContainer` remap-cache candidate:

- plugin matrix PASS at `Done (31.022s)`;
- real offline join still observed `PlayerJoinEvent detail=CodexJoinProbe`;
- sync and async scheduler tasks ticked;
- `COMPAT_PROBE command=ok events=4 ownServices=0`;
- restart/recovery PASS at `Done (20.065s)`, including `Saved the game`;
- forced-ticket persistence PASS with first/restart `Done (15.145s)` /
  `Done (10.071s)`;
- artifact hash verification PASS after refreshing generated hashes.

Previous current-source evidence after the plugin-startup name-log aggregation
rewrite and legacy provided-alias reverse-index cleanup:

- plugin matrix PASS at `Done (32.863s)`;
- real offline join still observed `PlayerJoinEvent detail=CodexJoinProbe`;
- sync and async scheduler tasks ticked;
- `COMPAT_PROBE command=ok events=4 ownServices=0`;
- restart/recovery PASS at `Done (23.341s)`, including `Saved the game`;
- forced-ticket persistence PASS with first/restart `Done (21.545s)` /
  `Done (13.276s)`;
- artifact hash verification PASS after refreshing generated hashes.

Previous current-source evidence after rejecting and reverting the Xoroshiro
direct-helper cycle:

- plugin matrix PASS at `Done (29.689s)`;
- real offline join still observed `PlayerJoinEvent detail=CodexJoinProbe`;
- sync and async scheduler tasks ticked;
- `COMPAT_PROBE command=ok events=4 ownServices=0`;
- restart/recovery PASS at `Done (19.938s)`, including `Saved the game`;
- forced-ticket persistence PASS with first/restart `Done (17.550s)` /
  `Done (11.041s)`.

Latest post-revert evidence after rejecting the `Aquifer`
surface-sampling offset candidate:

- plugin matrix PASS at `Done (32.234s)`;
- `CompatProbe` observed `PluginEnableEvent`, `ServerLoadEvent`,
  `PlayerJoinEvent`, and `PlayerQuitEvent` in stable order;
- sync and async scheduler tasks ticked;
- `COMPAT_PROBE command=ok events=4 ownServices=0`;
- `LibraryProbe` loaded `library-probe-dep.jar` through Paper plugin library
  classpath handling;
- restart/recovery PASS at `Done (20.809s)`, including `Saved the game`;
- forced-ticket persistence PASS with first/restart `Done (15.835s)` /
  `Done (11.609s)`.

Latest `ImprovedNoise.sampleAndLerp` flat-gradient build still passes this
matrix; the candidate only touches the worldgen hot path and does not change
plugin-visible semantics. The current post-build matrix run remained
`Done (32.234s)` after full build/hash verification.

Latest current-source evidence after the accepted-limited `ServerEntity`
delta-movement identity guard, accepted-limited `OwnableRewriteRule`
stream-free owner match, accepted-limited `ObfHelper` direct top-level mapping
maps plus pre-sized `StringPool`, and after the rejected `Climate.RTree`,
`ImprovedNoise.sampleAndLerp`, `NoiseChunk` empty-blender blend-cache, and
LZ4 no-outer-buffer stream-wrapper candidates were removed, with the current
`NoiseBasedChunkGenerator` primitive-cache candidate also passing the matrix:

- real plugin jars from `plugins/matrix/`;
- no mock server behavior;
- latest DirectoryStream plugin-directory scan + `DensityFunction` visitor holder/marker hooks + `PalettedContainer` reencode scratch buffer + `TopographicGraphSorter` capacity pre-size + Spigot load-after pre-size + plugin-loading allocation cleanup + Spigot load-order allocation cleanup + Paper plugin metadata dependency-cache optimization + plugin-directory scan/no-op add-plugin optimization + plugin-remapper hash-cache + exact-SHA skip-cache + batch-miss hash reuse + batch list/hash capacity hints + lazy remap-index cleanup + dirty remap-index writes + ReobfServer precomputed-server-before-mappings + atomic hard-link precomputed remap installs + direct `MessageDigest` InputStream hash + plugin-library skip-cache + deferred mappings load + `PaperReflection` stripped-method map/key/empty-descriptor reductions + waypoint azimuth/distance/inner-range + `LegacyPluginLoadingStrategy` provided-alias reverse index + `PluginInitializerManager` name-log aggregation + `ObfHelper` direct-map/StringPool + `ServerEntity` identity guard + `NoiseBasedChunkGenerator` primitive dimension cache, with current post-build matrix run `Done (31.022s)` after full build/hash verification;
- precomputed remapped plugin jars installed from mapping hash `478C3D7AE203F013AD5E055D2CF0165EC45ADD64943A054168678C09D284B223`;
- exact-SHA skip cache active for 8 plugin jars that do not require remapping;
- separate exact-SHA library skip cache active for 1 Paper plugin library jar;
- fresh `.paper-remapped/index.json`: `hashes=4`, `skippedHashes=8`;
- fresh `.paper-remapped/libraries/index.json`: `hashes=0`, `skippedHashes=1`;
- status ping protocol `773`;
- real protocol join: `CodexJoinProbe`;
- `LibraryProbe` loaded `library-probe-dep.jar` via Paper `PluginClasspathBuilder`/`JarLibrary` and logged `LIBRARY_PROBE dependency=loaded-from-plugin-library`;
- `COMPAT_PROBE event=PlayerJoinEvent sequence=3 detail=CodexJoinProbe`;
- sync and async scheduler tasks ticked;
- console commands `plugins`, `version`, `compatprobe`, `save-all flush` executed;
- `COMPAT_PROBE command=ok events=4 ownServices=0`;
- latest post-build current run passed at `Done (31.022s)`;
- restart/recovery rerun on the same matrix world passed at `Done (23.341s)`, `COMPAT_PROBE command=ok events=2 ownServices=0`, `Saved the game`, clean disable;
- forced-ticket persistence passed with first/restart `Done (21.545s)` / `Done (13.276s)`;
- default-off `network.optimize-non-flush-packet-sending` remained disabled for this matrix run;
- clean disable sequence observed.
- current run is compatibility evidence, not a startup-speed or all-plugins claim. The current strict post-revert 50-bot 32/32 rerun is blocked before Minecraft starts by host preflight (`load_per_cpu=0.807` > `0.750`), so no new final load-performance claim is made.

The earlier candidate-artifact matrix for the `SurfaceRules.SequenceRule` array
storage change also passed at `Done (33.652s)` with the same real plugin jar
set, but that candidate was later rejected on the strict 50-bot gate and
reverted. It still records the same known limitations: EssentialsX unsupported
server-version warning, WorldEdit not-tested warning, ProtocolLib
not-yet-tested warning, and Vault/LuckPerms hook usage through the test matrix.
That run is compatibility evidence only; it does not make a load-speed claim.

| Plugin | Jar | Startup | Commands | Events | Scheduler | Permissions/services | Persistence/config | Restart behavior | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LuckPerms | `LuckPerms-Bukkit-5.5.17.jar` | pass | not deeply tested | pass through join/login hooks | n/a | pass via Vault hook | H2 config/storage initialized | clean disable | PASS WITH LIMITS | deeper permission graph tests still needed |
| Vault | `Vault.jar` | pass | not deeply tested | n/a | n/a | Essentials economy + LuckPerms permission/chat hooks observed | config created | clean disable | PASS WITH LIMITS | update check reports no new version |
| PlaceholderAPI | `PlaceholderAPI-2.12.2.jar` | pass | not deeply tested | n/a | n/a | n/a | config path initialized | clean disable | PASS WITH LIMITS | `0 placeholder hook(s)` expected with no expansions |
| ProtocolLib | `ProtocolLib.jar` | pass | not deeply tested | participates in login warning | n/a | n/a | loaded | clean disable | PASS WITH LIMITS | warns MC `1.21.10` not tested by plugin |
| EssentialsX | `EssentialsX-2.21.2.jar` | pass with plugin warning | basic startup commands/providers loaded | PlayerLogin listener active | n/a | uses Vault based permissions (LuckPerms) | creates config/kits/worth/tpr/custom_items/motd | clean disable | PASS WITH LIMITS | plugin logs unsupported server version |
| WorldEdit | `worldedit-bukkit-7.4.2.jar` | pass | not deeply tested | adapter registered | n/a | WEPIF uses Vault | config written | clean disable | PASS WITH LIMITS | plugin warns this MC version is not tested |
| ViaVersion | `ViaVersion-5.9.0.jar` | pass | not deeply tested | protocol mappings loaded | scheduler shutdown observed | n/a | config initialized | clean disable | PASS WITH LIMITS | detects server version `1.21.9-1.21.10` |
| spark | `spark-1.10.172-bukkit.jar` | bundled spark used | n/a | n/a | background profiler starts | n/a | n/a | n/a | PASS WITH LIMITS | standalone jar skipped because Paper bundles spark |
| CoreProtect | `CoreProtect-CE-23.1.jar` | pass | not deeply tested | WorldEdit logging initialized | n/a | n/a | SQLite storage initialized | clean disable | PASS WITH LIMITS | data logging shutdown completes |
| Chunky | `Chunky-Bukkit-1.4.40.jar` | pass | not deeply tested | n/a | n/a | n/a | config initialized | clean disable | PASS WITH LIMITS | no generation job was started |
| CompatProbe | `CompatProbe-0.1.0.jar` | pass | `compatprobe` pass | PluginEnable, ServerLoad, PlayerJoin, PlayerQuit | sync/async pass | own services `0` | n/a | clean disable | PASS | local test probe |
| LibraryProbe | `LibraryProbe-0.1.0.jar` + `library-probe-dep.jar` | pass | n/a | n/a | n/a | n/a | n/a | clean disable | PASS | local Paper plugin-loader probe for `JarLibrary` classpath/remap path |

Known limitations:

- Plugin commands are not exhaustively tested.
- No FAWE jar was tested; WorldEdit was tested.
- This is offline-mode localhost harness for join; online authentication semantics are not tested.
