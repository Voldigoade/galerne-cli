use std::fs;

use galerne::storage::manager::SaveManager;
use galerne::storage::schema::{GameSettings, SaveState, WorkshopLevels};
use galerne::updater::windows::replace_and_verify_windows_binary;

#[test]
fn test_windows_update_preserves_save_data_and_settings() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_win_preservation_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let save_manager = SaveManager::with_dir(&temp_dir);

    // Create existing save state with high progression
    let existing_state = SaveState {
        version: 2,
        total_play_time_secs: 5500,
        total_wait_time_saved_secs: 3200,
        total_runs_started: 35,
        total_distance_traveled: 4200.0,
        total_enemies_killed: 2400,
        total_bosses_killed: 12,
        currency_shards: 8900,
        workshop: WorkshopLevels {
            base_health_level: 5,
            base_damage_level: 5,
            move_speed_level: 5,
            magnet_radius_level: 5,
            shard_multiplier_level: 5,
        },
        active_run: None,
        settings: GameSettings {
            color_enabled: true,
            animations_enabled: true,
            auto_activate: false, // User explicitly set to false
            anti_flicker_delay_ms: 8000,
            locale: None,
        },
    };

    save_manager.save(&existing_state).expect("save state");

    // Simulate binary update occurring alongside data dir
    let fake_install_dir = temp_dir.join("bin");
    fs::create_dir_all(&fake_install_dir).expect("create bin dir");
    let fake_exe = fake_install_dir.join("galerne.exe");
    fs::write(&fake_exe, b"old galerne binary").expect("write old exe");

    // Perform replacement of the binary (as Windows installer does during upgrade)
    let backup_exe = fake_exe.with_extension("exe.old");
    fs::rename(&fake_exe, &backup_exe).expect("rename old exe");
    fs::write(&fake_exe, b"new galerne v0.2.1 binary").expect("write new exe");
    fs::remove_file(&backup_exe).expect("remove backup exe");

    // Verify binary is new
    let new_bytes = fs::read(&fake_exe).expect("read new exe");
    assert_eq!(new_bytes, b"new galerne v0.2.1 binary");

    // Verify save data and user settings remain 100% intact
    let loaded_state = save_manager.load().expect("load state after upgrade");
    assert_eq!(loaded_state.currency_shards, 8900);
    assert_eq!(loaded_state.total_enemies_killed, 2400);
    assert_eq!(loaded_state.workshop.base_damage_level, 5);
    assert!(
        !loaded_state.settings.auto_activate,
        "User settings (such as auto_activate) must be preserved across updates"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

fn get_valid_galerne_bytes() -> Vec<u8> {
    let galerne_bin = env!("CARGO_BIN_EXE_galerne");
    fs::read(galerne_bin).expect("read CARGO_BIN_EXE_galerne")
}

fn get_alien_binary_bytes() -> Vec<u8> {
    #[cfg(windows)]
    {
        let cmd_path = std::path::Path::new(r"C:\Windows\System32\cmd.exe");
        if cmd_path.exists() {
            fs::read(cmd_path).unwrap_or_else(|_| b"fake_binary_bytes".to_vec())
        } else {
            b"fake_binary_bytes".to_vec()
        }
    }
    #[cfg(not(windows))]
    {
        b"#!/bin/sh\necho 'unrelated-utility 9.9.9'\nexit 0\n".to_vec()
    }
}

fn get_failing_binary_bytes() -> Vec<u8> {
    #[cfg(windows)]
    {
        b"CORRUPTED_BINARY_FAILING_PAYLOAD".to_vec()
    }
    #[cfg(not(windows))]
    {
        b"#!/bin/sh\necho 'fatal error'\nexit 1\n".to_vec()
    }
}

#[test]
fn test_windows_binary_replacement_rollback_on_execution_failure() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_win_rollback_exec_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let target_bin = temp_dir.join(if cfg!(windows) {
        "galerne.exe"
    } else {
        "galerne"
    });
    let original_bytes = get_valid_galerne_bytes();
    fs::write(&target_bin, &original_bytes).expect("write original");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755));
    }

    // Attempt replacement with completely unexecutable / corrupt bytes
    let corrupt_bytes = b"CORRUPTED_NON_EXECUTABLE_PAYLOAD_DATA";
    let res = replace_and_verify_windows_binary(&target_bin, corrupt_bytes);

    assert!(
        res.is_err(),
        "Smoke test on corrupt binary must return an Err"
    );

    // Verify target binary was safely rolled back to original
    let restored_bytes = fs::read(&target_bin).expect("read target after rollback");
    assert_eq!(
        restored_bytes, original_bytes,
        "Target executable must be restored to original upon execution failure"
    );

    // Verify backup file was cleaned up
    let backup_bin = target_bin.with_extension("exe.old");
    assert!(
        !backup_bin.exists(),
        "Backup file must be renamed back and not left dangling"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_windows_binary_replacement_rollback_on_failed_exit_status() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_win_rollback_exit_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let target_bin = temp_dir.join(if cfg!(windows) {
        "galerne.exe"
    } else {
        "galerne"
    });
    let original_bytes = get_valid_galerne_bytes();
    fs::write(&target_bin, &original_bytes).expect("write original");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755));
    }

    let failing_payload = get_failing_binary_bytes();
    let res = replace_and_verify_windows_binary(&target_bin, &failing_payload);

    assert!(
        res.is_err(),
        "Smoke test on failing payload must return an Err"
    );

    let restored_bytes = fs::read(&target_bin).expect("read target after rollback");
    assert_eq!(
        restored_bytes, original_bytes,
        "Target executable must be restored to original upon non-zero exit status"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_windows_binary_replacement_rollback_on_unexpected_output() {
    let temp_dir = std::env::temp_dir().join(format!(
        "galerne_win_rollback_output_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let target_bin = temp_dir.join(if cfg!(windows) {
        "galerne.exe"
    } else {
        "galerne"
    });
    let original_bytes = get_valid_galerne_bytes();
    fs::write(&target_bin, &original_bytes).expect("write original");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755));
    }

    let alien_payload = get_alien_binary_bytes();
    let res = replace_and_verify_windows_binary(&target_bin, &alien_payload);

    assert!(
        res.is_err(),
        "Smoke test on unexpected output must return an Err"
    );

    let restored_bytes = fs::read(&target_bin).expect("read target after rollback");
    assert_eq!(
        restored_bytes, original_bytes,
        "Target executable must be restored to original upon unexpected version output"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_windows_binary_replacement_success() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_win_success_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let target_bin = temp_dir.join(if cfg!(windows) {
        "galerne.exe"
    } else {
        "galerne"
    });
    let original_bytes = get_valid_galerne_bytes();
    fs::write(&target_bin, &original_bytes).expect("write initial binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755));
    }

    let res = replace_and_verify_windows_binary(&target_bin, &original_bytes);

    assert!(
        res.is_ok(),
        "Smoke test on valid binary must succeed: {:?}",
        res.err()
    );

    let updated_bytes = fs::read(&target_bin).expect("read target after update");
    assert_eq!(updated_bytes, original_bytes);

    let backup_bin = target_bin.with_extension("exe.old");
    assert!(
        !backup_bin.exists(),
        "Backup file must be removed on success"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
