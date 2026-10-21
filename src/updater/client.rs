//! Synchronous HTTPS client for GitHub release metadata and asset downloads.

use std::io::{self, Read};

use super::release::ReleaseMetadata;

pub const OFFICIAL_REPO: &str = "Voldigoade/galerne";

pub fn fetch_latest_release(repo: &str) -> io::Result<ReleaseMetadata> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let user_agent = format!("galerne/{}", crate::VERSION);

    let response = ureq::get(&url)
        .header("User-Agent", &user_agent)
        .header("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| io::Error::other(format!("HTTP error: {}", e)))?;

    let mut body = String::new();
    response
        .into_body()
        .into_reader()
        .read_to_string(&mut body)?;

    let metadata: ReleaseMetadata = serde_json::from_str(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    Ok(metadata)
}

pub fn download_bytes(url: &str) -> io::Result<Vec<u8>> {
    let user_agent = format!("galerne/{}", crate::VERSION);

    let response = ureq::get(url)
        .header("User-Agent", &user_agent)
        .call()
        .map_err(|e| io::Error::other(format!("Download failed: {}", e)))?;

    let mut bytes = Vec::new();
    response.into_body().into_reader().read_to_end(&mut bytes)?;

    Ok(bytes)
}
