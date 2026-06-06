#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/density-ap2-mapall/DensityAp2MapAllBench.java"
OUT="$ROOT/bench/density-ap2-mapall/.classes"
STAMP="${BENCH_STAMP:-$(date +%Y%m%d-%H%M%S)}"
REPORT="$ROOT/reports/density-ap2-mapall-bench-$STAMP.txt"
CURRENT_REPORT="$ROOT/reports/density-ap2-mapall-bench-current.txt"
RUNTIME_CP="$(cat "$ROOT/artifacts/optimized-runtime/classpath.txt")"
CLASSPATH="$OUT:$RUNTIME_CP"
JAVA_PROPS_ARRAY=()

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_PROPS_ARRAY=(${JAVA_PROPS})
else
  JAVA_PROPS_ARRAY=(
    "-Droots=${DENSITY_AP2_MAPALL_ROOTS:-256}"
    "-Ddepth=${DENSITY_AP2_MAPALL_DEPTH:-24}"
    "-Diterations=${DENSITY_AP2_MAPALL_ITERATIONS:-500}"
    "-Dwarmup=${DENSITY_AP2_MAPALL_WARMUP:-3}"
    "-Drounds=${DENSITY_AP2_MAPALL_ROUNDS:-7}"
  )
fi

mkdir -p "$OUT" "$ROOT/reports"
javac -proc:none -cp "$RUNTIME_CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "runtime_jar_sha256=$(sha256sum "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" | awk '{print $1}')"
  echo "classpath_sha256=$(printf '%s' "$CLASSPATH" | sha256sum | awk '{print $1}')"
  echo "command=java -Xms512m -Xmx512m ${JAVA_PROPS_ARRAY[*]} -cp <optimized-runtime-classpath> DensityAp2MapAllBench"
  java -Xms512m -Xmx512m "${JAVA_PROPS_ARRAY[@]}" -cp "$CLASSPATH" DensityAp2MapAllBench
} | tee "$REPORT" "$CURRENT_REPORT"
