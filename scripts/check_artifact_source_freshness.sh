#!/usr/bin/env bash
set -euo pipefail

ROOT="${ARTIFACT_SOURCE_FRESHNESS_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

if [[ -n "${REPORT:-}" ]]; then
  mkdir -p "$(dirname "$REPORT")"
  exec > >(tee -a "$REPORT")
fi

python3 - "$ROOT" <<'PY'
from __future__ import annotations

import datetime as dt
import os
import pathlib
import sys


root = pathlib.Path(sys.argv[1]).resolve()


def split_paths(name: str, defaults: list[pathlib.Path]) -> list[pathlib.Path]:
    raw = os.environ.get(name)
    if raw is None or raw == "":
        return defaults
    return [pathlib.Path(part) for part in raw.split(os.pathsep) if part]


def resolve_path(path: pathlib.Path) -> pathlib.Path:
    if path.is_absolute():
        return path
    return root / path


def inside_root(path: pathlib.Path) -> bool:
    try:
        path.resolve().relative_to(root)
    except ValueError:
        return False
    return True


def format_epoch(value: float | None) -> str:
    if value is None:
        return "missing"
    return dt.datetime.fromtimestamp(value, dt.timezone.utc).isoformat()


def bool_text(value: bool) -> str:
    return "true" if value else "false"


def env_float(name: str, default: float) -> float:
    raw = os.environ.get(name)
    if raw is None or raw == "":
        return default
    try:
        return float(raw)
    except ValueError:
        print(f"artifact_source_freshness_failure={name} must be a float, got {raw!r}")
        sys.exit(64)


def env_int(name: str, default: int) -> int:
    raw = os.environ.get(name)
    if raw is None or raw == "":
        return default
    try:
        return int(raw)
    except ValueError:
        print(f"artifact_source_freshness_failure={name} must be an integer, got {raw!r}")
        sys.exit(64)


paper_artifact = resolve_path(
    pathlib.Path(
        os.environ.get(
            "PAPER_ARTIFACT",
            "artifacts/optimized-paper-1.21.10-mojmap.jar",
        )
    )
)
if os.environ.get("NATIVE_ARTIFACT"):
    native_artifact_specs = [
        (
            "native",
            resolve_path(pathlib.Path(os.environ["NATIVE_ARTIFACT"])),
        )
    ]
elif os.environ.get("NATIVE_ARTIFACTS"):
    native_artifact_specs = [
        (
            "native" if index == 0 else f"native_{index + 1}",
            resolve_path(pathlib.Path(path)),
        )
        for index, path in enumerate(
            part for part in os.environ["NATIVE_ARTIFACTS"].split(os.pathsep) if part
        )
    ]
else:
    native_artifact_specs = [
        (
            "native",
            resolve_path(pathlib.Path("artifacts/optimized-runtime/native/libpaper_native_jni.so")),
        ),
        (
            "native_chunk_encode",
            resolve_path(pathlib.Path("artifacts/optimized-runtime/native/libpaper_native_chunk_encode_jni.so")),
        ),
    ]
paper_sources = split_paths(
    "PAPER_SOURCE_PATHS",
    [
        root / "upstream/Paper/paper-server/patches",
        root / "upstream/Paper/paper-server/src/generated/java",
        root / "upstream/Paper/paper-server/src/log4jPlugins/java",
        root / "upstream/Paper/paper-server/src/main/java",
        root / "upstream/Paper/paper-server/src/main/resources",
        root / "upstream/Paper/paper-server/src/minecraft/java",
        root / "upstream/Paper/paper-server/src/minecraft/resources",
        root / "upstream/Paper/paper-server/build.gradle.kts",
    ],
)
native_sources = split_paths("NATIVE_SOURCE_PATHS", [root / "native"])

mtime_slop = env_float("ARTIFACT_SOURCE_FRESHNESS_MTIME_SLOP_SECONDS", 0.0)
failure_exit = env_int("ARTIFACT_SOURCE_FRESHNESS_FAILURE_EXIT", 75)
max_failures = max(1, env_int("ARTIFACT_SOURCE_FRESHNESS_MAX_FAILURES", 20))

skip_dirs = {".git", ".gradle", "__pycache__", "build", "target"}


def include_source(path: pathlib.Path, kind: str) -> bool:
    if any(part in skip_dirs for part in path.parts):
        return False
    if kind == "native":
        return path.suffix == ".rs" or path.name in {"build.rs", "Cargo.toml", "Cargo.lock"}
    return True


