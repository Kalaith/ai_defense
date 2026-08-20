//! Wave spawning and progression (refactored).

use crate::data::{DamageMultipliers, EnemyDef};
use crate::engine::enemy::{Enemy, EnemyTuning, EnemyType};
use macroquad::prelude::Vec2;
use std::collections::HashMap;

pub enum WaveEvent {
    None,
    EnemyReachedEnd { enemy_type: EnemyType },
    ScoutReport { position: Vec2 },
    SaboteurStrike { position: Vec2 },
    CommanderPulse { position: Vec2, radius: f32 },
    WaveComplete,
}

pub struct WaveManager {
    pub current_wave: u32,
    pub enemies: Vec<Enemy>,
    pub spawn_queue: Vec<SpawnEntry>,
    pub spawn_timer: f32,
    pub wave_active: bool,
    pub spawn_interval: f32,
    pub commander_aura_radius: f32,
    pub commander_aura_speed_mult: f32,
    pub enemy_tuning: EnemyTuning,
    pub wave_budget_base: u32,
    pub wave_budget_per_wave: u32,
    pub wave_commander_every: u32,
    pub threat_budget_divisor: f32,
    pub threat_health_mult_per_awareness: f32,
    pub tier_2_awareness: f32,
    pub tier_3_awareness: f32,
    pub enemy_abilities_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct WaveTuning {
    pub spawn_interval: f32,
    pub commander_aura_radius: f32,
    pub commander_aura_speed_mult: f32,
    pub enemy_tuning: EnemyTuning,
    pub wave_budget_base: u32,
    pub wave_budget_per_wave: u32,
    pub wave_commander_every: u32,
    pub threat_budget_divisor: f32,
    pub threat_health_mult_per_awareness: f32,
    /// Awareness-level boundaries that unlock enemy tiers early — kept in
    /// sync with `ThreatSignature::reaction_tier` via `constants.threat`
    /// (both read the same JSON, avoiding the drift a hardcoded copy risks).
    pub tier_2_awareness: f32,
    pub tier_3_awareness: f32,
}

pub struct SpawnEntry {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub speed: f32,
    pub scrap_reward: f32,
    pub spawn_point: Vec2,
    pub path_id: String,
    pub damage_multipliers: DamageMultipliers,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewSpawnEntry {
    pub enemy_type: EnemyType,
    pub path_id: String,
}

/// Qualitative adaptation applied to a wave's composition, derived from the
/// loudest machine-awareness signature. The machines answer *how* the player
/// plays, not just how hard.
#[derive(Clone, Debug, Default)]
pub struct WaveAdaptation {
    /// Archetype the roster is weighted toward this wave, if any.
    pub preferred: Option<EnemyType>,
    /// Pull the preferred archetype into the eligible pool even if the wave
    /// schedule would not have unlocked its tier yet.
    pub early_unlock: bool,
}

impl WaveManager {
    pub fn new(tuning: WaveTuning) -> Self {
        Self {
            current_wave: 0,
            enemies: Vec::new(),
            spawn_queue: Vec::new(),
            spawn_timer: 0.0,
            wave_active: false,
            spawn_interval: tuning.spawn_interval,
            commander_aura_radius: tuning.commander_aura_radius,
            commander_aura_speed_mult: tuning.commander_aura_speed_mult,
            enemy_tuning: tuning.enemy_tuning,
            wave_budget_base: tuning.wave_budget_base,
            wave_budget_per_wave: tuning.wave_budget_per_wave,
            wave_commander_every: tuning.wave_commander_every,
            threat_budget_divisor: tuning.threat_budget_divisor,
            threat_health_mult_per_awareness: tuning.threat_health_mult_per_awareness,
            tier_2_awareness: tuning.tier_2_awareness,
            tier_3_awareness: tuning.tier_3_awareness,
            enemy_abilities_enabled: true,
        }
    }

