//! Factory sectors, awakening phases, and module management.

use crate::data::strings::text;
use crate::data::{EconomyConstants, SectorData, UpgradeDef};
use crate::engine::threat::ThreatKind;

#[derive(Clone, Debug)]
pub enum FactoryPhase {
    DormantRuin,
    StirringMachine,
    FunctionalSystem,
    SentientStructure,
    TranscendentEntity,
}

impl FactoryPhase {
    pub fn label(&self) -> &'static str {
        let phases = &text().factory_phases;
        match self {
            Self::DormantRuin => phases.dormant_ruin.as_str(),
            Self::StirringMachine => phases.stirring_machine.as_str(),
            Self::FunctionalSystem => phases.functional_system.as_str(),
            Self::SentientStructure => phases.sentient_structure.as_str(),
            Self::TranscendentEntity => phases.transcendent_entity.as_str(),
        }
    }
}

pub struct Sector {
    pub id: String,
    pub name: String,
    pub role: String,
    pub core_building: Option<String>,
    pub unlocked: bool,
    pub integrity: f32,
    pub max_integrity: f32,
    pub power_cost: f32,
    pub awakening_signature: ThreatKind,
    pub awakening_threat: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SectorAwakening {
    pub id: String,
    pub name: String,
    pub role: String,
    pub signature: ThreatKind,
    pub threat: f32,
}

pub struct Factory {
    pub phase: FactoryPhase,
    pub sectors: Vec<Sector>,
    pub purchased_upgrades: Vec<String>,
    pub difficulty_modifier: f32,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            phase: FactoryPhase::DormantRuin,
            sectors: Vec::new(),
            purchased_upgrades: Vec::new(),
            difficulty_modifier: 0.0,
        }
    }

    pub fn init_sectors(&mut self, sector_data: &[SectorData]) {
        self.sectors = sector_data
            .iter()
            .map(|sd| Sector {
                id: sd.id.clone(),
                name: sd.name.clone(),
                role: sd.role.clone(),
                core_building: sd.core_building.clone(),
                unlocked: sd.starts_unlocked,
                integrity: sd.max_integrity,
                max_integrity: sd.max_integrity,
                power_cost: sd.base_power_cost,
                awakening_signature: sd.awakening_signature,
                awakening_threat: sd.awakening_threat,
            })
            .collect();
    }

    pub fn unlocked_count(&self) -> usize {
        self.sectors.iter().filter(|s| s.unlocked).count()
    }

    pub fn is_sector_unlocked(&self, id: &str) -> bool {
        self.sectors.iter().any(|s| s.id == id && s.unlocked)
    }

    pub fn is_sector_active(&self, id: &str) -> bool {
        self.sectors
            .iter()
            .any(|s| s.id == id && s.unlocked && s.integrity > 0.0)
    }

    pub fn sector_for_core(&self, building_id: &str) -> Option<&Sector> {
        self.sectors
            .iter()
            .find(|sector| sector.core_building.as_deref() == Some(building_id))
    }

    /// Wake the sector controlled by a powered map core. The returned event is
    /// emitted only once, so loading or clicking an already-live core cannot
    /// repeatedly apply its awareness pulse.
    pub fn unlock_from_core(&mut self, building_id: &str) -> Option<SectorAwakening> {
        let sector = self
            .sectors
            .iter_mut()
            .find(|sector| sector.core_building.as_deref() == Some(building_id))?;
        if sector.unlocked {
            return None;
        }
        sector.unlocked = true;
        let event = SectorAwakening {
            id: sector.id.clone(),
            name: sector.name.clone(),
            role: sector.role.clone(),
            signature: sector.awakening_signature,
            threat: sector.awakening_threat,
        };
        self.check_awakening();
        Some(event)
    }

    /// Reconcile old saves that contain powered section cores but predate
    /// automatic sector awakening. This deliberately applies no threat pulse.
    pub fn sync_unlocked_cores(&mut self, powered_buildings: &[String]) {
        for sector in &mut self.sectors {
            if let Some(core) = &sector.core_building {
                if powered_buildings.iter().any(|id| id == core) {
                    sector.unlocked = true;
                }
            }
        }
        self.check_awakening();
    }

    pub fn power_generation(&self, economy: &EconomyConstants) -> f32 {
        let has_power_core = self.is_sector_active("power_core");
        if !has_power_core {
            return 0.0;
        }
        let other_unlocked = self
            .sectors
            .iter()
            .filter(|s| s.unlocked && s.integrity > 0.0 && s.id != "power_core")
            .count();
        economy.power_core_base_generation
            + other_unlocked as f32 * economy.power_per_other_unlocked_sector
    }

    pub fn power_consumption(&self) -> f32 {
        self.sectors
            .iter()
            .filter(|s| s.unlocked)
            .map(|s| s.power_cost)
            .sum()
    }

    pub fn check_awakening(&mut self) {
        let count = self.unlocked_count();
        self.phase = match count {
            0..=1 => FactoryPhase::DormantRuin,
            2 => FactoryPhase::StirringMachine,
            3 => FactoryPhase::FunctionalSystem,
            4..=5 => FactoryPhase::SentientStructure,
            _ => FactoryPhase::TranscendentEntity,
        };
    }

    pub fn has_upgrade(&self, id: &str) -> bool {
        self.purchased_upgrades.iter().any(|u| u == id)
    }

    pub fn can_purchase(&self, def: &UpgradeDef, scrap: f32, power: f32) -> bool {
        if self.has_upgrade(&def.id) {
            return false;
        }
        if !self.is_sector_unlocked(&def.sector) {
            return false;
        }
        for req in &def.requires {
            if !self.has_upgrade(req) {
                return false;
            }
        }
        scrap >= def.cost_scrap && power >= def.cost_power
    }

    pub fn prereqs_met(&self, def: &UpgradeDef) -> bool {
        def.requires.iter().all(|r| self.has_upgrade(r))
    }

    pub fn purchase_upgrade(&mut self, def: &UpgradeDef) {
        self.purchased_upgrades.push(def.id.clone());
        self.difficulty_modifier += def.difficulty_cost;
    }

    pub fn set_purchased_upgrades(&mut self, upgrades: Vec<String>, defs: &[UpgradeDef]) {
        self.purchased_upgrades = upgrades;
        self.difficulty_modifier = 0.0;
        for upgrade_id in &self.purchased_upgrades {
            if let Some(def) = defs.iter().find(|d| d.id == *upgrade_id) {
                self.difficulty_modifier += def.difficulty_cost;
            }
        }
    }

    /// Sum effect values for a given key across all purchased upgrades
    /// whose sector is currently active.
    pub fn upgrade_effect(&self, key: &str, defs: &[UpgradeDef]) -> f32 {
        let mut total = 0.0;
        for upgrade_id in &self.purchased_upgrades {
            if let Some(def) = defs.iter().find(|d| d.id == *upgrade_id) {
                if self.is_sector_active(&def.sector) {
                    if let Some(val) = def.effects.get(key) {
                        total += val;
                    }
                }
            }
        }
        total
    }
}

#[cfg(test)]
mod tests;
