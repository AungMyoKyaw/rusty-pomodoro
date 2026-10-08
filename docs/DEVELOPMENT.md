# Development

## Build and package

`make build` builds Slint binaries for macOS arm64/x86_64, Windows x86_64, and Linux x86_64. Cross-builds need Rust target support, MinGW (`x86_64-w64-mingw32-gcc`), and a Linux cross-linker (`x86_64-unknown-linux-gnu-gcc`). Override linker paths with `WINDOWS_LINKER` or `LINUX_LINKER`. Build one platform with `make build-macos`, `make build-windows`, or `make build-linux`. Outputs go under `target/<triple>/release/`.

- `make dev` runs the shared Slint UI.
- The default `rusty-pomodoro` binary uses native AppKit and is macOS-only.
- `make package-macos` creates a local, ad-hoc-signed app bundle. It is not notarized and does not replace `/Applications/Tomito.app`.
- `./scripts/package-slint-macos.sh` creates a separate Slint app bundle.
- Push a version tag such as `v1.0.0` to build and publish Linux x86_64, Windows x86_64, macOS arm64, and macOS x86_64 release assets. macOS bundles use ad-hoc signing, not Developer ID signing or notarization.

## Shared Slint UI

Run it directly:

```sh
cargo run --locked --release --features slint-ui --bin rusty-pomodoro-slint
```

The shared UI provides editable timer settings, five accent themes, timer controls, statistics, CSV export, and confirmed history reset. macOS adds tray controls, sounds, global shortcuts, and sleep/wake hooks. Windows/Linux do not implement those integrations. Closing behavior differs by platform: macOS hides to the tray/Dock; Windows/Linux exit.

The window defaults to 420×560 logical pixels (minimum 380×560). Settings use Durations, Behavior, and Appearance subpanels. See [Architecture and UX review](ARCHITECTURE.md) for module details and known UI findings.

The Slint variant shares configuration/history with other variants by default. Set `RUSTY_POMODORO_CONFIG_DIR` to isolate data. Settings and history on macOS remain under `~/Library/Application Support/tomito-rs/` for compatibility. Legacy `TOMITO_*` environment variables remain accepted.

## Shortcuts

Native macOS:

- Space: start/pause.
- Command+T/R/S/X: start/pause, restart, skip, stop.
- Command+1/2: timer/statistics. Command+comma: settings. Command+Q: quit.
- Global Control+Option+Space/R/S/X/T: start/pause, restart, skip, stop, show timer.

Shared Slint UI: Space, R, S, X, F, Escape. Optional egui uses the same keys; its macOS build does not use the native build's fixed global shortcuts.

## Validation

```sh
make fmt-check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --features egui-ui --all-targets
cargo clippy --features egui-ui --all-targets -- -D warnings
cargo test --features slint-ui --lib --bin rusty-pomodoro-slint
cargo clippy --features slint-ui --bin rusty-pomodoro-slint -- -D warnings
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" cargo run --release --features slint-ui --bin rusty-pomodoro-slint -- --smoke
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" RUSTY_POMODORO_CAPTURE_DIR="$(mktemp -d)" cargo run --features slint-ui --bin rusty-pomodoro-slint -- --smoke-visual
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" cargo run --release --features slint-ui --bin rusty-pomodoro-slint -- --smoke-expiry
./scripts/smoke-dock-macos.sh
./scripts/benchmark-ui-macos.sh --rounds 1 --samples 3
```

The Dock smoke test needs macOS Accessibility permission. `scripts/smoke-macos.applescript <pid>` exercises a fresh native test instance with isolated data.

For measurements and implementation scope, see [Performance](PERFORMANCE.md). For behavior evidence and parity gaps, see [Reverse engineering](REVERSE_ENGINEERING.md).
