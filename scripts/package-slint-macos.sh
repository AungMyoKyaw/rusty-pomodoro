#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { echo "macOS packaging requires macOS" >&2; exit 1; }
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
xcrun swiftc -sdk "$(xcrun --show-sdk-path)" scripts/generate-app-icon-macos.swift -framework AppKit -o "$work/icon-generator"
"$work/icon-generator" "$work/png"
mkdir -p "$work/Tomito.iconset"
for size in 16 32 128 256 512; do
    cp "$work/png/icon-$size.png" "$work/Tomito.iconset/icon_${size}x${size}.png"
    double=$((size * 2))
    cp "$work/png/icon-$double.png" "$work/Tomito.iconset/icon_${size}x${size}@2x.png"
done
iconutil -c icns "$work/Tomito.iconset" -o "$work/Tomito.icns"
cargo build --locked --release --features slint-ui --bin tomito-slint
bundle="dist/Tomito Portable.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
# Replace the executable inode, rather than modifying code mapped by a running instance.
cp target/release/tomito-slint "$bundle/Contents/MacOS/tomito-slint.new"
mv -f "$bundle/Contents/MacOS/tomito-slint.new" "$bundle/Contents/MacOS/tomito-slint"
cp packaging/Portable-Info.plist "$bundle/Contents/Info.plist"
cp "$work/Tomito.icns" "$bundle/Contents/Resources/Tomito.icns"
cp assets/Hack-Regular.txt "$bundle/Contents/Resources/Hack-License.txt"
codesign --force --sign - "$bundle"
echo "$bundle"
du -sh "$bundle"
