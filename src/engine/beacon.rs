//! Beacon strength, phases, and scavenging tiers.

use crate::data::strings::text;
use crate::data::EvacuationConstants;

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

pub fn phase_from_strength(strength: f32, evac: &EvacuationConstants) -> BeaconPhase {
    if strength >= evac.terminal_threshold {
        BeaconPhase::TerminalHowl
    } else if strength >= evac.screaming_threshold {
        BeaconPhase::ScreamingBeacon
    } else if strength >= evac.sustained_threshold {
        BeaconPhase::SustainedCall
    } else {
        BeaconPhase::WarmSignal
    }
}
