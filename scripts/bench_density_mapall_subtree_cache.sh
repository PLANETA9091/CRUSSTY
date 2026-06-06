#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/density-mapall-subtree-cache/DensityMapAllSubtreeCacheBench.java"
OUT="$ROOT/bench/density-mapall-subtree-cache/.classes"
STAMP="${BENCH_STAMP:-$(date +%Y%m%d-%H%M%S)}"
REPORT="$ROOT/reports/density-mapall-subtree-cache-bench-$STAMP.txt"
CURRENT_REPORT="$ROOT/reports/density-mapall-subtree-cache-bench-current.txt"
CLASSPATH="$OUT:$(cat "$ROOT/artifacts/optimized-runtime/classpath.txt")"

mkdir -p "$OUT" "$ROOT/reports"
javac -proc:none -cp "$(cat "$ROOT/artifacts/optimized-runtime/classpath.txt")" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "runtime_jar_sha256=$(sha256sum "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar" | awk '{print $1}')"
  echo "classpath_sha256=$(printf '%s' "$CLASSPATH" | sha256sum | awk '{print $1}')"
  echo "command=java ${JAVA_PROPS:-} -cp <optimized-runtime-classpath> DensityMapAllSubtreeCacheBench"
  java ${JAVA_PROPS:-} -cp "$CLASSPATH" DensityMapAllSubtreeCacheBench
} | tee "$REPORT" "$CURRENT_REPORT"
