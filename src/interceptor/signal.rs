//! Cross-platform native signal handler for the two-tier Ctrl+C invariant.
//! Operates without heavy external crates, using libc on Unix and kernel32 on Windows.

use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn handle_sigint(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

#[cfg(windows)]
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> i32 {
    if ctrl_type == 0 {
        // CTRL_C_EVENT
        INTERRUPTED.store(true, Ordering::SeqCst);
        1 // Handled, prevent immediate exit so Galerne can cancel child process
    } else {
        0
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleCtrlHandler(
        handler_routine: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
}

/// Registers the platform Ctrl+C handler.
pub fn setup_handler() {
    INTERRUPTED.store(false, Ordering::SeqCst);

    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, handle_sigint as *const () as usize);
    }

    #[cfg(windows)]
    unsafe {
        SetConsoleCtrlHandler(Some(console_ctrl_handler), 1);
    }
}

/// Returns true if a Ctrl+C signal was received in cooked mode.
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Resets the interrupt flag to false.
pub fn reset() {
    INTERRUPTED.store(false, Ordering::SeqCst);
}
