use galerne::game::runner::action::{ActionState, JumpPhase, PlayerInput};
use galerne::game::runner::items::{ItemKind, PickupItem};
use galerne::game::runner::obstacles::{Obstacle, ObstacleKind};
use galerne::game::runner::player::RunnerPlayer;

fn check_collision(player: &RunnerPlayer, obs: &Obstacle, chunk_start_x: f32) -> bool {
    let (p_x1, p_y1, p_x2, p_y2) = player.bounding_box();
    let (o_x1, o_y1, o_x2, o_y2) = obs.bounding_box(chunk_start_x);

    p_x1 < o_x2 && p_x2 > o_x1 && p_y1 < o_y2 && p_y2 > o_y1
}

#[test]
fn test_jump_clears_low_barrier() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);
    let barrier = Obstacle::new(10.0, ground_y, ObstacleKind::LowBarrier);

    // 1. Running upright on ground into barrier: collides
    assert!(
        check_collision(&player, &barrier, 0.0),
        "Player on ground must collide with low barrier"
    );

    // 2. Jumping at apex: clears low barrier
    player.action = ActionState::Jumping {
        phase: JumpPhase::Apex,
        vertical_velocity: 0.0,
        elapsed_secs: 0.3,
    };
    player.y = ground_y - 4.5; // Elevated in air

    assert!(
        !check_collision(&player, &barrier, 0.0),
        "Player at jump apex must cleanly clear low barrier"
    );
}

#[test]
fn test_slide_clears_overhead_pipe() {
    let ground_y = 18.0;
    let mut player = RunnerPlayer::new(10.0, ground_y);
    let pipe = Obstacle::new(10.0, ground_y, ObstacleKind::OverheadPipe);

    // 1. Running upright on ground into overhead pipe: collides (head hits pipe)
    assert!(
        check_collision(&player, &pipe, 0.0),
        "Upright player must hit overhead conduit"
    );

    // 2. Sliding on ground: crouched profile cleanly ducks under pipe
    player.handle_input(PlayerInput::Slide, ground_y);
    assert_eq!(player.hitbox_height(), 2);

    assert!(
        !check_collision(&player, &pipe, 0.0),
        "Sliding player must cleanly duck underneath overhead conduit"
    );
}

#[test]
fn test_instant_pickup_collection_without_blocking() {
    let ground_y = 18.0;
    let player = RunnerPlayer::new(10.0, ground_y);
    let (p_x1, p_y1, p_x2, p_y2) = player.bounding_box();

    let shard = PickupItem::new(11.0, ground_y - 1.0, ItemKind::GaleShard);
    let (i_x1, i_y1, i_x2, i_y2) = shard.bounding_box(0.0);

    let overlaps = p_x1 < i_x2 && p_x2 > i_x1 && p_y1 < i_y2 && p_y2 > i_y1;
    assert!(
        overlaps,
        "Player bounding box must overlap pickup item placed at player coordinates"
    );
}
