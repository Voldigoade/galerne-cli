//! Project-scoped bounded duration history and intelligent wait-time prediction.
//!
//! Tracks historical execution durations scoped to (project_id, command_class).
//! Never stores raw command arguments or raw absolute file paths.
//! Avoids displaying gameplay for short developer waits (~5 seconds) while activating
//! almost immediately (150ms anti-flicker guard) for historically long operations.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::matcher::CommandClass;

pub const HISTORY_SCHEMA_VERSION: u32 = 3;
pub const PREDICTED_LONG_THRESHOLD_SECS: f32 = 8.0;
pub const KNOWN_LONG_ANTI_FLICKER_MS: u64 = 150;
pub const CONSERVATIVE_FALLBACK_DELAY_MS: u64 = 8000;
pub const MIN_SAMPLES_FOR_PREDICTION: usize = 2;
pub const MAX_SAMPLES_PER_RECORD: usize = 10;
pub const MAX_TRACKED_ENTRIES: usize = 64;
pub const MAX_SERIALIZED_BYTES: usize = 10 * 1024; // Strictly enforce < 10 KiB on disk

fn default_schema_version() -> u32 {
    HISTORY_SCHEMA_VERSION
}

/// Activation decision for a given task execution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActivationDecision {
    /// Task is historically known to take >= 8.0s.
    /// Galerne activates near-immediately after a tiny 150ms anti-flicker guard.
    KnownLong {
        median_secs: f32,
        anti_flicker_delay_ms: u64,
    },
    /// Task is historically known to finish quickly (< 8.0s).
    /// Kept hidden during normal duration, but will activate if execution unexpectedly exceeds 8.0s.
    KnownShort {
        median_secs: f32,
        fallback_delay_ms: u64,
    },
    /// Task has insufficient history (< 2 samples).
    /// Uses conservative fallback delay (8.0s) so ~5s builds complete without ever flashing Galerne.
    Unknown { fallback_delay_ms: u64 },
}

impl ActivationDecision {
    pub fn effective_delay_ms(&self) -> u64 {
        match self {
            ActivationDecision::KnownLong {
                anti_flicker_delay_ms,
                ..
            } => *anti_flicker_delay_ms,
            ActivationDecision::KnownShort {
                fallback_delay_ms, ..
            } => *fallback_delay_ms,
            ActivationDecision::Unknown { fallback_delay_ms } => *fallback_delay_ms,
        }
    }

    pub fn is_known_long(&self) -> bool {
        matches!(self, ActivationDecision::KnownLong { .. })
    }

    pub fn is_known_short(&self) -> bool {
        matches!(self, ActivationDecision::KnownShort { .. })
    }
}

/// Timing samples and metadata for a specific project + command class.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectTimingRecord {
    pub durations: Vec<f32>,
    pub last_updated_epoch_secs: u64,
}

/// Bounded rolling execution history keyed by opaque project ID and normalized command class.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DurationHistory {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub entries: HashMap<String, ProjectTimingRecord>,
}

impl Default for DurationHistory {
    fn default() -> Self {
        Self {
            schema_version: HISTORY_SCHEMA_VERSION,
            entries: HashMap::new(),
        }
    }
}

impl DurationHistory {
    pub fn make_key(project_id: &str, class: CommandClass) -> String {
        format!("{}:{}", project_id, class.label())
    }

    pub fn entries_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get_samples(&self, project_id: &str, class: CommandClass) -> Option<&[f32]> {
        let key = Self::make_key(project_id, class);
        self.entries.get(&key).map(|r| r.durations.as_slice())
    }

