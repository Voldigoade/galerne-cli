use std::env;
use std::io::{self, Write};
use std::process;

use galerne::interceptor::execute_wrapped_task;
use galerne::shell::{run_doctor, setup_all_shells, uninstall_all_shells};
use galerne::storage::manager::SaveManager;
use galerne::storage::stats::format_career_stats;
use galerne::ui::cli::{parse_cli_action, print_help, CliAction};
use galerne::ui::dashboard::{run_dashboard, run_direct_play};

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = parse_cli_action(&args);

    let save_manager = SaveManager::default_paths();

    match action {
        CliAction::Dashboard => {
            if let Err(e) = run_dashboard() {
                eprintln!("Error running Galerne dashboard: {}", e);
                process::exit(1);
            }
        }
        CliAction::Play => {
            if let Err(e) = run_direct_play() {
                eprintln!("Error launching Galerne combat session: {}", e);
                process::exit(1);
            }
        }
        CliAction::Stats => {
            let state = save_manager.load().unwrap_or_default();
            println!("{}", format_career_stats(&state));
        }
        CliAction::Doctor => {
            if let Err(e) = run_doctor() {
                eprintln!("Error executing diagnostics: {}", e);
                process::exit(1);
            }
        }
        CliAction::Saves => {
            println!("Galerne Persistence & Archives\n");
            let save_path = save_manager.primary_save_path();
            println!("Active Save:     {}", save_path.display());
            let backup_path = save_manager.backup_save_path();
            println!("Backup Save:     {}", backup_path.display());

            match save_manager.list_archives() {
                Ok(archives) if !archives.is_empty() => {
                    println!("\nArchived Snapshots ({} available):", archives.len());
                    for (name, size) in archives {
                        println!("  - {:<28} ({} bytes)", name, size);
                    }
                    println!("\nTo restore an archive: galerne restore <archive_name>");
                }
                _ => {
                    println!("\nNo historical archives found.");
                }
            }
        }
        CliAction::Reset { force } => {
            if !force {
                print!("Are you sure you want to archive and reset career progression? [y/N]: ");
                let _ = io::stdout().flush();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err()
                    || !matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
                {
                    println!("Reset cancelled. Progression unchanged.");
                    return;
                }
            }

            println!("Archiving existing progression before reset...");
            match save_manager.archive_and_reset() {
                Ok(archive_path) => {
                    println!(
                        "✓ Previous save successfully archived to: {}",
                        archive_path.display()
                    );
                    println!("✓ Clean progression initialized. Your history is preserved in the archive.");
                }
                Err(e) => {
                    eprintln!("Failed to archive and reset: {}", e);
                    process::exit(1);
                }
            }
        }
        CliAction::Restore(archive_name) => {
            let clean_name = if let Some(filename) = std::path::Path::new(&archive_name).file_name()
            {
                filename.to_string_lossy().to_string()
            } else {
                archive_name.clone()
            };

            match save_manager.restore_archive(&clean_name) {
                Ok(_) => {
                    println!(
                        "✓ Successfully restored career progression from: {}",
                        clean_name
                    );
                }
                Err(e) => {
                    eprintln!("Failed to restore archive '{}': {}", clean_name, e);
                    process::exit(1);
                }
            }
        }
        CliAction::Setup => {
            if let Err(e) = setup_all_shells() {
                eprintln!("Failed to configure shell integration: {}", e);
                process::exit(1);
            }
        }
        CliAction::Enable => {
            let mut state = save_manager.load().unwrap_or_default();
            state.settings.auto_activate = true;
            if let Err(e) = save_manager.save(&state) {
                eprintln!("Failed to update settings: {}", e);
                process::exit(1);
            }
            println!("✓ Automatic wait-time activation enabled.");
        }
        CliAction::Disable => {
            let mut state = save_manager.load().unwrap_or_default();
            state.settings.auto_activate = false;
            if let Err(e) = save_manager.save(&state) {
                eprintln!("Failed to update settings: {}", e);
                process::exit(1);
            }
            println!("✓ Automatic wait-time activation disabled (progression preserved).");
        }
        CliAction::Uninstall => {
            if let Err(e) = uninstall_all_shells(false) {
                eprintln!("Failed to complete uninstallation: {}", e);
                process::exit(1);
            }
        }
        CliAction::Update { check_only } => {
            if let Err(e) = galerne::updater::run_update(check_only) {
                eprintln!("\nUpdate failed: {}", e);
                process::exit(1);
            }
        }
        CliAction::Wrap(cmd) => match execute_wrapped_task(&cmd) {
            Ok(exit_code) => {
                process::exit(exit_code);
            }
            Err(e) => {
                eprintln!("Galerne execution error: {}", e);
                process::exit(1);
            }
        },
        CliAction::Help => {
            print_help();
        }
        CliAction::Version => {
            println!("galerne {}", galerne::VERSION);
        }
    }
}
