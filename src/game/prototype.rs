use std::time::Duration;

use crossterm::event::{KeyCode, KeyEventKind};

const PLAYER_WIDTH: f32 = 20.0;
const PLAYER_HEIGHT: f32 = 24.0;
const FALLBACK_KEY_HOLD: Duration = Duration::from_millis(180);
const GRAVITY: f32 = 520.0;
const WALK_ACCELERATION: f32 = 360.0;
const WALK_SPEED: f32 = 92.0;
const GROUND_FRICTION: f32 = 560.0;

#[cfg(test)]
mod tests {
    use super::{InputState, PrototypeGame};
    use crossterm::event::{KeyCode, KeyEventKind};
    use std::time::Duration;

    const STEP: f32 = 1.0 / 60.0;

    #[test]
    fn directional_motion_uses_physics_time_and_brakes_after_release() {
        let mut game = PrototypeGame::new(236, 53);
        let mut input = InputState::default();
        let start = game.player_x();
        input.key_event(KeyCode::Right, KeyEventKind::Press, true, Duration::ZERO);
        for _ in 0..30 {
            game.update(STEP, &mut input, Duration::from_millis(500));
        }
        let after_hold = game.player_x();
        input.key_event(
            KeyCode::Right,
            KeyEventKind::Release,
            true,
            Duration::from_millis(501),
        );
        for _ in 0..60 {
            game.update(STEP, &mut input, Duration::from_millis(600));
        }

        assert!(after_hold > start + 8.0);
        assert!(game.horizontal_speed().abs() < 1.0);
        assert!(game.player_x() - after_hold < 18.0);
    }

    #[test]
    fn jump_rises_and_returns_to_the_platform() {
        let mut game = PrototypeGame::new(80, 24);
        let mut input = InputState::default();
        let start_y = game.player_foot_y();
        input.key_event(
            KeyCode::Char(' '),
            KeyEventKind::Press,
            true,
            Duration::ZERO,
        );

        for tick in 0..120 {
            game.update(STEP, &mut input, Duration::from_millis(tick * 17));
        }

        assert!(game.highest_jump_y() < start_y - 8.0);
        assert!(game.ground_y() - game.highest_jump_y() <= game.viewport_height() * 0.48);
        assert!(game.is_grounded());
        assert_eq!(game.player_foot_y(), start_y);
    }

    #[test]
    fn checkpoint_activates_before_the_first_hazard_and_respawns_safely() {
        let mut game = PrototypeGame::new(80, 24);
        let mut input = InputState::default();
        input.key_event(KeyCode::Right, KeyEventKind::Press, true, Duration::ZERO);

        for tick in 0..40 {
            game.update(STEP, &mut input, Duration::from_millis(tick * 17));
        }

        assert!(game.checkpoint_active());
        assert!(game.respawn_x + game.player_width() <= game.obstacle_x());
    }

    #[test]
    fn relay_requires_proximity_and_activates_on_interaction() {
        let mut game = PrototypeGame::new(236, 53);
        let mut input = InputState::default();
        input.key_event(
            KeyCode::Char('e'),
            KeyEventKind::Press,
            true,
            Duration::ZERO,
        );
        game.update(STEP, &mut input, Duration::ZERO);
        assert!(!game.relay_active());

        game.player_x = game.relay_x - 10.0 - 20.0;
        input.key_event(
            KeyCode::Char('e'),
            KeyEventKind::Press,
            true,
            Duration::from_millis(8),
        );
        game.update(STEP, &mut input, Duration::from_millis(8));
        assert!(!game.relay_active());

        game.player_x = game.relay_x - 10.0;
        input.key_event(
            KeyCode::Char('e'),
            KeyEventKind::Press,
            true,
            Duration::from_millis(17),
        );
        game.update(STEP, &mut input, Duration::from_millis(17));
        assert!(game.relay_active());
    }

