//! Core gameplay state: wave defense, factory management, resource loop.

mod assets;
mod helpers;
mod render;
mod render_hud;
mod render_map;
mod render_panels;
mod ui;
mod ui_advice;
mod update;

use crate::data::{EnemyDef, GameConstants, GameData, UpgradeDef};
use crate::engine::beacon::BeaconPhase;
use crate::engine::enemy::EnemyTuning;
use crate::engine::factory::Factory;
use crate::engine::map::MapState;
use crate::engine::population::Population;
use crate::engine::threat::{ReactionTier, ThreatSignature};
use crate::engine::tower::{ShotEffect, Tower};
use crate::engine::wave::{WaveManager, WaveTuning};
use crate::save::{
    SaveData, SavedBuilding, SavedPopulation, SavedResources, SavedSector, SavedSlot, SavedThreat,
    SavedTower,
};
use assets::GameplayAssets;
use macroquad::prelude::{vec2, Vec2};
use macroquad_toolkit::camera::{Camera2D as ToolkitCamera2D, Camera2DConfig, CameraBounds};

pub struct GameplayState {
    pub assets: GameplayAssets,
    pub constants: GameConstants,
    pub factory: Factory,
    pub wave_manager: WaveManager,
    pub threat: ThreatSignature,
    pub population: Population,
    pub resources: Resources,
    pub current_wave: u32,
    pub paused: bool,

    pub towers: Vec<Tower>,
    pub tower_stats: Vec<TowerUiStats>,
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
    pub autosave_enabled: bool,
    pub survival_proof_active: bool,

    pub base_health_scale_per_wave: f32,
    pub base_food_per_wave: f32,

    pub shutdown_triggered: bool,
    pub beacon_active: bool,

    // The sacrifice ledger: while the beacon screams it draws machines away from
    // other survivors, letting them evacuate. `pending_evacuees` accrues during
    // the current beacon window and only banks into the persistent
    // `survivors_evacuated` total on a clean shutdown — a defeat loses it. This
    // is the campaign's real score and the reason to hold a loud beacon longer.
    pub survivors_evacuated: u32,
    pub pending_evacuees: f32,
    pub next_evac_milestone: u32,

    // Permanent, per-cycle wave-budget escalation. The machines learn you are
    // the bait, so this never resets — it makes the free low-phase farm decay.
    pub machine_escalation: f32,

    // Beacon cycle loop: shutdown yields a salvage report, then the player
    // rebuilds and can raise the beacon again instead of the run ending.
    pub salvage_report: Option<SalvageReport>,
    pub beacon_cycles_completed: u32,
    pub cycle_baseline: CycleBaseline,
    pub end_campaign_requested: bool,

    pub coach: Coach,
    pub show_settings: bool,
    pub settings: crate::save::Settings,
    pub show_intro: bool,

    pub upgrade_defs: Vec<UpgradeDef>,
    pub beacon_start_difficulty_bonus: f32,
    pub unlocks: crate::data::UnlocksDef,
    pub enemy_defs: Vec<EnemyDef>,

    // Camera
    pub camera: ToolkitCamera2D,
}

pub struct Resources {
    pub power: f32,
    pub scrap: f32,
    pub data_cores: u32,
    pub water: f32,
}

/// Cumulative run totals captured when a beacon cycle starts, so the
/// end-of-cycle salvage report can show only that cycle's contribution.
#[derive(Clone, Debug, Default)]
pub struct CycleBaseline {
    pub wave: u32,
    pub sent: u32,
    pub returned: u32,
    pub lost: u32,
    pub scrap: f32,
    pub food: f32,
    pub population: u32,
}

