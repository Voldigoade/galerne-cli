---
name: terminal-runtime-engineering
description: Use when implementing native terminal interfaces, command execution wrappers, cross-platform terminal control, TUI event loops, or low-overhead rendering on Windows and Linux.
---

# Terminal Runtime Engineering

Guide for creating robust, high-performance native terminal applications and command wrappers on Windows and Linux. Reliability and clean terminal restoration are non-negotiable prerequisites.

## Absolute Invariant: Guaranteed Terminal Restoration

The terminal state must **always** be cleanly restored under every possible exit path:
1. Normal completion
2. Clean user exit (`q`, `Esc`)
3. Signals / cancellation (`Ctrl+C`, `SIGINT`, `SIGTERM`, `SIGHUP`)
4. Unhandled panics (`std::panic::set_hook`)
5. Child process execution failures or I/O errors

### RAII Guard Pattern & Panic Hook

Wrap raw terminal mode and alternate screen state in an RAII drop guard and install a panic hook before enabling raw mode:

```rust
pub struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    pub fn enter() -> Result<Self, std::io::Error> {
        // 1. Install panic hook to restore terminal before printing backtrace
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = Self::restore_terminal();
            default_hook(info);
        }));

        // 2. Enable raw mode & enter alternate screen
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;

        Ok(Self { active: true })
    }

    pub fn restore_terminal() -> Result<(), std::io::Error> {
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        let _ = crossterm::terminal::disable_raw_mode();
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = Self::restore_terminal();
        }
    }
}
```

## Signal Handling & Ctrl+C

- Catch `Ctrl+C` asynchronously or via signal hooks (e.g. `ctrlc` crate or platform signal handlers).
- Set an atomic shutdown flag (`AtomicBool`) so the event loop cleanly breaks and drops guards.
- Never terminate abruptly with `std::process::exit` without dropping RAII guards or manually restoring the terminal.

## Platform Terminal Behavior & Capabilities

- **TTY vs Pipe Detection**:
  - Check whether stdin and stdout are interactive terminals using `std::io::IsTerminal` (`std::io::stdout().is_terminal()`).
  - If output is piped, do not emit alternate screen escapes or raw control sequences; degrade gracefully to plain line-buffered stream output.
- **Windows Terminal vs Legacy Conhost**:
  - Modern Windows Terminal supports standard ANSI/VT sequences.
  - Legacy Windows `conhost.exe` requires explicitly enabling virtual terminal processing (`ENABLE_VIRTUAL_TERMINAL_PROCESSING` via Windows console API).
  - Verify terminal behavior across PowerShell, CMD, Bash, and Zsh.
- **Graceful Capability Degradation**:
  - Query terminal color depth (TrueColor -> 256 Color -> ANSI 16 Color -> Plain Monochrome).
  - Provide ASCII glyph fallbacks if UTF-8 box-drawing characters are unsupported or cause width ambiguities on certain Windows consoles.

## Event Loop & Input Handling

- **Non-blocking Polling**: Use `crossterm::event::poll(Duration)` to avoid pegging the CPU in a tight spin loop while keeping input latency low.
- **Keyboard Events**: Handle standard key codes, modifier keys, and terminal escape sequences. Account for terminal quirks (e.g., escape delay on POSIX).
- **Resize Handling**:
  - Handle `crossterm::event::Event::Resize(cols, rows)` or `SIGWINCH`.
  - Recalculate view layouts and trigger buffer reallocation on size change.
  - Protect against zero-sized or tiny terminal windows (`cols < 20` or `rows < 5`) without panicking.

## Low-Overhead Rendering & Double Buffering

- **Never Clear the Whole Screen Every Frame**: Full-screen redraws (`Clear(All)`) cause visible flickering, high CPU usage, and bandwidth saturation over SSH or slow terminal emulators.
- **Double Buffering (Diff Rendering)**:
  1. Maintain two buffers: `current_frame` and `previous_frame` (2D grid of `Cell { ch: char, style: Style }`).
  2. Each frame, render the logical state into `current_frame`.
  3. Compare `current_frame` against `previous_frame`.
  4. Only position cursor and write output for cells where `current[x][y] != previous[x][y]`.
  5. Swap or copy `current_frame` into `previous_frame`.
- **Buffered I/O**: Wrap stdout writes in `std::io::BufWriter` and flush once per frame.

## Child Process Execution & Command Wrapping

When wrapping or executing external commands:
- **Child Lifetime**: Wait for child process exit (`child.wait()`) to prevent zombie processes.
- **Exit Code Preservation**: Propagate child exit codes faithfully.
- **Environment & CWD**: Explicitly pass through or snapshot environment variables and current working directory.
- **Stdout/Stderr Handling**: Ensure child stderr does not corrupt active TUI layouts. If child command is interactive, temporarily suspend raw mode / leave alternate screen, run child, then re-enter raw mode.
- **Shell Quoting Hazards**: Avoid invoking external commands through shell strings (`sh -c` or `cmd.exe /c`) whenever direct arguments `Command::new(binary).args(...)` can be used.
