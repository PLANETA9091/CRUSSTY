#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

python3 - "$ROOT" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
source = root / "upstream/Paper/paper-server/patches/sources/net/minecraft/network/Connection.java.patch"
text = source.read_text()

guard = re.compile(
    r"^[ +]*private\s+void\s+flushQueue\s*\(\s*\)\s*\{"
    r".*?"
    r"^[ +]*if\s*\(\s*this\.pendingActions\.isEmpty\(\)\s*\)\s*\{\s*$"
    r"\s*^[ +]*return;\s*$"
    r"\s*^[ +]*\}\s*$"
    r"\s*^[ +]*synchronized\s*\(\s*this\.pendingActions\s*\)",
    re.M | re.S,
)
errors = []
if not guard.search(text):
    errors.append(f"{source}: missing expected pendingActions empty guard before synchronized(this.pendingActions) in flushQueue()")

if "this.canDeferFlush()" in text:
    errors.append(f"{source}: stale helper call 'this.canDeferFlush()' is not allowed")

if "final boolean deferFlush = flush && sendListener == null && !hasFinishListener;" in text:
    errors.append(f"{source}: unguarded deferred flush expression is not allowed")

if errors:
    raise SystemExit("\n".join(errors))

print("connection_deferred_flush_scope=PASS")
print(f"{source.relative_to(root)} flush_queue_pending_actions_empty_guard=present")
print(f"{source.relative_to(root)} stale_deferred_flush_helper_claims=absent")
PY
