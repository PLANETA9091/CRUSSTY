#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS="$ROOT/artifacts"
REPORTS="$ROOT/reports"
mkdir -p "$ARTIFACTS" "$REPORTS"

UA="rust-paper-compat-agent/0.1"
MC_VERSION="${MC_VERSION:-1.21.10}"

paper_json="$ARTIFACTS/paper-${MC_VERSION}-latest.json"
curl -fsSL -H "User-Agent: $UA" \
  "https://fill.papermc.io/v3/projects/paper/versions/${MC_VERSION}/builds" \
  | jq 'max_by(.id)' > "$paper_json"

paper_url="$(jq -r '.downloads["server:default"].url' "$paper_json")"
paper_name="$(jq -r '.downloads["server:default"].name' "$paper_json")"
paper_sha256="$(jq -r '.downloads["server:default"].checksums.sha256' "$paper_json")"
paper_path="$ARTIFACTS/$paper_name"

if [[ ! -f "$paper_path" ]]; then
  curl -fL -H "User-Agent: $UA" "$paper_url" -o "$paper_path"
fi
echo "${paper_sha256}  ${paper_path}" | sha256sum -c -

manifest="$ARTIFACTS/mojang-version-manifest.json"
curl -fsSL -H "User-Agent: $UA" \
  "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json" \
  -o "$manifest"

version_url="$(jq -r --arg v "$MC_VERSION" '.versions[] | select(.id == $v) | .url' "$manifest")"
version_json="$ARTIFACTS/vanilla-${MC_VERSION}.json"
curl -fsSL -H "User-Agent: $UA" "$version_url" -o "$version_json"

vanilla_url="$(jq -r '.downloads.server.url' "$version_json")"
vanilla_sha1="$(jq -r '.downloads.server.sha1' "$version_json")"
vanilla_path="$ARTIFACTS/vanilla-${MC_VERSION}.jar"

if [[ ! -f "$vanilla_path" ]]; then
  curl -fL -H "User-Agent: $UA" "$vanilla_url" -o "$vanilla_path"
fi
echo "${vanilla_sha1}  ${vanilla_path}" | sha1sum -c -

optimized_src="$ROOT/upstream/Paper/paper-server/build/libs/paper-bundler-${MC_VERSION}-R0.1-SNAPSHOT-mojmap.jar"
optimized_path="$ARTIFACTS/optimized-paper-${MC_VERSION}-mojmap.jar"
if [[ -f "$optimized_src" ]]; then
  cp "$optimized_src" "$optimized_path"
  sha256sum "$optimized_path" > "$optimized_path.sha256"
fi

jq -n \
  --arg mc_version "$MC_VERSION" \
  --arg paper_path "$paper_path" \
  --arg paper_sha256 "$paper_sha256" \
  --arg vanilla_path "$vanilla_path" \
  --arg vanilla_sha1 "$vanilla_sha1" \
  --arg optimized_path "$optimized_path" \
  '{
    mc_version: $mc_version,
    stock_paper: { path: $paper_path, sha256: $paper_sha256 },
    vanilla: { path: $vanilla_path, sha1: $vanilla_sha1 },
    optimized: { path: $optimized_path }
  }' > "$REPORTS/artifacts.json"

cat "$REPORTS/artifacts.json"

