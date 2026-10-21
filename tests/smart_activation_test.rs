use std::fs;

use galerne::interceptor::history::{
    ActivationDecision, DurationHistory, CONSERVATIVE_FALLBACK_DELAY_MS,
    KNOWN_LONG_ANTI_FLICKER_MS, MAX_SAMPLES_PER_RECORD, MAX_SERIALIZED_BYTES, MAX_TRACKED_ENTRIES,
    PREDICTED_LONG_THRESHOLD_SECS,
};
use galerne::interceptor::matcher::CommandClass;

const PROJ_A: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const PROJ_B: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

#[test]
fn test_unknown_command_has_conservative_fallback() {
    let history = DurationHistory::default();

    // 0 samples: Unknown
    let decision_0 = history.evaluate_decision(PROJ_A, CommandClass::CargoBuild);
    assert_eq!(
        decision_0,
        ActivationDecision::Unknown {
            fallback_delay_ms: CONSERVATIVE_FALLBACK_DELAY_MS
        }
    );
    assert_eq!(decision_0.effective_delay_ms(), 8000);

    // 1 sample (even if long): still Unknown because min samples is 2
    let mut history_1 = history.clone();
    history_1.record(PROJ_A, CommandClass::CargoBuild, 45.0);
    let decision_1 = history_1.evaluate_decision(PROJ_A, CommandClass::CargoBuild);
    assert_eq!(
        decision_1,
        ActivationDecision::Unknown {
            fallback_delay_ms: CONSERVATIVE_FALLBACK_DELAY_MS
        }
    );
    assert_eq!(decision_1.effective_delay_ms(), 8000);
}

#[test]
fn test_5_second_run_never_exceeds_conservative_threshold() {
    let history = DurationHistory::default();

    let decision = history.evaluate_decision(PROJ_A, CommandClass::CargoBuild);
    let effective_delay = decision.effective_delay_ms();

    // A ~5-second build (5000 ms) is strictly below 8000 ms fallback
    assert!(
        5000 < effective_delay,
        "A 5-second build must never exceed the fallback delay (effective: {} ms)",
        effective_delay
    );
}

#[test]
fn test_known_long_command_activates_near_immediately() {
    let mut history = DurationHistory::default();

    // 2 samples >= 8.0s
    history.record(PROJ_A, CommandClass::CargoBuild, 32.0);
    history.record(PROJ_A, CommandClass::CargoBuild, 38.0);

    let decision = history.evaluate_decision(PROJ_A, CommandClass::CargoBuild);
    match decision {
        ActivationDecision::KnownLong {
            median_secs,
            anti_flicker_delay_ms,
        } => {
            assert!(median_secs >= PREDICTED_LONG_THRESHOLD_SECS);
            assert_eq!(anti_flicker_delay_ms, KNOWN_LONG_ANTI_FLICKER_MS);
            assert_eq!(decision.effective_delay_ms(), 150);
        }
        other => panic!("Expected KnownLong decision, got {:?}", other),
    }
}

#[test]
fn test_known_short_command_has_fallback_threshold() {
    let mut history = DurationHistory::default();

    // 2 samples < 8.0s
    history.record(PROJ_A, CommandClass::CargoBuild, 1.2);
    history.record(PROJ_A, CommandClass::CargoBuild, 1.8);

    let decision = history.evaluate_decision(PROJ_A, CommandClass::CargoBuild);
    match decision {
        ActivationDecision::KnownShort {
            median_secs,
            fallback_delay_ms,
        } => {
            assert!(median_secs < PREDICTED_LONG_THRESHOLD_SECS);
            assert_eq!(fallback_delay_ms, CONSERVATIVE_FALLBACK_DELAY_MS);
            assert_eq!(decision.effective_delay_ms(), 8000);
        }
        other => panic!("Expected KnownShort decision, got {:?}", other),
    }
}

