use galerne::game::runner::action::PlayerInput;
use galerne::game::runner::state::RunnerState;
use galerne::terminal::buffer::Buffer;
use galerne::ui::i18n::Locale;

#[test]
fn test_runner_state_initialization_and_forward_progression() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    assert_eq!(game.distance, 0.0);
    assert!(game.speed > 10.0);
    assert!(!game.is_game_over);

    // Simulate 1.0 second of automatic running
    for _ in 0..30 {
        game.update(0.033, None);
    }

    assert!(
        game.distance > 0.0,
        "Distance must advance automatically without key tapping"
    );
    assert!(
        game.world.scroll_x > 0.0,
        "World must scroll as player advances"
    );
}

#[test]
fn test_runner_onboarding_clears_upon_clearing_first_obstacles() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    assert!(!game.onboarding_jump_done);
    assert!(!game.onboarding_slide_done);

    // Prompt appears for jump when near chunk 1 obstacle
    while game.distance < 20.0 {
        game.update(0.033, None);
    }
    assert!(game.current_prompt().is_some());

    // Jump over first obstacle
    game.update(0.033, Some(PlayerInput::Jump));
    while game.distance < 35.0 {
        game.update(0.033, None);
    }
    assert!(
        game.onboarding_jump_done,
        "Jump onboarding must be marked completed after clearing hurdle"
    );

    // Advance to slide obstacle
    while game.distance < 60.0 {
        game.update(0.033, None);
    }
    game.update(0.033, Some(PlayerInput::Slide));
    while game.distance < 85.0 {
        game.update(0.033, None);
    }
    assert!(
        game.onboarding_slide_done,
        "Slide onboarding must be marked completed after sliding conduit"
    );
}

#[test]
fn test_runner_rendering_produces_non_empty_visual_frame() {
    let game = RunnerState::new(80, 24, 0x12345678);
    let mut buf = Buffer::new(80, 24);

    game.render(&mut buf, Locale::En);

    // Ensure screen is rich and not empty: count non-space cells
    let mut non_space_cells = 0;
    for y in 0..24 {
        for x in 0..80 {
            if let Some(cell) = buf.get(x, y) {
                if cell.ch != ' ' {
                    non_space_cells += 1;
                }
            }
        }
    }

    assert!(
        non_space_cells > 100,
        "Rendered frame must contain rich scenery, backdrop and sprites (found {} cells)",
        non_space_cells
    );
}

#[test]
fn test_runner_snapshot_deterministic_resumption() {
    let mut game1 = RunnerState::new(80, 24, 0x9999);
    for _ in 0..45 {
        game1.update(0.033, None);
    }

    let snapshot = game1.create_snapshot();

    let mut game2 = RunnerState::new(80, 24, 0);
    game2.resume_snapshot(&snapshot);

    assert_eq!(game2.distance, game1.distance);
    assert_eq!(game2.speed, game1.speed);
    assert_eq!(game2.shards_collected, game1.shards_collected);
    assert_eq!(game2.player.y, game1.player.y);
    assert_eq!(game2.world.scroll_x, game1.world.scroll_x);
}
