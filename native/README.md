# Paper Native Modules

This workspace is the modular Rust migration layer for the Paper fork.

Rules:

- Rust modules must start as pure, parity-tested functions.
- Java/Paper runtime code must not call a Rust module until JNI overhead and
  fallback behavior are measured.
- Every module needs a Java parity bench before a production patch.
- If a Rust candidate fails strict gates, keep the diagnostic tool and roll back
  the production hook.

Current modules:

- `paper-native-chunk-encode-core` and `paper-native-chunk-encode-jni`:
  diagnostic-only chunk packet section/light encode prototype with Rust parity
  tests and a Java JNI byte-parity harness. It is not a Paper runtime hook yet;
  production use still requires a real packet golden oracle, JNI overhead
  measurement, fallback behavior, and strict server-gate evidence.
- `paper-native-core::aquifer_index_stride`: Aquifer cache-index stride
  parity model for the fixed-grid lookup workload.
- `paper-native-core::aquifer_positional_location`: Aquifer positional random
  location parity model for the `BlockPos.asLong(...)` lookup path.
- `paper-native-core::aquifer_surface_sampling`: Aquifer fixed-offset surface
  sampling parity model for the 13-sample workload.
- `paper-native-core::beardifier_bury`: Beardifier bury-contribution
  distance-falloff parity model for current vs direct arithmetic paths.
- `paper-native-core::biome_getbiome`: biome corner-selection parity model
  for the `BiomeManager.getBiome(...)` style workload.
- `paper-native-core::jigsaw_canattach`: `JigsawBlock.canAttach(...)`
  orientation, joint, and target parity model for old, optimized, and
  target-first decision shapes.
- `paper-native-core::marker_cache`: marker visitor/cache parity model for
  old marker allocation behavior vs cached-marker visitor behavior.
- `paper-native-core::nearby_player_map_capacity`: nearby-player-map capacity
  parity model for default vs presized fastutil rehash behavior.
- `paper-native-core::noise_generator_settings`: noise-generator-settings
  access parity model for holder-value, memoized-supplier, lazy-primitive,
  manual-lazy-object, and cached-int shapes.
- `paper-native-core::ore_feature_loop`: ore-blob old and optimized inner-loop
  parity model over Java-provided blob arrays.
- `paper-native-core::placed_feature_traversal`: placed-feature recursive
  traversal parity model with Java-compatible random ordering and hashing.
- `paper-native-core::spring_feature_mutable_pos`: SpringFeature neighbor
  check parity model for old `BlockPos` use vs mutable position reuse.
- `paper-native-core::xoroshiro_positional_direct`: positional Xoroshiro
  `nextFloat()` / `nextDouble()` parity model for old vs direct paths.
- `paper-native-core::yclamped_gradient`: `YClampedGradient` clamped-map /
  inline-lerp parity model for the gradient workload.
- `paper-native-core::climate`: bulk 7-parameter climate distance-sum and
  best-match helpers for node/query batches.
- `paper-native-core::climate_parameter_distance`: climate parameter-distance
  parity model for old, branch, and subtract-first distance shapes.
- `paper-native-core::climate_rtree`: pure Rust RTree build and
  current/bounded search helpers for the climate benchmark workload.
- `paper-native-core::compression`: Java-compatible LZ4 block-stream helpers
  with XXH32 checksum masking that matches `LZ4BlockOutputStream`.
- `paper-native-core::cubic_spline_create`: `CubicSpline` create/min-max scan
  parity model for iterator vs index loop shapes.
- `paper-native-core::plugin_meta_dependency`: Paper plugin metadata
  dependency-list extraction and cached repeated-access parity model for the
  required, soft, load-before, and load-after shapes.
- `paper-native-core::plugin_classloader_group`: plugin classloader-group
  lookup parity model for miss, hit-other, and hit-requester paths with the
  requester-skip lookup shape.
- `paper-native-core::plugin_directory_scan`: plugin-directory scan parity
  model for walk-depth1, list, and directory-stream scan shapes.
