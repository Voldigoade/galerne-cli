//! Runner player entity, multi-cell ASCII animation cycles, and discrete action physics.

use serde::{Deserialize, Serialize};

use super::action::{ActionState, InputBuffer, JumpPhase, PlayerInput};

pub const PLAYER_WIDTH: usize = 7;
pub const PLAYER_HEIGHT: usize = 4;
pub const SLIDE_WIDTH: usize = 10;
pub const SLIDE_HEIGHT: usize = 2;
pub const PLAYER_SPRITE_LEFT_OFFSET: usize = 6;

pub const JUMP_INITIAL_VELOCITY: f32 = -23.0;
pub const GRAVITY_NORMAL: f32 = 50.0;
pub const GRAVITY_APEX: f32 = 16.0;
pub const APEX_VELOCITY_THRESHOLD: f32 = 3.5;
pub const MAX_FALL_SPEED: f32 = 30.0;

pub const SLIDE_DURATION_SECS: f32 = 0.45;
pub const DASH_DURATION_SECS: f32 = 0.22;
pub const DASH_COOLDOWN_SECS: f32 = 1.0;
pub const COYOTE_TIME_WINDOW_SECS: f32 = 0.10;

const RUN_FRAME_0: &[&str] = &[
    "~~~~~\\       .---.",
    " ~~~~~\\_____/ o  \\__",
    "      [#]___/|\\   _/",
    "          _/ | \\__/",
    "         /   |  \\",
    "        /___/ \\___\\",
    "       /___/    \\___\\",
];
const RUN_FRAME_1: &[&str] = &[
    " ~~~~~\\      .---.",
    "~~~~~~\\_____/ o  \\__",
    "      [#]___/|\\   _/",
    "         __/ | \\___/",
    "        /   /|  \\",
    "       /___/  \\___\\",
    "      /___/      \\___\\",
];
const RUN_FRAME_2: &[&str] = &[
    "~~~~~~~\\      .---.",
    " ~~~~~~~\\____/ o  \\__",
    "      [#]___/|\\   _/",
    "          / | \\__/",
    "         /  | /  \\",
    "        /__/  \\___\\",
    "        \\__/   /___/",
];
const RUN_FRAME_3: &[&str] = &[
    "  ~~~~~\\      .---.",
    "~~~~~~~\\_____/ o  \\__",
    "      [#]___/|\\  _/",
    "        \\   | \\___/",
    "         \\  \\|   \\",
    "       ___/ /  \\___\\",
    "      /____/    \\___\\",
];
const JUMP_ASCENT: &[&str] = &[
    "  ~~~~~~\\      .---.",
    " ~~~~~~~~\\____/ o  \\__",
    "        [#]__/|\\   _/",
    "            / | \\__/",
    "           /  |",
    "         _/___/\\",
    "        /___/  \\__",
];
const JUMP_APEX: &[&str] = &[
    "~~~~~~~~\\      .---.",
    " ~~~~~~~~\\____/ o  \\__",
    "       [#]___/|\\  _/",
    "           / | \\__/",
    "          /  |  \\",
    "         /___/\\__\\",
    "          \\__/ \\__\\",
];
const JUMP_DESCENT: &[&str] = &[
    "~~~~\\          .---.",
    " ~~~~\\________/ o  \\__",
    "       [#]___/|\\   _/",
    "            / | \\__/",
    "           /  |  \\",
    "          /___/\\__\\",
    "         /___/  \\___\\",
];
const JUMP_LANDING: &[&str] = &[
    " ~~~~~\\       .---.",
    "  ~~~~~\\_____/ o  \\__",
    "      [#]___/|\\   _/",
    "         __/ | \\__/",
    "        /___/  \\",
    "       /____/\\__\\",
    "      /____/  \\___\\",
];
const GLIDING: &[&str] = &[
    "~~~~~~~~~~~~\\       .---.",
    " ~~~~~~~~~~~\\______/ o  \\__",
    "           [#]___/|\\   _/",
    "              _/ | \\__/",
    "             /   |   \\",
    "           _/___/ \\___\\",
    "          /___/     \\___\\",
];
const SLIDING: &[&str] = &[
    "~~~~~~~~\\____      .---.",
    "  ~~~~~~~\\ [#]\\___/ o  \\___",
    "          \\___|____/   \\__\\",
];
const DASHING: &[&str] = &[
    ">>>>>~~~~~~\\      .---.",
    ">>>>> ~~~~~\\_____/ o  \\__",
    "       [#]___/|\\   _/",
    "         __/ | \\____>",
    "        /___/  \\___/",
    "      _/____/    \\____\\",
    "     /_____/      \\____\\",
];
const HIT: &[&str] = &[
    "~~~~~~\\       .---.",
    " ~~~~~\\______/ x  \\__",
    "      [#]___/|\\   _/",
    "          _/ | \\__/",
    "         /___|   \\",
    "        /  / \\____\\",
    "       /__/   \\___/",
];
const ACTIVATING: &[&str] = &[
    "~~~~~\\       .---.*",
    " ~~~~~\\_____/ o  \\__+",
    "      [#]___/|\\   _/",
    "          _/ | \\__/",
    "         /   |  \\",
    "        /___/ \\___\\",
    "       /___/    \\___\\",
];
const RESPAWNING: &[&str] = &[
    " . .~~~~\\      .---.",
    "  ~~~~~~\\_____/ o  \\__",
    "      [#]___/|\\   _/",
    "          _/ | \\__/",
    "         /   |  \\",
    "        /___/ \\___\\",
    "       /___/    \\___\\",
];

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
                    self.input_buffer.push(PlayerInput::Jump);
                }
            }
            PlayerInput::Slide => {
                if self.is_on_ground(ground_y) {
                    self.action = ActionState::Sliding {
                        remaining_secs: SLIDE_DURATION_SECS,
                    };
                } else {
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
            PlayerInput::Interact => {}
        }
    }

    pub fn update(&mut self, dt: f32, ground_y: f32) {
        self.update_with_platform(dt, ground_y, None);
    }

    pub fn update_with_platform(&mut self, dt: f32, ground_y: f32, platform_y: Option<f32>) {
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

        let platform_support = platform_y.filter(|surface_y| (self.y - *surface_y).abs() < 0.05);
        if !self.is_on_ground(ground_y)
            && platform_support.is_none()
            && matches!(
                self.action,
                ActionState::Running { .. } | ActionState::Sliding { .. }
            )
        {
            if matches!(self.action, ActionState::Running { .. }) {
                self.start_falling_off_ledge();
            } else {
                self.action = ActionState::Jumping {
                    phase: JumpPhase::Descent,
                    vertical_velocity: 0.0,
                    elapsed_secs: 0.0,
                };
            }
        }

        match self.action {
            ActionState::Idle {
                ref mut cycle_timer,
                ref mut frame,
            } => {
                self.y = platform_support.unwrap_or(ground_y);
                *cycle_timer += dt;
                if *cycle_timer >= 0.25 {
                    *cycle_timer = 0.0;
                    *frame = (*frame + 1) % 2;
                }
            }
            ActionState::Running {
                ref mut cycle_timer,
                ref mut frame,
            } => {
                self.y = platform_support.unwrap_or(ground_y);
                *cycle_timer += dt;
                if *cycle_timer >= 0.12 {
                    *cycle_timer = 0.0;
                    *frame = (*frame + 1) % 4;
                }
            }
            ActionState::Jumping {
                ref mut phase,
                ref mut vertical_velocity,
                ref mut elapsed_secs,
            } => {
                let previous_y = self.y;
                *elapsed_secs += dt;

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

                let landing_y = platform_y
                    .filter(|surface_y| previous_y <= *surface_y && self.y >= *surface_y)
                    .or_else(|| (self.y >= ground_y).then_some(ground_y));
                if let Some(landing_y) = landing_y {
                    self.y = landing_y;

                    if let Some(buffered) = self.input_buffer.pop() {
                        self.handle_input(buffered, landing_y);
                    } else {
                        self.action = ActionState::Running {
                            cycle_timer: 0.0,
                            frame: 0,
                        };
                    }
                }
            }
            ActionState::Gliding {
                ref mut elapsed_secs,
                ref mut vertical_velocity,
            } => {
                let previous_y = self.y;
                *elapsed_secs += dt;
                *vertical_velocity = (*vertical_velocity + 10.0 * dt).min(6.0);
                self.y += *vertical_velocity * dt;

                let landing_y = platform_y
                    .filter(|surface_y| previous_y <= *surface_y && self.y >= *surface_y)
                    .or_else(|| (self.y >= ground_y).then_some(ground_y));
                if let Some(landing_y) = landing_y {
                    self.y = landing_y;
                    if let Some(buffered) = self.input_buffer.pop() {
                        self.handle_input(buffered, landing_y);
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
                self.y = platform_support.unwrap_or(ground_y);
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
                    if self.is_on_ground(ground_y) || platform_support.is_some() {
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
            ActionState::Hit {
                ref mut remaining_secs,
            } => {
                *remaining_secs -= dt;
                if *remaining_secs <= 0.0 {
                    self.action = ActionState::Running {
                        cycle_timer: 0.0,
                        frame: 0,
                    };
                }
            }
            ActionState::Activating {
                ref mut remaining_secs,
            } => {
                *remaining_secs -= dt;
                if *remaining_secs <= 0.0 {
                    self.action = ActionState::Running {
                        cycle_timer: 0.0,
                        frame: 0,
                    };
                }
            }
            ActionState::Respawning {
                ref mut remaining_secs,
            } => {
                *remaining_secs -= dt;
                if *remaining_secs <= 0.0 {
                    self.action = ActionState::Running {
                        cycle_timer: 0.0,
                        frame: 0,
                    };
                }
            }
        }
    }

    pub fn current_sprite(&self) -> &'static [&'static str] {
        match self.action {
            ActionState::Idle { frame, .. } => match frame {
                0 => RUN_FRAME_0,
                _ => RUN_FRAME_2,
            },
            ActionState::Running { frame, .. } => match frame {
                0 => RUN_FRAME_0,
                1 => RUN_FRAME_1,
                2 => RUN_FRAME_2,
                _ => RUN_FRAME_3,
            },
            ActionState::Jumping { phase, .. } => match phase {
                JumpPhase::Ascent => JUMP_ASCENT,
                JumpPhase::Apex => JUMP_APEX,
                JumpPhase::Descent => JUMP_DESCENT,
                JumpPhase::Landing => JUMP_LANDING,
            },
            ActionState::Gliding { .. } => GLIDING,
            ActionState::Sliding { .. } => SLIDING,
            ActionState::Dashing { .. } => DASHING,
            ActionState::Hit { .. } => HIT,
            ActionState::Activating { .. } => ACTIVATING,
            ActionState::Respawning { .. } => RESPAWNING,
        }
    }
}
