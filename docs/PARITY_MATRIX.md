# Parity Matrix

The native diagnostic surface now has a structural coverage audit:
`scripts/native_coverage_audit.py --strict-docs` reports `90` core modules,
`93` required native bench directories, `98` required scripts, `92` wrapper
files checked, `255` JNI exports checked, and `0` errors. That audit is about
modular coverage and tooling completeness, not runtime parity.

The `paper.nativeAreaMap` runtime path is source-wired through
`PaperNativeAreaMap` and `SingleUserAreaMap`. In the optimized runtime launcher
it is enabled by default when `libpaper_native_jni.so` is bundled, and it can
be disabled with `PAPER_NATIVE_AREA_MAP=false` or `PAPER_NATIVE_AREA_MAP=0`.
The Java path still falls back to Java loops if the system property or native
library is unavailable. Current `reports/native-area-map-bench.txt` evidence is
Java/native parity-clean for movement updates plus square add/remove batches:
`equivalence=PASS`, `update_native_speedup_vs_java=1.218x`,
`add_native_speedup_vs_java=1.216x`, and
`remove_native_speedup_vs_java=1.168x`. This is parity and hot-path evidence,
not an accepted load-gate result. The 2026-05-14 50-bot run with the hook
enabled still regressed the accepted baseline
(`tps1_avg=17.24`, `avg_tick_ms_avg=75.12`, `loaded_chunks_max=2766`,
`watchdog_thread_dumps=6`), and the latest preflight is blocked by host load
(`host_preflight_ok=false`, `load_per_cpu=1.946`, `idle_percent_1s=0.75`).

The new `scripts/bench_native_pack.sh` and `scripts/native_pack_report.py`
tools are orchestration/reporting layers for the same diagnostic surface. They
batch existing parity benches and summarize their outcomes. The current
`PACK_GROUPS=all` list covers all `97` real `bench_native_*.sh` scripts while
excluding only meta-runners, but it does not add runtime parity claims by
itself. The pack runner enforces that coverage contract when `all` is selected,
`scripts/native_coverage_audit.py` verifies the same real-script contract, and
`scripts/native_pack_report.py` validates declared counts, required
`pack_status`, `PACK_START`/`PACK_RESULT` set equality, manifest/group
consistency, and duplicate results. `scripts/verify_native_pack_complete.sh`
wraps the current all-pack contract, audit, report parser, hash check,
syntax checks across every `bench_native_*.sh`, and diff check.

Oracle targets:

- vanilla Minecraft Java Edition server `1.21.10`;
- stock Paper `1.21.10` build `130`.

No vanilla parity claim is made yet. The table records what is actually tested.

The 2026-05-13 native `lz4_stream_roundtrip` module is Java/native diagnostic
parity evidence for LZ4 block-stream round-trip restoration and modeled
capacity summaries, not runtime parity evidence. The bench compares Java and
Rust summaries for block sizes `32768`, `65536`, and `131072`; restored bytes
and modeled fields match and `equivalence=PASS`. Native is slower than Java
on this short run (`0.426x`, `0.404x`, `0.419x`), so no Paper runtime hook
has been installed.

The 2026-05-13 native `nbt_gzip_buffer_shape` module is Java/native
diagnostic parity evidence for NBT/GZIP buffer-shape counters, not GZIP output
parity and not runtime parity evidence. The bench compares current, gzip64k,
prebuffer64k, and both64k shapes; all summary values match and
`equivalence=PASS`. Native model counting is faster on this host (`1.735x`,
`1.830x`, `1.708x`, `1.699x`), but no Paper runtime hook has been installed.

The 2026-05-13 native `compression_threshold_shape` module is Java/native
diagnostic parity evidence for packet compression threshold/framing counters,
not zlib output parity and not runtime parity evidence. The bench compares
default and tight threshold mixes; all summary values match and
`equivalence=PASS`. Native model counting is faster on this host (`6.236x`,
`5.301x`), but no Paper runtime hook has been installed.

The 2026-05-13 native `obfhelper_maps` module is Java/native diagnostic
parity evidence for the mapping-bootstrap class/method/field map construction
and `StringPool` shapes, not runtime parity evidence.
`scripts/bench_native_obfhelper_maps.sh` compares old stream/default maps,
direct maps, and presized StringPool-backed maps against the Rust model on
the real `reobf.tiny` jar (`7554` classes, `47786` methods, `31113` fields);
all summary values match and `equivalence=PASS`. Native is slower than Java
on this host (`0.395x`, `0.398x`, `0.429x`), so no Paper runtime hook has
been installed.

The 2026-05-13 native `varint` module is Java/native diagnostic parity
evidence for VarInt and VarLong size, write-batch, and read-batch shapes, not
runtime parity evidence. `scripts/bench_native_varint.sh` compares Java and
Rust encoded bytes, decoded values, and byte-size calculations for
`1000000` VarInt values and `1000000` VarLong values; all values match and
`equivalence=PASS`. Native is slower than Java on all measured JNI shapes
(`0.340x`, `0.554x`, `0.301x`, `0.333x`, `0.638x`, `0.384x`), so no Paper
runtime hook has been installed.

The 2026-05-13 native `plugin_startup_rollup` module is Java/native diagnostic
parity evidence for combined plugin-name joining plus plugin startup log-name
aggregation, not runtime parity evidence.
`scripts/bench_native_plugin_startup_rollup.sh` compares old
`String.join(...)` + TreeSet and new manual-join + ArrayList sort/deduplicate
rollups for normal and debug delimiters; all summary values match and
`equivalence=PASS`. Native is slower than Java in absolute JNI timing on this
host, but the optimized same-runtime rollup remains faster than the old shape
(`3.065x` normal and `3.137x` debug in Java; `1.937x` normal and `1.948x`
debug in native). No Paper runtime hook has been installed.

The 2026-05-13 native `improved_noise_inline` module is Java/native
diagnostic parity evidence for old p-method, inline-byte-access,
flat-gradient, arithmetic, and switch-gradient `ImprovedNoise` sample shapes
around the live `ImprovedNoise` runtime hook, not a separate runtime hook.
`scripts/bench_native_improved_noise_inline.sh` compares all native summaries
against the Java old-p-method summary; all values match and
`equivalence=PASS`. Native is slower than Java on every measured inline shape
(`0.656x`, `0.670x`, `0.718x`, `0.732x`, `0.681x`), so this module stays
diagnostic-only.

The 2026-05-13 native `improved_noise_derivative` module is Java/native
diagnostic parity evidence for old, inline, int-table, and flat-gradient
`sampleWithDerivative` shapes around the live `ImprovedNoise` runtime hook,
not a separate runtime hook.
`scripts/bench_native_improved_noise_derivative.sh` compares native summaries
against the Java old-derivative summary on `8192` samples and `1000000`
iterations; all values match and `equivalence=PASS`. Native is faster on all
four measured shapes (`1.396x`, `1.351x`, `1.409x`, `1.423x`), so this module
stays diagnostic-only.

