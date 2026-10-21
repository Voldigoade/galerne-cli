use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use galerne::interceptor::matcher::{match_command, CommandClass};
use galerne::interceptor::supervisor::execute_wrapped_task;

fn create_mock_executable(dir: &std::path::Path, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = dir.join(name);
        fs::write(&script, "#!/bin/sh\nexit 42\n").expect("write mock script");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("set perms");
        script
    }
    #[cfg(windows)]
    {
        let script = dir.join(format!("{}.cmd", name));
        fs::write(&script, "@exit /b 42\r\n").expect("write mock cmd");
        script
    }
}

#[test]
fn test_match_command_identifies_non_eligible_and_interactive() {
    // 1. cargo run (interactive executable launch)
    assert_eq!(
        match_command(&["cargo".to_string(), "run".to_string()]),
        None
    );

    // 2. unsupported cargo subcommands
    assert_eq!(
        match_command(&["cargo".to_string(), "tree".to_string()]),
        None
    );
    assert_eq!(
        match_command(&["cargo".to_string(), "update".to_string()]),
        None
    );
    assert_eq!(
        match_command(&["cargo".to_string(), "new".to_string(), "my_app".to_string()]),
        None
    );

    // 3. npm run dev (development server / persistent daemon)
    assert_eq!(
        match_command(&["npm".to_string(), "run".to_string(), "dev".to_string()]),
        None
    );
    assert_eq!(
        match_command(&["npm".to_string(), "start".to_string()]),
        None
    );

    // 4. docker ps (non-build container operations)
    assert_eq!(
        match_command(&["docker".to_string(), "ps".to_string()]),
        None
    );
    assert_eq!(
        match_command(&[
            "docker".to_string(),
            "run".to_string(),
            "-it".to_string(),
            "ubuntu".to_string()
        ]),
        None
    );

    // 5. go run (interactive binary execution)
    assert_eq!(
        match_command(&["go".to_string(), "run".to_string(), "main.go".to_string()]),
        None
    );

    // 6. watch-mode flags across tools
    assert_eq!(
        match_command(&[
            "cargo".to_string(),
            "test".to_string(),
            "--watch".to_string()
        ]),
        None
    );
    assert_eq!(
        match_command(&["pytest".to_string(), "-w".to_string()]),
        None
    );
    assert_eq!(
        match_command(&["npm".to_string(), "test".to_string(), "--watch".to_string()]),
        None
    );
}

#[test]
fn test_match_command_identifies_supported_eligible_operations() {
    assert_eq!(
        match_command(&["cargo".to_string(), "build".to_string()]),
        Some(CommandClass::CargoBuild)
    );
    assert_eq!(
        match_command(&["cargo".to_string(), "test".to_string()]),
        Some(CommandClass::CargoTest)
    );
    assert_eq!(
        match_command(&["npm".to_string(), "run".to_string(), "build".to_string()]),
        Some(CommandClass::NpmBuild)
    );
    assert_eq!(
        match_command(&["docker".to_string(), "build".to_string(), ".".to_string()]),
        Some(CommandClass::DockerBuild)
    );
    assert_eq!(
        match_command(&["go".to_string(), "build".to_string()]),
        Some(CommandClass::GoBuild)
    );
    assert_eq!(
        match_command(&["pytest".to_string()]),
        Some(CommandClass::Pytest)
    );
}

#[test]
fn test_execute_wrapped_task_bypasses_supervision_immediately_for_non_eligible() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_bypass_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);

    // Create mock executables in temp directory
    let cargo_mock = create_mock_executable(&temp_dir, "cargo");
    let npm_mock = create_mock_executable(&temp_dir, "npm");
    let docker_mock = create_mock_executable(&temp_dir, "docker");
    let go_mock = create_mock_executable(&temp_dir, "go");

    let test_cases = [
        vec![cargo_mock.to_string_lossy().to_string(), "run".to_string()],
        vec![cargo_mock.to_string_lossy().to_string(), "tree".to_string()],
        vec![
            npm_mock.to_string_lossy().to_string(),
            "run".to_string(),
            "dev".to_string(),
        ],
        vec![docker_mock.to_string_lossy().to_string(), "ps".to_string()],
        vec![
            go_mock.to_string_lossy().to_string(),
            "run".to_string(),
            "main.go".to_string(),
        ],
        vec![
            cargo_mock.to_string_lossy().to_string(),
            "test".to_string(),
            "--watch".to_string(),
        ],
    ];

    for args in test_cases {
        let start = Instant::now();
        let code = execute_wrapped_task(&args).expect("execute non-eligible task directly");
        let elapsed = start.elapsed();

        assert_eq!(
            code, 42,
            "Direct execution must preserve exact exit code 42"
        );
        assert!(
            elapsed < Duration::from_millis(1000),
            "Non-eligible command {:?} must bypass Galerne immediately without anti-flicker delay (took {:?})",
            args,
            elapsed
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}
