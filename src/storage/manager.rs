//! High-level persistence manager handling backups, quarantine, and archive recovery.

use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::atomic::{atomic_write, decode_and_verify, encode_with_header, read_file, MAGIC_BYTES};
use super::schema::{SaveState, CURRENT_SCHEMA_VERSION};

#[derive(Debug)]
pub enum SaveLoadError {
    Missing,
    UnsupportedSchema { version: u32, current: u32 },
    Corrupt(io::Error),
    Io(io::Error),
    Migration(io::Error),
}

impl std::fmt::Display for SaveLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "No save file exists"),
            Self::UnsupportedSchema { version, current } => write!(
                f,
                "Save created by a newer version of Galerne (schema v{}, current v{})",
                version, current
            ),
            Self::Corrupt(error) => write!(f, "Save is corrupt: {}", error),
            Self::Io(error) => write!(f, "Could not read save: {}", error),
            Self::Migration(error) => write!(f, "Could not migrate save: {}", error),
        }
    }
}

impl std::error::Error for SaveLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Corrupt(error) | Self::Io(error) | Self::Migration(error) => Some(error),
            Self::Missing | Self::UnsupportedSchema { .. } => None,
        }
    }
}

impl From<SaveLoadError> for io::Error {
    fn from(error: SaveLoadError) -> Self {
        let kind = match &error {
            SaveLoadError::Missing => ErrorKind::NotFound,
            SaveLoadError::UnsupportedSchema { .. } | SaveLoadError::Corrupt(_) => {
                ErrorKind::InvalidData
            }
            SaveLoadError::Io(error) => error.kind(),
            SaveLoadError::Migration(_) => ErrorKind::InvalidData,
        };
        Error::new(kind, error)
    }
}

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

        if state.version != CURRENT_SCHEMA_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Cannot save schema v{} with this Galerne version (schema v{})",
                    state.version, CURRENT_SCHEMA_VERSION
                ),
            ));
        }

        self.reject_future_schema(&save_path)?;
        self.reject_future_schema(&backup_path)?;

        let payload = serde_json::to_vec(state)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;

        let encoded = encode_with_header(state.version, &payload);

        match fs::metadata(&save_path) {
            Ok(metadata) if metadata.is_file() => {
                fs::copy(&save_path, &backup_path)?;
            }
            Ok(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Primary save path is not a regular file",
                ));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        atomic_write(&save_path, &encoded)
    }

    /// Loads the persistent save state with automatic corruption detection and fallback.
    pub fn load(&self) -> Result<SaveState, SaveLoadError> {
        let save_path = self.primary_save_path();
        let backup_path = self.backup_save_path();

        match self.load_file(&save_path) {
            Ok(state) => Ok(state),
            Err(SaveLoadError::Missing) => match self.load_file(&backup_path) {
                Err(error @ SaveLoadError::Corrupt(_)) => {
                    self.quarantine_corrupt_file(&backup_path)
                        .map_err(SaveLoadError::Io)?;
                    Err(error)
                }
                result => result,
            },
            Err(error @ SaveLoadError::Corrupt(_)) => {
                self.quarantine_corrupt_file(&save_path)
                    .map_err(SaveLoadError::Io)?;

                match self.load_file(&backup_path) {
                    Ok(backup_state) => {
                        self.save(&backup_state).map_err(SaveLoadError::Io)?;
                        Ok(backup_state)
                    }
                    Err(SaveLoadError::Missing) => Err(error),
                    Err(SaveLoadError::Corrupt(backup_error)) => {
                        self.quarantine_corrupt_file(&backup_path)
                            .map_err(SaveLoadError::Io)?;
                        Err(SaveLoadError::Corrupt(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "primary and backup saves are corrupt: {}; {}",
                                error, backup_error
                            ),
                        )))
                    }
                    Err(backup_error) => Err(backup_error),
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn load_or_default(&self) -> Result<SaveState, SaveLoadError> {
        match self.load() {
            Err(SaveLoadError::Missing) => Ok(SaveState::default()),
            result => result,
        }
    }

    /// Creates a pre-migration backup before upgrading to a newer schema version.
    pub fn create_migration_backup(
        &self,
        from_version: u32,
        to_version: u32,
    ) -> io::Result<PathBuf> {
        let save_path = self.primary_save_path();
        self.create_migration_backup_from(&save_path, from_version, to_version)
    }

    fn create_migration_backup_from(
        &self,
        source_path: &Path,
        from_version: u32,
        to_version: u32,
    ) -> io::Result<PathBuf> {
        if !source_path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "No save to back up"));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let backup_name = format!(
            "save.dat.migration_v{}_to_v{}.{}.bak",
            from_version, to_version, timestamp
        );
        let backup_path = self.data_dir.join(&backup_name);

        fs::copy(source_path, &backup_path)?;
        Ok(backup_path)
    }

    fn load_file(&self, path: &Path) -> Result<SaveState, SaveLoadError> {
        let bytes = read_file(path).map_err(classify_read_error)?;
        if bytes.len() >= 8 && &bytes[..4] == MAGIC_BYTES {
            let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            if version > CURRENT_SCHEMA_VERSION {
                return Err(SaveLoadError::UnsupportedSchema {
                    version,
                    current: CURRENT_SCHEMA_VERSION,
                });
            }
        }
        let (version, payload) = decode_and_verify(&bytes).map_err(SaveLoadError::Corrupt)?;

        if version < CURRENT_SCHEMA_VERSION {
            self.create_migration_backup_from(path, version, CURRENT_SCHEMA_VERSION)
                .map_err(SaveLoadError::Io)?;
            let migrated = super::migration::migrate_save(version, CURRENT_SCHEMA_VERSION, payload)
                .map_err(SaveLoadError::Migration)?;
            if path == self.primary_save_path() {
                self.save(&migrated).map_err(SaveLoadError::Io)?;
            }
            return Ok(migrated);
        }

        let state: SaveState = serde_json::from_slice(payload)
            .map_err(|error| SaveLoadError::Corrupt(Error::new(ErrorKind::InvalidData, error)))?;
        Ok(state)
    }

    fn reject_future_schema(&self, path: &Path) -> io::Result<()> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };

        if bytes.len() >= 8 && &bytes[..4] == MAGIC_BYTES {
            let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            if version > CURRENT_SCHEMA_VERSION {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Refusing to replace save schema v{} with schema v{}",
                        version, CURRENT_SCHEMA_VERSION
                    ),
                ));
            }
        }
        Ok(())
    }

    fn quarantine_corrupt_file(&self, path: &Path) -> io::Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let original_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("save.dat");
        let corrupt_name = format!("{}.corrupt.{}", original_name, timestamp);
        let corrupt_path = self.data_dir.join(corrupt_name);
        fs::rename(path, &corrupt_path)?;
        Ok(corrupt_path)
    }

    /// Archives current save state and initializes fresh progression.
    pub fn archive_and_reset(&self) -> io::Result<PathBuf> {
        let save_path = self.primary_save_path();
        self.reject_future_schema(&save_path)?;
        self.reject_future_schema(&self.backup_save_path())?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let archive_name = format!("archive_{}.dat", timestamp);
        let archive_path = self.data_dir.join(&archive_name);

        if save_path.exists() {
            fs::rename(&save_path, &archive_path)?;
        } else {
            let backup_path = self.backup_save_path();
            if backup_path.exists() {
                fs::rename(&backup_path, &archive_path)?;
            }
        }

        self.save(&SaveState::default())?;
        let backup_path = self.backup_save_path();
        if backup_path.exists() {
            let _ = fs::remove_file(backup_path);
        }
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
        self.reject_future_schema(&self.primary_save_path())?;
        self.reject_future_schema(&self.backup_save_path())?;
        let archive_path = self.data_dir.join(archive_name);
        if !archive_path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Archive '{}' not found", archive_name),
            ));
        }

        // An archive is untrusted input; validate its header, checksum, and schema before replacing the active save.
        let state = self.load_file(&archive_path)?;
        self.save(&state)
    }
}

fn classify_read_error(error: io::Error) -> SaveLoadError {
    if error.kind() == ErrorKind::NotFound {
        SaveLoadError::Missing
    } else {
        SaveLoadError::Io(error)
    }
}
