//! Chunk-based deterministic world generation and parallax management.

use serde::{Deserialize, Serialize};

use super::items::{ItemKind, PickupItem};
use super::obstacles::{Obstacle, ObstacleKind};
use super::parallax::ParallaxEngine;

pub const CHUNK_WIDTH: usize = 50;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub index: usize,
    pub obstacles: Vec<Obstacle>,
    pub items: Vec<PickupItem>,
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

        // Pre-generate initial chunks to cover the viewport
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

        // Generate ahead to cover lookahead
        let lookahead_x = self.scroll_x + (self.viewport_width as f32) + (CHUNK_WIDTH as f32 * 2.0);
        self.ensure_chunks_generated(lookahead_x);

        // Prune chunks that have moved completely behind the viewport (left edge < scroll_x - 20)
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

        // Deterministic pseudo-random sequence for this chunk
        let mut rng = ChunkRng::new(self.seed, index as u64);

        if index == 0 {
            // First chunk: safe warm-up zone with occasional welcome shards
            items.push(PickupItem::new(25.0, ground_y - 1.0, ItemKind::GaleShard));
            items.push(PickupItem::new(35.0, ground_y - 1.0, ItemKind::GaleShard));
        } else if index == 1 {
            // Early onboarding chunk: single low barrier to introduce Jump
            obstacles.push(Obstacle::new(28.0, ground_y, ObstacleKind::LowBarrier));
            // Airborne shards reward clean jump
            items.push(PickupItem::new(28.5, ground_y - 5.0, ItemKind::GaleShard));
        } else if index == 2 {
            // Early onboarding chunk: single overhead conduit to introduce Slide
            obstacles.push(Obstacle::new(28.0, ground_y, ObstacleKind::OverheadPipe));
            // Ground shard placed directly beneath the overhead conduit
            items.push(PickupItem::new(30.0, ground_y, ItemKind::GaleShard));
        } else {
            // Dynamic obstacle and pickup placement with guaranteed reaction gaps
            let mut cursor_x = 10.0;
            while cursor_x < (CHUNK_WIDTH as f32 - 15.0) {
                let r = rng.next_f32();
                let obstacle_kind = if r < 0.45 {
                    ObstacleKind::LowBarrier
                } else if r < 0.80 {
                    ObstacleKind::OverheadPipe
                } else {
                    ObstacleKind::Pylon
                };

                obstacles.push(Obstacle::new(cursor_x, ground_y, obstacle_kind));

                // Place pickup item before or above obstacle
                let item_roll = rng.next_f32();
                if item_roll < 0.40 {
                    let kind = if item_roll < 0.25 {
                        ItemKind::GaleShard
                    } else if item_roll < 0.32 {
                        ItemKind::WindShield
                    } else if item_roll < 0.37 {
                        ItemKind::SpeedSurge
                    } else {
                        ItemKind::FlowMultiplier
                    };

                    let item_y = match obstacle_kind {
                        ObstacleKind::LowBarrier => ground_y - 4.5, // Airborne rewards jumping
                        ObstacleKind::OverheadPipe => ground_y,     // Low rewards sliding
                        ObstacleKind::Pylon => ground_y - 1.0,
                    };
                    items.push(PickupItem::new(cursor_x + 1.0, item_y, kind));
                }

                // Advance cursor by at least 16.0 columns (guaranteed reaction window)
                let step = 16.0 + (rng.next_f32() * 14.0);
                cursor_x += step;
            }
        }

        Chunk {
            index,
            obstacles,
            items,
        }
    }
}

/// Lightweight deterministic 64-bit linear congruential generator for procedural chunk derivation.
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
