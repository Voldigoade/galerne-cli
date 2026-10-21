//! Master runner game state coordinating player physics, chunk scrolling,
//! collision avoidance, pickups, a compact HUD, and deterministic resumption.

use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

use super::action::{ActionState, PlayerInput};
use super::items::{FloatingIndicator, ItemKind};
use super::obstacles::ObstacleKind;
use super::player::{RunnerPlayer, PLAYER_WIDTH};
use super::region::{sector_for_distance, SectorKind};
use super::wind::{TailwindTier, WindKind};
use super::world::{Chunk, ChunkWorld, LandmarkKind, Walkway, CHECKPOINTS, CHUNK_WIDTH};
use crate::terminal::buffer::{Buffer, Color, Style};
use crate::ui::i18n::{t, Locale};

pub const BASE_FORWARD_SPEED: f32 = 18.0;
pub const MAX_FORWARD_SPEED: f32 = 36.0;
pub const SPEED_ACCELERATION_RATE: f32 = 0.15;
const INTERACTION_RADIUS: f32 = 6.0;
const INTERACTION_PROMPT_RADIUS: f32 = 12.0;
const INTERACTION_ACCEPT_RADIUS: f32 = 14.0;
const FLOW_WINDOW_SECS: f32 = 4.0;
const FLOW_STEP: f32 = 0.05;
const FLOW_MAX_STEPS: u8 = 4;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerSnapshot {
    pub distance: f32,
    pub speed: f32,
    pub shards_collected: u32,
    pub player_y: f32,
    pub player_action: ActionState,
    pub player_shields: u32,
    pub player_max_shields: u32,
    pub scroll_x: f32,
    pub seed: u64,
    pub run_duration: f32,
    pub onboarding_jump_done: bool,
    pub onboarding_slide_done: bool,
    #[serde(default)]
    pub relay_restored: bool,
    #[serde(default)]
    pub canal_gate_opened: bool,
    #[serde(default)]
    pub last_checkpoint_id: u32,
    #[serde(default)]
    pub last_checkpoint_distance: f32,
    #[serde(default)]
    pub companion_met: bool,
    #[serde(default)]
    pub secret_cache_found: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerState {
    pub player: RunnerPlayer,
    pub world: ChunkWorld,
    pub speed: f32,
    pub distance: f32,
    pub shards_collected: u32,
    pub flow_multiplier: f32,
    pub workshop_shard_multiplier: f32,
    pub flow_timer_secs: f32,
    pub flow_steps: u8,
    pub floating_indicators: Vec<FloatingIndicator>,
    pub onboarding_jump_done: bool,
    pub onboarding_slide_done: bool,
    pub run_duration: f32,
    pub transition_remaining_secs: Option<f32>,
    pub transition_success: bool,
    pub active_magnet_timer: f32,
    pub active_surge_timer: f32,
    #[serde(default)]
    pub relay_restored: bool,
    #[serde(default)]
    pub canal_gate_opened: bool,
    #[serde(default)]
    pub relay_activation_timer: Option<f32>,
    #[serde(default)]
    pub last_checkpoint_id: u32,
    #[serde(default)]
    pub last_checkpoint_distance: f32,
    #[serde(default)]
    pub companion_met: bool,
    #[serde(default)]
    pub secret_cache_found: bool,
    #[serde(default)]
    pub tailwind_tier: TailwindTier,
    #[serde(skip)]
    locale: Locale,
    #[serde(default)]
    pub unlocked_checkpoints: Vec<u32>,
    #[serde(default)]
    pub collected_relics: Vec<ItemKind>,
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
            shards_collected: 0,
            flow_multiplier: 1.0,
            workshop_shard_multiplier: 1.0,
            flow_timer_secs: 0.0,
            flow_steps: 0,
            floating_indicators: Vec::with_capacity(8),
            onboarding_jump_done: false,
            onboarding_slide_done: false,
            run_duration: 0.0,
            transition_remaining_secs: None,
            transition_success: true,
            active_magnet_timer: 0.0,
            active_surge_timer: 0.0,
            relay_restored: false,
            canal_gate_opened: false,
            relay_activation_timer: None,
            last_checkpoint_id: 0,
            last_checkpoint_distance: 0.0,
            companion_met: false,
            secret_cache_found: false,
            tailwind_tier: TailwindTier::None,
            locale: Locale::En,
            unlocked_checkpoints: vec![0],
            collected_relics: Vec::new(),
        }
    }

    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    pub fn resize(&mut self, new_w: usize, new_h: usize) {
        let old_ground_y = self.world.ground_y as f32;
        let player_world_x = self.world.scroll_x + self.player.x + PLAYER_WIDTH as f32 / 2.0;
        let old_platform_y = self.world.walkway_y_at(player_world_x);
        let on_platform =
            old_platform_y.is_some_and(|surface_y| self.player.is_on_ground(surface_y));
        let on_ground = self.player.is_on_ground(old_ground_y);
        self.world.resize(new_w, new_h);
        let ground_y = self.world.ground_y as f32;
        self.player.x = (new_w as f32 * 0.22).clamp(8.0, 25.0);
        if on_platform || on_ground {
            let new_world_x = self.world.scroll_x + self.player.x + PLAYER_WIDTH as f32 / 2.0;
            self.player.y = if on_platform {
                self.world.walkway_y_at(new_world_x).unwrap_or(ground_y)
            } else {
                ground_y
            };
        }
    }

    pub fn create_snapshot(&self) -> RunnerSnapshot {
        RunnerSnapshot {
            distance: self.distance,
            speed: self.speed,
            shards_collected: self.shards_collected,
            player_y: self.player.y,
            player_action: self.player.action,
            player_shields: self.player.shields,
            player_max_shields: self.player.max_shields,
            scroll_x: self.world.scroll_x,
            seed: self.world.seed,
            run_duration: self.run_duration,
            onboarding_jump_done: self.onboarding_jump_done,
            onboarding_slide_done: self.onboarding_slide_done,
            relay_restored: self.relay_restored,
            canal_gate_opened: self.canal_gate_opened,
            last_checkpoint_id: self.last_checkpoint_id,
            last_checkpoint_distance: self.last_checkpoint_distance,
            companion_met: self.companion_met,
            secret_cache_found: self.secret_cache_found,
        }
    }

    pub fn resume_snapshot(&mut self, snap: &RunnerSnapshot) {
        self.distance = snap.distance;
        self.speed = snap.speed;
        self.shards_collected = snap.shards_collected;
        self.player.y = snap.player_y;
        self.player.action = snap.player_action;
        self.player.shields = snap.player_shields;
        self.player.max_shields = snap.player_max_shields;
        self.world.scroll_x = snap.scroll_x;
        self.world.seed = snap.seed;
        self.run_duration = snap.run_duration;
        self.onboarding_jump_done = snap.onboarding_jump_done;
        self.onboarding_slide_done = snap.onboarding_slide_done;
        self.relay_restored = snap.relay_restored;
        self.canal_gate_opened = snap.canal_gate_opened;
        self.last_checkpoint_id = snap.last_checkpoint_id;
        self.last_checkpoint_distance = snap.last_checkpoint_distance;
        self.companion_met = snap.companion_met;
        self.secret_cache_found = snap.secret_cache_found;
    }

    pub fn start_completion_transition(&mut self, success: bool) {
        self.transition_remaining_secs = Some(0.30);
        self.transition_success = success;
    }

    pub fn is_transition_done(&self) -> bool {
        self.transition_remaining_secs
            .map(|t| t <= 0.0)
            .unwrap_or(false)
    }

    pub fn current_prompt(&self) -> Option<&'static str> {
        if let Some(prompt) = self.nearby_interaction_prompt() {
            Some(prompt)
        } else if !self.onboarding_jump_done && self.distance >= 15.0 && self.distance <= 35.0 {
            Some("jump_prompt")
        } else if !self.onboarding_slide_done && self.distance >= 50.0 && self.distance <= 80.0 {
            Some("slide_prompt")
        } else {
            None
        }
    }

    fn nearby_interaction_prompt(&self) -> Option<&'static str> {
        let abs_player_x = self.world.scroll_x + self.player.x;
        let mut nearest: Option<(f32, &'static str)> = None;

        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            for landmark in &chunk.landmarks {
                let abs_x = chunk_start_x + landmark.rel_x;
                let distance = (abs_player_x - abs_x).abs();
                if distance > INTERACTION_PROMPT_RADIUS {
                    continue;
                }

                let prompt = match &landmark.kind {
                    LandmarkKind::CompanionKael { spoken } if !spoken && !self.companion_met => {
                        Some("prompt_greet_companion")
                    }
                    LandmarkKind::SecretCache { opened } if !opened && !self.secret_cache_found => {
                        Some("prompt_search_cache")
                    }
                    LandmarkKind::RelayNodeConsole { restored }
                        if !restored && !self.relay_restored =>
                    {
                        Some("prompt_connect_line")
                    }
                    LandmarkKind::CanalSluice { opened } if !opened && !self.canal_gate_opened => {
                        Some("prompt_open_sluice")
                    }
                    _ => None,
                };

                if let Some(prompt) = prompt {
                    if nearest.is_none_or(|(best_distance, _)| distance < best_distance) {
                        nearest = Some((distance, prompt));
                    }
                }
            }
        }

        nearest.map(|(_, prompt)| prompt)
    }

    fn next_checkpoint(&self) -> Option<(&str, u32)> {
        let player_x = self.world.scroll_x + self.player.x;
        let mut nearest: Option<(f32, &str)> = None;

        for &(id, world_x, name_key) in &CHECKPOINTS {
            if self.unlocked_checkpoints.contains(&id) {
                continue;
            }

            let distance = world_x as f32 - player_x;
            if distance < 0.0 {
                continue;
            }
            if nearest.is_none_or(|(best_distance, _)| distance < best_distance) {
                nearest = Some((distance, name_key));
            }
        }

        nearest.map(|(distance, name)| (name, distance.ceil() as u32))
    }

    pub fn update(&mut self, dt: f32, input: Option<PlayerInput>) {
        if let Some(ref mut t_rem) = self.transition_remaining_secs {
            *t_rem -= dt;
            self.speed = (self.speed - (self.speed * 8.0 * dt)).max(0.0);
            if *t_rem <= 0.0 {
                return;
            }
        }

        if let Some(ref mut t_act) = self.relay_activation_timer {
            *t_act -= dt;
            if *t_act <= 0.0 {
                self.relay_activation_timer = None;
            }
        }

        self.run_duration += dt;
        let ground_y = self.world.ground_y as f32;
        let player_world_x = self.world.scroll_x + self.player.x + PLAYER_WIDTH as f32 / 2.0;
        let current_platform_y = self.world.walkway_y_at(player_world_x);

        if let Some(inp) = input {
            let input_surface_y = current_platform_y
                .filter(|surface_y| self.player.is_on_ground(*surface_y))
                .unwrap_or(ground_y);
            self.player.handle_input(inp, input_surface_y);
        }

        if self.active_magnet_timer > 0.0 {
            self.active_magnet_timer = (self.active_magnet_timer - dt).max(0.0);
        }
        if self.flow_timer_secs > 0.0 {
            self.flow_timer_secs = (self.flow_timer_secs - dt).max(0.0);
            if self.flow_timer_secs == 0.0 {
                self.flow_multiplier = 1.0;
                self.flow_steps = 0;
            }
        }
        if self.active_surge_timer > 0.0 {
            self.active_surge_timer = (self.active_surge_timer - dt).max(0.0);
        }

        let speed_mult = if self.active_surge_timer > 0.0 {
            1.5
        } else {
            1.0
        };
        let tailwind_bonus = self.tailwind_tier.speed_bonus();
        self.speed = (self.speed + SPEED_ACCELERATION_RATE * dt).min(MAX_FORWARD_SPEED);
        let effective_speed = (self.speed + tailwind_bonus) * speed_mult;

        let dx = effective_speed * dt;
        self.distance += dx;
        let previous_scroll_x = self.world.scroll_x;
        self.world.update(dx);
        let player_world_x = self.world.scroll_x + self.player.x + PLAYER_WIDTH as f32 / 2.0;
        let platform_y = self.world.walkway_y_at(player_world_x);
        self.player.update_with_platform(dt, ground_y, platform_y);

        self.floating_indicators.retain_mut(|ind| ind.update(dt));

        if !self.onboarding_jump_done && self.distance > 35.0 {
            self.onboarding_jump_done = true;
        }
        if !self.onboarding_slide_done && self.distance > 80.0 {
            self.onboarding_slide_done = true;
        }

        self.update_collisions_and_interactions(dt, input, previous_scroll_x);
    }

    fn update_collisions_and_interactions(
        &mut self,
        dt: f32,
        input: Option<PlayerInput>,
        previous_scroll_x: f32,
    ) {
        let (p_x1, p_y1, p_x2, p_y2) = self.player.bounding_box();
        let scroll_x = self.world.scroll_x;
        let abs_player_x = scroll_x + self.player.x;
        let interaction_pressed = input == Some(PlayerInput::Interact);

        let mut collected_items = [None; 8];
        let mut collected_count = 0;
        let mut collided = false;
        let mut clean_obstacles = 0;
        let player_action = self.player.action;
        let on_elevated_route = self.player.y < self.world.ground_y as f32 - 2.0;

        let magnet_radius = if self.active_magnet_timer > 0.0 {
            16.0
        } else {
            0.0
        };

        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            for wind in &chunk.wind_currents {
                let (w_x1, w_y1, w_x2, w_y2) = wind.bounding_box(chunk_start_x);
                let screen_w_x1 = w_x1 - scroll_x;
                let screen_w_x2 = w_x2 - scroll_x;

                if p_x1 < screen_w_x2 && p_x2 > screen_w_x1 && p_y1 < w_y2 && p_y2 > w_y1 {
                    match wind.kind {
                        WindKind::Updraft => {
                            if matches!(
                                self.player.action,
                                ActionState::Jumping { .. } | ActionState::Gliding { .. }
                            ) {
                                self.player.action = ActionState::Gliding {
                                    elapsed_secs: 0.0,
                                    vertical_velocity: -12.0 * wind.strength,
                                };
                            }
                        }
                        WindKind::TailwindGust => {
                            self.speed = (self.speed + 8.0 * dt).min(32.0);
                        }
                        WindKind::CrosswindChasm => {
                            if matches!(
                                self.player.action,
                                ActionState::Jumping { .. } | ActionState::Gliding { .. }
                            ) {
                                self.player.action = ActionState::Gliding {
                                    elapsed_secs: 0.0,
                                    vertical_velocity: -6.0 * wind.strength,
                                };
                                self.speed = (self.speed + 12.0 * dt).min(34.0);
                            }
                        }
                    }
                }
            }
        }

        for chunk in self.world.active_chunks_mut() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            for lm in &mut chunk.landmarks {
                if self.canal_gate_opened {
                    if let LandmarkKind::CanalSluice { opened } = &mut lm.kind {
                        *opened = true;
                    }
                }
                let abs_x = chunk_start_x + lm.rel_x;
                let screen_x = abs_x - scroll_x;

                let distance = (self.player.x - screen_x).abs();
                let within_range = match &lm.kind {
                    LandmarkKind::CheckpointPylon { .. } => distance < INTERACTION_RADIUS,
                    _ => interaction_pressed && distance < INTERACTION_ACCEPT_RADIUS,
                };

                if within_range {
                    match &mut lm.kind {
                        LandmarkKind::CheckpointPylon { id, name_key } => {
                            if !self.unlocked_checkpoints.contains(id) {
                                self.unlocked_checkpoints.push(*id);
                                self.last_checkpoint_id = *id;
                                self.last_checkpoint_distance = self.distance;
                                self.player.invulnerable_timer = 1.5;
                                let translated_name = checkpoint_name(name_key, self.locale);
                                self.floating_indicators.push(FloatingIndicator::new(
                                    self.player.x + 1.0,
                                    self.player.y - 4.5,
                                    crate::ui::i18n::format_message(
                                        "checkpoint_message",
                                        self.locale,
                                        &[("name", translated_name)],
                                    ),
                                ));
                            }
                        }
                        LandmarkKind::CompanionKael { spoken }
                            if interaction_pressed && !*spoken && !self.companion_met =>
                        {
                            *spoken = true;
                            self.companion_met = true;
                            self.floating_indicators.push(FloatingIndicator::new(
                                self.player.x + 1.0,
                                self.player.y - 5.5,
                                t("kael_encounter_dialogue", self.locale),
                            ));
                        }
                        LandmarkKind::SecretCache { opened }
                            if interaction_pressed && !*opened && !self.secret_cache_found =>
                        {
                            *opened = true;
                            self.secret_cache_found = true;
                            self.floating_indicators.push(FloatingIndicator::new(
                                self.player.x + 1.0,
                                self.player.y - 5.0,
                                t("cache_found", self.locale),
                            ));
                        }
                        LandmarkKind::RelayNodeConsole { restored }
                            if interaction_pressed && !*restored && !self.relay_restored =>
                        {
                            *restored = true;
                            self.relay_restored = true;
                            self.relay_activation_timer = Some(1.5);
                            self.floating_indicators.push(FloatingIndicator::new(
                                self.player.x + 1.0,
                                self.player.y - 5.5,
                                t("relay_restored", self.locale),
                            ));
                        }
                        LandmarkKind::CanalSluice { opened }
                            if interaction_pressed && !*opened && !self.canal_gate_opened =>
                        {
                            *opened = true;
                            self.canal_gate_opened = true;
                            self.floating_indicators.push(FloatingIndicator::new(
                                self.player.x + 1.0,
                                self.player.y - 5.5,
                                t("canal_flow_restored", self.locale),
                            ));
                        }
                        LandmarkKind::CompanionKael { .. }
                        | LandmarkKind::SecretCache { .. }
                        | LandmarkKind::RelayNodeConsole { .. }
                        | LandmarkKind::CanalSluice { .. } => {}
                    }
                }
            }
        }

        for chunk in self.world.active_chunks_mut() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            let cache_closed = cache_contents_locked(chunk, self.secret_cache_found);
            for item in &mut chunk.items {
                if item.collected {
                    continue;
                }
                if cache_closed
                    && matches!(
                        item.kind,
                        ItemKind::AncientCompass | ItemKind::InductionFilament
                    )
                {
                    continue;
                }
                let item_abs_x = chunk_start_x + item.rel_x;

                if (item_abs_x - abs_player_x).abs() < magnet_radius {
                    let dir = (abs_player_x - item_abs_x).signum();
                    item.rel_x += dir * 25.0 * dt;
                }

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

            for obs in &mut chunk.obstacles {
                let (o_x1, o_y1, o_x2, o_y2) = obs.bounding_box(chunk_start_x);
                let screen_o_x1 = o_x1 - scroll_x;
                let screen_o_x2 = o_x2 - scroll_x;

                if self.player.invulnerable_timer <= 0.0
                    && !collided
                    && p_x1 < screen_o_x2
                    && p_x2 > screen_o_x1
                    && p_y1 < o_y2
                    && p_y2 > o_y1
                {
                    collided = true;
                    break;
                }

                let previous_screen_o_x2 = o_x2 - previous_scroll_x;
                if !obs.cleared
                    && previous_screen_o_x2 >= self.player.x
                    && screen_o_x2 < self.player.x
                {
                    obs.cleared = true;
                    if !collided
                        && (on_elevated_route || action_clears_obstacle(player_action, obs.kind))
                    {
                        clean_obstacles += 1;
                    }
                }
            }
        }

        for item_kind in collected_items.iter().flatten() {
            self.apply_item_pickup(*item_kind);
        }

        if collided {
            self.handle_player_collision();
        } else {
            for _ in 0..clean_obstacles {
                self.record_clean_traversal();
            }
        }
    }

    fn record_clean_traversal(&mut self) {
        self.flow_steps = (self.flow_steps + 1).min(FLOW_MAX_STEPS);
        self.flow_multiplier = 1.0 + f32::from(self.flow_steps) * FLOW_STEP;
        self.flow_timer_secs = FLOW_WINDOW_SECS;
    }

    fn reset_flow(&mut self) {
        self.flow_multiplier = 1.0;
        self.flow_timer_secs = 0.0;
        self.flow_steps = 0;
    }

    fn apply_item_pickup(&mut self, kind: ItemKind) {
        match kind {
            ItemKind::GaleShard => {
                let multiplier = self.workshop_shard_multiplier + self.flow_multiplier - 1.0;
                let gain = (10.0 * multiplier) as u32;
                self.shards_collected += gain;
            }
            ItemKind::SpeedSurge => {
                self.active_surge_timer = 5.0;
                self.player.invulnerable_timer = 5.0;
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    t("pickup_speed", self.locale),
                ));
            }
            ItemKind::WindShield => {
                if self.player.shields < self.player.max_shields {
                    self.player.shields += 1;
                }
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    t("pickup_shield", self.locale),
                ));
            }
            ItemKind::ShardMagnet => {
                self.active_magnet_timer = 8.0;
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 4.5,
                    t("pickup_magnet", self.locale),
                ));
            }
            ItemKind::ResonanceCore => {
                self.collected_relics.push(ItemKind::ResonanceCore);
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 5.0,
                    crate::ui::i18n::format_message(
                        "item_acquired",
                        self.locale,
                        &[(
                            "name",
                            t("item_resonance_core_name", self.locale).to_owned(),
                        )],
                    ),
                ));
            }
            ItemKind::InductionFilament => {
                self.collected_relics.push(ItemKind::InductionFilament);
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 5.0,
                    crate::ui::i18n::format_message(
                        "item_acquired",
                        self.locale,
                        &[(
                            "name",
                            t("item_induction_filament_name", self.locale).to_owned(),
                        )],
                    ),
                ));
            }
            ItemKind::AncientCompass => {
                self.collected_relics.push(ItemKind::AncientCompass);
                self.floating_indicators.push(FloatingIndicator::new(
                    self.player.x + 1.0,
                    self.player.y - 5.0,
                    crate::ui::i18n::format_message(
                        "item_acquired",
                        self.locale,
                        &[(
                            "name",
                            t("item_ancient_compass_name", self.locale).to_owned(),
                        )],
                    ),
                ));
            }
        }
    }

    pub fn handle_player_collision(&mut self) {
        if self.player.shields > 0 {
            self.player.shields -= 1;
            self.player.invulnerable_timer = 1.2;
            self.reset_flow();
            self.floating_indicators.push(FloatingIndicator::new(
                self.player.x + 1.0,
                self.player.y - 4.5,
                t("shield_broken", self.locale),
            ));
        } else {
            // Safe respawn at last checkpoint without losing items or discoveries
            self.player.action = ActionState::Respawning {
                remaining_secs: 0.5,
            };
            self.player.invulnerable_timer = 2.0;
            self.player.shields = 1;
            self.distance = self.last_checkpoint_distance;
            self.world.scroll_x = (self.last_checkpoint_distance - self.player.x).max(0.0);
            self.player.y = self.world.ground_y as f32;
            self.reset_flow();
            self.floating_indicators.push(FloatingIndicator::new(
                self.player.x + 1.0,
                self.player.y - 4.5,
                t("respawn_checkpoint", self.locale),
            ));
        }
    }

    pub fn render(&self, buf: &mut Buffer, locale: Locale) {
        let width = buf.width() as usize;
        let height = buf.height() as usize;
        let ground_y = self.world.ground_y;

        self.world.parallax.render_backdrop(
            buf,
            ground_y,
            self.relay_restored,
            self.canal_gate_opened,
        );

        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            for walkway in &chunk.walkways {
                render_walkway(buf, chunk_start_x, self.world.scroll_x, ground_y, walkway);
            }
        }

        let current_sector = sector_for_distance(self.distance);
        if current_sector == SectorKind::InteriorGantry {
            let ceiling_y = ground_y.saturating_sub(8);
            if ceiling_y < height {
                for x in 0..width {
                    buf.set_char(
                        x as u16,
                        ceiling_y as u16,
                        '=',
                        Color::DarkGrey,
                        Color::Black,
                    );
                    if ceiling_y + 1 < height
                        && (x + (self.world.scroll_x as usize)).is_multiple_of(16)
                    {
                        buf.draw_str_styled(
                            x as u16,
                            (ceiling_y + 1) as u16,
                            "[v]",
                            Style::new(Color::Yellow, Color::Black),
                        );
                    }
                }
            }
        }

        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            for wind in &chunk.wind_currents {
                let screen_x = (chunk_start_x + wind.rel_x) - self.world.scroll_x;
                let base_y = wind.y.round() as usize;
                let sprites = wind.sprite_lines();
                for (line_idx, line) in sprites.iter().rev().enumerate() {
                    let draw_y = base_y.saturating_sub(line_idx + 1);
                    if screen_x >= -20.0 && (screen_x as usize) < width && draw_y < height {
                        buf.draw_str_styled(
                            screen_x.max(0.0) as u16,
                            draw_y as u16,
                            line,
                            Style::new(Color::DarkCyan, Color::Black),
                        );
                    }
                }
            }
        }

        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            for lm in &chunk.landmarks {
                let screen_x = (chunk_start_x + lm.rel_x) - self.world.scroll_x;
                let base_y = lm.y.round() as usize;
                let sprites = lm.sprite();
                for (line_idx, line) in sprites.iter().rev().enumerate() {
                    let draw_y = base_y.saturating_sub(line_idx);
                    if screen_x >= 0.0 && (screen_x as usize) < width && draw_y < height {
                        let color = match &lm.kind {
                            LandmarkKind::CheckpointPylon { .. } => Color::Green,
                            LandmarkKind::CompanionKael { .. } => Color::Yellow,
                            LandmarkKind::SecretCache { .. } => Color::Magenta,
                            LandmarkKind::RelayNodeConsole { .. } => Color::Cyan,
                            LandmarkKind::CanalSluice { .. } => Color::Cyan,
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

        for chunk in self.world.active_chunks() {
            let chunk_start_x = (chunk.index * CHUNK_WIDTH) as f32;
            let cache_closed = cache_contents_locked(chunk, self.secret_cache_found);

            for item in &chunk.items {
                if item.collected {
                    continue;
                }
                if cache_closed
                    && matches!(
                        item.kind,
                        ItemKind::AncientCompass | ItemKind::InductionFilament
                    )
                {
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
                        ItemKind::ResonanceCore => Color::Cyan,
                        ItemKind::InductionFilament => Color::White,
                        ItemKind::AncientCompass => Color::Yellow,
                    };
                    buf.draw_str_styled(
                        screen_x as u16,
                        screen_y as u16,
                        sprite,
                        Style::new(color, Color::Black).bold(),
                    );
                }
            }

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

        if let Some(t) = self.relay_activation_timer {
            let phase = ((1.5 - t) * 8.0) as usize;
            let wave_symbols = ["~", "*", "+", "#", "*"];
            for x in 0..width {
                let sym = wave_symbols[(x / 4 + phase) % wave_symbols.len()];
                buf.set_char(
                    x as u16,
                    ground_y.saturating_sub(1) as u16,
                    sym.chars().next().unwrap(),
                    Color::Cyan,
                    Color::Black,
                );
            }
        }

        let (player_sprite, sprite_left_offset) =
            self.player.current_sprite_for_terminal(width, height);
        let base_y = self.player.y.round() as i32;
        let sprite_x = self.player.x.round() as i32 - sprite_left_offset as i32;
        let body_color = if self.player.invulnerable_timer > 0.0 {
            Color::Cyan
        } else {
            Color::White
        };

        for (line_idx, line) in player_sprite.iter().rev().enumerate() {
            let draw_y = base_y - line_idx as i32;
            if !(0..height as i32).contains(&draw_y) {
                continue;
            }

            for (column, glyph) in line.chars().enumerate() {
                let draw_x = sprite_x + column as i32;
                if !(0..width as i32).contains(&draw_x) {
                    continue;
                }
                if glyph == ' ' {
                    continue;
                }
                let fg_color = match glyph {
                    '~' => Color::Cyan,
                    '#' => Color::Yellow,
                    _ => body_color,
                };
                buf.set_styled(
                    draw_x as u16,
                    draw_y as u16,
                    glyph,
                    Style::new(fg_color, Color::Black).bold(),
                );
            }
        }

        for ind in &self.floating_indicators {
            let draw_x = ind.x.round() as u16;
            let label_y = ind
                .y
                .min(base_y as f32 - player_sprite.len() as f32 - 1.0)
                .max(3.0);
            let draw_y = label_y.round() as u16;
            if (draw_x as usize) < width && (draw_y as usize) < height {
                buf.draw_str_styled(
                    draw_x,
                    draw_y,
                    &ind.text,
                    Style::new(Color::Yellow, Color::Black).bold(),
                );
            }
        }

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

        let tailwind_label = if self.tailwind_tier != TailwindTier::None {
            let key = match self.tailwind_tier {
                TailwindTier::None => unreachable!(),
                TailwindTier::Tier1 => "tailwind_1",
                TailwindTier::Tier2 => "tailwind_2",
                TailwindTier::Tier3 => "tailwind_3",
                TailwindTier::Tier4 => "tailwind_4",
            };
            format!(" │ {}", t(key, locale))
        } else {
            String::new()
        };

        let hud_text = format!(
            " {} {:04}m │ {} {:02} │ {} x{:.1} │ {} {:03} │ {}{}",
            hud_dist_label,
            self.distance as u32,
            hud_spd_label,
            self.speed as u32,
            hud_flow_label,
            self.flow_multiplier,
            hud_shards_label,
            self.shards_collected,
            shield_icons,
            tailwind_label,
        );

        buf.draw_str_styled(
            1,
            0,
            &hud_text,
            Style::new(Color::White, Color::Black).bold(),
        );

        if let Some((name_key, distance)) = self.next_checkpoint() {
            let waypoint = format!(
                "{}   ",
                crate::ui::i18n::format_message(
                    "next_checkpoint",
                    locale,
                    &[
                        ("name", checkpoint_name(name_key, locale)),
                        ("distance", distance.to_string()),
                    ],
                )
            );
            buf.draw_str_styled(
                1,
                1,
                &waypoint,
                Style::new(Color::DarkCyan, Color::Black).bold(),
            );
        }

        if let Some(prompt_key) = self.current_prompt() {
            let prompt_text = t(prompt_key, locale);
            let prompt_x = (width.saturating_sub(prompt_text.width())) / 2;
            buf.draw_str_styled(
                prompt_x as u16,
                2,
                prompt_text,
                Style::new(Color::Yellow, Color::Black).bold(),
            );
        }
    }
}

fn cache_contents_locked(chunk: &Chunk, cache_found: bool) -> bool {
    !cache_found
        && chunk
            .landmarks
            .iter()
            .any(|landmark| matches!(&landmark.kind, LandmarkKind::SecretCache { opened: false }))
}

fn action_clears_obstacle(action: ActionState, kind: ObstacleKind) -> bool {
    if matches!(action, ActionState::Dashing { .. }) {
        return true;
    }

    match kind {
        ObstacleKind::LowBarrier | ObstacleKind::Pylon => {
            matches!(
                action,
                ActionState::Jumping { .. } | ActionState::Gliding { .. }
            )
        }
        ObstacleKind::OverheadPipe => matches!(action, ActionState::Sliding { .. }),
    }
}

fn checkpoint_name(name_key: &str, locale: Locale) -> String {
    let key = match name_key {
        "cp_departure" => "cp_departure",
        "cp_shanty_outpost" => "cp_shanty_outpost",
        "cp_shaft_airlock" => "cp_shaft_airlock",
        "cp_high_roost" => "cp_high_roost",
        "cp_relay_node" => "cp_relay_node",
        "cp_canal_gardens" => "cp_canal_gardens",
        _ => return name_key.to_owned(),
    };
    t(key, locale).to_owned()
}

fn render_walkway(
    buf: &mut Buffer,
    chunk_start_x: f32,
    scroll_x: f32,
    ground_y: usize,
    walkway: &Walkway,
) {
    let deck_y = (ground_y as f32 - walkway.rise).round() as i32;
    if deck_y < 4 || deck_y >= buf.height() as i32 {
        return;
    }

    let start_x = chunk_start_x + walkway.start_rel_x - scroll_x;
    let end_x = chunk_start_x + walkway.end_rel_x - scroll_x;
    let first_x = (start_x.ceil() as i32).max(0);
    let end_x = (end_x.ceil() as i32).min(buf.width() as i32);

    for screen_x in first_x..end_x {
        let world_x = screen_x as f32 + scroll_x;
        let local_x = (world_x - chunk_start_x - walkway.start_rel_x).floor() as usize;
        let anchor = local_x.is_multiple_of(10);
        if anchor {
            for cable_y in (deck_y - 4)..deck_y {
                buf.set_char(
                    screen_x as u16,
                    cable_y as u16,
                    '|',
                    Color::DarkGrey,
                    Color::Black,
                );
            }
        }
        buf.set_char(
            screen_x as u16,
            deck_y as u16,
            if anchor { '+' } else { '=' },
            Color::DarkGrey,
            Color::Black,
        );
    }
}
