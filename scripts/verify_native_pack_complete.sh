#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PACK_REQUIRE_ALL_REAL=1 PACK_LIST=1 PACK_GROUPS=all "${ROOT}/scripts/bench_native_pack.sh" >/dev/null
PACK_LIST_GROUPS=1 PACK_GROUPS=all "${ROOT}/scripts/bench_native_pack.sh" >/dev/null
PACK_MANIFEST=1 PACK_GROUPS=all "${ROOT}/scripts/bench_native_pack.sh" >/dev/null
python3 "${ROOT}/scripts/native_coverage_audit.py" --strict-docs
mapfile -t NATIVE_BENCH_SCRIPTS < <(find "${ROOT}/scripts" -maxdepth 1 -name 'bench_native_*.sh' -type f | sort)
bash -n "${NATIVE_BENCH_SCRIPTS[@]}"
python3 -m py_compile \
  "${ROOT}/scripts/native_coverage_audit.py" \
  "${ROOT}/scripts/native_pack_report.py"
PACK_REPORT="${PACK_REPORT:-${ROOT}/reports/native-pack-mega-all-complete-v4.txt}"
python3 "${ROOT}/scripts/native_pack_report.py" "${PACK_REPORT}"
sha256sum -c "${ROOT}/reports/paper-native-jni.sha256"
sha256sum -c "${ROOT}/reports/paper-native-chunk-encode-jni.sha256"
git diff --check
