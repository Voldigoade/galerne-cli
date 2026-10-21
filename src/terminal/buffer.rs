//! Cell and buffer definitions for low-overhead double-buffered terminal rendering.

pub use crossterm::style::Color;
use unicode_width::UnicodeWidthChar;

/// Visual styling attributes for a terminal cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub invert: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            dim: false,
            invert: false,
        }
    }
}

impl Style {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self {
            fg,
            bg,
            bold: false,
            dim: false,
            invert: false,
        }
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }
}

/// Visual attributes for a single terminal cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    /// Terminal columns occupied; zero marks a continuation cell for a wide glyph.
    pub width: u8,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub invert: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            width: 1,
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            dim: false,
            invert: false,
        }
    }
}

impl Cell {
    pub fn new(ch: char, style: Style) -> Self {
        let (ch, width) = display_glyph(ch);
        make_cell(ch, width, style)
    }
}

/// A 2D grid of terminal cells representing a single frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    /// Creates a new buffer initialized with blank cells.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![Cell::default(); size],
        }
    }

    /// Returns the buffer width.
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Returns the buffer height.
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns a reference to the cell at `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.cells.get(idx)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the cell at `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    /// Sets the cell at `(x, y)` if within bounds.
    #[inline]
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        self.set_glyph(
            x,
            y,
            cell.ch,
            Style {
                fg: cell.fg,
                bg: cell.bg,
                bold: cell.bold,
                dim: cell.dim,
                invert: cell.invert,
            },
        );
    }

    /// Sets a character with foreground and background colors at `(x, y)`.
    #[inline]
    pub fn set_char(&mut self, x: u16, y: u16, ch: char, fg: Color, bg: Color) {
        self.set_glyph(x, y, ch, Style::new(fg, bg));
    }

    /// Sets a formatted character with style at `(x, y)`.
    #[inline]
    pub fn set_styled(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set_glyph(x, y, ch, style);
    }

    /// Draws a string horizontally starting at `(x, y)`, clipping at the boundary.
    pub fn draw_str(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        self.draw_text(x, y, s, Style::new(fg, bg));
    }

    /// Draws a styled string horizontally starting at `(x, y)`.
    pub fn draw_str_styled(&mut self, x: u16, y: u16, s: &str, style: Style) {
        self.draw_text(x, y, s, style);
    }

    fn draw_text(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut column = x;
        for ch in text.chars() {
            let (ch, width) = display_glyph(ch);
            if width == 0 {
                continue;
            }
            if column.saturating_add(width as u16) > self.width {
                break;
            }
            self.set_glyph(column, y, ch, style);
            column += width as u16;
        }
    }

    fn set_glyph(&mut self, x: u16, y: u16, ch: char, style: Style) {
        let (ch, width) = display_glyph(ch);
        if width == 0 || x.saturating_add(width as u16) > self.width || y >= self.height {
            return;
        }

        if self.get(x, y).is_some_and(|cell| cell.width == 1) && width == 1 {
            if let Some(cell) = self.get_mut(x, y) {
                *cell = make_cell(ch, width, style);
            }
            return;
        }

        self.clear_glyph_at(x, y);
        for offset in 1..width {
            self.clear_glyph_at(x + offset as u16, y);
        }

        if let Some(cell) = self.get_mut(x, y) {
            *cell = make_cell(ch, width, style);
        }

        for offset in 1..width {
            if let Some(cell) = self.get_mut(x + offset as u16, y) {
                *cell = make_cell(' ', 0, style);
            }
        }
    }

    fn clear_glyph_at(&mut self, x: u16, y: u16) {
        let Some(cell) = self.get(x, y).copied() else {
            return;
        };

        let mut lead_x = x;
        if cell.width == 0 {
            for offset in 1..=2u16 {
                let Some(candidate_x) = x.checked_sub(offset) else {
                    continue;
                };
                if self
                    .get(candidate_x, y)
                    .is_some_and(|candidate| candidate.width > offset as u8)
                {
                    lead_x = candidate_x;
                    break;
                }
            }
        }

        let width = self
            .get(lead_x, y)
            .map(|lead| lead.width.max(1))
            .unwrap_or(1);
        for offset in 0..width {
            if let Some(cell) = self.get_mut(lead_x + offset as u16, y) {
                *cell = Cell::default();
            }
        }
    }

    /// Clears the buffer to blank default cells.
    pub fn clear(&mut self) {
        let default = Cell::default();
        self.cells.fill(default);
    }

    /// Resizes the buffer to new dimensions.
    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.cells.clear();
        self.cells.resize(size, Cell::default());
    }

    /// Clones contents of `other` into `self`, resizing if necessary.
    pub fn copy_from(&mut self, other: &Buffer) {
        if self.width != other.width || self.height != other.height {
            self.resize(other.width, other.height);
        }
        self.cells.clone_from(&other.cells);
    }

    /// Computes differences between `self` (current frame) and `other` (previous frame).
    /// Returns an iterator yielding `(x, y, &Cell)` for each mutated cell.
    pub fn diff<'a>(&'a self, other: &'a Buffer) -> impl Iterator<Item = (u16, u16, &'a Cell)> {
        let min_w = self.width.min(other.width);
        let min_h = self.height.min(other.height);

        (0..self.height).flat_map(move |y| {
            (0..self.width).filter_map(move |x| {
                let current = self.get(x, y)?;
                if x < min_w && y < min_h {
                    if let Some(prev) = other.get(x, y) {
                        if current == prev {
                            return None;
                        }
                    }
                }
                Some((x, y, current))
            })
        })
    }
}

fn display_glyph(ch: char) -> (char, u8) {
    if ch.is_ascii() {
        return (ch, 1);
    }

    match UnicodeWidthChar::width(ch) {
        Some(0) => (' ', 0),
        Some(1) => (ch, 1),
        Some(2) => (ch, 2),
        Some(_) | None => ('�', 1),
    }
}

fn make_cell(ch: char, width: u8, style: Style) -> Cell {
    Cell {
        ch,
        width,
        fg: style.fg,
        bg: style.bg,
        bold: style.bold,
        dim: style.dim,
        invert: style.invert,
    }
}
