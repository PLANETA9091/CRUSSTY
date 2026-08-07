#!/usr/bin/env bash
# v2: boot the kernel from versions/ with the native agent and modules.
# Usage: ./run.sh [extra kernel args]   (DIST_JAVA_OPTS overrides JVM flags)
set -euo pipefail
cd "$(dirname "$0")"
exec java -jar launcher/launcher.jar "$@"
