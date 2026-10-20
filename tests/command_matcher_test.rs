use galerne::interceptor::history::DurationHistory;
use galerne::interceptor::matcher::{match_command, CommandClass};

#[test]
fn test_eligible_command_matching() {
    // Cargo build
    let cmd = vec![
        "cargo".to_string(),
        "build".to_string(),
        "--release".to_string(),
    ];
    let matched = match_command(&cmd);
    assert_eq!(matched, Some(CommandClass::CargoBuild));

    // Cargo test
    let cmd = vec!["cargo".to_string(), "test".to_string()];
    assert_eq!(match_command(&cmd), Some(CommandClass::CargoTest));

    // NPM install
    let cmd = vec!["npm".to_string(), "install".to_string()];
    assert_eq!(match_command(&cmd), Some(CommandClass::NpmInstall));

    // Docker build
    let cmd = vec![
        "docker".to_string(),
        "build".to_string(),
        "-t".to_string(),
        "img".to_string(),
        ".".to_string(),
    ];
    assert_eq!(match_command(&cmd), Some(CommandClass::DockerBuild));

    // Make
    let cmd = vec!["make".to_string(), "-j8".to_string()];
    assert_eq!(match_command(&cmd), Some(CommandClass::Make));
}

#[test]
fn test_excluded_commands() {
    // Interactive cargo run must be excluded
    let cmd = vec!["cargo".to_string(), "run".to_string()];
    assert_eq!(match_command(&cmd), None);

    // Watch mode must be excluded
    let cmd = vec![
        "cargo".to_string(),
        "watch".to_string(),
        "-x".to_string(),
        "check".to_string(),
    ];
    assert_eq!(match_command(&cmd), None);

    let cmd = vec![
        "npm".to_string(),
        "test".to_string(),
        "--".to_string(),
        "--watch".to_string(),
    ];
    assert_eq!(match_command(&cmd), None);

    // Interactive shells/REPLs must be excluded
    let cmd = vec!["python".to_string()];
    assert_eq!(match_command(&cmd), None);

    let cmd = vec!["node".to_string()];
    assert_eq!(match_command(&cmd), None);
}

#[test]
fn test_duration_history_prediction() {
    let mut history = DurationHistory::default();

    // Unknown command defaults to not immediately launching (< 8s threshold)
    assert!(!history.is_predicted_long(CommandClass::CargoBuild));

    // Record a 15-second build and a 20-second build
    history.record(CommandClass::CargoBuild, 15.0);
    history.record(CommandClass::CargoBuild, 20.0);

    // Now it should predict long
    assert!(history.is_predicted_long(CommandClass::CargoBuild));

    // Fast command (e.g. 1.2s clippy check)
    history.record(CommandClass::CargoClippy, 1.2);
    assert!(!history.is_predicted_long(CommandClass::CargoClippy));
}