The 2026-05-13 native `hash_path_summary` module is Java/native diagnostic
parity evidence for SHA-256 hashing of real plugin/library jar paths, not
runtime parity evidence. `scripts/bench_native_hash_path.sh` compares Java
read-all and streaming summaries against the Rust read-all and streaming
models for `13` jars totaling `38017023` bytes; all values match and
`equivalence=PASS`. Native is slightly slower on read-all (`0.987x`) and
slower on streaming (`0.703x`), so no Paper runtime hook has been installed.

The 2026-05-13 native `nbt_compound_map_capacity` module is Java/native
diagnostic parity evidence for NBT compound-map capacity parsing over
decompressed region chunks, not runtime parity evidence.
`scripts/bench_native_nbt_compound_map_capacity.sh` compares Java and Rust
summaries for capacities `2`, `4`, `8`, and `16` on `256` chunks; all values
match and `equivalence=PASS`. Native is faster than the Java cap-8 baseline
on all tested capacities (`2.413x`, `2.648x`, `2.709x`, `2.722x`), but no
Paper runtime hook has been installed.

The 2026-05-13 native `paletted_reencode_scratch` module is Java/native
diagnostic parity evidence for old newarray, scratch-threadlocal, and direct
packed paletted reencode shapes, not runtime parity evidence.
`scripts/bench_native_paletted_reencode_scratch.sh` compares the Java and
Rust summaries; all values match and `equivalence=PASS`. Native is faster on
the old newarray shape (`2.284x`) but slower on the scratch-threadlocal and
direct-packed shapes (`0.571x`, `0.505x`), so no Paper runtime hook has been
installed.

The 2026-05-13 native `paletted_reencode_remap_cache` module is Java/native
diagnostic parity evidence for current-previous-only and cached-palette-id
paletted remap shapes, not runtime parity evidence.
`scripts/bench_native_paletted_reencode_remap_cache.sh` compares the Java
and Rust summaries; all values match and `equivalence=PASS`. Native is
slower on the current-previous-only shape (`0.737x`) and faster on the
cached-palette-id shape (`1.294x`), so no Paper runtime hook has been
installed.

The 2026-05-13 native `density_spline_context` module is Java/native
diagnostic parity evidence for old-wrapper and new-direct density-spline
context shapes, not runtime parity evidence.
`scripts/bench_native_density_spline_context.sh` compares Java and Rust
summaries; all values match and `equivalence=PASS`. Native is faster on both
measured shapes (`1.343x`, `1.328x`), but no Paper runtime hook has been
installed.

The 2026-05-13 native `density_visitor_hook` module is Java/native diagnostic
parity evidence for old unwrapping and hooked unwrapping visitor shapes, not
runtime parity evidence. `scripts/bench_native_density_visitor_hooks.sh`
compares Java and Rust summaries; all values match and `equivalence=PASS`.
Native is much faster on both measured shapes (`1782.305x`, `66.322x`), but
no Paper runtime hook has been installed.

The 2026-05-13 native `entity_chunk_transient` module is Java/native
diagnostic parity evidence for old and new mixed entity-chunk transient
shapes, not runtime parity evidence. `scripts/bench_native_entity_chunk_transient.sh`
compares Java and Rust summaries; all values match and `equivalence=PASS`.
Native is faster on both measured shapes (`14.323x`, `14.130x`), but no
Paper runtime hook has been installed.

The 2026-05-13 native `waypoint_chunk_update` module is Java/native diagnostic
parity evidence for distance-based versus chunk-long-key waypoint chunk-change
checks, not runtime parity evidence.
`scripts/bench_native_waypoint_chunk_update.sh` compares Java and Rust
summaries; all values match and `equivalence=PASS`. Java long-key checking is
faster than Java distance checking in the same runtime (`2.587x` on the
fresh short run), but native JNI is slower than Java for both measured shapes
(`0.266x`, `0.197x`). No Paper runtime hook has been installed.

The 2026-05-13 native `remapper_hash_threshold` module is Java/native
diagnostic parity evidence for plugin-remapper hash-cache build shapes, not
runtime parity evidence. `scripts/bench_native_remapper_hash_threshold.sh`
uses the existing Java remapper hash benchmark logic and real jars from
`plugins/matrix` plus `plugins/matrix-libraries`; Java/native count,
total-entry, checksum, and last-digest fields match for computeIfAbsent, put,
hybrid, and parallel modes at subset sizes `1`, `2`, `4`, `8`, and `12`.
Native is slower than Java at size `12` in the fresh short run, while native
parallel is faster than native put (`2.579x`). No Paper runtime hook has been
installed.

The 2026-05-13 native `waypoint_snapshot` module is Java/native diagnostic
parity evidence for `HashBasedTable` + `Tables.transpose(table)` snapshot
shapes, not runtime parity evidence. `scripts/bench_native_waypoint_snapshot.sh`
compares the Java toArray, sized-array, and manual snapshot shapes with the
Rust model; all values match and `equivalence=PASS`. Native is faster than
Java on all three measured shapes (`18362.610x`, `28326.422x`, `12246.901x`),
but no Paper runtime hook has been installed.

The 2026-05-13 native `waypoint_table_view` module is Java/native diagnostic
parity evidence for transposed-row versus column scan shapes over the
waypoint connection table, not runtime parity evidence.
`scripts/bench_native_waypoint_table_view.sh` compares the Java and Rust
summaries; all values match and `equivalence=PASS`. Native is faster than
Java on both measured shapes (`14612.526x`, `17070.012x`), but no Paper
runtime hook has been installed.

The 2026-05-13 native `waypoint_manager_skip` module is Java/native
diagnostic parity evidence for current/skip player and current/skip waypoint
full/partial shapes, not runtime parity evidence.
`scripts/bench_native_waypoint_manager_skip.sh` compares the Java and Rust
summaries; all values match and `equivalence=PASS`. Native is faster than
Java on all eight measured shapes (`3872.955x`, `2162.004x`, `3930.484x`,
`2412.447x`, `2649.895x`, `2225.330x`, `4522.427x`, `4337.273x`), but no
Paper runtime hook has been installed.

The 2026-05-13 native `improved_noise_floor` module is Java/native
diagnostic parity evidence for the current `Mth.floor(...)` path and the
`Math.floor(...)` shape inside `ImprovedNoise`, not runtime parity evidence.
`scripts/bench_native_improved_noise_floor.sh` compares the Java and Rust
summaries; both values match and `equivalence=PASS`. Native is slower than
Java on both measured shapes (`0.588x` current Mth floor, `0.701x` Math
floor), so this module stays diagnostic-only and does not change the live
`ImprovedNoise` runtime hook.

