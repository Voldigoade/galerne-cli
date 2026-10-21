//! Multi-cell authored ASCII obstacles with forgiving collision boxes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObstacleKind {
    /// Low hurdle or rubble (2 rows high on ground): must JUMP over.
    LowBarrier,
    /// High overhead conduit or steam pipe (hangs 2-3 rows above ground): must SLIDE under.
    OverheadPipe,
    /// Tall pylon or antenna (4 rows high): requires high jump or dash timing.
    Pylon,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Obstacle {
    pub rel_x: f32, // Relative x offset within the chunk (0.0 to CHUNK_WIDTH)
    pub y: f32,     // Base y position (on ground or elevated)
    pub kind: ObstacleKind,
    #[serde(default)]
    pub cleared: bool,
}

impl Obstacle {
    pub fn new(rel_x: f32, y: f32, kind: ObstacleKind) -> Self {
        Self {
            rel_x,
            y,
            kind,
            cleared: false,
        }
    }

    pub fn width(&self) -> usize {
        match self.kind {
            ObstacleKind::LowBarrier => 4,
            ObstacleKind::OverheadPipe => 7,
            ObstacleKind::Pylon => 4,
        }
    }

    pub fn height(&self) -> usize {
        match self.kind {
            ObstacleKind::LowBarrier => 2,
            ObstacleKind::OverheadPipe => 2,
            ObstacleKind::Pylon => 4,
        }
    }

    /// Absolute bounding box given chunk start x: (min_x, min_y, max_x, max_y).
    /// Inset by 0.4 cells for player-favoring collision fairness.
    pub fn bounding_box(&self, chunk_start_x: f32) -> (f32, f32, f32, f32) {
        let abs_x = chunk_start_x + self.rel_x;
        let w = self.width() as f32;
        let h = self.height() as f32;

        let (top_y, bottom_y) = match self.kind {
            ObstacleKind::LowBarrier | ObstacleKind::Pylon => (self.y - h + 1.0, self.y),
            ObstacleKind::OverheadPipe => {
                // Suspended above ground: height 2, positioned 2 rows above track
                let bottom = self.y - 1.8;
                let top = bottom - h + 1.0;
                (top, bottom)
            }
        };

        (abs_x + 0.4, top_y + 0.3, abs_x + w - 0.4, bottom_y - 0.2)
    }

    pub fn sprite(&self) -> &'static [&'static str] {
        match self.kind {
            ObstacleKind::LowBarrier => &[" /\\ ", "[##]"],
            ObstacleKind::OverheadPipe => &["[====:]", "~~::~:~"],
            ObstacleKind::Pylon => &[" /\\ ", "/||\\", "+==+", "|  |"],
        }
    }
}
