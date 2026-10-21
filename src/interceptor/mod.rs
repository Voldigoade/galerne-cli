//! Task interception, supervision, and signal routing.

pub mod history;
pub mod matcher;
pub mod process;
pub mod signal;
pub mod supervisor;
pub mod transition;

pub use history::DurationHistory;
pub use matcher::{match_command, CommandClass};
pub use process::{OutputBuffer, ProcessSupervisor};
pub use supervisor::execute_wrapped_task;
pub use transition::run_completion_transition;
