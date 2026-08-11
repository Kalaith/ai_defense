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
        &test_tuning(&data),
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

    let tuning = test_tuning(&data);
    let entries = |adaptation: &WaveAdaptation| {
        preview_wave_entries(
            1, // wave 1 normally only unlocks tier-1 (Scout/Drone)
            &data.enemy_defs,
            data.constants.waves.health_scale_per_wave,
            0.0,
            1,
            data.constants.waves.budget_multiplier,
            false,
            &tuning,
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
        &test_tuning(&data),
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
