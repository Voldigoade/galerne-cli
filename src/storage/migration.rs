//! Deterministic schema migrations and pre-migration backup infrastructure.

use std::io::{self, Error, ErrorKind};

use serde::Deserialize;

use super::schema::{GameSettings, SaveState, WorkshopLevels};

#[derive(Deserialize)]
struct V1SaveState {
    #[allow(dead_code)]
    version: u32,
    #[serde(default)]
    total_play_time_secs: u64,
    #[serde(default)]
    total_wait_time_saved_secs: u64,
    #[serde(default)]
    total_runs_started: u32,
    #[serde(default)]
    total_enemies_killed: u32,
    #[serde(default)]
    total_bosses_killed: u32,
    #[serde(default)]
    currency_shards: u32,
    #[serde(default)]
    workshop: WorkshopLevels,
    #[serde(default)]
    settings: GameSettings,
}

/// Migrates serialized JSON payload stepwise from `from_version` to `to_version`.
pub fn migrate_save(from_version: u32, to_version: u32, payload: &[u8]) -> io::Result<SaveState> {
    if from_version == to_version {
        let state: SaveState = serde_json::from_slice(payload)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        return Ok(state);
    }

    if from_version > to_version {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Cannot downgrade save from schema v{} to v{}",
                from_version, to_version
            ),
        ));
    }

    // Stepwise migration chain: v1 -> v2
    if from_version == 1 && to_version == 2 {
        let v1: V1SaveState = serde_json::from_slice(payload).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Invalid v1 payload: {}", e))
        })?;

        return Ok(SaveState {
            version: 2,
            total_play_time_secs: v1.total_play_time_secs,
            total_wait_time_saved_secs: v1.total_wait_time_saved_secs,
            total_runs_started: v1.total_runs_started,
            total_distance_traveled: 0.0,
            total_enemies_killed: v1.total_enemies_killed,
            total_bosses_killed: v1.total_bosses_killed,
            currency_shards: v1.currency_shards,
            workshop: v1.workshop,
            active_run: None,
            settings: v1.settings,
        });
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        format!(
            "No migration path available from schema v{} to v{}",
            from_version, to_version
        ),
    ))
}
