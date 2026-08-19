//! Save/load persistence for campaign progress.

use macroquad_toolkit::persistence::SaveRoot;
use macroquad_toolkit::settings::GameSettings;
use serde::{Deserialize, Serialize};

const GAME_NAME: &str = "last_assembly";
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
    #[serde(default)]
    pub vault_takeover_active: bool,
    #[serde(default)]
    pub vault_takeover_progress: f32,
    #[serde(default)]
    pub vault_upload_complete: bool,
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
    #[serde(default)]
    pub specialization_id: Option<String>,
    #[serde(default)]
    pub target_priority: Option<String>,
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
mod tests;
