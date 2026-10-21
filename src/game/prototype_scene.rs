use crossterm::style::Color;

use super::pixel::{ColorTier, PixelCanvas, Rgb};
use super::prototype::{Pose, PrototypeGame};
use crate::terminal::buffer::Buffer;
use crate::ui::i18n::{t, Locale};

const SKY_TOP: Rgb = Rgb(12, 28, 39);
const HAZE: Rgb = Rgb(62, 91, 81);
const FAR_STONE: Rgb = Rgb(31, 53, 57);
const FAR_EDGE: Rgb = Rgb(52, 78, 73);
const MID_STONE: Rgb = Rgb(43, 70, 66);
const STONE_LIGHT: Rgb = Rgb(79, 105, 84);
const SOIL: Rgb = Rgb(27, 43, 39);
const SOIL_DEEP: Rgb = Rgb(19, 34, 33);
const MOSS: Rgb = Rgb(113, 148, 93);
const MOSS_LIGHT: Rgb = Rgb(163, 183, 111);
const WINDOW: Rgb = Rgb(226, 169, 109);
const RELAY_GLOW: Rgb = Rgb(244, 198, 123);
const RELAY_ACTIVE: Rgb = Rgb(117, 220, 191);
const HAZARD_DARK: Rgb = Rgb(77, 53, 45);
const HAZARD: Rgb = Rgb(199, 103, 77);
const PLAYER_OUTLINE: Rgb = Rgb(12, 28, 34);
const PLAYER_SCARF: Rgb = Rgb(232, 157, 92);
const PLAYER_COAT: Rgb = Rgb(186, 206, 177);
const PLAYER_PACK: Rgb = Rgb(143, 84, 57);
const PLAYER_SKIN: Rgb = Rgb(220, 174, 128);
const PLAYER_ACCENT: Rgb = Rgb(85, 192, 177);
const PLAYER_LIGHT: Rgb = Rgb(241, 224, 183);

pub fn render(
    game: &PrototypeGame,
    width: u16,
    height: u16,
    locale: Locale,
    tier: ColorTier,
) -> Buffer {
    let scene_height = height.saturating_sub(2).max(2) * 2;
    let mut canvas = PixelCanvas::new(width.max(1), scene_height, SKY_TOP);
    draw_sky(&mut canvas, game.camera_x(), game.viewport_width());
    draw_distant_ridge(&mut canvas, game.camera_x(), game.viewport_width());
    draw_monument(&mut canvas, game);
    draw_viaduct(&mut canvas, game);
    draw_settlement(&mut canvas, game);
    draw_ground(&mut canvas, game);
    draw_landmarks(&mut canvas, game);
    draw_foreground_plants(&mut canvas, game);
    draw_player(&mut canvas, game);

    let pixels = canvas.to_buffer(tier);
    let mut buffer = Buffer::new(width, height);
    for y in 0..pixels.height() {
        for x in 0..pixels.width().min(buffer.width()) {
            if let (Some(source), Some(target)) = (pixels.get(x, y), buffer.get_mut(x, y + 1)) {
                *target = *source;
            }
        }
    }
    if tier == ColorTier::Ascii {
        draw_ascii_player(&mut buffer, game);
    }

    let hud_color = tier.map_color(Rgb(220, 208, 173));
    let quiet_color = tier.map_color(Rgb(119, 151, 131));
    let title = t("prototype_title", locale);
    buffer.draw_str(1, 0, title, hud_color, Color::Reset);
    let objective_key = if game.destination_reached() {
        "prototype_arrived"
    } else {
        "prototype_goal"
    };
    let objective = t(objective_key, locale);
    let objective_width = objective.chars().count() as u16;
    buffer.draw_str(
        width.saturating_sub(objective_width.saturating_add(2)),
        0,
        objective,
        quiet_color,
        Color::Reset,
    );

    let controls_key = if game.near_relay() && !game.relay_active() {
        "prototype_connect"
    } else if game.relay_active() {
        "prototype_connected"
    } else {
        "prototype_controls"
    };
    buffer.draw_str(
        1,
        height.saturating_sub(1),
        t(controls_key, locale),
        quiet_color,
        Color::Reset,
    );
    buffer
}

fn draw_sky(canvas: &mut PixelCanvas, camera_x: f32, width: f32) {
    let height = canvas.height() as i32;
    let canvas_width = canvas.width() as i32;
    for y in 0..height {
        let t = y as f32 / height.max(1) as f32;
        let color = if t < 0.70 {
            blend(SKY_TOP, Rgb(38, 65, 67), t / 0.70)
        } else {
            blend(Rgb(38, 65, 67), HAZE, (t - 0.70) / 0.30)
        };
        canvas.fill_rect(0, y, canvas_width, 1, color);
    }

    let sun_x = (width * 0.19 - camera_x * 0.035).round() as i32;
    let sun_y = (height as f32 * 0.27) as i32;
    canvas.fill_rect(sun_x - 3, sun_y - 3, 7, 7, Rgb(191, 171, 129));
    canvas.fill_rect(sun_x - 1, sun_y - 5, 3, 11, Rgb(191, 171, 129));
    canvas.fill_rect(sun_x - 5, sun_y - 1, 11, 3, Rgb(191, 171, 129));

    let cloud_a = (width * 0.08 - camera_x * 0.07).round() as i32;
    let cloud_b = (width * 0.57 - camera_x * 0.05).round() as i32;
    draw_cloud(canvas, cloud_a, (height as f32 * 0.17) as i32, 17);
    draw_cloud(canvas, cloud_b, (height as f32 * 0.31) as i32, 23);
    draw_line(
        canvas,
        0,
        (height as f32 * 0.40) as i32,
        canvas_width,
        (height as f32 * 0.35) as i32,
        Rgb(64, 93, 83),
    );
    draw_line(
        canvas,
        (width * 0.13 - camera_x * 0.12) as i32,
        (height as f32 * 0.46) as i32,
        (width * 0.83 - camera_x * 0.12) as i32,
        (height as f32 * 0.43) as i32,
        Rgb(54, 82, 76),
    );
}

