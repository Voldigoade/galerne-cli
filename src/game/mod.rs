//! Core gameplay simulation and entities.

pub mod rng;
pub mod runner;

pub use rng::FastRng;
pub use runner::{RunnerSnapshot, RunnerState};
