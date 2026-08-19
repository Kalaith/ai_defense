use super::*;
use std::collections::HashSet;

#[test]
fn embedded_game_data_loads_required_content() {
    let data = GameData::load();

    assert!(!data.tower_defs.is_empty(), "expected tower definitions");
    assert!(!data.enemy_defs.is_empty(), "expected enemy definitions");
    assert!(!data.sector_defs.is_empty(), "expected sector definitions");
    assert!(
        !data.upgrade_defs.is_empty(),
        "expected upgrade definitions"
    );
    assert!(!data.map_def.slots.is_empty(), "expected map slots");
    assert!(!data.map_def.paths.is_empty(), "expected enemy paths");
    assert!(
        data.map_def.paths.iter().any(|path| path.initially_active),
        "expected at least one initially active path"
    );

    let mut tower_ids = HashSet::new();
    for tower in &data.tower_defs {
        assert!(
            tower_ids.insert(&tower.id),
            "duplicate tower id {}",
            tower.id
        );
        assert!(tower.cost_scrap >= 0.0);
        assert!(tower.base_range > 0.0);
        assert_eq!(
            tower.specializations.len(),
            2,
            "tower {} needs exactly two final branches",
            tower.id
        );
        let mut branch_ids = HashSet::new();
        for branch in &tower.specializations {
            assert!(
                branch_ids.insert(&branch.id),
                "duplicate branch {}",
                branch.id
            );
            assert!(branch.cost_scrap > 0.0);
            assert!(!branch.description.is_empty());
        }
    }

    let mut enemy_ids = HashSet::new();
    for enemy in &data.enemy_defs {
        assert!(
            enemy_ids.insert(&enemy.id),
            "duplicate enemy id {}",
            enemy.id
        );
        assert!(enemy.base_health > 0.0);
        assert!(enemy.speed > 0.0);
        assert!(
            !enemy.counter_hint.is_empty(),
            "enemy {} needs a counter-build hint",
            enemy.id
        );
        for multiplier in [
            enemy.damage_multipliers.ballistic,
            enemy.damage_multipliers.laser,
            enemy.damage_multipliers.emp,
            enemy.damage_multipliers.area_denial,
            enemy.damage_multipliers.subversion,
        ] {
            assert!(multiplier > 0.0, "enemy multipliers must stay positive");
        }
        assert!(
            [
                enemy.damage_multipliers.ballistic,
                enemy.damage_multipliers.laser,
                enemy.damage_multipliers.emp,
                enemy.damage_multipliers.area_denial,
                enemy.damage_multipliers.subversion,
            ]
            .iter()
            .any(|multiplier| (*multiplier - 1.0).abs() >= 0.2),
            "enemy {} needs a meaningful resistance or weakness",
            enemy.id
        );
    }

    let building_ids: HashSet<&str> = data
        .map_def
        .building_slots
        .iter()
        .map(|building| building.id.as_str())
        .collect();
    let mut sector_cores = HashSet::new();
    for sector in &data.sector_defs {
        if sector.starts_unlocked {
            continue;
        }
        let core = sector
            .core_building
            .as_deref()
            .unwrap_or_else(|| panic!("locked sector {} needs a core building", sector.id));
        assert!(
            building_ids.contains(core),
            "sector {} references missing core {core}",
            sector.id
        );
        assert!(
            sector_cores.insert(core),
            "multiple sectors reference core {core}"
        );
        assert!(
            sector.awakening_threat > 0.0,
            "locked sector {} needs an awakening trade-off",
            sector.id
        );
    }
}