- `paper-native-core::plugin_loading_allocation`: plugin-loading startup
  allocation parity model for setup collection capacity, missing-set
  allocation, and validate-no-miss allocation shapes.
- `paper-native-core::legacy_provided_alias_removal`: legacy provided-alias
  cleanup parity model for old values-removeIf removal vs reverse-index
  alias removal.
- `paper-native-core::spigot_load_order_dependency`: Spigot load-order
  dependency parity model for loadAfter construction and back-reference
  removal checks.
- `paper-native-core::topographic_graph_sort_capacity`: plugin load-order
  topographic sort capacity parity model for default vs pre-sized containers.
- `paper-native-core::remapper_index_cleanup`: remapper-index cleanup parity
  model for eager cleanup work vs lazy count-check work.
- `paper-native-core::remapper_hash_threshold`: remapper hash-cache build
  parity model for computeIfAbsent, put, hybrid, and parallel jar hash modes.
- `paper-native-core::remapper_skip_hashes`: skip-hash content parser parity
  model for stream-style parsing vs direct loop parsing.
- `paper-native-core::static_cache_get`: `StaticCache2D.get(...)` parity
  model for `contains(...)` + `getIndex(...)` vs single-offset lookup.
- `paper-native-core`: pure Rust logic with unit tests.
- `paper-native-core::hash`: SHA-256 digest helper for large byte slices.
- `paper-native-core::blended_noise`: synthetic BlendedNoise octave-lookup
  parity model for old vs cached octave access.
- `paper-native-core::normal_noise`: folded two-Perlin normal-noise helpers
  and batch fill parity model built on top of `PerlinNoise` handles.
- `paper-native-core::improved_noise`: `ImprovedNoise` sample-and-lerp parity
  model for worldgen noise diagnostics.
- `paper-native-core::improved_noise_floor`: `ImprovedNoise` floor-path
  parity model for current `Mth.floor(...)` and `Math.floor(...)` shapes.
- `paper-native-core::perlin_noise`: `PerlinNoise.getValue(...)` octave-loop
  parity model built on top of `ImprovedNoise`, with separate diagnostic
  coverage for the no-y-scale 3-arg path and the legacy 6-arg path.
- `paper-native-core::varint`: VarInt / VarLong encode-decode and size helpers.
- `paper-native-core::position`: ChunkPos and SectionPos packing helpers.
- `paper-native-core::area_map`: `SingleUserAreaMap.update(...)` movement-delta
  math plus square add/remove batch helpers as a pure Rust parity module.
- `paper-native-core::reference_list`: `ReferenceList`/`IntIndexMap` parity
  model with batch summary checks for add/remove/contains/clear sequences.
- `paper-native-core::chunk_expire_count`: chunk-expire-count parity model
  for dynamic and cached count-path shapes.
- `paper-native-core::craftplayer_cansee`: `CraftPlayer#canSee(...)` parity
  model for current, guarded, candidate, and chunk-map candidate shapes.
- `paper-native-core::serverentity_delta_identity`: `ServerEntity.sendChanges()`
  delta-motion distance and identity-guard parity model.
- `paper-native-core::levelchunk_heightmap`: `LevelChunk` heightmap update
  parity model for old four-update and new combined-update shapes.
- `paper-native-core::protochunk_heightmap`: `ProtoChunk` heightmap set /
  update parity model for EnumSet foreach vs cached contains loops.
- `paper-native-core::range_choice`: range-choice fill-array parity model for
  old, constant-in, constant-out, both-constant, and both-dynamic shapes.
- `paper-native-core::surface_rules_sequence_array`: surface-rules sequence
  traversal parity model for list and array rule containers.
- `paper-native-core::surface_rules_test_rule_state`: surface-rules
  test-state parity model for old and new state-rule object shapes.
- `paper-native-core::waypoint_distance_guard`: waypoint range and
  really-far distance guard parity model for old vs guarded shapes.
