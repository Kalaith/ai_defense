//! Tower placement, targeting, and combat.

use crate::engine::enemy::{Enemy, EnemyType};
use macroquad::prelude::{Color, Vec2};

#[derive(Clone, Debug)]
pub enum TowerType {
    Ballistic,
    Laser,
    Emp,
    AreaDenial,
    Subversion,
}

pub struct Tower {
    pub tower_type: TowerType,
    pub tower_id: String,
    pub position: Vec2,
    pub level: u32,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
    pub cooldown: f32,
    pub is_active: bool,
    pub power_drain: f32,
    pub base_scrap_cost: f32,
}

#[derive(Clone, Debug)]
pub struct TowerTuning {
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
}

#[derive(Clone, Debug)]
pub enum ShotEffect {
    Line {
        from: Vec2,
        to: Vec2,
        color: Color,
        ttl: f32,
        max_ttl: f32,
    },
    Pulse {
        center: Vec2,
        radius: f32,
        color: Color,
        ttl: f32,
        max_ttl: f32,
    },
}

impl ShotEffect {
    pub fn line(from: Vec2, to: Vec2, color: Color, ttl: f32) -> Self {
        Self::Line {
            from,
            to,
            color,
            ttl,
            max_ttl: ttl,
        }
    }

    pub fn pulse(center: Vec2, radius: f32, color: Color, ttl: f32) -> Self {
        Self::Pulse {
            center,
            radius,
            color,
            ttl,
            max_ttl: ttl,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        match self {
            ShotEffect::Line { ttl, .. } | ShotEffect::Pulse { ttl, .. } => {
                *ttl -= dt;
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            ShotEffect::Line { ttl, .. } | ShotEffect::Pulse { ttl, .. } => *ttl > 0.0,
        }
    }

    pub fn alpha(&self) -> f32 {
        match self {
            ShotEffect::Line { ttl, max_ttl, .. } | ShotEffect::Pulse { ttl, max_ttl, .. } => {
                (*ttl / *max_ttl).clamp(0.0, 1.0)
            }
        }
    }
}

pub struct TowerCombatResult {
    pub scrap_earned: f32,
    pub effects: Vec<ShotEffect>,
    pub heat_generated: f32,
    pub death_positions: Vec<Vec2>,
}

impl Tower {
    pub fn new(
        tower_type: TowerType,
        tower_id: String,
        position: Vec2,
        range: f32,
        damage: f32,
        fire_rate: f32,
        power_drain: f32,
        base_scrap_cost: f32,
    ) -> Self {
        Self {
            tower_type,
            tower_id,
            position,
            level: 1,
            range,
            damage,
            fire_rate,
            cooldown: 0.0,
            is_active: true,
            power_drain,
            base_scrap_cost,
        }
    }

    pub fn can_fire(&self) -> bool {
        self.is_active && self.cooldown <= 0.0
    }

    pub fn fire(&mut self, fire_rate_mult: f32) {
        let rate = (self.fire_rate * fire_rate_mult).max(0.01);
        self.cooldown = 1.0 / rate;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.cooldown > 0.0 {
            self.cooldown -= dt;
        }
    }

    pub fn nearest_enemy<'a>(&self, enemies: &'a [Enemy]) -> Option<usize> {
        let mut best_idx = None;
        let mut best_dist = f32::MAX;
        for (i, enemy) in enemies.iter().enumerate() {
            if !enemy.is_alive {
                continue;
            }
            let dist = (enemy.position - self.position).length();
            if dist <= self.range && dist < best_dist {
                best_dist = dist;
                best_idx = Some(i);
            }
        }
        best_idx
    }

    pub fn color(&self) -> macroquad::prelude::Color {
        use macroquad::prelude::*;
        match self.tower_type {
            TowerType::Ballistic => ORANGE,
            TowerType::Laser => RED,
            TowerType::Emp => SKYBLUE,
            TowerType::AreaDenial => YELLOW,
            TowerType::Subversion => PURPLE,
        }
    }
}

/// Run tower combat: each tower finds a target, fires if ready, applies damage.
/// Returns total scrap earned from kills.
pub fn tick_towers(
    towers: &mut [Tower],
    enemies: &mut [Enemy],
    dt: f32,
    damage_mult: f32,
    fire_rate_mult: f32,
    range_mult: f32,
    scrap_mult: f32,
    tuning: &TowerTuning,
) -> TowerCombatResult {
    let mut scrap_earned = 0.0;
    let mut effects = Vec::new();
    let mut fired_count = 0.0;
    let mut death_positions = Vec::new();

    for tower in towers.iter_mut() {
        tower.tick(dt);
        if !tower.can_fire() {
            continue;
        }

        if matches!(tower.tower_type, TowerType::AreaDenial) {
            let mut hit_any = false;
            let range = tower.range * range_mult;
            for enemy in enemies.iter_mut() {
                if !enemy.is_alive {
                    continue;
                }
                let dist = (enemy.position - tower.position).length();
                if dist <= range {
                    hit_any = true;
                    let was_alive = enemy.is_alive;
                    enemy.take_damage(tower.damage * damage_mult * tuning.area_denial_damage_scale);
                    if was_alive && !enemy.is_alive {
                        scrap_earned += enemy.scrap_reward * scrap_mult;
                        death_positions.push(enemy.position);
                    }
                }
            }

            if hit_any {
                tower.fire(fire_rate_mult);
                fired_count += 1.0;
                effects.push(ShotEffect::pulse(tower.position, range, tower.color(), tuning.shot_ttl));
            }
            continue;
        }

        // Find nearest enemy in range
        let mut best_idx = None;
        let mut best_dist = f32::MAX;
        let range = tower.range * range_mult;
        for (i, enemy) in enemies.iter().enumerate() {
            if !enemy.is_alive {
                continue;
            }
            let dist = (enemy.position - tower.position).length();
            if dist <= range && dist < best_dist {
                best_dist = dist;
                best_idx = Some(i);
            }
        }

        if let Some(idx) = best_idx {
            tower.fire(fire_rate_mult);
            fired_count += 1.0;
            let target_pos = enemies[idx].position;
            let target_type = enemies[idx].enemy_type.clone();
            let mut damage = tower.damage * damage_mult;
            if matches!(tower.tower_type, TowerType::Laser) {
                match target_type {
                    EnemyType::HeavyUnit => damage *= tuning.laser_vs_heavy_mult,
                    EnemyType::Scout => damage *= tuning.laser_vs_scout_mult,
                    _ => {}
                }
            } else if matches!(tower.tower_type, TowerType::Ballistic) {
                if matches!(target_type, EnemyType::HeavyUnit) {
                    damage *= tuning.ballistic_vs_heavy_mult;
                }
            }

            let was_alive = enemies[idx].is_alive;
            enemies[idx].take_damage(damage);

            if matches!(tower.tower_type, TowerType::Emp) {
                enemies[idx].slowed_timer = enemies[idx].slowed_timer.max(tuning.emp_slow_duration);
            }

            if was_alive && !enemies[idx].is_alive {
                scrap_earned += enemies[idx].scrap_reward * scrap_mult;
                death_positions.push(enemies[idx].position);

                if matches!(enemies[idx].enemy_type, EnemyType::Commander) {
                    let burst_damage = enemies[idx].max_health * tuning.commander_death_fraction;
                    let burst_center = enemies[idx].position;
                    for enemy in enemies.iter_mut() {
                        if !enemy.is_alive {
                            continue;
                        }
                        let dist = (enemy.position - burst_center).length();
                        if dist <= tuning.commander_death_radius {
                            let was_burst_alive = enemy.is_alive;
                            enemy.take_damage(burst_damage);
                            if was_burst_alive && !enemy.is_alive {
                                scrap_earned += enemy.scrap_reward * scrap_mult;
                                death_positions.push(enemy.position);
                            }
                        }
                    }
                }

                if matches!(tower.tower_type, TowerType::Subversion) {
                    let chain_damage = enemies[idx].max_health * tuning.subversion_chain_damage_fraction * damage_mult;
                    let chain_center = enemies[idx].position;
                    for enemy in enemies.iter_mut() {
                        if !enemy.is_alive {
                            continue;
                        }
                        let dist = (enemy.position - chain_center).length();
                        if dist <= tuning.subversion_chain_radius {
                            let was_chain_alive = enemy.is_alive;
                            enemy.take_damage(chain_damage);
                            if was_chain_alive && !enemy.is_alive {
                                scrap_earned += enemy.scrap_reward * scrap_mult;
                                death_positions.push(enemy.position);
                            }
                        }
                    }
                }
            }

            effects.push(ShotEffect::line(tower.position, target_pos, tower.color(), tuning.shot_ttl));
        }
    }

    let heat_generated = fired_count * 0.01 * dt;
    TowerCombatResult {
        scrap_earned,
        effects,
        heat_generated,
        death_positions,
    }
}
