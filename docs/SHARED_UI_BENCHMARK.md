# Shared UI implementation and measured comparison

## Dock-icon revision

The current bundle now uses regular Dock activation, an original stopwatch icon and
Dock-click restoration of a hidden timer, while retaining the menu-bar timer.
**The measurements and hashes below describe the pre-Dock-icon revision.** The new
icon resources and integration change executable/bundle identity and size. No new RAM
comparison or unchanged performance claim is made for this revision.

## Result

The Slint 1.18.1 software-rendered variant is implemented, packaged and measured on
Apple Silicon/macOS 27.0.1 (26A434), Rust 1.99.0.

**Slint lowered idle physical footprint 58% versus egui, but its executable is 2.87x
larger. Native AppKit remains both the smallest and lowest-memory option.**
The native default was retained. This is a shared-UI alternative, not a successful
binary-size reduction or a full Tomito parity claim.

## Release executable sizes

| Variant | Exact Cargo release bytes | MiB |
| --- | ---: | ---: |
| Rust AppKit native | 372,048 | 0.355 |
| egui/glow | 2,689,680 | 2.565 |
| Slint/Winit/software | 7,708,256 | 7.351 |

Separately ad-hoc signed Slint artifact: `dist/Rusty Pomodoro Slint.app`.
Its executable is 7,663,568 bytes; complete bundle is 7,670,561 logical file bytes.
Signing changes the Mach-O signature storage, explaining the difference from Cargo.
The benchmark below used the Cargo release executable, not the re-signed copy.
Both passed real-window interaction smoke checks.

The original installed Tomito bundle previously measured 6,170,973 logical bytes.
Therefore this Slint bundle is about 24% larger, not smaller. Original Tomito memory
results in PERFORMANCE.md are historical measurements, not matched samples in this run.
Native, original and Slint functionality/architecture coverage differ.

## Memory and CPU

One fresh process per variant/scenario, three samples each. Values are medians except
peak, which is the maximum lifetime peak reported by vmmap for that process.
Footprint/RSS/peak are MiB. CPU is accumulated process CPU-time delta divided by actual
wall interval, expressed as a percentage of one core.

| Variant/scenario | Physical footprint | RSS | Peak footprint | CPU % |
| --- | ---: | ---: | ---: | ---: |
| Native idle | 19.5 | 83.3 | 20.0 | 0.00 |
| Native running | 21.8 | 85.8 | 22.0 | 0.53 |
| Native statistics | 21.5 | 83.7 | 22.0 | 0.00 |
| egui idle | 81.0 | 133.0 | 273.0 | 0.00 |
| egui running | 81.2 | 133.4 | 273.2 | 1.72 |
| egui statistics | 79.4 | 132.7 | 268.9 | 0.00 |
| Slint idle | 34.3 | 105.0 | 37.6 | 0.00 |
| Slint running | 34.5 | 105.6 | 37.7 | 0.89 |
| Slint statistics | 34.0 | 105.0 | 37.3 | 0.00 |

Slint's idle footprint is 42% of egui's, but 76% above native.
Idle peak fell from 273.0 to 37.6 MiB. RSS is different accounting: do not call the
34.3 MiB footprint an RSS result. The CPU samples do not establish universal ceilings.

### Conditions and limits

- All compilation completed before measurements. Release: size optimization, fat LTO,
  one codegen unit, stripped symbols and panic abort.
- Warm filesystem/shared-library caches, factory settings, empty history, serial launches.
  Five-second settling delay, nominal five-second intervals plus profiling overhead.
- Windows were on screen without requesting foreground activation. The desktop remained
  in use; other applications could obscure them. These are not dedicated-machine,
  continuously foreground-rendered or controlled compositor benchmarks.
- Benchmark windows ignore mouse input; menu/global/local timer actions are isolated.
  Scenario assertions abort rather than mislabel externally changed timer/panel state.
