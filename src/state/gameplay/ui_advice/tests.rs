use super::*;
use crate::data::GameData;

#[test]
fn objective_recommends_scrap_converter_repair_when_affordable() {
    let data = GameData::load();
    let state = GameplayState::new(&data);

    let advice = state.build_ui_advice(&data);

    assert!(
        advice.suggested_action.label.contains("Scrap Converter"),
        "expected scrap converter repair, got {}",
        advice.suggested_action.label
    );
}

#[test]
fn objective_falls_back_to_tower_when_repairs_are_not_affordable() {
    let data = GameData::load();
    let mut state = GameplayState::new(&data);
    state.resources.scrap = 25.0;

    let advice = state.build_ui_advice(&data);

    assert!(matches!(
        advice.suggested_action.target,
        AdviceTarget::TowerDef(_)
    ));
}

#[test]
fn alert_builder_emits_power_food_and_water_thresholds() {
    let data = GameData::load();
    let mut state = GameplayState::new(&data);
    state.resources.power = 0.0;
    state.population.food_supply = 1.0;
    state.resources.water = 1.0;
    state.towers.push(crate::engine::tower::Tower::new(
        crate::engine::tower::TowerType::Ballistic,
        "ballistic_turret".to_string(),
        vec2(100.0, 100.0),
        120.0,
        10.0,
        1.0,
        200.0,
        25.0,
        WHITE,
    ));
    state.towers[0].is_active = false;

    let advice = state.build_ui_advice(&data);

    assert!(advice.alerts.iter().any(|a| a.label == "POWER FAILURE"));
    assert!(advice.alerts.iter().any(|a| a.label == "FOOD SHORTAGE"));
    assert!(advice.alerts.iter().any(|a| a.label == "WATER SHORTAGE"));
}

#[test]
fn placement_coverage_detects_active_path_intersection() {
    let data = GameData::load();
    let state = GameplayState::new(&data);
    let path = state.map_state.active_paths()[0];
    let center = path.points[0];

    let covered = state.covered_paths_for_range(center, 10.0);

    assert!(covered.iter().any(|id| id == &path.id));
}