- `paper-native-core::waypoint_hotpath`: waypoint hotpath parity model for
  the tight current vs guarded player/waypoint checks used in the movement
  path.
- `paper-native-core::waypoint_chunk_update`: waypoint chunk-change parity
  model for distance checks vs chunk-long-key equality checks.
- `paper-native-core::waypoint_snapshot`: waypoint `HashBasedTable`
  transposed snapshot parity model for toArray, sized-array, and manual copy
  shapes.
- `paper-native-core::waypoint_table_view`: waypoint connection table
  transposed-row vs column scan parity model.
- `paper-native-core::waypoint_manager_skip`: waypoint manager current/skip
  player and waypoint full/partial parity model.
- `paper-native-core::ticket_pack`: persistent ticket packing summary model
  for the forced-chunks save path.
- `paper-native-core::ticket_compare`: ticket ordering summary model for the
  ticket compare path.
- `paper-native-core::ticketset_search`: ticket-set search parity model for
  binary, unchecked-binary, and linear-threshold search shapes.
- `paper-native-core::chunk_ticket_stage`: primitive chunk-ticket stage map
  summary model for get and mutation sweeps.
- `paper-native-core::carver_iteration`: carver iteration parity model for
  the old foreach-style loop vs indexed loop shapes.
- `paper-native-core::cave_carver_skip`: cave-carver skip parity model for
  the current and guarded skip checks.
- `paper-native-core::deflater_input_shape`: deflater input-shape parity
  model for copied vs sliced byte-buffer input shapes.
- `paper-native-core::noise_interpolator_fractions`: noise interpolator
  fraction parity model for the cached fraction lookup shapes.
- `paper-native-core::noise_interpolator_slice`: noise interpolator
  flat-slice parity model for the interpolator slice workload.
- `paper-native-core::noisechunk_blendcache`: noisechunk blend-cache parity
  model for the cache-backed blend lookup shape.
- `paper-native-core::noisechunk_wrap_capacity`: noisechunk wrapped-map
  capacity diagnostic model for the wrapped-entry sizing sweep.
- `paper-native-jni`: JNI exports used by Java benches and future guarded hooks.

Current checkpoint:

- `jni` is pinned to `0.19.0` with `jni-sys 0.3.0` so this checkout still
  builds on `rustc 1.75.0`.
- `waypoint_chunk_update` and `remapper_hash_threshold` now have pure Rust
  parity models, JNI exports, and standalone Java/native benchmarks. The
  release native library hash is
  `6c09aeedf3a9fb96166a93d8068bf0ff4b1bc0df854519c0b1bbbe6e1c3d8fc9`.
  Both benches pass equivalence. Waypoint chunk-update shows the useful
  same-runtime Java long-key signal (`2.587x`) but native JNI is slower than
  Java on both shapes (`0.266x`, `0.197x`). Remapper hash-threshold matches
  Java summary fields over `13` real plugin/library jars and subset sizes
  `1`, `2`, `4`, `8`, and `12`; native is slower than Java at size `12`,
  while native parallel beats native put (`2.579x`). They remain diagnostic
  only.
- `waypoint_snapshot`, `waypoint_table_view`, and `waypoint_manager_skip`
  now have pure Rust parity models, JNI exports, and standalone Java/native
  benchmarks. The release native library hash is
  `d3ddef2d4224c4f35fd40a55640555f020190fc88251fcdb8bc9a130a95dc2aa`.
  All three benches pass equivalence. Snapshot native is faster than Java on
  all three shapes (`18362.610x`, `28326.422x`, `12246.901x`), table-view
  native is faster on both shapes (`14612.526x`, `17070.012x`), and
  manager-skip native is faster on all eight shapes (`3872.955x`,
  `2162.004x`, `3930.484x`, `2412.447x`, `2649.895x`, `2225.330x`,
  `4522.427x`, `4337.273x`). They remain diagnostic only.
