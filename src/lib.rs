pub mod api;
pub mod auto_config;
pub mod cli;
pub mod config;
pub mod core;
pub mod translation;
pub mod ui;
pub mod utils;
pub mod wrapper;

#[cfg(feature = "self-update")]
pub mod updater;
