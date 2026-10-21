//! Authored scenery for the suspended settlements beyond the route.

use serde::{Deserialize, Serialize};

use crate::terminal::buffer::{Buffer, Color};

const SCENE_WIDTH: i32 = 420;

const ARCH_TALL: &[&str] = &[
    r"                                  .",
    r"                              _.-'|`-._",
    r"                      ______/   ||   \______",
    r"             ________/         ||          \________",
    r"        ____/                   ||                   \____",
    r"   ____/                       /||\                       \____",
    r" _/                           / || \                           \_",
    r"|____________________________/  ||  \____________________________|",
    r"             |                | || |                |",
    r"             |                | || |                |",
    r"       ______|______     _____|_||_|_____      _____|______",
    r"      |    []     | |   |  []   ||   []  |    |   []     |",
    r"      |__________|_|___|_______||_______|____|__________|",
    r"          ||  ||            ||||||            ||  ||",
];

const ARCH_COMPACT: &[&str] = &[
    r"             _.-'|`-._",
    r"      ______/   ||   \______",
    r"  ___/         /||\         \___",
    r" /____________/ || \____________\",
    r"       |       | || |       |",
    r"     __|_______|_||_|_______|__",
];

const ARRAY_TOWER: &[&str] = &[
    r"                            /\",
    r"                           /||\",
    r"                          /_||_\",
    r"                     ____/  ||  \____",
    r"          __________/      ||      \__________",
    r"     ____/                 ||                 \____",
    r" ___/______________________||______________________\___",
    r"                          _||_",
    r"                         | [] |",
    r"                         |____|",
    r"                           ||",
    r"                    _______||_______",
    r"                   |___[]______[]___|",
    r"                       ||      ||",
];

const ARRAY_TOWER_COMPACT: &[&str] = &[
    r"       /\",
    r"      /||\",
    r"  ___/_||_\___",
    r"     | [] |",
    r"     |____|",
    r"       ||",
    r"   ____||____",
];

const RETURNING_TRAM: &[&str] = &[
    r"      .--------.",
    r"  ____| []  [] |____",
    r"_/____|________|____\_",
    r"      o        o",
];

const SETTLEMENT_LEFT: &[&str] = &[
    r"       .                    ______",
    r"    __/ \_____       ______|  []  |___",
    r" __/  []    __\_____/  []  |_______|  \__",
    r"|_____|____|  |_____|______|___  ___|_____|",
    r"| [ ] | [] |  |  [] | [ ]  |  |_|_  |     |",
    r"|_____|____|__|_____|______|_______|_____|",
    r"   ||    ||     ||     ||      ||      ||",
    r"   || v  ||  /\ ||     ||  v   ||   /\ ||",
    r"___||_/\_||_/  \||_____||_/\___||__/  \||___",
    r"     ||  ||      ||       ||   ||      ||",
    r"  v  ||  ||  v   ||  v    ||   ||  v   ||  v",
    r" /\  ||  || /\   || /\   ||   || /\   || /\",
    r"/  \_||__||/  \__||/  \__||___||/  \__||/  \_",
];

const SETTLEMENT_RIGHT: &[&str] = &[
    r"                              .",
    r"              /\          ___/ \___",
    r"       ______/  \____    _|___[]___|_",
    r"  ____|___[]______  |___|__|_______|__|____",
    r" |  []   |   ___  | |   []   |  []   |     |",
    r" |_______|__|___|_|_|________|_______|_____|",
    r"   ||      ||      ||    ||       ||     ||",
    r"  v||  /\  || v   ||    ||  /\   ||  v  ||",
    r"___||_/  \_||_/\__||____||_/  \__||_/\__||___",
    r"   ||      ||  ||        ||      ||  ||",
    r"   ||  v   ||  ||   v    ||  v   ||  ||   v",
    r"  /||\    /||\ ||  /\   /||\    || /||\ /||\",
    r"_/ || \__/ || \||_/  \_/ || \___||/ || V || \_",
];

