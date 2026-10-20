//! Distinct enemy archetypes and boss encounters.

use crate::game::weapons::Projectile;
use crate::terminal::buffer::{Buffer, Color, Style};

/// Archetypes of adversaries spawned in the arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Drifter,
    Swarmer,
    Charger,
    Spitter,
    BossTempest,
}

/// An enemy entity with distinct behavioral state.
#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub kind: EnemyKind,
    pub state_timer: f32,
    pub attack_cooldown: f32,
    pub phase: u8,
    pub flash_timer: f32,
}

impl Enemy {
    /// Constructs a new enemy with stats corresponding to its archetype.
    pub fn new(x: f32, y: f32, kind: EnemyKind) -> Self {
        let (hp, speed) = match kind {
            EnemyKind::Drifter => (20.0, 6.0),
            EnemyKind::Swarmer => (10.0, 14.0),
            EnemyKind::Charger => (55.0, 5.0),
            EnemyKind::Spitter => (32.0, 7.5),
            EnemyKind::BossTempest => (500.0, 4.5),
        };

        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp,
            max_hp: hp,
            speed,
            kind,
            state_timer: 0.0,
            attack_cooldown: 1.5,
            phase: 0,
            flash_timer: 0.0,
        }
    }

    /// Returns true if the enemy has been vanquished.
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Inflicts damage and triggers damage flash.
    pub fn take_damage(&mut self, amount: f32) {
        self.hp = (self.hp - amount).max(0.0);
        self.flash_timer = 0.1;
    }

    /// Updates enemy movement and AI state based on player position.
    /// Can optionally return a spawned enemy projectile.
    pub fn update(&mut self, dt: f32, player_pos: (f32, f32)) -> Option<Projectile> {
        if self.is_dead() {
            return None;
        }

        if self.flash_timer > 0.0 {
            self.flash_timer = (self.flash_timer - dt).max(0.0);
        }

        self.state_timer += dt;
        let dx = player_pos.0 - self.x;
        let dy = (player_pos.1 - self.y) * 1.5;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        let dir_x = dx / dist;
        let dir_y = (dy / dist) * 0.65;

        let mut spawned_proj: Option<Projectile> = None;

        match self.kind {
            EnemyKind::Drifter => {
                // Steady tracking towards player
                self.vx = dir_x * self.speed;
                self.vy = dir_y * self.speed;
            }
            EnemyKind::Swarmer => {
                // Fast zig-zag movement: lateral oscillation
                let lateral_x = -dir_y * 1.5;
                let lateral_y = dir_x * 0.65;
                let wave = (self.state_timer * 10.0).sin();
                self.vx = (dir_x + lateral_x * wave * 0.7) * self.speed;
                self.vy = (dir_y + lateral_y * wave * 0.7) * self.speed;
            }
            EnemyKind::Charger => {
                // Alternates between slow aim (2.0s) and fast charge (0.8s)
                let cycle = self.state_timer % 3.0;
                if cycle < 2.0 {
                    // Aiming phase: slow approach
                    self.vx = dir_x * self.speed;
                    self.vy = dir_y * self.speed;
                } else {
                    // Charge phase: high velocity in locked direction
                    self.vx = dir_x * (self.speed * 4.5);
                    self.vy = dir_y * (self.speed * 4.5);
                }
            }
            EnemyKind::Spitter => {
                // Keeps standoff distance (10-15 units)
                if dist < 12.0 {
                    // Retreat
                    self.vx = -dir_x * self.speed;
                    self.vy = -dir_y * self.speed;
                } else if dist > 20.0 {
                    // Advance
                    self.vx = dir_x * self.speed;
                    self.vy = dir_y * self.speed;
                } else {
                    // Circle/hover
                    self.vx = -dir_y * self.speed * 0.8;
                    self.vy = dir_x * self.speed * 0.8;
                }

                // Fire periodic projectile
                self.attack_cooldown -= dt;
                if self.attack_cooldown <= 0.0 {
                    self.attack_cooldown = 2.2;
                    spawned_proj = Some(Projectile {
                        pos: (self.x, self.y),
                        velocity: (dir_x * 16.0, dir_y * 16.0),
                        damage: 15.0,
                        lifetime: 2.5,
                        piercing: 1,
                        is_enemy: true,
                        ch: '×',
                        color: Color::Green,
                    });
                }
            }
            EnemyKind::BossTempest => {
                // Boss attack cycle
                self.attack_cooldown -= dt;
                if self.attack_cooldown <= 0.0 {
                    self.attack_cooldown = 1.8;
                    self.phase = (self.phase + 1) % 3;

                    if self.phase == 0 {
                        // Ring projectile attack
                        spawned_proj = Some(Projectile {
                            pos: (self.x, self.y),
                            velocity: (dir_x * 20.0, dir_y * 20.0),
                            damage: 25.0,
                            lifetime: 3.0,
                            piercing: 1,
                            is_enemy: true,
                            ch: '☼',
                            color: Color::Magenta,
                        });
                    }
                }

                // Boss movement
                self.vx = dir_x * self.speed;
                self.vy = dir_y * self.speed;
            }
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        spawned_proj
    }

    /// Renders enemy glyph with appropriate visual hierarchy.
    pub fn render(&self, buf: &mut Buffer) {
        if self.is_dead() {
            return;
        }

        let ex = self.x.round() as u16;
        let ey = self.y.round() as u16;

        let (ch, color) = if self.flash_timer > 0.0 {
            ('!', Color::White)
        } else {
            match self.kind {
                EnemyKind::Drifter => ('*', Color::Red),
                EnemyKind::Swarmer => ('x', Color::Magenta),
                EnemyKind::Charger => ('#', Color::DarkRed),
                EnemyKind::Spitter => ('%', Color::Green),
                EnemyKind::BossTempest => ('Ω', Color::Yellow),
            }
        };

        buf.set_styled(ex, ey, ch, Style::new(color, Color::Reset).bold());
    }
}
