#!/usr/bin/env bash

resolve_launcher_path() {
  local candidate="$1"

  if [[ "$candidate" == /* ]]; then
    printf '%s\n' "$candidate"
    return 0
  fi

  if [[ -e "$candidate" ]]; then
    (
      cd "$(dirname "$candidate")"
      printf '%s/%s\n' "$(pwd -P)" "$(basename "$candidate")"
    )
    return 0
  fi

  if [[ -n "${ROOT:-}" && -e "$ROOT/$candidate" ]]; then
    (
      cd "$(dirname "$ROOT/$candidate")"
      printf '%s/%s\n' "$(pwd -P)" "$(basename "$candidate")"
    )
    return 0
  fi

  printf '%s\n' "$candidate"
}
