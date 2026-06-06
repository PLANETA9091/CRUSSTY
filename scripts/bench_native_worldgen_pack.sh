#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT="${ROOT}/reports/native-worldgen-pack.txt"
PACK_MODE="${PACK_MODE:-core}"
PACK_FAIL_FAST="${PACK_FAIL_FAST:-1}"

CORE_SCRIPTS=(
  scripts/bench_native_improved_noise.sh
  scripts/bench_native_improved_noise_inline.sh
  scripts/bench_native_improved_noise_floor.sh
  scripts/bench_native_improved_noise_derivative.sh
  scripts/bench_native_perlin_noise.sh
  scripts/bench_native_perlin_getvalue.sh
  scripts/bench_native_blended_noise.sh
  scripts/bench_native_noise_generator_settings.sh
)

FULL_EXTRA_SCRIPTS=(
  scripts/bench_native_noise_interpolator_fractions.sh
  scripts/bench_native_noise_interpolator_slice.sh
  scripts/bench_native_noisechunk_blendcache.sh
  scripts/bench_native_noisechunk_flatcache_context.sh
  scripts/bench_native_noisechunk_interpolator_array.sh
  scripts/bench_native_noisechunk_wrap_capacity.sh
  scripts/bench_native_aquifer_index_stride.sh
  scripts/bench_native_aquifer_positional_location.sh
  scripts/bench_native_aquifer_surface_sampling.sh
  scripts/bench_native_density_ap2_fill.sh
  scripts/bench_native_density_ap2_minmax_fill.sh
  scripts/bench_native_density_spline_context.sh
  scripts/bench_native_density_visitor_hooks.sh
  scripts/bench_native_range_choice.sh
  scripts/bench_native_surface_rules_sequence_array.sh
  scripts/bench_native_surface_rules_test_rule_state.sh
  scripts/bench_native_placed_feature_traversal.sh
  scripts/bench_native_ore_feature_loop.sh
  scripts/bench_native_carver_iteration.sh
  scripts/bench_native_cave_carver_skip.sh
  scripts/bench_native_jigsaw_canattach.sh
  scripts/bench_native_levelchunk_heightmap.sh
)

if [[ -n "${PACK_SCRIPTS:-}" ]]; then
  # shellcheck disable=SC2206
  SCRIPTS=(${PACK_SCRIPTS})
elif [[ "${PACK_MODE}" == "full" ]]; then
  SCRIPTS=("${CORE_SCRIPTS[@]}" "${FULL_EXTRA_SCRIPTS[@]}")
else
  SCRIPTS=("${CORE_SCRIPTS[@]}")
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
  echo "pack_mode=${PACK_MODE}"
  echo "script_count=${#SCRIPTS[@]}"
  echo "pack_fail_fast=${PACK_FAIL_FAST}"
} | tee "${REPORT}"

"${ROOT}/scripts/native_coverage_audit.py" | tee -a "${REPORT}"
"${ROOT}/scripts/build_native.sh" >/dev/null

failures=0
for script in "${SCRIPTS[@]}"; do
  script_path="${ROOT}/${script}"
  if [[ ! -x "${script_path}" && ! -f "${script_path}" ]]; then
    echo "PACK_RESULT script=${script} status=MISSING" | tee -a "${REPORT}"
    failures=$((failures + 1))
    if [[ "${PACK_FAIL_FAST}" == "1" ]]; then
      break
    fi
    continue
  fi

  echo "PACK_START script=${script}" | tee -a "${REPORT}"
  start_ns="$(date +%s%N)"
  if SKIP_NATIVE_BUILD=1 "${script_path}" >>"${REPORT}" 2>&1; then
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
  exit 1
fi

echo "pack_status=PASS failures=0" | tee -a "${REPORT}"
