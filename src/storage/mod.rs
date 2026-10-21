//! Local state persistence, integrity, and backups.

pub mod atomic;
pub mod manager;
pub mod migration;
pub mod schema;
pub mod stats;
pub mod workshop;

pub use manager::SaveManager;
pub use schema::{
    CompanionProgress, GameSettings, Inventory, InventorySlot, ItemKind, RefugeProgress,
    RunnerSnapshot, SaveState, WorkshopLevels, WorldProgress, CURRENT_SCHEMA_VERSION,
};
pub use stats::format_career_stats;
pub use workshop::{
    apply_workshop_to_runner, purchase_workshop_upgrade, WorkshopUpgradeType, MAX_WORKSHOP_LEVEL,
};
