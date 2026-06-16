#[path = "wave_impl.rs"]
mod wave_impl;
#[allow(unused_imports)]
pub use wave_impl::*;

/*

#[cfg(FALSE)]
mod legacy {
//! Wave spawning and progression.

use crate::data::EnemyDef;
use crate::engine::enemy::{Enemy, EnemyType, EnemyTuning};
use macroquad::prelude::Vec2;

pub enum WaveEvent {
    None,
    EnemyReachedEnd { enemy_type: EnemyType, scrap_reward: f32 },
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
}

pub fn preview_wave(
    wave_number: u32,
    enemy_defs: &[EnemyDef],
    base_health_scale: f32,
    threat_awareness: f32,
    tier_floor: u32,
    budget_multiplier: f32,
    force_commander: bool,
) -> Vec<EnemyType> {
    pub fn generate_wave(
        &mut self,
        wave_number: u32,
        enemy_defs: &[EnemyDef],
        base_health_scale: f32,
        threat_awareness: f32,
        tier_floor: u32,
        budget_multiplier: f32,
        force_commander: bool,
        spawn_point: Vec2,
    ) {
        self.current_wave = wave_number;
        self.wave_active = true;
        self.spawn_queue.clear();
        self.spawn_timer = 0.0;

        let queue = build_spawn_queue(
            wave_number,
            enemy_defs,
            base_health_scale,
            threat_awareness,
            tier_floor,
            budget_multiplier,
            force_commander,
            spawn_point,
            self.wave_budget_base,
            self.wave_budget_per_wave,
            self.wave_commander_every,
            self.threat_budget_divisor,
            self.threat_health_mult_per_awareness,
        );
        self.spawn_queue = queue;
    }

    let queue = build_spawn_queue(
        wave_number,
        enemy_defs,
        base_health_scale,
        threat_awareness,
        tier_floor,
        budget_multiplier,
        force_commander,
        Vec2::new(0.0, 0.0),
    );
    queue.into_iter().map(|e| e.enemy_type).collect()
}

pub struct SpawnEntry {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub speed: f32,
    pub scrap_reward: f32,
    pub spawn_point: Vec2,
}

impl WaveManager {
            self.wave_budget_base,
            self.wave_budget_per_wave,
            self.wave_commander_every,
            self.threat_budget_divisor,
            self.threat_health_mult_per_awareness,
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
        }
    }

    pub fn generate_wave(
        &mut self,
        wave_number: u32,
        enemy_defs: &[EnemyDef],
        base_health_scale: f32,
        threat_awareness: f32,
        tier_floor: u32,
        budget_multiplier: f32,
        force_commander: bool,
        spawn_point: Vec2,
    ) {
        self.current_wave = wave_number;
        self.wave_active = true;
        self.spawn_queue.clear();
        self.spawn_timer = 0.0;

        let queue = build_spawn_queue(
            wave_number,
            enemy_defs,
            base_health_scale,
            threat_awareness,
            tier_floor,
            budget_multiplier,
            force_commander,
            spawn_point,
        );
        self.spawn_queue = queue;
    }

    pub fn append_wave(
        &mut self,
        wave_number: u32,
        enemy_defs: &[EnemyDef],
        base_health_scale: f32,
        threat_awareness: f32,
        tier_floor: u32,
        budget_multiplier: f32,
        force_commander: bool,
        spawn_point: Vec2,
    ) {
        self.current_wave = wave_number;
        self.wave_active = true;
        let queue = build_spawn_queue(
            wave_number,
            enemy_defs,
            base_health_scale,
            threat_awareness,
            tier_floor,
            budget_multiplier,
            force_commander,
            spawn_point,
        );
        self.spawn_queue.extend(queue);
    }

    pub fn tick(&mut self, dt: f32, path: &[Vec2]) -> WaveEvent {
        if !self.wave_active {
            return WaveEvent::None;
        }

        // Spawn from queue
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
            ));
            self.spawn_timer = self.spawn_interval;
        }

        // Move enemies along path
        let mut reached_end_reward = None;
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

    wave_budget_base: u32,
    wave_budget_per_wave: u32,
    wave_commander_every: u32,
    threat_budget_divisor: f32,
    threat_health_mult_per_awareness: f32,
            let mut speed_mult = 1.0;
            if enemy.enemy_type != EnemyType::Commander {
                for pos in &commander_positions {
                    if (enemy.position - *pos).length() <= self.commander_aura_radius {
                        speed_mult = self.commander_aura_speed_mult;
                        break;
                    }
                }
            }

            let reached = enemy.move_along_path(path, dt, speed_mult);
            if reached {
                enemy.is_alive = false;
                reached_end_reward = Some((enemy.enemy_type.clone(), enemy.scrap_reward));
            }
        }

        if let Some((enemy_type, reward)) = reached_end_reward {
            return WaveEvent::EnemyReachedEnd { enemy_type, scrap_reward: reward };
        }

        // Check wave complete
        if self.spawn_queue.is_empty() && self.enemies.iter().all(|e| !e.is_alive) {
            self.wave_active = false;
            return WaveEvent::WaveComplete;
        }

        WaveEvent::None
    }

    pub fn alive_count(&self) -> usize {
        self.enemies.iter().filter(|e| e.is_alive).count()
    }
}

fn parse_enemy_type(s: &str) -> EnemyType {
    match s {
        "Scout" => EnemyType::Scout,
        "Drone" => EnemyType::Drone,
        "HeavyUnit" => EnemyType::HeavyUnit,
        "Saboteur" => EnemyType::Saboteur,
        "Commander" => EnemyType::Commander,
        _ => EnemyType::Drone,
    }
}

fn push_spawn(queue: &mut Vec<SpawnEntry>, def: &EnemyDef, health_scale: f32, spawn_point: Vec2) {
    queue.push(SpawnEntry {
        enemy_type: parse_enemy_type(&def.enemy_type),
        health: def.base_health * health_scale,
        speed: def.speed,
        scrap_reward: def.scrap_reward,
        spawn_point,
    });
}

fn build_spawn_queue(
    wave_number: u32,
    enemy_defs: &[EnemyDef],
    base_health_scale: f32,
    threat_awareness: f32,
    tier_floor: u32,
    budget_multiplier: f32,
    force_commander: bool,
    spawn_point: Vec2,
) -> Vec<SpawnEntry> {
    let threat_health_bonus = 1.0 + threat_awareness * 0.005;
    let scale = base_health_scale.powi(wave_number as i32) * threat_health_bonus;

    // Budget grows with wave number
    let threat_budget_bonus = (threat_awareness / 10.0).floor() as i32;
    let base_budget = 8 + wave_number as i32 * 4 + threat_budget_bonus;
    let mut budget = (base_budget as f32 * budget_multiplier).round().max(1.0) as i32;

    // Max tier unlocked by wave: wave 1-2 → tier 1, wave 3-4 → tier 2, wave 5+ → tier 3
    let mut max_tier = match wave_number {
        0..=2 => 1,
        3..=4 => 2,
        _ => 3,
    };

    // Threat reaction tier can unlock higher tiers early
    if threat_awareness >= 60.0 {
        max_tier = max_tier.max(3);
    } else if threat_awareness >= 25.0 {
        max_tier = max_tier.max(2);
    }

    max_tier = max_tier.max(tier_floor);

    // Eligible defs sorted by threat_value descending so we try expensive units first
    let mut eligible: Vec<&EnemyDef> = enemy_defs
        .iter()
        .filter(|d| d.tier <= max_tier)
        .collect();
    eligible.sort_by_key(|enemy| std::cmp::Reverse(enemy.threat_value));

    let mut queue = Vec::new();

    // Every 5th wave, guarantee a commander if tier allows or forced
    if (wave_number > 0 && wave_number % 5 == 0) || force_commander {
        if let Some(boss) = eligible.iter().find(|d| d.enemy_type == "Commander") {
            push_spawn(&mut queue, boss, scale, spawn_point);
            budget -= boss.threat_value as i32;
        }
    }

    // Fill remaining budget, cycling through eligible defs
    let mut safety = 200;
    while budget > 0 && safety > 0 {
        safety -= 1;
        // Pick the most expensive unit that fits, with some variety:
        // alternate between cheapest and most expensive that fits
        let pick = if queue.len() % 3 == 0 {
            // Occasionally pick cheapest for variety
            eligible.iter().rev().find(|d| (d.threat_value as i32) <= budget)
        } else {
            eligible.iter().find(|d| (d.threat_value as i32) <= budget)
        };

        match pick {
            Some(def) => {
                budget -= def.threat_value as i32;
                push_spawn(&mut queue, def, scale, spawn_point);
            }
            None => break,
        }
    }

    queue
}
}
*/
