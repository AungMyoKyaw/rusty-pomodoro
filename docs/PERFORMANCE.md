# Size and memory measurements

The historical native/original measurements below remain unchanged. The newly
implemented Slint software-rendered shared UI has a separate, matched release-process
comparison in [SHARED_UI_BENCHMARK.md](SHARED_UI_BENCHMARK.md). It reduces memory versus
egui, but its executable is substantially larger; it is not a size optimization.

Measured locally on Apple Silicon, macOS 27.0.1 (26A434), Rust 1.99.0.
Source app: Tomito 2.3.5. These are measurements, not universal guarantees.

## Shipped build

- Cargo release executable: **372,032 bytes** (363 KiB).
- Executable inside the signed bundle: **369,984 bytes** (361 KiB); re-signing changes signature storage.
- Complete signed bundle: **372,965 logical file bytes**. Filesystem allocation: 372 KiB.
- Original installed bundle: **6,170,973 logical file bytes**.
- Bundle reduction: approximately **94%**.
- Original executable: 2,269,248 bytes, universal arm64+x86_64.
- Optional egui executable: **2,673,152 bytes**.

Our executable is arm64-only on this machine. Architecture and feature coverage differ;
these numbers do not establish full functionality parity.
The native bundle has no renderer, embedded font, audio file, webview or database runtime.

## Memory

Footprint comes from `vmmap -summary`. RSS comes from `ps` and includes resident shared code.
Footprint, RSS, virtual size and GPU allocations must not be conflated.

| Scenario | Rust native footprint | Installed Tomito footprint |
| --- | ---: | ---: |
| Main window, idle | 22.1 MiB | 32.5 MiB |
| Settings and statistics visited/open | 37.3 MiB | 46.8 MiB |
| Running with both panels open | 38.5 MiB | Not measured in this state |

- Native main-window RSS: 90,992 KiB.
- Native RSS with both panels: 116,832 KiB.
- Original main-window RSS: 110,944 KiB in its corresponding sample.
- Original RSS after opening settings/statistics: approximately 136,176 KiB.
- Native footprint peak during panel tests: 39.0 MiB.
- Optional egui: **76.3 MiB** idle footprint and **263.5 MiB** startup peak.
  An earlier egui sample measured 93.2 MiB footprint; GPU/driver state varies.

The native default reduced measured main-window footprint about 32% versus Tomito
and about 71% versus the final egui sample. It is not a sub-10-MiB desktop app.
Controls and audio load additional platform resources. Closed auxiliary windows are released.

Idle CPU samples reached 0.0%. A running sample immediately after interaction reported 2.2%;
that is not a steady-state CPU benchmark. No universal RAM or CPU ceiling is promised.

## Optimization choices

- AppKit is the default. egui is an opt-in Cargo feature.
- Native build has no GPU context, backbuffers, texture atlas, rasterizer or embedded fonts.
- Auxiliary windows are created lazily and released when closed.
- No polling or worker thread. One one-shot NSTimer exists only while running.
- Continuous monotonic time avoids wall-clock jumps; sleep counts when sleep pausing is disabled.
- Carbon shortcuts use callbacks, not a global keyboard tap.
- History uses fixed 8 KiB/256-byte scan buffers, including corrupt oversized lines.
- Fixed seven-day cache invalidates on append/reset. CSV export streams to a buffered file.
- Atomic settings rename; no serde, SQLite or async runtime.
- Release profile: size optimization, fat LTO, one codegen unit, panic abort, stripped symbols.
- Optional egui: glow, no default fonts/AccessKit/wgpu/Wayland/persistence, one licensed font,
  one pass, fixed-size timer typography, and display-boundary repaint scheduling.

## Reproduce

```sh
./scripts/package-macos.sh
open "dist/Tomito RS.app"
# Wait for startup activity to settle, then use the actual PID:
./scripts/measure-macos.sh <pid>
```

Use `TOMITO_CONFIG_DIR=/tmp/your-isolated-test-directory` for development benchmarks.
Keep window/panel state, audio, architecture, OS, power mode and competing workloads consistent.
Measure the shipped release binary, not a debug build.
