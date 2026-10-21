//! Interactive dashboard and menus.

pub mod cli;
pub mod dashboard;

pub use cli::{parse_cli_action, print_help, CliAction};
pub use dashboard::run_dashboard;
