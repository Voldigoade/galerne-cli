//! Chunk-based deterministic world generation and parallax management.

use serde::{Deserialize, Serialize};

use super::items::{ItemKind, PickupItem};
use super::obstacles::{Obstacle, ObstacleKind};
use super::parallax::ParallaxEngine;
use super::wind::{WindCurrent, WindKind};

pub const CHUNK_WIDTH: usize = 50;
pub const CHECKPOINTS: [(u32, usize, &str); 5] = [
    (0, 4, "cp_departure"),
    (1, 260, "cp_shanty_outpost"),
    (2, 512, "cp_shaft_airlock"),
    (3, 885, "cp_high_roost"),
    (4, 1210, "cp_relay_node"),
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LandmarkKind {
    CheckpointPylon { id: u32, name_key: String },
    CompanionKael { spoken: bool },
    SecretCache { opened: bool },
    RelayNodeConsole { restored: bool },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Landmark {
    pub rel_x: f32,
    pub y: f32,
    pub kind: LandmarkKind,
}

impl Landmark {
    pub fn new(rel_x: f32, y: f32, kind: LandmarkKind) -> Self {
        Self { rel_x, y, kind }
    }

    pub fn sprite(&self) -> &'static [&'static str] {
        match &self.kind {
            LandmarkKind::CheckpointPylon { .. } => &[" [+] ", "/===\\", "| # |", "|---|"],
            LandmarkKind::CompanionKael { .. } => &[" ,(^)  ", "<{#}>~ ", "==[#]==="],
            LandmarkKind::SecretCache { opened: false } => &["  .---.  ", "=[*###*]="],
            LandmarkKind::SecretCache { opened: true } => {
                &[" .----.  ", "/|    |\\ ", "|| [] ||", "|______|"]
            }
            LandmarkKind::RelayNodeConsole { .. } => {
                &["   /|\\   ", "  [|04|] ", " /======\\", " |[====]|"]
            }
        }
    }

    pub fn bounding_box(&self, chunk_start_x: f32) -> (f32, f32, f32, f32) {
        let abs_x = chunk_start_x + self.rel_x;
        let w = match &self.kind {
            LandmarkKind::CheckpointPylon { .. } => 5.0,
            LandmarkKind::CompanionKael { .. } => 8.0,
            LandmarkKind::SecretCache { .. } => 9.0,
            LandmarkKind::RelayNodeConsole { .. } => 9.0,
        };
        (abs_x - 1.0, self.y - 4.0, abs_x + w + 1.0, self.y + 1.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub index: usize,
    pub obstacles: Vec<Obstacle>,
    pub items: Vec<PickupItem>,
    #[serde(default)]
    pub wind_currents: Vec<WindCurrent>,
    #[serde(default)]
    pub landmarks: Vec<Landmark>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChunkWorld {
    pub seed: u64,
    pub scroll_x: f32,
    pub ground_y: usize,
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub chunks: Vec<Chunk>,
    pub parallax: ParallaxEngine,
}

impl ChunkWorld {
    pub fn new(seed: u64, viewport_width: usize, viewport_height: usize) -> Self {
        let ground_y = viewport_height.saturating_sub(5).max(8);
        let mut world = Self {
            seed,
            scroll_x: 0.0,
            ground_y,
            viewport_width,
            viewport_height,
            chunks: Vec::with_capacity(8),
            parallax: ParallaxEngine::default(),
        };

        world.ensure_chunks_generated((viewport_width + CHUNK_WIDTH * 2) as f32);
        world
    }

    pub fn active_chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    pub fn active_chunks_mut(&mut self) -> &mut [Chunk] {
        &mut self.chunks
    }

    pub fn resize(&mut self, new_w: usize, new_h: usize) {
        self.viewport_width = new_w;
        self.viewport_height = new_h;
        self.ground_y = new_h.saturating_sub(5).max(8);
    }

    pub fn update(&mut self, dx: f32) {
        self.scroll_x += dx;
        self.parallax.update(dx);

        let lookahead_x = self.scroll_x + (self.viewport_width as f32) + (CHUNK_WIDTH as f32 * 2.0);
        self.ensure_chunks_generated(lookahead_x);

        let prune_cutoff_x = self.scroll_x - 20.0;
        self.chunks.retain(|c| {
            let chunk_end_x = ((c.index + 1) * CHUNK_WIDTH) as f32;
            chunk_end_x >= prune_cutoff_x
        });
    }

    pub fn ensure_chunks_generated(&mut self, target_x: f32) {
        let max_chunk_needed = (target_x / CHUNK_WIDTH as f32).ceil() as usize;
        let current_max = self.chunks.last().map(|c| c.index).unwrap_or(0);

        let start_index = if self.chunks.is_empty() {
            0
        } else {
            current_max + 1
        };
        for idx in start_index..=max_chunk_needed {
            let chunk = self.generate_chunk(idx);
            self.chunks.push(chunk);
        }
    }

    fn generate_chunk(&self, index: usize) -> Chunk {
        let ground_y = self.ground_y as f32;
        let mut obstacles = Vec::new();
        let mut items = Vec::new();
        let mut wind_currents = Vec::new();
        let mut landmarks = Vec::new();

        match index {
            0 => {
                items.push(PickupItem::new(25.0, ground_y - 1.0, ItemKind::GaleShard));
                items.push(PickupItem::new(38.0, ground_y - 1.0, ItemKind::GaleShard));
            }
            1 => {
                obstacles.push(Obstacle::new(28.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(28.5, ground_y - 5.0, ItemKind::GaleShard));
            }
            2 => {
                obstacles.push(Obstacle::new(28.0, ground_y, ObstacleKind::OverheadPipe));
                items.push(PickupItem::new(30.0, ground_y, ItemKind::GaleShard));
            }
            3 => {
                wind_currents.push(WindCurrent::new(
                    18.0,
                    ground_y,
                    14.0,
                    8.0,
                    WindKind::Updraft,
                ));
                items.push(PickupItem::new(22.0, ground_y - 5.0, ItemKind::GaleShard));
                items.push(PickupItem::new(26.0, ground_y - 6.5, ItemKind::GaleShard));
                items.push(PickupItem::new(30.0, ground_y - 5.0, ItemKind::WindShield));
            }
            4 => {
                obstacles.push(Obstacle::new(18.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(28.0, ground_y - 1.0, ItemKind::GaleShard));
                obstacles.push(Obstacle::new(38.0, ground_y, ObstacleKind::LowBarrier));
            }
            5 => {
                wind_currents.push(WindCurrent::new(
                    25.0,
                    ground_y,
                    18.0,
                    4.0,
                    WindKind::TailwindGust,
                ));
                items.push(PickupItem::new(35.0, ground_y - 1.0, ItemKind::SpeedSurge));
            }
            6 => {
                obstacles.push(Obstacle::new(25.0, ground_y, ObstacleKind::OverheadPipe));
                items.push(PickupItem::new(38.0, ground_y, ItemKind::GaleShard));
            }
            7 => {
                obstacles.push(Obstacle::new(18.0, ground_y, ObstacleKind::LowBarrier));
                obstacles.push(Obstacle::new(35.0, ground_y, ObstacleKind::OverheadPipe));
                items.push(PickupItem::new(35.0, ground_y, ItemKind::GaleShard));
            }
            8 => {
                landmarks.push(Landmark::new(
                    42.0,
                    ground_y,
                    LandmarkKind::SecretCache { opened: false },
                ));
                obstacles.push(Obstacle::new(28.0, ground_y, ObstacleKind::OverheadPipe));
                items.push(PickupItem::new(
                    42.0,
                    ground_y - 1.0,
                    ItemKind::AncientCompass,
                ));
                items.push(PickupItem::new(
                    45.0,
                    ground_y - 1.0,
                    ItemKind::InductionFilament,
                ));
            }
            9 => {
                obstacles.push(Obstacle::new(20.0, ground_y, ObstacleKind::Pylon));
                obstacles.push(Obstacle::new(38.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(
                    29.0,
                    ground_y - 1.0,
                    ItemKind::FlowMultiplier,
                ));
            }
            10 => {
                items.push(PickupItem::new(30.0, ground_y - 1.0, ItemKind::GaleShard));
            }
            11 => {
                items.push(PickupItem::new(20.0, ground_y - 1.0, ItemKind::GaleShard));
                items.push(PickupItem::new(35.0, ground_y - 1.0, ItemKind::GaleShard));
            }
            12 => {
                obstacles.push(Obstacle::new(25.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(35.0, ground_y - 1.0, ItemKind::WindShield));
            }
            13 => {
                wind_currents.push(WindCurrent::new(
                    15.0,
                    ground_y,
                    16.0,
                    10.0,
                    WindKind::Updraft,
                ));
                items.push(PickupItem::new(22.0, ground_y - 6.0, ItemKind::GaleShard));
                items.push(PickupItem::new(
                    28.0,
                    ground_y - 8.0,
                    ItemKind::FlowMultiplier,
                ));
            }
            14 => {
                wind_currents.push(WindCurrent::new(
                    10.0,
                    ground_y,
                    25.0,
                    4.0,
                    WindKind::TailwindGust,
                ));
                obstacles.push(Obstacle::new(32.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(38.0, ground_y - 1.0, ItemKind::GaleShard));
            }
            15 => {
                obstacles.push(Obstacle::new(22.0, ground_y, ObstacleKind::OverheadPipe));
                items.push(PickupItem::new(32.0, ground_y, ItemKind::GaleShard));
            }
            16 => {
                landmarks.push(Landmark::new(
                    25.0,
                    ground_y,
                    LandmarkKind::CompanionKael { spoken: false },
                ));
                items.push(PickupItem::new(15.0, ground_y - 1.0, ItemKind::GaleShard));
            }
            17 => {
                items.push(PickupItem::new(20.0, ground_y - 1.0, ItemKind::GaleShard));
            }
            18 => {
                wind_currents.push(WindCurrent::new(
                    8.0,
                    ground_y,
                    35.0,
                    10.0,
                    WindKind::CrosswindChasm,
                ));
                items.push(PickupItem::new(18.0, ground_y - 5.0, ItemKind::GaleShard));
                items.push(PickupItem::new(28.0, ground_y - 6.5, ItemKind::GaleShard));
                items.push(PickupItem::new(38.0, ground_y - 5.0, ItemKind::GaleShard));
            }
            19 => {
                obstacles.push(Obstacle::new(25.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(38.0, ground_y - 1.0, ItemKind::WindShield));
            }
            20 => {
                obstacles.push(Obstacle::new(26.0, ground_y, ObstacleKind::OverheadPipe));
                items.push(PickupItem::new(26.0, ground_y, ItemKind::GaleShard));
            }
            21 => {
                obstacles.push(Obstacle::new(20.0, ground_y, ObstacleKind::LowBarrier));
                obstacles.push(Obstacle::new(38.0, ground_y, ObstacleKind::LowBarrier));
                items.push(PickupItem::new(29.0, ground_y - 5.0, ItemKind::GaleShard));
            }
            22 => {
                wind_currents.push(WindCurrent::new(
                    18.0,
                    ground_y,
                    14.0,
                    8.0,
                    WindKind::Updraft,
                ));
                items.push(PickupItem::new(
                    24.0,
                    ground_y - 6.0,
                    ItemKind::ResonanceCore,
                ));
            }
            23 => {
                obstacles.push(Obstacle::new(25.0, ground_y, ObstacleKind::Pylon));
                items.push(PickupItem::new(38.0, ground_y - 1.0, ItemKind::SpeedSurge));
            }
            24 => {
                landmarks.push(Landmark::new(
                    25.0,
                    ground_y,
                    LandmarkKind::RelayNodeConsole { restored: false },
                ));
            }
            _ => {
                let mut rng = ChunkRng::new(self.seed, index as u64);
                wind_currents.push(WindCurrent::new(
                    10.0,
                    ground_y,
                    25.0,
                    4.0,
                    WindKind::TailwindGust,
                ));
                items.push(PickupItem::new(15.0, ground_y - 1.0, ItemKind::GaleShard));
                items.push(PickupItem::new(30.0, ground_y - 1.0, ItemKind::GaleShard));
                if rng.next_f32() < 0.35 {
                    obstacles.push(Obstacle::new(35.0, ground_y, ObstacleKind::LowBarrier));
                }
            }
        }

        for &(id, world_x, name_key) in &CHECKPOINTS {
            if world_x / CHUNK_WIDTH == index {
                landmarks.push(Landmark::new(
                    (world_x % CHUNK_WIDTH) as f32,
                    ground_y,
                    LandmarkKind::CheckpointPylon {
                        id,
                        name_key: name_key.to_owned(),
                    },
                ));
            }
        }

        Chunk {
            index,
            obstacles,
            items,
            wind_currents,
            landmarks,
        }
    }
}

/// Deterministic 64-bit linear congruential generator for procedural chunk derivation.
struct ChunkRng {
    state: u64,
}

impl ChunkRng {
    fn new(seed: u64, chunk_index: u64) -> Self {
        let mut state = seed.wrapping_add(chunk_index.wrapping_mul(0x9e3779b97f4a7c15));
        state ^= state >> 30;
        state = state.wrapping_mul(0xbf58476d1ce4e5b9);
        state ^= state >> 27;
        state = state.wrapping_mul(0x94d049bb133111eb);
        state ^= state >> 31;
        Self { state: state | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16777216.0
    }
}
