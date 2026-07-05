use crate::data::BuildingBoon;
use crate::engine::beacon::BeaconPhase;
use crate::engine::enemy::EnemyType;
use crate::engine::map::{BuildingState, MapBuilding};
use crate::engine::threat::{ReactionTier, ThreatSignature};
use crate::engine::tower::Tower;
use crate::engine::wave::preview_wave;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::rng;

use super::{GameplayState, Particle};

pub fn entrance_label(id: &str) -> &str {
    match id {
        "north_gate" => "North Gate",
        "south_gate" => "South Gate",
        "east_breach" => "East Breach",
        "northwest_breach" => "Northwest Breach",
        // Route ids double as entrance labels on the map.
        "main_west" => "West Approach",
        _ => id,
    }
}

pub fn reaction_tier_rank(tier: &ReactionTier) -> u32 {
    match tier {
        ReactionTier::Observation => 0,
        ReactionTier::Adaptation => 1,
        ReactionTier::Suppression => 2,
        ReactionTier::Extermination => 3,
    }
}

pub fn threat_color(threat: &ThreatSignature) -> Color {
    let level = threat.awareness_level();
    if level < 25.0 {
        dark::TEXT_DIM
    } else if level < 60.0 {
        dark::WARNING
    } else {
        dark::NEGATIVE
    }
}

pub fn beacon_color(phase: &BeaconPhase) -> Color {
    match phase {
        BeaconPhase::WarmSignal => dark::TEXT_DIM,
        BeaconPhase::SustainedCall => dark::WARNING,
        BeaconPhase::ScreamingBeacon => dark::NEGATIVE,
        BeaconPhase::TerminalHowl => Color::new(1.0, 0.4, 0.1, 1.0),
    }
}

pub fn enemy_label(enemy_type: &EnemyType) -> &'static str {
    match enemy_type {
        EnemyType::Scout => "Scout",
        EnemyType::Drone => "Drone",
        EnemyType::HeavyUnit => "Heavy",
        EnemyType::Saboteur => "Saboteur",
        EnemyType::Commander => "Commander",
    }
}

pub fn apply_upgrade_levels(tower: &mut Tower, level: u32, constants: &crate::data::GameConstants) {
    if level <= 1 {
        return;
    }
    for _ in 1..level {
        tower.level += 1;
        tower.damage *= constants.tower.upgrade_damage_mult;
        tower.range *= constants.tower.upgrade_range_mult;
    }
}

impl GameplayState {
    pub fn is_tower_unlocked(&self, tower_id: &str) -> bool {
        match self.unlocks.towers.get(tower_id) {
            Some(rule) => self.unlock_requires_met(&rule.requires),
            None => false,
        }
    }

    pub fn is_building_unlocked(&self, building: &MapBuilding) -> bool {
        let _ = building;
        true
    }

    pub fn is_building_type_unlocked(&self, building_type: &str) -> bool {
        let _ = building_type;
        true
    }

    fn unlock_requires_met(&self, requires: &[String]) -> bool {
        requires.iter().all(|req| self.requirement_met(req))
    }

    /// True when a single unlock token (a powered building, an unlocked sector,
    /// or a purchased upgrade) is satisfied.
    pub fn requirement_met(&self, req: &str) -> bool {
        self.map_state.is_building_powered(req)
            || self.factory.is_sector_unlocked(req)
            || self.factory.has_upgrade(req)
    }

    pub fn unlocked_building_boon(&self) -> BuildingBoon {
        let mut boon = BuildingBoon::default();
        for building in &self.map_state.buildings {
            if !building.is_active() {
                continue;
            }
            boon.scrap_per_sec += building.boon.scrap_per_sec;
            boon.food_per_sec += building.boon.food_per_sec;
            boon.water_per_sec += building.boon.water_per_sec;
            boon.power_per_sec += building.boon.power_per_sec;
        }
        boon
    }

    pub fn unlocked_building_threat_per_sec(&self) -> f32 {
        self.map_state
            .buildings
            .iter()
            .filter(|b| b.is_active())
            .map(|b| b.threat_per_sec)
            .sum()
    }

    pub fn nearest_unlocked_building(&self, pos: Vec2) -> Option<(usize, f32)> {
        let mut best = None;
        let mut best_dist = self.map_state.building_interact_radius;
        let max_x = self.map_state.max_visible_x() + 120.0;
        for (idx, building) in self.map_state.buildings.iter().enumerate() {
            if !self.map_state.is_building_visible(building) {
                continue;
            }
            if building.position.x > max_x {
                continue;
            }
            let dist = (building.position - pos).length();
            if dist <= best_dist {
                best_dist = dist;
                best = Some(idx);
            }
        }
        best.map(|idx| (idx, best_dist))
    }

    pub fn available_upgrades(&self) -> Vec<&crate::data::UpgradeDef> {
        let mut upgrades: Vec<&crate::data::UpgradeDef> = self
            .upgrade_defs
            .iter()
            .filter(|u| self.factory.is_sector_unlocked(&u.sector))
            .filter(|u| self.factory.has_upgrade(&u.id) || self.factory.prereqs_met(u))
            .collect();
        upgrades.sort_by(|a, b| a.id.cmp(&b.id));
        upgrades
    }

