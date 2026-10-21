//! Task interception, supervision, and signal routing.

pub mod history;
pub mod matcher;
pub mod process;
pub mod project;
pub mod signal;
pub mod supervisor;
pub mod transition;

pub use history::DurationHistory;
pub use matcher::{match_command, CommandClass};
pub use process::{OutputBuffer, ProcessSupervisor};
pub use project::{
    clean_canonicalize, derive_project_id, find_project_root, get_or_create_salt,
    resolve_current_project_id,
};
pub use supervisor::execute_wrapped_task;
pub use transition::run_completion_transition;
