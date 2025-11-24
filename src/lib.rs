pub mod api;
pub mod auto_config;
pub mod cli;
pub mod config;
pub mod core;
pub mod ui;
pub mod utils;

#[cfg(feature = "self-update")]
pub mod updater;