The 2026-05-13 native `surface_rules_sequence_array` module is Java/native
diagnostic parity evidence for list-enhanced, list-indexed, array-foreach,
and array-indexed traversal shapes, not runtime parity evidence.
`scripts/bench_native_surface_rules_sequence_array.sh` compares the Java and
Rust summaries; all values match and `equivalence=PASS`. Native is faster on
all four measured shapes (`2.456x`, `6.337x`, `1.938x`, `3.567x`), but no
Paper runtime hook has been installed.

The 2026-05-13 native `surface_rules_test_rule_state` module is Java/native
diagnostic parity evidence for old/new state-rule shapes across period-7 and
period-2 cases, not runtime parity evidence.
`scripts/bench_native_surface_rules_test_rule_state.sh` compares the Java
and Rust summaries; all values match and `equivalence=PASS`. Native is faster
than Java on every measured shape except period-2 native new-vs-old, which is
neutral/slightly slower (`0.990x`), so no Paper runtime hook has been
installed.

The 2026-05-13 native `placed_feature_traversal` module is Java/native
diagnostic parity evidence for recursive placement traversal, not runtime
parity evidence. `scripts/bench_native_placed_feature_traversal.sh` compares
the Java stream and recursive summaries against the Rust recursive model; all
values match and `equivalence=PASS`. Native is much faster than both Java
shapes on this host (`21.813x` vs Java stream, `29.905x` vs Java recursive),
but no Paper runtime hook has been installed.

The 2026-05-13 native `ore_feature_loop` module is Java/native diagnostic
parity evidence for old and optimized ore-blob inner loops, not runtime
parity evidence. `scripts/bench_native_ore_feature_loop.sh` compares the
Java and Rust checksums; all values match and `equivalence=PASS`. Native is
faster than Java on both measured shapes (`1.593x` old, `1.491x`
optimized), but no Paper runtime hook has been installed.

The 2026-05-13 native `ticketset_search` module is Java/native diagnostic
parity evidence for binary, unchecked-binary, and linear-threshold ticket
search shapes, not runtime parity evidence.
`scripts/bench_native_ticketset_search.sh` compares the Java and Rust
summary values; all values match and `equivalence=PASS`. Native is faster
than Java on all measured shapes (`3.220x`, `3.209x`, `3.608x`, `3.174x`,
`3.498x`), but no Paper runtime hook has been installed.

The 2026-05-13 native `protochunk_heightmap` module is Java/native diagnostic
parity evidence for old EnumSet foreach and cached-contains heightmap loops,
not runtime parity evidence. `scripts/bench_native_protochunk_heightmap.sh`
compares the Java and Rust summaries; all values match and `equivalence=PASS`.
Native is faster than Java on both measured shapes (`7.615x` old
EnumSet-foreach, `1.344x` cached-contains), but the cached Java loop remains
faster than Java old (`1.208x`) and native cached-contains is slower than
native old (`0.213x`). No Paper runtime hook has been installed.

The 2026-05-13 native `range_choice` module is Java/native diagnostic parity
evidence for old `fillArray(...)` and optimized constant-in/constant-out
range-choice shapes, not runtime parity evidence.
`scripts/bench_native_range_choice.sh` compares the Java and Rust
summaries across `in_constant_out_dynamic`, `in_dynamic_out_constant`,
`both_constant`, and `both_dynamic`; all summaries match and `equivalence=PASS`.
Java/native `forIndex(...)` counts match exactly. Native is faster than Java
only on some old shapes and slower on all optimized shapes, so no Paper
runtime hook has been installed.

The 2026-05-13 native `climate_parameter_distance` module is Java/native
diagnostic parity evidence for old, branch, and subtract-first climate
parameter-distance scoring, not runtime parity evidence.
`scripts/bench_native_climate_parameter_distance.sh` compares Java object
parameter batches against the Rust flat-array model; all summaries match and
`equivalence=PASS`. Native is faster than Java on all three measured shapes
(`3.124x`, `5.274x`, `3.072x`), but no Paper runtime hook has been installed.

The 2026-05-13 native `noise_generator_settings` module is Java/native
diagnostic parity evidence for holder-value, memoized-supplier,
lazy-primitive, manual-lazy-object, and cached-int access shapes, not runtime
parity evidence. `scripts/bench_native_noise_generator_settings.sh` compares
the Java generator variants with the Rust primitive-array model; all results
match and `equivalence=PASS`. Native is faster than Java on all five measured
shapes (`3.113x`, `6.056x`, `2.543x`, `3.514x`, `1.306x`), but the useful
same-runtime Java signal remains cached settings over holder-value
(`2.382x`). No Paper runtime hook has been installed.

The 2026-05-13 native `chunk_expire_count` module is Java/native diagnostic
parity evidence for dynamic and cached chunk-expire count paths, not runtime
parity evidence. `scripts/bench_native_chunk_expire_count.sh` compares all
hot/cold variants against the Rust model; all values match and
`equivalence=PASS`. Native is slower than Java on every measured shape, so no
Paper runtime hook has been installed.

The 2026-05-13 native `craftplayer_cansee` module is Java/native diagnostic
parity evidence for current, guarded, candidate, and chunk-map candidate
`CraftPlayer#canSee(...)` shapes, not runtime parity evidence.
`scripts/bench_native_craftplayer_cansee.sh` compares empty and populated
paths against the Rust model; all values match and `equivalence=PASS`. Native
is faster on every measured shape on this host, but no Paper runtime hook has
been installed.

The 2026-05-13 native `levelchunk_heightmap` module is Java/native diagnostic
parity evidence for old four-update and new combined-update heightmap shapes,
not runtime parity evidence. `scripts/bench_native_levelchunk_heightmap.sh`
compares Java and Rust summaries; all values match and `equivalence=PASS`.
Native wins on the old four-update shape (`1.484x`) but loses on the new
combined-update shape (`0.920x`), so no Paper runtime hook has been
installed.

The 2026-05-13 native `nearby_player_map_capacity` module is Java/native
diagnostic parity evidence for fastutil nearby-player-map capacity/rehash
behavior, not runtime parity evidence.
`scripts/bench_native_nearby_player_map.sh` compares the default-capacity and
presized map scenarios for 50- and 500-player benches against the Rust model;
all summaries match and `equivalence=PASS`. Native is faster than Java on
both scenarios, but this stays diagnostic only and no Paper runtime hook has
been installed.

The 2026-05-13 native `marker_cache` module is Java/native diagnostic parity
evidence for marker visitor allocation behavior, not runtime parity evidence.
`scripts/bench_native_marker_cache.sh` compares the old non-cached visitor
shape and the cached-marker visitor shape with the Rust model; all Java/native
summaries match and `equivalence=PASS`. Native wins only on the old shape and
loses on the cached shape, so no Paper runtime hook has been installed.

The 2026-05-13 native `waypoint_distance_guard` module is Java/native
diagnostic parity evidence for waypoint range and really-far distance guard
shapes, not runtime parity evidence.
`scripts/bench_native_waypoint_distance_guard.sh` compares old and guarded
range/really-far summaries with the Rust model; Java/native summaries match,
old/guarded summaries match, and `equivalence=PASS`. The guarded Java shapes
remain slower in this bench, so no Paper runtime hook has been installed.