    /// Computes the median duration for a specific project and command class.
    pub fn median_duration(&self, project_id: &str, class: CommandClass) -> Option<f32> {
        let samples = self.get_samples(project_id, class)?;
        if samples.is_empty() {
            return None;
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 1 {
            Some(sorted[mid])
        } else {
            Some((sorted[mid - 1] + sorted[mid]) * 0.5)
        }
    }

    /// Evaluates the activation policy decision for an upcoming execution.
    pub fn evaluate_decision(&self, project_id: &str, class: CommandClass) -> ActivationDecision {
        if let Some(samples) = self.get_samples(project_id, class) {
            if samples.len() >= MIN_SAMPLES_FOR_PREDICTION {
                if let Some(median) = self.median_duration(project_id, class) {
                    if median >= PREDICTED_LONG_THRESHOLD_SECS {
                        return ActivationDecision::KnownLong {
                            median_secs: median,
                            anti_flicker_delay_ms: KNOWN_LONG_ANTI_FLICKER_MS,
                        };
                    } else {
                        return ActivationDecision::KnownShort {
                            median_secs: median,
                            fallback_delay_ms: CONSERVATIVE_FALLBACK_DELAY_MS,
                        };
                    }
                }
            }
        }

        ActivationDecision::Unknown {
            fallback_delay_ms: CONSERVATIVE_FALLBACK_DELAY_MS,
        }
    }

    /// Records a completed task duration.
    /// User-cancelled tasks must NOT be recorded here to prevent history distortion.
    pub fn record(&mut self, project_id: &str, class: CommandClass, duration_secs: f32) {
        let key = Self::make_key(project_id, class);
        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let record = self
            .entries
            .entry(key)
            .or_insert_with(|| ProjectTimingRecord {
                durations: Vec::with_capacity(MAX_SAMPLES_PER_RECORD),
                last_updated_epoch_secs: now_epoch,
            });

        record.durations.push(duration_secs);
        if record.durations.len() > MAX_SAMPLES_PER_RECORD {
            record.durations.remove(0);
        }
        record.last_updated_epoch_secs = now_epoch;

        // Prune oldest entries if capacity or serialized size exceeded
        self.prune_to_bounds();
    }

    /// Prunes oldest entries to guarantee both entry limit and strict < 10 KiB serialized size.
    pub fn prune_to_bounds(&mut self) {
        while self.entries.len() > MAX_TRACKED_ENTRIES {
            if !self.prune_oldest_entry() {
                break;
            }
        }

        while self.serialized_size_bytes() > MAX_SERIALIZED_BYTES && !self.entries.is_empty() {
            if !self.prune_oldest_entry() {
                break;
            }
        }
    }

    pub fn serialized_size_bytes(&self) -> usize {
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(0)
    }

    fn prune_oldest_entry(&mut self) -> bool {
        let oldest_key = self
            .entries
            .iter()
            .min_by_key(|(_, r)| r.last_updated_epoch_secs)
            .map(|(k, _)| k.clone());

        if let Some(key) = oldest_key {
            self.entries.remove(&key);
            true
        } else {
            false
        }
    }

    /// Loads history safely from disk.
    /// Handles backward compatibility: if previous schema (v1/v2) is encountered,
    /// or if entries contain old project ID formats, discards them cleanly without crashing.
    pub fn load_from_file(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        let Ok(mut file) = File::open(path) else {
            return Self::default();
        };

        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            return Self::default();
        }

        // Check if file is valid JSON
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            // Check schema_version
            if val.get("schema_version").and_then(|v| v.as_u64())
                == Some(HISTORY_SCHEMA_VERSION as u64)
            {
                if let Ok(mut history) = serde_json::from_value::<DurationHistory>(val) {
                    // Retain only valid entries whose keys are <64-char-hex-hash>:<command_class>
                    history.entries.retain(|k, _| {
                        let parts: Vec<&str> = k.split(':').collect();
                        parts.len() == 2
                            && parts[0].len() == 64
                            && parts[0].chars().all(|c| c.is_ascii_hexdigit())
                    });
                    history.prune_to_bounds();
                    return history;
                }
            }
        }

        // If incompatible schema or corrupt, start clean default history
        Self::default()
    }

    /// Saves history safely to disk, guaranteeing bounds.
    pub fn save_to_file(&mut self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.prune_to_bounds();
        let json = serde_json::to_string(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}
