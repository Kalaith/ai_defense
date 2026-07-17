//! Threat signature tracking and machine awareness.

use crate::data::strings::text;
use crate::engine::enemy::EnemyType;

#[derive(Clone, Debug)]
pub enum ReactionTier {
    Observation,
    Adaptation,
    Suppression,
    Extermination,
}

impl ReactionTier {
    pub fn label(&self) -> &'static str {
        let tiers = &text().threat.tiers;
        match self {
            Self::Observation => tiers.observation.as_str(),
            Self::Adaptation => tiers.adaptation.as_str(),
            Self::Suppression => tiers.suppression.as_str(),
            Self::Extermination => tiers.extermination.as_str(),
        }
    }
}

/// The six machine-awareness signatures the factory emits. Which one is loudest
/// is surfaced to the player and biases what the machines send (see
/// `preferred_enemy`), so *how* you run the factory shapes the assault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreatKind {
    Energy,
    Heat,
    Data,
    Corruption,
    Noise,
    Territory,
}

impl ThreatKind {
    pub fn label(self) -> &'static str {
        let signatures = &text().threat.signatures;
        match self {
            Self::Energy => signatures.energy.as_str(),
            Self::Heat => signatures.heat.as_str(),
            Self::Data => signatures.data.as_str(),
            Self::Corruption => signatures.corruption.as_str(),
            Self::Noise => signatures.noise.as_str(),
            Self::Territory => signatures.territory.as_str(),
        }
    }

    /// The enemy archetype the machines favour when this signature dominates.
    pub fn preferred_enemy(self) -> EnemyType {
        match self {
            // A loud power grid draws fast drones to the source.
            Self::Energy => EnemyType::Drone,
            // Heavy armour wades into a hot firefight.
            Self::Heat => EnemyType::HeavyUnit,
            // Coordinated data traffic invites a command response.
            Self::Data => EnemyType::Commander,
            // Forbidden AI use invites infiltration.
            Self::Corruption => EnemyType::Saboteur,
            // A general racket brings a scout swarm.
            Self::Noise => EnemyType::Scout,
            // Sprawling territory is met with reclamation commanders.
            Self::Territory => EnemyType::Commander,
        }
    }

    /// Whether a high reading of this signature can pull its archetype into the
    /// roster earlier than the wave schedule would normally allow.
    pub fn allows_early_unlock(self) -> bool {
        matches!(self, Self::Heat | Self::Corruption | Self::Territory)
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

    pub fn reaction_tier(&self, constants: &crate::data::ThreatConstants) -> ReactionTier {
        let level = self.awareness_level();
        if level < constants.tier_2_awareness {
            ReactionTier::Observation
        } else if level < constants.tier_3_awareness {
            ReactionTier::Adaptation
        } else if level < constants.tier_4_awareness {
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

    pub fn add_energy(&mut self, amount: f32) {
        self.energy += amount;
    }

    pub fn add_data(&mut self, amount: f32) {
        self.data += amount;
    }

    pub fn add_corruption(&mut self, amount: f32) {
        self.corruption += amount;
    }

    pub fn add_territory(&mut self, amount: f32) {
        self.territory += amount;
    }

    /// The loudest signature and its value — what is currently drawing the most
    /// machine attention. Ties resolve in enum order.
    pub fn dominant(&self) -> (ThreatKind, f32) {
        [
            (ThreatKind::Energy, self.energy),
            (ThreatKind::Heat, self.heat),
            (ThreatKind::Data, self.data),
            (ThreatKind::Corruption, self.corruption),
            (ThreatKind::Noise, self.noise),
            (ThreatKind::Territory, self.territory),
        ]
        .into_iter()
        .fold((ThreatKind::Noise, 0.0), |best, cur| {
            if cur.1 > best.1 {
                cur
            } else {
                best
            }
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    fn signature_at(level: f32) -> ThreatSignature {
        let mut sig = ThreatSignature::new();
        sig.energy = level;
        sig
    }

    #[test]
    fn reaction_tier_boundaries_match_constants_json() {
        let data = GameData::load();
        let t = &data.constants.threat;

        assert!(matches!(
            signature_at(0.0).reaction_tier(t),
            ReactionTier::Observation
        ));
        assert!(matches!(
            signature_at(t.tier_2_awareness - 0.01).reaction_tier(t),
            ReactionTier::Observation
        ));
        assert!(matches!(
            signature_at(t.tier_2_awareness).reaction_tier(t),
            ReactionTier::Adaptation
        ));
        assert!(matches!(
            signature_at(t.tier_3_awareness - 0.01).reaction_tier(t),
            ReactionTier::Adaptation
        ));
        assert!(matches!(
            signature_at(t.tier_3_awareness).reaction_tier(t),
            ReactionTier::Suppression
        ));
        assert!(matches!(
            signature_at(t.tier_4_awareness - 0.01).reaction_tier(t),
            ReactionTier::Suppression
        ));
        assert!(matches!(
            signature_at(t.tier_4_awareness).reaction_tier(t),
            ReactionTier::Extermination
        ));
    }
}
