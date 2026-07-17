//! Data types, JSON loading, and game constants.

pub mod loader;
pub mod strings;

use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct GameData {
    pub constants: GameConstants,
    pub tower_defs: Vec<TowerDef>,
    pub enemy_defs: Vec<EnemyDef>,
    pub sector_defs: Vec<SectorData>,
    pub upgrade_defs: Vec<UpgradeDef>,
    pub unlocks: UnlocksDef,
    pub map_def: MapDef,
}

impl GameData {
    pub fn load() -> Self {
        let constants = loader::load_constants();
        let tower_defs = loader::load_tower_defs();
        let enemy_defs = loader::load_enemy_defs();
        let sector_defs = loader::load_sector_defs();
        let upgrade_defs = loader::load_upgrade_defs();
        let unlocks = loader::load_unlocks();
        let map_def = loader::load_map_def();

        Self {
            constants,
            tower_defs,
            enemy_defs,
            sector_defs,
            upgrade_defs,
            unlocks,
            map_def,
        }
    }

    pub fn tower_def_by_id(&self, id: &str) -> Option<&TowerDef> {
        self.tower_defs.iter().find(|t| t.id == id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConstants {
    pub starting: StartingConstants,
    pub economy: EconomyConstants,
    pub population: PopulationConstants,
    pub waves: WaveConstants,
    pub evacuation: EvacuationConstants,
    pub threat: ThreatConstants,
    pub enemy: EnemyConstants,
    pub tower: TowerConstants,
    pub sector: SectorConstants,
    pub scavenger: ScavengerConstants,
    pub ui: UiConstants,
    pub placement: PlacementConstants,
    pub particles: ParticleConstants,
    pub gameplay: GameplayConstants,
    pub map: MapConstants,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartingConstants {
    pub population: u32,
    pub power: f32,
    pub scrap: f32,
    pub morale: f32,
    pub health: f32,
    pub food_supply: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomyConstants {
    pub power_cap: f32,
    pub power_buffer_min_for_build: f32,
    pub productivity_scrap_rate: f32,
    /// Power generation while `power_core` is active: a flat base plus a
    /// per-sector bonus for every other unlocked, undamaged sector.
    pub power_core_base_generation: f32,
    pub power_per_other_unlocked_sector: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopulationConstants {
    pub food_consumption_per_person: f32,
    pub food_per_person_per_sec: f32,
    /// Consumption multiplier applied while the beacon is active — the holdout
    /// shelters and the factory runs hot, so holding the beacon strains food.
    pub beacon_food_multiplier: f32,
    pub starve_morale_loss_per_sec: f32,
    pub starve_health_loss_per_sec: f32,
    pub recover_morale_per_sec: f32,
    pub recover_health_per_sec: f32,
    pub death_interval_sec: f32,
    pub low_morale_threshold: f32,
    pub low_morale_productivity: f32,
    pub recover_morale_threshold: f32,
    pub recover_health_threshold: f32,
    pub death_health_threshold: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaveConstants {
    pub interval_seconds: f32,
    pub base_enemy_health: f32,
    pub health_scale_per_wave: f32,
    pub scrap_per_wave_base: f32,
    pub scrap_per_wave_per_wave: f32,
    pub food_per_wave: f32,
    pub spawn_interval: f32,
    pub budget_multiplier: f32,
    pub budget_base: u32,
    pub budget_per_wave: u32,
    pub commander_every: u32,
    /// Permanent wave-budget increase banked each time a beacon cycle completes.
    /// The machines learn you are the bait, so every cycle starts harder and the
    /// safe low-phase farm erodes.
    pub escalation_per_cycle: f32,
    /// Multiplier on the per-wave food reward while `logistics_hub` is active.
    pub logistics_hub_food_mult: f32,
}

/// Rate (survivors/sec) at which the active beacon draws machine attention away
/// from other human groups, letting them evacuate to safety elsewhere. Higher
/// beacon phases scream louder and evacuate faster — the core reward for greed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvacuationConstants {
    pub warm_rate: f32,
    pub sustained_rate: f32,
    pub screaming_rate: f32,
    pub terminal_rate: f32,
    /// Every N banked survivors triggers a milestone notification.
    pub milestone_interval: u32,
    /// Beacon-strength boundaries that select the current phase (see
    /// `beacon::phase_from_strength`). Below `sustained_threshold` is
    /// WarmSignal.
    pub sustained_threshold: f32,
    pub screaming_threshold: f32,
    pub terminal_threshold: f32,
    /// Weights for the beacon-strength formula (see
    /// `GameplayState::update_beacon`): how much each unlocked sector, unit of
    /// power throughput, the AI vault, and each surviving population point
    /// contribute to the strength that drives phase escalation.
    pub strength_per_unlocked_sector: f32,
    pub strength_per_power_throughput: f32,
    pub strength_per_ai_vault: f32,
    pub strength_per_population: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreatConstants {
    pub budget_divisor: f32,
    pub health_mult_per_awareness: f32,
    pub noise_scout_breach: f32,
    /// Per-second signature generation from how the factory is run. Each keys a
    /// distinct machine-awareness signal so the loudest one reflects the
    /// player's style (see ThreatKind). The flat decay in ThreatSignature means
    /// small footprints stay quiet and only sustained activity accumulates.
    pub energy_per_power_per_sec: f32,
    pub territory_per_sector_per_sec: f32,
    pub territory_per_tower_per_sec: f32,
    pub data_per_upgrade_per_sec: f32,
    pub corruption_per_sec: f32,
    /// Minimum dominant-signature value before the machines visibly adapt their
    /// wave composition, and the (higher) value at which they pull the favoured
    /// archetype in ahead of the tier schedule.
    pub adaptation_threshold: f32,
    pub adaptation_unlock_threshold: f32,
    /// Awareness-level boundaries for `ThreatSignature::reaction_tier` —
    /// the single source of truth for the tier that also gates enemy-unlock
    /// escalation (`wave_impl::build_spawn_queue`) and the HUD threat colour
    /// (`helpers::threat_color`). Below `tier_2` is Observation, below
    /// `tier_3` is Adaptation, below `tier_4` is Suppression, at or above
    /// `tier_4` is Extermination.
    pub tier_2_awareness: f32,
    pub tier_3_awareness: f32,
    pub tier_4_awareness: f32,
    /// Per-second decay applied to every signature except territory, which
    /// decays at this rate scaled by `territory_decay_mult` — territory lingers
    /// far longer than the others, balanced against the survival-proof test.
    pub decay_rate: f32,
    pub territory_decay_mult: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyConstants {
    pub scout_dodge_chance: f32,
    pub scout_dodge_duration: f32,
    pub hit_flash_duration: f32,
    pub saboteur_skip_chance: f32,
    pub slow_multiplier: f32,
    pub commander_aura_radius: f32,
    pub commander_aura_speed_mult: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TowerConstants {
    pub upgrade_max_level: u32,
    pub upgrade_damage_mult: f32,
    pub upgrade_range_mult: f32,
    pub shot_ttl: f32,
    pub emp_slow_duration: f32,
    pub area_denial_damage_scale: f32,
    pub subversion_chain_radius: f32,
    pub subversion_chain_damage_fraction: f32,
    pub commander_death_radius: f32,
    pub commander_death_fraction: f32,
    pub laser_vs_heavy_mult: f32,
    pub laser_vs_scout_mult: f32,
    pub ballistic_vs_heavy_mult: f32,
    /// Heat-signature contribution per tower shot per second (sole feed into
    /// the heat awareness signal — see `ThreatKind::Heat`).
    pub heat_per_shot: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectorConstants {
    pub repair_cost: f32,
    pub repair_amount: f32,
    pub bonus_damage_mult: f32,
    pub bonus_fire_rate_mult: f32,
    pub bonus_range_mult: f32,
    pub bonus_scrap_mult: f32,
    pub damage_default: f32,
    pub damage_saboteur: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScavengerConstants {
    pub initial_scavengers: u32,
    pub recall_interval: f32,
    pub warm_scrap: f32,
    pub warm_food: f32,
    pub sustained_scrap: f32,
    pub sustained_food: f32,
    pub sustained_pop_chance: f32,
    pub screaming_scrap: f32,
    pub screaming_food: f32,
    pub screaming_pop_gain: u32,
    pub terminal_scrap: f32,
    pub terminal_food: f32,
    pub terminal_pop_gain: u32,
    pub terminal_loss_chance: f32,
    pub shutdown_loss_chance: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiConstants {
    pub build_panel_w: f32,
    pub sector_panel_w: f32,
    pub top_bar_h: f32,
    pub command_strip_h: f32,
    pub bottom_context_h: f32,
    pub alert_max_visible: usize,
    pub hud_height: f32,
    pub wave_start_delay: f32,
    pub wave_flash_duration: f32,
    pub wave_flash_alpha: f32,
    pub wave_flash_text_size: f32,
    pub wave_flash_text_y_frac: f32,
    pub notification_ttl: f32,
    pub threat_flash_duration: f32,
    pub tower_hover_dist: f32,
    pub tower_base_radius: f32,
    pub tower_level_radius_inc: f32,
    pub tower_ring_offset: f32,
    pub tower_select_radius: f32,
    pub tooltip_w: f32,
    pub tooltip_h: f32,
    pub food_bar_w: f32,
    pub food_bar_h: f32,
    pub food_bar_max_per_pop: f32,
    pub food_bar_min_max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlacementConstants {
    pub grid: f32,
    pub path_min_dist: f32,
    pub tower_min_spacing: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParticleConstants {
    pub ttl: f32,
    pub count_min: u32,
    pub count_max: u32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub drag: f32,
    pub size: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameplayConstants {
    pub speed_multiplier: f32,
    pub beacon_start_delay: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapConstants {
    pub path: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapDef {
    pub map_size: [f32; 2],
    pub factory_core: [f32; 2],
    pub slot_interact_radius: f32,
    pub building_interact_radius: f32,
    pub slots: Vec<SlotDef>,
    pub building_slots: Vec<BuildingSlotDef>,
    pub paths: Vec<PathDef>,
    pub traces: Vec<TraceDef>,
    #[serde(default)]
    pub sections: Vec<SectionDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDef {
    pub id: String,
    pub label: String,
    pub core_building: String,
    #[serde(default)]
    pub buildings: Vec<String>,
    #[serde(default)]
    pub slots: Vec<String>,
    #[serde(default)]
    pub unlock_entrance: Option<String>,
    #[serde(default)]
    pub visible_at_start: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotDef {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub initial_state: String,
    pub clear_cost: f32,
    pub power_cost: f32,
    pub opens_entrance: Option<String>,
    #[serde(default)]
    pub requires_building_power: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathDef {
    pub id: String,
    pub entrance: [f32; 2],
    pub waypoints: Vec<[f32; 2]>,
    pub exit: [f32; 2],
    pub initially_active: bool,
    pub requires_entrance: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceDef {
    pub from: String,
    pub to: String,
    pub via: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingSlotDef {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub building_type: String,
    pub repair_cost: f32,
    pub power_cost: f32,
    pub boon: BuildingBoon,
    pub threat_per_sec: f32,
    #[serde(default)]
    pub opens_entrance: Option<String>,
    #[serde(default)]
    pub requires_power_from: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BuildingBoon {
    pub scrap_per_sec: f32,
    pub food_per_sec: f32,
    pub water_per_sec: f32,
    pub power_per_sec: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TowerType {
    Ballistic,
    Laser,
    Emp,
    AreaDenial,
    Subversion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EnemyType {
    Scout,
    Drone,
    HeavyUnit,
    Saboteur,
    Commander,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TowerDef {
    pub id: String,
    pub name: String,
    pub tower_type: TowerType,
    pub color: [f32; 4],
    pub cost_scrap: f32,
    pub cost_power: f32,
    pub base_damage: f32,
    pub base_range: f32,
    pub fire_rate: f32,
    pub description: String,
}

impl TowerDef {
    pub fn color(&self) -> Color {
        Color::from(self.color)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyDef {
    pub id: String,
    pub name: String,
    pub enemy_type: EnemyType,
    #[serde(default = "default_tier")]
    pub tier: u32,
    #[serde(default = "default_threat_value")]
    pub threat_value: u32,
    pub base_health: f32,
    pub speed: f32,
    pub scrap_reward: f32,
}

fn default_tier() -> u32 {
    1
}

fn default_threat_value() -> u32 {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectorData {
    pub id: String,
    pub name: String,
    pub starts_unlocked: bool,
    pub max_integrity: f32,
    pub base_power_cost: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeDef {
    pub id: String,
    pub name: String,
    pub sector: String,
    pub description: String,
    pub requires: Vec<String>,
    pub cost_scrap: f32,
    pub cost_power: f32,
    pub difficulty_cost: f32,
    pub effects: HashMap<String, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlocksDef {
    pub towers: HashMap<String, UnlockRule>,
    pub buildings: HashMap<String, UnlockRule>,
    #[serde(default)]
    pub difficulty_weights: UnlockDifficultyWeights,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlockRule {
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlockDifficultyWeights {
    pub per_sector: f32,
    pub per_tower: f32,
    pub per_building_type: f32,
    pub per_repaired_building: f32,
    pub per_upgrade: f32,
}

impl Default for UnlockDifficultyWeights {
    fn default() -> Self {
        Self {
            per_sector: 0.05,
            per_tower: 0.03,
            per_building_type: 0.03,
            per_repaired_building: 0.02,
            per_upgrade: 0.03,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn embedded_game_data_loads_required_content() {
        let data = GameData::load();

        assert!(!data.tower_defs.is_empty(), "expected tower definitions");
        assert!(!data.enemy_defs.is_empty(), "expected enemy definitions");
        assert!(!data.sector_defs.is_empty(), "expected sector definitions");
        assert!(
            !data.upgrade_defs.is_empty(),
            "expected upgrade definitions"
        );
        assert!(!data.map_def.slots.is_empty(), "expected map slots");
        assert!(!data.map_def.paths.is_empty(), "expected enemy paths");
        assert!(
            data.map_def.paths.iter().any(|path| path.initially_active),
            "expected at least one initially active path"
        );

        let mut tower_ids = HashSet::new();
        for tower in &data.tower_defs {
            assert!(
                tower_ids.insert(&tower.id),
                "duplicate tower id {}",
                tower.id
            );
            assert!(tower.cost_scrap >= 0.0);
            assert!(tower.base_range > 0.0);
        }

        let mut enemy_ids = HashSet::new();
        for enemy in &data.enemy_defs {
            assert!(
                enemy_ids.insert(&enemy.id),
                "duplicate enemy id {}",
                enemy.id
            );
            assert!(enemy.base_health > 0.0);
            assert!(enemy.speed > 0.0);
        }
    }
}
