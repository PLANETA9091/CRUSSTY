#!/usr/bin/env python3
"""Audit native Rust/JNI diagnostic coverage.

The audit intentionally checks the modular diagnostic surface, not Paper
runtime hooks. A module is considered structurally covered when the Rust core
module exists, the JNI crate references it, at least one native Java bench
directory exists, and at least one executable bench script exists.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CORE_LIB = ROOT / "native" / "paper-native-core" / "src" / "lib.rs"
JNI_LIB = ROOT / "native" / "paper-native-jni" / "src" / "lib.rs"
PARITY_DOC = ROOT / "docs" / "PARITY_MATRIX.md"
NATIVE_README = ROOT / "native" / "README.md"
PACK_RUNNER = ROOT / "scripts" / "bench_native_pack.sh"
PACK_META_RUNNERS = {"bench_native_pack.sh", "bench_native_worldgen_pack.sh"}


@dataclass(frozen=True)
class CoverageTarget:
    bench_dirs: tuple[str, ...]
    scripts: tuple[str, ...]
    doc_terms: tuple[str, ...]
    runtime_only: bool = False


@dataclass(frozen=True)
class ExternalCoverageTarget:
    core_source: Path
    jni_source: Path
    bench_dirs: tuple[str, ...]
    scripts: tuple[str, ...]
    doc_terms: tuple[str, ...]
    load_library: str


@dataclass(frozen=True)
class NativeMethod:
    source: Path
    package: str
    class_name: str
    method_name: str


def kebab(module: str) -> str:
    return module.replace("_", "-")


def default_target(module: str) -> CoverageTarget:
    return CoverageTarget(
        bench_dirs=(f"bench/native-{kebab(module)}",),
        scripts=(f"scripts/bench_native_{module}.sh",),
        doc_terms=(f"`{module}`", module),
    )


OVERRIDES: dict[str, CoverageTarget] = {
    "chunk_dependencies": CoverageTarget(
        bench_dirs=("bench/native-chunk-dependencies",),
        scripts=("scripts/bench_native_chunk_dependencies_array.sh",),
        doc_terms=("`chunk_dependencies`", "chunk dependencies"),
    ),
    "climate_rtree": CoverageTarget(
        bench_dirs=(
            "bench/climate-rtree-build",
            "bench/climate-rtree-search",
            "bench/climate-rtree-lifecycle",
        ),
        scripts=(
            "scripts/bench_native_climate_rtree_build.sh",
            "scripts/bench_native_climate_rtree_search.sh",
            "scripts/bench_native_climate_rtree_lifecycle.sh",
            "scripts/bench_native_climate_rtree_arena.sh",
            "scripts/bench_native_climate_rtree_batch_borrow.sh",
            "scripts/bench_native_climate_rtree_jni.sh",
        ),
        doc_terms=("`climate_rtree`", "Climate RTree"),
    ),
    "compression": CoverageTarget(
        bench_dirs=("bench/region-compression",),
        scripts=("scripts/bench_region_compression.sh",),
        doc_terms=("`compression`", "region compression", "LZ4"),
    ),
    "density_ap2_minmax_fill": CoverageTarget(
        bench_dirs=("bench/native-density-ap2-minmax",),
        scripts=("scripts/bench_native_density_ap2_minmax_fill.sh",),
        doc_terms=("`density_ap2_minmax_fill`", "density_ap2_minmax"),
    ),
    "density_visitor_hook": CoverageTarget(
        bench_dirs=("bench/native-density-visitor-hooks",),
        scripts=("scripts/bench_native_density_visitor_hooks.sh",),
        doc_terms=("`density_visitor_hook`", "density visitor"),
    ),
    "hash_path_summary": CoverageTarget(
        bench_dirs=("bench/native-hash-path",),
        scripts=("scripts/bench_native_hash_path.sh",),
        doc_terms=("`hash_path_summary`", "hash-path"),
    ),
    "nearby_player_map_capacity": CoverageTarget(
        bench_dirs=("bench/native-nearby-player-map",),
        scripts=("scripts/bench_native_nearby_player_map.sh",),
        doc_terms=("`nearby_player_map_capacity`", "nearby player"),
    ),
    "perlin_noise": CoverageTarget(
        bench_dirs=("bench/native-perlin-noise", "bench/native-perlin-getvalue"),
        scripts=("scripts/bench_native_perlin_noise.sh", "scripts/bench_native_perlin_getvalue.sh"),
        doc_terms=("`perlin_noise`", "PerlinGetValue", "perlin-noise"),
    ),
    "reference_list": CoverageTarget(
        bench_dirs=("bench/native-reference-list",),
        scripts=(
            "scripts/bench_native_reference_list.sh",
            "scripts/bench_native_reference_list_bridge.sh",
            "scripts/bench_native_reference_list_stateful.sh",
        ),
        doc_terms=("`reference_list`", "ReferenceList"),
    ),
    "waypoint_table_view": CoverageTarget(
        bench_dirs=("bench/native-waypoint-table",),
        scripts=("scripts/bench_native_waypoint_table_view.sh",),
        doc_terms=("`waypoint_table_view`", "waypoint table"),
    ),
}

EXTERNAL_TARGETS: dict[str, ExternalCoverageTarget] = {
    "chunk_packet_encode": ExternalCoverageTarget(
        core_source=ROOT / "native" / "paper-native-chunk-encode-core" / "src" / "lib.rs",
        jni_source=ROOT / "native" / "paper-native-chunk-encode-jni" / "src" / "lib.rs",
        bench_dirs=("bench/native-chunk-packet-encode",),
        scripts=("scripts/bench_native_chunk_packet_encode.sh",),
        doc_terms=("`paper-native-chunk-encode-core`", "`paper-native-chunk-encode-jni`", "chunk packet encode"),
        load_library="paper_native_chunk_encode_jni",
    ),
}


def parse_modules() -> list[str]:
    text = CORE_LIB.read_text(encoding="utf-8")
    return re.findall(r"^pub mod ([a-z0-9_]+);$", text, flags=re.MULTILINE)


def read_existing(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="replace")


def path_has_java_sources(path: Path) -> bool:
    return path.is_dir() and any(child.suffix == ".java" for child in path.iterdir())


def encode_jni_part(value: str) -> str:
    encoded: list[str] = []
    for char in value:
        if char == "_":
            encoded.append("_1")
        elif char == ";":
            encoded.append("_2")
        elif char == "[":
            encoded.append("_3")
        elif char == ".":
            encoded.append("_")
        else:
            encoded.append(char)
    return "".join(encoded)


def expected_jni_export(method: NativeMethod) -> str:
    owner = method.class_name
    if method.package:
        owner = f"{method.package}.{owner}"
    return f"Java_{encode_jni_part(owner)}_{encode_jni_part(method.method_name)}"


def native_methods_in_bench_dir(bench_dir: Path) -> list[NativeMethod]:
    if not bench_dir.is_dir():
        return []

    methods: list[NativeMethod] = []
    for source in sorted(bench_dir.glob("PaperNative*.java")):
        text = source.read_text(encoding="utf-8", errors="replace")
        package_match = re.search(r"^\s*package\s+([A-Za-z0-9_.]+)\s*;", text, flags=re.MULTILINE)
        package = package_match.group(1) if package_match else ""
        for method_name in re.findall(
            r"\bnative\s+[A-Za-z0-9_<>,.?[\]]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
            text,
        ):
            methods.append(
                NativeMethod(
                    source=source,
                    package=package,
                    class_name=source.stem,
                    method_name=method_name,
                )
            )
    return methods


def real_native_bench_script_names() -> list[str]:
    return sorted(
        path.name
        for path in (ROOT / "scripts").glob("bench_native_*.sh")
        if path.name not in PACK_META_RUNNERS
    )


def pack_all_script_names(errors: list[str]) -> list[str]:
    env = os.environ.copy()
    env.pop("PACK_SCRIPTS", None)
    env.update(
        {
            "PACK_LIST": "1",
            "PACK_GROUPS": "all",
            "PACK_REQUIRE_ALL_REAL": "1",
        }
    )
    try:
        completed = subprocess.run(
            [str(PACK_RUNNER)],
            cwd=ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as exc:
        errors.append(f"pack-contract: failed to execute {PACK_RUNNER.relative_to(ROOT)}: {exc}")
        return []

    if completed.returncode != 0:
        stderr = completed.stderr.strip().replace("\n", " | ")
        errors.append(
            "pack-contract: PACK_GROUPS=all list failed "
            f"with exit={completed.returncode}: {stderr}"
        )
        return []

    return sorted(
        Path(line.strip()).name
        for line in completed.stdout.splitlines()
        if line.strip()
    )


def pack_leaf_group_manifest(errors: list[str]) -> list[tuple[str, str]]:
    env = os.environ.copy()
    env.pop("PACK_SCRIPTS", None)
    env.update(
        {
            "PACK_MANIFEST": "1",
            "PACK_REQUIRE_ALL_REAL": "1",
            "PACK_REQUIRE_GROUP_OWNERSHIP": "1",
        }
    )
    try:
        completed = subprocess.run(
            [str(PACK_RUNNER)],
            cwd=ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as exc:
        errors.append(f"pack-contract: failed to execute manifest from {PACK_RUNNER.relative_to(ROOT)}: {exc}")
        return []

    if completed.returncode != 0:
        stderr = completed.stderr.strip().replace("\n", " | ")
        errors.append(
            "pack-contract: PACK_MANIFEST list failed "
            f"with exit={completed.returncode}: {stderr}"
        )
        return []

    manifest: list[tuple[str, str]] = []
    for raw_line in completed.stdout.splitlines():
        if not raw_line.strip():
            continue
        try:
            group, script = raw_line.split("\t", 1)
        except ValueError:
            errors.append(f"pack-contract: malformed PACK_MANIFEST line: {raw_line}")
            continue
        manifest.append((group, Path(script).name))
    return manifest


def audit(strict_docs: bool) -> int:
    modules = parse_modules()
    jni = read_existing(JNI_LIB)
    docs = read_existing(PARITY_DOC) + "\n" + read_existing(NATIVE_README)

    errors: list[str] = []
    warnings: list[str] = []
    covered_scripts: set[str] = set()
    covered_benches: set[str] = set()
    checked_wrappers: set[Path] = set()
    checked_exports: set[str] = set()
    checked_load_wrappers: set[Path] = set()

    for module in modules:
        target = OVERRIDES.get(module, default_target(module))
        source = ROOT / "native" / "paper-native-core" / "src" / f"{module}.rs"
        if not source.is_file():
            errors.append(f"{module}: missing core source {source.relative_to(ROOT)}")

        if not re.search(rf"\b{re.escape(module)}::", jni):
            errors.append(f"{module}: JNI source does not reference {module}::")

        for bench_dir in target.bench_dirs:
            bench_path = ROOT / bench_dir
            if not path_has_java_sources(bench_path):
                errors.append(f"{module}: missing native Java bench sources in {bench_dir}")
            else:
                covered_benches.add(bench_dir)
                for wrapper in sorted(bench_path.glob("PaperNative*.java")):
                    wrapper_key = wrapper.relative_to(ROOT)
                    if wrapper_key not in checked_load_wrappers:
                        wrapper_text = wrapper.read_text(encoding="utf-8", errors="replace")
                        if 'System.loadLibrary("paper_native_jni")' not in wrapper_text:
                            errors.append(f"{module}: wrapper missing System.loadLibrary in {wrapper_key}")
                        checked_load_wrappers.add(wrapper_key)
                for native_method in native_methods_in_bench_dir(bench_path):
                    checked_wrappers.add(native_method.source.relative_to(ROOT))
                    export = expected_jni_export(native_method)
                    checked_exports.add(export)
                    if export not in jni:
                        errors.append(
                            f"{module}: missing JNI export {export} for "
                            f"{native_method.source.relative_to(ROOT)}#{native_method.method_name}"
                        )

        for script in target.scripts:
            script_path = ROOT / script
            if not script_path.is_file():
                errors.append(f"{module}: missing bench script {script}")
            else:
                covered_scripts.add(script)

        if not any(term in docs for term in target.doc_terms):
            message = f"{module}: no doc term found in docs/PARITY_MATRIX.md or native/README.md"
            if strict_docs:
                errors.append(message)
            else:
                warnings.append(message)

    for name, target in EXTERNAL_TARGETS.items():
        jni_text = read_existing(target.jni_source)
        if not target.core_source.is_file():
            errors.append(f"{name}: missing core source {target.core_source.relative_to(ROOT)}")
        if not target.jni_source.is_file():
            errors.append(f"{name}: missing JNI source {target.jni_source.relative_to(ROOT)}")

        for bench_dir in target.bench_dirs:
            bench_path = ROOT / bench_dir
            if not path_has_java_sources(bench_path):
                errors.append(f"{name}: missing native Java bench sources in {bench_dir}")
            else:
                covered_benches.add(bench_dir)
                for wrapper in sorted(bench_path.glob("PaperNative*.java")):
                    wrapper_key = wrapper.relative_to(ROOT)
                    if wrapper_key not in checked_load_wrappers:
                        wrapper_text = wrapper.read_text(encoding="utf-8", errors="replace")
                        expected_load = f'System.loadLibrary("{target.load_library}")'
                        if expected_load not in wrapper_text:
                            errors.append(f"{name}: wrapper missing {expected_load} in {wrapper_key}")
                        checked_load_wrappers.add(wrapper_key)
                for native_method in native_methods_in_bench_dir(bench_path):
                    checked_wrappers.add(native_method.source.relative_to(ROOT))
                    export = expected_jni_export(native_method)
                    checked_exports.add(export)
                    if export not in jni_text:
                        errors.append(
                            f"{name}: missing JNI export {export} for "
                            f"{native_method.source.relative_to(ROOT)}#{native_method.method_name}"
                        )

        for script in target.scripts:
            script_path = ROOT / script
            if not script_path.is_file():
                errors.append(f"{name}: missing bench script {script}")
            else:
                covered_scripts.add(script)

        if not any(term in docs for term in target.doc_terms):
            message = f"{name}: no doc term found in docs/PARITY_MATRIX.md or native/README.md"
            if strict_docs:
                errors.append(message)
            else:
                warnings.append(message)

    real_pack_scripts = real_native_bench_script_names()
    all_pack_scripts = pack_all_script_names(errors)
    all_pack_counter = Counter(all_pack_scripts)
    all_pack_unique = sorted(all_pack_counter)
    all_pack_missing = sorted(set(real_pack_scripts) - set(all_pack_unique))
    all_pack_extra = sorted(set(all_pack_unique) - set(real_pack_scripts))
    all_pack_duplicates = sorted(script for script, count in all_pack_counter.items() if count > 1)
    leaf_manifest = pack_leaf_group_manifest(errors)
    leaf_groups = sorted({group for group, _ in leaf_manifest})
    leaf_scripts = sorted(script for _, script in leaf_manifest)
    leaf_script_counter = Counter(leaf_scripts)
    leaf_unique_scripts = sorted(leaf_script_counter)
    leaf_missing = sorted(set(real_pack_scripts) - set(leaf_unique_scripts))
    leaf_extra = sorted(set(leaf_unique_scripts) - set(real_pack_scripts))
    leaf_duplicates = sorted(script for script, count in leaf_script_counter.items() if count > 1)

    for script in all_pack_missing:
        errors.append(f"pack-contract: missing from PACK_GROUPS=all: {script}")
    for script in all_pack_extra:
        errors.append(f"pack-contract: extra in PACK_GROUPS=all: {script}")
    for script in all_pack_duplicates:
        errors.append(f"pack-contract: duplicate in PACK_GROUPS=all: {script}")
    for script in leaf_missing:
        errors.append(f"pack-contract: missing from leaf group manifest: {script}")
    for script in leaf_extra:
        errors.append(f"pack-contract: extra in leaf group manifest: {script}")
    for script in leaf_duplicates:
        errors.append(f"pack-contract: duplicate leaf group membership: {script}")

    print(f"modules_total={len(modules)}")
    print(f"required_bench_dirs_covered={len(covered_benches)}")
    print(f"required_scripts_covered={len(covered_scripts)}")
    print(f"native_wrappers_checked={len(checked_wrappers)}")
    print(f"native_load_wrappers_checked={len(checked_load_wrappers)}")
    print(f"native_exports_checked={len(checked_exports)}")
    print(f"pack_all_real_expected={len(real_pack_scripts)}")
    print(f"pack_all_scripts_listed={len(all_pack_scripts)}")
    print(f"pack_all_scripts_unique={len(all_pack_unique)}")
    print(f"pack_all_missing={len(all_pack_missing)}")
    print(f"pack_all_extra={len(all_pack_extra)}")
    print(f"pack_all_duplicates={len(all_pack_duplicates)}")
    print(f"pack_leaf_groups={len(leaf_groups)}")
    print(f"pack_leaf_group_memberships={len(leaf_scripts)}")
    print(f"pack_leaf_unique_scripts={len(leaf_unique_scripts)}")
    print(f"pack_leaf_missing={len(leaf_missing)}")
    print(f"pack_leaf_extra={len(leaf_extra)}")
    print(f"pack_leaf_duplicates={len(leaf_duplicates)}")
    print(f"warnings={len(warnings)}")
    print(f"errors={len(errors)}")

    for warning in warnings:
        print(f"WARNING {warning}")
    for error in errors:
        print(f"ERROR {error}")

    return 1 if errors else 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--strict-docs",
        action="store_true",
        help="treat missing documentation mentions as audit failures",
    )
    args = parser.parse_args(argv)
    return audit(strict_docs=args.strict_docs)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
