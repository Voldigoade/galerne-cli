//! Terminal control, raw mode guard, and double-buffer diff renderer.

pub mod buffer;
pub mod engine;
pub mod guard;

pub use buffer::{Buffer, Cell, Color};
pub use engine::Engine;
pub use guard::TerminalGuard;
