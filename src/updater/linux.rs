//! Linux atomic binary replacement with automated rollback and smoke verification.

use std::fs::{self, File};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

/// Atomically replaces the current executable with newly downloaded bytes.
///
/// Steps:
/// 1. Verifies write permissions on target directory.
/// 2. Writes new binary to a temporary sibling file in the same directory (same filesystem).
/// 3. Sets permissions to 0o755.
/// 4. Backs up current executable to `<target>.old.<pid>`.
/// 5. Atomically renames temporary file over target executable.
/// 6. Runs smoke test (`--version`). If smoke test fails, rolls back to backup.
/// 7. Cleans temporary and backup files.
pub fn apply_linux_update(target_exe: &Path, new_binary_bytes: &[u8]) -> io::Result<()> {
    let parent = target_exe.parent().ok_or_else(|| {
        io::Error::other(format!(
            "Target path '{}' has no parent directory",
            target_exe.display()
        ))
    })?;

    if !parent.exists() {
        return Err(io::Error::other(format!(
            "Destination directory '{}' does not exist",
            parent.display()
        )));
    }

    let pid = std::process::id();
    let temp_path = parent.join(format!(".galerne_new_{}", pid));
    let backup_path = parent.join(format!(".galerne_old_{}", pid));

    // 1. Write new binary to temporary sibling file
    let write_res = (|| -> io::Result<()> {
        let mut file = File::create(&temp_path).map_err(|e| {
            io::Error::other(format!(
                "Cannot write to '{}': {}. If installed to a system path, re-install to ~/.local/bin or run with appropriate permissions (sudo is not invoked automatically).",
                parent.display(),
                e
            ))
        })?;
        file.write_all(new_binary_bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    // 2. Backup existing executable
    let had_existing = target_exe.exists();
    if had_existing {
        if let Err(e) = fs::rename(target_exe, &backup_path) {
            let _ = fs::remove_file(&temp_path);
            return Err(io::Error::other(format!(
                "Failed to preserve existing binary as backup: {}",
                e
            )));
        }
    }

    // 3. Atomically rename temporary file over target executable
    if let Err(e) = fs::rename(&temp_path, target_exe) {
        // Restore backup if rename failed
        if had_existing {
            let _ = fs::rename(&backup_path, target_exe);
        }
        let _ = fs::remove_file(&temp_path);
        return Err(io::Error::other(format!(
            "Failed to atomically replace executable: {}",
            e
        )));
    }

    // 4. Smoke-test the newly installed executable
    let smoke_res = (|| -> io::Result<()> {
        let output = Command::new(target_exe)
            .arg("--version")
            .output()
            .map_err(|e| io::Error::other(format!("Execution check failed: {}", e)))?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Smoke test failed with exit code {:?}",
                output.status.code()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.to_lowercase().contains("galerne") {
            return Err(io::Error::other(format!(
                "Smoke test output did not contain 'galerne': '{}'",
                stdout.trim()
            )));
        }

        Ok(())
    })();

    // 5. If smoke test failed, roll back!
    if let Err(e) = smoke_res {
        let _ = fs::remove_file(target_exe);
        if had_existing {
            let _ = fs::rename(&backup_path, target_exe);
        }
        return Err(io::Error::other(format!(
            "New binary failed smoke test ({}). Restored previous version.",
            e
        )));
    }

    // 6. Success: clean backup
    if had_existing {
        let _ = fs::remove_file(&backup_path);
    }

    Ok(())
}
