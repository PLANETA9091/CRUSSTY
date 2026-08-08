#!/usr/bin/env bash
# Build the Python module (C shim embedding CPython).
# --embed gives the full embedding flags (on split-python distros like Arch
# the plain --ldflags lacks -lpythonX.Y; falls back to -lpython3.14).
set -euo pipefail
cd "$(dirname "$0")"
CFLAGS="$(python3-config --cflags 2>/dev/null || echo -I/usr/include/python3.14)"
LDFLAGS="$(python3-config --embed --ldflags 2>/dev/null || echo -lpython3.14)"
cc -shared -fPIC -O2 $CFLAGS \
    -I"$PWD/../../../cplug-sdk-c/include" \
    -o libhello_py.so shim.c $LDFLAGS
echo "built $(pwd)/libhello_py.so"