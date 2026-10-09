# Development

## Build and package

Slint is the default desktop UI on macOS, Windows, and Linux. `cargo run` and `make dev` launch the same software-rendered interface. On macOS, `make dev` builds a lightweight debug bundle in `dist/dev/` and launches it through LaunchServices so it opens in the desktop session with a Dock icon rather than a terminal/background service session. The debug bundle includes the actual stopwatch icon and uses `LSUIElement=false`. On other platforms it uses `cargo run`. It passes `--show` to display and focus the timer even when Hide on launch is saved. This overrides only startup visibility, not the saved setting. `cargo run --locked -- --show` overrides hidden startup but does not provide the app-bundle/LaunchServices registration used by `make dev`; prefer `make dev` on macOS for reliable foreground and Dock behavior. Ordinary launches still respect Hide on launch. `make dev` stays running and streams the app's logs; Ctrl+C stops only the instance launched by that command. It tracks an explicit per-launch PID rather than using macOS `open -W`.

`make build` builds Slint binaries for macOS arm64/x86_64, Windows x86_64, and Linux x86_64. Cross-builds need Rust target support, MinGW (`x86_64-w64-mingw32-gcc`), and a Linux cross-linker (`x86_64-unknown-linux-gnu-gcc`). Override linker paths with `WINDOWS_LINKER` or `LINUX_LINKER`. Build one platform with `make build-macos`, `make build-windows`, or `make build-linux`. Outputs go under `target/<triple>/release/rusty-pomodoro` (with `.exe` on Windows).

- `make package-macos` creates `dist/Rusty Pomodoro.app` with the Slint UI, icon, and font licenses. It is ad-hoc signed, not notarized, and does not replace `/Applications/Tomito.app`.
- `./scripts/package-slint-macos.sh` creates the compatibility bundle `dist/Rusty Pomodoro Slint.app` used by existing releases and the Homebrew cask. Both bundles use Slint.
- `rusty-pomodoro-slint` remains a compatibility binary for existing commands.
- Optional egui: `cargo run --locked --no-default-features --features egui-ui --bin rusty-pomodoro`. The retired AppKit shell is no longer compiled.
- Push a version tag such as `v1.0.0` to build and publish Linux x86_64, Windows x86_64, macOS arm64, and macOS x86_64 release assets. macOS bundles use ad-hoc signing, not Developer ID signing or notarization.

## Slint UI

Run directly:

```sh
cargo run --locked --release
```

Slint provides editable timer settings, five accent themes, timer controls, statistics, CSV export, and confirmed history reset. macOS adds tray controls, Dock restoration, sounds, global shortcuts, and sleep/wake hooks through platform APIs. Those APIs do not draw the timer, settings, or statistics UI. Windows/Linux do not implement those integrations. On macOS, closing hides to the tray/Dock; on Windows/Linux, closing exits.

The window defaults to 420×560 logical pixels (minimum 380×560). Settings use Durations, Behavior, and Appearance subpanels. See [Architecture and UX review](ARCHITECTURE.md) for module details and known UI findings.

Configuration/history remain compatible with previous variants. Set `RUSTY_POMODORO_CONFIG_DIR` to isolate data. Settings and history on macOS remain under `~/Library/Application Support/tomito-rs/`. Legacy `TOMITO_*` environment variables remain accepted.

## Shortcuts

Slint: Space (start/pause), R (restart), S (skip), X (stop), F (finish overtime), Escape (timer panel).

macOS global Control+Option+Space/R/S/X/T: start/pause, restart, skip, stop, show timer. Optional egui uses the same local keys but does not register these global shortcuts.

## Validation

```sh
make fmt-check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --no-default-features --features egui-ui --lib --bin rusty-pomodoro
cargo clippy --locked --no-default-features --features egui-ui --lib --bin rusty-pomodoro -- -D warnings
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" cargo run --locked --release -- --smoke
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" RUSTY_POMODORO_CAPTURE_DIR="$(mktemp -d)" cargo run --locked --release -- --smoke-visual
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" cargo run --locked --release -- --smoke-expiry
python3 scripts/smoke-startup-macos.py
make package-macos
./scripts/smoke-dock-macos.sh
./scripts/benchmark-ui-macos.sh --rounds 1 --samples 3
```

Smoke tests require an explicitly set, empty configuration directory. The Dock smoke test also needs macOS Accessibility permission. The benchmark compares current Slint and optional egui; historical native AppKit figures are not relabeled as Slint measurements. `scripts/smoke-macos.applescript` targets the retired AppKit controls and is not a test of the current UI.

For measurements and implementation scope, see [Performance](PERFORMANCE.md). For behavior evidence and parity gaps, see [Reverse engineering](REVERSE_ENGINEERING.md).
