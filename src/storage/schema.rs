//! Save schema definitions, migrations, and snapshots.

use serde::{Deserialize, Serialize};

use crate::game::runner::world::CHECKPOINTS;
pub use crate::game::runner::RunnerSnapshot;
use crate::ui::i18n::Locale;

pub const CURRENT_SCHEMA_VERSION: u32 = 3;

/// Traversal and world item kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    GaleShard,
    ResonanceCore,
    InductionFilament,
    AncientCompass,
}

impl ItemKind {
    pub fn name_key(&self) -> &'static str {
        match self {
            Self::GaleShard => "item_gale_shard_name",
            Self::ResonanceCore => "item_resonance_core_name",
            Self::InductionFilament => "item_induction_filament_name",
            Self::AncientCompass => "item_ancient_compass_name",
        }
    }

    pub fn desc_key(&self) -> &'static str {
        match self {
            Self::GaleShard => "item_gale_shard_desc",
            Self::ResonanceCore => "item_resonance_core_desc",
            Self::InductionFilament => "item_induction_filament_desc",
            Self::AncientCompass => "item_ancient_compass_desc",
        }
    }
}

/// A slot in the player's compact inventory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: ItemKind,
    pub count: u32,
}

/// Compact persistent inventory with unlimited capacity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<InventorySlot>,
}

impl Inventory {
    pub fn add(&mut self, item: ItemKind, count: u32) {
        if count == 0 {
            return;
        }
        if let Some(slot) = self.items.iter_mut().find(|s| s.item == item) {
            slot.count = slot.count.saturating_add(count);
        } else {
            self.items.push(InventorySlot { item, count });
        }
    }

    pub fn count(&self, item: ItemKind) -> u32 {
        self.items
            .iter()
            .find(|s| s.item == item)
            .map(|s| s.count)
            .unwrap_or(0)
    }

    pub fn has(&self, item: ItemKind) -> bool {
        self.count(item) > 0
    }
}

/// Progress across the authored world and regional objectives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldProgress {
    pub current_region_id: String,
    pub checkpoints_unlocked: Vec<u32>,
    pub relay_restored: bool,
    pub secret_cache_found: bool,
    pub high_truss_explored: bool,
    #[serde(default)]
    pub canal_gate_opened: bool,
}

impl Default for WorldProgress {
    fn default() -> Self {
        Self {
            current_region_id: "cinder_line_viaduct".to_string(),
            checkpoints_unlocked: vec![0], // Initial start point is checkpoint 0
            relay_restored: false,
            secret_cache_found: false,
            high_truss_explored: false,
            canal_gate_opened: false,
        }
    }
}

impl WorldProgress {
    pub fn record_checkpoint(&mut self, checkpoint_id: u32) {
        let authored_count = CHECKPOINTS.len() as u32;
        if checkpoint_id >= authored_count {
            self.checkpoints_unlocked
                .retain(|unlocked_id| *unlocked_id < authored_count);
        }
        if !self.checkpoints_unlocked.contains(&checkpoint_id) {
            self.checkpoints_unlocked.push(checkpoint_id);
        }
    }
}

/// Discovery, conversation, and refuge state for Malo.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionProgress {
    #[serde(rename = "met_kael")]
    pub met_companion: bool,
    #[serde(rename = "spoken_to_kael")]
    pub spoken_to_companion: bool,
    #[serde(rename = "kael_at_refuge")]
    pub companion_at_refuge: bool,
}

/// State and facilities of the persistent base camp (The Refuge).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefugeProgress {
    pub power_restored: bool,
    pub workbench_unlocked: bool,
    pub visits_count: u32,
}

/// The complete persistent game state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveState {
    pub version: u32,
    pub total_play_time_secs: u64,
    pub total_wait_time_saved_secs: u64,
    pub total_runs_started: u32,
    #[serde(default)]
    pub total_distance_traveled: f32,
    #[serde(default)]
    pub total_enemies_killed: u32,
    #[serde(default)]
    pub total_bosses_killed: u32,
    pub currency_shards: u32,
    pub workshop: WorkshopLevels,
    pub active_run: Option<RunnerSnapshot>,
    pub settings: GameSettings,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default)]
    pub world_progress: WorldProgress,
    #[serde(default)]
    pub companion: CompanionProgress,
    #[serde(default)]
    pub refuge: RefugeProgress,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            total_play_time_secs: 0,
            total_wait_time_saved_secs: 0,
            total_runs_started: 0,
            total_distance_traveled: 0.0,
            total_enemies_killed: 0,
            total_bosses_killed: 0,
            currency_shards: 0,
            workshop: WorkshopLevels::default(),
            active_run: None,
            settings: GameSettings::default(),
            inventory: Inventory::default(),
            world_progress: WorldProgress::default(),
            companion: CompanionProgress::default(),
            refuge: RefugeProgress::default(),
        }
    }
}

impl SaveState {
    pub fn workbench_available(&self) -> bool {
        self.refuge.workbench_unlocked || self.inventory.has(ItemKind::InductionFilament)
    }

    pub fn unlock_workbench_from_inventory(&mut self) {
        if self.inventory.has(ItemKind::InductionFilament) {
            self.refuge.workbench_unlocked = true;
        }
    }
}

/// Permanent workshop upgrades purchased with Gale Shards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkshopLevels {
    pub base_health_level: u32,
    pub base_damage_level: u32,
    pub move_speed_level: u32,
    pub magnet_radius_level: u32,
    pub shard_multiplier_level: u32,
}

/// User configurable settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSettings {
    pub color_enabled: bool,
    pub animations_enabled: bool,
    pub auto_activate: bool,
    pub anti_flicker_delay_ms: u64,
    #[serde(default)]
    pub locale: Option<Locale>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            color_enabled: true,
            animations_enabled: true,
            auto_activate: true,
            anti_flicker_delay_ms: 8000, // 8.0s conservative fallback delay for unknown/short tasks
            locale: None,
        }
    }
}
