use galerne::game::runner::action::{ActionState, PlayerInput};
use galerne::game::runner::items::{ItemKind, PickupItem};
use galerne::game::runner::obstacles::{Obstacle, ObstacleKind};
use galerne::game::runner::parallax::ParallaxEngine;
use galerne::game::runner::state::RunnerState;
use galerne::game::runner::wind::TailwindTier;
use galerne::game::runner::world::{
    Chunk, ChunkWorld, Landmark, LandmarkKind, CHECKPOINTS, CHUNK_WIDTH,
};
use galerne::storage::schema::ItemKind as StoredItemKind;
use galerne::storage::schema::SaveState;
use galerne::terminal::buffer::Buffer;
use galerne::ui::dashboard::{render_inventory_panel, render_refuge_panel, render_workshop_panel};
use galerne::ui::i18n::{format_message, t, Locale, ALL_LOCALES};

fn buffer_to_string(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.height() {
        for x in 0..buf.width() {
            let cell = buf.get(x, y).unwrap();
            if cell.width != 0 {
                out.push(cell.ch);
            }
        }
        out.push('\n');
    }
    out
}

fn white_silhouette_rows(frame: &Buffer, ground_y: u16) -> usize {
    (ground_y.saturating_sub(14)..ground_y)
        .filter(|&y| {
            (0..frame.width()).any(|x| {
                frame.get(x, y).is_some_and(|cell| {
                    cell.ch != ' ' && cell.fg == galerne::terminal::buffer::Color::White
                })
            })
        })
        .count()
}

#[test]
fn restored_line_resumes_the_suspended_tram() {
    let mut dormant = Buffer::new(236, 53);
    ParallaxEngine::default().render_backdrop(&mut dormant, 48, false, false);
    assert!(!buffer_to_string(&dormant).contains('o'));

    let mut active = ParallaxEngine::default();
    let mut running = Buffer::new(236, 53);
    active.render_backdrop(&mut running, 48, true, false);
    let wheels_at_restored_start: Vec<_> = (0..running.height())
        .flat_map(|y| (0..running.width()).map(move |x| (x, y)))
        .filter(|(x, y)| running.get(*x, *y).is_some_and(|cell| cell.ch == 'o'))
        .collect();
    assert_eq!(wheels_at_restored_start.len(), 2);

    active.update(60.0);
    let mut moving = Buffer::new(236, 53);
    active.render_backdrop(&mut moving, 48, true, false);
    let wheels_in_motion: Vec<_> = (0..moving.height())
        .flat_map(|y| (0..moving.width()).map(move |x| (x, y)))
        .filter(|(x, y)| moving.get(*x, *y).is_some_and(|cell| cell.ch == 'o'))
        .collect();
    assert_eq!(wheels_in_motion.len(), 2);
    assert_ne!(wheels_at_restored_start, wheels_in_motion);
}

#[test]
fn authored_midground_changes_across_the_first_journey() {
    let mut panorama = ParallaxEngine::default();
    let mut opening = Buffer::new(236, 53);
    panorama.render_backdrop(&mut opening, 48, false, false);

    panorama.update(933.3333);
    panorama.mid_offset = 420.0;
    let mut later = Buffer::new(236, 53);
    panorama.render_backdrop(&mut later, 48, false, false);

    let midground_rows = 29..47;
    let opening_midground: Vec<_> = midground_rows
        .clone()
        .map(|y| {
            (0..opening.width())
                .map(|x| opening.get(x, y).unwrap().ch)
                .collect::<String>()
        })
        .collect();
    let later_midground: Vec<_> = midground_rows
        .map(|y| {
            (0..later.width())
                .map(|x| later.get(x, y).unwrap().ch)
                .collect::<String>()
        })
        .collect();

    assert!(
        opening_midground != later_midground,
        "later travel should reveal authored scenery rather than the opening midground again"
    );

    for (width, height) in [(80, 24), (236, 53)] {
        let mut game = RunnerState::new(width, height, 0x12345678);
        game.distance = 933.0;
        game.world.update(933.0);
        let mut frame = Buffer::new(width as u16, height as u16);
        game.render(&mut frame, Locale::En);
        let rendered = buffer_to_string(&frame);
        println!("--- VISUAL QA: HIGH SPAN AT {width}x{height} ---\n{rendered}");
        assert!(rendered.contains("NEXT STOP: Switch House"));
        if width == 80 {
            assert!(rendered.contains("___..----..___"));
        }
    }
}

