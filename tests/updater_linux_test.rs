#[cfg(unix)]
mod unix_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use galerne::updater::linux::apply_linux_update;

    fn create_mock_script(path: &Path, content: &str) {
        fs::write(path, content).expect("write mock script");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("set perms");
    }

    #[test]
    fn test_atomic_replacement_success() {
        let temp_dir =
            std::env::temp_dir().join(format!("galerne_update_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");

        let target_exe = temp_dir.join("galerne");
        create_mock_script(&target_exe, "#!/bin/sh\necho 'galerne 0.1.0'\nexit 0\n");

        // New binary script that outputs "galerne 0.2.0"
        let new_binary = b"#!/bin/sh\necho 'galerne 0.2.0'\nexit 0\n";

        let result = apply_linux_update(&target_exe, new_binary);
        assert!(result.is_ok(), "Update must succeed: {:?}", result.err());

        // Verify content updated
        let output = std::process::Command::new(&target_exe)
            .arg("--version")
            .output()
            .expect("run updated exe");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("galerne 0.2.0"));

        // Verify temporary/backup files are cleaned up
        let files: Vec<_> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
            .collect();
        assert_eq!(files, vec!["galerne"], "Only target executable must remain");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_rollback_on_failed_smoke_test() {
        let temp_dir =
            std::env::temp_dir().join(format!("galerne_rollback_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");

        let target_exe = temp_dir.join("galerne");
        create_mock_script(
            &target_exe,
            "#!/bin/sh\necho 'galerne 0.1.0 (original)'\nexit 0\n",
        );

        // Corrupted new binary that fails when executed
        let broken_binary = b"#!/bin/sh\necho 'fatal crash'\nexit 1\n";

        let result = apply_linux_update(&target_exe, broken_binary);
        assert!(result.is_err(), "Update must fail when smoke test fails");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("smoke test"));

        // Assert that original executable was restored cleanly!
        let output = std::process::Command::new(&target_exe)
            .arg("--version")
            .output()
            .expect("run restored exe");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("original"),
            "Original executable must be restored after smoke test failure"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unwritable_directory_error() {
        let unwritable = Path::new("/proc/sys/fs/galerne");
        let dummy_binary = b"#!/bin/sh\nexit 0\n";

        let result = apply_linux_update(unwritable, dummy_binary);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("permission")
                || err.to_string().contains("Cannot write")
                || err.to_string().contains("read-only"),
            "Error must clearly indicate unwritable destination: {}",
            err
        );
    }
}
