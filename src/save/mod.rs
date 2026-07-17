//! Save/load persistence for campaign progress.

use macroquad_toolkit::persistence::SaveRoot;
use macroquad_toolkit::settings::GameSettings;
use serde::{Deserialize, Serialize};

const GAME_NAME: &str = "ai_defense";
const SAVE_FILE: &str = "save.json";
const SETTINGS_FILE: &str = "settings.json";

/// Persistent player settings + first-run state, stored separately from the
/// campaign save so they survive across runs. Volume/display fields live on
/// the shared [`GameSettings`]; the rest are genuinely game-specific.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub tutorial_seen: bool,
    pub game: GameSettings,
    pub default_fast_speed: bool,
    pub autosave: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tutorial_seen: false,
            game: GameSettings {
                master_volume: 0.8,
                sfx_volume: 0.8,
                ..GameSettings::default()
            },
            default_fast_speed: false,
            autosave: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        save_root()
            .and_then(|root| root.load_json(SETTINGS_FILE))
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        save_root()?.save_json(SETTINGS_FILE, self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub wave_reached: u32,
    pub resources: SavedResources,
    pub population: SavedPopulation,
    pub threat: SavedThreat,
    pub sectors: Vec<SavedSector>,
    #[serde(default)]
    pub purchased_upgrades: Vec<String>,
    pub towers: Vec<SavedTower>,
    #[serde(default)]
    pub slots: Vec<SavedSlot>,
    #[serde(default)]
    pub buildings: Vec<SavedBuilding>,
    /// Persistent sacrifice ledger and per-cycle assault escalation. Defaulted
    /// so saves written before these systems existed still load.
    #[serde(default)]
    pub survivors_evacuated: u32,
    #[serde(default)]
    pub machine_escalation: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedResources {
    pub power: f32,
    pub scrap: f32,
    pub data_cores: u32,
    #[serde(default)]
    pub water: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedPopulation {
    pub count: u32,
    pub morale: f32,
    pub health: f32,
    pub food_supply: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedThreat {
    pub energy: f32,
    pub heat: f32,
    pub data: f32,
    pub corruption: f32,
    pub noise: f32,
    pub territory: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedSector {
    pub id: String,
    pub unlocked: bool,
    pub integrity: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedTower {
    pub tower_id: String,
    pub x: f32,
    pub y: f32,
    pub level: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedSlot {
    pub id: String,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedBuilding {
    pub id: String,
    pub state: String,
}

impl SaveData {
    pub fn save(&self) -> Result<(), String> {
        save_root()?.save_json(SAVE_FILE, self)
    }

    pub fn load() -> Result<Self, String> {
        match save_root()?.load_json(SAVE_FILE) {
            Ok(data) => Ok(data),
            Err(toolkit_err) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Ok(json) = std::fs::read_to_string(SAVE_FILE) {
                        return serde_json::from_str(&json)
                            .map_err(|e| format!("Legacy save parse error: {e}"));
                    }
                }

                Err(toolkit_err)
            }
        }
    }

    pub fn exists() -> bool {
        let toolkit_save_exists = save_root()
            .map(|root| root.exists(SAVE_FILE))
            .unwrap_or(false);

        #[cfg(not(target_arch = "wasm32"))]
        {
            toolkit_save_exists || std::fs::metadata(SAVE_FILE).is_ok()
        }

        #[cfg(target_arch = "wasm32")]
        {
            toolkit_save_exists
        }
    }
}

fn save_root() -> Result<SaveRoot, String> {
    SaveRoot::app_data(GAME_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_save() -> SaveData {
        SaveData {
            version: 1,
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
        assert_eq!(restored.resources.water, 0.0);
    }
}
