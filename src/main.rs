use tomito_rs::{app, config, stats, timer};
#[cfg(feature = "egui-ui")]
mod egui_shell;
#[cfg(all(target_os = "macos", not(feature = "egui-ui")))]
mod hotkeys;
#[cfg(all(target_os = "macos", not(feature = "egui-ui")))]
mod mac_shell;
#[cfg(feature = "egui-ui")]
mod native;
#[cfg(feature = "egui-ui")]
use tomito_rs::theme;
#[cfg(feature = "egui-ui")]
mod ui;

#[cfg(feature = "egui-ui")]
fn main() -> eframe::Result {
  egui_shell::run()
}
#[cfg(all(target_os = "macos", not(feature = "egui-ui")))]
fn main() {
  mac_shell::run();
}
#[cfg(all(not(target_os = "macos"), not(feature = "egui-ui")))]
compile_error!("Windows/Linux: use --features egui-ui or --features slint-ui --bin tomito-slint.");
