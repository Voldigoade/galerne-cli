use galerne::game::runner::world::{ChunkWorld, CHUNK_WIDTH};

#[test]
fn test_chunk_generation_is_deterministic() {
    let seed = 0xabcdef1234567890;
    let mut world1 = ChunkWorld::new(seed, 80, 24);
    let mut world2 = ChunkWorld::new(seed, 80, 24);

    world1.update(100.0); // Advance 100 units
    world2.update(100.0);

    assert_eq!(world1.scroll_x, world2.scroll_x);
    assert_eq!(world1.active_chunks().len(), world2.active_chunks().len());

    for (c1, c2) in world1
        .active_chunks()
        .iter()
        .zip(world2.active_chunks().iter())
    {
        assert_eq!(c1.index, c2.index);
        assert_eq!(c1.obstacles.len(), c2.obstacles.len());
        assert_eq!(c1.items.len(), c2.items.len());
    }
}

#[test]
fn test_world_bounds_memory_and_prunes_past_chunks() {
    let mut world = ChunkWorld::new(0x42, 80, 24);

    // Run forward for a significant distance (3000 units = ~60 chunks)
    for _ in 0..100 {
        world.update(30.0);
    }

    // Number of active chunks should remain strictly bounded (e.g. <= 6 chunks)
    let active_count = world.active_chunks().len();
    assert!(
        active_count <= 6,
        "Active chunk count must be bounded to prevent memory growth (was {})",
        active_count
    );

    // Oldest chunk index must have advanced significantly beyond 0
    let oldest_index = world.active_chunks().first().map(|c| c.index).unwrap_or(0);
    assert!(
        oldest_index > 40,
        "Old chunks behind player must be pruned (oldest active index: {})",
        oldest_index
    );
}

#[test]
fn test_parallax_scroll_layers_move_at_distinct_rates() {
    let mut world = ChunkWorld::new(0x1337, 100, 30);
    world.update(50.0);

    let offset_sky = world.parallax.sky_offset;
    let offset_far = world.parallax.far_offset;
    let offset_mid = world.parallax.mid_offset;
    let offset_ground = world.scroll_x;

    assert!(
        offset_sky < offset_far,
        "Sky layer must scroll slower than far skyline"
    );
    assert!(
        offset_far < offset_mid,
        "Far layer must scroll slower than midground"
    );
    assert!(
        offset_mid < offset_ground,
        "Midground layer must scroll slower than ground track"
    );
}

#[test]
fn test_obstacles_maintain_minimum_reaction_gap() {
    let mut world = ChunkWorld::new(0x999, 120, 28);
    world.ensure_chunks_generated(500.0);

    // Check all generated obstacles across chunks
    let mut last_obstacle_x: Option<f32> = None;
    for chunk in world.active_chunks() {
        for obs in &chunk.obstacles {
            let abs_x = (chunk.index as f32 * CHUNK_WIDTH as f32) + obs.rel_x;
            if let Some(last_x) = last_obstacle_x {
                let gap = abs_x - last_x;
                assert!(
                    gap >= 12.0,
                    "Obstacles must maintain minimum reaction distance (found gap of {} at x={})",
                    gap,
                    abs_x
                );
            }
            last_obstacle_x = Some(abs_x);
        }
    }
}
