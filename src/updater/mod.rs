//! Safe, explicit, production-grade self-updater for Galerne.

pub mod checksum;
pub mod client;
pub mod release;
pub mod version;

pub use checksum::{compute_sha256, parse_sha256_file, verify_checksum};
pub use client::{download_bytes, fetch_latest_release, OFFICIAL_REPO};
pub use release::{select_platform_assets, ReleaseAsset, ReleaseMetadata};
pub use version::SemanticVersion;
