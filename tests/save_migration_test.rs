use std::fs;

use galerne::storage::atomic::{decode_and_verify, encode_with_header};
use galerne::storage::manager::SaveManager;

#[test]
fn test_v1_save_fixture_loads_perfectly() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_v1_fixture_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    // Create a rich v1 save fixture representing actual v0.1.1 player data
    let v1_json = serde_json::json!({
        "version": 1,
        "total_play_time_secs": 1420,
        "total_wait_time_saved_secs": 980,
        "total_runs_started": 14,
        "total_enemies_killed": 450,
        "total_bosses_killed": 3,
        "currency_shards": 1250,
        "workshop": {
            "base_health_level": 2,
            "base_damage_level": 3,
            "move_speed_level": 1,
            "magnet_radius_level": 2,
            "shard_multiplier_level": 1
        },
        "active_run": null,
        "settings": {
            "color_enabled": true,
            "animations_enabled": true,
            "auto_activate": true,
            "anti_flicker_delay_ms": 2500
        }
    });

    let payload = serde_json::to_vec(&v1_json).expect("serialize v1");
    let encoded_v1 = encode_with_header(1, &payload);
    fs::write(manager.primary_save_path(), &encoded_v1).expect("write v1 save");

    // Load must migrate cleanly from v1 to v3 while preserving all progression
    let loaded = manager.load().expect("load v1 save");
    assert_eq!(loaded.version, 3);
    assert_eq!(loaded.currency_shards, 1250);
    assert_eq!(
        loaded
            .inventory
            .count(galerne::storage::ItemKind::GaleShard),
        1250
    );
    assert_eq!(loaded.total_enemies_killed, 450);
    assert_eq!(loaded.workshop.base_damage_level, 3);
    assert!(loaded.settings.auto_activate);
    assert!(loaded.active_run.is_none());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_v2_save_migrates_cleanly_to_v3() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_v2_to_v3_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    let v2_json = serde_json::json!({
        "version": 2,
        "total_play_time_secs": 850,
        "total_wait_time_saved_secs": 620,
        "total_runs_started": 9,
        "total_distance_traveled": 342.5,
        "total_enemies_killed": 25,
        "total_bosses_killed": 0,
        "currency_shards": 850,
        "workshop": {
            "base_health_level": 2,
            "base_damage_level": 1,
            "move_speed_level": 1,
            "magnet_radius_level": 0,
            "shard_multiplier_level": 1
        },
        "active_run": null,
        "settings": {
            "color_enabled": true,
            "animations_enabled": true,
            "auto_activate": true,
            "anti_flicker_delay_ms": 8000
        }
    });

    let payload = serde_json::to_vec(&v2_json).expect("serialize v2");
    let encoded_v2 = encode_with_header(2, &payload);
    fs::write(manager.primary_save_path(), &encoded_v2).expect("write v2 save");

    let loaded = manager.load().expect("load v2 save");
    assert_eq!(loaded.version, 3);
    assert_eq!(loaded.currency_shards, 850);
    assert_eq!(
        loaded
            .inventory
            .count(galerne::storage::ItemKind::GaleShard),
        850
    );
    assert_eq!(loaded.total_distance_traveled, 342.5);
    assert_eq!(
        loaded.world_progress.current_region_id,
        "cinder_line_viaduct"
    );
    assert!(!loaded.world_progress.relay_restored);
    assert!(!loaded.companion.met_kael);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_v1_save_with_arena_active_run_migrates_cleanly_to_v2() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_v1_arena_mig_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    // Raw JSON payload of a v1 save with arena active run
    let v1_json = serde_json::json!({
        "version": 1,
        "total_play_time_secs": 250,
        "total_wait_time_saved_secs": 180,
        "total_runs_started": 5,
        "total_enemies_killed": 90,
        "total_bosses_killed": 1,
        "currency_shards": 420,
        "workshop": {
            "base_health_level": 1,
            "base_damage_level": 2,
            "move_speed_level": 0,
            "magnet_radius_level": 1,
            "shard_multiplier_level": 0
        },
        "active_run": {
            "player_hp": 80.0,
            "player_max_hp": 100.0,
            "player_x": 40.0,
            "player_y": 12.0,
            "player_speed": 18.0,
            "player_magnet_radius": 5.0,
            "player_level": 2,
            "player_xp": 10,
            "player_next_level_xp": 20,
            "shards_collected": 15,
            "weapon_damage": 25.0,
            "weapon_cooldown": 0.5,
            "weapon_cooldown_remaining": 0.1,
            "weapon_multishot": 1,
            "weapon_piercing": 1,
            "weapon_speed": 40.0,
            "weapon_range": 30.0,
            "wave": 1,
            "wave_timer": 15.0,
            "spawn_timer": 0.5,
            "run_duration": 15.0,
            "enemies_killed": 12,
            "boss_spawned": false,
            "rng_state": 12345,
            "enemies": [],
            "projectiles": [],
            "gems": [],
            "pending_level_up": false,
            "current_draft": null
        },
        "settings": {
            "color_enabled": true,
            "animations_enabled": true,
            "auto_activate": true,
            "anti_flicker_delay_ms": 8000
        }
    });

    let payload = serde_json::to_vec(&v1_json).expect("serialize v1 json");
    let encoded = encode_with_header(1, &payload);
    fs::write(manager.primary_save_path(), &encoded).expect("write v1 save");

    let loaded = manager.load().expect("load and migrate");
    assert_eq!(loaded.version, 3);
    assert_eq!(loaded.currency_shards, 420);
    assert_eq!(loaded.total_enemies_killed, 90);
    assert_eq!(loaded.workshop.base_damage_level, 2);
    assert!(
        loaded.active_run.is_none(),
        "Old arena active run must be cleanly discarded on v3 migration"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_pre_migration_backup_creation() {
    let temp_dir =
        std::env::temp_dir().join(format!("galerne_migration_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let manager = SaveManager::with_dir(&temp_dir);

    let v1_json = serde_json::json!({
        "version": 1,
        "total_play_time_secs": 500,
        "currency_shards": 300
    });
    let payload = serde_json::to_vec(&v1_json).expect("serialize v1");
    let encoded_v1 = encode_with_header(1, &payload);
    fs::write(manager.primary_save_path(), &encoded_v1).expect("write v1 save");

    // Perform migration backup verification
    let backup_path = manager
        .create_migration_backup(1, 3)
        .expect("create migration backup");

    assert!(
        backup_path.exists(),
        "Migration backup file must exist on disk"
    );
    let backup_name = backup_path.file_name().unwrap().to_string_lossy();
    assert!(
        backup_name.starts_with("save.dat.migration_v1_to_v3."),
        "Backup filename must clearly identify migration source and target versions"
    );

    // Verify backup content matches original
    let backup_bytes = fs::read(&backup_path).expect("read backup");
    let (ver, payload) = decode_and_verify(&backup_bytes).expect("verify backup");
    assert_eq!(ver, 1);
    let parsed: serde_json::Value = serde_json::from_slice(payload).expect("parse backup payload");
    assert_eq!(parsed["currency_shards"], 300);

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
