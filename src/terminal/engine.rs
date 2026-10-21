//! Terminal rendering engine implementing double-buffered diff emission.

use std::io::{self, stdout, BufWriter, Stdout, Write};
use std::time::Duration;

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal;
use crossterm::QueueableCommand;
use unicode_width::UnicodeWidthStr;

use super::buffer::Buffer;
use crate::ui::i18n::{format_message, t, Locale};

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
    pub fn render_too_small_notice(&mut self, locale: Locale) {
        draw_too_small_notice(&mut self.current, locale);
    }

    /// Compares `current` against `previous`, emits required ANSI sequences,
    /// flushes to standard output, and copies `current` into `previous`.
    pub fn flush(&mut self) -> io::Result<()> {
        emit_buffer_diff(
            &mut self.writer,
            &self.current,
            &self.previous,
            &mut self.current_fg,
            &mut self.current_bg,
        )?;
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

fn emit_buffer_diff<W: Write>(
    writer: &mut W,
    current: &Buffer,
    previous: &Buffer,
    current_fg: &mut Color,
    current_bg: &mut Color,
) -> io::Result<()> {
    let mut last_x = u16::MAX;
    let mut last_y = u16::MAX;

    for (x, y, cell) in current.diff(previous) {
        if cell.width == 0 {
            continue;
        }

        if x != last_x || y != last_y {
            writer.queue(MoveTo(x, y))?;
        }

        if cell.fg != *current_fg {
            writer.queue(SetForegroundColor(cell.fg))?;
            *current_fg = cell.fg;
        }
        if cell.bg != *current_bg {
            writer.queue(SetBackgroundColor(cell.bg))?;
            *current_bg = cell.bg;
        }

        writer.queue(Print(cell.ch))?;

        last_x = x.saturating_add(cell.width as u16);
        last_y = y;
    }

    if *current_fg != Color::Reset || *current_bg != Color::Reset {
        writer.queue(ResetColor)?;
        *current_fg = Color::Reset;
        *current_bg = Color::Reset;
    }
    writer.flush()
}

fn draw_too_small_notice(buffer: &mut Buffer, locale: Locale) {
    buffer.clear();
    let width = buffer.width();
    let height = buffer.height();
    let heading = t("terminal_too_small", locale);
    let dimensions = format_message(
        "terminal_size_required",
        locale,
        &[
            ("width", width.to_string()),
            ("height", height.to_string()),
            ("min_width", MIN_TERMINAL_WIDTH.to_string()),
            ("min_height", MIN_TERMINAL_HEIGHT.to_string()),
        ],
    );
    let instruction = t("terminal_expand_window", locale);

    let mid_y = height / 2;
    if mid_y <= 1 {
        return;
    }

    draw_centered(buffer, mid_y - 1, heading, Color::Red);
    draw_centered(buffer, mid_y, &dimensions, Color::Yellow);
    draw_centered(buffer, mid_y + 1, instruction, Color::DarkGrey);
}

fn draw_centered(buffer: &mut Buffer, y: u16, text: &str, color: Color) {
    let text_width = text.width().min(u16::MAX as usize) as u16;
    let x = buffer.width().saturating_sub(text_width) / 2;
    buffer.draw_str(x, y, text, color, Color::Reset);
}

#[cfg(test)]
mod tests {
    use super::draw_too_small_notice;
    use super::emit_buffer_diff;
    use crate::terminal::buffer::Buffer;
    use crate::terminal::buffer::Color;
    use crate::ui::i18n::Locale;

    #[test]
    fn terminal_size_notice_uses_selected_locale() {
        for (locale, expected) in [
            (
                Locale::En,
                [
                    "TERMINAL TOO SMALL",
                    "Current: 80x24 — minimum: 60x20",
                    "Please expand your terminal window.",
                ],
            ),
            (
                Locale::Fr,
                [
                    "TERMINAL TROP PETIT",
                    "Actuel : 80x24 — minimum : 60x20",
                    "Agrandissez la fenêtre du terminal.",
                ],
            ),
            (
                Locale::Es,
                [
                    "TERMINAL DEMASIADO PEQUEÑA",
                    "Actual: 80x24 — mínimo: 60x20",
                    "Aumenta el tamaño de la ventana del terminal.",
                ],
            ),
            (
                Locale::ZhCn,
                [
                    "终端窗口太小",
                    "当前：80x24 — 最低：60x20",
                    "请扩大终端窗口。",
                ],
            ),
        ] {
            let mut buffer = Buffer::new(80, 24);
            draw_too_small_notice(&mut buffer, locale);
            let rendered = (0..buffer.height())
                .map(|y| {
                    (0..buffer.width())
                        .filter_map(|x| {
                            let cell = buffer.get(x, y).unwrap();
                            (cell.width != 0).then_some(cell.ch)
                        })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");

            for text in expected {
                assert!(rendered.contains(text), "Missing {text:?} for {locale:?}");
            }
            if locale == Locale::ZhCn {
                assert_eq!(buffer.get(34, 11).unwrap().ch, '终');
            }
        }
    }

    #[test]
    fn diff_output_does_not_reposition_after_a_wide_glyph() {
        let previous = Buffer::new(8, 1);
        let mut current = Buffer::new(8, 1);
        current.draw_str(0, 0, "中A", Color::White, Color::Reset);
        let mut output = Vec::new();
        let mut foreground = Color::Reset;
        let mut background = Color::Reset;

        emit_buffer_diff(
            &mut output,
            &current,
            &previous,
            &mut foreground,
            &mut background,
        )
        .unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("中A"));
        assert!(!output.contains("\u{1b}[1;3H"));
    }
}