The 2026-05-13 native `remapper_index_cleanup` module is Java/native
diagnostic parity evidence for eager remapper-index cleanup work vs the lazy
count-check path, not runtime parity evidence.
`scripts/bench_native_remapper_index_cleanup.sh` compares Java and Rust
summaries for each shape; all Java/native summaries match and
`equivalence=PASS`. Native is slower than Java on both measured shapes, so no
Paper runtime hook has been installed.

The 2026-05-13 native `remapper_skip_hashes` module is Java/native diagnostic
parity evidence for skip-hash file content parsing, not runtime parity
evidence. `scripts/bench_native_remapper_skip_hashes.sh` compares the old
stream-style parser and the direct loop parser with the Rust model; all
summaries match and `equivalence=PASS`. Native is faster than Java on this
small string parser model, but no Paper runtime hook has been installed.

The 2026-05-13 native `plugin_directory_scan` module is Java/native
diagnostic parity evidence for plugin-directory scan shapes, not runtime
parity evidence. `scripts/bench_native_plugin_directory_scan.sh` compares
`Files.walk(depth=1)`, `Files.list`, and `DirectoryStream`-style scans against
the Rust filesystem model on `/root/rust/plugins/matrix`; all summaries match
and `equivalence=PASS`. No Paper runtime hook has been installed.

The 2026-05-13 native `spigot_load_order_dependency` module is Java/native
diagnostic parity evidence for Spigot load-order dependency work, not runtime
parity evidence. `scripts/bench_native_spigot_load_order_dependency.sh`
compares default vs pre-sized loadAfter construction and old temporary
HashSet removal checks vs direct hard/soft list checks with the Rust model;
all summaries match and `equivalence=PASS`. Native wins only on the direct
removed-count shape in this synthetic bench. No Paper runtime hook has been
installed.

The 2026-05-13 native `topographic_graph_sort_capacity` module is Java/native
diagnostic parity evidence for plugin load-order topographic sort capacity
work, not runtime parity evidence.
`scripts/bench_native_topographic_graph_sort_capacity.sh` compares old
default-capacity containers and pre-sized containers with the Rust model; all
summaries match and `equivalence=PASS`, but native is slower than Java on the
measured shapes. No Paper runtime hook has been installed.

The 2026-05-13 native `plugin_loading_allocation` module is Java/native
diagnostic parity evidence for plugin-loading startup allocation shapes, not
runtime parity evidence. `scripts/bench_native_plugin_loading_allocation.sh`
compares default vs pre-sized setup collections, eager vs lazy missing-set
allocation, and eager vs lazy validate-no-miss list allocation with the Rust
model; all summaries match and `equivalence=PASS`. Native is slower than Java
in the absolute JNI bench on this host, so no Paper runtime hook has been
installed.

The 2026-05-13 native `legacy_provided_alias_removal` module is Java/native
diagnostic parity evidence for legacy provided-alias cleanup, not runtime
parity evidence. `scripts/bench_native_legacy_provided_alias_removal.sh`
compares the old `pluginsProvided.values().removeIf(...)` shape and the
reverse provided-alias index shape with the Rust model; all summaries match
and `equivalence=PASS`. Native beats old Java removeIf on this host but loses
to the already optimized Java reverse-index path, so no Paper runtime hook has
been installed.

The 2026-05-12 native `plugin_meta_dependency` module is Java/native
diagnostic parity evidence for Paper plugin metadata dependency-list
extraction, not runtime parity evidence.
`scripts/bench_native_plugin_meta_dependency.sh` compares the old stream
shape, the direct loop shape, and the cached repeated-access shape with the
Rust model; all summaries match and `equivalence=PASS`. Native beats the old
stream path on this host, but the Java loop and cached paths remain faster.
No Paper runtime hook has been installed for this module.

The 2026-05-12 native `plugin_classloader_group` module is Java/native
diagnostic parity evidence for plugin classloader-group lookup, not runtime
parity evidence. `scripts/bench_native_plugin_classloader_group.sh`
compares the old lookup shape and the requester-skip shape across miss,
hit-other, and hit-requester paths with the Rust model; all summaries match
and `equivalence=PASS`, and native is faster on five of six measured shapes
on this host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `plugin_name_join` module is Java/native diagnostic
parity evidence for plugin-name joining, not runtime parity evidence.
`scripts/bench_native_plugin_name_join.sh` compares Java `String.join(...)`
and manual `StringBuilder` sample loops with the Rust model for normal and
debug delimiters; all summaries match and `equivalence=PASS`, but every
native shape is slower than Java on the current host.

The 2026-05-12 native `plugin_name_log` module is Java/native diagnostic
parity evidence for plugin startup log-name aggregation, not runtime parity
evidence. `scripts/bench_native_plugin_name_log.sh` compares the old TreeSet
shape and the ArrayList sort/deduplicate shape with the Rust model; all
summaries match and `equivalence=PASS`, but native is slower than Java on the
current host. The Java ArrayList sort/deduplicate shape remains the useful
same-runtime result.

The 2026-05-12 native `shift_noise_direct` module is Java/native diagnostic
parity evidence for the helper/direct `ShiftNoiseDirectBench` math shapes,
not runtime parity evidence. `scripts/bench_native_shift_noise_direct.sh`
compares the Java current/default, direct/default, current-A, direct-A,
current-B, and direct-B sample loops with the Rust model; all summaries match
and `equivalence=PASS`, and the native paths are faster on the current host.
No Paper runtime hook has been installed for this module.

The 2026-05-12 native `entity_bounding_box` module is Java/native diagnostic
parity evidence for the entity bounding-box update shape, not runtime parity
evidence. `scripts/bench_native_entity_bounding_box.sh` compares Java old
`EntityDimensions.makeBoundingBox(...)` then `setBoundingBox(...)` and Java
direct dimensions-based `setBoundingBox(...)` sample loops with the Rust
model; all summaries match and `equivalence=PASS`. Native is faster than
Java on both measured shapes on the current host, but no Paper runtime hook
has been installed, and this does not restore the previously rejected
`Entity.setPosRaw(...)` bounding-box shortcut.

The 2026-05-12 native `entity_lookup_status` module is Java/native diagnostic
parity evidence for `EntityLookup.getEntityStatus(...)` status and
accessibility lookup shapes, not runtime parity evidence.
`scripts/bench_native_entity_lookup_status.sh` compares Java old status,
direct status, old accessibility, and direct accessibility sample loops with
the Rust model; all summaries match and `equivalence=PASS`, and all native
paths are faster on the current host. No Paper runtime hook has been installed
for this module, and this does not restore the previously rejected
EntityLookup runtime candidates.

