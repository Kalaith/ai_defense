//! Core gameplay state: wave defense, factory management, resource loop.

mod update;
mod render;
#[path = "../../ui/gameplay.rs"]
mod ui;
mod helpers;

use crate::data::{GameConstants, GameData, UpgradeDef};
use crate::engine::beacon::BeaconPhase;
use crate::engine::factory::Factory;
use crate::engine::map::MapState;
use crate::engine::population::Population;
use crate::engine::enemy::EnemyTuning;
use crate::engine::threat::{ReactionTier, ThreatSignature};
use crate::engine::tower::{ShotEffect, Tower, TowerType};
use crate::engine::wave::{WaveManager, WaveTuning};
use crate::save::{SaveData, SavedBuilding, SavedPopulation, SavedResources, SavedSector, SavedSlot, SavedThreat, SavedTower};
use macroquad::prelude::{Vec2, vec2};

pub struct GameplayState {
    pub constants: GameConstants,
    pub factory: Factory,
    pub wave_manager: WaveManager,
    pub threat: ThreatSignature,
    pub population: Population,
    pub resources: Resources,
    pub current_wave: u32,
    pub paused: bool,

    pub towers: Vec<Tower>,
    pub map_state: MapState,

    pub shot_effects: Vec<ShotEffect>,

    pub threat_flash_timer: f32,
    pub last_reaction_tier: ReactionTier,

    pub beacon_strength: f32,
    pub beacon_phase: BeaconPhase,

    pub scavengers_out: u32,
    pub scavengers_sent: u32,
    pub scavengers_returned: u32,
    pub scavengers_lost: u32,
    pub scavenger_scrap_gained: f32,
    pub scavenger_food_gained: f32,
    pub scavenger_population_gained: u32,
    pub scavenger_recall_active: bool,
    pub scavenger_recall_timer: f32,

    pub notifications: Vec<Notification>,

    pub particles: Vec<Particle>,
    pub wave_flash_timer: f32,
    pub last_wave_started: u32,
    pub time_scale: f32,

    // Wave timing
    pub wave_timer: f32,
    pub wave_interval: f32,
    pub between_waves: bool,

    // Placement
    pub placing_tower: Option<String>,
    pub selected_tower: Option<usize>,
    pub selected_slot: Option<usize>,
    pub selected_building: Option<usize>,
    pub selected_core: bool,
    pub selected_upgrade: Option<String>,

    // Loss tracking
    pub factory_integrity: f32,

    pub base_health_scale_per_wave: f32,
    pub base_food_per_wave: f32,

    pub shutdown_triggered: bool,
    pub beacon_active: bool,

    pub upgrade_defs: Vec<UpgradeDef>,
    pub beacon_start_difficulty_bonus: f32,
    pub unlocks: crate::data::UnlocksDef,

    // Camera
    pub camera_offset: Vec2,
    pub camera_zoom: f32,
    pub prev_mouse_pos: Vec2,
}

pub struct Resources {
    pub power: f32,
    pub scrap: f32,
    pub data_cores: u32,
}

pub struct Notification {
    pub text: String,
    pub ttl: f32,
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub ttl: f32,
}

impl GameplayState {
    pub fn new(data: &GameData) -> Self {
        let constants = data.constants.clone();
        let map_state = MapState::from_def(data.map_def.clone());

        let mut factory = Factory::new();
        factory.init_sectors(&data.sector_defs);

        let threat = ThreatSignature::new();
        let last_reaction_tier = threat.reaction_tier();

        Self {
            constants,
            factory,
            wave_manager: WaveManager::new(WaveTuning {
                spawn_interval: data.constants.waves.spawn_interval,
                commander_aura_radius: data.constants.enemy.commander_aura_radius,
                commander_aura_speed_mult: data.constants.enemy.commander_aura_speed_mult,
                enemy_tuning: EnemyTuning {
                    scout_dodge_chance: data.constants.enemy.scout_dodge_chance,
                    scout_dodge_duration: data.constants.enemy.scout_dodge_duration,
                    hit_flash_duration: data.constants.enemy.hit_flash_duration,
                    saboteur_skip_chance: data.constants.enemy.saboteur_skip_chance,
                    slow_multiplier: data.constants.enemy.slow_multiplier,
                },
                wave_budget_base: data.constants.waves.budget_base,
                wave_budget_per_wave: data.constants.waves.budget_per_wave,
                wave_commander_every: data.constants.waves.commander_every,
                threat_budget_divisor: data.constants.threat.budget_divisor,
                threat_health_mult_per_awareness: data.constants.threat.health_mult_per_awareness,
            }),
            threat,
            population: Population::new(&data.constants),
            resources: Resources {
                power: data.constants.starting.power,
                scrap: data.constants.starting.scrap,
                data_cores: 0,
            },
            current_wave: 0,
            paused: false,

            towers: Vec::new(),
            map_state,

            shot_effects: Vec::new(),

            threat_flash_timer: 0.0,
            last_reaction_tier,

            beacon_strength: 0.0,
            beacon_phase: BeaconPhase::WarmSignal,

            scavengers_out: 0,
            scavengers_sent: 0,
            scavengers_returned: 0,
            scavengers_lost: 0,
            scavenger_scrap_gained: 0.0,
            scavenger_food_gained: 0.0,
            scavenger_population_gained: 0,
            scavenger_recall_active: false,
            scavenger_recall_timer: 0.0,

            notifications: Vec::new(),

            particles: Vec::new(),
            wave_flash_timer: 0.0,
            last_wave_started: 0,
            time_scale: 1.0,

            wave_timer: data.constants.ui.wave_start_delay,
            wave_interval: data.constants.waves.interval_seconds,
            between_waves: true,

            placing_tower: None,
            selected_tower: None,
            selected_slot: None,
            selected_building: None,
            selected_core: false,
            selected_upgrade: None,

            factory_integrity: 100.0,

            base_health_scale_per_wave: data.constants.waves.health_scale_per_wave,
            base_food_per_wave: data.constants.waves.food_per_wave,

            shutdown_triggered: false,
            beacon_active: false,

            upgrade_defs: data.upgrade_defs.clone(),
            beacon_start_difficulty_bonus: 0.0,
            unlocks: data.unlocks.clone(),

            camera_offset: vec2(600.0, 400.0),
            camera_zoom: 0.5,
            prev_mouse_pos: vec2(0.0, 0.0),
        }
    }

