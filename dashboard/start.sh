#!/usr/bin/env bash
set -e

: "${HOME:?HOME is not set}"
source "$HOME/.cargo/env"

echo "Ensuring wasm target"
rustup default stable
rustup target add wasm32-unknown-unknown
echo "Ensuring trunk"
cargo install trunk
echo "Serving dashboard"
trunk serve --release
