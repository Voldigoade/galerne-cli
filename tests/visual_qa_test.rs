use galerne::game::runner::action::{ActionState, PlayerInput};
use galerne::game::runner::state::RunnerState;
use galerne::game::runner::wind::TailwindTier;
use galerne::storage::schema::SaveState;
use galerne::terminal::buffer::Buffer;
use galerne::ui::dashboard::render_refuge_panel;
use galerne::ui::i18n::{t, Locale, ALL_LOCALES};

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
    assert!(
        sky_has_content,
        "Sky band must contain wind/atmospheric art"
    );

    // Distant horizon band (rows 9..24)
    let far_has_content = (9..24).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(
        far_has_content,
        "Far horizon must contain colossal tower/arch silhouettes"
    );

    // Midground band (rows 25..45)
    let mid_has_content = (25..45).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(mid_has_content, "Midground must contain shanties/cables");

    // Ground and pier drop (rows 46..52)
    let pier_has_content = (48..53).any(|y| lines[y].chars().any(|c| c != ' '));
    assert!(
        pier_has_content,
        "Lower region must contain viaduct trusses and pier drops"
    );
}

#[test]
fn test_visual_qa_wind_gliding_frame() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.player.action = ActionState::Gliding {
        elapsed_secs: 0.0,
        vertical_velocity: 0.0,
    };
    game.player.y = game.world.ground_y as f32 - 5.0;
    game.tailwind_tier = TailwindTier::Tier2;

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!(
        "--- VISUAL QA: WIND GLIDING FRAME (80x24) ---\n{}",
        rendered
    );
    assert!(rendered.contains("TW:II") || rendered.contains("FLOW"));
    assert!(rendered.contains('>') || rendered.contains('-') || rendered.contains('~'));
}

#[test]
fn test_visual_qa_interior_gantry_sector2_120x35() {
    let mut game = RunnerState::new(120, 35, 0x12345678);
    game.distance = 450.0;
    game.world.scroll_x = 450.0;
    game.world.ensure_chunks_generated(450.0 + 200.0);

    let mut buf = Buffer::new(120, 35);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!(
        "--- VISUAL QA: SECTOR 2 INTERIOR GANTRY (120x35) ---\n{}",
        rendered
    );
    assert_eq!(rendered.lines().count(), 35);
    assert!(rendered.contains("450m") || rendered.contains("DIST"));
}

#[test]
fn test_visual_qa_companion_roost_sector3_120x35() {
    let mut game = RunnerState::new(120, 35, 0x12345678);
    game.distance = 900.0;
    game.world.scroll_x = 900.0;
    game.world.ensure_chunks_generated(900.0 + 200.0);

    let mut buf = Buffer::new(120, 35);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!(
        "--- VISUAL QA: SECTOR 3 COMPANION ROOST (120x35) ---\n{}",
        rendered
    );
    assert_eq!(rendered.lines().count(), 35);
    assert!(rendered.contains("900m") || rendered.contains("DIST"));
}

#[test]
fn test_visual_qa_refuge_dormant_and_restored_80x24() {
    // 1. Dormant refuge
    let mut dormant_save = SaveState::default();
    dormant_save.world_progress.relay_restored = false;
    let mut buf_dormant = Buffer::new(80, 24);
    render_refuge_panel(&mut buf_dormant, &dormant_save, 2, 2, Locale::En);
    let rendered_dormant = buffer_to_string(&buf_dormant);
    println!(
        "--- VISUAL QA: REFUGE (DORMANT) (80x24) ---\n{}",
        rendered_dormant
    );
    assert!(rendered_dormant.contains("Status: quiet"));
    assert!(rendered_dormant.contains("[====]"));
    assert!(!rendered_dormant.contains("ABANDONED VIADUCT GANTRY"));

    // 2. Restored refuge
    let mut restored_save = SaveState::default();
    restored_save.world_progress.relay_restored = true;
    restored_save.companion.kael_at_refuge = true;
    restored_save.refuge.power_restored = true;
    let mut buf_restored = Buffer::new(80, 24);
    render_refuge_panel(&mut buf_restored, &restored_save, 2, 2, Locale::En);
    let rendered_restored = buffer_to_string(&buf_restored);
    println!(
        "--- VISUAL QA: REFUGE (RESTORED) (80x24) ---\n{}",
        rendered_restored
    );
    assert!(rendered_restored.contains("Status: lit"));
    assert!(rendered_restored.contains("[====]"));
    assert!(!rendered_restored.contains("POWERED VIADUCT GANTRY"));
    assert!(rendered_restored.contains("Kael: Stationed at the observation window"));
}

#[test]
fn test_refuge_panel_uses_selected_locale() {
    for &locale in ALL_LOCALES {
        let mut buf = Buffer::new(100, 24);
        render_refuge_panel(&mut buf, &SaveState::default(), 2, 2, locale);
        let rendered = buffer_to_string(&buf);
        assert!(
            rendered.contains(t("refuge_dormant_status", locale)),
            "refuge status should use locale {:?}",
            locale
        );
    }
}
