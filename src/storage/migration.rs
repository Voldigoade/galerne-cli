//! Deterministic schema migrations and pre-migration backup infrastructure.

use std::io::{self, Error, ErrorKind};

use super::schema::SaveState;

/// Migrates serialized JSON payload stepwise from `from_version` to `to_version`.
pub fn migrate_save(from_version: u32, to_version: u32, payload: &[u8]) -> io::Result<SaveState> {
    if from_version == to_version {
        let state: SaveState = serde_json::from_slice(payload)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        return Ok(state);
    }

    if from_version > to_version {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Cannot downgrade save from schema v{} to v{}",
                from_version, to_version
            ),
        ));
    }

    // Stepwise migration chain
    // Stepwise migration chain for future versions (e.g. v1 -> v2)
    Err(Error::new(
        ErrorKind::InvalidData,
        format!(
            "No migration path available from schema v{} to v{}",
            from_version, to_version
        ),
    ))
}
