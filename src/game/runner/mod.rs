//! Terminal-native endless runner subsystem.

pub mod action;
pub mod items;
pub mod obstacles;
pub mod parallax;
pub mod player;
pub mod region;
pub mod state;
pub mod wind;
pub mod world;

pub use state::{RunnerSnapshot, RunnerState};