- `improved_noise_floor`, `surface_rules_sequence_array`,
  `surface_rules_test_rule_state`, `placed_feature_traversal`,
  `ore_feature_loop`, and `ticketset_search` now have pure Rust parity
  models, JNI exports, and standalone Java/native benchmarks. The release
  native library hash is
  `6538828a942f7d1183a4cfed03d6a7dd12c85cb2af1d8aaa1e3100003a78dc1f`.
  All six benches pass equivalence. Improved-noise-floor native is slower
  than Java (`0.588x`, `0.701x`), surface-rules sequence-array native is
  faster on all four shapes (`2.456x`, `6.337x`, `1.938x`, `3.567x`),
  surface-rules test-state native is faster in the Java/native comparisons,
  placed-feature traversal native is faster than Java stream/recursive
  (`21.813x`, `29.905x`), ore-feature loop native is faster than Java
  old/optimized (`1.593x`, `1.491x`), and ticketset-search native is faster
  on all five shapes (`3.220x`, `3.209x`, `3.608x`, `3.174x`, `3.498x`).
  They remain diagnostic only.
- `protochunk_heightmap` and `range_choice` now have pure Rust parity
  models, JNI exports, and standalone Java/native benchmarks. The release
  native library hash is
  `59605ce068888933fd5d30313fd767cfcdfb9ea1729752182cae55082d3dc3a7`.
  Both benches pass equivalence. Protochunk-heightmap native beats Java on
  the old EnumSet foreach shape (`7.615x`) and the cached-contains shape
  (`1.344x`), while the range-choice native path only wins on some old
  fill-array shapes and loses on every optimized shape. Both remain
  diagnostic only.
- `climate_parameter_distance` and `noise_generator_settings` now have pure
  Rust parity models, JNI exports, and standalone Java/native benchmarks.
  The release native library hash is
  `2603ee607be92705ef1435de6ee2e5d499df6bcfdcbfb12e6ca8c7ef1815d383`.
  Both benches pass equivalence. Climate parameter-distance native is faster
  than Java on old, branch, and subtract-first shapes (`3.124x`, `5.274x`,
  `3.072x`). Noise-generator-settings native is faster than Java on all five
  shapes (`3.113x`, `6.056x`, `2.543x`, `3.514x`, `1.306x`), but the cached
  Java path still shows the strongest same-runtime improvement.
- `chunk_expire_count`, `craftplayer_cansee`, and `levelchunk_heightmap` now
  have pure Rust parity models, JNI exports, and standalone Java/native
  benchmarks. The release native library hash is
  `2603ee607be92705ef1435de6ee2e5d499df6bcfdcbfb12e6ca8c7ef1815d383`.
  All three benches pass equivalence. `chunk_expire_count` native is slower
  on every measured shape; `craftplayer_cansee` native is faster on all
  measured shapes; `levelchunk_heightmap` native wins on the old four-update
  shape but loses on the new combined-update shape. They stay diagnostic
  only.
- `marker_cache` and `waypoint_distance_guard` now have pure Rust parity
  models, JNI exports, and standalone Java/native benchmarks. The release
  native library hash is
  `b96ac2a4e7067c2453fc9ecb23f4ee12582c6f903ab394474e17225caa3efcdc`.
  Both benches pass equivalence. Marker-cache native is faster on the old
  shape (`1.311x`) but slower on the cached shape (`0.364x`), and waypoint
  native only wins slightly on guarded really-far (`1.018x`) while losing the
  other measured shapes. Both stay diagnostic only.
- `nearby_player_map_capacity` now has a pure Rust capacity/rehash model,
  JNI exports, and a standalone Java/native benchmark. The release native
  library hash is
  `0d2606fe5dc02d1b76c83816faf8b8ddd0adb6b3b0f11549c47f034c4a953d16`.
  The bench passes equivalence for both 50-player and 500-player scenarios;
  native is faster than Java on both, but the module stays diagnostic only.
