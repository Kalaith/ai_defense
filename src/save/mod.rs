//! Save/load persistence for campaign progress.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub wave_reached: u32,
    pub resources: SavedResources,
    pub population: SavedPopulation,
    pub threat: SavedThreat,
    pub sectors: Vec<SavedSector>,
    pub towers: Vec<SavedTower>,
    #[serde(default)]
    pub slots: Vec<SavedSlot>,
    #[serde(default)]
    pub buildings: Vec<SavedBuilding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedResources {
    pub power: f32,
    pub scrap: f32,
    pub data_cores: u32,
    #[serde(default)]
    pub water: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedPopulation {
    pub count: u32,
    pub morale: f32,
    pub health: f32,
    pub food_supply: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedThreat {
    pub energy: f32,
    pub heat: f32,
    pub data: f32,
    pub corruption: f32,
    pub noise: f32,
    pub territory: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSector {
    pub id: String,
    pub unlocked: bool,
    pub integrity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedTower {
    pub tower_id: String,
    pub x: f32,
    pub y: f32,
    pub level: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSlot {
    pub id: String,
    pub state: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedBuilding {
    pub id: String,
    pub state: String,
}

impl SaveData {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write("save.json", json)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string("save.json")?;
        let data: SaveData = serde_json::from_str(&json)?;
        Ok(data)
    }

    pub fn exists() -> bool {
        std::fs::metadata("save.json").is_ok()
    }
}
