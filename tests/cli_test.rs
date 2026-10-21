use galerne::storage::manager::SaveManager;
use galerne::storage::schema::SaveState;
use galerne::ui::cli::{parse_cli_action, CliAction};
use std::io::Write;
use std::process::{Command, Stdio};
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
        let data_dir = unique_data_dir(&format!("galerne-saves-cli-{locale}"));
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
    let data_dir = unique_data_dir("galerne-saves-cli-error");
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

#[test]
fn reset_command_uses_locale_specific_confirmation_and_keeps_the_archive() {
    for (locale, affirmative, expected) in [
        (
            "en",
            "y",
            [
                "Archive current progress and start a new journey? [y/N]:",
                "Preparing the new journey",
                "Previous save archived to:",
                "New journey started.",
            ],
        ),
        (
            "fr",
            "o",
            [
                "Archiver la progression et commencer un nouveau voyage ? [o/N] :",
                "Préparation du nouveau voyage",
                "Ancienne sauvegarde archivée ici :",
                "Nouveau voyage commencé.",
            ],
        ),
        (
            "es",
            "sí",
            [
                "¿Archivar el progreso y comenzar un nuevo viaje? [s/N]:",
                "Preparando el nuevo viaje",
                "Partida anterior archivada en:",
                "Nuevo viaje iniciado.",
            ],
        ),
        (
            "zh-CN",
            "是",
            [
                "归档当前进度并开始新的旅途？[是/否]：",
                "正在准备新的旅途",
                "旧存档已归档至：",
                "新的旅途已开始。",
            ],
        ),
    ] {
        let data_dir = unique_data_dir("galerne-reset-cli");
        let saves = SaveManager::with_dir(&data_dir);
        saves
            .save(&SaveState::default())
            .expect("test save should be created");

        let mut child = Command::new(env!("CARGO_BIN_EXE_galerne"))
            .arg("reset")
            .env("GALERNE_LANG", locale)
            .env("GALERNE_DATA_DIR", &data_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Galerne reset should run");
        writeln!(child.stdin.as_mut().unwrap(), "{affirmative}")
            .expect("confirmation should reach the command");
        let output = child
            .wait_with_output()
            .expect("Galerne reset should finish");
        let stdout = String::from_utf8(output.stdout).expect("command output should be UTF-8");
        let archive_count = saves
            .list_archives()
            .expect("archives should remain listable")
            .len();
        let cleanup_result = std::fs::remove_dir_all(&data_dir);

        assert!(output.status.success(), "reset should succeed: {stdout}");
        assert_eq!(archive_count, 1, "the previous save should be archived");
        assert!(
            cleanup_result.is_ok(),
            "test data directory should be removed"
        );
        for text in expected {
            assert!(
                stdout.contains(text),
                "reset output should use locale {locale} and contain {text:?}: {stdout}"
            );
        }
    }
}

#[test]
fn restore_command_uses_the_selected_locale() {
    for (locale, expected) in [
        ("en", "Restored archive_"),
        ("fr", "Archive restaurée : archive_"),
        ("es", "Archivo restaurado: archive_"),
        ("zh-CN", "已恢复归档 archive_"),
    ] {
        let data_dir = unique_data_dir("galerne-restore-cli");
        let saves = SaveManager::with_dir(&data_dir);
        saves
            .save(&SaveState::default())
            .expect("test save should be created");
        let archive_name = saves
            .archive_and_reset()
            .expect("test save should be archived")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let output = Command::new(env!("CARGO_BIN_EXE_galerne"))
            .args(["restore", &archive_name])
            .env("GALERNE_LANG", locale)
            .env("GALERNE_DATA_DIR", &data_dir)
            .output()
            .expect("Galerne restore should run");
        let stdout = String::from_utf8(output.stdout).expect("command output should be UTF-8");
        let cleanup_result = std::fs::remove_dir_all(&data_dir);

        assert!(output.status.success(), "restore should succeed: {stdout}");
        assert!(
            stdout.contains(expected),
            "restore should use {locale}: {stdout}"
        );
        assert!(
            stdout.contains(&archive_name),
            "restore output should identify the archive: {stdout}"
        );
        assert!(
            cleanup_result.is_ok(),
            "test data directory should be removed"
        );
    }
}

#[test]
fn reset_without_a_previous_save_does_not_claim_to_archive_one() {
    let data_dir = unique_data_dir("galerne-reset-empty");
    let output = Command::new(env!("CARGO_BIN_EXE_galerne"))
        .args(["reset", "--yes"])
        .env("GALERNE_LANG", "en")
        .env("GALERNE_DATA_DIR", &data_dir)
        .output()
        .expect("Galerne reset should run");
    let stdout = String::from_utf8(output.stdout).expect("command output should be UTF-8");
    let saves = SaveManager::with_dir(&data_dir);
    let archive_count = saves
        .list_archives()
        .expect("archives should remain listable")
        .len();
    let cleanup_result = std::fs::remove_dir_all(&data_dir);

    assert!(output.status.success(), "reset should succeed: {stdout}");
    assert!(
        stdout.contains("New journey started."),
        "reset should confirm the new journey: {stdout}"
    );
    assert!(
        !stdout.contains("Previous save archived to:"),
        "reset should not claim to archive a missing save: {stdout}"
    );
    assert_eq!(
        archive_count, 0,
        "no archive should be created without a save"
    );
    assert!(
        cleanup_result.is_ok(),
        "test data directory should be removed"
    );
}

fn unique_data_dir(prefix: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()))
}
