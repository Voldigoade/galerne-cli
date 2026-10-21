//! Multi-layer parallax scenery engine creating visual depth with zero allocations.

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
        self.sky_offset += world_dx * 0.08;
        self.far_offset += world_dx * 0.22;
        self.mid_offset += world_dx * 0.48;
        self.ground_offset += world_dx * 1.00;
        self.fg_offset += world_dx * 1.35;
    }

    /// Renders multi-layer backdrop onto terminal buffer.
    pub fn render_backdrop(&self, buf: &mut Buffer, ground_y: usize) {
        let width = buf.width() as usize;
        let height = buf.height() as usize;
        if height < 10 || width < 40 {
            return;
        }

        // 1. Sky & Wind Streaks (Top 2 to 4 rows)
        let sky_pattern =
            "   ~--~    ' . '   ~~~~~        '   .    ~--~      ~~~~    .   '   ~--~        ";
        let sky_shift = (self.sky_offset as usize) % sky_pattern.len();
        for y in 1..=3.min(ground_y.saturating_sub(10)) {
            let row_offset = (sky_shift + y * 13) % sky_pattern.len();
            for x in 0..width {
                let char_idx = (row_offset + x) % sky_pattern.len();
                let ch = sky_pattern.as_bytes()[char_idx] as char;
                if ch != ' ' {
                    buf.set_char(x as u16, y as u16, ch, Color::DarkGrey, Color::Black);
                }
            }
        }

        // 2. Far Skyline & Mountain/Tower Silhouettes (around ground_y - 8 to ground_y - 6)
        if ground_y > 8 {
            let far_pattern_top =
                "       /\\            /|\\                   /\\           /\\_             ";
            let far_pattern_bot = "  ____/  \\___/\'\\____/ | \\_____/\\_____/\'\\__/  \\_________/   \\______/\'\\___ ";
            let far_shift = (self.far_offset as usize) % far_pattern_top.len();

            let y_top = ground_y - 8;
            let y_bot = ground_y - 7;

            for x in 0..width {
                let idx = (far_shift + x) % far_pattern_top.len();
                let ch1 = far_pattern_top.as_bytes()[idx] as char;
                if ch1 != ' ' {
                    buf.set_char(x as u16, y_top as u16, ch1, Color::DarkGrey, Color::Black);
                }
                let ch2 = far_pattern_bot.as_bytes()[idx] as char;
                if ch2 != ' ' {
                    buf.set_char(x as u16, y_bot as u16, ch2, Color::DarkGrey, Color::Black);
                }
            }
        }

        // 3. Midground Gantries, Telegraph Poles & Power Lines (around ground_y - 5 to ground_y - 2)
        if ground_y > 5 {
            let wire_pattern =
                "---.___________.---.___________.---.___________.---.___________.---.____________";
            let pole_pattern =
                "   |           |   |           |   |           |   |           |   |            ";
            let mid_shift = (self.mid_offset as usize) % wire_pattern.len();

            let y_wire = ground_y - 4;
            let y_pole = ground_y - 3;
            let y_pole2 = ground_y - 2;

            for x in 0..width {
                let idx = (mid_shift + x) % wire_pattern.len();
                let ch_wire = wire_pattern.as_bytes()[idx] as char;
                if ch_wire != ' ' {
                    buf.set_char(
                        x as u16,
                        y_wire as u16,
                        ch_wire,
                        Color::DarkGrey,
                        Color::Black,
                    );
                }
                let ch_pole = pole_pattern.as_bytes()[idx] as char;
                if ch_pole != ' ' {
                    buf.set_char(
                        x as u16,
                        y_pole as u16,
                        ch_pole,
                        Color::DarkGrey,
                        Color::Black,
                    );
                    buf.set_char(
                        x as u16,
                        y_pole2 as u16,
                        ch_pole,
                        Color::DarkGrey,
                        Color::Black,
                    );
                }
            }
        }

        // 4. Track Surface (ground_y and ground_y + 1)
        if ground_y < height {
            let rail_shift = (self.ground_offset as usize) % 4;
            for x in 0..width {
                // Continuous iron track with sleepers
                buf.set_char(x as u16, ground_y as u16, '=', Color::White, Color::Black);
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
            }
        }
    }
}