fn draw_cloud(canvas: &mut PixelCanvas, x: i32, y: i32, width: i32) {
    let color = Rgb(65, 90, 83);
    canvas.fill_rect(x + 4, y, width / 2, 2, color);
    canvas.fill_rect(x, y + 2, width, 2, color);
    canvas.fill_rect(x + 3, y + 4, width - 5, 1, color);
}

fn draw_distant_ridge(canvas: &mut PixelCanvas, camera_x: f32, width: f32) {
    let h = canvas.height() as f32;
    let offset = (camera_x * 0.12).round() as i32;
    let points = [
        (-0.08, 0.67),
        (0.08, 0.46),
        (0.19, 0.61),
        (0.31, 0.35),
        (0.40, 0.58),
        (0.53, 0.43),
        (0.64, 0.64),
        (0.76, 0.38),
        (0.88, 0.59),
        (1.03, 0.43),
        (1.18, 0.67),
        (1.36, 0.40),
        (1.51, 0.62),
        (1.68, 0.44),
        (1.86, 0.67),
        (2.04, 0.49),
        (2.24, 0.70),
    ];
    let mut polygon = Vec::with_capacity(points.len() + 2);
    polygon.push((-offset, h as i32));
    for (x, y) in points {
        polygon.push(((x * width) as i32 - offset, (y * h) as i32));
    }
    polygon.push(((2.24 * width) as i32 - offset, h as i32));
    fill_polygon(canvas, &polygon, FAR_STONE);
    draw_line(
        canvas,
        (width * 0.31) as i32 - offset,
        (h * 0.35) as i32,
        (width * 0.40) as i32 - offset,
        (h * 0.58) as i32,
        FAR_EDGE,
    );
    draw_line(
        canvas,
        (width * 0.76) as i32 - offset,
        (h * 0.38) as i32,
        (width * 0.88) as i32 - offset,
        (h * 0.59) as i32,
        FAR_EDGE,
    );
}

fn draw_monument(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let w = game.viewport_width();
    let h = game.viewport_height();
    let scale = (w * 0.32).clamp(30.0, 84.0);
    let center = game.destination_x() + 2.0;
    let x = (center - scale * 0.5 - game.camera_x() * 0.24).round() as i32;
    let top = (h * 0.12) as i32;
    let base = (game.ground_y() - h * 0.07) as i32;
    let post = (scale * 0.115).round().max(4.0) as i32;
    let beam = (h * 0.085).round().max(5.0) as i32;
    let color = FAR_EDGE;
    let shadow = FAR_STONE;

    canvas.fill_rect(x, top, post, base - top, shadow);
    canvas.fill_rect(x + post, top, 2, base - top, color);
    canvas.fill_rect(
        x + scale as i32 - post,
        top + beam,
        post,
        base - top - beam,
        shadow,
    );
    canvas.fill_rect(
        x + scale as i32 - post - 2,
        top + beam,
        2,
        base - top - beam,
        color,
    );
    canvas.fill_rect(x + post, top, scale as i32 - post, beam, color);
    canvas.fill_rect(x + post, top + beam, scale as i32 - post, 2, shadow);
    canvas.fill_rect(x + scale as i32 / 2, top - 3, scale as i32 / 4, 2, color);
    canvas.fill_rect(x + scale as i32 / 2, top - 9, 2, 7, color);
    canvas.fill_rect(x + scale as i32 / 2 - 3, top - 10, 8, 2, Rgb(137, 158, 118));
    canvas.fill_rect(x + scale as i32 / 2 - 1, top - 8, 4, 3, Rgb(194, 154, 101));

    let deck_y = (h * 0.54) as i32;
    canvas.fill_rect(
        x - (scale * 0.14) as i32,
        deck_y,
        scale as i32 + (scale * 0.28) as i32,
        4,
        color,
    );
    canvas.fill_rect(x + post, deck_y + 4, 2, base - deck_y - 4, shadow);
    canvas.fill_rect(
        x + scale as i32 - post - 2,
        deck_y + 4,
        2,
        base - deck_y - 4,
        shadow,
    );

    let lit = if game.destination_reached() {
        RELAY_ACTIVE
    } else {
        Rgb(137, 158, 118)
    };
    for (wx, wy) in [
        (x + post + 5, top + beam + 8),
        (x + post + 15, top + beam + 8),
        (x + scale as i32 - post - 10, top + beam + 15),
        (x + post + 6, deck_y - 6),
    ] {
        canvas.fill_rect(wx, wy, 2, 3, lit);
    }

    draw_line(canvas, x + post / 2, top + 3, x - 8, deck_y, FAR_EDGE);
    draw_line(
        canvas,
        x + scale as i32 - post / 2,
        top + 3,
        x + scale as i32 + 9,
        deck_y,
        FAR_EDGE,
    );
    draw_line(
        canvas,
        x - 7,
        deck_y + 1,
        x + scale as i32 + 9,
        deck_y + 1,
        FAR_EDGE,
    );
}

