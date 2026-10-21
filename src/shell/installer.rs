//! Reversible, non-destructive shell integration for Bash, Zsh, and PowerShell.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const START_MARKER: &str = "# >>> galerne shell integration >>>";
pub const END_MARKER: &str = "# <<< galerne shell integration <<<";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    PowerShell,
}

impl ShellType {
    pub fn name(&self) -> &'static str {
        match self {
            ShellType::Bash => "Bash",
            ShellType::Zsh => "Zsh",
            ShellType::PowerShell => "PowerShell",
        }
    }

    pub fn generate_block(&self) -> String {
        match self {
            ShellType::Bash | ShellType::Zsh => format!(
                "{}\n\
                 # Automatically activates during developer wait times (cargo, npm, docker, make).\n\
                 function cargo() {{\n\
                     if command -v galerne >/dev/null 2>&1; then\n\
                         galerne wrap cargo \"$@\"\n\
                     else\n\
                         command cargo \"$@\"\n\
                     fi\n\
                 }}\n\
                 function npm() {{\n\
                     if command -v galerne >/dev/null 2>&1; then\n\
                         galerne wrap npm \"$@\"\n\
                     else\n\
                         command npm \"$@\"\n\
                     fi\n\
                 }}\n\
                 function docker() {{\n\
                     if command -v galerne >/dev/null 2>&1; then\n\
                         galerne wrap docker \"$@\"\n\
                     else\n\
                         command docker \"$@\"\n\
                     fi\n\
                 }}\n\
                 {}\n",
                START_MARKER, END_MARKER
            ),
            ShellType::PowerShell => format!(
                "{}\n\
                 # Automatically activates during developer wait times (cargo, npm, docker, make).\n\
                 function cargo {{\n\
                     if (Get-Command galerne -ErrorAction SilentlyContinue) {{\n\
                         & galerne wrap cargo @args\n\
                     }} else {{\n\
                         & (Get-Command -CommandType Application cargo) @args\n\
                     }}\n\
                 }}\n\
                 function npm {{\n\
                     if (Get-Command galerne -ErrorAction SilentlyContinue) {{\n\
                         & galerne wrap npm @args\n\
                     }} else {{\n\
                         & (Get-Command -CommandType Application npm) @args\n\
                     }}\n\
                 }}\n\
                 function docker {{\n\
                     if (Get-Command galerne -ErrorAction SilentlyContinue) {{\n\
                         & galerne wrap docker @args\n\
                     }} else {{\n\
                         & (Get-Command -CommandType Application docker) @args\n\
                     }}\n\
                 }}\n\
                 {}\n",
                START_MARKER, END_MARKER
            ),
        }
    }
}

/// Detects standard user shell configuration paths across operating systems.
pub fn detect_shell_configs() -> Vec<(ShellType, PathBuf)> {
    let mut configs = Vec::new();

    if let Ok(home) = env::var("HOME") {
        let home_p = PathBuf::from(home);

        let bashrc = home_p.join(".bashrc");
        if bashrc.exists() {
            configs.push((ShellType::Bash, bashrc));
        }

        let zshrc = home_p.join(".zshrc");
        if zshrc.exists() {
            configs.push((ShellType::Zsh, zshrc));
        }
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        let up = PathBuf::from(userprofile);
        let ps_profile = up
            .join("Documents")
            .join("WindowsPowerShell")
            .join("Microsoft.PowerShell_profile.ps1");
        if ps_profile.exists() {
            configs.push((ShellType::PowerShell, ps_profile));
        }
    }

    configs
}

/// Idempotently installs or updates the integration marker block inside a shell config file.
/// Returns true if the file was modified.
pub fn install_marker_block(path: &Path, shell: ShellType) -> io::Result<bool> {
    let new_block = shell.generate_block();

    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    if existing.contains(START_MARKER) && existing.contains(END_MARKER) {
        // Replace existing block cleanly
        let start_idx = existing.find(START_MARKER).unwrap();
        let end_idx = existing.find(END_MARKER).unwrap() + END_MARKER.len();

        let before = &existing[..start_idx];
        let after = &existing[end_idx..];
        let current_block = &existing[start_idx..end_idx];

        if current_block.trim() == new_block.trim() {
            return Ok(false); // Already identical, no change needed
        }

        let mut updated = String::with_capacity(existing.len() + new_block.len());
        updated.push_str(before);
        updated.push_str(&new_block);
        updated.push_str(after.trim_start_matches('\n'));

        fs::write(path, updated)?;
        Ok(true)
    } else {
        // Append marker block cleanly
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        if !existing.is_empty() && !existing.ends_with('\n') {
            writeln!(file)?;
        }
        write!(file, "{}", new_block)?;
        file.sync_all()?;
        Ok(true)
    }
}

/// Cleanly removes the integration marker block from a shell config file.
/// Returns true if the marker block was present and removed.
pub fn remove_marker_block(path: &Path) -> io::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)?;
    if let (Some(start_idx), Some(end_idx)) = (content.find(START_MARKER), content.find(END_MARKER))
    {
        let full_end = end_idx + END_MARKER.len();
        let before = &content[..start_idx];
        let after = &content[full_end..];

        let mut cleaned = String::with_capacity(content.len());
        cleaned.push_str(before.trim_end());
        cleaned.push('\n');
        cleaned.push_str(after.trim_start());

        fs::write(path, cleaned)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Automatically configures shell integration for all detected user shells.
pub fn setup_all_shells() -> io::Result<()> {
    let targets = detect_shell_configs();
    if targets.is_empty() {
        println!("No standard shell config files found automatically.");
        println!("Add the following function to your shell config (~/.bashrc, ~/.zshrc):\n");
        println!("{}", ShellType::Bash.generate_block());
        return Ok(());
    }

    for (shell, path) in targets {
        let updated = install_marker_block(&path, shell)?;
        if updated {
            println!(
                "✓ Installed Galerne integration in {} ({})",
                shell.name(),
                path.display()
            );
        } else {
            println!(
                "✓ Galerne integration already active in {} ({})",
                shell.name(),
                path.display()
            );
        }
    }

    println!("\nGalerne installed successfully.");
    println!("Command:            galerne");
    println!("Automatic game:     enabled");
    println!("Background daemon:  none");
    println!("Network required:   no\n");
    println!("Open a fresh terminal or run `source <config>` to activate.");
    println!("Run `galerne doctor` to verify your installation anytime.");
    Ok(())
}

/// Safely removes integration from all detected shells.
pub fn uninstall_all_shells(purge_saves: bool) -> io::Result<()> {
    let targets = detect_shell_configs();
    for (shell, path) in targets {
        if remove_marker_block(&path)? {
            println!(
                "✓ Removed Galerne integration from {} ({})",
                shell.name(),
                path.display()
            );
        }
    }

    let save_manager = crate::storage::manager::SaveManager::default_paths();
    let data_dir = save_manager.data_dir();

    if purge_saves {
        if data_dir.exists() {
            fs::remove_dir_all(data_dir)?;
            println!(
                "✓ Removed save directory and career data ({})",
                data_dir.display()
            );
        }
    } else {
        println!(
            "ℹ Preserved your save state and progression at {}",
            data_dir.display()
        );
        println!(
            "  (To delete career progression manually: rm -rf \"{}\")",
            data_dir.display()
        );
    }

    println!("\nGalerne integration has been cleanly uninstalled.");
    Ok(())
}
