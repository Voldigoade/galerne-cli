use galerne::ui::cli::{parse_cli_action, CliAction};

#[test]
fn test_cli_argument_parsing() {
    // Bare invocation -> Interactive Dashboard
    let args = vec!["galerne".to_string()];
    assert_eq!(parse_cli_action(&args), CliAction::Dashboard);

    // Subcommands
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "play".to_string()]),
        CliAction::Play
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "stats".to_string()]),
        CliAction::Stats
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "doctor".to_string()]),
        CliAction::Doctor
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "saves".to_string()]),
        CliAction::Saves
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "reset".to_string()]),
        CliAction::Reset { force: false }
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "reset".to_string(), "-y".to_string()]),
        CliAction::Reset { force: true }
    );
    assert_eq!(
        parse_cli_action(&[
            "galerne".to_string(),
            "reset".to_string(),
            "--yes".to_string()
        ]),
        CliAction::Reset { force: true }
    );
    assert_eq!(
        parse_cli_action(&[
            "galerne".to_string(),
            "restore".to_string(),
            "archive_12345.dat".to_string()
        ]),
        CliAction::Restore("archive_12345.dat".to_string())
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "setup".to_string()]),
        CliAction::Setup
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "enable".to_string()]),
        CliAction::Enable
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "disable".to_string()]),
        CliAction::Disable
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "uninstall".to_string()]),
        CliAction::Uninstall
    );
    assert_eq!(
        parse_cli_action(&["galerne".to_string(), "help".to_string()]),
        CliAction::Help
    );
}

#[test]
fn test_cli_wrap_argument_parsing() {
    let args = vec![
        "galerne".to_string(),
        "wrap".to_string(),
        "cargo".to_string(),
        "build".to_string(),
        "--release".to_string(),
    ];

    match parse_cli_action(&args) {
        CliAction::Wrap(cmd) => {
            assert_eq!(cmd, vec!["cargo", "build", "--release"]);
        }
        other => panic!("Expected CliAction::Wrap, got {:?}", other),
    }
}
