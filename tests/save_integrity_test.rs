use std::fs;
use std::path::PathBuf;

use galerne::storage::atomic::encode_with_header;
use galerne::storage::manager::{SaveLoadError, SaveManager};
use galerne::storage::schema::{SaveState, CURRENT_SCHEMA_VERSION};

fn temp_test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("galerne_tests").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("created temp test dir");
    dir
}

#[test]
fn test_save_round_trip() {
    let dir = temp_test_dir("round_trip");
    let manager = SaveManager::with_dir(dir);

    let state = SaveState {
        currency_shards: 150,
        total_enemies_killed: 42,
        ..Default::default()
    };

    manager.save(&state).expect("save should succeed");

    let loaded = manager.load().expect("load should succeed");
    assert_eq!(loaded.version, CURRENT_SCHEMA_VERSION);
    assert_eq!(loaded.currency_shards, 150);
    assert_eq!(loaded.total_enemies_killed, 42);
}

#[test]
fn test_save_corruption_recovery_from_backup() {
    let dir = temp_test_dir("corruption_recovery");
    let manager = SaveManager::with_dir(&dir);

    // 1. Initial valid save
    let initial = SaveState {
        currency_shards: 100,
        ..Default::default()
    };
    manager.save(&initial).expect("initial save");

    // 2. Second valid save (creates backup of initial)
    let second = SaveState {
        currency_shards: 200,
        ..Default::default()
    };
    manager.save(&second).expect("second save");

    let save_path = dir.join("save.dat");
    assert!(save_path.exists());
    let bak_path = dir.join("save.dat.bak");
    assert!(bak_path.exists(), "Backup should exist after second save");

    // 3. Corrupt primary save file by flipping bytes in payload
    let mut bytes = fs::read(&save_path).expect("read save");
    let len = bytes.len();
    bytes[len - 5] ^= 0xFF; // Corrupt payload byte
    fs::write(&save_path, bytes).expect("write corrupted file");

    // 4. Loading should detect CRC mismatch, quarantine corrupt file, and restore from backup
    let recovered = manager.load().expect("load should recover from backup");
    assert_eq!(
        recovered.currency_shards, 100,
        "Should recover 100 shards from known-good backup"
    );

    // Verify a corrupt file was quarantined
    let corrupt_files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("save.dat.corrupt")
        })
        .collect();
    assert_eq!(corrupt_files.len(), 1, "Corrupted file must be quarantined");
}

#[test]
fn test_future_version_rejection() {
    let dir = temp_test_dir("future_version");
    let manager = SaveManager::with_dir(&dir);

    let mut future_bytes = encode_with_header(CURRENT_SCHEMA_VERSION + 1, b"future payload");
    future_bytes[16] ^= 0xff;
    fs::write(manager.primary_save_path(), &future_bytes).expect("write future save");

    let result = manager.load_or_default();
    assert!(matches!(
        result,
        Err(SaveLoadError::UnsupportedSchema { .. })
    ));
    assert_eq!(
        fs::read(manager.primary_save_path()).expect("read future save"),
        future_bytes
    );
}

#[test]
fn test_save_refuses_to_replace_future_primary_or_backup() {
    let dir = temp_test_dir("future_save_guard");
    let manager = SaveManager::with_dir(&dir);
    let future_bytes = encode_with_header(CURRENT_SCHEMA_VERSION + 2, b"newer Galerne data");
    fs::write(manager.primary_save_path(), &future_bytes).expect("write future primary");

    assert!(manager.save(&SaveState::default()).is_err());
    assert_eq!(fs::read(manager.primary_save_path()).unwrap(), future_bytes);
    assert!(manager
        .save(&SaveState {
            version: CURRENT_SCHEMA_VERSION + 1,
            ..SaveState::default()
        })
        .is_err());

    fs::remove_file(manager.primary_save_path()).expect("remove future primary");
    manager
        .save(&SaveState::default())
        .expect("write current primary");
    let primary_before = fs::read(manager.primary_save_path()).unwrap();
    fs::write(manager.backup_save_path(), &future_bytes).expect("write future backup");

    assert!(manager.save(&SaveState::default()).is_err());
    assert_eq!(
        fs::read(manager.primary_save_path()).unwrap(),
        primary_before
    );
    assert_eq!(fs::read(manager.backup_save_path()).unwrap(), future_bytes);
}

