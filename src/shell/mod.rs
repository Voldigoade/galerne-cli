//! Shell integration and lifecycle management.

pub mod doctor;
pub mod installer;

pub use doctor::run_doctor;
pub use installer::{detect_shell_configs, setup_all_shells, uninstall_all_shells, ShellType};
