#!/usr/bin/env python3
from pathlib import Path
import sys


def first_chunk_version(path: Path):
    with path.open("rb") as handle:
        header = handle.read(4096)
        for index in range(1024):
            entry = header[index * 4:index * 4 + 4]
            offset = int.from_bytes(entry[:3], "big")
            sectors = entry[3]
            if not offset or not sectors:
                continue
            handle.seek(offset * 4096)
            chunk_header = handle.read(5)
            if len(chunk_header) != 5:
                return index, None, None
            length = int.from_bytes(chunk_header[:4], "big")
            version = chunk_header[4] & 0x7F
            return index, length, version
    return None


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "runs/plugin-matrix")
    regions = sorted(root.glob("world*/**/region/r.*.*.mca"))
    if not regions:
        print(f"no region files found under {root}", file=sys.stderr)
        return 1
    print("file,chunk_index,length,compression_version")
    bad = False
    for region in regions:
        result = first_chunk_version(region)
        if result is None:
            continue
        chunk_index, length, version = result
        print(f"{region},{chunk_index},{length},{version}")
        if version != 4:
            bad = True
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())

