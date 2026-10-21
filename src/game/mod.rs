//! Core gameplay simulation and entities.

pub mod pixel;
pub mod prototype;
pub mod prototype_scene;
pub mod rng;
pub mod runner;

pub use rng::FastRng;
pub use runner::{RunnerSnapshot, RunnerState};
