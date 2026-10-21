//! Windows updater via native setup installer delegation and PID handoff.

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// Stages the downloaded setup installer executable into temp,
/// launches it with `--upgrade --silent --wait-pid <current_pid> --target <target_exe>`,
/// and returns successfully so the calling process can exit cleanly.
pub fn apply_windows_update(setup_exe_bytes: &[u8], target_exe: &Path) -> io::Result<()> {
    let temp_dir = env::temp_dir();
    let setup_path = temp_dir.join(format!("Galerne-Setup-Upgrade-{}.exe", std::process::id()));

    let mut file = File::create(&setup_path)?;
    file.write_all(setup_exe_bytes)?;
    file.sync_all()?;
    drop(file);

    let current_pid = std::process::id();

    // Spawn detached setup process
    Command::new(&setup_path)
        .arg("--upgrade")
        .arg("--silent")
        .arg("--wait-pid")
        .arg(current_pid.to_string())
        .arg("--target")
        .arg(target_exe)
        .spawn()?;

    println!("[✓] Staged upgrade installer. Exiting current process for binary replacement.");
    Ok(())
}

/// Safely replaces target_bin with new binary_bytes and verifies smoke test (`--version`).
/// If smoke test fails (execution error, non-zero status, unexpected version output),
/// automatically rolls back to target_bin.exe.old and returns an Error.
pub fn replace_and_verify_windows_binary(target_bin: &Path, binary_bytes: &[u8]) -> io::Result<()> {
    let had_existing = target_bin.exists();
    let backup_bin = target_bin.with_extension("exe.old");

    if had_existing {
        fs::rename(target_bin, &backup_bin)?;
    }

    let write_res = (|| -> io::Result<()> {
        let mut file = File::create(target_bin)?;
        file.write_all(binary_bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(target_bin, fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    })();

    if let Err(e) = write_res {
        if had_existing {
            let _ = fs::rename(&backup_bin, target_bin);
        }
        return Err(e);
    }

    // Positively verify newly placed binary via smoke test
    let smoke_result = (|| -> io::Result<()> {
        let output = Command::new(target_bin)
            .arg("--version")
            .output()
            .map_err(|e| io::Error::other(format!("Execution check failed: {}", e)))?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Smoke test failed with exit code {:?}",
                output.status.code()
            )));
        }

        let out_str = String::from_utf8_lossy(&output.stdout);
        if !out_str.contains("galerne") {
            return Err(io::Error::other(format!(
                "Smoke test produced unexpected version output: '{}'",
                out_str.trim()
            )));
        }

        Ok(())
    })();

    if let Err(e) = smoke_result {
        if had_existing {
            let _ = fs::rename(&backup_bin, target_bin);
        }
        return Err(e);
    }

    if had_existing {
        let _ = fs::remove_file(&backup_bin);
    }

    Ok(())
}
