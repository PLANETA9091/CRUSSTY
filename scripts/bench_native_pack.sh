#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACK_GROUPS="${PACK_GROUPS:-worldgen}"
PACK_FAIL_FAST="${PACK_FAIL_FAST:-1}"
PACK_LABEL="${PACK_LABEL:-${PACK_GROUPS// /-}}"
PACK_LABEL="${PACK_LABEL//[^A-Za-z0-9_.-]/-}"
REPORT="${PACK_REPORT:-${ROOT}/reports/native-pack-${PACK_LABEL}.txt}"
PACK_REQUIRE_ALL_REAL="${PACK_REQUIRE_ALL_REAL:-1}"
PACK_REQUIRE_GROUP_OWNERSHIP="${PACK_REQUIRE_GROUP_OWNERSHIP:-1}"
PACK_WRITE_MANIFEST="${PACK_WRITE_MANIFEST:-0}"

LEAF_GROUPS=(
  worldgen
  worldgen-extra
  aquifer
  climate
  entity
  waypoint
  plugin
  storage
  network
  core
  ticket
)

WORLDGEN_SCRIPTS=(
  scripts/bench_native_improved_noise.sh
  scripts/bench_native_improved_noise_inline.sh
  scripts/bench_native_improved_noise_derivative.sh
  scripts/bench_native_perlin_noise.sh
  scripts/bench_native_normal_noise.sh
  scripts/bench_native_perlin_getvalue.sh
  scripts/bench_native_blended_noise.sh
  scripts/bench_native_noise_generator_settings.sh
  scripts/bench_native_density_ap2_fill.sh
  scripts/bench_native_density_ap2_minmax_fill.sh
  scripts/bench_native_density_visitor_hooks.sh
  scripts/bench_native_surface_rules_sequence_array.sh
  scripts/bench_native_surface_rules_test_rule_state.sh
  scripts/bench_native_placed_feature_traversal.sh
  scripts/bench_native_ore_feature_loop.sh
  scripts/bench_native_carver_iteration.sh
  scripts/bench_native_cave_carver_skip.sh
)

WORLDGEN_EXTRA_SCRIPTS=(
  scripts/bench_native_beardifier_bury.sh
  scripts/bench_native_biome_getbiome.sh
  scripts/bench_native_cubic_spline_create.sh
  scripts/bench_native_density_spline_context.sh
  scripts/bench_native_improved_noise_floor.sh
  scripts/bench_native_jigsaw_canattach.sh
  scripts/bench_native_noise_interpolator_fractions.sh
  scripts/bench_native_noise_interpolator_slice.sh
  scripts/bench_native_noisechunk_blendcache.sh
  scripts/bench_native_noisechunk_flatcache_context.sh
  scripts/bench_native_noisechunk_interpolator_array.sh
  scripts/bench_native_noisechunk_wrap_capacity.sh
  scripts/bench_native_range_choice.sh
  scripts/bench_native_shift_noise_direct.sh
  scripts/bench_native_spring_feature_mutable_pos.sh
  scripts/bench_native_xoroshiro_positional_direct.sh
  scripts/bench_native_yclamped_gradient.sh
)

AQUIFER_SCRIPTS=(
  scripts/bench_native_aquifer_index_stride.sh
  scripts/bench_native_aquifer_positional_location.sh
  scripts/bench_native_aquifer_surface_sampling.sh
)

CLIMATE_SCRIPTS=(
  scripts/bench_native_climate.sh
  scripts/bench_native_climate_parameter_distance.sh
  scripts/bench_native_climate_rtree_build.sh
  scripts/bench_native_climate_rtree_search.sh
  scripts/bench_native_climate_rtree_lifecycle.sh
  scripts/bench_native_climate_rtree_arena.sh
  scripts/bench_native_climate_rtree_batch_borrow.sh
  scripts/bench_native_climate_rtree_jni.sh
)

