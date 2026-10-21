//! Authored region definitions, sectors, checkpoints, and landmarks.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectorKind {
    LowerViaduct,
    InteriorGantry,
    HighSkyTruss,
    GaleNodeStation,
}

impl SectorKind {
    pub fn name_key(&self) -> &'static str {
        match self {
            Self::LowerViaduct => "sector_lower_viaduct",
            Self::InteriorGantry => "sector_interior_gantry",
            Self::HighSkyTruss => "sector_high_sky_truss",
            Self::GaleNodeStation => "sector_gale_node_station",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: u32,
    pub distance: f32,
    pub name_key: &'static str,
    pub sector: SectorKind,
}

pub const CINDER_LINE_TOTAL_LENGTH: f32 = 1300.0;

pub const CHECKPOINTS: &[Checkpoint] = &[
    Checkpoint {
        id: 0,
        distance: 0.0,
        name_key: "cp_departure",
        sector: SectorKind::LowerViaduct,
    },
    Checkpoint {
        id: 1,
        distance: 250.0,
        name_key: "cp_shanty_outpost",
        sector: SectorKind::LowerViaduct,
    },
    Checkpoint {
        id: 2,
        distance: 550.0,
        name_key: "cp_shaft_airlock",
        sector: SectorKind::InteriorGantry,
    },
    Checkpoint {
        id: 3,
        distance: 900.0,
        name_key: "cp_high_roost",
        sector: SectorKind::HighSkyTruss,
    },
    Checkpoint {
        id: 4,
        distance: 1250.0,
        name_key: "cp_relay_node",
        sector: SectorKind::GaleNodeStation,
    },
];

pub fn sector_for_distance(distance: f32) -> SectorKind {
    if distance < 300.0 {
        SectorKind::LowerViaduct
    } else if distance < 650.0 {
        SectorKind::InteriorGantry
    } else if distance < 1000.0 {
        SectorKind::HighSkyTruss
    } else {
        SectorKind::GaleNodeStation
    }
}

pub fn active_checkpoint_for_distance(distance: f32) -> &'static Checkpoint {
    let mut best = &CHECKPOINTS[0];
    for cp in CHECKPOINTS {
        if distance >= cp.distance {
            best = cp;
        } else {
            break;
        }
    }
    best
}
