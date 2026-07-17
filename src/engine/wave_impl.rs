//! Wave spawning and progression (refactored).

use crate::data::EnemyDef;
use crate::engine::enemy::{Enemy, EnemyTuning, EnemyType};
use macroquad::prelude::Vec2;
use std::collections::HashMap;

pub enum WaveEvent {
    None,
    EnemyReachedEnd { enemy_type: EnemyType },
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
        }
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
            self.tier_2_awareness,
            self.tier_3_awareness,
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
            ));
            self.spawn_timer = self.spawn_interval;
        }

        let mut reached_end = None;
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
                reached_end = Some(enemy.enemy_type.clone());
            }
        }

        if let Some(enemy_type) = reached_end {
            return WaveEvent::EnemyReachedEnd { enemy_type };
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
    wave_budget_base: u32,
    wave_budget_per_wave: u32,
    wave_commander_every: u32,
    threat_budget_divisor: f32,
    threat_health_mult_per_awareness: f32,
    tier_2_awareness: f32,
    tier_3_awareness: f32,
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
        wave_budget_base,
        wave_budget_per_wave,
        wave_commander_every,
        threat_budget_divisor,
        threat_health_mult_per_awareness,
        tier_2_awareness,
        tier_3_awareness,
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
    wave_budget_base: u32,
    wave_budget_per_wave: u32,
    wave_commander_every: u32,
    threat_budget_divisor: f32,
    threat_health_mult_per_awareness: f32,
    tier_2_awareness: f32,
    tier_3_awareness: f32,
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
        wave_budget_base,
        wave_budget_per_wave,
        wave_commander_every,
        threat_budget_divisor,
        threat_health_mult_per_awareness,
        tier_2_awareness,
        tier_3_awareness,
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
    wave_budget_base: u32,
    wave_budget_per_wave: u32,
    wave_commander_every: u32,
    threat_budget_divisor: f32,
    threat_health_mult_per_awareness: f32,
    tier_2_awareness: f32,
    tier_3_awareness: f32,
    adaptation: &WaveAdaptation,
) -> Vec<SpawnEntry> {
    if spawn_points.is_empty() {
        return Vec::new();
    }

    let threat_health_bonus = 1.0 + threat_awareness * threat_health_mult_per_awareness;
    let scale = base_health_scale.powi(wave_number as i32) * threat_health_bonus;

    let threat_budget_bonus = (threat_awareness / threat_budget_divisor).floor() as i32;
    let base_budget = wave_budget_base as i32
        + wave_number as i32 * wave_budget_per_wave as i32
        + threat_budget_bonus;
    let mut budget = (base_budget as f32 * budget_multiplier).round().max(1.0) as i32;

    let mut max_tier = match wave_number {
        0..=2 => 1,
        3..=4 => 2,
        _ => 3,
    };

    if threat_awareness >= tier_3_awareness {
        max_tier = max_tier.max(3);
    } else if threat_awareness >= tier_2_awareness {
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
        && wave_commander_every != 0
        && wave_number.is_multiple_of(wave_commander_every))
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
mod tests {
    use super::*;
    use crate::data::GameData;
    use macroquad::prelude::vec2;

    fn test_tuning(data: &GameData) -> WaveTuning {
        WaveTuning {
            spawn_interval: data.constants.waves.spawn_interval,
            commander_aura_radius: data.constants.enemy.commander_aura_radius,
            commander_aura_speed_mult: data.constants.enemy.commander_aura_speed_mult,
            enemy_tuning: EnemyTuning {
                scout_dodge_chance: data.constants.enemy.scout_dodge_chance,
                scout_dodge_duration: data.constants.enemy.scout_dodge_duration,
                hit_flash_duration: data.constants.enemy.hit_flash_duration,
                saboteur_skip_chance: data.constants.enemy.saboteur_skip_chance,
                slow_multiplier: data.constants.enemy.slow_multiplier,
            },
            wave_budget_base: data.constants.waves.budget_base,
            wave_budget_per_wave: data.constants.waves.budget_per_wave,
            wave_commander_every: data.constants.waves.commander_every,
            threat_budget_divisor: data.constants.threat.budget_divisor,
            threat_health_mult_per_awareness: data.constants.threat.health_mult_per_awareness,
            tier_2_awareness: data.constants.threat.tier_2_awareness,
            tier_3_awareness: data.constants.threat.tier_3_awareness,
        }
    }

    #[test]
    fn preview_wave_uses_loaded_enemy_data() {
        let data = GameData::load();

        let preview = preview_wave(
            1,
            &data.enemy_defs,
            data.constants.waves.health_scale_per_wave,
            0.0,
            1,
            data.constants.waves.budget_multiplier,
            false,
            data.constants.waves.budget_base,
            data.constants.waves.budget_per_wave,
            data.constants.waves.commander_every,
            data.constants.threat.budget_divisor,
            data.constants.threat.health_mult_per_awareness,
            data.constants.threat.tier_2_awareness,
            data.constants.threat.tier_3_awareness,
            &WaveAdaptation::default(),
        );

        assert!(!preview.is_empty(), "wave preview should spawn enemies");
        assert!(
            preview.iter().all(|kind| matches!(
                kind,
                EnemyType::Scout
                    | EnemyType::Drone
                    | EnemyType::HeavyUnit
                    | EnemyType::Saboteur
                    | EnemyType::Commander
            )),
            "preview contained an unknown enemy type"
        );
    }

    #[test]
    fn generated_wave_queues_enemies_across_active_spawn_points() {
        let data = GameData::load();
        let mut manager = WaveManager::new(test_tuning(&data));
        let spawn_points = vec![
            ("west".to_string(), vec2(10.0, 20.0)),
            ("north".to_string(), vec2(100.0, 120.0)),
        ];

        manager.generate_wave(
            3,
            &data.enemy_defs,
            data.constants.waves.health_scale_per_wave,
            15.0,
            1,
            data.constants.waves.budget_multiplier,
            false,
            &spawn_points,
            &WaveAdaptation::default(),
        );

        assert!(manager.wave_active);
        assert!(!manager.spawn_queue.is_empty(), "expected queued enemies");
        assert!(manager
            .spawn_queue
            .iter()
            .all(|entry| entry.health > 0.0 && entry.speed > 0.0));
        assert!(manager
            .spawn_queue
            .iter()
            .any(|entry| entry.path_id == "west"));
        assert!(manager
            .spawn_queue
            .iter()
            .any(|entry| entry.path_id == "north"));
    }

    #[test]
    fn adaptation_unlocks_and_biases_preferred_archetype() {
        let data = GameData::load();
        let spawn_points = vec![("west".to_string(), vec2(0.0, 0.0))];

        let entries = |adaptation: &WaveAdaptation| {
            preview_wave_entries(
                1, // wave 1 normally only unlocks tier-1 (Scout/Drone)
                &data.enemy_defs,
                data.constants.waves.health_scale_per_wave,
                0.0,
                1,
                data.constants.waves.budget_multiplier,
                false,
                data.constants.waves.budget_base,
                data.constants.waves.budget_per_wave,
                data.constants.waves.commander_every,
                data.constants.threat.budget_divisor,
                data.constants.threat.health_mult_per_awareness,
                data.constants.threat.tier_2_awareness,
                data.constants.threat.tier_3_awareness,
                &spawn_points,
                adaptation,
            )
        };

        // Baseline: a schedule-standard wave 1 has no tier-2 saboteurs.
        let baseline = entries(&WaveAdaptation::default());
        assert!(
            !baseline.iter().any(|e| e.enemy_type == EnemyType::Saboteur),
            "wave 1 should not contain saboteurs without adaptation"
        );

        // High corruption pulls saboteurs in ahead of schedule and weights the
        // roster toward them.
        let adapted = entries(&WaveAdaptation {
            preferred: Some(EnemyType::Saboteur),
            early_unlock: true,
        });
        assert!(
            adapted.iter().any(|e| e.enemy_type == EnemyType::Saboteur),
            "early-unlock adaptation should summon the preferred archetype"
        );
    }

    #[test]
    fn preview_wave_entries_include_enemy_type_and_path_id() {
        let data = GameData::load();
        let spawn_points = vec![
            ("west".to_string(), vec2(10.0, 20.0)),
            ("north".to_string(), vec2(100.0, 120.0)),
        ];

        let preview = preview_wave_entries(
            3,
            &data.enemy_defs,
            data.constants.waves.health_scale_per_wave,
            15.0,
            1,
            data.constants.waves.budget_multiplier,
            false,
            data.constants.waves.budget_base,
            data.constants.waves.budget_per_wave,
            data.constants.waves.commander_every,
            data.constants.threat.budget_divisor,
            data.constants.threat.health_mult_per_awareness,
            data.constants.threat.tier_2_awareness,
            data.constants.threat.tier_3_awareness,
            &spawn_points,
            &WaveAdaptation::default(),
        );

        assert!(!preview.is_empty(), "expected preview entries");
        assert!(
            preview
                .iter()
                .all(|entry| entry.path_id == "west" || entry.path_id == "north"),
            "preview should preserve path ids"
        );
        assert!(
            preview.iter().any(|entry| matches!(
                entry.enemy_type,
                EnemyType::Scout
                    | EnemyType::Drone
                    | EnemyType::HeavyUnit
                    | EnemyType::Saboteur
                    | EnemyType::Commander
            )),
            "preview should expose enemy types"
        );
    }
}
