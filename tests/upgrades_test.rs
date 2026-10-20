use galerne::game::state::GameState;
use galerne::game::upgrades::{UpgradeDraft, UpgradeKind};

#[test]
fn test_upgrade_pool_generation() {
    let mut state = GameState::new(60, 25, 12345);
    let draft = UpgradeDraft::generate(&mut state.rng);

    assert_eq!(draft.options.len(), 3, "Draft must present 3 choices");
    // Ensure choices are distinct
    assert_ne!(draft.options[0], draft.options[1]);
    assert_ne!(draft.options[1], draft.options[2]);
    assert_ne!(draft.options[0], draft.options[2]);
}

#[test]
fn test_upgrade_application_effects() {
    let mut state = GameState::new(60, 25, 999);
    let base_damage = state.weapon.damage;
    let base_cooldown = state.weapon.cooldown;
    let base_hp = state.player.max_hp;

    state.apply_upgrade(UpgradeKind::DamageBoost);
    assert!(state.weapon.damage > base_damage, "Damage should increase");

    state.apply_upgrade(UpgradeKind::FireRateBoost);
    assert!(
        state.weapon.cooldown < base_cooldown,
        "Cooldown should decrease"
    );

    state.apply_upgrade(UpgradeKind::MaxHealth);
    assert!(state.player.max_hp > base_hp, "Max HP should increase");

    state.apply_upgrade(UpgradeKind::Multishot);
    assert_eq!(state.weapon.multishot, 2, "Multishot should increase to 2");
}

#[test]
fn test_level_up_triggers_pending_draft() {
    let mut state = GameState::new(60, 25, 42);
    assert!(!state.pending_level_up);

    // Give enough XP to level up
    state.player.add_xp(50);
    assert_eq!(state.player.level, 2);
}
