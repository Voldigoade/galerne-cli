//! SHA-256 calculation and release checksum digest verification.

use sha2::{Digest, Sha256};

/// Computes the SHA-256 hex digest of a byte slice.
pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Parses the 64-character SHA-256 hex digest from a release `.sha256` file content.
/// Handles standard formats like `<digest>  <filename>` or a bare `<digest>`.
pub fn parse_sha256_file(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Take first whitespace-separated token
        let token = trimmed.split_whitespace().next().unwrap_or("");
        if token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(token.to_lowercase());
        }
    }
    None
}

/// Verifies whether the calculated SHA-256 of `bytes` matches the expected hex digest.
pub fn verify_checksum(bytes: &[u8], expected_digest: &str) -> bool {
    let computed = compute_sha256(bytes);
    computed.eq_ignore_ascii_case(expected_digest.trim())
}
