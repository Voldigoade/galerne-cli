use crate::terminal::buffer::Buffer;
use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTier {
    TrueColor,
    Color256,
    Ansi16,
    Ascii,
}

impl ColorTier {
    pub fn from_env(colorterm: Option<&str>, term: Option<&str>, no_color: bool) -> Self {
        if no_color || term.is_some_and(|value| value.eq_ignore_ascii_case("dumb")) {
            return Self::Ascii;
        }
        if colorterm.is_some_and(|value| {
            value.eq_ignore_ascii_case("truecolor") || value.eq_ignore_ascii_case("24bit")
        }) {
            Self::TrueColor
        } else if term.is_some_and(|value| value.contains("256color")) {
            Self::Color256
        } else {
            Self::Ansi16
        }
    }

    pub fn from_environment() -> Self {
        let term = std::env::var("TERM").ok();
        if std::env::var_os("NO_COLOR").is_some()
            || term
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case("dumb"))
        {
            return Self::Ascii;
        }
        if let Ok(override_tier) = std::env::var("GALERNE_COLOR") {
            match override_tier.to_ascii_lowercase().as_str() {
                "truecolor" | "24bit" => return Self::TrueColor,
                "256" | "256color" => return Self::Color256,
                "16" | "ansi16" => return Self::Ansi16,
                "ascii" | "mono" => return Self::Ascii,
                _ => {}
            }
        }

