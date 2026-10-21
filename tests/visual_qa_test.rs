use galerne::game::runner::action::PlayerInput;
use galerne::game::runner::state::RunnerState;
use galerne::terminal::buffer::Buffer;
use galerne::ui::i18n::Locale;

fn buffer_to_string(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.height() {
        for x in 0..buf.width() {
            let cell = buf.get(x, y).unwrap();
            out.push(cell.ch);
        }
        out.push('\n');
    }
    out
}

#[test]
fn test_visual_qa_running_frame() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    // Simulate 0.5s of running
    for _ in 0..15 {
        game.update(0.033, None);
    }

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!("--- VISUAL QA: RUNNING FRAME (80x24) ---\n{}", rendered);

    // Verify HUD elements are visible at row 0
    assert!(rendered.contains("DIST"));
    assert!(rendered.contains("SPD"));
    assert!(rendered.contains("FLOW"));
    assert!(rendered.contains("SHARDS"));

    // Verify ground rails exist
    assert!(rendered.contains("================"));

    // Verify player scarf or silhouette exists
    assert!(rendered.contains('~') || rendered.contains('/') || rendered.contains('\\'));
}

#[test]
fn test_visual_qa_jump_apex_frame() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    // Jump command
    game.update(0.016, Some(PlayerInput::Jump));
    // Advance to jump apex (~0.35s)
    for _ in 0..20 {
        game.update(0.016, None);
    }

    assert!(
        game.player.y < (game.world.ground_y as f32 - 4.0),
        "Player must be airborne"
    );

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!("--- VISUAL QA: JUMP APEX FRAME ---\n{}", rendered);
    assert!(rendered.contains('^') || rendered.contains('>'));
}

#[test]
fn test_visual_qa_slide_frame() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    // Slide command
    game.update(0.016, Some(PlayerInput::Slide));
    game.update(0.10, None);

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!("--- VISUAL QA: SLIDE FRAME ---\n{}", rendered);
    // Sliding player sprite has lower profile with underscores/brackets
    assert!(rendered.contains('_') || rendered.contains(')'));
}

#[test]
fn test_visual_qa_wide_terminal_200_cols() {
    let mut game = RunnerState::new(200, 30, 0x12345678);
    for _ in 0..30 {
        game.update(0.033, None);
    }

    let mut buf = Buffer::new(200, 30);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    // Verify rails span across columns
    assert!(rendered.contains("================================"));
    // Ensure no panic or wrap artifact on wide viewport
    assert_eq!(rendered.lines().count(), 30);
}

#[test]
fn test_visual_qa_large_terminal_236x53() {
    let mut game = RunnerState::new(236, 53, 0x12345678);
    for _ in 0..30 {
        game.update(0.033, None);
    }

    let mut buf = Buffer::new(236, 53);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(lines.len(), 53);

    // Verify non-empty content across all vertical bands in 236x53
    // Sky band (rows 1..8)
    let sky_has_content = (1..8).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(sky_has_content, "Sky band must contain wind/atmospheric art");

    // Distant horizon band (rows 9..24)
    let far_has_content = (9..24).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(far_has_content, "Far horizon must contain colossal tower/arch silhouettes");

    // Midground band (rows 25..45)
    let mid_has_content = (25..45).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(mid_has_content, "Midground must contain shanties/cables");

    // Ground and pier drop (rows 46..52)
    let pier_has_content = (48..53).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(pier_has_content, "Lower region must contain viaduct trusses and pier drops");
}
