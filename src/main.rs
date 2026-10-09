//! Slint is the default desktop UI; egui is an explicit alternative.
use rusty_pomodoro::{app, config, stats, timer};
#[cfg(feature = "egui-ui")]
mod egui_shell;
#[cfg(all(target_os = "macos", feature = "slint-ui", not(feature = "egui-ui")))]
mod hotkeys;
#[cfg(feature = "egui-ui")]
mod native;
#[cfg(all(feature = "slint-ui", not(feature = "egui-ui")))]
mod slint_native;
#[cfg(all(feature = "slint-ui", not(feature = "egui-ui")))]
mod slint_shell;
#[cfg(feature = "egui-ui")]
use rusty_pomodoro::theme;
#[cfg(feature = "egui-ui")]
mod ui;

#[cfg(feature = "egui-ui")]
fn main() -> eframe::Result {
  egui_shell::run()
}
#[cfg(all(feature = "slint-ui", not(feature = "egui-ui")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
  slint_shell::run()
}
#[cfg(not(any(feature = "slint-ui", feature = "egui-ui")))]
compile_error!("Enable the default Slint UI or select --features slint-ui or --features egui-ui.");
