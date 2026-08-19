//! The NEXT STEP advisor: picks the single most useful thing to do right now.
//!
//! Order matters — the first matching rule wins, so the list reads as a
//! priority ladder from "you are actively losing" down to "idle inspection".

use crate::data::strings::{fill, text};
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
        let t = &text().advice;
        if self.vault_takeover.active || self.vault_takeover.upload_complete {
            return SuggestedAction {
                label: t.vault_hold.clone(),
                detail: t.vault_hold_detail.clone(),
                cost: t.cost_online.clone(),
                risk: t.vault_takeover_risk.clone(),
                target: AdviceTarget::FactoryCore,
            };
        }
        if self.vault_takeover_ready() {
            return SuggestedAction {
                label: t.vault_takeover.clone(),
                detail: t.vault_takeover_detail.clone(),
                cost: t.vault_takeover_cost.clone(),
                risk: t.vault_takeover_risk.clone(),
                target: AdviceTarget::FactoryCore,
            };
        }
        // Towers are dark: nothing else matters until the grid is back.
        if power.offline_towers > 0 || (power.net < 0.0 && power.battery <= 0.0) {
            if let Some(idx) = self.best_power_system() {
                let building = &self.map_state.buildings[idx];
                return building_action(
                    idx,
                    self.building_display_name(building),
                    building,
                    &t.purpose_grid,
                );
            }
            return SuggestedAction {
                label: t.restore_power.clone(),
                detail: fill(
                    &t.restore_power_detail,
                    &[("n", &power.offline_towers.to_string())],
                ),
                cost: t.restore_power_cost.clone(),
                risk: t.restore_power_risk.clone(),
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
                    &t.purpose_food,
                );
            }
        }

        if let Some(idx) = self.best_income_system(true) {
            let building = &self.map_state.buildings[idx];
            return building_action(
                idx,
                self.building_display_name(building),
                building,
                &t.purpose_income,
            );
        }

        if self.towers.len() < 3 {
            if let Some(def) = data.tower_def_by_id("ballistic_turret") {
                if self.resources.scrap >= def.cost_scrap {
                    return SuggestedAction {
                        label: t.build_tower.clone(),
                        detail: t.build_tower_detail.clone(),
                        cost: fill(
                            &t.build_tower_cost,
                            &[
                                ("scrap", &format!("{:.0}", def.cost_scrap)),
                                ("power", &format!("{:.0}", def.cost_power)),
                            ],
                        ),
                        risk: t.build_tower_risk.clone(),
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
                &t.purpose_turn_online,
            );
        }

        if let Some(idx) = self.best_powerable_pad() {
            let slot = &self.map_state.slots[idx];
            return SuggestedAction {
                label: fill(&t.power_pad, &[("id", &slot.id)]),
                detail: t.power_pad_detail.clone(),
                cost: fill(&t.cost_scrap, &[("n", &format!("{:.0}", slot.power_cost))]),
                risk: slot
                    .opens_entrance
                    .as_ref()
                    .map(|p| fill(&t.risk_opens, &[("path", &self.path_display_name(p))]))
                    .unwrap_or_else(|| t.risk_none.clone()),
                target: AdviceTarget::Slot(idx),
            };
        }

        if !self.beacon_active && !self.towers.is_empty() {
            return SuggestedAction {
                label: t.start_beacon.clone(),
                detail: t.start_beacon_detail.clone(),
                cost: t.start_beacon_cost.clone(),
                risk: fill(
                    &t.start_beacon_risk,
                    &[("phase", self.beacon_phase.label())],
                ),
                target: AdviceTarget::BeaconStart,
            };
        }

        // A loud beacon is banked value at risk: get the teams and the cycle in
        // before the howl takes them.
        if self.beacon_active && self.beacon_phase.rank() >= BeaconPhase::ScreamingBeacon.rank() {
            if self.scavengers_out > 0 {
                return SuggestedAction {
                    label: t.recall.clone(),
                    detail: t.recall_detail.clone(),
                    cost: t.recall_cost.clone(),
                    risk: t.recall_risk.clone(),
                    target: AdviceTarget::ScavengerRecall,
                };
            }
            if self.current_wave >= 1 {
                return SuggestedAction {
                    label: t.shutdown.clone(),
                    detail: t.shutdown_detail.clone(),
                    cost: t.shutdown_cost.clone(),
                    risk: t.shutdown_risk.clone(),
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
                &t.purpose_next_damaged,
            );
        }

        SuggestedAction {
            label: t.console.clone(),
            detail: t.console_detail.clone(),
            cost: t.cost_none.clone(),
            risk: t.risk_none.clone(),
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
    let t = &text().advice;
    let noise = || {
        fill(
            &t.risk_noise,
            &[("n", &format!("{:.2}", building.threat_per_sec))],
        )
    };
    match building.state {
        BuildingState::Broken => SuggestedAction {
            label: fill(&t.repair, &[("name", &name)]),
            detail: purpose.to_string(),
            cost: fill(
                &t.cost_scrap,
                &[("n", &format!("{:.0}", building.repair_cost))],
            ),
            risk: noise(),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Repaired => SuggestedAction {
            label: fill(&t.power, &[("name", &name)]),
            detail: purpose.to_string(),
            cost: fill(
                &t.cost_scrap,
                &[("n", &format!("{:.0}", building.power_cost))],
            ),
            risk: noise(),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Powered => SuggestedAction {
            label: fill(&t.inspect, &[("name", &name)]),
            detail: purpose.to_string(),
            cost: t.cost_online.clone(),
            risk: noise(),
            target: AdviceTarget::Building(idx),
        },
        BuildingState::Disabled => SuggestedAction {
            label: fill(&t.inspect, &[("name", &name)]),
            detail: t.disabled_detail.clone(),
            cost: t.cost_locked.clone(),
            risk: t.risk_offline.clone(),
            target: AdviceTarget::Building(idx),
        },
    }
}
