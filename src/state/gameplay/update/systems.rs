//! Per-system ticks the frame loop drives in order: waves, combat, power,
//! population, boons, threat, and the cosmetic timers.

use crate::engine::beacon::BeaconPhase;
use crate::engine::enemy::EnemyType;
use crate::engine::tower::{tick_towers, TowerTuning};
use crate::engine::wave::WaveEvent;
use macroquad::prelude::*;
use std::collections::HashMap;

use super::super::GameplayState;

impl GameplayState {
    pub(super) fn update_wave_timers(&mut self, dt: f32) {
        if self.beacon_active && !self.shutdown_triggered && self.between_waves {
            self.wave_timer -= dt;
            if self.wave_timer <= 0.0 {
                self.current_wave += 1;
                self.between_waves = false;
            }
        }
    }

    pub(super) fn start_wave_if_ready(&mut self) {
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

    pub(super) fn tick_wave(&mut self, dt: f32) {
        let paths: HashMap<String, Vec<Vec2>> = self
            .map_state
            .active_paths_limited()
            .into_iter()
            .map(|p| (p.id.clone(), p.points))
            .collect();

        match self.wave_manager.tick(dt, &paths) {
            WaveEvent::EnemyReachedEnd { enemy_type } => self.handle_breach(enemy_type),
            WaveEvent::WaveComplete => self.handle_wave_complete(),
            WaveEvent::None => {}
        }
    }

    pub(super) fn update_combat(&mut self, dt: f32) {
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
            heat_per_shot: self.constants.tower.heat_per_shot,
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

    pub(super) fn update_power(&mut self, dt: f32) {
        let building_power = self.unlocked_building_boon().power_per_sec;
        let gen = self.factory.power_generation(&self.constants.economy) + building_power;
        let tower_drain: f32 = self
            .towers
            .iter()
            .filter(|t| t.is_active)
            .map(|t| t.power_drain)
            .sum();
        let consume = self.factory.power_consumption() + tower_drain;
        self.resources.power = (self.resources.power + (gen - consume) * dt)
            .clamp(0.0, self.constants.economy.power_cap);

        // Battery empty and still in deficit: shed towers until draw fits supply.
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

    pub(super) fn update_population(&mut self, dt: f32) {
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

    pub(super) fn update_building_boons(&mut self, dt: f32) {
        let boon = self.unlocked_building_boon();
        self.resources.scrap += boon.scrap_per_sec * dt;
        self.population.food_supply += boon.food_per_sec * dt;
    }

    pub(super) fn update_threat(&mut self, dt: f32) {
        // Gather everything the whole-self helpers need up front, before taking
        // the mutable borrow of self.threat below.
        let building_threat = self.unlocked_building_threat_per_sec();
        let power_gen = self.factory.power_generation(&self.constants.economy)
            + self.unlocked_building_boon().power_per_sec;
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

        self.threat.tick_decay(dt, &self.constants.threat);
        let current_tier = self.threat.reaction_tier(&self.constants.threat);
        if super::super::helpers::reaction_tier_rank(&current_tier)
            > super::super::helpers::reaction_tier_rank(&self.last_reaction_tier)
        {
            self.threat_flash_timer = self.constants.ui.threat_flash_duration;
        }
        self.last_reaction_tier = current_tier;
        if self.threat_flash_timer > 0.0 {
            self.threat_flash_timer = (self.threat_flash_timer - dt).max(0.0);
        }
    }

    pub(super) fn update_factory(&mut self) {
        self.factory.check_awakening();
        self.recalc_factory_integrity();
        self.map_state.update_section_visibility();
        self.sync_camera_bounds();
    }

    pub(super) fn update_notifications(&mut self, dt: f32) {
        for note in &mut self.notifications {
            note.ttl -= dt;
        }
        self.notifications.retain(|n| n.ttl > 0.0);
    }

    pub(super) fn update_wave_flash(&mut self, dt: f32) {
        if self.wave_flash_timer > 0.0 {
            self.wave_flash_timer = (self.wave_flash_timer - dt).max(0.0);
        }
    }

    /// An enemy reached the core: scouts only raise noise, everything else costs
    /// a life and damages the factory.
    fn handle_breach(&mut self, enemy_type: EnemyType) {
        match enemy_type {
            EnemyType::Scout => {
                self.threat
                    .add_noise(self.constants.threat.noise_scout_breach);
                self.push_notification(
                    crate::data::strings::text()
                        .notifications
                        .scout_breach
                        .clone(),
                );
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
            food_reward *= self.constants.waves.logistics_hub_food_mult;
        }
        self.population.food_supply += food_reward;

        self.autosave();

        if self.beacon_active && !self.shutdown_triggered {
            self.between_waves = true;
            self.wave_timer = self.wave_interval;
        }
    }
}
