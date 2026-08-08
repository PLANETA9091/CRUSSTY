#!/usr/bin/env bash
# Build the C module as libhello_c.so next to the manifest.
set -euo pipefail
cd "$(dirname "$0")"
gcc -shared -fPIC -O2 \
    -I"$PWD/../../cplug-abi" \
    -o libhello_c.so hello.c
echo "built $(pwd)/libhello_c.so"