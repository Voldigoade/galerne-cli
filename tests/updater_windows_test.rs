use std::fs;

use galerne::storage::manager::SaveManager;
use galerne::storage::schema::{GameSettings, SaveState, WorkshopLevels};

#[test]
fn test_windows_update_preserves_save_data_and_settings() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_win_preservation_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let save_manager = SaveManager::with_dir(&temp_dir);

    // Create existing save state with high progression
    let existing_state = SaveState {
        version: 1,
        total_play_time_secs: 5500,
        total_wait_time_saved_secs: 3200,
        total_runs_started: 35,
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
            anti_flicker_delay_ms: 2500,
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
    fs::write(&fake_exe, b"new galerne v0.2.0 binary").expect("write new exe");
    fs::remove_file(&backup_exe).expect("remove backup exe");

    // Verify binary is new
    let new_bytes = fs::read(&fake_exe).expect("read new exe");
    assert_eq!(new_bytes, b"new galerne v0.2.0 binary");

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