#[test]
fn restoring_the_canal_gate_changes_the_hanging_garden_in_the_world_frame() {
    let mut closed = RunnerState::new(236, 53, 0x12345678);
    let mut flowing = RunnerState::new(236, 53, 0x12345678);
    let travel = 1545.0 - closed.player.x;
    closed.world.update(travel);
    flowing.world.update(travel);
    closed.distance = closed.world.scroll_x;
    flowing.distance = flowing.world.scroll_x;
    closed
        .world
        .ensure_chunks_generated(closed.world.scroll_x + 336.0);
    flowing
        .world
        .ensure_chunks_generated(flowing.world.scroll_x + 336.0);
    flowing.update(0.033, Some(PlayerInput::Interact));
    closed.update(0.033, None);

    let mut dry_frame = Buffer::new(236, 53);
    let mut flowing_frame = Buffer::new(236, 53);
    closed.render(&mut dry_frame, Locale::En);
    flowing.render(&mut flowing_frame, Locale::En);

    println!(
        "--- VISUAL QA: CANAL GARDEN FLOWING (236x53) ---\n{}",
        buffer_to_string(&flowing_frame)
    );
    assert_ne!(
        buffer_to_string(&dry_frame),
        buffer_to_string(&flowing_frame)
    );
    let stream_cell = (0..flowing_frame.height())
        .flat_map(|y| (0..flowing_frame.width()).map(move |x| (x, y)))
        .find(|(x, y)| {
            flowing_frame.get(*x, *y).is_some_and(|cell| {
                cell.ch == '>' && cell.fg == galerne::terminal::buffer::Color::Cyan
            })
        })
        .expect("restored canal should render a cyan water channel");
    assert_ne!(dry_frame.get(stream_cell.0, stream_cell.1).unwrap().ch, '>');
}

#[test]
fn large_terminal_uses_a_taller_authored_traveler_than_the_compact_view() {
    let mut compact = RunnerState::new(80, 24, 0x12345678);
    let mut large = RunnerState::new(236, 53, 0x12345678);
    compact.world.chunks.clear();
    large.world.chunks.clear();

    let mut compact_frame = Buffer::new(80, 24);
    let mut large_frame = Buffer::new(236, 53);
    compact.render(&mut compact_frame, Locale::En);
    large.render(&mut large_frame, Locale::En);

    let compact_rows = white_silhouette_rows(&compact_frame, compact.world.ground_y as u16);
    let large_rows = white_silhouette_rows(&large_frame, large.world.ground_y as u16);

    println!(
        "--- VISUAL QA: ROOMY TERMINAL TRAVELER (236x53) ---\n{}",
        buffer_to_string(&large_frame)
    );
    assert!(compact_rows <= 8);
    assert!(
        large_rows >= 9,
        "large silhouette only spans {large_rows} rows"
    );
    assert!(large_rows > compact_rows);
}

#[test]
fn roomy_terminal_traveler_keeps_art_scale_across_actions() {
    for action in [
        ActionState::Jumping {
            phase: galerne::game::runner::action::JumpPhase::Ascent,
            vertical_velocity: -1.0,
            elapsed_secs: 0.0,
        },
        ActionState::Jumping {
            phase: galerne::game::runner::action::JumpPhase::Apex,
            vertical_velocity: 0.0,
            elapsed_secs: 0.0,
        },
        ActionState::Jumping {
            phase: galerne::game::runner::action::JumpPhase::Descent,
            vertical_velocity: 1.0,
            elapsed_secs: 0.0,
        },
        ActionState::Gliding {
            elapsed_secs: 0.0,
            vertical_velocity: 0.0,
        },
        ActionState::Sliding {
            remaining_secs: 0.2,
        },
        ActionState::Dashing {
            remaining_secs: 0.1,
        },
    ] {
        let mut game = RunnerState::new(236, 53, 0x12345678);
        game.world.chunks.clear();
        game.player.action = action;
        let mut frame = Buffer::new(236, 53);
        game.render(&mut frame, Locale::En);
        println!(
            "--- VISUAL QA: ROOMY TRAVELER {action:?} ---\n{}",
            buffer_to_string(&frame)
        );

        let minimum_rows = if matches!(action, ActionState::Sliding { .. }) {
            4
        } else {
            9
        };
        assert!(
            white_silhouette_rows(&frame, game.world.ground_y as u16) >= minimum_rows,
            "traveler lost its large silhouette during {action:?}"
        );
    }
}

