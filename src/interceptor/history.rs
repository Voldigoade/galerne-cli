//! Rolling task duration history and execution time prediction.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::matcher::CommandClass;

pub const PREDICTED_LONG_THRESHOLD_SECS: f32 = 8.0;
pub const MAX_SAMPLES_PER_CLASS: usize = 10;

/// Bounded rolling execution history per normalized command class.
/// Explicitly avoids storing raw command strings or arguments for privacy.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DurationHistory {
    pub records: HashMap<CommandClass, Vec<f32>>,
}

impl DurationHistory {
    /// Records a new duration sample for a given command class.
    pub fn record(&mut self, class: CommandClass, duration_secs: f32) {
        let samples = self.records.entry(class).or_default();
        samples.push(duration_secs);
        if samples.len() > MAX_SAMPLES_PER_CLASS {
            samples.remove(0);
        }
    }

    /// Computes the median duration for a command class.
    pub fn median_duration(&self, class: CommandClass) -> Option<f32> {
        let samples = self.records.get(&class)?;
        if samples.is_empty() {
            return None;
        }

        let mut sorted = samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 1 {
            Some(sorted[mid])
        } else {
            Some((sorted[mid - 1] + sorted[mid]) * 0.5)
        }
    }

    /// Determines if a task is predicted to take long enough to enter game view immediately.
    pub fn is_predicted_long(&self, class: CommandClass) -> bool {
        // Require at least 2 samples to predict reliably
        if let Some(samples) = self.records.get(&class) {
            if samples.len() >= 2 {
                if let Some(median) = self.median_duration(class) {
                    return median >= PREDICTED_LONG_THRESHOLD_SECS;
                }
            }
        }
        false
    }

    /// Loads history from a file, returning default if file does not exist.
    pub fn load_from_file(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        let Ok(mut file) = File::open(path) else {
            return Self::default();
        };

        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Saves history safely to disk.
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}
