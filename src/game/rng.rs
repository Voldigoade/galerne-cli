//! Lightweight deterministic xorshift64 pseudorandom generator.

use serde::{Deserialize, Serialize};

/// Lightweight deterministic xorshift64 pseudorandom generator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastRng {
    state: u64,
}

impl FastRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x8a5cd789635d2dff } else { seed },
        }
    }

    pub fn from_state(state: u64) -> Self {
        Self {
            state: if state == 0 {
                0x8a5cd789635d2dff
            } else {
                state
            },
        }
    }

    pub fn state(&self) -> u64 {
        self.state
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