#[test]
fn compact_glide_renders_a_deployed_kite_over_the_traveler() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.world.chunks.clear();
    game.player.action = ActionState::Gliding {
        elapsed_secs: 0.0,
        vertical_velocity: 0.0,
    };
    let mut frame = Buffer::new(80, 24);

    game.render(&mut frame, Locale::En);

    let sprite_rows = game.player.current_sprite();
    let sprite = sprite_rows.join("\n");
    let (canopy_row, canopy_column) = sprite_rows
        .iter()
        .enumerate()
        .find_map(|(row, line)| line.find(".------.").map(|column| (row, column)))
        .unwrap();
    let rendered = buffer_to_string(&frame);
    println!("--- VISUAL QA: COMPACT GLIDE (80x24) ---\n{rendered}");
    assert!(
        sprite.contains(".------."),
        "compact glide needs a clear canopy"
    );
    assert!(
        sprite.contains("||"),
        "compact glide needs visible suspension lines"
    );
    assert!(
        rendered.contains("o>"),
        "compact glide must keep the profile visible"
    );
    assert!(
        rendered.contains("<#>"),
        "compact glide must keep the pack visible"
    );
    let canopy_x = game.player.x.round() as i32
        - galerne::game::runner::player::PLAYER_SPRITE_LEFT_OFFSET as i32
        + canopy_column as i32;
    let canopy_y = game.player.y.round() as i32 - (sprite_rows.len() - 1 - canopy_row) as i32;
    let canopy_cell = frame.get(canopy_x as u16, canopy_y as u16).unwrap();
    assert_eq!(canopy_cell.ch, '.');
    assert_eq!(
        canopy_cell.fg,
        galerne::terminal::buffer::Color::White,
        "the opened canopy must render as player art"
    );
}

#[test]
fn first_region_obstacles_render_at_both_terminal_sizes() {
    for kind in [
        ObstacleKind::LowBarrier,
        ObstacleKind::OverheadPipe,
        ObstacleKind::Pylon,
    ] {
        for (width, height, obstacle_x) in [(80, 24, 54.0), (236, 53, 180.0)] {
            let mut game = RunnerState::new(width, height, 0x12345678);
            let ground_y = game.world.ground_y as f32;
            game.world.chunks.clear();
            let chunk_index = (obstacle_x / CHUNK_WIDTH as f32).floor() as usize;
            let chunk_start_x = (chunk_index * CHUNK_WIDTH) as f32;
            let obstacle = Obstacle::new(obstacle_x - chunk_start_x, ground_y, kind);
            game.world.chunks.push(Chunk {
                index: chunk_index,
                obstacles: vec![obstacle.clone()],
                items: Vec::new(),
                wind_currents: Vec::new(),
                landmarks: Vec::new(),
                walkways: Vec::new(),
            });

            let mut frame = Buffer::new(width as u16, height as u16);
            game.render(&mut frame, Locale::En);

            println!(
                "--- VISUAL QA: {kind:?} ({width}x{height}) ---\n{}",
                buffer_to_string(&frame)
            );
            let base_y = match kind {
                ObstacleKind::OverheadPipe => (ground_y - 2.0) as usize,
                ObstacleKind::LowBarrier | ObstacleKind::Pylon => ground_y as usize,
            };
            let color = match kind {
                ObstacleKind::LowBarrier => galerne::terminal::buffer::Color::Red,
                ObstacleKind::OverheadPipe => galerne::terminal::buffer::Color::Cyan,
                ObstacleKind::Pylon => galerne::terminal::buffer::Color::Yellow,
            };
            for (row, line) in obstacle.sprite().iter().enumerate() {
                let y = base_y - (obstacle.sprite().len() - 1 - row);
                for (column, glyph) in line.chars().enumerate().filter(|(_, glyph)| *glyph != ' ') {
                    let cell = frame
                        .get((obstacle_x as usize + column) as u16, y as u16)
                        .unwrap();
                    assert_eq!(cell.ch, glyph);
                    assert_eq!(cell.fg, color);
                }
            }
        }
    }
}

#[test]
fn flow_feedback_stays_in_the_hud() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.flow_multiplier = 1.2;
    let mut frame = Buffer::new(80, 24);

    game.render(&mut frame, Locale::En);

    let rendered = buffer_to_string(&frame);
    println!("--- VISUAL QA: QUIET FLOW FEEDBACK (80x24) ---\n{rendered}");
    assert!(rendered.lines().next().unwrap().contains("FLOW x1.2"));
    assert!(game.floating_indicators.is_empty());
}

