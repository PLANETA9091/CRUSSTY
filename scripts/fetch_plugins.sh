#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLUGINS="$ROOT/plugins/matrix"
REPORTS="$ROOT/reports"
mkdir -p "$PLUGINS" "$REPORTS"

download() {
  local id="$1"
  local version="$2"
  local filename="$3"
  local url="$4"
  local out="$PLUGINS/$filename"

  if [[ ! -f "$out" ]]; then
    curl -fL -H "User-Agent: rust-paper-compat-agent/0.1" "$url" -o "$out"
  fi
  local sha256
  sha256="$(sha256sum "$out" | awk '{print $1}')"
  printf '%s,%s,%s,%s,%s\n' "$id" "$version" "$filename" "$sha256" "$url" >> "$REPORTS/plugin-artifacts.csv.tmp"
}

rm -f "$REPORTS/plugin-artifacts.csv.tmp"
printf 'plugin,version,filename,sha256,url\n' > "$REPORTS/plugin-artifacts.csv.tmp"

download "LuckPerms" "5.5.17" "LuckPerms-Bukkit-5.5.17.jar" \
  "https://cdn.modrinth.com/data/Vebnzrzj/versions/OrIs0S6b/LuckPerms-Bukkit-5.5.17.jar"
download "Vault" "1.7.3" "Vault.jar" \
  "https://github.com/MilkBowl/Vault/releases/download/1.7.3/Vault.jar"
download "PlaceholderAPI" "2.12.2" "PlaceholderAPI-2.12.2.jar" \
  "https://github.com/PlaceholderAPI/PlaceholderAPI/releases/download/2.12.2/PlaceholderAPI-2.12.2.jar"
download "ProtocolLib" "5.4.0" "ProtocolLib.jar" \
  "https://github.com/dmulloy2/ProtocolLib/releases/download/5.4.0/ProtocolLib.jar"
download "EssentialsX" "2.21.2" "EssentialsX-2.21.2.jar" \
  "https://github.com/EssentialsX/Essentials/releases/download/2.21.2/EssentialsX-2.21.2.jar"
download "WorldEdit" "7.4.2" "worldedit-bukkit-7.4.2.jar" \
  "https://cdn.modrinth.com/data/1u6JkXh5/versions/p8T2aZ8U/worldedit-bukkit-7.4.2.jar"
download "ViaVersion" "5.9.0" "ViaVersion-5.9.0.jar" \
  "https://github.com/ViaVersion/ViaVersion/releases/download/5.9.0/ViaVersion-5.9.0.jar"
download "spark" "1.10.172" "spark-1.10.172-bukkit.jar" \
  "https://ci.lucko.me/job/spark/lastSuccessfulBuild/artifact/spark-bukkit/build/libs/spark-1.10.172-bukkit.jar"
download "CoreProtect" "23.1" "CoreProtect-CE-23.1.jar" \
  "https://cdn.modrinth.com/data/Lu3KuzdV/versions/HD2IvrxS/CoreProtect-CE-23.1.jar"
download "Chunky" "1.4.40" "Chunky-Bukkit-1.4.40.jar" \
  "https://cdn.modrinth.com/data/fALzjamp/versions/P3y2MXnd/Chunky-Bukkit-1.4.40.jar"

mv "$REPORTS/plugin-artifacts.csv.tmp" "$REPORTS/plugin-artifacts.csv"
cat "$REPORTS/plugin-artifacts.csv"