- `remapper_index_cleanup`, `remapper_skip_hashes`, and
  `plugin_directory_scan` now have pure Rust parity models, JNI exports, and
  standalone Java/native benchmarks. The release native library hash is
  `e7d8db4f384fb9596bdbd97b40cceffbb26db1b4ac59dc3ea715f75fe7be7c3b`.
  All three benches pass equivalence. Remapper-index cleanup native is slower
  than Java (`0.232x` old, `0.198x` new), skip-hashes native is faster than
  Java (`2.314x` old, `2.840x` new), and plugin-directory scan native is
  faster than Java on walk/list/directory-stream shapes (`2.267x`, `1.296x`,
  `1.190x`). They stay diagnostic only.
- `spigot_load_order_dependency` now has a pure Rust load-order dependency
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The bench passes equivalence. Native loses on the loadAfter copy shapes
  (`0.112x` old, `0.116x` new), but wins on the direct removed-count shape
  (`2.341x` vs Java new), so it stays diagnostic only.
- `topographic_graph_sort_capacity` now has a pure Rust graph-sort capacity
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The bench passes equivalence. Native is slower than Java on both measured
  shapes (`0.700x` old, `0.514x` new), while pre-sizing improves Java
  `1.685x` and native `1.236x`, so it stays diagnostic only.
- `plugin_loading_allocation` now has a pure Rust plugin-loading allocation
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The bench passes equivalence. Native is slower than Java in the absolute
  JNI bench on this host, while the same native model still shows setup
  allocation reduction (`2.780x`) and missing-set reduction (`1.173x`) with
  validate neutral/slightly worse (`0.980x`), so it stays diagnostic only.
- `legacy_provided_alias_removal` now has a pure Rust provided-alias cleanup
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The bench passes equivalence. Native beats old Java removeIf (`2.130x`) but
  loses to the already optimized Java reverse-index path (`0.422x`), so it
  stays diagnostic only.
- `plugin_meta_dependency` now has a pure Rust dependency-list parity model,
  JNI batch exports, and a standalone Java/native benchmark. The bench
  passes equivalence and native is faster on the old stream path (`2.589x`)
  but slower on the new loop (`0.840x`) and cached (`0.202x`) paths on this
  host, so it stays diagnostic only.
- `plugin_classloader_group` now has a pure Rust lookup parity model, JNI
  batch exports, and a standalone Java/native benchmark. The bench passes
  equivalence and native is faster on five of six measured shapes on this
  host (`3.723x` miss old, `1.393x` miss skip, `2.418x` hit-other old,
  `0.918x` hit-other skip, `1.839x` hit-requester old, `1.314x`
  hit-requester skip), so it stays diagnostic only.
- `hash` uses `sha2 0.10.9` with the asm backend enabled, but the Java
  baseline is still faster on the current x86_64 host.
- `VarInt` parity passes, but the current batch JNI path is still slower than
  the Java baseline, so it stays as a standalone diagnostic module.
- `position` now has its own parity bench and JNI batch exports, but the direct
  Java baseline is still faster on every measured shape, including the
  combined batch path.
- `serverentity_delta_identity` now has a pure Rust delta-motion parity model,
  JNI batch exports, and a standalone Java/native benchmark against the
  `ServerEntity.sendChanges()` old distance and identity-guard paths. The
  current bench passes equivalence. Native beats the old Java distance path
  (`1.273x`) but loses to the already optimized Java identity guard (`0.944x`),
  so it stays diagnostic only and should not replace the Java runtime guard
  with JNI.
- `static_cache_get` now has a pure Rust `StaticCache2D.get(...)` parity
  model, JNI batch exports, and a standalone Java/native benchmark. The
  current batch bench passes equivalence, but native is slower on this host
  (`733.176 ms` Java old vs `944.437 ms` native old, `693.851 ms` Java new vs
  `864.624 ms` native new), so it stays diagnostic only. This does not
  restore the previously rejected single-offset runtime shape.