#[test]
fn cache_action_prompt_is_rendered_in_each_supported_locale() {
    let mut game = RunnerState::new(80, 24, 0x8765);
    let ground_y = game.world.ground_y as f32;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: Vec::new(),
        items: Vec::new(),
        wind_currents: Vec::new(),
        landmarks: vec![Landmark::new(
            game.player.x + 1.0,
            ground_y,
            LandmarkKind::SecretCache { opened: false },
        )],
        walkways: Vec::new(),
    });

    for (locale, expected) in [
        (Locale::En, "E  SEARCH"),
        (Locale::Fr, "E  CHERCHER"),
        (Locale::Es, "E  BUSCAR"),
        (Locale::ZhCn, "E  搜索"),
    ] {
        let mut buf = Buffer::new(80, 24);
        game.render(&mut buf, locale);
        assert!(
            buffer_to_string(&buf).contains(expected),
            "Missing contextual prompt for {locale:?}"
        );
        if locale == Locale::ZhCn {
            assert_eq!(buf.get(36, 2).unwrap().ch, 'E');
        }
    }
}

#[test]
fn cache_relics_stay_out_of_sight_until_the_cache_is_opened() {
    let mut game = RunnerState::new(80, 24, 0x8765);
    let ground_y = game.world.ground_y as f32;
    let item_x = game.player.x + 1.0;
    game.world.chunks.clear();
    game.world.chunks.push(Chunk {
        index: 0,
        obstacles: Vec::new(),
        items: vec![
            PickupItem::new(item_x, ground_y - 1.0, ItemKind::AncientCompass),
            PickupItem::new(item_x + 2.0, ground_y - 1.0, ItemKind::InductionFilament),
        ],
        wind_currents: Vec::new(),
        landmarks: vec![Landmark::new(
            item_x,
            ground_y,
            LandmarkKind::SecretCache { opened: false },
        )],
        walkways: Vec::new(),
    });

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);
    let rendered = buffer_to_string(&buf);

    assert!(!rendered.contains("[(*)]"));
    assert!(!rendered.contains("[~|~]"));
}

#[test]
fn restoring_the_line_does_not_invent_a_refuge_companion() {
    let mut save = SaveState::default();
    save.world_progress.relay_restored = true;
    save.refuge.power_restored = true;

    let mut buf = Buffer::new(100, 30);
    render_refuge_panel(&mut buf, &save, 0, 0, Locale::En);
    let rendered = buffer_to_string(&buf);

    assert!(rendered.contains("Kael: Roaming the high viaduct"));
    assert!(!rendered.contains("Kael: Stationed at the observation window"));
}

#[test]
fn player_sprite_negative_space_preserves_the_scene_behind_it() {
    let mut game = RunnerState::new(30, 24, 0x12345678);
    game.world.chunks.clear();

    let mut buf = Buffer::new(30, 24);
    for y in 0..buf.height() {
        for x in 0..buf.width() {
            buf.set_char(
                x,
                y,
                '+',
                galerne::terminal::buffer::Color::DarkGrey,
                galerne::terminal::buffer::Color::Black,
            );
        }
    }

    game.render(&mut buf, Locale::En);

    assert_eq!(buf.get(8, 13).unwrap().ch, '+');
}

#[test]
fn next_checkpoint_is_shown_in_each_supported_locale() {
    let game = RunnerState::new(80, 24, 0x12345678);
    let player_x = game.world.scroll_x + game.player.x;
    let (_, checkpoint_x, checkpoint_name) = CHECKPOINTS
        .iter()
        .find(|(id, _, _)| !game.unlocked_checkpoints.contains(id))
        .unwrap();
    let distance = (*checkpoint_x as f32 - player_x).ceil() as u32;

    for &locale in ALL_LOCALES {
        let expected = format_message(
            "next_checkpoint",
            locale,
            &[
                ("name", t(checkpoint_name, locale).to_owned()),
                ("distance", distance.to_string()),
            ],
        );
        let mut buf = Buffer::new(80, 24);
        game.render(&mut buf, locale);
        let rendered = buffer_to_string(&buf);
        assert!(
            rendered.contains(&expected),
            "next checkpoint should use locale {locale:?}"
        );
    }
}

