#!/usr/bin/env bash
set -e

: "${HOME:?HOME is not set}"
source "$HOME/.cargo/env"

echo "Starting simulation"
cargo run --release
echo "Finished"
