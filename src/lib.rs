pub mod cli;
pub mod config;
pub mod core;
pub mod ui;
pub mod utils;
pub mod translation;
pub mod api;
pub mod wrapper;
pub mod auto_config;

#[cfg(feature = "self-update")]
pub mod updater;
