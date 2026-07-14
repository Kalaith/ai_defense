//! The NEXT STEP advisor: picks the single most useful thing to do right now.
//!
//! Order matters — the first matching rule wins, so the list reads as a
//! priority ladder from "you are actively losing" down to "idle inspection".

use crate::data::GameData;
use crate::engine::beacon::BeaconPhase;
use crate::engine::map::{BuildingState, MapBuilding};

use super::super::GameplayState;
use super::{AdviceTarget, PowerGridSnapshot, SuggestedAction};

impl GameplayState {
    pub(super) fn suggest_next_action(
        &self,
        data: &GameData,
        power: &PowerGridSnapshot,
    ) -> SuggestedAction {
        // Towers are dark: nothing else matters until the grid is back.
        if power.offline_towers > 0 || (power.net < 0.0 && power.battery <= 0.0) {
            if let Some(idx) = self.best_power_system() {
                let building = &self.map_state.buildings[idx];
                return building_action(
                    idx,
                    self.building_display_name(building),
                    building,
                    "Restore grid headroom",
                );
            }
            return SuggestedAction {
                label: "Restore power before building more towers".to_string(),
                detail: format!("{} towers are offline", power.offline_towers),
                cost: "Cost: repair power systems".to_string(),
                risk: "Risk: defenses inactive".to_string(),
                target: AdviceTarget::None,
            };
        }

        if self.food_seconds_remaining() <= 40.0 {
            if let Some(idx) = self.best_food_system() {
                let building = &self.map_state.buildings[idx];
                return building_action(
                    idx,
                    self.building_display_name(building),
                    building,
                    "Stabilize food production",
                );
            }
        }

        if let Some(idx) = self.best_income_system(true) {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                "Increase resource income",
            );
        }

        if self.towers.is_empty() {
            if let Some(def) = data.tower_def_by_id("ballistic_turret") {
                if self.resources.scrap >= def.cost_scrap {
                    return SuggestedAction {
                        label: "Build Ballistic Turret".to_string(),
                        detail: "Place it on a powered pad covering the main route".to_string(),
                        cost: format!(
                            "Cost: {:.0} scrap, {:.0} power",
                            def.cost_scrap, def.cost_power
                        ),
                        risk: "Risk: +0.50 noise".to_string(),
                        target: AdviceTarget::TowerDef(def.id.clone()),
                    };
                }
            }
        }

        if let Some(idx) = self.best_repaired_system() {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                "Turn repaired machinery online",
            );
        }

        if let Some(idx) = self.best_powerable_pad() {
            let slot = &self.map_state.slots[idx];
            return SuggestedAction {
                label: format!("Power {}", slot.id),
                detail: "Use this pad for stronger lane coverage".to_string(),
                cost: format!("Cost: {:.0} scrap", slot.power_cost),
                risk: slot
                    .opens_entrance
                    .as_ref()
                    .map(|p| format!("Risk: opens {}", self.path_display_name(p)))
                    .unwrap_or_else(|| "Risk: none".to_string()),
                target: AdviceTarget::Slot(idx),
            };
        }

        if !self.beacon_active && !self.towers.is_empty() {
            return SuggestedAction {
                label: "Start Beacon".to_string(),
                detail: "Draw enemies while scavengers gather resources".to_string(),
                cost: "Cost: starts Wave 1 timer".to_string(),
                risk: format!("Risk: {}", self.beacon_phase.label()),
                target: AdviceTarget::BeaconStart,
            };
        }

        // A loud beacon is banked value at risk: get the teams and the cycle in
        // before the howl takes them.
        if self.beacon_active && self.beacon_phase.rank() >= BeaconPhase::ScreamingBeacon.rank() {
            if self.scavengers_out > 0 {
                return SuggestedAction {
                    label: "Recall Scavengers".to_string(),
                    detail: "Bank resources before the signal gets worse".to_string(),
                    cost: "Cost: stops field gains".to_string(),
                    risk: "Risk: teams outside".to_string(),
                    target: AdviceTarget::ScavengerRecall,
                };
            }
            if self.current_wave >= 1 {
                return SuggestedAction {
                    label: "Shutdown Beacon".to_string(),
                    detail: "Stop new waves once the lane is stable".to_string(),
                    cost: "Cost: ends scavenging".to_string(),
                    risk: "Risk: active enemies remain".to_string(),
                    target: AdviceTarget::BeaconShutdown,
                };
            }
        }

        if let Some(idx) = self.best_income_system(false) {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                "Repair the next damaged system",
            );
        }

        SuggestedAction {
            label: "Inspect Factory Console".to_string(),
            detail: "Review available upgrades and unlocked machinery".to_string(),
            cost: "Cost: none".to_string(),
            risk: "Risk: none".to_string(),
            target: AdviceTarget::None,
        }
    }
}

/// Phrase a building recommendation as the next verb its state allows.
fn building_action(
    idx: usize,
    name: String,
    building: &MapBuilding,
    purpose: &str,
) -> SuggestedAction {
    match building.state {
        BuildingState::Broken => SuggestedAction {
            label: format!("Repair {}", name),
            detail: purpose.to_string(),
            cost: format!("Cost: {:.0} scrap", building.repair_cost),
            risk: format!("Risk: +{:.2} noise/s", building.threat_per_sec),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Repaired => SuggestedAction {
            label: format!("Power {}", name),
            detail: purpose.to_string(),
            cost: format!("Cost: {:.0} scrap", building.power_cost),
            risk: format!("Risk: +{:.2} noise/s", building.threat_per_sec),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Powered => SuggestedAction {
            label: format!("Inspect {}", name),
            detail: purpose.to_string(),
            cost: "Cost: online".to_string(),
            risk: format!("Risk: +{:.2} noise/s", building.threat_per_sec),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Disabled => SuggestedAction {
            label: format!("Inspect {}", name),
            detail: "System disabled".to_string(),
            cost: "Cost: locked".to_string(),
            risk: "Risk: offline".to_string(),
            target: AdviceTarget::Building(idx),
        },
    }
}
