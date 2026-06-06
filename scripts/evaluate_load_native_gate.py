#!/usr/bin/env python3
"""Evaluate run_load_test native module gates from a generated summary."""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import sys


SUMMARY_TOKEN_RE = re.compile(r"(?<!\S)([A-Za-z0-9_]+)=([^ \t\r\n]+)")
RUNTIME_FLAG_RE = re.compile(r"\b(native_[a-z0-9_]+)=(true|false)\b", re.IGNORECASE)

MODULE_ALIASES: dict[str, tuple[str, ...]] = {
    "all": (
        "climate_rtree",
        "area_map",
        "improved_noise",
        "normal_noise",
        "perlin_noise_no_y_scale",
        "chunk_packet_encode",
    ),
    "climate": ("climate_rtree",),
    "climate_rtree": ("climate_rtree",),
    "area": ("area_map",),
    "area_map": ("area_map",),
    "improved_noise": ("improved_noise",),
    "normal": ("normal_noise",),
    "normal_noise": ("normal_noise",),
    "perlin": ("perlin_noise",),
    "perlin_noise": ("perlin_noise",),
    "perlin_generic": ("perlin_noise_generic",),
    "perlin_noise_generic": ("perlin_noise_generic",),
    "perlin_no_y_scale": ("perlin_noise_no_y_scale",),
    "perlin_noise_no_y_scale": ("perlin_noise_no_y_scale",),
    "perlin_noyscale": ("perlin_noise_no_y_scale",),
    "chunk_packet_encode": ("chunk_packet_encode",),
    "chunk_encode": ("chunk_packet_encode",),
    "native_chunk_packet_encode": ("chunk_packet_encode",),
}

PRODUCTION_NATIVE_CHECKS: dict[str, tuple[str, ...]] = {
    "native_climate_rtree": ("climate_rtree",),
    "native_area_map": ("area_map",),
    "native_improved_noise": ("improved_noise",),
    "native_normal_noise": ("normal_noise",),
    "native_perlin_noise": ("perlin_noise_generic", "perlin_noise_no_y_scale"),
    "native_perlin_noise_generic": ("perlin_noise_generic",),
    "native_perlin_noise_no_y_scale": ("perlin_noise_no_y_scale",),
    "native_chunk_packet_encode": ("chunk_packet_encode",),
}


def parse_summary(path: pathlib.Path) -> dict[str, str]:
    values: dict[str, str] = {}
    with path.open(encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            if line == "bot_log_tail:":
                break
            if line.startswith("native_runtime_line="):
                values["native_runtime_line"] = line.split("=", 1)[1]
                continue
            for key, value in SUMMARY_TOKEN_RE.findall(line):
                values[key] = value
    return values


def summary_bool(values: dict[str, str], key: str) -> bool:
    return values.get(key, "").lower() == "true"


def parse_required_modules(raw_value: str) -> list[str]:
    modules: list[str] = []
    for raw_token in re.split(r"[\s,]+", raw_value.strip()):
        if not raw_token:
            continue
        token = raw_token.lower().replace("-", "_")
        if token not in MODULE_ALIASES:
            raise ValueError(f"Unknown LOAD_TEST_REQUIRE_NATIVE_MODULES token: {raw_token}")
        for module in MODULE_ALIASES[token]:
            if module not in modules:
                modules.append(module)
    return modules


def evaluate(values: dict[str, str], profile: str, required_modules: list[str]) -> list[str]:
    loaded_modules = {
        "climate_rtree": summary_bool(values, "native_climate_rtree_loaded"),
        "area_map": summary_bool(values, "native_area_map_loaded"),
        "improved_noise": summary_bool(values, "native_improved_noise_loaded"),
        "normal_noise": summary_bool(values, "native_normal_noise_loaded"),
        "perlin_noise": summary_bool(values, "native_perlin_noise_loaded"),
        "perlin_noise_generic": summary_bool(values, "native_perlin_noise_generic_loaded"),
        "perlin_noise_no_y_scale": summary_bool(values, "native_perlin_noise_no_y_scale_loaded"),
        "chunk_packet_encode": summary_bool(values, "native_chunk_packet_encode_loaded"),
    }

    failures: list[str] = []
    missing_modules = [module for module in required_modules if not loaded_modules[module]]
    if missing_modules:
        failures.append(
            "Native load gate failed; missing loaded modules: " + ",".join(missing_modules)
        )
        return failures

    native_runtime_line = values.get("native_runtime_line", "")
    if profile.startswith("production-") and native_runtime_line:
        runtime_flags = {
            key: value.lower() == "true"
            for key, value in RUNTIME_FLAG_RE.findall(native_runtime_line)
        }
        missing_advertised_modules = [
            key[len("native_") :]
            for key, modules in PRODUCTION_NATIVE_CHECKS.items()
            if runtime_flags.get(key) is True
            and not any(loaded_modules[module] for module in modules)
        ]
        if missing_advertised_modules:
            failures.append(
                "Production native load gate failed; advertised native modules were not loaded: "
                + ",".join(missing_advertised_modules)
            )

    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("summary", type=pathlib.Path, help="run_load_test summary file")
    parser.add_argument("--profile", default=os.environ.get("LOAD_TEST_GATE_PROFILE", ""))
    parser.add_argument(
        "--require-native-modules",
        default=os.environ.get("LOAD_TEST_REQUIRE_NATIVE_MODULES", ""),
        help="Comma/space separated native module names; defaults to LOAD_TEST_REQUIRE_NATIVE_MODULES.",
    )
    args = parser.parse_args()

    if not args.summary.is_file():
        print(f"Missing summary file: {args.summary}", file=sys.stderr)
        return 66

    try:
        required_modules = parse_required_modules(args.require_native_modules)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    failures = evaluate(parse_summary(args.summary), args.profile, required_modules)
    for failure in failures:
        print(failure, file=sys.stderr)
    if failures:
        return 76
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
