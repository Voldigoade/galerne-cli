//! Non-blocking instant-apply pickups with unobtrusive floating feedback.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Gale Shard currency: [*]
    GaleShard,
    /// Speed surge: [>>]
    SpeedSurge,
    /// Kinetic wind shield: (O)
    WindShield,
    /// Magnetic attractor: [+]
    ShardMagnet,
    /// Flow score multiplier: [x2]
    FlowMultiplier,
    /// Persistent ancient relic: [<#>]
    ResonanceCore,
    /// Persistent conduit component: [~|~]
    InductionFilament,
    /// Persistent ancient compass: [(*)]
    AncientCompass,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PickupItem {
    pub rel_x: f32,
    pub y: f32,
    pub kind: ItemKind,
    pub collected: bool,
}

impl PickupItem {
    pub fn new(rel_x: f32, y: f32, kind: ItemKind) -> Self {
        Self {
            rel_x,
            y,
            kind,
            collected: false,
        }
    }

    pub fn sprite(&self) -> &'static str {
        match self.kind {
            ItemKind::GaleShard => "[*]",
            ItemKind::SpeedSurge => "[>>]",
            ItemKind::WindShield => "(O)",
            ItemKind::ShardMagnet => "[+]",
            ItemKind::FlowMultiplier => "[x2]",
            ItemKind::ResonanceCore => "[<#>]",
            ItemKind::InductionFilament => "[~|~]",
            ItemKind::AncientCompass => "[(*)]",
        }
    }

    pub fn bounding_box(&self, chunk_start_x: f32) -> (f32, f32, f32, f32) {
        let abs_x = chunk_start_x + self.rel_x;
        let w = self.sprite().len() as f32;
        (abs_x - 0.2, self.y - 0.8, abs_x + w + 0.2, self.y + 0.8)
    }
}

/// Floating visual acknowledgement that rises and fades without blocking gameplay.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatingIndicator {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub lifetime_secs: f32,
    pub max_lifetime_secs: f32,
}

impl FloatingIndicator {
    pub fn new(x: f32, y: f32, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            text: text.into(),
            lifetime_secs: 0.65,
            max_lifetime_secs: 0.65,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime_secs -= dt;
        self.y -= dt * 2.5; // Slowly float upward
        self.lifetime_secs > 0.0
    }
}
