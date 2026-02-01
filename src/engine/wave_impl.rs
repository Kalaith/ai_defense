//! Wave spawning and progression (refactored).

use crate::data::EnemyDef;
use crate::engine::enemy::{Enemy, EnemyType, EnemyTuning};
use macroquad::prelude::Vec2;
use std::collections::HashMap;

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

pub struct SpawnEntry {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub speed: f32,
    pub scrap_reward: f32,
    pub spawn_point: Vec2,
    pub path_id: String,
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
        spawn_points: &[(String, Vec2)],
    ) {
        self.current_wave = wave_number;
        self.wave_active = true;
        self.spawn_queue.clear();
        self.spawn_timer = 0.0;

        self.spawn_queue = build_spawn_queue(
            wave_number,
            enemy_defs,
            base_health_scale,
            threat_awareness,
            tier_floor,
            budget_multiplier,
            force_commander,
            spawn_points,
            self.wave_budget_base,
            self.wave_budget_per_wave,
            self.wave_commander_every,
            self.threat_budget_divisor,
            self.threat_health_mult_per_awareness,
        );
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
        spawn_points: &[(String, Vec2)],
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
            spawn_points,
            self.wave_budget_base,
            self.wave_budget_per_wave,
            self.wave_commander_every,
            self.threat_budget_divisor,
            self.threat_health_mult_per_awareness,
        );
        self.spawn_queue.extend(queue);
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
            ));
            self.spawn_timer = self.spawn_interval;
        }

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
                reached_end_reward = Some((enemy.enemy_type.clone(), enemy.scrap_reward));
            }
        }

        if let Some((enemy_type, reward)) = reached_end_reward {
            return WaveEvent::EnemyReachedEnd { enemy_type, scrap_reward: reward };
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
}

pub fn preview_wave(
    wave_number: u32,
    enemy_defs: &[EnemyDef],
    base_health_scale: f32,
    threat_awareness: f32,
    tier_floor: u32,
    budget_multiplier: f32,
    force_commander: bool,
    wave_budget_base: u32,
    wave_budget_per_wave: u32,
    wave_commander_every: u32,
    threat_budget_divisor: f32,
    threat_health_mult_per_awareness: f32,
) -> Vec<EnemyType> {
    let dummy_spawn = vec![("preview".to_string(), Vec2::new(0.0, 0.0))];
    let queue = build_spawn_queue(
        wave_number,
        enemy_defs,
        base_health_scale,
        threat_awareness,
        tier_floor,
        budget_multiplier,
        force_commander,
        &dummy_spawn,
        wave_budget_base,
        wave_budget_per_wave,
        wave_commander_every,
        threat_budget_divisor,
        threat_health_mult_per_awareness,
    );
    queue.into_iter().map(|e| e.enemy_type).collect()
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

fn push_spawn(queue: &mut Vec<SpawnEntry>, def: &EnemyDef, health_scale: f32, spawn_point: Vec2, path_id: String) {
    queue.push(SpawnEntry {
        enemy_type: parse_enemy_type(&def.enemy_type),
        health: def.base_health * health_scale,
        speed: def.speed,
        scrap_reward: def.scrap_reward,
        spawn_point,
        path_id,
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
    spawn_points: &[(String, Vec2)],
    wave_budget_base: u32,
    wave_budget_per_wave: u32,
    wave_commander_every: u32,
    threat_budget_divisor: f32,
    threat_health_mult_per_awareness: f32,
) -> Vec<SpawnEntry> {
    if spawn_points.is_empty() {
        return Vec::new();
    }

    let threat_health_bonus = 1.0 + threat_awareness * threat_health_mult_per_awareness;
    let scale = base_health_scale.powi(wave_number as i32) * threat_health_bonus;

    let threat_budget_bonus = (threat_awareness / threat_budget_divisor).floor() as i32;
    let base_budget = wave_budget_base as i32 + wave_number as i32 * wave_budget_per_wave as i32 + threat_budget_bonus;
    let mut budget = (base_budget as f32 * budget_multiplier).round().max(1.0) as i32;

    let mut max_tier = match wave_number {
        0..=2 => 1,
        3..=4 => 2,
        _ => 3,
    };

    if threat_awareness >= 60.0 {
        max_tier = max_tier.max(3);
    } else if threat_awareness >= 25.0 {
        max_tier = max_tier.max(2);
    }

    max_tier = max_tier.max(tier_floor);

    let mut eligible: Vec<&EnemyDef> = enemy_defs.iter().filter(|d| d.tier <= max_tier).collect();
    eligible.sort_by(|a, b| b.threat_value.cmp(&a.threat_value));

    let mut queue = Vec::new();
    let mut path_robin = 0usize;

    if (wave_number > 0 && wave_number % wave_commander_every == 0) || force_commander {
        if let Some(boss) = eligible.iter().find(|d| d.enemy_type == "Commander") {
            let (ref pid, sp) = spawn_points[path_robin % spawn_points.len()];
            push_spawn(&mut queue, boss, scale, sp, pid.clone());
            budget -= boss.threat_value as i32;
            path_robin += 1;
        }
    }

    let mut safety = 200;
    while budget > 0 && safety > 0 {
        safety -= 1;
        let pick = if queue.len() % 3 == 0 {
            eligible.iter().rev().find(|d| (d.threat_value as i32) <= budget)
        } else {
            eligible.iter().find(|d| (d.threat_value as i32) <= budget)
        };

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
