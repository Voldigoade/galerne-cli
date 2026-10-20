//! Permanent meta-progression upgrades purchased with Gale Shards.

use crate::game::state::GameState;
use crate::storage::schema::{SaveState, WorkshopLevels};

pub const MAX_WORKSHOP_LEVEL: u32 = 5;

/// Meta-progression upgrades available in the Workshop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkshopUpgradeType {
    BaseHealth,
    BaseDamage,
    MoveSpeed,
    MagnetRadius,
    ShardMultiplier,
}

impl WorkshopUpgradeType {
    pub const ALL: [WorkshopUpgradeType; 5] = [
        WorkshopUpgradeType::BaseHealth,
        WorkshopUpgradeType::BaseDamage,
        WorkshopUpgradeType::MoveSpeed,
        WorkshopUpgradeType::MagnetRadius,
        WorkshopUpgradeType::ShardMultiplier,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            WorkshopUpgradeType::BaseHealth => "Hull Integrity",
            WorkshopUpgradeType::BaseDamage => "Plasma Capacitor",
            WorkshopUpgradeType::MoveSpeed => "Ion Thrusters",
            WorkshopUpgradeType::MagnetRadius => "Collector Coil",
            WorkshopUpgradeType::ShardMultiplier => "Yield Analyzer",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WorkshopUpgradeType::BaseHealth => "+15 Starting Max HP per rank",
            WorkshopUpgradeType::BaseDamage => "+10% Weapon Damage per rank",
            WorkshopUpgradeType::MoveSpeed => "+8% Movement Speed per rank",
            WorkshopUpgradeType::MagnetRadius => "+20% Pickup Range per rank",
            WorkshopUpgradeType::ShardMultiplier => "+25% Shard Drop Yield per rank",
        }
    }

    pub fn max_level(&self) -> u32 {
        MAX_WORKSHOP_LEVEL
    }

    pub fn cost(&self, current_level: u32) -> u32 {
        let base = match self {
            WorkshopUpgradeType::BaseHealth => 10,
            WorkshopUpgradeType::BaseDamage => 10,
            WorkshopUpgradeType::MoveSpeed => 15,
            WorkshopUpgradeType::MagnetRadius => 12,
            WorkshopUpgradeType::ShardMultiplier => 20,
        };
        base * (current_level + 1)
    }

    pub fn get_level(&self, levels: &WorkshopLevels) -> u32 {
        match self {
            WorkshopUpgradeType::BaseHealth => levels.base_health_level,
            WorkshopUpgradeType::BaseDamage => levels.base_damage_level,
            WorkshopUpgradeType::MoveSpeed => levels.move_speed_level,
            WorkshopUpgradeType::MagnetRadius => levels.magnet_radius_level,
            WorkshopUpgradeType::ShardMultiplier => levels.shard_multiplier_level,
        }
    }

    pub fn set_level(&self, levels: &mut WorkshopLevels, new_lvl: u32) {
        match self {
            WorkshopUpgradeType::BaseHealth => levels.base_health_level = new_lvl,
            WorkshopUpgradeType::BaseDamage => levels.base_damage_level = new_lvl,
            WorkshopUpgradeType::MoveSpeed => levels.move_speed_level = new_lvl,
            WorkshopUpgradeType::MagnetRadius => levels.magnet_radius_level = new_lvl,
            WorkshopUpgradeType::ShardMultiplier => levels.shard_multiplier_level = new_lvl,
        }
    }
}

/// Attempts to purchase a workshop upgrade rank. Returns true if successful.
pub fn purchase_workshop_upgrade(state: &mut SaveState, upgrade: WorkshopUpgradeType) -> bool {
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
    true
}

/// Applies persistent workshop enhancements to an active game state.
pub fn apply_workshop_to_game(workshop: &WorkshopLevels, game: &mut GameState) {
    if workshop.base_health_level > 0 {
        let bonus = (workshop.base_health_level as f32) * 15.0;
        game.player.max_hp += bonus;
        game.player.hp += bonus;
    }
    if workshop.base_damage_level > 0 {
        let mult = 1.0 + (workshop.base_damage_level as f32) * 0.10;
        game.weapon.damage *= mult;
    }
    if workshop.move_speed_level > 0 {
        let mult = 1.0 + (workshop.move_speed_level as f32) * 0.08;
        game.player.speed *= mult;
    }
    if workshop.magnet_radius_level > 0 {
        let mult = 1.0 + (workshop.magnet_radius_level as f32) * 0.20;
        game.player.magnet_radius *= mult;
    }
}
