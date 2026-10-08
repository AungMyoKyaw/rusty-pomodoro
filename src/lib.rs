//! Shared application, persistence and timer logic for all desktop shells.
pub mod app;
pub mod config;
pub mod stats;
#[cfg(feature = "egui-ui")]
pub mod theme;
pub mod timer;
