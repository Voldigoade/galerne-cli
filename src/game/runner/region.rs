//! Region segments used to shape traversal and scenery.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorKind {
    LowerViaduct,
    InteriorGantry,
    HighSkyTruss,
    SwitchHouse,
}

pub fn sector_for_distance(distance: f32) -> SectorKind {
    if distance < 300.0 {
        SectorKind::LowerViaduct
    } else if distance < 650.0 {
        SectorKind::InteriorGantry
    } else if distance < 1000.0 {
        SectorKind::HighSkyTruss
    } else {
        SectorKind::SwitchHouse
    }
}
