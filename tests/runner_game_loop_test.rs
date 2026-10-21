use galerne::game::runner::action::PlayerInput;
use galerne::game::runner::state::{RunnerSnapshot, RunnerState};
use galerne::game::runner::wind::TailwindTier;
use galerne::terminal::buffer::Buffer;
use galerne::ui::i18n::{t, Locale, ALL_LOCALES};

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
fn snapshots_drop_the_unused_score_and_still_read_older_snapshots() {
    let game = RunnerState::new(80, 24, 0x12345678);
    let snapshot = game.create_snapshot();
    let mut encoded = serde_json::to_value(&snapshot).unwrap();

    assert!(encoded.get("score").is_none());

    encoded["score"] = serde_json::json!(900);
    let old_snapshot: RunnerSnapshot = serde_json::from_value(encoded).unwrap();
    assert_eq!(old_snapshot.distance, snapshot.distance);
    assert_eq!(old_snapshot.shards_collected, snapshot.shards_collected);
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

#[test]
fn resumed_task_keeps_the_current_build_tailwind() {
    let mut previous_task = RunnerState::new(80, 24, 0x1111);
    previous_task.tailwind_tier = TailwindTier::Tier4;
    let mut old_snapshot = serde_json::to_value(previous_task.create_snapshot()).unwrap();
    old_snapshot["tailwind_tier"] = serde_json::json!("Tier4");
    old_snapshot["flow_multiplier"] = serde_json::json!(1.8);
    old_snapshot["flow_timer_secs"] = serde_json::json!(3.0);
    let snapshot = serde_json::from_value(old_snapshot).unwrap();

    let mut resumed_task = RunnerState::new(80, 24, 0x2222);
    resumed_task.tailwind_tier = TailwindTier::Tier2;
    resumed_task.workshop_shard_multiplier = 1.4;
    resumed_task.resume_snapshot(&snapshot);

    assert_eq!(resumed_task.tailwind_tier, TailwindTier::Tier2);
    assert_eq!(resumed_task.flow_multiplier, 1.0);
    assert_eq!(resumed_task.flow_timer_secs, 0.0);
}

#[test]
fn manual_resume_does_not_inherit_a_previous_build_tailwind() {
    let mut previous_task = RunnerState::new(80, 24, 0x1111);
    previous_task.tailwind_tier = TailwindTier::Tier4;
    let mut old_snapshot = serde_json::to_value(previous_task.create_snapshot()).unwrap();
    old_snapshot["tailwind_tier"] = serde_json::json!("Tier4");
    old_snapshot["flow_multiplier"] = serde_json::json!(1.8);
    old_snapshot["flow_timer_secs"] = serde_json::json!(3.0);
    let snapshot = serde_json::from_value(old_snapshot).unwrap();

    let mut manual_game = RunnerState::new(80, 24, 0x2222);
    manual_game.resume_snapshot(&snapshot);

    assert_eq!(manual_game.tailwind_tier, TailwindTier::None);
    assert_eq!(manual_game.flow_multiplier, 1.0);
    assert_eq!(manual_game.flow_timer_secs, 0.0);
}

#[test]
fn canal_sluice_opens_once_and_acknowledges_the_player_in_every_locale() {
    for &locale in ALL_LOCALES {
        let mut game = RunnerState::new(80, 24, 0x12345678);
        game.set_locale(locale);
        let travel = 1535.0 - game.player.x - 2.0;
        game.world.update(travel);
        game.world
            .ensure_chunks_generated(game.world.scroll_x + 100.0);

        assert_eq!(game.current_prompt(), Some("prompt_open_sluice"));
        let mut frame = Buffer::new(80, 24);
        game.render(&mut frame, locale);
        let prompt_row: String = (0..frame.width())
            .filter_map(|x| {
                frame
                    .get(x, 2)
                    .filter(|cell| cell.width != 0)
                    .map(|cell| cell.ch)
            })
            .collect();
        assert!(
            prompt_row.contains(t("prompt_open_sluice", locale)),
            "prompt row {prompt_row:?} should contain {:?} for {locale:?}",
            t("prompt_open_sluice", locale)
        );

        game.update(0.033, Some(PlayerInput::Interact));
        game.update(0.033, Some(PlayerInput::Interact));

        assert_eq!(game.current_prompt(), None);
        assert_eq!(
            game.floating_indicators
                .iter()
                .filter(|indicator| indicator.text == t("canal_flow_restored", locale))
                .count(),
            1
        );
    }
}

#[test]
fn canal_gate_state_survives_an_active_run_snapshot() {
    let mut opened_game = RunnerState::new(80, 24, 0x12345678);
    opened_game.canal_gate_opened = true;
    let snapshot = opened_game.create_snapshot();

    let mut resumed_game = RunnerState::new(80, 24, 0);
    resumed_game.resume_snapshot(&snapshot);

    assert!(resumed_game.canal_gate_opened);
}
