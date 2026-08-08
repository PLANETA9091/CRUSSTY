#!/usr/bin/env bash
# Build the Go module. go build -buildmode=c-shared needs a module dir and
# produces libhello_go.so plus a matching header we don't need.
set -euo pipefail
cd "$(dirname "$0")"
CGO_ENABLED=1 go build -buildmode=c-shared -trimpath -o libhello_go.so .
echo "built $(pwd)/libhello_go.so"