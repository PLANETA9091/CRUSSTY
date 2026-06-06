#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCH_DIR="$ROOT_DIR/bench/jigsaw-placement"
REPORT_DIR="$ROOT_DIR/reports"
REPORT_FILE="$REPORT_DIR/jigsaw-placement-bench.txt"
BUILD_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$BUILD_DIR"
}
trap cleanup EXIT

mkdir -p "$REPORT_DIR"
javac -d "$BUILD_DIR" "$BENCH_DIR/JigsawPlacementMaxBenchmark.java"

{
    printf 'benchmark=jigsaw-placement stream-max-vs-loop\n'
    printf 'source=%s\n' "$BENCH_DIR/JigsawPlacementMaxBenchmark.java"
    printf 'date_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    java -version 2>&1
    printf '\n'
    java -cp "$BUILD_DIR" JigsawPlacementMaxBenchmark
} | tee "$REPORT_FILE"
