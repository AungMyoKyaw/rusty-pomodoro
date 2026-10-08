# Tomito RS

A small Rust Pomodoro app independently rebuilt from the behavior of the installed Tomito app.

**Default on macOS: native AppKit, controlled entirely by Rust.**
Optional egui and shared software-rendered Slint interfaces remain available. AppKit avoids the GPU context and font atlas that made the measured egui build use substantially more memory.

## Run and build

```sh
make dev       # Run the portable Slint app for local development
make build     # Build macOS, Windows, and Linux release binaries
make package-macos
open "dist/Tomito RS.app"
```

`make build` builds Slint binaries for macOS arm64/x86_64, Windows x86_64, and Linux x86_64. It needs Rust target support plus MinGW (`x86_64-w64-mingw32-gcc`) and a Linux cross-linker (`x86_64-unknown-linux-gnu-gcc`) for cross-builds. Override linker paths with `WINDOWS_LINKER` or `LINUX_LINKER`. Build one platform with `make build-macos`, `make build-windows`, or `make build-linux`. Outputs land under `target/<triple>/release/`.

`make dev` runs the shared Slint UI. The default `tomito` binary uses native AppKit and is macOS-only. `make package-macos` creates its local/ad-hoc signed app bundle; it is not notarized and does not replace `/Applications/Tomito.app`. Windows/Linux runtime behavior is not tested.

`.editorconfig` sets two-space, space-only indentation for editors. Makefile recipes use tabs because Make requires them. `rustfmt.toml` configures Rust formatting to use two spaces; run `make fmt` to format or `make fmt-check` to verify.

## Shared UI: Slint software renderer

```sh
cargo run --locked --release --features slint-ui --bin tomito-slint
# Separate locally signed macOS bundle, leaving native bundle unchanged:
./scripts/package-slint-macos.sh
open "dist/Tomito Portable.app"
```

Timer/settings/statistics use the same custom layout, colors and bundled Hack font on
all supported desktop backends. Slint 1.18.1 uses Winit and software rendering only;
no GPU renderer is enabled. Built-in scrolling style is pinned to Fluent in
`.cargo/config.toml`. Native window borders/title bars still differ by OS.
This is an implemented, measured prototype, not a claim of Windows/Linux runtime validation.

The shared variant has editable saved durations/options, five accent themes, timer
controls and overtime Finish, today/week statistics, CSV export and confirmed history
reset. On macOS it has tray controls, sounds, global shortcuts and sleep/wake hooks.
It also appears in the macOS Dock with its own stopwatch icon.
Closing its macOS window hides it; clicking the Dock icon or tray **Show timer** restores it.
Quit and reopen an already-running older instance to load the Dock fix. On Windows/Linux,
closing exits; hide-on-start/launch is ignored to avoid stranding a trayless app.
Tray/audio/global-shortcut/sleep integration is not implemented there. Screen-reader
support is not enabled in the current minimal feature configuration.

Local keys: Space/R/S/X/F, Escape (return to timer). Numeric settings are drafts until
**Apply and save**. Preview sound uses the selected draft. Configuration/history location
is shared with other variants by default; use `TOMITO_CONFIG_DIR` when comparing.

## Features

Focus/short/long breaks, adjustable cycles, automatic starts, pause/restart/skip/stop,
overtime Finish, native menu-bar countdown, global shortcuts, hide/restore,
keep-in-front, sleep/wake behavior, real macOS sounds, theme accents,
day/week statistics, seven-day chart, CSV export, confirmed reset.

Native settings apply with **Apply and save**, or when the Settings window closes.
CSV exports to `tomito-stats.csv` beside the activity log.
Settings and history live in `~/Library/Application Support/tomito-rs/` on macOS.
Set `TOMITO_CONFIG_DIR` to isolate development/testing data.

## Native shortcuts

- Space in the timer window: start/pause.
- Command+T/R/S/X: start/pause, restart, skip, stop.
- Command+1/2: show timer/statistics. Command+comma: settings. Command+Q: quit.
- Global Control+Option+Space/R/S/X/T: start/pause, restart, skip, stop, show timer.
- If another app owns a global shortcut, the timer reports the conflict; menu controls still work.

Optional egui: Space, R, S, X, F (overtime Finish), Escape (close panel).
Its macOS menu bar and system sounds work, but the fixed global shortcuts belong to the native default build.

## Validation

```sh
make fmt-check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --features egui-ui --all-targets
cargo clippy --features egui-ui --all-targets -- -D warnings
cargo test --features slint-ui --lib --bin tomito-slint
cargo clippy --features slint-ui --bin tomito-slint -- -D warnings
# Real Slint window: synthetic key/pointer events and persisted-state assertions
TOMITO_CONFIG_DIR="$(mktemp -d)" cargo run --release --features slint-ui --bin tomito-slint -- --smoke
# Actual 60-second expiry while hidden, restore and recorded activity:
TOMITO_CONFIG_DIR="$(mktemp -d)" cargo run --release --features slint-ui --bin tomito-slint -- --smoke-expiry
# Dock icon and actual Dock-click restoration (macOS Accessibility required):
./scripts/smoke-dock-macos.sh
# All builds finish before measurements begin:
./scripts/benchmark-ui-macos.sh --rounds 1 --samples 3
```

`scripts/smoke-macos.applescript <pid>` exercises a fresh native test instance with an isolated config directory.
It needs macOS Accessibility permission for the invoking automation host.

## Scope and performance

See [behavior inventory](docs/REVERSE_ENGINEERING.md) for evidence and explicit parity gaps.
See [measurements](docs/PERFORMANCE.md) for binary/bundle size, physical footprint, RSS, CPU and profiling commands.

This is not a complete pixel-identical or AppleScript-compatible replacement.
No original executable, icons, recordings or other proprietary assets are bundled.
