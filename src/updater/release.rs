//! GitHub Release metadata representations and platform asset resolution.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub tag_name: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetResolutionError {
    UnsupportedPlatform(String),
    MissingBinaryAsset(String),
    MissingChecksumAsset(String),
}

impl std::fmt::Display for AssetResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetResolutionError::UnsupportedPlatform(p) => {
                write!(f, "Platform '{}' does not have official prebuilt binary releases. Please build from source using: cargo install --git https://github.com/Voldigoade/galerne.git --bin galerne", p)
            }
            AssetResolutionError::MissingBinaryAsset(name) => {
                write!(f, "Release asset '{}' not found in release metadata", name)
            }
            AssetResolutionError::MissingChecksumAsset(name) => {
                write!(f, "Checksum asset '{}' not found in release metadata", name)
            }
        }
    }
}

impl std::error::Error for AssetResolutionError {}

/// Resolves the expected binary and checksum assets for the current platform.
pub fn select_platform_assets(
    metadata: &ReleaseMetadata,
    os: &str,
    arch: &str,
) -> Result<(ReleaseAsset, ReleaseAsset), AssetResolutionError> {
    if arch != "x86_64" && arch != "amd64" {
        return Err(AssetResolutionError::UnsupportedPlatform(format!(
            "{}-{}",
            os, arch
        )));
    }

    let (bin_pattern, sha_pattern) = match os {
        "linux" => (
            "galerne-x86_64-unknown-linux-gnu",
            "galerne-x86_64-unknown-linux-gnu.sha256",
        ),
        "windows" => (
            "Galerne-Setup-x86_64.exe",
            "Galerne-Setup-x86_64.exe.sha256",
        ),
        other => return Err(AssetResolutionError::UnsupportedPlatform(other.to_string())),
    };

    let bin_asset = metadata
        .assets
        .iter()
        .find(|a| a.name == bin_pattern)
        .cloned()
        .ok_or_else(|| AssetResolutionError::MissingBinaryAsset(bin_pattern.to_string()))?;

    let sha_asset = metadata
        .assets
        .iter()
        .find(|a| a.name == sha_pattern)
        .cloned()
        .ok_or_else(|| AssetResolutionError::MissingChecksumAsset(sha_pattern.to_string()))?;

    Ok((bin_asset, sha_asset))
}