        Self::from_env(
            std::env::var("COLORTERM").ok().as_deref(),
            term.as_deref(),
            false,
        )
    }

    pub fn map_color(self, color: Rgb) -> Color {
        terminal_color(color, self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelCanvas {
    width: u16,
    height: u16,
    pixels: Vec<Rgb>,
}

impl PixelCanvas {
    pub fn new(width: u16, height: u16, fill: Rgb) -> Self {
        Self {
            width,
            height,
            pixels: vec![fill; width as usize * height as usize],
        }
    }

    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.pixels[y as usize * self.width as usize + x as usize] = color;
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Rgb) {
        if width <= 0 || height <= 0 {
            return;
        }
        let x0 = x.max(0).min(self.width as i32);
        let y0 = y.max(0).min(self.height as i32);
        let x1 = x.saturating_add(width).min(self.width as i32);
        let y1 = y.saturating_add(height).min(self.height as i32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        for py in y0..y1 {
            let start = py as usize * self.width as usize + x0 as usize;
            let end = py as usize * self.width as usize + x1 as usize;
            self.pixels[start..end].fill(color);
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn to_buffer(&self, tier: ColorTier) -> Buffer {
        let mut output = Buffer::new(self.width, self.height.div_ceil(2));
        for y in 0..output.height() {
            for x in 0..self.width {
                let upper = self.pixel(x, y * 2);
                let lower = self.pixel(x, y * 2 + 1);
                if tier == ColorTier::Ascii {
                    let average = (u16::from(luminance(upper)) + u16::from(luminance(lower))) / 2;
                    let glyph = match average {
                        184..=255 => '#',
                        96..=183 => '+',
                        70..=95 => '.',
                        _ => ' ',
                    };
                    output.set_char(x, y, glyph, Color::Reset, Color::Reset);
                } else {
                    output.set_char(
                        x,
                        y,
                        '▀',
                        terminal_color(upper, tier),
                        terminal_color(lower, tier),
                    );
                }
            }
        }
        output
    }

    fn pixel(&self, x: u16, y: u16) -> Rgb {
        if x >= self.width || y >= self.height {
            return Rgb(0, 0, 0);
        }
        self.pixels[y as usize * self.width as usize + x as usize]
    }
}

fn luminance(Rgb(r, g, b): Rgb) -> u8 {
    ((u16::from(r) * 54 + u16::from(g) * 183 + u16::from(b) * 19) / 256) as u8
}

fn terminal_color(Rgb(r, g, b): Rgb, tier: ColorTier) -> Color {
    match tier {
        ColorTier::TrueColor => Color::Rgb { r, g, b },
        ColorTier::Color256 => {
            let red = nearest_cube(r);
            let green = nearest_cube(g);
            let blue = nearest_cube(b);
            Color::AnsiValue(16 + 36 * red + 6 * green + blue)
        }
        ColorTier::Ansi16 => nearest_ansi16(Rgb(r, g, b)),
        ColorTier::Ascii => Color::Reset,
    }
}

fn nearest_cube(channel: u8) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    LEVELS
        .iter()
        .enumerate()
        .min_by_key(|(_, level)| channel.abs_diff(**level))
        .map(|(index, _)| index as u8)
        .unwrap_or(0)
}

fn nearest_ansi16(color: Rgb) -> Color {
    const PALETTE: [(Color, Rgb); 16] = [
        (Color::Black, Rgb(0, 0, 0)),
        (Color::DarkRed, Rgb(128, 0, 0)),
        (Color::DarkGreen, Rgb(0, 128, 0)),
        (Color::DarkYellow, Rgb(128, 128, 0)),
        (Color::DarkBlue, Rgb(0, 0, 128)),
        (Color::DarkMagenta, Rgb(128, 0, 128)),
        (Color::DarkCyan, Rgb(0, 128, 128)),
        (Color::Grey, Rgb(192, 192, 192)),
        (Color::DarkGrey, Rgb(128, 128, 128)),
        (Color::Red, Rgb(255, 0, 0)),
        (Color::Green, Rgb(0, 255, 0)),
        (Color::Yellow, Rgb(255, 255, 0)),
        (Color::Blue, Rgb(0, 0, 255)),
        (Color::Magenta, Rgb(255, 0, 255)),
        (Color::Cyan, Rgb(0, 255, 255)),
        (Color::White, Rgb(255, 255, 255)),
    ];
    PALETTE
        .iter()
        .min_by_key(|(_, candidate)| color_distance(color, *candidate))
        .map(|(terminal, _)| *terminal)
        .unwrap_or(Color::Black)
}

fn color_distance(Rgb(r, g, b): Rgb, Rgb(pr, pg, pb): Rgb) -> u32 {
    let red = i32::from(r) - i32::from(pr);
    let green = i32::from(g) - i32::from(pg);
    let blue = i32::from(b) - i32::from(pb);
    (red * red + green * green + blue * blue) as u32
}

#[cfg(test)]
mod tests {
    use super::{ColorTier, PixelCanvas, Rgb};
    use crate::terminal::buffer::Color;

    #[test]
    fn pairs_two_authored_pixels_into_a_truecolor_half_block() {
        let mut canvas = PixelCanvas::new(1, 2, Rgb(0, 0, 0));
        canvas.set(0, 0, Rgb(12, 34, 56));
        canvas.set(0, 1, Rgb(78, 90, 123));

        let buffer = canvas.to_buffer(ColorTier::TrueColor);
        let cell = buffer.get(0, 0).unwrap();

        assert_eq!(cell.ch, '▀');
        assert_eq!(
            cell.fg,
            Color::Rgb {
                r: 12,
                g: 34,
                b: 56
            }
        );
        assert_eq!(
            cell.bg,
            Color::Rgb {
                r: 78,
                g: 90,
                b: 123
            }
        );
    }

    #[test]
    fn ascii_tier_keeps_pixel_shapes_visible_without_color() {
        let mut canvas = PixelCanvas::new(1, 2, Rgb(0, 0, 0));
        canvas.set(0, 0, Rgb(255, 255, 255));
        canvas.set(0, 1, Rgb(255, 255, 255));

        let buffer = canvas.to_buffer(ColorTier::Ascii);
        let cell = buffer.get(0, 0).unwrap();

        assert!(matches!(cell.ch, '#' | '+' | '.' | ' '));
        assert_eq!(cell.ch, '#');
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
    }

    #[test]
    fn ascii_tier_keeps_muted_sky_pixels_blank() {
        let canvas = PixelCanvas::new(1, 2, Rgb(38, 65, 67));
        let buffer = canvas.to_buffer(ColorTier::Ascii);

        assert_eq!(buffer.get(0, 0).unwrap().ch, ' ');
    }

    #[test]
    fn terminal_color_capabilities_select_the_highest_safe_tier() {
        assert_eq!(
            ColorTier::from_env(Some("truecolor"), Some("xterm-256color"), false),
            ColorTier::TrueColor
        );
        assert_eq!(
            ColorTier::from_env(None, Some("xterm-256color"), false),
            ColorTier::Color256
        );
        assert_eq!(
            ColorTier::from_env(None, Some("xterm"), false),
            ColorTier::Ansi16
        );
        assert_eq!(
            ColorTier::from_env(Some("truecolor"), Some("xterm-256color"), true),
            ColorTier::Ascii
        );
        assert_eq!(
            ColorTier::from_env(None, Some("dumb"), false),
            ColorTier::Ascii
        );
    }

    #[test]
    fn reduced_palettes_quantize_to_xterm_256_and_ansi_16_colors() {
        let mut canvas = PixelCanvas::new(1, 2, Rgb(0, 0, 0));
        canvas.set(0, 0, Rgb(255, 0, 0));
        canvas.set(0, 1, Rgb(0, 0, 255));

        let colors = canvas.to_buffer(ColorTier::Color256);
        assert_eq!(colors.get(0, 0).unwrap().fg, Color::AnsiValue(196));
        assert_eq!(colors.get(0, 0).unwrap().bg, Color::AnsiValue(21));
        let colors = canvas.to_buffer(ColorTier::Ansi16);
        assert_eq!(colors.get(0, 0).unwrap().fg, Color::Red);
        assert_eq!(colors.get(0, 0).unwrap().bg, Color::Blue);
    }

    #[test]
    fn clipped_rectangles_do_not_write_outside_the_pixel_canvas() {
        let mut canvas = PixelCanvas::new(2, 2, Rgb(0, 0, 0));
        canvas.fill_rect(-1, -1, 2, 2, Rgb(255, 255, 255));
        canvas.fill_rect(3, 0, 4, 1, Rgb(255, 0, 0));

        let frame = canvas.to_buffer(ColorTier::TrueColor);
        assert_eq!(
            frame.get(0, 0).unwrap().fg,
            Color::Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(frame.get(1, 0).unwrap().fg, Color::Rgb { r: 0, g: 0, b: 0 });
    }
}
