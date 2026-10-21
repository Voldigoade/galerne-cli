//! Master supervision coordinator integrating task execution, anti-flicker delay,
//! and two-tier Ctrl+C signal routing.

use std::io::{self, stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::game::state::{GameInput, GameState};
use crate::interceptor::history::DurationHistory;
use crate::interceptor::matcher::match_command;
use crate::interceptor::process::ProcessSupervisor;
use crate::interceptor::signal;
use crate::interceptor::transition::run_completion_transition;
use crate::storage::manager::SaveManager;
use crate::storage::workshop::apply_workshop_to_game;
use crate::terminal::engine::Engine;
use crate::terminal::guard::TerminalGuard;

/// Runs an eligible developer task, supervising execution and presenting gameplay during wait time.
/// Returns the exit code of the child process.
pub fn execute_wrapped_task(args: &[String]) -> io::Result<i32> {
    if args.is_empty() {
        return Ok(0);
    }

    let save_manager = SaveManager::default_paths();
    let mut save_state = save_manager.load().unwrap_or_default();

    // If auto-activation is disabled, execute directly without gameplay
    if !save_state.settings.auto_activate {
        let mut cmd = std::process::Command::new(&args[0]);
        if args.len() > 1 {
            cmd.args(&args[1..]);
        }
        let status = cmd.status()?;
        return Ok(status
            .code()
            .unwrap_or(if status.success() { 0 } else { 1 }));
    }

    // Check command eligibility: filter out non-eligible commands, interactive programs,
    // unsupported subcommands (e.g. cargo run, docker ps, npm run dev), and watch modes.
    let matched_class = match match_command(args) {
        Some(class) => class,
        None => {
            // Bypass Galerne completely with inherited stdin/stdout/stderr
            let mut cmd = std::process::Command::new(&args[0]);
            if args.len() > 1 {
                cmd.args(&args[1..]);
            }
            let status = cmd.status()?;
            return Ok(status
                .code()
                .unwrap_or(if status.success() { 0 } else { 1 }));
        }
    };

    let project_id =
        crate::interceptor::project::resolve_current_project_id(save_manager.data_dir());
    let history_path = save_manager.data_dir().join("history.json");
    let mut history = DurationHistory::load_from_file(&history_path);

    let decision = history.evaluate_decision(&project_id, matched_class);

    let start_time = Instant::now();

    // Install cross-platform Ctrl+C handler
    signal::setup_handler();
    signal::reset();

    // Spawn child process
    let mut supervisor = ProcessSupervisor::spawn(args)?;

    let anti_flicker_delay = Duration::from_millis(decision.effective_delay_ms());

    // Phase 1: Pre-game observation period with live output streaming
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
        // Task completed within observation window. Flush any remaining logs.
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

    // Phase 2: Enter Game View
    let guard = TerminalGuard::enter()?;
    let mut engine = Engine::new()?;

    // Load or initialize gameplay session
    let mut game = GameState::new(engine.width(), engine.height(), rand_seed());
    apply_workshop_to_game(&save_state.workshop, &mut game);

    // If an active run snapshot exists, resume it with exact deterministic state
    if let Some(ref snapshot) = save_state.active_run {
        game.resume_snapshot(snapshot);
    }

    let initial_xp = game.player.xp;
    let initial_shards = game.player.shards;

    let target_frame_duration = Duration::from_millis(38); // ~26 FPS
    let mut last_frame = Instant::now();
    let mut input = GameInput::default();
    let mut left_game_view = false;
    let mut child_completed_in_game = None;

    while child_completed_in_game.is_none() && !left_game_view {
        // Check if child process has finished
        if let Some(status) = supervisor.try_wait()? {
            child_completed_in_game = Some(status);
            break;
        }

        // Handle user input
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

        if left_game_view {
            break;
        }

        // Tick simulation
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        game.update(dt, input);
        // Reset single-frame input impulses
        input = GameInput::default();

        // Render frame
        if engine.is_too_small() {
            engine.render_too_small_notice();
        } else {
            game.render(engine.buffer_mut());
        }
        engine.flush()?;

        // Frame timing limiter
        let elapsed = last_frame.elapsed();
        if elapsed < target_frame_duration {
            thread::sleep(target_frame_duration - elapsed);
        }
    }

    // Calculate progression earned during wait time
    let xp_gained = game.player.xp.saturating_sub(initial_xp);
    let shards_gained = game.player.shards.saturating_sub(initial_shards);
    let total_duration = start_time.elapsed().as_secs();

    // Persist career stats and exact active run
    save_state.total_play_time_secs += game.run_duration as u64;
    save_state.total_wait_time_saved_secs += total_duration;
    save_state.total_enemies_killed += game.enemies_killed;
    save_state.currency_shards += shards_gained;

    if game.is_game_over {
        save_state.active_run = None;
    } else {
        save_state.active_run = Some(game.create_snapshot());
    }

    if let Err(e) = save_manager.save(&save_state) {
        eprintln!(
            "[galerne] Warning: Failed to persist career progression: {}",
            e
        );
    }

    if let Some(status) = child_completed_in_game {
        // Child completed while in game view: play brief completion transition
        let true_duration = start_time.elapsed().as_secs_f32();
        run_completion_transition(&mut engine, status.success(), xp_gained, shards_gained);

        // Restore terminal to cooked mode
        drop(guard);

        // Print only buffered lines that arrived during game view
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

    // Print all buffered lines collected during game view
    print_buffered_logs(&supervisor);

    // Enter Task View: live streaming until child finishes or second Ctrl+C arrives
    signal::reset();

    let mut was_cancelled = false;
    let exit_status = loop {
        // Check if child has terminated
        if let Some(status) = supervisor.try_wait()? {
            print_buffered_logs(&supervisor);
            break status;
        }

        // Check for second Ctrl+C in task view
        if signal::is_interrupted() {
            was_cancelled = true;
            let _ = supervisor.send_interrupt();
            let status = supervisor.wait()?;
            print_buffered_logs(&supervisor);
            break status;
        }

        // Stream arriving output live
        print_buffered_logs(&supervisor);
        thread::sleep(Duration::from_millis(20));
    };

    // Measure TRUE child duration at actual termination
    let true_duration = start_time.elapsed().as_secs_f32();

    // User-cancelled tasks must NEVER poison prediction history on ANY path
    if !is_cancelled_status(was_cancelled, &exit_status) {
        history.record(&project_id, matched_class, true_duration);
        let _ = history.save_to_file(&history_path);
    }

    // Update career total wait time with true measured duration
    save_state.total_wait_time_saved_secs += true_duration as u64;
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
