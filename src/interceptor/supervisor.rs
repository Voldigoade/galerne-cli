//! Master supervision coordinator integrating task execution, anti-flicker delay,
//! and two-tier Ctrl+C signal routing.

use std::io::{self, stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::game::runner::action::PlayerInput;
use crate::game::runner::RunnerState;
use crate::interceptor::history::DurationHistory;
use crate::interceptor::matcher::match_command;
use crate::interceptor::process::ProcessSupervisor;
use crate::interceptor::signal;
use crate::interceptor::transition::run_completion_transition;
use crate::storage::manager::SaveManager;
use crate::storage::schema::SaveState;
use crate::storage::workshop::apply_workshop_to_runner;
use crate::terminal::engine::Engine;
use crate::terminal::guard::TerminalGuard;
use crate::ui::i18n::Locale;

/// Runs an eligible developer task, supervising execution and presenting gameplay during wait time.
/// Returns the exit code of the child process.
pub fn execute_wrapped_task(args: &[String]) -> io::Result<i32> {
    if args.is_empty() {
        return Ok(0);
    }

    let save_manager = SaveManager::default_paths();
    let mut save_state = match save_manager.load_or_default() {
        Ok(save_state) => save_state,
        Err(error) => {
            eprintln!(
                "[galerne] Save unavailable; running task without gameplay: {}",
                error
            );
            return run_direct_task(args);
        }
    };

    // If auto-activation is disabled, execute directly without gameplay
    if !save_state.settings.auto_activate {
        return run_direct_task(args);
    }

    // Check command eligibility: filter out non-eligible commands, interactive programs,
    // unsupported subcommands (e.g. cargo run, docker ps, npm run dev), and watch modes.
    let matched_class = match match_command(args) {
        Some(class) => class,
        None => {
            // Bypass Galerne completely with inherited stdin/stdout/stderr
            return run_direct_task(args);
        }
    };

    let project_id =
        crate::interceptor::project::resolve_current_project_id(save_manager.data_dir());
    let history_path = save_manager.data_dir().join("history.json");
    let mut history = DurationHistory::load_from_file(&history_path);

    let decision = history.evaluate_decision(&project_id, matched_class);

    let start_time = Instant::now();
    let mut wait_time_accounting = WaitTimeAccounting::default();

    signal::setup_handler();
    signal::reset();

    let mut supervisor = ProcessSupervisor::spawn(args)?;

    let anti_flicker_delay = Duration::from_millis(decision.effective_delay_ms());

    let observation_start = Instant::now();
    let mut child_completed_early = None;

    while observation_start.elapsed() < anti_flicker_delay {
        // Stream arriving output live to stdout so developer commands never appear frozen
        print_buffered_logs(&supervisor);

        // Pre-game Ctrl+C behaves like normal immediate task cancellation
        if signal::is_interrupted() {
            let _ = supervisor.send_interrupt();
            let status = supervisor.wait()?;
            print_buffered_logs(&supervisor);
            return Ok(status.code().unwrap_or(130));
        }

        if let Some(status) = supervisor.try_wait()? {
            child_completed_early = Some(status);
            break;
        }

        thread::sleep(Duration::from_millis(15));
    }

    if let Some(status) = child_completed_early {
        print_buffered_logs(&supervisor);
        let duration_secs = start_time.elapsed().as_secs_f32();
        if !is_cancelled_status(false, &status) {
            history.record(&project_id, matched_class, duration_secs);
            let _ = history.save_to_file(&history_path);
        }
        return Ok(status
            .code()
            .unwrap_or(if status.success() { 0 } else { 1 }));
    }

    let guard = TerminalGuard::enter()?;
    let mut engine = Engine::new()?;

    let locale = save_state.settings.locale.unwrap_or_else(Locale::detect);
    let mut game = RunnerState::new(
        engine.width() as usize,
        engine.height() as usize,
        rand_seed(),
    );
    game.set_locale(locale);
    let predicted_secs = history
        .median_duration(&project_id, matched_class)
        .unwrap_or(0.0);
    game.tailwind_tier =
        crate::game::runner::wind::TailwindTier::from_predicted_duration(predicted_secs);
    apply_workshop_to_runner(&save_state.workshop, &mut game);

    // If an active run snapshot exists, resume it with exact deterministic state
    if let Some(ref snapshot) = save_state.active_run {
        game.resume_snapshot(snapshot);
    } else {
        save_state.total_runs_started += 1;
    }
    game.canal_gate_opened |= save_state.world_progress.canal_gate_opened;

    let initial_shards = game.shards_collected;
    let initial_dist = game.distance;

    let target_frame_duration = Duration::from_millis(33); // ~30 FPS
    let mut last_frame = Instant::now();
    let mut left_game_view = false;
    let mut child_completed_in_game = None;
    let mut next_input = None;

    while child_completed_in_game.is_none() && !left_game_view {
        if let Some(status) = supervisor.try_wait()? {
            let completion_duration = start_time.elapsed();
            game.start_completion_transition(status.success());
            child_completed_in_game = Some((status, completion_duration));
        }

        while let Some(ev) = Engine::poll_event(Duration::from_millis(0))? {
            match ev {
                // FIRST Ctrl+C or Esc/q: leaves game view immediately without killing child task!
                Event::Key(k)
                    if k.modifiers.contains(KeyModifiers::CONTROL)
                        && k.code == KeyCode::Char('c') =>
                {
                    left_game_view = true;
                    break;
                }
                Event::Key(k) if k.code == KeyCode::Esc || k.code == KeyCode::Char('q') => {
                    left_game_view = true;
                    break;
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

        if left_game_view {
            break;
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        game.update(dt, next_input);
        next_input = None;

        if game.is_transition_done() {
            break;
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

    let dist_gained = (game.distance - initial_dist).max(0.0);
    let shards_gained = game.shards_collected.saturating_sub(initial_shards);
    let total_duration = start_time.elapsed().as_secs();

    save_state.total_play_time_secs += total_duration;
    if let Some((_, completion_duration)) = child_completed_in_game.as_ref() {
        wait_time_accounting.record_completion(&mut save_state, *completion_duration);
    }
    save_state.total_distance_traveled += dist_gained;
    save_state.currency_shards += shards_gained;
    if shards_gained > 0 {
        save_state
            .inventory
            .add(crate::storage::ItemKind::GaleShard, shards_gained);
    }
    if game.relay_restored {
        save_state.world_progress.relay_restored = true;
        save_state.refuge.power_restored = true;
    }
    if game.companion_met {
        save_state.companion.met_companion = true;
    }
    if save_state.world_progress.relay_restored && save_state.companion.met_companion {
        save_state.companion.companion_at_refuge = true;
    }
    if game.secret_cache_found {
        save_state.world_progress.secret_cache_found = true;
    }
    save_state.world_progress.canal_gate_opened |= game.canal_gate_opened;
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
    save_state.unlock_workbench_from_inventory();
    for &cp_id in &game.unlocked_checkpoints {
        save_state.world_progress.record_checkpoint(cp_id);
    }

    save_state.active_run = Some(game.create_snapshot());

    if let Err(e) = save_manager.save(&save_state) {
        eprintln!(
            "[galerne] Warning: Failed to persist career progression: {}",
            e
        );
    }

    if let Some((status, completion_duration)) = child_completed_in_game {
        let true_duration = completion_duration.as_secs_f32();
        run_completion_transition(
            &mut engine,
            status.success(),
            dist_gained as u32,
            shards_gained,
            locale,
        );

        // Restore terminal to cooked mode
        drop(guard);

        print_buffered_logs(&supervisor);

        if !is_cancelled_status(false, &status) {
            history.record(&project_id, matched_class, true_duration);
            let _ = history.save_to_file(&history_path);
        }

        return Ok(status
            .code()
            .unwrap_or(if status.success() { 0 } else { 1 }));
    }

    // First Ctrl+C or Esc/q triggered: restore cooked terminal mode immediately
    drop(guard);

    print_buffered_logs(&supervisor);

    // Enter Task View: live streaming until child finishes or second Ctrl+C arrives
    signal::reset();

    let mut was_cancelled = false;
    let exit_status = loop {
        if let Some(status) = supervisor.try_wait()? {
            print_buffered_logs(&supervisor);
            break status;
        }

        if signal::is_interrupted() {
            was_cancelled = true;
            let _ = supervisor.send_interrupt();
            let status = supervisor.wait()?;
            print_buffered_logs(&supervisor);
            break status;
        }

        print_buffered_logs(&supervisor);
        thread::sleep(Duration::from_millis(20));
    };

    // Measure TRUE child duration at actual termination
    let completion_duration = start_time.elapsed();
    let true_duration = completion_duration.as_secs_f32();

    // User-cancelled tasks must NEVER poison prediction history on ANY path
    if !is_cancelled_status(was_cancelled, &exit_status) {
        history.record(&project_id, matched_class, true_duration);
        let _ = history.save_to_file(&history_path);
    }

    wait_time_accounting.record_completion(&mut save_state, completion_duration);
    let _ = save_manager.save(&save_state);

    Ok(exit_status
        .code()
        .unwrap_or(if exit_status.success() { 0 } else { 130 }))
}

/// Determines whether an execution was cancelled by user action or termination signals.
pub fn is_cancelled_status(was_user_cancelled: bool, status: &std::process::ExitStatus) -> bool {
    if was_user_cancelled {
        return true;
    }
    if let Some(code) = status.code() {
        if code == 130 || code == 143 {
            return true;
        }
        #[cfg(windows)]
        {
            if code == -1073741510 || code as u32 == 0xC000013A {
                return true;
            }
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            if sig == 2 || sig == 15 || sig == 9 {
                return true;
            }
        }
    }
    false
}

fn print_buffered_logs(supervisor: &ProcessSupervisor) {
    let lines = supervisor.output.drain_lines();
    if !lines.is_empty() {
        let mut out = stdout();
        for line in lines {
            let _ = writeln!(out, "{}", line);
        }
        let _ = out.flush();
    }
}

fn rand_seed() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xdeadbeef12345678)
}

fn run_direct_task(args: &[String]) -> io::Result<i32> {
    let mut command = std::process::Command::new(&args[0]);
    if args.len() > 1 {
        command.args(&args[1..]);
    }
    let status = command.status()?;
    Ok(status
        .code()
        .unwrap_or(if status.success() { 0 } else { 1 }))
}

#[derive(Default)]
struct WaitTimeAccounting {
    recorded: bool,
}

impl WaitTimeAccounting {
    fn record_completion(&mut self, save_state: &mut SaveState, duration: Duration) {
        if self.recorded {
            return;
        }
        save_state.total_wait_time_saved_secs = save_state
            .total_wait_time_saved_secs
            .saturating_add(duration.as_secs());
        self.recorded = true;
    }
}

#[cfg(test)]
mod tests {
    use super::WaitTimeAccounting;
    use crate::storage::schema::SaveState;
    use std::time::Duration;

    #[test]
    fn wait_time_is_recorded_once_after_leaving_game_view() {
        let mut save_state = SaveState::default();
        let mut accounting = WaitTimeAccounting::default();
        let game_view_duration = Duration::from_secs(5);
        save_state.total_play_time_secs += game_view_duration.as_secs();

        assert_eq!(save_state.total_wait_time_saved_secs, 0);

        let child_duration = Duration::from_secs(15);
        accounting.record_completion(&mut save_state, child_duration);
        accounting.record_completion(&mut save_state, child_duration);

        assert_eq!(save_state.total_play_time_secs, 5);
        assert_eq!(save_state.total_wait_time_saved_secs, 15);
    }

    #[test]
    fn in_game_completion_records_the_child_duration_once() {
        let mut save_state = SaveState::default();
        let mut accounting = WaitTimeAccounting::default();
        let child_duration = Duration::from_secs(9);

        accounting.record_completion(&mut save_state, child_duration);
        accounting.record_completion(&mut save_state, child_duration);

        assert_eq!(save_state.total_wait_time_saved_secs, 9);
    }
}