- `cubic_spline_create` now has a pure Rust `CubicSpline` create/min-max scan
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The current batch bench passes equivalence and native is faster on this
  host (`1.392x` iterator, `1.419x` index), so it stays diagnostic only until
  a guarded runtime hook and strict gate exist. This does not restore the
  previously rejected `CubicSpline.Multipoint.mapAll` runtime cleanup.
- `jigsaw_canattach` now has a pure Rust `JigsawBlock.canAttach(...)` parity
  model, JNI batch exports, and a standalone Java/native benchmark. The
  current batch bench passes equivalence and native is much faster on this
  host (`31.019x` old, `32.693x` optimized, `10.879x` target-first), so it
  stays diagnostic only until a guarded runtime hook and strict gate exist.
  This does not restore the previously rejected target-first runtime patch.
- `biome_getbiome` now has a pure Rust corner-selection parity model, JNI
  batch exports, and a standalone Java/native benchmark. The current batch
  bench passes equivalence and native is faster on this host (`1.151x`
  current, `1.138x` optimized), so it stays diagnostic only until a guarded
  runtime hook and strict gate exist.
- `spring_feature_mutable_pos` now has a pure Rust SpringFeature neighbor-check
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The current batch bench passes equivalence and native is faster on this
  host (`1.816x` old, `1.528x` mutable), so it stays diagnostic only until a
  guarded runtime hook and strict gate exist.
- `beardifier_bury` now has a pure Rust distance-falloff parity model, JNI
  batch exports, and a standalone Java/native benchmark. The current batch
  bench passes equivalence, but the native path is much slower on this host
  (`0.353x` current, `0.271x` optimized), so it stays diagnostic only.
- `yclamped_gradient` now has a pure Rust clamped-map parity model, JNI batch
  exports, and a standalone Java/native benchmark. The current batch bench
  passes equivalence, but the native path is much slower on this host
  (`0.454x` current, `0.435x` optimized), so it stays diagnostic only.
- `xoroshiro_positional_direct` now has a pure Rust positional Xoroshiro
  float/double parity model, JNI batch exports, and a standalone Java/native
  benchmark. The current batch bench passes equivalence and is faster than
  Java on every measured shape (`2.626x` old float, `1.434x` direct float,
  `2.727x` old double, `1.310x` direct double), so it stays diagnostic only.
- `aquifer_positional_location` now has a pure Rust positional location
  parity model, JNI batch exports, and a standalone Java/native benchmark.
  The latest rerun passes equivalence, the old path is faster than Java, and
  the direct path is slightly slower on this host (`1.456x` old, `0.972x`
  direct), so it stays diagnostic only.
- `climate` has bulk JNI parity benches for both the batch distance sum and
  best-match paths. On the current host the native path wins on both shapes:
  `44.859 ms` vs Java `198.545 ms` for the sum path and `95.798 ms` vs
  `132.167 ms` for best-match on `1024 x 8192 x 7`. It still stays diagnostic
  until there is a guarded Paper runtime use site and strict server-gate
  evidence.
- `climate_rtree` is now covered by standalone pure Rust diagnostics plus JNI
  build, search, and combined lifecycle benches. On the current host the Rust
  handle path beats the Java baseline for build, search, and
  build_search_free lifecycle workloads, and the latest runs match the Java
  input, tree, and search checksums too. The recursive search helpers now
  carry the known best distance down into child recursion and skip the second
  exact-distance pass when a child is already a leaf. Both public batch
  defaults now use the clone-backed helper; the borrowed-current,
  borrowed-bounded, direct-current, and arena paths stay diagnostic. A direct
  current specialization was tried and still does not win repeatably enough to
  become the default, and the borrowed bounded path lost its earlier edge once
  the leaf fast path landed. An owned arena variant was also tried, but on
  this host it was slower than the existing Rc-backed batch lifecycle path, so
  it stays diagnostic only.
