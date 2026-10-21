use galerne::game::runner::action::{ActionState, JumpPhase, PlayerInput};
use galerne::game::runner::player::RunnerPlayer;

#[test]
fn test_player_initial_state_is_running() {
    let ground_y = 18.0;
    let player = RunnerPlayer::new(10.0, ground_y);
    assert_eq!(player.x, 10.0);
    assert_eq!(player.y, ground_y);
    assert!(matches!(player.action, ActionState::Running { .. }));
    assert!(player.is_on_ground(ground_y));
}

#[test]
fn test_jump_trajectory_launch_apex_descent_landing() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);

    // Trigger jump
    player.handle_input(PlayerInput::Jump, ground_y);
    assert!(matches!(
        player.action,
        ActionState::Jumping {
            phase: JumpPhase::Ascent,
            ..
        }
    ));

    // Simulate ascent
    let mut reached_apex = false;
    let mut min_y = ground_y;

    for _ in 0..40 {
        player.update(0.033, ground_y);
        if player.y < min_y {
            min_y = player.y;
        }
        if matches!(
            player.action,
            ActionState::Jumping {
                phase: JumpPhase::Apex,
                ..
            }
        ) {
            reached_apex = true;
        }
    }

    assert!(reached_apex, "Jump should reach apex phase");
    assert!(
        min_y < ground_y - 4.0,
        "Jump should raise player by at least 4 terminal rows (min_y={})",
        min_y
    );

    // Simulate through landing
    for _ in 0..60 {
        player.update(0.033, ground_y);
    }

    // After completing trajectory, player should be safely on ground in Running state
    assert_eq!(player.y, ground_y);
    assert!(matches!(player.action, ActionState::Running { .. }));
}

#[test]
fn test_slide_duration_and_hitbox_reduction() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);

    let normal_height = player.hitbox_height();
    assert_eq!(normal_height, 4);

    player.handle_input(PlayerInput::Slide, ground_y);
    assert!(matches!(player.action, ActionState::Sliding { .. }));
    assert_eq!(
        player.hitbox_height(),
        2,
        "Slide must reduce hitbox height to 2 rows"
    );

    // After slide duration (~0.45s), player recovers to Running
    for _ in 0..20 {
        player.update(0.033, ground_y);
    }

    assert!(matches!(player.action, ActionState::Running { .. }));
    assert_eq!(
        player.hitbox_height(),
        4,
        "Normal hitbox height must be restored"
    );
}

#[test]
fn test_dash_speed_boost_and_cooldown() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);

    assert!(player.can_dash());
    player.handle_input(PlayerInput::Dash, ground_y);

    assert!(matches!(player.action, ActionState::Dashing { .. }));
    assert!(
        !player.can_dash(),
        "Cannot dash while dash is active or on cooldown"
    );

    // Simulate dash duration
    for _ in 0..10 {
        player.update(0.033, ground_y);
    }
    assert!(matches!(player.action, ActionState::Running { .. }));
    assert!(
        !player.can_dash(),
        "Cooldown must prevent immediate re-dash"
    );

    // Simulate cooldown period (~1.0s)
    for _ in 0..35 {
        player.update(0.033, ground_y);
    }
    assert!(
        player.can_dash(),
        "Dash should become available after cooldown"
    );
}

#[test]
fn test_coyote_time_allows_late_jump() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);

    // Simulate walking off a platform ledge (airborne without jumping)
    player.y = ground_y;
    player.start_falling_off_ledge();

    // 50ms into fall (within 100ms coyote time window), jump input should still succeed
    player.update(0.05, ground_y + 10.0);
    assert!(player.in_coyote_time());

    player.handle_input(PlayerInput::Jump, ground_y + 10.0);
    assert!(
        matches!(player.action, ActionState::Jumping { .. }),
        "Coyote time must allow jump"
    );
}

#[test]
fn test_input_buffering_triggers_jump_on_landing() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);

    // Initial jump
    player.handle_input(PlayerInput::Jump, ground_y);

    // Advance until descent (nearly landed, e.g. y = 16.5)
    while player.y > 14.0
        && matches!(
            player.action,
            ActionState::Jumping {
                phase: JumpPhase::Ascent,
                ..
            }
        )
    {
        player.update(0.033, ground_y);
    }
    while player.y < 16.5 {
        player.update(0.033, ground_y);
    }

    // Press Jump while still 1.5 units in the air (buffered input)
    player.handle_input(PlayerInput::Jump, ground_y);
    assert!(player.has_buffered_input(PlayerInput::Jump));

    // Touch down on ground
    while !player.is_on_ground(ground_y) {
        player.update(0.016, ground_y);
    }
    player.update(0.016, ground_y);

    // Buffered jump should immediately launch into a new Jump without requiring a second keypress!
    assert!(matches!(
        player.action,
        ActionState::Jumping {
            phase: JumpPhase::Ascent,
            ..
        }
    ));
}
