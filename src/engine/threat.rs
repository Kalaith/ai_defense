//! Threat signature tracking and machine awareness.

#[derive(Clone, Debug)]
pub enum ReactionTier {
    Observation,
    Adaptation,
    Suppression,
    Extermination,
}

impl ReactionTier {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Observation => "Observation",
            Self::Adaptation => "Adaptation",
            Self::Suppression => "Suppression",
            Self::Extermination => "Extermination",
        }
    }
}

pub struct ThreatSignature {
    pub energy: f32,
    pub heat: f32,
    pub data: f32,
    pub corruption: f32,
    pub noise: f32,
    pub territory: f32,
}

impl ThreatSignature {
    pub fn new() -> Self {
        Self {
            energy: 0.0,
            heat: 0.0,
            data: 0.0,
            corruption: 0.0,
            noise: 0.0,
            territory: 0.0,
        }
    }

    pub fn awareness_level(&self) -> f32 {
        self.energy + self.heat + self.data + self.corruption + self.noise + self.territory
    }

    pub fn reaction_tier(&self) -> ReactionTier {
        let level = self.awareness_level();
        if level < 25.0 {
            ReactionTier::Observation
        } else if level < 60.0 {
            ReactionTier::Adaptation
        } else if level < 100.0 {
            ReactionTier::Suppression
        } else {
            ReactionTier::Extermination
        }
    }

    pub fn add_heat(&mut self, amount: f32) {
        self.heat += amount;
    }

    pub fn add_noise(&mut self, amount: f32) {
        self.noise += amount;
    }

    pub fn add_territory(&mut self, amount: f32) {
        self.territory += amount;
    }

    pub fn tick_decay(&mut self, dt: f32) {
        let decay_rate = 0.5 * dt;
        self.energy = (self.energy - decay_rate).max(0.0);
        self.heat = (self.heat - decay_rate).max(0.0);
        self.data = (self.data - decay_rate).max(0.0);
        self.corruption = (self.corruption - decay_rate).max(0.0);
        self.noise = (self.noise - decay_rate).max(0.0);
        self.territory = (self.territory - decay_rate * 0.2).max(0.0);
    }

    pub fn add_from_wave(&mut self, wave_number: u32) {
        self.noise += 1.0 + wave_number as f32 * 0.5;
        self.heat += 0.5;
    }

    pub fn add_from_sector(&mut self, _sector_id: &str) {
        self.territory += 3.0;
        self.energy += 2.0;
    }
}
