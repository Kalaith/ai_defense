//! Construction and restoration entry points for the live gameplay state.

use crate::data::{GameConstants, GameData};
use crate::engine::enemy::EnemyTuning;
use crate::engine::factory::Factory;
use crate::engine::map::MapState;
use crate::engine::threat::ThreatSignature;
use crate::engine::tower::{TargetPriority, Tower};
use crate::engine::wave::{WaveManager, WaveTuning};
use macroquad::prelude::vec2;
use macroquad_toolkit::camera::{Camera2D as ToolkitCamera2D, Camera2DConfig, CameraBounds};

use super::assets::GameplayAssets;
use super::{Coach, CycleBaseline, DefenseReplay, GameplayState, Resources, TowerUiStats};
use crate::engine::beacon::BeaconPhase;
use crate::engine::population::Population;
use crate::engine::vault::VaultTakeover;

pub(super) fn wave_tuning(constants: &GameConstants) -> WaveTuning {
    WaveTuning {
        spawn_interval: constants.waves.spawn_interval,
        commander_aura_radius: constants.enemy.commander_aura_radius,
        commander_aura_speed_mult: constants.enemy.commander_aura_speed_mult,
        enemy_tuning: EnemyTuning {
            scout_dodge_chance: constants.enemy.scout_dodge_chance,
            scout_dodge_duration: constants.enemy.scout_dodge_duration,
            scout_report_interval: constants.enemy.scout_report_interval,
            hit_flash_duration: constants.enemy.hit_flash_duration,
            saboteur_skip_chance: constants.enemy.saboteur_skip_chance,
            saboteur_strike_interval: constants.enemy.saboteur_strike_interval,
            slow_multiplier: constants.enemy.slow_multiplier,
            commander_pulse_interval: constants.enemy.commander_pulse_interval,
            commander_shield_duration: constants.enemy.commander_shield_duration,
            commander_shield_radius: constants.enemy.commander_shield_radius,
            commander_shield_multiplier: constants.enemy.commander_shield_multiplier,
        },
        wave_budget_base: constants.waves.budget_base,
        wave_budget_per_wave: constants.waves.budget_per_wave,
        wave_commander_every: constants.waves.commander_every,
        threat_budget_divisor: constants.threat.budget_divisor,
        threat_health_mult_per_awareness: constants.threat.health_mult_per_awareness,
        tier_2_awareness: constants.threat.tier_2_awareness,
        tier_3_awareness: constants.threat.tier_3_awareness,
    }
}

impl GameplayState {
    pub fn new(data: &GameData) -> Self {
        let constants = data.constants.clone();
        let map_state = MapState::from_def(data.map_def.clone());

        // Frame the camera on the currently-revealed area so the map opens small
        // and grows as sections are powered back online (see sync_camera_bounds).
        let (vmin, vmax) = map_state.visible_bounds();
        // The old sidebars masked the empty world margin to the left of the
        // first revealed wing. With the map now occupying the full viewport,
        // frame the first live section near the left edge instead.
        let view_center = (vmin + vmax) * 0.5 + vec2((vmin.x + 20.0).max(0.0), 0.0);
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
        let initial_depth = map_state.deepest_visible_depth();

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

        let mut state = Self {
            assets: GameplayAssets::load(),
            constants,
            factory,
            wave_manager: WaveManager::new(wave_tuning(&data.constants)),
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

            particles: macroquad_toolkit::fx::ParticleSystem::with_capacity(usize::MAX),
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
            defense_replay: DefenseReplay::KineticLine,

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
            vault_takeover: VaultTakeover::default(),

            // Onboarding shows only until the player has seen it once.
            coach: Coach {
                active: coach_active,
                step: 0,
            },
            show_settings: false,
            settings,
            // Fresh runs open on the premise card; continuing a save skips it.
            show_intro: true,
            show_workforce: false,
            show_build_panel: false,
            show_sector_panel: false,
            show_beacon_panel: false,

            upgrade_defs: data.upgrade_defs.clone(),
            beacon_start_difficulty_bonus: 0.0,
            unlocks: data.unlocks.clone(),
            enemy_defs: data.enemy_defs.clone(),
            last_depth_level: initial_depth,
            depth_directives: vec![None; 4],
            pending_depth_directive: None,
            section_awakening: None,

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
        };
        state.seed_starting_defense(data);
        state
    }

    fn seed_starting_defense(&mut self, data: &GameData) {
        for (tower_id, slot_id, priority) in [
            ("ballistic_turret", "slot_01", TargetPriority::First),
            ("laser_emitter", "slot_02", TargetPriority::Strongest),
        ] {
            let Some(def) = data.tower_def_by_id(tower_id) else {
                continue;
            };
            let Some(slot_idx) = self
                .map_state
                .slots
                .iter()
                .position(|slot| slot.id == slot_id)
            else {
                continue;
            };
            if self.map_state.slots[slot_idx].tower_index.is_some() {
                continue;
            }
            let mut tower = Tower::new(
                def.tower_type.clone(),
                def.id.clone(),
                self.map_state.slots[slot_idx].position,
                def.base_range,
                def.base_damage,
                def.fire_rate,
                def.cost_power,
                def.cost_scrap,
                def.color(),
            );
            tower.target_priority = priority;
            let tower_idx = self.towers.len();
            self.towers.push(tower);
            self.tower_stats.push(TowerUiStats::default());
            self.map_state.slots[slot_idx].tower_index = Some(tower_idx);
        }
    }
}
