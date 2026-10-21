//! Native Windows & Linux Installer for Galerne.
//! Provides a trustworthy, standalone setup executable (`Galerne-Setup-x86_64.exe`).

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

const EMBEDDED_BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/embedded_galerne.bin"));

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_uninstall = args.iter().any(|a| a == "--uninstall" || a == "-u");
    let is_upgrade = args.iter().any(|a| a == "--upgrade");
    let is_silent = args
        .iter()
        .any(|a| a == "--silent" || a == "-s" || a == "-q" || a == "--quiet");
    let show_help = args.iter().any(|a| a == "--help" || a == "-h");

    let wait_pid: Option<u32> = args
        .windows(2)
        .find(|w| w[0] == "--wait-pid")
        .and_then(|w| w[1].parse().ok());
    let target_path: Option<PathBuf> = args
        .windows(2)
        .find(|w| w[0] == "--target")
        .map(|w| PathBuf::from(&w[1]));

    if show_help {
        println!("Galerne Native Setup v{}\n", galerne::VERSION);
        println!("USAGE:");
        println!("  Galerne-Setup-x86_64.exe [OPTIONS]\n");
        println!("OPTIONS:");
        println!("  -s, --silent       Install non-interactively to standard user path");
        println!("  -u, --uninstall    Cleanly remove Galerne binary, PATH entry, and shell hooks");
        println!("      --upgrade      Perform upgrade handoff replacing target binary safely");
        println!("  -h, --help         Show this help overview\n");
        return;
    }

    if !is_silent {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!(
            "║               GALERNE NATIVE INSTALLER v{}               ║",
            galerne::VERSION
        );
        println!("║   Native terminal game for developer wait times            ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
    }

    if is_upgrade {
        perform_upgrade(is_silent, wait_pid, target_path);
    } else if is_uninstall {
        perform_uninstall(is_silent);
    } else {
        perform_install(is_silent);
    }
}

fn get_install_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data)
                .join("Programs")
                .join("galerne")
        } else if let Ok(userprofile) = env::var("USERPROFILE") {
            PathBuf::from(userprofile).join(".galerne").join("bin")
        } else {
            PathBuf::from("C:\\galerne")
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".local").join("bin")
        } else {
            PathBuf::from("/usr/local/bin")
        }
    }
}

fn get_binary_name() -> &'static str {
    #[cfg(windows)]
    {
        "galerne.exe"
    }
    #[cfg(not(windows))]
    {
        "galerne"
    }
}

