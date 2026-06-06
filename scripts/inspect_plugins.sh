#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLUGINS="$ROOT/plugins/matrix"
REPORT="$ROOT/reports/plugin-inspection.csv"

mkdir -p "$ROOT/reports"
printf 'filename,descriptor,name,version,main,status\n' > "$REPORT"

for jar in "$PLUGINS"/*.jar; do
  filename="$(basename "$jar")"
  descriptor=""
  if jar tf "$jar" | rg -q '^paper-plugin.yml$'; then
    descriptor="paper-plugin.yml"
  elif jar tf "$jar" | rg -q '^plugin.yml$'; then
    descriptor="plugin.yml"
  else
    printf '%s,,,,missing descriptor\n' "$filename" >> "$REPORT"
    continue
  fi

  tmp="$(mktemp -d)"
  (cd "$tmp" && jar xf "$jar" "$descriptor")
  name="$(awk -F': *' '$1 == "name" {print $2; exit}' "$tmp/$descriptor" | tr -d '\r')"
  version="$(awk -F': *' '$1 == "version" {print $2; exit}' "$tmp/$descriptor" | tr -d '\r')"
  main="$(awk -F': *' '$1 == "main" {print $2; exit}' "$tmp/$descriptor" | tr -d '\r')"
  rm -rf "$tmp"

  if [[ -z "$name" || -z "$main" ]]; then
    printf '%s,%s,%s,%s,%s,missing name/main\n' "$filename" "$descriptor" "$name" "$version" "$main" >> "$REPORT"
  else
    printf '%s,%s,%s,%s,%s,ok\n' "$filename" "$descriptor" "$name" "$version" "$main" >> "$REPORT"
  fi
done

cat "$REPORT"

