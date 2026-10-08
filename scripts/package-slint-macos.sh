#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { echo "macOS packaging requires macOS" >&2; exit 1; }
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
cargo build --locked --release --features slint-ui --bin rusty-pomodoro-slint
bundle="dist/Rusty Pomodoro Slint.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
# Replace the executable inode, rather than modifying code mapped by a running instance.
cp target/release/rusty-pomodoro-slint "$bundle/Contents/MacOS/rusty-pomodoro-slint.new"
mv -f "$bundle/Contents/MacOS/rusty-pomodoro-slint.new" "$bundle/Contents/MacOS/rusty-pomodoro-slint"
cp packaging/Portable-Info.plist "$bundle/Contents/Info.plist"
cp "$work/RustyPomodoro.icns" "$bundle/Contents/Resources/RustyPomodoro.icns"
cp assets/Hack-Regular.txt "$bundle/Contents/Resources/Hack-License.txt"
codesign --force --sign - "$bundle"
echo "$bundle"
du -sh "$bundle"
