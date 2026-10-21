use std::fs;

use galerne::storage::atomic::{decode_and_verify, encode_with_header};
use galerne::storage::manager::SaveManager;
use galerne::storage::schema::{GameSettings, SaveState, WorkshopLevels};

fn create_v1_save_bytes(state: &SaveState) -> Vec<u8> {
    let payload = serde_json::to_vec(state).expect("serialize v1");
    encode_with_header(1, &payload)
}

#[test]
fn test_v1_save_fixture_loads_perfectly() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_v1_fixture_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    // Create a rich v1 save fixture representing actual v0.1.1 player data
    let original = SaveState {
        version: 1,
        total_play_time_secs: 1420,
        total_wait_time_saved_secs: 980,
        total_runs_started: 14,
        total_enemies_killed: 450,
        total_bosses_killed: 3,
        currency_shards: 1250,
        workshop: WorkshopLevels {
            base_health_level: 2,
            base_damage_level: 3,
            move_speed_level: 1,
            magnet_radius_level: 2,
            shard_multiplier_level: 1,
        },
        active_run: None,
        settings: GameSettings {
            color_enabled: true,
            animations_enabled: true,
            auto_activate: true,
            anti_flicker_delay_ms: 2500,
        },
    };

    let encoded_v1 = create_v1_save_bytes(&original);
    fs::write(manager.primary_save_path(), &encoded_v1).expect("write v1 save");

    // Load must parse cleanly without modification
    let loaded = manager.load().expect("load v1 save");
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.currency_shards, 1250);
    assert_eq!(loaded.total_enemies_killed, 450);
    assert_eq!(loaded.workshop.base_damage_level, 3);
    assert!(loaded.settings.auto_activate);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_pre_migration_backup_creation() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_migration_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    let original = SaveState {
        version: 1,
        total_play_time_secs: 500,
        currency_shards: 300,
        ..Default::default()
    };
    let encoded_v1 = create_v1_save_bytes(&original);
    fs::write(manager.primary_save_path(), &encoded_v1).expect("write v1 save");

    // Perform migration backup verification
    let backup_path = manager
        .create_migration_backup(1, 2)
        .expect("create migration backup");

    assert!(
        backup_path.exists(),
        "Migration backup file must exist on disk"
    );
    let backup_name = backup_path.file_name().unwrap().to_string_lossy();
    assert!(
        backup_name.starts_with("save.dat.migration_v1_to_v2."),
        "Backup filename must clearly identify migration source and target versions"
    );

    // Verify backup content matches original
    let backup_bytes = fs::read(&backup_path).expect("read backup");
    let (ver, payload) = decode_and_verify(&backup_bytes).expect("verify backup");
    assert_eq!(ver, 1);
    let parsed: SaveState = serde_json::from_slice(payload).expect("parse backup payload");
    assert_eq!(parsed.currency_shards, 300);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_future_schema_version_is_rejected_without_overwriting() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_future_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    // Save with schema version 999
    let payload = b"{\"version\": 999}";
    let future_bytes = encode_with_header(999, payload);
    fs::write(manager.primary_save_path(), &future_bytes).expect("write future save");

    let result = manager.load();
    assert!(result.is_err(), "Future schema version must be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("newer version of Galerne"));

    // Ensure future file was NOT deleted or overwritten with defaults
    assert!(
        manager.primary_save_path().exists(),
        "Future save must be preserved untouched"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
