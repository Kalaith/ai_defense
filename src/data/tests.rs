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
    }
}
