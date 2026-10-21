//! Command-line argument parsing and command action classification.

/// Intended application action dispatched from CLI arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliAction {
    Dashboard,
    Play,
    Stats,
    Doctor,
    Saves,
    Reset,
    Setup,
    Enable,
    Disable,
    Uninstall,
    Wrap(Vec<String>),
    Help,
    Version,
}

/// Parses command line arguments into a high-level application action.
pub fn parse_cli_action(args: &[String]) -> CliAction {
    if args.len() <= 1 {
        return CliAction::Dashboard;
    }

    let cmd = args[1].to_lowercase();
    match cmd.as_str() {
        "play" => CliAction::Play,
        "stats" => CliAction::Stats,
        "doctor" => CliAction::Doctor,
        "saves" => CliAction::Saves,
        "reset" => CliAction::Reset,
        "setup" => CliAction::Setup,
        "enable" => CliAction::Enable,
        "disable" => CliAction::Disable,
        "uninstall" => CliAction::Uninstall,
        "wrap" => {
            if args.len() > 2 {
                CliAction::Wrap(args[2..].to_vec())
            } else {
                CliAction::Help
            }
        }
        "help" | "--help" | "-h" => CliAction::Help,
        "version" | "--version" | "-V" => CliAction::Version,
        _ => {
            // If unknown argument starts with `--` or `-`, show help
            if cmd.starts_with('-') {
                CliAction::Help
            } else {
                // Treat unknown direct invocation as an implicit wrap target
                CliAction::Wrap(args[1..].to_vec())
            }
        }
    }
}

pub fn print_help() {
    println!(
        "Galerne {} — Ultra-lightweight persistent terminal game for developer wait times\n\n\
        USAGE:\n  \
          galerne                     Open interactive control center\n  \
          galerne play                Launch immediate manual gameplay session\n  \
          galerne stats               Display career progression and wait time saved\n  \
          galerne doctor              Run read-only system diagnostics\n  \
          galerne saves               Inspect active saves and archived backups\n  \
          galerne reset               Archive current progress and initialize fresh state\n  \
          galerne setup               Configure shell integration (Bash, Zsh, PowerShell)\n  \
          galerne enable              Enable automatic wait-time game activation\n  \
          galerne disable             Disable automatic game activation\n  \
          galerne uninstall           Safely remove shell hooks and application state\n  \
          galerne wrap <command...>   Explicitly execute and supervise a development command\n  \
          galerne help                Display this help overview\n",
        crate::VERSION
    );
}
