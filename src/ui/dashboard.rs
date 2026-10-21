//! Interactive terminal dashboard and standalone game entrypoints.

use std::fs;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::game::rng::FastRng;
use crate::game::runner::action::PlayerInput;
use crate::game::runner::RunnerState;
use crate::shell::doctor::collect_diagnostics;
use crate::storage::manager::SaveManager;
use crate::storage::stats::format_career_stats;
use crate::storage::workshop::{
    apply_workshop_to_runner, purchase_workshop_upgrade, WorkshopUpgradeType,
};
use crate::terminal::buffer::{Color, Style};
use crate::terminal::engine::Engine;
use crate::terminal::guard::TerminalGuard;
use crate::ui::i18n::{format_message, t, Locale};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DashboardTab {
    Play,
    Refuge,
    Inventory,
    Workshop,
    Stats,
    Saves,
    Doctor,
    Settings,
    Help,
    Exit,
}

impl DashboardTab {
    const ALL: [DashboardTab; 10] = [
        DashboardTab::Play,
        DashboardTab::Refuge,
        DashboardTab::Inventory,
        DashboardTab::Workshop,
        DashboardTab::Stats,
        DashboardTab::Saves,
        DashboardTab::Doctor,
        DashboardTab::Settings,
        DashboardTab::Help,
        DashboardTab::Exit,
    ];

    fn label(&self, locale: Locale) -> &'static str {
        match self {
            DashboardTab::Play => t("menu_play", locale),
            DashboardTab::Refuge => t("menu_refuge", locale),
            DashboardTab::Inventory => t("menu_inventory", locale),
            DashboardTab::Workshop => t("menu_workshop", locale),
            DashboardTab::Stats => t("menu_stats", locale),
            DashboardTab::Saves => t("menu_saves", locale),
            DashboardTab::Doctor => t("menu_doctor", locale),
            DashboardTab::Settings => t("menu_settings", locale),
            DashboardTab::Help => t("menu_help", locale),
            DashboardTab::Exit => t("menu_exit", locale),
        }
    }
}

/// Renders the Refuge panel content onto a buffer.
pub fn render_refuge_panel(
    buf: &mut crate::terminal::buffer::Buffer,
    save_state: &crate::storage::schema::SaveState,
    px: u16,
    py: u16,
    locale: Locale,
) {
    let bg = Color::Reset;
    let is_restored = save_state.world_progress.relay_restored;
    let header = if is_restored {
        t("refuge_restored_status", locale)
    } else {
        t("refuge_dormant_status", locale)
    };
    let status_color = if is_restored {
        Color::Cyan
    } else {
        Color::DarkYellow
    };
    buf.draw_str_styled(px, py, header, Style::new(status_color, bg).bold());

    let gantry_ascii = if is_restored {
        &[
            "       .---------------------------------------.       ",
            "      /  [*]   /=======\\     [*]  \\      ",
            "     /===========================================\\     ",
            "     |  __      [====]       ____      ,(^) |     ",
            "     | |##|    .-----------.     |====|    <{#}>~|     ",
            "     | |##|    | * * * * * |     |====|     / \\  |     ",
            "    ===============================================    ",
            "       |        [*]   ─────   [*]   ─────         |    ",
        ]
    } else {
        &[
            "       .---------------------------------------.       ",
            "      /  [ ]   /=======\\   [ ]  \\      ",
            "     /===========================================\\     ",
            "     |  __      [====]       ____       [ ] |     ",
            "     | |  |    .-----------.     |####|     .---.|     ",
            "     | |__|    | _ _ _ _ _ |     |####|     |   ||     ",
            "    ===============================================    ",
            "       |        [ ]   ─────   [ ]   ─────         |    ",
        ]
    };

    for (idx, line) in gantry_ascii.iter().enumerate() {
        let style = if is_restored
            && (line.contains("[*]") || line.contains("##") || line.contains("===="))
        {
            Style::new(Color::Cyan, bg).bold()
        } else {
            Style::new(Color::White, bg)
        };
        buf.draw_str_styled(px, py + 2 + idx as u16, line, style);
    }

    let y_info = py + 11;
    let kael_status = if save_state.companion.kael_at_refuge {
        t("refuge_kael_present", locale)
    } else if save_state.companion.met_kael {
        t("refuge_companion_met", locale)
    } else {
        t("refuge_kael_absent", locale)
    };
    buf.draw_str(
        px,
        y_info,
        kael_status,
        if save_state.companion.kael_at_refuge {
            Color::Yellow
        } else if save_state.companion.met_kael {
            Color::Cyan
        } else {
            Color::DarkGrey
        },
        bg,
    );

    let workbench_status = t("refuge_workbench_status", locale);
    buf.draw_str(px, y_info + 1, workbench_status, Color::White, bg);

    let progress_summary = format_message(
        "refuge_progress",
        locale,
        &[
            (
                "unlocked",
                save_state
                    .world_progress
                    .checkpoints_unlocked
                    .len()
                    .min(5)
                    .to_string(),
            ),
            (
                "status",
                t(
                    if is_restored {
                        "line_restored"
                    } else {
                        "line_dormant"
                    },
                    locale,
                )
                .to_owned(),
            ),
        ],
    );
    buf.draw_str_styled(
        px,
        y_info + 3,
        &progress_summary,
        Style::new(Color::Cyan, bg).bold(),
    );

    if is_restored && save_state.companion.kael_at_refuge {
        buf.draw_str_styled(
            px,
            y_info + 5,
            t("kael_refuge_dialogue", locale),
            Style::new(Color::Yellow, bg).bold(),
        );
    }
}

