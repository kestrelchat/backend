#!/usr/bin/env bash
set -e
cargo build \
    --bin kestrel-wings

trap 'pkill -f kestrel-' SIGINT
cargo run --bin kestrel-wings