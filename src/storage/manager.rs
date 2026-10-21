//! High-level persistence manager handling backups, quarantine, and archive recovery.

use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::atomic::{atomic_write, decode_and_verify, encode_with_header, read_file};
use super::schema::{SaveState, CURRENT_SCHEMA_VERSION};

pub struct SaveManager {
    data_dir: PathBuf,
}

impl SaveManager {
    /// Constructs a save manager using default system paths or `GALERNE_DATA_DIR`.
    pub fn default_paths() -> Self {
        let dir = if let Ok(custom) = env::var("GALERNE_DATA_DIR") {
            PathBuf::from(custom)
        } else if let Ok(xdg) = env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg).join("galerne")
        } else if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("galerne")
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("galerne")
        } else if let Ok(userprofile) = env::var("USERPROFILE") {
            PathBuf::from(userprofile).join(".galerne")
        } else {
            PathBuf::from(".galerne")
        };

        Self { data_dir: dir }
    }

    /// Constructs a save manager targeting a specific directory (useful for testing).
    pub fn with_dir<P: Into<PathBuf>>(dir: P) -> Self {
        Self {
            data_dir: dir.into(),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn primary_save_path(&self) -> PathBuf {
        self.data_dir.join("save.dat")
    }

    pub fn backup_save_path(&self) -> PathBuf {
        self.data_dir.join("save.dat.bak")
    }

    /// Saves the state atomically, rotating previous good save to `.bak`.
    pub fn save(&self, state: &SaveState) -> io::Result<()> {
        let save_path = self.primary_save_path();
        let backup_path = self.backup_save_path();

        let payload = serde_json::to_vec(state)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;

        let encoded = encode_with_header(state.version, &payload);

        // If primary save already exists and is valid, back it up first
        if save_path.exists() {
            let _ = fs::copy(&save_path, &backup_path);
        }

        atomic_write(&save_path, &encoded)
    }

    /// Loads the persistent save state with automatic corruption detection and fallback.
    pub fn load(&self) -> io::Result<SaveState> {
        let save_path = self.primary_save_path();
        let backup_path = self.backup_save_path();

        if !save_path.exists() {
            if backup_path.exists() {
                return self.load_file(&backup_path);
            }
            return Ok(SaveState::default());
        }

        match self.load_file(&save_path) {
            Ok(state) => Ok(state),
            Err(e) if e.to_string().contains("Save created by a newer version") => {
                // Do NOT quarantine or overwrite newer saves
                Err(e)
            }
            Err(_corruption_err) => {
                // Primary is corrupted: quarantine it
                self.quarantine_corrupt_file(&save_path);

                // Attempt to restore from backup
                if backup_path.exists() {
                    match self.load_file(&backup_path) {
                        Ok(backup_state) => {
                            // Re-save recovered backup to primary path
                            let _ = self.save(&backup_state);
                            return Ok(backup_state);
                        }
                        Err(_) => {
                            self.quarantine_corrupt_file(&backup_path);
                        }
                    }
                }

                // If backup is also unreadable or missing, initialize fresh state
                let fresh = SaveState::default();
                let _ = self.save(&fresh);
                Ok(fresh)
            }
        }
    }

    /// Creates a pre-migration backup before upgrading to a newer schema version.
    pub fn create_migration_backup(
        &self,
        from_version: u32,
        to_version: u32,
    ) -> io::Result<PathBuf> {
        let save_path = self.primary_save_path();
        if !save_path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "No primary save to back up",
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let backup_name = format!(
            "save.dat.migration_v{}_to_v{}.{}.bak",
            from_version, to_version, timestamp
        );
        let backup_path = self.data_dir.join(&backup_name);

        fs::copy(&save_path, &backup_path)?;
        Ok(backup_path)
    }

    fn load_file(&self, path: &Path) -> io::Result<SaveState> {
        let bytes = read_file(path)?;
        let (version, payload) = decode_and_verify(&bytes)?;

        if version > CURRENT_SCHEMA_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Save created by a newer version of Galerne (schema v{}, current v{})",
                    version, CURRENT_SCHEMA_VERSION
                ),
            ));
        }

        if version < CURRENT_SCHEMA_VERSION {
            // Create automatic pre-migration backup
            let _ = self.create_migration_backup(version, CURRENT_SCHEMA_VERSION);
            let migrated =
                super::migration::migrate_save(version, CURRENT_SCHEMA_VERSION, payload)?;
            self.save(&migrated)?;
            return Ok(migrated);
        }

        let state: SaveState = serde_json::from_slice(payload)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        Ok(state)
    }

    fn quarantine_corrupt_file(&self, path: &Path) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let corrupt_name = format!("save.dat.corrupt.{}", timestamp);
        let corrupt_path = self.data_dir.join(corrupt_name);
        let _ = fs::rename(path, corrupt_path);
    }

    /// Archives current save state and initializes fresh progression.
    pub fn archive_and_reset(&self) -> io::Result<PathBuf> {
        let save_path = self.primary_save_path();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let archive_name = format!("archive_{}.dat", timestamp);
        let archive_path = self.data_dir.join(&archive_name);

        if save_path.exists() {
            fs::rename(&save_path, &archive_path)?;
        }
        let backup_path = self.backup_save_path();
        if backup_path.exists() {
            let _ = fs::remove_file(backup_path);
        }

        // Initialize fresh state
        self.save(&SaveState::default())?;
        Ok(archive_path)
    }

    /// Lists all archived save files.
    pub fn list_archives(&self) -> io::Result<Vec<(String, u64)>> {
        if !self.data_dir.exists() {
            return Ok(Vec::new());
        }

        let mut archives = Vec::new();
        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("archive_") && name.ends_with(".dat") {
                let metadata = entry.metadata()?;
                archives.push((name, metadata.len()));
            }
        }
        archives.sort_by(|a, b| b.0.cmp(&a.0)); // Newest first
        Ok(archives)
    }

    /// Restores an archived save over the active save file.
    pub fn restore_archive(&self, archive_name: &str) -> io::Result<()> {
        let archive_path = self.data_dir.join(archive_name);
        if !archive_path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Archive '{}' not found", archive_name),
            ));
        }

        // Validate archive integrity before restoring
        let state = self.load_file(&archive_path)?;
        self.save(&state)
    }
}
