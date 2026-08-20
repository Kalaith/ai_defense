//! Player-facing operating directives for the factory's deeper bands.

use serde::{Deserialize, Serialize};

/// A depth directive is chosen once when a new factory band is reached. The
/// choice stays with that band for the rest of the campaign, making expansion
/// a strategic commitment rather than a purely automatic stat increase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthDirective {
    ReinforcedRoutes,
    SalvageLifts,
    EvacuationRelay,
}

impl DepthDirective {
    pub const ALL: [Self; 3] = [
        Self::ReinforcedRoutes,
        Self::SalvageLifts,
        Self::EvacuationRelay,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReinforcedRoutes => "reinforced_routes",
            Self::SalvageLifts => "salvage_lifts",
            Self::EvacuationRelay => "evacuation_relay",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "reinforced_routes" => Some(Self::ReinforcedRoutes),
            "salvage_lifts" => Some(Self::SalvageLifts),
            "evacuation_relay" => Some(Self::EvacuationRelay),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "depth/tests.rs"]
mod tests;
