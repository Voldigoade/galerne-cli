//! Command-line argument parsing and command action classification.

/// Intended application action dispatched from CLI arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliAction {
    Dashboard,
    Play,
    Stats,
    Doctor,
    Saves,
    Reset { force: bool },
    Restore(String),
    Setup,
    Enable,
    Disable,
    Uninstall,
    Wrap(Vec<String>),
    Update { check_only: bool },
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
        "reset" => {
            let force = args
                .iter()
                .skip(2)
                .any(|a| a == "-y" || a == "--yes" || a == "-f" || a == "--force");
            CliAction::Reset { force }
        }
        "restore" => {
            if args.len() > 2 {
                CliAction::Restore(args[2].clone())
            } else {
                CliAction::Help
            }
        }
        "setup" => CliAction::Setup,
        "enable" => CliAction::Enable,
        "disable" => CliAction::Disable,
        "uninstall" => CliAction::Uninstall,
        "update" => {
            let check_only = args.iter().skip(2).any(|a| a == "--check" || a == "-c");
            CliAction::Update { check_only }
        }
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
        "Galerne {} — Native terminal game for developer wait times\n\n\
        USAGE:\n  \
          galerne                     Open the journey dashboard\n  \
          galerne play                Begin a manual journey\n  \
          galerne stats               Display career progression and wait time saved\n  \
          galerne doctor              Check terminal, saves, and shell integration\n  \
          galerne saves               Inspect active saves and archived backups\n  \
          galerne reset [-y|--yes]    Archive current progress and initialize fresh state\n  \
          galerne restore <archive>   Restore a previous archived save snapshot\n  \
          galerne setup               Configure shell integration (Bash, Zsh, PowerShell)\n  \
          galerne enable              Enable automatic wait-time game activation\n  \
          galerne disable             Disable automatic game activation\n  \
          galerne uninstall           Safely remove shell hooks and application state\n  \
          galerne update              Check for and safely install the latest release\n  \
          galerne update --check      Check release availability without modifying installation\n  \
          galerne wrap <command...>   Explicitly execute and supervise a development command\n  \
          galerne help                Display this help overview\n",
        crate::VERSION
    );
}