const SETTLEMENT_COMPACT: &[&str] = &[
    r"    /\       ______      ___",
    r" __/  \_____| []  |_____|[ ]|__",
    r"|_[ ]__|_[]_|_____|_[]__|___|__|",
    r"   ||    ||     ||     ||    ||",
    r" v || /\ ||  v  || /\  || v  ||",
    r"__/||/  \||_/\__||/  \_||_/\_||__",
];

const HANGING_HOMES: &[&str] = &[
    r"                      .----.",
    r"         _________..-'      `-.___",
    r"  ______|  []      []    _______   `----.__",
    r" |   _   |___________ ___|  []  |      ____|",
    r" |__|_|__|  ___  ___|_|__|______|_____|",
    r" |  [ ]  | |   ||   |    |  []   |    |",
    r" |_______|_|___||___|____|_______|____|",
    r"     ||     ||   ||       ||       ||",
    r"  v  || /\  ||   || v     ||  /\   ||",
    r" _\__||/  \_||___||/ \____||_/  \__||_",
    r"      ||     ||   ||        ||       ||",
];

const HANGING_HOMES_COMPACT: &[&str] = &[
    r"      .----.   ___",
    r"___  | [] |__|[] |__",
    r"|[ ]_|____|_[]_|___|",
    r" ||   ||   v  ||  ||",
    r" v|| /||\   /||\ ||",
    r"__||/ || \_/ || \||_",
];

const WISP_A: &[&str] = &[r"      ~~~~~~~\_                    ."];
const WISP_B: &[&str] = &[r"                 .       ~~~~~~~~~~"];
const VINES: &[&str] = &[r"  v    /\     v       /\    v   /\"];

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
        self.ground_offset += world_dx;
        self.fg_offset += world_dx * 1.35;
    }

    pub fn render_backdrop(&self, buf: &mut Buffer, ground_y: usize, relay_restored: bool) {
        let width = buf.width() as usize;
        let height = buf.height() as usize;
        if height < 10 || width < 40 || ground_y < 5 {
            return;
        }

        let sky_rows = (ground_y * 22 / 100).clamp(2, 8);
        let far_rows = (ground_y * 38 / 100).clamp(3, 16);
        let far_start = 1 + sky_rows;
        let far_end = (far_start + far_rows).min(ground_y.saturating_sub(2));
        let mid_start = far_end;
        let mid_end = ground_y.saturating_sub(1);

        draw_scene_art(buf, self.sky_offset, 20, 1, WISP_A, Color::DarkGrey, false);
        let second_wisp_x = if width >= 120 { 190 } else { 54 };
        draw_scene_art(
            buf,
            self.sky_offset,
            second_wisp_x,
            4.min(ground_y.saturating_sub(4)) as i32,
            WISP_B,
            Color::DarkGrey,
            false,
        );

        let far_art = if far_end.saturating_sub(far_start) >= ARCH_TALL.len() {
            ARCH_TALL
        } else {
            ARCH_COMPACT
        };
        draw_scene_art(
            buf,
            self.far_offset,
            -8,
            (far_end.saturating_sub(far_art.len())) as i32,
            far_art,
            Color::DarkGrey,
            relay_restored,
        );
        let tower_art = if far_end.saturating_sub(far_start) >= ARRAY_TOWER.len() {
            ARRAY_TOWER
        } else {
            ARRAY_TOWER_COMPACT
        };
        let tower_x = if width >= 120 { 150 } else { 52 };
        draw_scene_art(
            buf,
            self.far_offset,
            tower_x,
            (far_end.saturating_sub(tower_art.len())) as i32,
            tower_art,
            Color::DarkGrey,
            relay_restored,
        );
        if relay_restored {
            let tram_x = 18 + (self.far_offset * 0.4) as i32 % 88;
            draw_returning_tram(
                buf,
                self.far_offset,
                tram_x,
                far_end.saturating_sub(6) as i32,
            );
        }

        let settlement = if mid_end.saturating_sub(mid_start) >= SETTLEMENT_LEFT.len() {
            SETTLEMENT_LEFT
        } else {
            SETTLEMENT_COMPACT
        };
        draw_scene_art(
            buf,
            self.mid_offset,
            -6,
            (mid_end.saturating_sub(settlement.len())) as i32,
            settlement,
            Color::DarkGrey,
            relay_restored,
        );
        draw_scene_art(
            buf,
            self.mid_offset,
            148,
            (mid_end.saturating_sub(SETTLEMENT_RIGHT.len())) as i32,
            SETTLEMENT_RIGHT,
            Color::DarkGrey,
            relay_restored,
        );
        let center_x = if width >= 120 { 64 } else { 42 };
        let hanging_homes = if mid_end.saturating_sub(mid_start) >= HANGING_HOMES.len() {
            HANGING_HOMES
        } else {
            HANGING_HOMES_COMPACT
        };
        draw_scene_art(
            buf,
            self.mid_offset,
            center_x,
            (mid_end.saturating_sub(hanging_homes.len())) as i32,
            hanging_homes,
            Color::DarkGrey,
            relay_restored,
        );
        draw_scene_art(
            buf,
            self.mid_offset,
            74,
            mid_end.saturating_sub(3) as i32,
            VINES,
            Color::DarkGreen,
            false,
        );

        for x in 0..width {
            let rail_color = if relay_restored {
                Color::Cyan
            } else {
                Color::White
            };
            buf.set_char(x as u16, ground_y as u16, '=', rail_color, Color::Black);

            if ground_y + 1 < height {
                let sleeper_ch = if (x + self.ground_offset as usize).is_multiple_of(4) {
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
                let truss_ch = if (x + self.ground_offset as usize) % 4 < 2 {
                    '/'
                } else {
                    '\\'
                };
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
                let pier_col = (x + self.ground_offset as usize) % 24;
                if pier_col == 0 || pier_col == 5 {
                    for py in (ground_y + 4)..height {
                        buf.set_char(x as u16, py as u16, '|', Color::DarkGrey, Color::Black);
                    }
                }
            }
        }
    }
}