    pub fn set_enemy_abilities_enabled(&mut self, enabled: bool) {
        self.enemy_abilities_enabled = enabled;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generate_wave(
        &mut self,
        wave_number: u32,
        enemy_defs: &[EnemyDef],
        base_health_scale: f32,
        threat_awareness: f32,
        tier_floor: u32,
        budget_multiplier: f32,
        force_commander: bool,
        spawn_points: &[(String, Vec2)],
        adaptation: &WaveAdaptation,
    ) {
        self.current_wave = wave_number;
        self.wave_active = true;
        self.spawn_queue.clear();
        self.spawn_timer = 0.0;

        let tuning = self.tuning();

        self.spawn_queue = build_spawn_queue(
            wave_number,
            enemy_defs,
            base_health_scale,
            threat_awareness,
            tier_floor,
            budget_multiplier,
            force_commander,
            spawn_points,
            &tuning,
            adaptation,
        );
    }

    pub fn tick(&mut self, dt: f32, paths: &HashMap<String, Vec<Vec2>>) -> WaveEvent {
        if !self.wave_active {
            return WaveEvent::None;
        }

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 && !self.spawn_queue.is_empty() {
            let entry = self.spawn_queue.remove(0);
            self.enemies.push(Enemy::new(
                entry.enemy_type,
                entry.spawn_point,
                entry.health,
                entry.speed,
                entry.scrap_reward,
                self.enemy_tuning.clone(),
                entry.path_id,
                entry.damage_multipliers,
            ));
            self.spawn_timer = self.spawn_interval;
        }

        let mut reached_end = None;
        let mut scout_report = None;
        let mut saboteur_strike = None;
        let mut commander_pulse = None;
        let commander_positions: Vec<Vec2> = self
            .enemies
            .iter()
            .filter(|e| e.is_alive && e.enemy_type == EnemyType::Commander)
            .map(|e| e.position)
            .collect();

        for enemy in self.enemies.iter_mut() {
            if !enemy.is_alive {
                continue;
            }
            enemy.tick_timers(dt);

            if self.enemy_abilities_enabled && enemy.is_alive && enemy.use_ability() {
                match enemy.enemy_type {
                    EnemyType::Scout => scout_report = Some(enemy.position),
                    EnemyType::Saboteur => saboteur_strike = Some(enemy.position),
                    EnemyType::Commander => {
                        let pulse_radius = self.enemy_tuning.commander_shield_radius;
                        commander_pulse = Some((enemy.position, pulse_radius));
                    }
                    EnemyType::Drone | EnemyType::HeavyUnit => {}
                }
            }

            let mut speed_mult = 1.0;
            if enemy.enemy_type != EnemyType::Commander {
                for pos in &commander_positions {
                    if (enemy.position - *pos).length() <= self.commander_aura_radius {
                        speed_mult = self.commander_aura_speed_mult;
                        break;
                    }
                }
            }

            // Look up this enemy's path
            let reached = if let Some(path) = paths.get(&enemy.path_id) {
                enemy.move_along_path(path, dt, speed_mult)
            } else {
                // Fallback: use any available path
                if let Some((_id, path)) = paths.iter().next() {
                    enemy.move_along_path(path, dt, speed_mult)
                } else {
                    false
                }
            };
            if reached {
                enemy.is_alive = false;
                reached_end = Some(enemy.enemy_type.clone());
            }
        }

        if let Some((pulse_center, pulse_radius)) = commander_pulse {
            for ally in &mut self.enemies {
                if ally.is_alive && (ally.position - pulse_center).length() <= pulse_radius {
                    ally.shield_timer = ally
                        .shield_timer
                        .max(self.enemy_tuning.commander_shield_duration);
                }
            }
        }

        if let Some(enemy_type) = reached_end {
            return WaveEvent::EnemyReachedEnd { enemy_type };
        }

        if let Some((position, radius)) = commander_pulse {
            return WaveEvent::CommanderPulse { position, radius };
        }
        if let Some(position) = saboteur_strike {
            return WaveEvent::SaboteurStrike { position };
        }
        if let Some(position) = scout_report {
            return WaveEvent::ScoutReport { position };
        }

        if self.spawn_queue.is_empty() && self.enemies.iter().all(|e| !e.is_alive) {
            self.wave_active = false;
            return WaveEvent::WaveComplete;
        }

        WaveEvent::None
    }

    pub fn alive_count(&self) -> usize {
        self.enemies.iter().filter(|e| e.is_alive).count()
    }

    /// Reconstruct the `WaveTuning` this manager was built from, for callers
    /// (e.g. wave preview) that need to pass it to the free spawn-queue
    /// functions without duplicating the manager's own live state.
    pub fn tuning(&self) -> WaveTuning {
        WaveTuning {
            spawn_interval: self.spawn_interval,
            commander_aura_radius: self.commander_aura_radius,
            commander_aura_speed_mult: self.commander_aura_speed_mult,
            enemy_tuning: self.enemy_tuning.clone(),
            wave_budget_base: self.wave_budget_base,
            wave_budget_per_wave: self.wave_budget_per_wave,
            wave_commander_every: self.wave_commander_every,
            threat_budget_divisor: self.threat_budget_divisor,
            threat_health_mult_per_awareness: self.threat_health_mult_per_awareness,
            tier_2_awareness: self.tier_2_awareness,
            tier_3_awareness: self.tier_3_awareness,
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub fn preview_wave(
    wave_number: u32,
    enemy_defs: &[EnemyDef],
    base_health_scale: f32,
    threat_awareness: f32,
    tier_floor: u32,
    budget_multiplier: f32,
    force_commander: bool,
    tuning: &WaveTuning,
    adaptation: &WaveAdaptation,
) -> Vec<EnemyType> {
    let dummy_spawn = vec![("preview".to_string(), Vec2::new(0.0, 0.0))];
    let entries = preview_wave_entries(
        wave_number,
        enemy_defs,
        base_health_scale,
        threat_awareness,
        tier_floor,
        budget_multiplier,
        force_commander,
        tuning,
        &dummy_spawn,
        adaptation,
    );
    entries.into_iter().map(|e| e.enemy_type).collect()
}

#[allow(clippy::too_many_arguments)]
pub fn preview_wave_entries(
    wave_number: u32,
    enemy_defs: &[EnemyDef],
    base_health_scale: f32,
    threat_awareness: f32,
    tier_floor: u32,
    budget_multiplier: f32,
    force_commander: bool,
    tuning: &WaveTuning,
    spawn_points: &[(String, Vec2)],
    adaptation: &WaveAdaptation,
) -> Vec<PreviewSpawnEntry> {
    let queue = build_spawn_queue(
        wave_number,
        enemy_defs,
        base_health_scale,
        threat_awareness,
        tier_floor,
        budget_multiplier,
        force_commander,
        spawn_points,
        tuning,
        adaptation,
    );
    queue
        .into_iter()
        .map(|e| PreviewSpawnEntry {
            enemy_type: e.enemy_type,
            path_id: e.path_id,
        })
        .collect()
}

fn push_spawn(
    queue: &mut Vec<SpawnEntry>,
    def: &EnemyDef,
    health_scale: f32,
    spawn_point: Vec2,
    path_id: String,
) {
    queue.push(SpawnEntry {
        enemy_type: def.enemy_type.clone(),
        health: def.base_health * health_scale,
        speed: def.speed,
        scrap_reward: def.scrap_reward,
        spawn_point,
        path_id,
        damage_multipliers: def.damage_multipliers.clone(),
    });
}

#[allow(clippy::too_many_arguments)]
fn build_spawn_queue(
    wave_number: u32,
    enemy_defs: &[EnemyDef],
    base_health_scale: f32,
    threat_awareness: f32,
    tier_floor: u32,
    budget_multiplier: f32,
    force_commander: bool,
    spawn_points: &[(String, Vec2)],
    tuning: &WaveTuning,
    adaptation: &WaveAdaptation,
) -> Vec<SpawnEntry> {
    if spawn_points.is_empty() {
        return Vec::new();
    }

    let threat_health_bonus = 1.0 + threat_awareness * tuning.threat_health_mult_per_awareness;
    let scale = base_health_scale.powi(wave_number as i32) * threat_health_bonus;

    let threat_budget_bonus = (threat_awareness / tuning.threat_budget_divisor).floor() as i32;
    let base_budget = tuning.wave_budget_base as i32
        + wave_number as i32 * tuning.wave_budget_per_wave as i32
        + threat_budget_bonus;
    let mut budget = (base_budget as f32 * budget_multiplier).round().max(1.0) as i32;

    let mut max_tier = match wave_number {
        0..=2 => 1,
        3..=4 => 2,
        _ => 3,
    };

    if threat_awareness >= tuning.tier_3_awareness {
        max_tier = max_tier.max(3);
    } else if threat_awareness >= tuning.tier_2_awareness {
        max_tier = max_tier.max(2);
    }

    max_tier = max_tier.max(tier_floor);

    // Qualitative adaptation: a dominant signature can pull its favoured
    // archetype into the roster ahead of the normal tier schedule (e.g. high
    // corruption summons saboteurs early).
    if adaptation.early_unlock {
        if let Some(pref) = &adaptation.preferred {
            if let Some(def) = enemy_defs.iter().find(|d| &d.enemy_type == pref) {
                max_tier = max_tier.max(def.tier);
            }
        }
    }

    let mut eligible: Vec<&EnemyDef> = enemy_defs.iter().filter(|d| d.tier <= max_tier).collect();
    eligible.sort_by_key(|enemy| std::cmp::Reverse(enemy.threat_value));

    let mut queue = Vec::new();
    let mut path_robin = 0usize;

    if (wave_number > 0
        && tuning.wave_commander_every != 0
        && wave_number.is_multiple_of(tuning.wave_commander_every))
        || force_commander
    {
        if let Some(boss) = eligible
            .iter()
            .find(|d| d.enemy_type == EnemyType::Commander)
        {
            let (ref pid, sp) = spawn_points[path_robin % spawn_points.len()];
            push_spawn(&mut queue, boss, scale, sp, pid.clone());
            budget -= boss.threat_value as i32;
            path_robin += 1;
        }
    }

    // The adapted archetype, if it made it into the eligible pool. Budget-based
    // filling means biasing toward it shifts *which* enemies arrive, not how
    // much total threat — the wave's character changes, not its weight.
    let preferred_def: Option<&EnemyDef> = adaptation
        .preferred
        .as_ref()
        .and_then(|pref| eligible.iter().copied().find(|d| &d.enemy_type == pref));

    let mut safety = 200;
    while budget > 0 && safety > 0 {
        safety -= 1;
        // Bias every other fill toward the adapted archetype when it fits;
        // otherwise keep the original high/low variety cadence.
        let biased =
            preferred_def.filter(|d| queue.len() % 2 == 1 && (d.threat_value as i32) <= budget);
        let pick = biased.or_else(|| {
            if queue.len() % 3 == 0 {
                eligible
                    .iter()
                    .copied()
                    .rev()
                    .find(|d| (d.threat_value as i32) <= budget)
            } else {
                eligible
                    .iter()
                    .copied()
                    .find(|d| (d.threat_value as i32) <= budget)
            }
        });

        match pick {
            Some(def) => {
                budget -= def.threat_value as i32;
                let (ref pid, sp) = spawn_points[path_robin % spawn_points.len()];
                push_spawn(&mut queue, def, scale, sp, pid.clone());
                path_robin += 1;
            }
            None => break,
        }
    }

    queue
}

#[cfg(test)]
#[path = "wave_impl/tests.rs"]
mod tests;
