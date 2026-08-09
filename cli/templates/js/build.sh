#!/usr/bin/env bash
# Build the JS module (QuickJS shim).
# Links against a locally built libqjs (see modules/examples-multilang/js/qjs
# in the Crussty repo, or build quickjs-ng yourself); QJS_DIR points at the
# directory containing libqjs.so and quickjs.h.
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
  test -f "$SDK_INC/cplug-abi.h" || { echo "cplug-abi.h not found; set CRUSSTY_SDK to your Crussty repo" >&2; exit 1; }
fi

QJS_DIR="${QJS_DIR:-}"
if [ -z "$QJS_DIR" ]; then
  for c in ./qjs ../../../modules/examples-multilang/js/qjs; do
    if [ -f "$c/libqjs.so" ] || [ -f "$c/libqjs.so.0" ]; then QJS_DIR="$c"; break; fi
  done
  if [ -z "$QJS_DIR" ]; then
    echo "libqjs not found; set QJS_DIR=<dir with libqjs.so + quickjs.h>" >&2
    exit 1
  fi
fi
test -f "$QJS_DIR/quickjs.h" || { echo "quickjs.h not found in QJS_DIR=$QJS_DIR" >&2; exit 1; }

cc -shared -fPIC -O2 \
    -I"$SDK_INC" \
    -I"$QJS_DIR" \
    -o lib"$NAME".so shim.c \
    -L"$QJS_DIR" -Wl,-rpath,'$ORIGIN/qjs' -lqjs
echo "built $(pwd)/lib$NAME.so"