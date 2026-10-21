//! Galerne — Ultra-lightweight persistent terminal game designed for developer wait times.

pub mod game;
pub mod interceptor;
pub mod shell;
pub mod storage;
pub mod terminal;
pub mod ui;
pub mod updater;

/// Application version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