#[test]
fn canal_gardens_is_the_next_destination_after_the_switch_house() {
    let mut game = RunnerState::new(120, 30, 0x12345678);
    game.distance = 1210.0;
    game.world.scroll_x = 1210.0;
    game.unlocked_checkpoints.extend([1, 2, 3, 4]);

    for &locale in ALL_LOCALES {
        let player_x = game.world.scroll_x + game.player.x;
        let distance = (1570.0 - player_x).ceil() as u32;
        let expected = format_message(
            "next_checkpoint",
            locale,
            &[
                ("name", t("cp_canal_gardens", locale).to_owned()),
                ("distance", distance.to_string()),
            ],
        );
        let mut buf = Buffer::new(120, 30);
        game.render(&mut buf, locale);
        assert!(
            buffer_to_string(&buf).contains(&expected),
            "canal gardens waypoint should be localized for {locale:?}"
        );
    }
}

#[test]
fn canal_crossing_shows_the_upper_route_pipe_and_waypoint() {
    for (width, height) in [(80, 24), (236, 53)] {
        let mut game = RunnerState::new(width, height, 0x12345678);
        let route_start = 1365.0;
        let scroll_x = route_start - game.player.x - 3.5;
        game.distance = scroll_x;
        game.world.scroll_x = scroll_x;
        game.world.parallax.update(scroll_x);
        game.world.ensure_chunks_generated(1571.0);
        game.unlocked_checkpoints.extend([1, 2, 3, 4]);
        game.relay_restored = true;

        let mut buf = Buffer::new(width as u16, height as u16);
        game.render(&mut buf, Locale::En);
        let rendered = buffer_to_string(&buf);
        let walkway_x = (route_start - scroll_x) as u16 + 30;
        let walkway_y = (game.world.ground_y - 5) as u16;

        println!("--- VISUAL QA: CANAL CROSSING AT {width}x{height} ---\n{rendered}");
        assert_eq!(buf.get(walkway_x, walkway_y).unwrap().ch, '=');
        assert!(rendered.contains("|=[o]==|"));
        assert!(rendered.contains("NEXT STOP: Canal gardens"));
    }
}

#[test]
fn fresh_minimum_sized_game_shows_the_first_destination() {
    let game = RunnerState::new(80, 24, 0x12345678);
    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    let rendered = buffer_to_string(&buf);
    println!("--- VISUAL QA: FIRST WAYPOINT (80x24) ---\n{}", rendered);
    assert!(rendered.contains("NEXT STOP: Span homes"));
}

#[test]
fn next_checkpoint_advances_when_the_current_one_is_reached() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.unlocked_checkpoints.push(1);

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    assert!(buffer_to_string(&buf).contains("NEXT STOP: Lift shaft door"));
}

#[test]
fn first_region_renders_a_raised_walkway_above_the_lower_track() {
    let mut game = RunnerState::new(236, 53, 0x12345678);
    game.distance = 160.0;
    game.world.update(160.0);
    game.player.y = game.world.ground_y as f32 - 5.0;

    let mut buf = Buffer::new(236, 53);
    game.render(&mut buf, Locale::En);

    println!(
        "--- VISUAL QA: RAISED SPAN (236x53) ---\n{}",
        buffer_to_string(&buf)
    );
    assert_eq!(buf.get(65, 43).unwrap().ch, '=');
}

#[test]
fn raised_walkway_remains_readable_at_minimum_terminal_size() {
    let mut game = RunnerState::new(80, 24, 0x12345678);
    game.distance = 160.0;
    game.world.update(160.0);
    game.player.y = game.world.ground_y as f32 - 5.0;

    let mut buf = Buffer::new(80, 24);
    game.render(&mut buf, Locale::En);

    println!(
        "--- VISUAL QA: RAISED SPAN (80x24) ---\n{}",
        buffer_to_string(&buf)
    );
    assert_eq!(buf.get(51, 14).unwrap().ch, '=');
}

