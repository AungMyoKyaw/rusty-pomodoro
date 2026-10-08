# Tomito behavior inventory

Inspected installation: `/Applications/Tomito.app`, version 2.3.5 (38).
This is a behavior-oriented independent implementation, not recovered source code.
The installed app and its saved preferences were not patched or imported.

## Evidence

- `Contents/Info.plist`: menu-bar/accessory application, AppleScript enabled, macOS 10.15 minimum, universal x86_64/arm64 executable.
- `Resources/Tomito.sdef`: start, stop, pause, resume, skip, restart, finish, show, hide, toggle, toggleTimerWidget, quit, getRemainingTime, getCurrentActivityType.
- General settings NIB: session/short/long durations, 2–10-session long-break cycle, automatic starts, break disabling, completed-only statistics, overtime, hide on launch/start, show on finish, keep in front.
- Advanced settings NIB: pause on sleep, resume on wake, prevent sleep, reset counter after 30 minutes of inactivity.
- Appearance/sounds/shortcut resources and executable symbol strings: themes, status-item appearance, configurable shortcuts, completion and ticking sounds, volume controls.
- Live UI: a compact timer, activity label, session counter, start/action controls, day/week statistics and calendar navigation.
- Live AppleScript queries returned a session with 45:00 remaining in the user's installed configuration. This does not establish the app's factory defaults.

No proprietary executable, font, icon, theme asset, or sound file is included in this project.

## Implemented

- Configurable focus, short break, long break, 2–10-session cycle.
- Start/pause/resume/restart/skip/stop, manual overtime finish, reset session counter.
- Automatic activity starts, disabling all breaks or only long breaks.
- Native menu-bar countdown and controls. Close hides the timer without stopping it; restore from the menu bar or global shortcut.
- Hide on launch/start, show on completion, keep in front.
- Pause/resume on Mac sleep/wake; a continuous monotonic clock includes sleep when pause-on-sleep is disabled.
- Native system completion sounds and optional per-second ticking, muted during breaks.
- Theme accent choices and native dark appearance.
- Today and last-seven-day totals, seven-day focus bars, completed-only filtering, streaming CSV export, confirmed reset.
- Atomic plain-text settings writes, append-only history; history is not imported from Tomito.
- Native app-local shortcuts and five fixed global shortcuts.

## Differences and remaining parity

- Rust factory durations are 25/5/15 minutes, not the observed installed 45-minute setting.
- Native theme choices change the activity accent, not every AppKit control. The optional egui UI has five full palettes.
- Completion audio uses macOS sounds, not Tomito's bundled recordings. There are no custom recordings or volume sliders.
- Global shortcuts are fixed Control+Option combinations, not user-rebindable.
- No AppleScript suite, compact secondary timer widget, calendar date navigation, launch-at-login settings, sleep-prevention assertion, automatic inactivity counter reset, or notification-center banners.
- Statistics assign an activity to its completion/skip date in the current local timezone on macOS/Linux. CSV preserves the original epoch timestamp. Windows currently uses UTC grouping.
- Timer progress/counter are not restored across application exits.
- Native views are intentionally compact and not pixel-identical to the original. macOS is the tested target; egui provides a build path for other desktops.

## Runtime validation

The native app was launched and its real controls exercised through macOS Accessibility:
global start/pause/restart/skip/stop, close-to-hide and global restore,
settings clamping/save, statistics and five-column CSV export.
A real one-minute run reached overtime and recorded 108 seconds after Finish.
A separate hidden one-minute break completed and wrote a completed 60-second entry;
the next session started automatically while the main window remained hidden.
Sleep transitions are unit-tested without putting the user's Mac to sleep.
See `PERFORMANCE.md` for measurements and limits.
