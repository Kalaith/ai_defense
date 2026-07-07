use crate::data::GameData;
use crate::engine::beacon::{phase_from_strength, BeaconPhase};
use crate::engine::enemy::EnemyType;
use crate::engine::map::SlotState;
use crate::engine::tower::tick_towers;
use crate::engine::tower::TowerTuning;
use crate::engine::wave::WaveEvent;
use crate::state::{RunSummary, StateTransition};
use macroquad::prelude::*;
use macroquad_toolkit::rng;
use std::collections::HashMap;

use super::GameplayState;

impl GameplayState {
    pub(crate) fn enable_survival_proof(&mut self) {
        self.autosave_enabled = false;
        self.survival_proof_active = true;
        self.wave_manager.enemy_tuning.scout_dodge_chance = 0.0;
        self.wave_manager.enemy_tuning.saboteur_skip_chance = 0.0;
        // The proof exercises combat math over the full route; progressive
        // disclosure would truncate enemy paths at the revealed frontier.
        self.map_state.reveal_all_sections();
    }

    pub(crate) fn update_survival_proof(&mut self, data: &GameData) -> Option<StateTransition> {
        if !self.survival_proof_active {
            return self.update();
        }

        self.paused = false;
        self.wave_manager.enemy_tuning.scout_dodge_chance = 0.0;
        self.wave_manager.enemy_tuning.saboteur_skip_chance = 0.0;

        for _ in 0..80 {
            self.keep_building_survival_proof_defense(data);

            if !self.beacon_active && !self.shutdown_triggered {
                self.start_beacon();
            }

            if self.current_wave >= 10
                && !self.wave_manager.wave_active
                && self.wave_manager.spawn_queue.is_empty()
            {
                self.shutdown_triggered = true;
                self.beacon_active = false;
                self.between_waves = false;
                return Some(StateTransition::ToResults {
                    summary: self.build_run_summary(true),
                });
            }

            let dt = 0.1;
            self.update_wave_timers(dt);
            self.start_wave_if_ready();
            self.tick_wave(dt);
            self.update_combat(dt);
            self.update_power(dt);
            self.update_population(dt);
            self.update_building_boons(dt);
            self.update_threat(dt);
            self.update_factory();
            self.update_notifications(dt);
            self.update_wave_flash(dt);

            if self.is_game_over() {
                return Some(StateTransition::ToResults {
                    summary: self.build_run_summary(false),
                });
            }
        }

        None
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        if self.end_campaign_requested {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(true),
            });
        }

        self.handle_camera_input();

        // The intro card and salvage report freeze the sim; each handles its own
        // buttons during draw.
        if self.show_intro || self.salvage_report.is_some() {
            return None;
        }

        self.handle_input();
        if self.paused {
            return None;
        }

        let dt = get_frame_time() * self.time_scale;
        self.validate_selection();

        if self.beacon_active {
            self.update_beacon();
            self.update_evacuation(dt);
        }

        self.update_wave_timers(dt);
        self.start_wave_if_ready();
        self.update_scavenger_recall(dt);
        self.tick_wave(dt);

        self.update_combat(dt);
        self.update_power(dt);
        self.update_population(dt);
        self.update_building_boons(dt);
        self.update_threat(dt);
        self.update_factory();
        self.update_notifications(dt);
        self.update_wave_flash(dt);
        self.update_coach();

        if self.shutdown_triggered && self.enemies_cleared() {
            self.finish_beacon_cycle();
            return None;
        }

        if self.is_game_over() {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(false),
            });
        }

        None
    }

    /// Survivors/sec the beacon is currently evacuating, scaling with phase: a
    /// louder beacon draws more machines away, so more people escape elsewhere.
    fn evacuation_rate(&self) -> f32 {
        let evac = &self.constants.evacuation;
        match self.beacon_phase {
            BeaconPhase::WarmSignal => evac.warm_rate,
            BeaconPhase::SustainedCall => evac.sustained_rate,
            BeaconPhase::ScreamingBeacon => evac.screaming_rate,
            BeaconPhase::TerminalHowl => evac.terminal_rate,
        }
    }

    /// Accrue evacuees into the pending pool while the beacon is up. Nothing is
    /// banked until a clean shutdown — hold too long and lose it all to defeat.
    fn update_evacuation(&mut self, dt: f32) {
        self.pending_evacuees += self.evacuation_rate() * dt;
    }

    /// Beacon shut down and the field is clear: bank any remaining scavengers,
    /// commit this cycle's evacuees to the persistent ledger, ratchet up the
    /// permanent assault escalation, produce the salvage report, and drop back
    /// to base-build mode so the player can rebuild and raise the beacon again.
    fn finish_beacon_cycle(&mut self) {
        self.resolve_remaining_scavengers();

        // Bank pending evacuees; carry the fractional remainder to the next
        // cycle so slow low-phase accrual is never silently lost to rounding.
        let banked = self.pending_evacuees.floor();
        let banked_u = banked as u32;
        self.survivors_evacuated += banked_u;
        self.pending_evacuees -= banked;
        self.announce_evac_milestones();

        // The machines learn the beacon is bait: every completed cycle makes the
        // next assault permanently heavier, decaying the safe low-phase farm.
        self.machine_escalation += self.constants.waves.escalation_per_cycle;

        self.beacon_cycles_completed += 1;
        self.salvage_report = Some(self.build_salvage_report(banked_u));
        self.beacon_start_difficulty_bonus = 0.0;
        self.autosave();
    }

    /// Push a milestone line each time the banked total crosses a new interval,
    /// giving the open-ended campaign a felt sense of a rising goal.
    fn announce_evac_milestones(&mut self) {
        let interval = self.constants.evacuation.milestone_interval.max(1);
        while self.survivors_evacuated >= self.next_evac_milestone {
            let reached = self.next_evac_milestone;
            self.push_notification(format!(
                "{reached} survivors have reached safe territory because you held the line."
            ));
            self.next_evac_milestone += interval;
        }
    }

    /// Force any field teams still out to return home when the beacon goes dark,
    /// each risking loss at the shutdown loss rate.
    fn resolve_remaining_scavengers(&mut self) {
        let still_out = self.scavengers_out;
        if still_out == 0 {
            return;
        }
        let mut lost = 0u32;
        for _ in 0..still_out {
            if rng::chance(self.constants.scavenger.shutdown_loss_chance) {
                lost += 1;
            }
        }
        let returned = still_out - lost;
        self.scavengers_out = 0;
        self.scavengers_lost += lost;
        self.scavengers_returned += returned;
        self.scavenger_recall_active = false;
    }

    fn build_salvage_report(&self, evacuated_cycle: u32) -> super::SalvageReport {
        let b = &self.cycle_baseline;
        super::SalvageReport {
            cycle: self.beacon_cycles_completed,
            waves: self.current_wave.saturating_sub(b.wave),
            scavengers_sent: self.scavengers_sent.saturating_sub(b.sent),
            scavengers_returned: self.scavengers_returned.saturating_sub(b.returned),
            scavengers_lost: self.scavengers_lost.saturating_sub(b.lost),
            scrap: (self.scavenger_scrap_gained - b.scrap).max(0.0),
            food: (self.scavenger_food_gained - b.food).max(0.0),
            population: self
                .scavenger_population_gained
                .saturating_sub(b.population),
            beacon_phase: self.beacon_phase.clone(),
            survivors_evacuated_cycle: evacuated_cycle,
            survivors_evacuated_total: self.survivors_evacuated,
            escalation_pct: self.machine_escalation * 100.0,
        }
    }

    /// Dismiss the salvage report and return to base-build mode for the next
    /// beacon cycle.
    pub(crate) fn dismiss_salvage_report(&mut self) {
        self.salvage_report = None;
        self.shutdown_triggered = false;
        self.beacon_active = false;
        self.between_waves = true;
        self.wave_timer = self.constants.ui.wave_start_delay;
        self.push_notification(
            "Field secured. Rebuild the line, then raise the beacon again.".to_string(),
        );
    }

    fn handle_camera_input(&mut self) {
        let dt = get_frame_time();
        self.camera.update(dt, false);
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.camera.screen_to_world(screen_pos)
    }

    fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            if self.show_settings {
                self.show_settings = false;
            } else if self.placing_tower.is_some() {
                self.placing_tower = None;
            } else {
                self.paused = !self.paused;
            }
        }

        if is_key_pressed(KeyCode::Space) {
            self.time_scale = if self.time_scale > 1.0 {
                1.0
            } else {
                self.constants.gameplay.speed_multiplier
            };
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            self.placing_tower = None;
            self.selected_tower = None;
            self.selected_slot = None;
            self.selected_building = None;
            self.selected_core = false;
            self.selected_upgrade = None;
        }
    }

    fn validate_selection(&mut self) {
        if let Some(idx) = self.selected_tower {
            if idx >= self.towers.len() {
                self.selected_tower = None;
            }
        }
        if let Some(idx) = self.selected_building {
            let unlocked = self
                .map_state
                .buildings
                .get(idx)
                .map(|b| self.is_building_unlocked(b))
                .unwrap_or(false);
            if !unlocked {
                self.selected_building = None;
            }
        }
    }

    fn update_wave_timers(&mut self, dt: f32) {
        if self.beacon_active && !self.shutdown_triggered && self.between_waves {
            self.wave_timer -= dt;
            if self.wave_timer <= 0.0 {
                self.current_wave += 1;
                self.between_waves = false;
            }
        }
    }

    fn start_wave_if_ready(&mut self) {
        if !self.beacon_active
            || self.shutdown_triggered
            || self.between_waves
            || self.wave_manager.wave_active
        {
            return;
        }

        let spawn_points: Vec<(String, Vec2)> = self
            .map_state
            .active_paths()
            .iter()
            .map(|p| (p.id.clone(), p.entrance))
            .collect();

        if spawn_points.is_empty() {
            return;
        }

        let force_commander = self.beacon_phase == BeaconPhase::TerminalHowl;
        let budget_multiplier =
            self.constants.waves.budget_multiplier + self.beacon_start_difficulty_bonus;
        let adaptation = self.wave_adaptation();
        self.wave_manager.generate_wave(
            self.current_wave,
            &self.enemy_defs,
            self.base_health_scale_per_wave,
            self.threat.awareness_level(),
            self.beacon_phase.tier_floor(),
            budget_multiplier,
            force_commander,
            &spawn_points,
            &adaptation,
        );
        self.tower_stats
            .resize_with(self.towers.len(), Default::default);
        for stats in &mut self.tower_stats {
            *stats = Default::default();
        }
        self.threat.add_from_wave(self.current_wave);
        self.wave_flash_timer = self.constants.ui.wave_flash_duration;
        self.last_wave_started = self.current_wave;
    }

    fn tick_wave(&mut self, dt: f32) {
        let paths: HashMap<String, Vec<Vec2>> = self
            .map_state
            .active_paths_limited()
            .into_iter()
            .map(|p| (p.id.clone(), p.points))
            .collect();

        let event = self.wave_manager.tick(dt, &paths);
        match event {
            WaveEvent::EnemyReachedEnd { enemy_type } => self.handle_breach(enemy_type),
            WaveEvent::WaveComplete => self.handle_wave_complete(),
            WaveEvent::None => {}
        }
    }

    fn update_combat(&mut self, dt: f32) {
        let damage_mult = if self.factory.is_sector_active("assembly_hall") {
            self.constants.sector.bonus_damage_mult
        } else {
            1.0
        };
        let fire_rate_mult = if self.factory.is_sector_active("robotics_bay") {
            self.constants.sector.bonus_fire_rate_mult
        } else {
            1.0
        };
        let range_mult = if self.factory.is_sector_active("ai_vault") {
            self.constants.sector.bonus_range_mult
        } else {
            1.0
        };
        let scrap_mult = if self.factory.is_sector_active("logistics_hub") {
            self.constants.sector.bonus_scrap_mult
        } else {
            1.0
        };

        let tuning = TowerTuning {
            shot_ttl: self.constants.tower.shot_ttl,
            emp_slow_duration: self.constants.tower.emp_slow_duration,
            area_denial_damage_scale: self.constants.tower.area_denial_damage_scale,
            subversion_chain_radius: self.constants.tower.subversion_chain_radius,
            subversion_chain_damage_fraction: self.constants.tower.subversion_chain_damage_fraction,
            commander_death_radius: self.constants.tower.commander_death_radius,
            commander_death_fraction: self.constants.tower.commander_death_fraction,
            laser_vs_heavy_mult: self.constants.tower.laser_vs_heavy_mult,
            laser_vs_scout_mult: self.constants.tower.laser_vs_scout_mult,
            ballistic_vs_heavy_mult: self.constants.tower.ballistic_vs_heavy_mult,
        };

        let combat = tick_towers(
            &mut self.towers,
            &mut self.wave_manager.enemies,
            dt,
            damage_mult,
            fire_rate_mult,
            range_mult,
            scrap_mult,
            &tuning,
        );
        self.resources.scrap += combat.scrap_earned;
        self.shot_effects.extend(combat.effects);
        self.tower_stats
            .resize_with(self.towers.len(), Default::default);
        for (idx, stats) in combat.tower_stats.iter().enumerate() {
            if let Some(total) = self.tower_stats.get_mut(idx) {
                total.shots_this_wave += stats.shots;
                total.hits_this_wave += stats.hits;
                total.kills_this_wave += stats.kills;
            }
        }
        if combat.heat_generated > 0.0 {
            self.threat.add_heat(combat.heat_generated);
        }
        if !combat.death_positions.is_empty() {
            self.spawn_death_particles(&combat.death_positions);
        }

        for effect in &mut self.shot_effects {
            effect.tick(dt);
        }
        self.shot_effects.retain(|e| e.is_alive());
        self.update_particles(dt);
    }

    fn update_power(&mut self, dt: f32) {
        let building_power = self.unlocked_building_boon().power_per_sec;
        let gen = self.factory.power_generation() + building_power;
        let tower_drain: f32 = self
            .towers
            .iter()
            .filter(|t| t.is_active)
            .map(|t| t.power_drain)
            .sum();
        let consume = self.factory.power_consumption() + tower_drain;
        self.resources.power = (self.resources.power + (gen - consume) * dt)
            .clamp(0.0, self.constants.economy.power_cap);

        if self.resources.power <= 0.0 && gen < consume {
            let mut current_drain = consume;
            for tower in self.towers.iter_mut() {
                if current_drain <= gen {
                    break;
                }
                if tower.is_active {
                    tower.is_active = false;
                    current_drain -= tower.power_drain;
                }
            }
        }

        if self.resources.power > 0.0 {
            for tower in self.towers.iter_mut() {
                if !tower.is_active {
                    tower.is_active = true;
                }
            }
        }
    }

    fn update_population(&mut self, dt: f32) {
        // Holding the beacon strains food: the holdout shelters and the factory
        // runs hot. This is the pressure that forces the player up the risk
        // curve — low beacon phases can't feed a growing holdout for long.
        let consumption_mult = if self.beacon_active {
            self.constants.population.beacon_food_multiplier
        } else {
            1.0
        };
        self.population.tick(dt, &self.constants, consumption_mult);
        self.resources.scrap += self.population.productivity(&self.constants)
            * self.constants.economy.productivity_scrap_rate
            * dt;
    }

    fn update_building_boons(&mut self, dt: f32) {
        let boon = self.unlocked_building_boon();
        self.resources.scrap += boon.scrap_per_sec * dt;
        self.population.food_supply += boon.food_per_sec * dt;
    }

    fn update_threat(&mut self, dt: f32) {
        // Gather everything the whole-self helpers need up front, before taking
        // the mutable borrow of self.threat below.
        let building_threat = self.unlocked_building_threat_per_sec();
        let power_gen =
            self.factory.power_generation() + self.unlocked_building_boon().power_per_sec;
        let sectors = self.factory.unlocked_count() as f32;
        let towers = self.towers.len() as f32;
        let research_active = self.factory.is_sector_active("research_lab");
        let ai_vault_active = self.factory.is_sector_active("ai_vault");
        let upgrades = self.factory.purchased_upgrades.len() as f32;

        let c = &self.constants.threat;
        let energy_rate = c.energy_per_power_per_sec;
        let territory_rate =
            sectors * c.territory_per_sector_per_sec + towers * c.territory_per_tower_per_sec;
        let data_rate = upgrades * c.data_per_upgrade_per_sec;
        let corruption_rate = c.corruption_per_sec;

        // Each distinct signature reflects *how* the factory is run, so the
        // loudest one (surfaced in the HUD) drives what the machines send.
        if building_threat > 0.0 {
            self.threat.add_noise(building_threat * dt);
        }
        self.threat.add_energy(power_gen * energy_rate * dt);
        self.threat.add_territory(territory_rate * dt);
        if research_active {
            self.threat.add_data(data_rate * dt);
        }
        if ai_vault_active {
            self.threat.add_corruption(corruption_rate * dt);
        }

        self.threat.tick_decay(dt);
        let current_tier = self.threat.reaction_tier();
        if super::helpers::reaction_tier_rank(&current_tier)
            > super::helpers::reaction_tier_rank(&self.last_reaction_tier)
        {
            self.threat_flash_timer = self.constants.ui.threat_flash_duration;
        }
        self.last_reaction_tier = current_tier;
        if self.threat_flash_timer > 0.0 {
            self.threat_flash_timer = (self.threat_flash_timer - dt).max(0.0);
        }
    }

    fn update_factory(&mut self) {
        self.factory.check_awakening();
        self.recalc_factory_integrity();
        self.map_state.update_section_visibility();
        self.sync_camera_bounds();
    }

    fn update_notifications(&mut self, dt: f32) {
        for note in &mut self.notifications {
            note.ttl -= dt;
        }
        self.notifications.retain(|n| n.ttl > 0.0);
    }

    fn update_wave_flash(&mut self, dt: f32) {
        if self.wave_flash_timer > 0.0 {
            self.wave_flash_timer = (self.wave_flash_timer - dt).max(0.0);
        }
    }

    fn handle_breach(&mut self, enemy_type: EnemyType) {
        match enemy_type {
            EnemyType::Scout => {
                self.threat
                    .add_noise(self.constants.threat.noise_scout_breach);
                self.push_notification("Scout slipped through — threat increased".to_string());
            }
            EnemyType::Saboteur => {
                if self.population.count > 0 {
                    self.population.count -= 1;
                }
                self.damage_lowest_integrity_sector(self.constants.sector.damage_saboteur);
            }
            _ => {
                if self.population.count > 0 {
                    self.population.count -= 1;
                }
                self.damage_random_sector(self.constants.sector.damage_default);
            }
        }
    }

    fn handle_wave_complete(&mut self) {
        self.resources.scrap += self.constants.waves.scrap_per_wave_base
            + self.current_wave as f32 * self.constants.waves.scrap_per_wave_per_wave;
        let mut food_reward = self.base_food_per_wave;
        if self.factory.is_sector_active("logistics_hub") {
            food_reward *= 1.5;
        }
        self.population.food_supply += food_reward;

        self.autosave();

        if self.beacon_active && !self.shutdown_triggered {
            self.between_waves = true;
            self.wave_timer = self.wave_interval;
        }
    }

    pub(crate) fn update_beacon(&mut self) {
        let unlocked = self.factory.unlocked_count() as f32;
        let power_throughput = self.factory.power_generation();
        let ai_vault_tier = if self.factory.is_sector_active("ai_vault") {
            1.0
        } else {
            0.0
        };
        let population = self.population.count as f32;

        self.beacon_strength = (unlocked * 2.0)
            + (power_throughput / 10.0)
            + (ai_vault_tier * 5.0)
            + (population / 20.0);

        let next_phase = phase_from_strength(self.beacon_strength);
        if next_phase.rank() > self.beacon_phase.rank() {
            if let Some(flavor) = beacon_phase_flavor(&next_phase) {
                self.push_notification(flavor.to_string());
            }
            self.beacon_phase = next_phase;
        }
    }

    fn update_scavenger_recall(&mut self, dt: f32) {
        if !self.scavenger_recall_active {
            return;
        }
        if self.scavengers_out == 0 {
            self.scavenger_recall_active = false;
            return;
        }

        self.scavenger_recall_timer += dt;
        if self.scavenger_recall_timer < self.constants.scavenger.recall_interval {
            return;
        }
        self.scavenger_recall_timer = 0.0;

        let (scrap, food, pop_gain) = match self.beacon_phase {
            BeaconPhase::WarmSignal => (
                self.constants.scavenger.warm_scrap,
                self.constants.scavenger.warm_food,
                0,
            ),
            BeaconPhase::SustainedCall => {
                let pop = if rng::chance(self.constants.scavenger.sustained_pop_chance) {
                    1
                } else {
                    0
                };
                (
                    self.constants.scavenger.sustained_scrap,
                    self.constants.scavenger.sustained_food,
                    pop,
                )
            }
            BeaconPhase::ScreamingBeacon => (
                self.constants.scavenger.screaming_scrap,
                self.constants.scavenger.screaming_food,
                self.constants.scavenger.screaming_pop_gain,
            ),
            BeaconPhase::TerminalHowl => {
                if rng::chance(self.constants.scavenger.terminal_loss_chance) {
                    self.scavengers_out = self.scavengers_out.saturating_sub(1);
                    self.scavengers_lost += 1;
                    self.push_notification("Scavenger team lost in the field".to_string());
                    return;
                }
                (
                    self.constants.scavenger.terminal_scrap,
                    self.constants.scavenger.terminal_food,
                    self.constants.scavenger.terminal_pop_gain,
                )
            }
        };

        self.scavengers_out = self.scavengers_out.saturating_sub(1);
        self.scavengers_returned += 1;
        self.resources.scrap += scrap;
        self.population.food_supply += food;
        self.population.count += pop_gain;

        self.scavenger_scrap_gained += scrap;
        self.scavenger_food_gained += food;
        self.scavenger_population_gained += pop_gain;

        let note = format!(
            "Scavengers returned: +{:.0} scrap, +{:.0} food, +{} pop",
            scrap, food, pop_gain
        );
        self.push_notification(note);
    }

    fn enemies_cleared(&self) -> bool {
        self.wave_manager.spawn_queue.is_empty() && self.wave_manager.alive_count() == 0
    }

    fn is_game_over(&self) -> bool {
        self.population.count == 0 || self.factory_integrity <= 0.0
    }

    fn build_run_summary(&self, survived: bool) -> RunSummary {
        RunSummary {
            waves_survived: self.current_wave,
            beacon_phase: self.beacon_phase.clone(),
            scavengers_sent: self.scavengers_sent,
            scavengers_returned: self.scavengers_returned,
            scavengers_lost: self.scavengers_lost,
            scavenger_scrap: self.scavenger_scrap_gained,
            scavenger_food: self.scavenger_food_gained,
            scavenger_population: self.scavenger_population_gained,
            factory_online: self.factory_online_count(),
            population_surviving: self.population.count,
            shutdown_triggered: survived,
            survivors_evacuated: self.survivors_evacuated,
            // A defeat forfeits the current beacon window's un-banked evacuees.
            evacuees_lost: if survived {
                0
            } else {
                self.pending_evacuees.floor() as u32
            },
        }
    }

    pub(crate) fn push_notification(&mut self, text: String) {
        self.notifications.push(super::Notification {
            text,
            ttl: self.constants.ui.notification_ttl,
        });
    }

    fn keep_building_survival_proof_defense(&mut self, data: &GameData) {
        const BUILD_PLAN: &[(&str, &str)] = &[
            ("ballistic_turret", "slot_01"),
            ("ballistic_turret", "slot_02"),
            ("ballistic_turret", "slot_03"),
            ("ballistic_turret", "slot_04"),
            ("ballistic_turret", "slot_05"),
            ("ballistic_turret", "slot_07"),
            ("ballistic_turret", "slot_09"),
            ("ballistic_turret", "slot_11"),
            ("ballistic_turret", "slot_13"),
            ("ballistic_turret", "slot_15"),
            ("ballistic_turret", "slot_17"),
        ];

        for (tower_id, slot_id) in BUILD_PLAN {
            self.try_build_survival_proof_tower(data, tower_id, slot_id);
        }
    }

    fn try_build_survival_proof_tower(&mut self, data: &GameData, tower_id: &str, slot_id: &str) {
        let Some(slot_idx) = self
            .map_state
            .slots
            .iter()
            .position(|slot| slot.id == slot_id)
        else {
            return;
        };
        let slot = &self.map_state.slots[slot_idx];
        if slot.state != SlotState::Powered || slot.tower_index.is_some() {
            return;
        }
        let Some(def) = data.tower_def_by_id(tower_id) else {
            return;
        };

        let grid_generation =
            self.factory.power_generation() + self.unlocked_building_boon().power_per_sec;
        let current_drain = self.factory.power_consumption()
            + self
                .towers
                .iter()
                .filter(|tower| tower.is_active)
                .map(|tower| tower.power_drain)
                .sum::<f32>();
        if current_drain + def.cost_power > grid_generation {
            return;
        }

        let before = self.towers.len();
        self.try_place_tower_on_slot(slot_idx, tower_id, data);
        if self.towers.len() > before {
            self.selected_tower = None;
        }
    }
}

