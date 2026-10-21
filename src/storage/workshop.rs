//! Persistent equipment improvements used by the refuge workbench.

use crate::game::runner::RunnerState;
use crate::storage::schema::{SaveState, WorkshopLevels};

pub const MAX_WORKSHOP_LEVEL: u32 = 5;

/// Improvements available at the refuge workbench.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkshopUpgradeType {
    ShieldCapacity,
    DashRecovery,
    MoveSpeed,
    MagnetRadius,
    ShardMultiplier,
}

impl WorkshopUpgradeType {
    pub const ALL: [WorkshopUpgradeType; 5] = [
        WorkshopUpgradeType::ShieldCapacity,
        WorkshopUpgradeType::DashRecovery,
        WorkshopUpgradeType::MoveSpeed,
        WorkshopUpgradeType::MagnetRadius,
        WorkshopUpgradeType::ShardMultiplier,
    ];

    pub fn title_key(&self) -> &'static str {
        match self {
            WorkshopUpgradeType::ShieldCapacity => "wind_shield",
            WorkshopUpgradeType::DashRecovery => "dash_recovery",
            WorkshopUpgradeType::MoveSpeed => "move_speed",
            WorkshopUpgradeType::MagnetRadius => "magnet_radius",
            WorkshopUpgradeType::ShardMultiplier => "shard_yield",
        }
    }

    pub fn max_level(&self) -> u32 {
        MAX_WORKSHOP_LEVEL
    }

    pub fn cost(&self, current_level: u32) -> u32 {
        let base = match self {
            WorkshopUpgradeType::ShieldCapacity => 10,
            WorkshopUpgradeType::DashRecovery => 10,
            WorkshopUpgradeType::MoveSpeed => 15,
            WorkshopUpgradeType::MagnetRadius => 12,
            WorkshopUpgradeType::ShardMultiplier => 20,
        };
        base * (current_level + 1)
    }

    pub fn get_level(&self, levels: &WorkshopLevels) -> u32 {
        match self {
            WorkshopUpgradeType::ShieldCapacity => levels.base_health_level,
            WorkshopUpgradeType::DashRecovery => levels.base_damage_level,
            WorkshopUpgradeType::MoveSpeed => levels.move_speed_level,
            WorkshopUpgradeType::MagnetRadius => levels.magnet_radius_level,
            WorkshopUpgradeType::ShardMultiplier => levels.shard_multiplier_level,
        }
    }

    pub fn set_level(&self, levels: &mut WorkshopLevels, new_lvl: u32) {
        match self {
            WorkshopUpgradeType::ShieldCapacity => levels.base_health_level = new_lvl,
            WorkshopUpgradeType::DashRecovery => levels.base_damage_level = new_lvl,
            WorkshopUpgradeType::MoveSpeed => levels.move_speed_level = new_lvl,
            WorkshopUpgradeType::MagnetRadius => levels.magnet_radius_level = new_lvl,
            WorkshopUpgradeType::ShardMultiplier => levels.shard_multiplier_level = new_lvl,
        }
    }
}

/// Attempts to purchase a workshop upgrade rank. Returns true if successful.
pub fn purchase_workshop_upgrade(state: &mut SaveState, upgrade: WorkshopUpgradeType) -> bool {
    if !state.workbench_available() {
        return false;
    }

    let current_lvl = upgrade.get_level(&state.workshop);
    if current_lvl >= upgrade.max_level() {
        return false;
    }

    let cost = upgrade.cost(current_lvl);
    if state.currency_shards < cost {
        return false;
    }

    state.currency_shards -= cost;
    upgrade.set_level(&mut state.workshop, current_lvl + 1);
    state.refuge.workbench_unlocked = true;
    true
}

/// Applies persistent workshop enhancements to an active runner game state.
pub fn apply_workshop_to_runner(workshop: &WorkshopLevels, runner: &mut RunnerState) {
    if workshop.base_health_level > 0 {
        runner.player.shields += workshop.base_health_level.div_ceil(2);
    }
    if workshop.base_damage_level > 0 {
        runner.player.dash_cooldown_duration *= 1.0 - (workshop.base_damage_level as f32 * 0.08);
    }
    if workshop.move_speed_level > 0 {
        runner.speed += (workshop.move_speed_level as f32) * 1.5;
    }
    if workshop.shard_multiplier_level > 0 {
        runner.workshop_shard_multiplier = 1.0 + (workshop.shard_multiplier_level as f32) * 0.20;
    }
}
