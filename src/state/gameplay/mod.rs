//! Core gameplay state: wave defense, factory management, resource loop.

mod assets;
mod camera;
mod depth;
mod helpers;
mod lifecycle;
mod persistence;
mod render;
mod render_depth;
mod render_hud;
mod render_map;
mod render_panels;
mod ui;
mod ui_advice;
mod update;

use crate::data::{EnemyDef, GameConstants, UpgradeDef};
use crate::engine::beacon::BeaconPhase;
use crate::engine::depth::DepthDirective;
use crate::engine::factory::Factory;
use crate::engine::map::MapState;
use crate::engine::population::Population;
use crate::engine::threat::{ReactionTier, ThreatSignature};
use crate::engine::tower::{ShotEffect, Tower};
use crate::engine::vault::VaultTakeover;
use crate::engine::wave::WaveManager;
use crate::save::Settings;
use macroquad::prelude::Vec2;
use macroquad_toolkit::camera::Camera2D as ToolkitCamera2D;

pub struct GameplayState {
    pub assets: Option<assets::GameplayAssets>,
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
    pub vault_takeover: VaultTakeover,

    pub coach: Coach,
    pub show_settings: bool,
    pub settings: Settings,
    pub show_intro: bool,
    pub show_workforce: bool,

    pub upgrade_defs: Vec<UpgradeDef>,
    pub beacon_start_difficulty_bonus: f32,
    pub unlocks: crate::data::UnlocksDef,
    pub enemy_defs: Vec<EnemyDef>,

    /// Highest map band reached. This is derived from progressive section
    /// visibility and is kept here only so a new depth can announce itself
    /// once instead of every frame.
    pub last_depth_level: u32,
    pub depth_directives: Vec<Option<DepthDirective>>,
    pub pending_depth_directive: Option<u32>,

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

#[cfg(test)]
mod tests;
