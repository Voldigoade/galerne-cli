//! Master runner game state coordinating player physics, chunk scrolling,
//! collision avoidance, instant pickups, minimal HUD, and deterministic resumption.

use serde::{Deserialize, Serialize};

use super::action::{ActionState, PlayerInput};
use super::items::{FloatingIndicator, ItemKind};
use super::player::RunnerPlayer;
use super::world::{ChunkWorld, CHUNK_WIDTH};
use crate::terminal::buffer::{Buffer, Color, Style};
use crate::ui::i18n::{t, Locale};

pub const BASE_FORWARD_SPEED: f32 = 18.0;
pub const MAX_FORWARD_SPEED: f32 = 36.0;
pub const SPEED_ACCELERATION_RATE: f32 = 0.15; // Slowly ramps up over time

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerSnapshot {
    pub distance: f32,
    pub speed: f32,
    pub score: u32,
    pub shards_collected: u32,
    pub flow_multiplier: f32,
    pub player_y: f32,
    pub player_action: ActionState,
    pub player_shields: u32,
    pub player_max_shields: u32,
    pub scroll_x: f32,
    pub seed: u64,
    pub run_duration: f32,
    pub onboarding_jump_done: bool,
    pub onboarding_slide_done: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerState {
    pub player: RunnerPlayer,
    pub world: ChunkWorld,
    pub speed: f32,
    pub distance: f32,
    pub score: u32,
    pub shards_collected: u32,
    pub flow_multiplier: f32,
    pub floating_indicators: Vec<FloatingIndicator>,
    pub onboarding_jump_done: bool,
    pub onboarding_slide_done: bool,
    pub is_game_over: bool,
    pub run_duration: f32,
    pub transition_remaining_secs: Option<f32>,
    pub transition_success: bool,
    pub active_magnet_timer: f32,
    pub active_multiplier_timer: f32,
    pub active_surge_timer: f32,
}

impl RunnerState {
    pub fn new(viewport_width: usize, viewport_height: usize, seed: u64) -> Self {
        let ground_y = viewport_height.saturating_sub(5).max(8);
        let player_x = (viewport_width as f32 * 0.22).clamp(8.0, 25.0);

        Self {
            player: RunnerPlayer::new(player_x, ground_y as f32),
            world: ChunkWorld::new(seed, viewport_width, viewport_height),
            speed: BASE_FORWARD_SPEED,
            distance: 0.0,
            score: 0,
            shards_collected: 0,
            flow_multiplier: 1.0,
            floating_indicators: Vec::with_capacity(8),
            onboarding_jump_done: false,
            onboarding_slide_done: false,
            is_game_over: false,
            run_duration: 0.0,
            transition_remaining_secs: None,
            transition_success: true,
            active_magnet_timer: 0.0,
            active_multiplier_timer: 0.0,
            active_surge_timer: 0.0,
        }
    }

    pub fn resize(&mut self, new_w: usize, new_h: usize) {
        self.world.resize(new_w, new_h);
        let ground_y = self.world.ground_y as f32;
        self.player.x = (new_w as f32 * 0.22).clamp(8.0, 25.0);
        if self.player.is_on_ground(ground_y) {
            self.player.y = ground_y;
        }
    }

    pub fn create_snapshot(&self) -> RunnerSnapshot {
        RunnerSnapshot {
            distance: self.distance,
            speed: self.speed,
            score: self.score,
            shards_collected: self.shards_collected,
            flow_multiplier: self.flow_multiplier,
            player_y: self.player.y,
            player_action: self.player.action,
            player_shields: self.player.shields,
            player_max_shields: self.player.max_shields,
            scroll_x: self.world.scroll_x,
            seed: self.world.seed,
            run_duration: self.run_duration,
            onboarding_jump_done: self.onboarding_jump_done,
            onboarding_slide_done: self.onboarding_slide_done,
        }
    }

    pub fn resume_snapshot(&mut self, snap: &RunnerSnapshot) {
        self.distance = snap.distance;
        self.speed = snap.speed;
        self.score = snap.score;
        self.shards_collected = snap.shards_collected;
        self.flow_multiplier = snap.flow_multiplier;
        self.player.y = snap.player_y;
        self.player.action = snap.player_action;
        self.player.shields = snap.player_shields;
        self.player.max_shields = snap.player_max_shields;
        self.world.scroll_x = snap.scroll_x;
        self.world.seed = snap.seed;
        self.run_duration = snap.run_duration;
        self.onboarding_jump_done = snap.onboarding_jump_done;
        self.onboarding_slide_done = snap.onboarding_slide_done;
    }

    pub fn start_completion_transition(&mut self, success: bool) {
        self.transition_remaining_secs = Some(0.30); // 300ms rapid deceleration
        self.transition_success = success;
    }

    pub fn is_transition_done(&self) -> bool {
        self.transition_remaining_secs
            .map(|t| t <= 0.0)
            .unwrap_or(false)
    }

    pub fn current_prompt(&self) -> Option<&'static str> {
        // Onboarding prompts
        if !self.onboarding_jump_done && self.distance >= 15.0 && self.distance <= 35.0 {
            Some("jump_prompt")
        } else if !self.onboarding_slide_done && self.distance >= 50.0 && self.distance <= 80.0 {
            Some("slide_prompt")
        } else {
            None
        }
    }

    pub fn update(&mut self, dt: f32, input: Option<PlayerInput>) {
        if self.is_game_over {
            return;
        }

        // Handle completion transition deceleration
        if let Some(ref mut t_rem) = self.transition_remaining_secs {
            *t_rem -= dt;
            self.speed = (self.speed - (self.speed * 8.0 * dt)).max(0.0);
            if *t_rem <= 0.0 {
                return;
            }
        }

        self.run_duration += dt;
        let ground_y = self.world.ground_y as f32;

        // Process discrete input
        if let Some(inp) = input {
            self.player.handle_input(inp, ground_y);
        }

        // Update timers
        if self.active_magnet_timer > 0.0 {
            self.active_magnet_timer = (self.active_magnet_timer - dt).max(0.0);
        }
        if self.active_multiplier_timer > 0.0 {
            self.active_multiplier_timer = (self.active_multiplier_timer - dt).max(0.0);
            if self.active_multiplier_timer <= 0.0 {
                self.flow_multiplier = 1.0;
            }
        }
        if self.active_surge_timer > 0.0 {
            self.active_surge_timer = (self.active_surge_timer - dt).max(0.0);
        }

        // Calculate dynamic cruising speed
        let speed_mult = if self.active_surge_timer > 0.0 {
            1.5
        } else {
            1.0
        };
        self.speed = (self.speed + SPEED_ACCELERATION_RATE * dt).min(MAX_FORWARD_SPEED);
        let effective_speed = self.speed * speed_mult;

        // Advance world and player
        let dx = effective_speed * dt;
        self.distance += dx;
        self.world.update(dx);
        self.player.update(dt, ground_y);

        // Update floating indicators
        self.floating_indicators.retain_mut(|ind| ind.update(dt));

        // Check onboarding completion
        if !self.onboarding_jump_done && self.distance > 35.0 {
            self.onboarding_jump_done = true;
        }
        if !self.onboarding_slide_done && self.distance > 80.0 {
            self.onboarding_slide_done = true;
        }

        // Collision checking
        let (p_x1, p_y1, p_x2, p_y2) = self.player.bounding_box();
        let scroll_x = self.world.scroll_x;
        let abs_player_x = scroll_x + self.player.x;

        // Magnet attraction
        let has_magnet = self.active_magnet_timer > 0.0;
        let magnet_radius = if has_magnet { 18.0 } else { 3.5 };

        let mut collected_items = [None; 8];
        let mut collected_count = 0;
        let mut collided = false;

        for chunk in self.world.active_chunks_mut() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;

            // 1. Pickups
            for item in &mut chunk.items {
                if item.collected {
                    continue;
                }
                let item_abs_x = chunk_start_x + item.rel_x;

                // Magnetic pull
                if (item_abs_x - abs_player_x).abs() < magnet_radius {
                    let dir = (abs_player_x - item_abs_x).signum();
                    item.rel_x += dir * 25.0 * dt;
                }

                // Overlap check
                let screen_x1 = item_abs_x - scroll_x;
                let screen_x2 = screen_x1 + item.sprite().len() as f32;
                if self.player.x < screen_x2
                    && self.player.x + 5.0 > screen_x1
                    && self.player.y >= item.y - 1.5
                    && self.player.y - 4.0 <= item.y + 1.5
                {
                    item.collected = true;
                    if collected_count < collected_items.len() {
                        collected_items[collected_count] = Some(item.kind);
                        collected_count += 1;
                    }
                }
            }

            // 2. Obstacles
            if self.player.invulnerable_timer <= 0.0 && !collided {
                for obs in &chunk.obstacles {
                    let (o_x1, o_y1, o_x2, o_y2) = obs.bounding_box(chunk_start_x);
                    let screen_o_x1 = o_x1 - scroll_x;
                    let screen_o_x2 = o_x2 - scroll_x;

                    if p_x1 < screen_o_x2 && p_x2 > screen_o_x1 && p_y1 < o_y2 && p_y2 > o_y1 {
                        collided = true;
                        break;
                    }
                }
            }
        }

        for item_kind in collected_items.iter().flatten() {
            self.apply_item_pickup(*item_kind);
        }

        if collided {
            self.handle_player_collision();
        }
    }

    fn apply_item_pickup(&mut self, kind: ItemKind) {
        match kind {
            ItemKind::GaleShard => {
                let gain = (10.0 * self.flow_multiplier) as u32;
                self.shards_collected += gain;
                self.score += gain * 5;
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    format!("+{} SHARDS", gain),
                ));
            }
            ItemKind::SpeedSurge => {
                self.active_surge_timer = 5.0;
                self.player.invulnerable_timer = 5.0;
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    ">> SPEED SURGE",
                ));
            }
            ItemKind::WindShield => {
                if self.player.shields < self.player.max_shields {
                    self.player.shields += 1;
                }
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    "+1 SHIELD (O)",
                ));
            }
            ItemKind::ShardMagnet => {
                self.active_magnet_timer = 8.0;
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    "[+] MAGNET ATTRACTOR",
                ));
            }
            ItemKind::FlowMultiplier => {
                self.flow_multiplier = 2.0;
                self.active_multiplier_timer = 8.0;
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    "x2 FLOW MULTIPLIER",
                ));
            }
        }
    }

    fn handle_player_collision(&mut self) {
        if self.player.shields > 0 {
            self.player.shields -= 1;
            self.player.invulnerable_timer = 1.2; // 1.2s post-hit invulnerability
            self.flow_multiplier = 1.0;
            self.floating_indicators.push(FloatingIndicator::new(
                self.player.x + 1.0,
                self.player.y - 4.5,
                "SHIELD BROKEN",
            ));
        } else {
            // Defeat without shields
            self.is_game_over = true;
            self.floating_indicators.push(FloatingIndicator::new(
                self.player.x + 1.0,
                self.player.y - 4.5,
                "CRITICAL IMPACT",
            ));
        }
    }

    /// Renders frame to double-buffered TerminalBuffer.
    pub fn render(&self, buf: &mut Buffer, locale: Locale) {
        let width = buf.width() as usize;
        let height = buf.height() as usize;
        let ground_y = self.world.ground_y;

        // 1. Render multi-layer parallax backdrop (sky, skyline, midground wires, rails)
        self.world.parallax.render_backdrop(buf, ground_y);

        // 2. Render chunk obstacles and items
        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;

            // Items
            for item in &chunk.items {
                if item.collected {
                    continue;
                }
                let screen_x = (chunk_start_x + item.rel_x) - self.world.scroll_x;
                let screen_y = item.y.round() as usize;

                if screen_x >= 0.0 && (screen_x as usize) < width && screen_y < height {
                    let sprite = item.sprite();
                    let color = match item.kind {
                        ItemKind::GaleShard => Color::Yellow,
                        ItemKind::SpeedSurge => Color::Cyan,
                        ItemKind::WindShield => Color::Green,
                        ItemKind::ShardMagnet => Color::Blue,
                        ItemKind::FlowMultiplier => Color::Magenta,
                    };
                    buf.draw_str_styled(
                        screen_x as u16,
                        screen_y as u16,
                        sprite,
                        Style::new(color, Color::Black).bold(),
                    );
                }
            }

            // Obstacles
            for obs in &chunk.obstacles {
                let screen_x = (chunk_start_x + obs.rel_x) - self.world.scroll_x;
                let base_y = match obs.kind {
                    super::obstacles::ObstacleKind::LowBarrier
                    | super::obstacles::ObstacleKind::Pylon => obs.y.round() as usize,
                    super::obstacles::ObstacleKind::OverheadPipe => (obs.y - 2.0).round() as usize,
                };

                let sprite_lines = obs.sprite();
                for (line_idx, line) in sprite_lines.iter().rev().enumerate() {
                    let draw_y = base_y.saturating_sub(line_idx);
                    if screen_x >= 0.0 && (screen_x as usize) < width && draw_y < height {
                        let color = match obs.kind {
                            super::obstacles::ObstacleKind::LowBarrier => Color::Red,
                            super::obstacles::ObstacleKind::OverheadPipe => Color::Cyan,
                            super::obstacles::ObstacleKind::Pylon => Color::Yellow,
                        };
                        buf.draw_str_styled(
                            screen_x as u16,
                            draw_y as u16,
                            line,
                            Style::new(color, Color::Black).bold(),
                        );
                    }
                }
            }
        }

        // 3. Render Player multi-cell ASCII sprite
        let player_sprite = self.player.current_sprite();
        let base_y = self.player.y.round() as usize;
        let is_invulnerable = self.player.invulnerable_timer > 0.0;

        for (line_idx, line) in player_sprite.iter().rev().enumerate() {
            let draw_y = base_y.saturating_sub(line_idx);
            if draw_y < height {
                let fg_color = if is_invulnerable {
                    Color::Cyan
                } else if self.player.shields > 0 {
                    Color::Green
                } else {
                    Color::White
                };
                buf.draw_str_styled(
                    self.player.x as u16,
                    draw_y as u16,
                    line,
                    Style::new(fg_color, Color::Black).bold(),
                );
            }
        }

        // 4. Render floating text indicators
        for ind in &self.floating_indicators {
            let draw_x = ind.x.round() as u16;
            let draw_y = ind.y.round() as u16;
            if (draw_x as usize) < width && (draw_y as usize) < height {
                buf.draw_str_styled(
                    draw_x,
                    draw_y,
                    &ind.text,
                    Style::new(Color::Yellow, Color::Black).bold(),
                );
            }
        }

        // 5. Minimal unobtrusive Top HUD
        // [ DIST 240m | SPD 22 | FLOW x2.5 | SHARDS 38 | SHIELDS (O)(O) ]
        let hud_dist_label = t("hud_distance", locale);
        let hud_spd_label = t("hud_speed", locale);
        let hud_flow_label = t("hud_multiplier", locale);
        let hud_shards_label = t("hud_shards", locale);

        let shield_icons = match self.player.shields {
            0 => "[-]",
            1 => "(O)",
            2 => "(O)(O)",
            _ => "(O)(O)(O)",
        };

        let hud_text = format!(
            " {} {:04}m │ {} {:02} │ {} x{:.1} │ {} {:03} │ {}",
            hud_dist_label,
            self.distance as u32,
            hud_spd_label,
            self.speed as u32,
            hud_flow_label,
            self.flow_multiplier,
            hud_shards_label,
            self.shards_collected,
            shield_icons
        );

        buf.draw_str_styled(
            1,
            0,
            &hud_text,
            Style::new(Color::White, Color::DarkGrey).bold(),
        );

        // 6. Onboarding Prompt (if active)
        if let Some(prompt_key) = self.current_prompt() {
            let prompt_text = t(prompt_key, locale);
            let px = (width.saturating_sub(prompt_text.len())) / 2;
            buf.draw_str_styled(
                px as u16,
                (ground_y.saturating_sub(9)) as u16,
                prompt_text,
                Style::new(Color::Yellow, Color::Black).bold(),
            );
        }
    }
}
