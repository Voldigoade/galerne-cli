use std::time::Duration;

use galerne::interceptor::process::{OutputBuffer, ProcessSupervisor};

#[test]
fn test_child_process_success_and_exit_code() {
    let mut supervisor = ProcessSupervisor::spawn(&[
        "sh".to_string(),
        "-c".to_string(),
        "echo 'Hello from child'; exit 0".to_string(),
    ])
    .expect("spawn should succeed");

    let status = supervisor.wait().expect("wait should succeed");
    assert!(status.success(), "Process should exit with success code 0");
    assert_eq!(status.code(), Some(0));

    let logs = supervisor.output.drain_lines();
    let text = logs.join("\n");
    assert!(
        text.contains("Hello from child"),
        "Output buffer captured child stdout"
    );
}

#[test]
fn test_child_process_failure_exit_code_preservation() {
    let mut supervisor = ProcessSupervisor::spawn(&[
        "sh".to_string(),
        "-c".to_string(),
        "echo 'Compilation error' >&2; exit 101".to_string(),
    ])
    .expect("spawn should succeed");

    let status = supervisor.wait().expect("wait should succeed");
    assert!(!status.success(), "Process should fail");
    assert_eq!(
        status.code(),
        Some(101),
        "Exit code 101 preserved faithfully"
    );

    let logs = supervisor.output.drain_lines();
    let text = logs.join("\n");
    assert!(
        text.contains("Compilation error"),
        "Output buffer captured child stderr"
    );
}

#[test]
fn test_output_buffer_bounded_capacity() {
    let buffer = OutputBuffer::new(5); // Bound to 5 lines
    for i in 0..10 {
        buffer.push_line(format!("Line {}", i));
    }

    let lines = buffer.drain_lines();
    assert_eq!(lines.len(), 5, "Buffer must not exceed bounded capacity");
    assert_eq!(lines[0], "Line 5");
    assert_eq!(lines[4], "Line 9");
}

#[test]
fn test_non_blocking_try_wait() {
    let mut supervisor = ProcessSupervisor::spawn(&[
        "sh".to_string(),
        "-c".to_string(),
        "sleep 0.1; exit 0".to_string(),
    ])
    .expect("spawn");

    // Immediately after spawn, process is still running
    let poll1 = supervisor.try_wait().expect("poll");
    assert!(poll1.is_none(), "Process should still be running");

    // Wait until process exits with a 3-second timeout to accommodate VM scheduling variance
    let start = std::time::Instant::now();
    let mut exited = false;
    while start.elapsed() < Duration::from_secs(3) {
        if let Some(status) = supervisor.try_wait().expect("poll") {
            assert!(status.success());
            exited = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(exited, "Process should have exited within timeout");
}

#[test]
fn test_disabled_auto_activate_bypasses_gameplay_completely() {
    let temp_dir = std::env::temp_dir().join("galerne_test_disable_auto");
    let _ = std::fs::create_dir_all(&temp_dir);
    std::env::set_var("GALERNE_DATA_DIR", &temp_dir);

    let save_manager = galerne::storage::manager::SaveManager::with_dir(&temp_dir);
    let mut state = galerne::storage::schema::SaveState::default();
    state.settings.auto_activate = false;
    save_manager.save(&state).expect("saved disabled state");

    let start = std::time::Instant::now();
    let exit_code = galerne::interceptor::execute_wrapped_task(&[
        "sh".to_string(),
        "-c".to_string(),
        "exit 42".to_string(),
    ])
    .expect("wrapped task");

    let elapsed = start.elapsed();
    assert_eq!(exit_code, 42, "Direct command exit code must be preserved");
    assert!(
        elapsed < Duration::from_millis(1000),
        "Must not wait through anti-flicker delay or open game view when disabled (took {:?})",
        elapsed
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
