use galerne::ui::cli::{parse_cli_action, CliAction};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[test]
fn saves_command_uses_the_selected_locale() {
    for (locale, expected) in [
        (
            "en",
            [
                "SAVES & ARCHIVES",
                "Save path:",
                "Backup path:",
                "Saved journeys: 1",
                "archive_123.dat",
                "4 bytes",
                "Restore an archive with",
            ],
        ),
        (
            "fr",
            [
                "SAUVEGARDES ET ARCHIVES",
                "Chemin de sauvegarde :",
                "Chemin de la copie de sauvegarde :",
                "Voyages archivés : 1",
                "archive_123.dat",
                "4 octets",
                "Restaurez une archive avec",
            ],
        ),
        (
            "es",
            [
                "PARTIDAS Y ARCHIVOS",
                "Ruta de guardado:",
                "Ruta de la copia de seguridad:",
                "Viajes guardados: 1",
                "archive_123.dat",
                "4 bytes",
                "Restaura un archivo con",
            ],
        ),
        (
            "zh-CN",
            [
                "存档与归档",
                "存档路径：",
                "备份存档路径：",
                "已保存旅途：1",
                "archive_123.dat",
                "4 字节",
                "使用 `galerne restore <archive>` 恢复归档。",
            ],
        ),
    ] {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let data_dir = std::env::temp_dir().join(format!(
            "galerne-saves-cli-{}-{locale}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&data_dir).expect("test data directory should be created");
        std::fs::write(data_dir.join("archive_123.dat"), b"test")
            .expect("archive listing fixture should be created");

        let output = Command::new(env!("CARGO_BIN_EXE_galerne"))
            .arg("saves")
            .env("GALERNE_LANG", locale)
            .env("GALERNE_DATA_DIR", &data_dir)
            .output()
            .expect("Galerne saves should run");
        let stdout = String::from_utf8(output.stdout).expect("command output should be UTF-8");
        let cleanup_result = std::fs::remove_dir_all(&data_dir);

        assert!(output.status.success());
        assert!(
            cleanup_result.is_ok(),
            "test data directory should be removed"
        );
        for text in expected {
            assert!(
                stdout.contains(text),
                "saves output should use locale {locale} and contain {text:?}: {stdout}"
            );
        }
    }
}

#[test]
fn saves_command_reports_archive_listing_errors() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let data_dir = std::env::temp_dir().join(format!(
        "galerne-saves-cli-error-{}-{nonce}",
        std::process::id()
    ));
    std::fs::write(&data_dir, b"not a directory").expect("blocking file should be created");

    let output = Command::new(env!("CARGO_BIN_EXE_galerne"))
        .arg("saves")
        .env("GALERNE_LANG", "fr")
        .env("GALERNE_DATA_DIR", &data_dir)
        .output()
        .expect("Galerne saves should run");
    let stderr = String::from_utf8(output.stderr).expect("command output should be UTF-8");
    let cleanup_result = std::fs::remove_file(&data_dir);

    assert!(
        !output.status.success(),
        "listing failure should return an error"
    );
    assert!(
        stderr.contains("Impossible de lister les archives"),
        "the localized error should explain the failure: {stderr}"
    );
    assert!(cleanup_result.is_ok(), "blocking file should be removed");
}