fn draw_scene_art(
    buf: &mut Buffer,
    offset: f32,
    art_x: i32,
    art_y: i32,
    art: &[&str],
    base_color: Color,
    relay_restored: bool,
) {
    let phase = offset.rem_euclid(SCENE_WIDTH as f32) as i32;
    let width = buf.width() as i32;

    for cycle in [-SCENE_WIDTH, 0, SCENE_WIDTH] {
        let start_x = art_x + cycle - phase;
        for (row, line) in art.iter().enumerate() {
            let y = art_y + row as i32;
            if !(0..buf.height() as i32).contains(&y) {
                continue;
            }
            for (column, glyph) in line.chars().enumerate() {
                let x = start_x + column as i32;
                if !(0..width).contains(&x) {
                    continue;
                }
                if glyph == ' ' {
                    continue;
                }
                let color = match glyph {
                    '#' | 'v' | '^' => Color::DarkGreen,
                    '[' | ']' if base_color == Color::DarkGrey => Color::Yellow,
                    '=' | '+' | '|' if relay_restored => Color::DarkCyan,
                    _ => base_color,
                };
                buf.set_char(x as u16, y as u16, glyph, color, Color::Black);
            }
        }
    }
}

fn draw_returning_tram(buf: &mut Buffer, offset: f32, art_x: i32, art_y: i32) {
    let phase = offset.rem_euclid(SCENE_WIDTH as f32) as i32;

    for cycle in [-SCENE_WIDTH, 0, SCENE_WIDTH] {
        let start_x = art_x + cycle - phase;
        for (row, line) in RETURNING_TRAM.iter().enumerate() {
            let y = art_y + row as i32;
            if !(0..buf.height() as i32).contains(&y) {
                continue;
            }
            for (column, glyph) in line.chars().enumerate() {
                let x = start_x + column as i32;
                if !(0..buf.width() as i32).contains(&x) {
                    continue;
                }
                if glyph == ' ' {
                    buf.set_char(x as u16, y as u16, ' ', Color::Reset, Color::Black);
                    continue;
                }
                let color = match glyph {
                    '[' | ']' => Color::Yellow,
                    '=' | '|' => Color::DarkCyan,
                    _ => Color::DarkGrey,
                };
                buf.set_char(x as u16, y as u16, glyph, color, Color::Black);
            }
        }
    }
}
