#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p dist
cargo build --locked --release --bin tomito
cp target/release/tomito dist/bench-native
cargo build --locked --release --features egui-ui --bin tomito
cp target/release/tomito dist/bench-egui
cargo build --locked --release --features slint-ui --bin tomito-slint
# Compilation finishes before timing; no benchmark data touches user config.
exec python3 scripts/benchmark-ui-macos.py "$@"
