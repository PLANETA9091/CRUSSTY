#!/usr/bin/env python3
"""Build pack/cmenus-pack.zip from pack/assets + pack.mcmeta.

Deterministic: sorted entries, fixed DOS timestamps. The zip is committed;
CI rebuilds it and fails on drift (git diff --exit-code).
"""
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACK = ROOT / "pack"
OUT = PACK / "cmenus-pack.zip"

FIXED_DATE = (2026, 1, 1, 0, 0, 0)


def main() -> None:
    entries: list[Path] = [PACK / "pack.mcmeta"]
    assets = PACK / "assets"
    for p in sorted(assets.rglob("*")):
        if p.is_file() and "__pycache__" not in str(p):
            entries.append(p)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for p in entries:
            arc = p.relative_to(PACK).as_posix()
            info = zipfile.ZipInfo(arc, date_time=FIXED_DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, p.read_bytes())
    print(f"pack written: {OUT} ({OUT.stat().st_size} bytes, {len(entries)} files)")


if __name__ == "__main__":
    main()
