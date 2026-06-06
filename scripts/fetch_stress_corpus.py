#!/usr/bin/env python3
"""Download a curated plugin/datapack stress corpus from Modrinth.

This is evidence preparation only. A downloaded corpus is not a compatibility
or performance claim until a boot/load gate consumes it and publishes results.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import pathlib
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORTS = ROOT / "reports"
PLUGIN_DIR = ROOT / "plugins" / "stress"
DATAPACK_DIR = ROOT / "datapacks" / "stress"
CSV_REPORT = REPORTS / "stress-corpus-artifacts.csv"
JSON_REPORT = REPORTS / "stress-corpus-manifest.json"
USER_AGENT = "rust-paper-stress-corpus/0.1"

GAME_VERSION_PRIORITY = (
    "1.21.10",
    "1.21.9",
    "1.21.8",
    "1.21.7",
    "1.21.6",
    "1.21.5",
    "1.21.4",
    "1.21.3",
    "1.21.2",
    "1.21.1",
    "1.21",
)
PLUGIN_LOADER_PRIORITY = ("paper", "folia", "purpur", "bukkit", "spigot")
DATAPACK_LOADER_PRIORITY = ("datapack",)


@dataclass(frozen=True)
class Target:
    side: str
    slug: str
    intent: str
    loaders: tuple[str, ...]
    risk: str


PLUGIN_TARGETS = (
    Target("plugin", "worldguard", "region/protection event checks", PLUGIN_LOADER_PRIORITY, "world-edit"),
    Target("plugin", "viabackwards", "packet/protocol translation pressure", PLUGIN_LOADER_PRIORITY, "network"),
    Target("plugin", "minimotd", "ping/listener formatting surface", PLUGIN_LOADER_PRIORITY, "network"),
    Target("plugin", "geyser", "proxy/translation plugin startup surface", PLUGIN_LOADER_PRIORITY, "network"),
    Target("plugin", "skinsrestorer", "profile/cache/login hooks", PLUGIN_LOADER_PRIORITY, "login"),
    Target("plugin", "packetevents", "packet event library load path", PLUGIN_LOADER_PRIORITY, "network"),
    Target("plugin", "freedomchat", "chat event pipeline", PLUGIN_LOADER_PRIORITY, "chat"),
    Target("plugin", "simple-voice-chat", "network/session plugin startup surface", PLUGIN_LOADER_PRIORITY, "network"),
    Target("plugin", "interactivechat", "chat/item/placeholder event surface", PLUGIN_LOADER_PRIORITY, "chat"),
    Target("plugin", "advanced-portals", "teleport and region trigger surface", PLUGIN_LOADER_PRIORITY, "teleport"),
    Target("plugin", "tab-was-taken", "scoreboard/tab/team update pressure", PLUGIN_LOADER_PRIORITY, "scoreboard"),
    Target("plugin", "decentholograms", "entity/display update pressure", PLUGIN_LOADER_PRIORITY, "entities"),
    Target("plugin", "bluemap", "map render/web plugin startup surface", PLUGIN_LOADER_PRIORITY, "map-render"),
    Target("plugin", "squaremap", "map render/web plugin startup surface", PLUGIN_LOADER_PRIORITY, "map-render"),
    Target("plugin", "lesslag", "adaptive lag-control plugin surface", PLUGIN_LOADER_PRIORITY, "optimization"),
    Target("plugin", "circuitbreaker", "lag-control and response throttling surface", PLUGIN_LOADER_PRIORITY, "optimization"),
    Target("plugin", "lagscope", "runtime lag monitoring and control surface", PLUGIN_LOADER_PRIORITY, "optimization"),
    Target("plugin", "chunkpreloader999", "chunk preload and maintenance surface", PLUGIN_LOADER_PRIORITY, "worldgen"),
    Target("plugin", "clearlaggenhanced", "manual entity-clear and lag-control surface", PLUGIN_LOADER_PRIORITY, "optimization"),
    Target("plugin", "serverpulse", "metrics and monitoring surface", PLUGIN_LOADER_PRIORITY, "monitoring"),
    Target("plugin", "skygridx", "custom terrain and generation surface", PLUGIN_LOADER_PRIORITY, "worldgen"),
    Target("plugin", "g1axworldgenerator", "world-generation plugin surface", PLUGIN_LOADER_PRIORITY, "worldgen"),
)


DATAPACK_TARGETS = (
    Target("datapack", "terralith", "overworld biome/noise/worldgen stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
    Target("datapack", "incendium", "nether biome/noise/structure stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
    Target("datapack", "nullscape", "end biome/noise/worldgen stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
    Target("datapack", "structory", "structure pool/template stress", DATAPACK_LOADER_PRIORITY, "structures"),
    Target("datapack", "tectonic", "terrain/noise settings stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
    Target("datapack", "dungeons-and-taverns", "large structure set stress", DATAPACK_LOADER_PRIORITY, "structures"),
    Target("datapack", "geophilic", "biome decoration stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
    Target("datapack", "continents", "terrain layout/noise stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
    Target("datapack", "explorify", "structure/feature placement stress", DATAPACK_LOADER_PRIORITY, "structures"),
    Target("datapack", "amplified-nether", "nether terrain/noise stress", DATAPACK_LOADER_PRIORITY, "worldgen"),
)


def api_json(path: str) -> Any:
    url = f"https://api.modrinth.com/v2/{path}"
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read().decode("utf-8"))


def choose_file(files: list[dict[str, Any]]) -> dict[str, Any]:
    primary = [file for file in files if file.get("primary")]
    candidates = primary or files
    if not candidates:
        raise ValueError("version has no downloadable files")
    return candidates[0]


def select_version(target: Target) -> dict[str, Any]:
    versions = api_json(f"project/{urllib.parse.quote(target.slug, safe='')}/version")
    if not isinstance(versions, list):
        raise ValueError(f"unexpected version response for {target.slug}")

    for game_version in GAME_VERSION_PRIORITY:
        for version in versions:
            if game_version not in version.get("game_versions", []):
                continue
            loaders = tuple(version.get("loaders", []))
            if not any(loader in loaders for loader in target.loaders):
                continue
            file = choose_file(version.get("files", []))
            matched_loader = next((loader for loader in target.loaders if loader in loaders), loaders[0])
            return {
                "project_id": version.get("project_id", ""),
                "version_id": version.get("id", ""),
                "version_number": version.get("version_number", ""),
                "date_published": version.get("date_published", ""),
                "game_version": game_version,
                "all_game_versions": version.get("game_versions", []),
                "loader": matched_loader,
                "all_loaders": list(loaders),
                "file": file,
            }

    raise ValueError(f"no compatible version found for {target.slug}")


def project_title(slug: str) -> str:
    project = api_json(f"project/{urllib.parse.quote(slug, safe='')}")
    return str(project.get("title") or slug)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download(url: str, out: pathlib.Path, force: bool) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    if out.exists() and not force:
        return

    tmp = out.with_suffix(out.suffix + ".tmp")
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=120) as response, tmp.open("wb") as handle:
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            handle.write(chunk)
    tmp.replace(out)


def fetch_target(target: Target, force: bool) -> dict[str, Any]:
    selected = select_version(target)
    title = project_title(target.slug)
    file = selected["file"]
    filename = pathlib.Path(file["filename"]).name
    out_dir = PLUGIN_DIR if target.side == "plugin" else DATAPACK_DIR
    out_path = out_dir / filename
    download(str(file["url"]), out_path, force)
    sha256 = sha256_file(out_path)
    return {
        "side": target.side,
        "slug": target.slug,
        "title": title,
        "intent": target.intent,
        "risk": target.risk,
        "version": selected["version_number"],
        "version_id": selected["version_id"],
        "published": selected["date_published"],
        "game_version": selected["game_version"],
        "all_game_versions": selected["all_game_versions"],
        "loader": selected["loader"],
        "all_loaders": selected["all_loaders"],
        "filename": filename,
        "path": str(out_path.relative_to(ROOT)),
        "sha256": sha256,
        "url": file["url"],
        "size": out_path.stat().st_size,
        "status": "downloaded",
    }


def write_reports(rows: list[dict[str, Any]]) -> None:
    REPORTS.mkdir(parents=True, exist_ok=True)
    csv_tmp = CSV_REPORT.with_suffix(CSV_REPORT.suffix + ".tmp")
    json_tmp = JSON_REPORT.with_suffix(JSON_REPORT.suffix + ".tmp")
    fieldnames = (
        "side",
        "slug",
        "title",
        "version",
        "game_version",
        "loader",
        "filename",
        "path",
        "sha256",
        "size",
        "risk",
        "intent",
        "url",
        "status",
    )
    with csv_tmp.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow({field: row.get(field, "") for field in fieldnames})
    manifest = {
        "generated_at_unix": int(time.time()),
        "game_version_priority": list(GAME_VERSION_PRIORITY),
        "plugin_count": sum(1 for row in rows if row["side"] == "plugin"),
        "datapack_count": sum(1 for row in rows if row["side"] == "datapack"),
        "rows": rows,
        "claim_scope": "downloaded stress corpus only; not a compatibility or performance claim",
    }
    json_tmp.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    csv_tmp.replace(CSV_REPORT)
    json_tmp.replace(JSON_REPORT)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--force", action="store_true", help="redownload existing artifacts")
    parser.add_argument(
        "--side",
        choices=("all", "plugins", "datapacks"),
        default="all",
        help="which corpus side to fetch",
    )
    args = parser.parse_args()

    targets: list[Target] = []
    if args.side in ("all", "plugins"):
        targets.extend(PLUGIN_TARGETS)
    if args.side in ("all", "datapacks"):
        targets.extend(DATAPACK_TARGETS)

    rows: list[dict[str, Any]] = []
    errors: list[str] = []
    for target in targets:
        try:
            row = fetch_target(target, args.force)
            rows.append(row)
            print(
                f"{row['side']} {row['slug']} {row['version']} "
                f"{row['game_version']} {row['filename']} {row['sha256']}",
                flush=True,
            )
        except (OSError, urllib.error.URLError, ValueError, KeyError) as exc:
            errors.append(f"{target.side}:{target.slug}: {exc}")

    if rows:
        write_reports(rows)
        print(f"wrote {CSV_REPORT}")
        print(f"wrote {JSON_REPORT}")

    if errors:
        print("fetch_stress_corpus errors:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
