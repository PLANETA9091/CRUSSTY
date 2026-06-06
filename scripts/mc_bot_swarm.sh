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
BOT_SWARM_NODE_MAX_OLD_SPACE_MB="${BOT_SWARM_NODE_MAX_OLD_SPACE_MB:-512}"
BOT_SWARM_NODE_MAX_SEMI_SPACE_MB="${BOT_SWARM_NODE_MAX_SEMI_SPACE_MB:-16}"

for numeric_var in BOT_SWARM_NODE_MAX_OLD_SPACE_MB BOT_SWARM_NODE_MAX_SEMI_SPACE_MB; do
  if [[ ! "${!numeric_var}" =~ ^[0-9]+$ ]]; then
    echo "$numeric_var must be a non-negative integer; use 0 to disable this Node heap flag." >&2
    exit 64
  fi
done

NODE_HEAP_FLAGS=()
if (( BOT_SWARM_NODE_MAX_OLD_SPACE_MB > 0 )); then
  NODE_HEAP_FLAGS+=("--max-old-space-size=$BOT_SWARM_NODE_MAX_OLD_SPACE_MB")
fi
if (( BOT_SWARM_NODE_MAX_SEMI_SPACE_MB > 0 )); then
  NODE_HEAP_FLAGS+=("--max-semi-space-size=$BOT_SWARM_NODE_MAX_SEMI_SPACE_MB")
fi

if [[ ! -f "$PACKAGE" ]] || ! node -e "const pkg=require('$PACKAGE'); process.exit(pkg.version === '$VERSION' ? 0 : 1)" 2>/dev/null; then
  mkdir -p "$CLIENT_DIR"
  env -u NODE_TLS_REJECT_UNAUTHORIZED npm install --prefix "$CLIENT_DIR" --no-audit --no-fund --silent "minecraft-protocol@$VERSION"
fi

exec env -u NODE_TLS_REJECT_UNAUTHORIZED NODE_PATH="$CLIENT_DIR/node_modules" "${TASKSET_CMD[@]}" node "${NODE_HEAP_FLAGS[@]}" "$ROOT/scripts/mc_bot_swarm.cjs" "$@"
