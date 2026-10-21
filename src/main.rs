use std::env;
use std::io::{self, Write};
use std::process;

use galerne::interceptor::execute_wrapped_task;
use galerne::shell::{run_doctor, setup_all_shells, uninstall_all_shells};
use galerne::storage::manager::SaveManager;
use galerne::storage::stats::format_career_stats;
use galerne::ui::cli::{parse_cli_action, print_help, CliAction};
use galerne::ui::dashboard::{run_dashboard, run_direct_play};
use galerne::ui::i18n::{format_message, t, Locale};

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
                eprintln!("Error launching Galerne journey: {}", e);
                process::exit(1);
            }
        }
        CliAction::Stats => {
            let state = load_save_state(&save_manager);
            let locale = state.settings.locale.unwrap_or_else(Locale::detect);
            println!("{}", format_career_stats(&state, locale));
        }
        CliAction::Doctor => {
            if let Err(e) = run_doctor() {
                eprintln!("Error executing diagnostics: {}", e);
                process::exit(1);
            }
        }
        CliAction::Saves => {
            let locale = Locale::detect();
            println!("{}\n", t("saves_heading", locale));
            let save_path = save_manager.primary_save_path();
            println!(
                "{}",
                format_message(
                    "path_label",
                    locale,
                    &[("path", save_path.display().to_string())],
                )
            );
            let backup_path = save_manager.backup_save_path();
            println!(
                "{}",
                format_message(
                    "backup_path_label",
                    locale,
                    &[("path", backup_path.display().to_string())],
                )
            );

            match save_manager.list_archives() {
                Ok(archives) if !archives.is_empty() => {
                    println!(
                        "\n{}",
                        format_message(
                            "archives_count",
                            locale,
                            &[("count", archives.len().to_string())],
                        )
                    );
                    for (name, size) in archives {
                        println!(
                            "  - {}",
                            format_message(
                                "archive_entry",
                                locale,
                                &[("name", format!("{name:<28}")), ("size", size.to_string()),],
                            )
                        );
                    }
                    println!("\n{}", t("cli_restore_hint", locale));
                }
                Ok(_) => {
                    println!("\n{}", t("archives_empty", locale));
                }
                Err(error) => {
                    eprintln!(
                        "{}",
                        format_message(
                            "cli_archives_error",
                            locale,
                            &[("error", error.to_string())],
                        )
                    );
                    process::exit(1);
                }
            }
        }
        CliAction::Reset { force } => {
            let locale = Locale::detect();
            if !force {
                print!("{}", t("cli_reset_prompt", locale));
                let _ = io::stdout().flush();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() || !is_confirmation(&input, locale) {
                    println!("{}", t("cli_reset_cancelled", locale));
                    return;
                }
            }

            println!("{}", t("cli_reset_start", locale));
            match save_manager.archive_and_reset() {
                Ok(archive_path) => {
                    if archive_path.exists() {
                        println!(
                            "✓ {}",
                            format_message(
                                "cli_reset_archive_path",
                                locale,
                                &[("path", archive_path.display().to_string())],
                            )
                        );
                    }
                    println!("✓ {}", t("cli_reset_done", locale));
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        format_message("cli_reset_error", locale, &[("error", e.to_string())])
                    );
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

            let locale = Locale::detect();
            match save_manager.restore_archive(&clean_name) {
                Ok(_) => {
                    println!(
                        "✓ {}",
                        format_message("archive_restored", locale, &[("name", clean_name)])
                    );
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        format_message(
                            "archive_restore_failed",
                            locale,
                            &[("error", e.to_string())],
                        )
                    );
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
            let locale = Locale::detect();
            let mut state = load_save_state(&save_manager);
            state.settings.auto_activate = true;
            if let Err(e) = save_manager.save(&state) {
                eprintln!("Failed to update settings: {}", e);
                process::exit(1);
            }
            println!("✓ {}", t("cli_auto_enabled", locale));
        }
        CliAction::Disable => {
            let locale = Locale::detect();
            let mut state = load_save_state(&save_manager);
            state.settings.auto_activate = false;
            if let Err(e) = save_manager.save(&state) {
                eprintln!("Failed to update settings: {}", e);
                process::exit(1);
            }
            println!("✓ {}", t("cli_auto_disabled", locale));
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
            print_help(Locale::detect());
        }
        CliAction::Version => {
            println!("galerne {}", galerne::VERSION);
        }
    }
}

fn load_save_state(save_manager: &SaveManager) -> galerne::storage::SaveState {
    match save_manager.load_or_default() {
        Ok(state) => state,
        Err(error) => {
            eprintln!("Failed to load save: {}", error);
            process::exit(1);
        }
    }
}

fn is_confirmation(input: &str, locale: Locale) -> bool {
    let answer = input.trim().to_lowercase();
    let accepted = match locale {
        Locale::En => &["y", "yes"][..],
        Locale::Fr => &["o", "oui", "y", "yes"][..],
        Locale::Es => &["s", "si", "sí", "y", "yes"][..],
        Locale::ZhCn => &["是", "y", "yes"][..],
    };
    accepted.contains(&answer.as_str())
}
