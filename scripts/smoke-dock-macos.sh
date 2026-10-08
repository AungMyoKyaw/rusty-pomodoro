#!/bin/sh
# macOS Accessibility permission is required to exercise the actual Dock item.
set -eu
cd "$(dirname "$0")/.."
work=$(mktemp -d /tmp/tomito-dock-smoke.XXXXXX)
pid=""
register="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
bundle="$work/Tomito Dock Test $$.app"
name="Tomito Dock Test $$"
cleanup() {
    if [ -n "$pid" ]; then kill -TERM "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi
    "$register" -u "$bundle" 2>/dev/null || true
    rm -rf "$work"
}
trap cleanup EXIT HUP INT TERM
xcrun swiftc -sdk "$(xcrun --show-sdk-path)" scripts/dock-info-macos.swift -framework AppKit -o "$work/info"
cp -R "dist/Tomito Portable.app" "$bundle"
plutil -replace CFBundleIdentifier -string "io.local.tomito-portable.docktest.$$" "$bundle/Contents/Info.plist"
plutil -replace CFBundleName -string "$name" "$bundle/Contents/Info.plist"
plutil -replace CFBundleDisplayName -string "$name" "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
mkdir "$work/config"
printf 'hide_on_launch=true\n' > "$work/config/settings.conf"
TOMITO_CONFIG_DIR="$work/config" TOMITO_BENCHMARK=1 "$bundle/Contents/MacOS/tomito-slint" > "$work/app.log" 2>&1 &
pid=$!
sleep 2
"$work/info" "$pid" > "$work/before.json"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["activationPolicy"]==0 and d["customStopwatchIcon"] and not d["timerVisible"],d' "$work/before.json"
"$register" -f "$bundle"
# Start hidden rather than relying on Slint's disabled widget-accessibility tree.
osascript -e "tell application \"System Events\" to tell process \"Dock\" to perform action \"AXPress\" of UI element \"$name\" of list 1"
sleep 1
"$work/info" "$pid" > "$work/dock-click.json"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["activationPolicy"]==0 and d["customStopwatchIcon"] and d["timerVisible"],d; print(d)' "$work/dock-click.json"
osascript -e 'with timeout of 5 seconds' -e "tell application \"$bundle\" to reopen" -e 'end timeout'
sleep 1
"$work/info" "$pid" > "$work/reopen.json"
python3 -c 'import json,sys; assert json.load(open(sys.argv[1]))["timerVisible"]' "$work/reopen.json"
echo "PASS regular Dock app, custom icon, hidden-launch actual Dock-click restoration and reopen"
