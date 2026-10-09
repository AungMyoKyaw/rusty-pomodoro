#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { echo "macOS packaging requires macOS" >&2; exit 1; }
# Keep the existing release/Homebrew bundle name available as a compatibility alias.
case "${1:-}" in
    "") binary=rusty-pomodoro; bundle="dist/Rusty Pomodoro.app"; plist=packaging/Info.plist ;;
    --compat-slint) binary=rusty-pomodoro-slint; bundle="dist/Rusty Pomodoro Slint.app"; plist=packaging/Portable-Info.plist ;;
    *) echo "Usage: $0 [--compat-slint]" >&2; exit 2 ;;
esac
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
xcrun swiftc -sdk "$(xcrun --show-sdk-path)" scripts/generate-app-icon-macos.swift -framework AppKit -o "$work/icon-generator"
"$work/icon-generator" "$work/png"
mkdir -p "$work/RustyPomodoro.iconset"
for size in 16 32 128 256 512; do
    cp "$work/png/icon-$size.png" "$work/RustyPomodoro.iconset/icon_${size}x${size}.png"
    double=$((size * 2))
    cp "$work/png/icon-$double.png" "$work/RustyPomodoro.iconset/icon_${size}x${size}@2x.png"
done
iconutil -c icns "$work/RustyPomodoro.iconset" -o "$work/RustyPomodoro.icns"
cargo build --locked --release --bin "$binary"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
# Replace the executable inode, rather than modifying code mapped by a running instance.
cp "target/release/$binary" "$bundle/Contents/MacOS/$binary.new"
mv -f "$bundle/Contents/MacOS/$binary.new" "$bundle/Contents/MacOS/$binary"
cp "$plist" "$bundle/Contents/Info.plist"
cp "$work/RustyPomodoro.icns" "$bundle/Contents/Resources/RustyPomodoro.icns"
cp assets/Hack-Regular.txt "$bundle/Contents/Resources/Hack-License.txt"
cp assets/NotoSans-OFL.txt "$bundle/Contents/Resources/NotoSans-License.txt"
codesign --force --sign - "$bundle"
echo "$bundle"
du -sh "$bundle"
