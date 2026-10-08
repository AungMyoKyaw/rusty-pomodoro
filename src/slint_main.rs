//! Shared software-rendered desktop UI; native and egui variants remain available.
use tomito_rs::{app, config, stats, timer};
#[cfg(target_os = "macos")]
mod hotkeys;
mod slint_native;
mod slint_shell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  slint_shell::run()
}