    pub fn compute_beacon_start_difficulty_bonus(&self) -> f32 {
        let unlocked_sectors = self.factory.sectors.iter().filter(|s| s.unlocked).count() as f32;
        let unlocked_towers = self
            .unlocks
            .towers
            .keys()
            .filter(|id| self.is_tower_unlocked(id))
            .count() as f32;
        let unlocked_building_types = self
            .unlocks
            .buildings
            .keys()
            .filter(|t| self.is_building_type_unlocked(t))
            .count() as f32;
        let repaired_buildings = self
            .map_state
            .buildings
            .iter()
            .filter(|b| self.is_building_unlocked(b) && b.state != BuildingState::Broken)
            .count() as f32;
        let purchased_upgrades = self.factory.purchased_upgrades.len() as f32;
        let weights = &self.unlocks.difficulty_weights;

        (unlocked_sectors * weights.per_sector)
            + (unlocked_towers * weights.per_tower)
            + (unlocked_building_types * weights.per_building_type)
            + (repaired_buildings * weights.per_repaired_building)
            + (purchased_upgrades * weights.per_upgrade)
    }

    #[allow(dead_code)]
    pub fn build_wave_preview(&self) -> Vec<(EnemyType, usize)> {
        let next_wave = self.current_wave + 1;
        let force_commander = self.beacon_phase == BeaconPhase::TerminalHowl;
        let budget_bonus = if self.beacon_active {
            self.beacon_start_difficulty_bonus
        } else {
            0.0
        };
        let preview = preview_wave(
            next_wave,
            &self.enemy_defs,
            self.base_health_scale_per_wave,
            self.threat.awareness_level(),
            self.beacon_phase.tier_floor(),
            1.0 + budget_bonus,
            force_commander,
            self.constants.waves.budget_base,
            self.constants.waves.budget_per_wave,
            self.constants.waves.commander_every,
            self.constants.threat.budget_divisor,
            self.constants.threat.health_mult_per_awareness,
        );

        let mut counts: Vec<(EnemyType, usize)> = Vec::new();
        for enemy_type in preview {
            if let Some(entry) = counts.iter_mut().find(|(t, _)| *t == enemy_type) {
                entry.1 += 1;
            } else {
                counts.push((enemy_type, 1));
            }
        }
        counts
    }

    pub fn spawn_death_particles(&mut self, positions: &[Vec2]) {
        for pos in positions {
            let count = rng::gen_range(
                self.constants.particles.count_min as i32,
                (self.constants.particles.count_max + 1) as i32,
            );
            for _ in 0..count {
                let angle = rng::gen_range(0.0, std::f32::consts::TAU);
                let speed = rng::gen_range(
                    self.constants.particles.speed_min,
                    self.constants.particles.speed_max,
                );
                let velocity = vec2(angle.cos(), angle.sin()) * speed;
                self.particles.push(Particle {
                    position: *pos,
                    velocity,
                    ttl: self.constants.particles.ttl,
                });
            }
        }
    }

    pub fn update_particles(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.velocity *= self.constants.particles.drag;
            p.ttl -= dt;
        }
        self.particles.retain(|p| p.ttl > 0.0);
    }

    pub fn damage_random_sector(&mut self, amount: f32) {
        let unlocked_indices: Vec<usize> = self
            .factory
            .sectors
            .iter()
            .enumerate()
            .filter(|(_, s)| s.unlocked)
            .map(|(i, _)| i)
            .collect();

        if unlocked_indices.is_empty() {
            return;
        }

        let idx = if unlocked_indices.len() == 1 {
            unlocked_indices[0]
        } else {
            let pick = rng::gen_range(0, unlocked_indices.len());
            unlocked_indices[pick]
        };

        if let Some(sector) = self.factory.sectors.get_mut(idx) {
            sector.integrity = (sector.integrity - amount).max(0.0);
        }
    }

    pub fn damage_lowest_integrity_sector(&mut self, amount: f32) {
        let mut lowest_idx = None;
        let mut lowest_ratio = f32::MAX;
        for (i, sector) in self.factory.sectors.iter().enumerate() {
            if !sector.unlocked {
                continue;
            }
            let ratio = sector.integrity / sector.max_integrity;
            if ratio < lowest_ratio {
                lowest_ratio = ratio;
                lowest_idx = Some(i);
            }
        }

        if let Some(idx) = lowest_idx {
            if let Some(sector) = self.factory.sectors.get_mut(idx) {
                sector.integrity = (sector.integrity - amount).max(0.0);
            }
        }
    }

    pub fn recalc_factory_integrity(&mut self) {
        let mut total_ratio = 0.0;
        let mut count = 0.0;
        for sector in &self.factory.sectors {
            if sector.unlocked {
                total_ratio += (sector.integrity / sector.max_integrity).clamp(0.0, 1.0);
                count += 1.0;
            }
        }
        self.factory_integrity = if count > 0.0 {
            (total_ratio / count) * 100.0
        } else {
            0.0
        };
    }

    pub fn factory_online_count(&self) -> usize {
        self.factory
            .sectors
            .iter()
            .filter(|s| s.unlocked && s.integrity > 0.0)
            .count()
    }

    pub fn upgrade_tower(&mut self, idx: usize) {
        if !self.factory.is_sector_active("research_lab") {
            return;
        }
        let max_level_bonus =
            self.factory
                .upgrade_effect("max_tower_level_bonus", &self.upgrade_defs) as u32;
        let max_level = self.constants.tower.upgrade_max_level + max_level_bonus;
        let Some(tower) = self.towers.get_mut(idx) else {
            return;
        };
        if tower.level >= max_level {
            return;
        }
        let cost_reduction = self
            .factory
            .upgrade_effect("upgrade_cost_reduction", &self.upgrade_defs);
        let cost = tower.base_scrap_cost * tower.level as f32 * (1.0 - cost_reduction);
        if self.resources.scrap < cost {
            return;
        }
        self.resources.scrap -= cost;
        tower.level += 1;
        tower.damage *= self.constants.tower.upgrade_damage_mult;
        tower.range *= self.constants.tower.upgrade_range_mult;
    }
}
