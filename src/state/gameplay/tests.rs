use super::*;
use crate::data::GameData;
use crate::engine::tower::TargetPriority;

#[test]
fn fresh_campaign_starts_with_complementary_sentries_without_spending_scrap() {
    let data = GameData::load();
    let state = GameplayState::new(&data);

    assert_eq!(state.resources.scrap, data.constants.starting.scrap);
    assert_eq!(state.towers.len(), 2);
    assert_eq!(state.towers[0].tower_id, "ballistic_turret");
    assert_eq!(state.towers[0].target_priority, TargetPriority::First);
    assert_eq!(state.towers[1].tower_id, "laser_emitter");
    assert_eq!(state.towers[1].target_priority, TargetPriority::Strongest);
}

#[test]
fn building_unlock_rules_gate_dependent_building_types() {
    let data = GameData::load();
    let mut state = GameplayState::new(&data);

    assert!(state.is_building_type_unlocked("logistics_hub_core"));
    assert!(!state.is_building_type_unlocked("scrap_converter"));

    state.factory.unlock_from_core("building_11");

    assert!(state.is_building_type_unlocked("scrap_converter"));
}
