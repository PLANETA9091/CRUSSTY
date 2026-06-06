#!/usr/bin/env python3
"""Inspect downloaded stress plugins/datapacks for basic server-loadability."""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
import zipfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
PLUGIN_DIR = ROOT / "plugins" / "stress"
DATAPACK_DIR = ROOT / "datapacks" / "stress"
REPORTS = ROOT / "reports"
JSON_REPORT = REPORTS / "stress-corpus-inspection.json"
TEXT_REPORT = REPORTS / "stress-corpus-inspection.txt"


def simple_yaml_value(text: str, key: str) -> str:
    match = re.search(rf"(?m)^\s*{re.escape(key)}\s*:\s*['\"]?([^'\"\r\n#]+)", text)
    return match.group(1).strip() if match else ""


def inspect_plugin(path: pathlib.Path) -> dict[str, Any]:
    row: dict[str, Any] = {
        "side": "plugin",
        "filename": path.name,
        "path": str(path.relative_to(ROOT)),
        "status": "ok",
        "descriptor": "",
        "name": "",
        "version": "",
        "main": "",
        "api_version": "",
        "depend": "",
        "softdepend": "",
        "file_count": 0,
        "errors": [],
    }
    try:
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            row["file_count"] = len(names)
            descriptor = ""
            for candidate in ("paper-plugin.yml", "plugin.yml"):
                if candidate in names:
                    descriptor = candidate
                    break
            if not descriptor:
                row["status"] = "fail"
                row["errors"].append("missing plugin.yml or paper-plugin.yml")
                return row
            text = archive.read(descriptor).decode("utf-8", errors="replace")
            row["descriptor"] = descriptor
            row["name"] = simple_yaml_value(text, "name")
            row["version"] = simple_yaml_value(text, "version")
            row["main"] = simple_yaml_value(text, "main")
            row["api_version"] = simple_yaml_value(text, "api-version")
            row["depend"] = simple_yaml_value(text, "depend")
            row["softdepend"] = simple_yaml_value(text, "softdepend")
            if not row["name"]:
                row["errors"].append("missing name")
            if not row["main"]:
                row["errors"].append("missing main")
    except zipfile.BadZipFile:
        row["status"] = "fail"
        row["errors"].append("bad zip/jar")
    except OSError as exc:
        row["status"] = "fail"
        row["errors"].append(str(exc))

    if row["errors"]:
        row["status"] = "fail"
    return row


def find_root_pack(names: list[str]) -> str:
    if "pack.mcmeta" in names:
        return ""
    candidates = [name for name in names if name.endswith("/pack.mcmeta")]
    if len(candidates) == 1:
        return candidates[0][: -len("pack.mcmeta")]
    return "__missing__"


def inspect_datapack(path: pathlib.Path) -> dict[str, Any]:
    row: dict[str, Any] = {
        "side": "datapack",
        "filename": path.name,
        "path": str(path.relative_to(ROOT)),
        "status": "ok",
        "pack_format": "",
        "description": "",
        "root_prefix": "",
        "namespace_count": 0,
        "worldgen_file_count": 0,
        "structure_file_count": 0,
        "tag_file_count": 0,
        "function_file_count": 0,
        "errors": [],
    }
    try:
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            root_prefix = find_root_pack(names)
            if root_prefix == "__missing__":
                row["status"] = "fail"
                row["errors"].append("missing pack.mcmeta")
                return row
            row["root_prefix"] = root_prefix
            meta = json.loads(archive.read(root_prefix + "pack.mcmeta").decode("utf-8"))
            pack = meta.get("pack", {})
            row["pack_format"] = pack.get("pack_format", "")
            row["description"] = pack.get("description", "")
            data_prefix = root_prefix + "data/"
            namespaces = {
                name[len(data_prefix) :].split("/", 1)[0]
                for name in names
                if name.startswith(data_prefix) and len(name[len(data_prefix) :].split("/", 1)) > 1
            }
            row["namespace_count"] = len(namespaces)
            row["worldgen_file_count"] = sum(1 for name in names if "/worldgen/" in name and not name.endswith("/"))
            row["structure_file_count"] = sum(
                1 for name in names if ("/structure/" in name or "/structures/" in name) and not name.endswith("/")
            )
            row["tag_file_count"] = sum(1 for name in names if "/tags/" in name and not name.endswith("/"))
            row["function_file_count"] = sum(1 for name in names if "/function" in name and name.endswith(".mcfunction"))
            if not namespaces:
                row["errors"].append("no data namespaces")
            if row["worldgen_file_count"] == 0 and row["structure_file_count"] == 0:
                row["errors"].append("no worldgen or structure files")
    except zipfile.BadZipFile:
        row["status"] = "fail"
        row["errors"].append("bad zip/datapack")
    except (OSError, json.JSONDecodeError) as exc:
        row["status"] = "fail"
        row["errors"].append(str(exc))

    if row["errors"]:
        row["status"] = "fail"
    return row


def write_reports(rows: list[dict[str, Any]], failures: list[dict[str, Any]]) -> None:
    REPORTS.mkdir(parents=True, exist_ok=True)
    plugin_count = sum(1 for row in rows if row["side"] == "plugin")
    datapack_count = sum(1 for row in rows if row["side"] == "datapack")
    result = {
        "stress_corpus_inspection_pass": not failures,
        "plugin_count": plugin_count,
        "datapack_count": datapack_count,
        "failure_count": len(failures),
        "rows": rows,
        "failures": failures,
        "claim_scope": "descriptor/datapack shape inspection only; not a runtime compatibility claim",
    }
    JSON_REPORT.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        f"stress_corpus_inspection_pass={str(not failures).lower()}",
        f"plugin_count={plugin_count}",
        f"datapack_count={datapack_count}",
        f"failure_count={len(failures)}",
    ]
    for row in rows:
        if row["side"] == "plugin":
            lines.append(
                "plugin "
                f"status={row['status']} filename={row['filename']} "
                f"descriptor={row.get('descriptor', '')} name={row.get('name', '')} "
                f"version={row.get('version', '')} main={row.get('main', '')}"
            )
        else:
            lines.append(
                "datapack "
                f"status={row['status']} filename={row['filename']} "
                f"pack_format={row.get('pack_format', '')} namespaces={row.get('namespace_count', 0)} "
                f"worldgen={row.get('worldgen_file_count', 0)} structures={row.get('structure_file_count', 0)}"
            )
        if row.get("errors"):
            lines.append(f"  errors={'; '.join(row['errors'])}")
    TEXT_REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--min-plugins", type=int, default=12)
    parser.add_argument("--min-datapacks", type=int, default=8)
    args = parser.parse_args()

    plugin_paths = sorted(PLUGIN_DIR.glob("*.jar"))
    datapack_paths = sorted(DATAPACK_DIR.glob("*.zip"))
    rows = [inspect_plugin(path) for path in plugin_paths]
    rows.extend(inspect_datapack(path) for path in datapack_paths)

    failures = [row for row in rows if row["status"] != "ok"]
    plugin_count = sum(1 for row in rows if row["side"] == "plugin")
    datapack_count = sum(1 for row in rows if row["side"] == "datapack")
    if plugin_count < args.min_plugins:
        failures.append({"side": "plugin", "filename": "__count__", "status": "fail", "errors": [f"plugin_count<{args.min_plugins}"]})
    if datapack_count < args.min_datapacks:
        failures.append({"side": "datapack", "filename": "__count__", "status": "fail", "errors": [f"datapack_count<{args.min_datapacks}"]})

    write_reports(rows, failures)
    print(TEXT_REPORT.read_text(encoding="utf-8"), end="")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