/// Runs the interactive terminal home dashboard.
pub fn run_dashboard() -> io::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut engine = Engine::new()?;

    let save_manager = SaveManager::default_paths();
    let mut save_state = save_manager.load_or_default()?;

    let mut current_idx = 0;
    let mut focus_panel = false;
    let mut workshop_idx = 0;
    let mut settings_idx = 0;
    let mut saves_idx = 0;
    let mut status_banner: Option<(String, Instant)> = None;

    loop {
        let locale = save_state.settings.locale.unwrap_or_else(Locale::detect);

        if engine.is_too_small() {
            engine.render_too_small_notice(locale);
            engine.flush()?;
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let (w, h) = (engine.width(), engine.height());
        let buf = engine.buffer_mut();
        buf.clear();

        let box_fg = if focus_panel {
            Color::Yellow
        } else {
            Color::Cyan
        };
        let bg = Color::Reset;

        buf.set_char(0, 0, '╭', box_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, 0, '─', box_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), 0, '╮', box_fg, bg);

        for y in 1..h.saturating_sub(1) {
            buf.set_char(0, y, '│', box_fg, bg);
            buf.set_char(w.saturating_sub(1), y, '│', box_fg, bg);
        }

        buf.set_char(0, h.saturating_sub(1), '╰', box_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, h.saturating_sub(1), '─', box_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), h.saturating_sub(1), '╯', box_fg, bg);

        let header = format!("GALERNE v{}", crate::VERSION);
        let hx = (w.saturating_sub(header.len() as u16)) / 2;
        buf.draw_str_styled(hx, 0, &header, Style::new(Color::Yellow, bg).bold());

        buf.set_char(0, 2, '├', box_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, 2, '─', box_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), 2, '┤', box_fg, bg);

        let split_x = 24u16.min(w / 3);
        buf.set_char(split_x, 2, '┬', box_fg, bg);
        for y in 3..h.saturating_sub(3) {
            buf.set_char(split_x, y, '│', box_fg, bg);
        }
        buf.set_char(split_x, h.saturating_sub(3), '┴', box_fg, bg);

        buf.set_char(0, h.saturating_sub(3), '├', box_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, h.saturating_sub(3), '─', box_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), h.saturating_sub(3), '┤', box_fg, bg);

        for (i, tab) in DashboardTab::ALL.iter().enumerate() {
            let row_y = 4 + (i as u16 * 2);
            let is_sel = i == current_idx;
            let prefix = if is_sel {
                if focus_panel {
                    "· "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };
            let line = format!("{}{:<14}", prefix, tab.label(locale));

            let style = if is_sel {
                if focus_panel {
                    Style::new(Color::Cyan, bg).bold()
                } else {
                    Style::new(Color::Black, Color::Cyan).bold()
                }
            } else {
                Style::new(Color::White, bg)
            };
            buf.draw_str_styled(3, row_y, &line, style);
        }

        let current_tab = DashboardTab::ALL[current_idx];
        let px = split_x + 3;
        let py = 4;

        match current_tab {
            DashboardTab::Play => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("deploy_operation", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );
                if let Some(ref run) = save_state.active_run {
                    buf.draw_str(px, py + 2, t("active_run_desc", locale), Color::White, bg);
                    let info = format_message(
                        "active_run_summary",
                        locale,
                        &[
                            ("distance", format!("{:.0}", run.distance)),
                            ("speed", format!("{:.0}", run.speed)),
                            ("score", run.score.to_string()),
                            ("shards", run.shards_collected.to_string()),
                            ("shields", run.player_shields.to_string()),
                        ],
                    );
                    buf.draw_str_styled(px, py + 3, &info, Style::new(Color::Cyan, bg).bold());
                    buf.draw_str(
                        px,
                        py + 5,
                        t("press_enter_resume", locale),
                        Color::Green,
                        bg,
                    );
                } else {
                    buf.draw_str(px, py + 2, t("system_idle", locale), Color::White, bg);
                    buf.draw_str(
                        px,
                        py + 3,
                        t("play_controls_hint", locale),
                        Color::DarkGrey,
                        bg,
                    );
                    let shards_str = format_message(
                        "shards_available",
                        locale,
                        &[("count", save_state.currency_shards.to_string())],
                    );
                    buf.draw_str_styled(
                        px,
                        py + 5,
                        &shards_str,
                        Style::new(Color::Cyan, bg).bold(),
                    );
                }
            }
            DashboardTab::Refuge => {
                render_refuge_panel(buf, &save_state, px, py, locale);
            }
            DashboardTab::Inventory => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("inventory_title", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );

                let inv = &save_state.inventory;
                let shards_count = inv
                    .count(crate::storage::ItemKind::GaleShard)
                    .max(save_state.currency_shards);
                let core_count = inv.count(crate::storage::ItemKind::ResonanceCore);
                let filament_count = inv.count(crate::storage::ItemKind::InductionFilament);
                let compass_count = inv.count(crate::storage::ItemKind::AncientCompass);

                let items = [
                    (
                        t("item_gale_shard_name", locale),
                        shards_count,
                        t("item_gale_shard_desc", locale),
                        Color::Yellow,
                        "[*]",
                    ),
                    (
                        t("item_resonance_core_name", locale),
                        core_count,
                        t("item_resonance_core_desc", locale),
                        Color::Cyan,
                        "[<#>]",
                    ),
                    (
                        t("item_induction_filament_name", locale),
                        filament_count,
                        t("item_induction_filament_desc", locale),
                        Color::White,
                        "[~|~]",
                    ),
                    (
                        t("item_ancient_compass_name", locale),
                        compass_count,
                        t("item_ancient_compass_desc", locale),
                        Color::Magenta,
                        "[(*)]",
                    ),
                ];

                for (idx, (name, count, desc, color, glyph)) in items.iter().enumerate() {
                    let y = py + 2 + (idx as u16 * 3);
                    let title = format!("{} {}  x{:02}", glyph, name, count);
                    buf.draw_str_styled(px, y, &title, Style::new(*color, bg).bold());
                    buf.draw_str(px + 4, y + 1, desc, Color::DarkGrey, bg);
                }
            }
            DashboardTab::Workshop => {
                let w_header = format_message(
                    "workshop_header",
                    locale,
                    &[
                        ("title", t("workshop_title", locale).to_owned()),
                        ("count", save_state.currency_shards.to_string()),
                    ],
                );
                buf.draw_str_styled(px, py, &w_header, Style::new(Color::Yellow, bg).bold());

                for (idx, up_type) in WorkshopUpgradeType::ALL.iter().enumerate() {
                    let y = py + 2 + (idx as u16 * 2);
                    let lvl = up_type.get_level(&save_state.workshop);
                    let is_sel = idx == workshop_idx && focus_panel;

                    let cost_str = if lvl >= up_type.max_level() {
                        t("max_level", locale).to_string()
                    } else {
                        format_message(
                            "shard_cost",
                            locale,
                            &[("count", up_type.cost(lvl).to_string())],
                        )
                    };

                    let rank = format_message(
                        "rank_value",
                        locale,
                        &[("rank", lvl.to_string()), ("max", "5".to_owned())],
                    );
                    let line = format!(
                        "{} {:<20} {}  [{}]",
                        if is_sel { "▶" } else { " " },
                        t(up_type.title_key(), locale),
                        rank,
                        cost_str
                    );

                    let style = if is_sel {
                        Style::new(Color::Black, Color::Yellow).bold()
                    } else if idx == workshop_idx {
                        Style::new(Color::Yellow, bg)
                    } else {
                        Style::new(Color::White, bg)
                    };
                    buf.draw_str_styled(px, y, &line, style);
                }
                buf.draw_str(px, py + 13, t("workshop_help", locale), Color::DarkGrey, bg);
            }
            DashboardTab::Stats => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("stats_heading", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );
                let stats_text = format_career_stats(&save_state, locale);
                for (line_idx, line) in stats_text.lines().enumerate() {
                    buf.draw_str(px, py + 2 + line_idx as u16, line, Color::White, bg);
                }
            }
            DashboardTab::Saves => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("saves_heading", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );
                let save_path = save_manager.primary_save_path();
                let p_str = format_message(
                    "path_label",
                    locale,
                    &[("path", save_path.display().to_string())],
                );
                buf.draw_str(px, py + 2, &p_str, Color::DarkGrey, bg);

                let archives = save_manager.list_archives().unwrap_or_default();

                if save_path.exists() {
                    buf.draw_str(px, py + 4, t("save_verified", locale), Color::Green, bg);
                } else {
                    buf.draw_str(px, py + 4, t("save_missing", locale), Color::DarkYellow, bg);
                }

                if archives.is_empty() {
                    buf.draw_str(px, py + 6, t("archives_empty", locale), Color::DarkGrey, bg);
                } else {
                    buf.draw_str_styled(
                        px,
                        py + 6,
                        &format_message(
                            "archives_count",
                            locale,
                            &[("count", archives.len().to_string())],
                        ),
                        Style::new(Color::Cyan, bg),
                    );

                    for (idx, (name, size)) in archives.iter().take(5).enumerate() {
                        let y = py + 8 + (idx as u16);
                        let is_sel = idx == saves_idx && focus_panel;
                        let line = format!(
                            "{} {:<28} ({} bytes)",
                            if is_sel { "▶" } else { " " },
                            name,
                            size
                        );
                        let style = if is_sel {
                            Style::new(Color::Black, Color::Cyan).bold()
                        } else {
                            Style::new(Color::White, bg)
                        };
                        buf.draw_str_styled(px, y, &line, style);
                    }

                    buf.draw_str(
                        px,
                        py + 13,
                        t("archives_controls", locale),
                        Color::DarkGrey,
                        bg,
                    );
                }
            }
            DashboardTab::Doctor => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("doctor_live_heading", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );
                let diagnostics = collect_diagnostics();
                for (idx, diag) in diagnostics.iter().enumerate() {
                    let y = py + 2 + idx as u16;
                    let symbol = if diag.is_ok { "✓" } else { "!" };
                    let color = if diag.is_ok {
                        Color::Green
                    } else {
                        Color::Yellow
                    };
                    let text = format!(
                        "[{}] {:<18} {}",
                        symbol,
                        format!("{}:", diag.label(locale)),
                        diag.status(locale)
                    );
                    buf.draw_str_styled(px, y, &text, Style::new(color, bg));
                }
            }
            DashboardTab::Settings => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("settings_title", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );
                let s = &save_state.settings;

                let lang_val = match s.locale {
                    None => t("language_auto", locale),
                    Some(value) => value.name(),
                };

                let settings_rows = [
                    (
                        t("color_effects", locale),
                        if s.color_enabled {
                            t("setting_enabled", locale)
                        } else {
                            t("setting_disabled", locale)
                        },
                        t("setting_toggle_hint", locale),
                    ),
                    (
                        t("animations", locale),
                        if s.animations_enabled {
                            t("setting_enabled", locale)
                        } else {
                            t("setting_disabled", locale)
                        },
                        t("setting_toggle_hint", locale),
                    ),
                    (
                        t("auto_activate", locale),
                        if s.auto_activate {
                            t("setting_enabled", locale)
                        } else {
                            t("setting_disabled", locale)
                        },
                        t("setting_toggle_hint", locale),
                    ),
                    (
                        t("anti_flicker_delay", locale),
                        &format!("{} ms", s.anti_flicker_delay_ms),
                        t("setting_delay_hint", locale),
                    ),
                    (
                        t("language", locale),
                        lang_val,
                        t("setting_language_hint", locale),
                    ),
                ];

                for (idx, (label, val, hint)) in settings_rows.iter().enumerate() {
                    let y = py + 2 + (idx as u16 * 2);
                    let is_sel = idx == settings_idx && focus_panel;

                    let line = format!(
                        "{} {:<24} {:<14} {}",
                        if is_sel { "▶" } else { " " },
                        format!("{}:", label),
                        val,
                        if is_sel { hint } else { "" }
                    );

                    let style = if is_sel {
                        Style::new(Color::Black, Color::Yellow).bold()
                    } else if idx == settings_idx {
                        Style::new(Color::Yellow, bg)
                    } else {
                        Style::new(Color::White, bg)
                    };
                    buf.draw_str_styled(px, y, &line, style);
                }

                buf.draw_str(px, py + 13, t("footer_panel", locale), Color::DarkGrey, bg);
            }
            DashboardTab::Help => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("help_heading", locale),
                    Style::new(Color::Yellow, bg).bold(),
                );
                buf.draw_str(px, py + 2, t("help_jump", locale), Color::Cyan, bg);
                buf.draw_str(px, py + 3, t("help_slide", locale), Color::Cyan, bg);
                buf.draw_str(px, py + 4, t("help_dash", locale), Color::Cyan, bg);
                buf.draw_str(px, py + 5, t("help_interact", locale), Color::Cyan, bg);
                buf.draw_str(px, py + 6, t("help_wind", locale), Color::Cyan, bg);
                buf.draw_str(px, py + 7, t("help_leave", locale), Color::Cyan, bg);
                buf.draw_str(
                    px,
                    py + 9,
                    t("help_tasks_heading", locale),
                    Color::White,
                    bg,
                );
                buf.draw_str(
                    px,
                    py + 10,
                    t("help_activation", locale),
                    Color::DarkGrey,
                    bg,
                );
                buf.draw_str(px, py + 11, t("help_continue", locale), Color::DarkGrey, bg);
            }
            DashboardTab::Exit => {
                buf.draw_str_styled(
                    px,
                    py,
                    t("exit_heading", locale),
                    Style::new(Color::Red, bg).bold(),
                );
                buf.draw_str(px, py + 2, t("exit_prompt", locale), Color::White, bg);
            }
        }

        if let Some((ref msg, time)) = status_banner {
            if time.elapsed() < Duration::from_secs(4) {
                buf.draw_str_styled(
                    px,
                    h.saturating_sub(4),
                    msg,
                    Style::new(Color::Green, bg).bold(),
                );
            }
        }

        let footer = if focus_panel {
            t("footer_panel", locale)
        } else {
            t("footer_menu", locale)
        };
        buf.draw_str_styled(
            3,
            h.saturating_sub(2),
            footer,
            Style::new(Color::DarkGrey, bg),
        );

        engine.flush()?;

        if let Some(ev) = Engine::poll_event(Duration::from_millis(100))? {
            match ev {
                Event::Key(k) if k.code == KeyCode::Char('q') && !focus_panel => {
                    break;
                }
                Event::Key(k)
                    if k.modifiers.contains(KeyModifiers::CONTROL)
                        && k.code == KeyCode::Char('c') =>
                {
                    break;
                }
                Event::Key(k) => match k.code {
                    KeyCode::Tab => {
                        focus_panel = !focus_panel;
                    }
                    KeyCode::Esc => {
                        if focus_panel {
                            focus_panel = false;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Up => {
                        if !focus_panel {
                            current_idx = if current_idx == 0 {
                                DashboardTab::ALL.len() - 1
                            } else {
                                current_idx - 1
                            };
                        } else {
                            match current_tab {
                                DashboardTab::Workshop => {
                                    workshop_idx = if workshop_idx == 0 {
                                        WorkshopUpgradeType::ALL.len() - 1
                                    } else {
                                        workshop_idx - 1
                                    };
                                }
                                DashboardTab::Settings => {
                                    settings_idx = if settings_idx == 0 {
                                        4
                                    } else {
                                        settings_idx - 1
                                    };
                                }
                                DashboardTab::Saves => {
                                    let archives = save_manager.list_archives().unwrap_or_default();
                                    if !archives.is_empty() {
                                        saves_idx = if saves_idx == 0 {
                                            archives.len() - 1
                                        } else {
                                            saves_idx - 1
                                        };
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Down => {
                        if !focus_panel {
                            current_idx = (current_idx + 1) % DashboardTab::ALL.len();
                        } else {
                            match current_tab {
                                DashboardTab::Workshop => {
                                    workshop_idx =
                                        (workshop_idx + 1) % WorkshopUpgradeType::ALL.len();
                                }
                                DashboardTab::Settings => {
                                    settings_idx = (settings_idx + 1) % 5;
                                }
                                DashboardTab::Saves => {
                                    let archives = save_manager.list_archives().unwrap_or_default();
                                    if !archives.is_empty() {
                                        saves_idx = (saves_idx + 1) % archives.len();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Left => {
                        if focus_panel && current_tab == DashboardTab::Settings {
                            toggle_or_adjust_setting(&mut save_state, settings_idx, -1);
                            if let Err(e) = save_manager.save(&save_state) {
                                status_banner = Some((
                                    format_message(
                                        "save_error",
                                        locale,
                                        &[("error", e.to_string())],
                                    ),
                                    Instant::now(),
                                ));
                            }
                        } else {
                            focus_panel = false;
                        }
                    }
                    KeyCode::Right => {
                        if focus_panel && current_tab == DashboardTab::Settings {
                            toggle_or_adjust_setting(&mut save_state, settings_idx, 1);
                            if let Err(e) = save_manager.save(&save_state) {
                                status_banner = Some((
                                    format_message(
                                        "save_error",
                                        locale,
                                        &[("error", e.to_string())],
                                    ),
                                    Instant::now(),
                                ));
                            }
                        } else {
                            focus_panel = true;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if focus_panel && current_tab == DashboardTab::Settings {
                            toggle_or_adjust_setting(&mut save_state, settings_idx, 1);
                            if let Err(e) = save_manager.save(&save_state) {
                                status_banner = Some((
                                    format_message(
                                        "save_error",
                                        locale,
                                        &[("error", e.to_string())],
                                    ),
                                    Instant::now(),
                                ));
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if focus_panel && current_tab == DashboardTab::Saves {
                            let archives = save_manager.list_archives().unwrap_or_default();
                            if saves_idx < archives.len() {
                                let (name, _) = &archives[saves_idx];
                                let path = save_manager.data_dir().join(name);
                                if fs::remove_file(path).is_ok() {
                                    status_banner = Some((
                                        format_message(
                                            "archive_deleted",
                                            locale,
                                            &[("name", name.clone())],
                                        ),
                                        Instant::now(),
                                    ));
                                    saves_idx = saves_idx.saturating_sub(1);
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if !focus_panel {
                            match current_tab {
                                DashboardTab::Play => {
                                    run_manual_game(&mut engine, &save_manager, &mut save_state)?;
                                }
                                DashboardTab::Exit => {
                                    break;
                                }
                                _ => {
                                    focus_panel = true;
                                }
                            }
                        } else {
                            match current_tab {
                                DashboardTab::Play => {
                                    run_manual_game(&mut engine, &save_manager, &mut save_state)?;
                                }
                                DashboardTab::Workshop => {
                                    let target = WorkshopUpgradeType::ALL[workshop_idx];
                                    if purchase_workshop_upgrade(&mut save_state, target) {
                                        if let Err(e) = save_manager.save(&save_state) {
                                            status_banner = Some((
                                                format_message(
                                                    "save_error",
                                                    locale,
                                                    &[("error", e.to_string())],
                                                ),
                                                Instant::now(),
                                            ));
                                        } else {
                                            status_banner = Some((
                                                format_message(
                                                    "upgrade_purchased",
                                                    locale,
                                                    &[(
                                                        "upgrade",
                                                        t(target.title_key(), locale).to_owned(),
                                                    )],
                                                ),
                                                Instant::now(),
                                            ));
                                        }
                                    } else {
                                        status_banner = Some((
                                            t("shards_insufficient", locale).to_owned(),
                                            Instant::now(),
                                        ));
                                    }
                                }
                                DashboardTab::Settings => {
                                    toggle_or_adjust_setting(&mut save_state, settings_idx, 1);
                                    if let Err(e) = save_manager.save(&save_state) {
                                        status_banner = Some((
                                            format_message(
                                                "save_error",
                                                locale,
                                                &[("error", e.to_string())],
                                            ),
                                            Instant::now(),
                                        ));
                                    }
                                }
                                DashboardTab::Saves => {
                                    let archives = save_manager.list_archives().unwrap_or_default();
                                    if saves_idx < archives.len() {
                                        let (name, _) = &archives[saves_idx];
                                        match save_manager.restore_archive(name) {
                                            Ok(_) => {
                                                save_state = save_manager.load_or_default()?;
                                                status_banner = Some((
                                                    format_message(
                                                        "archive_restored",
                                                        locale,
                                                        &[("name", name.clone())],
                                                    ),
                                                    Instant::now(),
                                                ));
                                            }
                                            Err(e) => {
                                                status_banner = Some((
                                                    format_message(
                                                        "archive_restore_failed",
                                                        locale,
                                                        &[("error", e.to_string())],
                                                    ),
                                                    Instant::now(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                DashboardTab::Exit => {
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    let _ = engine.check_resize();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn toggle_or_adjust_setting(
    save_state: &mut crate::storage::schema::SaveState,
    idx: usize,
    delta: i32,
) {
    match idx {
        0 => {
            save_state.settings.color_enabled = !save_state.settings.color_enabled;
        }
        1 => {
            save_state.settings.animations_enabled = !save_state.settings.animations_enabled;
        }
        2 => {
            save_state.settings.auto_activate = !save_state.settings.auto_activate;
        }
        3 => {
            let thresholds = [1000u64, 1500, 2000, 2500, 3000, 4000, 5000, 8000];
            let current = save_state.settings.anti_flicker_delay_ms;
            let current_pos = thresholds.iter().position(|&t| t == current).unwrap_or(7);
            let new_pos = if delta > 0 {
                (current_pos + 1) % thresholds.len()
            } else if current_pos == 0 {
                thresholds.len() - 1
            } else {
                current_pos - 1
            };
            save_state.settings.anti_flicker_delay_ms = thresholds[new_pos];
        }
        4 => {
            let locales = [
                None,
                Some(Locale::En),
                Some(Locale::Fr),
                Some(Locale::Es),
                Some(Locale::ZhCn),
            ];
            let current = save_state.settings.locale;
            let current_pos = locales.iter().position(|&l| l == current).unwrap_or(0);
            let new_pos = if delta > 0 {
                (current_pos + 1) % locales.len()
            } else if current_pos == 0 {
                locales.len() - 1
            } else {
                current_pos - 1
            };
            save_state.settings.locale = locales[new_pos];
        }
        _ => {}
    }
}

/// Runs a standalone manual play session directly from the CLI.
pub fn run_direct_play() -> io::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut engine = Engine::new()?;
    let save_manager = SaveManager::default_paths();
    let mut save_state = save_manager.load_or_default()?;
    run_manual_game(&mut engine, &save_manager, &mut save_state)
}

/// Runs a standalone manual play session from the dashboard or direct CLI invocation.
fn sync_game_to_save_state(
    game: &RunnerState,
    save_state: &mut crate::storage::schema::SaveState,
    initial_shards: u32,
    initial_dist: f32,
) {
    let shards_gained = game.shards_collected.saturating_sub(initial_shards);
    save_state.currency_shards += shards_gained;
    if shards_gained > 0 {
        save_state
            .inventory
            .add(crate::storage::ItemKind::GaleShard, shards_gained);
    }
    save_state.total_distance_traveled += (game.distance - initial_dist).max(0.0);

    if game.relay_restored {
        save_state.world_progress.relay_restored = true;
        save_state.refuge.power_restored = true;
    }
    if game.companion_met {
        save_state.companion.met_kael = true;
    }
    if save_state.world_progress.relay_restored && save_state.companion.met_kael {
        save_state.companion.kael_at_refuge = true;
    }
    if game.secret_cache_found {
        save_state.world_progress.secret_cache_found = true;
    }
    for relic in &game.collected_relics {
        match relic {
            crate::game::runner::items::ItemKind::ResonanceCore => {
                save_state
                    .inventory
                    .add(crate::storage::ItemKind::ResonanceCore, 1);
            }
            crate::game::runner::items::ItemKind::InductionFilament => {
                save_state
                    .inventory
                    .add(crate::storage::ItemKind::InductionFilament, 1);
            }
            crate::game::runner::items::ItemKind::AncientCompass => {
                save_state
                    .inventory
                    .add(crate::storage::ItemKind::AncientCompass, 1);
            }
            _ => {}
        }
    }
    for cp_id in &game.unlocked_checkpoints {
        if !save_state
            .world_progress
            .checkpoints_unlocked
            .contains(cp_id)
        {
            save_state.world_progress.checkpoints_unlocked.push(*cp_id);
        }
    }
}

pub fn run_manual_game(
    engine: &mut Engine,
    save_manager: &SaveManager,
    save_state: &mut crate::storage::schema::SaveState,
) -> io::Result<()> {
    let locale = save_state.settings.locale.unwrap_or_else(Locale::detect);
    let mut game = RunnerState::new(
        engine.width() as usize,
        engine.height() as usize,
        rand_seed(),
    );
    game.set_locale(locale);
    apply_workshop_to_runner(&save_state.workshop, &mut game);

    if let Some(ref snapshot) = save_state.active_run {
        game.resume_snapshot(snapshot);
    } else {
        save_state.total_runs_started += 1;
    }

    let start = Instant::now();
    let initial_shards = game.shards_collected;
    let initial_dist = game.distance;
    let target_frame_duration = Duration::from_millis(33); // ~30 FPS
    let mut last_frame = Instant::now();
    let mut next_input = None;

    loop {
        while let Some(ev) = Engine::poll_event(Duration::from_millis(0))? {
            match ev {
                Event::Key(k) if k.code == KeyCode::Esc || k.code == KeyCode::Char('q') => {
                    save_state.total_play_time_secs += start.elapsed().as_secs();
                    sync_game_to_save_state(&game, save_state, initial_shards, initial_dist);
                    if game.is_game_over {
                        save_state.active_run = None;
                    } else {
                        save_state.active_run = Some(game.create_snapshot());
                    }
                    if let Err(e) = save_manager.save(save_state) {
                        eprintln!(
                            "[galerne] Warning: Failed to persist career progression: {}",
                            e
                        );
                    }
                    return Ok(());
                }
                Event::Key(k)
                    if k.modifiers.contains(KeyModifiers::CONTROL)
                        && k.code == KeyCode::Char('c') =>
                {
                    save_state.total_play_time_secs += start.elapsed().as_secs();
                    sync_game_to_save_state(&game, save_state, initial_shards, initial_dist);
                    if !game.is_game_over {
                        save_state.active_run = Some(game.create_snapshot());
                    }
                    if let Err(e) = save_manager.save(save_state) {
                        eprintln!(
                            "[galerne] Warning: Failed to persist career progression: {}",
                            e
                        );
                    }
                    return Ok(());
                }
                Event::Key(k) => match k.code {
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        next_input = Some(PlayerInput::Interact);
                    }
                    KeyCode::Char(' ') | KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        next_input = Some(PlayerInput::Jump);
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        next_input = Some(PlayerInput::Slide);
                    }
                    KeyCode::Right
                    | KeyCode::Char('x')
                    | KeyCode::Char('X')
                    | KeyCode::Char('d')
                    | KeyCode::Char('D') => {
                        next_input = Some(PlayerInput::Dash);
                    }
                    _ => {}
                },
                Event::Resize(_w, _h) => {
                    let _ = engine.check_resize();
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        game.update(dt, next_input);
        next_input = None;

        if game.is_game_over {
            save_state.total_play_time_secs += start.elapsed().as_secs();
            save_state.currency_shards += game.shards_collected.saturating_sub(initial_shards);
            save_state.total_distance_traveled += (game.distance - initial_dist).max(0.0);
            save_state.active_run = None;
            let _ = save_manager.save(save_state);
            return Ok(());
        }

        if engine.is_too_small() {
            engine.render_too_small_notice(locale);
        } else {
            engine.buffer_mut().clear();
            game.render(engine.buffer_mut(), locale);
        }
        engine.flush()?;

        let elapsed = last_frame.elapsed();
        if elapsed < target_frame_duration {
            thread::sleep(target_frame_duration - elapsed);
        }
    }
}

fn rand_seed() -> u64 {
    let mut rng = FastRng::new(0);
    rng.next_u64()
}
