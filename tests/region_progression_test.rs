use galerne::game::runner::action::{ActionState, PlayerInput};
use galerne::game::runner::region::{sector_for_distance, SectorKind};
use galerne::game::runner::state::RunnerState;
use galerne::terminal::buffer::Buffer;
use galerne::ui::i18n::Locale;

#[test]
fn test_region_progression_unlocks_landmarks() {
    let mut game = RunnerState::new(120, 30, 0x12345678);

    assert_eq!(sector_for_distance(0.0), SectorKind::LowerViaduct);
    assert_eq!(sector_for_distance(450.0), SectorKind::InteriorGantry);
    assert_eq!(sector_for_distance(800.0), SectorKind::HighSkyTruss);
    assert_eq!(sector_for_distance(1200.0), SectorKind::SwitchHouse);

    assert_eq!(game.last_checkpoint_id, 0);

    for _ in 0..3000 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        if game.distance >= 1260.0 {
            break;
        }
    }

    assert!(
        game.distance >= 1250.0,
        "Player should reach the switch house, distance={}",
        game.distance
    );
    assert!(
        game.unlocked_checkpoints.contains(&4),
        "Checkpoint 4 must be unlocked"
    );
    assert!(
        game.companion_met,
        "The companion should be met in the upper district"
    );
    assert!(game.secret_cache_found, "The route cache should be found");
    assert!(
        game.relay_restored,
        "The line should be restored near the end of the region"
    );
}

#[test]
fn test_safe_checkpoint_respawn_preserves_inventory_and_progress() {
    let mut game = RunnerState::new(100, 25, 0x12345678);

    for _ in 0..500 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        if game.distance >= 265.0 {
            break;
        }
    }

    assert!(game.unlocked_checkpoints.contains(&1));
    assert_eq!(game.last_checkpoint_id, 1);
    let checkpoint_dist = game.last_checkpoint_distance;

    let saved_shards = game.shards_collected;
    assert!(
        saved_shards > 0,
        "Should have collected shards along the way"
    );

    game.player.shields = 0;
    game.player.invulnerable_timer = 0.0;
    game.handle_player_collision();

    assert!(
        !game.is_game_over,
        "Death must not result in hard Game Over"
    );
    assert_eq!(game.distance, checkpoint_dist);
    assert!(matches!(game.player.action, ActionState::Respawning { .. }));
    assert_eq!(game.player.shields, 1);
    assert!(game.player.invulnerable_timer > 0.0);
    assert_eq!(game.shards_collected, saved_shards);
    assert!(game.unlocked_checkpoints.contains(&1));
}

#[test]
fn test_wind_updraft_triggers_gliding() {
    let mut game = RunnerState::new(100, 25, 0x12345678);
    game.player.invulnerable_timer = 10.0;

    for _ in 0..300 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        let abs_x = game.world.scroll_x + game.player.x;
        if abs_x >= 170.0 {
            break;
        }
    }

    game.update(0.016, Some(PlayerInput::Jump));

    let mut entered_glide = false;
    for _ in 0..40 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        if matches!(game.player.action, ActionState::Gliding { .. }) {
            entered_glide = true;
            break;
        }
    }

    assert!(
        entered_glide,
        "Jumping into an Updraft vent must initiate aerodynamic Gliding"
    );
}

#[test]
fn test_line_restoration_visual_sequence() {
    let mut game = RunnerState::new(120, 30, 0x12345678);

    for _ in 0..3000 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        if game.distance >= 1260.0 {
            break;
        }
    }

    assert!(game.relay_restored);
    assert!(game.relay_activation_timer.is_some());

    let mut buf = Buffer::new(120, 30);
    game.render(&mut buf, Locale::En);

    let ground_y = game.world.ground_y;
    let surge_row: String = (0..buf.width())
        .map(|x| buf.get(x, (ground_y - 1) as u16).unwrap().ch)
        .collect();

    assert!(
        surge_row.contains('~') || surge_row.contains('*') || surge_row.contains('+'),
        "Restoration should animate across the track"
    );
}