- Slint and egui have 400x460 logical content size. Native main is 380x300; its statistics
  window is a separate 440x440 window. Native comparison therefore includes different UI
  architecture and pixel area; it is not a pure renderer-only A/B.
- Statistics opened at startup; **settings were not part of this measured scenario**.
  It is not the earlier “both auxiliary panels open” measurement.
- Only one final process per scenario: three samples are correlated, not independent
  repeated trials. Background workload, driver allocations, visibility and startup
  conditions affect results. An earlier exploratory egui idle sample reached 231.3 MiB.
- Physical footprint and process RSS include different shared/graphics accounting.
  VMmap's peak is lifetime peak, not an independently sampled startup time series.
- A first exploratory two-round run and an externally interrupted run are retained under
  `dist/benchmarks-exploratory` and `dist/benchmarks-interrupted`; neither supplies this table.
- Raw final data, exact executable SHA-256 identities, process IDs, CPU intervals and
  every vmmap output: `dist/benchmarks/results.json` and sibling files.
  Hashes were checked against the final release files after measuring.

## Implemented behavior and platform status

Common Rust state lives in src/lib.rs; all frontends reuse timer/config/history logic.
The Slint application provides timer controls, overtime Finish, duration/cycle editing,
persisted options, five accent themes, today/week statistics, seven-day chart, CSV export
and confirmed history reset. Custom palette/layout/Hack font are shared; built-in
scrolling uses pinned Fluent styling and dark color scheme. Native title bars and DPI
behavior are still OS-specific.

macOS integrations: tray countdown/control menu, system sound/preview, global shortcuts,
sleep/wake behavior, hide/restore and keep-in-front. Other-platform integration modules
are stubs. Windows/Linux close exits; hide-on-launch/start is ignored there to avoid
stranding a trayless application. Screen-reader integration is not enabled.

**Only macOS was built and run here. Windows/Linux builds and runtime remain unverified.**
No cross-platform feature-parity or pixel-identical result is claimed.

## Validation

- Shared/native tests: 28; egui configuration: 30; Slint configuration: 30.
- Combined-feature suite: 32 tests passed.
- Strict clippy passed for default, egui-only, Slint-only and combined-feature targets.
- Formatting passed; release builds, plist lint and bundle signature verification passed.
- Real Slint window smoke exercised keyboard and pointer navigation, start/pause/restart/
  skip/stop, duration clamping, save/reload, theme/sound/option callbacks, statistics,
  CSV export, reset and close/restore. Both release and signed bundle smoke passed.
- The final signed artifact passed a real 60-second hidden timer expiry: window restored,
  short break selected and exactly 60 focus seconds recorded. Its SHA-256 matched before
  and after the check: `5cc530fa49d5e187387e60cf3777a44a23854842ab5d0f9bbf8e364a8299b7e1`.
- Window-only captures exposed a clipped statistics footer; the chart became scrollable
  and summary spacing was reduced before final packaging/measurement.
- Minimal-feature dependency audit confirmed software renderer/softbuffer and no enabled
  wgpu, femtovg, glutin, eframe, egui or Skia GPU renderer. BackendSelector explicitly
  forces Winit/software; SLINT_BACKEND cannot substitute another renderer.

## Reproduce

Run from the project directory:

```sh
cargo run --locked --release --features slint-ui --bin rusty-pomodoro-slint
./scripts/package-slint-macos.sh
open "dist/Rusty Pomodoro Slint.app"
./scripts/benchmark-ui-macos.sh --rounds 1 --samples 3
```

For repeated trials, use `--rounds 2` or higher. The script only terminates its own children
and uses temporary settings/history directories. It does not replace the native bundle
or stop the existing user processes.

```sh
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" cargo run --release --features slint-ui --bin rusty-pomodoro-slint -- --smoke
RUSTY_POMODORO_CONFIG_DIR="$(mktemp -d)" cargo run --release --features slint-ui --bin rusty-pomodoro-slint -- --smoke-expiry
```

Smoke modes refuse an unset or nonempty test directory, preventing accidental history
reset in ordinary user data.
