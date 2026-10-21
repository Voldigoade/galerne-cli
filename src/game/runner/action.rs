//! Discrete player action state machine with time-based arcs,
//! coyote time, and input buffering for terminal input ergonomics.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerInput {
    Jump,
    Slide,
    Dash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JumpPhase {
    Ascent,
    Apex,
    Descent,
    Landing,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionState {
    Running {
        cycle_timer: f32,
        frame: u8,
    },
    Jumping {
        phase: JumpPhase,
        vertical_velocity: f32,
        elapsed_secs: f32,
    },
    Gliding {
        elapsed_secs: f32,
        vertical_velocity: f32,
    },
    Sliding {
        remaining_secs: f32,
    },
    Dashing {
        remaining_secs: f32,
    },
    Hit {
        remaining_secs: f32,
    },
    Activating {
        remaining_secs: f32,
    },
    Respawning {
        remaining_secs: f32,
    },
    Idle {
        cycle_timer: f32,
        frame: u8,
    },
}

impl Default for ActionState {
    fn default() -> Self {
        ActionState::Running {
            cycle_timer: 0.0,
            frame: 0,
        }
    }
}

/// Short input buffer for presses made just before landing or recovery.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputBuffer {
    pub buffered_input: Option<PlayerInputWrapper>,
    pub buffer_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerInputWrapper {
    Jump,
    Slide,
    Dash,
}

impl From<PlayerInput> for PlayerInputWrapper {
    fn from(input: PlayerInput) -> Self {
        match input {
            PlayerInput::Jump => PlayerInputWrapper::Jump,
            PlayerInput::Slide => PlayerInputWrapper::Slide,
            PlayerInput::Dash => PlayerInputWrapper::Dash,
        }
    }
}

impl From<PlayerInputWrapper> for PlayerInput {
    fn from(wrapper: PlayerInputWrapper) -> Self {
        match wrapper {
            PlayerInputWrapper::Jump => PlayerInput::Jump,
            PlayerInputWrapper::Slide => PlayerInput::Slide,
            PlayerInputWrapper::Dash => PlayerInput::Dash,
        }
    }
}

impl InputBuffer {
    pub const BUFFER_WINDOW_SECS: f32 = 0.150; // 150ms early press window

    pub fn push(&mut self, input: PlayerInput) {
        self.buffered_input = Some(input.into());
        self.buffer_timer = Self::BUFFER_WINDOW_SECS;
    }

    pub fn pop(&mut self) -> Option<PlayerInput> {
        let input = self.buffered_input.take()?.into();
        self.buffer_timer = 0.0;
        Some(input)
    }

    pub fn peek(&self) -> Option<PlayerInput> {
        self.buffered_input.map(|w| w.into())
    }

    pub fn update(&mut self, dt: f32) {
        if self.buffer_timer > 0.0 {
            self.buffer_timer -= dt;
            if self.buffer_timer <= 0.0 {
                self.buffered_input = None;
            }
        }
    }
}
