//! "Which one?" queries backing the advisor: the best pad to power, the best
//! machine to bring online, the best pad for a given tower.

use crate::data::{GameData, TowerDef};
use crate::engine::map::{BuildingState, MapBuilding, MapSlot, SlotState};

use super::super::GameplayState;
use super::InvalidPadReason;

impl GameplayState {
    pub fn best_slot_for_tower_id(&self, tower_id: &str, data: &GameData) -> Option<usize> {
        data.tower_def_by_id(tower_id)
            .and_then(|def| self.best_slot_for_tower(def))
    }

    /// Prefer the pad covering the most routes; break ties toward the pad
    /// furthest forward (nearest the route, furthest from the core).
    pub fn best_slot_for_tower(&self, tower_def: &TowerDef) -> Option<usize> {
        let range = self.effective_tower_range(tower_def.base_range);
        self.map_state
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                self.map_state.is_slot_visible(slot)
                    && slot.state == SlotState::Powered
                    && slot.tower_index.is_none()
            })
            .map(|(idx, slot)| {
                let covered = self.covered_paths_for_range(slot.position, range).len() as f32;
                let route_dist = self.distance_to_active_route(slot.position);
                let core_dist = (slot.position - self.map_state.factory_core).length();
                (idx, covered, route_dist, core_dist)
            })
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(idx, _, _, _)| idx)
    }

    /// A building the advisor may recommend: visible AND within the revealed
    /// frontier. Buildings that belong to no section (e.g. the power turbine)
    /// pass `is_building_visible` unconditionally, so without the x-gate the
    /// advisor would suggest machinery the player can't even scroll to.
    fn building_reachable(&self, building: &MapBuilding) -> bool {
        self.map_state.is_building_visible(building)
            && building.position.x <= self.map_state.max_visible_x() + 120.0
    }

    pub(super) fn best_power_system(&self) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.building_reachable(b) && b.boon.power_per_sec > 0.0)
            .filter(|(_, b)| {
                matches!(b.state, BuildingState::Broken | BuildingState::Repaired)
                    && self.can_progress_building(b)
            })
            .max_by(|(_, a), (_, b)| {
                a.boon
                    .power_per_sec
                    .partial_cmp(&b.boon.power_per_sec)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    pub(super) fn best_food_system(&self) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.building_reachable(b))
            .filter(|(_, b)| b.boon.food_per_sec > 0.0 || b.boon.water_per_sec > 0.0)
            .filter(|(_, b)| {
                matches!(b.state, BuildingState::Broken | BuildingState::Repaired)
                    && self.can_progress_building(b)
            })
            .max_by(|(_, a), (_, b)| {
                (a.boon.food_per_sec + a.boon.water_per_sec)
                    .partial_cmp(&(b.boon.food_per_sec + b.boon.water_per_sec))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    pub(super) fn best_income_system(&self, affordable_only: bool) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.building_reachable(b))
            .filter(|(_, b)| b.state == BuildingState::Broken)
            .filter(|(_, b)| !affordable_only || self.resources.scrap >= b.repair_cost)
            .max_by(|(_, a), (_, b)| {
                building_score(a)
                    .partial_cmp(&building_score(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    pub(super) fn best_repaired_system(&self) -> Option<usize> {
        self.map_state
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| self.building_reachable(b))
            .filter(|(_, b)| b.state == BuildingState::Repaired)
            .filter(|(_, b)| self.resources.scrap >= b.power_cost)
            .max_by(|(_, a), (_, b)| {
                building_score(a)
                    .partial_cmp(&building_score(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    pub(super) fn best_powerable_pad(&self) -> Option<usize> {
        self.map_state
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                self.map_state.is_slot_visible(slot)
                    && slot.state == SlotState::Cleared
                    && self.resources.scrap >= slot.power_cost
            })
            .map(|(idx, slot)| {
                let route_dist = self.distance_to_active_route(slot.position);
                let core_dist = (slot.position - self.map_state.factory_core).length();
                (idx, route_dist, core_dist)
            })
            .min_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(idx, _, _)| idx)
    }

    /// Whether the player can afford this building's next state transition.
    fn can_progress_building(&self, building: &MapBuilding) -> bool {
        match building.state {
            BuildingState::Broken => self.resources.scrap >= building.repair_cost,
            BuildingState::Repaired => self.resources.scrap >= building.power_cost,
            BuildingState::Powered | BuildingState::Disabled => false,
        }
    }

    pub(super) fn invalid_pad_reason(
        &self,
        slot: &MapSlot,
        can_afford: bool,
    ) -> Option<InvalidPadReason> {
        if slot.tower_index.is_some() {
            return Some(InvalidPadReason::Occupied);
        }
        if slot.state != SlotState::Powered {
            return Some(InvalidPadReason::NeedsPower);
        }
        if !can_afford {
            return Some(InvalidPadReason::TooExpensive);
        }
        None
    }

    /// A tower may be built if the grid can carry it outright, or if the battery
    /// holds enough charge to absorb the deficit for now.
    pub(super) fn has_power_buffer_for(&self, def: &TowerDef) -> bool {
        let generated =
            self.factory.power_generation() + self.unlocked_building_boon().power_per_sec;
        let current_drain: f32 = self.factory.power_consumption()
            + self
                .towers
                .iter()
                .filter(|t| t.is_active)
                .map(|t| t.power_drain)
                .sum::<f32>();
        let net_after = generated - current_drain - def.cost_power;
        net_after >= 0.0
            || self.resources.power >= self.constants.economy.power_buffer_min_for_build
    }
}

/// Rank buildings by what bringing them online is worth, biased toward scrap
/// because scrap gates every other repair.
fn building_score(building: &MapBuilding) -> f32 {
    let scrap = building.boon.scrap_per_sec * 100.0;
    let power = building.boon.power_per_sec * 80.0;
    let food = building.boon.food_per_sec * 70.0;
    let water = building.boon.water_per_sec * 60.0;
    let preferred = if building.building_type.contains("scrap") {
        1000.0
    } else {
        0.0
    };
    preferred + scrap + power + food + water - building.repair_cost * 0.01
}
