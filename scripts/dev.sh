#!/bin/sh
# Launch macOS development in the desktop session, not a terminal/background service session.
set -eu
cd "$(dirname "$0")/.."
cargo=${CARGO:-cargo}
if [ "$(uname -s)" != Darwin ]; then
    exec "$cargo" run --locked -- --show
fi
"$cargo" build --locked --bin rusty-pomodoro
bundle=${RUSTY_POMODORO_DEV_BUNDLE:-"dist/dev/Rusty Pomodoro Dev.app"}
name=$(basename "$bundle" .app)
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources" dist/dev
# Cache the real Dock icon; do not run Swift/iconutil on every development launch.
icon="dist/dev/RustyPomodoro.icns"
if [ ! -f "$icon" ] || [ scripts/generate-app-icon-macos.swift -nt "$icon" ]; then
    work=$(mktemp -d)
    trap 'rm -rf "$work"' EXIT HUP INT TERM
    xcrun swiftc -sdk "$(xcrun --show-sdk-path)" scripts/generate-app-icon-macos.swift -framework AppKit -o "$work/icon-generator"
    "$work/icon-generator" "$work/png"
    mkdir "$work/RustyPomodoro.iconset"
    for size in 16 32 128 256 512; do
        cp "$work/png/icon-$size.png" "$work/RustyPomodoro.iconset/icon_${size}x${size}.png"
        double=$((size * 2))
        cp "$work/png/icon-$double.png" "$work/RustyPomodoro.iconset/icon_${size}x${size}@2x.png"
    done
    iconutil -c icns "$work/RustyPomodoro.iconset" -o "$icon"
    rm -rf "$work"
    trap - EXIT HUP INT TERM
fi
cp "$icon" "$bundle/Contents/Resources/RustyPomodoro.icns"
cp target/debug/rusty-pomodoro "$bundle/Contents/MacOS/rusty-pomodoro.new"
mv -f "$bundle/Contents/MacOS/rusty-pomodoro.new" "$bundle/Contents/MacOS/rusty-pomodoro"
cp packaging/Info.plist "$bundle/Contents/Info.plist"
identifier=io.local.rusty-pomodoro.dev
if [ "${RUSTY_POMODORO_DEV_BUNDLE+x}" = x ]; then
    # Isolated test bundles must not share LaunchServices identity with a running dev app.
    identifier="$identifier.$(printf %s "$bundle" | cksum | awk '{print $1}')"
fi
plutil -replace CFBundleIdentifier -string "$identifier" "$bundle/Contents/Info.plist"
plutil -replace CFBundleName -string "$name" "$bundle/Contents/Info.plist"
plutil -replace CFBundleDisplayName -string "$name" "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
# Explicitly forward config overrides; LaunchServices does not inherit the shell environment.
set --
if [ "${RUSTY_POMODORO_CONFIG_DIR+x}" = x ]; then
    set -- "$@" --env "RUSTY_POMODORO_CONFIG_DIR=$RUSTY_POMODORO_CONFIG_DIR"
fi
if [ "${TOMITO_CONFIG_DIR+x}" = x ]; then
    set -- "$@" --env "TOMITO_CONFIG_DIR=$TOMITO_CONFIG_DIR"
fi
# open -W can launch successfully but fail GetProcessPID and return immediately.
# Have this specific Slint instance identify itself, then supervise its lifetime.
launch_work=$(mktemp -d)
app_pid=""
log_pid=""
cleanup() {
    if [ -n "$log_pid" ]; then
        kill "$log_pid" 2>/dev/null || true
        wait "$log_pid" 2>/dev/null || true
    fi
    if [ -n "$app_pid" ]; then
        kill -TERM "$app_pid" 2>/dev/null || true
    fi
    rm -rf "$launch_work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
log="$launch_work/app.log"
: > "$log"
open -n "$bundle" "$@" \
    --env "RUSTY_POMODORO_DEV_PID_FILE=$launch_work/app.pid" \
    --stdout "$log" --stderr "$log" --args --show
attempt=0
while [ ! -s "$launch_work/app.pid" ]; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 100 ]; then
        echo "Slint development app did not report startup within 10 seconds." >&2
        tail -n +1 "$log" >&2
        exit 1
    fi
    sleep 0.1
done
read -r app_pid < "$launch_work/app.pid"
case "$app_pid" in
    ""|*[!0-9]*) app_pid=""; echo "Invalid Slint development PID." >&2; exit 1 ;;
esac
if [ "$app_pid" -le 1 ]; then
    app_pid=""
    echo "Invalid Slint development PID." >&2
    exit 1
fi
echo "Running Slint dev app (PID $app_pid). Press Ctrl+C to stop."
tail -n +1 -f "$log" &
log_pid=$!
while kill -0 "$app_pid" 2>/dev/null; do
    sleep 0.2
done
# Do not signal a PID again after observing that this instance exited.
app_pid=""
