//! Complete game state container and simulation coordinator.

use crate::game::arena::Arena;
use crate::game::enemies::{Enemy, EnemyKind};
use crate::game::player::Player;
use crate::game::upgrades::{UpgradeDraft, UpgradeKind};
use crate::game::weapons::{Projectile, Weapon};
use crate::terminal::buffer::{Buffer, Color, Style};

/// Lightweight deterministic xorshift64 pseudorandom generator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FastRng {
    state: u64,
}

impl FastRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x8a5cd789635d2dff } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Returns a float in `[0.0, 1.0)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16777216.0
    }

    /// Returns a float in `[min, max)`.
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Returns an integer in `[min, max]`.
    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % ((max - min + 1) as u64)) as u32
    }
}

/// Movement input applied to player.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameInput {
    pub move_x: f32,
    pub move_y: f32,
}

/// An experience gem or persistent currency dropped by defeated enemies.
#[derive(Clone, Debug, PartialEq)]
pub struct XpGem {
    pub x: f32,
    pub y: f32,
    pub value: u32,
    pub is_shard: bool,
}

/// Ephemeral floating damage number for visual feedback.
#[derive(Clone, Debug, PartialEq)]
pub struct CombatNumber {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub lifetime: f32,
    pub color: Color,
}

/// Full game simulation state.
#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub arena: Arena,
    pub player: Player,
    pub weapon: Weapon,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub gems: Vec<XpGem>,
    pub combat_numbers: Vec<CombatNumber>,
    pub wave: u32,
    pub wave_timer: f32,
    pub spawn_timer: f32,
    pub run_duration: f32,
    pub enemies_killed: u32,
    pub boss_spawned: bool,
    pub is_game_over: bool,
    pub pending_level_up: bool,
    pub current_draft: Option<UpgradeDraft>,
    pub rng: FastRng,
}

impl GameState {
    /// Initializes a new game instance.
    pub fn new(width: u16, height: u16, seed: u64) -> Self {
        let arena = Arena::new(width, height);
        let start_x = (arena.min_x + arena.max_x) * 0.5;
        let start_y = (arena.min_y + arena.max_y) * 0.5;

        Self {
            arena,
            player: Player::new(start_x, start_y),
            weapon: Weapon::gale_blaster(),
            enemies: Vec::with_capacity(64),
            projectiles: Vec::with_capacity(128),
            gems: Vec::with_capacity(128),
            combat_numbers: Vec::with_capacity(32),
            wave: 1,
            wave_timer: 0.0,
            spawn_timer: 0.0,
            run_duration: 0.0,
            enemies_killed: 0,
            boss_spawned: false,
            is_game_over: false,
            pending_level_up: false,
            current_draft: None,
            rng: FastRng::new(seed),
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }

    /// Applies a selected in-run upgrade to player or weapon stats.
    pub fn apply_upgrade(&mut self, upgrade: UpgradeKind) {
        match upgrade {
            UpgradeKind::DamageBoost => {
                self.weapon.damage *= 1.25;
            }
            UpgradeKind::FireRateBoost => {
                self.weapon.cooldown = (self.weapon.cooldown * 0.8).max(0.08);
            }
            UpgradeKind::MoveSpeed => {
                self.player.speed *= 1.20;
            }
            UpgradeKind::Multishot => {
                self.weapon.multishot += 1;
            }
            UpgradeKind::Piercing => {
                self.weapon.piercing += 1;
            }
            UpgradeKind::MaxHealth => {
                self.player.max_hp += 30.0;
                self.player.heal(30.0);
            }
            UpgradeKind::MagnetRange => {
                self.player.magnet_radius *= 1.5;
            }
        }
        self.pending_level_up = false;
        self.current_draft = None;
    }

    /// Advances the simulation by `dt` seconds.
    pub fn update(&mut self, dt: f32, input: GameInput) {
        if self.is_game_over || self.pending_level_up {
            return;
        }

        self.run_duration += dt;
        self.wave_timer += dt;

        // Wave progression every 40 seconds
        if self.wave_timer >= 40.0 {
            self.wave += 1;
            self.wave_timer = 0.0;
        }

        // 1. Player update
        self.player.apply_input(input);
        self.player.update(dt);
        let (cx, cy) = self.arena.clamp_player(self.player.x, self.player.y);
        self.player.x = cx;
        self.player.y = cy;

        // 2. Weapon update & automatic firing
        self.weapon.update(dt);
        let mut new_projs = self
            .weapon
            .try_fire_all((self.player.x, self.player.y), &self.enemies);
        self.projectiles.append(&mut new_projs);

        // 3. Enemy spawning
        self.spawn_timer += dt;
        let spawn_interval = (2.0 - (self.wave as f32 * 0.2)).max(0.4);
        if self.spawn_timer >= spawn_interval {
            self.spawn_timer = 0.0;
            self.spawn_wave_enemies();
        }

        // 4. Update enemies & collect enemy projectiles
        let player_pos = (self.player.x, self.player.y);
        for enemy in &mut self.enemies {
            if let Some(proj) = enemy.update(dt, player_pos) {
                self.projectiles.push(proj);
            }

            // Enemy touches player -> deal contact damage
            let dx = enemy.x - self.player.x;
            let dy = (enemy.y - self.player.y) * 1.5;
            if dx * dx + dy * dy < 2.25 {
                let contact_dmg = match enemy.kind {
                    EnemyKind::BossTempest => 30.0,
                    EnemyKind::Charger => 22.0,
                    _ => 12.0,
                };
                if self.player.take_damage(contact_dmg) {
                    self.combat_numbers.push(CombatNumber {
                        x: self.player.x,
                        y: self.player.y - 1.0,
                        text: format!("-{}", contact_dmg as u32),
                        lifetime: 0.8,
                        color: Color::Red,
                    });
                }
            }
        }

        // 5. Update projectiles & collisions
        let mut i = 0;
        while i < self.projectiles.len() {
            let expired = self.projectiles[i].update(dt);
            let out_of_bounds = self
                .arena
                .is_out_of_bounds(self.projectiles[i].pos.0, self.projectiles[i].pos.1);

            if expired || out_of_bounds {
                self.projectiles.swap_remove(i);
                continue;
            }

            let proj = &mut self.projectiles[i];

            if proj.is_enemy {
                // Enemy projectile hitting player
                let dx = proj.pos.0 - self.player.x;
                let dy = (proj.pos.1 - self.player.y) * 1.5;
                if dx * dx + dy * dy < 1.8 {
                    if self.player.take_damage(proj.damage) {
                        self.combat_numbers.push(CombatNumber {
                            x: self.player.x,
                            y: self.player.y - 1.0,
                            text: format!("-{}", proj.damage as u32),
                            lifetime: 0.8,
                            color: Color::Red,
                        });
                    }
                    self.projectiles.swap_remove(i);
                    continue;
                }
            } else {
                // Player projectile hitting enemy
                let mut hit_enemy = false;
                for enemy in &mut self.enemies {
                    if enemy.is_dead() {
                        continue;
                    }
                    let dx = proj.pos.0 - enemy.x;
                    let dy = (proj.pos.1 - enemy.y) * 1.5;
                    let hit_radius = if enemy.kind == EnemyKind::BossTempest {
                        3.5
                    } else {
                        1.5
                    };
                    if dx * dx + dy * dy < hit_radius {
                        enemy.take_damage(proj.damage);
                        self.combat_numbers.push(CombatNumber {
                            x: enemy.x,
                            y: enemy.y - 0.8,
                            text: format!("{}", proj.damage as u32),
                            lifetime: 0.6,
                            color: Color::Yellow,
                        });
                        hit_enemy = true;
                        break;
                    }
                }

                if hit_enemy {
                    if proj.piercing > 1 {
                        proj.piercing -= 1;
                    } else {
                        self.projectiles.swap_remove(i);
                        continue;
                    }
                }
            }

            i += 1;
        }

        // 6. Handle enemy death & drops
        let mut j = 0;
        while j < self.enemies.len() {
            if self.enemies[j].is_dead() {
                let dead = self.enemies.swap_remove(j);
                self.enemies_killed += 1;

                // Drop XP gem
                let xp_val = match dead.kind {
                    EnemyKind::BossTempest => 150,
                    EnemyKind::Charger => 25,
                    EnemyKind::Spitter => 20,
                    _ => 10,
                };
                self.gems.push(XpGem {
                    x: dead.x,
                    y: dead.y,
                    value: xp_val,
                    is_shard: false,
                });

                // Chance to drop Gale Shard (persistent currency)
                if self.rng.next_f32() < 0.25 || dead.kind == EnemyKind::BossTempest {
                    self.gems.push(XpGem {
                        x: dead.x + 0.5,
                        y: dead.y,
                        value: if dead.kind == EnemyKind::BossTempest {
                            25
                        } else {
                            2
                        },
                        is_shard: true,
                    });
                }
            } else {
                j += 1;
            }
        }

        // 7. Update collectibles (magnet to player)
        let magnet_sq = self.player.magnet_radius * self.player.magnet_radius;
        let mut k = 0;
        while k < self.gems.len() {
            let gem = &mut self.gems[k];
            let dx = self.player.x - gem.x;
            let dy = (self.player.y - gem.y) * 1.5;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < 1.44 {
                // Collected
                if gem.is_shard {
                    self.player.shards += gem.value;
                } else if self.player.add_xp(gem.value) {
                    self.pending_level_up = true;
                    self.current_draft = Some(UpgradeDraft::generate(&mut self.rng));
                }
                self.gems.swap_remove(k);
            } else if dist_sq < magnet_sq {
                // Vacuum toward player
                let dist = dist_sq.sqrt().max(0.01);
                gem.x += (dx / dist) * 18.0 * dt;
                gem.y += (dy / dist) * 18.0 * dt * 0.65;
                k += 1;
            } else {
                k += 1;
            }
        }

        // 8. Update combat floating numbers
        let mut m = 0;
        while m < self.combat_numbers.len() {
            self.combat_numbers[m].lifetime -= dt;
            self.combat_numbers[m].y -= dt * 1.5; // Float upwards
            if self.combat_numbers[m].lifetime <= 0.0 {
                self.combat_numbers.swap_remove(m);
            } else {
                m += 1;
            }
        }

        // 9. Check player death
        if self.player.hp <= 0.0 {
            self.is_game_over = true;
        }
    }

    fn spawn_wave_enemies(&mut self) {
        // Spawn around arena borders
        let side = self.rng.range_u32(0, 3);
        let (sx, sy) = match side {
            0 => (
                self.rng.range_f32(self.arena.min_x, self.arena.max_x),
                self.arena.min_y + 0.5,
            ), // Top
            1 => (
                self.rng.range_f32(self.arena.min_x, self.arena.max_x),
                self.arena.max_y - 0.5,
            ), // Bottom
            2 => (
                self.arena.min_x + 0.5,
                self.rng.range_f32(self.arena.min_y, self.arena.max_y),
            ), // Left
            _ => (
                self.arena.max_x - 0.5,
                self.rng.range_f32(self.arena.min_y, self.arena.max_y),
            ), // Right
        };

        // Select enemy kind based on wave
        let roll = self.rng.next_f32();
        let kind = if self.wave >= 5 && !self.boss_spawned {
            self.boss_spawned = true;
            EnemyKind::BossTempest
        } else if self.wave >= 3 && roll < 0.25 {
            EnemyKind::Charger
        } else if self.wave >= 2 && roll < 0.45 {
            EnemyKind::Spitter
        } else if roll < 0.70 {
            EnemyKind::Swarmer
        } else {
            EnemyKind::Drifter
        };

        self.enemies.push(Enemy::new(sx, sy, kind));
    }

    /// Renders the complete game scene to the terminal buffer.
    pub fn render(&self, buf: &mut Buffer) {
        buf.clear();

        // 1. Render HUD at top row
        self.render_hud(buf);

        // 2. Render arena borders
        self.arena.render_walls(buf);

        // 3. Render gems
        for gem in &self.gems {
            let gx = gem.x.round() as u16;
            let gy = gem.y.round() as u16;
            if gem.is_shard {
                buf.set_styled(gx, gy, '♦', Style::new(Color::Cyan, Color::Reset).bold());
            } else {
                buf.set_styled(gx, gy, '◆', Style::new(Color::Blue, Color::Reset).bold());
            }
        }

        // 4. Render enemies
        for enemy in &self.enemies {
            enemy.render(buf);
        }

        // 5. Render projectiles
        for proj in &self.projectiles {
            proj.render(buf);
        }

        // 6. Render player
        self.player.render(buf);

        // 7. Render combat numbers
        for num in &self.combat_numbers {
            let nx = num.x.round() as u16;
            let ny = num.y.round() as u16;
            buf.draw_str_styled(
                nx,
                ny,
                &num.text,
                Style::new(num.color, Color::Reset).bold(),
            );
        }

        // 8. Game over banner if fallen
        if self.is_game_over {
            self.render_game_over(buf);
        } else if let Some(ref draft) = self.current_draft {
            // Render level up upgrade selection modal
            draft.render_modal(buf);
        }
    }

    fn render_hud(&self, buf: &mut Buffer) {
        let w = buf.width();
        let hp_pct = (self.player.hp / self.player.max_hp).clamp(0.0, 1.0);
        let hp_bars = (hp_pct * 10.0).round() as usize;
        let hp_str = format!("HP [{}{}]", "=".repeat(hp_bars), " ".repeat(10 - hp_bars));

        let xp_pct = (self.player.xp as f32 / self.player.next_level_xp as f32).clamp(0.0, 1.0);
        let xp_bars = (xp_pct * 8.0).round() as usize;
        let xp_str = format!(
            "LVL {} [{}{}]",
            self.player.level,
            "#".repeat(xp_bars),
            " ".repeat(8 - xp_bars)
        );

        let time_secs = self.run_duration as u32;
        let hud_left = format!("{}  {}  WAVE {}", hp_str, xp_str, self.wave);
        let hud_right = format!(
            "SHARDS: {} | TIME {:02}:{:02} | ESC: exit",
            self.player.shards,
            time_secs / 60,
            time_secs % 60
        );

        buf.draw_str_styled(
            1,
            0,
            &hud_left,
            Style::new(Color::Green, Color::Reset).bold(),
        );
        if w > hud_right.len() as u16 + 2 {
            let rx = w - hud_right.len() as u16 - 1;
            buf.draw_str_styled(rx, 0, &hud_right, Style::new(Color::Cyan, Color::Reset));
        }
    }

    fn render_game_over(&self, buf: &mut Buffer) {
        let w = buf.width();
        let h = buf.height();
        let mid_y = h / 2;

        let banner = "══ RUN COMPLETE ══";
        let stats = format!(
            "Kills: {}  |  Level: {}  |  Shards: +{}",
            self.enemies_killed, self.player.level, self.player.shards
        );
        let prompt = "Press Enter or Esc to return";

        let bx = w.saturating_sub(banner.len() as u16) / 2;
        let sx = w.saturating_sub(stats.len() as u16) / 2;
        let px = w.saturating_sub(prompt.len() as u16) / 2;

        buf.draw_str_styled(
            bx,
            mid_y.saturating_sub(1),
            banner,
            Style::new(Color::Red, Color::Reset).bold(),
        );
        buf.draw_str_styled(sx, mid_y, &stats, Style::new(Color::White, Color::Reset));
        buf.draw_str_styled(
            px,
            mid_y.saturating_add(1),
            prompt,
            Style::new(Color::DarkGrey, Color::Reset),
        );
    }
}
