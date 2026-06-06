#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/bench/netty-lazy-execute/NettyLazyExecuteBench.java"
OUT="$ROOT/build/bench/netty-lazy-execute"
REPORT="$ROOT/reports/netty-lazy-execute-bench.txt"
BUNDLER_LIB_ROOT="${BUNDLER_LIB_ROOT:-$ROOT/artifacts/optimized-runtime/bundler/libraries}"
GRADLE_NETTY_ROOT="${GRADLE_NETTY_ROOT:-$HOME/.gradle/caches/modules-2/files-2.1/io.netty}"
NETTY_JARS=()
JAVA_ARGS=()

find_bundled_netty_jars() {
  local netty_root="$1/io/netty"

  [[ -d "$netty_root" ]] || return 1
  mapfile -t NETTY_JARS < <(find "$netty_root" -type f -name 'netty-*.jar' | sort)
  [[ "${#NETTY_JARS[@]}" -gt 0 ]]
}

find_latest_gradle_netty_jar() {
  local module="$1"
  local module_root="$GRADLE_NETTY_ROOT/$module"

  [[ -d "$module_root" ]] || return 1
  find "$module_root" -type f -name "$module-*.jar" | sort -V | tail -n 1
}

find_gradle_netty_jars() {
  local modules=(netty-common netty-buffer netty-resolver netty-transport)
  local module
  local jar

  NETTY_JARS=()
  for module in "${modules[@]}"; do
    jar="$(find_latest_gradle_netty_jar "$module")"
    [[ -n "$jar" ]] || return 1
    NETTY_JARS+=("$jar")
  done
}

NETTY_SOURCE=""
if find_bundled_netty_jars "$BUNDLER_LIB_ROOT"; then
  NETTY_SOURCE="$BUNDLER_LIB_ROOT"
else
  find_gradle_netty_jars
  NETTY_SOURCE="$GRADLE_NETTY_ROOT"
fi

CP="$(IFS=:; echo "${NETTY_JARS[*]}")"

if [[ -n "${JAVA_PROPS:-}" ]]; then
  # shellcheck disable=SC2206
  JAVA_ARGS=(${JAVA_PROPS})
fi

mkdir -p "$OUT" "$(dirname "$REPORT")"
javac -cp "$CP" -d "$OUT" "$SRC"

{
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host=$(hostname)"
  echo "kernel=$(uname -srmo)"
  lscpu | sed -n 's/^Model name:[[:space:]]*/cpu_model=/p; s/^CPU(s):[[:space:]]*/cpu_count=/p'
  echo "java=$(java -version 2>&1 | head -n 1)"
  echo "netty_source=$NETTY_SOURCE"
  echo "netty_jars_count=${#NETTY_JARS[@]}"
  echo "classpath=$CP"
  echo "command=java ${JAVA_ARGS[*]} -cp \"$OUT:$CP\" NettyLazyExecuteBench"
  java "${JAVA_ARGS[@]}" -cp "$OUT:$CP" NettyLazyExecuteBench
} | tee "$REPORT"