/// Short narrative line pushed as a notification when the beacon escalates to a
/// new phase. `WarmSignal` is the opening state, so it has no line.
fn beacon_phase_flavor(phase: &BeaconPhase) -> Option<&'static str> {
    match phase {
        BeaconPhase::WarmSignal => None,
        BeaconPhase::SustainedCall => {
            Some("The beacon steadies into a sustained call — the swarm has your scent.")
        }
        BeaconPhase::ScreamingBeacon => {
            Some("Screaming beacon. Everything within range is turning toward you.")
        }
        BeaconPhase::TerminalHowl => {
            Some("Terminal howl — the factory is a dying star. Bank what you can.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_defense_can_survive_ten_waves() {
        let data = GameData::load();
        let mut state = GameplayState::new(&data);
        state.enable_survival_proof();

        for _ in 0..180 {
            if let Some(StateTransition::ToResults { summary }) = state.update_survival_proof(&data)
            {
                let (dom, dom_val) = state.threat.dominant();
                assert!(
                    summary.shutdown_triggered,
                    "defeat instead of shutdown: wave {}, pop {}, integrity {:.1}, food {:.1}, awareness {:.1}, loudest {} {:.1}",
                    summary.waves_survived,
                    summary.population_surviving,
                    state.factory_integrity,
                    state.population.food_supply,
                    state.threat.awareness_level(),
                    dom.label(),
                    dom_val,
                );
                assert_eq!(summary.waves_survived, 10);
                assert!(summary.population_surviving > 0);
                return;
            }
            assert!(
                !state.is_game_over(),
                "game over before ten waves: wave {}, population {}, integrity {:.1}, towers {}, scrap {:.1}, power {:.1}",
                state.current_wave,
                state.population.count,
                state.factory_integrity,
                state.towers.len(),
                state.resources.scrap,
                state.resources.power
            );
        }

        panic!("survival proof did not produce a wave-10 result");
    }

    #[test]
    fn beacon_shutdown_yields_report_then_allows_another_cycle() {
        let data = GameData::load();
        let mut state = GameplayState::new(&data);
        state.autosave_enabled = false;

        // Raise the beacon: a fresh field team goes out and this cycle's
        // baseline is snapshotted.
        state.start_beacon();
        assert!(state.beacon_active);
        let teams = data.constants.scavenger.initial_scavengers;
        assert_eq!(state.scavengers_out, teams);

        // Shut it down; with no enemies in the field the cycle finishes and
        // produces a salvage report instead of ending the run.
        state.trigger_shutdown();
        assert!(state.enemies_cleared());
        state.finish_beacon_cycle();

        let report = state
            .salvage_report
            .as_ref()
            .expect("shutdown with a clear field should produce a salvage report");
        assert_eq!(report.cycle, 1);
        assert_eq!(state.beacon_cycles_completed, 1);
        assert_eq!(state.scavengers_out, 0, "field teams resolve on shutdown");

        // Dismiss the report: back to base-build mode, beacon down, restartable.
        state.dismiss_salvage_report();
        assert!(state.salvage_report.is_none());
        assert!(!state.beacon_active);
        assert!(!state.shutdown_triggered);

        // A second beacon cycle sends another fresh team out.
        state.start_beacon();
        assert!(state.beacon_active);
        assert_eq!(state.scavengers_out, teams);
        assert_eq!(state.scavengers_sent, teams * 2);
    }
}