fn perform_install(silent: bool) {
    let install_dir = get_install_dir();
    let bin_name = get_binary_name();
    let target_bin = install_dir.join(bin_name);

    if !silent {
        println!("Destination Directory: {}", install_dir.display());
        println!("Target Executable:     {}", target_bin.display());
        print!("\nProceed with installation? [Y/n]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim().to_lowercase();
            if !trimmed.is_empty() && trimmed != "y" && trimmed != "yes" {
                println!("Installation cancelled.");
                return;
            }
        }
    }

    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("[✕] Error creating destination directory: {}", e);
        std::process::exit(1);
    }

    let binary_bytes = if !EMBEDDED_BINARY.is_empty() {
        EMBEDDED_BINARY.to_vec()
    } else {
        // Try reading sibling binary in current exe directory
        let current_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let sibling_bin = current_dir.join(bin_name);

        if sibling_bin.exists() {
            match fs::read(&sibling_bin) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!(
                        "[✕] Failed to read sibling '{}': {}",
                        sibling_bin.display(),
                        e
                    );
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!(
                "[✕] Error: Embedded binary not present and '{}' not found alongside installer.",
                bin_name
            );
            std::process::exit(1);
        }
    };

    let write_res = (|| -> io::Result<()> {
        let mut file = File::create(&target_bin)?;
        file.write_all(&binary_bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    })();

    if let Err(e) = write_res {
        eprintln!(
            "[✕] Failed to write binary to '{}': {}",
            target_bin.display(),
            e
        );
        std::process::exit(1);
    }
    println!("[✓] Installed binary: {}", target_bin.display());

    add_to_user_path(&install_dir);

    if let Err(e) = galerne::shell::setup_all_shells() {
        eprintln!(
            "[!] Warning: Shell integration setup encountered an issue: {}",
            e
        );
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║       GALERNE HAS BEEN INSTALLED SUCCESSFULLY!             ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    println!("Quick Start:");
    println!("  1. Open a fresh terminal window (PowerShell, Bash, or Zsh).");
    println!("  2. Run `galerne play` to begin a manual journey.");
    println!("  3. Run `galerne` to open the journey dashboard.");
    println!("  4. Run your regular builds (cargo build, npm install, etc.) —");
    println!("     Eligible long tasks can open Galerne while they run; short and unsupported tasks finish normally.\n");
}

fn perform_uninstall(silent: bool) {
    let install_dir = get_install_dir();
    let bin_name = get_binary_name();
    let target_bin = install_dir.join(bin_name);

    if !silent {
        print!("Are you sure you want to uninstall Galerne? [y/N]: ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err()
            || !matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
        {
            println!("Uninstallation cancelled.");
            return;
        }
    }

    if target_bin.exists() {
        if let Err(e) = fs::remove_file(&target_bin) {
            eprintln!("[!] Warning removing binary: {}", e);
        } else {
            println!("[✓] Removed binary: {}", target_bin.display());
        }
    }

    let _ = galerne::shell::uninstall_all_shells(false);

    remove_from_user_path(&install_dir);

    println!("\n[✓] Galerne uninstalled cleanly.");
}

fn add_to_user_path(dir: &Path) {
    let dir_str = dir.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        // Use PowerShell to update User PATH environment variable in Windows registry safely
        let ps_cmd = format!(
            "$dir = '{}'; $p = [Environment]::GetEnvironmentVariable('Path', 'User'); if (-not ($p -split ';' -contains $dir)) {{ [Environment]::SetEnvironmentVariable('Path', \"$p;$dir\", 'User') }}",
            dir_str.replace("'", "''")
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .status();
        println!("[✓] Added to User PATH: {}", dir_str);
    }

    #[cfg(not(windows))]
    {
        let current_path = env::var("PATH").unwrap_or_default();
        if !current_path.split(':').any(|p| p == dir_str) {
            println!("[ℹ] Please ensure '{}' is in your $PATH.", dir_str);
        } else {
            println!("[✓] Destination '{}' is already in your $PATH.", dir_str);
        }
    }
}

fn remove_from_user_path(dir: &Path) {
    let dir_str = dir.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        let ps_cmd = format!(
            "$dir = '{}'; $p = [Environment]::GetEnvironmentVariable('Path', 'User'); $newP = ($p -split ';' | Where-Object {{ $_ -ne $dir }}) -join ';'; [Environment]::SetEnvironmentVariable('Path', $newP, 'User')",
            dir_str.replace("'", "''")
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .status();
        println!("[✓] Removed from User PATH.");
    }

    #[cfg(not(windows))]
    {
        let _ = dir_str;
    }
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            s.contains(&pid.to_string())
        } else {
            false
        }
    }
    #[cfg(unix)]
    {
        let ret = unsafe { libc::kill(pid as libc::pid_t, 0) };
        ret == 0
    }
}

fn wait_for_process_exit(pid: u32, max_wait_secs: u64) -> bool {
    let start = std::time::Instant::now();
    let max_dur = std::time::Duration::from_secs(max_wait_secs);

    while start.elapsed() < max_dur {
        if !is_process_running(pid) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    !is_process_running(pid)
}

fn perform_upgrade(silent: bool, wait_pid: Option<u32>, target_path: Option<PathBuf>) {
    if let Some(pid) = wait_pid {
        if !silent {
            println!(
                "Waiting for previous Galerne process (PID {}) to exit...",
                pid
            );
        }
        if !wait_for_process_exit(pid, 15) {
            eprintln!(
                "[✕] Error: Previous process (PID {}) did not exit within timeout. Upgrade aborted safely.",
                pid
            );
            std::process::exit(1);
        }
    }

    let target_bin = target_path.unwrap_or_else(|| get_install_dir().join(get_binary_name()));

    if let Some(parent) = target_bin.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let binary_bytes = if !EMBEDDED_BINARY.is_empty() {
        EMBEDDED_BINARY.to_vec()
    } else {
        let current_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let sibling_bin = current_dir.join(get_binary_name());
        if sibling_bin.exists() {
            fs::read(&sibling_bin).unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    if binary_bytes.is_empty() {
        eprintln!("[✕] Error: No upgrade payload available");
        std::process::exit(1);
    }

    if let Err(e) =
        galerne::updater::windows::replace_and_verify_windows_binary(&target_bin, &binary_bytes)
    {
        eprintln!(
            "[✕] Upgrade smoke check failed: {}. Rolled back to previous version.",
            e
        );
        std::process::exit(1);
    }

    if let Some(parent) = target_bin.parent() {
        add_to_user_path(parent);
    }
    let _ = galerne::shell::setup_all_shells();

    if !silent {
        println!("[✓] Galerne successfully upgraded to v{}", galerne::VERSION);
    }
}