fn draw_viaduct(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let w = game.viewport_width();
    let h = game.viewport_height();
    let offset = (game.camera_x() * 0.42).round() as i32;
    let y = (h * 0.53) as i32;
    let start = (w * 0.06) as i32 - offset;
    let end = (w * 1.27) as i32 - offset;
    canvas.fill_rect(start, y, end - start, 4, MID_STONE);
    canvas.fill_rect(start, y + 4, end - start, 2, FAR_STONE);

    for (ratio, depth) in [
        (0.18, 0.18),
        (0.38, 0.32),
        (0.61, 0.14),
        (0.96, 0.34),
        (1.18, 0.22),
    ] {
        let x = (w * ratio) as i32 - offset;
        let pier_bottom = (h * depth + y as f32).min(game.ground_y() - 3.0) as i32;
        canvas.fill_rect(x, y + 6, 4, pier_bottom - y - 6, MID_STONE);
        canvas.fill_rect(x + 4, y + 6, 1, pier_bottom - y - 6, FAR_EDGE);
        draw_line(canvas, x - 5, y, x + 2, y + 13, FAR_EDGE);
        draw_line(canvas, x + 8, y, x + 2, y + 13, FAR_EDGE);
    }

    let cable = Rgb(68, 96, 81);
    draw_line(
        canvas,
        start,
        y - 8,
        (w * 0.42) as i32 - offset,
        y + 2,
        cable,
    );
    draw_line(
        canvas,
        (w * 0.42) as i32 - offset,
        y + 2,
        (w * 0.81) as i32 - offset,
        y - 12,
        cable,
    );
    draw_line(
        canvas,
        (w * 0.81) as i32 - offset,
        y - 12,
        end,
        y + 2,
        cable,
    );
}

fn draw_settlement(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let w = game.viewport_width();
    let h = game.viewport_height();
    let offset = (game.camera_x() * 0.66).round() as i32;
    let deck_y = (h * 0.53) as i32;

    for (ratio, width, height) in [
        (0.17, 18, 17),
        (0.30, 15, 13),
        (0.40, 12, 10),
        (1.23, 17, 16),
        (1.41, 13, 12),
    ] {
        let x = (w * ratio) as i32 - offset;
        let y = deck_y - height;
        canvas.fill_rect(x, y, width, height, MID_STONE);
        canvas.fill_rect(x + 2, y + 2, width - 4, 2, STONE_LIGHT);
        canvas.fill_rect(x + width - 3, y + 3, 2, height - 3, FAR_STONE);
        fill_polygon(
            canvas,
            &[(x - 2, y), (x + width / 2, y - 5), (x + width + 2, y)],
            FAR_EDGE,
        );
        for window_x in [x + 4, x + width - 6] {
            canvas.fill_rect(window_x, y + 5, 2, 3, WINDOW);
            canvas.fill_rect(window_x, y + 10, 2, 2, Rgb(182, 134, 92));
        }
        canvas.fill_rect(x + width / 2, y - 8, 1, 4, STONE_LIGHT);
    }

    let awning_x = (w * 0.34) as i32 - offset;
    canvas.fill_rect(awning_x, deck_y - 8, 8, 2, Rgb(139, 115, 76));
    canvas.fill_rect(awning_x + 2, deck_y - 6, 1, 6, FAR_EDGE);
    canvas.fill_rect(awning_x + 7, deck_y - 6, 1, 6, FAR_EDGE);
    canvas.fill_rect(awning_x + 3, deck_y - 5, 3, 2, WINDOW);
}

fn draw_ground(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let h = game.viewport_height() as i32;
    let ground = game.ground_y() as i32;
    let camera = game.camera_x().round() as i32;
    let gap_start = game.gap_start() as i32 - camera;
    let gap_end = game.gap_end() as i32 - camera;
    let width = canvas.width() as i32;

    canvas.fill_rect(0, ground + 4, width, h - ground - 4, SOIL_DEEP);
    canvas.fill_rect(0, ground + 10, width, 3, SOIL);
    canvas.fill_rect(0, ground + 19, width, 2, Rgb(26, 46, 38));
    for (x, y, size) in [
        (-12, ground + 27, 42),
        (37, ground + 34, 55),
        (108, ground + 25, 34),
    ] {
        canvas.fill_rect(x, y, size, 2, Rgb(31, 48, 39));
    }

    for (left, right) in [(0, gap_start), (gap_end, width + 4)] {
        if right <= left {
            continue;
        }
        canvas.fill_rect(left, ground, right - left, 4, MOSS);
        canvas.fill_rect(left, ground + 4, right - left, h - ground - 4, SOIL);
        canvas.fill_rect(left, ground + 6, right - left, 2, SOIL_DEEP);
        canvas.fill_rect(left, ground, right - left, 1, MOSS_LIGHT);
    }

    canvas.fill_rect(gap_start - 2, ground - 1, 2, 3, STONE_LIGHT);
    canvas.fill_rect(gap_end, ground - 1, 2, 3, STONE_LIGHT);
}

