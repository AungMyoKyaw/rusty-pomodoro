#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p dist
# The AppKit frontend is retired; historical native measurements remain in docs.
cargo build --locked --release --no-default-features --features egui-ui --bin rusty-pomodoro
cp target/release/rusty-pomodoro dist/bench-egui
cargo build --locked --release --bin rusty-pomodoro-slint
# Compilation finishes before timing; no benchmark data touches user config.
exec python3 scripts/benchmark-ui-macos.py "$@"
