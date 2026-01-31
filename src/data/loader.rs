//! JSON data loading utilities.

use super::{EnemyDef, GameConstants, SectorData, TowerDef};

const CONSTANTS_JSON: &str = include_str!("../../assets/constants.json");
const TOWERS_JSON: &str = include_str!("../../assets/towers.json");
const ENEMIES_JSON: &str = include_str!("../../assets/enemies.json");
const SECTORS_JSON: &str = include_str!("../../assets/sectors.json");

pub fn load_constants() -> GameConstants {
    serde_json::from_str(CONSTANTS_JSON)
        .expect("Failed to load constants.json; game requires valid constants")
}

pub fn load_tower_defs() -> Vec<TowerDef> {
    match serde_json::from_str(TOWERS_JSON) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load towers.json: {e}");
            Vec::new()
        }
    }
}

pub fn load_enemy_defs() -> Vec<EnemyDef> {
    match serde_json::from_str(ENEMIES_JSON) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load enemies.json: {e}");
            Vec::new()
        }
    }
}

pub fn load_sector_defs() -> Vec<SectorData> {
    match serde_json::from_str(SECTORS_JSON) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load sectors.json: {e}");
            Vec::new()
        }
    }
}
