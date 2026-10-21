//! Authored scenery for the suspended settlements beyond the route.

use serde::{Deserialize, Serialize};

use crate::terminal::buffer::{Buffer, Color};

#[derive(Clone, Copy)]
struct ScenicArt {
    x: i32,
    full: &'static [&'static str],
    compact: &'static [&'static str],
    color: Color,
}

impl ScenicArt {
    const fn new(
        x: i32,
        full: &'static [&'static str],
        compact: &'static [&'static str],
        color: Color,
    ) -> Self {
        Self {
            x,
            full,
            compact,
            color,
        }
    }
}

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

const BRIDGE_SPAN_WIDTH: i32 = 48;
const BRIDGE_SPANS: &[&[&str]] = &[
    &[
        "   |              |                 |",
        "  /|\\            /|\\               /|\\",
        "__/ | \\_____/ | \\______/ | \\___/ |",
        "   ||            ||                ||",
    ],
    &[
        "|             |                 |",
        "|\\           /|\\               /|",
        "| \\___/ |  \\_____/ |  \\___/ |",
        "||           ||                ||",
    ],
    &[
        "     |                  |       |",
        "    /|\\                /|\\     /|\\",
        "  / | \\____/  |   \\___/ |  \\_/",
        "    ||                 ||      ||",
    ],
];

const CABLE_BRIDGE: &[&str] = &[
    r"                                  .",
    r"                    ___..----..___",
    r"       _______.----'      ||      `----._______",
    r"  ____/       ||          ||          ||       \____",
    r" /____________||__________||__________||____________\",
    r"        |      ||     ___/||\___      ||      |",
    r"    ____|______||____|   ||||   |_____||______|____",
    r"   |  []   []  ||    | [] || [] |     ||  []   []  |",
    r"   |__________||____|____|__|____|____||__________|",
    r"       ||      ||       ||  ||       ||      ||",
    r"   v   || /\   ||   v   ||  ||   /\  ||  v   ||",
    r"___||__/  \___||__/ \___||__||__/  \_||_____||_____",
];

const CABLE_BRIDGE_COMPACT: &[&str] = &[
    r"      ___..----..___",
    r" ____/   ||    ||   \____",
    r"|_______ ||____|| _______|",
    r"  []  |__||_[]_||__|  []",
    r"  ||    ||  ||  ||    ||",
    r"_v||__/  \_||_/  \__||v_",
];

const ROOTED_HOMES: &[&str] = &[
    r"       v          .----.              v",
    r"  ____/ \_____.--' []  `--.____    ___/ \__",
    r" |  []    []  |____    ____|  |___| []   [] |",
    r" |____________| [] |__| [] |__|   |_________|",
    r"      ||       |___|  |___|       ||",
    r"  v   ||   /\   ||    ||   /\     ||   v",
    r"__||__||__/  \__||____||__/  \____||___||____",
    r"  ||      ||       ||       ||        ||",
    r"~~~~~   ~~~~~~   ~~~~~~   ~~~~~~   ~~~~~~~~",
];

const ROOTED_HOMES_COMPACT: &[&str] = &[
    r"  v    .----.      /\",
    r" ___..' []  `.___|[]|__",
    r"| [] |_____| [] |___|[]|",
    r"  ||   /\  ||  /\  ||",
    r"_v||__/  \_||_/  \_||v_",
    r"~~~~~   ~~~~~~   ~~~~~~",
];

const ROOF_GARDEN: &[&str] = &[
    r"        /\       v     .       /\",
    r"   ____/  \_____/ \____/ \_____/  \___",
    r"  | []  []  []  | []  []  []  []  []  |",
    r"  |_____________|_____________________|",
    r"      ||     ||     ||       ||",
    r"  v   || /\  ||  v  ||  /\   ||   v",
    r"__||__||/  \_||_____||_/  \__||___||___",
    r"  ||       ||      ||      ||",
    r"~~~~~~   ~~~~~~  ~~~~~~~  ~~~~~~~~",
];

const ROOF_GARDEN_COMPACT: &[&str] = &[
    r"   /\    v     /\",
    r"__/  \__/ \___/  \__",
    r"| []  [] | []  []  |",
    r"|________|_________|",
    r"  ||  /\  ||  /\ ||",
    r"_v||_/  \_||_/  \||v_",
];

