//! Tower placement, targeting, and combat.

use crate::data::SpecializationEffect;
use crate::engine::enemy::{Enemy, EnemyType};
use macroquad::prelude::{Color, Vec2};
use macroquad_toolkit::timing::Cooldown;

/// `TowerType` lives in `data` — it's a plain data-driven enum (see
/// `TowerDef::tower_type`), and data must never depend on engine (§2.1).
pub use crate::data::TowerType;

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
    pub color: Color,
    pub specialization_id: Option<String>,
    pub specialization_effect: Option<SpecializationEffect>,
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
    pub heat_per_shot: f32,
}

/// A world-space shot trail (line or ring) that fades out over its
/// lifetime. Geometry stays local; the countdown/fade fraction is delegated
/// to the shared [`Cooldown`] timer (it counts down from its duration to
/// zero, exactly matching the old `ttl`/`max_ttl` pair).
#[derive(Clone, Debug)]
pub enum ShotEffect {
    Line {
        from: Vec2,
        to: Vec2,
        color: Color,
        life: Cooldown,
    },
    Pulse {
        center: Vec2,
        radius: f32,
        color: Color,
        life: Cooldown,
    },
}

impl ShotEffect {
    pub fn line(from: Vec2, to: Vec2, color: Color, ttl: f32) -> Self {
        Self::Line {
            from,
            to,
            color,
            life: Cooldown::new_armed(ttl),
        }
    }

    pub fn pulse(center: Vec2, radius: f32, color: Color, ttl: f32) -> Self {
        Self::Pulse {
            center,
            radius,
            color,
            life: Cooldown::new_armed(ttl),
        }
    }

    pub fn tick(&mut self, dt: f32) {
        match self {
            ShotEffect::Line { life, .. } | ShotEffect::Pulse { life, .. } => life.tick(dt),
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            ShotEffect::Line { life, .. } | ShotEffect::Pulse { life, .. } => !life.is_ready(),
        }
    }

    pub fn alpha(&self) -> f32 {
        match self {
            ShotEffect::Line { life, .. } | ShotEffect::Pulse { life, .. } => {
                life.fraction_remaining()
            }
        }
    }
}

pub struct TowerCombatResult {
    pub scrap_earned: f32,
    pub effects: Vec<ShotEffect>,
    pub heat_generated: f32,
    pub death_positions: Vec<Vec2>,
    pub tower_stats: Vec<TowerTickStats>,
}

#[derive(Clone, Debug, Default)]
pub struct TowerTickStats {
    pub shots: u32,
    pub hits: u32,
    pub kills: u32,
}

impl Tower {
    // Each param is a distinct tower-def field with no natural grouping; a
    // config struct would just move the field list, not shrink it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tower_type: TowerType,
        tower_id: String,
        position: Vec2,
        range: f32,
        damage: f32,
        fire_rate: f32,
        power_drain: f32,
        base_scrap_cost: f32,
        color: Color,
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
            color,
            specialization_id: None,
            specialization_effect: None,
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

    pub fn color(&self) -> macroquad::prelude::Color {
        self.color
    }

    pub fn specialize(&mut self, id: String, effect: SpecializationEffect) {
        if self.specialization_id.is_some() {
            return;
        }
        match effect {
            SpecializationEffect::RapidFire => self.fire_rate *= 1.75,
            SpecializationEffect::WideField => self.range *= 1.45,
            _ => {}
        }
        self.specialization_id = Some(id);
        self.specialization_effect = Some(effect);
    }
}

