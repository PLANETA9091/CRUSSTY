#!/usr/bin/env python3
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def fail_missing_inputs(missing: list[Path]) -> None:
    lines = [
        "missing required artifact report inputs:",
        *[f"  - {display_path(path)}" for path in missing],
        "",
        "This usually means the full EULA-enabled optimized runtime prep was not run.",
        "Run:",
        "  MC_EULA_AGREE=true ./scripts/build_optimized.sh",
        "",
        "No reports were written.",
    ]
    raise SystemExit("\n".join(lines))


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_text_hash(path: Path) -> str:
    return path.read_text(encoding="utf-8").strip().split()[0]


def main() -> None:
    reports = ROOT / "reports"
    artifacts = ROOT / "artifacts"
    runtime = artifacts / "optimized-runtime"

    optimized = artifacts / "optimized-paper-1.21.10-mojmap.jar"
    stock = artifacts / "paper-1.21.10-130.jar"
    vanilla = artifacts / "vanilla-1.21.10.jar"
    app_cds = runtime / "app-cds.jsa"
    run_sh = runtime / "run.sh"
    runtime_hash_file = runtime / "runtime.jar.sha256"
    native_lib = runtime / "native" / "libpaper_native_jni.so"
    native_lib_hash_file = runtime / "native" / "libpaper_native_jni.so.sha256"
    chunk_encode_native_lib = runtime / "native" / "libpaper_native_chunk_encode_jni.so"
    chunk_encode_native_lib_hash_file = runtime / "native" / "libpaper_native_chunk_encode_jni.so.sha256"
    chunk_encode_native_required = False
    if run_sh.is_file():
        chunk_encode_native_required = "PAPER_NATIVE_CHUNK_PACKET_ENCODE" in run_sh.read_text(encoding="utf-8")

    missing = [
        path
        for path in (
            optimized,
            stock,
            vanilla,
            app_cds,
            run_sh,
            runtime_hash_file,
            native_lib,
            native_lib_hash_file,
            *(
                (chunk_encode_native_lib, chunk_encode_native_lib_hash_file)
                if chunk_encode_native_required
                else ()
            ),
        )
        if not path.is_file()
    ]

    mappings_hash_file = runtime / "mappings.hash"
    remap_dirs = sorted((runtime / "remap-classpath").glob("*.jar"))
    if mappings_hash_file.is_file():
        mappings_hash = read_text_hash(mappings_hash_file)
    elif remap_dirs:
        mappings_hash = remap_dirs[-1].stem
    else:
        mappings_hash = None
        missing.extend(
            [
                mappings_hash_file,
                runtime / "remap-classpath" / "*.jar",
                runtime / "reversed-mappings" / "*.tiny",
                runtime / "plugin-remaps" / "<mappings-hash>" / "skipped-hashes.txt",
                runtime / "plugin-remaps" / "<mappings-hash>" / "libraries" / "skipped-hashes.txt",
            ]
        )

    if mappings_hash is not None:
        remap_classpath = runtime / "remap-classpath" / f"{mappings_hash}.jar"
        reversed_mappings = runtime / "reversed-mappings" / f"{mappings_hash}.tiny"
        plugin_remap_dir = runtime / "plugin-remaps" / mappings_hash
        plugin_skips = plugin_remap_dir / "skipped-hashes.txt"
        library_skips = plugin_remap_dir / "libraries" / "skipped-hashes.txt"
        missing.extend(
            path
            for path in (
                remap_classpath,
                reversed_mappings,
                plugin_skips,
                library_skips,
            )
            if not path.is_file()
        )
    else:
        remap_classpath = runtime / "remap-classpath" / "<mappings-hash>.jar"
        reversed_mappings = runtime / "reversed-mappings" / "<mappings-hash>.tiny"
        plugin_remap_dir = runtime / "plugin-remaps" / "<mappings-hash>"
        plugin_skips = plugin_remap_dir / "skipped-hashes.txt"
        library_skips = plugin_remap_dir / "libraries" / "skipped-hashes.txt"

    if missing:
        fail_missing_inputs(missing)

    library_probe = ROOT / "plugins" / "matrix" / "LibraryProbe-0.1.0.jar"
    library_probe_dep = ROOT / "plugins" / "matrix-libraries" / "library-probe-dep.jar"
    matrix_plugins = sorted((ROOT / "plugins" / "matrix").glob("*.jar"))
    matrix_libraries = sorted((ROOT / "plugins" / "matrix-libraries").glob("*.jar"))
    stress_plugins = sorted((ROOT / "plugins" / "stress").glob("*.jar"))
    stress_datapacks = sorted((ROOT / "datapacks" / "stress").glob("*.zip"))

    optimized_runtime = {
        "run_sh": {
            "path": str(run_sh),
            "sha256": sha256(run_sh),
        },
        "app_cds": {
            "path": str(app_cds),
            "sha256": sha256(app_cds),
        },
        "runtime_jar_sha256_file": {
            "path": str(runtime_hash_file),
            "sha256": sha256(runtime_hash_file),
            "runtime_jar_sha256": read_text_hash(runtime_hash_file),
        },
        "native_library": {
            "path": str(native_lib),
            "sha256": sha256(native_lib),
        },
        "native_library_sha256_file": {
            "path": str(native_lib_hash_file),
            "sha256": sha256(native_lib_hash_file),
            "native_library_sha256": read_text_hash(native_lib_hash_file),
        },
        "remap_classpath": {
            "path": str(remap_classpath),
            "sha256": sha256(remap_classpath),
        },
    }
    if chunk_encode_native_lib.is_file():
        optimized_runtime["chunk_encode_native_library"] = {
            "path": str(chunk_encode_native_lib),
            "sha256": sha256(chunk_encode_native_lib),
        }
        optimized_runtime["chunk_encode_native_library_sha256_file"] = {
            "path": str(chunk_encode_native_lib_hash_file),
            "sha256": sha256(chunk_encode_native_lib_hash_file),
            "chunk_encode_native_library_sha256": read_text_hash(chunk_encode_native_lib_hash_file),
        }

    data = {
        "mc_version": "1.21.10",
        "stock_paper": {
            "path": str(stock),
            "sha256": sha256(stock),
        },
        "vanilla": {
            "path": str(vanilla),
            "sha1": hashlib.sha1(vanilla.read_bytes()).hexdigest(),
        },
        "optimized": {
            "path": str(optimized),
            "sha256": sha256(optimized),
        },
        "optimized_runtime": optimized_runtime,
        "plugin_matrix": [
            {
                "path": str(path),
                "sha256": sha256(path),
            }
            for path in matrix_plugins
        ],
        "plugin_matrix_libraries": [
            {
                "path": str(path),
                "sha256": sha256(path),
            }
            for path in matrix_libraries
        ],
        "stress_plugins": [
            {
                "path": str(path),
                "sha256": sha256(path),
            }
            for path in stress_plugins
        ],
        "stress_datapacks": [
            {
                "path": str(path),
                "sha256": sha256(path),
            }
            for path in stress_datapacks
        ],
    }

    (reports / "artifacts.json").write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    paths = [
        optimized,
        stock,
        vanilla,
        app_cds,
        run_sh,
        runtime_hash_file,
        native_lib,
        native_lib_hash_file,
        *(
            (chunk_encode_native_lib, chunk_encode_native_lib_hash_file)
            if chunk_encode_native_lib.is_file()
            else ()
        ),
        reversed_mappings,
        remap_classpath,
        plugin_skips,
        library_skips,
        *matrix_plugins,
        *matrix_libraries,
        *stress_plugins,
        *stress_datapacks,
    ]
    with (reports / "artifact-hashes.txt").open("w", encoding="utf-8") as handle:
        for path in paths:
            handle.write(f"{sha256(path)}  {path}\n")


if __name__ == "__main__":
    main()