ENTITY_SCRIPTS=(
  scripts/bench_native_entity_bounding_box.sh
  scripts/bench_native_entity_lookup_status.sh
  scripts/bench_native_entity_chunk_transient.sh
  scripts/bench_native_serverentity_delta_identity.sh
  scripts/bench_native_chunk_expire_count.sh
  scripts/bench_native_craftplayer_cansee.sh
  scripts/bench_native_levelchunk_heightmap.sh
  scripts/bench_native_protochunk_heightmap.sh
  scripts/bench_native_nearby_player_map.sh
  scripts/bench_native_marker_cache.sh
  scripts/bench_native_ownable_rule.sh
)

WAYPOINT_SCRIPTS=(
  scripts/bench_native_waypoint_distance_guard.sh
  scripts/bench_native_waypoint_chunk_update.sh
  scripts/bench_native_waypoint_snapshot.sh
  scripts/bench_native_waypoint_table_view.sh
  scripts/bench_native_waypoint_manager_skip.sh
  scripts/bench_native_waypoint_hotpath.sh
)

PLUGIN_SCRIPTS=(
  scripts/bench_native_plugin_name_join.sh
  scripts/bench_native_plugin_name_log.sh
  scripts/bench_native_plugin_startup_rollup.sh
  scripts/bench_native_plugin_directory_scan.sh
  scripts/bench_native_plugin_meta_dependency.sh
  scripts/bench_native_plugin_classloader_group.sh
  scripts/bench_native_plugin_loading_allocation.sh
  scripts/bench_native_spigot_load_order_dependency.sh
  scripts/bench_native_legacy_provided_alias_removal.sh
  scripts/bench_native_topographic_graph_sort_capacity.sh
  scripts/bench_native_remapper_index_cleanup.sh
  scripts/bench_native_remapper_skip_hashes.sh
  scripts/bench_native_remapper_hash_threshold.sh
  scripts/bench_native_obfhelper_maps.sh
)

STORAGE_SCRIPTS=(
  scripts/bench_native_varint.sh
  scripts/bench_native_hash.sh
  scripts/bench_native_hash_path.sh
  scripts/bench_native_nbt_compound_map_capacity.sh
  scripts/bench_native_nbt_gzip_buffer_shape.sh
  scripts/bench_native_lz4_stream_roundtrip.sh
  scripts/bench_native_compression_threshold_shape.sh
  scripts/bench_native_deflater_input_shape.sh
  scripts/bench_native_paletted_reencode_scratch.sh
  scripts/bench_native_paletted_reencode_remap_cache.sh
)

NETWORK_SCRIPTS=(
  scripts/bench_native_chunk_packet_encode.sh
)

CORE_SCRIPTS=(
  scripts/bench_native_position.sh
  scripts/bench_native_area_map.sh
  scripts/bench_native_chunk_dependencies_array.sh
  scripts/bench_native_reference_list.sh
  scripts/bench_native_reference_list_bridge.sh
  scripts/bench_native_reference_list_stateful.sh
  scripts/bench_native_static_cache_get.sh
)

TICKET_SCRIPTS=(
  scripts/bench_native_ticket_compare.sh
  scripts/bench_native_ticket_pack.sh
  scripts/bench_native_ticketset_search.sh
  scripts/bench_native_chunk_ticket_stage.sh
)

SCRIPTS=()
PACK_HAS_ALL=0
ALL_REAL_EXPECTED_COUNT=""
ALL_REAL_COVERED_COUNT=""
LEAF_GROUP_COUNT=""
LEAF_GROUP_MEMBERSHIP_COUNT=""

append_unique() {
  local candidate existing
  for candidate in "$@"; do
    for existing in "${SCRIPTS[@]}"; do
      if [[ "${existing}" == "${candidate}" ]]; then
        continue 2
      fi
    done
    SCRIPTS+=("${candidate}")
  done
}

