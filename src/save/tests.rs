use super::*;

fn sample_save() -> SaveData {
    SaveData {
        version: 6,
        wave_reached: 7,
        resources: SavedResources {
            power: 42.5,
            scrap: 130.0,
            data_cores: 2,
            water: 15.0,
        },
        population: SavedPopulation {
            count: 12,
            morale: 0.6,
            health: 0.9,
            food_supply: 22.0,
            workforce_policy: Some("salvage".to_string()),
        },
        threat: SavedThreat {
            energy: 3.0,
            heat: 1.5,
            data: 0.0,
            corruption: 4.2,
            noise: 6.6,
            territory: 0.8,
        },
        sectors: vec![SavedSector {
            id: "power_core".to_string(),
            unlocked: true,
            integrity: 80.0,
        }],
        purchased_upgrades: vec!["fast_repair".to_string()],
        towers: vec![SavedTower {
            tower_id: "ballistic".to_string(),
            x: 100.0,
            y: 200.0,
            level: 2,
            specialization_id: Some("rail_driver".to_string()),
            target_priority: Some("strongest".to_string()),
        }],
        slots: vec![SavedSlot {
            id: "slot_a".to_string(),
            state: "Powered".to_string(),
        }],
        buildings: vec![SavedBuilding {
            id: "scrap_converter".to_string(),
            state: "Repaired".to_string(),
        }],
        survivors_evacuated: 25,
        machine_escalation: 0.36,
        vault_takeover_active: true,
        vault_takeover_progress: 27.5,
        vault_upload_complete: false,
        depth_directives: vec![
            "reinforced_routes".to_string(),
            String::new(),
            "evacuation_relay".to_string(),
        ],
    }
}

/// A save written today must read back byte-for-byte through the same
/// JSON path `SaveRoot` uses — a mismatch here is a save-corruption bug,
/// the only genuinely fallible I/O in the codebase.
#[test]
fn save_data_round_trips_through_json() {
    let original = sample_save();
    let json = serde_json::to_string(&original).expect("serialize save data");
    let restored: SaveData = serde_json::from_str(&json).expect("deserialize save data");
    assert_eq!(original, restored);
}

/// Saves written before the evacuation ledger, escalation, slots, and
/// buildings existed must still load — those fields carry #[serde(default)]
/// precisely so an old save file doesn't strand a returning player.
#[test]
fn legacy_save_missing_newer_fields_loads_with_defaults() {
    let legacy_json = r#"{
        "version": 1,
        "wave_reached": 3,
        "resources": { "power": 10.0, "scrap": 5.0, "data_cores": 0 },
        "population": { "count": 8, "morale": 0.5, "health": 1.0, "food_supply": 10.0 },
        "threat": { "energy": 0.0, "heat": 0.0, "data": 0.0, "corruption": 0.0, "noise": 0.0, "territory": 0.0 },
        "sectors": [],
        "towers": []
    }"#;

    let restored: SaveData =
        serde_json::from_str(legacy_json).expect("legacy save should still deserialize");
    assert_eq!(restored.purchased_upgrades, Vec::<String>::new());
    assert_eq!(restored.slots, Vec::<SavedSlot>::new());
    assert_eq!(restored.buildings, Vec::<SavedBuilding>::new());
    assert_eq!(restored.survivors_evacuated, 0);
    assert_eq!(restored.machine_escalation, 0.0);
    assert!(!restored.vault_takeover_active);
    assert_eq!(restored.vault_takeover_progress, 0.0);
    assert!(!restored.vault_upload_complete);
    assert_eq!(restored.resources.water, 0.0);
    assert_eq!(restored.population.workforce_policy, None);
}
