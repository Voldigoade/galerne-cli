//! Command-line argument parsing and command action classification.

use crate::ui::i18n::{t, Locale};

/// Intended application action dispatched from CLI arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliAction {
    Dashboard,
    Play,
    Prototype,
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
        "prototype" => CliAction::Prototype,
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

pub fn print_help(locale: Locale) {
    print!("{}", help_text(locale));
}

fn help_text(locale: Locale) -> String {
    let commands = [
        ("galerne", "cli_dashboard"),
        ("galerne play", "cli_play"),
        ("galerne prototype", "cli_prototype"),
        ("galerne stats", "cli_stats"),
        ("galerne doctor", "cli_doctor"),
        ("galerne saves", "cli_saves"),
        ("galerne reset [-y|--yes]", "cli_reset"),
        ("galerne restore <archive>", "cli_restore"),
        ("galerne setup", "cli_setup"),
        ("galerne enable", "cli_enable"),
        ("galerne disable", "cli_disable"),
        ("galerne uninstall", "cli_uninstall"),
        ("galerne update", "cli_update"),
        ("galerne update --check", "cli_update_check"),
        ("galerne wrap <command...>", "cli_wrap"),
        ("galerne help", "cli_help"),
        ("galerne --version", "cli_version"),
    ];
    let mut output = format!(
        "Galerne {} — {}\n\n{}",
        crate::VERSION,
        t("cli_tagline", locale),
        t("cli_usage", locale)
    );

    for (command, description) in commands {
        output.push_str(&format!("\n  {command:<30} {}", t(description, locale)));
    }
    output.push('\n');
    output
}