#[test]
fn test_reset_and_cli_settings_refuse_future_saves() {
    let dir = temp_test_dir("future_save_callers");
    let manager = SaveManager::with_dir(&dir);
    let future_bytes = encode_with_header(CURRENT_SCHEMA_VERSION + 1, b"future progression");
    fs::write(manager.primary_save_path(), &future_bytes).expect("write future save");

    assert!(manager.archive_and_reset().is_err());
    assert_eq!(fs::read(manager.primary_save_path()).unwrap(), future_bytes);

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_galerne"))
        .arg("enable")
        .env("GALERNE_DATA_DIR", &dir)
        .output()
        .expect("run enable command");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("newer version of Galerne"));
    assert_eq!(fs::read(manager.primary_save_path()).unwrap(), future_bytes);
}

#[test]
fn test_missing_save_is_distinct_from_corruption_and_io_failure() {
    let dir = temp_test_dir("missing_save");
    let manager = SaveManager::with_dir(&dir);
    assert!(matches!(manager.load(), Err(SaveLoadError::Missing)));
    assert_eq!(manager.load_or_default().unwrap(), SaveState::default());

    let blocked_path = dir.join("not_a_directory");
    fs::write(&blocked_path, b"file").expect("create non-directory parent");
    let io_manager = SaveManager::with_dir(blocked_path);
    assert!(matches!(io_manager.load(), Err(SaveLoadError::Io(_))));
}

#[test]
fn test_unrecoverable_corruption_is_quarantined_without_creating_defaults() {
    let dir = temp_test_dir("unrecoverable_corruption");
    let manager = SaveManager::with_dir(&dir);
    let corrupt_bytes = b"damaged save bytes";
    fs::write(manager.primary_save_path(), corrupt_bytes).expect("write corrupt primary");

    assert!(matches!(manager.load(), Err(SaveLoadError::Corrupt(_))));
    assert!(!manager.primary_save_path().exists());
    let quarantined = fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("save.dat.corrupt")
        })
        .expect("preserve corrupt original in quarantine");
    assert_eq!(fs::read(quarantined.path()).unwrap(), corrupt_bytes);
}

#[test]
fn test_active_run_persistence_and_resume() {
    let dir = temp_test_dir("active_run");
    let manager = SaveManager::with_dir(dir);

    let mut runner = galerne::game::runner::RunnerState::new(80, 24, 0x12345678);
    runner.distance = 125.5;
    runner.speed = 24.0;
    runner.score = 350;
    runner.shards_collected = 18;
    runner.player.shields = 2;

    let snapshot = runner.create_snapshot();
    let state = SaveState {
        active_run: Some(snapshot),
        ..Default::default()
    };

    manager.save(&state).expect("save active run");

    let loaded = manager.load().expect("load active run");
    assert!(loaded.active_run.is_some());
    let loaded_snap = loaded.active_run.unwrap();

    let mut restored_runner = galerne::game::runner::RunnerState::new(80, 24, 0);
    restored_runner.resume_snapshot(&loaded_snap);

    assert_eq!(restored_runner.distance, 125.5);
    assert_eq!(restored_runner.speed, 24.0);
    assert_eq!(restored_runner.score, 350);
    assert_eq!(restored_runner.shards_collected, 18);
    assert_eq!(restored_runner.player.shields, 2);
}

#[test]
fn test_archive_creation_and_restoration() {
    let dir = temp_test_dir("archive_restore");
    let manager = SaveManager::with_dir(&dir);

    // Initial state with career progression
    let state = SaveState {
        currency_shards: 350,
        total_runs_started: 12,
        total_enemies_killed: 88,
        ..Default::default()
    };
    manager.save(&state).expect("saved initial state");

    // Perform archive and reset
    let archive_path = manager.archive_and_reset().expect("archive and reset");
    assert!(archive_path.exists());

    // After reset, save state must be fresh default
    let fresh_state = manager.load().expect("load fresh");
    assert_eq!(fresh_state.currency_shards, 0);
    assert_eq!(fresh_state.total_enemies_killed, 0);

    // List archives
    let archives = manager.list_archives().expect("list archives");
    assert_eq!(archives.len(), 1);
    let (archive_name, _) = &archives[0];

    // Restore the archive
    manager
        .restore_archive(archive_name)
        .expect("restore archive");

    // Load state again: career progression must be fully restored!
    let restored_state = manager.load().expect("load restored");
    assert_eq!(restored_state.currency_shards, 350);
    assert_eq!(restored_state.total_runs_started, 12);
    assert_eq!(restored_state.total_enemies_killed, 88);
}