The 2026-05-12 native `chunk_dependencies` module is Java/native diagnostic
parity evidence for chunk dependency radius lookup, not runtime parity
evidence. `scripts/bench_native_chunk_dependencies_array.sh` compares the old
immutable-list Java sample loop and the fixed-array Java sample loop with the
Rust models; all summaries match and `equivalence=PASS`, and both native
paths are faster on the current host. No Paper runtime hook has been installed
for this module.

The 2026-05-12 native `ownable_rule` module is Java/native diagnostic parity
evidence for descriptor owner matching, not runtime parity evidence.
`scripts/bench_native_ownable_rule.sh` compares the old stream/descriptor
conversion Java sample loop and the new direct loop Java sample loop with the
Rust models; all summaries match and `equivalence=PASS`, and both native
paths are faster on the current host. No Paper runtime hook has been installed
for this module.

The 2026-05-12 native `noisechunk_interpolator_array` module is Java/native
diagnostic parity evidence for the list, indexed-list, and array interpolator
loop shapes, not runtime parity evidence.
`scripts/bench_native_noisechunk_interpolator_array.sh` compares the Java
list/indexed-list/array sample loops with the Rust models; all summaries
match and `equivalence=PASS`, and the native paths are faster on the current
host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `noisechunk_flatcache_context` module is Java/native
diagnostic parity evidence for the old/new false-context and old/new true-
context shapes around the `NoiseChunk.FlatCache` allocation path, not runtime
parity evidence. `scripts/bench_native_noisechunk_flatcache_context.sh`
compares the Java sample loops with the Rust models; all summaries match and
`equivalence=PASS`, but the native paths are slower on this host. This does
not restore the previously rejected `NoiseChunk.FlatCache` runtime candidate.

The 2026-05-12 native `density_ap2_fill` module is Java/native diagnostic
parity evidence for the old/scratch flat and old/scratch nested
`DensityFunctions.Ap2.fillArray(ADD)` sample shapes, not runtime parity
evidence. `scripts/bench_native_density_ap2_fill.sh` compares the Java sample
loops with the Rust models; all summaries match, `equivalence=PASS`, and
`reentrant_equivalence=PASS`. This does not restore the previously rejected
runtime scratch-buffer candidate.

The 2026-05-12 native `density_ap2_minmax_fill` module is Java/native
diagnostic parity evidence for the old/new `DensityFunctions.Ap2.fillArray`
`MIN` and `MAX` sample shapes, not runtime parity evidence.
`scripts/bench_native_density_ap2_minmax_fill.sh` compares all six Java
scenarios with the Rust models; all summaries match and `equivalence=PASS`.
The Rust model also covers Java `Math.min` / `Math.max` signed-zero and `NaN`
semantics. This does not restore the previously rejected runtime fast-path
candidate.

The 2026-05-12 native `serverentity_delta_identity` module is Java/native
diagnostic parity evidence for the `ServerEntity.sendChanges()` delta-motion
old distance path and identity-guard path, not runtime parity evidence for a
native hook. `scripts/bench_native_serverentity_delta_identity.sh` compares
the Java `Vec3` sample loop with the Rust primitive-array model; all summaries
match and `equivalence=PASS`. Native wins against the old distance path but
is slower than the already optimized Java identity guard on this host.

The 2026-05-12 native `static_cache_get` module is Java/native diagnostic
parity evidence for the `StaticCache2D.get(...)` single-offset lookup
workload, not runtime parity evidence. `scripts/bench_native_static_cache_get.sh`
compares the old `contains(...)` + `getIndex(...)` Java sample loop and the
new single-offset Java sample loop with the Rust model; all summaries match
and `equivalence=PASS`, but both native shapes are slower on this host. This
does not restore the previously rejected single-offset runtime shape.

The 2026-05-12 native `cubic_spline_create` module is Java/native diagnostic
parity evidence for the `CubicSpline` create/min-max scan workload, not
runtime parity evidence. `scripts/bench_native_cubic_spline_create.sh`
compares Java iterator/index sample loops with the Rust models; all summaries
match and `equivalence=PASS`, and both native shapes are faster on the
current host. No Paper runtime hook has been installed for this module, and
this does not restore the previously rejected
`CubicSpline.Multipoint.mapAll` runtime cleanup.

The 2026-05-12 native `jigsaw_canattach` module is Java/native diagnostic
parity evidence for `JigsawBlock.canAttach(...)`, not runtime parity
evidence. `scripts/bench_native_jigsaw_canattach.sh` compares old,
optimized, and target-first Java sample loops with the Rust models; all
summaries match and `equivalence=PASS`, and all native shapes are faster on
the current host. No Paper runtime hook has been installed for this module,
and this does not restore the previously rejected target-first runtime patch.

The 2026-05-12 native `spring_feature_mutable_pos` module is Java/native
diagnostic parity evidence for the SpringFeature old `BlockPos` neighbor
checks vs mutable position reuse workload, not runtime parity evidence.
`scripts/bench_native_spring_feature_mutable_pos.sh` compares the Java
old/mutable sample loops with the Rust models; all summaries match and
`equivalence=PASS`, and both native shapes are faster on the current host.
No Paper runtime hook has been installed for this module.

The 2026-05-12 native `biome_getbiome` module is Java/native diagnostic
parity evidence for the biome corner-selection workload, not runtime parity
evidence. `scripts/bench_native_biome_getbiome.sh` compares the Java
current/optimized sample loops with the Rust models; all summaries match and
`equivalence=PASS`, and both native shapes are faster on the current host.
No Paper runtime hook has been installed for this module.

The 2026-05-12 native `beardifier_bury` module is Java/native diagnostic
parity evidence for the `Beardifier.getBuryContribution(...)` bury falloff
workload, not runtime parity evidence. `scripts/bench_native_beardifier_bury.sh`
compares the Java current/optimized sample loops with the Rust models; all
summaries match and `equivalence=PASS`, but both native shapes are slower on
the current host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `yclamped_gradient` module is Java/native diagnostic
parity evidence for the `YClampedGradient` clamped-map / inline-lerp workload,
not runtime parity evidence. `scripts/bench_native_yclamped_gradient.sh`
compares the Java current/optimized sample loops with the Rust models; all
summaries match and `equivalence=PASS`, but both native shapes are slower on
the current host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `xoroshiro_positional_direct` module is Java/native
diagnostic parity evidence for positional `XoroshiroRandomSource.nextFloat()`
and `nextDouble()` summaries, not runtime parity evidence.
`scripts/bench_native_xoroshiro_positional_direct.sh` compares Java old/direct
sample loops with the Rust models; all summaries match and `equivalence=PASS`,
and the native paths are faster on all four measured shapes on the current
host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `aquifer_positional_location` module is Java/native
diagnostic parity evidence for the Aquifer positional-random location path,
not runtime parity evidence. `scripts/bench_native_aquifer_positional_location.sh`
compares the Java old/direct sample loops with the Rust models; all summaries
match and `equivalence=PASS`. The latest same-library rerun has the native old
path faster and the native direct path slightly slower on the current host.
No Paper runtime hook has been installed for this module.

