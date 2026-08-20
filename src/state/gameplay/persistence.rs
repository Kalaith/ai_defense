//! Save migration and serialization for the live gameplay state.

use crate::data::GameData;
use crate::engine::beacon::BeaconPhase;
use crate::engine::depth::DepthDirective;
use crate::engine::map::{BuildingState, SlotState};
use crate::engine::population::WorkforcePolicy;
use crate::engine::tower::Tower;
use crate::engine::wave::WaveManager;
use crate::save::{
    SaveData, SavedBuilding, SavedPopulation, SavedResources, SavedSector, SavedSlot, SavedThreat,
    SavedTower,
};
use macroquad::prelude::Vec2;

use super::helpers;
use super::lifecycle::wave_tuning;
use super::GameplayState;

impl GameplayState {
    pub fn from_save(data: &GameData, save: SaveData) -> Self {
        let mut state = GameplayState::new(data);
        state.apply_save(data, save);
        state
    }

    fn apply_save(&mut self, data: &GameData, save: SaveData) {
        self.current_wave = save.wave_reached;
        self.resources = super::Resources {
            power: save.resources.power,
            scrap: save.resources.scrap,
            data_cores: save.resources.data_cores,
            water: if save.version < 2 {
                data.constants.starting.water_supply
            } else {
                save.resources.water
            },
        };

        self.population.count = save.population.count;
        self.population.morale = save.population.morale;
        self.population.health = save.population.health;
        self.population.food_supply = save.population.food_supply;
        self.population.workforce_policy = save
            .population
            .workforce_policy
            .as_deref()
            .map(WorkforcePolicy::from_str)
            .unwrap_or_default();

        for (idx, directive) in save.depth_directives.iter().enumerate() {
            if let Some(parsed) = DepthDirective::from_str(directive) {
                if let Some(slot) = self.depth_directives.get_mut(idx) {
                    *slot = Some(parsed);
                }
            }
        }

        self.threat.energy = save.threat.energy;
        self.threat.heat = save.threat.heat;
        self.threat.data = save.threat.data;
        self.threat.corruption = save.threat.corruption;
        self.threat.noise = save.threat.noise;
        self.threat.territory = save.threat.territory;
        self.last_reaction_tier = self.threat.reaction_tier(&self.constants.threat);

        for saved in save.sectors {
            if let Some(sector) = self.factory.sectors.iter_mut().find(|s| s.id == saved.id) {
                sector.unlocked = saved.unlocked;
                sector.integrity = saved.integrity;
            }
        }
        self.factory
            .set_purchased_upgrades(save.purchased_upgrades, &self.upgrade_defs);
        self.factory.check_awakening();

        for saved_slot in &save.slots {
            if let Some(slot) = self
                .map_state
                .slots
                .iter_mut()
                .find(|s| s.id == saved_slot.id)
            {
                slot.state = SlotState::from_str(&saved_slot.state);
            }
        }
        for saved_building in &save.buildings {
            if let Some(building) = self
                .map_state
                .buildings
                .iter_mut()
                .find(|b| b.id == saved_building.id)
            {
                building.state = BuildingState::from_str(&saved_building.state);
            }
        }
        self.map_state.rebuild_unlocks();
        let powered_cores: Vec<String> = self
            .map_state
            .buildings
            .iter()
            .filter(|building| building.is_active())
            .map(|building| building.id.clone())
            .collect();
        self.factory.sync_unlocked_cores(&powered_cores);

        self.towers.clear();
        self.tower_stats.clear();
        for saved in save.towers {
            if let Some(def) = data.tower_def_by_id(&saved.tower_id) {
                let mut tower = Tower::new(
                    def.tower_type.clone(),
                    def.id.clone(),
                    Vec2::new(saved.x, saved.y),
                    def.base_range,
                    def.base_damage,
                    def.fire_rate,
                    def.cost_power,
                    def.cost_scrap,
                    def.color(),
                );
                helpers::apply_upgrade_levels(&mut tower, saved.level, &self.constants);
                if let Some(specialization_id) = &saved.specialization_id {
                    if let Some(specialization) = def
                        .specializations
                        .iter()
                        .find(|specialization| &specialization.id == specialization_id)
                    {
                        tower.specialize(specialization.id.clone(), specialization.effect);
                    }
                }
                if let Some(target_priority) = &saved.target_priority {
                    tower.target_priority =
                        crate::engine::tower::TargetPriority::from_str(target_priority);
                }
                let tower_idx = self.towers.len();
                for slot in self.map_state.slots.iter_mut() {
                    if (slot.position - Vec2::new(saved.x, saved.y)).length() < 2.0 {
                        slot.tower_index = Some(tower_idx);
                        break;
                    }
                }
                self.towers.push(tower);
                self.tower_stats.push(super::TowerUiStats::default());
            }
        }

        self.between_waves = true;
        self.wave_timer = self.wave_interval;
        self.wave_manager = WaveManager::new(wave_tuning(&self.constants));
        self.selected_tower = None;
        self.selected_slot = None;
        self.selected_building = None;
        self.selected_core = false;
        self.selected_upgrade = None;
        self.coach.active = false;
        self.show_intro = false;

        self.survivors_evacuated = save.survivors_evacuated;
        self.machine_escalation = save.machine_escalation;
        self.vault_takeover.active = save.vault_takeover_active;
        self.vault_takeover.progress = save.vault_takeover_progress;
        self.vault_takeover.upload_complete = save.vault_upload_complete;
        if self.vault_takeover.active {
            self.beacon_active = true;
            self.beacon_phase = BeaconPhase::TerminalHowl;
            self.between_waves = true;
            self.wave_timer = self.constants.gameplay.beacon_start_delay;
            self.beacon_start_difficulty_bonus = self.compute_beacon_start_difficulty_bonus()
                + self.machine_escalation
                + self.constants.vault.wave_budget_bonus;
        }
        let interval = self.constants.evacuation.milestone_interval.max(1);
        self.next_evac_milestone = (self.survivors_evacuated / interval + 1) * interval;

        self.update_beacon();
    }

