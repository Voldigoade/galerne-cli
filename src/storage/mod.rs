//! Local state persistence, integrity, and backups.

pub mod atomic;
pub mod manager;
pub mod schema;

pub use manager::SaveManager;
pub use schema::{
    ActiveRunSnapshot, GameSettings, SaveState, WorkshopLevels, CURRENT_SCHEMA_VERSION,
};