fn draw_landmarks(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let camera = game.camera_x().round() as i32;
    let ground = game.ground_y() as i32;
    let height = game.viewport_height() as i32;
    let scale = game.player_scale();
    let size = |value: i32| (value as f32 * scale).round() as i32;
    let checkpoint = game.checkpoint_x() as i32 - camera;

    canvas.fill_rect(
        checkpoint - size(9),
        ground - size(3),
        size(18),
        size(3),
        FAR_STONE,
    );
    canvas.fill_rect(
        checkpoint - size(8),
        ground - size(25),
        size(16),
        size(3),
        STONE_LIGHT,
    );
    canvas.fill_rect(
        checkpoint - size(7),
        ground - size(22),
        size(2),
        size(18),
        FAR_EDGE,
    );
    canvas.fill_rect(
        checkpoint + size(5),
        ground - size(22),
        size(2),
        size(18),
        FAR_EDGE,
    );
    canvas.fill_rect(
        checkpoint - size(4),
        ground - size(18),
        size(8),
        size(10),
        SOIL_DEEP,
    );
    canvas.fill_rect(
        checkpoint - size(3),
        ground - size(8),
        size(6),
        size(2),
        MID_STONE,
    );
    canvas.fill_rect(
        checkpoint - size(10),
        ground - size(26),
        size(20),
        size(2),
        FAR_STONE,
    );
    let checkpoint_light = if game.checkpoint_active() {
        RELAY_ACTIVE
    } else {
        Rgb(104, 153, 136)
    };
    canvas.fill_rect(
        checkpoint - size(2),
        ground - size(22),
        size(4),
        size(5),
        checkpoint_light,
    );
    canvas.fill_rect(
        checkpoint - size(1),
        ground - size(31),
        size(2),
        size(5),
        checkpoint_light,
    );
    canvas.fill_rect(
        checkpoint - size(5),
        ground - size(30),
        size(10),
        size(2),
        STONE_LIGHT,
    );

    let obstacle = game.obstacle_x() as i32 - camera;
    canvas.fill_rect(
        obstacle - size(2),
        ground - size(4),
        size(14),
        size(4),
        HAZARD_DARK,
    );
    fill_polygon(
        canvas,
        &[
            (obstacle, ground - size(3)),
            (obstacle + size(3), ground - size(15)),
            (obstacle + size(6), ground - size(3)),
        ],
        HAZARD,
    );
    fill_polygon(
        canvas,
        &[
            (obstacle + size(5), ground - size(3)),
            (obstacle + size(9), ground - size(12)),
            (obstacle + size(13), ground - size(3)),
        ],
        HAZARD,
    );
    canvas.fill_rect(
        obstacle + size(2),
        ground - size(7),
        size(8),
        size(2),
        Rgb(119, 69, 54),
    );

    let relay = game.relay_x() as i32 - camera;
    let light = if game.relay_active() {
        RELAY_ACTIVE
    } else {
        RELAY_GLOW
    };
    canvas.fill_rect(
        relay - size(7),
        ground - size(3),
        size(14),
        size(3),
        MID_STONE,
    );
    canvas.fill_rect(
        relay - size(6),
        ground - size(19),
        size(12),
        size(16),
        FAR_EDGE,
    );
    canvas.fill_rect(
        relay - size(4),
        ground - size(17),
        size(8),
        size(8),
        SOIL_DEEP,
    );
    canvas.fill_rect(relay - size(2), ground - size(16), size(4), size(5), light);
    canvas.fill_rect(
        relay - size(4),
        ground - size(18),
        size(8),
        size(2),
        STONE_LIGHT,
    );
    canvas.fill_rect(relay - size(9), ground - size(14), size(3), size(2), light);
    canvas.fill_rect(relay + size(6), ground - size(14), size(3), size(2), light);
    canvas.fill_rect(
        relay - size(8),
        ground - size(10),
        size(2),
        size(5),
        MID_STONE,
    );
    canvas.fill_rect(
        relay + size(6),
        ground - size(10),
        size(2),
        size(5),
        MID_STONE,
    );

    let horizon = (height as f32 * 0.68) as i32;
    canvas.fill_rect(
        game.destination_x() as i32 - camera - 1,
        horizon,
        2,
        (ground - horizon).max(0),
        Rgb(49, 75, 66),
    );
}

fn draw_foreground_plants(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let camera = game.camera_x().round() as i32;
    let ground = game.ground_y() as i32;
    let plants: &[(f32, i32, i32)] = if game.viewport_width() < 120.0 {
        &[(0.94, 8, 6)]
    } else {
        &[
            (0.10, 12, 9),
            (0.37, 9, 8),
            (0.61, 13, 10),
            (0.78, 8, 7),
            (1.13, 12, 9),
        ]
    };
    for &(ratio, height, spread) in plants {
        let x = (game.viewport_width() * ratio) as i32 - camera;
        canvas.fill_rect(x, ground - height, 1, height, Rgb(57, 99, 66));
        fill_polygon(
            canvas,
            &[
                (x, ground - height - 3),
                (x - spread, ground - height + 4),
                (x, ground - height + 2),
            ],
            Rgb(69, 118, 75),
        );
        fill_polygon(
            canvas,
            &[
                (x, ground - height),
                (x + spread, ground - height + 5),
                (x, ground - height + 3),
            ],
            Rgb(87, 133, 79),
        );
        canvas.fill_rect(x - 4, ground - 1, 8, 1, Rgb(49, 92, 60));
    }

    if game.viewport_width() < 120.0 {
        return;
    }

    let tree_x = (game.viewport_width() * 0.21) as i32 - (game.camera_x() * 0.83) as i32;
    let tree_top = ground - (game.viewport_height() * 0.43) as i32;
    canvas.fill_rect(
        tree_x,
        tree_top + 12,
        3,
        ground - tree_top - 12,
        Rgb(66, 82, 55),
    );
    fill_polygon(
        canvas,
        &[
            (tree_x + 1, tree_top),
            (tree_x - 10, tree_top + 20),
            (tree_x + 12, tree_top + 20),
        ],
        Rgb(65, 105, 72),
    );
    fill_polygon(
        canvas,
        &[
            (tree_x + 1, tree_top + 7),
            (tree_x - 13, tree_top + 27),
            (tree_x + 14, tree_top + 27),
        ],
        Rgb(77, 120, 77),
    );
    canvas.fill_rect(tree_x - 3, tree_top + 20, 2, 2, Rgb(132, 153, 88));
    canvas.fill_rect(tree_x + 5, tree_top + 14, 2, 2, Rgb(130, 152, 88));
}

fn draw_player(canvas: &mut PixelCanvas, game: &PrototypeGame) {
    let rows = sprite_rows(game.pose(), game.animation_time());
    let source_width = rows.iter().map(|row| row.len()).max().unwrap_or(20);
    let scale = game.player_scale();
    let width = (source_width as f32 * scale).round().max(1.0) as usize;
    let height = (rows.len() as f32 * scale).round().max(1.0) as usize;
    let x = (game.player_x() - game.camera_x()).round() as i32;
    let y = (game.player_foot_y() - game.player_height()).round() as i32;
    for dest_y in 0..height {
        let source_y = dest_y * rows.len() / height;
        let row = rows[source_y].as_bytes();
        for dest_x in 0..width {
            let source_x = dest_x * source_width / width;
            let code = row.get(source_x).copied().unwrap_or(b'.');
            if code == b'.' {
                continue;
            }
            let px = if game.facing() < 0.0 {
                x + width as i32 - 1 - dest_x as i32
            } else {
                x + dest_x as i32
            };
            let color = player_color(code, game.pose());
            canvas.set(px, y + dest_y as i32, color);
        }
    }
}

