use std::convert::TryFrom;

use jni::objects::{JClass, JString, ReleaseMode};
use jni::sys::{
    jboolean, jbooleanArray, jbyteArray, jdouble, jdoubleArray, jfloatArray, jint, jintArray, jlong, jlongArray,
    jobjectArray,
};
use jni::JNIEnv;
use paper_native_core::{
    aquifer_index_stride, aquifer_positional_location, aquifer_surface_sampling, area_map,
    beardifier_bury, biome_getbiome, blended_noise, carver_iteration, cave_carver_skip,
    chunk_expire_count, climate, climate_parameter_distance, climate_rtree,
    chunk_dependencies, chunk_ticket_stage, compression, craftplayer_cansee,
    cubic_spline_create, density_ap2_fill, density_spline_context, density_visitor_hook,
    density_ap2_minmax_fill, deflater_input_shape, entity_bounding_box, entity_chunk_transient,
    entity_lookup_status, hash, hash_path_summary, improved_noise,
    improved_noise_derivative, improved_noise_floor, improved_noise_inline,
    jigsaw_canattach, legacy_provided_alias_removal, levelchunk_heightmap, marker_cache,
    nearby_player_map_capacity, noise_generator_settings, nbt_compound_map_capacity,
    compression_threshold_shape, lz4_stream_roundtrip, nbt_gzip_buffer_shape,
    normal_noise,
    protochunk_heightmap,
    range_choice,
    obfhelper_maps,
    noise_interpolator_fractions, noise_interpolator_slice, noisechunk_blendcache, noisechunk_flatcache_context,
    noisechunk_interpolator_array, noisechunk_wrap_capacity, ore_feature_loop, ownable_rule, paletted_reencode_remap_cache,
    paletted_reencode_scratch, placed_feature_traversal, perlin_noise, plugin_classloader_group,
    plugin_directory_scan, plugin_loading_allocation, plugin_meta_dependency, plugin_name_join,
    plugin_name_log, plugin_startup_rollup, position, reference_list, remapper_index_cleanup, remapper_skip_hashes,
    serverentity_delta_identity, shift_noise_direct, spigot_load_order_dependency,
    spring_feature_mutable_pos, static_cache_get, surface_rules_sequence_array,
    surface_rules_test_rule_state, ticket_compare, ticket_pack,
    ticketset_search, topographic_graph_sort_capacity, varint, waypoint_distance_guard,
    waypoint_chunk_update, waypoint_hotpath, waypoint_manager_skip, waypoint_snapshot, waypoint_table_view,
    xoroshiro_positional_direct, yclamped_gradient,
    remapper_hash_threshold,
};

const REFERENCE_LIST_SUMMARY_FIELDS: usize = 7;

struct ClimateRTreeHandle {
    root: climate_rtree::NodeRef,
    leaves: Vec<climate_rtree::NodeRef>,
}

struct ImprovedNoiseHandle {
    noise: improved_noise::ImprovedNoise,
}

struct PerlinNoiseHandle {
    noise: perlin_noise::PerlinNoise,
}

struct ReferenceListHandle {
    list: reference_list::ReferenceList,
}

#[derive(Clone, Copy)]
enum EntityLookupStatusKind {
    OldStatus,
    DirectStatus,
    OldAccessible,
    DirectAccessible,
}

#[derive(Clone, Copy)]
enum EntityBoundingBoxKind {
    OldMakeThenSet,
    DirectDimensionsSet,
}

#[derive(Clone, Copy)]
enum ShiftNoiseDirectKind {
    CurrentDefault,
    DirectDefault,
    CurrentA,
    DirectA,
    CurrentB,
    DirectB,
}

#[derive(Clone, Copy)]
enum PluginNameJoinKind {
    StringJoin,
    ManualJoin,
}

#[derive(Clone, Copy)]
enum PluginNameLogKind {
    OldTreeset,
    NewArrayListSort,
}

#[derive(Clone, Copy)]
enum PluginStartupRollupKind {
    OldTreesetStringJoin,
    NewArrayListSortManualJoin,
}

#[derive(Clone, Copy)]
enum PluginMetaDependencyKind {
    OldStream,
    NewLoop,
    Cached,
}

#[derive(Clone, Copy)]
enum PluginClassLoaderGroupKind {
    OldLookup,
    SkipRequester,
}

#[derive(Clone, Copy)]
enum PluginLoadingAllocationKind {
    OldDefaultCapacitySetup,
    NewPresizedSetup,
    OldEagerMissingSet,
    NewLazyMissingSet,
    OldEagerValidate,
    NewLazyValidate,
}

#[derive(Clone, Copy)]
enum ObfHelperMapsKind {
    OldStreamDefault,
    DirectMaps,
    PresizedStringPool,
}

#[derive(Clone, Copy)]
enum LegacyProvidedAliasRemovalKind {
    OldValuesRemoveIf,
    NewReverseAliasRemove,
}

#[derive(Clone, Copy)]
enum SpigotLoadOrderDependencyKind {
    OldLoadAfterBuild,
    NewLoadAfterBuild,
    OldRemovedCount,
    NewRemovedCount,
}

#[derive(Clone, Copy)]
enum TopographicGraphSortCapacityKind {
    OldDefaultCapacity,
    NewPresized,
}

#[derive(Clone, Copy)]
enum RemapperIndexCleanupKind {
    OldEagerCleanup,
    NewLazyCleanup,
}

#[derive(Clone, Copy)]
enum RemapperSkipHashesKind {
    OldStream,
    NewLoop,
}

#[derive(Clone, Copy)]
enum PluginDirectoryScanKind {
    OldWalkDepth1,
    NewList,
    DirectoryStream,
}

#[derive(Clone, Copy)]
enum ChunkExpireCountKind {
    DynamicCompute,
    DynamicManual,
    CachedCompute,
    CachedHybrid,
    CachedManual,
}

#[derive(Clone, Copy)]
enum CraftPlayerCanSeeKind {
    CurrentEmpty,
    GuardedEmpty,
    CandidateEmpty,
    CurrentPopulated,
    GuardedPopulated,
    CandidatePopulated,
    ChunkMapCandidateEmpty,
    ChunkMapCandidatePopulated,
}

#[derive(Clone, Copy)]
enum LevelChunkHeightmapKind {
    OldFourUpdate,
    NewCombinedUpdate,
}

#[derive(Clone, Copy)]
enum MarkerCacheKind {
    Old,
    Cached,
}

#[derive(Clone, Copy)]
enum NearbyPlayerMapCapacityKind {
    Default,
    Presized,
}

#[derive(Clone, Copy)]
enum WaypointDistanceGuardKind {
    OldAtOrBeyondRange,
    GuardedAtOrBeyondRange,
    OldReallyFar,
    GuardedReallyFar,
}

#[derive(Clone, Copy)]
enum WaypointChunkUpdateKind {
    Distance,
    LongKey,
}

#[derive(Clone, Copy)]
enum PalettedReencodeScratchKind {
    OldNewArray,
    ScratchThreadLocal,
    DirectPacked,
}

#[derive(Clone, Copy)]
enum PalettedReencodeRemapCacheKind {
    CurrentPreviousOnly,
    CachedPaletteIds,
}

#[derive(Clone, Copy)]
enum DensitySplineContextKind {
    OldWrapper,
    NewDirect,
}

#[derive(Clone, Copy)]
enum DensityVisitorHookKind {
    OldUnwrapping,
    HookedUnwrapping,
}

#[derive(Clone, Copy)]
enum EntityChunkTransientKind {
    OldMixed,
    NewMixed,
}

#[derive(Clone, Copy)]
enum WaypointManagerSkipKind {
    CurrentPlayerFull,
    SkipPlayerFull,
    CurrentPlayerPartial,
    SkipPlayerPartial,
    CurrentWaypointFull,
    SkipWaypointFull,
    CurrentWaypointPartial,
    SkipWaypointPartial,
}

#[derive(Clone, Copy)]
enum ClimateParameterDistanceKind {
    Old,
    Branch,
    SubtractFirst,
}

#[derive(Clone, Copy)]
enum NoiseGeneratorSettingsKind {
    HolderValue,
    MemoizedSupplier,
    LazyPrimitive,
    ManualLazyObject,
    CachedInt,
}

#[derive(Clone, Copy)]
enum ProtoChunkHeightmapKind {
    OldEnumSetForeach,
    NewCachedContains,
}

#[derive(Clone, Copy)]
enum RangeChoiceKind {
    OldFillArray,
    OptimizedFillArray,
}

#[derive(Clone, Copy)]
enum ImprovedNoiseFloorKind {
    CurrentMthFloor,
    MathFloor,
}

#[derive(Clone, Copy)]
enum SurfaceRulesSequenceArrayKind {
    ListEnhanced,
    ListIndexed,
    ArrayForeach,
    ArrayIndexed,
}

#[derive(Clone, Copy)]
enum SurfaceRulesTestRuleStateKind {
    OldStateRule,
    NewStateRule,
}

#[derive(Clone, Copy)]
enum OreFeatureLoopKind {
    OldLoop,
    OptimizedLoop,
}

#[derive(Clone, Copy)]
enum TicketSetSearchKind {
    Binary,
    UncheckedBinary,
    Linear,
}

#[derive(Clone, Copy)]
enum WaypointSnapshotKind {
    ToArray,
    SizedArray,
    Manual,
}

#[derive(Clone, Copy)]
enum WaypointTableViewKind {
    TransposeRow,
    Column,
}