    fn build_save_data(&self) -> SaveData {
        SaveData {
            version: 6,
            wave_reached: self.current_wave,
            resources: SavedResources {
                power: self.resources.power,
                scrap: self.resources.scrap,
                data_cores: self.resources.data_cores,
                water: self.resources.water,
            },
            population: SavedPopulation {
                count: self.population.count,
                morale: self.population.morale,
                health: self.population.health,
                food_supply: self.population.food_supply,
                workforce_policy: Some(self.population.workforce_policy.as_str().to_string()),
            },
            threat: SavedThreat {
                energy: self.threat.energy,
                heat: self.threat.heat,
                data: self.threat.data,
                corruption: self.threat.corruption,
                noise: self.threat.noise,
                territory: self.threat.territory,
            },
            sectors: self
                .factory
                .sectors
                .iter()
                .map(|s| SavedSector {
                    id: s.id.clone(),
                    unlocked: s.unlocked,
                    integrity: s.integrity,
                })
                .collect(),
            purchased_upgrades: self.factory.purchased_upgrades.clone(),
            towers: self
                .towers
                .iter()
                .map(|t| SavedTower {
                    tower_id: t.tower_id.clone(),
                    x: t.position.x,
                    y: t.position.y,
                    level: t.level,
                    specialization_id: t.specialization_id.clone(),
                    target_priority: Some(t.target_priority.as_str().to_string()),
                })
                .collect(),
            slots: self
                .map_state
                .slots
                .iter()
                .map(|s| SavedSlot {
                    id: s.id.clone(),
                    state: s.state.as_str().to_string(),
                })
                .collect(),
            buildings: self
                .map_state
                .buildings
                .iter()
                .map(|b| SavedBuilding {
                    id: b.id.clone(),
                    state: b.state.as_str().to_string(),
                })
                .collect(),
            survivors_evacuated: self.survivors_evacuated,
            machine_escalation: self.machine_escalation,
            vault_takeover_active: self.vault_takeover.active,
            vault_takeover_progress: self.vault_takeover.progress,
            vault_upload_complete: self.vault_takeover.upload_complete,
            depth_directives: self
                .depth_directives
                .iter()
                .map(|directive| {
                    directive.map_or_else(String::new, |value| value.as_str().to_string())
                })
                .collect(),
        }
    }

    pub(super) fn autosave(&self) {
        if !self.autosave_enabled {
            return;
        }
        let _ = self.build_save_data().save();
    }
}
