#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TASKSET_CMD=()
if [[ -n "${BENCHMARK_CPUSET:-}" ]]; then
  if ! command -v taskset >/dev/null 2>&1; then
    echo "BENCHMARK_CPUSET is set but taskset is unavailable." >&2
    exit 1
  fi
  TASKSET_CMD=(taskset -c "$BENCHMARK_CPUSET")
fi
CLIENT_DIR="$ROOT/target/mc-join-client"
PACKAGE="$CLIENT_DIR/node_modules/minecraft-protocol/package.json"
VERSION="1.66.0"

if [[ ! -f "$PACKAGE" ]] || ! node -e "const pkg=require('$PACKAGE'); process.exit(pkg.version === '$VERSION' ? 0 : 1)" 2>/dev/null; then
  mkdir -p "$CLIENT_DIR"
  env -u NODE_TLS_REJECT_UNAUTHORIZED npm install --prefix "$CLIENT_DIR" --no-audit --no-fund --silent "minecraft-protocol@$VERSION"
fi

env -u NODE_TLS_REJECT_UNAUTHORIZED NODE_PATH="$CLIENT_DIR/node_modules" "${TASKSET_CMD[@]}" node "$ROOT/scripts/mc_join_client.cjs" "$@"
