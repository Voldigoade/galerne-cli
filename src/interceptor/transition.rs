//! Completion dissolve/collapse transition animation when a task completes.

use std::thread;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use crate::terminal::buffer::{Buffer, Color, Style};
use crate::terminal::engine::Engine;
use crate::ui::i18n::{format_message, t, Locale};

/// Renders a short, polished ~250ms transition effect when the developer task terminates.
/// Skips immediately if the player presses any key.
pub fn run_completion_transition(
    engine: &mut Engine,
    success: bool,
    dist_gained: u32,
    shards_gained: u32,
    locale: Locale,
) {
    let w = engine.width();
    let h = engine.height();

    // Frame 1: Glitch / dissolve random cells
    {
        let buf = engine.buffer_mut();
        let glitch_chars = ['░', '▒', '▓', '·', ' '];
        for y in 2..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                if (x + y * 7) % 3 == 0 {
                    let g_ch = glitch_chars[((x ^ y) as usize) % glitch_chars.len()];
                    buf.set_styled(x, y, g_ch, Style::new(Color::DarkGrey, Color::Reset));
                }
            }
        }
        let _ = engine.flush();
    }

    if check_skip_keypress(Duration::from_millis(80)) {
        return;
    }

    // Frame 2: Centered status banner
    {
        render_completion_banner(
            engine.buffer_mut(),
            success,
            dist_gained,
            shards_gained,
            locale,
        );
        let _ = engine.flush();
    }

    let _ = check_skip_keypress(Duration::from_millis(170));
}

fn render_completion_banner(
    buffer: &mut Buffer,
    success: bool,
    dist_gained: u32,
    shards_gained: u32,
    locale: Locale,
) {
    let width = buffer.width();
    let height = buffer.height();
    let box_w = 40u16.min(width.saturating_sub(4));
    let box_h = 7u16.min(height.saturating_sub(2));
    if box_w < 4 || box_h < 7 {
        return;
    }

    let start_x = (width - box_w) / 2;
    let start_y = (height - box_h) / 2;
    let frame_color = if success { Color::Cyan } else { Color::Red };
    let title_color = if success { Color::Green } else { Color::Red };
    let bg = Color::Reset;

    buffer.set_char(start_x, start_y, '╔', frame_color, bg);
    for x in (start_x + 1)..(start_x + box_w - 1) {
        buffer.set_char(x, start_y, '═', frame_color, bg);
    }
    buffer.set_char(start_x + box_w - 1, start_y, '╗', frame_color, bg);

    for y in (start_y + 1)..(start_y + box_h - 1) {
        buffer.set_char(start_x, y, '║', frame_color, bg);
        for x in (start_x + 1)..(start_x + box_w - 1) {
            buffer.set_char(x, y, ' ', Color::Reset, bg);
        }
        buffer.set_char(start_x + box_w - 1, y, '║', frame_color, bg);
    }

    buffer.set_char(start_x, start_y + box_h - 1, '╚', frame_color, bg);
    for x in (start_x + 1)..(start_x + box_w - 1) {
        buffer.set_char(x, start_y + box_h - 1, '═', frame_color, bg);
    }
    buffer.set_char(
        start_x + box_w - 1,
        start_y + box_h - 1,
        '╝',
        frame_color,
        bg,
    );

    let title = t(
        if success {
            "completion_success"
        } else {
            "completion_failure"
        },
        locale,
    );
    let distance = format_message(
        "completion_distance",
        locale,
        &[("distance", dist_gained.to_string())],
    );
    let shards = format_message(
        "completion_shards",
        locale,
        &[("shards", shards_gained.to_string())],
    );
    let saved = t("completion_saved", locale);

    draw_centered_in_box(
        buffer,
        start_x,
        box_w,
        start_y + 1,
        title,
        title_color,
        true,
    );
    draw_centered_in_box(
        buffer,
        start_x,
        box_w,
        start_y + 2,
        &distance,
        Color::White,
        false,
    );
    draw_centered_in_box(
        buffer,
        start_x,
        box_w,
        start_y + 3,
        &shards,
        Color::White,
        false,
    );
    draw_centered_in_box(
        buffer,
        start_x,
        box_w,
        start_y + 4,
        saved,
        Color::DarkGrey,
        false,
    );
}

fn draw_centered_in_box(
    buffer: &mut Buffer,
    start_x: u16,
    box_width: u16,
    y: u16,
    text: &str,
    color: Color,
    bold: bool,
) {
    let text_width = text.width().min(u16::MAX as usize) as u16;
    let x = start_x + (box_width.saturating_sub(text_width)) / 2;
    let style = Style::new(color, Color::Reset);
    buffer.draw_str_styled(x, y, text, if bold { style.bold() } else { style });
}

fn check_skip_keypress(duration: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        let remaining = duration.saturating_sub(start.elapsed());
        if let Ok(Some(_)) = Engine::poll_event(remaining.min(Duration::from_millis(20))) {
            return true;
        }
        thread::sleep(Duration::from_millis(5));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::render_completion_banner;
    use crate::terminal::buffer::Buffer;
    use crate::ui::i18n::Locale;

    #[test]
    fn completion_banner_uses_selected_locale_at_the_render_callsite() {
        for (locale, expected) in [
            (
                Locale::En,
                [
                    "TASK COMPLETE",
                    "Journey +45m",
                    "+3 shards",
                    "Progress saved",
                ],
            ),
            (
                Locale::Fr,
                [
                    "TÂCHE TERMINÉE",
                    "Parcours : +45 m",
                    "+3 éclats",
                    "Progression sauvegardée",
                ],
            ),
            (
                Locale::Es,
                [
                    "TAREA COMPLETADA",
                    "Recorrido: +45 m",
                    "+3 fragmentos",
                    "Progreso guardado",
                ],
            ),
            (
                Locale::ZhCn,
                ["任务完成", "行程：+45米", "+3 碎片", "进度已保存"],
            ),
        ] {
            let mut buffer = Buffer::new(60, 20);
            render_completion_banner(&mut buffer, true, 45, 3, locale);
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
                assert_eq!(buffer.get(26, 7).unwrap().ch, '任');
            }
        }
    }

    #[test]
    fn failed_task_banner_uses_the_failure_translation() {
        let mut buffer = Buffer::new(60, 20);
        render_completion_banner(&mut buffer, false, 12, 0, Locale::Es);
        let title_row = (0..buffer.width())
            .map(|x| buffer.get(x, 7).unwrap().ch)
            .collect::<String>();

        assert!(title_row.contains("TAREA FALLIDA"));
    }
}