    pub fn from_save(data: &GameData, save: SaveData) -> Self {
        let mut state = GameplayState::new(data);
        state.apply_save(data, save);
        state
    }

    fn apply_save(&mut self, data: &GameData, save: SaveData) {
        self.current_wave = save.wave_reached;
        self.resources = Resources {
            power: save.resources.power,
            scrap: save.resources.scrap,
            data_cores: save.resources.data_cores,
        };

        self.population.count = save.population.count;
        self.population.morale = save.population.morale;
        self.population.health = save.population.health;
        self.population.food_supply = save.population.food_supply;

        self.threat.energy = save.threat.energy;
        self.threat.heat = save.threat.heat;
        self.threat.data = save.threat.data;
        self.threat.corruption = save.threat.corruption;
        self.threat.noise = save.threat.noise;
        self.threat.territory = save.threat.territory;
        self.last_reaction_tier = self.threat.reaction_tier();

        for saved in save.sectors {
            if let Some(sector) = self.factory.sectors.iter_mut().find(|s| s.id == saved.id) {
                sector.unlocked = saved.unlocked;
                sector.integrity = saved.integrity;
            }
        }
        self.factory.set_purchased_upgrades(save.purchased_upgrades, &self.upgrade_defs);
        self.factory.check_awakening();

        // Restore slot states
        for saved_slot in &save.slots {
            if let Some(slot) = self.map_state.slots.iter_mut().find(|s| s.id == saved_slot.id) {
                slot.state = crate::engine::map::SlotState::from_str(&saved_slot.state);
            }
        }
        // Restore building states
        for saved_building in &save.buildings {
            if let Some(building) = self.map_state.buildings.iter_mut().find(|b| b.id == saved_building.id) {
                building.state = crate::engine::map::BuildingState::from_str(&saved_building.state);
            }
        }
        self.map_state.rebuild_unlocks();

        self.towers.clear();
        for saved in save.towers {
            if let Some(def) = data.tower_def_by_id(&saved.tower_id) {
                let tt = match def.tower_type.as_str() {
                    "Ballistic" => TowerType::Ballistic,
                    "Laser" => TowerType::Laser,
                    "Emp" => TowerType::Emp,
                    "AreaDenial" => TowerType::AreaDenial,
                    "Subversion" => TowerType::Subversion,
                    _ => TowerType::Ballistic,
                };
                let mut tower = Tower::new(
                    tt,
                    def.id.clone(),
                    Vec2::new(saved.x, saved.y),
                    def.base_range,
                    def.base_damage,
                    def.fire_rate,
                    def.cost_power,
                    def.cost_scrap,
                );
                helpers::apply_upgrade_levels(&mut tower, saved.level, &self.constants);
                let tower_idx = self.towers.len();
                // Link tower back to slot by position
                for slot in self.map_state.slots.iter_mut() {
                    if (slot.position - Vec2::new(saved.x, saved.y)).length() < 2.0 {
                        slot.tower_index = Some(tower_idx);
                        break;
                    }
                }
                self.towers.push(tower);
            }
        }

        self.between_waves = true;
        self.wave_timer = self.wave_interval;
        self.wave_manager = WaveManager::new(WaveTuning {
            spawn_interval: self.constants.waves.spawn_interval,
            commander_aura_radius: self.constants.enemy.commander_aura_radius,
            commander_aura_speed_mult: self.constants.enemy.commander_aura_speed_mult,
            enemy_tuning: EnemyTuning {
                scout_dodge_chance: self.constants.enemy.scout_dodge_chance,
                scout_dodge_duration: self.constants.enemy.scout_dodge_duration,
                hit_flash_duration: self.constants.enemy.hit_flash_duration,
                saboteur_skip_chance: self.constants.enemy.saboteur_skip_chance,
                slow_multiplier: self.constants.enemy.slow_multiplier,
            },
            wave_budget_base: self.constants.waves.budget_base,
            wave_budget_per_wave: self.constants.waves.budget_per_wave,
            wave_commander_every: self.constants.waves.commander_every,
            threat_budget_divisor: self.constants.threat.budget_divisor,
            threat_health_mult_per_awareness: self.constants.threat.health_mult_per_awareness,
        });
        self.selected_tower = None;
        self.selected_slot = None;
        self.selected_building = None;
        self.selected_core = false;
        self.selected_upgrade = None;
        self.update_beacon();
    }

    fn build_save_data(&self) -> SaveData {
        SaveData {
            version: 1,
            wave_reached: self.current_wave,
            resources: SavedResources {
                power: self.resources.power,
                scrap: self.resources.scrap,
                data_cores: self.resources.data_cores,
                water: 0.0,
            },
            population: SavedPopulation {
                count: self.population.count,
                morale: self.population.morale,
                health: self.population.health,
                food_supply: self.population.food_supply,
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
        }
    }

    fn autosave(&self) {
        let _ = self.build_save_data().save();
    }
}
