# Rusty Pomodoro

A Rust Pomodoro desktop app rebuilt from the behavior of the installed Tomito app.

The default macOS app uses native AppKit. Windows and Linux use the shared Slint UI; those platforms build, but runtime behavior is not tested.

## Quick start

Run the shared UI locally:

```sh
make dev
```

Build the native macOS app and open it:

```sh
make package-macos
open "dist/Rusty Pomodoro.app"
```

The app includes focus and break timers, configurable cycles, themes, shortcuts, and day/week statistics with CSV export. See [Development](docs/DEVELOPMENT.md) for build targets, platform details, and validation.

## Project docs

- [Architecture and UX review](docs/ARCHITECTURE.md)
- [Behavior inventory and parity gaps](docs/REVERSE_ENGINEERING.md)
- [Performance measurements](docs/PERFORMANCE.md)

This is not a complete pixel-identical or AppleScript-compatible replacement. No original executable or proprietary assets are included.
