//! Core gameplay simulation and entities.

pub mod arena;
pub mod enemies;
pub mod player;
pub mod state;
pub mod weapons;

pub use arena::Arena;
pub use enemies::{Enemy, EnemyKind};
pub use player::Player;
pub use state::{FastRng, GameInput, GameState};
pub use weapons::{Projectile, Weapon};