/// Run tower combat: each tower finds a target, fires if ready, applies damage.
/// Returns total scrap earned from kills.
// Params are independent inputs from separate subsystems (enemies, path,
// power grid, dt); bundling them would just hide the coupling in a struct.
#[allow(clippy::too_many_arguments)]
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
    let mut tower_stats = vec![TowerTickStats::default(); towers.len()];

    for (tower_idx, tower) in towers.iter_mut().enumerate() {
        tower.tick(dt);
        if !tower.can_fire() {
            continue;
        }

        if matches!(tower.tower_type, TowerType::AreaDenial) {
            let mut hit_any = false;
            let mut hits = 0;
            let mut kills = 0;
            let range = tower.range * range_mult;
            for enemy in enemies.iter_mut() {
                if !enemy.is_alive {
                    continue;
                }
                let dist = (enemy.position - tower.position).length();
                if dist <= range {
                    hit_any = true;
                    hits += 1;
                    let was_alive = enemy.is_alive;
                    let specialization_mult = if matches!(
                        tower.specialization_effect,
                        Some(SpecializationEffect::Shredder)
                    ) {
                        1.75
                    } else {
                        1.0
                    };
                    enemy.take_damage(
                        tower.damage
                            * damage_mult
                            * tuning.area_denial_damage_scale
                            * specialization_mult,
                        &tower.tower_type,
                    );
                    if was_alive && !enemy.is_alive {
                        kills += 1;
                        scrap_earned += enemy.scrap_reward * scrap_mult;
                        death_positions.push(enemy.position);
                    }
                }
            }

            if hit_any {
                tower.fire(fire_rate_mult);
                fired_count += 1.0;
                tower_stats[tower_idx].shots += 1;
                tower_stats[tower_idx].hits += hits;
                tower_stats[tower_idx].kills += kills;
                effects.push(ShotEffect::pulse(
                    tower.position,
                    range,
                    tower.color(),
                    tuning.shot_ttl,
                ));
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
            tower_stats[tower_idx].shots += 1;
            tower_stats[tower_idx].hits += 1;
            let target_pos = enemies[idx].position;
            let mut damage = tower.damage * damage_mult;
            if matches!(
                tower.specialization_effect,
                Some(SpecializationEffect::Execute)
            ) && enemies[idx].health <= enemies[idx].max_health * 0.5
            {
                damage *= 1.75;
            }
            if matches!(
                tower.specialization_effect,
                Some(SpecializationEffect::CommandBreaker)
            ) && enemies[idx].enemy_type == EnemyType::Commander
            {
                damage *= 2.25;
            }

            let was_alive = enemies[idx].is_alive;
            enemies[idx].take_damage_with_resistance(
                damage,
                &tower.tower_type,
                matches!(
                    tower.specialization_effect,
                    Some(SpecializationEffect::ArmorPiercing)
                ),
            );

            if matches!(tower.tower_type, TowerType::Emp) {
                let slow_duration = if matches!(
                    tower.specialization_effect,
                    Some(SpecializationEffect::DeepSlow)
                ) {
                    tuning.emp_slow_duration * 2.5
                } else {
                    tuning.emp_slow_duration
                };
                enemies[idx].slowed_timer = enemies[idx].slowed_timer.max(slow_duration);
            }

            if matches!(
                tower.specialization_effect,
                Some(SpecializationEffect::ChainBeam)
            ) {
                let second = enemies
                    .iter()
                    .enumerate()
                    .filter(|(other_idx, enemy)| *other_idx != idx && enemy.is_alive)
                    .filter_map(|(other_idx, enemy)| {
                        let dist = (enemy.position - target_pos).length();
                        (dist <= 100.0).then_some((other_idx, dist))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(other_idx, _)| other_idx);
                if let Some(other_idx) = second {
                    let was_second_alive = enemies[other_idx].is_alive;
                    let second_pos = enemies[other_idx].position;
                    enemies[other_idx].take_damage(damage * 0.5, &tower.tower_type);
                    tower_stats[tower_idx].hits += 1;
                    effects.push(ShotEffect::line(
                        target_pos,
                        second_pos,
                        tower.color(),
                        tuning.shot_ttl,
                    ));
                    if was_second_alive && !enemies[other_idx].is_alive {
                        tower_stats[tower_idx].kills += 1;
                        scrap_earned += enemies[other_idx].scrap_reward * scrap_mult;
                        death_positions.push(second_pos);
                    }
                }
            }

            if matches!(
                tower.specialization_effect,
                Some(SpecializationEffect::ArcPulse)
            ) {
                for (other_idx, enemy) in enemies.iter_mut().enumerate() {
                    if other_idx == idx || !enemy.is_alive {
                        continue;
                    }
                    if (enemy.position - target_pos).length() > 80.0 {
                        continue;
                    }
                    let was_other_alive = enemy.is_alive;
                    enemy.take_damage(damage * 0.25, &tower.tower_type);
                    enemy.slowed_timer = enemy.slowed_timer.max(tuning.emp_slow_duration);
                    tower_stats[tower_idx].hits += 1;
                    if was_other_alive && !enemy.is_alive {
                        tower_stats[tower_idx].kills += 1;
                        scrap_earned += enemy.scrap_reward * scrap_mult;
                        death_positions.push(enemy.position);
                    }
                }
                effects.push(ShotEffect::pulse(
                    target_pos,
                    80.0,
                    tower.color(),
                    tuning.shot_ttl,
                ));
            }

            if was_alive && !enemies[idx].is_alive {
                tower_stats[tower_idx].kills += 1;
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
                            enemy.take_damage(burst_damage, &tower.tower_type);
                            if was_burst_alive && !enemy.is_alive {
                                tower_stats[tower_idx].kills += 1;
                                scrap_earned += enemy.scrap_reward * scrap_mult;
                                death_positions.push(enemy.position);
                            }
                        }
                    }
                }

                if matches!(tower.tower_type, TowerType::Subversion) {
                    let cascade = matches!(
                        tower.specialization_effect,
                        Some(SpecializationEffect::ViralCascade)
                    );
                    let chain_damage = enemies[idx].max_health
                        * tuning.subversion_chain_damage_fraction
                        * damage_mult
                        * if cascade { 1.6 } else { 1.0 };
                    let chain_center = enemies[idx].position;
                    let chain_radius =
                        tuning.subversion_chain_radius * if cascade { 1.6 } else { 1.0 };
                    for enemy in enemies.iter_mut() {
                        if !enemy.is_alive {
                            continue;
                        }
                        let dist = (enemy.position - chain_center).length();
                        if dist <= chain_radius {
                            let was_chain_alive = enemy.is_alive;
                            enemy.take_damage(chain_damage, &tower.tower_type);
                            if was_chain_alive && !enemy.is_alive {
                                tower_stats[tower_idx].kills += 1;
                                scrap_earned += enemy.scrap_reward * scrap_mult;
                                death_positions.push(enemy.position);
                            }
                        }
                    }
                }
            }

            effects.push(ShotEffect::line(
                tower.position,
                target_pos,
                tower.color(),
                tuning.shot_ttl,
            ));
        }
    }

    let heat_generated = fired_count * tuning.heat_per_shot * dt;
    TowerCombatResult {
        scrap_earned,
        effects,
        heat_generated,
        death_positions,
        tower_stats,
    }
}

#[cfg(test)]
mod tests;
