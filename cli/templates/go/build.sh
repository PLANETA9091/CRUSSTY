#!/usr/bin/env bash
# Build the Go module. go build -buildmode=c-shared needs a module dir and
# produces lib<name>.so plus a matching header we don't need.
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

CGO_CFLAGS="-I$SDK_INC" CGO_ENABLED=1 go build -buildmode=c-shared -trimpath -o lib"$NAME".so .
echo "built $(pwd)/lib$NAME.so"