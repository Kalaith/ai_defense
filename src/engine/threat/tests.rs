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

/// Territory decays slower than the other five signatures — that's the
/// dial balanced against the survival-proof test (see
/// docs/design-core-loop-gap notes). Pin the ratio so a future constants
/// edit can't silently flatten it.
#[test]
fn territory_decays_slower_than_other_signatures() {
    let data = GameData::load();
    let t = &data.constants.threat;
    let mut sig = ThreatSignature {
        energy: 10.0,
        heat: 10.0,
        data: 10.0,
        corruption: 10.0,
        noise: 10.0,
        territory: 10.0,
    };

    sig.tick_decay(1.0, t);

    let uniform_decay = t.decay_rate;
    let territory_decay = t.decay_rate * t.territory_decay_mult;
    assert_eq!(sig.energy, (10.0 - uniform_decay).max(0.0));
    assert_eq!(sig.heat, (10.0 - uniform_decay).max(0.0));
    assert_eq!(sig.territory, (10.0 - territory_decay).max(0.0));
    assert!(
        sig.territory > sig.energy,
        "territory should linger longer than the other signatures"
    );
}

#[test]
fn decay_never_pushes_a_signature_below_zero() {
    let data = GameData::load();
    let t = &data.constants.threat;
    let mut sig = signature_at(0.2);
    sig.territory = 0.05;

    sig.tick_decay(10.0, t);

    assert_eq!(sig.energy, 0.0);
    assert_eq!(sig.territory, 0.0);
}
