use crate::engine::beacon::{phase_from_strength, BeaconPhase};
use crate::engine::enemy::EnemyType;
use crate::engine::tower::tick_towers;
use crate::engine::tower::TowerTuning;
use crate::engine::wave::WaveEvent;
use crate::state::{RunSummary, StateTransition};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::collections::HashMap;

use super::GameplayState;

impl GameplayState {
    pub fn update(&mut self) -> Option<StateTransition> {
        self.handle_camera_input();
        self.handle_input();
        if self.paused {
            return None;
        }

        let dt = get_frame_time() * self.time_scale;
        self.validate_selection();

        if self.beacon_active {
            self.update_beacon();
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

        if self.shutdown_triggered && self.enemies_cleared() {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(),
            });
        }

        if self.is_game_over() {
            return Some(StateTransition::ToResults {
                summary: self.build_run_summary(),
            });
        }

        None
    }

    fn handle_camera_input(&mut self) {
        let dt = get_frame_time();
        let pan_speed = 400.0 / self.camera_zoom;

        // WASD panning
        if is_key_down(KeyCode::W) { self.camera_offset.y -= pan_speed * dt; }
        if is_key_down(KeyCode::S) { self.camera_offset.y += pan_speed * dt; }
        if is_key_down(KeyCode::A) { self.camera_offset.x -= pan_speed * dt; }
        if is_key_down(KeyCode::D) { self.camera_offset.x += pan_speed * dt; }

        // Middle-mouse drag
        let (mx, my) = mouse_position();
        let mouse_pos = vec2(mx, my);
        if is_mouse_button_down(MouseButton::Middle) {
            let delta = self.prev_mouse_pos - mouse_pos;
            self.camera_offset += delta / self.camera_zoom;
        }
        self.prev_mouse_pos = mouse_pos;

        // Scroll wheel zoom
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            let factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
            self.camera_zoom = (self.camera_zoom * factor).clamp(0.25, 2.0);
        }

        // Clamp camera to map bounds
        let map_w = self.map_state.map_size.x;
        let map_h = self.map_state.map_size.y;
        self.camera_offset.x = self.camera_offset.x.clamp(0.0, map_w);
        self.camera_offset.y = self.camera_offset.y.clamp(0.0, map_h);
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let sw = screen_width();
        let sh = screen_height();
        let world_x = self.camera_offset.x + (screen_pos.x - sw / 2.0) / self.camera_zoom;
        let world_y = self.camera_offset.y + (screen_pos.y - sh / 2.0) / self.camera_zoom;
        vec2(world_x, world_y)
    }

    fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            if self.placing_tower.is_some() {
                self.placing_tower = None;
            } else {
                self.paused = !self.paused;
            }
        }

        if is_key_pressed(KeyCode::Space) {
            self.time_scale = if self.time_scale > 1.0 { 1.0 } else { self.constants.gameplay.speed_multiplier };
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
        if !self.beacon_active || self.shutdown_triggered || self.between_waves || self.wave_manager.wave_active {
            return;
        }

        let spawn_points: Vec<(String, Vec2)> = self.map_state.active_paths()
            .iter()
            .map(|p| (p.id.clone(), p.entrance))
            .collect();

        if spawn_points.is_empty() {
            return;
        }

        let force_commander = self.beacon_phase == BeaconPhase::TerminalHowl;
        let budget_multiplier = self.constants.waves.budget_multiplier + self.beacon_start_difficulty_bonus;
        self.wave_manager.generate_wave(
            self.current_wave,
            &self.enemy_defs,
            self.base_health_scale_per_wave,
            self.threat.awareness_level(),
            self.beacon_phase.tier_floor(),
            budget_multiplier,
            force_commander,
            &spawn_points,
        );
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
        let damage_mult = if self.factory.is_sector_active("assembly_hall") { self.constants.sector.bonus_damage_mult } else { 1.0 };
        let fire_rate_mult = if self.factory.is_sector_active("robotics_bay") { self.constants.sector.bonus_fire_rate_mult } else { 1.0 };
        let range_mult = if self.factory.is_sector_active("ai_vault") { self.constants.sector.bonus_range_mult } else { 1.0 };
        let scrap_mult = if self.factory.is_sector_active("logistics_hub") { self.constants.sector.bonus_scrap_mult } else { 1.0 };

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
        let tower_drain: f32 = self.towers.iter().filter(|t| t.is_active).map(|t| t.power_drain).sum();
        let consume = self.factory.power_consumption() + tower_drain;
        self.resources.power = (self.resources.power + (gen - consume) * dt).clamp(0.0, self.constants.economy.power_cap);

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
        self.population.tick(dt, &self.constants);
        self.resources.scrap += self.population.productivity(&self.constants) * self.constants.economy.productivity_scrap_rate * dt;
    }

    fn update_building_boons(&mut self, dt: f32) {
        let boon = self.unlocked_building_boon();
        self.resources.scrap += boon.scrap_per_sec * dt;
        self.population.food_supply += boon.food_per_sec * dt;
    }

    fn update_threat(&mut self, dt: f32) {
        // Building threat contribution
        let building_threat = self.unlocked_building_threat_per_sec();
        if building_threat > 0.0 {
            self.threat.add_noise(building_threat * dt);
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
                self.threat.add_noise(self.constants.threat.noise_scout_breach);
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
        let ai_vault_tier = if self.factory.is_sector_active("ai_vault") { 1.0 } else { 0.0 };
        let population = self.population.count as f32;

        self.beacon_strength = (unlocked * 2.0)
            + (power_throughput / 10.0)
            + (ai_vault_tier * 5.0)
            + (population / 20.0);

        let next_phase = phase_from_strength(self.beacon_strength);
        if next_phase.rank() > self.beacon_phase.rank() {
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
            BeaconPhase::WarmSignal => {
                (self.constants.scavenger.warm_scrap, self.constants.scavenger.warm_food, 0)
            }
            BeaconPhase::SustainedCall => {
                let pop = if gen_range(0.0, 1.0) < self.constants.scavenger.sustained_pop_chance {
                    1
                } else {
                    0
                };
                (self.constants.scavenger.sustained_scrap, self.constants.scavenger.sustained_food, pop)
            }
            BeaconPhase::ScreamingBeacon => {
                (
                    self.constants.scavenger.screaming_scrap,
                    self.constants.scavenger.screaming_food,
                    self.constants.scavenger.screaming_pop_gain,
                )
            }
            BeaconPhase::TerminalHowl => {
                if gen_range(0.0, 1.0) < self.constants.scavenger.terminal_loss_chance {
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

        let note = format!("Scavengers returned: +{:.0} scrap, +{:.0} food, +{} pop", scrap, food, pop_gain);
        self.push_notification(note);
    }

    fn enemies_cleared(&self) -> bool {
        self.wave_manager.spawn_queue.is_empty() && self.wave_manager.alive_count() == 0
    }

    fn is_game_over(&self) -> bool {
        self.population.count == 0 || self.factory_integrity <= 0.0
    }

    fn build_run_summary(&self) -> RunSummary {
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
            shutdown_triggered: self.shutdown_triggered,
        }
    }

    pub(crate) fn push_notification(&mut self, text: String) {
        self.notifications.push(super::Notification {
            text,
            ttl: self.constants.ui.notification_ttl,
        });
    }
}