fn draw_ascii_player(buffer: &mut Buffer, game: &PrototypeGame) {
    const IDLE: [&str; 8] = [
        "   o~~~   ",
        "  /#==~   ",
        "  /###    ",
        "   ###    ",
        "   ###    ",
        "  /# \\    ",
        " /  \\     ",
        "/    \\    ",
    ];
    const RUN: [&str; 8] = [
        "   o~~~   ",
        "  /#==~   ",
        "  /###    ",
        "   ###    ",
        "   ##\\   ",
        " /#   \\   ",
        "/     \\  ",
        "     /    ",
    ];
    const JUMP: [&str; 8] = [
        " ~~~o     ",
        "    #==~  ",
        "   /###   ",
        "   /##\\  ",
        "    #     ",
        "   / \\    ",
        "  /   \\   ",
        " /     \\  ",
    ];
    const GLIDE: [&str; 8] = [
        "~~~o~~~~~ ",
        "  /#==~   ",
        " /####\\  ",
        "   ###    ",
        "   ###    ",
        "  /# \\    ",
        " /  \\     ",
        "/    \\    ",
    ];
    const SLIDE: [&str; 8] = [
        "          ",
        "          ",
        "  o=====~ ",
        " /#####   ",
        "/  #  \\   ",
        "   / \\    ",
        "          ",
        "          ",
    ];
    const DASH: [&str; 8] = [
        "          ",
        "   o====~>",
        "  /#####  ",
        " /  ###   ",
        "/   #  \\  ",
        "    / \\   ",
        "          ",
        "          ",
    ];
    const INTERACT: [&str; 8] = [
        "   o!~~   ",
        "  /#==~   ",
        "  /###    ",
        "   ###    ",
        "   ###    ",
        "  /# \\    ",
        " /  \\     ",
        "/    \\    ",
    ];
    const HIT: [&str; 8] = [
        " !   !    ",
        "  o~~~    ",
        " /#==~    ",
        "  ###     ",
        "  ###     ",
        " /# \\     ",
        "/  \\      ",
        "          ",
    ];
    let rows = match game.pose() {
        Pose::Jump | Pose::Land => &JUMP,
        Pose::Fall => &JUMP,
        Pose::Glide => &GLIDE,
        Pose::Slide => &SLIDE,
        Pose::Dash => &DASH,
        Pose::Interact => &INTERACT,
        Pose::Hit => &HIT,
        Pose::Run => &RUN,
        Pose::Idle | Pose::Turn | Pose::Respawn => &IDLE,
    };
    let x = (game.player_x() - game.camera_x()).round() as i32;
    let foot = (game.player_foot_y() * 0.5).round() as i32 + 1;
    let top = foot - rows.len() as i32;
    for (row_index, row) in rows.iter().enumerate() {
        let chars = if game.facing() < 0.0 {
            row.chars()
                .rev()
                .map(|character| match character {
                    '/' => '\\',
                    '\\' => '/',
                    other => other,
                })
                .collect::<Vec<_>>()
        } else {
            row.chars().collect()
        };
        for (column, character) in chars.into_iter().enumerate() {
            if let Some(cell) =
                buffer.get_mut((x + column as i32) as u16, (top + row_index as i32) as u16)
            {
                cell.ch = character;
                cell.fg = Color::Reset;
                cell.bg = Color::Reset;
            }
        }
    }
}

fn sprite_rows(pose: Pose, phase: f32) -> &'static [&'static str; 24] {
    match pose {
        Pose::Run => {
            if (phase * 11.0) as i32 % 2 == 0 {
                &RUN_A
            } else {
                &RUN_B
            }
        }
        Pose::Dash => &DASH,
        Pose::Jump => &JUMP,
        Pose::Fall => &FALL,
        Pose::Glide => &GLIDE,
        Pose::Slide => &SLIDE,
        Pose::Interact => &INTERACT,
        Pose::Hit => &HIT,
        Pose::Respawn => &RESPAWN,
        Pose::Land => &LAND,
        Pose::Idle => &IDLE,
        Pose::Turn => &TURN,
    }
}

fn player_color(code: u8, pose: Pose) -> Rgb {
    if pose == Pose::Respawn {
        return match code {
            b'.' => PLAYER_OUTLINE,
            _ => PLAYER_ACCENT,
        };
    }
    match code {
        b'O' => PLAYER_OUTLINE,
        b'S' => PLAYER_SCARF,
        b'C' => PLAYER_COAT,
        b'P' => PLAYER_PACK,
        b'F' => PLAYER_SKIN,
        b'G' => PLAYER_ACCENT,
        b'W' => PLAYER_LIGHT,
        b'L' => Rgb(79, 70, 57),
        b'I' => PLAYER_LIGHT,
        _ => PLAYER_OUTLINE,
    }
}

fn blend(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    Rgb(
        (f32::from(a.0) + (f32::from(b.0) - f32::from(a.0)) * t) as u8,
        (f32::from(a.1) + (f32::from(b.1) - f32::from(a.1)) * t) as u8,
        (f32::from(a.2) + (f32::from(b.2) - f32::from(a.2)) * t) as u8,
    )
}

fn fill_polygon(canvas: &mut PixelCanvas, points: &[(i32, i32)], color: Rgb) {
    if points.len() < 3 {
        return;
    }
    let min_y = points.iter().map(|(_, y)| *y).min().unwrap_or(0).max(0);
    let max_y = points
        .iter()
        .map(|(_, y)| *y)
        .max()
        .unwrap_or(0)
        .min(canvas.height() as i32 - 1);
    for y in min_y..=max_y {
        let mut intersections = Vec::new();
        for index in 0..points.len() {
            let (x1, y1) = points[index];
            let (x2, y2) = points[(index + 1) % points.len()];
            if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                let x = x1 as f32 + (y - y1) as f32 * (x2 - x1) as f32 / (y2 - y1) as f32;
                intersections.push(x.round() as i32);
            }
        }
        intersections.sort_unstable();
        for pair in intersections.chunks(2) {
            if let [left, right] = pair {
                canvas.fill_rect(*left, y, right - left + 1, 1, color);
            }
        }
    }
}

