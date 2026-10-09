# Rusty Pomodoro

<!-- impeccable:product-schema 1 -->

## Platform

web

This record scopes the product website, not the native desktop implementation.

## Stack

Assumption for this task: static HTML, CSS, and JavaScript, deployed through GitHub Pages. No framework or build step is needed.

## Users

Inferred audience: people working at a desktop who want focus/break structure without a browser-based workspace. This is an implementation assumption, not a confirmed user preference.

## Product Purpose

Introduce Rusty Pomodoro, demonstrate its real interface, and help visitors run the open-source desktop app.

## Capabilities and Constraints

Repository evidence: focus, short-break and long-break timers; configurable cycles and durations; five accent themes; day/week statistics and CSV export. Slint is the default desktop UI on macOS, Windows and Linux. macOS system integrations use platform APIs; the timer, settings and statistics use Slint. Windows/Linux runtime behavior is not tested; their builds lack macOS-only tray, sounds, global shortcuts and sleep hooks. macOS bundles are ad-hoc signed and not notarized. No published GitHub releases were present during website creation. The website must not invent binary downloads, adoption counts, testimonials or performance guarantees.

## Brand Commitments

Preserve the Rusty Pomodoro name and real application icon and screenshots. AGPL-3.0-or-later license.

## Evidence on Hand

`README.md`, `docs/DEVELOPMENT.md`, `docs/PERFORMANCE.md`, `assets/screenshots/`, `Cargo.toml`. Screenshot statistics show an empty history, not fabricated customer activity.

## Product Principles

- Demonstrate the actual application.
- Keep installation claims honest and platform-specific.
- Make source and documentation easy to reach.
