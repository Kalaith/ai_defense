//! Enemy types, pathing, and behavior.

use macroquad::prelude::Vec2;
use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EnemyType {
    Scout,
    Drone,
    HeavyUnit,
    Saboteur,
    Commander,
}

#[derive(Clone, Debug)]
pub struct EnemyTuning {
    pub scout_dodge_chance: f32,
    pub scout_dodge_duration: f32,
    pub hit_flash_duration: f32,
    pub saboteur_skip_chance: f32,
    pub slow_multiplier: f32,
}

pub struct Enemy {
    pub enemy_type: EnemyType,
    pub position: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub path_index: usize,
    pub is_alive: bool,
    pub scrap_reward: f32,
    pub slowed_timer: f32,
    pub hit_flash_timer: f32,
    pub dodge_timer: f32,
    pub tuning: EnemyTuning,
    pub path_id: String,
}

impl Enemy {
    pub fn new(
        enemy_type: EnemyType,
        position: Vec2,
        health: f32,
        speed: f32,
        scrap_reward: f32,
        tuning: EnemyTuning,
        path_id: String,
    ) -> Self {
        Self {
            enemy_type,
            position,
            health,
            max_health: health,
            speed,
            path_index: 0,
            is_alive: true,
            scrap_reward,
            slowed_timer: 0.0,
            hit_flash_timer: 0.0,
            dodge_timer: 0.0,
            tuning,
            path_id,
        }
    }

    pub fn take_damage(&mut self, amount: f32) {
        if self.enemy_type == EnemyType::Scout {
            if gen_range(0.0, 1.0) < self.tuning.scout_dodge_chance {
                self.dodge_timer = self.tuning.scout_dodge_duration;
                return;
            }
        }

        self.health -= amount;
        self.hit_flash_timer = self.tuning.hit_flash_duration;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.is_alive = false;
        }
    }

    pub fn tick_timers(&mut self, dt: f32) {
        if self.hit_flash_timer > 0.0 {
            self.hit_flash_timer = (self.hit_flash_timer - dt).max(0.0);
        }
        if self.dodge_timer > 0.0 {
            self.dodge_timer = (self.dodge_timer - dt).max(0.0);
        }
    }

    pub fn move_along_path(&mut self, path: &[Vec2], dt: f32, speed_multiplier: f32) -> bool {
        if self.path_index >= path.len() {
            return true; // Reached end
        }

        let mut speed = self.speed * speed_multiplier;
        if self.slowed_timer > 0.0 {
            speed *= self.tuning.slow_multiplier;
            self.slowed_timer = (self.slowed_timer - dt).max(0.0);
        }

        let target = path[self.path_index];
        let direction = target - self.position;
        let distance = direction.length();

        if distance < 2.0 {
            self.path_index += 1;
            if self.enemy_type == EnemyType::Saboteur && self.path_index < path.len().saturating_sub(1) {
                if gen_range(0.0, 1.0) < self.tuning.saboteur_skip_chance {
                    self.path_index += 1;
                }
            }
        } else {
            self.position += direction.normalize() * speed * dt;
        }

        false
    }
}
