//! Weapon systems and projectile physics.

use serde::{Deserialize, Serialize};

use crate::game::enemies::Enemy;
use crate::terminal::buffer::{Buffer, Color, Style};

mod color_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = match color {
            Color::Cyan => "cyan",
            Color::Yellow => "yellow",
            Color::Red => "red",
            Color::Green => "green",
            Color::Magenta => "magenta",
            Color::Blue => "blue",
            Color::White => "white",
            _ => "reset",
        };
        serializer.serialize_str(name)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "cyan" => Color::Cyan,
            "yellow" => Color::Yellow,
            "red" => Color::Red,
            "green" => Color::Green,
            "magenta" => Color::Magenta,
            "blue" => Color::Blue,
            "white" => Color::White,
            _ => Color::Reset,
        })
    }
}

/// A projectile traveling across the arena.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    pub pos: (f32, f32),
    pub velocity: (f32, f32),
    pub damage: f32,
    pub lifetime: f32,
    pub piercing: u8,
    pub is_enemy: bool,
    pub ch: char,
    #[serde(with = "color_serde")]
    pub color: Color,
}

impl Projectile {
    /// Advances projectile position along velocity vector. Returns true if expired.
    pub fn update(&mut self, dt: f32) -> bool {
        self.pos.0 += self.velocity.0 * dt;
        self.pos.1 += self.velocity.1 * dt;
        self.lifetime -= dt;
        self.lifetime <= 0.0
    }

    /// Renders projectile to the frame buffer.
    pub fn render(&self, buf: &mut Buffer) {
        let x = self.pos.0.round() as u16;
        let y = self.pos.1.round() as u16;
        buf.set_styled(x, y, self.ch, Style::new(self.color, Color::Reset).bold());
    }
}

/// The player's primary weapon (Gale Blaster).
#[derive(Clone, Debug, PartialEq)]
pub struct Weapon {
    pub name: String,
    pub damage: f32,
    pub cooldown: f32,
    pub cooldown_remaining: f32,
    pub range: f32,
    pub speed: f32,
    pub multishot: u8,
    pub piercing: u8,
}

impl Weapon {
    /// Creates the standard Gale Blaster weapon.
    pub fn gale_blaster() -> Self {
        Self {
            name: "Gale Blaster".to_string(),
            damage: 25.0,
            cooldown: 0.45,
            cooldown_remaining: 0.0,
            range: 30.0,
            speed: 38.0,
            multishot: 1,
            piercing: 1,
        }
    }

    /// Updates weapon cooldown timer.
    pub fn update(&mut self, dt: f32) {
        if self.cooldown_remaining > 0.0 {
            self.cooldown_remaining = (self.cooldown_remaining - dt).max(0.0);
        }
    }

    /// Attempts to fire a single projectile at the closest target.
    pub fn try_fire(&mut self, player_pos: (f32, f32), enemies: &[Enemy]) -> Option<Projectile> {
        let mut projs = self.try_fire_all(player_pos, enemies);
        projs.pop()
    }

    /// Automatically targets the nearest enemies within range and returns spawned projectiles.
    pub fn try_fire_all(&mut self, player_pos: (f32, f32), enemies: &[Enemy]) -> Vec<Projectile> {
        if self.cooldown_remaining > 0.0 || enemies.is_empty() {
            return Vec::new();
        }

        // Find nearest living enemy within range
        let mut nearest: Option<((f32, f32), f32)> = None;
        for enemy in enemies {
            if enemy.is_dead() {
                continue;
            }
            let dx = enemy.x - player_pos.0;
            let dy = (enemy.y - player_pos.1) * 1.5; // Compensate for terminal vertical compression
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= self.range * self.range {
                match nearest {
                    None => nearest = Some(((dx, dy), dist_sq)),
                    Some((_, best_sq)) if dist_sq < best_sq => nearest = Some(((dx, dy), dist_sq)),
                    _ => {}
                }
            }
        }

        let Some(((target_dx, target_dy), dist_sq)) = nearest else {
            return Vec::new();
        };

        self.cooldown_remaining = self.cooldown;
        let dist = dist_sq.sqrt().max(0.001);
        let base_dir_x = target_dx / dist;
        let base_dir_y = (target_dy / dist) * 0.65; // Adjust vertical speed

        let mut projectiles = Vec::with_capacity(self.multishot as usize);

        for i in 0..self.multishot {
            // Apply slight spread angle for multishot
            let spread_offset = if self.multishot > 1 {
                let step = (i as f32) - ((self.multishot - 1) as f32 * 0.5);
                step * 0.18
            } else {
                0.0
            };

            let vx = (base_dir_x - spread_offset * base_dir_y) * self.speed;
            let vy = (base_dir_y + spread_offset * base_dir_x) * self.speed;

            projectiles.push(Projectile {
                pos: player_pos,
                velocity: (vx, vy),
                damage: self.damage,
                lifetime: 1.2,
                piercing: self.piercing,
                is_enemy: false,
                ch: '•',
                color: Color::Yellow,
            });
        }

        projectiles
    }
}
