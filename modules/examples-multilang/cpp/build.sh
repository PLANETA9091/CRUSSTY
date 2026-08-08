#!/usr/bin/env bash
# Build the C++ module.
set -euo pipefail
cd "$(dirname "$0")"
g++ -shared -fPIC -O2 -std=c++17 \
    -I"$PWD/../../cplug-abi" \
    -o libhello_cpp.so hello.cpp
echo "built $(pwd)/libhello_cpp.so"