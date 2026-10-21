use galerne::game::runner::action::{ActionState, JumpPhase, PlayerInput};
use galerne::game::runner::items::{ItemKind, PickupItem};
use galerne::game::runner::obstacles::{Obstacle, ObstacleKind};
use galerne::game::runner::player::RunnerPlayer;
use galerne::game::runner::state::RunnerState;
use galerne::game::runner::world::{Chunk, Walkway};

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
fn overhead_pipe_art_matches_its_traversal_span() {
    let pipe = Obstacle::new(10.0, 18.0, ObstacleKind::OverheadPipe);
    let sprite = pipe.sprite();

    assert_eq!(pipe.width(), 8);
    assert_eq!(pipe.height(), 4);
    assert_eq!(sprite.len(), pipe.height());
    assert!(sprite
        .iter()
        .all(|line| line.chars().count() == pipe.width()));
    assert!(sprite.join("\n").contains("[o]"));
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

#[test]
fn clean_jump_builds_quiet_flow_after_clearing_a_barrier() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    let obstacle_x = game.player.x + 8.0;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: vec![Obstacle::new(
            obstacle_x,
            ground_y,
            ObstacleKind::LowBarrier,
        )],
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: Vec::new(),
    });

    game.update(0.033, Some(PlayerInput::Jump));
    while game.world.scroll_x < obstacle_x + 5.0 - game.player.x {
        game.update(0.033, None);
    }

    assert!(game.flow_multiplier > 1.0);
    assert!(game.flow_multiplier <= 1.2);
    assert!(game.floating_indicators.is_empty());
}

#[test]
fn clean_slide_builds_flow_after_a_pipe_clearance() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    let obstacle_x = game.player.x + 8.0;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: vec![Obstacle::new(
            obstacle_x,
            ground_y,
            ObstacleKind::OverheadPipe,
        )],
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: Vec::new(),
    });

    game.update(0.033, Some(PlayerInput::Slide));
    while game.world.scroll_x < obstacle_x + 8.0 - game.player.x {
        game.update(0.033, None);
    }

    assert!(game.flow_multiplier > 1.0);
    assert!(game.floating_indicators.is_empty());
}

#[test]
fn taking_the_raised_route_builds_flow_without_a_banner() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    let obstacle_x = game.player.x + 8.0;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: vec![Obstacle::new(
            obstacle_x,
            ground_y,
            ObstacleKind::LowBarrier,
        )],
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: vec![Walkway::new(0.0, 50.0, 5.0)],
    });
    game.player.y = ground_y - 5.0;

    while game.world.scroll_x < obstacle_x + 5.0 - game.player.x {
        game.update(0.033, None);
    }

    assert!(game.flow_multiplier > 1.0);
    assert!(game.floating_indicators.is_empty());
}

#[test]
fn a_quiet_stretch_lets_flow_fade_back_to_its_base() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    let obstacle_x = game.player.x + 8.0;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: vec![Obstacle::new(
            obstacle_x,
            ground_y,
            ObstacleKind::LowBarrier,
        )],
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: Vec::new(),
    });

    game.update(0.033, Some(PlayerInput::Jump));
    while game.world.scroll_x < obstacle_x + 5.0 - game.player.x {
        game.update(0.033, None);
    }
    assert!(game.flow_multiplier > 1.0);

    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 100,
        obstacles: Vec::new(),
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: Vec::new(),
    });
    game.speed = 0.0;
    game.update(4.1, None);

    assert_eq!(game.flow_multiplier, 1.0);
    assert_eq!(game.flow_timer_secs, 0.0);
}

#[test]
fn a_clean_sequence_caps_flow_and_a_hit_resets_it_without_a_flow_banner() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: [26.0, 32.0, 38.0, 44.0]
            .into_iter()
            .map(|x| Obstacle::new(x, ground_y, ObstacleKind::LowBarrier))
            .collect(),
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: vec![Walkway::new(0.0, 50.0, 5.0)],
    });
    game.player.y = ground_y - 5.0;

    while game.world.scroll_x < 31.0 {
        game.update(0.033, None);
    }

    assert_eq!(game.flow_multiplier, 1.2);
    assert!(game.floating_indicators.is_empty());

    game.handle_player_collision();

    assert_eq!(game.flow_multiplier, 1.0);
    assert_eq!(game.flow_timer_secs, 0.0);
    assert_eq!(game.floating_indicators.len(), 1);
}

#[test]
fn shard_collection_updates_the_hud_value_without_a_popup() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: Vec::new(),
        items: vec![PickupItem::new(
            game.player.x + 1.0,
            ground_y - 1.0,
            ItemKind::GaleShard,
        )],
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: Vec::new(),
    });

    game.update(0.01, None);

    assert_eq!(game.shards_collected, 10);
    assert!(game.floating_indicators.is_empty());
}

#[test]
fn resuming_past_an_obstacle_does_not_award_flow_for_it() {
    let mut game = RunnerState::new(80, 24, 42);
    let ground_y = game.world.ground_y as f32;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: vec![Obstacle::new(25.0, ground_y, ObstacleKind::LowBarrier)],
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: Vec::new(),
        walkways: Vec::new(),
    });
    game.distance = 40.0;
    game.world.scroll_x = 40.0;
    game.player.y = ground_y - 4.5;
    game.player.action = ActionState::Jumping {
        phase: JumpPhase::Apex,
        vertical_velocity: 0.0,
        elapsed_secs: 0.3,
    };

    game.update(0.033, None);

    assert_eq!(game.flow_multiplier, 1.0);
    assert_eq!(game.flow_timer_secs, 0.0);
}
