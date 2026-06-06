#!/usr/bin/env bash

load_test_default_bot_swarm_shards() {
  local scenario="$1"
  local bot_count="$2"

  if [[ ! "$bot_count" =~ ^[0-9]+$ ]] || (( bot_count < 1 )); then
    echo "1"
    return 0
  fi

  if [[ "$scenario" == "mixed-gameplay" && "$bot_count" -ge 100 ]]; then
    echo "$(( (bot_count + 49) / 50 ))"
    return 0
  fi

  if (( bot_count >= 500 )); then
    echo "$(( (bot_count + 49) / 50 ))"
    return 0
  fi

  echo "1"
}

load_test_should_use_shared_action_gate() {
  local shards="$1"
  local action_mode="$2"

  [[ "$shards" =~ ^[0-9]+$ ]] && (( shards > 1 )) && [[ "$action_mode" != "timer" ]]
}

load_test_server_ready_regex() {
  printf '%s\n' '^(?:(?:\x1B\[[0-9;?]*[ -/]*[@-~])|[>[:space:]\r])*\[[0-9]{2}:[0-9]{2}:[0-9]{2} INFO\]: Done \([0-9.]+s\)! For help, type "help"(?:(?:\x1B\[[0-9;?]*[ -/]*[@-~])|[>[:space:]\r])*$'
}
