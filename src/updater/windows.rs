//! Windows updater via native setup installer delegation and PID handoff.

use std::env;
use std::fs::File;
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
