//! Semantic versioning parser, ordering, and compatibility checker.

use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub is_prerelease: bool,
}

impl SemanticVersion {
    /// Parses a semantic version string (e.g. "v0.2.0", "0.2.1-rc1").
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        let stripped = trimmed
            .strip_prefix('v')
            .or_else(|| trimmed.strip_prefix('V'))
            .unwrap_or(trimmed);

        let (core, is_prerelease) = if let Some((base, _pre)) = stripped.split_once('-') {
            (base, true)
        } else {
            (stripped, false)
        };

        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        let patch = parts[2].parse::<u32>().ok()?;

        Some(Self {
            major,
            minor,
            patch,
            is_prerelease,
        })
    }

    /// Determines if `self` is a newer stable release than `other`.
    /// Prereleases on the normal stable channel are ignored.
    pub fn is_newer_than(&self, other: &Self) -> bool {
        if self.is_prerelease {
            return false;
        }
        self > other
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                non_eq => non_eq,
            },
            non_eq => non_eq,
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)?;
        if self.is_prerelease {
            write!(f, "-pre")?;
        }
        Ok(())
    }
}
