#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNS="$ROOT/runs/eula-gate"
LOGS="$ROOT/logs"
mkdir -p "$RUNS" "$LOGS"

run_one() {
  local name="$1"
  local jar="$2"
  local dir="$RUNS/$name"
  local log="$LOGS/${name}-eula-gate.log"

  rm -rf "$dir"
  mkdir -p "$dir"
  (
    cd "$dir"
    timeout 90s java -Xms512M -Xmx1G -jar "$jar" --nogui > "$log" 2>&1 || true
  )

  if ! rg -q "EULA|eula" "$log"; then
    echo "missing EULA gate evidence for $name; see $log" >&2
    exit 1
  fi
  echo "$name: reached EULA gate ($log)"
}

run_one "vanilla-1.21.10" "$ROOT/artifacts/vanilla-1.21.10.jar"
run_one "stock-paper-1.21.10" "$ROOT/artifacts/paper-1.21.10-130.jar"
run_one "optimized-paper-1.21.10" "$ROOT/artifacts/optimized-paper-1.21.10-mojmap.jar"