def iter_sources(paths: list[pathlib.Path], kind: str, failures: list[str]) -> list[pathlib.Path]:
    seen: set[pathlib.Path] = set()
    sources: list[pathlib.Path] = []
    for raw_path in paths:
        path = resolve_path(raw_path)
        if not inside_root(path):
            failures.append(f"{kind} source path escapes root path={path}")
            continue
        if not path.exists():
            failures.append(f"{kind} source path missing path={path}")
            continue
        if path.is_file():
            resolved = path.resolve()
            if include_source(resolved, kind) and resolved not in seen:
                seen.add(resolved)
                sources.append(path)
            continue
        if not path.is_dir():
            failures.append(f"{kind} source path is not file or directory path={path}")
            continue
        for dirpath, dirnames, filenames in os.walk(path):
            dirnames[:] = [name for name in dirnames if name not in skip_dirs]
            base = pathlib.Path(dirpath)
            if not inside_root(base):
                dirnames[:] = []
                failures.append(f"{kind} source directory escapes root path={base}")
                continue
            for filename in filenames:
                candidate = base / filename
                if not include_source(candidate, kind):
                    continue
                resolved = candidate.resolve()
                if resolved in seen:
                    continue
                if not inside_root(resolved):
                    failures.append(f"{kind} source file escapes root path={candidate}")
                    continue
                seen.add(resolved)
                sources.append(candidate)
    return sources


def artifact_mtime(path: pathlib.Path, kind: str, failures: list[str]) -> float | None:
    if not inside_root(path):
        failures.append(f"{kind} artifact path escapes root path={path}")
        return None
    if not path.is_file():
        failures.append(f"{kind} artifact missing path={path}")
        return None
    try:
        stat = path.stat()
    except OSError as exc:
        failures.append(f"{kind} artifact stat failed path={path} error={exc}")
        return None
    if stat.st_size <= 0:
        failures.append(f"{kind} artifact is empty path={path}")
    return stat.st_mtime


def check_kind(
    kind: str,
    artifact: pathlib.Path,
    source_paths: list[pathlib.Path],
    failures: list[str],
) -> dict[str, str]:
    local_failures: list[str] = []
    artifact_time = artifact_mtime(artifact, kind, local_failures)
    sources = iter_sources(source_paths, kind, local_failures)
    if not sources:
        local_failures.append(f"{kind} has no source files")

    newer: list[tuple[pathlib.Path, float]] = []
    latest_source_time: float | None = None
    latest_source_path: pathlib.Path | None = None
    if artifact_time is not None:
        for source in sources:
            try:
                source_time = source.stat().st_mtime
            except OSError as exc:
                local_failures.append(f"{kind} source stat failed path={source} error={exc}")
                continue
            if latest_source_time is None or source_time > latest_source_time:
                latest_source_time = source_time
                latest_source_path = source
            if source_time > artifact_time + mtime_slop:
                newer.append((source, source_time))

    if newer and artifact_time is not None:
        for source, source_time in newer[:max_failures]:
            local_failures.append(
                f"{kind} source newer than artifact source={source} "
                f"source_mtime={format_epoch(source_time)} artifact={artifact} "
                f"artifact_mtime={format_epoch(artifact_time)}"
            )
        if len(newer) > max_failures:
            local_failures.append(
                f"{kind} source newer than artifact additional_count={len(newer) - max_failures}"
            )

    failures.extend(local_failures)
    return {
        f"{kind}_artifact": str(artifact),
        f"{kind}_artifact_mtime": format_epoch(artifact_time),
        f"{kind}_sources_checked": str(len(sources)),
        f"{kind}_latest_source": str(latest_source_path) if latest_source_path else "",
        f"{kind}_latest_source_mtime": format_epoch(latest_source_time),
        f"{kind}_source_newer_count": str(len(newer)),
        f"{kind}_pass": bool_text(not local_failures),
    }


failures: list[str] = []
paper_result = check_kind("paper", paper_artifact, paper_sources, failures)
native_results = [
    check_kind(kind, artifact, native_sources, failures)
    for kind, artifact in native_artifact_specs
]

print(f"artifact_source_freshness_root={root}")
for result in (paper_result, *native_results):
    for key, value in result.items():
        print(f"artifact_source_freshness_{key}={value}")
print(f"artifact_source_freshness_failure_count={len(failures)}")
for failure in failures:
    print(f"artifact_source_freshness_failure={failure}")
passed = not failures
print(f"artifact_source_freshness_pass={bool_text(passed)}")
print(f"artifact_source_freshness={'PASS' if passed else 'FAIL'}")
sys.exit(0 if passed else failure_exit)
PY
