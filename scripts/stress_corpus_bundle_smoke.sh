#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

python3 - "$TMP" <<'PY'
from __future__ import annotations

import csv
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
reports = root / "reports"
logs = root / "logs"
artifacts = root / "artifacts"
reports.mkdir(parents=True)
logs.mkdir(parents=True)


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write(path: Path, text: str | bytes) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(text, bytes):
        path.write_bytes(text)
    else:
        path.write_text(text, encoding="utf-8")
    return path


optimized = write(artifacts / "optimized-paper-1.21.10-mojmap.jar", b"optimized\n")
run_sh = write(artifacts / "optimized-runtime" / "run.sh", "#!/bin/sh\nexec java -jar runtime.jar \"$@\"\n")
native = write(artifacts / "optimized-runtime" / "native" / "libpaper_native_jni.so", b"native\n")
optimized_sha = sha(optimized)
run_sh_sha = sha(run_sh)
native_sha = sha(native)
artifacts_json = write(
    reports / "artifacts.json",
    json.dumps(
        {
            "optimized": {"path": str(optimized), "sha256": optimized_sha},
            "optimized_runtime": {
                "run_sh": {"path": str(run_sh), "sha256": run_sh_sha},
                "native_library": {"path": str(native), "sha256": native_sha},
            },
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
)

server_log = write(
    logs / "stress-corpus-gate.log",
    "\n".join(
        [
            '[00:00:00 INFO]: Done (42.500s)! For help, type "help"',
            "[00:00:01 INFO]: StressProbe joined the game",
            "[00:00:02 INFO]: [CompatProbe] COMPAT_PROBE event=PlayerJoinEvent sequence=10 detail=StressProbe",
            "[00:00:03 INFO]: StressProbe left the game",
            "[00:00:04 INFO]: There are 13 data pack(s) enabled: [vanilla], [paper], [file/Terralith.zip]",
        ]
    )
    + "\n",
)
influx_log = write(logs / "stress-corpus-influx-mock.log", "mock influx accepted writes\n")
write(reports / "stress-corpus-hard-errors.txt", "")
write(
    reports / "stress-corpus-status.json",
    json.dumps(
        {
            "version": {"name": "Paper 1.21.10", "protocol": 773},
            "players": {"max": 69, "online": 0},
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
)
write(
    reports / "stress-corpus-join.txt",
    "\n".join(
        [
            "join_attempt=1",
            "join_client=login username=StressProbe host=127.0.0.1 port=25565",
            "join_client=end username=StressProbe reason=join-check-complete",
        ]
    )
    + "\n",
)
write(
    reports / "stress-corpus-summary.txt",
    "\n".join(
        [
            "stress_corpus_gate=PASS",
            "failure_reason=none",
            "matrix_plugin_count=12",
            "stress_plugin_count=22",
            "plugin_count=34",
            "datapack_count=10",
            "done_seconds=42.500",
            "java_opts=-Xms4G -Xmx16G",
            f"launcher={run_sh}",
            f"artifacts_json={artifacts_json}",
            f"optimized_artifact_path={optimized}",
            f"optimized_artifact_sha256={optimized_sha}",
            f"optimized_runtime_run_sh={run_sh}",
            f"optimized_runtime_run_sh_sha256={run_sh_sha}",
            f"optimized_runtime_native_library={native}",
            f"optimized_runtime_native_library_sha256={native_sha}",
            "launcher_kind=optimized-runtime",
            "server_port=25565",
            f"log={server_log}",
            f"serverpulse_influx_log={influx_log}",
            f"status_json={reports / 'stress-corpus-status.json'}",
            f"hard_errors={reports / 'stress-corpus-hard-errors.txt'}",
        ]
    )
    + "\n",
)

rows = []
for index in range(22):
    rows.append(
        {
            "side": "plugin",
            "slug": f"plugin-{index}",
            "title": f"Plugin {index}",
            "version": "1.0.0",
            "game_version": "1.21.10",
            "loader": "paper",
            "filename": f"plugin-{index}.jar",
            "path": f"plugins/stress/plugin-{index}.jar",
            "sha256": hashlib.sha256(f"plugin-{index}".encode()).hexdigest(),
            "size": "1234",
            "risk": "test",
            "intent": "smoke",
            "url": "https://example.invalid/plugin.jar",
            "status": "downloaded",
        }
    )
for index in range(10):
    rows.append(
        {
            "side": "datapack",
            "slug": f"datapack-{index}",
            "title": f"Datapack {index}",
            "version": "1.0.0",
            "game_version": "1.21.10",
            "loader": "datapack",
            "filename": f"datapack-{index}.zip",
            "path": f"datapacks/stress/datapack-{index}.zip",
            "sha256": hashlib.sha256(f"datapack-{index}".encode()).hexdigest(),
            "size": "2345",
            "risk": "worldgen",
            "intent": "smoke",
            "url": "https://example.invalid/datapack.zip",
            "status": "downloaded",
        }
    )
with (reports / "stress-corpus-artifacts.csv").open("w", encoding="utf-8", newline="") as handle:
    writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
    writer.writeheader()
    writer.writerows(rows)
write(
    reports / "stress-corpus-manifest.json",
    json.dumps(
        {
            "claim_scope": "downloaded stress corpus only; not a compatibility or performance claim",
            "plugin_count": 22,
            "datapack_count": 10,
            "rows": rows,
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
)
inspection_lines = [
    "stress_corpus_inspection_pass=true",
    "plugin_count=22",
    "datapack_count=10",
    "failure_count=0",
]
inspection_lines.extend(f"plugin status=ok filename=plugin-{index}.jar" for index in range(22))
inspection_lines.extend(f"datapack status=ok filename=datapack-{index}.zip" for index in range(10))
write(reports / "stress-corpus-inspection.txt", "\n".join(inspection_lines) + "\n")
write(
    reports / "stress-corpus-inspection.json",
    json.dumps(
        {
            "claim_scope": "descriptor/datapack shape inspection only; not a runtime compatibility claim",
            "plugin_count": 22,
            "datapack_count": 10,
            "failure_count": 0,
            "failures": [],
            "rows": [],
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
)
PY

python3 "$ROOT/scripts/export_stress_corpus_bundle.py" \
  --summary "$TMP/reports/stress-corpus-summary.txt" \
  --out-dir "$TMP/bundle" > "$TMP/export.out"
rg -q '^stress_corpus_bundle_export_pass=true$' "$TMP/export.out"
rg -q '^evidence_file_count=11$' "$TMP/export.out"

python3 "$ROOT/scripts/validate_stress_corpus_bundle.py" "$TMP/bundle" > "$TMP/validate.out"
rg -q '^stress_corpus_bundle_validation_pass=true$' "$TMP/validate.out"
rg -q '^plugin_count=34$' "$TMP/validate.out"
rg -q '^datapack_count=10$' "$TMP/validate.out"
test -f "$TMP/bundle/CLAIM.md"
test -f "$TMP/bundle/evidence/stress-corpus-gate.log"

cp -a "$TMP/bundle" "$TMP/bad-hard-errors"
printf '1:[00:00:00 ERROR]: synthetic hard failure\n' \
  > "$TMP/bad-hard-errors/evidence/stress-corpus-hard-errors.txt"
if python3 "$ROOT/scripts/validate_stress_corpus_bundle.py" "$TMP/bad-hard-errors" \
  > "$TMP/bad-hard-errors.out" 2>&1; then
  echo "Expected non-empty hard errors to fail validation." >&2
  exit 1
fi
rg -q '^stress_corpus_bundle_validation_pass=false$' "$TMP/bad-hard-errors.out"
rg -q 'stress-corpus-hard-errors.txt must be empty' "$TMP/bad-hard-errors.out"

cp "$TMP/reports/stress-corpus-summary.txt" "$TMP/bad-summary.txt"
sed -i 's/^stress_corpus_gate=PASS$/stress_corpus_gate=FAIL/' "$TMP/bad-summary.txt"
if python3 "$ROOT/scripts/export_stress_corpus_bundle.py" \
  --summary "$TMP/bad-summary.txt" \
  --out-dir "$TMP/bad-export" > "$TMP/bad-export.out" 2>&1; then
  echo "Expected failed summary to fail export." >&2
  exit 1
fi
rg -q '^stress_corpus_bundle_export_pass=false$' "$TMP/bad-export.out"
rg -q 'stress_corpus_gate must be PASS' "$TMP/bad-export.out"

echo "stress_corpus_bundle_smoke=PASS"
