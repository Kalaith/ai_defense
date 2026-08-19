use super::*;

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