#[derive(Clone, Copy)]
enum RemapperHashThresholdKind {
    ComputeIfAbsent,
    Put,
    Hybrid,
    Parallel,
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePlacedFeatureTraversal_recursiveSummary(
    mut env: JNIEnv,
    _class: JClass,
    seed: jlong,
    traversals: jint,
    dst: jlongArray,
) -> jint {
    match placed_feature_traversal_summary(&mut env, seed, traversals, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeOreFeatureLoop_oldLoopSummary(
    mut env: JNIEnv,
    _class: JClass,
    center_x: jdoubleArray,
    center_y: jdoubleArray,
    center_z: jdoubleArray,
    radius: jdoubleArray,
    min_x: jintArray,
    min_y: jintArray,
    min_z: jintArray,
    max_x: jintArray,
    max_y: jintArray,
    max_z: jintArray,
    width: jint,
    height: jint,
    origin_x: jint,
    origin_y: jint,
    origin_z: jint,
    dst: jlongArray,
) -> jint {
    match ore_feature_loop_summary(
        &mut env,
        center_x,
        center_y,
        center_z,
        radius,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        width,
        height,
        origin_x,
        origin_y,
        origin_z,
        dst,
        OreFeatureLoopKind::OldLoop,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeOreFeatureLoop_optimizedLoopSummary(
    mut env: JNIEnv,
    _class: JClass,
    center_x: jdoubleArray,
    center_y: jdoubleArray,
    center_z: jdoubleArray,
    radius: jdoubleArray,
    min_x: jintArray,
    min_y: jintArray,
    min_z: jintArray,
    max_x: jintArray,
    max_y: jintArray,
    max_z: jintArray,
    width: jint,
    height: jint,
    origin_x: jint,
    origin_y: jint,
    origin_z: jint,
    dst: jlongArray,
) -> jint {
    match ore_feature_loop_summary(
        &mut env,
        center_x,
        center_y,
        center_z,
        radius,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        width,
        height,
        origin_x,
        origin_y,
        origin_z,
        dst,
        OreFeatureLoopKind::OptimizedLoop,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTicketSetSearch_binarySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match ticketset_search_summary(&mut env, iterations, 0, dst, TicketSetSearchKind::Binary) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTicketSetSearch_uncheckedBinarySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match ticketset_search_summary(
        &mut env,
        iterations,
        0,
        dst,
        TicketSetSearchKind::UncheckedBinary,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTicketSetSearch_linearSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    linear_limit: jint,
    dst: jlongArray,
) -> jint {
    match ticketset_search_summary(
        &mut env,
        iterations,
        linear_limit,
        dst,
        TicketSetSearchKind::Linear,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointSnapshot_toArraySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_snapshot_summary(&mut env, iterations, dst, WaypointSnapshotKind::ToArray) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointSnapshot_sizedArraySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_snapshot_summary(&mut env, iterations, dst, WaypointSnapshotKind::SizedArray) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointSnapshot_manualSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_snapshot_summary(&mut env, iterations, dst, WaypointSnapshotKind::Manual) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointTableView_transposeRowSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_table_view_summary(&mut env, iterations, dst, WaypointTableViewKind::TransposeRow) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointTableView_columnSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_table_view_summary(&mut env, iterations, dst, WaypointTableViewKind::Column) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointChunkUpdate_distanceSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_chunk_update_summary(
        &mut env,
        iterations,
        dst,
        WaypointChunkUpdateKind::Distance,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointChunkUpdate_longKeySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_chunk_update_summary(
        &mut env,
        iterations,
        dst,
        WaypointChunkUpdateKind::LongKey,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePalettedReencodeScratch_oldNewArraySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match paletted_reencode_scratch_summary(
        &mut env,
        iterations,
        dst,
        PalettedReencodeScratchKind::OldNewArray,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePalettedReencodeScratch_scratchThreadLocalSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match paletted_reencode_scratch_summary(
        &mut env,
        iterations,
        dst,
        PalettedReencodeScratchKind::ScratchThreadLocal,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePalettedReencodeScratch_directPackedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match paletted_reencode_scratch_summary(
        &mut env,
        iterations,
        dst,
        PalettedReencodeScratchKind::DirectPacked,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePalettedReencodeRemapCache_currentPreviousOnlySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match paletted_reencode_remap_cache_summary(
        &mut env,
        iterations,
        dst,
        PalettedReencodeRemapCacheKind::CurrentPreviousOnly,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePalettedReencodeRemapCache_cachedPaletteIdsSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match paletted_reencode_remap_cache_summary(
        &mut env,
        iterations,
        dst,
        PalettedReencodeRemapCacheKind::CachedPaletteIds,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensitySplineContext_oldWrapperSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_spline_context_summary(
        &mut env,
        iterations,
        dst,
        DensitySplineContextKind::OldWrapper,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensitySplineContext_newDirectSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_spline_context_summary(
        &mut env,
        iterations,
        dst,
        DensitySplineContextKind::NewDirect,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityVisitorHook_oldUnwrappingSummary(
    mut env: JNIEnv,
    _class: JClass,
    roots: jint,
    depth: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_visitor_hook_summary(
        &mut env,
        roots,
        depth,
        iterations,
        dst,
        DensityVisitorHookKind::OldUnwrapping,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityVisitorHook_hookedUnwrappingSummary(
    mut env: JNIEnv,
    _class: JClass,
    roots: jint,
    depth: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_visitor_hook_summary(
        &mut env,
        roots,
        depth,
        iterations,
        dst,
        DensityVisitorHookKind::HookedUnwrapping,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityChunkTransient_oldMixedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    non_transient_mask: jint,
    thread_id: jlong,
    dst: jlongArray,
) -> jint {
    match entity_chunk_transient_summary(
        &mut env,
        iterations,
        non_transient_mask,
        thread_id,
        dst,
        EntityChunkTransientKind::OldMixed,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityChunkTransient_newMixedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    non_transient_mask: jint,
    thread_id: jlong,
    dst: jlongArray,
) -> jint {
    match entity_chunk_transient_summary(
        &mut env,
        iterations,
        non_transient_mask,
        thread_id,
        dst,
        EntityChunkTransientKind::NewMixed,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperHashThreshold_computeIfAbsentSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    input_paths: jobjectArray,
    dst: jlongArray,
) -> jint {
    match remapper_hash_threshold_summary(
        &mut env,
        iterations,
        input_paths,
        dst,
        RemapperHashThresholdKind::ComputeIfAbsent,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperHashThreshold_putSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    input_paths: jobjectArray,
    dst: jlongArray,
) -> jint {
    match remapper_hash_threshold_summary(
        &mut env,
        iterations,
        input_paths,
        dst,
        RemapperHashThresholdKind::Put,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperHashThreshold_hybridSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    input_paths: jobjectArray,
    dst: jlongArray,
) -> jint {
    match remapper_hash_threshold_summary(
        &mut env,
        iterations,
        input_paths,
        dst,
        RemapperHashThresholdKind::Hybrid,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperHashThreshold_parallelSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    input_paths: jobjectArray,
    dst: jlongArray,
) -> jint {
    match remapper_hash_threshold_summary(
        &mut env,
        iterations,
        input_paths,
        dst,
        RemapperHashThresholdKind::Parallel,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_currentPlayerFullSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::CurrentPlayerFull,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_skipPlayerFullSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::SkipPlayerFull,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_currentPlayerPartialSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::CurrentPlayerPartial,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_skipPlayerPartialSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::SkipPlayerPartial,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_currentWaypointFullSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::CurrentWaypointFull,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_skipWaypointFullSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::SkipWaypointFull,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_currentWaypointPartialSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::CurrentWaypointPartial,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointManagerSkip_skipWaypointPartialSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match waypoint_manager_skip_summary(
        &mut env,
        iterations,
        dst,
        WaypointManagerSkipKind::SkipWaypointPartial,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_oldAzimuthValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::old_azimuth_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_directAzimuthValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::direct_azimuth_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_oldAtOrBeyondRangeValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::old_at_or_beyond_range_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_guardedAtOrBeyondRangeValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::guarded_at_or_beyond_range_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_oldReallyFarValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::old_really_far_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_guardedReallyFarValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::guarded_really_far_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_oldChunkVisibleValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::old_chunk_visible_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_cachedChunkVisibleValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::cached_chunk_visible_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_oldWaypointManagerValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::old_waypoint_manager_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointHotPath_optimizedWaypointManagerValue(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jdouble {
    waypoint_hotpath::optimized_waypoint_manager_value(usize::try_from(iterations).unwrap_or(0)) as jdouble
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeVarInt_byteSize(
    _env: JNIEnv,
    _class: JClass,
    value: jint,
) -> jint {
    varint::varint_size(value) as jint
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeVarInt_longByteSize(
    _env: JNIEnv,
    _class: JClass,
    value: jlong,
) -> jint {
    varint::varlong_size(value) as jint
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeVarInt_writeBatch(
    mut env: JNIEnv,
    _class: JClass,
    values: jintArray,
    dst: jbyteArray,
) -> jint {
    match write_batch(&mut env, values, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeVarInt_writeLongBatch(
    mut env: JNIEnv,
    _class: JClass,
    values: jlongArray,
    dst: jbyteArray,
) -> jint {
    match write_long_batch(&mut env, values, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeVarInt_readBatch(
    mut env: JNIEnv,
    _class: JClass,
    src: jbyteArray,
    dst: jintArray,
) -> jint {
    match read_batch(&mut env, src, dst) {
        Ok(consumed) => consumed as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeVarInt_readLongBatch(
    mut env: JNIEnv,
    _class: JClass,
    src: jbyteArray,
    dst: jlongArray,
) -> jint {
    match read_long_batch(&mut env, src, dst) {
        Ok(consumed) => consumed as jint,
        Err(code) => code,
    }
}

fn write_batch(env: &mut JNIEnv, values: jintArray, dst: jbyteArray) -> Result<usize, jint> {
    let value_count = env.get_array_length(values).map_err(|_| -1)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;

    let input = env
        .get_int_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let input = unsafe { std::slice::from_raw_parts(input.as_ptr(), value_count) };

    let required = input
        .iter()
        .fold(0usize, |total, value| total + varint::varint_size(*value));
    if required > dst_len {
        return Err(-(required as jint));
    }

    let output = env
        .get_byte_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let output = unsafe { std::slice::from_raw_parts_mut(output.as_ptr() as *mut u8, dst_len) };

    let mut offset = 0usize;
    for value in input {
        let written = varint::write_varint(*value, &mut output[offset..]).ok_or(-5)?;
        offset += written;
    }

    Ok(offset)
}

fn write_long_batch(env: &mut JNIEnv, values: jlongArray, dst: jbyteArray) -> Result<usize, jint> {
    let value_count = env.get_array_length(values).map_err(|_| -1)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;

    let input = env
        .get_long_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let input = unsafe { std::slice::from_raw_parts(input.as_ptr(), value_count) };

    let required = input
        .iter()
        .fold(0usize, |total, value| total + varint::varlong_size(*value));
    if required > dst_len {
        return Err(-(required as jint));
    }

    let output = env
        .get_byte_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let output = unsafe { std::slice::from_raw_parts_mut(output.as_ptr() as *mut u8, dst_len) };

    let mut offset = 0usize;
    for value in input {
        let written = varint::write_varlong(*value, &mut output[offset..]).ok_or(-5)?;
        offset += written;
    }

    Ok(offset)
}

fn read_batch(env: &mut JNIEnv, src: jbyteArray, dst: jintArray) -> Result<usize, jint> {
    let value_count = env.get_array_length(dst).map_err(|_| -1)? as usize;
    let src_len = env.get_array_length(src).map_err(|_| -2)? as usize;

    let input = env
        .get_byte_array_elements(src, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let input = unsafe { std::slice::from_raw_parts(input.as_ptr() as *const u8, src_len) };

    let output = env
        .get_int_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let output = unsafe { std::slice::from_raw_parts_mut(output.as_ptr(), value_count) };

    let mut offset = 0usize;
    for value in output {
        let (decoded, consumed) = varint::read_varint(&input[offset..]).map_err(|_| -5)?;
        *value = decoded;
        offset += consumed;
    }

    Ok(offset)
}

fn read_long_batch(env: &mut JNIEnv, src: jbyteArray, dst: jlongArray) -> Result<usize, jint> {
    let value_count = env.get_array_length(dst).map_err(|_| -1)? as usize;
    let src_len = env.get_array_length(src).map_err(|_| -2)? as usize;

    let input = env
        .get_byte_array_elements(src, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let input = unsafe { std::slice::from_raw_parts(input.as_ptr() as *const u8, src_len) };

    let output = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let output = unsafe { std::slice::from_raw_parts_mut(output.as_ptr(), value_count) };

    let mut offset = 0usize;
    for value in output {
        let (decoded, consumed) = varint::read_varlong(&input[offset..]).map_err(|_| -5)?;
        *value = decoded;
        offset += consumed;
    }

    Ok(offset)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeHash_sha256Digest(
    mut env: JNIEnv,
    _class: JClass,
    data: jbyteArray,
    dst: jbyteArray,
) -> jint {
    match sha256_digest(&mut env, data, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimate_nodeDistanceSumBatch(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    dst: jlongArray,
) -> jint {
    match climate_node_distance_sum_batch(&mut env, node_mins, node_maxs, queries, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimate_nodeBestMatchBatch(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> jint {
    match climate_node_best_match_batch(
        &mut env,
        node_mins,
        node_maxs,
        queries,
        best_indices,
        best_scores,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimate_nodeBestMatchUniqueBatch(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> jint {
    match climate_node_best_match_unique_batch(
        &mut env,
        node_mins,
        node_maxs,
        queries,
        best_indices,
        best_scores,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateParameterDistance_oldDistanceSum(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    dst: jlongArray,
) -> jint {
    match climate_parameter_distance_summary(
        &mut env,
        node_mins,
        node_maxs,
        queries,
        dst,
        ClimateParameterDistanceKind::Old,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateParameterDistance_branchDistanceSum(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    dst: jlongArray,
) -> jint {
    match climate_parameter_distance_summary(
        &mut env,
        node_mins,
        node_maxs,
        queries,
        dst,
        ClimateParameterDistanceKind::Branch,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateParameterDistance_subtractFirstDistanceSum(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    dst: jlongArray,
) -> jint {
    match climate_parameter_distance_summary(
        &mut env,
        node_mins,
        node_maxs,
        queries,
        dst,
        ClimateParameterDistanceKind::SubtractFirst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseGeneratorSettings_holderValueSettings(
    mut env: JNIEnv,
    _class: JClass,
    sea_levels: jintArray,
    min_ys: jintArray,
    heights: jintArray,
    iterations: jint,
    dst: jintArray,
) -> jint {
    match noise_generator_settings_summary(
        &mut env,
        sea_levels,
        min_ys,
        heights,
        iterations,
        dst,
        NoiseGeneratorSettingsKind::HolderValue,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseGeneratorSettings_memoizedSupplierSettings(
    mut env: JNIEnv,
    _class: JClass,
    sea_levels: jintArray,
    min_ys: jintArray,
    heights: jintArray,
    iterations: jint,
    dst: jintArray,
) -> jint {
    match noise_generator_settings_summary(
        &mut env,
        sea_levels,
        min_ys,
        heights,
        iterations,
        dst,
        NoiseGeneratorSettingsKind::MemoizedSupplier,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseGeneratorSettings_lazyPrimitiveSettings(
    mut env: JNIEnv,
    _class: JClass,
    sea_levels: jintArray,
    min_ys: jintArray,
    heights: jintArray,
    iterations: jint,
    dst: jintArray,
) -> jint {
    match noise_generator_settings_summary(
        &mut env,
        sea_levels,
        min_ys,
        heights,
        iterations,
        dst,
        NoiseGeneratorSettingsKind::LazyPrimitive,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseGeneratorSettings_manualLazyObjectSettings(
    mut env: JNIEnv,
    _class: JClass,
    sea_levels: jintArray,
    min_ys: jintArray,
    heights: jintArray,
    iterations: jint,
    dst: jintArray,
) -> jint {
    match noise_generator_settings_summary(
        &mut env,
        sea_levels,
        min_ys,
        heights,
        iterations,
        dst,
        NoiseGeneratorSettingsKind::ManualLazyObject,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseGeneratorSettings_cachedIntSettings(
    mut env: JNIEnv,
    _class: JClass,
    sea_levels: jintArray,
    min_ys: jintArray,
    heights: jintArray,
    iterations: jint,
    dst: jintArray,
) -> jint {
    match noise_generator_settings_summary(
        &mut env,
        sea_levels,
        min_ys,
        heights,
        iterations,
        dst,
        NoiseGeneratorSettingsKind::CachedInt,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeProtoChunkHeightmap_oldEnumSetForeachSummary(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jlong {
    protochunk_heightmap_summary(iterations, ProtoChunkHeightmapKind::OldEnumSetForeach)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeProtoChunkHeightmap_newCachedContainsSummary(
    _env: JNIEnv,
    _class: JClass,
    iterations: jint,
) -> jlong {
    protochunk_heightmap_summary(iterations, ProtoChunkHeightmapKind::NewCachedContains)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRangeChoice_oldFillArraySummary(
    mut env: JNIEnv,
    _class: JClass,
    inputs: jdoubleArray,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    scenario: jint,
    dst: jlongArray,
) -> jint {
    match range_choice_summary(
        &mut env,
        inputs,
        block_x,
        block_y,
        block_z,
        scenario,
        dst,
        RangeChoiceKind::OldFillArray,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRangeChoice_optimizedFillArraySummary(
    mut env: JNIEnv,
    _class: JClass,
    inputs: jdoubleArray,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    scenario: jint,
    dst: jlongArray,
) -> jint {
    match range_choice_summary(
        &mut env,
        inputs,
        block_x,
        block_y,
        block_z,
        scenario,
        dst,
        RangeChoiceKind::OptimizedFillArray,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseFloor_currentMthFloorSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_floor_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        ImprovedNoiseFloorKind::CurrentMthFloor,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseFloor_mathFloorSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_floor_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        ImprovedNoiseFloorKind::MathFloor,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSurfaceRulesSequenceArray_listEnhancedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    rules: jint,
    dst: jlongArray,
) -> jint {
    match surface_rules_sequence_array_summary(
        &mut env,
        iterations,
        rules,
        dst,
        SurfaceRulesSequenceArrayKind::ListEnhanced,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSurfaceRulesSequenceArray_listIndexedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    rules: jint,
    dst: jlongArray,
) -> jint {
    match surface_rules_sequence_array_summary(
        &mut env,
        iterations,
        rules,
        dst,
        SurfaceRulesSequenceArrayKind::ListIndexed,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSurfaceRulesSequenceArray_arrayForeachSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    rules: jint,
    dst: jlongArray,
) -> jint {
    match surface_rules_sequence_array_summary(
        &mut env,
        iterations,
        rules,
        dst,
        SurfaceRulesSequenceArrayKind::ArrayForeach,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSurfaceRulesSequenceArray_arrayIndexedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    rules: jint,
    dst: jlongArray,
) -> jint {
    match surface_rules_sequence_array_summary(
        &mut env,
        iterations,
        rules,
        dst,
        SurfaceRulesSequenceArrayKind::ArrayIndexed,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSurfaceRulesTestRuleState_oldStateRuleSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    period: jint,
    dst: jlongArray,
) -> jint {
    match surface_rules_test_rule_state_summary(
        &mut env,
        iterations,
        period,
        dst,
        SurfaceRulesTestRuleStateKind::OldStateRule,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSurfaceRulesTestRuleState_newStateRuleSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    period: jint,
    dst: jlongArray,
) -> jint {
    match surface_rules_test_rule_state_summary(
        &mut env,
        iterations,
        period,
        dst,
        SurfaceRulesTestRuleStateKind::NewStateRule,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimate_nodeBestMatchUniqueBatch(
    env: JNIEnv,
    class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> jint {
    Java_PaperNativeClimate_nodeBestMatchUniqueBatch(env, class, node_mins, node_maxs, queries, best_indices, best_scores)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_buildTreeHandle(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
) -> jlong {
    climate_rtree_build_tree_handle(&mut env, node_mins, node_maxs).unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_freeTreeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    climate_rtree_free_tree_handle(handle);
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_checksumTreeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    climate_rtree_checksum_tree_handle(handle)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimateRTree_nativeBuildTreeHandle(
    mut env: JNIEnv,
    _class: JClass,
    node_mins: jlongArray,
    node_maxs: jlongArray,
) -> jlong {
    climate_rtree_build_tree_handle(&mut env, node_mins, node_maxs).unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimateRTree_nativeFreeTreeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    climate_rtree_free_tree_handle(handle);
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimateRTree_nativeChecksumTreeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    climate_rtree_checksum_tree_handle(handle)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimateRTree_nativeSearchCurrentOnePacked(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    value0: jlong,
    value1: jlong,
    value2: jlong,
    value3: jlong,
    value4: jlong,
    value5: jlong,
    value6: jlong,
    previous_index: jint,
) -> jint {
    match climate_rtree_search_current_one_packed(
        handle,
        [value0, value1, value2, value3, value4, value5, value6],
        previous_index,
    ) {
        Ok(index) => index,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimateRTree_nativeSearchBoundedOnePacked(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    value0: jlong,
    value1: jlong,
    value2: jlong,
    value3: jlong,
    value4: jlong,
    value5: jlong,
    value6: jlong,
    previous_index: jint,
) -> jint {
    match climate_rtree_search_bounded_one_packed(
        handle,
        [value0, value1, value2, value3, value4, value5, value6],
        previous_index,
    ) {
        Ok(index) => index,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_searchCurrentOnePacked(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    value0: jlong,
    value1: jlong,
    value2: jlong,
    value3: jlong,
    value4: jlong,
    value5: jlong,
    value6: jlong,
    previous_index: jint,
) -> jint {
    match climate_rtree_search_current_one_packed(
        handle,
        [value0, value1, value2, value3, value4, value5, value6],
        previous_index,
    ) {
        Ok(index) => index,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_searchBoundedOnePacked(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    value0: jlong,
    value1: jlong,
    value2: jlong,
    value3: jlong,
    value4: jlong,
    value5: jlong,
    value6: jlong,
    previous_index: jint,
) -> jint {
    match climate_rtree_search_bounded_one_packed(
        handle,
        [value0, value1, value2, value3, value4, value5, value6],
        previous_index,
    ) {
        Ok(index) => index,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_biome_PaperNativeClimateRTree_nativeSearchBoundedBatchPacked(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    queries: jlongArray,
    previous_index: jint,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> jint {
    if handle == 0 {
        return -1;
    }

    let queries_len = match env.get_array_length(queries) {
        Ok(len) => len as usize,
        Err(_) => return -2,
    };
    if queries_len % climate_rtree::PARAMETER_COUNT != 0 {
        return -3;
    }

    let query_count = queries_len / climate_rtree::PARAMETER_COUNT;
    let best_indices_len = match env.get_array_length(best_indices) {
        Ok(len) => len as usize,
        Err(_) => return -4,
    };
    if best_indices_len < query_count {
        return -(query_count as jint);
    }
    let best_scores_len = match env.get_array_length(best_scores) {
        Ok(len) => len as usize,
        Err(_) => return -5,
    };
    if best_scores_len < query_count {
        return -(query_count as jint);
    }

    let queries_elements = match env.get_long_array_elements(queries, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -6,
    };
    let best_indices_elements = match env.get_int_array_elements(best_indices, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -7,
    };
    let best_scores_elements = match env.get_long_array_elements(best_scores, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -8,
    };

    let queries = unsafe { std::slice::from_raw_parts(queries_elements.as_ptr() as *const i64, queries_len) };
    let best_indices = unsafe {
        std::slice::from_raw_parts_mut(best_indices_elements.as_ptr() as *mut i32, best_indices_len)
    };
    let best_scores = unsafe {
        std::slice::from_raw_parts_mut(best_scores_elements.as_ptr() as *mut i64, best_scores_len)
    };
    let handle = unsafe { &*(handle as *const ClimateRTreeHandle) };

    match climate_rtree::search_bounded_index_batch_borrowed(
        &handle.root,
        &handle.leaves,
        queries,
        previous_index,
        best_indices,
        best_scores,
    ) {
        Ok(written) => written as jint,
        Err(code) => match code {
            climate_rtree::ClimateRTreeBatchError::InvalidInputLength => -9,
            climate_rtree::ClimateRTreeBatchError::OutputTooSmall(required) => -(required as jint),
        },
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_searchCurrentBatch(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> jint {
    match climate_rtree_search_batch(
        &mut env,
        handle,
        queries,
        best_indices,
        best_scores,
        climate_rtree::search_current_batch,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeClimateRTree_searchBoundedBatch(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> jint {
    match climate_rtree_search_batch(
        &mut env,
        handle,
        queries,
        best_indices,
        best_scores,
        climate_rtree::search_bounded_batch_borrowed,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

fn sha256_digest(env: &mut JNIEnv, data: jbyteArray, dst: jbyteArray) -> Result<usize, jint> {
    let data_len = env.get_array_length(data).map_err(|_| -1)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < hash::SHA256_DIGEST_LEN {
        return Err(-(hash::SHA256_DIGEST_LEN as jint));
    }

    let data_elements = env
        .get_byte_array_elements(data, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let dst_elements = env
        .get_byte_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;

    let data = unsafe { std::slice::from_raw_parts(data_elements.as_ptr() as *const u8, data_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut u8, hash::SHA256_DIGEST_LEN)
    };

    hash::sha256_digest_into(data, dst).ok_or(-5)
}

fn climate_node_distance_sum_batch(
    env: &mut JNIEnv,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let node_mins_len = env.get_array_length(node_mins).map_err(|_| -1)? as usize;
    let node_maxs_len = env.get_array_length(node_maxs).map_err(|_| -2)? as usize;
    if node_mins_len != node_maxs_len {
        return Err(-3);
    }

    if node_mins_len % climate::PARAMETER_COUNT != 0 {
        return Err(-4);
    }

    let queries_len = env.get_array_length(queries).map_err(|_| -5)? as usize;
    if queries_len % climate::PARAMETER_COUNT != 0 {
        return Err(-6);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -7)? as usize;
    if dst_len < 1 {
        return Err(-8);
    }

    let node_mins_elements = env
        .get_long_array_elements(node_mins, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let node_maxs_elements = env
        .get_long_array_elements(node_maxs, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let queries_elements = env
        .get_long_array_elements(queries, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;

    let node_mins = unsafe {
        std::slice::from_raw_parts(node_mins_elements.as_ptr() as *const i64, node_mins_len)
    };
    let node_maxs = unsafe {
        std::slice::from_raw_parts(node_maxs_elements.as_ptr() as *const i64, node_maxs_len)
    };
    let queries = unsafe {
        std::slice::from_raw_parts(queries_elements.as_ptr() as *const i64, queries_len)
    };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, dst_len) };

    climate::node_distance_sum_batch(node_mins, node_maxs, queries, dst)
        .map_err(|code| match code {
            climate::ClimateError::InvalidInputLength => -13,
            climate::ClimateError::OutputTooSmall(required) => -(required as jint),
        })
}

fn climate_node_best_match_batch(
    env: &mut JNIEnv,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> Result<usize, jint> {
    let node_mins_len = env.get_array_length(node_mins).map_err(|_| -1)? as usize;
    let node_maxs_len = env.get_array_length(node_maxs).map_err(|_| -2)? as usize;
    if node_mins_len != node_maxs_len {
        return Err(-3);
    }

    if node_mins_len == 0 || node_mins_len % climate::PARAMETER_COUNT != 0 {
        return Err(-4);
    }

    let queries_len = env.get_array_length(queries).map_err(|_| -5)? as usize;
    if queries_len % climate::PARAMETER_COUNT != 0 {
        return Err(-6);
    }

    let query_count = queries_len / climate::PARAMETER_COUNT;
    let best_indices_len = env.get_array_length(best_indices).map_err(|_| -7)? as usize;
    if best_indices_len < query_count {
        return Err(-(query_count as jint));
    }

    let best_scores_len = env.get_array_length(best_scores).map_err(|_| -8)? as usize;
    if best_scores_len < query_count {
        return Err(-(query_count as jint));
    }

    let node_mins_elements = env
        .get_long_array_elements(node_mins, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let node_maxs_elements = env
        .get_long_array_elements(node_maxs, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let queries_elements = env
        .get_long_array_elements(queries, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let best_indices_elements = env
        .get_int_array_elements(best_indices, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;
    let best_scores_elements = env
        .get_long_array_elements(best_scores, ReleaseMode::CopyBack)
        .map_err(|_| -13)?;

    let node_mins = unsafe {
        std::slice::from_raw_parts(node_mins_elements.as_ptr() as *const i64, node_mins_len)
    };
    let node_maxs = unsafe {
        std::slice::from_raw_parts(node_maxs_elements.as_ptr() as *const i64, node_maxs_len)
    };
    let queries = unsafe {
        std::slice::from_raw_parts(queries_elements.as_ptr() as *const i64, queries_len)
    };
    let best_indices = unsafe {
        std::slice::from_raw_parts_mut(best_indices_elements.as_ptr() as *mut i32, best_indices_len)
    };
    let best_scores = unsafe {
        std::slice::from_raw_parts_mut(best_scores_elements.as_ptr() as *mut i64, best_scores_len)
    };

    climate::node_best_match_batch(node_mins, node_maxs, queries, best_indices, best_scores)
        .map_err(|code| match code {
        climate::ClimateError::InvalidInputLength => -14,
        climate::ClimateError::OutputTooSmall(required) => -(required as jint),
    })
}

fn climate_node_best_match_unique_batch(
    env: &mut JNIEnv,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
) -> Result<usize, jint> {
    let node_mins_len = env.get_array_length(node_mins).map_err(|_| -1)? as usize;
    let node_maxs_len = env.get_array_length(node_maxs).map_err(|_| -2)? as usize;
    if node_mins_len != node_maxs_len {
        return Err(-3);
    }

    if node_mins_len == 0 || node_mins_len % climate::PARAMETER_COUNT != 0 {
        return Err(-4);
    }

    let queries_len = env.get_array_length(queries).map_err(|_| -5)? as usize;
    if queries_len % climate::PARAMETER_COUNT != 0 {
        return Err(-6);
    }

    let query_count = queries_len / climate::PARAMETER_COUNT;
    let best_indices_len = env.get_array_length(best_indices).map_err(|_| -7)? as usize;
    if best_indices_len < query_count {
        return Err(-(query_count as jint));
    }

    let best_scores_len = env.get_array_length(best_scores).map_err(|_| -8)? as usize;
    if best_scores_len < query_count {
        return Err(-(query_count as jint));
    }

    let node_mins_elements = env
        .get_long_array_elements(node_mins, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let node_maxs_elements = env
        .get_long_array_elements(node_maxs, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let queries_elements = env
        .get_long_array_elements(queries, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let best_indices_elements = env
        .get_int_array_elements(best_indices, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;
    let best_scores_elements = env
        .get_long_array_elements(best_scores, ReleaseMode::CopyBack)
        .map_err(|_| -13)?;

    let node_mins = unsafe {
        std::slice::from_raw_parts(node_mins_elements.as_ptr() as *const i64, node_mins_len)
    };
    let node_maxs = unsafe {
        std::slice::from_raw_parts(node_maxs_elements.as_ptr() as *const i64, node_maxs_len)
    };
    let queries = unsafe {
        std::slice::from_raw_parts(queries_elements.as_ptr() as *const i64, queries_len)
    };
    let best_indices = unsafe {
        std::slice::from_raw_parts_mut(best_indices_elements.as_ptr() as *mut i32, best_indices_len)
    };
    let best_scores = unsafe {
        std::slice::from_raw_parts_mut(best_scores_elements.as_ptr() as *mut i64, best_scores_len)
    };

    climate::node_best_match_unique_batch(node_mins, node_maxs, queries, best_indices, best_scores)
        .map_err(|code| match code {
            climate::ClimateError::InvalidInputLength => -14,
            climate::ClimateError::OutputTooSmall(required) => -(required as jint),
        })
}

fn climate_parameter_distance_summary(
    env: &mut JNIEnv,
    node_mins: jlongArray,
    node_maxs: jlongArray,
    queries: jlongArray,
    dst: jlongArray,
    kind: ClimateParameterDistanceKind,
) -> Result<usize, jint> {
    let node_mins_len = env.get_array_length(node_mins).map_err(|_| -1)? as usize;
    let node_maxs_len = env.get_array_length(node_maxs).map_err(|_| -2)? as usize;
    if node_mins_len != node_maxs_len {
        return Err(-3);
    }

    let queries_len = env.get_array_length(queries).map_err(|_| -4)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < 1 {
        return Err(-6);
    }

    let node_mins_elements = env
        .get_long_array_elements(node_mins, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let node_maxs_elements = env
        .get_long_array_elements(node_maxs, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let queries_elements = env
        .get_long_array_elements(queries, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;

    let node_mins = unsafe {
        std::slice::from_raw_parts(node_mins_elements.as_ptr() as *const i64, node_mins_len)
    };
    let node_maxs = unsafe {
        std::slice::from_raw_parts(node_maxs_elements.as_ptr() as *const i64, node_maxs_len)
    };
    let queries = unsafe {
        std::slice::from_raw_parts(queries_elements.as_ptr() as *const i64, queries_len)
    };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, dst_len) };

    let value = match kind {
        ClimateParameterDistanceKind::Old => {
            climate_parameter_distance::old_distance_sum(node_mins, node_maxs, queries)
        }
        ClimateParameterDistanceKind::Branch => {
            climate_parameter_distance::branch_distance_sum(node_mins, node_maxs, queries)
        }
        ClimateParameterDistanceKind::SubtractFirst => {
            climate_parameter_distance::subtract_first_distance_sum(node_mins, node_maxs, queries)
        }
    }
    .map_err(|code| match code {
        climate::ClimateError::InvalidInputLength => -11,
        climate::ClimateError::OutputTooSmall(required) => -(required as jint),
    })?;

    dst[0] = value;
    Ok(1)
}

fn noise_generator_settings_summary(
    env: &mut JNIEnv,
    sea_levels: jintArray,
    min_ys: jintArray,
    heights: jintArray,
    iterations: jint,
    dst: jintArray,
    kind: NoiseGeneratorSettingsKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let sea_levels_len = env.get_array_length(sea_levels).map_err(|_| -2)? as usize;
    let min_ys_len = env.get_array_length(min_ys).map_err(|_| -3)? as usize;
    let heights_len = env.get_array_length(heights).map_err(|_| -4)? as usize;
    if sea_levels_len != min_ys_len || sea_levels_len != heights_len {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < 1 {
        return Err(-7);
    }

    let sea_levels_elements = env
        .get_int_array_elements(sea_levels, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let min_ys_elements = env
        .get_int_array_elements(min_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let heights_elements = env
        .get_int_array_elements(heights, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let dst_elements = env
        .get_int_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -11)?;

    let sea_levels = unsafe {
        std::slice::from_raw_parts(sea_levels_elements.as_ptr() as *const i32, sea_levels_len)
    };
    let min_ys = unsafe {
        std::slice::from_raw_parts(min_ys_elements.as_ptr() as *const i32, min_ys_len)
    };
    let heights = unsafe {
        std::slice::from_raw_parts(heights_elements.as_ptr() as *const i32, heights_len)
    };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i32, dst_len) };

    let value = match kind {
        NoiseGeneratorSettingsKind::HolderValue => noise_generator_settings::holder_value_settings_sum(
            sea_levels,
            min_ys,
            heights,
            iterations,
        ),
        NoiseGeneratorSettingsKind::MemoizedSupplier => noise_generator_settings::memoized_supplier_settings_sum(
            sea_levels,
            min_ys,
            heights,
            iterations,
        ),
        NoiseGeneratorSettingsKind::LazyPrimitive => noise_generator_settings::lazy_primitive_settings_sum(
            sea_levels,
            min_ys,
            heights,
            iterations,
        ),
        NoiseGeneratorSettingsKind::ManualLazyObject => noise_generator_settings::manual_lazy_object_settings_sum(
            sea_levels,
            min_ys,
            heights,
            iterations,
        ),
        NoiseGeneratorSettingsKind::CachedInt => noise_generator_settings::cached_int_settings_sum(
            sea_levels,
            min_ys,
            heights,
            iterations,
        ),
    }
    .map_err(|code| match code {
        noise_generator_settings::NoiseGeneratorSettingsError::InvalidInputLength => -12,
    })?;

    dst[0] = value;
    Ok(1)
}

fn protochunk_heightmap_summary(iterations: jint, kind: ProtoChunkHeightmapKind) -> jlong {
    let Ok(iterations) = usize::try_from(iterations) else {
        return -1;
    };

    let result = match kind {
        ProtoChunkHeightmapKind::OldEnumSetForeach => {
            protochunk_heightmap::old_enumset_foreach_summary(iterations)
        }
        ProtoChunkHeightmapKind::NewCachedContains => {
            protochunk_heightmap::new_cached_contains_summary(iterations)
        }
    };

    match result {
        Ok(value) => value as jlong,
        Err(protochunk_heightmap::ProtoChunkHeightmapError::InvalidInputLength) => -2,
    }
}

fn range_choice_summary(
    env: &mut JNIEnv,
    inputs: jdoubleArray,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    scenario: jint,
    dst: jlongArray,
    kind: RangeChoiceKind,
) -> Result<usize, jint> {
    let inputs_len = env.get_array_length(inputs).map_err(|_| -1)? as usize;
    let block_x_len = env.get_array_length(block_x).map_err(|_| -2)? as usize;
    let block_y_len = env.get_array_length(block_y).map_err(|_| -3)? as usize;
    let block_z_len = env.get_array_length(block_z).map_err(|_| -4)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;

    if dst_len < range_choice::SUMMARY_FIELDS {
        return Err(-(range_choice::SUMMARY_FIELDS as jint));
    }

    let scenario = match scenario {
        0 => range_choice::ScenarioKind::InConstantOutDynamic,
        1 => range_choice::ScenarioKind::InDynamicOutConstant,
        2 => range_choice::ScenarioKind::BothConstant,
        3 => range_choice::ScenarioKind::BothDynamic,
        _ => return Err(-6),
    };

    let inputs_elements = env
        .get_double_array_elements(inputs, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let block_x_elements = env
        .get_int_array_elements(block_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let block_y_elements = env
        .get_int_array_elements(block_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let block_z_elements = env
        .get_int_array_elements(block_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -11)?;

    let inputs = unsafe { std::slice::from_raw_parts(inputs_elements.as_ptr(), inputs_len) };
    let block_x = unsafe { std::slice::from_raw_parts(block_x_elements.as_ptr(), block_x_len) };
    let block_y = unsafe { std::slice::from_raw_parts(block_y_elements.as_ptr(), block_y_len) };
    let block_z = unsafe { std::slice::from_raw_parts(block_z_elements.as_ptr(), block_z_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            range_choice::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        RangeChoiceKind::OldFillArray => {
            range_choice::old_fill_array_summary(inputs, block_x, block_y, block_z, scenario)
        }
        RangeChoiceKind::OptimizedFillArray => {
            range_choice::optimized_fill_array_summary(inputs, block_x, block_y, block_z, scenario)
        }
    }
    .map_err(|code| match code {
        range_choice::RangeChoiceError::InvalidInputLength => -12,
    })?;

    dst[0] = summary.checksum as i64;
    dst[1] = summary.for_index_calls as i64;

    Ok(range_choice::SUMMARY_FIELDS)
}

fn improved_noise_floor_summary(
    env: &mut JNIEnv,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
    kind: ImprovedNoiseFloorKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let permutation_len = env.get_array_length(permutation).map_err(|_| -2)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < improved_noise_floor::SUMMARY_FIELDS {
        return Err(-(improved_noise_floor::SUMMARY_FIELDS as jint));
    }

    let permutation_elements = env
        .get_byte_array_elements(permutation, ReleaseMode::NoCopyBack)
        .map_err(|_| -4)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;

    let permutation = unsafe {
        std::slice::from_raw_parts(permutation_elements.as_ptr() as *const u8, permutation_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            improved_noise_floor::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        ImprovedNoiseFloorKind::CurrentMthFloor => {
            improved_noise_floor::current_mth_floor_summary(permutation, iterations)
        }
        ImprovedNoiseFloorKind::MathFloor => {
            improved_noise_floor::math_floor_summary(permutation, iterations)
        }
    }
    .map_err(|code| match code {
        improved_noise_floor::ImprovedNoiseFloorError::InvalidPermutationLength => -6,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(improved_noise_floor::SUMMARY_FIELDS)
}

fn surface_rules_sequence_array_summary(
    env: &mut JNIEnv,
    iterations: jint,
    rules: jint,
    dst: jlongArray,
    kind: SurfaceRulesSequenceArrayKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let rules = usize::try_from(rules).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < surface_rules_sequence_array::SUMMARY_FIELDS {
        return Err(-(surface_rules_sequence_array::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            surface_rules_sequence_array::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        SurfaceRulesSequenceArrayKind::ListEnhanced => {
            surface_rules_sequence_array::list_enhanced_summary(iterations, rules)
        }
        SurfaceRulesSequenceArrayKind::ListIndexed => {
            surface_rules_sequence_array::list_indexed_summary(iterations, rules)
        }
        SurfaceRulesSequenceArrayKind::ArrayForeach => {
            surface_rules_sequence_array::array_foreach_summary(iterations, rules)
        }
        SurfaceRulesSequenceArrayKind::ArrayIndexed => {
            surface_rules_sequence_array::array_indexed_summary(iterations, rules)
        }
    }
    .map_err(|code| match code {
        surface_rules_sequence_array::SurfaceRulesSequenceArrayError::InvalidRuleCount => -5,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.total;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_rule;

    Ok(surface_rules_sequence_array::SUMMARY_FIELDS)
}

fn surface_rules_test_rule_state_summary(
    env: &mut JNIEnv,
    iterations: jint,
    period: jint,
    dst: jlongArray,
    kind: SurfaceRulesTestRuleStateKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let period = usize::try_from(period).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < surface_rules_test_rule_state::SUMMARY_FIELDS {
        return Err(-(surface_rules_test_rule_state::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            surface_rules_test_rule_state::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        SurfaceRulesTestRuleStateKind::OldStateRule => {
            surface_rules_test_rule_state::old_state_rule_summary(iterations, period)
        }
        SurfaceRulesTestRuleStateKind::NewStateRule => {
            surface_rules_test_rule_state::new_state_rule_summary(iterations, period)
        }
    }
    .map_err(|code| match code {
        surface_rules_test_rule_state::SurfaceRulesTestRuleStateError::InvalidPeriod => -5,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.hits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_hit as i64;

    Ok(surface_rules_test_rule_state::SUMMARY_FIELDS)
}

fn placed_feature_traversal_summary(
    env: &mut JNIEnv,
    seed: jlong,
    traversals: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let traversals = usize::try_from(traversals).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < placed_feature_traversal::SUMMARY_FIELDS {
        return Err(-(placed_feature_traversal::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            placed_feature_traversal::SUMMARY_FIELDS,
        )
    };

    let summary = placed_feature_traversal::recursive_summary(seed as i64, traversals);
    dst[0] = summary.count as i64;
    dst[1] = summary.hash as i64;

    Ok(placed_feature_traversal::SUMMARY_FIELDS)
}

fn ore_feature_loop_summary(
    env: &mut JNIEnv,
    center_x: jdoubleArray,
    center_y: jdoubleArray,
    center_z: jdoubleArray,
    radius: jdoubleArray,
    min_x: jintArray,
    min_y: jintArray,
    min_z: jintArray,
    max_x: jintArray,
    max_y: jintArray,
    max_z: jintArray,
    width: jint,
    height: jint,
    origin_x: jint,
    origin_y: jint,
    origin_z: jint,
    dst: jlongArray,
    kind: OreFeatureLoopKind,
) -> Result<usize, jint> {
    let center_x_len = env.get_array_length(center_x).map_err(|_| -1)? as usize;
    let center_y_len = env.get_array_length(center_y).map_err(|_| -2)? as usize;
    let center_z_len = env.get_array_length(center_z).map_err(|_| -3)? as usize;
    let radius_len = env.get_array_length(radius).map_err(|_| -4)? as usize;
    let min_x_len = env.get_array_length(min_x).map_err(|_| -5)? as usize;
    let min_y_len = env.get_array_length(min_y).map_err(|_| -6)? as usize;
    let min_z_len = env.get_array_length(min_z).map_err(|_| -7)? as usize;
    let max_x_len = env.get_array_length(max_x).map_err(|_| -8)? as usize;
    let max_y_len = env.get_array_length(max_y).map_err(|_| -9)? as usize;
    let max_z_len = env.get_array_length(max_z).map_err(|_| -10)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -11)? as usize;
    if dst_len < ore_feature_loop::SUMMARY_FIELDS {
        return Err(-(ore_feature_loop::SUMMARY_FIELDS as jint));
    }

    let center_x_elements = env
        .get_double_array_elements(center_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let center_y_elements = env
        .get_double_array_elements(center_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let center_z_elements = env
        .get_double_array_elements(center_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let radius_elements = env
        .get_double_array_elements(radius, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let min_x_elements = env
        .get_int_array_elements(min_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let min_y_elements = env
        .get_int_array_elements(min_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let min_z_elements = env
        .get_int_array_elements(min_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;
    let max_x_elements = env
        .get_int_array_elements(max_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -19)?;
    let max_y_elements = env
        .get_int_array_elements(max_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -20)?;
    let max_z_elements = env
        .get_int_array_elements(max_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -21)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -22)?;

    let arrays = ore_feature_loop::OreFeatureLoopArrays {
        center_x: unsafe { std::slice::from_raw_parts(center_x_elements.as_ptr(), center_x_len) },
        center_y: unsafe { std::slice::from_raw_parts(center_y_elements.as_ptr(), center_y_len) },
        center_z: unsafe { std::slice::from_raw_parts(center_z_elements.as_ptr(), center_z_len) },
        radius: unsafe { std::slice::from_raw_parts(radius_elements.as_ptr(), radius_len) },
        min_x: unsafe { std::slice::from_raw_parts(min_x_elements.as_ptr(), min_x_len) },
        min_y: unsafe { std::slice::from_raw_parts(min_y_elements.as_ptr(), min_y_len) },
        min_z: unsafe { std::slice::from_raw_parts(min_z_elements.as_ptr(), min_z_len) },
        max_x: unsafe { std::slice::from_raw_parts(max_x_elements.as_ptr(), max_x_len) },
        max_y: unsafe { std::slice::from_raw_parts(max_y_elements.as_ptr(), max_y_len) },
        max_z: unsafe { std::slice::from_raw_parts(max_z_elements.as_ptr(), max_z_len) },
    };
    let config = ore_feature_loop::OreFeatureLoopConfig {
        width,
        height,
        origin_x,
        origin_y,
        origin_z,
    };
    let summary = match kind {
        OreFeatureLoopKind::OldLoop => ore_feature_loop::old_loop_summary(arrays, config),
        OreFeatureLoopKind::OptimizedLoop => {
            ore_feature_loop::optimized_loop_summary(arrays, config)
        }
    }
    .map_err(|code| match code {
        ore_feature_loop::OreFeatureLoopError::LengthMismatch => -23,
    })?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            ore_feature_loop::SUMMARY_FIELDS,
        )
    };
    dst[0] = summary.checksum as i64;

    Ok(ore_feature_loop::SUMMARY_FIELDS)
}

fn ticketset_search_summary(
    env: &mut JNIEnv,
    iterations: jint,
    linear_limit: jint,
    dst: jlongArray,
    kind: TicketSetSearchKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let linear_limit = usize::try_from(linear_limit).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < ticketset_search::SUMMARY_FIELDS {
        return Err(-(ticketset_search::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            ticketset_search::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        TicketSetSearchKind::Binary => ticketset_search::binary_summary(iterations),
        TicketSetSearchKind::UncheckedBinary => {
            ticketset_search::unchecked_binary_summary(iterations)
        }
        TicketSetSearchKind::Linear => ticketset_search::linear_summary(iterations, linear_limit),
    }
    .map_err(|code| match code {
        ticketset_search::TicketSetSearchError::InvalidIterations => -5,
    })?;
    dst[0] = summary.value;

    Ok(ticketset_search::SUMMARY_FIELDS)
}

fn waypoint_snapshot_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: WaypointSnapshotKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < waypoint_snapshot::SUMMARY_FIELDS {
        return Err(-(waypoint_snapshot::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            waypoint_snapshot::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        WaypointSnapshotKind::ToArray => waypoint_snapshot::to_array_summary(iterations),
        WaypointSnapshotKind::SizedArray => waypoint_snapshot::sized_array_summary(iterations),
        WaypointSnapshotKind::Manual => waypoint_snapshot::manual_summary(iterations),
    }
    .map_err(|code| match code {
        waypoint_snapshot::WaypointSnapshotError::InvalidIterations => -4,
    })?;
    dst[0] = summary.value;

    Ok(waypoint_snapshot::SUMMARY_FIELDS)
}

fn waypoint_table_view_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: WaypointTableViewKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < waypoint_table_view::SUMMARY_FIELDS {
        return Err(-(waypoint_table_view::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            waypoint_table_view::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        WaypointTableViewKind::TransposeRow => waypoint_table_view::transpose_row_summary(iterations),
        WaypointTableViewKind::Column => waypoint_table_view::column_summary(iterations),
    }
    .map_err(|code| match code {
        waypoint_table_view::WaypointTableViewError::InvalidIterations => -4,
    })?;
    dst[0] = summary.value;

    Ok(waypoint_table_view::SUMMARY_FIELDS)
}

fn waypoint_chunk_update_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: WaypointChunkUpdateKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < waypoint_chunk_update::SUMMARY_FIELDS {
        return Err(-(waypoint_chunk_update::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            waypoint_chunk_update::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        WaypointChunkUpdateKind::Distance => waypoint_chunk_update::distance_summary(iterations),
        WaypointChunkUpdateKind::LongKey => waypoint_chunk_update::long_key_summary(iterations),
    };
    dst[0] = summary.value;

    Ok(waypoint_chunk_update::SUMMARY_FIELDS)
}

fn remapper_hash_threshold_summary(
    env: &mut JNIEnv,
    iterations: jint,
    input_paths: jobjectArray,
    dst: jlongArray,
    kind: RemapperHashThresholdKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let input_paths = java_string_array_to_vec(env, input_paths).map_err(|code| code)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < remapper_hash_threshold::SUMMARY_FIELDS {
        return Err(-(remapper_hash_threshold::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            remapper_hash_threshold::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        RemapperHashThresholdKind::ComputeIfAbsent => {
            remapper_hash_threshold::compute_if_absent_summary(&input_paths, iterations)
        }
        RemapperHashThresholdKind::Put => {
            remapper_hash_threshold::put_summary(&input_paths, iterations)
        }
        RemapperHashThresholdKind::Hybrid => {
            remapper_hash_threshold::hybrid_summary(&input_paths, iterations)
        }
        RemapperHashThresholdKind::Parallel => {
            remapper_hash_threshold::parallel_summary(&input_paths, iterations)
        }
    }
    .map_err(|code| match code {
        remapper_hash_threshold::RemapperHashThresholdError::InvalidIterations => -5,
        remapper_hash_threshold::RemapperHashThresholdError::EmptyInputs => -6,
        remapper_hash_threshold::RemapperHashThresholdError::Io => -7,
    })?;
    dst[0] = summary.count as i64;
    dst[1] = summary.total_entries as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_digest as i64;

    Ok(remapper_hash_threshold::SUMMARY_FIELDS)
}

fn waypoint_manager_skip_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: WaypointManagerSkipKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < waypoint_manager_skip::SUMMARY_FIELDS {
        return Err(-(waypoint_manager_skip::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            waypoint_manager_skip::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        WaypointManagerSkipKind::CurrentPlayerFull => {
            waypoint_manager_skip::current_player_full_summary(iterations)
        }
        WaypointManagerSkipKind::SkipPlayerFull => {
            waypoint_manager_skip::skip_player_full_summary(iterations)
        }
        WaypointManagerSkipKind::CurrentPlayerPartial => {
            waypoint_manager_skip::current_player_partial_summary(iterations)
        }
        WaypointManagerSkipKind::SkipPlayerPartial => {
            waypoint_manager_skip::skip_player_partial_summary(iterations)
        }
        WaypointManagerSkipKind::CurrentWaypointFull => {
            waypoint_manager_skip::current_waypoint_full_summary(iterations)
        }
        WaypointManagerSkipKind::SkipWaypointFull => {
            waypoint_manager_skip::skip_waypoint_full_summary(iterations)
        }
        WaypointManagerSkipKind::CurrentWaypointPartial => {
            waypoint_manager_skip::current_waypoint_partial_summary(iterations)
        }
        WaypointManagerSkipKind::SkipWaypointPartial => {
            waypoint_manager_skip::skip_waypoint_partial_summary(iterations)
        }
    }
    .map_err(|code| match code {
        waypoint_manager_skip::WaypointManagerSkipError::InvalidIterations => -4,
    })?;
    dst[0] = summary.value;

    Ok(waypoint_manager_skip::SUMMARY_FIELDS)
}

fn climate_rtree_build_tree_handle(
    env: &mut JNIEnv,
    node_mins: jlongArray,
    node_maxs: jlongArray,
) -> Result<jlong, jint> {
    let node_mins_len = env.get_array_length(node_mins).map_err(|_| -1)? as usize;
    let node_maxs_len = env.get_array_length(node_maxs).map_err(|_| -2)? as usize;
    if node_mins_len != node_maxs_len {
        return Err(-3);
    }
    if node_mins_len == 0 || node_mins_len % climate_rtree::PARAMETER_COUNT != 0 {
        return Err(-4);
    }

    let node_mins_elements = env
        .get_long_array_elements(node_mins, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let node_maxs_elements = env
        .get_long_array_elements(node_maxs, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;

    let node_mins = unsafe {
        std::slice::from_raw_parts(node_mins_elements.as_ptr() as *const i64, node_mins_len)
    };
    let node_maxs = unsafe {
        std::slice::from_raw_parts(node_maxs_elements.as_ptr() as *const i64, node_maxs_len)
    };

    let (root, leaves) = climate_rtree::build_from_flat_with_leaves(node_mins, node_maxs).map_err(|_| -7)?;
    Ok(Box::into_raw(Box::new(ClimateRTreeHandle { root, leaves })) as jlong)
}

fn climate_rtree_free_tree_handle(handle: jlong) {
    if handle == 0 {
        return;
    }

    unsafe {
        drop(Box::from_raw(handle as *mut ClimateRTreeHandle));
    }
}

fn climate_rtree_checksum_tree_handle(handle: jlong) -> jlong {
    if handle == 0 {
        return 0;
    }

    let handle = unsafe { &*(handle as *const ClimateRTreeHandle) };
    climate_rtree::checksum_tree(&handle.root) as jlong
}

fn climate_rtree_search_current_one_packed(
    handle: jlong,
    query: [i64; climate_rtree::PARAMETER_COUNT],
    previous_index: jint,
) -> Result<jint, jint> {
    if handle == 0 {
        return Err(-1);
    }

    let handle = unsafe { &*(handle as *const ClimateRTreeHandle) };
    let (index, _) = climate_rtree::search_current_index(
        &handle.root,
        &handle.leaves,
        &query,
        previous_index,
    )
    .map_err(|code| match code {
        climate_rtree::ClimateRTreeBatchError::InvalidInputLength => -2,
        climate_rtree::ClimateRTreeBatchError::OutputTooSmall(required) => -(required as jint),
    })?;
    Ok(index as jint)
}

fn climate_rtree_search_bounded_one_packed(
    handle: jlong,
    query: [i64; climate_rtree::PARAMETER_COUNT],
    previous_index: jint,
) -> Result<jint, jint> {
    if handle == 0 {
        return Err(-1);
    }

    let handle = unsafe { &*(handle as *const ClimateRTreeHandle) };
    let (index, _) = climate_rtree::search_bounded_index_borrowed(
        &handle.root,
        &handle.leaves,
        &query,
        previous_index,
    )
    .map_err(|code| match code {
        climate_rtree::ClimateRTreeBatchError::InvalidInputLength => -2,
        climate_rtree::ClimateRTreeBatchError::OutputTooSmall(required) => -(required as jint),
    })?;
    Ok(index as jint)
}

fn climate_rtree_search_batch(
    env: &mut JNIEnv,
    handle: jlong,
    queries: jlongArray,
    best_indices: jintArray,
    best_scores: jlongArray,
    search: fn(
        &climate_rtree::NodeRef,
        &[i64],
        &mut [i32],
        &mut [i64],
    ) -> Result<usize, climate_rtree::ClimateRTreeBatchError>,
) -> Result<usize, jint> {
    if handle == 0 {
        return Err(-1);
    }

    let queries_len = env.get_array_length(queries).map_err(|_| -2)? as usize;
    if queries_len % climate_rtree::PARAMETER_COUNT != 0 {
        return Err(-3);
    }

    let query_count = queries_len / climate_rtree::PARAMETER_COUNT;
    let best_indices_len = env.get_array_length(best_indices).map_err(|_| -4)? as usize;
    if best_indices_len < query_count {
        return Err(-(query_count as jint));
    }
    let best_scores_len = env.get_array_length(best_scores).map_err(|_| -5)? as usize;
    if best_scores_len < query_count {
        return Err(-(query_count as jint));
    }

    let queries_elements = env
        .get_long_array_elements(queries, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let best_indices_elements = env
        .get_int_array_elements(best_indices, ReleaseMode::CopyBack)
        .map_err(|_| -7)?;
    let best_scores_elements = env
        .get_long_array_elements(best_scores, ReleaseMode::CopyBack)
        .map_err(|_| -8)?;

    let queries = unsafe {
        std::slice::from_raw_parts(queries_elements.as_ptr() as *const i64, queries_len)
    };
    let best_indices = unsafe {
        std::slice::from_raw_parts_mut(best_indices_elements.as_ptr() as *mut i32, best_indices_len)
    };
    let best_scores = unsafe {
        std::slice::from_raw_parts_mut(best_scores_elements.as_ptr() as *mut i64, best_scores_len)
    };
    let handle = unsafe { &*(handle as *const ClimateRTreeHandle) };

    search(&handle.root, queries, best_indices, best_scores).map_err(|code| match code {
        climate_rtree::ClimateRTreeBatchError::InvalidInputLength => -9,
        climate_rtree::ClimateRTreeBatchError::OutputTooSmall(required) => -(required as jint),
    })
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLz4_lz4BlockCompress(
    mut env: JNIEnv,
    _class: JClass,
    input: jbyteArray,
    block_size: jint,
    dst: jbyteArray,
) -> jint {
    match lz4_block_compress(&mut env, input, block_size, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_NativeLz4BlockOutputStream_lz4BlockCompress(
    mut env: JNIEnv,
    _class: JClass,
    input: jbyteArray,
    input_len: jint,
    block_size: jint,
    dst: jbyteArray,
) -> jint {
    match lz4_block_compress_with_len(&mut env, input, input_len, block_size, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLz4_lz4BlockDecompress(
    mut env: JNIEnv,
    _class: JClass,
    input: jbyteArray,
    dst: jbyteArray,
) -> jint {
    match lz4_block_decompress(&mut env, input, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLz4StreamRoundtrip_roundtripSummary(
    mut env: JNIEnv,
    _class: JClass,
    payload: jbyteArray,
    block_size: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match lz4_stream_roundtrip_summary(
        &mut env,
        payload,
        block_size,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNbtGzipBufferShape_shapeSummary(
    mut env: JNIEnv,
    _class: JClass,
    write_lengths: jintArray,
    repeats: jint,
    outer_buffer_size: jint,
    gzip_buffer_size: jint,
    dst: jlongArray,
) -> jint {
    match nbt_gzip_buffer_shape_summary(
        &mut env,
        write_lengths,
        repeats,
        outer_buffer_size,
        gzip_buffer_size,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCompressionThresholdShape_thresholdSummary(
    mut env: JNIEnv,
    _class: JClass,
    packet_lengths: jintArray,
    thresholds: jintArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match compression_threshold_shape_summary(
        &mut env,
        packet_lengths,
        thresholds,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDeflaterInputShape_copiedSummary(
    mut env: JNIEnv,
    _class: JClass,
    payload: jbyteArray,
    offsets: jintArray,
    lengths: jintArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match deflater_input_shape_summary(
        &mut env,
        payload,
        offsets,
        lengths,
        iterations,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDeflaterInputShape_sliceSummary(
    mut env: JNIEnv,
    _class: JClass,
    payload: jbyteArray,
    offsets: jintArray,
    lengths: jintArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match deflater_input_shape_summary(
        &mut env,
        payload,
        offsets,
        lengths,
        iterations,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

fn lz4_block_compress(
    env: &mut JNIEnv,
    input: jbyteArray,
    block_size: jint,
    dst: jbyteArray,
) -> Result<usize, jint> {
    let input_len = env.get_array_length(input).map_err(|_| -2)?;
    lz4_block_compress_with_len(env, input, input_len, block_size, dst)
}

fn lz4_block_compress_with_len(
    env: &mut JNIEnv,
    input: jbyteArray,
    input_len: jint,
    block_size: jint,
    dst: jbyteArray,
) -> Result<usize, jint> {
    let block_size = usize::try_from(block_size).map_err(|_| -1)?;
    let input_array_len = env.get_array_length(input).map_err(|_| -2)? as usize;
    let input_len = usize::try_from(input_len).map_err(|_| -2)?;
    if input_len > input_array_len {
        return Err(-2);
    }
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    let max_len = compression::lz4_block_stream_max_compressed_len(input_len, block_size)
        .map_err(|_| -4)?;
    if dst_len < max_len {
        return Err(-(max_len as jint));
    }

    let input_elements = env
        .get_byte_array_elements(input, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let dst_elements = env
        .get_byte_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;

    let input = unsafe { std::slice::from_raw_parts(input_elements.as_ptr() as *const u8, input_len) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut u8, dst_len) };

    compression::lz4_block_stream_compress_into(input, block_size, dst).map_err(|code| match code {
        compression::CompressionError::InvalidBlockSize => -4,
        compression::CompressionError::OutputTooSmall(required) => -(required as jint),
        compression::CompressionError::CorruptStream => -7,
        compression::CompressionError::UnexpectedEof => -8,
        compression::CompressionError::Lz4Compress => -9,
        compression::CompressionError::Lz4Decompress => -10,
    })
}

fn lz4_block_decompress(
    env: &mut JNIEnv,
    input: jbyteArray,
    dst: jbyteArray,
) -> Result<usize, jint> {
    let input_len = env.get_array_length(input).map_err(|_| -1)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;

    let input_elements = env
        .get_byte_array_elements(input, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let dst_elements = env
        .get_byte_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;

    let input = unsafe { std::slice::from_raw_parts(input_elements.as_ptr() as *const u8, input_len) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut u8, dst_len) };

    compression::lz4_block_stream_decompress_into(input, dst).map_err(|code| match code {
        compression::CompressionError::InvalidBlockSize => -5,
        compression::CompressionError::OutputTooSmall(required) => -(required as jint),
        compression::CompressionError::CorruptStream => -6,
        compression::CompressionError::UnexpectedEof => -7,
        compression::CompressionError::Lz4Compress => -8,
        compression::CompressionError::Lz4Decompress => -9,
    })
}

fn lz4_stream_roundtrip_summary(
    env: &mut JNIEnv,
    payload: jbyteArray,
    block_size: jint,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let block_size = usize::try_from(block_size).map_err(|_| -1)?;
    let iterations = usize::try_from(iterations).map_err(|_| -2)?;
    let payload_len = env.get_array_length(payload).map_err(|_| -3)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < lz4_stream_roundtrip::SUMMARY_FIELDS {
        return Err(-(lz4_stream_roundtrip::SUMMARY_FIELDS as jint));
    }

    let payload_elements = env
        .get_byte_array_elements(payload, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;

    let payload = unsafe { std::slice::from_raw_parts(payload_elements.as_ptr() as *const u8, payload_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            lz4_stream_roundtrip::SUMMARY_FIELDS,
        )
    };

    let summary = lz4_stream_roundtrip::roundtrip_summary(payload, block_size, iterations)
        .map_err(|code| match code {
            lz4_stream_roundtrip::Lz4StreamRoundtripError::InvalidIterations => -2,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Compression(
                compression::CompressionError::InvalidBlockSize,
            ) => -7,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Compression(
                compression::CompressionError::OutputTooSmall(required),
            ) => -(required as jint),
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Compression(
                compression::CompressionError::CorruptStream,
            ) => -8,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Compression(
                compression::CompressionError::UnexpectedEof,
            ) => -9,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Compression(
                compression::CompressionError::Lz4Compress,
            ) => -10,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Compression(
                compression::CompressionError::Lz4Decompress,
            ) => -11,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Decompression(
                compression::CompressionError::InvalidBlockSize,
            ) => -12,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Decompression(
                compression::CompressionError::OutputTooSmall(required),
            ) => -(required as jint),
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Decompression(
                compression::CompressionError::CorruptStream,
            ) => -13,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Decompression(
                compression::CompressionError::UnexpectedEof,
            ) => -14,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Decompression(
                compression::CompressionError::Lz4Compress,
            ) => -15,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::Decompression(
                compression::CompressionError::Lz4Decompress,
            ) => -16,
            lz4_stream_roundtrip::Lz4StreamRoundtripError::CorruptRoundtrip => -17,
        })?;

    dst[0] = summary.iterations as i64;
    dst[1] = summary.input_bytes as i64;
    dst[2] = summary.restored_bytes as i64;
    dst[3] = summary.compressed_bytes as i64;
    dst[4] = summary.restored_checksum as i64;
    dst[5] = summary.compressed_checksum as i64;
    dst[6] = summary.last_compressed_bytes as i64;

    Ok(lz4_stream_roundtrip::SUMMARY_FIELDS)
}

fn nbt_gzip_buffer_shape_summary(
    env: &mut JNIEnv,
    write_lengths: jintArray,
    repeats: jint,
    outer_buffer_size: jint,
    gzip_buffer_size: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let repeats = usize::try_from(repeats).map_err(|_| -1)?;
    let outer_buffer_size = usize::try_from(outer_buffer_size).map_err(|_| -2)?;
    let gzip_buffer_size = usize::try_from(gzip_buffer_size).map_err(|_| -3)?;
    let write_lengths_len = env.get_array_length(write_lengths).map_err(|_| -4)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < nbt_gzip_buffer_shape::SUMMARY_FIELDS {
        return Err(-(nbt_gzip_buffer_shape::SUMMARY_FIELDS as jint));
    }

    let write_lengths_elements = env
        .get_int_array_elements(write_lengths, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -7)?;

    let write_lengths = unsafe {
        std::slice::from_raw_parts(write_lengths_elements.as_ptr(), write_lengths_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            nbt_gzip_buffer_shape::SUMMARY_FIELDS,
        )
    };

    let summary = nbt_gzip_buffer_shape::shape_summary(
        write_lengths,
        repeats,
        outer_buffer_size,
        gzip_buffer_size,
    )
    .map_err(|code| match code {
        nbt_gzip_buffer_shape::NbtGzipBufferShapeError::InvalidWriteLength => -8,
        nbt_gzip_buffer_shape::NbtGzipBufferShapeError::InvalidRepeats => -1,
        nbt_gzip_buffer_shape::NbtGzipBufferShapeError::InvalidBufferSize => -2,
    })?;

    dst[0] = summary.write_calls as i64;
    dst[1] = summary.input_bytes as i64;
    dst[2] = summary.outer_flushes as i64;
    dst[3] = summary.gzip_input_calls as i64;
    dst[4] = summary.direct_writes as i64;
    dst[5] = summary.modeled_gzip_chunks as i64;
    dst[6] = summary.largest_write as i64;
    dst[7] = summary.checksum as i64;

    Ok(nbt_gzip_buffer_shape::SUMMARY_FIELDS)
}

fn compression_threshold_shape_summary(
    env: &mut JNIEnv,
    packet_lengths: jintArray,
    thresholds: jintArray,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let packet_lengths_len = env.get_array_length(packet_lengths).map_err(|_| -2)? as usize;
    let thresholds_len = env.get_array_length(thresholds).map_err(|_| -3)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < compression_threshold_shape::SUMMARY_FIELDS {
        return Err(-(compression_threshold_shape::SUMMARY_FIELDS as jint));
    }

    let packet_lengths_elements = env
        .get_int_array_elements(packet_lengths, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let thresholds_elements = env
        .get_int_array_elements(thresholds, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -7)?;

    let packet_lengths = unsafe {
        std::slice::from_raw_parts(packet_lengths_elements.as_ptr(), packet_lengths_len)
    };
    let thresholds = unsafe {
        std::slice::from_raw_parts(thresholds_elements.as_ptr(), thresholds_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            compression_threshold_shape::SUMMARY_FIELDS,
        )
    };

    let summary = compression_threshold_shape::threshold_summary(
        packet_lengths,
        thresholds,
        iterations,
    )
    .map_err(|code| match code {
        compression_threshold_shape::CompressionThresholdShapeError::InvalidPacketLength => -8,
        compression_threshold_shape::CompressionThresholdShapeError::InvalidThreshold => -9,
        compression_threshold_shape::CompressionThresholdShapeError::InvalidIterations => -1,
    })?;

    dst[0] = summary.packets as i64;
    dst[1] = summary.thresholds as i64;
    dst[2] = summary.total_payload_bytes as i64;
    dst[3] = summary.bypassed_packets as i64;
    dst[4] = summary.compressed_packets as i64;
    dst[5] = summary.framed_bytes as i64;
    dst[6] = summary.compression_input_bytes as i64;
    dst[7] = summary.checksum as i64;

    Ok(compression_threshold_shape::SUMMARY_FIELDS)
}

fn deflater_input_shape_summary(
    env: &mut JNIEnv,
    payload: jbyteArray,
    offsets: jintArray,
    lengths: jintArray,
    iterations: jint,
    dst: jlongArray,
    copied: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let payload_len = env.get_array_length(payload).map_err(|_| -2)? as usize;
    let offsets_len = env.get_array_length(offsets).map_err(|_| -3)? as usize;
    let lengths_len = env.get_array_length(lengths).map_err(|_| -4)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < deflater_input_shape::SUMMARY_FIELDS {
        return Err(-(deflater_input_shape::SUMMARY_FIELDS as jint));
    }
    if offsets_len != lengths_len {
        return Err(-6);
    }

    let payload_elements = env
        .get_byte_array_elements(payload, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let offsets_elements = env
        .get_int_array_elements(offsets, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let lengths_elements = env
        .get_int_array_elements(lengths, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;

    let payload = unsafe {
        std::slice::from_raw_parts(payload_elements.as_ptr() as *const u8, payload_len)
    };
    let offsets = unsafe {
        std::slice::from_raw_parts(offsets_elements.as_ptr() as *const i32, offsets_len)
    };
    let lengths = unsafe {
        std::slice::from_raw_parts(lengths_elements.as_ptr() as *const i32, lengths_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            deflater_input_shape::SUMMARY_FIELDS,
        )
    };

    let summary = if copied {
        deflater_input_shape::copied_summary(payload, offsets, lengths, iterations)
    } else {
        deflater_input_shape::slice_summary(payload, offsets, lengths, iterations)
    }
    .map_err(|code| match code {
        deflater_input_shape::DeflaterInputShapeError::InvalidInputLength => -11,
        deflater_input_shape::DeflaterInputShapeError::InvalidIterations => -12,
        deflater_input_shape::DeflaterInputShapeError::InvalidSlice => -13,
    })?;

    dst[0] = summary.visits as i64;
    dst[1] = summary.total_bytes as i64;
    dst[2] = summary.copied_bytes as i64;
    dst[3] = summary.payload_checksum as i64;
    dst[4] = summary.shape_checksum as i64;

    Ok(deflater_input_shape::SUMMARY_FIELDS)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePosition_chunkPackBatch(
    mut env: JNIEnv,
    _class: JClass,
    xs: jintArray,
    zs: jintArray,
    dst: jlongArray,
) -> jint {
    match chunk_pack_batch(&mut env, xs, zs, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePosition_chunkHashBatch(
    mut env: JNIEnv,
    _class: JClass,
    xs: jintArray,
    zs: jintArray,
    dst: jintArray,
) -> jint {
    match chunk_hash_batch(&mut env, xs, zs, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePosition_sectionPackBatch(
    mut env: JNIEnv,
    _class: JClass,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    dst: jlongArray,
) -> jint {
    match section_pack_batch(&mut env, xs, ys, zs, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePosition_positionCombinedBatch(
    mut env: JNIEnv,
    _class: JClass,
    chunk_xs: jintArray,
    chunk_zs: jintArray,
    section_xs: jintArray,
    section_ys: jintArray,
    section_zs: jintArray,
    chunk_dst: jlongArray,
    hash_dst: jintArray,
    section_dst: jlongArray,
) -> jint {
    match position_combined_batch(
        &mut env,
        chunk_xs,
        chunk_zs,
        section_xs,
        section_ys,
        section_zs,
        chunk_dst,
        hash_dst,
        section_dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAreaMap_updateSummaryBatch(
    mut env: JNIEnv,
    _class: JClass,
    from_xs: jintArray,
    from_zs: jintArray,
    old_distances: jintArray,
    to_xs: jintArray,
    to_zs: jintArray,
    new_distances: jintArray,
    dst: jlongArray,
) -> jint {
    match area_map_update_summary_batch(
        &mut env,
        from_xs,
        from_zs,
        old_distances,
        to_xs,
        to_zs,
        new_distances,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAreaMap_squareSummaryBatch(
    mut env: JNIEnv,
    _class: JClass,
    op: jint,
    chunk_xs: jintArray,
    chunk_zs: jintArray,
    distances: jintArray,
    dst: jlongArray,
) -> jint {
    match area_map_square_summary_batch(&mut env, op, chunk_xs, chunk_zs, distances, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_ca_spottedleaf_moonrise_common_misc_PaperNativeAreaMap_nativeUpdateOpsBatch(
    mut env: JNIEnv,
    _class: JClass,
    from_x: jint,
    from_z: jint,
    old_distance: jint,
    to_x: jint,
    to_z: jint,
    new_distance: jint,
    operations: jbyteArray,
    chunk_keys: jlongArray,
) -> jint {
    match area_map_update_ops_batch(
        &mut env,
        from_x,
        from_z,
        old_distance,
        to_x,
        to_z,
        new_distance,
        operations,
        chunk_keys,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_ca_spottedleaf_moonrise_common_misc_PaperNativeAreaMap_nativeSquareOpsBatch(
    mut env: JNIEnv,
    _class: JClass,
    op: jint,
    chunk_x: jint,
    chunk_z: jint,
    distance: jint,
    operations: jbyteArray,
    chunk_keys: jlongArray,
) -> jint {
    match area_map_square_ops_batch(
        &mut env,
        op,
        chunk_x,
        chunk_z,
        distance,
        operations,
        chunk_keys,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeReferenceList_runOps(
    mut env: JNIEnv,
    _class: JClass,
    linear_search_limit: jint,
    initial_values: jintArray,
    operations: jbyteArray,
    values: jintArray,
    dst: jlongArray,
) -> jint {
    match reference_list_run_ops(
        &mut env,
        linear_search_limit,
        initial_values,
        operations,
        values,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeReferenceList_createHandle(
    _env: JNIEnv,
    _class: JClass,
    linear_search_limit: jint,
) -> jlong {
    match usize::try_from(linear_search_limit) {
        Ok(limit) => Box::into_raw(Box::new(ReferenceListHandle {
            list: reference_list::ReferenceList::new(limit),
        })) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeReferenceList_freeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    reference_list_free_handle(handle);
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeReferenceList_applyOp(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    op: jint,
    value: jint,
) -> jint {
    match reference_list_apply_op(handle, op, value) {
        Ok(encoded) => encoded,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeReferenceList_orderChecksumHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    reference_list_order_checksum_handle(handle)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTicketPack_packSummary(
    mut env: JNIEnv,
    _class: JClass,
    positions: jlongArray,
    ticket_types: jbyteArray,
    ticket_levels: jintArray,
    tickets_per_chunk: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match ticket_pack_pack_summary(
        &mut env,
        positions,
        ticket_types,
        ticket_levels,
        tickets_per_chunk,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTicketCompare_compareIndexedBatch(
    mut env: JNIEnv,
    _class: JClass,
    levels: jintArray,
    type_ids: jlongArray,
    has_identifier_comparators: jbyteArray,
    identifiers: jintArray,
    left_indices: jintArray,
    right_indices: jintArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match ticket_compare_compare_indexed_batch(
        &mut env,
        levels,
        type_ids,
        has_identifier_comparators,
        identifiers,
        left_indices,
        right_indices,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeChunkTicketStage_runBatch(
    mut env: JNIEnv,
    _class: JClass,
    query_keys: jlongArray,
    staged_keys: jlongArray,
    staged_values: jbyteArray,
    mutation_keys: jlongArray,
    get_iterations: jint,
    mutation_iterations: jint,
    dst: jlongArray,
) -> jint {
    match chunk_ticket_stage_run_batch(
        &mut env,
        query_keys,
        staged_keys,
        staged_values,
        mutation_keys,
        get_iterations,
        mutation_iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAquiferSurfaceSampling_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match aquifer_surface_sampling_batch_summary(&mut env, iterations, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAquiferSurfaceSampling_newBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match aquifer_surface_sampling_batch_summary(&mut env, iterations, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAquiferIndexStride_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match aquifer_index_stride_batch_summary(&mut env, iterations, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAquiferIndexStride_newBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match aquifer_index_stride_batch_summary(&mut env, iterations, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAquiferPositionalLocation_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
) -> jint {
    match aquifer_positional_location_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        seed_lo_salt,
        seed_hi_salt,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeAquiferPositionalLocation_directBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
) -> jint {
    match aquifer_positional_location_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        seed_lo_salt,
        seed_hi_salt,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeXoroshiroPositionalDirect_oldFloatBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
) -> jint {
    match xoroshiro_positional_direct_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        seed_lo_salt,
        seed_hi_salt,
        dst,
        false,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeXoroshiroPositionalDirect_directFloatBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
) -> jint {
    match xoroshiro_positional_direct_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        seed_lo_salt,
        seed_hi_salt,
        dst,
        true,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeXoroshiroPositionalDirect_oldDoubleBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
) -> jint {
    match xoroshiro_positional_direct_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        seed_lo_salt,
        seed_hi_salt,
        dst,
        false,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeXoroshiroPositionalDirect_directDoubleBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
) -> jint {
    match xoroshiro_positional_direct_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        seed_lo_salt,
        seed_hi_salt,
        dst,
        true,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeYClampedGradient_currentBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_ys: jintArray,
    from_ys: jintArray,
    to_ys: jintArray,
    from_values: jdoubleArray,
    to_values: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match yclamped_gradient_batch_summary(
        &mut env,
        iterations,
        block_ys,
        from_ys,
        to_ys,
        from_values,
        to_values,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeYClampedGradient_optimizedBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_ys: jintArray,
    from_ys: jintArray,
    to_ys: jintArray,
    from_values: jdoubleArray,
    to_values: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match yclamped_gradient_batch_summary(
        &mut env,
        iterations,
        block_ys,
        from_ys,
        to_ys,
        from_values,
        to_values,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeBeardifierBury_currentBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match beardifier_bury_batch_summary(&mut env, iterations, xs, ys, zs, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeBeardifierBury_optimizedBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match beardifier_bury_batch_summary(&mut env, iterations, xs, ys, zs, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeBiomeGetBiome_currentBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    seeds: jlongArray,
    block_xs: jintArray,
    block_ys: jintArray,
    block_zs: jintArray,
    dst: jlongArray,
) -> jint {
    match biome_getbiome_batch_summary(
        &mut env, iterations, seeds, block_xs, block_ys, block_zs, dst, false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeBiomeGetBiome_optimizedBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    seeds: jlongArray,
    block_xs: jintArray,
    block_ys: jintArray,
    block_zs: jintArray,
    dst: jlongArray,
) -> jint {
    match biome_getbiome_batch_summary(
        &mut env, iterations, seeds, block_xs, block_ys, block_zs, dst, true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSpringFeatureMutablePos_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    requires_below: jbooleanArray,
    rock_count: jintArray,
    hole_count: jintArray,
    dst: jlongArray,
) -> jint {
    match spring_feature_mutable_pos_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        requires_below,
        rock_count,
        hole_count,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSpringFeatureMutablePos_mutableBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    requires_below: jbooleanArray,
    rock_count: jintArray,
    hole_count: jintArray,
    dst: jlongArray,
) -> jint {
    match spring_feature_mutable_pos_batch_summary(
        &mut env,
        iterations,
        xs,
        ys,
        zs,
        requires_below,
        rock_count,
        hole_count,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeJigsawCanAttach_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    orientation_fronts: jintArray,
    orientation_tops: jintArray,
    parent_orientations: jintArray,
    child_orientations: jintArray,
    parent_rollables: jbooleanArray,
    parent_targets: jintArray,
    child_names: jintArray,
    dst: jlongArray,
) -> jint {
    match jigsaw_canattach_batch_summary(
        &mut env,
        iterations,
        orientation_fronts,
        orientation_tops,
        parent_orientations,
        child_orientations,
        parent_rollables,
        parent_targets,
        child_names,
        dst,
        false,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeJigsawCanAttach_optimizedBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    orientation_fronts: jintArray,
    orientation_tops: jintArray,
    parent_orientations: jintArray,
    child_orientations: jintArray,
    parent_rollables: jbooleanArray,
    parent_targets: jintArray,
    child_names: jintArray,
    dst: jlongArray,
) -> jint {
    match jigsaw_canattach_batch_summary(
        &mut env,
        iterations,
        orientation_fronts,
        orientation_tops,
        parent_orientations,
        child_orientations,
        parent_rollables,
        parent_targets,
        child_names,
        dst,
        true,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeJigsawCanAttach_targetFirstBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    orientation_fronts: jintArray,
    orientation_tops: jintArray,
    parent_orientations: jintArray,
    child_orientations: jintArray,
    parent_rollables: jbooleanArray,
    parent_targets: jintArray,
    child_names: jintArray,
    dst: jlongArray,
) -> jint {
    match jigsaw_canattach_batch_summary(
        &mut env,
        iterations,
        orientation_fronts,
        orientation_tops,
        parent_orientations,
        child_orientations,
        parent_rollables,
        parent_targets,
        child_names,
        dst,
        false,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCubicSplineCreate_oldIteratorSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    min_values: jfloatArray,
    max_values: jfloatArray,
    dst: jlongArray,
) -> jint {
    match cubic_spline_create_batch_summary(
        &mut env,
        iterations,
        min_values,
        max_values,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCubicSplineCreate_indexSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    min_values: jfloatArray,
    max_values: jfloatArray,
    dst: jlongArray,
) -> jint {
    match cubic_spline_create_batch_summary(
        &mut env,
        iterations,
        min_values,
        max_values,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeStaticCacheGet_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    min_x: jint,
    min_z: jint,
    size_x: jint,
    size_z: jint,
    values: jintArray,
    dst: jlongArray,
) -> jint {
    match static_cache_get_batch_summary(
        &mut env,
        iterations,
        min_x,
        min_z,
        size_x,
        size_z,
        values,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCarverIteration_foreachSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    set_offsets: jintArray,
    values: jintArray,
    dst: jlongArray,
) -> jint {
    match carver_iteration_batch_summary(
        &mut env,
        iterations,
        set_offsets,
        values,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCarverIteration_indexedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    set_offsets: jintArray,
    values: jintArray,
    dst: jlongArray,
) -> jint {
    match carver_iteration_batch_summary(
        &mut env,
        iterations,
        set_offsets,
        values,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCaveCarverSkip_oldLambdaSummary(
    mut env: JNIEnv,
    _class: JClass,
    carves: jint,
    floor_levels: jdoubleArray,
    relative_x: jdoubleArray,
    relative_y: jdoubleArray,
    relative_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match cave_carver_skip_batch_summary(
        &mut env,
        carves,
        floor_levels,
        relative_x,
        relative_y,
        relative_z,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCaveCarverSkip_reusedCheckerSummary(
    mut env: JNIEnv,
    _class: JClass,
    carves: jint,
    floor_levels: jdoubleArray,
    relative_x: jdoubleArray,
    relative_y: jdoubleArray,
    relative_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match cave_carver_skip_batch_summary(
        &mut env,
        carves,
        floor_levels,
        relative_x,
        relative_y,
        relative_z,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCaveCarverSkip_directHelperSummary(
    mut env: JNIEnv,
    _class: JClass,
    carves: jint,
    floor_levels: jdoubleArray,
    relative_x: jdoubleArray,
    relative_y: jdoubleArray,
    relative_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match cave_carver_skip_batch_summary(
        &mut env,
        carves,
        floor_levels,
        relative_x,
        relative_y,
        relative_z,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseInterpolatorFractions_divisionSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match noise_interpolator_fractions_division_summary(&mut env, iterations, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseInterpolatorFractions_arraySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    cell_width_fractions: jdoubleArray,
    cell_height_fractions: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match noise_interpolator_fractions_array_summary(
        &mut env,
        iterations,
        cell_width_fractions,
        cell_height_fractions,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseInterpolatorSlice_oldJaggedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
) -> jint {
    match noise_interpolator_slice_summary(
        &mut env,
        iterations,
        interpolators,
        cell_count_xz,
        cell_count_y,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseInterpolatorSlice_flatSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
) -> jint {
    match noise_interpolator_slice_summary(
        &mut env,
        iterations,
        interpolators,
        cell_count_xz,
        cell_count_y,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkBlendCache_oldEmptyBlenderSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_blendcache_summary(&mut env, iterations, size_xz, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkBlendCache_newEmptyBlenderSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_blendcache_summary(&mut env, iterations, size_xz, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkFlatCacheContext_oldFalseContextSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_flatcache_context_summary(&mut env, iterations, size_xz, dst, 0) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkFlatCacheContext_newFalseContextSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_flatcache_context_summary(&mut env, iterations, size_xz, dst, 1) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkFlatCacheContext_oldTrueContextSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_flatcache_context_summary(&mut env, iterations, size_xz, dst, 2) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkFlatCacheContext_newTrueContextSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_flatcache_context_summary(&mut env, iterations, size_xz, dst, 3) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkInterpolatorArray_listSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_interpolator_array_summary(
        &mut env,
        iterations,
        interpolators,
        cell_count_xz,
        cell_count_y,
        dst,
        0,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkInterpolatorArray_indexedListSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_interpolator_array_summary(
        &mut env,
        iterations,
        interpolators,
        cell_count_xz,
        cell_count_y,
        dst,
        1,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNoiseChunkInterpolatorArray_arraySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_interpolator_array_summary(
        &mut env,
        iterations,
        interpolators,
        cell_count_xz,
        cell_count_y,
        dst,
        2,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_PaperNativeNoiseChunkWrapCapacity_shapeSummary(
    mut env: JNIEnv,
    _class: JClass,
    entries: jintArray,
    expected_sizes: jintArray,
    load_factors: jfloatArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match noisechunk_wrap_capacity_summary(
        &mut env,
        entries,
        expected_sizes,
        load_factors,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityAp2Fill_oldFlatSummary(
    mut env: JNIEnv,
    _class: JClass,
    length: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_ap2_fill_summary(&mut env, length, iterations, dst, 0) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityAp2Fill_scratchFlatSummary(
    mut env: JNIEnv,
    _class: JClass,
    length: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_ap2_fill_summary(&mut env, length, iterations, dst, 1) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityAp2Fill_oldNestedSummary(
    mut env: JNIEnv,
    _class: JClass,
    length: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_ap2_fill_summary(&mut env, length, iterations, dst, 2) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityAp2Fill_scratchNestedSummary(
    mut env: JNIEnv,
    _class: JClass,
    length: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_ap2_fill_summary(&mut env, length, iterations, dst, 3) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityAp2MinMaxFill_oldSummary(
    mut env: JNIEnv,
    _class: JClass,
    scenario_index: jint,
    length: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_ap2_minmax_fill_summary(&mut env, scenario_index, length, iterations, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeDensityAp2MinMaxFill_newSummary(
    mut env: JNIEnv,
    _class: JClass,
    scenario_index: jint,
    length: jint,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match density_ap2_minmax_fill_summary(&mut env, scenario_index, length, iterations, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeChunkDependencies_oldImmutableListSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match chunk_dependencies_summary(&mut env, iterations, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeChunkDependencies_arraySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match chunk_dependencies_summary(&mut env, iterations, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeOwnableRule_oldStreamSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match ownable_rule_summary(&mut env, iterations, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeOwnableRule_newLoopSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match ownable_rule_summary(&mut env, iterations, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityBoundingBox_oldMakeThenSetSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    widths: jfloatArray,
    heights: jfloatArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match entity_bounding_box_summary(
        &mut env,
        iterations,
        widths,
        heights,
        xs,
        ys,
        zs,
        dst,
        EntityBoundingBoxKind::OldMakeThenSet,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityBoundingBox_directDimensionsSetSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    widths: jfloatArray,
    heights: jfloatArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match entity_bounding_box_summary(
        &mut env,
        iterations,
        widths,
        heights,
        xs,
        ys,
        zs,
        dst,
        EntityBoundingBoxKind::DirectDimensionsSet,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeShiftNoiseDirect_currentDefaultSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
) -> jint {
    match shift_noise_direct_summary(
        &mut env,
        iterations,
        block_x,
        block_y,
        block_z,
        dst,
        ShiftNoiseDirectKind::CurrentDefault,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeShiftNoiseDirect_directDefaultSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
) -> jint {
    match shift_noise_direct_summary(
        &mut env,
        iterations,
        block_x,
        block_y,
        block_z,
        dst,
        ShiftNoiseDirectKind::DirectDefault,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeShiftNoiseDirect_currentASummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
) -> jint {
    match shift_noise_direct_summary(
        &mut env,
        iterations,
        block_x,
        block_y,
        block_z,
        dst,
        ShiftNoiseDirectKind::CurrentA,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeShiftNoiseDirect_directASummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
) -> jint {
    match shift_noise_direct_summary(
        &mut env,
        iterations,
        block_x,
        block_y,
        block_z,
        dst,
        ShiftNoiseDirectKind::DirectA,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeShiftNoiseDirect_currentBSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
) -> jint {
    match shift_noise_direct_summary(
        &mut env,
        iterations,
        block_x,
        block_y,
        block_z,
        dst,
        ShiftNoiseDirectKind::CurrentB,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeShiftNoiseDirect_directBSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
) -> jint {
    match shift_noise_direct_summary(
        &mut env,
        iterations,
        block_x,
        block_y,
        block_z,
        dst,
        ShiftNoiseDirectKind::DirectB,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginNameJoin_stringJoinSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    delimiter: JString,
    dst: jlongArray,
) -> jint {
    match plugin_name_join_summary(
        &mut env,
        iterations,
        names,
        delimiter,
        dst,
        PluginNameJoinKind::StringJoin,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginNameJoin_manualJoinSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    delimiter: JString,
    dst: jlongArray,
) -> jint {
    match plugin_name_join_summary(
        &mut env,
        iterations,
        names,
        delimiter,
        dst,
        PluginNameJoinKind::ManualJoin,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginNameLog_oldTreesetSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    paper_names: jobjectArray,
    bukkit_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match plugin_name_log_summary(
        &mut env,
        iterations,
        paper_names,
        bukkit_names,
        dst,
        PluginNameLogKind::OldTreeset,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginNameLog_newArrayListSortSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    paper_names: jobjectArray,
    bukkit_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match plugin_name_log_summary(
        &mut env,
        iterations,
        paper_names,
        bukkit_names,
        dst,
        PluginNameLogKind::NewArrayListSort,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginStartupRollup_oldSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    delimiter: JString,
    paper_names: jobjectArray,
    bukkit_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match plugin_startup_rollup_summary(
        &mut env,
        iterations,
        names,
        delimiter,
        paper_names,
        bukkit_names,
        dst,
        PluginStartupRollupKind::OldTreesetStringJoin,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginStartupRollup_newSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    delimiter: JString,
    paper_names: jobjectArray,
    bukkit_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match plugin_startup_rollup_summary(
        &mut env,
        iterations,
        names,
        delimiter,
        paper_names,
        bukkit_names,
        dst,
        PluginStartupRollupKind::NewArrayListSortManualJoin,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeObfHelperMaps_oldStreamDefaultSummary(
    mut env: JNIEnv,
    _class: JClass,
    class_mapped_names: jobjectArray,
    class_original_names: jobjectArray,
    method_counts: jintArray,
    field_counts: jintArray,
    method_mapped_names: jobjectArray,
    method_mapped_descriptors: jobjectArray,
    method_original_names: jobjectArray,
    method_original_descriptors: jobjectArray,
    field_mapped_names: jobjectArray,
    field_original_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match obfhelper_maps_summary(
        &mut env,
        class_mapped_names,
        class_original_names,
        method_counts,
        field_counts,
        method_mapped_names,
        method_mapped_descriptors,
        method_original_names,
        method_original_descriptors,
        field_mapped_names,
        field_original_names,
        dst,
        ObfHelperMapsKind::OldStreamDefault,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeObfHelperMaps_directMapsSummary(
    mut env: JNIEnv,
    _class: JClass,
    class_mapped_names: jobjectArray,
    class_original_names: jobjectArray,
    method_counts: jintArray,
    field_counts: jintArray,
    method_mapped_names: jobjectArray,
    method_mapped_descriptors: jobjectArray,
    method_original_names: jobjectArray,
    method_original_descriptors: jobjectArray,
    field_mapped_names: jobjectArray,
    field_original_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match obfhelper_maps_summary(
        &mut env,
        class_mapped_names,
        class_original_names,
        method_counts,
        field_counts,
        method_mapped_names,
        method_mapped_descriptors,
        method_original_names,
        method_original_descriptors,
        field_mapped_names,
        field_original_names,
        dst,
        ObfHelperMapsKind::DirectMaps,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeObfHelperMaps_presizedStringPoolSummary(
    mut env: JNIEnv,
    _class: JClass,
    class_mapped_names: jobjectArray,
    class_original_names: jobjectArray,
    method_counts: jintArray,
    field_counts: jintArray,
    method_mapped_names: jobjectArray,
    method_mapped_descriptors: jobjectArray,
    method_original_names: jobjectArray,
    method_original_descriptors: jobjectArray,
    field_mapped_names: jobjectArray,
    field_original_names: jobjectArray,
    dst: jlongArray,
) -> jint {
    match obfhelper_maps_summary(
        &mut env,
        class_mapped_names,
        class_original_names,
        method_counts,
        field_counts,
        method_mapped_names,
        method_mapped_descriptors,
        method_original_names,
        method_original_descriptors,
        field_mapped_names,
        field_original_names,
        dst,
        ObfHelperMapsKind::PresizedStringPool,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginMetaDependency_oldStreamSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    required: jbooleanArray,
    join_classpath: jbooleanArray,
    load: jintArray,
    dst: jlongArray,
) -> jint {
    match plugin_meta_dependency_summary(
        &mut env,
        iterations,
        names,
        required,
        join_classpath,
        load,
        dst,
        PluginMetaDependencyKind::OldStream,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginMetaDependency_newLoopSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    required: jbooleanArray,
    join_classpath: jbooleanArray,
    load: jintArray,
    dst: jlongArray,
) -> jint {
    match plugin_meta_dependency_summary(
        &mut env,
        iterations,
        names,
        required,
        join_classpath,
        load,
        dst,
        PluginMetaDependencyKind::NewLoop,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginMetaDependency_cachedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    names: jobjectArray,
    required: jbooleanArray,
    join_classpath: jbooleanArray,
    load: jintArray,
    dst: jlongArray,
) -> jint {
    match plugin_meta_dependency_summary(
        &mut env,
        iterations,
        names,
        required,
        join_classpath,
        load,
        dst,
        PluginMetaDependencyKind::Cached,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginClassLoaderGroup_oldLookupSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    loader_names: jobjectArray,
    result_lengths: jintArray,
    requester_index: jint,
    query: JString,
    dst: jlongArray,
) -> jint {
    match plugin_classloader_group_summary(
        &mut env,
        iterations,
        loader_names,
        result_lengths,
        requester_index,
        query,
        dst,
        PluginClassLoaderGroupKind::OldLookup,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginClassLoaderGroup_skipRequesterSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    loader_names: jobjectArray,
    result_lengths: jintArray,
    requester_index: jint,
    query: JString,
    dst: jlongArray,
) -> jint {
    match plugin_classloader_group_summary(
        &mut env,
        iterations,
        loader_names,
        result_lengths,
        requester_index,
        query,
        dst,
        PluginClassLoaderGroupKind::SkipRequester,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginLoadingAllocation_oldDefaultCapacitySetupSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match plugin_loading_allocation_summary(
        &mut env,
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        dst,
        PluginLoadingAllocationKind::OldDefaultCapacitySetup,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginLoadingAllocation_newPresizedSetupSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match plugin_loading_allocation_summary(
        &mut env,
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        dst,
        PluginLoadingAllocationKind::NewPresizedSetup,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginLoadingAllocation_oldEagerMissingSetSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match plugin_loading_allocation_summary(
        &mut env,
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        dst,
        PluginLoadingAllocationKind::OldEagerMissingSet,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginLoadingAllocation_newLazyMissingSetSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match plugin_loading_allocation_summary(
        &mut env,
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        dst,
        PluginLoadingAllocationKind::NewLazyMissingSet,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginLoadingAllocation_oldEagerValidateSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match plugin_loading_allocation_summary(
        &mut env,
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        dst,
        PluginLoadingAllocationKind::OldEagerValidate,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginLoadingAllocation_newLazyValidateSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match plugin_loading_allocation_summary(
        &mut env,
        iterations,
        provider_names,
        provided_aliases,
        dependencies,
        dependencies_per_provider,
        dst,
        PluginLoadingAllocationKind::NewLazyValidate,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLegacyProvidedAliasRemoval_oldValuesRemoveIfSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    aliases: jobjectArray,
    aliases_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match legacy_provided_alias_removal_summary(
        &mut env,
        iterations,
        provider_names,
        aliases,
        aliases_per_provider,
        dst,
        LegacyProvidedAliasRemovalKind::OldValuesRemoveIf,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLegacyProvidedAliasRemoval_newReverseAliasRemoveSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    aliases: jobjectArray,
    aliases_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match legacy_provided_alias_removal_summary(
        &mut env,
        iterations,
        provider_names,
        aliases,
        aliases_per_provider,
        dst,
        LegacyProvidedAliasRemovalKind::NewReverseAliasRemove,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSpigotLoadOrderDependency_oldLoadAfterBuildSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    load_after: jobjectArray,
    dst: jlongArray,
) -> jint {
    match spigot_load_order_dependency_build_summary(
        &mut env,
        iterations,
        load_after,
        dst,
        SpigotLoadOrderDependencyKind::OldLoadAfterBuild,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSpigotLoadOrderDependency_newLoadAfterBuildSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    load_after: jobjectArray,
    dst: jlongArray,
) -> jint {
    match spigot_load_order_dependency_build_summary(
        &mut env,
        iterations,
        load_after,
        dst,
        SpigotLoadOrderDependencyKind::NewLoadAfterBuild,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSpigotLoadOrderDependency_oldRemovedCountSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    hard_dependencies: jobjectArray,
    soft_dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match spigot_load_order_dependency_removed_count_summary(
        &mut env,
        iterations,
        provider_names,
        hard_dependencies,
        soft_dependencies,
        dependencies_per_provider,
        dst,
        SpigotLoadOrderDependencyKind::OldRemovedCount,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeSpigotLoadOrderDependency_newRemovedCountSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    provider_names: jobjectArray,
    hard_dependencies: jobjectArray,
    soft_dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
) -> jint {
    match spigot_load_order_dependency_removed_count_summary(
        &mut env,
        iterations,
        provider_names,
        hard_dependencies,
        soft_dependencies,
        dependencies_per_provider,
        dst,
        SpigotLoadOrderDependencyKind::NewRemovedCount,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTopographicGraphSortCapacity_oldDefaultCapacitySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    successor_offsets: jintArray,
    successors: jintArray,
    in_degree: jintArray,
    dst: jlongArray,
) -> jint {
    match topographic_graph_sort_capacity_summary(
        &mut env,
        iterations,
        successor_offsets,
        successors,
        in_degree,
        dst,
        TopographicGraphSortCapacityKind::OldDefaultCapacity,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeTopographicGraphSortCapacity_newPresizedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    successor_offsets: jintArray,
    successors: jintArray,
    in_degree: jintArray,
    dst: jlongArray,
) -> jint {
    match topographic_graph_sort_capacity_summary(
        &mut env,
        iterations,
        successor_offsets,
        successors,
        in_degree,
        dst,
        TopographicGraphSortCapacityKind::NewPresized,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperIndexCleanup_oldEagerCleanupSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    input_paths: jobjectArray,
    input_hashes: jobjectArray,
    remapped_hashes: jobjectArray,
    remapped_paths: jobjectArray,
    skipped_hashes: jobjectArray,
    dst: jlongArray,
) -> jint {
    match remapper_index_cleanup_summary(
        &mut env,
        iterations,
        input_paths,
        input_hashes,
        remapped_hashes,
        remapped_paths,
        skipped_hashes,
        dst,
        RemapperIndexCleanupKind::OldEagerCleanup,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperIndexCleanup_newLazyCleanupSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    input_paths: jobjectArray,
    input_hashes: jobjectArray,
    remapped_hashes: jobjectArray,
    remapped_paths: jobjectArray,
    skipped_hashes: jobjectArray,
    dst: jlongArray,
) -> jint {
    match remapper_index_cleanup_summary(
        &mut env,
        iterations,
        input_paths,
        input_hashes,
        remapped_hashes,
        remapped_paths,
        skipped_hashes,
        dst,
        RemapperIndexCleanupKind::NewLazyCleanup,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperSkipHashes_oldStreamSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    content: JString,
    dst: jlongArray,
) -> jint {
    match remapper_skip_hashes_summary(
        &mut env,
        iterations,
        content,
        dst,
        RemapperSkipHashesKind::OldStream,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeRemapperSkipHashes_newLoopSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    content: JString,
    dst: jlongArray,
) -> jint {
    match remapper_skip_hashes_summary(
        &mut env,
        iterations,
        content,
        dst,
        RemapperSkipHashesKind::NewLoop,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginDirectoryScan_oldWalkDepth1Summary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    directory: JString,
    dst: jlongArray,
) -> jint {
    match plugin_directory_scan_summary(
        &mut env,
        iterations,
        directory,
        dst,
        PluginDirectoryScanKind::OldWalkDepth1,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginDirectoryScan_newListSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    directory: JString,
    dst: jlongArray,
) -> jint {
    match plugin_directory_scan_summary(
        &mut env,
        iterations,
        directory,
        dst,
        PluginDirectoryScanKind::NewList,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePluginDirectoryScan_directoryStreamSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    directory: JString,
    dst: jlongArray,
) -> jint {
    match plugin_directory_scan_summary(
        &mut env,
        iterations,
        directory,
        dst,
        PluginDirectoryScanKind::DirectoryStream,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeMarkerCache_oldSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    roots: jint,
    depth: jint,
    dst: jlongArray,
) -> jint {
    match marker_cache_summary(
        &mut env,
        iterations,
        roots,
        depth,
        dst,
        MarkerCacheKind::Old,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeMarkerCache_cachedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    roots: jint,
    depth: jint,
    dst: jlongArray,
) -> jint {
    match marker_cache_summary(
        &mut env,
        iterations,
        roots,
        depth,
        dst,
        MarkerCacheKind::Cached,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeChunkExpireCount_hotSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    section_bits: jint,
    chunk_bits: jint,
    kind: jint,
    dst: jlongArray,
) -> jint {
    match chunk_expire_count_summary(
        &mut env,
        iterations,
        section_bits,
        chunk_bits,
        kind,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeChunkExpireCount_coldSummary(
    mut env: JNIEnv,
    _class: JClass,
    section_bits: jint,
    chunk_bits: jint,
    kind: jint,
    dst: jlongArray,
) -> jint {
    match chunk_expire_count_summary(&mut env, 0, section_bits, chunk_bits, kind, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_emptyCurrentSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::CurrentEmpty,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_emptyGuardedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::GuardedEmpty,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_emptyCandidateSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::CandidateEmpty,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_emptyChunkMapCandidateSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::ChunkMapCandidateEmpty,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_populatedCurrentSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::CurrentPopulated,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_populatedGuardedSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::GuardedPopulated,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_populatedCandidateSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::CandidatePopulated,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeCraftPlayerCanSee_populatedChunkMapCandidateSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match craftplayer_cansee_summary(
        &mut env,
        iterations,
        CraftPlayerCanSeeKind::ChunkMapCandidatePopulated,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLevelChunkHeightmap_oldFourUpdateSummary(
    mut env: JNIEnv,
    _class: JClass,
    batches: jint,
    dst: jlongArray,
) -> jint {
    match levelchunk_heightmap_summary(
        &mut env,
        batches,
        dst,
        LevelChunkHeightmapKind::OldFourUpdate,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeLevelChunkHeightmap_newCombinedUpdateSummary(
    mut env: JNIEnv,
    _class: JClass,
    batches: jint,
    dst: jlongArray,
) -> jint {
    match levelchunk_heightmap_summary(
        &mut env,
        batches,
        dst,
        LevelChunkHeightmapKind::NewCombinedUpdate,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNearbyPlayerMap_defaultCapacitySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    player_count: jint,
    dst: jlongArray,
) -> jint {
    match nearby_player_map_capacity_summary(
        &mut env,
        iterations,
        player_count,
        dst,
        NearbyPlayerMapCapacityKind::Default,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNearbyPlayerMap_presizedCapacitySummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    player_count: jint,
    dst: jlongArray,
) -> jint {
    match nearby_player_map_capacity_summary(
        &mut env,
        iterations,
        player_count,
        dst,
        NearbyPlayerMapCapacityKind::Presized,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointDistanceGuard_oldAtOrBeyondRangeSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    source_x: jdoubleArray,
    source_y: jdoubleArray,
    source_z: jdoubleArray,
    receiver_x: jdoubleArray,
    receiver_y: jdoubleArray,
    receiver_z: jdoubleArray,
    range: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match waypoint_distance_guard_summary(
        &mut env,
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        range,
        dst,
        WaypointDistanceGuardKind::OldAtOrBeyondRange,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointDistanceGuard_guardedAtOrBeyondRangeSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    source_x: jdoubleArray,
    source_y: jdoubleArray,
    source_z: jdoubleArray,
    receiver_x: jdoubleArray,
    receiver_y: jdoubleArray,
    receiver_z: jdoubleArray,
    range: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match waypoint_distance_guard_summary(
        &mut env,
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        range,
        dst,
        WaypointDistanceGuardKind::GuardedAtOrBeyondRange,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointDistanceGuard_oldReallyFarSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    source_x: jdoubleArray,
    source_y: jdoubleArray,
    source_z: jdoubleArray,
    receiver_x: jdoubleArray,
    receiver_y: jdoubleArray,
    receiver_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match waypoint_distance_guard_summary(
        &mut env,
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        std::ptr::null_mut(),
        dst,
        WaypointDistanceGuardKind::OldReallyFar,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeWaypointDistanceGuard_guardedReallyFarSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    source_x: jdoubleArray,
    source_y: jdoubleArray,
    source_z: jdoubleArray,
    receiver_x: jdoubleArray,
    receiver_y: jdoubleArray,
    receiver_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match waypoint_distance_guard_summary(
        &mut env,
        iterations,
        source_x,
        source_y,
        source_z,
        receiver_x,
        receiver_y,
        receiver_z,
        std::ptr::null_mut(),
        dst,
        WaypointDistanceGuardKind::GuardedReallyFar,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityLookupStatus_oldStatusSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match entity_lookup_status_summary(&mut env, iterations, dst, EntityLookupStatusKind::OldStatus) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityLookupStatus_directStatusSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match entity_lookup_status_summary(&mut env, iterations, dst, EntityLookupStatusKind::DirectStatus) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityLookupStatus_oldAccessibleSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match entity_lookup_status_summary(&mut env, iterations, dst, EntityLookupStatusKind::OldAccessible) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeEntityLookupStatus_directAccessibleSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match entity_lookup_status_summary(&mut env, iterations, dst, EntityLookupStatusKind::DirectAccessible) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeStaticCacheGet_newBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    min_x: jint,
    min_z: jint,
    size_x: jint,
    size_z: jint,
    values: jintArray,
    dst: jlongArray,
) -> jint {
    match static_cache_get_batch_summary(
        &mut env,
        iterations,
        min_x,
        min_z,
        size_x,
        size_z,
        values,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeServerEntityDeltaIdentity_oldDistanceSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    current_x: jdoubleArray,
    current_y: jdoubleArray,
    current_z: jdoubleArray,
    last_x: jdoubleArray,
    last_y: jdoubleArray,
    last_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match serverentity_delta_identity_batch_summary(
        &mut env,
        iterations,
        None,
        current_x,
        current_y,
        current_z,
        last_x,
        last_y,
        last_z,
        dst,
        false,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeServerEntityDeltaIdentity_identityGuardSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    same_identity: jbyteArray,
    current_x: jdoubleArray,
    current_y: jdoubleArray,
    current_z: jdoubleArray,
    last_x: jdoubleArray,
    last_y: jdoubleArray,
    last_z: jdoubleArray,
    dst: jlongArray,
) -> jint {
    match serverentity_delta_identity_batch_summary(
        &mut env,
        iterations,
        Some(same_identity),
        current_x,
        current_y,
        current_z,
        last_x,
        last_y,
        last_z,
        dst,
        true,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeBlendedNoise_oldBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match blended_noise_batch_summary(&mut env, iterations, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeBlendedNoise_cachedBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match blended_noise_batch_summary(&mut env, iterations, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_noiseBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    xo: jni::sys::jdouble,
    yo: jni::sys::jdouble,
    zo: jni::sys::jdouble,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_batch_summary(
        &mut env,
        permutation,
        xs,
        ys,
        zs,
        y_scales,
        y_maxes,
        xo,
        yo,
        zo,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_buildHandle(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    xo: jni::sys::jdouble,
    yo: jni::sys::jdouble,
    zo: jni::sys::jdouble,
) -> jlong {
    improved_noise_build_handle(&mut env, permutation, xo, yo, zo).unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_freeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    improved_noise_free_handle(handle);
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_noise(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    y_max: jni::sys::jdouble,
) -> jni::sys::jdouble {
    improved_noise_noise(handle, x, y, z, y_scale, y_max)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_noiseNoYScale(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
) -> jni::sys::jdouble {
    improved_noise_noise_no_y_scale(handle, x, y, z)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_fill(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    output: jdoubleArray,
) -> jint {
    match improved_noise_fill(&mut env, handle, xs, ys, zs, y_scales, y_maxes, output) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoise_fillNoYScale(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    output: jdoubleArray,
) -> jint {
    match improved_noise_fill_no_y_scale(&mut env, handle, xs, ys, zs, output) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeImprovedNoise_nativeBuildHandle(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    xo: jni::sys::jdouble,
    yo: jni::sys::jdouble,
    zo: jni::sys::jdouble,
) -> jlong {
    improved_noise_build_handle(&mut env, permutation, xo, yo, zo).unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeImprovedNoise_nativeFreeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    improved_noise_free_handle(handle);
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeImprovedNoise_nativeNoise(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    y_max: jni::sys::jdouble,
) -> jni::sys::jdouble {
    improved_noise_noise(handle, x, y, z, y_scale, y_max)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeImprovedNoise_nativeNoiseNoYScale(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
) -> jni::sys::jdouble {
    improved_noise_noise_no_y_scale(handle, x, y, z)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeImprovedNoise_nativeFill(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    output: jdoubleArray,
) -> jint {
    match improved_noise_fill(&mut env, handle, xs, ys, zs, y_scales, y_maxes, output) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeImprovedNoise_nativeFillNoYScale(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    output: jdoubleArray,
) -> jint {
    match improved_noise_fill_no_y_scale(&mut env, handle, xs, ys, zs, output) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseInline_oldPMethodSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_inline_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        improved_noise_inline::ImprovedNoiseInlineKind::OldPMethod,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseInline_inlineByteAccessSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_inline_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        improved_noise_inline::ImprovedNoiseInlineKind::InlineByteAccess,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseInline_flatGradientSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_inline_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        improved_noise_inline::ImprovedNoiseInlineKind::FlatGradient,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseInline_arithmeticSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_inline_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        improved_noise_inline::ImprovedNoiseInlineKind::Arithmetic,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseInline_switchGradientSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_inline_summary(
        &mut env,
        permutation,
        iterations,
        dst,
        improved_noise_inline::ImprovedNoiseInlineKind::SwitchGradient,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseDerivative_oldDerivativeSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    grid_x: jintArray,
    grid_y: jintArray,
    grid_z: jintArray,
    delta_x: jdoubleArray,
    delta_y: jdoubleArray,
    delta_z: jdoubleArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_derivative_summary(
        &mut env,
        permutation,
        grid_x,
        grid_y,
        grid_z,
        delta_x,
        delta_y,
        delta_z,
        iterations,
        dst,
        improved_noise_derivative::ImprovedNoiseDerivativeKind::Old,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseDerivative_inlineDerivativeSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    grid_x: jintArray,
    grid_y: jintArray,
    grid_z: jintArray,
    delta_x: jdoubleArray,
    delta_y: jdoubleArray,
    delta_z: jdoubleArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_derivative_summary(
        &mut env,
        permutation,
        grid_x,
        grid_y,
        grid_z,
        delta_x,
        delta_y,
        delta_z,
        iterations,
        dst,
        improved_noise_derivative::ImprovedNoiseDerivativeKind::Inline,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseDerivative_intTableDerivativeSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    grid_x: jintArray,
    grid_y: jintArray,
    grid_z: jintArray,
    delta_x: jdoubleArray,
    delta_y: jdoubleArray,
    delta_z: jdoubleArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_derivative_summary(
        &mut env,
        permutation,
        grid_x,
        grid_y,
        grid_z,
        delta_x,
        delta_y,
        delta_z,
        iterations,
        dst,
        improved_noise_derivative::ImprovedNoiseDerivativeKind::IntTable,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeImprovedNoiseDerivative_flatGradientDerivativeSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutation: jbyteArray,
    grid_x: jintArray,
    grid_y: jintArray,
    grid_z: jintArray,
    delta_x: jdoubleArray,
    delta_y: jdoubleArray,
    delta_z: jdoubleArray,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match improved_noise_derivative_summary(
        &mut env,
        permutation,
        grid_x,
        grid_y,
        grid_z,
        delta_x,
        delta_y,
        delta_z,
        iterations,
        dst,
        improved_noise_derivative::ImprovedNoiseDerivativeKind::FlatGradient,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeHashPath_readAllSummary(
    mut env: JNIEnv,
    _class: JClass,
    paths: jobjectArray,
    dst: jlongArray,
) -> jint {
    match hash_path_summary_jni(&mut env, paths, 0, dst, false) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeHashPath_streamingSummary(
    mut env: JNIEnv,
    _class: JClass,
    paths: jobjectArray,
    buffer_size: jint,
    dst: jlongArray,
) -> jint {
    match hash_path_summary_jni(&mut env, paths, buffer_size, dst, true) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativeNbtCompoundMapCapacity_parseCapacitySummary(
    mut env: JNIEnv,
    _class: JClass,
    data: jbyteArray,
    offsets: jintArray,
    lengths: jintArray,
    capacity: jint,
    dst: jlongArray,
) -> jint {
    match nbt_compound_map_capacity_summary(&mut env, data, offsets, lengths, capacity, dst) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePerlinNoise_getValueBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutations: jbyteArray,
    active: jbyteArray,
    y_origins: jdoubleArray,
    amplitudes: jdoubleArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    use_fixed_ys: jbyteArray,
    lowest_freq_input_factor: jni::sys::jdouble,
    lowest_freq_value_factor: jni::sys::jdouble,
    iterations: jint,
    dst: jlongArray,
) -> jint {
    match perlin_noise_get_value_batch_summary(
        &mut env,
        permutations,
        active,
        y_origins,
        amplitudes,
        xs,
        ys,
        zs,
        y_scales,
        y_maxes,
        use_fixed_ys,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
        iterations,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativePerlinNoise_nativeBuildHandle(
    mut env: JNIEnv,
    _class: JClass,
    permutations: jbyteArray,
    active: jbyteArray,
    x_origins: jdoubleArray,
    y_origins: jdoubleArray,
    z_origins: jdoubleArray,
    amplitudes: jdoubleArray,
    lowest_freq_input_factor: jni::sys::jdouble,
    lowest_freq_value_factor: jni::sys::jdouble,
) -> jlong {
    perlin_noise_build_handle(
        &mut env,
        permutations,
        active,
        x_origins,
        y_origins,
        z_origins,
        amplitudes,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
    )
    .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativePerlinNoise_nativeFreeHandle(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    perlin_noise_free_handle(handle);
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativePerlinNoise_nativeGetValue(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    y_max: jni::sys::jdouble,
    use_fixed_y: jboolean,
) -> jni::sys::jdouble {
    perlin_noise_get_value(handle, x, y, z, y_scale, y_max, use_fixed_y)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativePerlinNoise_nativeGetValueNoYScale(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
) -> jni::sys::jdouble {
    perlin_noise_get_value_no_y_scale(handle, x, y, z)
}

#[no_mangle]
pub extern "system" fn Java_PaperNativePerlinGetValue_getValueVariantBatchSummary(
    mut env: JNIEnv,
    _class: JClass,
    permutations: jbyteArray,
    active: jbyteArray,
    y_origins: jdoubleArray,
    amplitudes: jdoubleArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    lowest_freq_input_factor: jdouble,
    lowest_freq_value_factor: jdouble,
    iterations: jint,
    variant: jint,
    dst: jlongArray,
) -> jint {
    match perlin_noise_get_value_variant_batch_summary(
        &mut env,
        permutations,
        active,
        y_origins,
        amplitudes,
        xs,
        ys,
        zs,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
        iterations,
        variant,
        dst,
    ) {
        Ok(written) => written as jint,
        Err(code) => code,
    }
}

fn cubic_spline_create_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    min_values: jfloatArray,
    max_values: jfloatArray,
    dst: jlongArray,
    index: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let min_values_len = env.get_array_length(min_values).map_err(|_| -2)? as usize;
    let max_values_len = env.get_array_length(max_values).map_err(|_| -3)? as usize;
    if min_values_len != max_values_len {
        return Err(-4);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < cubic_spline_create::SUMMARY_FIELDS {
        return Err(-(cubic_spline_create::SUMMARY_FIELDS as jint));
    }

    let min_values_elements = env
        .get_float_array_elements(min_values, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let max_values_elements = env
        .get_float_array_elements(max_values, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let min_values = unsafe { std::slice::from_raw_parts(min_values_elements.as_ptr(), min_values_len) };
    let max_values = unsafe { std::slice::from_raw_parts(max_values_elements.as_ptr(), max_values_len) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -8)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            cubic_spline_create::SUMMARY_FIELDS,
        )
    };

    let summary = if index {
        cubic_spline_create::index_summary(iterations, min_values, max_values)
    } else {
        cubic_spline_create::old_iterator_summary(iterations, min_values, max_values)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sink_bits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_pair_bits as i64;

    Ok(cubic_spline_create::SUMMARY_FIELDS)
}

fn carver_iteration_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    set_offsets: jintArray,
    values: jintArray,
    dst: jlongArray,
    indexed: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let set_offsets_len = env.get_array_length(set_offsets).map_err(|_| -2)? as usize;
    if set_offsets_len < 2 {
        return Err(-3);
    }

    let values_len = env.get_array_length(values).map_err(|_| -4)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < carver_iteration::SUMMARY_FIELDS {
        return Err(-(carver_iteration::SUMMARY_FIELDS as jint));
    }

    let set_offsets_elements = env
        .get_int_array_elements(set_offsets, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let values_elements = env
        .get_int_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let set_offsets = unsafe {
        std::slice::from_raw_parts(set_offsets_elements.as_ptr(), set_offsets_len)
    };
    let values = unsafe { std::slice::from_raw_parts(values_elements.as_ptr(), values_len) };

    if set_offsets[0] != 0 {
        return Err(-8);
    }
    let mut previous = 0;
    for offset in set_offsets {
        if *offset < previous {
            return Err(-9);
        }
        previous = *offset;
    }
    if usize::try_from(previous).map_err(|_| -10)? != values_len {
        return Err(-11);
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            carver_iteration::SUMMARY_FIELDS,
        )
    };

    let summary = if indexed {
        carver_iteration::indexed_summary(iterations, set_offsets, values)
    } else {
        carver_iteration::foreach_summary(iterations, set_offsets, values)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.checksum as i64;

    Ok(carver_iteration::SUMMARY_FIELDS)
}

fn cave_carver_skip_batch_summary(
    env: &mut JNIEnv,
    carves: jint,
    floor_levels: jdoubleArray,
    relative_x: jdoubleArray,
    relative_y: jdoubleArray,
    relative_z: jdoubleArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let carves = usize::try_from(carves).map_err(|_| -1)?;
    let floor_levels_len = env.get_array_length(floor_levels).map_err(|_| -2)? as usize;
    let relative_x_len = env.get_array_length(relative_x).map_err(|_| -3)? as usize;
    let relative_y_len = env.get_array_length(relative_y).map_err(|_| -4)? as usize;
    let relative_z_len = env.get_array_length(relative_z).map_err(|_| -5)? as usize;
    if relative_y_len != relative_x_len || relative_z_len != relative_x_len {
        return Err(-6);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -7)? as usize;
    if dst_len < cave_carver_skip::SUMMARY_FIELDS {
        return Err(-(cave_carver_skip::SUMMARY_FIELDS as jint));
    }

    let floor_levels_elements = env
        .get_double_array_elements(floor_levels, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let relative_x_elements = env
        .get_double_array_elements(relative_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let relative_y_elements = env
        .get_double_array_elements(relative_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let relative_z_elements = env
        .get_double_array_elements(relative_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;

    let floor_levels = unsafe {
        std::slice::from_raw_parts(floor_levels_elements.as_ptr(), floor_levels_len)
    };
    let relative_x = unsafe {
        std::slice::from_raw_parts(relative_x_elements.as_ptr(), relative_x_len)
    };
    let relative_y = unsafe {
        std::slice::from_raw_parts(relative_y_elements.as_ptr(), relative_y_len)
    };
    let relative_z = unsafe {
        std::slice::from_raw_parts(relative_z_elements.as_ptr(), relative_z_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            cave_carver_skip::SUMMARY_FIELDS,
        )
    };

    let summary = cave_carver_skip::direct_helper_summary(
        carves,
        floor_levels,
        relative_x,
        relative_y,
        relative_z,
    );

    dst[0] = summary.count as i64;
    dst[1] = summary.guard as i64;

    Ok(cave_carver_skip::SUMMARY_FIELDS)
}

fn noise_interpolator_fractions_division_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < noise_interpolator_fractions::SUMMARY_FIELDS {
        return Err(-(noise_interpolator_fractions::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noise_interpolator_fractions::SUMMARY_FIELDS,
        )
    };

    let summary = noise_interpolator_fractions::division_summary(iterations);
    dst[0] = summary.count as i64;
    dst[1] = summary.sink_bits as i64;
    dst[2] = summary.checksum as i64;

    Ok(noise_interpolator_fractions::SUMMARY_FIELDS)
}

fn noise_interpolator_fractions_array_summary(
    env: &mut JNIEnv,
    iterations: jint,
    cell_width_fractions: jdoubleArray,
    cell_height_fractions: jdoubleArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let cell_width_fractions_len = env.get_array_length(cell_width_fractions).map_err(|_| -2)? as usize;
    let cell_height_fractions_len = env.get_array_length(cell_height_fractions).map_err(|_| -3)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if cell_width_fractions_len == 0 || cell_height_fractions_len == 0 {
        return Err(-5);
    }
    if dst_len < noise_interpolator_fractions::SUMMARY_FIELDS {
        return Err(-(noise_interpolator_fractions::SUMMARY_FIELDS as jint));
    }

    let cell_width_fractions_elements = env
        .get_double_array_elements(cell_width_fractions, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let cell_height_fractions_elements = env
        .get_double_array_elements(cell_height_fractions, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -8)?;

    let cell_width_fractions = unsafe {
        std::slice::from_raw_parts(cell_width_fractions_elements.as_ptr(), cell_width_fractions_len)
    };
    let cell_height_fractions = unsafe {
        std::slice::from_raw_parts(cell_height_fractions_elements.as_ptr(), cell_height_fractions_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noise_interpolator_fractions::SUMMARY_FIELDS,
        )
    };

    let summary = noise_interpolator_fractions::array_summary(
        iterations,
        cell_width_fractions,
        cell_height_fractions,
    );

    dst[0] = summary.count as i64;
    dst[1] = summary.sink_bits as i64;
    dst[2] = summary.checksum as i64;

    Ok(noise_interpolator_fractions::SUMMARY_FIELDS)
}

fn noise_interpolator_slice_summary(
    env: &mut JNIEnv,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
    old_jagged: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let interpolators = usize::try_from(interpolators).map_err(|_| -2)?;
    let cell_count_xz = usize::try_from(cell_count_xz).map_err(|_| -3)?;
    let cell_count_y = usize::try_from(cell_count_y).map_err(|_| -4)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < noise_interpolator_slice::SUMMARY_FIELDS {
        return Err(-(noise_interpolator_slice::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noise_interpolator_slice::SUMMARY_FIELDS,
        )
    };

    let summary = if old_jagged {
        noise_interpolator_slice::old_jagged_summary(iterations, interpolators, cell_count_xz, cell_count_y)
    } else {
        noise_interpolator_slice::flat_summary(iterations, interpolators, cell_count_xz, cell_count_y)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.selected as i64;
    dst[2] = summary.sink_bits as i64;

    Ok(noise_interpolator_slice::SUMMARY_FIELDS)
}

fn noisechunk_blendcache_summary(
    env: &mut JNIEnv,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
    old: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let size_xz = usize::try_from(size_xz).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < noisechunk_blendcache::SUMMARY_FIELDS {
        return Err(-(noisechunk_blendcache::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noisechunk_blendcache::SUMMARY_FIELDS,
        )
    };

    let summary = if old {
        noisechunk_blendcache::old_empty_blender_summary(iterations, size_xz)
    } else {
        noisechunk_blendcache::new_empty_blender_summary(iterations, size_xz)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.selected as i64;
    dst[2] = summary.sink_bits as i64;

    Ok(noisechunk_blendcache::SUMMARY_FIELDS)
}

fn noisechunk_flatcache_context_summary(
    env: &mut JNIEnv,
    iterations: jint,
    size_xz: jint,
    dst: jlongArray,
    mode: u8,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let size_xz = usize::try_from(size_xz).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < noisechunk_flatcache_context::SUMMARY_FIELDS {
        return Err(-(noisechunk_flatcache_context::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noisechunk_flatcache_context::SUMMARY_FIELDS,
        )
    };

    let summary = match mode {
        0 => noisechunk_flatcache_context::old_false_context_summary(iterations, size_xz),
        1 => noisechunk_flatcache_context::new_false_context_summary(iterations, size_xz),
        2 => noisechunk_flatcache_context::old_true_context_summary(iterations, size_xz),
        3 => noisechunk_flatcache_context::new_true_context_summary(iterations, size_xz),
        _ => return Err(-5),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.selected as i64;
    dst[2] = summary.sink_bits as i64;

    Ok(noisechunk_flatcache_context::SUMMARY_FIELDS)
}

fn noisechunk_interpolator_array_summary(
    env: &mut JNIEnv,
    iterations: jint,
    interpolators: jint,
    cell_count_xz: jint,
    cell_count_y: jint,
    dst: jlongArray,
    variant: u8,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let interpolators = usize::try_from(interpolators).map_err(|_| -2)?;
    let cell_count_xz = usize::try_from(cell_count_xz).map_err(|_| -3)?;
    let cell_count_y = usize::try_from(cell_count_y).map_err(|_| -4)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < noisechunk_interpolator_array::SUMMARY_FIELDS {
        return Err(-(noisechunk_interpolator_array::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noisechunk_interpolator_array::SUMMARY_FIELDS,
        )
    };

    let summary = match variant {
        0 => noisechunk_interpolator_array::list_summary(iterations, interpolators, cell_count_xz, cell_count_y),
        1 => noisechunk_interpolator_array::indexed_list_summary(iterations, interpolators, cell_count_xz, cell_count_y),
        2 => noisechunk_interpolator_array::array_summary(iterations, interpolators, cell_count_xz, cell_count_y),
        _ => return Err(-7),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.selected as i64;
    dst[2] = summary.sink_bits as i64;

    Ok(noisechunk_interpolator_array::SUMMARY_FIELDS)
}

fn noisechunk_wrap_capacity_summary(
    env: &mut JNIEnv,
    entries: jintArray,
    expected_sizes: jintArray,
    load_factors: jfloatArray,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let entries_len = env.get_array_length(entries).map_err(|_| -1)? as usize;
    let expected_len = env.get_array_length(expected_sizes).map_err(|_| -2)? as usize;
    let load_factors_len = env.get_array_length(load_factors).map_err(|_| -3)? as usize;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < noisechunk_wrap_capacity::SUMMARY_FIELDS {
        return Err(-(noisechunk_wrap_capacity::SUMMARY_FIELDS as jint));
    }

    let entries_elements = env
        .get_int_array_elements(entries, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let expected_elements = env
        .get_int_array_elements(expected_sizes, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let load_factors_elements = env
        .get_float_array_elements(load_factors, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -8)?;

    let entries = unsafe {
        std::slice::from_raw_parts(entries_elements.as_ptr() as *const i32, entries_len)
    };
    let expected_sizes = unsafe {
        std::slice::from_raw_parts(expected_elements.as_ptr() as *const i32, expected_len)
    };
    let load_factors = unsafe {
        std::slice::from_raw_parts(
            load_factors_elements.as_ptr() as *const f32,
            load_factors_len,
        )
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            noisechunk_wrap_capacity::SUMMARY_FIELDS,
        )
    };

    let summary =
        noisechunk_wrap_capacity::shape_summary(entries, expected_sizes, load_factors, iterations)
            .map_err(|code| match code {
                noisechunk_wrap_capacity::NoiseChunkWrapCapacityError::InvalidInputLength => -9,
                noisechunk_wrap_capacity::NoiseChunkWrapCapacityError::InvalidExpected => -10,
                noisechunk_wrap_capacity::NoiseChunkWrapCapacityError::InvalidEntryCount => -11,
                noisechunk_wrap_capacity::NoiseChunkWrapCapacityError::InvalidLoadFactor => -12,
                noisechunk_wrap_capacity::NoiseChunkWrapCapacityError::TooLarge => -13,
            })?;

    dst[0] = summary.samples as i64;
    dst[1] = summary.variants as i64;
    dst[2] = summary.total_entries as i64;
    dst[3] = summary.total_initial_n as i64;
    dst[4] = summary.total_initial_max_fill as i64;
    dst[5] = summary.total_final_n as i64;
    dst[6] = summary.total_growths as i64;
    dst[7] = summary.checksum as i64;

    Ok(noisechunk_wrap_capacity::SUMMARY_FIELDS)
}

fn density_ap2_fill_summary(
    env: &mut JNIEnv,
    length: jint,
    iterations: jint,
    dst: jlongArray,
    variant: u8,
) -> Result<usize, jint> {
    let length = usize::try_from(length).map_err(|_| -1)?;
    let iterations = usize::try_from(iterations).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < density_ap2_fill::SUMMARY_FIELDS {
        return Err(-(density_ap2_fill::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            density_ap2_fill::SUMMARY_FIELDS,
        )
    };

    let summary = match variant {
        0 => density_ap2_fill::old_flat_summary(length, iterations),
        1 => density_ap2_fill::scratch_flat_summary(length, iterations),
        2 => density_ap2_fill::old_nested_summary(length, iterations),
        3 => density_ap2_fill::scratch_nested_summary(length, iterations),
        _ => return Err(-5),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sink_bits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(density_ap2_fill::SUMMARY_FIELDS)
}

fn density_ap2_minmax_fill_summary(
    env: &mut JNIEnv,
    scenario_index: jint,
    length: jint,
    iterations: jint,
    dst: jlongArray,
    optimized: bool,
) -> Result<usize, jint> {
    let scenario_index = usize::try_from(scenario_index).map_err(|_| -1)?;
    let length = usize::try_from(length).map_err(|_| -2)?;
    let iterations = usize::try_from(iterations).map_err(|_| -3)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < density_ap2_minmax_fill::SUMMARY_FIELDS {
        return Err(-(density_ap2_minmax_fill::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            density_ap2_minmax_fill::SUMMARY_FIELDS,
        )
    };

    let summary = if optimized {
        density_ap2_minmax_fill::new_summary(scenario_index, length, iterations)
    } else {
        density_ap2_minmax_fill::old_summary(scenario_index, length, iterations)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sink_bits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(density_ap2_minmax_fill::SUMMARY_FIELDS)
}

fn paletted_reencode_scratch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: PalettedReencodeScratchKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < paletted_reencode_scratch::SUMMARY_FIELDS {
        return Err(-(paletted_reencode_scratch::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            paletted_reencode_scratch::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PalettedReencodeScratchKind::OldNewArray => {
            paletted_reencode_scratch::old_newarray_summary(iterations)
        }
        PalettedReencodeScratchKind::ScratchThreadLocal => {
            paletted_reencode_scratch::scratch_threadlocal_summary(iterations)
        }
        PalettedReencodeScratchKind::DirectPacked => {
            paletted_reencode_scratch::direct_packed_summary(iterations)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.guard;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last;

    Ok(paletted_reencode_scratch::SUMMARY_FIELDS)
}

fn paletted_reencode_remap_cache_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: PalettedReencodeRemapCacheKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < paletted_reencode_remap_cache::SUMMARY_FIELDS {
        return Err(-(paletted_reencode_remap_cache::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            paletted_reencode_remap_cache::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PalettedReencodeRemapCacheKind::CurrentPreviousOnly => {
            paletted_reencode_remap_cache::current_previous_only_summary(iterations)
        }
        PalettedReencodeRemapCacheKind::CachedPaletteIds => {
            paletted_reencode_remap_cache::cached_palette_ids_summary(iterations)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.guard;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last;

    Ok(paletted_reencode_remap_cache::SUMMARY_FIELDS)
}

fn density_spline_context_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: DensitySplineContextKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < density_spline_context::SUMMARY_FIELDS {
        return Err(-(density_spline_context::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            density_spline_context::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        DensitySplineContextKind::OldWrapper => {
            density_spline_context::old_wrapper_summary(iterations)
        }
        DensitySplineContextKind::NewDirect => {
            density_spline_context::new_direct_summary(iterations)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(density_spline_context::SUMMARY_FIELDS)
}

fn density_visitor_hook_summary(
    env: &mut JNIEnv,
    roots: jint,
    depth: jint,
    iterations: jint,
    dst: jlongArray,
    kind: DensityVisitorHookKind,
) -> Result<usize, jint> {
    let roots = usize::try_from(roots).map_err(|_| -1)?;
    let depth = usize::try_from(depth).map_err(|_| -2)?;
    let iterations = usize::try_from(iterations).map_err(|_| -3)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < density_visitor_hook::SUMMARY_FIELDS {
        return Err(-(density_visitor_hook::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            density_visitor_hook::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        DensityVisitorHookKind::OldUnwrapping => {
            density_visitor_hook::old_unwrapping_summary(roots, depth, iterations)
        }
        DensityVisitorHookKind::HookedUnwrapping => {
            density_visitor_hook::hooked_unwrapping_summary(roots, depth, iterations)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.holder_allocations as i64;
    dst[2] = summary.marker_allocations as i64;
    dst[3] = summary.guard;

    Ok(density_visitor_hook::SUMMARY_FIELDS)
}

fn entity_chunk_transient_summary(
    env: &mut JNIEnv,
    iterations: jint,
    non_transient_mask: jint,
    thread_id: jlong,
    dst: jlongArray,
    kind: EntityChunkTransientKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let non_transient_mask = i32::try_from(non_transient_mask).map_err(|_| -2)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < entity_chunk_transient::SUMMARY_FIELDS {
        return Err(-(entity_chunk_transient::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            entity_chunk_transient::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        EntityChunkTransientKind::OldMixed => {
            entity_chunk_transient::old_mixed_summary(iterations, non_transient_mask, thread_id as i64)
        }
        EntityChunkTransientKind::NewMixed => {
            entity_chunk_transient::new_mixed_summary(iterations, non_transient_mask, thread_id as i64)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.value;
    dst[2] = summary.non_transient_count as i64;
    dst[3] = summary.last_ticket;

    Ok(entity_chunk_transient::SUMMARY_FIELDS)
}

fn chunk_dependencies_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    array: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < chunk_dependencies::SUMMARY_FIELDS {
        return Err(-(chunk_dependencies::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            chunk_dependencies::SUMMARY_FIELDS,
        )
    };

    let summary = if array {
        chunk_dependencies::array_summary(iterations)
    } else {
        chunk_dependencies::old_immutable_list_summary(iterations)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.value;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_value;

    Ok(chunk_dependencies::SUMMARY_FIELDS)
}

fn ownable_rule_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    new_loop: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < ownable_rule::SUMMARY_FIELDS {
        return Err(-(ownable_rule::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            ownable_rule::SUMMARY_FIELDS,
        )
    };

    let summary = if new_loop {
        ownable_rule::new_loop_summary(iterations)
    } else {
        ownable_rule::old_stream_summary(iterations)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.matches as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_match as i64;

    Ok(ownable_rule::SUMMARY_FIELDS)
}

fn plugin_name_join_summary(
    env: &mut JNIEnv,
    iterations: jint,
    names: jobjectArray,
    delimiter: JString,
    dst: jlongArray,
    kind: PluginNameJoinKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let names = java_string_array_to_vec(env, names).map_err(|code| code)?;
    let delimiter: String = env.get_string(delimiter).map_err(|_| -3)?.into();
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < plugin_name_join::SUMMARY_FIELDS {
        return Err(-(plugin_name_join::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_name_join::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PluginNameJoinKind::StringJoin => {
            plugin_name_join::string_join_summary(iterations, &names, &delimiter)
        }
        PluginNameJoinKind::ManualJoin => {
            plugin_name_join::manual_join_summary(iterations, &names, &delimiter)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total_length as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_hash as i64;

    Ok(plugin_name_join::SUMMARY_FIELDS)
}

fn plugin_name_log_summary(
    env: &mut JNIEnv,
    iterations: jint,
    paper_names: jobjectArray,
    bukkit_names: jobjectArray,
    dst: jlongArray,
    kind: PluginNameLogKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let paper_names = java_string_array_to_vec(env, paper_names).map_err(|code| code)?;
    let bukkit_names = java_string_array_to_vec(env, bukkit_names).map_err(|code| code)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < plugin_name_log::SUMMARY_FIELDS {
        return Err(-(plugin_name_log::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_name_log::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PluginNameLogKind::OldTreeset => {
            plugin_name_log::old_treeset_summary(iterations, &paper_names, &bukkit_names)
        }
        PluginNameLogKind::NewArrayListSort => {
            plugin_name_log::new_arraylist_sort_summary(iterations, &paper_names, &bukkit_names)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(plugin_name_log::SUMMARY_FIELDS)
}

fn plugin_startup_rollup_summary(
    env: &mut JNIEnv,
    iterations: jint,
    names: jobjectArray,
    delimiter: JString,
    paper_names: jobjectArray,
    bukkit_names: jobjectArray,
    dst: jlongArray,
    kind: PluginStartupRollupKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let names = java_string_array_to_vec(env, names).map_err(|code| code)?;
    let delimiter: String = env.get_string(delimiter).map_err(|_| -3)?.into();
    let paper_names = java_string_array_to_vec(env, paper_names).map_err(|code| code)?;
    let bukkit_names = java_string_array_to_vec(env, bukkit_names).map_err(|code| code)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < plugin_startup_rollup::SUMMARY_FIELDS {
        return Err(-(plugin_startup_rollup::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_startup_rollup::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PluginStartupRollupKind::OldTreesetStringJoin => plugin_startup_rollup::old_summary(
            iterations,
            &names,
            &delimiter,
            &paper_names,
            &bukkit_names,
        ),
        PluginStartupRollupKind::NewArrayListSortManualJoin => plugin_startup_rollup::new_summary(
            iterations,
            &names,
            &delimiter,
            &paper_names,
            &bukkit_names,
        ),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.join_total_length as i64;
    dst[2] = summary.log_total as i64;
    dst[3] = summary.checksum as i64;
    dst[4] = summary.join_last_hash as i64;
    dst[5] = summary.log_last_total as i64;
    dst[6] = summary.join_checksum as i64;
    dst[7] = summary.log_checksum as i64;

    Ok(plugin_startup_rollup::SUMMARY_FIELDS)
}

#[allow(clippy::too_many_arguments)]
fn obfhelper_maps_summary(
    env: &mut JNIEnv,
    class_mapped_names: jobjectArray,
    class_original_names: jobjectArray,
    method_counts: jintArray,
    field_counts: jintArray,
    method_mapped_names: jobjectArray,
    method_mapped_descriptors: jobjectArray,
    method_original_names: jobjectArray,
    method_original_descriptors: jobjectArray,
    field_mapped_names: jobjectArray,
    field_original_names: jobjectArray,
    dst: jlongArray,
    kind: ObfHelperMapsKind,
) -> Result<usize, jint> {
    let class_mapped_names = java_string_array_to_vec(env, class_mapped_names).map_err(|code| code)?;
    let class_original_names = java_string_array_to_vec(env, class_original_names).map_err(|code| code)?;
    let method_counts = java_usize_array_to_vec(env, method_counts).map_err(|code| code)?;
    let field_counts = java_usize_array_to_vec(env, field_counts).map_err(|code| code)?;
    let method_mapped_names = java_string_array_to_vec(env, method_mapped_names).map_err(|code| code)?;
    let method_mapped_descriptors =
        java_string_array_to_vec(env, method_mapped_descriptors).map_err(|code| code)?;
    let method_original_names =
        java_string_array_to_vec(env, method_original_names).map_err(|code| code)?;
    let method_original_descriptors =
        java_string_array_to_vec(env, method_original_descriptors).map_err(|code| code)?;
    let field_mapped_names = java_string_array_to_vec(env, field_mapped_names).map_err(|code| code)?;
    let field_original_names =
        java_string_array_to_vec(env, field_original_names).map_err(|code| code)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < obfhelper_maps::SUMMARY_FIELDS {
        return Err(-(obfhelper_maps::SUMMARY_FIELDS as jint));
    }

    let fixture = obfhelper_maps::ObfHelperMapsFixture {
        class_mapped_names: &class_mapped_names,
        class_original_names: &class_original_names,
        method_counts: &method_counts,
        field_counts: &field_counts,
        method_mapped_names: &method_mapped_names,
        method_mapped_descriptors: &method_mapped_descriptors,
        method_original_names: &method_original_names,
        method_original_descriptors: &method_original_descriptors,
        field_mapped_names: &field_mapped_names,
        field_original_names: &field_original_names,
    };

    let summary = match kind {
        ObfHelperMapsKind::OldStreamDefault => {
            obfhelper_maps::old_stream_default_summary(&fixture)
        }
        ObfHelperMapsKind::DirectMaps => obfhelper_maps::direct_maps_summary(&fixture),
        ObfHelperMapsKind::PresizedStringPool => {
            obfhelper_maps::presized_string_pool_summary(&fixture)
        }
    }
    .map_err(|code| match code {
        obfhelper_maps::ObfHelperMapsError::InvalidInputLength => -20,
        obfhelper_maps::ObfHelperMapsError::InvalidCount => -21,
        obfhelper_maps::ObfHelperMapsError::DuplicateKey => -22,
    })?;

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            obfhelper_maps::SUMMARY_FIELDS,
        )
    };

    dst[0] = summary.class_count as i64;
    dst[1] = summary.entry_count as i64;
    dst[2] = summary.fingerprint as i64;

    Ok(obfhelper_maps::SUMMARY_FIELDS)
}

fn plugin_meta_dependency_summary(
    env: &mut JNIEnv,
    iterations: jint,
    names: jobjectArray,
    required: jbooleanArray,
    join_classpath: jbooleanArray,
    load: jintArray,
    dst: jlongArray,
    kind: PluginMetaDependencyKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let names = java_string_array_to_vec(env, names).map_err(|code| code)?;

    let required_len = env.get_array_length(required).map_err(|_| -3)? as usize;
    let join_classpath_len = env.get_array_length(join_classpath).map_err(|_| -4)? as usize;
    let load_len = env.get_array_length(load).map_err(|_| -5)? as usize;
    if names.len() != required_len || names.len() != join_classpath_len || names.len() != load_len {
        return Err(-6);
    }

    let required_elements = env
        .get_boolean_array_elements(required, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let join_classpath_elements = env
        .get_boolean_array_elements(join_classpath, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let load_elements = env
        .get_int_array_elements(load, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;

    let required_raw = unsafe { std::slice::from_raw_parts(required_elements.as_ptr(), required_len) };
    let join_classpath_raw = unsafe {
        std::slice::from_raw_parts(join_classpath_elements.as_ptr(), join_classpath_len)
    };
    let load = unsafe { std::slice::from_raw_parts(load_elements.as_ptr(), load_len) };

    let required = required_raw.iter().map(|value| *value != 0).collect::<Vec<_>>();
    let join_classpath = join_classpath_raw.iter().map(|value| *value != 0).collect::<Vec<_>>();

    let dst_len = env.get_array_length(dst).map_err(|_| -7)? as usize;
    if dst_len < plugin_meta_dependency::SUMMARY_FIELDS {
        return Err(-7);
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -11)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_meta_dependency::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PluginMetaDependencyKind::OldStream => plugin_meta_dependency::old_stream_summary(
            iterations,
            &names,
            &required,
            &join_classpath,
            load,
        ),
        PluginMetaDependencyKind::NewLoop => plugin_meta_dependency::new_loop_summary(
            iterations,
            &names,
            &required,
            &join_classpath,
            load,
        ),
        PluginMetaDependencyKind::Cached => plugin_meta_dependency::cached_summary(
            iterations,
            &names,
            &required,
            &join_classpath,
            load,
        ),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(plugin_meta_dependency::SUMMARY_FIELDS)
}

fn plugin_classloader_group_summary(
    env: &mut JNIEnv,
    iterations: jint,
    loader_names: jobjectArray,
    result_lengths: jintArray,
    requester_index: jint,
    query: JString,
    dst: jlongArray,
    kind: PluginClassLoaderGroupKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let loader_names = java_string_array_to_vec(env, loader_names).map_err(|code| code)?;
    let result_lengths_len = env.get_array_length(result_lengths).map_err(|_| -4)? as usize;
    if loader_names.len() != result_lengths_len {
        return Err(-5);
    }

    let requester_index = usize::try_from(requester_index).map_err(|_| -6)?;
    if requester_index >= loader_names.len() {
        return Err(-7);
    }

    let query: String = env.get_string(query).map_err(|_| -8)?.into();

    let result_lengths_elements = env
        .get_int_array_elements(result_lengths, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let result_lengths = unsafe { std::slice::from_raw_parts(result_lengths_elements.as_ptr(), result_lengths_len) };

    let dst_len = env.get_array_length(dst).map_err(|_| -10)? as usize;
    if dst_len < plugin_classloader_group::SUMMARY_FIELDS {
        return Err(-(plugin_classloader_group::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -11)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_classloader_group::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PluginClassLoaderGroupKind::OldLookup => plugin_classloader_group::old_lookup_summary(
            iterations,
            &loader_names,
            result_lengths,
            requester_index,
            &query,
        ),
        PluginClassLoaderGroupKind::SkipRequester => {
            plugin_classloader_group::skip_requester_lookup_summary(
                iterations,
                &loader_names,
                result_lengths,
                requester_index,
                &query,
            )
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.result_sum;
    dst[2] = summary.attempts as i64;
    dst[3] = summary.checksum as i64;
    dst[4] = summary.last_result;

    Ok(plugin_classloader_group::SUMMARY_FIELDS)
}

fn plugin_loading_allocation_summary(
    env: &mut JNIEnv,
    iterations: jint,
    provider_names: jobjectArray,
    provided_aliases: jobjectArray,
    dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
    kind: PluginLoadingAllocationKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dependencies_per_provider = usize::try_from(dependencies_per_provider).map_err(|_| -2)?;
    let provider_names = java_string_array_to_vec(env, provider_names).map_err(|code| code)?;
    let provided_aliases =
        java_string_array_to_option_vec(env, provided_aliases).map_err(|code| code)?;
    let dependencies = java_string_array_to_vec(env, dependencies).map_err(|code| code)?;
    if provider_names.len() != provided_aliases.len() {
        return Err(-3);
    }
    if dependencies.len() != provider_names.len() * dependencies_per_provider {
        return Err(-4);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < plugin_loading_allocation::SUMMARY_FIELDS {
        return Err(-(plugin_loading_allocation::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_loading_allocation::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        PluginLoadingAllocationKind::OldDefaultCapacitySetup => {
            plugin_loading_allocation::old_default_capacity_setup_summary(
                iterations,
                &provider_names,
                &provided_aliases,
                &dependencies,
                dependencies_per_provider,
            )
        }
        PluginLoadingAllocationKind::NewPresizedSetup => {
            plugin_loading_allocation::new_presized_setup_summary(
                iterations,
                &provider_names,
                &provided_aliases,
                &dependencies,
                dependencies_per_provider,
            )
        }
        PluginLoadingAllocationKind::OldEagerMissingSet => {
            plugin_loading_allocation::old_eager_missing_set_summary(
                iterations,
                &provider_names,
                &provided_aliases,
                &dependencies,
                dependencies_per_provider,
            )
        }
        PluginLoadingAllocationKind::NewLazyMissingSet => {
            plugin_loading_allocation::new_lazy_missing_set_summary(
                iterations,
                &provider_names,
                &provided_aliases,
                &dependencies,
                dependencies_per_provider,
            )
        }
        PluginLoadingAllocationKind::OldEagerValidate => {
            plugin_loading_allocation::old_eager_validate_summary(
                iterations,
                &provider_names,
                &provided_aliases,
                &dependencies,
                dependencies_per_provider,
            )
        }
        PluginLoadingAllocationKind::NewLazyValidate => {
            plugin_loading_allocation::new_lazy_validate_summary(
                iterations,
                &provider_names,
                &provided_aliases,
                &dependencies,
                dependencies_per_provider,
            )
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(plugin_loading_allocation::SUMMARY_FIELDS)
}

fn legacy_provided_alias_removal_summary(
    env: &mut JNIEnv,
    iterations: jint,
    provider_names: jobjectArray,
    aliases: jobjectArray,
    aliases_per_provider: jint,
    dst: jlongArray,
    kind: LegacyProvidedAliasRemovalKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let aliases_per_provider = usize::try_from(aliases_per_provider).map_err(|_| -2)?;
    let provider_names = java_string_array_to_vec(env, provider_names).map_err(|code| code)?;
    let aliases = java_string_array_to_vec(env, aliases).map_err(|code| code)?;
    if aliases.len() != provider_names.len() * aliases_per_provider {
        return Err(-3);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < legacy_provided_alias_removal::SUMMARY_FIELDS {
        return Err(-(legacy_provided_alias_removal::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            legacy_provided_alias_removal::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        LegacyProvidedAliasRemovalKind::OldValuesRemoveIf => {
            legacy_provided_alias_removal::old_values_removeif_summary(
                iterations,
                &provider_names,
                &aliases,
                aliases_per_provider,
            )
        }
        LegacyProvidedAliasRemovalKind::NewReverseAliasRemove => {
            legacy_provided_alias_removal::new_reverse_alias_remove_summary(
                iterations,
                &provider_names,
                &aliases,
                aliases_per_provider,
            )
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(legacy_provided_alias_removal::SUMMARY_FIELDS)
}

fn spigot_load_order_dependency_build_summary(
    env: &mut JNIEnv,
    iterations: jint,
    load_after: jobjectArray,
    dst: jlongArray,
    kind: SpigotLoadOrderDependencyKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let load_after = java_string_array_to_vec(env, load_after).map_err(|code| code)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < spigot_load_order_dependency::SUMMARY_FIELDS {
        return Err(-(spigot_load_order_dependency::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            spigot_load_order_dependency::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        SpigotLoadOrderDependencyKind::OldLoadAfterBuild => {
            spigot_load_order_dependency::old_load_after_build_summary(iterations, &load_after)
        }
        SpigotLoadOrderDependencyKind::NewLoadAfterBuild => {
            spigot_load_order_dependency::new_load_after_build_summary(iterations, &load_after)
        }
        SpigotLoadOrderDependencyKind::OldRemovedCount
        | SpigotLoadOrderDependencyKind::NewRemovedCount => return Err(-4),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(spigot_load_order_dependency::SUMMARY_FIELDS)
}

fn spigot_load_order_dependency_removed_count_summary(
    env: &mut JNIEnv,
    iterations: jint,
    provider_names: jobjectArray,
    hard_dependencies: jobjectArray,
    soft_dependencies: jobjectArray,
    dependencies_per_provider: jint,
    dst: jlongArray,
    kind: SpigotLoadOrderDependencyKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dependencies_per_provider = usize::try_from(dependencies_per_provider).map_err(|_| -2)?;
    let provider_names = java_string_array_to_vec(env, provider_names).map_err(|code| code)?;
    let hard_dependencies = java_string_array_to_vec(env, hard_dependencies).map_err(|code| code)?;
    let soft_dependencies = java_string_array_to_vec(env, soft_dependencies).map_err(|code| code)?;
    if hard_dependencies.len() != provider_names.len() * dependencies_per_provider {
        return Err(-3);
    }
    if soft_dependencies.len() != hard_dependencies.len() {
        return Err(-4);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < spigot_load_order_dependency::SUMMARY_FIELDS {
        return Err(-(spigot_load_order_dependency::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            spigot_load_order_dependency::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        SpigotLoadOrderDependencyKind::OldRemovedCount => {
            spigot_load_order_dependency::old_removed_count_summary(
                iterations,
                &provider_names,
                &hard_dependencies,
                &soft_dependencies,
                dependencies_per_provider,
            )
        }
        SpigotLoadOrderDependencyKind::NewRemovedCount => {
            spigot_load_order_dependency::new_removed_count_summary(
                iterations,
                &provider_names,
                &hard_dependencies,
                &soft_dependencies,
                dependencies_per_provider,
            )
        }
        SpigotLoadOrderDependencyKind::OldLoadAfterBuild
        | SpigotLoadOrderDependencyKind::NewLoadAfterBuild => return Err(-7),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(spigot_load_order_dependency::SUMMARY_FIELDS)
}

fn topographic_graph_sort_capacity_summary(
    env: &mut JNIEnv,
    iterations: jint,
    successor_offsets: jintArray,
    successors: jintArray,
    in_degree: jintArray,
    dst: jlongArray,
    kind: TopographicGraphSortCapacityKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let successor_offsets = java_int_array_to_vec(env, successor_offsets).map_err(|code| code)?;
    let successors = java_int_array_to_vec(env, successors).map_err(|code| code)?;
    let in_degree = java_int_array_to_vec(env, in_degree).map_err(|code| code)?;

    if successor_offsets.len() != in_degree.len() + 1 {
        return Err(-2);
    }
    if successors.len() != *successor_offsets.last().unwrap() as usize {
        return Err(-3);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < topographic_graph_sort_capacity::SUMMARY_FIELDS {
        return Err(-(topographic_graph_sort_capacity::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            topographic_graph_sort_capacity::SUMMARY_FIELDS,
        )
    };

    let successor_offsets = successor_offsets
        .into_iter()
        .map(|value| value as usize)
        .collect::<Vec<_>>();
    let successors = successors
        .into_iter()
        .map(|value| value as usize)
        .collect::<Vec<_>>();
    let in_degree = in_degree
        .into_iter()
        .map(|value| value as usize)
        .collect::<Vec<_>>();

    let summary = match kind {
        TopographicGraphSortCapacityKind::OldDefaultCapacity => {
            topographic_graph_sort_capacity::old_default_capacity_summary(
                iterations,
                &successor_offsets,
                &successors,
                &in_degree,
            )
        }
        TopographicGraphSortCapacityKind::NewPresized => {
            topographic_graph_sort_capacity::new_presized_summary(
                iterations,
                &successor_offsets,
                &successors,
                &in_degree,
            )
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(topographic_graph_sort_capacity::SUMMARY_FIELDS)
}

fn remapper_index_cleanup_summary(
    env: &mut JNIEnv,
    iterations: jint,
    input_paths: jobjectArray,
    input_hashes: jobjectArray,
    remapped_hashes: jobjectArray,
    remapped_paths: jobjectArray,
    skipped_hashes: jobjectArray,
    dst: jlongArray,
    kind: RemapperIndexCleanupKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let input_paths = java_string_array_to_vec(env, input_paths).map_err(|code| code)?;
    let input_hashes = java_string_array_to_vec(env, input_hashes).map_err(|code| code)?;
    let remapped_hashes = java_string_array_to_vec(env, remapped_hashes).map_err(|code| code)?;
    let remapped_paths = java_string_array_to_vec(env, remapped_paths).map_err(|code| code)?;
    let skipped_hashes = java_string_array_to_vec(env, skipped_hashes).map_err(|code| code)?;

    if input_paths.len() != input_hashes.len() {
        return Err(-2);
    }
    if remapped_hashes.len() != remapped_paths.len() {
        return Err(-3);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < remapper_index_cleanup::SUMMARY_FIELDS {
        return Err(-(remapper_index_cleanup::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            remapper_index_cleanup::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        RemapperIndexCleanupKind::OldEagerCleanup => {
            remapper_index_cleanup::old_eager_cleanup_summary(
                iterations,
                &input_paths,
                &input_hashes,
                &remapped_hashes,
                &remapped_paths,
                &skipped_hashes,
            )
        }
        RemapperIndexCleanupKind::NewLazyCleanup => {
            remapper_index_cleanup::new_lazy_cleanup_summary(
                iterations,
                &input_paths,
                &input_hashes,
                &remapped_hashes,
                &remapped_paths,
                &skipped_hashes,
            )
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(remapper_index_cleanup::SUMMARY_FIELDS)
}

fn remapper_skip_hashes_summary(
    env: &mut JNIEnv,
    iterations: jint,
    content: JString,
    dst: jlongArray,
    kind: RemapperSkipHashesKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let content: String = env.get_string(content).map_err(|_| -2)?.into();

    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < remapper_skip_hashes::SUMMARY_FIELDS {
        return Err(-(remapper_skip_hashes::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            remapper_skip_hashes::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        RemapperSkipHashesKind::OldStream => {
            remapper_skip_hashes::old_stream_summary(iterations, &content)
        }
        RemapperSkipHashesKind::NewLoop => {
            remapper_skip_hashes::new_loop_summary(iterations, &content)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(remapper_skip_hashes::SUMMARY_FIELDS)
}

fn plugin_directory_scan_summary(
    env: &mut JNIEnv,
    iterations: jint,
    directory: JString,
    dst: jlongArray,
    kind: PluginDirectoryScanKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let directory: String = env.get_string(directory).map_err(|_| -2)?.into();

    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < plugin_directory_scan::SUMMARY_FIELDS {
        return Err(-(plugin_directory_scan::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            plugin_directory_scan::SUMMARY_FIELDS,
        )
    };

    let path = std::path::Path::new(&directory);
    let summary = match kind {
        PluginDirectoryScanKind::OldWalkDepth1 => {
            plugin_directory_scan::old_walk_depth1_summary(iterations, path)
        }
        PluginDirectoryScanKind::NewList => {
            plugin_directory_scan::new_list_summary(iterations, path)
        }
        PluginDirectoryScanKind::DirectoryStream => {
            plugin_directory_scan::directory_stream_summary(iterations, path)
        }
    }
    .map_err(|_| -5)?;

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(plugin_directory_scan::SUMMARY_FIELDS)
}

fn chunk_expire_count_summary(
    env: &mut JNIEnv,
    iterations: jint,
    section_bits: jint,
    chunk_bits: jint,
    kind: jint,
    dst: jlongArray,
    hot: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let section_bits = usize::try_from(section_bits).map_err(|_| -2)?;
    let chunk_bits = usize::try_from(chunk_bits).map_err(|_| -3)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < chunk_expire_count::SUMMARY_FIELDS {
        return Err(-(chunk_expire_count::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            chunk_expire_count::SUMMARY_FIELDS,
        )
    };

    let kind = match kind {
        0 => ChunkExpireCountKind::DynamicCompute,
        1 => ChunkExpireCountKind::DynamicManual,
        2 => ChunkExpireCountKind::CachedCompute,
        3 => ChunkExpireCountKind::CachedHybrid,
        4 => ChunkExpireCountKind::CachedManual,
        _ => return Err(-6),
    };

    let summary = match (hot, kind) {
        (true, ChunkExpireCountKind::DynamicCompute) => {
            chunk_expire_count::dynamic_compute_hot_summary(iterations, section_bits, chunk_bits)
        }
        (true, ChunkExpireCountKind::DynamicManual) => {
            chunk_expire_count::dynamic_manual_hot_summary(iterations, section_bits, chunk_bits)
        }
        (true, ChunkExpireCountKind::CachedCompute) => {
            chunk_expire_count::cached_compute_hot_summary(iterations, section_bits, chunk_bits)
        }
        (true, ChunkExpireCountKind::CachedHybrid) => {
            chunk_expire_count::cached_hybrid_hot_summary(iterations, section_bits, chunk_bits)
        }
        (true, ChunkExpireCountKind::CachedManual) => {
            chunk_expire_count::cached_manual_hot_summary(iterations, section_bits, chunk_bits)
        }
        (false, ChunkExpireCountKind::DynamicCompute) => {
            chunk_expire_count::dynamic_compute_cold_summary(section_bits, chunk_bits)
        }
        (false, ChunkExpireCountKind::DynamicManual) => {
            chunk_expire_count::dynamic_manual_cold_summary(section_bits, chunk_bits)
        }
        (false, ChunkExpireCountKind::CachedCompute) => {
            chunk_expire_count::cached_compute_cold_summary(section_bits, chunk_bits)
        }
        (false, ChunkExpireCountKind::CachedHybrid) => {
            chunk_expire_count::cached_hybrid_cold_summary(section_bits, chunk_bits)
        }
        (false, ChunkExpireCountKind::CachedManual) => {
            chunk_expire_count::cached_manual_cold_summary(section_bits, chunk_bits)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(chunk_expire_count::SUMMARY_FIELDS)
}

fn craftplayer_cansee_summary(
    env: &mut JNIEnv,
    iterations: jint,
    kind: CraftPlayerCanSeeKind,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < craftplayer_cansee::SUMMARY_FIELDS {
        return Err(-(craftplayer_cansee::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            craftplayer_cansee::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        CraftPlayerCanSeeKind::CurrentEmpty => {
            craftplayer_cansee::current_empty_summary(iterations)
        }
        CraftPlayerCanSeeKind::GuardedEmpty => {
            craftplayer_cansee::guarded_empty_summary(iterations)
        }
        CraftPlayerCanSeeKind::CandidateEmpty => {
            craftplayer_cansee::candidate_empty_summary(iterations)
        }
        CraftPlayerCanSeeKind::ChunkMapCandidateEmpty => {
            craftplayer_cansee::chunkmap_candidate_empty_summary(iterations)
        }
        CraftPlayerCanSeeKind::CurrentPopulated => {
            craftplayer_cansee::current_populated_summary(iterations)
        }
        CraftPlayerCanSeeKind::GuardedPopulated => {
            craftplayer_cansee::guarded_populated_summary(iterations)
        }
        CraftPlayerCanSeeKind::CandidatePopulated => {
            craftplayer_cansee::candidate_populated_summary(iterations)
        }
        CraftPlayerCanSeeKind::ChunkMapCandidatePopulated => {
            craftplayer_cansee::chunkmap_candidate_populated_summary(iterations)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(craftplayer_cansee::SUMMARY_FIELDS)
}

fn levelchunk_heightmap_summary(
    env: &mut JNIEnv,
    batches: jint,
    dst: jlongArray,
    kind: LevelChunkHeightmapKind,
) -> Result<usize, jint> {
    let batches = usize::try_from(batches).map_err(|_| -1)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < levelchunk_heightmap::SUMMARY_FIELDS {
        return Err(-(levelchunk_heightmap::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            levelchunk_heightmap::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        LevelChunkHeightmapKind::OldFourUpdate => {
            levelchunk_heightmap::old_four_update_summary(batches)
        }
        LevelChunkHeightmapKind::NewCombinedUpdate => {
            levelchunk_heightmap::new_combined_update_summary(batches)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(levelchunk_heightmap::SUMMARY_FIELDS)
}

fn marker_cache_summary(
    env: &mut JNIEnv,
    iterations: jint,
    roots: jint,
    depth: jint,
    dst: jlongArray,
    kind: MarkerCacheKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let roots = usize::try_from(roots).map_err(|_| -2)?;
    let depth = usize::try_from(depth).map_err(|_| -3)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < marker_cache::SUMMARY_FIELDS {
        return Err(-(marker_cache::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            marker_cache::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        MarkerCacheKind::Old => marker_cache::old_marker_cache_summary(iterations, roots, depth),
        MarkerCacheKind::Cached => marker_cache::cached_marker_cache_summary(iterations, roots, depth),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(marker_cache::SUMMARY_FIELDS)
}

fn nearby_player_map_capacity_summary(
    env: &mut JNIEnv,
    iterations: jint,
    player_count: jint,
    dst: jlongArray,
    kind: NearbyPlayerMapCapacityKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let player_count = usize::try_from(player_count).map_err(|_| -2)?;

    let dst_len = env.get_array_length(dst).map_err(|_| -3)? as usize;
    if dst_len < nearby_player_map_capacity::SUMMARY_FIELDS {
        return Err(-(nearby_player_map_capacity::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            nearby_player_map_capacity::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        NearbyPlayerMapCapacityKind::Default => {
            nearby_player_map_capacity::default_capacity_summary(iterations, player_count)
        }
        NearbyPlayerMapCapacityKind::Presized => {
            nearby_player_map_capacity::presized_capacity_summary(iterations, player_count)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(nearby_player_map_capacity::SUMMARY_FIELDS)
}

fn waypoint_distance_guard_summary(
    env: &mut JNIEnv,
    iterations: jint,
    source_x: jdoubleArray,
    source_y: jdoubleArray,
    source_z: jdoubleArray,
    receiver_x: jdoubleArray,
    receiver_y: jdoubleArray,
    receiver_z: jdoubleArray,
    range: jdoubleArray,
    dst: jlongArray,
    kind: WaypointDistanceGuardKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let source_x = java_double_array_to_vec(env, source_x).map_err(|code| code)?;
    let source_y = java_double_array_to_vec(env, source_y).map_err(|code| code)?;
    let source_z = java_double_array_to_vec(env, source_z).map_err(|code| code)?;
    let receiver_x = java_double_array_to_vec(env, receiver_x).map_err(|code| code)?;
    let receiver_y = java_double_array_to_vec(env, receiver_y).map_err(|code| code)?;
    let receiver_z = java_double_array_to_vec(env, receiver_z).map_err(|code| code)?;
    let range = if matches!(
        kind,
        WaypointDistanceGuardKind::OldAtOrBeyondRange | WaypointDistanceGuardKind::GuardedAtOrBeyondRange
    ) {
        Some(java_double_array_to_vec(env, range).map_err(|code| code)?)
    } else {
        None
    };

    let len = source_x.len();
    if source_y.len() != len
        || source_z.len() != len
        || receiver_x.len() != len
        || receiver_y.len() != len
        || receiver_z.len() != len
    {
        return Err(-2);
    }
    if let Some(ref range) = range {
        if range.len() != len {
            return Err(-3);
        }
    }
    if !len.is_power_of_two() {
        return Err(-4);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < waypoint_distance_guard::SUMMARY_FIELDS {
        return Err(-(waypoint_distance_guard::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            waypoint_distance_guard::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        WaypointDistanceGuardKind::OldAtOrBeyondRange => {
            waypoint_distance_guard::old_at_or_beyond_range_summary(
                iterations,
                &source_x,
                &source_y,
                &source_z,
                &receiver_x,
                &receiver_y,
                &receiver_z,
                range.as_ref().expect("range"),
            )
        }
        WaypointDistanceGuardKind::GuardedAtOrBeyondRange => {
            waypoint_distance_guard::guarded_at_or_beyond_range_summary(
                iterations,
                &source_x,
                &source_y,
                &source_z,
                &receiver_x,
                &receiver_y,
                &receiver_z,
                range.as_ref().expect("range"),
            )
        }
        WaypointDistanceGuardKind::OldReallyFar => waypoint_distance_guard::old_really_far_summary(
            iterations,
            &source_x,
            &source_y,
            &source_z,
            &receiver_x,
            &receiver_y,
            &receiver_z,
        ),
        WaypointDistanceGuardKind::GuardedReallyFar => {
            waypoint_distance_guard::guarded_really_far_summary(
                iterations,
                &source_x,
                &source_y,
                &source_z,
                &receiver_x,
                &receiver_y,
                &receiver_z,
            )
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.total as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_total as i64;

    Ok(waypoint_distance_guard::SUMMARY_FIELDS)
}

fn java_string_array_to_vec(env: &mut JNIEnv, names: jobjectArray) -> Result<Vec<String>, jint> {
    let len = env.get_array_length(names).map_err(|_| -2)? as usize;
    let mut values = Vec::with_capacity(len);
    for index in 0..len {
        let obj = env
            .get_object_array_element(names, index as i32)
            .map_err(|_| -2)?;
        let value: String = env
            .get_string(JString::from(obj))
            .map_err(|_| -2)?
            .into();
        values.push(value);
    }
    Ok(values)
}

fn java_string_array_to_option_vec(
    env: &mut JNIEnv,
    names: jobjectArray,
) -> Result<Vec<Option<String>>, jint> {
    let len = env.get_array_length(names).map_err(|_| -2)? as usize;
    let mut values = Vec::with_capacity(len);
    for index in 0..len {
        let obj = env
            .get_object_array_element(names, index as i32)
            .map_err(|_| -2)?;
        if obj.into_inner().is_null() {
            values.push(None);
        } else {
            let value: String = env
                .get_string(JString::from(obj))
                .map_err(|_| -2)?
                .into();
            values.push(Some(value));
        }
    }
    Ok(values)
}

fn java_int_array_to_vec(env: &mut JNIEnv, values: jintArray) -> Result<Vec<i32>, jint> {
    let len = env.get_array_length(values).map_err(|_| -2)? as usize;
    let elements = env
        .get_int_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -2)?;
    let raw = unsafe { std::slice::from_raw_parts(elements.as_ptr(), len) };
    Ok(raw.to_vec())
}

fn java_usize_array_to_vec(env: &mut JNIEnv, values: jintArray) -> Result<Vec<usize>, jint> {
    let raw = java_int_array_to_vec(env, values)?;
    let mut converted = Vec::with_capacity(raw.len());
    for value in raw {
        converted.push(usize::try_from(value).map_err(|_| -2)?);
    }
    Ok(converted)
}

fn java_double_array_to_vec(env: &mut JNIEnv, values: jdoubleArray) -> Result<Vec<f64>, jint> {
    let len = env.get_array_length(values).map_err(|_| -2)? as usize;
    let elements = env
        .get_double_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -2)?;
    let raw = unsafe { std::slice::from_raw_parts(elements.as_ptr(), len) };
    Ok(raw.to_vec())
}

fn shift_noise_direct_summary(
    env: &mut JNIEnv,
    iterations: jint,
    block_x: jintArray,
    block_y: jintArray,
    block_z: jintArray,
    dst: jlongArray,
    kind: ShiftNoiseDirectKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;

    let block_x_len = env.get_array_length(block_x).map_err(|_| -2)? as usize;
    let block_y_len = env.get_array_length(block_y).map_err(|_| -3)? as usize;
    let block_z_len = env.get_array_length(block_z).map_err(|_| -4)? as usize;
    if block_x_len != block_y_len || block_x_len != block_z_len || iterations > block_x_len {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < shift_noise_direct::SUMMARY_FIELDS {
        return Err(-(shift_noise_direct::SUMMARY_FIELDS as jint));
    }

    let block_x_elements = env
        .get_int_array_elements(block_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let block_y_elements = env
        .get_int_array_elements(block_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let block_z_elements = env
        .get_int_array_elements(block_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;

    let block_x = unsafe { std::slice::from_raw_parts(block_x_elements.as_ptr(), block_x_len) };
    let block_y = unsafe { std::slice::from_raw_parts(block_y_elements.as_ptr(), block_y_len) };
    let block_z = unsafe { std::slice::from_raw_parts(block_z_elements.as_ptr(), block_z_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            shift_noise_direct::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        ShiftNoiseDirectKind::CurrentDefault => {
            shift_noise_direct::current_default_summary(iterations, block_x, block_y, block_z)
        }
        ShiftNoiseDirectKind::DirectDefault => {
            shift_noise_direct::direct_default_summary(iterations, block_x, block_y, block_z)
        }
        ShiftNoiseDirectKind::CurrentA => {
            shift_noise_direct::current_a_summary(iterations, block_x, block_y, block_z)
        }
        ShiftNoiseDirectKind::DirectA => {
            shift_noise_direct::direct_a_summary(iterations, block_x, block_y, block_z)
        }
        ShiftNoiseDirectKind::CurrentB => {
            shift_noise_direct::current_b_summary(iterations, block_x, block_y, block_z)
        }
        ShiftNoiseDirectKind::DirectB => {
            shift_noise_direct::direct_b_summary(iterations, block_x, block_y, block_z)
        }
    }
    .map_err(|code| match code {
        shift_noise_direct::ShiftNoiseDirectError::InvalidInputLength => -11,
        shift_noise_direct::ShiftNoiseDirectError::InvalidVariant => -12,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.xor_bits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(shift_noise_direct::SUMMARY_FIELDS)
}

fn entity_bounding_box_summary(
    env: &mut JNIEnv,
    iterations: jint,
    widths: jfloatArray,
    heights: jfloatArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    dst: jlongArray,
    kind: EntityBoundingBoxKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;

    let widths_len = env.get_array_length(widths).map_err(|_| -2)? as usize;
    let heights_len = env.get_array_length(heights).map_err(|_| -3)? as usize;
    let xs_len = env.get_array_length(xs).map_err(|_| -4)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -5)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -6)? as usize;
    if widths_len != heights_len || widths_len != xs_len || widths_len != ys_len || widths_len != zs_len {
        return Err(-7);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -8)? as usize;
    if dst_len < entity_bounding_box::SUMMARY_FIELDS {
        return Err(-(entity_bounding_box::SUMMARY_FIELDS as jint));
    }

    let widths_elements = env
        .get_float_array_elements(widths, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let heights_elements = env
        .get_float_array_elements(heights, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -14)?;

    let widths = unsafe { std::slice::from_raw_parts(widths_elements.as_ptr(), widths_len) };
    let heights = unsafe { std::slice::from_raw_parts(heights_elements.as_ptr(), heights_len) };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            entity_bounding_box::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        EntityBoundingBoxKind::OldMakeThenSet => {
            entity_bounding_box::old_make_then_set_summary(iterations, widths, heights, xs, ys, zs)
        }
        EntityBoundingBoxKind::DirectDimensionsSet => {
            entity_bounding_box::direct_dimensions_set_summary(iterations, widths, heights, xs, ys, zs)
        }
    }
    .map_err(|code| match code {
        entity_bounding_box::EntityBoundingBoxError::InvalidInputLength => -15,
        entity_bounding_box::EntityBoundingBoxError::InvalidShape => -16,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.value_bits as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(entity_bounding_box::SUMMARY_FIELDS)
}

fn entity_lookup_status_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    kind: EntityLookupStatusKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < entity_lookup_status::SUMMARY_FIELDS {
        return Err(-(entity_lookup_status::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            entity_lookup_status::SUMMARY_FIELDS,
        )
    };

    let summary = match kind {
        EntityLookupStatusKind::OldStatus => entity_lookup_status::old_status_summary(iterations),
        EntityLookupStatusKind::DirectStatus => entity_lookup_status::direct_status_summary(iterations),
        EntityLookupStatusKind::OldAccessible => {
            entity_lookup_status::old_accessible_summary(iterations)
        }
        EntityLookupStatusKind::DirectAccessible => {
            entity_lookup_status::direct_accessible_summary(iterations)
        }
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.value;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_value;

    Ok(entity_lookup_status::SUMMARY_FIELDS)
}

fn static_cache_get_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    min_x: jint,
    min_z: jint,
    size_x: jint,
    size_z: jint,
    values: jintArray,
    dst: jlongArray,
    new: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let size_x = usize::try_from(size_x).map_err(|_| -2)?;
    let size_z = usize::try_from(size_z).map_err(|_| -3)?;
    let values_len = env.get_array_length(values).map_err(|_| -4)? as usize;
    if values_len != size_x.saturating_mul(size_z) {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < static_cache_get::SUMMARY_FIELDS {
        return Err(-(static_cache_get::SUMMARY_FIELDS as jint));
    }

    let values_elements = env
        .get_int_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let values = unsafe { std::slice::from_raw_parts(values_elements.as_ptr(), values_len) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -8)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            static_cache_get::SUMMARY_FIELDS,
        )
    };

    let summary = if new {
        static_cache_get::new_batch_summary(iterations, min_x, min_z, size_x, size_z, values)
    } else {
        static_cache_get::old_batch_summary(iterations, min_x, min_z, size_x, size_z, values)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_value;

    Ok(static_cache_get::SUMMARY_FIELDS)
}

fn serverentity_delta_identity_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    same_identity: Option<jbyteArray>,
    current_x: jdoubleArray,
    current_y: jdoubleArray,
    current_z: jdoubleArray,
    last_x: jdoubleArray,
    last_y: jdoubleArray,
    last_z: jdoubleArray,
    dst: jlongArray,
    identity_guard: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let entries = env.get_array_length(current_x).map_err(|_| -2)? as usize;
    if entries == 0 || !entries.is_power_of_two() {
        return Err(-3);
    }

    let current_y_len = env.get_array_length(current_y).map_err(|_| -4)? as usize;
    let current_z_len = env.get_array_length(current_z).map_err(|_| -5)? as usize;
    let last_x_len = env.get_array_length(last_x).map_err(|_| -6)? as usize;
    let last_y_len = env.get_array_length(last_y).map_err(|_| -7)? as usize;
    let last_z_len = env.get_array_length(last_z).map_err(|_| -8)? as usize;
    if current_y_len != entries || current_z_len != entries || last_x_len != entries || last_y_len != entries || last_z_len != entries {
        return Err(-9);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -10)? as usize;
    if dst_len < serverentity_delta_identity::SUMMARY_FIELDS {
        return Err(-(serverentity_delta_identity::SUMMARY_FIELDS as jint));
    }

    let same_identity_elements = if identity_guard {
        let same_identity = same_identity.ok_or(-11)?;
        let same_identity_len = env.get_array_length(same_identity).map_err(|_| -12)? as usize;
        if same_identity_len != entries {
            return Err(-13);
        }
        Some(
            env.get_byte_array_elements(same_identity, ReleaseMode::NoCopyBack)
                .map_err(|_| -14)?,
        )
    } else {
        None
    };

    let current_x_elements = env
        .get_double_array_elements(current_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let current_y_elements = env
        .get_double_array_elements(current_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let current_z_elements = env
        .get_double_array_elements(current_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let last_x_elements = env
        .get_double_array_elements(last_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;
    let last_y_elements = env
        .get_double_array_elements(last_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -19)?;
    let last_z_elements = env
        .get_double_array_elements(last_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -20)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -21)?;

    let current_x = unsafe { std::slice::from_raw_parts(current_x_elements.as_ptr(), entries) };
    let current_y = unsafe { std::slice::from_raw_parts(current_y_elements.as_ptr(), entries) };
    let current_z = unsafe { std::slice::from_raw_parts(current_z_elements.as_ptr(), entries) };
    let last_x = unsafe { std::slice::from_raw_parts(last_x_elements.as_ptr(), entries) };
    let last_y = unsafe { std::slice::from_raw_parts(last_y_elements.as_ptr(), entries) };
    let last_z = unsafe { std::slice::from_raw_parts(last_z_elements.as_ptr(), entries) };
    let same_identity = same_identity_elements.as_ref().map(|elements| unsafe {
        std::slice::from_raw_parts(elements.as_ptr(), entries)
    });
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            serverentity_delta_identity::SUMMARY_FIELDS,
        )
    };

    let summary = if identity_guard {
        serverentity_delta_identity::identity_guard_summary(
            iterations,
            same_identity.ok_or(-22)?,
            current_x,
            current_y,
            current_z,
            last_x,
            last_y,
            last_z,
        )
    } else {
        serverentity_delta_identity::old_distance_summary(
            iterations,
            current_x,
            current_y,
            current_z,
            last_x,
            last_y,
            last_z,
        )
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sends as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_distance_bits as i64;

    Ok(serverentity_delta_identity::SUMMARY_FIELDS)
}

fn aquifer_index_stride_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    optimized: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < aquifer_index_stride::SUMMARY_FIELDS {
        return Err(-(aquifer_index_stride::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            aquifer_index_stride::SUMMARY_FIELDS,
        )
    };

    let summary = if optimized {
        aquifer_index_stride::new_loop_summary(iterations)
    } else {
        aquifer_index_stride::old_loop_summary(iterations)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.result;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_value;

    Ok(aquifer_index_stride::SUMMARY_FIELDS)
}

fn aquifer_positional_location_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
    direct: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let xs_len = env.get_array_length(xs).map_err(|_| -2)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -3)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -4)? as usize;
    if xs_len != iterations || ys_len != iterations || zs_len != iterations {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < aquifer_positional_location::SUMMARY_FIELDS {
        return Err(-(aquifer_positional_location::SUMMARY_FIELDS as jint));
    }

    let xs_elements = env
        .get_int_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let ys_elements = env
        .get_int_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let zs_elements = env
        .get_int_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), iterations) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), iterations) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), iterations) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            aquifer_positional_location::SUMMARY_FIELDS,
        )
    };

    let summary = if direct {
        aquifer_positional_location::direct_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt)
    } else {
        aquifer_positional_location::old_batch_summary(xs, ys, zs, seed_lo_salt, seed_hi_salt)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(aquifer_positional_location::SUMMARY_FIELDS)
}

fn xoroshiro_positional_direct_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    seed_lo_salt: jlong,
    seed_hi_salt: jlong,
    dst: jlongArray,
    direct: bool,
    float_mode: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let xs_len = env.get_array_length(xs).map_err(|_| -2)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -3)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -4)? as usize;
    if xs_len != iterations || ys_len != iterations || zs_len != iterations {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < xoroshiro_positional_direct::SUMMARY_FIELDS {
        return Err(-(xoroshiro_positional_direct::SUMMARY_FIELDS as jint));
    }

    let xs_elements = env
        .get_int_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let ys_elements = env
        .get_int_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let zs_elements = env
        .get_int_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), iterations) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), iterations) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), iterations) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            xoroshiro_positional_direct::SUMMARY_FIELDS,
        )
    };

    let summary = match (direct, float_mode) {
        (false, true) => xoroshiro_positional_direct::old_float_batch_summary(
            xs,
            ys,
            zs,
            seed_lo_salt,
            seed_hi_salt,
        ),
        (true, true) => xoroshiro_positional_direct::direct_float_batch_summary(
            xs,
            ys,
            zs,
            seed_lo_salt,
            seed_hi_salt,
        ),
        (false, false) => xoroshiro_positional_direct::old_double_batch_summary(
            xs,
            ys,
            zs,
            seed_lo_salt,
            seed_hi_salt,
        ),
        (true, false) => xoroshiro_positional_direct::direct_double_batch_summary(
            xs,
            ys,
            zs,
            seed_lo_salt,
            seed_hi_salt,
        ),
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(xoroshiro_positional_direct::SUMMARY_FIELDS)
}

fn yclamped_gradient_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    block_ys: jintArray,
    from_ys: jintArray,
    to_ys: jintArray,
    from_values: jdoubleArray,
    to_values: jdoubleArray,
    dst: jlongArray,
    optimized: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let block_ys_len = env.get_array_length(block_ys).map_err(|_| -2)? as usize;
    let from_ys_len = env.get_array_length(from_ys).map_err(|_| -3)? as usize;
    let to_ys_len = env.get_array_length(to_ys).map_err(|_| -4)? as usize;
    let from_values_len = env.get_array_length(from_values).map_err(|_| -5)? as usize;
    let to_values_len = env.get_array_length(to_values).map_err(|_| -6)? as usize;
    if block_ys_len != iterations
        || from_ys_len != iterations
        || to_ys_len != iterations
        || from_values_len != iterations
        || to_values_len != iterations
    {
        return Err(-7);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -8)? as usize;
    if dst_len < yclamped_gradient::SUMMARY_FIELDS {
        return Err(-(yclamped_gradient::SUMMARY_FIELDS as jint));
    }

    let block_ys_elements = env
        .get_int_array_elements(block_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let from_ys_elements = env
        .get_int_array_elements(from_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let to_ys_elements = env
        .get_int_array_elements(to_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let from_values_elements = env
        .get_double_array_elements(from_values, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let to_values_elements = env
        .get_double_array_elements(to_values, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;

    let block_ys = unsafe { std::slice::from_raw_parts(block_ys_elements.as_ptr(), iterations) };
    let from_ys = unsafe { std::slice::from_raw_parts(from_ys_elements.as_ptr(), iterations) };
    let to_ys = unsafe { std::slice::from_raw_parts(to_ys_elements.as_ptr(), iterations) };
    let from_values =
        unsafe { std::slice::from_raw_parts(from_values_elements.as_ptr(), iterations) };
    let to_values = unsafe { std::slice::from_raw_parts(to_values_elements.as_ptr(), iterations) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -14)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            yclamped_gradient::SUMMARY_FIELDS,
        )
    };

    let summary = if optimized {
        yclamped_gradient::optimized_batch_summary(
            block_ys,
            from_ys,
            to_ys,
            from_values,
            to_values,
        )
    } else {
        yclamped_gradient::current_batch_summary(block_ys, from_ys, to_ys, from_values, to_values)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(yclamped_gradient::SUMMARY_FIELDS)
}

fn beardifier_bury_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    dst: jlongArray,
    optimized: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let xs_len = env.get_array_length(xs).map_err(|_| -2)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -3)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -4)? as usize;
    if xs_len != iterations || ys_len != iterations || zs_len != iterations {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < beardifier_bury::SUMMARY_FIELDS {
        return Err(-(beardifier_bury::SUMMARY_FIELDS as jint));
    }

    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), iterations) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), iterations) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), iterations) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            beardifier_bury::SUMMARY_FIELDS,
        )
    };

    let summary = if optimized {
        beardifier_bury::optimized_batch_summary(xs, ys, zs)
    } else {
        beardifier_bury::current_batch_summary(xs, ys, zs)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(beardifier_bury::SUMMARY_FIELDS)
}

fn biome_getbiome_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    seeds: jlongArray,
    block_xs: jintArray,
    block_ys: jintArray,
    block_zs: jintArray,
    dst: jlongArray,
    optimized: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let seeds_len = env.get_array_length(seeds).map_err(|_| -2)? as usize;
    let block_xs_len = env.get_array_length(block_xs).map_err(|_| -3)? as usize;
    let block_ys_len = env.get_array_length(block_ys).map_err(|_| -4)? as usize;
    let block_zs_len = env.get_array_length(block_zs).map_err(|_| -5)? as usize;
    if seeds_len != iterations
        || block_xs_len != iterations
        || block_ys_len != iterations
        || block_zs_len != iterations
    {
        return Err(-6);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -7)? as usize;
    if dst_len < biome_getbiome::SUMMARY_FIELDS {
        return Err(-(biome_getbiome::SUMMARY_FIELDS as jint));
    }

    let seeds_elements = env
        .get_long_array_elements(seeds, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let block_xs_elements = env
        .get_int_array_elements(block_xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let block_ys_elements = env
        .get_int_array_elements(block_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let block_zs_elements = env
        .get_int_array_elements(block_zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let seeds = unsafe { std::slice::from_raw_parts(seeds_elements.as_ptr(), iterations) };
    let block_xs = unsafe { std::slice::from_raw_parts(block_xs_elements.as_ptr(), iterations) };
    let block_ys = unsafe { std::slice::from_raw_parts(block_ys_elements.as_ptr(), iterations) };
    let block_zs = unsafe { std::slice::from_raw_parts(block_zs_elements.as_ptr(), iterations) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            biome_getbiome::SUMMARY_FIELDS,
        )
    };

    let summary = if optimized {
        biome_getbiome::optimized_batch_summary(seeds, block_xs, block_ys, block_zs)
    } else {
        biome_getbiome::current_batch_summary(seeds, block_xs, block_ys, block_zs)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_values as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_value as i64;

    Ok(biome_getbiome::SUMMARY_FIELDS)
}

fn spring_feature_mutable_pos_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    requires_below: jbooleanArray,
    rock_count: jintArray,
    hole_count: jintArray,
    dst: jlongArray,
    mutable: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let positions = env.get_array_length(xs).map_err(|_| -2)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -3)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -4)? as usize;
    let requires_below_len = env.get_array_length(requires_below).map_err(|_| -5)? as usize;
    let rock_count_len = env.get_array_length(rock_count).map_err(|_| -6)? as usize;
    let hole_count_len = env.get_array_length(hole_count).map_err(|_| -7)? as usize;
    if positions != ys_len
        || positions != zs_len
        || positions != requires_below_len
        || positions != rock_count_len
        || positions != hole_count_len
    {
        return Err(-8);
    }
    if iterations > 0 && (positions == 0 || !positions.is_power_of_two()) {
        return Err(-9);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -10)? as usize;
    if dst_len < spring_feature_mutable_pos::SUMMARY_FIELDS {
        return Err(-(spring_feature_mutable_pos::SUMMARY_FIELDS as jint));
    }

    let xs_elements = env
        .get_int_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let ys_elements = env
        .get_int_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let zs_elements = env
        .get_int_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let requires_below_elements = env
        .get_boolean_array_elements(requires_below, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let rock_count_elements = env
        .get_int_array_elements(rock_count, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let hole_count_elements = env
        .get_int_array_elements(hole_count, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), positions) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), positions) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), positions) };
    let requires_below_raw =
        unsafe { std::slice::from_raw_parts(requires_below_elements.as_ptr(), positions) };
    let rock_count = unsafe { std::slice::from_raw_parts(rock_count_elements.as_ptr(), positions) };
    let hole_count = unsafe { std::slice::from_raw_parts(hole_count_elements.as_ptr(), positions) };
    let requires_below = requires_below_raw
        .iter()
        .map(|value| *value != 0)
        .collect::<Vec<_>>();

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -17)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            spring_feature_mutable_pos::SUMMARY_FIELDS,
        )
    };

    let summary = if mutable {
        spring_feature_mutable_pos::mutable_batch_summary(
            xs,
            ys,
            zs,
            &requires_below,
            rock_count,
            hole_count,
            iterations,
        )
    } else {
        spring_feature_mutable_pos::old_batch_summary(
            xs,
            ys,
            zs,
            &requires_below,
            rock_count,
            hole_count,
            iterations,
        )
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.success_count as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_decision as i64;

    Ok(spring_feature_mutable_pos::SUMMARY_FIELDS)
}

fn jigsaw_canattach_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    orientation_fronts: jintArray,
    orientation_tops: jintArray,
    parent_orientations: jintArray,
    child_orientations: jintArray,
    parent_rollables: jbooleanArray,
    parent_targets: jintArray,
    child_names: jintArray,
    dst: jlongArray,
    optimized: bool,
    target_first: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let orientation_fronts_len = env.get_array_length(orientation_fronts).map_err(|_| -2)? as usize;
    let orientation_tops_len = env.get_array_length(orientation_tops).map_err(|_| -3)? as usize;
    if orientation_fronts_len != orientation_tops_len {
        return Err(-4);
    }

    let positions = env.get_array_length(parent_orientations).map_err(|_| -5)? as usize;
    let child_orientations_len = env.get_array_length(child_orientations).map_err(|_| -6)? as usize;
    let parent_rollables_len = env.get_array_length(parent_rollables).map_err(|_| -7)? as usize;
    let parent_targets_len = env.get_array_length(parent_targets).map_err(|_| -8)? as usize;
    let child_names_len = env.get_array_length(child_names).map_err(|_| -9)? as usize;
    if positions != child_orientations_len
        || positions != parent_rollables_len
        || positions != parent_targets_len
        || positions != child_names_len
    {
        return Err(-10);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -11)? as usize;
    if dst_len < jigsaw_canattach::SUMMARY_FIELDS {
        return Err(-(jigsaw_canattach::SUMMARY_FIELDS as jint));
    }

    let orientation_fronts_elements = env
        .get_int_array_elements(orientation_fronts, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let orientation_tops_elements = env
        .get_int_array_elements(orientation_tops, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let parent_orientations_elements = env
        .get_int_array_elements(parent_orientations, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let child_orientations_elements = env
        .get_int_array_elements(child_orientations, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let parent_rollables_elements = env
        .get_boolean_array_elements(parent_rollables, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let parent_targets_elements = env
        .get_int_array_elements(parent_targets, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let child_names_elements = env
        .get_int_array_elements(child_names, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;

    let orientation_fronts = unsafe {
        std::slice::from_raw_parts(orientation_fronts_elements.as_ptr(), orientation_fronts_len)
    };
    let orientation_tops = unsafe {
        std::slice::from_raw_parts(orientation_tops_elements.as_ptr(), orientation_tops_len)
    };
    let parent_orientations = unsafe {
        std::slice::from_raw_parts(parent_orientations_elements.as_ptr(), positions)
    };
    let child_orientations = unsafe {
        std::slice::from_raw_parts(child_orientations_elements.as_ptr(), positions)
    };
    let parent_rollables_raw = unsafe {
        std::slice::from_raw_parts(parent_rollables_elements.as_ptr(), positions)
    };
    let parent_rollables = parent_rollables_raw
        .iter()
        .map(|value| *value != 0)
        .collect::<Vec<_>>();
    let parent_targets = unsafe {
        std::slice::from_raw_parts(parent_targets_elements.as_ptr(), positions)
    };
    let child_names = unsafe { std::slice::from_raw_parts(child_names_elements.as_ptr(), positions) };

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -19)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, jigsaw_canattach::SUMMARY_FIELDS)
    };

    let summary = if target_first {
        jigsaw_canattach::target_first_batch_summary(
            iterations,
            orientation_fronts,
            orientation_tops,
            parent_orientations,
            child_orientations,
            &parent_rollables,
            parent_targets,
            child_names,
        )
    } else if optimized {
        jigsaw_canattach::optimized_batch_summary(
            iterations,
            orientation_fronts,
            orientation_tops,
            parent_orientations,
            child_orientations,
            &parent_rollables,
            parent_targets,
            child_names,
        )
    } else {
        jigsaw_canattach::old_batch_summary(
            iterations,
            orientation_fronts,
            orientation_tops,
            parent_orientations,
            child_orientations,
            &parent_rollables,
            parent_targets,
            child_names,
        )
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.success_count as i64;
    dst[2] = summary.checksum as i64;
    dst[3] = summary.last_decision as i64;

    Ok(jigsaw_canattach::SUMMARY_FIELDS)
}

fn aquifer_surface_sampling_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    optimized: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < aquifer_surface_sampling::SUMMARY_FIELDS {
        return Err(-(aquifer_surface_sampling::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            aquifer_surface_sampling::SUMMARY_FIELDS,
        )
    };

    let summary = if optimized {
        aquifer_surface_sampling::new_loop_summary(iterations)
    } else {
        aquifer_surface_sampling::old_loop_summary(iterations)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(aquifer_surface_sampling::SUMMARY_FIELDS)
}

fn blended_noise_batch_summary(
    env: &mut JNIEnv,
    iterations: jint,
    dst: jlongArray,
    cached: bool,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < blended_noise::SUMMARY_FIELDS {
        return Err(-(blended_noise::SUMMARY_FIELDS as jint));
    }

    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, blended_noise::SUMMARY_FIELDS)
    };

    let summary = if cached {
        blended_noise::cached_loop_summary(iterations)
    } else {
        blended_noise::old_loop_summary(iterations)
    };

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(blended_noise::SUMMARY_FIELDS)
}

fn chunk_pack_batch(
    env: &mut JNIEnv,
    xs: jintArray,
    zs: jintArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let count = matching_pair_len(env, xs, zs)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < count {
        return Err(-(count as jint));
    }

    let x_elements = env
        .get_int_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let z_elements = env
        .get_int_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -7)?;

    let xs = unsafe { std::slice::from_raw_parts(x_elements.as_ptr(), count) };
    let zs = unsafe { std::slice::from_raw_parts(z_elements.as_ptr(), count) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, count) };

    for index in 0..count {
        dst[index] = position::chunk_as_long(xs[index], zs[index]);
    }
    Ok(count)
}

fn position_combined_batch(
    env: &mut JNIEnv,
    chunk_xs: jintArray,
    chunk_zs: jintArray,
    section_xs: jintArray,
    section_ys: jintArray,
    section_zs: jintArray,
    chunk_dst: jlongArray,
    hash_dst: jintArray,
    section_dst: jlongArray,
) -> Result<usize, jint> {
    let chunk_count = matching_pair_len(env, chunk_xs, chunk_zs)?;
    let section_count = matching_triple_len(env, section_xs, section_ys, section_zs)?;
    if chunk_count != section_count {
        return Err(-10);
    }

    let chunk_dst_len = env.get_array_length(chunk_dst).map_err(|_| -11)? as usize;
    let hash_dst_len = env.get_array_length(hash_dst).map_err(|_| -12)? as usize;
    let section_dst_len = env.get_array_length(section_dst).map_err(|_| -13)? as usize;
    if chunk_dst_len < chunk_count || hash_dst_len < chunk_count || section_dst_len < chunk_count {
        return Err(-(chunk_count as jint));
    }

    let chunk_x_elements = env
        .get_int_array_elements(chunk_xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let chunk_z_elements = env
        .get_int_array_elements(chunk_zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let section_x_elements = env
        .get_int_array_elements(section_xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let section_y_elements = env
        .get_int_array_elements(section_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let section_z_elements = env
        .get_int_array_elements(section_zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;
    let chunk_dst_elements = env
        .get_long_array_elements(chunk_dst, ReleaseMode::CopyBack)
        .map_err(|_| -19)?;
    let hash_dst_elements = env
        .get_int_array_elements(hash_dst, ReleaseMode::CopyBack)
        .map_err(|_| -20)?;
    let section_dst_elements = env
        .get_long_array_elements(section_dst, ReleaseMode::CopyBack)
        .map_err(|_| -21)?;

    let chunk_xs = unsafe { std::slice::from_raw_parts(chunk_x_elements.as_ptr(), chunk_count) };
    let chunk_zs = unsafe { std::slice::from_raw_parts(chunk_z_elements.as_ptr(), chunk_count) };
    let section_xs = unsafe { std::slice::from_raw_parts(section_x_elements.as_ptr(), chunk_count) };
    let section_ys = unsafe { std::slice::from_raw_parts(section_y_elements.as_ptr(), chunk_count) };
    let section_zs = unsafe { std::slice::from_raw_parts(section_z_elements.as_ptr(), chunk_count) };
    let chunk_dst =
        unsafe { std::slice::from_raw_parts_mut(chunk_dst_elements.as_ptr() as *mut i64, chunk_count) };
    let hash_dst = unsafe { std::slice::from_raw_parts_mut(hash_dst_elements.as_ptr(), chunk_count) };
    let section_dst =
        unsafe { std::slice::from_raw_parts_mut(section_dst_elements.as_ptr() as *mut i64, chunk_count) };

    for index in 0..chunk_count {
        let chunk_x = chunk_xs[index];
        let chunk_z = chunk_zs[index];
        chunk_dst[index] = position::chunk_as_long(chunk_x, chunk_z);
        hash_dst[index] = position::chunk_hash(chunk_x, chunk_z);
        section_dst[index] =
            position::section_as_long(section_xs[index], section_ys[index], section_zs[index]);
    }

    Ok(chunk_count)
}

fn area_map_update_summary_batch(
    env: &mut JNIEnv,
    from_xs: jintArray,
    from_zs: jintArray,
    old_distances: jintArray,
    to_xs: jintArray,
    to_zs: jintArray,
    new_distances: jintArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let count = env.get_array_length(from_xs).map_err(|_| -1)? as usize;
    let from_zs_len = env.get_array_length(from_zs).map_err(|_| -2)? as usize;
    let old_distances_len = env.get_array_length(old_distances).map_err(|_| -3)? as usize;
    let to_xs_len = env.get_array_length(to_xs).map_err(|_| -4)? as usize;
    let to_zs_len = env.get_array_length(to_zs).map_err(|_| -5)? as usize;
    let new_distances_len = env.get_array_length(new_distances).map_err(|_| -6)? as usize;
    if count != from_zs_len
        || count != old_distances_len
        || count != to_xs_len
        || count != to_zs_len
        || count != new_distances_len
    {
        return Err(-7);
    }

    let required = count.checked_mul(5).ok_or(-8)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -9)? as usize;
    if dst_len < required {
        return Err(-(required as jint));
    }

    let from_xs_elements = env
        .get_int_array_elements(from_xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let from_zs_elements = env
        .get_int_array_elements(from_zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let old_distances_elements = env
        .get_int_array_elements(old_distances, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let to_xs_elements = env
        .get_int_array_elements(to_xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let to_zs_elements = env
        .get_int_array_elements(to_zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let new_distances_elements = env
        .get_int_array_elements(new_distances, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -16)?;

    let from_xs = unsafe { std::slice::from_raw_parts(from_xs_elements.as_ptr(), count) };
    let from_zs = unsafe { std::slice::from_raw_parts(from_zs_elements.as_ptr(), count) };
    let old_distances =
        unsafe { std::slice::from_raw_parts(old_distances_elements.as_ptr(), count) };
    let to_xs = unsafe { std::slice::from_raw_parts(to_xs_elements.as_ptr(), count) };
    let to_zs = unsafe { std::slice::from_raw_parts(to_zs_elements.as_ptr(), count) };
    let new_distances =
        unsafe { std::slice::from_raw_parts(new_distances_elements.as_ptr(), count) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, required) };

    for index in 0..count {
        let summary = area_map::summarize_update(
            from_xs[index],
            from_zs[index],
            old_distances[index],
            to_xs[index],
            to_zs[index],
            new_distances[index],
        )
        .map_err(|code| match code {
            area_map::AreaMapError::NegativeDistance => -17,
        })?;

        let out = index * 5;
        dst[out] = summary.add_count as i64;
        dst[out + 1] = summary.remove_count as i64;
        dst[out + 2] = summary.add_checksum as i64;
        dst[out + 3] = summary.remove_checksum as i64;
        dst[out + 4] = summary.order_checksum as i64;
    }

    Ok(count)
}

fn area_map_square_summary_batch(
    env: &mut JNIEnv,
    op: jint,
    chunk_xs: jintArray,
    chunk_zs: jintArray,
    distances: jintArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let count = matching_triple_len(env, chunk_xs, chunk_zs, distances)?;
    let required = count.checked_mul(5).ok_or(-8)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -9)? as usize;
    if dst_len < required {
        return Err(-(required as jint));
    }

    let chunk_xs_elements = env
        .get_int_array_elements(chunk_xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let chunk_zs_elements = env
        .get_int_array_elements(chunk_zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let distances_elements = env
        .get_int_array_elements(distances, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -13)?;

    let chunk_xs = unsafe { std::slice::from_raw_parts(chunk_xs_elements.as_ptr(), count) };
    let chunk_zs = unsafe { std::slice::from_raw_parts(chunk_zs_elements.as_ptr(), count) };
    let distances = unsafe { std::slice::from_raw_parts(distances_elements.as_ptr(), count) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, required) };

    let op = area_map_op_from_jint(op)?;
    for index in 0..count {
        let summary = area_map::summarize_square(
            op,
            chunk_xs[index],
            chunk_zs[index],
            distances[index],
        )
        .map_err(|code| match code {
            area_map::AreaMapError::NegativeDistance => -14,
        })?;

        let out = index * 5;
        dst[out] = summary.add_count as i64;
        dst[out + 1] = summary.remove_count as i64;
        dst[out + 2] = summary.add_checksum as i64;
        dst[out + 3] = summary.remove_checksum as i64;
        dst[out + 4] = summary.order_checksum as i64;
    }

    Ok(count)
}

fn area_map_update_ops_batch(
    env: &mut JNIEnv,
    from_x: jint,
    from_z: jint,
    old_distance: jint,
    to_x: jint,
    to_z: jint,
    new_distance: jint,
    operations: jbyteArray,
    chunk_keys: jlongArray,
) -> Result<usize, jint> {
    let op_len = env.get_array_length(operations).map_err(|_| -2)? as usize;
    let chunk_keys_len = env.get_array_length(chunk_keys).map_err(|_| -3)? as usize;

    let max_required = area_map_max_update_ops_capacity(old_distance, new_distance)?;
    if op_len < max_required || chunk_keys_len < max_required {
        let mut required = 0usize;
        area_map::for_each_update(
            from_x,
            from_z,
            old_distance,
            to_x,
            to_z,
            new_distance,
            |_, _, _| required += 1,
        )
        .map_err(|code| match code {
            area_map::AreaMapError::NegativeDistance => -1,
        })?;
        if op_len < required || chunk_keys_len < required {
            return Err(-(required as jint));
        }
    }

    let op_elements = env
        .get_byte_array_elements(operations, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let chunk_key_elements = env
        .get_long_array_elements(chunk_keys, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;

    let operations = unsafe {
        std::slice::from_raw_parts_mut(op_elements.as_ptr() as *mut i8, op_len)
    };
    let chunk_keys = unsafe {
        std::slice::from_raw_parts_mut(chunk_key_elements.as_ptr() as *mut i64, chunk_keys_len)
    };

    let mut written = 0usize;
    area_map::for_each_update(
        from_x,
        from_z,
        old_distance,
        to_x,
        to_z,
        new_distance,
        |op, chunk_x, chunk_z| {
            operations[written] = match op {
                area_map::AreaOp::Add => 0,
                area_map::AreaOp::Remove => 1,
            };
            chunk_keys[written] = position::chunk_as_long(chunk_x, chunk_z);
            written += 1;
        },
    )
    .map_err(|code| match code {
        area_map::AreaMapError::NegativeDistance => -1,
    })?;

    Ok(written)
}

fn area_map_square_ops_batch(
    env: &mut JNIEnv,
    op: jint,
    chunk_x: jint,
    chunk_z: jint,
    distance: jint,
    operations: jbyteArray,
    chunk_keys: jlongArray,
) -> Result<usize, jint> {
    let op_len = env.get_array_length(operations).map_err(|_| -2)? as usize;
    let chunk_keys_len = env.get_array_length(chunk_keys).map_err(|_| -3)? as usize;

    let required = area_map_square_ops_capacity(distance)?;
    if op_len < required || chunk_keys_len < required {
        return Err(-(required as jint));
    }

    let op_elements = env
        .get_byte_array_elements(operations, ReleaseMode::CopyBack)
        .map_err(|_| -4)?;
    let chunk_key_elements = env
        .get_long_array_elements(chunk_keys, ReleaseMode::CopyBack)
        .map_err(|_| -5)?;

    let operations = unsafe {
        std::slice::from_raw_parts_mut(op_elements.as_ptr() as *mut i8, op_len)
    };
    let chunk_keys = unsafe {
        std::slice::from_raw_parts_mut(chunk_key_elements.as_ptr() as *mut i64, chunk_keys_len)
    };

    let mut written = 0usize;
    let op = area_map_op_from_jint(op)?;
    area_map::for_each_square(op, chunk_x, chunk_z, distance, |op, chunk_x, chunk_z| {
        operations[written] = match op {
            area_map::AreaOp::Add => 0,
            area_map::AreaOp::Remove => 1,
        };
        chunk_keys[written] = position::chunk_as_long(chunk_x, chunk_z);
        written += 1;
    })
    .map_err(|code| match code {
        area_map::AreaMapError::NegativeDistance => -1,
    })?;

    Ok(written)
}

fn reference_list_run_ops(
    env: &mut JNIEnv,
    linear_search_limit: jint,
    initial_values: jintArray,
    operations: jbyteArray,
    values: jintArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let linear_search_limit = usize::try_from(linear_search_limit).map_err(|_| -1)?;
    let initial_len = env.get_array_length(initial_values).map_err(|_| -2)? as usize;
    let operations_len = env.get_array_length(operations).map_err(|_| -3)? as usize;
    let values_len = env.get_array_length(values).map_err(|_| -4)? as usize;
    if operations_len != values_len {
        return Err(-5);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < REFERENCE_LIST_SUMMARY_FIELDS {
        return Err(-(REFERENCE_LIST_SUMMARY_FIELDS as jint));
    }

    let initial_elements = env
        .get_int_array_elements(initial_values, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let operation_elements = env
        .get_byte_array_elements(operations, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let value_elements = env
        .get_int_array_elements(values, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;

    let initial_values = unsafe { std::slice::from_raw_parts(initial_elements.as_ptr(), initial_len) };
    let operations =
        unsafe { std::slice::from_raw_parts(operation_elements.as_ptr() as *const u8, operations_len) };
    let values = unsafe { std::slice::from_raw_parts(value_elements.as_ptr(), values_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, REFERENCE_LIST_SUMMARY_FIELDS)
    };

    let summary = reference_list::run_ops(
        linear_search_limit,
        initial_values,
        operations,
        values,
    )
    .map_err(|code| match code {
        reference_list::ReferenceListError::LengthMismatch => -5,
        reference_list::ReferenceListError::InvalidOperation => -11,
        reference_list::ReferenceListError::InvalidValue => -12,
    })?;

    dst[0] = summary.add_true as i64;
    dst[1] = summary.remove_true as i64;
    dst[2] = summary.contains_true as i64;
    dst[3] = summary.false_count as i64;
    dst[4] = summary.final_size as i64;
    dst[5] = summary.event_checksum as i64;
    dst[6] = summary.order_checksum as i64;

    Ok(REFERENCE_LIST_SUMMARY_FIELDS)
}

fn reference_list_free_handle(handle: jlong) {
    if handle == 0 {
        return;
    }

    unsafe {
        drop(Box::from_raw(handle as *mut ReferenceListHandle));
    }
}

fn reference_list_apply_op(handle: jlong, op: jint, value: jint) -> Result<jint, jint> {
    if handle == 0 {
        return Err(-1);
    }
    if value == i32::MIN {
        return Err(-2);
    }

    let handle = unsafe { &mut *(handle as *mut ReferenceListHandle) };
    let result = match op {
        0 => handle.list.add(value),
        1 => handle.list.remove(value),
        2 => handle.list.contains(value),
        3 => {
            handle.list.clear();
            true
        }
        _ => return Err(-3),
    };
    let size = handle.list.size();
    if size > (i32::MAX as usize >> 1) {
        return Err(-4);
    }

    Ok(((size as jint) << 1) | if result { 1 } else { 0 })
}

fn reference_list_order_checksum_handle(handle: jlong) -> jlong {
    if handle == 0 {
        return 0;
    }

    let handle = unsafe { &*(handle as *const ReferenceListHandle) };
    handle.list.order_checksum() as jlong
}

fn ticket_pack_pack_summary(
    env: &mut JNIEnv,
    positions: jlongArray,
    ticket_types: jbyteArray,
    ticket_levels: jintArray,
    tickets_per_chunk: jint,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let tickets_per_chunk = usize::try_from(tickets_per_chunk).map_err(|_| -1)?;
    let iterations = usize::try_from(iterations).map_err(|_| -2)?;

    let positions_len = env.get_array_length(positions).map_err(|_| -3)? as usize;
    let types_len = env.get_array_length(ticket_types).map_err(|_| -4)? as usize;
    let levels_len = env.get_array_length(ticket_levels).map_err(|_| -5)? as usize;
    let expected_ticket_count = positions_len.checked_mul(tickets_per_chunk).ok_or(-6)?;
    if types_len != expected_ticket_count || levels_len != expected_ticket_count {
        return Err(-7);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -8)? as usize;
    if dst_len < ticket_pack::SUMMARY_FIELDS {
        return Err(-(ticket_pack::SUMMARY_FIELDS as jint));
    }

    let positions_elements = env
        .get_long_array_elements(positions, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let ticket_types_elements = env
        .get_byte_array_elements(ticket_types, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let ticket_levels_elements = env
        .get_int_array_elements(ticket_levels, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -12)?;

    let positions = unsafe { std::slice::from_raw_parts(positions_elements.as_ptr() as *const i64, positions_len) };
    let ticket_types = unsafe {
        std::slice::from_raw_parts(ticket_types_elements.as_ptr() as *const u8, types_len)
    };
    let ticket_levels = unsafe {
        std::slice::from_raw_parts(ticket_levels_elements.as_ptr() as *const i32, levels_len)
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, ticket_pack::SUMMARY_FIELDS)
    };

    let summary = ticket_pack::pack_summary(
        positions,
        ticket_types,
        ticket_levels,
        tickets_per_chunk,
        iterations,
    )
    .map_err(|code| match code {
        ticket_pack::TicketPackError::LengthMismatch => -13,
        ticket_pack::TicketPackError::InvalidTicketType => -14,
    })?;

    dst[0] = summary.persistent_count as i64;
    dst[1] = summary.level_sum as i64;
    dst[2] = summary.position_checksum as i64;
    dst[3] = summary.consume_value as i64;
    dst[4] = summary.sink as i64;

    Ok(ticket_pack::SUMMARY_FIELDS)
}

fn ticket_compare_compare_indexed_batch(
    env: &mut JNIEnv,
    levels: jintArray,
    type_ids: jlongArray,
    has_identifier_comparators: jbyteArray,
    identifiers: jintArray,
    left_indices: jintArray,
    right_indices: jintArray,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let levels_len = env.get_array_length(levels).map_err(|_| -2)? as usize;
    let type_ids_len = env.get_array_length(type_ids).map_err(|_| -3)? as usize;
    let comparators_len = env
        .get_array_length(has_identifier_comparators)
        .map_err(|_| -4)? as usize;
    let identifiers_len = env.get_array_length(identifiers).map_err(|_| -5)? as usize;
    if levels_len != type_ids_len
        || levels_len != comparators_len
        || levels_len != identifiers_len
    {
        return Err(-6);
    }

    let left_len = env.get_array_length(left_indices).map_err(|_| -7)? as usize;
    let right_len = env.get_array_length(right_indices).map_err(|_| -8)? as usize;
    if left_len != right_len {
        return Err(-9);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -10)? as usize;
    if dst_len < 4 {
        return Err(-11);
    }

    let levels_elements = env
        .get_int_array_elements(levels, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let type_ids_elements = env
        .get_long_array_elements(type_ids, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let comparators_elements = env
        .get_byte_array_elements(has_identifier_comparators, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let identifiers_elements = env
        .get_int_array_elements(identifiers, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let left_elements = env
        .get_int_array_elements(left_indices, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let right_elements = env
        .get_int_array_elements(right_indices, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -18)?;

    let levels = unsafe { std::slice::from_raw_parts(levels_elements.as_ptr(), levels_len) };
    let type_ids =
        unsafe { std::slice::from_raw_parts(type_ids_elements.as_ptr() as *const i64, type_ids_len) };
    let comparators = unsafe {
        std::slice::from_raw_parts(comparators_elements.as_ptr() as *const u8, comparators_len)
    };
    let identifiers =
        unsafe { std::slice::from_raw_parts(identifiers_elements.as_ptr(), identifiers_len) };
    let left_indices = unsafe { std::slice::from_raw_parts(left_elements.as_ptr(), left_len) };
    let right_indices = unsafe { std::slice::from_raw_parts(right_elements.as_ptr(), right_len) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, 4) };

    let summary = ticket_compare::compare_indexed_batch(
        levels,
        type_ids,
        comparators,
        identifiers,
        left_indices,
        right_indices,
        iterations,
    )
    .map_err(|code| match code {
        ticket_compare::TicketCompareError::LengthMismatch => -19,
        ticket_compare::TicketCompareError::InvalidIndex => -20,
    })?;

    dst[0] = summary.compare_sum;
    dst[1] = summary.negative_count as i64;
    dst[2] = summary.zero_count as i64;
    dst[3] = summary.positive_count as i64;

    Ok(4)
}

fn chunk_ticket_stage_run_batch(
    env: &mut JNIEnv,
    query_keys: jlongArray,
    staged_keys: jlongArray,
    staged_values: jbyteArray,
    mutation_keys: jlongArray,
    get_iterations: jint,
    mutation_iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let get_iterations = usize::try_from(get_iterations).map_err(|_| -1)?;
    let mutation_iterations = usize::try_from(mutation_iterations).map_err(|_| -2)?;

    let query_len = env.get_array_length(query_keys).map_err(|_| -3)? as usize;
    let staged_len = env.get_array_length(staged_keys).map_err(|_| -4)? as usize;
    let values_len = env.get_array_length(staged_values).map_err(|_| -5)? as usize;
    let mutation_len = env.get_array_length(mutation_keys).map_err(|_| -6)? as usize;
    if staged_len != values_len {
        return Err(-7);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -8)? as usize;
    if dst_len < chunk_ticket_stage::SUMMARY_FIELDS {
        return Err(-9);
    }

    let query_elements = env
        .get_long_array_elements(query_keys, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let staged_key_elements = env
        .get_long_array_elements(staged_keys, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let staged_value_elements = env
        .get_byte_array_elements(staged_values, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let mutation_key_elements = env
        .get_long_array_elements(mutation_keys, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -14)?;

    let query_keys = unsafe { std::slice::from_raw_parts(query_elements.as_ptr() as *const i64, query_len) };
    let staged_keys = unsafe { std::slice::from_raw_parts(staged_key_elements.as_ptr() as *const i64, staged_len) };
    let staged_values = unsafe { std::slice::from_raw_parts(staged_value_elements.as_ptr() as *const i8, values_len) };
    let mutation_keys =
        unsafe { std::slice::from_raw_parts(mutation_key_elements.as_ptr() as *const i64, mutation_len) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, chunk_ticket_stage::SUMMARY_FIELDS) };

    let summary = chunk_ticket_stage::run_batch(
        query_keys,
        staged_keys,
        staged_values,
        mutation_keys,
        get_iterations,
        mutation_iterations,
    )
    .map_err(|code| match code {
        chunk_ticket_stage::ChunkTicketStageError::LengthMismatch => -15,
    })?;

    dst[0] = summary.get_sum;
    dst[1] = summary.mutation_sum;
    dst[2] = summary.final_size as i64;
    dst[3] = summary.state_checksum as i64;

    Ok(chunk_ticket_stage::SUMMARY_FIELDS)
}

fn improved_noise_batch_summary(
    env: &mut JNIEnv,
    permutation: jbyteArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    xo: jni::sys::jdouble,
    yo: jni::sys::jdouble,
    zo: jni::sys::jdouble,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let permutation_len = env.get_array_length(permutation).map_err(|_| -2)? as usize;
    if permutation_len != improved_noise::PERMUTATION_LENGTH {
        return Err(-3);
    }

    let xs_len = env.get_array_length(xs).map_err(|_| -4)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -5)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -6)? as usize;
    let y_scales_len = env.get_array_length(y_scales).map_err(|_| -7)? as usize;
    let y_maxes_len = env.get_array_length(y_maxes).map_err(|_| -8)? as usize;
    if xs_len != ys_len || xs_len != zs_len || xs_len != y_scales_len || xs_len != y_maxes_len {
        return Err(-9);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -10)? as usize;
    if dst_len < improved_noise::SUMMARY_FIELDS {
        return Err(-(improved_noise::SUMMARY_FIELDS as jint));
    }

    let permutation_elements = env
        .get_byte_array_elements(permutation, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let y_scales_elements = env
        .get_double_array_elements(y_scales, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let y_maxes_elements = env
        .get_double_array_elements(y_maxes, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -17)?;

    let permutation = unsafe {
        std::slice::from_raw_parts(
            permutation_elements.as_ptr() as *const u8,
            permutation_len,
        )
    };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };
    let y_scales = unsafe { std::slice::from_raw_parts(y_scales_elements.as_ptr(), y_scales_len) };
    let y_maxes = unsafe { std::slice::from_raw_parts(y_maxes_elements.as_ptr(), y_maxes_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, improved_noise::SUMMARY_FIELDS)
    };

    let summary = improved_noise::noise_batch_summary(
        permutation,
        xs,
        ys,
        zs,
        y_scales,
        y_maxes,
        xo,
        yo,
        zo,
        iterations,
    )
    .map_err(|code| match code {
        improved_noise::ImprovedNoiseError::InvalidPermutationLength => -18,
        improved_noise::ImprovedNoiseError::LengthMismatch => -19,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(improved_noise::SUMMARY_FIELDS)
}

fn improved_noise_build_handle(
    env: &mut JNIEnv,
    permutation: jbyteArray,
    xo: jni::sys::jdouble,
    yo: jni::sys::jdouble,
    zo: jni::sys::jdouble,
) -> Result<jlong, jint> {
    let permutation_len = env.get_array_length(permutation).map_err(|_| -1)? as usize;
    if permutation_len != improved_noise::PERMUTATION_LENGTH {
        return Err(-2);
    }

    let permutation_elements = env
        .get_byte_array_elements(permutation, ReleaseMode::NoCopyBack)
        .map_err(|_| -3)?;
    let permutation = unsafe {
        std::slice::from_raw_parts(
            permutation_elements.as_ptr() as *const u8,
            permutation_len,
        )
    };

    let noise = improved_noise::ImprovedNoise::new(permutation, xo, yo, zo).map_err(|code| match code {
        improved_noise::ImprovedNoiseError::InvalidPermutationLength => -4,
        improved_noise::ImprovedNoiseError::LengthMismatch => -5,
    })?;
    let handle = Box::new(ImprovedNoiseHandle { noise });
    Ok(Box::into_raw(handle) as jlong)
}

fn improved_noise_free_handle(handle: jlong) {
    if handle != 0 {
        let handle = handle as *mut ImprovedNoiseHandle;
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

fn improved_noise_noise(
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    y_max: jni::sys::jdouble,
) -> jni::sys::jdouble {
    if handle == 0 {
        return f64::NAN;
    }

    let handle = unsafe { &*(handle as *const ImprovedNoiseHandle) };
    handle.noise.noise(x, y, z, y_scale, y_max)
}

fn improved_noise_noise_no_y_scale(
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
) -> jni::sys::jdouble {
    if handle == 0 {
        return f64::NAN;
    }

    let handle = unsafe { &*(handle as *const ImprovedNoiseHandle) };
    handle.noise.noise_no_y_scale(x, y, z)
}

fn improved_noise_fill(
    env: &mut JNIEnv,
    handle: jlong,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    output: jdoubleArray,
) -> Result<usize, jint> {
    if handle == 0 {
        return Err(-1);
    }

    let xs_len = env.get_array_length(xs).map_err(|_| -2)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -3)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -4)? as usize;
    let y_scales_len = env.get_array_length(y_scales).map_err(|_| -5)? as usize;
    let y_maxes_len = env.get_array_length(y_maxes).map_err(|_| -6)? as usize;
    let output_len = env.get_array_length(output).map_err(|_| -7)? as usize;
    if ys_len != xs_len
        || zs_len != xs_len
        || y_scales_len != xs_len
        || y_maxes_len != xs_len
        || output_len != xs_len
    {
        return Err(-8);
    }

    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let y_scales_elements = env
        .get_double_array_elements(y_scales, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let y_maxes_elements = env
        .get_double_array_elements(y_maxes, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let mut output_elements = env
        .get_double_array_elements(output, ReleaseMode::CopyBack)
        .map_err(|_| -14)?;

    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };
    let y_scales = unsafe { std::slice::from_raw_parts(y_scales_elements.as_ptr(), y_scales_len) };
    let y_maxes = unsafe { std::slice::from_raw_parts(y_maxes_elements.as_ptr(), y_maxes_len) };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    let handle = unsafe { &*(handle as *const ImprovedNoiseHandle) };

    if improved_noise::fill_positions(&handle.noise, xs, ys, zs, y_scales, y_maxes, values).is_err() {
        output_elements.discard();
        return Err(-15);
    }

    Ok(output_len)
}

fn improved_noise_fill_no_y_scale(
    env: &mut JNIEnv,
    handle: jlong,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    output: jdoubleArray,
) -> Result<usize, jint> {
    if handle == 0 {
        return Err(-1);
    }

    let xs_len = env.get_array_length(xs).map_err(|_| -2)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -3)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -4)? as usize;
    let output_len = env.get_array_length(output).map_err(|_| -5)? as usize;
    if ys_len != xs_len || zs_len != xs_len || output_len != xs_len {
        return Err(-6);
    }

    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let mut output_elements = env
        .get_double_array_elements(output, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;

    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    let handle = unsafe { &*(handle as *const ImprovedNoiseHandle) };

    if improved_noise::fill_positions_no_y_scale(&handle.noise, xs, ys, zs, values).is_err() {
        output_elements.discard();
        return Err(-11);
    }

    Ok(output_len)
}

fn improved_noise_inline_summary(
    env: &mut JNIEnv,
    permutation: jbyteArray,
    iterations: jint,
    dst: jlongArray,
    kind: improved_noise_inline::ImprovedNoiseInlineKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let permutation_len = env.get_array_length(permutation).map_err(|_| -2)? as usize;
    if permutation_len != improved_noise_inline::PERMUTATION_LENGTH {
        return Err(-3);
    }
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < improved_noise_inline::SUMMARY_FIELDS {
        return Err(-(improved_noise_inline::SUMMARY_FIELDS as jint));
    }

    let permutation_elements = env
        .get_byte_array_elements(permutation, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -6)?;

    let permutation = unsafe {
        std::slice::from_raw_parts(
            permutation_elements.as_ptr() as *const u8,
            permutation_len,
        )
    };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            improved_noise_inline::SUMMARY_FIELDS,
        )
    };

    let summary = improved_noise_inline::loop_summary(permutation, iterations, kind)
        .map_err(|_| -7)?;
    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(improved_noise_inline::SUMMARY_FIELDS)
}

#[allow(clippy::too_many_arguments)]
fn improved_noise_derivative_summary(
    env: &mut JNIEnv,
    permutation: jbyteArray,
    grid_x: jintArray,
    grid_y: jintArray,
    grid_z: jintArray,
    delta_x: jdoubleArray,
    delta_y: jdoubleArray,
    delta_z: jdoubleArray,
    iterations: jint,
    dst: jlongArray,
    kind: improved_noise_derivative::ImprovedNoiseDerivativeKind,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let permutation_len = env.get_array_length(permutation).map_err(|_| -2)? as usize;
    if permutation_len != improved_noise_derivative::PERMUTATION_LENGTH {
        return Err(-3);
    }

    let grid_x_len = env.get_array_length(grid_x).map_err(|_| -4)? as usize;
    let grid_y_len = env.get_array_length(grid_y).map_err(|_| -5)? as usize;
    let grid_z_len = env.get_array_length(grid_z).map_err(|_| -6)? as usize;
    let delta_x_len = env.get_array_length(delta_x).map_err(|_| -7)? as usize;
    let delta_y_len = env.get_array_length(delta_y).map_err(|_| -8)? as usize;
    let delta_z_len = env.get_array_length(delta_z).map_err(|_| -9)? as usize;
    if grid_x_len != grid_y_len
        || grid_x_len != grid_z_len
        || grid_x_len != delta_x_len
        || grid_x_len != delta_y_len
        || grid_x_len != delta_z_len
    {
        return Err(-10);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -11)? as usize;
    if dst_len < improved_noise_derivative::SUMMARY_FIELDS {
        return Err(-(improved_noise_derivative::SUMMARY_FIELDS as jint));
    }

    let permutation_elements = env
        .get_byte_array_elements(permutation, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let grid_x_elements = env
        .get_int_array_elements(grid_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let grid_y_elements = env
        .get_int_array_elements(grid_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let grid_z_elements = env
        .get_int_array_elements(grid_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let delta_x_elements = env
        .get_double_array_elements(delta_x, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let delta_y_elements = env
        .get_double_array_elements(delta_y, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let delta_z_elements = env
        .get_double_array_elements(delta_z, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -19)?;

    let permutation = unsafe {
        std::slice::from_raw_parts(
            permutation_elements.as_ptr() as *const u8,
            permutation_len,
        )
    };
    let grid_x = unsafe { std::slice::from_raw_parts(grid_x_elements.as_ptr(), grid_x_len) };
    let grid_y = unsafe { std::slice::from_raw_parts(grid_y_elements.as_ptr(), grid_y_len) };
    let grid_z = unsafe { std::slice::from_raw_parts(grid_z_elements.as_ptr(), grid_z_len) };
    let delta_x = unsafe { std::slice::from_raw_parts(delta_x_elements.as_ptr(), delta_x_len) };
    let delta_y = unsafe { std::slice::from_raw_parts(delta_y_elements.as_ptr(), delta_y_len) };
    let delta_z = unsafe { std::slice::from_raw_parts(delta_z_elements.as_ptr(), delta_z_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            improved_noise_derivative::SUMMARY_FIELDS,
        )
    };

    let summary = improved_noise_derivative::derivative_summary(
        permutation,
        grid_x,
        grid_y,
        grid_z,
        delta_x,
        delta_y,
        delta_z,
        iterations,
        kind,
    )
    .map_err(|code| match code {
        improved_noise_derivative::ImprovedNoiseDerivativeError::InvalidPermutationLength => -20,
        improved_noise_derivative::ImprovedNoiseDerivativeError::LengthMismatch => -21,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(improved_noise_derivative::SUMMARY_FIELDS)
}

fn hash_path_summary_jni(
    env: &mut JNIEnv,
    paths: jobjectArray,
    buffer_size: jint,
    dst: jlongArray,
    streaming: bool,
) -> Result<usize, jint> {
    let paths = java_string_array_to_vec(env, paths).map_err(|code| code)?;
    let buffer_size = usize::try_from(buffer_size).map_err(|_| -1)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -2)? as usize;
    if dst_len < hash_path_summary::SUMMARY_FIELDS {
        return Err(-(hash_path_summary::SUMMARY_FIELDS as jint));
    }
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -3)?;
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            hash_path_summary::SUMMARY_FIELDS,
        )
    };

    let summary = if streaming {
        hash_path_summary::streaming_summary(&paths, buffer_size).map_err(|_| -4)?
    } else {
        hash_path_summary::read_all_summary(&paths).map_err(|_| -4)?
    };

    dst[0] = summary.inputs as i64;
    dst[1] = summary.bytes as i64;
    dst[2] = summary.digest_checksum as i64;
    dst[3] = summary.last_digest_head as i64;

    Ok(hash_path_summary::SUMMARY_FIELDS)
}

fn nbt_compound_map_capacity_summary(
    env: &mut JNIEnv,
    data: jbyteArray,
    offsets: jintArray,
    lengths: jintArray,
    capacity: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let capacity = usize::try_from(capacity).map_err(|_| -1)?;
    let data_len = env.get_array_length(data).map_err(|_| -2)? as usize;
    let offsets_len = env.get_array_length(offsets).map_err(|_| -3)? as usize;
    let lengths_len = env.get_array_length(lengths).map_err(|_| -4)? as usize;
    if offsets_len != lengths_len {
        return Err(-5);
    }
    let dst_len = env.get_array_length(dst).map_err(|_| -6)? as usize;
    if dst_len < nbt_compound_map_capacity::SUMMARY_FIELDS {
        return Err(-(nbt_compound_map_capacity::SUMMARY_FIELDS as jint));
    }

    let data_elements = env
        .get_byte_array_elements(data, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let offsets_elements = env
        .get_int_array_elements(offsets, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let lengths_elements = env
        .get_int_array_elements(lengths, ReleaseMode::NoCopyBack)
        .map_err(|_| -9)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -10)?;

    let data = unsafe { std::slice::from_raw_parts(data_elements.as_ptr() as *const u8, data_len) };
    let offsets = unsafe { std::slice::from_raw_parts(offsets_elements.as_ptr(), offsets_len) };
    let lengths = unsafe { std::slice::from_raw_parts(lengths_elements.as_ptr(), lengths_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(
            dst_elements.as_ptr() as *mut i64,
            nbt_compound_map_capacity::SUMMARY_FIELDS,
        )
    };

    let summary = nbt_compound_map_capacity::parse_capacity_summary(
        data,
        offsets,
        lengths,
        capacity,
    )
    .map_err(|code| match code {
        nbt_compound_map_capacity::NbtCompoundMapCapacityError::LengthMismatch => -11,
        nbt_compound_map_capacity::NbtCompoundMapCapacityError::InvalidRange => -12,
        nbt_compound_map_capacity::NbtCompoundMapCapacityError::UnexpectedEof => -13,
        nbt_compound_map_capacity::NbtCompoundMapCapacityError::InvalidData => -14,
    })?;

    dst[0] = summary.chunks as i64;
    dst[1] = summary.compounds as i64;
    dst[2] = summary.entries as i64;
    dst[3] = summary.max_entries as i64;
    dst[4] = summary.bucket0 as i64;
    dst[5] = summary.bucket1_to_2 as i64;
    dst[6] = summary.bucket3_to_4 as i64;
    dst[7] = summary.bucket5_to_6 as i64;
    dst[8] = summary.bucket7_to_13 as i64;
    dst[9] = summary.bucket14_plus as i64;
    dst[10] = summary.checksum as i64;

    Ok(nbt_compound_map_capacity::SUMMARY_FIELDS)
}

fn perlin_noise_get_value_batch_summary(
    env: &mut JNIEnv,
    permutations: jbyteArray,
    active: jbyteArray,
    y_origins: jdoubleArray,
    amplitudes: jdoubleArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    y_scales: jdoubleArray,
    y_maxes: jdoubleArray,
    use_fixed_ys: jbyteArray,
    lowest_freq_input_factor: jni::sys::jdouble,
    lowest_freq_value_factor: jni::sys::jdouble,
    iterations: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;

    let active_len = env.get_array_length(active).map_err(|_| -2)? as usize;
    if active_len == 0 {
        return Err(-3);
    }
    let permutation_len = env.get_array_length(permutations).map_err(|_| -4)? as usize;
    if permutation_len != active_len * improved_noise::PERMUTATION_LENGTH {
        return Err(-5);
    }

    let y_origins_len = env.get_array_length(y_origins).map_err(|_| -6)? as usize;
    let amplitudes_len = env.get_array_length(amplitudes).map_err(|_| -7)? as usize;
    if y_origins_len != active_len || amplitudes_len != active_len {
        return Err(-8);
    }

    let xs_len = env.get_array_length(xs).map_err(|_| -9)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -10)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -11)? as usize;
    let y_scales_len = env.get_array_length(y_scales).map_err(|_| -12)? as usize;
    let y_maxes_len = env.get_array_length(y_maxes).map_err(|_| -13)? as usize;
    let use_fixed_len = env.get_array_length(use_fixed_ys).map_err(|_| -14)? as usize;
    if xs_len != ys_len
        || xs_len != zs_len
        || xs_len != y_scales_len
        || xs_len != y_maxes_len
        || xs_len != use_fixed_len
    {
        return Err(-15);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -16)? as usize;
    if dst_len < perlin_noise::SUMMARY_FIELDS {
        return Err(-(perlin_noise::SUMMARY_FIELDS as jint));
    }

    let permutations_elements = env
        .get_byte_array_elements(permutations, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let active_elements = env
        .get_byte_array_elements(active, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;
    let y_origins_elements = env
        .get_double_array_elements(y_origins, ReleaseMode::NoCopyBack)
        .map_err(|_| -19)?;
    let amplitudes_elements = env
        .get_double_array_elements(amplitudes, ReleaseMode::NoCopyBack)
        .map_err(|_| -20)?;
    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -21)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -22)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -23)?;
    let y_scales_elements = env
        .get_double_array_elements(y_scales, ReleaseMode::NoCopyBack)
        .map_err(|_| -24)?;
    let y_maxes_elements = env
        .get_double_array_elements(y_maxes, ReleaseMode::NoCopyBack)
        .map_err(|_| -25)?;
    let use_fixed_elements = env
        .get_byte_array_elements(use_fixed_ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -26)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -27)?;

    let permutations = unsafe {
        std::slice::from_raw_parts(permutations_elements.as_ptr() as *const u8, permutation_len)
    };
    let active =
        unsafe { std::slice::from_raw_parts(active_elements.as_ptr() as *const u8, active_len) };
    let y_origins = unsafe { std::slice::from_raw_parts(y_origins_elements.as_ptr(), y_origins_len) };
    let amplitudes = unsafe { std::slice::from_raw_parts(amplitudes_elements.as_ptr(), amplitudes_len) };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };
    let y_scales = unsafe { std::slice::from_raw_parts(y_scales_elements.as_ptr(), y_scales_len) };
    let y_maxes = unsafe { std::slice::from_raw_parts(y_maxes_elements.as_ptr(), y_maxes_len) };
    let use_fixed_ys =
        unsafe { std::slice::from_raw_parts(use_fixed_elements.as_ptr() as *const u8, use_fixed_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, perlin_noise::SUMMARY_FIELDS)
    };

    let summary = perlin_noise::get_value_batch_summary(
        permutations,
        active,
        y_origins,
        amplitudes,
        xs,
        ys,
        zs,
        y_scales,
        y_maxes,
        use_fixed_ys,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
        iterations,
    )
    .map_err(|code| match code {
        perlin_noise::PerlinNoiseError::InvalidOctaveCount => -28,
        perlin_noise::PerlinNoiseError::InvalidInputLength => -29,
        perlin_noise::PerlinNoiseError::InvalidPermutationLength => -30,
        perlin_noise::PerlinNoiseError::InvalidVariant => -31,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(perlin_noise::SUMMARY_FIELDS)
}

fn perlin_noise_build_handle(
    env: &mut JNIEnv,
    permutations: jbyteArray,
    active: jbyteArray,
    x_origins: jdoubleArray,
    y_origins: jdoubleArray,
    z_origins: jdoubleArray,
    amplitudes: jdoubleArray,
    lowest_freq_input_factor: jni::sys::jdouble,
    lowest_freq_value_factor: jni::sys::jdouble,
) -> Result<jlong, jint> {
    let active_len = env.get_array_length(active).map_err(|_| -1)? as usize;
    if active_len == 0 {
        return Err(-2);
    }
    let permutation_len = env.get_array_length(permutations).map_err(|_| -3)? as usize;
    if permutation_len != active_len * improved_noise::PERMUTATION_LENGTH {
        return Err(-4);
    }

    let x_origins_len = env.get_array_length(x_origins).map_err(|_| -5)? as usize;
    let y_origins_len = env.get_array_length(y_origins).map_err(|_| -6)? as usize;
    let z_origins_len = env.get_array_length(z_origins).map_err(|_| -7)? as usize;
    let amplitudes_len = env.get_array_length(amplitudes).map_err(|_| -8)? as usize;
    if x_origins_len != active_len
        || y_origins_len != active_len
        || z_origins_len != active_len
        || amplitudes_len != active_len
    {
        return Err(-9);
    }

    let permutations_elements = env
        .get_byte_array_elements(permutations, ReleaseMode::NoCopyBack)
        .map_err(|_| -10)?;
    let active_elements = env
        .get_byte_array_elements(active, ReleaseMode::NoCopyBack)
        .map_err(|_| -11)?;
    let x_origins_elements = env
        .get_double_array_elements(x_origins, ReleaseMode::NoCopyBack)
        .map_err(|_| -12)?;
    let y_origins_elements = env
        .get_double_array_elements(y_origins, ReleaseMode::NoCopyBack)
        .map_err(|_| -13)?;
    let z_origins_elements = env
        .get_double_array_elements(z_origins, ReleaseMode::NoCopyBack)
        .map_err(|_| -14)?;
    let amplitudes_elements = env
        .get_double_array_elements(amplitudes, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;

    let permutations = unsafe {
        std::slice::from_raw_parts(permutations_elements.as_ptr() as *const u8, permutation_len)
    };
    let active =
        unsafe { std::slice::from_raw_parts(active_elements.as_ptr() as *const u8, active_len) };
    let x_origins = unsafe { std::slice::from_raw_parts(x_origins_elements.as_ptr(), x_origins_len) };
    let y_origins = unsafe { std::slice::from_raw_parts(y_origins_elements.as_ptr(), y_origins_len) };
    let z_origins = unsafe { std::slice::from_raw_parts(z_origins_elements.as_ptr(), z_origins_len) };
    let amplitudes = unsafe { std::slice::from_raw_parts(amplitudes_elements.as_ptr(), amplitudes_len) };

    let noise = perlin_noise::PerlinNoise::new_from_flat_with_origins(
        permutations,
        active,
        x_origins,
        y_origins,
        z_origins,
        amplitudes,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
    )
    .map_err(|code| match code {
        perlin_noise::PerlinNoiseError::InvalidOctaveCount => -16,
        perlin_noise::PerlinNoiseError::InvalidInputLength => -17,
        perlin_noise::PerlinNoiseError::InvalidPermutationLength => -18,
        perlin_noise::PerlinNoiseError::InvalidVariant => -19,
    })?;
    let handle = Box::new(PerlinNoiseHandle { noise });
    Ok(Box::into_raw(handle) as jlong)
}

fn perlin_noise_free_handle(handle: jlong) {
    if handle != 0 {
        let handle = handle as *mut PerlinNoiseHandle;
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

fn perlin_noise_get_value(
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    y_max: jni::sys::jdouble,
    use_fixed_y: jboolean,
) -> jni::sys::jdouble {
    if handle == 0 {
        return f64::NAN;
    }

    let handle = unsafe { &*(handle as *const PerlinNoiseHandle) };
    handle
        .noise
        .get_value(x, y, z, y_scale, y_max, use_fixed_y != 0)
}

fn perlin_noise_get_value_no_y_scale(
    handle: jlong,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
) -> jni::sys::jdouble {
    if handle == 0 {
        return f64::NAN;
    }

    let handle = unsafe { &*(handle as *const PerlinNoiseHandle) };
    handle.noise.get_value_direct_math_wrap(x, y, z)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeCheck(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    1
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeGetValue(
    _env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    x: jni::sys::jdouble,
    y: jni::sys::jdouble,
    z: jni::sys::jdouble,
) -> jni::sys::jdouble {
    if first_handle == 0 || second_handle == 0 {
        return f64::NAN;
    }

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    normal_noise::get_value(&first.noise, &second.noise, value_factor, x, y, z)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillVertical(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    x: jni::sys::jdouble,
    start_y: jni::sys::jdouble,
    y_step: jni::sys::jdouble,
    z: jni::sys::jdouble,
    output: jdoubleArray,
) -> jint {
    if first_handle == 0 || second_handle == 0 {
        return -1;
    }

    let output_len = match env.get_array_length(output) {
        Ok(len) => len as usize,
        Err(_) => return -2,
    };

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    let output_elements = match env.get_double_array_elements(output, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -3,
    };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    normal_noise::fill_vertical(&first.noise, &second.noise, value_factor, x, start_y, y_step, z, values);

    output_len as jint
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillCell(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    cell_width: jint,
    cell_height: jint,
    base_x: jni::sys::jdouble,
    base_y: jni::sys::jdouble,
    base_z: jni::sys::jdouble,
    xz_scale: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    output: jdoubleArray,
) -> jint {
    if first_handle == 0 || second_handle == 0 {
        return -1;
    }

    let cell_width = match usize::try_from(cell_width) {
        Ok(value) => value,
        Err(_) => return -2,
    };
    let cell_height = match usize::try_from(cell_height) {
        Ok(value) => value,
        Err(_) => return -3,
    };
    let output_len = match env.get_array_length(output) {
        Ok(len) => len as usize,
        Err(_) => return -4,
    };

    let expected_len = match cell_width
        .checked_mul(cell_width)
        .and_then(|value| value.checked_mul(cell_height))
    {
        Some(value) => value,
        None => return -5,
    };
    if output_len != expected_len {
        return -5;
    }

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    let mut output_elements = match env.get_double_array_elements(output, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -6,
    };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    if normal_noise::fill_cell(
        &first.noise,
        &second.noise,
        value_factor,
        cell_width,
        cell_height,
        base_x,
        base_y,
        base_z,
        xz_scale,
        y_scale,
        values,
    )
    .is_err()
    {
        output_elements.discard();
        return -5;
    }

    output_len as jint
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillPositions(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    output: jdoubleArray,
) -> jint {
    if first_handle == 0 || second_handle == 0 {
        return -1;
    }

    let xs_len = match env.get_array_length(xs) {
        Ok(len) => len as usize,
        Err(_) => return -2,
    };
    let ys_len = match env.get_array_length(ys) {
        Ok(len) => len as usize,
        Err(_) => return -3,
    };
    let zs_len = match env.get_array_length(zs) {
        Ok(len) => len as usize,
        Err(_) => return -4,
    };
    if xs_len != ys_len || xs_len != zs_len {
        return -5;
    }

    let output_len = match env.get_array_length(output) {
        Ok(len) => len as usize,
        Err(_) => return -6,
    };
    if output_len != xs_len {
        return -7;
    }

    let xs_elements = match env.get_double_array_elements(xs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -8,
    };
    let ys_elements = match env.get_double_array_elements(ys, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -9,
    };
    let zs_elements = match env.get_double_array_elements(zs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -10,
    };

    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    let mut output_elements = match env.get_double_array_elements(output, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -12,
    };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    if normal_noise::fill_positions(&first.noise, &second.noise, value_factor, xs, ys, zs, values).is_err() {
        output_elements.discard();
        return -11;
    }

    output_len as jint
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillScaledPositions(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    xz_scale: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
    output: jdoubleArray,
) -> jint {
    normal_noise_fill_int3(
        env,
        first_handle,
        second_handle,
        value_factor,
        xs,
        ys,
        zs,
        output,
        |first, second, value_factor, xs, ys, zs, output| {
            normal_noise::fill_scaled_positions(
                first,
                second,
                value_factor,
                xs,
                ys,
                zs,
                xz_scale,
                y_scale,
                output,
            )
        },
    )
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillShiftedPositionsInPlace(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    shift_x_and_output: jdoubleArray,
    shift_y: jdoubleArray,
    shift_z: jdoubleArray,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    xz_scale: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
) -> jint {
    normal_noise_fill_shifted_in_place(
        env,
        first_handle,
        second_handle,
        value_factor,
        shift_x_and_output,
        shift_y,
        shift_z,
        xs,
        ys,
        zs,
        xz_scale,
        y_scale,
    )
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillShiftPositions(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    output: jdoubleArray,
) -> jint {
    normal_noise_fill_int3(
        env,
        first_handle,
        second_handle,
        value_factor,
        xs,
        ys,
        zs,
        output,
        normal_noise::fill_shift_positions,
    )
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillShiftA(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jintArray,
    zs: jintArray,
    output: jdoubleArray,
) -> jint {
    normal_noise_fill_int2(
        env,
        first_handle,
        second_handle,
        value_factor,
        xs,
        zs,
        output,
        normal_noise::fill_shift_a,
    )
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_world_level_levelgen_synth_PaperNativeNormalNoise_nativeFillShiftB(
    env: JNIEnv,
    _class: JClass,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jintArray,
    zs: jintArray,
    output: jdoubleArray,
) -> jint {
    normal_noise_fill_int2(
        env,
        first_handle,
        second_handle,
        value_factor,
        xs,
        zs,
        output,
        normal_noise::fill_shift_b,
    )
}

fn normal_noise_fill_shifted_in_place(
    env: JNIEnv,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    shift_x_and_output: jdoubleArray,
    shift_y: jdoubleArray,
    shift_z: jdoubleArray,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    xz_scale: jni::sys::jdouble,
    y_scale: jni::sys::jdouble,
) -> jint {
    if first_handle == 0 || second_handle == 0 {
        return -1;
    }

    let output_len = match env.get_array_length(shift_x_and_output) {
        Ok(len) => len as usize,
        Err(_) => return -2,
    };
    let shift_y_len = match env.get_array_length(shift_y) {
        Ok(len) => len as usize,
        Err(_) => return -3,
    };
    let shift_z_len = match env.get_array_length(shift_z) {
        Ok(len) => len as usize,
        Err(_) => return -4,
    };
    let xs_len = match env.get_array_length(xs) {
        Ok(len) => len as usize,
        Err(_) => return -5,
    };
    let ys_len = match env.get_array_length(ys) {
        Ok(len) => len as usize,
        Err(_) => return -6,
    };
    let zs_len = match env.get_array_length(zs) {
        Ok(len) => len as usize,
        Err(_) => return -7,
    };
    if output_len != shift_y_len
        || output_len != shift_z_len
        || output_len != xs_len
        || output_len != ys_len
        || output_len != zs_len
    {
        return -8;
    }

    let shift_y_elements = match env.get_double_array_elements(shift_y, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -9,
    };
    let shift_z_elements = match env.get_double_array_elements(shift_z, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -10,
    };
    let xs_elements = match env.get_int_array_elements(xs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -11,
    };
    let ys_elements = match env.get_int_array_elements(ys, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -12,
    };
    let zs_elements = match env.get_int_array_elements(zs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -13,
    };

    let shift_y = unsafe { std::slice::from_raw_parts(shift_y_elements.as_ptr(), shift_y_len) };
    let shift_z = unsafe { std::slice::from_raw_parts(shift_z_elements.as_ptr(), shift_z_len) };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    let mut output_elements = match env.get_double_array_elements(shift_x_and_output, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -14,
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    if normal_noise::fill_shifted_positions_in_place(
        &first.noise,
        &second.noise,
        value_factor,
        output,
        shift_y,
        shift_z,
        xs,
        ys,
        zs,
        xz_scale,
        y_scale,
    )
    .is_err()
    {
        output_elements.discard();
        return -15;
    }

    output_len as jint
}

fn normal_noise_fill_int3(
    env: JNIEnv,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    output: jdoubleArray,
    fill: impl FnOnce(&perlin_noise::PerlinNoise, &perlin_noise::PerlinNoise, f64, &[i32], &[i32], &[i32], &mut [f64]) -> Result<(), &'static str>,
) -> jint {
    if first_handle == 0 || second_handle == 0 {
        return -1;
    }

    let xs_len = match env.get_array_length(xs) {
        Ok(len) => len as usize,
        Err(_) => return -2,
    };
    let ys_len = match env.get_array_length(ys) {
        Ok(len) => len as usize,
        Err(_) => return -3,
    };
    let zs_len = match env.get_array_length(zs) {
        Ok(len) => len as usize,
        Err(_) => return -4,
    };
    if xs_len != ys_len || xs_len != zs_len {
        return -5;
    }

    let output_len = match env.get_array_length(output) {
        Ok(len) => len as usize,
        Err(_) => return -6,
    };
    if output_len != xs_len {
        return -7;
    }

    let xs_elements = match env.get_int_array_elements(xs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -8,
    };
    let ys_elements = match env.get_int_array_elements(ys, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -9,
    };
    let zs_elements = match env.get_int_array_elements(zs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -10,
    };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    let mut output_elements = match env.get_double_array_elements(output, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -12,
    };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    if fill(&first.noise, &second.noise, value_factor, xs, ys, zs, values).is_err() {
        output_elements.discard();
        return -11;
    }

    output_len as jint
}

fn normal_noise_fill_int2(
    env: JNIEnv,
    first_handle: jlong,
    second_handle: jlong,
    value_factor: jni::sys::jdouble,
    xs: jintArray,
    zs: jintArray,
    output: jdoubleArray,
    fill: impl FnOnce(&perlin_noise::PerlinNoise, &perlin_noise::PerlinNoise, f64, &[i32], &[i32], &mut [f64]) -> Result<(), &'static str>,
) -> jint {
    if first_handle == 0 || second_handle == 0 {
        return -1;
    }

    let xs_len = match env.get_array_length(xs) {
        Ok(len) => len as usize,
        Err(_) => return -2,
    };
    let zs_len = match env.get_array_length(zs) {
        Ok(len) => len as usize,
        Err(_) => return -3,
    };
    if xs_len != zs_len {
        return -5;
    }

    let output_len = match env.get_array_length(output) {
        Ok(len) => len as usize,
        Err(_) => return -6,
    };
    if output_len != xs_len {
        return -7;
    }

    let xs_elements = match env.get_int_array_elements(xs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -8,
    };
    let zs_elements = match env.get_int_array_elements(zs, ReleaseMode::NoCopyBack) {
        Ok(elements) => elements,
        Err(_) => return -10,
    };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };

    let first = unsafe { &*(first_handle as *const PerlinNoiseHandle) };
    let second = unsafe { &*(second_handle as *const PerlinNoiseHandle) };
    let mut output_elements = match env.get_double_array_elements(output, ReleaseMode::CopyBack) {
        Ok(elements) => elements,
        Err(_) => return -12,
    };
    let values = unsafe { std::slice::from_raw_parts_mut(output_elements.as_ptr(), output_len) };
    if fill(&first.noise, &second.noise, value_factor, xs, zs, values).is_err() {
        output_elements.discard();
        return -11;
    }

    output_len as jint
}

fn perlin_noise_get_value_variant_batch_summary(
    env: &mut JNIEnv,
    permutations: jbyteArray,
    active: jbyteArray,
    y_origins: jdoubleArray,
    amplitudes: jdoubleArray,
    xs: jdoubleArray,
    ys: jdoubleArray,
    zs: jdoubleArray,
    lowest_freq_input_factor: jdouble,
    lowest_freq_value_factor: jdouble,
    iterations: jint,
    variant: jint,
    dst: jlongArray,
) -> Result<usize, jint> {
    let iterations = usize::try_from(iterations).map_err(|_| -1)?;
    let variant = u8::try_from(variant)
        .ok()
        .and_then(|code| perlin_noise::PerlinGetValueVariant::try_from(code).ok())
        .ok_or(-2)?;

    let active_len = env.get_array_length(active).map_err(|_| -3)? as usize;
    if active_len == 0 {
        return Err(-4);
    }
    let permutation_len = env.get_array_length(permutations).map_err(|_| -5)? as usize;
    if permutation_len != active_len * improved_noise::PERMUTATION_LENGTH {
        return Err(-6);
    }

    let y_origins_len = env.get_array_length(y_origins).map_err(|_| -7)? as usize;
    let amplitudes_len = env.get_array_length(amplitudes).map_err(|_| -8)? as usize;
    if y_origins_len != active_len || amplitudes_len != active_len {
        return Err(-9);
    }

    let xs_len = env.get_array_length(xs).map_err(|_| -10)? as usize;
    let ys_len = env.get_array_length(ys).map_err(|_| -11)? as usize;
    let zs_len = env.get_array_length(zs).map_err(|_| -12)? as usize;
    if xs_len != ys_len || xs_len != zs_len {
        return Err(-13);
    }

    let dst_len = env.get_array_length(dst).map_err(|_| -14)? as usize;
    if dst_len < perlin_noise::SUMMARY_FIELDS {
        return Err(-(perlin_noise::SUMMARY_FIELDS as jint));
    }

    let permutations_elements = env
        .get_byte_array_elements(permutations, ReleaseMode::NoCopyBack)
        .map_err(|_| -15)?;
    let active_elements = env
        .get_byte_array_elements(active, ReleaseMode::NoCopyBack)
        .map_err(|_| -16)?;
    let y_origins_elements = env
        .get_double_array_elements(y_origins, ReleaseMode::NoCopyBack)
        .map_err(|_| -17)?;
    let amplitudes_elements = env
        .get_double_array_elements(amplitudes, ReleaseMode::NoCopyBack)
        .map_err(|_| -18)?;
    let xs_elements = env
        .get_double_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -19)?;
    let ys_elements = env
        .get_double_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -20)?;
    let zs_elements = env
        .get_double_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -21)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -22)?;

    let permutations = unsafe {
        std::slice::from_raw_parts(permutations_elements.as_ptr() as *const u8, permutation_len)
    };
    let active =
        unsafe { std::slice::from_raw_parts(active_elements.as_ptr() as *const u8, active_len) };
    let y_origins = unsafe { std::slice::from_raw_parts(y_origins_elements.as_ptr(), y_origins_len) };
    let amplitudes = unsafe { std::slice::from_raw_parts(amplitudes_elements.as_ptr(), amplitudes_len) };
    let xs = unsafe { std::slice::from_raw_parts(xs_elements.as_ptr(), xs_len) };
    let ys = unsafe { std::slice::from_raw_parts(ys_elements.as_ptr(), ys_len) };
    let zs = unsafe { std::slice::from_raw_parts(zs_elements.as_ptr(), zs_len) };
    let dst = unsafe {
        std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, perlin_noise::SUMMARY_FIELDS)
    };

    let summary = perlin_noise::get_value_variant_batch_summary(
        permutations,
        active,
        y_origins,
        amplitudes,
        xs,
        ys,
        zs,
        lowest_freq_input_factor,
        lowest_freq_value_factor,
        iterations,
        variant,
    )
    .map_err(|code| match code {
        perlin_noise::PerlinNoiseError::InvalidOctaveCount => -23,
        perlin_noise::PerlinNoiseError::InvalidInputLength => -24,
        perlin_noise::PerlinNoiseError::InvalidPermutationLength => -25,
        perlin_noise::PerlinNoiseError::InvalidVariant => -26,
    })?;

    dst[0] = summary.count as i64;
    dst[1] = summary.sum_bits as i64;
    dst[2] = summary.value_checksum as i64;
    dst[3] = summary.last_bits as i64;

    Ok(perlin_noise::SUMMARY_FIELDS)
}

fn area_map_max_update_ops_capacity(
    old_distance: jint,
    new_distance: jint,
) -> Result<usize, jint> {
    if old_distance < 0 || new_distance < 0 {
        return Err(-1);
    }

    let radius = i64::from(old_distance.max(new_distance));
    let side = radius
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(-6)?;
    let required = side
        .checked_mul(side)
        .and_then(|value| value.checked_mul(2))
        .ok_or(-6)?;
    usize::try_from(required).map_err(|_| -6)
}

fn area_map_square_ops_capacity(distance: jint) -> Result<usize, jint> {
    if distance < 0 {
        return Err(-1);
    }

    let radius = i64::from(distance);
    let side = radius
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(-6)?;
    let required = side.checked_mul(side).ok_or(-6)?;
    usize::try_from(required).map_err(|_| -6)
}

fn area_map_op_from_jint(op: jint) -> Result<area_map::AreaOp, jint> {
    match op {
        0 => Ok(area_map::AreaOp::Add),
        1 => Ok(area_map::AreaOp::Remove),
        _ => Err(-7),
    }
}

fn chunk_hash_batch(
    env: &mut JNIEnv,
    xs: jintArray,
    zs: jintArray,
    dst: jintArray,
) -> Result<usize, jint> {
    let count = matching_pair_len(env, xs, zs)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -4)? as usize;
    if dst_len < count {
        return Err(-(count as jint));
    }

    let x_elements = env
        .get_int_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -5)?;
    let z_elements = env
        .get_int_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let dst_elements = env
        .get_int_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -7)?;

    let xs = unsafe { std::slice::from_raw_parts(x_elements.as_ptr(), count) };
    let zs = unsafe { std::slice::from_raw_parts(z_elements.as_ptr(), count) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr(), count) };

    for index in 0..count {
        dst[index] = position::chunk_hash(xs[index], zs[index]);
    }
    Ok(count)
}

fn section_pack_batch(
    env: &mut JNIEnv,
    xs: jintArray,
    ys: jintArray,
    zs: jintArray,
    dst: jlongArray,
) -> Result<usize, jint> {
    let count = matching_triple_len(env, xs, ys, zs)?;
    let dst_len = env.get_array_length(dst).map_err(|_| -5)? as usize;
    if dst_len < count {
        return Err(-(count as jint));
    }

    let x_elements = env
        .get_int_array_elements(xs, ReleaseMode::NoCopyBack)
        .map_err(|_| -6)?;
    let y_elements = env
        .get_int_array_elements(ys, ReleaseMode::NoCopyBack)
        .map_err(|_| -7)?;
    let z_elements = env
        .get_int_array_elements(zs, ReleaseMode::NoCopyBack)
        .map_err(|_| -8)?;
    let dst_elements = env
        .get_long_array_elements(dst, ReleaseMode::CopyBack)
        .map_err(|_| -9)?;

    let xs = unsafe { std::slice::from_raw_parts(x_elements.as_ptr(), count) };
    let ys = unsafe { std::slice::from_raw_parts(y_elements.as_ptr(), count) };
    let zs = unsafe { std::slice::from_raw_parts(z_elements.as_ptr(), count) };
    let dst = unsafe { std::slice::from_raw_parts_mut(dst_elements.as_ptr() as *mut i64, count) };

    for index in 0..count {
        dst[index] = position::section_as_long(xs[index], ys[index], zs[index]);
    }
    Ok(count)
}

fn matching_pair_len(env: &mut JNIEnv, first: jintArray, second: jintArray) -> Result<usize, jint> {
    let first_len = env.get_array_length(first).map_err(|_| -1)? as usize;
    let second_len = env.get_array_length(second).map_err(|_| -2)? as usize;
    if first_len != second_len {
        return Err(-3);
    }
    Ok(first_len)
}

fn matching_triple_len(
    env: &mut JNIEnv,
    first: jintArray,
    second: jintArray,
    third: jintArray,
) -> Result<usize, jint> {
    let first_len = env.get_array_length(first).map_err(|_| -1)? as usize;
    let second_len = env.get_array_length(second).map_err(|_| -2)? as usize;
    let third_len = env.get_array_length(third).map_err(|_| -3)? as usize;
    if first_len != second_len || first_len != third_len {
        return Err(-4);
    }
    Ok(first_len)
}
