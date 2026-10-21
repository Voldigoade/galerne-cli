//! Multi-layer parallax scenery for the scrolling world.

use serde::{Deserialize, Serialize};

use crate::terminal::buffer::{Buffer, Color};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParallaxEngine {
    pub sky_offset: f32,
    pub far_offset: f32,
    pub mid_offset: f32,
    pub ground_offset: f32,
    pub fg_offset: f32,
}

impl Default for ParallaxEngine {
    fn default() -> Self {
        Self {
            sky_offset: 0.0,
            far_offset: 0.0,
            mid_offset: 0.0,
            ground_offset: 0.0,
            fg_offset: 0.0,
        }
    }
}

impl ParallaxEngine {
    pub fn update(&mut self, world_dx: f32) {
        self.sky_offset += world_dx * 0.06;
        self.far_offset += world_dx * 0.18;
        self.mid_offset += world_dx * 0.45;
        self.ground_offset += world_dx * 1.00;
        self.fg_offset += world_dx * 1.35;
    }

    /// Renders multi-layer backdrop onto terminal buffer with dynamic vertical scaling.
    pub fn render_backdrop(&self, buf: &mut Buffer, ground_y: usize, relay_restored: bool) {
        let width = buf.width() as usize;
        let height = buf.height() as usize;
        if height < 10 || width < 40 || ground_y < 5 {
            return;
        }

        // Proportional vertical partition based on available sky height above ground_y
        let sky_rows = (ground_y * 22 / 100).clamp(2, 8);
        let far_rows = (ground_y * 38 / 100).clamp(3, 16);

        let sky_patterns = [
            b"   ~--~       ' . '    ~~~~~        '   .     ~--~       ~~~~     .   '    ~--~       ",
            b" .     ` - - .       '    ~==~      .        '     ` - - ~     .     '     ` - - .    ",
            b"     '        .    ~==~        .    '       .         ~==~         .    '        .    ",
        ];
        let sky_shift = (self.sky_offset as usize) % sky_patterns[0].len();
        for (idx, pattern) in sky_patterns.iter().cycle().take(sky_rows).enumerate() {
            let y = 1 + idx;
            if y >= ground_y {
                break;
            }
            let row_offset = (sky_shift + idx * 17) % pattern.len();
            for x in 0..width {
                let ch = pattern[(row_offset + x) % pattern.len()] as char;
                if ch != ' ' {
                    buf.set_char(x as u16, y as u16, ch, Color::DarkGrey, Color::Black);
                }
            }
        }

        let far_start = 1 + sky_rows;
        let far_end = (far_start + far_rows).min(ground_y.saturating_sub(2));
        if far_end > far_start {
            let tower_top = b"       /===\\                                   /===\\                                ";
            let tower_mid = b"      / | | \\         .---.                   / | | \\         .---.                 ";
            let tower_low = b"     /  | |  \\_______/     \\_________________/  | |  \\_______/     \\________________";
            let arch_str = b"____|===| |===|_____/       \\_______________|===| |===|_____/       \\_______________";
            let pier_body = b"    |   | |   |                             |   | |   |                             ";

            let far_shift = (self.far_offset as usize) % tower_top.len();
            let far_h = far_end - far_start;

            for (i, y) in (far_start..far_end).enumerate() {
                let pattern: &[u8] = if i == 0 {
                    tower_top
                } else if i == 1 {
                    tower_mid
                } else if i + 2 == far_h {
                    tower_low
                } else if i + 1 == far_h {
                    arch_str
                } else {
                    pier_body
                };

                let row_offset = far_shift % pattern.len();
                for x in 0..width {
                    let ch = pattern[(row_offset + x) % pattern.len()] as char;
                    if ch != ' ' {
                        let color = if relay_restored && (ch == '|' || ch == '=') {
                            Color::DarkCyan
                        } else {
                            Color::DarkGrey
                        };
                        buf.set_char(x as u16, y as u16, ch, color, Color::Black);
                    }
                }
            }
        }

        let mid_start = far_end;
        let mid_end = ground_y.saturating_sub(1);
        if mid_end > mid_start {
            let wire_pat =
                b"---.___________.---.___________.---.___________.---.___________.---.____________";
            let shack_top = b"   |   /###\\   |   |           |   |   /###\\   |   |           |   |   /###\\   |    ";
            let shack_bot = b"   |  |[.] [ ] |   |           |   |  |[.] [ ] |   |           |   |  |[.] [ ] |    ";
            let pipe_pat = b"===+===========+===+===========+===+===========+===+===========+===+===========+====";

            let mid_shift = (self.mid_offset as usize) % wire_pat.len();
            let mid_h = mid_end - mid_start;

            for (i, y) in (mid_start..mid_end).enumerate() {
                let pattern: &[u8] = if i == 0 {
                    wire_pat
                } else if i + 2 == mid_h {
                    shack_top
                } else if i + 1 == mid_h {
                    shack_bot
                } else {
                    pipe_pat
                };

                let row_offset = mid_shift % pattern.len();
                for x in 0..width {
                    let ch = pattern[(row_offset + x) % pattern.len()] as char;
                    if ch != ' ' {
                        let color = match ch {
                            '#' => Color::DarkYellow,
                            '.' if pattern == shack_bot => Color::Yellow,
                            '=' | '+' if relay_restored => Color::Cyan,
                            _ => Color::DarkGrey,
                        };
                        buf.set_char(x as u16, y as u16, ch, color, Color::Black);
                    }
                }
            }
        }

        if ground_y < height {
            let rail_shift = (self.ground_offset as usize) % 4;
            for x in 0..width {
                let rail_color = if relay_restored {
                    Color::Cyan
                } else {
                    Color::White
                };
                buf.set_char(x as u16, ground_y as u16, '=', rail_color, Color::Black);

                if ground_y + 1 < height {
                    let sleeper_ch = if (x + rail_shift).is_multiple_of(4) {
                        '|'
                    } else {
                        '-'
                    };
                    buf.set_char(
                        x as u16,
                        (ground_y + 1) as u16,
                        sleeper_ch,
                        Color::DarkGrey,
                        Color::Black,
                    );
                }

                if ground_y + 2 < height {
                    let truss_ch = if (x + rail_shift) % 4 < 2 { '/' } else { '\\' };
                    buf.set_char(
                        x as u16,
                        (ground_y + 2) as u16,
                        truss_ch,
                        Color::DarkGrey,
                        Color::Black,
                    );
                }

                if ground_y + 3 < height {
                    buf.set_char(
                        x as u16,
                        (ground_y + 3) as u16,
                        '=',
                        Color::DarkGrey,
                        Color::Black,
                    );
                }

                if ground_y + 4 < height {
                    let pier_col = (x + rail_shift) % 24;
                    if pier_col == 0 || pier_col == 5 {
                        for py in (ground_y + 4)..height {
                            buf.set_char(x as u16, py as u16, '|', Color::DarkGrey, Color::Black);
                        }
                    }
                }
            }
        }
    }
}
