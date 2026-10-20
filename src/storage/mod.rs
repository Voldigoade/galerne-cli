//! Local state persistence, integrity, and backups.

pub mod atomic;
pub mod manager;
pub mod schema;
pub mod stats;
pub mod workshop;

pub use manager::SaveManager;
pub use schema::{
    ActiveRunSnapshot, GameSettings, SaveState, WorkshopLevels, CURRENT_SCHEMA_VERSION,
};
pub use stats::format_career_stats;
pub use workshop::{
    apply_workshop_to_game, purchase_workshop_upgrade, WorkshopUpgradeType, MAX_WORKSHOP_LEVEL,
};
