use galerne::ui::cli::{parse_cli_action, CliAction};
use std::process::Command;

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
        parse_cli_action(&["galerne".to_string(), "update".to_string()]),
        CliAction::Update { check_only: false }
    );
    assert_eq!(
        parse_cli_action(&[
            "galerne".to_string(),
            "update".to_string(),
            "--check".to_string()
        ]),
        CliAction::Update { check_only: true }
    );
    assert_eq!(
        parse_cli_action(&[
            "galerne".to_string(),
            "update".to_string(),
            "-c".to_string()
        ]),
        CliAction::Update { check_only: true }
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

#[test]
fn command_line_help_uses_the_selected_locale() {
    for (locale, expected) in [
        ("fr", "Commencer un voyage manuel"),
        ("es", "Iniciar un viaje manual"),
        ("zh-CN", "开始手动旅程"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_galerne"))
            .arg("help")
            .env("GALERNE_LANG", locale)
            .output()
            .expect("Galerne help should run");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
        assert!(
            stdout.contains(expected),
            "help should use locale {locale}: {stdout}"
        );
    }
}