The 2026-05-12 native `aquifer_index_stride` module is Java/native diagnostic
parity evidence for the fixed-grid Aquifer cache-index stride workload, not
runtime parity evidence. `scripts/bench_native_aquifer_index_stride.sh`
compares the Java old/new sample loops with the Rust models; all summaries
match and `equivalence=PASS`, and both native shapes are faster on the current
host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `aquifer_surface_sampling` module is Java/native
diagnostic parity evidence for the Aquifer fixed-offset surface-sampling
workload, not runtime parity evidence.
`scripts/bench_native_aquifer_surface_sampling.sh` compares the Java old/new
offset loops with the Rust models; all summaries match and `equivalence=PASS`,
and both native shapes are faster on the current host. No Paper runtime hook
has been installed for this module.

The 2026-05-12 native `blended_noise` module is Java/native diagnostic parity
evidence for the synthetic BlendedNoise octave-lookup workload, not runtime
parity evidence. `scripts/bench_native_blended_noise.sh` compares the Java
old/cached loop shapes with the Rust models; all summaries match and
`equivalence=PASS`, but both native shapes are slower on the current host.
No Paper runtime hook has been installed for this module.

The 2026-05-12 native `chunk_ticket_stage` module is Java/native diagnostic
parity evidence for the primitive chunk-ticket staging map workload, not
runtime parity evidence. `scripts/bench_native_chunk_ticket_stage.sh`
compares the Java fastutil map workload with the Rust long-byte map model;
all summaries match and `equivalence=PASS`, but the native path is slower on
the current host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `improved_noise` module is Java/native diagnostic
parity evidence for the `ImprovedNoise` sample-and-lerp path around the live
`ImprovedNoise` runtime hook, not runtime parity evidence.
`scripts/bench_native_improved_noise.sh` compares the Java noise loop with the
Rust model over the same permutation and input arrays; all summaries match and
`equivalence=PASS`, and the native path is slightly faster on the current
host. This module stays diagnostic-only.

The 2026-05-12 native `perlin_noise` module is Java/native diagnostic parity
evidence for the `PerlinNoise.getValue(...)` octave loop around the live
`PerlinNoise` runtime hook, not runtime parity evidence.
`scripts/bench_native_perlin_noise.sh` compares the Java perlin loop with the
Rust model over the same permutations, amplitudes, origins, and inputs; all
summaries match and `equivalence=PASS`, and the native path is slightly
faster on the current host. This module stays diagnostic-only.

The 2026-05-27 native `normal_noise` module is Java/native diagnostic parity
evidence for the folded two-Perlin `NormalNoise` scalar and batch-fill
helpers built on top of `PaperNativePerlinNoise` handles, not runtime parity
evidence. `scripts/bench_native_normal_noise.sh` compares the Java model with
the native scalar, vertical, cell, and positions shapes; all outputs match
and `equivalence=PASS`. Native is faster on the current host across all four
shapes (`1.138x`, `1.158x`, `1.158x`, `1.114x`), but this stays diagnostic
only and no Paper runtime hook has been installed.

The 2026-05-14 native `PaperNativePerlinGetValue` export extends diagnostic
parity to the six Java `PerlinGetValueBench` shapes: delegating, direct,
direct-local, guarded direct-local, direct no-y-scale, and direct Math.floor
wrap. `scripts/bench_native_perlin_getvalue.sh` matched Java summaries for
all six variants (`equivalence=PASS`). This is not runtime parity evidence;
it is a diagnostic export, not an additional runtime hook.

The 2026-05-12 native `ticket_compare` module is Java/native diagnostic parity
evidence for ticket ordering, not runtime parity evidence.
`scripts/bench_native_ticket_compare.sh` compares Java `Ticket.compareTo`
semantics against the Rust summary model; all summaries match and
`equivalence=PASS`, but the native path is slower on the current host. No
Paper runtime hook has been installed for this module.

The 2026-05-12 native `ticket_pack` module is Java/native diagnostic parity
evidence for the persistent-ticket save path, not runtime parity evidence.
`scripts/bench_native_ticket_pack.sh` compares the Java and Rust summary
models for `TicketStorage.packTickets()`; all summaries match and
`equivalence=PASS`, but the native path is slightly slower on the current
host. No Paper runtime hook has been installed for this module.

The 2026-05-12 native `ReferenceList` module is Java/native diagnostic parity
evidence, not runtime parity evidence. `scripts/bench_native_reference_list.sh`
compares the runtime Java `ReferenceList` with the Rust integer-token model
across transition, dense, and random operation streams; all summaries match
and `equivalence=PASS`. No Paper runtime hook has been installed for this
module.

The 2026-05-11 native LZ4 module is Java block-stream interop evidence, not a
runtime parity claim. `scripts/bench_region_compression.sh` verifies that Java
streams decode through native code and native streams decode through Java code;
the selected C LZ4 backend matches Java's compressed bytes on the current
region-shaped payloads. No Paper runtime hook has been installed yet.

The current 2026-05-10 rollback runtime after rejecting the temporary
`NoiseChunk.NoiseInterpolator.compute(...)` fraction-array candidate is
compatibility-verified, but it is not new vanilla/Paper differential parity
evidence. The candidate passed a focused bit-exact benchmark, but the strict
50-bot 32/32 spectator gate regressed to `16.75/63.54/2891` with
`watchdog_thread_dumps=3` and `nearby_players_stack_hits=7`, so it was
removed from production. The restored runtime passes build/hash/plugin/
restart/forced-ticket gates, but no vanilla/Paper differential world was run
for this cycle.

The current 2026-05-10 rollback runtime after rejecting the temporary
player-loader cached-manager and `NearbyPlayers` limit64 candidates is
compatibility-verified, but it is not new vanilla/Paper differential parity
evidence. Both candidates were removed after strict 50-bot 32/32 spectator
regressions (`17.45/65.35/2412` with 4 watchdog dumps, then
`16.90/88.49/2365` with 6 watchdog dumps). The restored runtime passes
build/hash/plugin/restart/forced-ticket gates, but no vanilla/Paper
differential world was run for this cycle.

The current 2026-05-10 rollback runtime after removing the temporary
`ProtoChunk` heightmap candidate is compatibility-verified, but it is not
additional parity evidence. The fresh strict 50-bot run was stable yet not
accepted (`18.08/96.12/2609` with `watchdog_thread_dumps=3`), and no
vanilla/Paper differential world was run for this rollback.

The 2026-05-10 waypoint chunk-key update-condition candidate is not parity
evidence and is not in production. It passed a focused equivalence benchmark,
but the strict 50-bot 32/32 spectator gate regressed to
`17.99/63.66/2516` and was rolled back. The restored runtime passes
build/hash/plugin/restart/forced-ticket gates and a fresh 50-bot rollback
baseline (`18.29/50.90/2441`, no kicks/errors/watchdog/sync-load), but no
vanilla/Paper differential world was run for this candidate.

