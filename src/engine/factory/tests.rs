use super::*;

#[test]
fn powering_a_section_core_unlocks_it_once() {
    let defs = crate::data::loader::load_sector_defs();
    let mut factory = Factory::new();
    factory.init_sectors(&defs);

    let event = factory
        .unlock_from_core("building_11")
        .expect("logistics core should awaken a sector");

    assert_eq!(event.id, "logistics_hub");
    assert_eq!(event.role, "Life Support");
    assert!(event.threat > 0.0);
    assert!(factory.is_sector_unlocked("logistics_hub"));
    assert!(factory.unlock_from_core("building_11").is_none());
}

#[test]
fn old_powered_cores_reconcile_without_an_event() {
    let defs = crate::data::loader::load_sector_defs();
    let mut factory = Factory::new();
    factory.init_sectors(&defs);

    factory.sync_unlocked_cores(&["building_12".to_string(), "building_13".to_string()]);

    assert!(factory.is_sector_unlocked("robotics_bay"));
    assert!(factory.is_sector_unlocked("research_lab"));
}
