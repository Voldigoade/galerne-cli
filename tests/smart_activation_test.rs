use std::fs;

use galerne::interceptor::history::{
    ActivationDecision, DurationHistory, CONSERVATIVE_FALLBACK_DELAY_MS,
    KNOWN_LONG_ANTI_FLICKER_MS, MAX_SAMPLES_PER_RECORD, MAX_TRACKED_ENTRIES,
    PREDICTED_LONG_THRESHOLD_SECS,
};
use galerne::interceptor::matcher::CommandClass;

#[test]
fn test_unknown_command_has_conservative_fallback() {
    let history = DurationHistory::default();
    let proj = "p_alpha1234567890";

    // 0 samples: Unknown
    let decision_0 = history.evaluate_decision(proj, CommandClass::CargoBuild);
    assert_eq!(
        decision_0,
        ActivationDecision::Unknown {
            fallback_delay_ms: CONSERVATIVE_FALLBACK_DELAY_MS
        }
    );
    assert_eq!(decision_0.effective_delay_ms(), 8000);

    // 1 sample (even if long): still Unknown because min samples is 2
    let mut history_1 = history.clone();
    history_1.record(proj, CommandClass::CargoBuild, 45.0);
    let decision_1 = history_1.evaluate_decision(proj, CommandClass::CargoBuild);
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
    let proj = "p_alpha1234567890";

    let decision = history.evaluate_decision(proj, CommandClass::CargoBuild);
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
    let proj = "p_heavyrepo12345";

    // 2 samples >= 8.0s
    history.record(proj, CommandClass::CargoBuild, 32.0);
    history.record(proj, CommandClass::CargoBuild, 38.0);

    let decision = history.evaluate_decision(proj, CommandClass::CargoBuild);
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
    let proj = "p_tinyrepo123456";

    // 2 samples < 8.0s
    history.record(proj, CommandClass::CargoBuild, 1.2);
    history.record(proj, CommandClass::CargoBuild, 1.8);

    let decision = history.evaluate_decision(proj, CommandClass::CargoBuild);
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
    let proj_heavy = "p_project_heavy_1234";
    let proj_light = "p_project_light_5678";

    // Same command class (CargoBuild) in two different projects
    history.record(proj_heavy, CommandClass::CargoBuild, 40.0);
    history.record(proj_heavy, CommandClass::CargoBuild, 50.0);

    history.record(proj_light, CommandClass::CargoBuild, 1.5);
    history.record(proj_light, CommandClass::CargoBuild, 2.0);

    let dec_heavy = history.evaluate_decision(proj_heavy, CommandClass::CargoBuild);
    let dec_light = history.evaluate_decision(proj_light, CommandClass::CargoBuild);

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
    let proj = "p_bounded_test_123";

    // Record 25 samples
    for i in 1..=25 {
        history.record(proj, CommandClass::CargoBuild, i as f32);
    }

    let samples = history
        .get_samples(proj, CommandClass::CargoBuild)
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
fn test_max_tracked_entries_lru_pruning() {
    let mut history = DurationHistory::default();

    // Insert more entries than MAX_TRACKED_ENTRIES
    for i in 0..(MAX_TRACKED_ENTRIES + 10) {
        let proj = format!("p_proj_{:06}", i);
        history.record(&proj, CommandClass::CargoBuild, 10.0);
    }

    assert!(
        history.entries_count() <= MAX_TRACKED_ENTRIES,
        "Total entries in history must never exceed MAX_TRACKED_ENTRIES (was {})",
        history.entries_count()
    );
}

#[test]
fn test_v011_history_file_backward_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_hist_compat_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let hist_path = temp_dir.join("history.json");

    // Old v0.1.1 format
    let old_v011_json = r#"{
        "records": {
            "CargoBuild": [12.0, 15.0],
            "NpmInstall": [25.0]
        }
    }"#;
    fs::write(&hist_path, old_v011_json).expect("write old history");

    // Loading must succeed without panic, migrating cleanly to empty v2 schema
    let loaded = DurationHistory::load_from_file(&hist_path);
    assert_eq!(loaded.schema_version, 2);
    assert_eq!(loaded.entries_count(), 0);

    // Saving and reloading new format
    let mut updated = loaded;
    updated.record("p_new_proj_123", CommandClass::CargoBuild, 30.0);
    updated.save_to_file(&hist_path).expect("save new history");

    let reloaded = DurationHistory::load_from_file(&hist_path);
    assert_eq!(reloaded.schema_version, 2);
    assert_eq!(reloaded.entries_count(), 1);

    let _ = fs::remove_dir_all(&temp_dir);
}