#[test]
fn test_two_projects_maintain_strictly_isolated_predictions() {
    let mut history = DurationHistory::default();

    // Same command class (CargoBuild) in two different projects
    history.record(PROJ_A, CommandClass::CargoBuild, 40.0);
    history.record(PROJ_A, CommandClass::CargoBuild, 50.0);

    history.record(PROJ_B, CommandClass::CargoBuild, 1.5);
    history.record(PROJ_B, CommandClass::CargoBuild, 2.0);

    let dec_heavy = history.evaluate_decision(PROJ_A, CommandClass::CargoBuild);
    let dec_light = history.evaluate_decision(PROJ_B, CommandClass::CargoBuild);

    // Heavy project activates in 150ms
    assert!(dec_heavy.is_known_long());
    assert_eq!(dec_heavy.effective_delay_ms(), 150);

    // Light project requires 8000ms fallback
    assert!(dec_light.is_known_short());
    assert_eq!(dec_light.effective_delay_ms(), 8000);
}

#[test]
fn test_bounded_history_retention() {
    let mut history = DurationHistory::default();

    // Record 25 samples
    for i in 1..=25 {
        history.record(PROJ_A, CommandClass::CargoBuild, i as f32);
    }

    let samples = history
        .get_samples(PROJ_A, CommandClass::CargoBuild)
        .expect("samples exist");

    assert_eq!(
        samples.len(),
        MAX_SAMPLES_PER_RECORD,
        "Must cap samples per key at MAX_SAMPLES_PER_RECORD"
    );
    // Should retain the most recent samples (16..=25)
    assert_eq!(samples.first().copied(), Some(16.0));
    assert_eq!(samples.last().copied(), Some(25.0));
}

#[test]
fn test_max_tracked_entries_lru_pruning_and_size_bound() {
    let mut history = DurationHistory::default();

    // Insert more entries than MAX_TRACKED_ENTRIES
    for i in 0..(MAX_TRACKED_ENTRIES + 10) {
        let proj = format!("{:064x}", i);
        history.record(&proj, CommandClass::CargoBuild, 10.0);
    }

    assert!(
        history.entries_count() <= MAX_TRACKED_ENTRIES,
        "Total entries in history must never exceed MAX_TRACKED_ENTRIES (was {})",
        history.entries_count()
    );

    // Enforce strict < 10 KiB bound
    let serialized_len = history.serialized_size_bytes();
    assert!(
        serialized_len <= MAX_SERIALIZED_BYTES,
        "Serialized history must be <= {} bytes (was {} bytes)",
        MAX_SERIALIZED_BYTES,
        serialized_len
    );
}

#[test]
fn test_legacy_history_file_backward_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_hist_compat_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let hist_path = temp_dir.join("history.json");

    // Old v0.1.1 un-keyed format
    let old_v011_json = r#"{
        "records": {
            "CargoBuild": [12.0, 15.0],
            "NpmInstall": [25.0]
        }
    }"#;
    fs::write(&hist_path, old_v011_json).expect("write old history");

    // Loading must succeed without panic, migrating cleanly to empty v3 schema
    let loaded = DurationHistory::load_from_file(&hist_path);
    assert_eq!(loaded.schema_version, 3);
    assert_eq!(loaded.entries_count(), 0);

    // Old v2 format with CRC32 'p_' prefix keys
    let old_v2_json = r#"{
        "schema_version": 2,
        "entries": {
            "p_12345678abcdef01:cargo_build": {
                "durations": [20.0],
                "last_updated_epoch_secs": 1000
            }
        }
    }"#;
    fs::write(&hist_path, old_v2_json).expect("write v2 history");

    let loaded_v2 = DurationHistory::load_from_file(&hist_path);
    assert_eq!(loaded_v2.schema_version, 3);
    assert_eq!(
        loaded_v2.entries_count(),
        0,
        "Old v2 CRC32 project keys must be discarded"
    );

    // Saving and reloading new format
    let mut updated = loaded_v2;
    updated.record(PROJ_A, CommandClass::CargoBuild, 30.0);
    updated.save_to_file(&hist_path).expect("save new history");

    let reloaded = DurationHistory::load_from_file(&hist_path);
    assert_eq!(reloaded.schema_version, 3);
    assert_eq!(reloaded.entries_count(), 1);

    let _ = fs::remove_dir_all(&temp_dir);
}
