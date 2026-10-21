use std::thread;
use std::time::Duration;

use galerne::interceptor::process::ProcessSupervisor;
use galerne::interceptor::signal;

#[test]
fn test_child_continues_running_and_streams_output() {
    let args = if cfg!(windows) {
        vec![
            "cmd".to_string(),
            "/c".to_string(),
            "echo Line 1 & ping 127.0.0.1 -n 2 >nul & echo Line 2 & ping 127.0.0.1 -n 2 >nul & echo Line 3".to_string(),
        ]
    } else {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Line 1'; sleep 0.15; echo 'Line 2'; sleep 0.15; echo 'Line 3'".to_string(),
        ]
    };

    let mut supervisor = ProcessSupervisor::spawn(&args).expect("spawned supervisor");

    // Wait slightly to let line 1 arrive
    thread::sleep(Duration::from_millis(50));
    let initial_lines = supervisor.output.drain_lines();
    assert!(
        initial_lines.iter().any(|l| l.contains("Line 1")),
        "Line 1 should arrive while process is running"
    );

    // Assert process is still running (has not exited yet)
    let poll = supervisor.try_wait().expect("try_wait");
    assert!(poll.is_none(), "Process must still be running");

    // Let process run to completion
    let status = supervisor.wait().expect("wait");
    assert!(status.success());

    let final_lines = supervisor.output.drain_lines();
    assert!(final_lines.iter().any(|l| l.contains("Line 2")));
    assert!(final_lines.iter().any(|l| l.contains("Line 3")));
}

#[test]
fn test_child_interruption_via_signal() {
    let args = if cfg!(windows) {
        vec![
            "cmd".to_string(),
            "/c".to_string(),
            "echo Started & ping 127.0.0.1 -n 15 >nul & echo Should not reach here".to_string(),
        ]
    } else {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Started'; sleep 10; echo 'Should not reach here'".to_string(),
        ]
    };

    let mut supervisor = ProcessSupervisor::spawn(&args).expect("spawned supervisor");

    // Verify it is alive
    thread::sleep(Duration::from_millis(100));
    assert!(supervisor.try_wait().expect("poll").is_none());

    // Send interrupt signal (second Ctrl+C in task view)
    supervisor.send_interrupt().expect("send_interrupt");

    let status = supervisor.wait().expect("wait");
    assert!(
        !status.success(),
        "Interrupted process must report non-zero / terminated status"
    );

    let logs = supervisor.output.drain_lines();
    assert!(logs.iter().any(|l| l.contains("Started")));
    assert!(!logs.iter().any(|l| l.contains("Should not reach here")));
}

#[test]
fn test_signal_state_machine() {
    signal::setup_handler();
    assert!(!signal::is_interrupted());

    // Reset works idempotently
    signal::reset();
    assert!(!signal::is_interrupted());
}
