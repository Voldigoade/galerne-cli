//! Player entity representing the developer's avatar.

use crate::game::state::GameInput;
use crate::terminal::buffer::{Buffer, Color, Style};

/// The player avatar.
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub level: u32,
    pub xp: u32,
    pub next_level_xp: u32,
    pub shards: u32,
    pub invulnerable_timer: f32,
    pub magnet_radius: f32,
}

impl Player {
    /// Creates a new player at given starting coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp: 100.0,
            max_hp: 100.0,
            speed: 16.0,
            level: 1,
            xp: 0,
            next_level_xp: 25,
            shards: 0,
            invulnerable_timer: 0.0,
            magnet_radius: 6.0,
        }
    }

    /// Sets velocity from input vector. Scales vertical speed for terminal aspect ratio.
    pub fn apply_input(&mut self, input: GameInput) {
        let (dx, dy) = if input.move_x != 0.0 && input.move_y != 0.0 {
            // Normalize diagonal movement
            let inv_sqrt2 = 0.70710677;
            (input.move_x * inv_sqrt2, input.move_y * inv_sqrt2)
        } else {
            (input.move_x, input.move_y)
        };

        // Terminals cells are taller than they are wide (~2:1 aspect ratio),
        // so vertical step is scaled by 0.65 to provide smooth visual movement.
        self.vx = dx * self.speed;
        self.vy = dy * (self.speed * 0.65);
    }

    /// Advances position and timers.
    pub fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        if self.invulnerable_timer > 0.0 {
            self.invulnerable_timer = (self.invulnerable_timer - dt).max(0.0);
        }
    }

    /// Applies damage unless player is currently invulnerable. Returns true if damage taken.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.invulnerable_timer <= 0.0 && self.hp > 0.0 {
            self.hp = (self.hp - amount).max(0.0);
            self.invulnerable_timer = 0.5; // Half second invulnerability
            true
        } else {
            false
        }
    }

    /// Restores health up to max HP.
    pub fn heal(&mut self, amount: f32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Adds XP and checks for level-up. Returns true on level-up.
    pub fn add_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        if self.xp >= self.next_level_xp {
            self.xp -= self.next_level_xp;
            self.level += 1;
            // Progressive level curve
            self.next_level_xp = (self.next_level_xp as f32 * 1.35) as u32 + 15;
            true
        } else {
            false
        }
    }

    /// Renders player glyph to buffer.
    pub fn render(&self, buf: &mut Buffer) {
        let px = self.x.round() as u16;
        let py = self.y.round() as u16;

        let style = if self.invulnerable_timer > 0.0 {
            // Flash white when invulnerable
            Style::new(Color::White, Color::Reset).bold()
        } else {
            Style::new(Color::Cyan, Color::Reset).bold()
        };

        buf.set_styled(px, py, '@', style);
    }
}
