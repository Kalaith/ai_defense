use super::*;

#[test]
fn population_consumes_food_and_water() {
    let constants = crate::data::loader::load_constants();
    let mut population = Population::new(&constants);
    let mut water = constants.starting.water_supply;

    population.tick(10.0, &constants, 1.0, 1.0, &mut water);

    assert!(population.food_supply < constants.starting.food_supply);
    assert!(water < constants.starting.water_supply);
}

#[test]
fn running_dry_harms_a_fed_holdout() {
    let constants = crate::data::loader::load_constants();
    let mut population = Population::new(&constants);
    let starting_health = population.health;
    let starting_morale = population.morale;
    let mut water = 0.0;

    population.tick(1.0, &constants, 1.0, 1.0, &mut water);

    assert_eq!(water, 0.0);
    assert!(population.food_supply > 0.0);
    assert!(population.health < starting_health);
    assert!(population.morale < starting_morale);
}

#[test]
fn prolonged_thirst_kills_people() {
    let constants = crate::data::loader::load_constants();
    let mut population = Population::new(&constants);
    population.count = 2;
    population.health = constants.population.death_health_threshold - 1.0;
    let mut water = 0.0;

    population.tick(
        constants.population.death_interval_sec,
        &constants,
        1.0,
        1.0,
        &mut water,
    );

    assert_eq!(population.count, 1);
}

#[test]
fn beacon_pressure_accelerates_water_use() {
    let constants = crate::data::loader::load_constants();
    let mut quiet = Population::new(&constants);
    let mut active = Population::new(&constants);
    let mut quiet_water = constants.starting.water_supply;
    let mut active_water = constants.starting.water_supply;

    quiet.tick(10.0, &constants, 1.0, 1.0, &mut quiet_water);
    active.tick(
        10.0,
        &constants,
        1.0,
        constants.population.beacon_water_multiplier,
        &mut active_water,
    );

    assert!(active_water < quiet_water);
}

#[test]
fn workforce_policies_have_distinct_tradeoffs() {
    let constants = crate::data::loader::load_constants();
    let balanced = WorkforcePolicy::Balanced;
    let sustain = WorkforcePolicy::Sustain;
    let salvage = WorkforcePolicy::Salvage;
    let defense = WorkforcePolicy::Defense;

    assert!(sustain.consumption_mult(&constants) < balanced.consumption_mult(&constants));
    assert!(salvage.productivity_mult(&constants) > balanced.productivity_mult(&constants));
    assert!(defense.tower_damage_mult(&constants) > balanced.tower_damage_mult(&constants));
    assert!(salvage.noise_per_sec(&constants) > balanced.noise_per_sec(&constants));
}

#[test]
fn overcrowding_erodes_morale_without_directly_killing_people() {
    let constants = crate::data::loader::load_constants();
    let mut population = Population::new(&constants);
    let starting_morale = population.morale;
    let starting_count = population.count;

    population.apply_overcrowding(3, 10.0, &constants);

    assert!(population.morale < starting_morale);
    assert_eq!(population.count, starting_count);
}
