use galerne::game::runner::action::{ActionState, InputBuffer, PlayerInput, PlayerInputWrapper};
use galerne::game::runner::items::{ItemKind, PickupItem};
use galerne::game::runner::region::{sector_for_distance, SectorKind};
use galerne::game::runner::state::RunnerState;
use galerne::game::runner::world::{Chunk, Landmark, LandmarkKind};
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
        let input = match game.current_prompt() {
            Some("prompt_greet_companion" | "prompt_search_cache" | "prompt_connect_line") => {
                Some(PlayerInput::Interact)
            }
            _ => None,
        };
        game.update(0.033, input);
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
fn jumping_onto_the_suspended_span_keeps_the_traveler_on_its_height() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.distance = 134.0;
    game.world.scroll_x = 134.0;

    for frame in 0usize..100 {
        game.player.invulnerable_timer = 10.0;
        let input = (frame == 0).then_some(PlayerInput::Jump);
        game.update(1.0 / 60.0, input);
    }

    assert_eq!(game.player.y, game.world.ground_y as f32 - 5.0);
    assert!(matches!(game.player.action, ActionState::Running { .. }));
    assert_eq!(game.player.shields, 2);
}

#[test]
fn lower_track_stays_available_under_the_suspended_span() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.distance = 134.0;
    game.world.scroll_x = 134.0;

    for _ in 0..100 {
        game.player.invulnerable_timer = 10.0;
        game.update(1.0 / 60.0, None);
    }

    assert_eq!(game.player.y, game.world.ground_y as f32);
    assert!(matches!(game.player.action, ActionState::Running { .. }));
    assert_eq!(game.player.shields, 1);
}

#[test]
fn raised_route_clears_rubble_that_crosses_the_lower_rail() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.distance = 190.0;
    game.world.scroll_x = 190.0;
    game.player.y = game.world.ground_y as f32 - 5.0;
    game.player.action = ActionState::Running {
        cycle_timer: 0.0,
        frame: 0,
    };

    for _ in 0..120 {
        game.update(1.0 / 60.0, None);
    }

    assert_eq!(game.player.shields, 1);
    assert!(game.player.y > game.world.ground_y as f32 - 5.0);
    assert!(game.player.y < game.world.ground_y as f32);
}

#[test]
fn leaving_the_suspended_span_returns_the_traveler_to_the_lower_track() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.distance = 215.0;
    game.world.scroll_x = 215.0;
    game.player.y = game.world.ground_y as f32 - 5.0;
    game.player.action = ActionState::Running {
        cycle_timer: 0.0,
        frame: 0,
    };

    for _ in 0..18 {
        game.update(1.0 / 60.0, None);
    }

    assert!(game.player.y > game.world.ground_y as f32 - 5.0);
    assert!(game.player.y < game.world.ground_y as f32);
    assert!(matches!(game.player.action, ActionState::Jumping { .. }));
}

#[test]
fn test_line_restoration_visual_sequence() {
    let mut game = RunnerState::new(120, 30, 0x12345678);

    for _ in 0..3000 {
        game.player.invulnerable_timer = 10.0;
        let input = match game.current_prompt() {
            Some("prompt_greet_companion" | "prompt_search_cache" | "prompt_connect_line") => {
                Some(PlayerInput::Interact)
            }
            _ => None,
        };
        game.update(0.033, input);
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
fn state_with_nearby_landmark(kind: LandmarkKind) -> RunnerState {
    let mut game = RunnerState::new(80, 24, 0x1324);
    let ground_y = game.world.ground_y as f32;
    let landmark_x = game.player.x + 1.0;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: Vec::new(),
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: vec![Landmark::new(landmark_x, ground_y, kind)],
        walkways: Vec::new(),
    });
    game
}