The 2026-05-10 `NearbyPlayers` limit `3` / `chunkTicketStage` capacity cycle
is not parity evidence. The strict 50-bot 32/32 spectator runs were stable,
but they did not beat the accepted reference (`18.06/46.77/2396` and
`17.83/62.80/2427` vs about `18.27/47.85/2380`), and the
`chunkTicketStage` microbench regressed on `get` and mutation
(`0.903x` / `0.983x`). The runtime was rolled back to limit `2`, and no
vanilla/Paper differential world was run for this candidate.

The 2026-05-10 `ReferenceList.remove(...)` transition fast path is not
vanilla/Paper differential parity evidence. It passed a focused equivalence
benchmark and build/hash/plugin/restart/forced-ticket gates, but no
vanilla/Paper differential world was run. The strict 50-bot 32/32 spectator
gate completed with all bots and no sync-load/NearbyPlayers stack hits, but
it was not accepted because it still had `watchdog_thread_dumps=3` and only
`18.07 TPS / 51.73 ms / 2782 chunks` versus the accepted reference around
`18.27/47.85/2380`.

The 2026-05-10 `NearbyPlayers` map-capacity candidate is not parity evidence
and is not in the production path. It passed a standalone equivalence
benchmark, but the strict 50-bot 32/32 spectator gate failed the accepted
reference (`17.95/52.03/2059` vs about `18.27/47.85/2380`) and was rolled
back. The rollback artifact passes build/hash/plugin/restart/forced-ticket
gates, but no vanilla/Paper differential world was run for this candidate.

The 2026-05-09 `JigsawBlock.canAttach(...)` target-first candidate is not
vanilla/Paper differential world evidence. It passed a standalone equivalence
benchmark across the can-attach condition (`target_first_speedup=12.354x` in
the benchmark shape) plus build/hash/plugin/restart/forced-ticket gates, but
the strict 50-bot 32/32 spectator gate is currently blocked by host preflight
(`load_per_cpu=1.187` > `0.750`) and no differential structure-generation
world was run for it.

The 2026-05-09 `DensityFunctions.Ap2.fillArray(ADD)` scratch-buffer candidate
is not parity or production-performance evidence. It passed a standalone
equivalence and reentrant-equivalence benchmark (`flat_speedup=3.536x`,
`nested_speedup=1.573x`), but the clean strict 50-bot 32/32 spectator gate
failed the accepted reference (`17.75/78.14/1933` vs about
`18.27/47.85/2380`) and was removed from production. Post-rollback
build/hash/plugin/restart/forced-ticket gates pass, but no vanilla/Paper
differential world was run for this candidate.

The 2026-05-09 POI main-thread scheduling fix is stability evidence, not
parity evidence. The noisy 50-bot run after that fix completed with
`thread_check_failures=0`, `off_main_poi_hits=0`, and `stability_failures=0`,
but no vanilla/Paper differential world was run for it and no accepted load
baseline was updated.

The 2026-05-09 `DensityFunctions.MarkerOrMarked.mapAll(...)` applyMarker-hook
candidate is not parity or production-performance evidence. It passed a
standalone equivalence benchmark (`4.982x`, marker allocations
`1920000 -> 84000`), but the strict 50-bot server gate failed the accepted
reference line (`17.84/67.37/2081` vs about `18.27/47.85/2380`) and was
removed from production. Post-rollback build/hash verification passes, but no
vanilla/Paper differential world was run for this candidate.

The 2026-05-09 `BlendedNoise` octave-cache candidate is not parity or
production-performance evidence. It passed standalone equivalence
(`cached_octaves_speedup=1.178x`), but the strict 50-bot server gate failed the
accepted reference line (`17.93/56.72/2079` vs about `18.27/47.85/2380`) and
was removed from production. Post-rollback build/hash/plugin/restart/
forced-ticket gates pass, but no vanilla/Paper differential world was run for
this candidate.

The 2026-05-09 `PlacedFeature.placeWithContext(...)` traversal rewrite is not
vanilla differential parity evidence. It passed build/hash/plugin/restart/
forced-ticket gates and a focused traversal microbench, but the strict
50-bot spectator run was not accepted (`17.71/42.70/1928`,
`watchdog_thread_dumps=1`). No vanilla/Paper differential world was run for
this candidate.

The 2026-05-09 spectator movement no-sync-load reset/final snap candidate is
not vanilla differential parity evidence. It passes build and the tested
plugin/restart/forced-ticket gates, but the strict 50-bot 32/32 spectator load
gate is blocked by host preflight (`load_per_cpu=0.885` > `0.750`), and no
vanilla/Paper differential world was run for this candidate.

The 2026-05-08 `NoiseChunk` marker wrapper cache is not vanilla differential
parity evidence. It passed a standalone equivalence/allocation benchmark
(`5.181x`, marker allocations `1920000 -> 84000`) and build/hash/plugin/restart
gates, but the strict 50-bot gate is blocked by host preflight
(`load_per_cpu=0.807` > `0.750`). The noisy 50-bot run
(`17.38/429.99/2745`) is diagnostic-only and not an accepted baseline.

The 2026-05-08 `Beardifier.getBuryContribution(...)` direct-branch candidate is
not parity or performance evidence for the production runtime. It passed a
standalone equivalence microbench (`1.176x`) but failed the strict 50-bot
accepted-baseline comparison (`17.97/65.67/2539` vs `18.27/47.85/2380`) and was
reverted. Post-revert build/hash/plugin/restart/forced-ticket gates passed, but
the post-revert 50-bot run (`16.57/112.19/3212`) is also not a new accepted load
baseline.

The current `ProtoChunk.setBlockState(...)` heightmap iterator removal is
build/plugin/restart/forced-ticket verified and microbench-positive, but the
strict 50-bot spectator gate is blocked by host preflight
(`load_per_cpu=0.792`), so it is not additional parity evidence yet.

The rejected `NoiseChunk` interpolator indexed-traversal and rejected
`Xoroshiro` positional direct-helper cycles are not parity evidence. They
passed standalone equivalence but failed the strict 50-bot accepted baseline
and were removed from production. The current `PalettedContainer`
remap-cache candidate has only microbench plus functional-gate evidence; the
strict 50-bot rebaseline is blocked by host preflight, so the accepted load
evidence remains the earlier `18.27/47.85/2380` run. A newer
`DensityFunctions.RangeChoice.fillArray(...)` constant-out fast-path is built
and compatibility-passing, but its strict 50-bot gate is also blocked by host
preflight, so it is not additional parity evidence yet.

