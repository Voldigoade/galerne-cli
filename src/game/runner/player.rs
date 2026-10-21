//! Runner player entity, multi-cell ASCII animation cycles, and discrete action physics.

use serde::{Deserialize, Serialize};

use super::action::{ActionState, InputBuffer, JumpPhase, PlayerInput};

pub const PLAYER_WIDTH: usize = 5;
pub const PLAYER_HEIGHT: usize = 4;
pub const SLIDE_WIDTH: usize = 6;
pub const SLIDE_HEIGHT: usize = 2;

pub const JUMP_INITIAL_VELOCITY: f32 = -23.0;
pub const GRAVITY_NORMAL: f32 = 50.0;
pub const GRAVITY_APEX: f32 = 16.0;
pub const APEX_VELOCITY_THRESHOLD: f32 = 3.5;
pub const MAX_FALL_SPEED: f32 = 30.0;

pub const SLIDE_DURATION_SECS: f32 = 0.45;
pub const DASH_DURATION_SECS: f32 = 0.22;
pub const DASH_COOLDOWN_SECS: f32 = 1.0;
pub const COYOTE_TIME_WINDOW_SECS: f32 = 0.10;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerPlayer {
    pub x: f32,
    pub y: f32,
    pub action: ActionState,
    pub input_buffer: InputBuffer,
    pub coyote_timer: f32,
    pub dash_cooldown_timer: f32,
    #[serde(default = "default_dash_cooldown_duration")]
    pub dash_cooldown_duration: f32,
    pub invulnerable_timer: f32,
    pub shields: u32,
    pub max_shields: u32,
}

fn default_dash_cooldown_duration() -> f32 {
    DASH_COOLDOWN_SECS
}

impl RunnerPlayer {
    pub fn new(x: f32, ground_y: f32) -> Self {
        Self {
            x,
            y: ground_y,
            action: ActionState::Running {
                cycle_timer: 0.0,
                frame: 0,
            },
            input_buffer: InputBuffer::default(),
            coyote_timer: 0.0,
            dash_cooldown_timer: 0.0,
            dash_cooldown_duration: DASH_COOLDOWN_SECS,
            invulnerable_timer: 0.0,
            shields: 1,
            max_shields: 3,
        }
    }

    pub fn is_on_ground(&self, ground_y: f32) -> bool {
        (self.y - ground_y).abs() < 0.05
    }

    pub fn in_coyote_time(&self) -> bool {
        self.coyote_timer > 0.0
    }

    pub fn can_dash(&self) -> bool {
        self.dash_cooldown_timer <= 0.0 && !matches!(self.action, ActionState::Dashing { .. })
    }

    pub fn hitbox_height(&self) -> usize {
        match self.action {
            ActionState::Sliding { .. } => SLIDE_HEIGHT,
            _ => PLAYER_HEIGHT,
        }
    }

    pub fn hitbox_width(&self) -> usize {
        match self.action {
            ActionState::Sliding { .. } => SLIDE_WIDTH,
            _ => PLAYER_WIDTH,
        }
    }

