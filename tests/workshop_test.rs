use galerne::storage::schema::{SaveState, WorkshopLevels};
use galerne::storage::workshop::{purchase_workshop_upgrade, WorkshopUpgradeType};

#[test]
fn test_workshop_upgrade_costs_and_purchase() {
    let mut state = SaveState {
        currency_shards: 50,
        ..Default::default()
    };

    // Level 0 cost is 10
    let cost = WorkshopUpgradeType::BaseDamage.cost(0);
    assert_eq!(cost, 10);

    let purchased = purchase_workshop_upgrade(&mut state, WorkshopUpgradeType::BaseDamage);
    assert!(purchased, "Purchase should succeed with 50 shards");
    assert_eq!(state.workshop.base_damage_level, 1);
    assert_eq!(state.currency_shards, 40);

    // Next level cost should scale
    let next_cost = WorkshopUpgradeType::BaseDamage.cost(1);
    assert_eq!(next_cost, 20);
}

#[test]
fn test_insufficient_shards_rejection() {
    let mut state = SaveState {
        currency_shards: 5,
        ..Default::default()
    };

    let purchased = purchase_workshop_upgrade(&mut state, WorkshopUpgradeType::BaseHealth);
    assert!(!purchased, "Purchase should fail when shards < cost");
    assert_eq!(state.workshop.base_health_level, 0);
    assert_eq!(state.currency_shards, 5);
}

#[test]
fn test_max_level_clamping() {
    let mut state = SaveState {
        currency_shards: 1000,
        workshop: WorkshopLevels {
            base_damage_level: 5,
            ..Default::default()
        },
        ..Default::default()
    };

    let purchased = purchase_workshop_upgrade(&mut state, WorkshopUpgradeType::BaseDamage);
    assert!(!purchased, "Cannot purchase beyond max level 5");
    assert_eq!(state.currency_shards, 1000);
}