const WATER_STATION: &[&str] = &[
    r"                 .",
    r"       _________/|\_________",
    r"  ____/     []   |   []      \____",
    r" |____  ________ | ________  _____|",
    r"      ||  []   |___|   []  ||",
    r"  ____||_______|   |_______||____",
    r" |  []   []  |  ___  |  []   []  |",
    r" |__________|_|___|_|___________|",
    r"      ||       || ||       ||",
    r"  v   ||  /\   || ||  /\   ||  v",
    r"__||__||_/  \__||_||_/  \__||__||___",
    r"~~~~~~    ~~~~~~~    ~~~~~~~    ~~~~~",
];

const WATER_STATION_COMPACT: &[&str] = &[
    r"       .   /|\",
    r"  ____/ [] | [] \____",
    r" |____|____|____|____|",
    r"   ||  |___|  ||",
    r"_v_||_/  \_||_\_||v_",
    r"~~~~~   ~~~~~   ~~~~~",
];

const HANGING_GARDEN: &[&str] = &[
    r"          v       .--------------------.        v",
    r"     _____/ \_____| [] [] [] [] [] []   |______/ \___",
    r"    |   []    []  |____________________|  []     []  |",
    r"    |_____________|    _    _    _    _|____________|",
    r"         ||     |___| |[]| |[]| |[]| |___|     ||",
    r"    v    || /\   ||   ||   ||   ||   ||   /\  ||   v",
    r" ___||___||/  \__||___||___||___||___||__/  \_||___||___",
    r"       ||       ||       ||       ||       ||",
    r" ~~~~~~~   ~~~~~~~~   ~~~~~~~~   ~~~~~~~~~~",
];

const HANGING_GARDEN_COMPACT: &[&str] = &[
    r"    v   .-----------.   v",
    r" ___/\__| [] [] []  |__/\___",
    r"|________|__________|________|",
    r"   ||   |[]| || |[]|   ||",
    r"_v_||__/  \_||_/  \__||_||v_",
    r"~~~~~   ~~~~~~   ~~~~~~",
];

const FLOWING_HANGING_GARDEN: &[&str] = &[
    r"          v       .--------------------.        v",
    r"     _____/ \_____| [] [] [] [] [] []   |______/ \___",
    r"    |   []    []  |____________________|  []     []  |",
    r"    |_____________|    _    _    _    _|____________|",
    r"         ||     |___| |[]| |[]| |[]| |___|     ||",
    r"    v    || /\   ||   ||   ||   ||   ||   /\  ||   v",
    r" ___||___||/  \__||___||___||___||___||__/  \_||___||___",
    r"       ||       ||       ||       ||       ||",
    r" ~~~~~~~~~~~~>>>>>>~~~~~~~~~~~~",
];

const FLOWING_HANGING_GARDEN_COMPACT: &[&str] = &[
    r"    v   .-----------.   v",
    r" ___/\__| [] [] []  |__/\___",
    r"|________|__________|________|",
    r"   ||   |[]| || |[]|   ||",
    r"_v_||__/  \_||_/  \__||_||v_",
    r"~~~~~>>>>~~~~~",
];

const DISTANT_SPAN: &[&str] = &[
    r"                                .",
    r"                __..---..___.-'|`-.__",
    r"       ______.-'       ||         `-.______",
    r"  ____/                 ||                  \____",
    r"_/_____________________||______________________\_",
    r"                        ||",
    r"                    ____||____",
    r"                   |____[]____|",
    r"                       ||",
    r"                 ______||______",
];

const DISTANT_SPAN_COMPACT: &[&str] = &[
    r"         __..---..__",
    r"   ____-'     ||    `-____",
    r"__/__________||__________\__",
    r"              ||",
    r"           ___||___",
    r"          |___[]___|",
];

const HIGH_TOWER: &[&str] = &[
    r"                               /\",
    r"                              /||\",
    r"                       _____/_||_\_____",
    r"              ________/       ||       \________",
    r"         ____/                 ||                 \____",
    r"   _____/_____________________||_____________________\_____",
    r"                              _||_",
    r"                             | [] |",
    r"                             |____|",
    r"                               ||",
];