    #[test]
    fn falling_into_a_gap_returns_to_the_checkpoint_without_losing_it() {
        let mut game = PrototypeGame::new(80, 24);
        game.checkpoint_active = true;
        game.respawn_x = 12.0;
        game.player_x = game.gap_start() + 3.5;
        game.grounded = false;
        game.player_foot_y = game.ground_y;
        game.velocity_y = 80.0;
        let mut input = InputState::default();

        for tick in 0..120 {
            game.update(STEP, &mut input, Duration::from_millis(tick * 17));
        }

        assert!(game.is_grounded());
        assert!(game.checkpoint_active());
        assert_eq!(game.player_x(), game.respawn_x);
        assert!(game.player_x() + game.player_width() <= game.obstacle_x());
    }

    #[test]
    fn keyboard_repeat_cadence_does_not_change_physics_speed() {
        let mut sparse = PrototypeGame::new(236, 53);
        let mut frequent = PrototypeGame::new(236, 53);
        let mut sparse_input = InputState::default();
        let mut frequent_input = InputState::default();
        sparse_input.key_event(KeyCode::Right, KeyEventKind::Press, true, Duration::ZERO);
        frequent_input.key_event(KeyCode::Right, KeyEventKind::Press, true, Duration::ZERO);

        for tick in 0..60 {
            let now = Duration::from_millis(tick * 17);
            if tick % 15 == 0 {
                sparse_input.key_event(KeyCode::Right, KeyEventKind::Repeat, true, now);
            }
            if tick % 3 == 0 {
                frequent_input.key_event(KeyCode::Right, KeyEventKind::Repeat, true, now);
            }
            sparse.update(STEP, &mut sparse_input, now);
            frequent.update(STEP, &mut frequent_input, now);
        }

        assert_eq!(sparse.horizontal_speed(), frequent.horizontal_speed());
        assert_eq!(sparse.player_x(), frequent.player_x());
    }

    #[test]
    fn unsupported_key_release_falls_back_to_a_bounded_input_hold() {
        let mut game = PrototypeGame::new(236, 53);
        let mut input = InputState::default();
        input.key_event(KeyCode::Right, KeyEventKind::Press, false, Duration::ZERO);
        for tick in 0..60 {
            game.update(STEP, &mut input, Duration::from_millis(tick * 17));
        }

        assert!(game.horizontal_speed().abs() < 1.0);
        assert!(game.player_x() > 23.0);
        assert!(game.player_x() < 40.0);
    }

    #[test]
    fn solid_hazard_blocks_ground_travel_until_the_player_jumps_over_it() {
        let mut game = PrototypeGame::new(80, 24);
        let mut input = InputState::default();
        input.key_event(KeyCode::Right, KeyEventKind::Press, true, Duration::ZERO);
        for tick in 0..90 {
            game.update(STEP, &mut input, Duration::from_millis(tick * 17));
        }
        let obstacle_x = game.obstacle_x();
        assert!(game.player_x() + game.player_width() <= obstacle_x + 0.1);

        input.key_event(
            KeyCode::Char(' '),
            KeyEventKind::Press,
            true,
            Duration::from_secs(2),
        );
        for tick in 90..120 {
            game.update(STEP, &mut input, Duration::from_millis(tick * 17));
        }
        assert!(game.player_x() > obstacle_x + 11.0);
    }

