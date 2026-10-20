//! Task interception, supervision, and signal routing.

pub mod history;
pub mod matcher;

pub use history::DurationHistory;
pub use matcher::{match_command, CommandClass};
