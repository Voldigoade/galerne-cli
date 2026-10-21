//! Safe, explicit, production-grade self-updater for Galerne.

pub mod checksum;
pub mod client;
pub mod linux;
pub mod release;
pub mod version;
pub mod windows;

pub use checksum::{compute_sha256, parse_sha256_file, verify_checksum};
pub use client::{download_bytes, fetch_latest_release, OFFICIAL_REPO};
pub use linux::apply_linux_update;
pub use release::{select_platform_assets, ReleaseAsset, ReleaseMetadata};
pub use version::SemanticVersion;
pub use windows::apply_windows_update;

use std::io::{self, Write};

/// Executes the explicit update process.
/// If `check_only` is true, queries and displays release availability without modifying the system.
pub fn run_update(check_only: bool) -> io::Result<()> {
    println!("Checking for Galerne updates from {}...", OFFICIAL_REPO);

    let current_version_str = crate::VERSION;
    let current_ver = SemanticVersion::parse(current_version_str).ok_or_else(|| {
        io::Error::other(format!(
            "Unable to parse current version '{}'",
            current_version_str
        ))
    })?;

    let metadata = fetch_latest_release(OFFICIAL_REPO)?;

    let latest_ver = SemanticVersion::parse(&metadata.tag_name).ok_or_else(|| {
        io::Error::other(format!(
            "Unable to parse release tag '{}'",
            metadata.tag_name
        ))
    })?;

    println!("Current version: {}", current_ver);
    println!("Latest version:  {}", latest_ver);

    if !latest_ver.is_newer_than(&current_ver) {
        println!("\nGalerne {}\nAlready up to date.", current_ver);
        return Ok(());
    }

    println!("\nUpdate available: {} -> {}", current_ver, latest_ver);

    if check_only {
        println!("Run `galerne update` to download and install this update.");
        return Ok(());
    }

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    println!("Selecting release asset for {} ({})...", os, arch);
    let (bin_asset, sha_asset) =
        select_platform_assets(&metadata, os, arch).map_err(|e| io::Error::other(e.to_string()))?;

    println!(
        "Downloading {} ({} bytes)...",
        bin_asset.name, bin_asset.size
    );
    let payload = download_bytes(&bin_asset.browser_download_url)?;

    println!("Downloading checksum {}...", sha_asset.name);
    let sha_bytes = download_bytes(&sha_asset.browser_download_url)?;
    let sha_content = String::from_utf8_lossy(&sha_bytes);

    let expected_digest = parse_sha256_file(&sha_content).ok_or_else(|| {
        io::Error::other(format!(
            "Failed to parse 64-character SHA-256 hex digest from {}",
            sha_asset.name
        ))
    })?;

    print!("Verifying SHA-256 integrity digest... ");
    let _ = io::stdout().flush();
    if !verify_checksum(&payload, &expected_digest) {
        println!("[FAILED]");
        return Err(io::Error::other(format!(
            "Checksum mismatch! Expected SHA-256 '{}', computed '{}'",
            expected_digest,
            compute_sha256(&payload)
        )));
    }
    println!("[OK]");

    let current_exe = std::env::current_exe()?;

    #[cfg(unix)]
    {
        println!(
            "Applying atomic binary replacement to {}...",
            current_exe.display()
        );
        apply_linux_update(&current_exe, &payload)?;
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!(
            "║    GALERNE HAS BEEN UPDATED SUCCESSFULLY TO {}!    ║",
            latest_ver
        );
        println!("╚════════════════════════════════════════════════════════════╝\n");
    }

    #[cfg(windows)]
    {
        println!("Staging Windows upgrade installer...");
        apply_windows_update(&payload, &current_exe)?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(io::Error::other(
            "Self-update is not supported on this operating system",
        ));
    }

    Ok(())
}
