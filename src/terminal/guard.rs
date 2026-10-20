//! RAII guard for terminal raw mode, alternate screen, and panic cleanup.

use std::io::{self, stdout, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

static TERMINAL_IN_RAW_MODE: AtomicBool = AtomicBool::new(false);

/// RAII Drop guard ensuring the terminal is always cleanly restored
/// under any exit condition (normal, user cancel, signal, or panic).
pub struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    /// Enters raw mode, hides cursor, and switches to the alternate screen.
    /// Also installs a panic hook to guarantee terminal restoration before backtraces.
    pub fn enter() -> io::Result<Self> {
        // Install panic hook once before entering raw mode
        static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
        if !HOOK_INSTALLED.swap(true, Ordering::SeqCst) {
            let default_hook = panic::take_hook();
            panic::set_hook(Box::new(move |panic_info| {
                let _ = Self::restore_terminal();
                default_hook(panic_info);
            }));
        }

        crossterm::terminal::enable_raw_mode()?;
        let mut out = stdout();
        crossterm::execute!(
            out,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;
        out.flush()?;

        TERMINAL_IN_RAW_MODE.store(true, Ordering::SeqCst);
        Ok(Self { active: true })
    }

    /// Explicitly restores the terminal to cooked mode, normal screen, and visible cursor.
    pub fn restore_terminal() -> io::Result<()> {
        if TERMINAL_IN_RAW_MODE.swap(false, Ordering::SeqCst) {
            let mut out = stdout();
            let _ = crossterm::execute!(
                out,
                crossterm::cursor::Show,
                crossterm::terminal::LeaveAlternateScreen
            );
            let _ = out.flush();
            let _ = crossterm::terminal::disable_raw_mode();
        }
        Ok(())
    }

    /// Disarms the guard so drop will not restore the terminal again.
    pub fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = Self::restore_terminal();
            self.active = false;
        }
    }
}
