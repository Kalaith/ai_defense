//! Beacon strength, phases, and scavenging tiers.

use crate::data::strings::text;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BeaconPhase {
    WarmSignal,
    SustainedCall,
    ScreamingBeacon,
    TerminalHowl,
}

impl BeaconPhase {
    pub fn label(&self) -> &'static str {
        let phases = &text().beacon.phases;
        match self {
            Self::WarmSignal => phases.warm_signal.as_str(),
            Self::SustainedCall => phases.sustained_call.as_str(),
            Self::ScreamingBeacon => phases.screaming_beacon.as_str(),
            Self::TerminalHowl => phases.terminal_howl.as_str(),
        }
    }

    pub fn tier_floor(&self) -> u32 {
        match self {
            Self::WarmSignal => 1,
            Self::SustainedCall => 2,
            Self::ScreamingBeacon => 3,
            Self::TerminalHowl => 3,
        }
    }

    pub fn rank(&self) -> u32 {
        match self {
            Self::WarmSignal => 0,
            Self::SustainedCall => 1,
            Self::ScreamingBeacon => 2,
            Self::TerminalHowl => 3,
        }
    }
}

pub fn phase_from_strength(strength: f32) -> BeaconPhase {
    if strength >= 61.0 {
        BeaconPhase::TerminalHowl
    } else if strength >= 36.0 {
        BeaconPhase::ScreamingBeacon
    } else if strength >= 16.0 {
        BeaconPhase::SustainedCall
    } else {
        BeaconPhase::WarmSignal
    }
}
