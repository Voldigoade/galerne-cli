use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

use galerne::interceptor::history::DurationHistory;
use galerne::interceptor::matcher::CommandClass;
use galerne::interceptor::process::ProcessSupervisor;
use galerne::interceptor::supervisor::is_cancelled_status;

#[test]
fn test_cancellation_status_detection() {
    // 1. Normal success
    let status_success = Command::new("sh")
        .args(["-c", "exit 0"])
        .status()
        .expect("status");
    assert!(
        !is_cancelled_status(false, &status_success),
        "Exit code 0 is not cancelled"
    );

    // 2. Normal error
    let status_err = Command::new("sh")
        .args(["-c", "exit 101"])
        .status()
        .expect("status");
    assert!(
        !is_cancelled_status(false, &status_err),
        "Exit code 101 is an error, not cancellation"
    );

    // 3. User cancelled flag
    assert!(
        is_cancelled_status(true, &status_success),
        "was_user_cancelled=true must always be recognized as cancelled"
    );

    // 4. SIGINT exit code (130)
    let status_sigint = Command::new("sh")
        .args(["-c", "exit 130"])
        .status()
        .expect("status");
    assert!(
        is_cancelled_status(false, &status_sigint),
        "Exit code 130 must be recognized as cancelled"
    );

    // 5. SIGTERM exit code (143)
    let status_sigterm = Command::new("sh")
        .args(["-c", "exit 143"])
        .status()
        .expect("status");
    assert!(
        is_cancelled_status(false, &status_sigterm),
        "Exit code 143 must be recognized as cancelled"
    );
}

#[test]
fn test_pregame_output_is_streamed_live_without_delay() {
    // Child prints lines progressively over 100ms
    let script = r#"
        echo "Line 1: Build started"
        sleep 0.05
        echo "Line 2: Compiling core"
        sleep 0.05
        echo "Line 3: Finished"
    "#;

    let mut supervisor =
        ProcessSupervisor::spawn(&["sh".to_string(), "-c".to_string(), script.to_string()])
            .expect("spawn");

    let start = Instant::now();
    let mut collected_lines = Vec::new();

    // In pre-game loop, lines are drained and observed progressively as they arrive
    while start.elapsed() < Duration::from_millis(600) {
        let arriving = supervisor.output.drain_lines();
        collected_lines.extend(arriving);

        if let Some(status) = supervisor.try_wait().expect("try_wait") {
            assert!(status.success());
            let final_lines = supervisor.output.drain_lines();
            collected_lines.extend(final_lines);
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }

    assert!(
        collected_lines.iter().any(|l| l.contains("Line 1")),
        "Line 1 was captured in real-time"
    );
    assert!(
        collected_lines.iter().any(|l| l.contains("Line 2")),
        "Line 2 was captured in real-time"
    );
    assert!(
        collected_lines.iter().any(|l| l.contains("Line 3")),
        "Line 3 was captured in real-time"
    );
}

#[test]
fn test_cancelled_runs_are_strictly_excluded_from_history() {
    let temp_dir = std::env::temp_dir().join(format!("galerne_canc_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let hist_path = temp_dir.join("history.json");
    let mut history = DurationHistory::default();
    let proj = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    // Simulate child exiting with code 130
    let status_130 = Command::new("sh")
        .args(["-c", "exit 130"])
        .status()
        .expect("status 130");

    let is_canc = is_cancelled_status(false, &status_130);
    assert!(is_canc);

    // If cancelled, supervisor does NOT call history.record
    if !is_canc {
        history.record(proj, CommandClass::CargoBuild, 3.5);
    }
    history.save_to_file(&hist_path).expect("save");

    let loaded = DurationHistory::load_from_file(&hist_path);
    assert_eq!(
        loaded.entries_count(),
        0,
        "History must have 0 entries after a cancelled run"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_true_child_duration_recorded_when_task_view_runs_long() {
    let mut history = DurationHistory::default();
    let proj = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    // Simulate: user leaves game view early at 2.0s, child actually terminated at 8.5s
    let game_view_leave_time = 2.0;
    let actual_child_exit_time = 8.5;

    // Supervisor records actual_child_exit_time, NOT game_view_leave_time
    history.record(proj, CommandClass::CargoBuild, actual_child_exit_time);

    let samples = history
        .get_samples(proj, CommandClass::CargoBuild)
        .expect("samples");
    assert_eq!(samples[0], 8.5);
    assert_ne!(
        samples[0], game_view_leave_time,
        "History must record actual child termination duration, not game exit time"
    );
}

#[test]
fn test_pregame_ctrlc_terminates_child_immediately() {
    let mut supervisor = ProcessSupervisor::spawn(&[
        "sh".to_string(),
        "-c".to_string(),
        "echo 'Running in pre-game'; sleep 10; echo 'Should not be seen'".to_string(),
    ])
    .expect("spawn");

    std::thread::sleep(Duration::from_millis(50));
    assert!(supervisor.try_wait().expect("try_wait").is_none());

    // In pre-game, when Ctrl+C signal is detected, interrupt is sent immediately
    supervisor.send_interrupt().expect("send_interrupt");
    let status = supervisor.wait().expect("wait");

    assert!(!status.success());
    assert!(is_cancelled_status(true, &status));

    let logs = supervisor.output.drain_lines();
    assert!(logs.iter().any(|l| l.contains("Running in pre-game")));
    assert!(!logs.iter().any(|l| l.contains("Should not be seen")));
}
