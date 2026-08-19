use super::*;
use crate::engine::enemy::EnemyTuning;
use macroquad::prelude::vec2;

fn test_tuning() -> TowerTuning {
    TowerTuning {
        shot_ttl: 0.1,
        emp_slow_duration: 2.0,
        area_denial_damage_scale: 0.6,
        subversion_chain_radius: 90.0,
        subversion_chain_damage_fraction: 0.3,
        commander_death_radius: 150.0,
        commander_death_fraction: 0.1,
        heat_per_shot: 0.01,
    }
}

fn test_enemy(enemy_type: EnemyType, position: Vec2, health: f32) -> Enemy {
    Enemy::new(
        enemy_type,
        position,
        health,
        50.0,
        5.0,
        EnemyTuning {
            scout_dodge_chance: 0.0,
            scout_dodge_duration: 0.0,
            hit_flash_duration: 0.1,
            saboteur_skip_chance: 0.0,
            slow_multiplier: 0.5,
        },
        "west".to_string(),
        crate::data::DamageMultipliers::default(),
    )
}

fn ready_tower(tower_type: TowerType, position: Vec2, range: f32, damage: f32) -> Tower {
    Tower::new(
        tower_type,
        "test_tower".to_string(),
        position,
        range,
        damage,
        1.0,
        5.0,
        10.0,
        Color::new(1.0, 1.0, 1.0, 1.0),
    )
}

#[test]
fn ballistic_tower_kills_enemy_in_range_and_earns_scrap() {
    let mut towers = vec![ready_tower(
        TowerType::Ballistic,
        vec2(0.0, 0.0),
        100.0,
        50.0,
    )];
    let mut enemies = vec![test_enemy(EnemyType::Scout, vec2(10.0, 0.0), 10.0)];

    let result = tick_towers(
        &mut towers,
        &mut enemies,
        0.1,
        1.0,
        1.0,
        1.0,
        1.0,
        &test_tuning(),
    );

    assert!(!enemies[0].is_alive, "enemy should die to a 50-damage hit");
    assert_eq!(result.scrap_earned, 5.0);
    assert_eq!(result.death_positions.len(), 1);
    assert_eq!(result.tower_stats[0].kills, 1);
    assert!(towers[0].cooldown > 0.0, "firing should start the cooldown");
}

#[test]
fn tower_does_not_fire_at_enemy_outside_range() {
    let mut towers = vec![ready_tower(
        TowerType::Ballistic,
        vec2(0.0, 0.0),
        10.0,
        50.0,
    )];
    let mut enemies = vec![test_enemy(EnemyType::Scout, vec2(500.0, 0.0), 10.0)];

    let result = tick_towers(
        &mut towers,
        &mut enemies,
        0.1,
        1.0,
        1.0,
        1.0,
        1.0,
        &test_tuning(),
    );

    assert!(
        enemies[0].is_alive,
        "out-of-range enemy should be untouched"
    );
    assert_eq!(result.scrap_earned, 0.0);
    assert_eq!(
        towers[0].cooldown, 0.0,
        "a tower that didn't fire shouldn't cool down"
    );
}

#[test]
fn laser_tower_applies_heavy_unit_multiplier() {
    let tuning = test_tuning();
    let mut towers = vec![ready_tower(TowerType::Laser, vec2(0.0, 0.0), 100.0, 10.0)];
    let mut enemies = vec![test_enemy(EnemyType::HeavyUnit, vec2(5.0, 0.0), 1000.0)];
    enemies[0].damage_multipliers.laser = 1.5;

    tick_towers(&mut towers, &mut enemies, 0.1, 1.0, 1.0, 1.0, 1.0, &tuning);

    let expected_damage = 10.0 * enemies[0].damage_multipliers.laser;
    assert_eq!(enemies[0].health, 1000.0 - expected_damage);
}

#[test]
fn reactive_drone_plating_punishes_ballistic_damage() {
    let mut towers = vec![ready_tower(
        TowerType::Ballistic,
        vec2(0.0, 0.0),
        100.0,
        20.0,
    )];
    let mut enemies = vec![test_enemy(EnemyType::Drone, vec2(5.0, 0.0), 100.0)];
    enemies[0].damage_multipliers.ballistic = 0.55;

    tick_towers(
        &mut towers,
        &mut enemies,
        0.1,
        1.0,
        1.0,
        1.0,
        1.0,
        &test_tuning(),
    );

    assert_eq!(enemies[0].health, 89.0);
}

#[test]
fn infiltrator_phase_mesh_is_weak_to_emp() {
    let mut towers = vec![ready_tower(TowerType::Emp, vec2(0.0, 0.0), 100.0, 10.0)];
    let mut enemies = vec![test_enemy(EnemyType::Saboteur, vec2(5.0, 0.0), 100.0)];
    enemies[0].damage_multipliers.emp = 1.5;

    tick_towers(
        &mut towers,
        &mut enemies,
        0.1,
        1.0,
        1.0,
        1.0,
        1.0,
        &test_tuning(),
    );

    assert_eq!(enemies[0].health, 85.0);
    assert!(enemies[0].slowed_timer > 0.0);
}

#[test]
fn commander_death_deals_splash_damage_to_nearby_enemies() {
    let tuning = test_tuning();
    let mut towers = vec![ready_tower(
        TowerType::Ballistic,
        vec2(0.0, 0.0),
        100.0,
        1000.0,
    )];
    // Bystander health (0.1) sits well below the burst damage
    // (max_health * commander_death_fraction = 5.0 * 0.1 = 0.5), so the
    // splash is guaranteed to finish it off.
    let mut enemies = vec![
        test_enemy(EnemyType::Commander, vec2(10.0, 0.0), 10.0),
        test_enemy(EnemyType::Scout, vec2(15.0, 0.0), 0.1),
    ];

    let result = tick_towers(&mut towers, &mut enemies, 0.1, 1.0, 1.0, 1.0, 1.0, &tuning);

    assert!(
        !enemies[0].is_alive,
        "commander should die to the direct hit"
    );
    assert!(
        !enemies[1].is_alive,
        "nearby enemy should die to commander death-burst splash damage"
    );
    assert_eq!(
        result.scrap_earned, 10.0,
        "both the commander and the splash kill should award scrap"
    );
}

#[test]
fn area_denial_tower_hits_every_enemy_in_range() {
    let tuning = test_tuning();
    let mut towers = vec![ready_tower(
        TowerType::AreaDenial,
        vec2(0.0, 0.0),
        50.0,
        20.0,
    )];
    let mut enemies = vec![
        test_enemy(EnemyType::Scout, vec2(10.0, 0.0), 100.0),
        test_enemy(EnemyType::Scout, vec2(-10.0, 0.0), 100.0),
    ];

    tick_towers(&mut towers, &mut enemies, 0.1, 1.0, 1.0, 1.0, 1.0, &tuning);

    let expected_damage = 20.0 * tuning.area_denial_damage_scale;
    assert_eq!(enemies[0].health, 100.0 - expected_damage);
    assert_eq!(enemies[1].health, 100.0 - expected_damage);
}
