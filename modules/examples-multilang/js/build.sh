#!/usr/bin/env bash
# Build the JS module (QuickJS shim).
# Links against a locally built libqjs (the distro's prebuilt libqjs is
# built against a newer glibc than the JVM environment's). A working libqjs
# is committed next to this script in ./qjs (see README there); override
# with QJS_DIR to use a quickjs-ng build tree instead.
set -euo pipefail
cd "$(dirname "$0")"
QJS_DIR="${QJS_DIR:-$PWD/qjs}"
cc -shared -fPIC -O2 \
    -I"$PWD/../../../cplug-sdk-c/include" \
    -I"$QJS_DIR/../.." \
    -o libhello_js.so shim.c \
    -L"$QJS_DIR" -Wl,-rpath,"$QJS_DIR" -lqjs
echo "built $(pwd)/libhello_js.so"