const HIGH_TOWER_COMPACT: &[&str] = &[
    r"          /\",
    r"      ___/||\___",
    r"   __/    ||    \__",
    r"  /_______||_______\",
    r"         |[]|",
    r"          ||",
];

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

    pub fn render_backdrop(
        &self,
        buf: &mut Buffer,
        ground_y: usize,
        relay_restored: bool,
        canal_gate_opened: bool,
    ) {
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
        draw_scene_art(
            buf,
            self.sky_offset,
            500,
            3.min(ground_y.saturating_sub(4)) as i32,
            WISP_A,
            Color::DarkGrey,
            false,
        );

        let far_band_height = far_end.saturating_sub(far_start);
        // Each layer is a one-pass authored panorama, so these are layer positions, not world coordinates.
        draw_band_art(
            buf,
            self.far_offset,
            far_end,
            far_band_height,
            ScenicArt::new(-8, ARCH_TALL, ARCH_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        let tower_x = if width >= 120 { 150 } else { 52 };
        draw_band_art(
            buf,
            self.far_offset,
            far_end,
            far_band_height,
            ScenicArt::new(tower_x, ARRAY_TOWER, ARRAY_TOWER_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.far_offset,
            far_end,
            far_band_height,
            ScenicArt::new(220, CABLE_BRIDGE, CABLE_BRIDGE_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.far_offset,
            far_end,
            far_band_height,
            ScenicArt::new(720, HIGH_TOWER, HIGH_TOWER_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.far_offset,
            far_end,
            far_band_height,
            ScenicArt::new(1080, DISTANT_SPAN, DISTANT_SPAN_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        if relay_restored {
            let tram_x = 18 + (self.far_offset * 0.4) as i32 % 88;
            draw_returning_tram(buf, tram_x, far_end.saturating_sub(6) as i32);
        }

        let mid_band_height = mid_end.saturating_sub(mid_start);
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            ScenicArt::new(-6, SETTLEMENT_LEFT, SETTLEMENT_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            ScenicArt::new(148, SETTLEMENT_RIGHT, SETTLEMENT_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        let center_x = if width >= 120 { 64 } else { 42 };
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            ScenicArt::new(
                center_x,
                HANGING_HOMES,
                HANGING_HOMES_COMPACT,
                Color::DarkGrey,
            ),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            ScenicArt::new(420, ROOTED_HOMES, ROOTED_HOMES_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            ScenicArt::new(660, ROOF_GARDEN, ROOF_GARDEN_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            ScenicArt::new(540, WATER_STATION, WATER_STATION_COMPACT, Color::DarkGrey),
            relay_restored,
        );
        let hanging_garden = if canal_gate_opened {
            ScenicArt::new(
                780,
                FLOWING_HANGING_GARDEN,
                FLOWING_HANGING_GARDEN_COMPACT,
                Color::DarkGrey,
            )
        } else {
            ScenicArt::new(780, HANGING_GARDEN, HANGING_GARDEN_COMPACT, Color::DarkGrey)
        };
        draw_band_art(
            buf,
            self.mid_offset,
            mid_end,
            mid_band_height,
            hanging_garden,
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
        }

        draw_bridge_understructure(buf, ground_y, self.ground_offset, relay_restored);
    }
}

fn draw_bridge_understructure(
    buf: &mut Buffer,
    ground_y: usize,
    offset: f32,
    relay_restored: bool,
) {
    let scroll_x = offset.floor() as i32;
    let first_span = scroll_x.div_euclid(BRIDGE_SPAN_WIDTH);
    let visible_span_count = (buf.width() as i32 + BRIDGE_SPAN_WIDTH - 1) / BRIDGE_SPAN_WIDTH + 1;

    for span_offset in 0..visible_span_count {
        let span_index = first_span + span_offset;
        let span_x = span_index * BRIDGE_SPAN_WIDTH - scroll_x;
        let art_index = span_index.rem_euclid(BRIDGE_SPANS.len() as i32) as usize;
        draw_scene_art(
            buf,
            0.0,
            span_x,
            ground_y as i32 + 1,
            BRIDGE_SPANS[art_index],
            Color::DarkGrey,
            relay_restored,
        );
    }
}

fn draw_band_art(
    buf: &mut Buffer,
    offset: f32,
    band_end: usize,
    band_height: usize,
    art: ScenicArt,
    relay_restored: bool,
) {
    let lines = if band_height >= art.full.len() {
        art.full
    } else {
        art.compact
    };
    draw_scene_art(
        buf,
        offset,
        art.x,
        band_end.saturating_sub(lines.len()) as i32,
        lines,
        art.color,
        relay_restored,
    );
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
    let start_x = art_x - offset as i32;
    let width = buf.width() as i32;

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
                '~' => Color::DarkCyan,
                '>' => Color::Cyan,
                '[' | ']' if base_color == Color::DarkGrey => Color::Yellow,
                '=' | '+' | '|' if relay_restored => Color::DarkCyan,
                _ => base_color,
            };
            buf.set_char(x as u16, y as u16, glyph, color, Color::Black);
        }
    }
}

fn draw_returning_tram(buf: &mut Buffer, start_x: i32, art_y: i32) {
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