append_group() {
  case "$1" in
    worldgen) append_unique "${WORLDGEN_SCRIPTS[@]}" ;;
    worldgen-extra) append_unique "${WORLDGEN_EXTRA_SCRIPTS[@]}" ;;
    aquifer) append_unique "${AQUIFER_SCRIPTS[@]}" ;;
    climate) append_unique "${CLIMATE_SCRIPTS[@]}" ;;
    entity) append_unique "${ENTITY_SCRIPTS[@]}" ;;
    waypoint) append_unique "${WAYPOINT_SCRIPTS[@]}" ;;
    plugin) append_unique "${PLUGIN_SCRIPTS[@]}" ;;
    storage) append_unique "${STORAGE_SCRIPTS[@]}" ;;
    network) append_unique "${NETWORK_SCRIPTS[@]}" ;;
    core) append_unique "${CORE_SCRIPTS[@]}" ;;
    ticket) append_unique "${TICKET_SCRIPTS[@]}" ;;
    all)
      PACK_HAS_ALL=1
      append_unique "${WORLDGEN_SCRIPTS[@]}"
      append_unique "${WORLDGEN_EXTRA_SCRIPTS[@]}"
      append_unique "${AQUIFER_SCRIPTS[@]}"
      append_unique "${CLIMATE_SCRIPTS[@]}"
      append_unique "${ENTITY_SCRIPTS[@]}"
      append_unique "${WAYPOINT_SCRIPTS[@]}"
      append_unique "${PLUGIN_SCRIPTS[@]}"
      append_unique "${STORAGE_SCRIPTS[@]}"
      append_unique "${NETWORK_SCRIPTS[@]}"
      append_unique "${CORE_SCRIPTS[@]}"
      append_unique "${TICKET_SCRIPTS[@]}"
      ;;
    *)
      echo "unknown PACK_GROUPS entry: $1" >&2
      exit 2
      ;;
  esac
}

scripts_for_group() {
  case "$1" in
    worldgen) printf '%s\n' "${WORLDGEN_SCRIPTS[@]}" ;;
    worldgen-extra) printf '%s\n' "${WORLDGEN_EXTRA_SCRIPTS[@]}" ;;
    aquifer) printf '%s\n' "${AQUIFER_SCRIPTS[@]}" ;;
    climate) printf '%s\n' "${CLIMATE_SCRIPTS[@]}" ;;
    entity) printf '%s\n' "${ENTITY_SCRIPTS[@]}" ;;
    waypoint) printf '%s\n' "${WAYPOINT_SCRIPTS[@]}" ;;
    plugin) printf '%s\n' "${PLUGIN_SCRIPTS[@]}" ;;
    storage) printf '%s\n' "${STORAGE_SCRIPTS[@]}" ;;
    network) printf '%s\n' "${NETWORK_SCRIPTS[@]}" ;;
    core) printf '%s\n' "${CORE_SCRIPTS[@]}" ;;
    ticket) printf '%s\n' "${TICKET_SCRIPTS[@]}" ;;
    *)
      echo "unknown pack group: $1" >&2
      exit 2
      ;;
  esac
}

group_manifest() {
  local group script
  for group in "${LEAF_GROUPS[@]}"; do
    while IFS= read -r script; do
      [[ -n "${script}" ]] || continue
      printf '%s\t%s\n' "${group}" "${script}"
    done < <(scripts_for_group "${group}")
  done
}

list_groups() {
  local group count
  for group in "${LEAF_GROUPS[@]}"; do
    count="$(scripts_for_group "${group}" | wc -l | tr -d ' ')"
    printf '%s\t%s\n' "${group}" "${count}"
  done
}

expected_all_real_scripts() {
  find "${ROOT}/scripts" -maxdepth 1 -name 'bench_native_*.sh' -printf '%f\n' \
    | grep -v -E '^bench_native_(pack|worldgen_pack)\.sh$' \
    | sort
}