- `area_map` now has pure Rust tests, a Java/JNI parity bench against the
  runtime `SingleUserAreaMap` movement and square-update implementations, and
  a runtime hook that the optimized launcher auto-enables when the bundled
  native library is present. The current bench passes equivalence and measures
  update `525.032 ms` Java vs `402.476 ms` native (`1.305x`), add
  `602.181 ms` vs `499.638 ms` (`1.205x`), and remove `548.668 ms` vs
  `532.226 ms` (`1.031x`) on this host. The hook still is not strict-gate
  accepted.
- `blended_noise` now has a pure Rust synthetic octave-lookup parity model,
  JNI batch exports for old/cached paths, and a standalone Java/native
  benchmark. The current batch bench passes equivalence, but both measured
  native shapes are slower than Java on this host (`0.828x` old,
  `0.865x` cached), so it stays diagnostic only.
- `aquifer_index_stride` now has a pure Rust cache-index stride parity model,
  JNI batch exports, and a standalone Java/native benchmark over the
  fixed-grid Aquifer cache walk. The current batch bench passes equivalence
  and is faster than Java on this host (`263.596 ms` vs `288.438 ms` old,
  `263.117 ms` vs `319.463 ms` new), so it stays diagnostic only.
- `aquifer_surface_sampling` now has a pure Rust fixed-offset parity model,
  JNI batch exports, and a standalone Java/native benchmark over the 13-sample
  Aquifer surface path. The current batch bench passes equivalence and is
  faster than Java on this host (`275.199 ms` vs `295.584 ms` old,
  `230.479 ms` vs `272.365 ms` new), so it stays diagnostic only.
- `reference_list` now has a pure Rust integer-token parity model, JNI batch
  summary export, and a standalone Java/native benchmark across transition,
  dense, and random scenarios. The current batch bench passes equivalence and
  shows the native batch path ahead on this host
  (`1.877x` transition, `1.536x` dense, `1.699x` random), but it stays
  diagnostic only for now.
- `ticket_pack` now has a pure Rust persistent-ticket packing parity model,
  JNI batch export, and a standalone Java/native benchmark over the forced
  ticket save path. The current batch bench passes equivalence, but the native
  summary path is slightly slower than Java on this host
  (`621.271 ms` vs `588.246 ms`), so it stays diagnostic only.
- `ticket_compare` now has a pure Rust parity model for level/type/identifier
  ticket ordering, JNI batch export, and a standalone Java/native benchmark.
  The current batch bench passes equivalence, but the native summary path is
  slower than Java on this host (`222.437 ms` vs `190.711 ms`), so it stays
  diagnostic only.
- `chunk_ticket_stage` now has a pure Rust long-byte map parity model for the
  chunk-ticket staging workload, JNI batch export, and a standalone
  Java/native benchmark. The current batch bench passes equivalence, but the
  native summary path is slower than Java on this host (`262.183 ms` vs
  `199.714 ms`), so it stays diagnostic only.
- `improved_noise` now has a pure Rust sample-and-lerp parity model, JNI batch
  export, and a standalone Java/native benchmark over the `ImprovedNoise`
  hot path. The current batch bench passes equivalence and is slightly faster
  than Java on this host (`38.572 ms` vs `42.014 ms`), but it stays diagnostic
  until a guarded Paper runtime hook and strict server gate exist.
- `perlin_noise` now has a pure Rust octave-loop parity model, JNI batch
  export, and standalone Java/native benchmark coverage over
  `PerlinNoise.getValue`. The current batch bench passes equivalence and the
  no-y-scale shape is faster than Java on this host, but it stays diagnostic
  or explicit-opt-in until a guarded Paper runtime hook and strict server gate
  prove the server profile benefits.
- `compression` now has Java/native interop coverage through the region
  compression bench. The selected C LZ4 backend matches Java's compressed
  size on the current workload and is faster (`277.301 ms` vs `321.627 ms`),
  but it remains diagnostic. The attempted stream-wrapper runtime hook was
  removed from Paper after the stream bench stayed slower than Java default
  (`4365.214 ms` vs `3292.509 ms`).
