use super::*;
use crate::data::GameData;
use macroquad::prelude::vec2;
use std::collections::{HashMap, HashSet};

fn test_tuning(data: &GameData) -> WaveTuning {
    WaveTuning {
        spawn_interval: data.constants.waves.spawn_interval,
        commander_aura_radius: data.constants.enemy.commander_aura_radius,
        commander_aura_speed_mult: data.constants.enemy.commander_aura_speed_mult,
        enemy_tuning: EnemyTuning {
            scout_dodge_chance: data.constants.enemy.scout_dodge_chance,
            scout_dodge_duration: data.constants.enemy.scout_dodge_duration,
            scout_report_interval: data.constants.enemy.scout_report_interval,
            hit_flash_duration: data.constants.enemy.hit_flash_duration,
            saboteur_skip_chance: data.constants.enemy.saboteur_skip_chance,
            saboteur_strike_interval: data.constants.enemy.saboteur_strike_interval,
            slow_multiplier: data.constants.enemy.slow_multiplier,
            commander_pulse_interval: data.constants.enemy.commander_pulse_interval,
            commander_shield_duration: data.constants.enemy.commander_shield_duration,
            commander_shield_radius: data.constants.enemy.commander_shield_radius,
            commander_shield_multiplier: data.constants.enemy.commander_shield_multiplier,
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
fn commander_pulse_shields_nearby_escort() {
    let data = GameData::load();
    let mut manager = WaveManager::new(test_tuning(&data));
    let tuning = manager.enemy_tuning.clone();
    let mut commander = Enemy::from_spawn(EnemySpawn {
        enemy_type: EnemyType::Commander,
        position: vec2(10.0, 10.0),
        health: 100.0,
        speed: 0.0,
        scrap_reward: 1.0,
        tuning: tuning.clone(),
        path_id: "west".to_string(),
        damage_multipliers: Default::default(),
    });
    commander.ability_timer = 0.0;
    let escort = Enemy::from_spawn(EnemySpawn {
        enemy_type: EnemyType::Drone,
        position: vec2(40.0, 10.0),
        health: 100.0,
        speed: 0.0,
        scrap_reward: 1.0,
        tuning,
        path_id: "west".to_string(),
        damage_multipliers: Default::default(),
    });
    manager.enemies = vec![commander, escort];
    manager.wave_active = true;

    let paths = std::collections::HashMap::from([(
        "west".to_string(),
        vec![vec2(10.0, 10.0), vec2(500.0, 10.0)],
    )]);
    let event = manager.tick(0.1, &paths);

    assert!(matches!(event, WaveEvent::CommanderPulse { .. }));
    assert!(manager.enemies[1].is_shielded());
}

#[test]
fn scout_reports_and_saboteurs_strike_on_independent_timers() {
    let data = GameData::load();
    let mut manager = WaveManager::new(test_tuning(&data));
    let tuning = manager.enemy_tuning.clone();
    let mut scout = Enemy::from_spawn(EnemySpawn {
        enemy_type: EnemyType::Scout,
        position: vec2(10.0, 10.0),
        health: 20.0,
        speed: 0.0,
        scrap_reward: 1.0,
        tuning: tuning.clone(),
        path_id: "west".to_string(),
        damage_multipliers: Default::default(),
    });
    scout.ability_timer = 0.0;
    manager.enemies.push(scout);
    manager.wave_active = true;
    let paths = std::collections::HashMap::from([(
        "west".to_string(),
        vec![vec2(10.0, 10.0), vec2(500.0, 10.0)],
    )]);
    assert!(matches!(
        manager.tick(0.1, &paths),
        WaveEvent::ScoutReport { .. }
    ));

    manager.enemies.clear();
    let mut saboteur = Enemy::from_spawn(EnemySpawn {
        enemy_type: EnemyType::Saboteur,
        position: vec2(10.0, 10.0),
        health: 20.0,
        speed: 0.0,
        scrap_reward: 1.0,
        tuning,
        path_id: "west".to_string(),
        damage_multipliers: Default::default(),
    });
    saboteur.ability_timer = 0.0;
    manager.enemies.push(saboteur);
    manager.wave_active = true;
    assert!(matches!(
        manager.tick(0.1, &paths),
        WaveEvent::SaboteurStrike { .. }
    ));
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

#[test]
fn simultaneous_entrances_receive_the_full_wave_pressure() {
    let data = GameData::load();
    let spawn_points = vec![
        ("west".to_string(), vec2(0.0, 0.0)),
        ("north".to_string(), vec2(0.0, 400.0)),
        ("south".to_string(), vec2(0.0, 800.0)),
    ];
    let mut manager = WaveManager::new(test_tuning(&data));
    manager.set_enemy_abilities_enabled(false);
    manager.generate_wave(
        8,
        &data.enemy_defs,
        data.constants.waves.health_scale_per_wave,
        65.0,
        3,
        data.constants.waves.budget_multiplier,
        false,
        &spawn_points,
        &WaveAdaptation::default(),
    );
    let queued = manager.spawn_queue.len();
    assert!(
        queued >= spawn_points.len(),
        "late wave must be large enough to pressure every open entrance"
    );

    let paths: HashMap<String, Vec<_>> = spawn_points
        .iter()
        .map(|(id, start)| (id.clone(), vec![*start, *start + vec2(10_000.0, 0.0)]))
        .collect();
    for _ in 0..queued {
        manager.tick(manager.spawn_interval + 0.01, &paths);
    }

    let attacked: HashSet<&str> = manager
        .enemies
        .iter()
        .filter(|enemy| enemy.is_alive)
        .map(|enemy| enemy.path_id.as_str())
        .collect();
    assert_eq!(manager.enemies.len(), queued, "no spawn pressure was lost");
    assert_eq!(
        attacked.len(),
        spawn_points.len(),
        "all simultaneous entrances must receive a live attacker"
    );
}

#[test]
fn late_wave_spike_adds_a_commander_and_material_pressure() {
    let data = GameData::load();
    let tuning = test_tuning(&data);
    let spawn_points = vec![("west".to_string(), vec2(0.0, 0.0))];
    let early = build_spawn_queue(
        3,
        &data.enemy_defs,
        data.constants.waves.health_scale_per_wave,
        0.0,
        1,
        data.constants.waves.budget_multiplier,
        false,
        &spawn_points,
        &tuning,
        &WaveAdaptation::default(),
    );
    let late = build_spawn_queue(
        10,
        &data.enemy_defs,
        data.constants.waves.health_scale_per_wave,
        70.0,
        3,
        data.constants.waves.budget_multiplier,
        true,
        &spawn_points,
        &tuning,
        &WaveAdaptation::default(),
    );
    let early_health: f32 = early.iter().map(|entry| entry.health).sum();
    let late_health: f32 = late.iter().map(|entry| entry.health).sum();

    assert!(
        late.iter()
            .any(|entry| entry.enemy_type == EnemyType::Commander),
        "the late spike must contain its commander"
    );
    assert!(late.len() > early.len(), "late wave lost its larger roster");
    assert!(
        late_health > early_health * 4.0,
        "late spike has insufficient health pressure: early {early_health:.1}, late {late_health:.1}"
    );
}
