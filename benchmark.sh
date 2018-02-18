#!/bin/bash
set -e

TIME=/usr/bin/time
TIME_FILTER=
CARGO_RUN_CMD="cargo run --release --bin"

cargo build --release &> /dev/null
echo Benchmarking revision $(git rev-parse HEAD)
echo Benchmarking pixel-search
$TIME $CARGO_RUN_CMD pixel_search -- apples.png --size 128 --winsize 7 2>&1 | tail -n2 | head -n1
echo Benchmarking quilt
$TIME $CARGO_RUN_CMD quilt -- apples.png --size 1024 --blocksize 64 --overlap 12 2>&1 | tail -n2 | head -n1
