//! Cell and buffer definitions for low-overhead double-buffered terminal rendering.

pub use crossterm::style::Color;

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
        Self {
            ch,
            fg: style.fg,
            bg: style.bg,
            bold: style.bold,
            dim: style.dim,
            invert: style.invert,
        }
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
        if let Some(c) = self.get_mut(x, y) {
            *c = cell;
        }
    }

    /// Sets a character with foreground and background colors at `(x, y)`.
    #[inline]
    pub fn set_char(&mut self, x: u16, y: u16, ch: char, fg: Color, bg: Color) {
        if let Some(c) = self.get_mut(x, y) {
            c.ch = ch;
            c.fg = fg;
            c.bg = bg;
            c.bold = false;
            c.dim = false;
            c.invert = false;
        }
    }

    /// Sets a formatted character with style at `(x, y)`.
    #[inline]
    pub fn set_styled(&mut self, x: u16, y: u16, ch: char, style: Style) {
        if let Some(c) = self.get_mut(x, y) {
            c.ch = ch;
            c.fg = style.fg;
            c.bg = style.bg;
            c.bold = style.bold;
            c.dim = style.dim;
            c.invert = style.invert;
        }
    }

    /// Draws a string horizontally starting at `(x, y)`, clipping at the boundary.
    pub fn draw_str(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        for (cur_x, ch) in (x..self.width).zip(s.chars()) {
            self.set_char(cur_x, y, ch, fg, bg);
        }
    }

    /// Draws a styled string horizontally starting at `(x, y)`.
    pub fn draw_str_styled(&mut self, x: u16, y: u16, s: &str, style: Style) {
        for (cur_x, ch) in (x..self.width).zip(s.chars()) {
            self.set_styled(cur_x, y, ch, style);
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
