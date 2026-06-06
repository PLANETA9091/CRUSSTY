#!/usr/bin/env bash

seed_luckperms_vault_unsafe_lookups() {
  local run_dir="$1"
  local root_dir="${2:-$ROOT}"
  local luckperms_jar=""
  local candidate=""

  for candidate in "$run_dir"/plugins/LuckPerms*.jar "$root_dir"/plugins/matrix/LuckPerms*.jar; do
    if [[ -f "$candidate" ]]; then
      luckperms_jar="$candidate"
      break
    fi
  done

  if [[ -z "$luckperms_jar" ]]; then
    return 0
  fi

  mkdir -p "$run_dir/plugins/LuckPerms"

  local config="$run_dir/plugins/LuckPerms/config.yml"
  local tmp="$config.tmp"
  local nested_tmp="$config.nested.tmp"
  if ! unzip -p "$luckperms_jar" config.yml > "$tmp" 2>/dev/null; then
    if ! unzip -p "$luckperms_jar" luckperms-bukkit.jarinjar > "$nested_tmp" 2>/dev/null \
      || ! unzip -p "$nested_tmp" config.yml > "$tmp" 2>/dev/null; then
      cat > "$tmp" <<'YAML'
vault-unsafe-lookups: true
YAML
    fi
  fi
  rm -f "$nested_tmp"

  python3 - "$tmp" "$config" <<'PY'
from pathlib import Path
import re
import sys

template = Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
path = Path(sys.argv[2])

updated, count = re.subn(
    r"(?m)^(\s*vault-unsafe-lookups:\s*)false(\s*)$",
    r"\1true\2",
    template,
    count=1,
)
if count == 0 and not re.search(r"(?m)^\s*vault-unsafe-lookups:\s*true\s*$", template):
    if not template.endswith("\n"):
        template += "\n"
    updated = template + "vault-unsafe-lookups: true\n"

path.write_text(updated, encoding="utf-8")
PY

  rm -f "$tmp" "$nested_tmp"
}
