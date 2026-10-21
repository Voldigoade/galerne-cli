//! Completion dissolve/collapse transition animation when a task completes.

use std::thread;
use std::time::Duration;

use crate::terminal::buffer::{Color, Style};
use crate::terminal::engine::Engine;

/// Renders a short, polished ~250ms transition effect when the developer task terminates.
/// Skips immediately if the player presses any key.
pub fn run_completion_transition(
    engine: &mut Engine,
    success: bool,
    dist_gained: u32,
    shards_gained: u32,
) {
    let w = engine.width();
    let h = engine.height();
    let _mid_y = h / 2;

    let (title, title_color) = if success {
        ("◆ TASK COMPLETE ◆", Color::Green)
    } else {
        ("✕ TASK FAILED ✕", Color::Red)
    };

    let summary = format!("+{:<3}m DIST  |  +{:<2} SHARDS", dist_gained, shards_gained);
    let note = "progress saved";

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
        let buf = engine.buffer_mut();
        let box_w = 40u16.min(w.saturating_sub(4));
        let box_h = 7u16.min(h.saturating_sub(2));
        let start_x = (w.saturating_sub(box_w)) / 2;
        let start_y = (h.saturating_sub(box_h)) / 2;

        let frame_color = if success { Color::Cyan } else { Color::Red };
        let bg = Color::Reset;

        // Box border
        buf.set_char(start_x, start_y, '╔', frame_color, bg);
        for x in (start_x + 1)..(start_x + box_w - 1) {
            buf.set_char(x, start_y, '═', frame_color, bg);
        }
        buf.set_char(start_x + box_w - 1, start_y, '╗', frame_color, bg);

        for y in (start_y + 1)..(start_y + box_h - 1) {
            buf.set_char(start_x, y, '║', frame_color, bg);
            for x in (start_x + 1)..(start_x + box_w - 1) {
                buf.set_char(x, y, ' ', Color::Reset, bg);
            }
            buf.set_char(start_x + box_w - 1, y, '║', frame_color, bg);
        }

        buf.set_char(start_x, start_y + box_h - 1, '╚', frame_color, bg);
        for x in (start_x + 1)..(start_x + box_w - 1) {
            buf.set_char(x, start_y + box_h - 1, '═', frame_color, bg);
        }
        buf.set_char(
            start_x + box_w - 1,
            start_y + box_h - 1,
            '╝',
            frame_color,
            bg,
        );

        // Content
        let tx = start_x + (box_w.saturating_sub(title.len() as u16)) / 2;
        let sx = start_x + (box_w.saturating_sub(summary.len() as u16)) / 2;
        let nx = start_x + (box_w.saturating_sub(note.len() as u16)) / 2;

        buf.draw_str_styled(tx, start_y + 1, title, Style::new(title_color, bg).bold());
        buf.draw_str_styled(sx, start_y + 3, &summary, Style::new(Color::White, bg));
        buf.draw_str_styled(nx, start_y + 4, note, Style::new(Color::DarkGrey, bg));

        let _ = engine.flush();
    }

    let _ = check_skip_keypress(Duration::from_millis(170));
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
