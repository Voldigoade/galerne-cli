//! Terminal rendering engine implementing double-buffered diff emission.

use std::io::{self, stdout, BufWriter, Stdout, Write};
use std::time::Duration;

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal;
use crossterm::QueueableCommand;

use super::buffer::Buffer;

pub const MIN_TERMINAL_WIDTH: u16 = 60;
pub const MIN_TERMINAL_HEIGHT: u16 = 20;

/// Rendering engine utilizing double-buffering to minimize ANSI output.
pub struct Engine {
    current: Buffer,
    previous: Buffer,
    writer: BufWriter<Stdout>,
    current_fg: Color,
    current_bg: Color,
}

impl Engine {
    /// Creates a new engine initialized to the current terminal dimensions.
    pub fn new() -> io::Result<Self> {
        let (w, h) = terminal::size()?;
        let writer = BufWriter::with_capacity(16 * 1024, stdout());
        Ok(Self {
            current: Buffer::new(w, h),
            previous: Buffer::new(w, h),
            writer,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
        })
    }

    /// Returns a mutable reference to the current working buffer.
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.current
    }

    /// Returns an immutable reference to the current working buffer.
    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.current
    }

    /// Returns current buffer width.
    #[inline]
    pub fn width(&self) -> u16 {
        self.current.width()
    }

    /// Returns current buffer height.
    #[inline]
    pub fn height(&self) -> u16 {
        self.current.height()
    }

    /// Checks if current terminal dimensions are too small for gameplay.
    pub fn is_too_small(&self) -> bool {
        self.current.width() < MIN_TERMINAL_WIDTH || self.current.height() < MIN_TERMINAL_HEIGHT
    }

    /// Checks for resize events and updates buffer dimensions.
    pub fn check_resize(&mut self) -> io::Result<bool> {
        let (w, h) = terminal::size()?;
        if w != self.current.width() || h != self.current.height() {
            self.current.resize(w, h);
            self.previous.resize(w, h);
            // Force redraw of entire screen on next flush
            self.previous.clear();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Renders a centered message if the terminal is below minimum viable dimensions.
    pub fn render_too_small_notice(&mut self) {
        self.current.clear();
        let w = self.current.width();
        let h = self.current.height();

        let msg1 = "TERMINAL TOO SMALL";
        let msg2 = format!(
            "Current: {}x{} — Required: {}x{}",
            w, h, MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT
        );
        let msg3 = "Please expand your terminal window.";

        let mid_y = h / 2;
        if mid_y > 1 {
            let x1 = w.saturating_sub(msg1.len() as u16) / 2;
            let x2 = w.saturating_sub(msg2.len() as u16) / 2;
            let x3 = w.saturating_sub(msg3.len() as u16) / 2;

            self.current
                .draw_str(x1, mid_y.saturating_sub(1), msg1, Color::Red, Color::Reset);
            self.current
                .draw_str(x2, mid_y, &msg2, Color::Yellow, Color::Reset);
            self.current.draw_str(
                x3,
                mid_y.saturating_add(1),
                msg3,
                Color::DarkGrey,
                Color::Reset,
            );
        }
    }

    /// Compares `current` against `previous`, emits required ANSI sequences,
    /// flushes to standard output, and copies `current` into `previous`.
    pub fn flush(&mut self) -> io::Result<()> {
        let mut last_x = u16::MAX;
        let mut last_y = u16::MAX;

        let writer = &mut self.writer;
        let cur_fg = &mut self.current_fg;
        let cur_bg = &mut self.current_bg;

        for (x, y, cell) in self.current.diff(&self.previous) {
            // Reposition cursor only if not contiguous
            if x != last_x || y != last_y {
                writer.queue(MoveTo(x, y))?;
            }

            if cell.fg != *cur_fg {
                writer.queue(SetForegroundColor(cell.fg))?;
                *cur_fg = cell.fg;
            }
            if cell.bg != *cur_bg {
                writer.queue(SetBackgroundColor(cell.bg))?;
                *cur_bg = cell.bg;
            }

            writer.queue(Print(cell.ch))?;

            last_x = x.saturating_add(1);
            last_y = y;
        }

        // Reset styling and flush buffered bytes to stdout
        if *cur_fg != Color::Reset || *cur_bg != Color::Reset {
            writer.queue(ResetColor)?;
            *cur_fg = Color::Reset;
            *cur_bg = Color::Reset;
        }
        writer.flush()?;

        // Copy current frame to previous
        self.previous.copy_from(&self.current);
        Ok(())
    }

    /// Polls for keyboard/terminal events up to `timeout`.
    pub fn poll_event(timeout: Duration) -> io::Result<Option<Event>> {
        if event::poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}
