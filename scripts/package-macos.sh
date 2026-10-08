#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { echo "macOS required" >&2; exit 1; }
cargo build --locked --release
bundle="dist/Tomito RS.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp target/release/tomito "$bundle/Contents/MacOS/tomito"
cp packaging/Info.plist "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
echo "$bundle"
du -sh "$bundle"
