//! Beacon strength, phases, and scavenging tiers.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BeaconPhase {
    WarmSignal,
    SustainedCall,
    ScreamingBeacon,
    TerminalHowl,
}

impl BeaconPhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WarmSignal => "Warm Signal",
            Self::SustainedCall => "Sustained Call",
            Self::ScreamingBeacon => "Screaming Beacon",
            Self::TerminalHowl => "Terminal Howl",
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