selected_script_names() {
  local script
  for script in "${SCRIPTS[@]}"; do
    basename "${script}"
  done | sort -u
}

leaf_group_script_names() {
  group_manifest | awk -F'\t' '{ n=split($2, parts, "/"); print parts[n] }' | sort
}

validate_group_ownership() {
  local missing=()
  local extra=()
  local duplicates=()

  mapfile -t missing < <(comm -23 <(expected_all_real_scripts) <(leaf_group_script_names | sort -u))
  mapfile -t extra < <(comm -13 <(expected_all_real_scripts) <(leaf_group_script_names | sort -u))
  mapfile -t duplicates < <(leaf_group_script_names | uniq -d)

  LEAF_GROUP_COUNT="${#LEAF_GROUPS[@]}"
  LEAF_GROUP_MEMBERSHIP_COUNT="$(leaf_group_script_names | wc -l | tr -d ' ')"

  if (( ${#missing[@]} != 0 || ${#extra[@]} != 0 || ${#duplicates[@]} != 0 )); then
    echo "native pack leaf-group ownership is not exact" >&2
    if (( ${#missing[@]} != 0 )); then
      printf 'missing_from_leaf_groups=%s\n' "${missing[@]}" >&2
    fi
    if (( ${#extra[@]} != 0 )); then
      printf 'extra_in_leaf_groups=%s\n' "${extra[@]}" >&2
    fi
    if (( ${#duplicates[@]} != 0 )); then
      printf 'duplicate_leaf_group_member=%s\n' "${duplicates[@]}" >&2
    fi
    exit 2
  fi
}

validate_all_real_coverage() {
  local missing=()
  local extra=()

  mapfile -t missing < <(comm -23 <(expected_all_real_scripts) <(selected_script_names))
  mapfile -t extra < <(comm -13 <(expected_all_real_scripts) <(selected_script_names))

  ALL_REAL_EXPECTED_COUNT="$(expected_all_real_scripts | wc -l | tr -d ' ')"
  ALL_REAL_COVERED_COUNT="$(selected_script_names | wc -l | tr -d ' ')"

  if (( ${#missing[@]} != 0 || ${#extra[@]} != 0 )); then
    echo "PACK_GROUPS=all does not match real bench_native script coverage" >&2
    if (( ${#missing[@]} != 0 )); then
      printf 'missing_from_all=%s\n' "${missing[@]}" >&2
    fi
    if (( ${#extra[@]} != 0 )); then
      printf 'extra_in_all=%s\n' "${extra[@]}" >&2
    fi
    exit 2
  fi
}

if [[ -n "${PACK_SCRIPTS:-}" ]]; then
  # shellcheck disable=SC2206
  SCRIPTS=(${PACK_SCRIPTS})
else
  # shellcheck disable=SC2206
  GROUP_LIST=(${PACK_GROUPS})
  for group in "${GROUP_LIST[@]}"; do
    append_group "${group}"
  done
fi

if [[ "${PACK_LIST_GROUPS:-0}" == "1" ]]; then
  list_groups
  exit 0
fi

if [[ "${PACK_MANIFEST:-0}" == "1" ]]; then
  group_manifest
  exit 0
fi

if [[ -z "${PACK_SCRIPTS:-}" && "${PACK_REQUIRE_GROUP_OWNERSHIP}" == "1" ]]; then
  validate_group_ownership
fi

if [[ -z "${PACK_SCRIPTS:-}" && "${PACK_HAS_ALL}" == "1" && "${PACK_REQUIRE_ALL_REAL}" == "1" ]]; then
  validate_all_real_coverage
fi

if [[ "${PACK_LIST:-0}" == "1" ]]; then
  printf '%s\n' "${SCRIPTS[@]}"
  exit 0
fi

mkdir -p "${ROOT}/reports"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "pack_groups=${PACK_GROUPS}"
  echo "script_count=${#SCRIPTS[@]}"
  if [[ -n "${LEAF_GROUP_COUNT}" ]]; then
    echo "leaf_group_count=${LEAF_GROUP_COUNT}"
    echo "leaf_group_memberships=${LEAF_GROUP_MEMBERSHIP_COUNT}"
  fi
  if [[ -n "${ALL_REAL_EXPECTED_COUNT}" ]]; then
    echo "all_real_scripts_expected=${ALL_REAL_EXPECTED_COUNT}"
    echo "all_real_scripts_covered=${ALL_REAL_COVERED_COUNT}"
  fi
  echo "pack_fail_fast=${PACK_FAIL_FAST}"
  echo "pack_heavy_defaults=${PACK_HEAVY_DEFAULTS:-1}"
} | tee "${REPORT}"

if [[ "${PACK_WRITE_MANIFEST}" == "1" ]]; then
  group_manifest | awk -F'\t' '{ printf("PACK_MANIFEST group=%s script=%s\n", $1, $2) }' | tee -a "${REPORT}"
fi

"${ROOT}/scripts/native_coverage_audit.py" --strict-docs | tee -a "${REPORT}"
"${ROOT}/scripts/build_native.sh" >/dev/null

failures=0
for script in "${SCRIPTS[@]}"; do
  script_path="${ROOT}/${script}"
  if [[ ! -f "${script_path}" ]]; then
    echo "PACK_RESULT script=${script} status=MISSING" | tee -a "${REPORT}"
    failures=$((failures + 1))
    if [[ "${PACK_FAIL_FAST}" == "1" ]]; then
      break
    fi
    continue
  fi

  echo "PACK_START script=${script}" | tee -a "${REPORT}"
  start_ns="$(date +%s%N)"
  ENV_ARGS=(SKIP_NATIVE_BUILD=1)
  if [[ "${PACK_HEAVY_DEFAULTS:-1}" == "1" ]]; then
    case "${script}" in
      scripts/bench_native_waypoint_hotpath.sh)
        ENV_ARGS+=(
          "WAYPOINT_HOTPATH_WARMUP=${PACK_WAYPOINT_HOTPATH_WARMUP:-0}"
          "WAYPOINT_HOTPATH_ROUNDS=${PACK_WAYPOINT_HOTPATH_ROUNDS:-1}"
        )
        ;;
      scripts/bench_native_remapper_hash_threshold.sh)
        ENV_ARGS+=(
          "HASH_BENCH_ITERATIONS=${PACK_HASH_BENCH_ITERATIONS:-3}"
          "HASH_BENCH_WARMUP=${PACK_HASH_BENCH_WARMUP:-1}"
          "HASH_BENCH_ROUNDS=${PACK_HASH_BENCH_ROUNDS:-2}"
        )
        ;;
    esac
  fi

  if env "${ENV_ARGS[@]}" "${script_path}" >>"${REPORT}" 2>&1; then
    end_ns="$(date +%s%N)"
    duration_ms=$(((end_ns - start_ns) / 1000000))
    echo "PACK_RESULT script=${script} status=PASS duration_ms=${duration_ms}" | tee -a "${REPORT}"
  else
    end_ns="$(date +%s%N)"
    duration_ms=$(((end_ns - start_ns) / 1000000))
    echo "PACK_RESULT script=${script} status=FAIL duration_ms=${duration_ms}" | tee -a "${REPORT}"
    failures=$((failures + 1))
    if [[ "${PACK_FAIL_FAST}" == "1" ]]; then
      break
    fi
  fi
done

if [[ "${failures}" -ne 0 ]]; then
  echo "pack_status=FAIL failures=${failures}" | tee -a "${REPORT}"
  "${ROOT}/scripts/native_pack_report.py" "${REPORT}" | tee -a "${REPORT}" || true
  exit 1
fi

echo "pack_status=PASS failures=0" | tee -a "${REPORT}"
"${ROOT}/scripts/native_pack_report.py" "${REPORT}" | tee -a "${REPORT}"