#[test]
fn a_cache_is_not_opened_or_looted_by_running_past() {
    let ground_y = 19.0;
    let mut game = state_with_nearby_landmark(LandmarkKind::SecretCache { opened: false });
    game.world.chunks[0].items.extend([
        PickupItem::new(
            game.player.x + 1.0,
            ground_y - 1.0,
            ItemKind::AncientCompass,
        ),
        PickupItem::new(
            game.player.x + 3.0,
            ground_y - 1.0,
            ItemKind::InductionFilament,
        ),
    ]);

    game.update(0.01, None);

    assert!(!game.secret_cache_found);
    assert!(game.collected_relics.is_empty());
    assert!(matches!(
        &game.world.chunks[0].landmarks[0].kind,
        LandmarkKind::SecretCache { opened: false }
    ));
}

#[test]
fn opening_the_cache_changes_its_visible_state() {
    let closed = Landmark::new(10.0, 19.0, LandmarkKind::SecretCache { opened: false });
    let open = Landmark::new(10.0, 19.0, LandmarkKind::SecretCache { opened: true });

    assert_ne!(closed.sprite(), open.sprite());
}

#[test]
fn nearby_cache_shows_a_contextual_action_prompt() {
    let game = state_with_nearby_landmark(LandmarkKind::SecretCache { opened: false });

    assert_eq!(game.current_prompt(), Some("prompt_search_cache"));
}

#[test]
fn interaction_works_when_pressed_on_the_first_prompt_frame() {
    let mut game = state_with_nearby_landmark(LandmarkKind::SecretCache { opened: false });
    game.world.chunks[0].landmarks[0].rel_x = game.player.x + 12.0;

    assert_eq!(game.current_prompt(), Some("prompt_search_cache"));

    game.update(0.033, Some(PlayerInput::Interact));

    assert!(game.secret_cache_found);
}

#[test]
fn interacting_with_the_cache_releases_its_relics() {
    let mut game = state_with_nearby_landmark(LandmarkKind::SecretCache { opened: false });
    let ground_y = game.world.ground_y as f32;
    game.world.chunks[0].items.extend([
        PickupItem::new(
            game.player.x + 1.0,
            ground_y - 1.0,
            ItemKind::AncientCompass,
        ),
        PickupItem::new(
            game.player.x + 3.0,
            ground_y - 1.0,
            ItemKind::InductionFilament,
        ),
    ]);

    game.update(0.01, Some(PlayerInput::Interact));

    assert!(game.secret_cache_found);
    assert!(game.collected_relics.contains(&ItemKind::AncientCompass));
    assert!(game.collected_relics.contains(&ItemKind::InductionFilament));
}

#[test]
fn a_switchhouse_does_not_reconnect_by_running_past() {
    let mut game = state_with_nearby_landmark(LandmarkKind::RelayNodeConsole { restored: false });

    game.update(0.01, None);

    assert!(!game.relay_restored);
    assert!(game.relay_activation_timer.is_none());
}

#[test]
fn interacting_with_the_switchhouse_reconnects_the_line() {
    let mut game = state_with_nearby_landmark(LandmarkKind::RelayNodeConsole { restored: false });

    game.update(0.01, Some(PlayerInput::Interact));

    assert!(game.relay_restored);
    assert!(game.relay_activation_timer.is_some());
}

#[test]
fn a_companion_is_not_met_by_running_past() {
    let mut game = state_with_nearby_landmark(LandmarkKind::CompanionKael { spoken: false });

    game.update(0.01, None);

    assert!(!game.companion_met);
    assert!(matches!(
        &game.world.chunks[0].landmarks[0].kind,
        LandmarkKind::CompanionKael { spoken: false }
    ));
}

#[test]
fn greeting_the_companion_marks_the_encounter_once() {
    let mut game = state_with_nearby_landmark(LandmarkKind::CompanionKael { spoken: false });

    game.update(0.01, Some(PlayerInput::Interact));
    game.update(0.01, Some(PlayerInput::Interact));

    assert!(game.companion_met);
    assert_eq!(game.floating_indicators.len(), 1);
}

#[test]
fn interaction_is_not_stored_as_a_movement_action() {
    let mut buffer = InputBuffer::default();

    buffer.push(PlayerInput::Jump);
    buffer.push(PlayerInput::Interact);

    assert_eq!(buffer.buffered_input, Some(PlayerInputWrapper::Jump));
    assert_eq!(buffer.peek(), Some(PlayerInput::Jump));
}
