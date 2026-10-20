use galerne::game::arena::Arena;
use galerne::game::enemies::{Enemy, EnemyKind};
use galerne::game::player::Player;
use galerne::game::state::{GameInput, GameState};
use galerne::game::weapons::Weapon;

#[test]
fn test_arena_clamping() {
    let arena = Arena::new(40, 20);
    let (cx, cy) = arena.clamp_player(45.0, -5.0);
    assert!(cx <= 38.0);
    assert!(cy >= 1.0);
}

#[test]
fn test_player_movement_and_input() {
    let mut player = Player::new(20.0, 10.0);
    player.apply_input(GameInput {
        move_x: 1.0,
        move_y: 0.0,
    });
    player.update(0.1);
    assert!(player.x > 20.0, "Player moved right");
}

#[test]
fn test_weapon_auto_targeting_and_projectile_spawn() {
    let mut weapon = Weapon::gale_blaster();
    let player_pos = (10.0, 10.0);
    let enemies = vec![Enemy::new(15.0, 10.0, EnemyKind::Drifter)];

    // Force weapon cooldown to ready
    weapon.cooldown_remaining = 0.0;
    let proj = weapon.try_fire(player_pos, &enemies);

    assert!(
        proj.is_some(),
        "Weapon should fire at nearest enemy in range"
    );
    let p = proj.unwrap();
    assert_eq!(p.pos, player_pos);
    assert!(
        p.velocity.0 > 0.0,
        "Projectile moves towards enemy on the right"
    );
}

#[test]
fn test_enemy_archetypes_and_damage() {
    let mut enemy = Enemy::new(10.0, 10.0, EnemyKind::Drifter);
    assert_eq!(enemy.hp, 20.0);

    enemy.take_damage(15.0);
    assert_eq!(enemy.hp, 5.0);
    assert!(!enemy.is_dead());

    enemy.take_damage(10.0);
    assert!(enemy.is_dead());
}

#[test]
fn test_game_simulation_step() {
    let mut state = GameState::new(60, 25, 42); // Seeded RNG
    assert_eq!(state.player.hp, state.player.max_hp);

    // Run 10 ticks
    for _ in 0..10 {
        state.update(
            0.04,
            GameInput {
                move_x: 0.0,
                move_y: 0.0,
            },
        );
    }

    assert_eq!(state.wave, 1);
    assert!(!state.is_game_over());
}
