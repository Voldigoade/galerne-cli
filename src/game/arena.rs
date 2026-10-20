//! Arena boundaries and layout for the top-down terminal combat arena.

use crate::terminal::buffer::{Buffer, Color};

/// Defines the playable area and wall bounds inside the terminal.
#[derive(Clone, Debug, PartialEq)]
pub struct Arena {
    pub width: u16,
    pub height: u16,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl Arena {
    /// Creates a new arena bounded by the provided terminal width and height.
    /// Reserving rows for HUD and borders.
    pub fn new(width: u16, height: u16) -> Self {
        let min_x = 1.0;
        let max_x = (width.saturating_sub(2)) as f32;
        let min_y = 2.0; // Reserve top row for HUD
        let max_y = (height.saturating_sub(2)) as f32;

        Self {
            width,
            height,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Clamps an entity's coordinates within the playable arena boundaries.
    pub fn clamp_player(&self, x: f32, y: f32) -> (f32, f32) {
        let cx = x.clamp(self.min_x + 0.5, self.max_x - 0.5);
        let cy = y.clamp(self.min_y + 0.5, self.max_y - 0.5);
        (cx, cy)
    }

    /// Checks if a projectile or entity has exited the playable area.
    pub fn is_out_of_bounds(&self, x: f32, y: f32) -> bool {
        x < self.min_x || x > self.max_x || y < self.min_y || y > self.max_y
    }

    /// Renders the ASCII boundary walls to the buffer.
    pub fn render_walls(&self, buf: &mut Buffer) {
        let w = self.width;
        let h = self.height;
        if w < 10 || h < 6 {
            return;
        }

        let wall_fg = Color::DarkGrey;
        let bg = Color::Reset;

        // Top border (under HUD)
        buf.set_char(0, 1, '┌', wall_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, 1, '─', wall_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), 1, '┐', wall_fg, bg);

        // Side walls
        for y in 2..h.saturating_sub(1) {
            buf.set_char(0, y, '│', wall_fg, bg);
            buf.set_char(w.saturating_sub(1), y, '│', wall_fg, bg);
        }

        // Bottom wall
        buf.set_char(0, h.saturating_sub(1), '└', wall_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, h.saturating_sub(1), '─', wall_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), h.saturating_sub(1), '┘', wall_fg, bg);
    }
}