/// Result of a single beacon cycle, shown inline when the beacon is shut down
/// and the field clears — after which the player rebuilds and can raise the
/// beacon again for another cycle.
#[derive(Clone, Debug)]
pub struct SalvageReport {
    pub cycle: u32,
    pub waves: u32,
    pub scavengers_sent: u32,
    pub scavengers_returned: u32,
    pub scavengers_lost: u32,
    pub scrap: f32,
    pub food: f32,
    pub population: u32,
    pub beacon_phase: BeaconPhase,
    /// Survivors the beacon let escape during this cycle, and the running total.
    pub survivors_evacuated_cycle: u32,
    pub survivors_evacuated_total: u32,
    /// Permanent assault escalation now in effect, as a percentage, so the
    /// player sees the farm getting harder each cycle.
    pub escalation_pct: f32,
}

pub struct Notification {
    pub text: String,
    pub ttl: f32,
}

/// First-run onboarding coach: a short, skippable sequence that walks the
/// player through the core loop. Advances as each step is actually performed.
#[derive(Clone, Debug, Default)]
pub struct Coach {
    pub active: bool,
    pub step: usize,
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub ttl: f32,
}

#[derive(Clone, Debug, Default)]
pub struct TowerUiStats {
    pub shots_this_wave: u32,
    pub hits_this_wave: u32,
    pub kills_this_wave: u32,
}

impl GameplayState {
    pub fn new(data: &GameData) -> Self {
        let constants = data.constants.clone();
        let map_state = MapState::from_def(data.map_def.clone());

        // Frame the camera on the currently-revealed area so the map opens small
        // and grows as sections are powered back online (see sync_camera_bounds).
        let (vmin, vmax) = map_state.visible_bounds();
        let view_center = (vmin + vmax) * 0.5;
        let content_w = (vmax.x - vmin.x).max(1.0) + 360.0;
        let content_h = (vmax.y - vmin.y).max(1.0) + 320.0;
        let init_zoom = (900.0 / content_w).min(560.0 / content_h).clamp(0.6, 1.7);
        let cam_pad = 240.0;
        let cam_min = vec2((vmin.x - cam_pad).max(0.0), (vmin.y - cam_pad).max(0.0));
        let cam_max = vec2(
            (vmax.x + cam_pad).min(map_state.map_size.x),
            (vmax.y + cam_pad).min(map_state.map_size.y),
        );

        let mut factory = Factory::new();
        factory.init_sectors(&data.sector_defs);

        let threat = ThreatSignature::new();
        let last_reaction_tier = threat.reaction_tier(&data.constants.threat);

        let settings = crate::save::Settings::load();
        let coach_active = !settings.tutorial_seen;
        let start_time_scale = if settings.default_fast_speed {
            data.constants.gameplay.speed_multiplier
        } else {
            1.0
        };
        let autosave_enabled = settings.autosave;

        Self {
            assets: GameplayAssets::load(),
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
                tier_2_awareness: data.constants.threat.tier_2_awareness,
                tier_3_awareness: data.constants.threat.tier_3_awareness,
            }),
            threat,
            population: Population::new(&data.constants),
            resources: Resources {
                power: data.constants.starting.power,
                scrap: data.constants.starting.scrap,
                data_cores: 0,
                water: data.constants.starting.water_supply,
            },
            current_wave: 0,
            paused: false,

            towers: Vec::new(),
            tower_stats: Vec::new(),
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
            time_scale: start_time_scale,

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
            autosave_enabled,
            survival_proof_active: false,

            base_health_scale_per_wave: data.constants.waves.health_scale_per_wave,
            base_food_per_wave: data.constants.waves.food_per_wave,

            shutdown_triggered: false,
            beacon_active: false,

            survivors_evacuated: 0,
            pending_evacuees: 0.0,
            next_evac_milestone: data.constants.evacuation.milestone_interval,
            machine_escalation: 0.0,

            salvage_report: None,
            beacon_cycles_completed: 0,
            cycle_baseline: CycleBaseline::default(),
            end_campaign_requested: false,

            // Onboarding shows only until the player has seen it once.
            coach: Coach {
                active: coach_active,
                step: 0,
            },
            show_settings: false,
            settings,
            // Fresh runs open on the premise card; continuing a save skips it.
            show_intro: true,