| Area | Vanilla oracle | Paper oracle | Optimized build | Status | Evidence |
| --- | --- | --- | --- | --- | --- |
| EULA gate launch | pass | pass | pass | PASS | `./scripts/eula_gate_smoke.sh` |
| boot/status ping | measured boot | measured boot | measured boot + status | PASS | `./scripts/boot_benchmark.sh`, `plugin-matrix-status.json` |
| handshake/login/status | status only | status only | status + real offline join | PARTIAL | `mc_status_ping.py`, `minecraft-protocol` join |
| player join | not tested | not tested | pass | PARTIAL | `PlayerJoinEvent detail=CodexJoinProbe` |
| chunk load/save/generation | basic fresh generation | basic fresh generation | basic generation/save/restart | PARTIAL | boot logs, `save-all flush`, restart check |
| region compression format | vanilla default | Paper default | Java-compatible LZ4 block stream | PASS FOR PATCH | `scripts/check_region_compression.py`, `scripts/bench_region_compression.sh` |
| blocks/redstone/liquids | not run | not run | not run | NOT COMPLETE | differential world missing |
| entities/AI/pathfinding/combat | not run | not run | not run | NOT COMPLETE | differential world missing |
| inventories/crafting/loot | not run | not run | not run | NOT COMPLETE | scripted actions missing |
| dimensions/portals/teleports | basic dimensions boot | basic dimensions boot | basic dimensions boot | PARTIAL | overworld/nether/end generation logs |
| commands/command blocks | not run | basic console commands | basic console commands | PARTIAL | `plugins`, `version`, `compatprobe`, `save-all flush` |
| datapacks/registries/recipes/tags | recipe load observed | recipe load observed | recipe load observed | PARTIAL | `Loaded 1461 recipes`, `1574 advancements` |
| plugin lifecycle/events/scheduler | n/a | not tested | pass for matrix/probe | PARTIAL | latest plugin matrix `Done (31.651s)`, `CompatProbe` scheduler/event/command pass |
| crash/restart persistence | not run | not run | pass for basic restart | PARTIAL | `restart_recovery_check.sh` |
| load stability | not run | not run | 50-bot completed-delta rerun | PARTIAL | `load-50bots-20260505-noiseinterp-delta-complete-summary.txt`; `tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, `moved_too_quickly_warnings=1`, not 20 TPS stable |
| soak stability | not run | not run | not run | NOT COMPLETE | no long soak yet |

Next parity work must add differential scripted worlds for redstone/liquids/entities/inventories/datapacks and compare vanilla/Paper/optimized output with the same seed/config.

Latest accepted load rerun evidence exists separately in `reports/load-50bots-20260505-noiseinterp-delta-complete-summary.txt` (`tps1_avg=18.27`, `avg_tick_ms_avg=47.85`, `loaded_chunks_max=2380`, `moved_too_quickly_warnings=1`). Later `Climate.Node` field-cache, `CubicSpline.mapAll`, `BlendedNoise.compute`, `FindTopSurface`, `preliminarySurfaceLevel`, `PerlinNoise.activeOctaves`, `NoiseChunk.wrapLoadFactor095`, `NoiseChunk.lazyBlendCaches`, `Climate.Sampler SampleState`, unlimited chunk rates, `ImprovedNoise.gradDot`, `Mth.lerp2/lerp3`, `SurfaceRules.SequenceRule`, `PalettedContainer.reencodeContents`, and spectator no-sync-load movement experiments are recorded only as rejected evidence in `reports/load-50bots-20260507-climate-node-fields-gate-summary.txt`, `reports/load-50bots-20260507-cubicspline-mapall-gate-summary.txt`, `reports/load-50bots-20260507-blendednoise-scale-gate-summary.txt`, `reports/load-50bots-20260507-findtopsurface-threadlocal-summary.txt`, `reports/load-50bots-20260507-prelim-quart-mask-summary.txt`, `reports/load-50bots-20260507-perlin-active-octaves-summary.txt`, `reports/load-50bots-20260507-wrap-loadfactor095-summary.txt`, `reports/load-50bots-20260507-lazy-blend-cache-summary.txt`, `reports/load-50bots-20260507-climate-samplestate-gate-summary.txt`, `reports/load-50bots-20260507-rates-unlimited-summary.txt`, `reports/load-50bots-20260507-improvednoise-graddot-inline-summary.txt`, `reports/load-50bots-20260507-mth-lerp-inline-summary.txt`, `reports/load-50bots-20260507-surfacerules-sequence-index-summary.txt`, `reports/load-50bots-20260507-paletted-zero-reencode-summary.txt`, and `reports/load-50bots-20260507-spectator-nosyncload-summary.txt`. The plugin-remapper SHA cache reuse has plugin matrix evidence but does not change vanilla mechanics and is not differential parity evidence. None of these runs is vanilla parity evidence because they are not differential oracle comparisons.

The 2026-05-08 `ImprovedNoise.sampleAndLerp` inline-byte candidate is also rejected evidence, not parity evidence. It passed standalone equivalence but failed the server load baseline (`17.78/62.90/2693`) and was removed from production. The 2026-05-08 `NoiseChunk` empty-blender blend-cache allocation skip is likewise rejected evidence: standalone allocation benchmark passed (`41.207x`), but the server load gate failed the accepted baseline (`17.96/158.83/2424`) and the patch was removed. The postrevert strict 50-bot rerun completed without watchdog/sync-load (`17.79/86.26/2981`) but did not become the accepted load baseline.

The 2026-05-08 `NoiseChunk.FlatCache` context-allocation candidate is also rejected evidence, not parity evidence. It passed a standalone equivalence/allocation benchmark (`false_context_speedup=1.142x`, `24` bytes saved per false-path iteration), but the real 50-bot 32/32 gate regressed to `15.36 TPS / 254.43 ms / 1621 chunks` with `watchdog_thread_dumps=1`. The candidate was reverted, and post-revert plugin matrix/restart/forced-ticket gates passed. A clean post-revert 50-bot rerun is blocked by host preflight, while a noisy 10-bot smoke only serves as crash/regression evidence.

The 2026-05-08 `SurfaceRules.SequenceRule` runtime-array and array-indexed
loop candidates are not parity evidence either. They preserve the rule-source
codec/list contract and passed build/plugin/restart/forced-ticket gates, but no
vanilla/Paper differential world was run. The latest strict 50-bot load gate is
blocked by host preflight (`load_per_cpu=1.004` > `0.750`), and the forced
noisy 10-bot run (`18.61/243.85/2492`, no watchdog/sync-load) is diagnostic
only.

The 2026-05-13 native Rust `NoiseChunk` wrap-capacity and `Deflater` input
shape batches are parity evidence for the new Rust modules only. Both benches
passed (`equivalence=PASS`, `script_status=PASS`), but they remain diagnostic
only and are not wired into the Paper runtime. The 2026-05-14 expansion of
the wrap-capacity matrix from 10 to 13 variants is still only diagnostic
evidence; the previous strict 50-bot gate rejection continues to block a
NoiseChunk runtime capacity hook.
