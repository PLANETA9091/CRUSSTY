#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/scripts/load_test_defaults.sh"

check_default_shards() {
  local scenario="$1"
  local count="$2"
  local expected="$3"
  local actual

  actual="$(load_test_default_bot_swarm_shards "$scenario" "$count")"
  if [[ "$actual" != "$expected" ]]; then
    echo "Expected default shards for scenario=$scenario count=$count to be $expected, got $actual." >&2
    return 1
  fi
}

check_shared_gate() {
  local shards="$1"
  local mode="$2"
  local expected="$3"
  local actual=false

  if load_test_should_use_shared_action_gate "$shards" "$mode"; then
    actual=true
  fi
  if [[ "$actual" != "$expected" ]]; then
    echo "Expected shared gate for shards=$shards mode=$mode to be $expected, got $actual." >&2
    return 1
  fi
}

check_default_shards mixed-gameplay 50 1
check_default_shards mixed-gameplay 99 1
check_default_shards mixed-gameplay 100 2
check_default_shards mixed-gameplay 149 3
check_default_shards mixed-gameplay 200 4
check_default_shards mixed-gameplay 250 5
check_default_shards mixed-gameplay 500 10
check_default_shards block 499 1
check_default_shards block 500 10
check_default_shards movement 500 10
check_default_shards movement 100 1

check_shared_gate 1 all-ready false
check_shared_gate 2 all-ready true
check_shared_gate 10 ready-count true
check_shared_gate 10 timer false

ready_regex="$(load_test_server_ready_regex)"
if ! printf '%s\n' '[14:31:17 INFO]: Done (325.003s)! For help, type "help"' | rg -q "$ready_regex"; then
  echo "Expected server-ready regex to match the Paper done line." >&2
  exit 1
fi
if ! printf '%b\n' '\033[m> \r  \r[14:31:17 INFO]: Done (325.003s)! For help, type "help"' | rg -q "$ready_regex"; then
  echo "Expected server-ready regex to match prompt-prefixed Paper done lines." >&2
  exit 1
fi
if ! printf '%b\n' '[14:31:17 INFO]: Done (325.003s)! For help, type "help"\033[0m\r' | rg -q "$ready_regex"; then
  echo "Expected server-ready regex to match ANSI/control-suffixed Paper done lines." >&2
  exit 1
fi
if printf '%s\n' '[14:29:23 INFO]: [Geyser-Spigot] Done (7.137s)! Run /geyser help for help!' | rg -q "$ready_regex"; then
  echo "Server-ready regex must not match plugin-specific done lines." >&2
  exit 1
fi
if printf '%s\n' '[14:29:23 INFO]: [Plugin] nested [14:31:17 INFO]: Done (325.003s)! For help, type "help"' | rg -q "$ready_regex"; then
  echo "Server-ready regex must not match plugin-embedded Paper-looking done lines." >&2
  exit 1
fi

echo "run_load_test_sharding_defaults_smoke=PASS"