    #[test]
    fn the_first_jump_reaches_the_relay_at_both_supported_viewport_sizes() {
        for (width, height) in [(80, 24), (236, 53)] {
            let mut game = PrototypeGame::new(width, height);
            let mut input = InputState::default();
            input.key_event(KeyCode::Right, KeyEventKind::Press, true, Duration::ZERO);

            let mut elapsed = Duration::ZERO;
            for _ in 0..240 {
                game.update(STEP, &mut input, elapsed);
                elapsed += Duration::from_millis(17);
                if game.horizontal_speed() == 0.0
                    && (game.player_x() + game.player_width() - game.obstacle_x()).abs() < 0.1
                {
                    break;
                }
            }

            assert!(game.checkpoint_active());
            assert!(game.is_grounded());
            input.key_event(KeyCode::Char(' '), KeyEventKind::Press, true, elapsed);
            let mut released_right = false;
            for tick in 0..42 {
                if tick == 1 {
                    input.key_event(KeyCode::Char(' '), KeyEventKind::Release, true, elapsed);
                }
                game.update(STEP, &mut input, elapsed);
                elapsed += Duration::from_millis(17);
                if !released_right && game.player_x() > game.gap_end() + 1.0 {
                    input.key_event(KeyCode::Right, KeyEventKind::Release, true, elapsed);
                    released_right = true;
                }
            }

            assert!(
                game.player_x() > game.gap_end(),
                "{width}x{height}: player {} did not clear gap end {}",
                game.player_x(),
                game.gap_end()
            );
            assert!(game.near_relay());
            input.key_event(KeyCode::Char('e'), KeyEventKind::Press, true, elapsed);
            game.update(STEP, &mut input, elapsed);
            assert!(game.relay_active());
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pose {
    #[default]
    Idle,
    Run,
    Turn,
    Jump,
    Fall,
    Land,
    Slide,
    Dash,
    Glide,
    Interact,
    Hit,
    Respawn,
}

#[derive(Clone, Debug, Default)]
pub struct InputState {
    left: bool,
    right: bool,
    jump: bool,
    drop: bool,
    dash: bool,
    jump_pressed: bool,
    dash_pressed: bool,
    interact_pressed: bool,
    release_events: bool,
    last_activity: Option<Duration>,
}

impl InputState {
    pub fn key_event(
        &mut self,
        key: KeyCode,
        kind: KeyEventKind,
        release_events: bool,
        at: Duration,
    ) {
        self.release_events = release_events;
        let pressed = kind != KeyEventKind::Release;
        if pressed {
            self.last_activity = Some(at);
        }

        match normalized_key(key) {
            ControlKey::Left => self.left = pressed,
            ControlKey::Right => self.right = pressed,
            ControlKey::Jump => {
                self.jump = pressed;
                if kind == KeyEventKind::Press {
                    self.jump_pressed = true;
                }
            }
            ControlKey::Drop => self.drop = pressed,
            ControlKey::Dash => {
                self.dash = pressed;
                if kind == KeyEventKind::Press {
                    self.dash_pressed = true;
                }
            }
            ControlKey::Interact if kind == KeyEventKind::Press => {
                self.interact_pressed = true;
            }
            _ => {}
        }
    }

    fn direction(&self) -> f32 {
        match (self.left, self.right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        }
    }

    fn refresh_fallback(&mut self, at: Duration) {
        if !self.release_events
            && self
                .last_activity
                .is_some_and(|last| at.saturating_sub(last) > FALLBACK_KEY_HOLD)
        {
            self.clear_held();
        }
    }

    fn clear_held(&mut self) {
        self.left = false;
        self.right = false;
        self.jump = false;
        self.drop = false;
        self.dash = false;
    }

    fn take_jump_pressed(&mut self) -> bool {
        std::mem::take(&mut self.jump_pressed)
    }

    fn take_dash_pressed(&mut self) -> bool {
        std::mem::take(&mut self.dash_pressed)
    }

    fn take_interact_pressed(&mut self) -> bool {
        std::mem::take(&mut self.interact_pressed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlKey {
    Left,
    Right,
    Jump,
    Drop,
    Dash,
    Interact,
    Other,
}

fn normalized_key(key: KeyCode) -> ControlKey {
    match key {
        KeyCode::Left | KeyCode::Char('a' | 'A') => ControlKey::Left,
        KeyCode::Right | KeyCode::Char('d' | 'D') => ControlKey::Right,
        KeyCode::Up | KeyCode::Char('w' | 'W' | ' ') => ControlKey::Jump,
        KeyCode::Down | KeyCode::Char('s' | 'S') => ControlKey::Drop,
        KeyCode::Char('x' | 'X') => ControlKey::Dash,
        KeyCode::Char('e' | 'E') | KeyCode::Enter => ControlKey::Interact,
        _ => ControlKey::Other,
    }
}

pub struct PrototypeGame {
    viewport_width: f32,
    viewport_height: f32,
    world_width: f32,
    ground_y: f32,
    player_x: f32,
    player_scale: f32,
    movement_scale: f32,
    player_foot_y: f32,
    velocity_x: f32,
    velocity_y: f32,
    highest_jump_y: f32,
    grounded: bool,
    facing: f32,
    pose: Pose,
    pose_timer: f32,
    animation_time: f32,
    dash_timer: f32,
    hit_cooldown: f32,
    camera_x: f32,
    checkpoint_x: f32,
    checkpoint_active: bool,
    respawn_x: f32,
    relay_x: f32,
    relay_active: bool,
    destination_x: f32,
    destination_reached: bool,
}

impl PrototypeGame {
    pub fn new(width: u16, height: u16) -> Self {
        let viewport_width = f32::from(width.max(1));
        let viewport_height = f32::from(height.saturating_sub(2).max(2) * 2);
        let player_scale = (viewport_width / 128.0)
            .min(viewport_height / 74.0)
            .clamp(0.58, 1.0);
        let movement_scale = (viewport_width / 236.0).clamp(0.34, 1.5);
        let ground_y = viewport_height * 0.77;
        Self {
            viewport_width,
            viewport_height,
            world_width: viewport_width * 3.2,
            ground_y,
            player_x: viewport_width * 0.10,
            player_scale,
            movement_scale,
            player_foot_y: ground_y,
            velocity_x: 0.0,
            velocity_y: 0.0,
            highest_jump_y: ground_y,
            grounded: true,
            facing: 1.0,
            pose: Pose::Idle,
            pose_timer: 0.0,
            animation_time: 0.0,
            dash_timer: 0.0,
            hit_cooldown: 0.0,
            camera_x: 0.0,
            checkpoint_x: viewport_width * 0.29,
            checkpoint_active: false,
            respawn_x: viewport_width * 0.08,
            relay_x: viewport_width * 0.69,
            relay_active: false,
            destination_x: viewport_width * 0.88,
            destination_reached: false,
        }
    }

    pub fn update(&mut self, dt: f32, input: &mut InputState, at: Duration) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }

        input.refresh_fallback(at);
        self.animation_time += dt;
        self.pose_timer = (self.pose_timer - dt).max(0.0);
        self.dash_timer = (self.dash_timer - dt).max(0.0);
        self.hit_cooldown = (self.hit_cooldown - dt).max(0.0);

        if input.take_jump_pressed() && self.grounded {
            self.velocity_y = -145.0;
            self.grounded = false;
            self.pose = Pose::Jump;
            self.pose_timer = 0.14;
            if input.direction() != 0.0 {
                self.velocity_x = input.direction() * jump_speed(self.movement_scale);
            }
        }
        if input.take_dash_pressed() && self.grounded {
            self.dash_timer = 0.18;
            self.facing = if input.direction() == 0.0 {
                self.facing
            } else {
                input.direction()
            };
        }

        let direction = input.direction();
        if direction != 0.0 {
            if direction != self.facing {
                self.facing = direction;
                self.pose = Pose::Turn;
                self.pose_timer = 0.10;
            }
            let speed_limit = if self.grounded {
                WALK_SPEED * self.movement_scale
            } else {
                jump_speed(self.movement_scale)
            };
            self.velocity_x = (self.velocity_x
                + direction * WALK_ACCELERATION * self.movement_scale * dt)
                .clamp(-speed_limit, speed_limit);
        } else {
            self.velocity_x = approach_zero(self.velocity_x, GROUND_FRICTION * dt);
        }

        if self.dash_timer > 0.0 {
            self.velocity_x = self.facing * 190.0 * self.movement_scale;
        } else if self.grounded && input.drop {
            self.velocity_x = self.facing
                * self
                    .velocity_x
                    .abs()
                    .max(WALK_SPEED * 0.72 * self.movement_scale);
        }

        let old_x = self.player_x;
        let next_x =
            (old_x + self.velocity_x * dt).clamp(0.0, self.world_width - self.player_width());
        let feet = self.player_foot_y;
        let old_grounded = self.grounded;
        let (gap_start, gap_end) = self.gap();
        let crosses_gap = next_x + self.player_width() > gap_start && next_x < gap_end;

        if !self.grounded {
            let gravity = if input.drop { GRAVITY * 1.65 } else { GRAVITY };
            if input.jump && self.velocity_y > 0.0 {
                self.velocity_y = (self.velocity_y + gravity * dt).min(46.0);
            } else {
                self.velocity_y += gravity * dt;
            }
        }

        self.player_x = next_x;
        if !crosses_gap && !old_grounded && self.player_foot_y <= self.ground_y {
            let next_feet = self.player_foot_y + self.velocity_y * dt;
            if next_feet >= self.ground_y {
                self.player_foot_y = self.ground_y;
                self.velocity_y = 0.0;
                self.grounded = true;
                self.pose = Pose::Land;
                self.pose_timer = 0.11;
            } else {
                self.player_foot_y = next_feet;
            }
        } else if self.grounded && crosses_gap {
            self.grounded = false;
            self.velocity_y = 10.0;
        } else if !self.grounded {
            self.player_foot_y += self.velocity_y * dt;
        } else {
            self.player_foot_y = self.ground_y;
        }

        self.highest_jump_y = self.highest_jump_y.min(self.player_foot_y);
        if self.player_x + self.player_width() * 0.5 >= self.checkpoint_x && !self.checkpoint_active
        {
            self.checkpoint_active = true;
            self.respawn_x = self.checkpoint_x - self.player_width() * 0.5;
        }
        if input.take_interact_pressed()
            && !self.relay_active
            && (self.player_x + self.player_width() * 0.5 - self.relay_x).abs() < 16.0
        {
            self.relay_active = true;
            self.pose = Pose::Interact;
            self.pose_timer = 0.35;
        }

        let obstacle_x = self.viewport_width * 0.45;
        let obstacle_width = 11.0 * self.player_scale;
        let obstacle_height = 15.0 * self.player_scale;
        let overlaps_obstacle = self.player_x + self.player_width() > obstacle_x
            && self.player_x < obstacle_x + obstacle_width
            && self.player_foot_y > self.ground_y - obstacle_height;
        if overlaps_obstacle {
            self.player_x = if self.facing > 0.0 {
                obstacle_x - self.player_width()
            } else {
                obstacle_x + obstacle_width
            };
            if self.grounded || self.velocity_y >= 0.0 {
                self.velocity_x = 0.0;
            }
            if self.hit_cooldown == 0.0 {
                self.hit_cooldown = 0.55;
                self.pose = Pose::Hit;
                self.pose_timer = 0.15;
            }
        }

        if !self.grounded && self.player_foot_y > self.viewport_height + 24.0 {
            self.player_x = self.respawn_x;
            self.player_foot_y = self.ground_y;
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            self.grounded = true;
            self.pose = Pose::Respawn;
            self.pose_timer = 0.22;
        }

        self.destination_reached |= self.player_x + self.player_width() * 0.5 >= self.destination_x;
        self.camera_x = (self.player_x + self.player_width() * 0.5 - self.viewport_width * 0.43)
            .clamp(0.0, (self.world_width - self.viewport_width).max(0.0));

        self.choose_pose(input, direction, old_x, feet);
    }

    fn choose_pose(&mut self, input: &InputState, direction: f32, old_x: f32, old_feet: f32) {
        if self.pose_timer > 0.0 {
            return;
        }
        if !self.grounded {
            self.pose = if self.velocity_y < 0.0 {
                Pose::Jump
            } else if input.jump {
                Pose::Glide
            } else {
                Pose::Fall
            };
        } else if input.drop {
            self.pose = Pose::Slide;
        } else if self.dash_timer > 0.0 {
            self.pose = Pose::Dash;
        } else if direction != 0.0 || (self.player_x - old_x).abs() > 0.2 {
            self.pose = Pose::Run;
        } else if old_feet > self.ground_y {
            self.pose = Pose::Land;
        } else {
            self.pose = Pose::Idle;
        }
    }

    fn gap(&self) -> (f32, f32) {
        (self.viewport_width * 0.56, self.viewport_width * 0.63)
    }

    pub fn player_x(&self) -> f32 {
        self.player_x
    }

    pub fn player_width(&self) -> f32 {
        PLAYER_WIDTH * self.player_scale
    }

    pub fn player_height(&self) -> f32 {
        PLAYER_HEIGHT * self.player_scale
    }

    pub fn player_scale(&self) -> f32 {
        self.player_scale
    }

    pub fn player_foot_y(&self) -> f32 {
        self.player_foot_y
    }

    pub fn horizontal_speed(&self) -> f32 {
        self.velocity_x
    }

    pub fn highest_jump_y(&self) -> f32 {
        self.highest_jump_y
    }

    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    pub fn pose(&self) -> Pose {
        self.pose
    }

    pub fn facing(&self) -> f32 {
        self.facing
    }

    pub fn animation_time(&self) -> f32 {
        self.animation_time
    }

    pub fn viewport_width(&self) -> f32 {
        self.viewport_width
    }

    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }

    pub fn ground_y(&self) -> f32 {
        self.ground_y
    }

    pub fn camera_x(&self) -> f32 {
        self.camera_x
    }

    pub fn checkpoint_x(&self) -> f32 {
        self.checkpoint_x
    }

    pub fn checkpoint_active(&self) -> bool {
        self.checkpoint_active
    }

    pub fn relay_x(&self) -> f32 {
        self.relay_x
    }

    pub fn relay_active(&self) -> bool {
        self.relay_active
    }

    pub fn destination_x(&self) -> f32 {
        self.destination_x
    }

    pub fn destination_reached(&self) -> bool {
        self.destination_reached
    }

    pub fn near_relay(&self) -> bool {
        (self.player_x + self.player_width() * 0.5 - self.relay_x).abs() < 16.0
    }

    pub fn obstacle_x(&self) -> f32 {
        self.viewport_width * 0.45
    }

    pub fn gap_start(&self) -> f32 {
        self.gap().0
    }

    pub fn gap_end(&self) -> f32 {
        self.gap().1
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let new_width = f32::from(width.max(1));
        let new_height = f32::from(height.saturating_sub(2).max(2) * 2);
        let scale_x = new_width / self.viewport_width;
        let scale_y = new_height / self.viewport_height;
        let new_player_scale = (new_width / 128.0).min(new_height / 74.0).clamp(0.58, 1.0);
        let new_movement_scale = (new_width / 236.0).clamp(0.34, 1.5);
        let altitude = (self.ground_y - self.player_foot_y) * scale_y;
        self.viewport_width = new_width;
        self.viewport_height = new_height;
        self.world_width = new_width * 3.2;
        self.ground_y = new_height * 0.77;
        self.player_x *= scale_x;
        self.player_scale = new_player_scale;
        self.movement_scale = new_movement_scale;
        self.player_foot_y = if self.grounded {
            self.ground_y
        } else {
            self.ground_y - altitude
        };
        self.checkpoint_x *= scale_x;
        self.respawn_x *= scale_x;
        self.relay_x *= scale_x;
        self.destination_x *= scale_x;
        self.camera_x *= scale_x;
        self.highest_jump_y = self.ground_y - (self.ground_y - self.highest_jump_y) * scale_y;
    }
}

fn approach_zero(value: f32, amount: f32) -> f32 {
    if value.abs() <= amount {
        0.0
    } else {
        value - value.signum() * amount
    }
}

fn jump_speed(movement_scale: f32) -> f32 {
    120.0 * movement_scale.sqrt()
}
