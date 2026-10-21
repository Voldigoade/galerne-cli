//! Core gameplay simulation and entities.

pub mod arena;
pub mod enemies;
pub mod player;
pub mod runner;
pub mod state;
pub mod upgrades;
pub mod weapons;

pub use arena::Arena;
pub use enemies::{Enemy, EnemyKind};
pub use player::Player;
pub use runner::{RunnerSnapshot, RunnerState};
pub use state::{FastRng, GameInput, GameState};
pub use upgrades::{UpgradeDraft, UpgradeKind};
pub use weapons::{Projectile, Weapon};
