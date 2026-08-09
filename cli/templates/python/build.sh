#!/usr/bin/env bash
# Build the Python module (C shim embedding CPython).
# --embed gives the full embedding flags (on split-python distros like Arch
# the plain --ldflags lacks -lpythonX.Y; falls back to -lpython3.X).
set -euo pipefail
cd "$(dirname "$0")"
NAME=__NAME__

if [ -n "${CRUSSTY_SDK:-}" ]; then SDK_INC="$CRUSSTY_SDK/cplug-sdk-c/include"
elif [ -f "../../cplug-sdk-c/include/cplug-abi.h" ]; then SDK_INC="../../cplug-sdk-c/include"
elif [ -f "../../../cplug-sdk-c/include/cplug-abi.h" ]; then SDK_INC="../../../cplug-sdk-c/include"
else
  SDK_DIR="${CRUSSTY_SDK_DIR:-/tmp/crussty-sdk}"
  if [ ! -f "$SDK_DIR/cplug-sdk-c/include/cplug-abi.h" ]; then
    git clone --depth 1 --filter=blob:none --sparse \
      https://github.com/PLANETA9091/CRUSSTY "$SDK_DIR" >/dev/null 2>&1 || true
    (cd "$SDK_DIR" && git sparse-checkout set cplug-sdk-c >/dev/null 2>&1 || true)
  fi
  SDK_INC="$SDK_DIR/cplug-sdk-c/include"
fi
test -f "$SDK_INC/cplug-abi.h" || { echo "cplug-abi.h not found; set CRUSSTY_SDK to your Crussty repo" >&2; exit 1; }

CFLAGS="$(python3-config --cflags 2>/dev/null || echo -I/usr/include/python3.14)"
LDFLAGS="$(python3-config --embed --ldflags 2>/dev/null || echo -lpython3.14)"
cc -shared -fPIC -O2 $CFLAGS \
    -I"$SDK_INC" \
    -o lib"$NAME".so shim.c $LDFLAGS
echo "built $(pwd)/lib$NAME.so"