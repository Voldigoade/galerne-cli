//! Interactive terminal control center dashboard and standalone game runner.

use std::fs;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::game::state::{FastRng, GameInput, GameState};
use crate::shell::doctor::collect_diagnostics;
use crate::storage::manager::SaveManager;
use crate::storage::stats::format_career_stats;
use crate::storage::workshop::{
    apply_workshop_to_game, purchase_workshop_upgrade, WorkshopUpgradeType,
};
use crate::terminal::buffer::{Color, Style};
use crate::terminal::engine::Engine;
use crate::terminal::guard::TerminalGuard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DashboardTab {
    Play,
    Workshop,
    Stats,
    Saves,
    Doctor,
    Settings,
    Help,
    Exit,
}

impl DashboardTab {
    const ALL: [DashboardTab; 8] = [
        DashboardTab::Play,
        DashboardTab::Workshop,
        DashboardTab::Stats,
        DashboardTab::Saves,
        DashboardTab::Doctor,
        DashboardTab::Settings,
        DashboardTab::Help,
        DashboardTab::Exit,
    ];

    fn label(&self) -> &'static str {
        match self {
            DashboardTab::Play => "PLAY",
            DashboardTab::Workshop => "WORKSHOP",
            DashboardTab::Stats => "STATS",
            DashboardTab::Saves => "SAVES",
            DashboardTab::Doctor => "DOCTOR",
            DashboardTab::Settings => "SETTINGS",
            DashboardTab::Help => "HELP",
            DashboardTab::Exit => "EXIT",
        }
    }
}

