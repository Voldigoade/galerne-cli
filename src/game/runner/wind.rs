//! Wind physics, atmospheric updrafts, tailwind gusts, and glide mechanics.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindKind {
    Updraft,
    TailwindGust,
    CrosswindChasm,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindCurrent {
    pub rel_x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub kind: WindKind,
    pub strength: f32,
}

impl WindCurrent {
    pub fn new(rel_x: f32, y: f32, width: f32, height: f32, kind: WindKind) -> Self {
        Self {
            rel_x,
            y,
            width,
            height,
            kind,
            strength: 1.0,
        }
    }

    pub fn bounding_box(&self, chunk_start_x: f32) -> (f32, f32, f32, f32) {
        let abs_x = chunk_start_x + self.rel_x;
        (abs_x, self.y - self.height, abs_x + self.width, self.y)
    }

    pub fn sprite_lines(&self) -> &'static [&'static str] {
        match self.kind {
            WindKind::Updraft => &[
                "  ^   ^   ^  ",
                " / \\ / \\ / \\ ",
                " :   :   :   ",
                "   |   |   | ",
            ],
            WindKind::TailwindGust => &["~ ~ ~ ~ >> >> >>", ">> >> >> ~ ~ ~ ~"],
            WindKind::CrosswindChasm => {
                &["~~~ ~~~ ~~~ ~~~ ", "  ^   ^   ^   ^ ", "~~~~~~~~~~~~~~~~"]
            }
        }
    }
}

/// Build-duration Tailwind bonus tiers derived from predicted task duration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TailwindTier {
    #[default]
    None = 0,
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Tier4 = 4,
}

impl TailwindTier {
    pub fn from_predicted_duration(secs: f32) -> Self {
        if secs < 10.0 {
            TailwindTier::None
        } else if secs < 30.0 {
            TailwindTier::Tier1
        } else if secs < 90.0 {
            TailwindTier::Tier2
        } else if secs < 180.0 {
            TailwindTier::Tier3
        } else {
            TailwindTier::Tier4
        }
    }

    pub fn speed_bonus(&self) -> f32 {
        match self {
            TailwindTier::None => 0.0,
            TailwindTier::Tier1 => 1.5,
            TailwindTier::Tier2 => 3.0,
            TailwindTier::Tier3 => 4.5,
            TailwindTier::Tier4 => 6.0,
        }
    }

    pub fn flow_multiplier_bonus(&self) -> f32 {
        match self {
            TailwindTier::None => 0.0,
            TailwindTier::Tier1 => 0.1,
            TailwindTier::Tier2 => 0.2,
            TailwindTier::Tier3 => 0.35,
            TailwindTier::Tier4 => 0.5,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TailwindTier::None => "",
            TailwindTier::Tier1 => "TAILWIND I",
            TailwindTier::Tier2 => "TAILWIND II",
            TailwindTier::Tier3 => "TAILWIND III",
            TailwindTier::Tier4 => "TAILWIND IV",
        }
    }
}
