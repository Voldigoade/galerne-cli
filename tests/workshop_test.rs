use galerne::storage::schema::{RefugeProgress, SaveState, WorkshopLevels};
use galerne::storage::workshop::{purchase_workshop_upgrade, WorkshopUpgradeType};

#[test]
fn copper_braid_unlocks_the_workbench_and_locked_upgrades_cannot_be_bought() {
    let mut state = SaveState {
        currency_shards: 50,
        ..Default::default()
    };

    assert!(!purchase_workshop_upgrade(
        &mut state,
        WorkshopUpgradeType::ShieldCapacity
    ));
    assert_eq!(state.currency_shards, 50);
    assert_eq!(state.workshop.base_health_level, 0);

    state
        .inventory
        .add(galerne::storage::ItemKind::InductionFilament, 1);
    assert!(purchase_workshop_upgrade(
        &mut state,
        WorkshopUpgradeType::ShieldCapacity
    ));
    assert!(state.refuge.workbench_unlocked);
    assert_eq!(state.workshop.base_health_level, 1);
}

#[test]
fn copper_braid_unlocks_the_persistent_workbench_flag() {
    let mut state = SaveState::default();
    state
        .inventory
        .add(galerne::storage::ItemKind::InductionFilament, 1);

    state.unlock_workbench_from_inventory();

    assert!(state.refuge.workbench_unlocked);
    assert!(state.workbench_available());
}

#[test]
fn test_workshop_upgrade_costs_and_purchase() {
    let mut state = SaveState {
        currency_shards: 50,
        refuge: RefugeProgress {
            workbench_unlocked: true,
            ..Default::default()
        },
        ..Default::default()
    };

    // Level 0 cost is 10
    let cost = WorkshopUpgradeType::DashRecovery.cost(0);
    assert_eq!(cost, 10);

    let purchased = purchase_workshop_upgrade(&mut state, WorkshopUpgradeType::DashRecovery);
    assert!(purchased, "Purchase should succeed with 50 shards");
    assert_eq!(state.workshop.base_damage_level, 1);
    assert_eq!(state.currency_shards, 40);

    // Next level cost should scale
    let next_cost = WorkshopUpgradeType::DashRecovery.cost(1);
    assert_eq!(next_cost, 20);
}

#[test]
fn test_insufficient_shards_rejection() {
    let mut state = SaveState {
        currency_shards: 5,
        refuge: RefugeProgress {
            workbench_unlocked: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let purchased = purchase_workshop_upgrade(&mut state, WorkshopUpgradeType::ShieldCapacity);
    assert!(!purchased, "Purchase should fail when shards < cost");
    assert_eq!(state.workshop.base_health_level, 0);
    assert_eq!(state.currency_shards, 5);
}

#[test]
fn test_max_level_clamping() {
    let mut state = SaveState {
        currency_shards: 1000,
        refuge: RefugeProgress {
            workbench_unlocked: true,
            ..Default::default()
        },
        workshop: WorkshopLevels {
            base_damage_level: 5,
            ..Default::default()
        },
        ..Default::default()
    };

    let purchased = purchase_workshop_upgrade(&mut state, WorkshopUpgradeType::DashRecovery);
    assert!(!purchased, "Cannot purchase beyond max level 5");
    assert_eq!(state.currency_shards, 1000);
}

#[test]
fn test_apply_workshop_to_runner() {
    let workshop = WorkshopLevels {
        base_health_level: 2,
        base_damage_level: 2,
        move_speed_level: 2,
        magnet_radius_level: 2,
        shard_multiplier_level: 2,
    };
    let mut runner = galerne::game::runner::RunnerState::new(80, 24, 42);
    galerne::storage::workshop::apply_workshop_to_runner(&workshop, &mut runner);
    assert_eq!(runner.player.shields, 2); // 1 base + 1 from rank 2
    assert!(runner.speed > 18.0);
    assert!(runner.workshop_shard_multiplier > 1.0);
    assert_eq!(runner.flow_multiplier, 1.0);
}