/// Runs the interactive terminal home dashboard.
pub fn run_dashboard() -> io::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut engine = Engine::new()?;

    let save_manager = SaveManager::default_paths();
    let mut save_state = save_manager.load().unwrap_or_default();

    let mut current_idx = 0;
    let mut focus_panel = false;
    let mut workshop_idx = 0;
    let mut settings_idx = 0;
    let mut saves_idx = 0;
    let mut status_banner: Option<(String, Instant)> = None;

    loop {
        if engine.is_too_small() {
            engine.render_too_small_notice();
            engine.flush()?;
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let (w, h) = (engine.width(), engine.height());
        let buf = engine.buffer_mut();
        buf.clear();

        // 1. Draw outer frame
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

        // Header Title
        let header = format!("GALERNE v{}", crate::VERSION);
        let hx = (w.saturating_sub(header.len() as u16)) / 2;
        buf.draw_str_styled(hx, 0, &header, Style::new(Color::Yellow, bg).bold());

        // Horizontal divider under header
        buf.set_char(0, 2, '├', box_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, 2, '─', box_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), 2, '┤', box_fg, bg);

        // Vertical split at column 24
        let split_x = 24u16.min(w / 3);
        buf.set_char(split_x, 2, '┬', box_fg, bg);
        for y in 3..h.saturating_sub(3) {
            buf.set_char(split_x, y, '│', box_fg, bg);
        }
        buf.set_char(split_x, h.saturating_sub(3), '┴', box_fg, bg);

        // Horizontal divider above footer
        buf.set_char(0, h.saturating_sub(3), '├', box_fg, bg);
        for x in 1..w.saturating_sub(1) {
            buf.set_char(x, h.saturating_sub(3), '─', box_fg, bg);
        }
        buf.set_char(w.saturating_sub(1), h.saturating_sub(3), '┤', box_fg, bg);

        // Left Navigation Menu
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
            let line = format!("{}{:<14}", prefix, tab.label());

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

        // Right Preview Panel
        let current_tab = DashboardTab::ALL[current_idx];
        let px = split_x + 3;
        let py = 4;

        match current_tab {
            DashboardTab::Play => {
                buf.draw_str_styled(
                    px,
                    py,
                    "DEPLOYMENT CONTROLS",
                    Style::new(Color::Yellow, bg).bold(),
                );
                if let Some(ref run) = save_state.active_run {
                    buf.draw_str(
                        px,
                        py + 2,
                        "Active Operation in Progress (interrupted):",
                        Color::White,
                        bg,
                    );
                    let info = format!(
                        "Wave {}  |  Level {}  |  HP {:.0}/{:.0}  |  Enemies Active: {}  |  Projectiles: {}",
                        run.wave,
                        run.player_level,
                        run.player_hp,
                        run.player_max_hp,
                        run.enemies.len(),
                        run.projectiles.len()
                    );
                    buf.draw_str_styled(px, py + 3, &info, Style::new(Color::Cyan, bg).bold());
                    buf.draw_str(
                        px,
                        py + 5,
                        "Press Enter to Resume Operation exactly where left off.",
                        Color::Green,
                        bg,
                    );
                } else {
                    buf.draw_str(px, py + 2, "System Status: Idle", Color::White, bg);
                    buf.draw_str(
                        px,
                        py + 3,
                        "Press Enter to launch a manual combat deployment.",
                        Color::DarkGrey,
                        bg,
                    );
                    let shards_str =
                        format!("Gale Shards Available: {}", save_state.currency_shards);
                    buf.draw_str_styled(
                        px,
                        py + 5,
                        &shards_str,
                        Style::new(Color::Cyan, bg).bold(),
                    );
                }
            }
            DashboardTab::Workshop => {
                let w_header = format!(
                    "WORKSHOP ENHANCEMENTS — {} SHARDS",
                    save_state.currency_shards
                );
                buf.draw_str_styled(px, py, &w_header, Style::new(Color::Yellow, bg).bold());

                for (idx, up_type) in WorkshopUpgradeType::ALL.iter().enumerate() {
                    let y = py + 2 + (idx as u16 * 2);
                    let lvl = up_type.get_level(&save_state.workshop);
                    let is_sel = idx == workshop_idx && focus_panel;

                    let cost_str = if lvl >= up_type.max_level() {
                        "MAX".to_string()
                    } else {
                        format!("{} Shards", up_type.cost(lvl))
                    };

                    let line = format!(
                        "{} {:<18} Rank {}/5  [{}]",
                        if is_sel { "▶" } else { " " },
                        up_type.title(),
                        lvl,
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
                buf.draw_str(
                    px,
                    py + 13,
                    "↑/↓: select item   Enter: purchase rank   Tab/Esc: back to menu",
                    Color::DarkGrey,
                    bg,
                );
            }
            DashboardTab::Stats => {
                buf.draw_str_styled(
                    px,
                    py,
                    "CAREER METRICS",
                    Style::new(Color::Yellow, bg).bold(),
                );
                let stats_text = format_career_stats(&save_state);
                for (line_idx, line) in stats_text.lines().enumerate() {
                    buf.draw_str(px, py + 2 + line_idx as u16, line, Color::White, bg);
                }
            }
            DashboardTab::Saves => {
                buf.draw_str_styled(
                    px,
                    py,
                    "SAVE & BACKUP INTEGRITY",
                    Style::new(Color::Yellow, bg).bold(),
                );
                let save_path = save_manager.primary_save_path();
                let p_str = format!("Path: {}", save_path.display());
                buf.draw_str(px, py + 2, &p_str, Color::DarkGrey, bg);

                let archives = save_manager.list_archives().unwrap_or_default();

                if archives.is_empty() {
                    buf.draw_str(
                        px,
                        py + 4,
                        "Active Save: Present (CRC32 Verified)",
                        Color::Green,
                        bg,
                    );
                    buf.draw_str(
                        px,
                        py + 6,
                        "No historical archives found. Use `galerne reset` to create an archive.",
                        Color::DarkGrey,
                        bg,
                    );
                } else {
                    buf.draw_str_styled(
                        px,
                        py + 4,
                        &format!("Archived Snapshots ({} available):", archives.len()),
                        Style::new(Color::Cyan, bg),
                    );

                    for (idx, (name, size)) in archives.iter().take(5).enumerate() {
                        let y = py + 6 + (idx as u16);
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
                        py + 12,
                        "Enter: restore selected archive   d: delete archive   Tab/Esc: back",
                        Color::DarkGrey,
                        bg,
                    );
                }
            }
            DashboardTab::Doctor => {
                buf.draw_str_styled(
                    px,
                    py,
                    "SYSTEM DIAGNOSTICS (LIVE RUNTIME FINDINGS)",
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
                        format!("{}:", diag.label),
                        diag.status
                    );
                    buf.draw_str_styled(px, y, &text, Style::new(color, bg));
                }
            }
            DashboardTab::Settings => {
                buf.draw_str_styled(
                    px,
                    py,
                    "PREFERENCES (INTERACTIVE)",
                    Style::new(Color::Yellow, bg).bold(),
                );
                let s = &save_state.settings;

                let settings_rows = [
                    (
                        "Color Mode",
                        if s.color_enabled {
                            "Enabled"
                        } else {
                            "Disabled"
                        },
                        "[Enter/Space]: toggle",
                    ),
                    (
                        "Transition Animations",
                        if s.animations_enabled {
                            "Enabled"
                        } else {
                            "Disabled"
                        },
                        "[Enter/Space]: toggle",
                    ),
                    (
                        "Auto-Activation",
                        if s.auto_activate {
                            "Enabled"
                        } else {
                            "Disabled"
                        },
                        "[Enter/Space]: toggle",
                    ),
                    (
                        "Anti-Flicker Threshold",
                        &format!("{} ms", s.anti_flicker_delay_ms),
                        "[Enter/Space/←/→]: cycle duration",
                    ),
                ];

                for (idx, (label, val, hint)) in settings_rows.iter().enumerate() {
                    let y = py + 2 + (idx as u16 * 2);
                    let is_sel = idx == settings_idx && focus_panel;

                    let line = format!(
                        "{} {:<24} {:<10} {}",
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

                buf.draw_str(
                    px,
                    py + 12,
                    "↑/↓: navigate   Enter/Space/←/→: change setting   Tab/Esc: back to menu",
                    Color::DarkGrey,
                    bg,
                );
            }
            DashboardTab::Help => {
                buf.draw_str_styled(
                    px,
                    py,
                    "CONTROLS & HOW IT WORKS",
                    Style::new(Color::Yellow, bg).bold(),
                );
                buf.draw_str(px, py + 2, "Controls during combat:", Color::White, bg);
                buf.draw_str(
                    px,
                    py + 3,
                    "  Arrows or WASD : Move & Dodge",
                    Color::Cyan,
                    bg,
                );
                buf.draw_str(
                    px,
                    py + 4,
                    "  Automatic      : Gale Blaster auto-fires at nearest foes",
                    Color::Cyan,
                    bg,
                );
                buf.draw_str(
                    px,
                    py + 5,
                    "  Ctrl+C / Esc   : Suspend gameplay and return instantly",
                    Color::Cyan,
                    bg,
                );
                buf.draw_str(
                    px,
                    py + 7,
                    "During development tasks (cargo build, npm install...):",
                    Color::White,
                    bg,
                );
                buf.draw_str(
                    px,
                    py + 8,
                    "  Galerne appears for long tasks (~150ms for known long, ~8s fallback).",
                    Color::DarkGrey,
                    bg,
                );
                buf.draw_str(
                    px,
                    py + 9,
                    "  Pressing Ctrl+C leaves the game without killing your build!",
                    Color::DarkGrey,
                    bg,
                );
            }
            DashboardTab::Exit => {
                buf.draw_str_styled(px, py, "EXIT GALERNE", Style::new(Color::Red, bg).bold());
                buf.draw_str(
                    px,
                    py + 2,
                    "Press Enter to return to your shell prompt.",
                    Color::White,
                    bg,
                );
            }
        }

        // Temporary status message banner
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

        // Footer instructions
        let footer = if focus_panel {
            "↑/↓: navigate item   Enter: action   Tab/Esc: back to menu   Ctrl+C/q: exit"
        } else {
            "↑/↓: navigate menu   Tab/Enter: enter panel   Esc/q: exit"
        };
        buf.draw_str_styled(
            3,
            h.saturating_sub(2),
            footer,
            Style::new(Color::DarkGrey, bg),
        );

        engine.flush()?;

        // Handle Dashboard Inputs
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
                                        3
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
                                    settings_idx = (settings_idx + 1) % 4;
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
                                status_banner =
                                    Some((format!("Error saving: {}", e), Instant::now()));
                            }
                        } else {
                            focus_panel = false;
                        }
                    }
                    KeyCode::Right => {
                        if focus_panel && current_tab == DashboardTab::Settings {
                            toggle_or_adjust_setting(&mut save_state, settings_idx, 1);
                            if let Err(e) = save_manager.save(&save_state) {
                                status_banner =
                                    Some((format!("Error saving: {}", e), Instant::now()));
                            }
                        } else {
                            focus_panel = true;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if focus_panel && current_tab == DashboardTab::Settings {
                            toggle_or_adjust_setting(&mut save_state, settings_idx, 1);
                            if let Err(e) = save_manager.save(&save_state) {
                                status_banner =
                                    Some((format!("Error saving: {}", e), Instant::now()));
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
                                        format!("✓ Deleted archive {}", name),
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
                                                format!("Error saving: {}", e),
                                                Instant::now(),
                                            ));
                                        } else {
                                            status_banner = Some((
                                                format!("✓ Purchased rank for {}", target.title()),
                                                Instant::now(),
                                            ));
                                        }
                                    } else {
                                        status_banner = Some((
                                            "Not enough Gale Shards!".to_string(),
                                            Instant::now(),
                                        ));
                                    }
                                }
                                DashboardTab::Settings => {
                                    toggle_or_adjust_setting(&mut save_state, settings_idx, 1);
                                    if let Err(e) = save_manager.save(&save_state) {
                                        status_banner =
                                            Some((format!("Error saving: {}", e), Instant::now()));
                                    }
                                }
                                DashboardTab::Saves => {
                                    let archives = save_manager.list_archives().unwrap_or_default();
                                    if saves_idx < archives.len() {
                                        let (name, _) = &archives[saves_idx];
                                        match save_manager.restore_archive(name) {
                                            Ok(_) => {
                                                save_state =
                                                    save_manager.load().unwrap_or_default();
                                                status_banner = Some((
                                                    format!("✓ Restored {} successfully", name),
                                                    Instant::now(),
                                                ));
                                            }
                                            Err(e) => {
                                                status_banner = Some((
                                                    format!("Failed to restore: {}", e),
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
            let thresholds = [1000u64, 1500, 2000, 2500, 3000, 4000, 5000];
            let current = save_state.settings.anti_flicker_delay_ms;
            let current_pos = thresholds.iter().position(|&t| t == current).unwrap_or(3);
            let new_pos = if delta > 0 {
                (current_pos + 1) % thresholds.len()
            } else if current_pos == 0 {
                thresholds.len() - 1
            } else {
                current_pos - 1
            };
            save_state.settings.anti_flicker_delay_ms = thresholds[new_pos];
        }
        _ => {}
    }
}

/// Runs a standalone manual play session directly from the CLI.
pub fn run_direct_play() -> io::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut engine = Engine::new()?;
    let save_manager = SaveManager::default_paths();
    let mut save_state = save_manager.load().unwrap_or_default();
    run_manual_game(&mut engine, &save_manager, &mut save_state)
}

/// Runs a standalone manual play session from the dashboard or direct CLI invocation.
pub fn run_manual_game(
    engine: &mut Engine,
    save_manager: &SaveManager,
    save_state: &mut crate::storage::schema::SaveState,
) -> io::Result<()> {
    let mut game = GameState::new(engine.width(), engine.height(), rand_seed());
    apply_workshop_to_game(&save_state.workshop, &mut game);

    if let Some(ref snapshot) = save_state.active_run {
        game.resume_snapshot(snapshot);
    }

    let start = Instant::now();
    let initial_shards = game.player.shards;
    let target_frame_duration = Duration::from_millis(38); // ~26 FPS
    let mut last_frame = Instant::now();
    let mut input = GameInput::default();

    loop {
        while let Some(ev) = Engine::poll_event(Duration::from_millis(0))? {
            match ev {
                Event::Key(k) if k.code == KeyCode::Esc || k.code == KeyCode::Char('q') => {
                    // Save and exit back to menu
                    save_state.total_play_time_secs += start.elapsed().as_secs();
                    save_state.currency_shards += game.player.shards.saturating_sub(initial_shards);
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
                    save_state.currency_shards += game.player.shards.saturating_sub(initial_shards);
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
                Event::Key(k) => {
                    if game.pending_level_up {
                        if let Some(ref mut draft) = game.current_draft {
                            match k.code {
                                KeyCode::Up => draft.select_prev(),
                                KeyCode::Down => draft.select_next(),
                                KeyCode::Enter => {
                                    let sel = draft.current_selection();
                                    game.apply_upgrade(sel);
                                }
                                _ => {}
                            }
                        }
                    } else {
                        match k.code {
                            KeyCode::Up | KeyCode::Char('w') => input.move_y = -1.0,
                            KeyCode::Down | KeyCode::Char('s') => input.move_y = 1.0,
                            KeyCode::Left | KeyCode::Char('a') => input.move_x = -1.0,
                            KeyCode::Right | KeyCode::Char('d') => input.move_x = 1.0,
                            _ => {}
                        }
                    }
                }
                Event::Resize(w, h) => {
                    let _ = engine.check_resize();
                    game.arena = crate::game::arena::Arena::new(w, h);
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        game.update(dt, input);
        input = GameInput::default();

        if engine.is_too_small() {
            engine.render_too_small_notice();
        } else {
            game.render(engine.buffer_mut());
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