#[test]
fn checkpoint_landmarks_match_the_waypoint_positions() {
    let mut world = ChunkWorld::new(0x12345678, 80, 24);
    world.ensure_chunks_generated(CHECKPOINTS.last().unwrap().1 as f32);

    for &(id, world_x, name_key) in &CHECKPOINTS {
        let chunk_index = world_x / CHUNK_WIDTH;
        let chunk = world
            .chunks
            .iter()
            .find(|chunk| chunk.index == chunk_index)
            .unwrap();
        let local_x = chunk
            .landmarks
            .iter()
            .find_map(|landmark| match &landmark.kind {
                LandmarkKind::CheckpointPylon {
                    id: landmark_id,
                    name_key: landmark_name,
                } if *landmark_id == id && landmark_name == name_key => Some(landmark.rel_x),
                _ => None,
            })
            .unwrap();

        assert_eq!(
            chunk_index as f32 * CHUNK_WIDTH as f32 + local_x,
            world_x as f32
        );
    }
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
    assert!(rendered.contains("<#>") && rendered.contains('~'));
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
    println!(
        "--- VISUAL QA: SETTLEMENT APPROACH (236x53) ---\n{}",
        rendered
    );

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
        let mut save = SaveState::default();
        render_refuge_panel(&mut buf, &save, 2, 2, locale);
        let rendered = buffer_to_string(&buf);
        assert!(
            rendered.contains(t("refuge_dormant_status", locale)),
            "refuge status should use locale {:?}",
            locale
        );
        if locale == Locale::En {
            println!("--- WORKBENCH LOCKED REFUGE (100x24) ---\n{rendered}");
        }
        assert!(rendered.contains(t("refuge_workbench_locked", locale)));
        assert!(rendered.contains("1/6"));

        save.inventory
            .add(galerne::storage::ItemKind::InductionFilament, 1);
        let mut repaired = Buffer::new(100, 24);
        render_refuge_panel(&mut repaired, &save, 2, 2, locale);
        let repaired = buffer_to_string(&repaired);
        if locale == Locale::En {
            println!("--- WORKBENCH REPAIRED REFUGE (100x24) ---\n{repaired}");
        }
        assert!(repaired.contains(t("refuge_workbench_status", locale)));
        assert!(!repaired.contains(t("refuge_workbench_locked", locale)));
    }
}

#[test]
fn locked_workshop_explains_what_opens_it_in_each_locale() {
    for &locale in ALL_LOCALES {
        let mut save = SaveState::default();
        let mut buf = Buffer::new(100, 24);
        render_workshop_panel(&mut buf, &save, 2, 2, locale, 0, false);
        let rendered = buffer_to_string(&buf);
        if locale == Locale::En {
            println!("--- WORKSHOP LOCKED (100x24) ---\n{rendered}");
        }

        assert!(rendered.contains(t("refuge_workbench_locked", locale)));
        assert!(rendered.contains(t("workbench_locked_hint", locale)));
        assert!(!rendered.contains(t("wind_shield", locale)));

        save.inventory
            .add(galerne::storage::ItemKind::InductionFilament, 1);
        let mut repaired = Buffer::new(100, 24);
        render_workshop_panel(&mut repaired, &save, 2, 2, locale, 0, false);
        let repaired = buffer_to_string(&repaired);
        assert!(repaired.contains(t("wind_shield", locale)));
        assert!(repaired.contains(t("workshop_help", locale)));
        assert!(!repaired.contains(t("refuge_workbench_locked", locale)));
    }
}

#[test]
fn inventory_items_use_localized_names_and_descriptions_in_each_locale() {
    let mut save = SaveState::default();
    let items = [
        StoredItemKind::GaleShard,
        StoredItemKind::ResonanceCore,
        StoredItemKind::InductionFilament,
        StoredItemKind::AncientCompass,
    ];
    for item in items {
        save.inventory.add(item, 1);
    }

    for &locale in ALL_LOCALES {
        for (width, height, px) in [(60, 20, 23), (80, 24, 27), (236, 53, 27)] {
            let mut buf = Buffer::new(width, height);
            render_inventory_panel(&mut buf, &save, px, 4, locale);
            let rendered = buffer_to_string(&buf);
            if locale == Locale::En {
                println!("--- INVENTORY PANEL ({width}x{height}) ---\n{rendered}");
            }
            let compact_rendered: String =
                rendered.chars().filter(|ch| !ch.is_whitespace()).collect();
            let title_key = if width < 80 || height < 24 {
                "menu_inventory"
            } else {
                "inventory_title"
            };
            let expected_title: String = t(title_key, locale)
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect();
            assert!(
                compact_rendered.contains(&expected_title),
                "Inventory heading {title_key} was clipped in {locale:?} at {width}x{height}"
            );

            for item in items {
                let mut keys = vec![item.name_key()];
                if width >= 80 && height >= 24 {
                    keys.push(item.desc_key());
                }
                for key in keys {
                    let expected: String = t(key, locale)
                        .chars()
                        .filter(|ch| !ch.is_whitespace())
                        .collect();
                    assert!(
                        compact_rendered.contains(&expected),
                        "Inventory text {key} was clipped in {locale:?} at {width}x{height}"
                    );
                }
            }
        }
    }
}