fn draw_line(canvas: &mut PixelCanvas, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        canvas.set(x, y, color);
        if x == x1 && y == y1 {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
}

const IDLE: [&str; 24] = [
    "......OOO...........",
    ".....OOOOO..........",
    "....OOFFFFOO........",
    "....OFFFFFWWO.......",
    ".....OOFFFFOO.......",
    "......OOOOOO........",
    ".....OOO.OO.PPP.....",
    "..SSSOO.OOOO.PPPG...",
    "...SSOO.OCCCC.PPG...",
    "....SOO.CCCCCO.PPG..",
    ".....OO.CCCGCC.OO...",
    "......OCCCCCCO.......",
    "......OCCCCCCO.......",
    "......OCCGGCCO.......",
    "......OCCCCCCO.......",
    ".......OLLLLO........",
    ".......OLLLLO........",
    ".......OLLLLO........",
    "......OOL..LOO.......",
    ".....OOL....LOO......",
    "....OOL......LOO.....",
    "...OOL........LOO....",
    "..OO............OO...",
    "..OO............OO...",
];

const RUN_A: [&str; 24] = [
    "......OOO...........",
    ".....OOOOO..........",
    "....OOFFFFOO........",
    "....OFFFFFWWO.......",
    ".....OOFFFFOO.......",
    "......OOOOOO........",
    ".....OOO.OO.PPP.....",
    "..SSSOO.OOOO.PPPG...",
    "...SSOO.OCCCC.PPG...",
    "....SOO.CCCCCO.PPG..",
    ".....OO.CCCGCC.OO...",
    "......OCCCCCCO.......",
    "......OCCCCCCO.......",
    "......OCCGGCCO.......",
    "......OCCCCCCO.......",
    ".......OLLLLO........",
    "......OOLLLLO.........",
    ".....OOLLLLLO.........",
    "....OOL....LOO........",
    "...OOL......LOO.......",
    "..OOL........LOO......",
    "..OO..........LOO.....",
    "...............OO......",
    ".......................",
];

const RUN_B: [&str; 24] = [
    "......OOO...........",
    ".....OOOOO..........",
    "....OOFFFFOO........",
    "....OFFFFFWWO.......",
    ".....OOFFFFOO.......",
    "......OOOOOO........",
    ".....OOO.OO.PPP.....",
    "..SSSOO.OOOO.PPPG...",
    "...SSOO.OCCCC.PPG...",
    "....SOO.CCCCCO.PPG..",
    ".....OO.CCCGCC.OO...",
    "......OCCCCCCO.......",
    "......OCCCCCCO.......",
    "......OCCGGCCO.......",
    "......OCCCCCCO.......",
    ".......OLLLLO........",
    "........OLLLLO.......",
    ".........OLLLLO......",
    "......OOL....LOO.....",
    ".....OOL......LOO....",
    "....OOL........LOO...",
    "...OOL..........LOO..",
    "..OO...............OO.",
    "..OO...............OO.",
];

const TURN: [&str; 24] = [
    "........OOOOO.......",
    ".......OOOOOOO......",
    "......OOFFFFOO.......",
    "......OFFFFFWWO......",
    ".......OOFFFFOO......",
    "........OOOOOO.......",
    ".......OOO.OO.PPP....",
    "SSSSSS.OOOO..PPPG....",
    "...SSSO.CCCC.PPG....",
    "......O.CCCCCO.PPG...",
    "......O.CCCGCC.OO....",
    ".......OCCCCCCO.......",
    ".......OCCCCCCO.......",
    ".......OCCGGCCO.......",
    ".......OCCCCCCO.......",
    "........OLLLLO........",
    "........OLLLLO........",
    ".......OLLLLLO........",
    "......OOL...LOO.......",
    ".....OOL.....LOO......",
    "....OOL.......LOO.....",
    "...OOL.........LOO....",
    "..OO.............OO...",
    "..OO.............OO...",
];

const JUMP: [&str; 24] = [
    "....SSS..............",
    "...SSSSSOO...........",
    ".....OOOOOO..........",
    "....OOFFFFOO.........",
    "....OFFFFFWWO........",
    ".....OOFFFFOO........",
    "......OOOOOO..PPP....",
    "...SSSOO.OOOO.PPPG...",
    ".....SOO.OCCCC.PPG...",
    "......OO.CCCCCO.PPG..",
    ".......O.CCCGCC.OO...",
    ".......OCCCCCCO.......",
    "......OCCCCCCO........",
    ".....OCCCCCCCO........",
    ".....OCCCCCCCO........",
    "......OLLLLO..........",
    ".......OLLLLO.........",
    "........OLLLLO........",
    "........OLO.OO........",
    ".......OLO...OO.......",
    "......OLO.....OO......",
    ".....OO........OO.....",
    "....OO..........OO....",
    "...OO............OO...",
];

const FALL: [&str; 24] = [
    ".........SSS.........",
    "........SSSSSOO......",
    "......OOOOOO.........",
    ".....OOFFFFOO........",
    ".....OFFFFFWWO.......",
    "......OOFFFFOO.......",
    ".......OOOOOO..PPP...",
    "...SSSOO.OOOO.PPPG...",
    ".....SOO.OCCCC.PPG...",
    "......OO.CCCCCO.PPG..",
    ".......O.CCCGCC.OO...",
    ".......OCCCCCCO.......",
    ".......OCCCCCCO.......",
    "......OCCCCCCCO........",
    ".....OCCCCCCCCO........",
    "....OLLLLLLLLLLO.......",
    "...OOL.LLLLLLLLO........",
    "..OOL...LLLLLLLO.........",
    ".OOL.....LLLLLO...........",
    "OO........LLLLO............",
    "..........OL..LO...........",
    ".........OO....OO..........",
    "........OO......OO.........",
    ".......OO........OO........",
];

const GLIDE: [&str; 24] = [
    "SSSSSSSSS.......SSSSSSS",
    "....SSSSSSSOOOOSSSS.....",
    ".......OOOOOO...........",
    "......OOFFFFOO..........",
    "......OFFFFFWWO.........",
    ".......OOFFFFOO.........",
    "........OOOOOO..PPP.....",
    ".......OOO.OO.PPPG......",
    "....SSOO.OOOO.PPPG......",
    "..SSSSSO.OCCCC.PPG......",
    "....SSO.OCCCCCO.PPG.....",
    ".......OCCCCCCO..........",
    ".......OCCCCCCO..........",
    ".......OCCGGCCO..........",
    ".......OCCCCCCO..........",
    "........OLLLLO...........",
    ".........OLLLLO..........",
    ".........OLLLLO..........",
    "..........OL..LO.........",
    "..........OL..LO.........",
    ".........OO....OO........",
    "........OO......OO.......",
    "......OO............OO...",
    ".....OO..............OO..",
];

const DASH: [&str; 24] = [
    "....................",
    "....................",
    "......OOOOO.........",
    ".....OOFFFFOO........",
    ".....OFFFFFWWO.......",
    "......OOFFFFOO.......",
    ".......OOOOOO..PPP...",
    "SSSSSSOO.OOOO.PPPG...",
    "...SSSOO.CCCC.PPG....",
    "......OO.CCCCCOPPG...",
    ".......O.CCCGCCOO....",
    ".......OCCCCCCO.......",
    "......OCCCCCCO........",
    ".....OCCCCCCCO.........",
    "....OCCCCCCCCO.........",
    "...OLLLLLLLLLLO........",
    "..OOL.LLLLLLLLO.........",
    ".OOL...LLLLLLO...........",
    "OO......LLLLO.............",
    "........LLLLO.............",
    "........OL..LO............",
    ".......OO....OO...........",
    ".....OO..............OO...",
    "...OO..................OO.",
];

const SLIDE: [&str; 24] = [
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
    "......OOOOOO........",
    ".....OOFFFFOO........",
    "....OFFFFFWWO........",
    ".....OOFFFFOO....PPP.",
    "......OOOOOOO...PPPG.",
    "...SSOO.OCCCCCCOPPG..",
    ".SSSSSOOCCCCGGCCO....",
    "...SSOOCCCCCCCOOO....",
    "......OLLLLLLO........",
    ".....OLLLLLLLLO.......",
    "....OLLLLLLLLLLO......",
    "..OOL..........OO.....",
    ".OO..............OO....",
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
];

const INTERACT: [&str; 24] = [
    "......OOO...........",
    ".....OOOOO..........",
    "....OOFFFFOO........",
    "....OFFFFFWWO.......",
    ".....OOFFFFOO.......",
    "......OOOOOO........",
    ".....OOO.OO.PPP.....",
    "..SSSOO.OOOO.PPPG...",
    "...SSOO.OCCCC.PPG...",
    "....SOO.CCCCCO.PPG..",
    ".....OO.CCCGCC.OO...",
    "......OCCCCCCO.......",
    "......OCCCCCCO.......",
    "......OCCGGCCO.......",
    "......OCCCCCCO.......",
    ".......OLLLLO........",
    "......OLLLLLLO.......",
    ".....OLLLLILLLLO......",
    ".....OO....LOO.......",
    "....OO......LOO......",
    "...OO........LOO.....",
    "..OO..........OO.....",
    ".OO............OO....",
    "....................",
];

const HIT: [&str; 24] = [
    "........OOO.........",
    ".......OOOOO........",
    "......OOFFFFOO.......",
    "......OFFFFFWWO......",
    ".......OOFFFFOO......",
    "........OOOOOO.......",
    ".......OOO.OO.PPP....",
    ".SSSSSOO.OOOO.PPPG...",
    "...SSSOO.OCCCC.PPG...",
    "......OO.CCCCCO.PPG..",
    ".......O.CCCGCC.OO...",
    ".......OCCCCCCO.......",
    "......OCCCCCCO........",
    ".....OCCCCCCCO.........",
    "....OCCCCCCCCO.........",
    "...OOLLLLLLLLO.........",
    "..OOL.LLLLLLLO..........",
    ".OOL...LLLLLLO...........",
    "OO......LLLLO.............",
    "........LLLLO.............",
    "........OL..LO............",
    ".......OO....OO...........",
    "......OO......OO..........",
    ".....OO........OO.........",
];

const LAND: [&str; 24] = [
    "....................",
    "....................",
    "....OOOOOOOO........",
    "...OOFFFFFOOO.......",
    "...OFFFFFWWOO.......",
    "....OOFFFFOO.........",
    ".....OOOOOO..PPP.....",
    "..SSSOO.OOOO.PPPG...",
    "...SSOO.OCCCC.PPG...",
    "....SOO.CCCCCO.PPG..",
    ".....OO.CCCGCC.OO...",
    "......OCCCCCCO.......",
    "......OCCCCCCO.......",
    "......OCCGGCCO.......",
    "......OCCCCCCO.......",
    ".....OLLLLLLO.........",
    "....OLLLLLLLLO........",
    "...OLLLLLLLLLLO.......",
    "..OO..........OO......",
    ".OO............OO.....",
    "....................",
    "....................",
    "....................",
    "....................",
];

const RESPAWN: [&str; 24] = [
    "......OOO...........",
    ".....OOOOO..........",
    "....OOFFFFOO........",
    "....OFFFFFWWO.......",
    ".....OOFFFFOO.......",
    "......OOOOOO........",
    ".....OOO.OO.PPP.....",
    "..SSSOO.OOOO.PPPG...",
    "...SSOO.OCCCC.PPG...",
    "....SOO.CCCCCO.PPG..",
    ".....OO.CCCGCC.OO...",
    "......OCCCCCCO.......",
    "......OCCCCCCO.......",
    "......OCCGGCCO.......",
    "......OCCCCCCO.......",
    ".......OLLLLO........",
    ".......OLLLLO........",
    ".......OLLLLO........",
    "......OOL..LOO.......",
    ".....OOL....LOO......",
    "....OOL......LOO.....",
    "...OOL........LOO....",
    "..OO............OO...",
    "..OO............OO...",
];

#[cfg(test)]
mod tests {
    use super::render;
    use crate::game::pixel::{ColorTier, Rgb};
    use crate::game::prototype::{Pose, PrototypeGame};
    use crate::terminal::buffer::Color;
    use crate::ui::i18n::Locale;

    #[test]
    fn authored_scene_fits_small_and_large_terminal_frames() {
        for (width, height) in [(80, 24), (236, 53)] {
            let game = PrototypeGame::new(width, height);
            let frame = render(&game, width, height, Locale::En, ColorTier::TrueColor);
            assert_eq!((frame.width(), frame.height()), (width, height));
            assert!(frame.get(width / 2, 3).is_some_and(|cell| cell.ch == '▀'));
            assert!(frame
                .get(width / 2, height / 2)
                .is_some_and(|cell| matches!(cell.fg, Color::Rgb { .. })));
        }
    }

    #[test]
    fn pixel_sprite_palettes_are_not_ascii_line_art() {
        let frame = render(
            &PrototypeGame::new(80, 24),
            80,
            24,
            Locale::En,
            ColorTier::TrueColor,
        );
        let mut unique_colors = std::collections::HashSet::new();
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                if let Some(Color::Rgb { r, g, b }) = frame.get(x, y).map(|cell| cell.fg) {
                    unique_colors.insert(Rgb(r, g, b));
                }
            }
        }
        assert!(unique_colors.len() > 12);
    }

    #[test]
    fn checkpoint_beacon_stays_visible_in_the_large_scene() {
        let game = PrototypeGame::new(236, 53);
        let frame = render(&game, 236, 53, Locale::En, ColorTier::TrueColor);
        let light = Color::Rgb {
            r: 104,
            g: 153,
            b: 136,
        };
        let center = game.checkpoint_x().round() as u16;
        let ground = game.ground_y().round() as u16;
        let visible = (center.saturating_sub(2)..=center + 2).any(|x| {
            (ground.saturating_sub(22)..ground.saturating_sub(16)).any(|pixel_y| {
                frame
                    .get(x, pixel_y / 2 + 1)
                    .is_some_and(|cell| cell.fg == light || cell.bg == light)
            })
        });

        assert!(visible, "foreground scenery obscured the checkpoint beacon");
    }

    #[test]
    fn hud_uses_the_selected_terminal_color_tier() {
        for tier in [
            ColorTier::TrueColor,
            ColorTier::Color256,
            ColorTier::Ansi16,
            ColorTier::Ascii,
        ] {
            let frame = render(&PrototypeGame::new(80, 24), 80, 24, Locale::En, tier);
            let title = frame.get(1, 0).expect("title begins in the first column");
            assert_eq!(title.fg, tier.map_color(Rgb(220, 208, 173)));
            if tier != ColorTier::TrueColor {
                assert!(!matches!(title.fg, Color::Rgb { .. }));
            }
        }
    }

    #[test]
    fn ascii_fallback_keeps_the_traveler_visibly_distinct() {
        let game = PrototypeGame::new(80, 24);
        let frame = render(&game, 80, 24, Locale::En, ColorTier::Ascii);
        let x = game.player_x().round() as u16;
        let foot = (game.player_foot_y() * 0.5).round() as u16 + 1;
        let top = foot - 8;
        let mut rendered = String::new();
        for y in top..foot {
            for px in x..x + 10 {
                if let Some(cell) = frame.get(px, y) {
                    rendered.push(cell.ch);
                }
            }
        }

        for marker in ['o', '#', '~', '='] {
            assert!(
                rendered.contains(marker),
                "missing traveler marker {marker:?}"
            );
        }
    }

    #[test]
    fn traversal_states_have_distinct_authored_silhouettes() {
        assert_ne!(
            super::sprite_rows(Pose::Turn, 0.0),
            super::sprite_rows(Pose::Idle, 0.0)
        );
        assert_ne!(
            super::sprite_rows(Pose::Jump, 0.0),
            super::sprite_rows(Pose::Fall, 0.0)
        );
        assert_ne!(
            super::sprite_rows(Pose::Fall, 0.0),
            super::sprite_rows(Pose::Glide, 0.0)
        );
        assert_ne!(
            super::sprite_rows(Pose::Run, 0.0),
            super::sprite_rows(Pose::Dash, 0.0)
        );
    }

    #[test]
    fn visible_prototype_copy_uses_each_supported_locale() {
        for (locale, title, controls) in [
            (Locale::En, "The overgrown span", "Move"),
            (Locale::Fr, "La travée envahie", "Bouger"),
            (Locale::Es, "El viaducto invadido", "Mover"),
            (Locale::ZhCn, "被绿意覆盖的高架桥", "移动"),
        ] {
            let frame = render(
                &PrototypeGame::new(236, 53),
                236,
                53,
                locale,
                ColorTier::TrueColor,
            );
            let row = |y: u16| {
                (0..frame.width())
                    .filter_map(|x| {
                        frame
                            .get(x, y)
                            .filter(|cell| cell.width != 0)
                            .map(|cell| cell.ch)
                    })
                    .collect::<String>()
            };
            assert!(row(0).contains(title), "missing title in {locale:?}");
            assert!(row(52).contains(controls), "missing controls in {locale:?}");
        }
    }
}
