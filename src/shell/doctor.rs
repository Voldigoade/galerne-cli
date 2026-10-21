//! Checks for terminal, storage, and shell integration health.

use std::env;
use std::fs;
use std::io::{self, IsTerminal};

use crossterm::terminal;

use crate::shell::installer::{detect_shell_configs, START_MARKER};
use crate::storage::manager::SaveManager;
use crate::ui::i18n::{format_message, t, Locale};

#[derive(Clone, Debug)]
pub struct DiagnosticItem {
    pub label_key: &'static str,
    pub status_key: &'static str,
    pub status_args: Vec<(&'static str, String)>,
    pub is_ok: bool,
}

impl DiagnosticItem {
    fn new(
        label_key: &'static str,
        status_key: &'static str,
        status_args: Vec<(&'static str, String)>,
        is_ok: bool,
    ) -> Self {
        Self {
            label_key,
            status_key,
            status_args,
            is_ok,
        }
    }

    pub fn label(&self, locale: Locale) -> &'static str {
        t(self.label_key, locale)
    }

    pub fn status(&self, locale: Locale) -> String {
        format_message(self.status_key, locale, &self.status_args)
    }
}

pub fn collect_diagnostics() -> Vec<DiagnosticItem> {
    let mut items = Vec::new();
    items.push(DiagnosticItem::new(
        "target_platform",
        "doctor_platform",
        vec![
            ("os", env::consts::OS.to_owned()),
            ("arch", env::consts::ARCH.to_owned()),
        ],
        true,
    ));

    let is_tty = io::stdout().is_terminal();
    items.push(DiagnosticItem::new(
        "standard_output",
        if is_tty {
            "doctor_tty_interactive"
        } else {
            "doctor_tty_pipe"
        },
        Vec::new(),
        is_tty,
    ));

    match terminal::size() {
        Ok((width, height)) => {
            items.push(DiagnosticItem::new(
                "terminal_size",
                "doctor_terminal_dimensions",
                vec![("width", width.to_string()), ("height", height.to_string())],
                width >= 60 && height >= 20,
            ));
        }
        Err(_) => items.push(DiagnosticItem::new(
            "terminal_size",
            "doctor_terminal_unknown",
            Vec::new(),
            false,
        )),
    }

    let no_color = env::var("NO_COLOR").is_ok();
    items.push(DiagnosticItem::new(
        "doctor_color_setting",
        if no_color {
            "doctor_color_disabled"
        } else {
            "doctor_color_ansi"
        },
        Vec::new(),
        !no_color,
    ));

    let save_manager = SaveManager::default_paths();
    let data_dir = save_manager.data_dir();
    let data_dir_status = if data_dir.exists() {
        match fs::metadata(data_dir) {
            Ok(metadata) if !metadata.permissions().readonly() => (
                "doctor_storage_writable",
                format!("{}", data_dir.display()),
                true,
            ),
            _ => (
                "doctor_storage_denied",
                format!("{}", data_dir.display()),
                false,
            ),
        }
    } else {
        (
            "doctor_storage_uncreated",
            format!("{}", data_dir.display()),
            true,
        )
    };
    items.push(DiagnosticItem::new(
        "data_storage",
        data_dir_status.0,
        vec![("path", data_dir_status.1)],
        data_dir_status.2,
    ));

    let primary_save = save_manager.primary_save_path();
    if primary_save.exists() {
        match save_manager.load() {
            Ok(state) => items.push(DiagnosticItem::new(
                "save_integrity",
                "doctor_save_valid",
                vec![("shards", state.currency_shards.to_string())],
                true,
            )),
            Err(error) => items.push(DiagnosticItem::new(
                "save_integrity",
                "doctor_save_error",
                vec![("error", error.to_string())],
                false,
            )),
        }
    } else {
        items.push(DiagnosticItem::new(
            "save_integrity",
            "doctor_save_missing",
            Vec::new(),
            true,
        ));
    }

    let backup_save = save_manager.backup_save_path();
    items.push(DiagnosticItem::new(
        "backup_label",
        if backup_save.exists() {
            "doctor_backup_present"
        } else {
            "doctor_backup_pending"
        },
        Vec::new(),
        true,
    ));

    let detected_shells = detect_shell_configs();
    let integrated_count = detected_shells
        .iter()
        .filter(|(_, path)| {
            fs::read_to_string(path)
                .map(|content| content.contains(START_MARKER))
                .unwrap_or(false)
        })
        .count();
    items.push(DiagnosticItem::new(
        "shell_hook",
        if integrated_count > 0 {
            "doctor_shell_active"
        } else {
            "doctor_shell_missing"
        },
        vec![("count", integrated_count.to_string())],
        integrated_count > 0,
    ));

    items.push(DiagnosticItem::new(
        "resident_daemons",
        "doctor_background_none",
        Vec::new(),
        true,
    ));
    items.push(DiagnosticItem::new(
        "network_policy",
        "doctor_network_update_only",
        Vec::new(),
        true,
    ));

    items
}

pub fn run_doctor() -> io::Result<()> {
    let locale = Locale::detect();
    println!(
        "Galerne v{} {}\n",
        crate::VERSION,
        t("doctor_title", locale)
    );

    let current_exe = env::current_exe().unwrap_or_default();
    println!(
        "[✓] {}: {} (v{})",
        t("executable", locale),
        current_exe.display(),
        crate::VERSION
    );

    for item in collect_diagnostics() {
        let symbol = if item.is_ok { "✓" } else { "!" };
        println!(
            "[{}] {:<18} {}",
            symbol,
            format!("{}:", item.label(locale)),
            item.status(locale)
        );
    }

    println!("\n{}", t("doctor_complete", locale));
    Ok(())
}
