//! Privacy-preserving project identity derivation for wait-time prediction.
//!
//! Maps repositories and working directories to opaque, salted identifiers
//! without persisting raw file paths or command lines.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// Validates whether a `.git` path represents an actual git repository or worktree.
fn is_valid_git_marker(marker: &Path) -> bool {
    if marker.is_dir() {
        // A real git repository directory contains a HEAD or config file
        marker.join("HEAD").exists() || marker.join("config").exists()
    } else if marker.is_file() {
        // A git worktree or submodule .git file contains `gitdir:`
        if let Ok(content) = fs::read_to_string(marker) {
            content.contains("gitdir:")
        } else {
            false
        }
    } else {
        false
    }
}

/// Canonicalizes a path if possible, stripping the Windows verbatim prefix (`\\?\`).
pub fn clean_canonicalize(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        let s = canonical.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
        canonical
    } else {
        path.to_path_buf()
    }
}

/// Discovers the root directory of the current project or workspace.
/// Walks up the directory hierarchy looking for a valid `.git` repository or worktree.
/// Falls back to the provided starting directory if no valid `.git` is found.
pub fn find_project_root(start_dir: &Path) -> PathBuf {
    let current = if start_dir.is_relative() {
        std::env::current_dir()
            .map(|cwd| cwd.join(start_dir))
            .unwrap_or_else(|_| start_dir.to_path_buf())
    } else {
        start_dir.to_path_buf()
    };

    let current = clean_canonicalize(&current);

    let mut cursor = current.as_path();
    loop {
        let git_marker = cursor.join(".git");
        if is_valid_git_marker(&git_marker) {
            return cursor.to_path_buf();
        }

        match cursor.parent() {
            Some(parent) => cursor = parent,
            None => break,
        }
    }

    current
}

/// Derives a privacy-preserving opaque project identifier using SHA-256 over normalized path and local salt.
///
/// Output format: 64-character lowercase hex string.
pub fn derive_project_id(root: &Path, salt: &str) -> String {
    let normalized = normalize_path(root);

    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(normalized.as_bytes());
    let digest = hasher.finalize();

    format!("{:064x}", digest)
}

/// Retrieves or initializes a persistent installation-local 32-byte salt stored in `<data_dir>/salt`.
/// Salt is encoded as a 64-character lowercase hex string.
pub fn get_or_create_salt(data_dir: &Path) -> io::Result<String> {
    let salt_path = data_dir.join("salt");
    if salt_path.exists() {
        if let Ok(content) = fs::read_to_string(&salt_path) {
            let trimmed = content.trim();
            if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(trimmed.to_string());
            }
        }
    }

    if let Some(parent) = salt_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Generate 32 bytes of cryptographically secure random entropy from OS source
    let mut salt_bytes = [0u8; 32];
    getrandom::getrandom(&mut salt_bytes)
        .map_err(|e| io::Error::other(format!("OS random source error: {}", e)))?;

    let generated_salt: String = salt_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let _ = fs::write(&salt_path, &generated_salt);
    Ok(generated_salt)
}

/// Resolves the current workspace's project ID using cwd and persistent salt.
pub fn resolve_current_project_id(data_dir: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = find_project_root(&cwd);
    let salt = get_or_create_salt(data_dir).unwrap_or_else(|_| "galerne_fallback_salt".to_string());
    derive_project_id(&root, &salt)
}

fn normalize_path(path: &Path) -> String {
    let p = clean_canonicalize(path);
    #[allow(unused_mut)]
    let mut s = p.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        // On Windows, normalize path separators and casing for consistent hashing
        s = s.replace('/', "\\").to_lowercase();
    }
    s
}
