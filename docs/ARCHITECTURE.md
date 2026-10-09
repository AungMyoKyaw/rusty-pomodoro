# Current architecture and UI review

This guide describes the checked-out repository. For behavior observed in the installed Tomito app and explicit parity gaps, see [REVERSE_ENGINEERING.md](REVERSE_ENGINEERING.md). Historical macOS measurements and their limits remain in [PERFORMANCE.md](PERFORMANCE.md) and [SHARED_UI_BENCHMARK.md](SHARED_UI_BENCHMARK.md).

## Architecture

### Shared core

- `src/timer.rs`: monotonic-clock timer state machine; Session, Short Break, Long Break; pause, overtime and cycle rules.
- `src/config.rs`: bounded settings, plain-text `key=value` persistence and legacy `TOMITO_*` environment aliases.
- `src/stats.rs`: append-only activity history, fixed seven-day cache, local-date totals and streaming CSV export.
- `src/app.rs`: application orchestration, settings and timer effects. It does not draw UI or call platform APIs.
- `src/lib.rs`: shared modules. `theme.rs` is compiled only for egui.

### Desktop frontends

| Entry point | UI | Platform role |
| --- | --- | --- |
| `src/main.rs` | Default Slint UI; optional egui via `egui-ui` | Shared software-rendered desktop UI; macOS tray, Dock, sounds, shortcuts and sleep/wake |
| `src/slint_main.rs` | Compatibility binary for the same Slint UI | Preserves existing Slint commands and release executable names |
| `src/egui_shell.rs` | Optional eframe/egui UI | AppKit status item and native sound on macOS |

Default builds enable `slint-ui` and use Winit with the software renderer on every desktop platform. Only macOS runtime behavior has been validated. Slint draws the timer, settings, and statistics; platform APIs still provide macOS system integrations. `src/mac_shell.rs` is retired source and is not compiled by any entry point. Slint has a settings draft/validation flow; optional egui settings apply as edited.

### Data flow

Frontends translate user and operating-system events into `App` operations. They read display state and consume effect flags such as hide/show, notification, tick and exit. Timer rules remain in the shared core. Settings live in `settings.conf`; activity rows live in `activities.csv`, both under the app configuration directory unless `RUSTY_POMODORO_CONFIG_DIR` overrides it.

## UI/UX review

Reviewed real software-renderer screenshots from `--smoke-visual` at 420×560, 380×560 and 640×720 logical sizes. Inspected idle/running timer, duration/appearance settings, empty/populated statistics and reset confirmation. Captures used isolated temporary config/history.

### Findings and change

- Timer’s visible keyboard guide listed Space, R and S, but omitted supported X (Stop) and F (Finish during overtime). This hid useful controls and made the guide incomplete.
- Updated the timer guide to show all five timer shortcuts across two readable lines. Timer hierarchy, warm-dark palette and existing navigation stay intact.
- Settings expose field hints, unsaved-draft state, inline validation, and Apply/Discard actions. Statistics show empty-state guidance, seven-day comparison and an explicit reset confirmation. Preserve these patterns.

### Validation scope

`--smoke-visual` captures real Slint windows across three sizes and seven states per size; it does not validate every OS compositor, screen reader, high-DPI scale or other frontend. Slint accessibility support is enabled and UI controls carry labels/roles, but screen-reader behavior has not been independently validated.
