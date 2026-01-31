//! Human holdout: population, morale, food, and productivity.

use crate::data::GameConstants;

pub struct Population {
    pub count: u32,
    pub morale: f32,
    pub health: f32,
    pub food_supply: f32,
    pub food_consumption_per_person: f32,
    pub death_timer: f32,
}

impl Population {
    pub fn new(constants: &GameConstants) -> Self {
        Self {
            count: constants.starting.population,
            morale: constants.starting.morale,
            health: constants.starting.health,
            food_supply: constants.starting.food_supply,
            food_consumption_per_person: constants.population.food_consumption_per_person,
            death_timer: 0.0,
        }
    }

    pub fn productivity(&self, constants: &GameConstants) -> f32 {
        let morale_factor = if self.morale < constants.population.low_morale_threshold {
            constants.population.low_morale_productivity
        } else {
            self.morale / 100.0
        };
        let health_factor = self.health / 100.0;
        self.count as f32 * morale_factor * health_factor
    }

    pub fn daily_food_cost(&self) -> f32 {
        self.count as f32 * self.food_consumption_per_person
    }

    pub fn consume_food(&mut self) {
        let cost = self.daily_food_cost();
        if self.food_supply >= cost {
            self.food_supply -= cost;
        } else {
            self.food_supply = 0.0;
            self.morale -= 5.0;
            self.health -= 3.0;
        }

        self.morale = self.morale.clamp(0.0, 100.0);
        self.health = self.health.clamp(0.0, 100.0);
    }

    pub fn tick(&mut self, dt: f32, constants: &GameConstants) {
        let consumption = self.count as f32 * constants.population.food_per_person_per_sec * dt;
        if consumption > 0.0 {
            self.food_supply = (self.food_supply - consumption).max(0.0);
        }

        if self.food_supply <= 0.0 {
            self.morale -= constants.population.starve_morale_loss_per_sec * dt;
            self.health -= constants.population.starve_health_loss_per_sec * dt;

            if self.health < constants.population.death_health_threshold && self.count > 0 {
                self.death_timer += dt;
                while self.death_timer >= constants.population.death_interval_sec && self.count > 0 {
                    self.count -= 1;
                    self.death_timer -= constants.population.death_interval_sec;
                }
            }
        } else {
            self.death_timer = 0.0;
            if self.morale < constants.population.recover_morale_threshold {
                self.morale += constants.population.recover_morale_per_sec * dt;
            }
            if self.health < constants.population.recover_health_threshold {
                self.health += constants.population.recover_health_per_sec * dt;
            }
        }

        self.morale = self.morale.clamp(0.0, 100.0);
        self.health = self.health.clamp(0.0, 100.0);
    }
}
