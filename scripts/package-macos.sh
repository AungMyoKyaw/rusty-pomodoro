#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { echo "macOS required" >&2; exit 1; }
cargo build --locked --release
bundle="dist/Rusty Pomodoro.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp target/release/rusty-pomodoro "$bundle/Contents/MacOS/rusty-pomodoro"
cp packaging/Info.plist "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
echo "$bundle"
du -sh "$bundle"
