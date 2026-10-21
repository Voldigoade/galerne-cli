//! Deterministic schema migrations and pre-migration backup infrastructure.

use std::io::{self, Error, ErrorKind};

use serde::Deserialize;

use super::schema::{
    CompanionProgress, GameSettings, Inventory, ItemKind, RefugeProgress, SaveState,
    WorkshopLevels, WorldProgress,
};

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

#[derive(Deserialize)]
struct V2SaveState {
    #[allow(dead_code)]
    version: u32,
    #[serde(default)]
    total_play_time_secs: u64,
    #[serde(default)]
    total_wait_time_saved_secs: u64,
    #[serde(default)]
    total_runs_started: u32,
    #[serde(default)]
    total_distance_traveled: f32,
    #[serde(default)]
    total_enemies_killed: u32,
    #[serde(default)]
    total_bosses_killed: u32,
    #[serde(default)]
    currency_shards: u32,
    #[serde(default)]
    workshop: WorkshopLevels,
    #[serde(default)]
    active_run: Option<crate::game::runner::RunnerSnapshot>,
    #[serde(default)]
    settings: GameSettings,
}

fn v1_to_v2(v1: V1SaveState) -> V2SaveState {
    V2SaveState {
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
    }
}

fn v2_to_v3(v2: V2SaveState) -> SaveState {
    let mut inventory = Inventory::default();
    if v2.currency_shards > 0 {
        inventory.add(ItemKind::GaleShard, v2.currency_shards);
    }

    SaveState {
        version: 3,
        total_play_time_secs: v2.total_play_time_secs,
        total_wait_time_saved_secs: v2.total_wait_time_saved_secs,
        total_runs_started: v2.total_runs_started,
        total_distance_traveled: v2.total_distance_traveled,
        total_enemies_killed: v2.total_enemies_killed,
        total_bosses_killed: v2.total_bosses_killed,
        currency_shards: v2.currency_shards,
        workshop: v2.workshop,
        active_run: v2.active_run,
        settings: v2.settings,
        inventory,
        world_progress: WorldProgress::default(),
        companion: CompanionProgress::default(),
        refuge: RefugeProgress::default(),
    }
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

    if from_version == 1 && to_version == 2 {
        let v1: V1SaveState = serde_json::from_slice(payload).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Invalid v1 payload: {}", e))
        })?;
        let v2 = v1_to_v2(v1);
        return Ok(SaveState {
            version: 2,
            total_play_time_secs: v2.total_play_time_secs,
            total_wait_time_saved_secs: v2.total_wait_time_saved_secs,
            total_runs_started: v2.total_runs_started,
            total_distance_traveled: v2.total_distance_traveled,
            total_enemies_killed: v2.total_enemies_killed,
            total_bosses_killed: v2.total_bosses_killed,
            currency_shards: v2.currency_shards,
            workshop: v2.workshop,
            active_run: v2.active_run,
            settings: v2.settings,
            inventory: Inventory::default(),
            world_progress: WorldProgress::default(),
            companion: CompanionProgress::default(),
            refuge: RefugeProgress::default(),
        });
    }

    if from_version == 2 && to_version == 3 {
        let v2: V2SaveState = serde_json::from_slice(payload).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Invalid v2 payload: {}", e))
        })?;
        return Ok(v2_to_v3(v2));
    }

    if from_version == 1 && to_version == 3 {
        let v1: V1SaveState = serde_json::from_slice(payload).map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Invalid v1 payload: {}", e))
        })?;
        let v2 = v1_to_v2(v1);
        return Ok(v2_to_v3(v2));
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        format!(
            "No migration path available from schema v{} to v{}",
            from_version, to_version
        ),
    ))
}
