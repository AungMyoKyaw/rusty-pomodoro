//! Compatibility entry point for the default Slint desktop UI.
use rusty_pomodoro::{app, config, stats, timer};
#[cfg(target_os = "macos")]
mod hotkeys;
mod slint_native;
mod slint_shell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  slint_shell::run()
}