    /// Forgiving bounding box: inset by 0.5 cells so near-misses feel thrilling rather than unfair.
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        let h = self.hitbox_height() as f32;
        let w = self.hitbox_width() as f32;
        let min_x = self.x + 0.5;
        let max_x = self.x + w - 0.5;
        let min_y = (self.y - h + 1.0) + 0.4;
        let max_y = self.y + 0.2;
        (min_x, min_y, max_x, max_y)
    }

    pub fn has_buffered_input(&self, expected: PlayerInput) -> bool {
        self.input_buffer.peek() == Some(expected)
    }

    pub fn start_falling_off_ledge(&mut self) {
        if matches!(self.action, ActionState::Running { .. }) {
            self.coyote_timer = COYOTE_TIME_WINDOW_SECS;
            self.action = ActionState::Jumping {
                phase: JumpPhase::Descent,
                vertical_velocity: 0.0,
                elapsed_secs: 0.0,
            };
        }
    }

    pub fn handle_input(&mut self, input: PlayerInput, ground_y: f32) {
        match input {
            PlayerInput::Jump => {
                if self.is_on_ground(ground_y) || self.in_coyote_time() {
                    self.coyote_timer = 0.0;
                    self.action = ActionState::Jumping {
                        phase: JumpPhase::Ascent,
                        vertical_velocity: JUMP_INITIAL_VELOCITY,
                        elapsed_secs: 0.0,
                    };
                } else {
                    // Buffer jump for when player touches down
                    self.input_buffer.push(PlayerInput::Jump);
                }
            }
            PlayerInput::Slide => {
                if self.is_on_ground(ground_y) {
                    self.action = ActionState::Sliding {
                        remaining_secs: SLIDE_DURATION_SECS,
                    };
                } else {
                    // Buffer slide for landing
                    self.input_buffer.push(PlayerInput::Slide);
                }
            }
            PlayerInput::Dash => {
                if self.can_dash() {
                    self.action = ActionState::Dashing {
                        remaining_secs: DASH_DURATION_SECS,
                    };
                    self.dash_cooldown_timer = self.dash_cooldown_duration;
                    self.invulnerable_timer = DASH_DURATION_SECS + 0.05;
                } else {
                    self.input_buffer.push(PlayerInput::Dash);
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32, ground_y: f32) {
        if self.dash_cooldown_timer > 0.0 {
            self.dash_cooldown_timer = (self.dash_cooldown_timer - dt).max(0.0);
        }
        if self.invulnerable_timer > 0.0 {
            self.invulnerable_timer = (self.invulnerable_timer - dt).max(0.0);
        }
        if self.coyote_timer > 0.0 {
            self.coyote_timer = (self.coyote_timer - dt).max(0.0);
        }
        self.input_buffer.update(dt);

        match self.action {
            ActionState::Running {
                ref mut cycle_timer,
                ref mut frame,
            } => {
                self.y = ground_y;
                *cycle_timer += dt;
                if *cycle_timer >= 0.12 {
                    *cycle_timer = 0.0;
                    *frame = (*frame + 1) % 3;
                }
            }
            ActionState::Jumping {
                ref mut phase,
                ref mut vertical_velocity,
                ref mut elapsed_secs,
            } => {
                *elapsed_secs += dt;

                // Apex float calculation: lower gravity around velocity zero
                let gravity = if vertical_velocity.abs() < APEX_VELOCITY_THRESHOLD {
                    *phase = JumpPhase::Apex;
                    GRAVITY_APEX
                } else if *vertical_velocity > 0.0 {
                    *phase = JumpPhase::Descent;
                    GRAVITY_NORMAL
                } else {
                    *phase = JumpPhase::Ascent;
                    GRAVITY_NORMAL
                };

                *vertical_velocity = (*vertical_velocity + gravity * dt).min(MAX_FALL_SPEED);
                self.y += *vertical_velocity * dt;

                if self.y >= ground_y {
                    self.y = ground_y;

                    // Check buffered input immediately upon touchdown!
                    if let Some(buffered) = self.input_buffer.pop() {
                        self.handle_input(buffered, ground_y);
                    } else {
                        self.action = ActionState::Running {
                            cycle_timer: 0.0,
                            frame: 0,
                        };
                    }
                }
            }
            ActionState::Sliding {
                ref mut remaining_secs,
            } => {
                self.y = ground_y;
                *remaining_secs -= dt;
                if *remaining_secs <= 0.0 {
                    if let Some(buffered) = self.input_buffer.pop() {
                        self.handle_input(buffered, ground_y);
                    } else {
                        self.action = ActionState::Running {
                            cycle_timer: 0.0,
                            frame: 0,
                        };
                    }
                }
            }
            ActionState::Dashing {
                ref mut remaining_secs,
            } => {
                *remaining_secs -= dt;
                if *remaining_secs <= 0.0 {
                    if self.is_on_ground(ground_y) {
                        self.action = ActionState::Running {
                            cycle_timer: 0.0,
                            frame: 0,
                        };
                    } else {
                        self.action = ActionState::Jumping {
                            phase: JumpPhase::Descent,
                            vertical_velocity: 5.0,
                            elapsed_secs: 0.0,
                        };
                    }
                }
            }
        }
    }

    /// Returns authored multi-cell ASCII sprite lines for current visual state.
    pub fn current_sprite(&self) -> &'static [&'static str] {
        match self.action {
            ActionState::Running { frame, .. } => match frame {
                0 => &[" [o] ", "--|>-", " / \\ ", "/   \\"],
                1 => &[" [o]~", " /|==", " / | ", "<   \\"],
                _ => &[" [o]=", "==|\\ ", "  | \\", " /   >"],
            },
            ActionState::Jumping { phase, .. } => match phase {
                JumpPhase::Ascent => &[" [o]~", " /|\\ ", " / \\ ", "^   ^"],
                JumpPhase::Apex => &["~[o]~", "--|--", " ( ) ", " \\ / "],
                JumpPhase::Descent | JumpPhase::Landing => &[" [o] ", "\\/|\\/", "  |  ", " v v "],
            },
            ActionState::Sliding { .. } => &["==[o]===", "--(____)-"],
            ActionState::Dashing { .. } => &[">>[o]>", "==|===", " / \\==", ">>  >>"],
        }
    }
}
