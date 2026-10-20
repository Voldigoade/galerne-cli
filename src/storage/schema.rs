//! Save schema definitions, migrations, and snapshots.

use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The complete persistent game state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveState {
    pub version: u32,
    pub total_play_time_secs: u64,
    pub total_wait_time_saved_secs: u64,
    pub total_runs_started: u32,
    pub total_enemies_killed: u32,
    pub total_bosses_killed: u32,
    pub currency_shards: u32,
    pub workshop: WorkshopLevels,
    pub active_run: Option<ActiveRunSnapshot>,
    pub settings: GameSettings,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            total_play_time_secs: 0,
            total_wait_time_saved_secs: 0,
            total_runs_started: 0,
            total_enemies_killed: 0,
            total_bosses_killed: 0,
            currency_shards: 0,
            workshop: WorkshopLevels::default(),
            active_run: None,
            settings: GameSettings::default(),
        }
    }
}

/// Permanent workshop upgrades purchased with Gale Shards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkshopLevels {
    pub base_health_level: u32,
    pub base_damage_level: u32,
    pub move_speed_level: u32,
    pub magnet_radius_level: u32,
    pub shard_multiplier_level: u32,
}

/// Snapshot of an interrupted active run for exact continuation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveRunSnapshot {
    pub player_hp: f32,
    pub player_max_hp: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub player_level: u32,
    pub player_xp: u32,
    pub shards_collected: u32,
    pub weapon_damage: f32,
    pub weapon_cooldown: f32,
    pub weapon_multishot: u8,
    pub weapon_piercing: u8,
    pub wave: u32,
    pub run_duration: f32,
    pub seed: u64,
}

/// User configurable settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSettings {
    pub color_enabled: bool,
    pub animations_enabled: bool,
    pub auto_activate: bool,
    pub anti_flicker_delay_ms: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            color_enabled: true,
            animations_enabled: true,
            auto_activate: true,
            anti_flicker_delay_ms: 2500, // 2.5s anti-flicker delay for unknown tasks
        }
    }
}