            upgrade_defs: data.upgrade_defs.clone(),
            beacon_start_difficulty_bonus: 0.0,
            unlocks: data.unlocks.clone(),
            enemy_defs: data.enemy_defs.clone(),

            camera: ToolkitCamera2D::with_config(
                view_center,
                init_zoom,
                Camera2DConfig {
                    drag_button: Some(macroquad::prelude::MouseButton::Middle),
                    min_zoom: 0.25,
                    max_zoom: 2.0,
                    pan_speed: 400.0,
                    zoom_in_factor: 1.1,
                    zoom_out_factor: 1.0 / 1.1,
                    mouse_wheel_zoom_to_cursor: false,
                    keyboard_zoom_enabled: false,
                    bounds: Some(CameraBounds::new(cam_min, cam_max)),
                    ..Default::default()
                },
            ),
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

        // Restore slot states
        for saved_slot in &save.slots {
            if let Some(slot) = self
                .map_state
                .slots
                .iter_mut()
                .find(|s| s.id == saved_slot.id)
            {
                slot.state = crate::engine::map::SlotState::from_str(&saved_slot.state);
            }
        }
        // Restore building states
        for saved_building in &save.buildings {
            if let Some(building) = self
                .map_state
                .buildings
                .iter_mut()
                .find(|b| b.id == saved_building.id)
            {
                building.state = crate::engine::map::BuildingState::from_str(&saved_building.state);
            }
        }
        self.map_state.rebuild_unlocks();

        self.towers.clear();
        self.tower_stats.clear();
        for saved in save.towers {
            if let Some(def) = data.tower_def_by_id(&saved.tower_id) {
                let tt = def.tower_type.clone();
                let mut tower = Tower::new(
                    tt,
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
                let tower_idx = self.towers.len();
                // Link tower back to slot by position
                for slot in self.map_state.slots.iter_mut() {
                    if (slot.position - Vec2::new(saved.x, saved.y)).length() < 2.0 {
                        slot.tower_index = Some(tower_idx);
                        break;
                    }
                }
                self.towers.push(tower);
                self.tower_stats.push(TowerUiStats::default());
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
            tier_2_awareness: self.constants.threat.tier_2_awareness,
            tier_3_awareness: self.constants.threat.tier_3_awareness,
        });
        self.selected_tower = None;
        self.selected_slot = None;
        self.selected_building = None;
        self.selected_core = false;
        self.selected_upgrade = None;
        self.coach.active = false;
        self.show_intro = false;

        self.survivors_evacuated = save.survivors_evacuated;
        self.machine_escalation = save.machine_escalation;
        let interval = self.constants.evacuation.milestone_interval.max(1);
        self.next_evac_milestone = (self.survivors_evacuated / interval + 1) * interval;

        self.update_beacon();
    }

    fn build_save_data(&self) -> SaveData {
        SaveData {
            version: 2,
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
            survivors_evacuated: self.survivors_evacuated,
            machine_escalation: self.machine_escalation,
        }
    }

    /// Grow the camera's pannable area to match the currently-revealed sections
    /// so newly-powered wings of the factory become reachable, while keeping the
    /// unrevealed expanse out of reach.
    pub(crate) fn sync_camera_bounds(&mut self) {
        let (vmin, vmax) = self.map_state.visible_bounds();
        let pad = 240.0;
        let bmin = vec2((vmin.x - pad).max(0.0), (vmin.y - pad).max(0.0));
        let bmax = vec2(
            (vmax.x + pad).min(self.map_state.map_size.x),
            (vmax.y + pad).min(self.map_state.map_size.y),
        );
        self.camera.set_bounds(Some(CameraBounds::new(bmin, bmax)));
    }

    fn autosave(&self) {
        if !self.autosave_enabled {
            return;
        }
        let _ = self.build_save_data().save();
    }
}
