//! Privacy-preserving project identity derivation for wait-time prediction.
//!
//! Maps repositories and working directories to opaque, salted identifiers
//! without persisting raw file paths or command lines.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

/// Discovers the root directory of the current project or workspace.
/// Walks up the directory hierarchy looking for a valid `.git` repository or worktree.
/// Falls back to the provided starting directory if no valid `.git` is found.
pub fn find_project_root(start_dir: &Path) -> PathBuf {
    let mut current = if start_dir.is_relative() {
        std::env::current_dir()
            .map(|cwd| cwd.join(start_dir))
            .unwrap_or_else(|_| start_dir.to_path_buf())
    } else {
        start_dir.to_path_buf()
    };

    // Canonicalize if possible to resolve symlinks
    if let Ok(canonical) = current.canonicalize() {
        current = canonical;
    }

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

/// Derives a privacy-preserving opaque project identifier from the root path and a local salt.
///
/// Output format: `p_<16 hex chars>`
pub fn derive_project_id(root: &Path, salt: &str) -> String {
    let normalized = normalize_path(root);

    let mut h1 = crc32fast::Hasher::new();
    h1.update(salt.as_bytes());
    h1.update(b":path_a:");
    h1.update(normalized.as_bytes());
    let c1 = h1.finalize();

    let mut h2 = crc32fast::Hasher::new();
    h2.update(salt.as_bytes());
    h2.update(b":path_b:");
    h2.update(normalized.as_bytes());
    let c2 = h2.finalize();

    format!("p_{:08x}{:08x}", c1, c2)
}

/// Retrieves or initializes a persistent installation-local salt stored in `<data_dir>/salt`.
pub fn get_or_create_salt(data_dir: &Path) -> io::Result<String> {
    let salt_path = data_dir.join("salt");
    if salt_path.exists() {
        if let Ok(content) = fs::read_to_string(&salt_path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    if let Some(parent) = salt_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Generate random salt using high-resolution timestamps, process ID, and memory entropy
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789);
    let pid = std::process::id();
    let stack_marker = &now as *const u128 as usize;

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&now.to_le_bytes());
    hasher.update(&pid.to_le_bytes());
    hasher.update(&stack_marker.to_le_bytes());
    let r1 = hasher.finalize();

    let mut hasher2 = crc32fast::Hasher::new();
    hasher2.update(&pid.to_le_bytes());
    hasher2.update(&now.to_be_bytes());
    hasher2.update(b"salt_seed");
    let r2 = hasher2.finalize();

    let generated_salt = format!("{:08x}{:08x}", r1, r2);
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
    let p = if let Ok(c) = path.canonicalize() {
        c
    } else {
        path.to_path_buf()
    };

    #[allow(unused_mut)]
    let mut s = p.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        // On Windows, normalize path separators and casing for consistent hashing
        s = s.replace('/', "\\").to_lowercase();
    }
    s
}
