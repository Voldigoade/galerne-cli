use galerne::game::runner::action::{ActionState, PlayerInput};
use galerne::game::runner::region::{sector_for_distance, SectorKind};
use galerne::game::runner::state::RunnerState;
use galerne::terminal::buffer::Buffer;
use galerne::ui::i18n::Locale;

#[test]
fn test_authored_sectors_and_checkpoints() {
    let mut game = RunnerState::new(120, 30, 0x12345678);

    assert_eq!(sector_for_distance(0.0), SectorKind::LowerViaduct);
    assert_eq!(sector_for_distance(450.0), SectorKind::InteriorGantry);
    assert_eq!(sector_for_distance(800.0), SectorKind::HighSkyTruss);
    assert_eq!(sector_for_distance(1200.0), SectorKind::GaleNodeStation);

    // Initial state: starts at Checkpoint 0
    assert_eq!(game.last_checkpoint_id, 0);

    // Advance through the entire Cinder Line Viaduct (~1300m)
    for _ in 0..3000 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        if game.distance >= 1260.0 {
            break;
        }
    }

    assert!(
        game.distance >= 1250.0,
        "Player should reach the Gale Node Station, distance={}",
        game.distance
    );
    assert!(
        game.unlocked_checkpoints.contains(&4),
        "Checkpoint 4 (Relay Node) must be unlocked"
    );
    assert!(game.companion_met, "Companion Kael must be met in Sector 3");
    assert!(
        game.secret_cache_found,
        "Secret cache in Sector 2 must be found"
    );
    assert!(
        game.relay_restored,
        "Relay Station 04 must be restored at 1250m"
    );
}

#[test]
fn test_safe_checkpoint_respawn_preserves_inventory_and_progress() {
    let mut game = RunnerState::new(100, 25, 0x12345678);

    // Advance to Checkpoint 1 (at 250m)
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

    // Force shields to 0 and trigger collision directly
    game.player.shields = 0;
    game.player.invulnerable_timer = 0.0;
    game.handle_player_collision();

    // INVARIANTS:
    // 1. Player is NOT game over
    assert!(
        !game.is_game_over,
        "Death must not result in hard Game Over"
    );
    // 2. Player respawns at checkpoint distance
    assert_eq!(game.distance, checkpoint_dist);
    // 3. Player action is Respawning with fresh shield and invulnerability
    assert!(matches!(game.player.action, ActionState::Respawning { .. }));
    assert_eq!(game.player.shields, 1);
    assert!(game.player.invulnerable_timer > 0.0);
    // 4. Shards and progression are fully preserved!
    assert_eq!(game.shards_collected, saved_shards);
    assert!(game.unlocked_checkpoints.contains(&1));
}

#[test]
fn test_wind_updraft_triggers_gliding() {
    let mut game = RunnerState::new(100, 25, 0x12345678);
    game.player.invulnerable_timer = 10.0;

    // Advance until player reaches the Updraft vent in chunk 3 (168m to 182m)
    for _ in 0..300 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        let abs_x = game.world.scroll_x + game.player.x;
        if abs_x >= 170.0 {
            break;
        }
    }

    // Trigger jump into the updraft
    game.update(0.016, Some(PlayerInput::Jump));

    // Next few frames inside updraft should buoy player into Gliding
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
fn test_relay_restoration_visual_sequence() {
    let mut game = RunnerState::new(120, 30, 0x12345678);

    // Advance directly to the Relay Station console
    for _ in 0..3000 {
        game.player.invulnerable_timer = 10.0;
        game.update(0.033, None);
        if game.distance >= 1260.0 {
            break;
        }
    }

    assert!(game.relay_restored);
    assert!(game.relay_activation_timer.is_some());

    // Render frame during activation surge
    let mut buf = Buffer::new(120, 30);
    game.render(&mut buf, Locale::En);

    // Buffer must contain radiant activation pulses (~, *, +, #)
    let ground_y = game.world.ground_y;
    let surge_row: String = (0..buf.width())
        .map(|x| buf.get(x, (ground_y - 1) as u16).unwrap().ch)
        .collect();

    assert!(
        surge_row.contains('~') || surge_row.contains('*') || surge_row.contains('+'),
        "Surge row must display radiant energy waves during activation"
    );
}
