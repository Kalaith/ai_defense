//! Human holdout: population, morale, food, and productivity.

use crate::data::GameConstants;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkforcePolicy {
    Sustain,
    #[default]
    Balanced,
    Salvage,
    Defense,
}

impl WorkforcePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sustain => "sustain",
            Self::Balanced => "balanced",
            Self::Salvage => "salvage",
            Self::Defense => "defense",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "sustain" => Self::Sustain,
            "salvage" => Self::Salvage,
            "defense" => Self::Defense,
            _ => Self::Balanced,
        }
    }

    pub fn consumption_mult(self, constants: &GameConstants) -> f32 {
        match self {
            Self::Sustain => constants.population.workforce.sustain_consumption_mult,
            Self::Balanced => 1.0,
            Self::Salvage => constants.population.workforce.salvage_consumption_mult,
            Self::Defense => constants.population.workforce.defense_consumption_mult,
        }
    }

    pub fn productivity_mult(self, constants: &GameConstants) -> f32 {
        match self {
            Self::Sustain => constants.population.workforce.sustain_productivity_mult,
            Self::Balanced => 1.0,
            Self::Salvage => constants.population.workforce.salvage_productivity_mult,
            Self::Defense => constants.population.workforce.defense_productivity_mult,
        }
    }

    pub fn tower_damage_mult(self, constants: &GameConstants) -> f32 {
        match self {
            Self::Sustain => constants.population.workforce.sustain_damage_mult,
            Self::Defense => constants.population.workforce.defense_damage_mult,
            Self::Balanced | Self::Salvage => 1.0,
        }
    }

    pub fn noise_per_sec(self, constants: &GameConstants) -> f32 {
        match self {
            Self::Salvage => constants.population.workforce.salvage_noise_per_sec,
            Self::Sustain | Self::Balanced | Self::Defense => 0.0,
        }
    }
}

pub struct Population {
    pub count: u32,
    pub morale: f32,
    pub health: f32,
    pub food_supply: f32,
    pub death_timer: f32,
    pub workforce_policy: WorkforcePolicy,
}

impl Population {
    pub fn new(constants: &GameConstants) -> Self {
        Self {
            count: constants.starting.population,
            morale: constants.starting.morale,
            health: constants.starting.health,
            food_supply: constants.starting.food_supply,
            death_timer: 0.0,
            workforce_policy: WorkforcePolicy::default(),
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

    /// Advance the holdout's food and water demand. Water lives in the shared
    /// resource ledger because factory buildings produce it directly.
    pub fn tick(
        &mut self,
        dt: f32,
        constants: &GameConstants,
        food_consumption_mult: f32,
        water_consumption_mult: f32,
        water_supply: &mut f32,
    ) {
        let food_consumption = self.count as f32
            * constants.population.food_per_person_per_sec
            * food_consumption_mult
            * dt;
        let water_consumption = self.count as f32
            * constants.population.water_per_person_per_sec
            * water_consumption_mult
            * dt;
        if food_consumption > 0.0 {
            self.food_supply = (self.food_supply - food_consumption).max(0.0);
        }
        if water_consumption > 0.0 {
            *water_supply = (*water_supply - water_consumption).max(0.0);
        }

        let starving = self.food_supply <= 0.0;
        let thirsty = *water_supply <= 0.0;

        if starving {
            self.morale -= constants.population.starve_morale_loss_per_sec * dt;
            self.health -= constants.population.starve_health_loss_per_sec * dt;
        }
        if thirsty {
            self.morale -= constants.population.thirst_morale_loss_per_sec * dt;
            self.health -= constants.population.thirst_health_loss_per_sec * dt;
        }

        if starving || thirsty {
            if self.health < constants.population.death_health_threshold && self.count > 0 {
                self.death_timer += dt;
                while self.death_timer >= constants.population.death_interval_sec && self.count > 0
                {
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

    pub fn apply_overcrowding(&mut self, excess: u32, dt: f32, constants: &GameConstants) {
        if excess == 0 {
            return;
        }
        self.morale = (self.morale
            - excess as f32
                * constants
                    .population
                    .overcrowding_morale_loss_per_person_per_sec
                * dt)
            .max(0.0);
    }
}

#[cfg(test)]
mod tests;
